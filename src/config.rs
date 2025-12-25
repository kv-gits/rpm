use anyhow::Result;
use dirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_port: u16,
    pub server_host: String,
    pub master_password_hash: Option<String>,
    pub encryption_algorithm: String,
    pub passwords_directory: Option<PathBuf>,
    pub encryption_key_salt: Option<String>, // Base64 encoded salt for key derivation
    /// Время хранения пароля в буфере обмена в секундах (0 = не очищать автоматически)
    #[serde(default = "default_clipboard_timeout")]
    pub clipboard_timeout_seconds: u64,
    /// Выбранная тема TUI: "textual_dark", "vscode_style", "opencode_style"
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Выбранный язык интерфейса: "ru", "en", "zh"
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_theme() -> String {
    "textual_dark".to_string()
}

fn default_clipboard_timeout() -> u64 {
    30 // 30 секунд по умолчанию
}

fn default_language() -> String {
    "en".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_port: 8765,
            server_host: "127.0.0.1".to_string(),
            master_password_hash: None,
            encryption_algorithm: "aes256-gcm".to_string(),
            passwords_directory: None,
            encryption_key_salt: None,
            clipboard_timeout_seconds: default_clipboard_timeout(),
            theme: default_theme(),
            language: default_language(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rpm");
        Ok(config_dir.join("config.toml"))
    }

    /// Получить путь к директории с паролями
    /// Если не задана в конфиге, возвращает дефолтную директорию данных
    pub fn passwords_directory_path(&self) -> PathBuf {
        self.passwords_directory.clone().unwrap_or_else(|| {
            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("rpm")
                .join("passwords")
        })
    }

    /// Получить путь к файлу конфигурации
    pub fn config_file_path(&self) -> Result<PathBuf> {
        Self::config_path()
    }
}

/// Конфигурация директории с паролями
/// Хранится в файле `.rpm_config` внутри директории
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub master_password_hash: Option<String>,
    pub encryption_key_salt: Option<String>, // Base64 encoded salt for key derivation
}

impl DirectoryConfig {
    /// Путь к файлу конфигурации директории
    fn config_path(directory: &Path) -> PathBuf {
        directory.join(".rpm_config")
    }

    /// Загрузить конфигурацию директории
    pub fn load(directory: &Path) -> Result<Self> {
        let config_path = Self::config_path(directory);
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: DirectoryConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Возвращаем пустую конфигурацию, если файл не существует
            Ok(DirectoryConfig {
                master_password_hash: None,
                encryption_key_salt: None,
            })
        }
    }

    /// Сохранить конфигурацию директории
    pub fn save(&self, directory: &Path) -> Result<()> {
        // Убеждаемся, что директория существует
        std::fs::create_dir_all(directory)?;
        
        let config_path = Self::config_path(directory);
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    /// Проверить, установлен ли мастер-пароль для директории
    pub fn has_master_password(&self) -> bool {
        self.master_password_hash.is_some()
    }
}

