use std::{fmt, io, result::Result as StdResult};

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    IOError,
    JibbyError,
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            message: kind.as_str().to_owned(),
        }
    }
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::IOError => "io error",
            ErrorKind::JibbyError => "invalid value",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            kind: ErrorKind::IOError,
            message: error.to_string(),
        }
    }
}

pub type Result<T, E = Error> = StdResult<T, E>;
