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

use crate::migrator::Migrator;

/// Opens an in memory sqlite database for testing.
pub fn create_test_db() -> Connection {
    let mut connection = Connection::open_in_memory().expect("Failed to open sqlite connection");
    let mut migrator = Migrator::new();
    migrator.load_embedded().unwrap();
    migrator.migrate(&mut connection).unwrap();
    connection
}
