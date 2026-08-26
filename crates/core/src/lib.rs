// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

pub mod error;
pub mod fmt;
pub mod migrator;
mod money;
pub mod plot;
pub mod service;

pub use error::{Error, Result};
pub use money::{Currency, Money};
use rusqlite::Connection;
use std::path::PathBuf;

use crate::migrator::Migrator;

/// Opens an in memory sqlite database for testing.
pub fn create_test_db() -> Connection {
    let mut connection = Connection::open_in_memory().expect("Failed to open sqlite connection");
    let mut migrator = Migrator::new();
    migrator.load_embedded().unwrap();
    migrator.migrate(&mut connection).unwrap();
    connection
}

/// Returns the path to the application's data directory.
///
/// # Panics
/// Panics if the system data directory cannot be found.
pub fn data_dir() -> PathBuf {
    dirs::data_dir().unwrap().join("Mukwa")
}

/// Returns the path to the application's config directory.
///
/// # Panics
/// Panics if the system's config directory cannot be found.
pub fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap().join("Mukwa")
}

/// Returns the path to the application's log directory.
///
/// ## Platform specific
///
/// |Platform | Value                                |
/// | ------- | ------------------------------------ |
/// | Linux   | `$XDG_STATE_HOME`/Mukwa/logs         |
/// | macOS   | `$HOME`/Library/Logs/Mukwa           |
/// | Windows | `{LocalAppData}`/Mukwa/logs |
///
/// ## Panics
/// Panics if the system directories cannot be found.
pub fn log_dir() -> PathBuf {
    if cfg!(windows) {
        dirs::data_local_dir().unwrap().join("Mukwa/logs")
    } else if cfg!(target_os = "macos") {
        dirs::home_dir().unwrap().join("Library/Logs/Mukwa")
    } else {
        dirs::state_dir().unwrap().join("Mukwa/logs")
    }
}
