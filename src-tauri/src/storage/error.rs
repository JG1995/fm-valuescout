use std::fmt;

/// Internal error type for storage operations.
/// Command wrappers convert these to String for the Tauri boundary.
#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    Duplicate(String),
    Validation(String),
    Database(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound(msg) => write!(f, "{}", msg),
            StorageError::Duplicate(msg) => write!(f, "{}", msg),
            StorageError::Validation(msg) => write!(f, "{}", msg),
            StorageError::Database(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<StorageError> for String {
    fn from(err: StorageError) -> String {
        err.to_string()
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> StorageError {
        StorageError::Database(err.to_string())
    }
}
