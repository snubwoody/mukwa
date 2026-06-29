use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Sqlite error: {0}")]
    RusqliteError(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Invalid migration: {0}")]
    InvalidMigration(String),
    #[error(transparent)]
    UuidError(#[from] uuid::Error),
    #[error("Parse error: {0}")]
    ParseError(String),
}

impl Error {
    pub fn invalid_migration(message: &str) -> Self {
        Self::InvalidMigration(message.to_owned())
    }
}
