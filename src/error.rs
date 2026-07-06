use std::io;
use std::num::{ParseFloatError, ParseIntError};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

// TODO: use Error struct instead
// see <https://github.com/snubwoody/folio/blob/main/crates/folio-desktop/src/error.rs>
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error(transparent)]
    RusqliteError(#[from] rusqlite::Error),
    #[error(transparent)]
    JiffError(#[from] jiff::Error),
    #[error(transparent)]
    UuidError(#[from] uuid::Error),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("Invalid migration: {0}")]
    InvalidMigration(String),
    #[error("Error: {0}")]
    Generic(String),
}

impl Error {
    pub fn not_found(message: &str) -> Self {
        Self::NotFound(message.to_owned())
    }

    pub fn generic(message: &str) -> Self {
        Self::Generic(message.to_owned())
    }

    pub fn invalid_migration(message: &str) -> Self {
        Self::InvalidMigration(message.to_owned())
    }
}
