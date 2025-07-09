use std::fmt;
use std::num;
use std::string::FromUtf8Error;

#[derive(Debug, PartialEq)]
pub enum RESPError {
    FromUtf8,
    OutOfBounds(usize),
    IncorrectLength(RESPLength),
    ParseInt,
    WrongType,
    Unknown,
}

impl fmt::Display for RESPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RESPError::FromUtf8 => write!(f, "Cannot convert from UTF-8"),
            RESPError::IncorrectLength(length) => write!(f, "Incorrect Length: {}", length),
            RESPError::OutOfBounds(index) => write!(f, "Index out of bounds at index {}", index),
            RESPError::ParseInt => write!(f, "Cannot parse string into integer"),
            RESPError::WrongType => write!(f, "Wrong Prefix for RESP type"),
            RESPError::Unknown => write!(f, "Unknown Format for RESP String"),
        }
    }
}

impl From<FromUtf8Error> for RESPError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::FromUtf8
    }
}

impl From<num::ParseIntError> for RESPError {
    fn from(_err: num::ParseIntError) -> Self {
        Self::ParseInt
    }
}

pub type RESPResult<T> = Result<T, RESPError>;

pub type RESPLength = i32;
