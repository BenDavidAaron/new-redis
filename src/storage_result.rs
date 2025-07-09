use std::fmt;

#[derive(Debug)]

pub enum StorageError {
    IncorrectRequest,
    CommandNotAvailable(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::IncorrectRequest => write!(f, "The client sent an incorrect request!"),
            StorageError::CommandNotAvailable(cmd) => write!(f, "The requested command `{}` is not available!", cmd),
        }
    }
}

pub type StorageResult<T> = Result<T, StorageError>;