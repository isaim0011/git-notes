use thiserror::Error;

#[derive(Error, Debug)]
pub enum GnError {
    #[error("Git error: {0}")]
    Git(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, GnError>;
