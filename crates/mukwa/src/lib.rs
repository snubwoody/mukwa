// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

pub mod controllers;
mod settings;
mod state;
#[cfg(test)]
mod tests;
pub mod ui;

pub use mukwa_core::error::{Error, Result};
pub use settings::SettingsStore;

pub use crate::state::AppState;
use crate::ui::MainWindow;
use mukwa_core::migrator::Migrator;
use mukwa_core::service::Service;
use rusqlite::Connection;
use slint::ComponentHandle;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

fn init_service(dir: impl AsRef<Path>) -> Result<Service> {
    let path = dir.as_ref().join("data.sqlite");
    debug!("Opening SQLite database at {:?}", &path);
    let mut connection = Connection::open(&path)?;
    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate(&mut connection)?;

    connection.pragma_update(None, "journal_mode", "WAL")?;
    let service = Service::new(connection);
    service.check_credit_account_categories()?;
    Ok(service)
}

/// Run the application.
pub fn run() -> Result<()> {
    let data_dir = if cfg!(debug_assertions) {
        PathBuf::from(".mukwa")
    } else {
        mukwa_core::data_dir()
    };

    let settings_dir = if cfg!(debug_assertions) {
        PathBuf::from(".mukwa")
    } else {
        mukwa_core::config_dir()
    };

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&settings_dir)?;
    let main_window = MainWindow::new()?;
    let service = init_service(data_dir)?;
    let settings = SettingsStore::open(settings_dir.join("settings.toml"))?;
    let state = AppState::new(service)?;
    controllers::bind_all(&main_window, &state, &settings)?;

    #[cfg(windows)]
    {
        use slint::winit_030::{
            WinitWindowAccessor,
            winit::platform::windows::{CornerPreference, WindowExtWindows},
        };
        let main_window_weak = main_window.as_weak();

        slint::spawn_local(async move {
            let main_window = main_window_weak.unwrap();
            let handle = main_window.window().winit_window().await.unwrap();
            handle.set_corner_preference(CornerPreference::Round);
        })
        .unwrap();
    }

    #[cfg(target_os = "linux")]
    slint::set_xdg_app_id("com.wakunguma.Mukwa")?;

    main_window.run()?;
    Ok(())
}
