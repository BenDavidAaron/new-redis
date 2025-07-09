use std::fmt;

#[derive(Debug, PartialEq)]
pub enum StorageError {
    IncorrectRequest,
    StorageUnavailable,
    CommandNotAvailable(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::IncorrectRequest => write!(f, "The client sent an incorrect request!"),
            StorageError::StorageUnavailable => write!(f, "The storage is currently unavailable!"),
            StorageError::CommandNotAvailable(cmd) => write!(f, "The requested command `{}` is not available!", cmd),
        }
    }
}

pub type StorageResult<T> = Result<T, StorageError>;