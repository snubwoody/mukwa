use std::io;
use std::num::ParseFloatError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error(transparent)]
    UuidError(#[from] uuid::Error),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("{0}")]
    NotFound(String),
}

impl Error {
    pub fn not_found(message: &str) -> Self {
        Self::NotFound(message.to_owned())
    }
}
