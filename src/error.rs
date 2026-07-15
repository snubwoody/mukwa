// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::{Display, Formatter};
use std::io;
use std::num::{ParseFloatError, ParseIntError};

pub type Result<T> = std::result::Result<T, Error>;

/// Extension trait that provides extra context for errors.
pub trait ErrorExt<T, E> {
    /// Wraps the error value with additional context.
    fn context<C>(self, context: C) -> std::result::Result<T, Error>
    where
        C: Display + Send + Sync + 'static;

    /// Wraps the error value with additional context that is evaluated lazily.
    fn with_context<C, F>(self, f: F) -> std::result::Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

#[derive(Debug)]
pub struct Error {
    message: String,
    source: Option<Box<dyn std::error::Error + Send>>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

impl Error {
    /// Creates a new `Error`.
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_owned(),
            source: None,
        }
    }

    /// Creates a new `Error` with an underlying error source.
    pub fn with_source<E: std::error::Error + Send + 'static>(message: &str, source: E) -> Self {
        Self {
            message: message.to_owned(),
            source: Some(Box::new(source)),
        }
    }

    /// Returns a multiline string containing the error message and sources.
    pub fn report(&self) -> String {
        let mut message = format!("Error: {}", self);
        if self.source.is_some() {
            message.push_str("\n\tCaused by:")
        }
        let mut source = self
            .source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static));
        let mut index = 1;

        while let Some(s) = source {
            message.push_str(&format!("\n\t\t{index}: {}", s));
            source = s.source();
            index += 1
        }
        message
    }
}

impl<T, E> ErrorExt<T, E> for std::result::Result<T, E>
where
    E: std::error::Error + Send + 'static,
{
    fn context<C>(self, context: C) -> std::result::Result<T, Error>
    where
        C: Display,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Error::with_source(context.to_string().as_str(), error)),
        }
    }

    fn with_context<C, F>(self, context: F) -> std::result::Result<T, Error>
    where
        C: Display,
        F: FnOnce() -> C,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(Error::with_source(context().to_string().as_str(), error)),
        }
    }
}

/// Generates `From` impl blocks for external errors.
macro_rules! from_error {
    ($($t:ty),+) => {
        $(
            impl From<$t> for Error{
                fn from(value: $t) -> Self {
                    Error::with_source(&value.to_string(),value)
                }
            }
        )+
    };
}

from_error! {
    jiff::Error,
    io::Error,
    uuid::Error,
    ParseFloatError,
    ParseIntError,
    rusqlite::Error,
    std::str::Utf8Error
}
