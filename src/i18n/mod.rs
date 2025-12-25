use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh")]
    Chinese,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        match code {
            "en" => Language::English,
            "zh" => Language::Chinese,
            _ => Language::Russian,
        }
    }

    pub fn to_code(&self) -> &'static str {
        match self {
            Language::Russian => "ru",
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Russian => "Русский",
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Language::Russian, Language::English, Language::Chinese]
    }
}

pub struct I18n {
    translations: HashMap<String, String>,
    language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        let mut i18n = Self {
            translations: HashMap::new(),
            language,
        };
        i18n.load_translations();
        i18n
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
        self.translations.clear();
        self.load_translations();
    }

    pub fn get_language(&self) -> Language {
        self.language
    }

    fn load_translations(&mut self) {
        let translations = match self.language {
            Language::Russian => get_russian_translations(),
            Language::English => get_english_translations(),
            Language::Chinese => get_chinese_translations(),
        };
        self.translations = translations;
    }

    pub fn t<'a>(&'a self, key: &'a str) -> Cow<'a, str> {
        self.translations
            .get(key)
            .map(|s| Cow::Borrowed(s.as_str()))
            .unwrap_or_else(|| Cow::Borrowed(key))
    }

    /// Получить перевод как &str (для совместимости с ratatui)
    /// Использует минимальный lifetime из self и key
    pub fn ts<'a>(&'a self, key: &'a str) -> &'a str {
        // Если перевод найден, возвращаем его (lifetime 'a связан с self)
        // Если перевод не найден, возвращаем сам ключ (lifetime 'a связан с key)
        // Компилятор требует, чтобы оба имели одинаковый lifetime 'a
        self.translations.get(key).map(|s| s.as_str()).unwrap_or(key)
    }
}

fn get_russian_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // Master password screen
    map.insert("master_password_title".to_string(), "RPM - Rust Password Manager".to_string());
    map.insert("master_password_create_title".to_string(), "RPM - Rust Password Manager - Создание мастер-пароля".to_string());
    map.insert("master_password_enter".to_string(), "Введите мастер-пароль".to_string());
    map.insert("master_password_directory_label".to_string(), "Директория с паролями (оставьте пустым для использования пути по умолчанию):".to_string());
    map.insert("master_password_directory".to_string(), "Директория".to_string());
    map.insert("master_password_directory_active".to_string(), "Директория (активно)".to_string());
    map.insert("master_password_label".to_string(), "Пароль:".to_string());
    map.insert("master_password".to_string(), "Пароль".to_string());
    map.insert("master_password_active".to_string(), "Пароль (активно)".to_string());
    map.insert("master_password_confirm_label".to_string(), "Подтверждение:".to_string());
    map.insert("master_password_confirm".to_string(), "Подтверждение".to_string());
    map.insert("master_password_confirm_active".to_string(), "Подтверждение (активно)".to_string());
    map.insert("master_password_show_hide".to_string(), "Ctrl+H - показать/скрыть".to_string());
    map.insert("master_password_footer_create".to_string(), "Enter - продолжить/создать | ↑↓ - переключение полей | Ctrl+H - показать/скрыть пароль | Esc - выход".to_string());
    map.insert("master_password_footer_enter".to_string(), "Enter - подтвердить | Ctrl+H - показать/скрыть пароль | Esc - выход".to_string());
    
    // Main screen
    map.insert("main_search".to_string(), "Поиск (начните вводить для фильтрации)".to_string());
    map.insert("main_passwords".to_string(), "Passwords".to_string());
    map.insert("main_footer".to_string(), "F1 - помощь | Ctrl+Q - выход | Ctrl+N - новый пароль | Ctrl+E - редактировать | Ctrl+C - копировать пароль | Ctrl+S - настройки | ↑↓ - навигация | Esc - сброс поиска | Введите для поиска".to_string());
    
    // Settings screen
    map.insert("settings_title".to_string(), "Настройки".to_string());
    map.insert("settings_save_path_label".to_string(), "Путь сохранения паролей:".to_string());
    map.insert("settings_save_path_title".to_string(), "Текущий путь".to_string());
    map.insert("settings_config_path_label".to_string(), "Путь к конфигурационному файлу:".to_string());
    map.insert("settings_config_path_title".to_string(), "Файл конфигурации".to_string());
    map.insert("settings_config_path_error".to_string(), "Не удалось определить".to_string());
    map.insert("settings_directory_label".to_string(), "Директория с паролями (оставьте пустым для использования пути по умолчанию):".to_string());
    map.insert("settings_directory".to_string(), "Путь к директории".to_string());
    map.insert("settings_directory_active".to_string(), "Путь к директории (активно)".to_string());
    map.insert("settings_clipboard_timeout_label".to_string(), "Время хранения пароля в буфере обмена (секунды, 0 = не очищать):".to_string());
    map.insert("settings_clipboard_timeout".to_string(), "Время хранения".to_string());
    map.insert("settings_clipboard_timeout_active".to_string(), "Время хранения (активно)".to_string());
    map.insert("settings_theme_label".to_string(), "Тема интерфейса:".to_string());
    map.insert("settings_theme".to_string(), "Тема | Enter - выбрать".to_string());
    map.insert("settings_theme_active".to_string(), "Тема (активно) | Enter - выбрать".to_string());
    map.insert("settings_language_label".to_string(), "Язык интерфейса:".to_string());
    map.insert("settings_language".to_string(), "Язык | Enter - выбрать".to_string());
    map.insert("settings_language_active".to_string(), "Язык (активно) | Enter - выбрать".to_string());
    map.insert("settings_footer".to_string(), "Enter - сохранить/выбрать | Esc - отмена | ↑↓ - переключение полей | Введите значение".to_string());
    
    // Password entry screen
    map.insert("password_entry_create_title".to_string(), "Создание нового пароля".to_string());
    map.insert("password_entry_edit_title".to_string(), "Редактирование пароля".to_string());
    map.insert("password_entry_name_label".to_string(), "Имя:".to_string());
    map.insert("password_entry_name".to_string(), "Имя".to_string());
    map.insert("password_entry_name_active".to_string(), "Имя (активно)".to_string());
    map.insert("password_entry_password_label".to_string(), "Пароль:".to_string());
    map.insert("password_entry_password".to_string(), "Пароль | Ctrl+H - показать/скрыть".to_string());
    map.insert("password_entry_password_active".to_string(), "Пароль (активно) | Ctrl+H - показать/скрыть".to_string());
    map.insert("password_entry_footer".to_string(), "Enter - сохранить | Esc - отмена | ↑↓ - переключение полей | Ctrl+H - показать/скрыть пароль | Ctrl+G - генератор паролей".to_string());
    
    // Password generator screen
    map.insert("password_generator_title".to_string(), "Генератор паролей".to_string());
    map.insert("password_generator_length_label".to_string(), "Длина пароля:".to_string());
    map.insert("password_generator_length".to_string(), "Длина".to_string());
    map.insert("password_generator_length_active".to_string(), "Длина (активно)".to_string());
    map.insert("password_generator_exclude_label".to_string(), "Символы для исключения (по умолчанию пусто):".to_string());
    map.insert("password_generator_exclude".to_string(), "Исключения".to_string());
    map.insert("password_generator_exclude_active".to_string(), "Исключения (активно)".to_string());
    map.insert("password_generator_charsets_label".to_string(), "Наборы символов:".to_string());
    map.insert("password_generator_uppercase".to_string(), "Заглавные буквы (A-Z)".to_string());
    map.insert("password_generator_lowercase".to_string(), "Строчные буквы (a-z)".to_string());
    map.insert("password_generator_digits".to_string(), "Цифры (0-9)".to_string());
    map.insert("password_generator_special".to_string(), "Спецсимволы (!@#$%...)".to_string());
    map.insert("password_generator_footer".to_string(), "Enter - сгенерировать и вставить | Esc - отмена | ↑↓ - навигация | Space - переключить галочку | F1 - справка".to_string());
    
    // Theme selection screen
    map.insert("theme_selection_title".to_string(), "Выбор темы интерфейса".to_string());
    map.insert("theme_selection_list_title".to_string(), "Выберите тему (↑↓ для навигации)".to_string());
    map.insert("theme_selection_footer".to_string(), "Enter - выбрать тему | Esc - отмена | ↑↓ - навигация | F1 - справка".to_string());
    
    // Language selection screen
    map.insert("language_selection_title".to_string(), "Выбор языка интерфейса".to_string());
    map.insert("language_selection_list_title".to_string(), "Выберите язык (↑↓ для навигации)".to_string());
    map.insert("language_selection_footer".to_string(), "Enter - выбрать язык | Esc - отмена | ↑↓ - навигация | F1 - справка".to_string());
    
    // Help screen
    map.insert("help_title".to_string(), "Справка - Горячие клавиши".to_string());
    map.insert("help_navigation".to_string(), "Навигация: используйте прокрутку для просмотра".to_string());
    map.insert("help_footer".to_string(), "F1 / Esc - закрыть справку".to_string());
    map.insert("help_separator".to_string(), "═══════════════════════════════════════════════════════════════".to_string());
    map.insert("help_main_screen_title".to_string(), "ГЛАВНЫЙ ЭКРАН".to_string());
    map.insert("help_main_ctrl_q".to_string(), "  Ctrl+Q          - Выход из приложения".to_string());
    map.insert("help_main_ctrl_n".to_string(), "  Ctrl+N          - Создать новый пароль".to_string());
    map.insert("help_main_ctrl_e".to_string(), "  Ctrl+E          - Редактировать выбранный пароль".to_string());
    map.insert("help_main_ctrl_c".to_string(), "  Ctrl+C          - Копировать пароль в буфер обмена".to_string());
    map.insert("help_main_ctrl_s".to_string(), "  Ctrl+S          - Открыть настройки".to_string());
    map.insert("help_main_f1".to_string(), "  F1              - Открыть эту справку".to_string());
    map.insert("help_main_f2".to_string(), "  F2              - Открыть настройки".to_string());
    map.insert("help_main_arrows".to_string(), "  ↑ / ↓           - Навигация по списку".to_string());
    map.insert("help_main_esc".to_string(), "  Esc             - Сбросить поиск".to_string());
    map.insert("help_main_backspace".to_string(), "  Backspace       - Удалить символ из поиска".to_string());
    map.insert("help_main_type".to_string(), "  Ввод текста     - Поиск по паролям (fuzzy search)".to_string());
    map.insert("help_master_password_title".to_string(), "ЭКРАН МАСТЕР-ПАРОЛЯ".to_string());
    map.insert("help_master_password_enter".to_string(), "  Enter           - Продолжить/создать мастер-пароль".to_string());
    map.insert("help_master_password_arrows".to_string(), "  ↑ / ↓           - Переключение между полями".to_string());
    map.insert("help_master_password_ctrl_h".to_string(), "  Ctrl+H          - Показать/скрыть пароль".to_string());
    map.insert("help_master_password_f1".to_string(), "  F1              - Открыть справку".to_string());
    map.insert("help_master_password_esc".to_string(), "  Esc             - Выход из приложения".to_string());
    map.insert("help_master_password_backspace".to_string(), "  Backspace       - Удалить символ".to_string());
    map.insert("help_password_entry_title".to_string(), "ЭКРАН СОЗДАНИЯ/РЕДАКТИРОВАНИЯ ПАРОЛЯ".to_string());
    map.insert("help_password_entry_enter".to_string(), "  Enter           - Сохранить пароль".to_string());
    map.insert("help_password_entry_esc".to_string(), "  Esc             - Отмена и возврат к главному экрану".to_string());
    map.insert("help_password_entry_arrows".to_string(), "  ↑ / ↓           - Переключение между полями (имя/пароль)".to_string());
    map.insert("help_password_entry_ctrl_h".to_string(), "  Ctrl+H          - Показать/скрыть пароль".to_string());
    map.insert("help_password_entry_ctrl_g".to_string(), "  Ctrl+G          - Открыть генератор паролей".to_string());
    map.insert("help_password_entry_f1".to_string(), "  F1              - Открыть справку".to_string());
    map.insert("help_password_entry_backspace".to_string(), "  Backspace       - Удалить символ".to_string());
    map.insert("help_password_generator_title".to_string(), "ЭКРАН ГЕНЕРАТОРА ПАРОЛЕЙ".to_string());
    map.insert("help_password_generator_enter".to_string(), "  Enter           - Сгенерировать пароль и вставить".to_string());
    map.insert("help_password_generator_esc".to_string(), "  Esc             - Отмена и возврат к экрану пароля".to_string());
    map.insert("help_password_generator_arrows".to_string(), "  ↑ / ↓           - Навигация по элементам".to_string());
    map.insert("help_password_generator_space".to_string(), "  Space           - Переключить галочку (для наборов символов)".to_string());
    map.insert("help_password_generator_backspace".to_string(), "  Backspace       - Удалить символ в активном поле".to_string());
    map.insert("help_password_generator_type".to_string(), "  Ввод символов   - Ввод в активное поле (длина/исключения)".to_string());
    map.insert("help_password_generator_f1".to_string(), "  F1              - Открыть справку".to_string());
    map.insert("help_settings_title".to_string(), "ЭКРАН НАСТРОЕК".to_string());
    map.insert("help_settings_enter".to_string(), "  Enter           - Сохранить настройки".to_string());
    map.insert("help_settings_esc".to_string(), "  Esc / Q         - Отмена и возврат к главному экрану".to_string());
    map.insert("help_settings_arrows".to_string(), "  ↑ / ↓           - Переключение между полями".to_string());
    map.insert("help_settings_f1".to_string(), "  F1              - Открыть справку".to_string());
    map.insert("help_settings_backspace".to_string(), "  Backspace       - Удалить символ".to_string());
    map.insert("help_help_title".to_string(), "СПРАВКА".to_string());
    map.insert("help_help_close".to_string(), "  F1 / Esc        - Закрыть справку и вернуться".to_string());
    
    // Common
    map.insert("show".to_string(), "показать".to_string());
    map.insert("hide".to_string(), "скрыть".to_string());
    
    map
}

fn get_english_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // Master password screen
    map.insert("master_password_title".to_string(), "RPM - Rust Password Manager".to_string());
    map.insert("master_password_create_title".to_string(), "RPM - Rust Password Manager - Create Master Password".to_string());
    map.insert("master_password_enter".to_string(), "Enter master password".to_string());
    map.insert("master_password_directory_label".to_string(), "Passwords directory (leave empty to use default path):".to_string());
    map.insert("master_password_directory".to_string(), "Directory".to_string());
    map.insert("master_password_directory_active".to_string(), "Directory (active)".to_string());
    map.insert("master_password_label".to_string(), "Password:".to_string());
    map.insert("master_password".to_string(), "Password".to_string());
    map.insert("master_password_active".to_string(), "Password (active)".to_string());
    map.insert("master_password_confirm_label".to_string(), "Confirm:".to_string());
    map.insert("master_password_confirm".to_string(), "Confirm".to_string());
    map.insert("master_password_confirm_active".to_string(), "Confirm (active)".to_string());
    map.insert("master_password_show_hide".to_string(), "Ctrl+H - show/hide".to_string());
    map.insert("master_password_footer_create".to_string(), "Enter - continue/create | ↑↓ - switch fields | Ctrl+H - show/hide password | Esc - exit".to_string());
    map.insert("master_password_footer_enter".to_string(), "Enter - confirm | Ctrl+H - show/hide password | Esc - exit".to_string());
    
    // Main screen
    map.insert("main_search".to_string(), "Search (start typing to filter)".to_string());
    map.insert("main_passwords".to_string(), "Passwords".to_string());
    map.insert("main_footer".to_string(), "F1 - help | Ctrl+Q - quit | Ctrl+N - new password | Ctrl+E - edit | Ctrl+C - copy password | Ctrl+S - settings | ↑↓ - navigation | Esc - reset search | Type to search".to_string());
    
    // Settings screen
    map.insert("settings_title".to_string(), "Settings".to_string());
    map.insert("settings_save_path_label".to_string(), "Passwords save path:".to_string());
    map.insert("settings_save_path_title".to_string(), "Current path".to_string());
    map.insert("settings_config_path_label".to_string(), "Configuration file path:".to_string());
    map.insert("settings_config_path_title".to_string(), "Configuration file".to_string());
    map.insert("settings_config_path_error".to_string(), "Could not determine".to_string());
    map.insert("settings_directory_label".to_string(), "Passwords directory (leave empty to use default path):".to_string());
    map.insert("settings_directory".to_string(), "Directory path".to_string());
    map.insert("settings_directory_active".to_string(), "Directory path (active)".to_string());
    map.insert("settings_clipboard_timeout_label".to_string(), "Clipboard timeout (seconds, 0 = don't clear):".to_string());
    map.insert("settings_clipboard_timeout".to_string(), "Timeout".to_string());
    map.insert("settings_clipboard_timeout_active".to_string(), "Timeout (active)".to_string());
    map.insert("settings_theme_label".to_string(), "Interface theme:".to_string());
    map.insert("settings_theme".to_string(), "Theme | Enter - select".to_string());
    map.insert("settings_theme_active".to_string(), "Theme (active) | Enter - select".to_string());
    map.insert("settings_language_label".to_string(), "Interface language:".to_string());
    map.insert("settings_language".to_string(), "Language | Enter - select".to_string());
    map.insert("settings_language_active".to_string(), "Language (active) | Enter - select".to_string());
    map.insert("settings_footer".to_string(), "Enter - save/select | Esc - cancel | ↑↓ - switch fields | Enter value".to_string());
    
    // Password entry screen
    map.insert("password_entry_create_title".to_string(), "Create New Password".to_string());
    map.insert("password_entry_edit_title".to_string(), "Edit Password".to_string());
    map.insert("password_entry_name_label".to_string(), "Name:".to_string());
    map.insert("password_entry_name".to_string(), "Name".to_string());
    map.insert("password_entry_name_active".to_string(), "Name (active)".to_string());
    map.insert("password_entry_password_label".to_string(), "Password:".to_string());
    map.insert("password_entry_password".to_string(), "Password | Ctrl+H - show/hide".to_string());
    map.insert("password_entry_password_active".to_string(), "Password (active) | Ctrl+H - show/hide".to_string());
    map.insert("password_entry_footer".to_string(), "Enter - save | Esc - cancel | ↑↓ - switch fields | Ctrl+H - show/hide password | Ctrl+G - password generator".to_string());
    
    // Password generator screen
    map.insert("password_generator_title".to_string(), "Password Generator".to_string());
    map.insert("password_generator_length_label".to_string(), "Password length:".to_string());
    map.insert("password_generator_length".to_string(), "Length".to_string());
    map.insert("password_generator_length_active".to_string(), "Length (active)".to_string());
    map.insert("password_generator_exclude_label".to_string(), "Characters to exclude (empty by default):".to_string());
    map.insert("password_generator_exclude".to_string(), "Exclude".to_string());
    map.insert("password_generator_exclude_active".to_string(), "Exclude (active)".to_string());
    map.insert("password_generator_charsets_label".to_string(), "Character sets:".to_string());
    map.insert("password_generator_uppercase".to_string(), "Uppercase letters (A-Z)".to_string());
    map.insert("password_generator_lowercase".to_string(), "Lowercase letters (a-z)".to_string());
    map.insert("password_generator_digits".to_string(), "Digits (0-9)".to_string());
    map.insert("password_generator_special".to_string(), "Special characters (!@#$%...)".to_string());
    map.insert("password_generator_footer".to_string(), "Enter - generate and insert | Esc - cancel | ↑↓ - navigation | Space - toggle checkbox | F1 - help".to_string());
    
    // Theme selection screen
    map.insert("theme_selection_title".to_string(), "Select Interface Theme".to_string());
    map.insert("theme_selection_list_title".to_string(), "Select theme (↑↓ for navigation)".to_string());
    map.insert("theme_selection_footer".to_string(), "Enter - select theme | Esc - cancel | ↑↓ - navigation | F1 - help".to_string());
    
    // Language selection screen
    map.insert("language_selection_title".to_string(), "Select Interface Language".to_string());
    map.insert("language_selection_list_title".to_string(), "Select language (↑↓ for navigation)".to_string());
    map.insert("language_selection_footer".to_string(), "Enter - select language | Esc - cancel | ↑↓ - navigation | F1 - help".to_string());
    
    // Help screen
    map.insert("help_title".to_string(), "Help - Hotkeys".to_string());
    map.insert("help_navigation".to_string(), "Navigation: use scroll to view".to_string());
    map.insert("help_footer".to_string(), "F1 / Esc - close help".to_string());
    map.insert("help_separator".to_string(), "═══════════════════════════════════════════════════════════════".to_string());
    map.insert("help_main_screen_title".to_string(), "MAIN SCREEN".to_string());
    map.insert("help_main_ctrl_q".to_string(), "  Ctrl+Q          - Quit application".to_string());
    map.insert("help_main_ctrl_n".to_string(), "  Ctrl+N          - Create new password".to_string());
    map.insert("help_main_ctrl_e".to_string(), "  Ctrl+E          - Edit selected password".to_string());
    map.insert("help_main_ctrl_c".to_string(), "  Ctrl+C          - Copy password to clipboard".to_string());
    map.insert("help_main_ctrl_s".to_string(), "  Ctrl+S          - Open settings".to_string());
    map.insert("help_main_f1".to_string(), "  F1              - Open this help".to_string());
    map.insert("help_main_f2".to_string(), "  F2              - Open settings".to_string());
    map.insert("help_main_arrows".to_string(), "  ↑ / ↓           - Navigate list".to_string());
    map.insert("help_main_esc".to_string(), "  Esc             - Reset search".to_string());
    map.insert("help_main_backspace".to_string(), "  Backspace       - Delete character from search".to_string());
    map.insert("help_main_type".to_string(), "  Type text       - Search passwords (fuzzy search)".to_string());
    map.insert("help_master_password_title".to_string(), "MASTER PASSWORD SCREEN".to_string());
    map.insert("help_master_password_enter".to_string(), "  Enter           - Continue/create master password".to_string());
    map.insert("help_master_password_arrows".to_string(), "  ↑ / ↓           - Switch between fields".to_string());
    map.insert("help_master_password_ctrl_h".to_string(), "  Ctrl+H          - Show/hide password".to_string());
    map.insert("help_master_password_f1".to_string(), "  F1              - Open help".to_string());
    map.insert("help_master_password_esc".to_string(), "  Esc             - Quit application".to_string());
    map.insert("help_master_password_backspace".to_string(), "  Backspace       - Delete character".to_string());
    map.insert("help_password_entry_title".to_string(), "PASSWORD ENTRY SCREEN".to_string());
    map.insert("help_password_entry_enter".to_string(), "  Enter           - Save password".to_string());
    map.insert("help_password_entry_esc".to_string(), "  Esc             - Cancel and return to main screen".to_string());
    map.insert("help_password_entry_arrows".to_string(), "  ↑ / ↓           - Switch between fields (name/password)".to_string());
    map.insert("help_password_entry_ctrl_h".to_string(), "  Ctrl+H          - Show/hide password".to_string());
    map.insert("help_password_entry_ctrl_g".to_string(), "  Ctrl+G          - Open password generator".to_string());
    map.insert("help_password_entry_f1".to_string(), "  F1              - Open help".to_string());
    map.insert("help_password_entry_backspace".to_string(), "  Backspace       - Delete character".to_string());
    map.insert("help_password_generator_title".to_string(), "PASSWORD GENERATOR SCREEN".to_string());
    map.insert("help_password_generator_enter".to_string(), "  Enter           - Generate password and insert".to_string());
    map.insert("help_password_generator_esc".to_string(), "  Esc             - Cancel and return to password screen".to_string());
    map.insert("help_password_generator_arrows".to_string(), "  ↑ / ↓           - Navigate elements".to_string());
    map.insert("help_password_generator_space".to_string(), "  Space           - Toggle checkbox (for character sets)".to_string());
    map.insert("help_password_generator_backspace".to_string(), "  Backspace       - Delete character in active field".to_string());
    map.insert("help_password_generator_type".to_string(), "  Type characters - Input in active field (length/exclude)".to_string());
    map.insert("help_password_generator_f1".to_string(), "  F1              - Open help".to_string());
    map.insert("help_settings_title".to_string(), "SETTINGS SCREEN".to_string());
    map.insert("help_settings_enter".to_string(), "  Enter           - Save settings".to_string());
    map.insert("help_settings_esc".to_string(), "  Esc / Q         - Cancel and return to main screen".to_string());
    map.insert("help_settings_arrows".to_string(), "  ↑ / ↓           - Switch between fields".to_string());
    map.insert("help_settings_f1".to_string(), "  F1              - Open help".to_string());
    map.insert("help_settings_backspace".to_string(), "  Backspace       - Delete character".to_string());
    map.insert("help_help_title".to_string(), "HELP".to_string());
    map.insert("help_help_close".to_string(), "  F1 / Esc        - Close help and return".to_string());
    
    // Common
    map.insert("show".to_string(), "show".to_string());
    map.insert("hide".to_string(), "hide".to_string());
    
    map
}

fn get_chinese_translations() -> HashMap<String, String> {
    let mut map = HashMap::new();
    
    // Master password screen
    map.insert("master_password_title".to_string(), "RPM - Rust Password Manager".to_string());
    map.insert("master_password_create_title".to_string(), "RPM - Rust Password Manager - 创建主密码".to_string());
    map.insert("master_password_enter".to_string(), "输入主密码".to_string());
    map.insert("master_password_directory_label".to_string(), "密码目录（留空以使用默认路径）：".to_string());
    map.insert("master_password_directory".to_string(), "目录".to_string());
    map.insert("master_password_directory_active".to_string(), "目录（活动）".to_string());
    map.insert("master_password_label".to_string(), "密码：".to_string());
    map.insert("master_password".to_string(), "密码".to_string());
    map.insert("master_password_active".to_string(), "密码（活动）".to_string());
    map.insert("master_password_confirm_label".to_string(), "确认：".to_string());
    map.insert("master_password_confirm".to_string(), "确认".to_string());
    map.insert("master_password_confirm_active".to_string(), "确认（活动）".to_string());
    map.insert("master_password_show_hide".to_string(), "Ctrl+H - 显示/隐藏".to_string());
    map.insert("master_password_footer_create".to_string(), "Enter - 继续/创建 | ↑↓ - 切换字段 | Ctrl+H - 显示/隐藏密码 | Esc - 退出".to_string());
    map.insert("master_password_footer_enter".to_string(), "Enter - 确认 | Ctrl+H - 显示/隐藏密码 | Esc - 退出".to_string());
    
    // Main screen
    map.insert("main_search".to_string(), "搜索（开始输入以过滤）".to_string());
    map.insert("main_passwords".to_string(), "密码".to_string());
    map.insert("main_footer".to_string(), "F1 - 帮助 | Ctrl+Q - 退出 | Ctrl+N - 新密码 | Ctrl+E - 编辑 | Ctrl+C - 复制密码 | Ctrl+S - 设置 | ↑↓ - 导航 | Esc - 重置搜索 | 输入以搜索".to_string());
    
    // Settings screen
    map.insert("settings_title".to_string(), "设置".to_string());
    map.insert("settings_save_path_label".to_string(), "密码保存路径：".to_string());
    map.insert("settings_save_path_title".to_string(), "当前路径".to_string());
    map.insert("settings_config_path_label".to_string(), "配置文件路径：".to_string());
    map.insert("settings_config_path_title".to_string(), "配置文件".to_string());
    map.insert("settings_config_path_error".to_string(), "无法确定".to_string());
    map.insert("settings_directory_label".to_string(), "密码目录（留空以使用默认路径）：".to_string());
    map.insert("settings_directory".to_string(), "目录路径".to_string());
    map.insert("settings_directory_active".to_string(), "目录路径（活动）".to_string());
    map.insert("settings_clipboard_timeout_label".to_string(), "剪贴板超时（秒，0 = 不清除）：".to_string());
    map.insert("settings_clipboard_timeout".to_string(), "超时".to_string());
    map.insert("settings_clipboard_timeout_active".to_string(), "超时（活动）".to_string());
    map.insert("settings_theme_label".to_string(), "界面主题：".to_string());
    map.insert("settings_theme".to_string(), "主题 | Enter - 选择".to_string());
    map.insert("settings_theme_active".to_string(), "主题（活动） | Enter - 选择".to_string());
    map.insert("settings_language_label".to_string(), "界面语言：".to_string());
    map.insert("settings_language".to_string(), "语言 | Enter - 选择".to_string());
    map.insert("settings_language_active".to_string(), "语言（活动） | Enter - 选择".to_string());
    map.insert("settings_footer".to_string(), "Enter - 保存/选择 | Esc - 取消 | ↑↓ - 切换字段 | 输入值".to_string());
    
    // Password entry screen
    map.insert("password_entry_create_title".to_string(), "创建新密码".to_string());
    map.insert("password_entry_edit_title".to_string(), "编辑密码".to_string());
    map.insert("password_entry_name_label".to_string(), "名称：".to_string());
    map.insert("password_entry_name".to_string(), "名称".to_string());
    map.insert("password_entry_name_active".to_string(), "名称（活动）".to_string());
    map.insert("password_entry_password_label".to_string(), "密码：".to_string());
    map.insert("password_entry_password".to_string(), "密码 | Ctrl+H - 显示/隐藏".to_string());
    map.insert("password_entry_password_active".to_string(), "密码（活动） | Ctrl+H - 显示/隐藏".to_string());
    map.insert("password_entry_footer".to_string(), "Enter - 保存 | Esc - 取消 | ↑↓ - 切换字段 | Ctrl+H - 显示/隐藏密码 | Ctrl+G - 密码生成器".to_string());
    
    // Password generator screen
    map.insert("password_generator_title".to_string(), "密码生成器".to_string());
    map.insert("password_generator_length_label".to_string(), "密码长度：".to_string());
    map.insert("password_generator_length".to_string(), "长度".to_string());
    map.insert("password_generator_length_active".to_string(), "长度（活动）".to_string());
    map.insert("password_generator_exclude_label".to_string(), "要排除的字符（默认为空）：".to_string());
    map.insert("password_generator_exclude".to_string(), "排除".to_string());
    map.insert("password_generator_exclude_active".to_string(), "排除（活动）".to_string());
    map.insert("password_generator_charsets_label".to_string(), "字符集：".to_string());
    map.insert("password_generator_uppercase".to_string(), "大写字母 (A-Z)".to_string());
    map.insert("password_generator_lowercase".to_string(), "小写字母 (a-z)".to_string());
    map.insert("password_generator_digits".to_string(), "数字 (0-9)".to_string());
    map.insert("password_generator_special".to_string(), "特殊字符 (!@#$%...)".to_string());
    map.insert("password_generator_footer".to_string(), "Enter - 生成并插入 | Esc - 取消 | ↑↓ - 导航 | Space - 切换复选框 | F1 - 帮助".to_string());
    
    // Theme selection screen
    map.insert("theme_selection_title".to_string(), "选择界面主题".to_string());
    map.insert("theme_selection_list_title".to_string(), "选择主题（↑↓ 导航）".to_string());
    map.insert("theme_selection_footer".to_string(), "Enter - 选择主题 | Esc - 取消 | ↑↓ - 导航 | F1 - 帮助".to_string());
    
    // Language selection screen
    map.insert("language_selection_title".to_string(), "选择界面语言".to_string());
    map.insert("language_selection_list_title".to_string(), "选择语言（↑↓ 导航）".to_string());
    map.insert("language_selection_footer".to_string(), "Enter - 选择语言 | Esc - 取消 | ↑↓ - 导航 | F1 - 帮助".to_string());
    
    // Help screen
    map.insert("help_title".to_string(), "帮助 - 快捷键".to_string());
    map.insert("help_navigation".to_string(), "导航：使用滚动查看".to_string());
    map.insert("help_footer".to_string(), "F1 / Esc - 关闭帮助".to_string());
    map.insert("help_separator".to_string(), "═══════════════════════════════════════════════════════════════".to_string());
    map.insert("help_main_screen_title".to_string(), "主屏幕".to_string());
    map.insert("help_main_ctrl_q".to_string(), "  Ctrl+Q          - 退出应用程序".to_string());
    map.insert("help_main_ctrl_n".to_string(), "  Ctrl+N          - 创建新密码".to_string());
    map.insert("help_main_ctrl_e".to_string(), "  Ctrl+E          - 编辑所选密码".to_string());
    map.insert("help_main_ctrl_c".to_string(), "  Ctrl+C          - 复制密码到剪贴板".to_string());
    map.insert("help_main_ctrl_s".to_string(), "  Ctrl+S          - 打开设置".to_string());
    map.insert("help_main_f1".to_string(), "  F1              - 打开此帮助".to_string());
    map.insert("help_main_f2".to_string(), "  F2              - 打开设置".to_string());
    map.insert("help_main_arrows".to_string(), "  ↑ / ↓           - 导航列表".to_string());
    map.insert("help_main_esc".to_string(), "  Esc             - 重置搜索".to_string());
    map.insert("help_main_backspace".to_string(), "  Backspace       - 从搜索中删除字符".to_string());
    map.insert("help_main_type".to_string(), "  输入文本       - 搜索密码（模糊搜索）".to_string());
    map.insert("help_master_password_title".to_string(), "主密码屏幕".to_string());
    map.insert("help_master_password_enter".to_string(), "  Enter           - 继续/创建主密码".to_string());
    map.insert("help_master_password_arrows".to_string(), "  ↑ / ↓           - 在字段之间切换".to_string());
    map.insert("help_master_password_ctrl_h".to_string(), "  Ctrl+H          - 显示/隐藏密码".to_string());
    map.insert("help_master_password_f1".to_string(), "  F1              - 打开帮助".to_string());
    map.insert("help_master_password_esc".to_string(), "  Esc             - 退出应用程序".to_string());
    map.insert("help_master_password_backspace".to_string(), "  Backspace       - 删除字符".to_string());
    map.insert("help_password_entry_title".to_string(), "密码输入屏幕".to_string());
    map.insert("help_password_entry_enter".to_string(), "  Enter           - 保存密码".to_string());
    map.insert("help_password_entry_esc".to_string(), "  Esc             - 取消并返回主屏幕".to_string());
    map.insert("help_password_entry_arrows".to_string(), "  ↑ / ↓           - 在字段之间切换（名称/密码）".to_string());
    map.insert("help_password_entry_ctrl_h".to_string(), "  Ctrl+H          - 显示/隐藏密码".to_string());
    map.insert("help_password_entry_ctrl_g".to_string(), "  Ctrl+G          - 打开密码生成器".to_string());
    map.insert("help_password_entry_f1".to_string(), "  F1              - 打开帮助".to_string());
    map.insert("help_password_entry_backspace".to_string(), "  Backspace       - 删除字符".to_string());
    map.insert("help_password_generator_title".to_string(), "密码生成器屏幕".to_string());
    map.insert("help_password_generator_enter".to_string(), "  Enter           - 生成密码并插入".to_string());
    map.insert("help_password_generator_esc".to_string(), "  Esc             - 取消并返回密码屏幕".to_string());
    map.insert("help_password_generator_arrows".to_string(), "  ↑ / ↓           - 导航元素".to_string());
    map.insert("help_password_generator_space".to_string(), "  Space           - 切换复选框（字符集）".to_string());
    map.insert("help_password_generator_backspace".to_string(), "  Backspace       - 删除活动字段中的字符".to_string());
    map.insert("help_password_generator_type".to_string(), "  输入字符       - 在活动字段中输入（长度/排除）".to_string());
    map.insert("help_password_generator_f1".to_string(), "  F1              - 打开帮助".to_string());
    map.insert("help_settings_title".to_string(), "设置屏幕".to_string());
    map.insert("help_settings_enter".to_string(), "  Enter           - 保存设置".to_string());
    map.insert("help_settings_esc".to_string(), "  Esc / Q         - 取消并返回主屏幕".to_string());
    map.insert("help_settings_arrows".to_string(), "  ↑ / ↓           - 在字段之间切换".to_string());
    map.insert("help_settings_f1".to_string(), "  F1              - 打开帮助".to_string());
    map.insert("help_settings_backspace".to_string(), "  Backspace       - 删除字符".to_string());
    map.insert("help_help_title".to_string(), "帮助".to_string());
    map.insert("help_help_close".to_string(), "  F1 / Esc        - 关闭帮助并返回".to_string());
    
    // Common
    map.insert("show".to_string(), "显示".to_string());
    map.insert("hide".to_string(), "隐藏".to_string());
    
    map
}

