use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpmError {
    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("TUI error: {0}")]
    Tui(String),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Tray error: {0}")]
    Tray(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type RpmResult<T> = Result<T, RpmError>;

