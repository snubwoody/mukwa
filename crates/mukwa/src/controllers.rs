// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::settings::SettingsStore;
use crate::state::AppState;
use crate::ui;
use crate::ui::MainWindow;
use slint::{ComponentHandle, Model};

mod analytics;
mod api;
mod calendar;
mod global;
mod settings;

pub fn bind_all(window: &MainWindow, state: &AppState, settings: &SettingsStore) {
    calendar::bind(window);
    analytics::bind(window, state);
    settings::bind(window, settings.clone());
    analytics::bind(window, state);
    global::bind(window, state);
    api::bind(window);

    bind_combobox_api(window);
}

fn bind_combobox_api(window: &MainWindow) {
    let api = window.global::<ui::ComboBoxApi>();

    api.on_find_index(|options, value| {
        for (index, option) in options.iter().enumerate() {
            if option.value == value {
                return index as i32;
            }
        }
        -1
    });
}
