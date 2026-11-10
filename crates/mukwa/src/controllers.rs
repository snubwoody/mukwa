// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use mukwa_core::service::Service;

use crate::settings::SettingsStore;
use crate::ui::MainWindow;

mod account;
mod analytics;
mod api;
mod auto_update;
mod calendar;
mod category;
mod global;
mod import_csv;
mod settings;
mod transaction;

/// Binds the state and callbacks of all Slint globals to Rust code.
pub fn bind_all(
    window: &MainWindow,
    service: &Service,
    settings: &SettingsStore,
) -> crate::Result<()> {
    account::bind(window, service)?;
    calendar::bind(window);
    analytics::bind(window, service);
    settings::bind(window, settings.clone());
    global::bind(window);
    api::bind(window);
    import_csv::bind(window, service);
    category::bind(window, service)?;
    transaction::bind(window, service.clone())?;
    auto_update::bind(window)?;
    Ok(())
}
