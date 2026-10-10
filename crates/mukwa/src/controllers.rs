// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::settings::SettingsStore;
use crate::state::AppState;
use crate::ui::MainWindow;

mod analytics;
mod api;
mod calendar;
mod category;
mod global;
mod import_csv;
mod settings;
mod transaction;

pub fn bind_all(
    window: &MainWindow,
    state: &AppState,
    settings: &SettingsStore,
) -> crate::Result<()> {
    let service = state.service();

    calendar::bind(window);
    analytics::bind(window, state);
    settings::bind(window, settings.clone());
    analytics::bind(window, state);
    global::bind(window, state);
    api::bind(window);
    import_csv::bind(window, state);
    category::bind(window, state)?;
    transaction::bind(window, service)?;
    Ok(())
}
