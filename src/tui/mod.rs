use crate::config::{Config, DirectoryConfig};
use crate::crypto::{CryptoManager, SecureKey};
use crate::crypto::key_derivation;
use crate::errors::RpmResult;
use crate::i18n::{I18n, Language};
use crate::storage::PasswordStorage;
use crate::tray::TrayHandle;
use arboard::Clipboard;
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, STANDARD_NO_PAD as BASE64_STANDARD_NO_PAD};
use base64::Engine;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rand::RngCore;
use rand::rngs::OsRng;
use rand::Rng;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;
use ratatui::Terminal;

mod theme;
use theme::{get_theme_by_name, Theme};
use std::collections::HashSet;
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};
use zeroize::Zeroize;

#[derive(Debug, Clone, PartialEq)]
enum Screen {
    MasterPassword,
    Main,
    Settings,
    PasswordEntry { is_edit: bool, filename: Option<String> },
    PasswordGenerator { return_to_edit: bool, return_filename: Option<String> },
    Help,
    ThemeSelection,
    LanguageSelection,
}

pub struct TuiState {
    pub should_quit: bool,
    pub selected_index: usize,
    pub current_screen: Screen,
    pub passwords_dir_input: String,
    pub config: Config,
    pub search_query: String,
    pub all_items: Vec<String>,
    pub filtered_items: Vec<String>,
    // Master password and encryption key
    pub master_password_input: String,
    pub master_password_confirm: String,
    pub master_password_field: usize, // For creation: 0 = directory, 1 = password, 2 = confirm. For entry: 0 = password
    pub master_password_show_password: bool, // Show password in plain text
    pub is_creating_master_password: bool, // true if creating new, false if entering existing
    pub encryption_key: Option<SecureKey>,
    // Password entry screen state
    pub password_entry_name: String,
    pub password_entry_password: String,
    pub password_entry_show_password: bool,
    pub password_entry_field: usize, // 0 = name, 1 = password
    // Mapping from displayed name to filename
    pub name_to_filename: Vec<(String, String)>, // (display_name, filename)
    // Clipboard cleanup task handle
    pub clipboard_cleanup_handle: Option<JoinHandle<()>>,
    // Persistent clipboard instance to avoid "dropped very quickly" warning
    pub clipboard: Option<Arc<StdMutex<Clipboard>>>,
    // Settings screen state
    pub clipboard_timeout_input: String,
    pub settings_field: usize, // 0 = directory, 1 = clipboard timeout, 2 = theme, 3 = language
    // Theme selection screen state
    pub theme_selection_index: usize, // 0 = textual_dark, 1 = vscode_style, 2 = opencode_style
    // Language selection screen state
    pub language_selection_index: usize, // 0 = Russian, 1 = English (default), 2 = Chinese
    // Localization
    pub i18n: I18n,
    // Password generator screen state
    pub password_generator_length: String,
    pub password_generator_exclude_chars: String,
    pub password_generator_use_uppercase: bool,
    pub password_generator_use_lowercase: bool,
    pub password_generator_use_digits: bool,
    pub password_generator_use_special: bool,
    pub password_generator_selected_field: usize, // 0 = length, 1 = exclude_chars, 2-5 = checkboxes
}

pub async fn run_tui(
    crypto: CryptoManager,
    _tray: TrayHandle,
    config: Config,
    shutdown_tx: watch::Sender<()>,
) -> RpmResult<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut storage = PasswordStorage::new(&config, crypto.clone());

    // Check if master password is already set for the current directory
    let passwords_dir = config.passwords_directory_path();
    let dir_config = DirectoryConfig::load(&passwords_dir)
        .unwrap_or_else(|_| DirectoryConfig {
            master_password_hash: None,
            encryption_key_salt: None,
        });
    let is_creating_master_password = !dir_config.has_master_password();

    // Initialize i18n
    let language = Language::from_code(&config.language);
    let i18n = I18n::new(language);

    let mut state = TuiState {
        should_quit: false,
        selected_index: 0,
        current_screen: Screen::MasterPassword,
        passwords_dir_input: config
            .passwords_directory
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        config: config.clone(),
        search_query: String::new(),
        all_items: Vec::new(),
        filtered_items: Vec::new(),
        master_password_input: String::new(),
        master_password_confirm: String::new(),
        master_password_field: 0,
        master_password_show_password: false,
        is_creating_master_password,
        encryption_key: None,
        password_entry_name: String::new(),
        password_entry_password: String::new(),
        password_entry_show_password: false,
        password_entry_field: 0,
        name_to_filename: Vec::new(),
        clipboard_cleanup_handle: None,
        clipboard: None,
        clipboard_timeout_input: config.clipboard_timeout_seconds.to_string(),
        settings_field: 0,
        theme_selection_index: match config.theme.as_str() {
            "vscode_style" => 1,
            "opencode_style" => 2,
            _ => 0, // textual_dark по умолчанию
        },
        password_generator_length: String::new(),
        password_generator_exclude_chars: String::new(),
        password_generator_use_uppercase: true,
        password_generator_use_lowercase: true,
        password_generator_use_digits: true,
        password_generator_use_special: false,
        password_generator_selected_field: 0,
        language_selection_index: match config.language.as_str() {
            "ru" => 0,
            "zh" => 2,
            _ => 1, // English by default
        },
        i18n,
    };
    let mut list_state = ListState::default();

    loop {
        terminal.draw(|f| ui(f, &state, &mut list_state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match state.current_screen.clone() {
                    Screen::MasterPassword => {
                        // Проверяем F1 для открытия help
                        if key.code == KeyCode::F(1) {
                            state.current_screen = Screen::Help;
                        } else {
                            match key.code {
                            KeyCode::Enter => {
                                if state.is_creating_master_password {
                                    // Creating new master password
                                    if state.master_password_field == 0 {
                                        // Save directory and move to password field
                                        if !state.passwords_dir_input.trim().is_empty() {
                                            state.config.passwords_directory =
                                                Some(PathBuf::from(state.passwords_dir_input.trim()));
                                        } else {
                                            state.config.passwords_directory = None;
                                        }
                                        
                                        if let Err(e) = state.config.save() {
                                            eprintln!("Failed to save config: {}", e);
                                        }
                                        
                                        // Пересоздаем storage с новой директорией
                                        storage = PasswordStorage::new(&state.config, crypto.clone());
                                        
                                        // Move to password field
                                        state.master_password_field = 1;
                                    } else if state.master_password_field == 1 {
                                        // Move to confirmation field
                                        state.master_password_field = 2;
                                    } else {
                                        // Check if passwords match
                                        if state.master_password_input != state.master_password_confirm {
                                            // Passwords don't match, reset
                                            state.master_password_input.clear();
                                            state.master_password_confirm.clear();
                                            state.master_password_field = 1;
                                            continue;
                                        }

                                        // Ensure directory is saved in config (in case user used Tab to skip)
                                        if !state.passwords_dir_input.trim().is_empty() {
                                            state.config.passwords_directory =
                                                Some(PathBuf::from(state.passwords_dir_input.trim()));
                                        } else {
                                            state.config.passwords_directory = None;
                                        }
                                        
                                        if let Err(e) = state.config.save() {
                                            eprintln!("Failed to save config: {}", e);
                                        }
                                        
                                        // Пересоздаем storage с правильной директорией
                                        storage = PasswordStorage::new(&state.config, crypto.clone());

                                        // Save master password hash to directory config
                                        let passwords_dir = state.config.passwords_directory_path();
                                        let mut dir_config = DirectoryConfig::load(&passwords_dir)
                                            .unwrap_or_else(|_| DirectoryConfig {
                                                master_password_hash: None,
                                                encryption_key_salt: None,
                                            });
                                        
                                        let hash = crypto.hash_password(&state.master_password_input)?;
                                        dir_config.master_password_hash = Some(hash);
                                        
                                        // Generate salt if not exists
                                        if dir_config.encryption_key_salt.is_none() {
                                            let mut salt_bytes = [0u8; 32];
                                            rand::thread_rng().fill_bytes(&mut salt_bytes);
                                            dir_config.encryption_key_salt = Some(BASE64_STANDARD_NO_PAD.encode(&salt_bytes));
                                        }
                                        
                                        if let Err(e) = dir_config.save(&passwords_dir) {
                                            eprintln!("Failed to save directory config: {}", e);
                                        }
                                    }
                                } else {
                                    // Entering existing master password
                                    // Verify password against directory config
                                    let passwords_dir = state.config.passwords_directory_path();
                                    let dir_config = DirectoryConfig::load(&passwords_dir)
                                        .unwrap_or_else(|_| DirectoryConfig {
                                            master_password_hash: None,
                                            encryption_key_salt: None,
                                        });
                                    
                                    if let Some(ref stored_hash) = dir_config.master_password_hash {
                                        match crypto.verify_password(&state.master_password_input, stored_hash) {
                                            Ok(true) => {
                                                // Password correct
                                            }
                                            Ok(false) => {
                                                // Password incorrect, reset
                                                state.master_password_input.clear();
                                                continue;
                                            }
                                            Err(_) => {
                                                // Error verifying, reset
                                                state.master_password_input.clear();
                                                continue;
                                            }
                                        }
                                    } else {
                                        // No master password set for directory, reset
                                        state.master_password_input.clear();
                                        continue;
                                    }
                                }

                                // Derive encryption key from master password
                                let passwords_dir = state.config.passwords_directory_path();
                                let dir_config = DirectoryConfig::load(&passwords_dir)
                                    .unwrap_or_else(|_| DirectoryConfig {
                                        master_password_hash: None,
                                        encryption_key_salt: None,
                                    });
                                
                                let salt = if let Some(salt_str) = &dir_config.encryption_key_salt {
                                    // Try decoding without padding first (new format), then with padding (old format for compatibility)
                                    BASE64_STANDARD_NO_PAD.decode(salt_str)
                                        .or_else(|_| BASE64_STANDARD.decode(salt_str))
                                        .map_err(|e| crate::errors::RpmError::Crypto(format!("Invalid salt: {}", e)))?
                                } else {
                                    // Generate new salt (should not happen if creating, but handle it)
                                    let mut salt_bytes = [0u8; 32];
                                    rand::thread_rng().fill_bytes(&mut salt_bytes);
                                    let salt_str = BASE64_STANDARD_NO_PAD.encode(&salt_bytes);
                                    let mut dir_config = DirectoryConfig::load(&passwords_dir)
                                        .unwrap_or_else(|_| DirectoryConfig {
                                            master_password_hash: None,
                                            encryption_key_salt: None,
                                        });
                                    dir_config.encryption_key_salt = Some(salt_str.clone());
                                    if let Err(e) = dir_config.save(&passwords_dir) {
                                        eprintln!("Failed to save directory config: {}", e);
                                    }
                                    salt_bytes.to_vec()
                                };

                                let key = key_derivation::derive_key(&state.master_password_input, Some(&salt))?;
                                state.encryption_key = Some(SecureKey::new(key));

                                // Clear master password from memory
                                state.master_password_input.zeroize();
                                state.master_password_input.clear();
                                state.master_password_confirm.zeroize();
                                state.master_password_confirm.clear();

                                // Load def file and decrypt names
                                if let Some(ref key) = state.encryption_key {
                                    match storage.list_decrypted_names(key.as_slice()) {
                                        Ok(names) => {
                                            state.name_to_filename = names.clone();
                                            state.all_items = names.iter().map(|(_, name)| name.clone()).collect();
                                            state.filtered_items = state.all_items.clone();
                                        }
                                        Err(_) => {
                                            // Empty list if def file doesn't exist or can't be decrypted
                                            state.all_items = Vec::new();
                                            state.filtered_items = Vec::new();
                                        }
                                    }
                                }

                                state.current_screen = Screen::Main;
                                if !state.filtered_items.is_empty() {
                                    list_state.select(Some(0));
                                }
                            }
                            KeyCode::Up => {
                                if state.is_creating_master_password {
                                    // Switch between directory, password and confirm fields (backward)
                                    if state.master_password_field > 0 {
                                        state.master_password_field -= 1;
                                    } else {
                                        state.master_password_field = 2; // Wrap to last field
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if state.is_creating_master_password {
                                    // Switch between directory, password and confirm fields (forward)
                                    state.master_password_field = (state.master_password_field + 1) % 3;
                                }
                            }
                            KeyCode::Esc => {
                                state.should_quit = true;
                                let _ = shutdown_tx.send(());
                            }
                            KeyCode::Backspace => {
                                if state.is_creating_master_password {
                                    match state.master_password_field {
                                        0 => {
                                            state.passwords_dir_input.pop();
                                        }
                                        1 => {
                                            state.master_password_input.pop();
                                        }
                                        2 => {
                                            state.master_password_confirm.pop();
                                        }
                                        _ => {}
                                    }
                                } else {
                                    // Entering existing password - only one field
                                    state.master_password_input.pop();
                                }
                            }
                            KeyCode::Char(c) => {
                                // Handle Ctrl+H for password visibility (only for password fields, not directory)
                                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'h' {
                                    if !state.is_creating_master_password || state.master_password_field != 0 {
                                        state.master_password_show_password = !state.master_password_show_password;
                                    }
                                } else if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                    // Only process regular characters without Ctrl modifier
                                    if state.is_creating_master_password {
                                        match state.master_password_field {
                                            0 => {
                                                state.passwords_dir_input.push(c);
                                            }
                                            1 => {
                                                state.master_password_input.push(c);
                                            }
                                            2 => {
                                                state.master_password_confirm.push(c);
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        // Entering existing password - only one field
                                        state.master_password_input.push(c);
                                    }
                                }
                            }
                            _ => {}
                            }
                        }
                    }
                    Screen::PasswordEntry { is_edit, filename } => {
                        // Проверяем Ctrl+G для открытия генератора паролей
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('g') {
                            // Открываем генератор паролей
                            // Инициализируем значения по умолчанию, если они пустые
                            if state.password_generator_length.is_empty() {
                                state.password_generator_length = "16".to_string();
                            }
                            // Сохраняем информацию о том, из какого экрана мы пришли
                            state.current_screen = Screen::PasswordGenerator { 
                                return_to_edit: is_edit, 
                                return_filename: filename.clone() 
                            };
                        }
                        // Проверяем F1 для открытия help
                        else if key.code == KeyCode::F(1) {
                            state.current_screen = Screen::Help;
                        } else {
                            match key.code {
                            KeyCode::Esc => {
                                // Cancel and return to main screen
                                state.password_entry_name.clear();
                                state.password_entry_password.clear();
                                state.password_entry_show_password = false;
                                state.password_entry_field = 0;
                                state.current_screen = Screen::Main;
                            }
                            KeyCode::Up => {
                                // Switch between fields (backward)
                                if state.password_entry_field > 0 {
                                    state.password_entry_field -= 1;
                                } else {
                                    state.password_entry_field = 1; // Wrap to last field
                                }
                            }
                            KeyCode::Down => {
                                // Switch between fields (forward)
                                state.password_entry_field = (state.password_entry_field + 1) % 2;
                            }
                            KeyCode::Enter => {
                                // Save password
                                if state.password_entry_name.trim().is_empty() {
                                    // Name is required
                                    continue;
                                }

                                if let Some(ref key) = state.encryption_key {
                                    if is_edit {
                                        // Update existing entry
                                        if let Some(ref filename) = filename {
                                            // Update password file
                                            let _ = storage.update_password_file(filename, &state.password_entry_password, key.as_slice());
                                            // Update name in def file
                                            let _ = storage.update_entry(filename, &state.password_entry_name, key.as_slice());
                                        }
                                    } else {
                                        // Create new entry
                                        let new_filename = storage.add_entry(&state.password_entry_name, key.as_slice())?;
                                        // Save password to the file with the generated filename
                                        let _ = storage.update_password_file(&new_filename, &state.password_entry_password, key.as_slice());
                                    }

                                    // Reload list
                                    match storage.list_decrypted_names(key.as_slice()) {
                                        Ok(names) => {
                                            state.name_to_filename = names.clone();
                                            state.all_items = names.iter().map(|(_, name)| name.clone()).collect();
                                            filter_items(&mut state);
                                        }
                                        Err(_) => {}
                                    }

                                    // Clear and return to main
                                    state.password_entry_name.clear();
                                    state.password_entry_password.clear();
                                    state.password_entry_show_password = false;
                                    state.password_entry_field = 0;
                                    state.current_screen = Screen::Main;
                                    if !state.filtered_items.is_empty() {
                                        list_state.select(Some(0));
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                if state.password_entry_field == 0 {
                                    state.password_entry_name.pop();
                                } else {
                                    state.password_entry_password.pop();
                                }
                            }
                            KeyCode::Char(c) => {
                                // Handle Ctrl+H for password visibility
                                if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'h' {
                                    if state.password_entry_field == 1 {
                                        state.password_entry_show_password = !state.password_entry_show_password;
                                    }
                                } else if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                    // Only process regular characters without Ctrl modifier
                                    if state.password_entry_field == 0 {
                                        state.password_entry_name.push(c);
                                    } else {
                                        state.password_entry_password.push(c);
                                    }
                                }
                            }
                            _ => {}
                            }
                        }
                    }
                    Screen::Main => {
                        // Проверяем Ctrl+Q для выхода
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
                            state.should_quit = true;
                            // Send shutdown signal to stop all components
                            let _ = shutdown_tx.send(());
                        }
                        // Проверяем Ctrl+N для создания нового пароля
                        else if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('n') {
                            state.password_entry_name.clear();
                            state.password_entry_password.clear();
                            state.password_entry_show_password = false;
                            state.password_entry_field = 0;
                            state.current_screen = Screen::PasswordEntry { is_edit: false, filename: None };
                        }
                        // Проверяем Ctrl+E для редактирования выбранного пароля
                        else if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('e') {
                            if !state.filtered_items.is_empty() && state.selected_index < state.filtered_items.len() {
                                let selected_name = &state.filtered_items[state.selected_index];
                                // Find filename for this name
                                let filename = state.name_to_filename.iter()
                                    .find(|(_, name)| name == selected_name)
                                    .map(|(filename, _)| filename.clone());

                                if let Some(ref filename) = filename {
                                    if let Some(ref key) = state.encryption_key {
                                        // Load password
                                        match storage.load_password_file(filename, key.as_slice()) {
                                            Ok(password) => {
                                                state.password_entry_name = selected_name.clone();
                                                state.password_entry_password = password;
                                                state.password_entry_show_password = false;
                                                state.password_entry_field = 0;
                                                state.current_screen = Screen::PasswordEntry { 
                                                    is_edit: true, 
                                                    filename: Some(filename.clone()) 
                                                };
                                            }
                                            Err(_) => {
                                                // Could not load password, still allow editing name
                                                state.password_entry_name = selected_name.clone();
                                                state.password_entry_password.clear();
                                                state.password_entry_show_password = false;
                                                state.password_entry_field = 0;
                                                state.current_screen = Screen::PasswordEntry { 
                                                    is_edit: true, 
                                                    filename: Some(filename.clone()) 
                                                };
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Проверяем Ctrl+S для настроек
                        else if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                            // Переход в настройки по Ctrl+S
                            state.current_screen = Screen::Settings;
                        }
                        // Проверяем Ctrl+C для копирования пароля в буфер обмена
                        else if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                            if !state.filtered_items.is_empty() && state.selected_index < state.filtered_items.len() {
                                let selected_name = &state.filtered_items[state.selected_index];
                                // Find filename for this name
                                let filename = state.name_to_filename.iter()
                                    .find(|(_, name)| name == selected_name)
                                    .map(|(filename, _)| filename.clone());

                                if let Some(ref filename) = filename {
                                    if let Some(ref key) = state.encryption_key {
                                        // Cancel previous cleanup task if exists
                                        if let Some(handle) = state.clipboard_cleanup_handle.take() {
                                            handle.abort();
                                        }

                                        // Load password
                                        match storage.load_password_file(filename, key.as_slice()) {
                                            Ok(mut password) => {
                                                // Get or create persistent clipboard instance
                                                let clipboard_arc = if let Some(ref existing) = state.clipboard {
                                                    existing.clone()
                                                } else {
                                                    match Clipboard::new() {
                                                        Ok(clipboard) => {
                                                            let arc = Arc::new(StdMutex::new(clipboard));
                                                            state.clipboard = Some(arc.clone());
                                                            arc
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Failed to initialize clipboard: {}", e);
                                                            password.zeroize();
                                                            continue;
                                                        }
                                                    }
                                                };

                                                // Copy to clipboard using persistent instance
                                                {
                                                    let mut clipboard = clipboard_arc.lock().unwrap();
                                                    if let Err(e) = clipboard.set_text(&password) {
                                                        eprintln!("Failed to copy to clipboard: {}", e);
                                                        password.zeroize();
                                                        continue;
                                                    }
                                                }

                                                // Schedule clipboard cleanup if timeout is set
                                                let timeout_seconds = state.config.clipboard_timeout_seconds;
                                                if timeout_seconds > 0 {
                                                    let clipboard_for_cleanup = clipboard_arc.clone();
                                                    let handle = tokio::spawn(async move {
                                                        sleep(Duration::from_secs(timeout_seconds)).await;
                                                        let mut clipboard = clipboard_for_cleanup.lock().unwrap();
                                                        // Clear clipboard by setting empty string
                                                        let _ = clipboard.set_text("");
                                                    });
                                                    state.clipboard_cleanup_handle = Some(handle);
                                                }

                                                // Clear password from memory
                                                password.zeroize();
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to load password: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Обработка обычных клавиш (без Ctrl)
                        else if !key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                            KeyCode::Esc => {
                                // Сброс поиска при нажатии Esc
                                if !state.search_query.is_empty() {
                                    state.search_query.clear();
                                    filter_items(&mut state);
                                    state.selected_index = 0;
                                    list_state.select(if state.filtered_items.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    });
                                }
                            }
                            KeyCode::F(1) => {
                                // Переход в help по F1
                                state.current_screen = Screen::Help;
                            }
                            KeyCode::F(2) => {
                                // Переход в настройки по F2
                                state.current_screen = Screen::Settings;
                            }
                            KeyCode::Up => {
                                if !state.filtered_items.is_empty() && state.selected_index > 0 {
                                    state.selected_index -= 1;
                                    list_state.select(Some(state.selected_index));
                                }
                            }
                            KeyCode::Down => {
                                if !state.filtered_items.is_empty() 
                                    && state.selected_index < state.filtered_items.len().saturating_sub(1) {
                                    state.selected_index += 1;
                                    list_state.select(Some(state.selected_index));
                                }
                            }
                            KeyCode::Backspace => {
                                if !state.search_query.is_empty() {
                                    state.search_query.pop();
                                    filter_items(&mut state);
                                    // Сбрасываем индекс если он выходит за границы
                                    if state.selected_index >= state.filtered_items.len() {
                                        state.selected_index = state.filtered_items.len().saturating_sub(1);
                                    }
                                    list_state.select(if state.filtered_items.is_empty() {
                                        None
                                    } else {
                                        Some(state.selected_index.min(state.filtered_items.len().saturating_sub(1)))
                                    });
                                }
                            }
                            KeyCode::Char(c) => {
                                state.search_query.push(c);
                                filter_items(&mut state);
                                // Сбрасываем индекс если он выходит за границы
                                if state.selected_index >= state.filtered_items.len() {
                                    state.selected_index = state.filtered_items.len().saturating_sub(1);
                                }
                                list_state.select(if state.filtered_items.is_empty() {
                                    None
                                } else {
                                    Some(state.selected_index.min(state.filtered_items.len().saturating_sub(1)))
                                });
                            }
                            _ => {}
                            }
                        }
                    }
                    Screen::Help => {
                        match key.code {
                            KeyCode::Esc | KeyCode::F(1) => {
                                // Закрыть help и вернуться к предыдущему экрану
                                state.current_screen = Screen::Main;
                            }
                            _ => {}
                        }
                    }
                    Screen::Settings => {
                        // Проверяем F1 для открытия help
                        if key.code == KeyCode::F(1) {
                            state.current_screen = Screen::Help;
                        } else {
                            match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                // Сохраняем настройки перед выходом
                                if !state.passwords_dir_input.trim().is_empty() {
                                    state.config.passwords_directory =
                                        Some(PathBuf::from(state.passwords_dir_input.trim()));
                                } else {
                                    state.config.passwords_directory = None;
                                }
                                
                                // Сохраняем время хранения в буфере обмена
                                if let Ok(timeout) = state.clipboard_timeout_input.trim().parse::<u64>() {
                                    state.config.clipboard_timeout_seconds = timeout;
                                }
                                
                                if let Err(e) = state.config.save() {
                                    // В реальном приложении здесь должна быть обработка ошибки
                                    eprintln!("Failed to save config: {}", e);
                                }
                                
                                // Пересоздаем storage с новой директорией
                                storage = PasswordStorage::new(&state.config, crypto.clone());
                                
                                // Проверяем наличие мастер-пароля для новой директории
                                let passwords_dir = state.config.passwords_directory_path();
                                let dir_config = DirectoryConfig::load(&passwords_dir)
                                    .unwrap_or_else(|_| DirectoryConfig {
                                        master_password_hash: None,
                                        encryption_key_salt: None,
                                    });
                                
                                if !dir_config.has_master_password() {
                                    // Нужно установить мастер-пароль для директории
                                    state.master_password_input.clear();
                                    state.master_password_confirm.clear();
                                    state.master_password_field = 0;
                                    state.master_password_show_password = false;
                                    state.is_creating_master_password = true;
                                    state.encryption_key = None; // Сбрасываем ключ при смене директории
                                    state.current_screen = Screen::MasterPassword;
                                } else {
                                    // Мастер-пароль уже установлен, но нужно запросить его для входа
                                    state.master_password_input.clear();
                                    state.master_password_confirm.clear();
                                    state.master_password_field = 0;
                                    state.master_password_show_password = false;
                                    state.is_creating_master_password = false;
                                    state.encryption_key = None; // Сбрасываем ключ при смене директории
                                    state.current_screen = Screen::MasterPassword;
                                }
                            }
                            KeyCode::Up => {
                                // Switch between fields (backward)
                                if state.settings_field > 0 {
                                    state.settings_field -= 1;
                                } else {
                                    state.settings_field = 3; // Wrap to last field (language)
                                }
                            }
                            KeyCode::Down => {
                                // Switch between fields (forward)
                                state.settings_field = (state.settings_field + 1) % 4;
                            }
                            KeyCode::Backspace => {
                                if state.settings_field == 0 {
                                    state.passwords_dir_input.pop();
                                } else if state.settings_field == 1 {
                                    state.clipboard_timeout_input.pop();
                                }
                                // Fields 2 (theme) and 3 (language) не редактируются через Backspace
                            }
                            KeyCode::Enter => {
                                // Если выбрано поле темы, открываем экран выбора темы
                                if state.settings_field == 2 {
                                    state.current_screen = Screen::ThemeSelection;
                                } else if state.settings_field == 3 {
                                    // Если выбрано поле языка, открываем экран выбора языка
                                    state.current_screen = Screen::LanguageSelection;
                                } else {
                                    // Сохраняем и выходим
                                    if !state.passwords_dir_input.trim().is_empty() {
                                        state.config.passwords_directory =
                                            Some(PathBuf::from(state.passwords_dir_input.trim()));
                                    } else {
                                        state.config.passwords_directory = None;
                                    }
                                    
                                    // Сохраняем время хранения в буфере обмена
                                    if let Ok(timeout) = state.clipboard_timeout_input.trim().parse::<u64>() {
                                        state.config.clipboard_timeout_seconds = timeout;
                                    }
                                    
                                    if let Err(e) = state.config.save() {
                                        eprintln!("Failed to save config: {}", e);
                                    }
                                    
                                    // Пересоздаем storage с новой директорией
                                    storage = PasswordStorage::new(&state.config, crypto.clone());
                                    
                                    // Проверяем наличие мастер-пароля для новой директории
                                    let passwords_dir = state.config.passwords_directory_path();
                                    let dir_config = DirectoryConfig::load(&passwords_dir)
                                        .unwrap_or_else(|_| DirectoryConfig {
                                            master_password_hash: None,
                                            encryption_key_salt: None,
                                        });
                                    
                                    if !dir_config.has_master_password() {
                                        // Нужно установить мастер-пароль для директории
                                        state.master_password_input.clear();
                                        state.master_password_confirm.clear();
                                        state.master_password_field = 0;
                                        state.master_password_show_password = false;
                                        state.is_creating_master_password = true;
                                        state.encryption_key = None; // Сбрасываем ключ при смене директории
                                        state.current_screen = Screen::MasterPassword;
                                    } else {
                                        // Мастер-пароль уже установлен, но нужно запросить его для входа
                                        state.master_password_input.clear();
                                        state.master_password_confirm.clear();
                                        state.master_password_field = 0;
                                        state.master_password_show_password = false;
                                        state.is_creating_master_password = false;
                                        state.encryption_key = None; // Сбрасываем ключ при смене директории
                                        state.current_screen = Screen::MasterPassword;
                                    }
                                }
                            }
                            KeyCode::Char(c) => {
                                if state.settings_field == 0 {
                                    state.passwords_dir_input.push(c);
                                } else {
                                    // Only allow digits for timeout
                                    if c.is_ascii_digit() {
                                        state.clipboard_timeout_input.push(c);
                                    }
                                }
                            }
                            _ => {}
                            }
                        }
                    }
                    Screen::ThemeSelection => {
                        // Проверяем F1 для открытия help
                        if key.code == KeyCode::F(1) {
                            state.current_screen = Screen::Help;
                        } else {
                            match key.code {
                            KeyCode::Esc => {
                                // Возвращаемся к настройкам
                                state.current_screen = Screen::Settings;
                            }
                            KeyCode::Up => {
                                if state.theme_selection_index > 0 {
                                    state.theme_selection_index -= 1;
                                } else {
                                    state.theme_selection_index = 2; // Wrap to last
                                }
                            }
                            KeyCode::Down => {
                                state.theme_selection_index = (state.theme_selection_index + 1) % 3;
                            }
                            KeyCode::Enter => {
                                // Сохраняем выбранную тему
                                let theme_name = match state.theme_selection_index {
                                    1 => "vscode_style",
                                    2 => "opencode_style",
                                    _ => "textual_dark",
                                };
                                state.config.theme = theme_name.to_string();
                                
                                if let Err(e) = state.config.save() {
                                    eprintln!("Failed to save config: {}", e);
                                }
                                
                                // Возвращаемся к настройкам
                                state.current_screen = Screen::Settings;
                            }
                            _ => {}
                            }
                        }
                    }
                    Screen::LanguageSelection => {
                        // Проверяем F1 для открытия help
                        if key.code == KeyCode::F(1) {
                            state.current_screen = Screen::Help;
                        } else {
                            match key.code {
                            KeyCode::Esc => {
                                // Возвращаемся к настройкам
                                state.current_screen = Screen::Settings;
                            }
                            KeyCode::Up => {
                                if state.language_selection_index > 0 {
                                    state.language_selection_index -= 1;
                                } else {
                                    state.language_selection_index = 2; // Wrap to last
                                }
                            }
                            KeyCode::Down => {
                                state.language_selection_index = (state.language_selection_index + 1) % 3;
                            }
                            KeyCode::Enter => {
                                // Сохраняем выбранный язык
                                let language_code = match state.language_selection_index {
                                    1 => "en",
                                    2 => "zh",
                                    _ => "ru",
                                };
                                state.config.language = language_code.to_string();
                                
                                // Обновляем i18n
                                let language = Language::from_code(language_code);
                                state.i18n.set_language(language);
                                
                                if let Err(e) = state.config.save() {
                                    eprintln!("Failed to save config: {}", e);
                                }
                                
                                // Возвращаемся к настройкам
                                state.current_screen = Screen::Settings;
                            }
                            _ => {}
                            }
                        }
                    }
                    Screen::PasswordGenerator { return_to_edit, return_filename } => {
                        // Проверяем F1 для открытия help
                        if key.code == KeyCode::F(1) {
                            state.current_screen = Screen::Help;
                        } else {
                            match key.code {
                            KeyCode::Esc => {
                                // Закрыть генератор и вернуться к PasswordEntry
                                // Восстанавливаем предыдущий экран с сохраненными параметрами
                                state.current_screen = Screen::PasswordEntry { 
                                    is_edit: return_to_edit, 
                                    filename: return_filename.clone() 
                                };
                            }
                            KeyCode::Up => {
                                if state.password_generator_selected_field > 0 {
                                    state.password_generator_selected_field -= 1;
                                }
                            }
                            KeyCode::Down => {
                                // Максимум 5 полей: 0=length, 1=exclude_chars, 2-5=checkboxes
                                if state.password_generator_selected_field < 5 {
                                    state.password_generator_selected_field += 1;
                                }
                            }
                            KeyCode::Char(' ') => {
                                // Переключение галочек только для полей 2-5
                                // Для полей ввода (0-1) пробел обрабатывается в KeyCode::Char(c)
                                if state.password_generator_selected_field >= 2 && state.password_generator_selected_field <= 5 {
                                    match state.password_generator_selected_field {
                                        2 => state.password_generator_use_uppercase = !state.password_generator_use_uppercase,
                                        3 => state.password_generator_use_lowercase = !state.password_generator_use_lowercase,
                                        4 => state.password_generator_use_digits = !state.password_generator_use_digits,
                                        5 => state.password_generator_use_special = !state.password_generator_use_special,
                                        _ => {}
                                    }
                                } else {
                                    // Если пробел в поле ввода, обрабатываем как обычный символ
                                    match state.password_generator_selected_field {
                                        0 => {
                                            // Поле длины - пробел не добавляем
                                        }
                                        1 => {
                                            // Поле исключений - добавляем пробел
                                            state.password_generator_exclude_chars.push(' ');
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            KeyCode::Enter => {
                                // Генерируем пароль и вставляем его
                                match generate_password(&state) {
                                    Ok(password) => {
                                        state.password_entry_password = password;
                                        // Возвращаемся к экрану PasswordEntry с сохраненными параметрами
                                        state.current_screen = Screen::PasswordEntry { 
                                            is_edit: return_to_edit, 
                                            filename: return_filename.clone() 
                                        };
                                    }
                                    Err(e) => {
                                        // Ошибка генерации - можно показать сообщение, но пока просто игнорируем
                                        eprintln!("Ошибка генерации пароля: {}", e);
                                    }
                                }
                            }
                            KeyCode::Backspace => {
                                // Удаление символа в активном поле ввода
                                match state.password_generator_selected_field {
                                    0 => {
                                        state.password_generator_length.pop();
                                    }
                                    1 => {
                                        state.password_generator_exclude_chars.pop();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Char(c) => {
                                // Ввод символов в активное поле
                                match state.password_generator_selected_field {
                                    0 => {
                                        // Поле длины - только цифры
                                        if c.is_ascii_digit() {
                                            state.password_generator_length.push(c);
                                        }
                                    }
                                    1 => {
                                        // Поле исключений - любые символы
                                        state.password_generator_exclude_chars.push(c);
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                            }
                        }
                    }
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    // Cancel clipboard cleanup task if exists
    if let Some(handle) = state.clipboard_cleanup_handle {
        handle.abort();
    }

    // Clear encryption key from memory before exit
    if let Some(mut key) = state.encryption_key {
        key.zeroize();
    }
    state.master_password_input.zeroize();
    state.master_password_confirm.zeroize();
    state.password_entry_password.zeroize();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, state: &TuiState, list_state: &mut ListState) {
    // Загружаем тему из конфига
    let theme = get_theme_by_name(&state.config.theme);
    
    // Устанавливаем фон для всего экрана
    f.render_widget(
        Block::default()
            .style(theme.bg_style()),
        f.size()
    );
    
    match state.current_screen {
        Screen::MasterPassword => render_master_password_screen(f, state, &theme),
        Screen::Main => render_main_screen(f, state, list_state, &theme),
        Screen::Settings => render_settings_screen(f, state, &theme),
        Screen::PasswordEntry { .. } => render_password_entry_screen(f, state, &theme),
        Screen::PasswordGenerator { .. } => render_password_generator_screen(f, state, &theme),
        Screen::Help => render_help_screen(f, state, &theme),
        Screen::ThemeSelection => render_theme_selection_screen(f, state, &theme),
        Screen::LanguageSelection => render_language_selection_screen(f, state, &theme),
    }
}

fn filter_items(state: &mut TuiState) {
    if state.search_query.is_empty() {
        state.filtered_items = state.all_items.clone();
    } else {
        let matcher = SkimMatcherV2::default();
        let mut scored_items: Vec<(i64, String)> = state
            .all_items
            .iter()
            .filter_map(|item| {
                matcher.fuzzy_match(item, &state.search_query).map(|score| (score, item.clone()))
            })
            .collect();
        
        // Сортируем по релевантности (больший score = лучшее совпадение)
        scored_items.sort_by(|a, b| b.0.cmp(&a.0));
        
        state.filtered_items = scored_items.into_iter().map(|(_, item)| item).collect();
    }
}

fn generate_password(state: &TuiState) -> RpmResult<String> {
    use crate::errors::RpmError;
    
    // Проверяем, что выбран хотя бы один набор символов
    if !state.password_generator_use_uppercase
        && !state.password_generator_use_lowercase
        && !state.password_generator_use_digits
        && !state.password_generator_use_special
    {
        return Err(RpmError::Crypto("Необходимо выбрать хотя бы один набор символов".to_string()));
    }
    
    // Парсим длину пароля
    let length: usize = state.password_generator_length.trim().parse()
        .map_err(|_| RpmError::Crypto("Неверная длина пароля".to_string()))?;
    
    if length < 1 {
        return Err(RpmError::Crypto("Длина пароля должна быть не менее 1".to_string()));
    }
    
    if length > 256 {
        return Err(RpmError::Crypto("Длина пароля не должна превышать 256".to_string()));
    }
    
    // Собираем доступные символы
    let mut available_chars = Vec::new();
    
    if state.password_generator_use_uppercase {
        available_chars.extend('A'..='Z');
    }
    if state.password_generator_use_lowercase {
        available_chars.extend('a'..='z');
    }
    if state.password_generator_use_digits {
        available_chars.extend('0'..='9');
    }
    if state.password_generator_use_special {
        available_chars.extend("!@#$%^&*()_+-=[]{}|;:,.<>?".chars());
    }
    
    // Исключаем символы из exclude_chars
    let exclude_set: HashSet<char> = state.password_generator_exclude_chars.chars().collect();
    available_chars.retain(|&c| !exclude_set.contains(&c));
    
    // Проверяем, что после исключения остались символы
    if available_chars.is_empty() {
        return Err(RpmError::Crypto("После исключения символов не осталось доступных символов".to_string()));
    }
    
    // Генерируем пароль используя криптографически стойкий генератор
    let mut rng = OsRng;
    let password: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..available_chars.len());
            available_chars[idx]
        })
        .collect();
    
    Ok(password)
}

fn render_main_screen(f: &mut Frame, state: &TuiState, list_state: &mut ListState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Поле поиска
            Constraint::Min(0),    // Основной контент
            Constraint::Length(3), // Футер
        ])
        .split(f.size());

    // Поле ввода для поиска
    let search_input = Paragraph::new(state.search_query.as_str())
        .style(theme.accent_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
                .title(state.i18n.ts("main_search"))
        );
    f.render_widget(search_input, chunks[0]);

    // Main content area
    let items: Vec<ListItem> = state
        .filtered_items
        .iter()
        .map(|item| ListItem::new(item.as_str()).style(theme.text_style()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.surface_style())
                .title(format!("{} ({})", state.i18n.ts("main_passwords"), state.filtered_items.len()))
        )
        .highlight_style(theme.selection_style())
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], list_state);

    // Footer
    let footer = Paragraph::new(state.i18n.ts("main_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[2]);
}

fn render_settings_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Основной контент
            Constraint::Length(3), // Футер
        ])
        .split(f.size());

    // Окно настроек
    let settings_content = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Заголовок
            Constraint::Length(1), // Метка для пути сохранения
            Constraint::Length(3), // Путь сохранения
            Constraint::Length(1), // Метка для конфига
            Constraint::Length(3), // Путь конфига
            Constraint::Length(1), // Метка для директории
            Constraint::Length(3), // Поле ввода директории
            Constraint::Length(1), // Метка для времени хранения
            Constraint::Length(3), // Поле ввода времени хранения
            Constraint::Length(1), // Метка для темы
            Constraint::Length(3), // Поле выбора темы
            Constraint::Length(1), // Метка для языка
            Constraint::Length(3), // Поле выбора языка
            Constraint::Min(0),    // Остальное пространство
        ])
        .split(chunks[0]);

    let settings_title = Paragraph::new(state.i18n.ts("settings_title"))
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(settings_title, settings_content[0]);

    // Информация о пути сохранения файлов
    let save_path_label = Paragraph::new(state.i18n.ts("settings_save_path_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(save_path_label, settings_content[1]);

    let save_path = state.config.passwords_directory_path();
    let save_path_text = save_path.to_string_lossy().to_string();
    let save_path_display = Paragraph::new(save_path_text.as_str())
        .style(theme.accent_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.surface_style())
                .title(state.i18n.ts("settings_save_path_title")),
        );
    f.render_widget(save_path_display, settings_content[2]);

    // Информация о пути к конфигурационному файлу
    let config_path_label = Paragraph::new(state.i18n.ts("settings_config_path_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(config_path_label, settings_content[3]);

    let config_path_text = state.config.config_file_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| state.i18n.ts("settings_config_path_error").to_string());
    let config_path_display = Paragraph::new(config_path_text.as_str())
        .style(theme.accent_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.surface_style())
                .title(state.i18n.ts("settings_config_path_title")),
        );
    f.render_widget(config_path_display, settings_content[4]);

    let dir_label = Paragraph::new(state.i18n.ts("settings_directory_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(dir_label, settings_content[5]);

    let dir_style = if state.settings_field == 0 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };

    let dir_title = if state.settings_field == 0 {
        state.i18n.ts("settings_directory_active")
    } else {
        state.i18n.ts("settings_directory")
    };

    let dir_border_style = if state.settings_field == 0 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };

    let dir_input = Paragraph::new(state.passwords_dir_input.as_str())
        .style(dir_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(dir_border_style)
                .style(theme.surface_style())
                .title(dir_title),
        );
    f.render_widget(dir_input, settings_content[6]);

    // Метка для времени хранения в буфере обмена
    let timeout_label = Paragraph::new(state.i18n.ts("settings_clipboard_timeout_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(timeout_label, settings_content[7]);

    let timeout_style = if state.settings_field == 1 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };

    let timeout_title = if state.settings_field == 1 {
        state.i18n.ts("settings_clipboard_timeout_active")
    } else {
        state.i18n.ts("settings_clipboard_timeout")
    };

    let timeout_border_style = if state.settings_field == 1 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };

    let timeout_input = Paragraph::new(state.clipboard_timeout_input.as_str())
        .style(timeout_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(timeout_border_style)
                .style(theme.surface_style())
                .title(timeout_title),
        );
    f.render_widget(timeout_input, settings_content[8]);

    // Метка для темы
    let theme_label = Paragraph::new(state.i18n.ts("settings_theme_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(theme_label, settings_content[9]);

    let theme_style = if state.settings_field == 2 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };

    let theme_title = if state.settings_field == 2 {
        state.i18n.ts("settings_theme_active")
    } else {
        state.i18n.ts("settings_theme")
    };

    let theme_border_style = if state.settings_field == 2 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };

    let current_theme_name = match state.config.theme.as_str() {
        "vscode_style" => "VS Code Dark+",
        "opencode_style" => "OpenCode / Dark Modern",
        _ => "Textual / Modern Web",
    };

    let theme_display = Paragraph::new(current_theme_name)
        .style(theme_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme_border_style)
                .style(theme.surface_style())
                .title(theme_title),
        );
    f.render_widget(theme_display, settings_content[10]);

    // Метка для языка
    let language_label = Paragraph::new(state.i18n.ts("settings_language_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(language_label, settings_content[11]);

    let language_style = if state.settings_field == 3 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };

    let language_title = if state.settings_field == 3 {
        state.i18n.ts("settings_language_active")
    } else {
        state.i18n.ts("settings_language")
    };

    let language_border_style = if state.settings_field == 3 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };

    let current_language = Language::from_code(&state.config.language);
    let language_display = Paragraph::new(current_language.display_name())
        .style(language_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(language_border_style)
                .style(theme.surface_style())
                .title(language_title),
        );
    f.render_widget(language_display, settings_content[12]);

    // Footer
    let footer = Paragraph::new(state.i18n.ts("settings_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[1]);
}

fn render_master_password_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let constraints = if state.is_creating_master_password {
        vec![
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ]
    } else {
        vec![
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size());

    let title_text = if state.is_creating_master_password {
        state.i18n.ts("master_password_create_title")
    } else {
        state.i18n.ts("master_password_title")
    };

    let title = Paragraph::new(title_text)
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(title, chunks[1]);

    if state.is_creating_master_password {
        // Creating new master password - show directory, password, and confirm fields
        let dir_label = Paragraph::new(state.i18n.ts("master_password_directory_label"))
            .style(theme.text_style())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(dir_label, chunks[2]);

        let dir_style = if state.master_password_field == 0 {
            theme.active_input_style()
        } else {
            theme.inactive_input_style()
        };

        let dir_title = if state.master_password_field == 0 {
            state.i18n.ts("master_password_directory_active")
        } else {
            state.i18n.ts("master_password_directory")
        };

        let dir_border_style = if state.master_password_field == 0 {
            theme.active_border_style()
        } else {
            theme.inactive_border_style()
        };

        let dir_input = Paragraph::new(state.passwords_dir_input.as_str())
            .style(dir_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(dir_border_style)
                    .style(theme.surface_style())
                    .title(dir_title),
            );
        f.render_widget(dir_input, chunks[3]);

        let password_label = Paragraph::new(state.i18n.ts("master_password_label"))
            .style(theme.text_style())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(password_label, chunks[4]);

        let password_display = if state.master_password_input.is_empty() {
            String::new()
        } else if state.master_password_show_password {
            state.master_password_input.clone()
        } else {
            "*".repeat(state.master_password_input.len())
        };

        let password_style = if state.master_password_field == 1 {
            theme.active_input_style()
        } else {
            theme.inactive_input_style()
        };

        let password_title = if state.master_password_field == 1 {
            format!("{} | Ctrl+H - {}", state.i18n.ts("master_password_active"), if state.master_password_show_password { state.i18n.ts("hide") } else { state.i18n.ts("show") })
        } else {
            state.i18n.ts("master_password").to_string()
        };

        let password_border_style = if state.master_password_field == 1 {
            theme.active_border_style()
        } else {
            theme.inactive_border_style()
        };

        let password_input = Paragraph::new(password_display.as_str())
            .style(password_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(password_border_style)
                    .style(theme.surface_style())
                    .title(password_title),
            );
        f.render_widget(password_input, chunks[5]);

        let confirm_label = Paragraph::new(state.i18n.ts("master_password_confirm_label"))
            .style(theme.text_style())
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(confirm_label, chunks[6]);

        let confirm_display = if state.master_password_confirm.is_empty() {
            String::new()
        } else if state.master_password_show_password {
            state.master_password_confirm.clone()
        } else {
            "*".repeat(state.master_password_confirm.len())
        };

        let confirm_style = if state.master_password_field == 2 {
            theme.active_input_style()
        } else {
            theme.inactive_input_style()
        };

        let confirm_title = if state.master_password_field == 2 {
            format!("{} | Ctrl+H - {}", state.i18n.ts("master_password_confirm_active"), if state.master_password_show_password { state.i18n.ts("hide") } else { state.i18n.ts("show") })
        } else {
            state.i18n.ts("master_password_confirm").to_string()
        };

        let confirm_border_style = if state.master_password_field == 2 {
            theme.active_border_style()
        } else {
            theme.inactive_border_style()
        };

        let confirm_input = Paragraph::new(confirm_display.as_str())
            .style(confirm_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(confirm_border_style)
                    .style(theme.surface_style())
                    .title(confirm_title),
            );
        f.render_widget(confirm_input, chunks[7]);

        let footer = Paragraph::new(state.i18n.ts("master_password_footer_create"))
            .style(theme.dimmed_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.inactive_border_style())
                    .style(theme.status_bar_style())
            );
        f.render_widget(footer, chunks[9]);
    } else {
        // Entering existing master password - show one field
        let password_display = if state.master_password_input.is_empty() {
            String::new()
        } else if state.master_password_show_password {
            state.master_password_input.clone()
        } else {
            "*".repeat(state.master_password_input.len())
        };

        let password_title = format!("{} | Ctrl+H - {}", state.i18n.ts("master_password_enter"), if state.master_password_show_password { state.i18n.ts("hide") } else { state.i18n.ts("show") });

        let password_input = Paragraph::new(password_display.as_str())
            .style(theme.accent_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.active_border_style())
                    .style(theme.surface_style())
                    .title(password_title),
            );
        f.render_widget(password_input, chunks[2]);

        let footer = Paragraph::new(state.i18n.ts("master_password_footer_enter"))
            .style(theme.dimmed_style())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.inactive_border_style())
                    .style(theme.status_bar_style())
            );
        f.render_widget(footer, chunks[4]);
    }
}

fn render_password_entry_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title_text = if matches!(state.current_screen, Screen::PasswordEntry { is_edit: true, .. }) {
        state.i18n.ts("password_entry_edit_title")
    } else {
        state.i18n.ts("password_entry_create_title")
    };

    let title = Paragraph::new(title_text)
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(title, chunks[0]);

    let name_label = Paragraph::new(state.i18n.ts("password_entry_name_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(name_label, chunks[1]);

    let name_style = if state.password_entry_field == 0 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };

    let name_title = if state.password_entry_field == 0 {
        state.i18n.ts("password_entry_name_active")
    } else {
        state.i18n.ts("password_entry_name")
    };

    let name_border_style = if state.password_entry_field == 0 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };

    let name_input = Paragraph::new(state.password_entry_name.as_str())
        .style(name_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(name_border_style)
                .style(theme.surface_style())
                .title(name_title),
        );
    f.render_widget(name_input, chunks[2]);

    let password_label = Paragraph::new(state.i18n.ts("password_entry_password_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(password_label, chunks[3]);

    let password_display = if state.password_entry_show_password {
        state.password_entry_password.clone()
    } else {
        "*".repeat(state.password_entry_password.len())
    };

    let password_style = if state.password_entry_field == 1 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };

    let password_title = if state.password_entry_field == 1 {
        format!("{} | Ctrl+H - {}", state.i18n.ts("password_entry_password_active"), if state.password_entry_show_password { state.i18n.ts("hide") } else { state.i18n.ts("show") })
    } else {
        format!("{} | Ctrl+H - {}", state.i18n.ts("password_entry_password"), if state.password_entry_show_password { state.i18n.ts("hide") } else { state.i18n.ts("show") })
    };

    let password_border_style = if state.password_entry_field == 1 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };

    let password_input = Paragraph::new(password_display.as_str())
        .style(password_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(password_border_style)
                .style(theme.surface_style())
                .title(password_title),
        );
    f.render_widget(password_input, chunks[4]);

    let footer = Paragraph::new(state.i18n.ts("password_entry_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[6]);
}

fn render_help_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Заголовок
            Constraint::Min(0),    // Основной контент
            Constraint::Length(3), // Футер
        ])
        .split(f.size());

    // Заголовок
    let title = Paragraph::new(state.i18n.ts("help_title"))
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(title, chunks[0]);

    // Основной контент с описанием горячих клавиш
    let help_text = vec![
        state.i18n.ts("help_separator"),
        state.i18n.ts("help_main_screen_title"),
        state.i18n.ts("help_separator"),
        "",
        state.i18n.ts("help_main_ctrl_q"),
        state.i18n.ts("help_main_ctrl_n"),
        state.i18n.ts("help_main_ctrl_e"),
        state.i18n.ts("help_main_ctrl_c"),
        state.i18n.ts("help_main_ctrl_s"),
        state.i18n.ts("help_main_f1"),
        state.i18n.ts("help_main_f2"),
        state.i18n.ts("help_main_arrows"),
        state.i18n.ts("help_main_esc"),
        state.i18n.ts("help_main_backspace"),
        state.i18n.ts("help_main_type"),
        "",
        state.i18n.ts("help_separator"),
        state.i18n.ts("help_master_password_title"),
        state.i18n.ts("help_separator"),
        "",
        state.i18n.ts("help_master_password_enter"),
        state.i18n.ts("help_master_password_arrows"),
        state.i18n.ts("help_master_password_ctrl_h"),
        state.i18n.ts("help_master_password_f1"),
        state.i18n.ts("help_master_password_esc"),
        state.i18n.ts("help_master_password_backspace"),
        "",
        state.i18n.ts("help_separator"),
        state.i18n.ts("help_password_entry_title"),
        state.i18n.ts("help_separator"),
        "",
        state.i18n.ts("help_password_entry_enter"),
        state.i18n.ts("help_password_entry_esc"),
        state.i18n.ts("help_password_entry_arrows"),
        state.i18n.ts("help_password_entry_ctrl_h"),
        state.i18n.ts("help_password_entry_ctrl_g"),
        state.i18n.ts("help_password_entry_f1"),
        state.i18n.ts("help_password_entry_backspace"),
        "",
        state.i18n.ts("help_separator"),
        state.i18n.ts("help_password_generator_title"),
        state.i18n.ts("help_separator"),
        "",
        state.i18n.ts("help_password_generator_enter"),
        state.i18n.ts("help_password_generator_esc"),
        state.i18n.ts("help_password_generator_arrows"),
        state.i18n.ts("help_password_generator_space"),
        state.i18n.ts("help_password_generator_backspace"),
        state.i18n.ts("help_password_generator_type"),
        state.i18n.ts("help_password_generator_f1"),
        "",
        state.i18n.ts("help_separator"),
        state.i18n.ts("help_settings_title"),
        state.i18n.ts("help_separator"),
        "",
        state.i18n.ts("help_settings_enter"),
        state.i18n.ts("help_settings_esc"),
        state.i18n.ts("help_settings_arrows"),
        state.i18n.ts("help_settings_f1"),
        state.i18n.ts("help_settings_backspace"),
        "",
        state.i18n.ts("help_separator"),
        state.i18n.ts("help_help_title"),
        state.i18n.ts("help_separator"),
        "",
        state.i18n.ts("help_help_close"),
        "",
    ];

    let help_content = Paragraph::new(help_text.join("\n"))
        .style(theme.text_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.surface_style())
                .title(state.i18n.ts("help_navigation")),
        );
    f.render_widget(help_content, chunks[1]);

    // Футер
    let footer = Paragraph::new(state.i18n.ts("help_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[2]);
}

fn render_password_generator_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Заголовок
            Constraint::Length(1), // Метка для длины
            Constraint::Length(3), // Поле ввода длины
            Constraint::Length(1), // Метка для исключений
            Constraint::Length(3), // Поле ввода исключений
            Constraint::Length(1), // Пустая строка
            Constraint::Length(1), // Метка для галочек
            Constraint::Length(1), // Заглавные буквы
            Constraint::Length(1), // Строчные буквы
            Constraint::Length(1), // Цифры
            Constraint::Length(1), // Спецсимволы
            Constraint::Min(0),    // Остальное пространство
            Constraint::Length(3), // Футер
        ])
        .split(f.size());

    // Заголовок
    let title = Paragraph::new(state.i18n.ts("password_generator_title"))
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(title, chunks[0]);

    // Метка для длины
    let length_label = Paragraph::new(state.i18n.ts("password_generator_length_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(length_label, chunks[1]);

    // Поле ввода длины
    let length_style = if state.password_generator_selected_field == 0 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };
    let length_title = if state.password_generator_selected_field == 0 {
        state.i18n.ts("password_generator_length_active")
    } else {
        state.i18n.ts("password_generator_length")
    };
    let length_border_style = if state.password_generator_selected_field == 0 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };
    let length_input = Paragraph::new(state.password_generator_length.as_str())
        .style(length_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(length_border_style)
                .style(theme.surface_style())
                .title(length_title),
        );
    f.render_widget(length_input, chunks[2]);

    // Метка для исключений
    let exclude_label = Paragraph::new(state.i18n.ts("password_generator_exclude_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(exclude_label, chunks[3]);

    // Поле ввода исключений
    let exclude_style = if state.password_generator_selected_field == 1 {
        theme.active_input_style()
    } else {
        theme.inactive_input_style()
    };
    let exclude_title = if state.password_generator_selected_field == 1 {
        state.i18n.ts("password_generator_exclude_active")
    } else {
        state.i18n.ts("password_generator_exclude")
    };
    let exclude_border_style = if state.password_generator_selected_field == 1 {
        theme.active_border_style()
    } else {
        theme.inactive_border_style()
    };
    let exclude_input = Paragraph::new(state.password_generator_exclude_chars.as_str())
        .style(exclude_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(exclude_border_style)
                .style(theme.surface_style())
                .title(exclude_title),
        );
    f.render_widget(exclude_input, chunks[4]);

    // Метка для галочек
    let checkboxes_label = Paragraph::new(state.i18n.ts("password_generator_charsets_label"))
        .style(theme.text_style())
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(checkboxes_label, chunks[6]);

    // Галочки
    let checkbox_style = |field_idx: usize| {
        if state.password_generator_selected_field == field_idx {
            theme.active_input_style()
        } else {
            theme.text_style()
        }
    };

    // Заглавные буквы
    let uppercase_mark = if state.password_generator_use_uppercase { "[✓]" } else { "[ ]" };
    let uppercase_text = format!("{} {}", uppercase_mark, state.i18n.ts("password_generator_uppercase"));
    let uppercase_para = Paragraph::new(uppercase_text.as_str())
        .style(checkbox_style(2))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(uppercase_para, chunks[7]);

    // Строчные буквы
    let lowercase_mark = if state.password_generator_use_lowercase { "[✓]" } else { "[ ]" };
    let lowercase_text = format!("{} {}", lowercase_mark, state.i18n.ts("password_generator_lowercase"));
    let lowercase_para = Paragraph::new(lowercase_text.as_str())
        .style(checkbox_style(3))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(lowercase_para, chunks[8]);

    // Цифры
    let digits_mark = if state.password_generator_use_digits { "[✓]" } else { "[ ]" };
    let digits_text = format!("{} {}", digits_mark, state.i18n.ts("password_generator_digits"));
    let digits_para = Paragraph::new(digits_text.as_str())
        .style(checkbox_style(4))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(digits_para, chunks[9]);

    // Спецсимволы
    let special_mark = if state.password_generator_use_special { "[✓]" } else { "[ ]" };
    let special_text = format!("{} {}", special_mark, state.i18n.ts("password_generator_special"));
    let special_para = Paragraph::new(special_text.as_str())
        .style(checkbox_style(5))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(special_para, chunks[10]);

    // Футер
    let footer = Paragraph::new(state.i18n.ts("password_generator_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[12]);
}

fn render_theme_selection_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Заголовок
            Constraint::Min(0),    // Основной контент
            Constraint::Length(3), // Футер
        ])
        .split(f.size());

    // Заголовок
    let title = Paragraph::new(state.i18n.ts("theme_selection_title"))
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(title, chunks[0]);

    // Список тем
    let themes = vec![
        ("Textual / Modern Web", "textual_dark", "Глубокий темный фон с яркими зелеными акцентами"),
        ("VS Code Dark+", "vscode_style", "Классический стиль IDE с мягкими цветами"),
        ("OpenCode / Dark Modern", "opencode_style", "Нейтральный современный вид"),
    ];

    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(idx, (name, theme_id, desc))| {
            let prefix = if state.theme_selection_index == idx { ">> " } else { "   " };
            let is_selected = state.config.theme == *theme_id;
            let marker = if is_selected { " [✓]" } else { " [ ]" };
            let text = format!("{}{}{}\n     {}", prefix, marker, name, desc);
            ListItem::new(text)
                .style(if state.theme_selection_index == idx {
                    theme.selection_style()
                } else {
                    theme.text_style()
                })
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.surface_style())
                .title(state.i18n.ts("theme_selection_list_title"))
        );

    f.render_widget(list, chunks[1]);

    // Футер
    let footer = Paragraph::new(state.i18n.ts("theme_selection_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[2]);
}

fn render_language_selection_screen(f: &mut Frame, state: &TuiState, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Заголовок
            Constraint::Min(0),    // Основной контент
            Constraint::Length(3), // Футер
        ])
        .split(f.size());

    // Заголовок
    let title = Paragraph::new(state.i18n.ts("language_selection_title"))
        .style(theme.title_style())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.active_border_style())
                .style(theme.surface_style())
        );
    f.render_widget(title, chunks[0]);

    // Список языков
    let languages = Language::all();

    let items: Vec<ListItem> = languages
        .iter()
        .enumerate()
        .map(|(idx, lang)| {
            let prefix = if state.language_selection_index == idx { ">> " } else { "   " };
            let is_selected = state.config.language == lang.to_code();
            let marker = if is_selected { " [✓]" } else { " [ ]" };
            let text = format!("{}{}{}", prefix, marker, lang.display_name());
            ListItem::new(text)
                .style(if state.language_selection_index == idx {
                    theme.selection_style()
                } else {
                    theme.text_style()
                })
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.surface_style())
                .title(state.i18n.ts("language_selection_list_title"))
        );

    f.render_widget(list, chunks[1]);

    // Футер
    let footer = Paragraph::new(state.i18n.ts("language_selection_footer"))
        .style(theme.dimmed_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.inactive_border_style())
                .style(theme.status_bar_style())
        );
    f.render_widget(footer, chunks[2]);
}

