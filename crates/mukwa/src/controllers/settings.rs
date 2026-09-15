// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::settings::SettingsStore;
use crate::ui;
use slint::{ComponentHandle, Global, ToSharedString};
use tracing::warn;

pub fn bind(window: &ui::MainWindow, settings: SettingsStore) {
    let settings_state = window.global::<ui::Settings>();

    settings_state.set_currency_code(settings.currency_code().to_shared_string());
    settings_state.set_font_family(settings.font_family().to_shared_string());

    settings_state.on_set_currency_code({
        let settings_state = settings_state.as_weak();
        let store = settings.clone();
        move |code| {
            if let Err(err) = store.set_currency_code(&code) {
                warn!("Failed to set currency code: {err}");
                return;
            }
            settings_state.unwrap().set_currency_code(code);
        }
    });

    settings_state.on_set_font_family({
        let settings_state = settings_state.as_weak();
        let store = settings.clone();
        move |family| {
            if let Err(err) = store.set_font_family(&family) {
                warn!("Failed to set font family: {err}");
                return;
            }
            settings_state.unwrap().set_font_family(family);
        }
    });
}
