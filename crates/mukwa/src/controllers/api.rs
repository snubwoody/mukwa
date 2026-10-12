// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use crate::ui::{Api, AutoUpdaterState};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::auto_update::{
    self, Artifact, Manifest, Platform, Release, download_update, install_update,
};
use mukwa_core::error::ErrorExt;
use mukwa_core::fmt::CurrencyFormatter;
use semver::Version;
use slint::{ComponentHandle, Global, ToSharedString};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info, warn};

pub fn bind(main_window: &ui::MainWindow) {
    let api = main_window.global::<ui::Api>();

    let _ = slint::spawn_local({
        let api = api.as_weak();
        async move {
            // Give the app some time to setup the state and callbacks
            smol::Timer::after(Duration::from_secs(10)).await;
            let api = api.unwrap();
            if let Err(err) = check_for_update(&api)
                .await
                .context("Failed to check for update")
            {
                warn!("{}", err.report());
            }
        }
    });

    api.on_update_and_restart({
        move || {
            let update_path = dirs::data_local_dir()
                .unwrap()
                .join("Mukwa")
                .join("updates")
                .join("Mukwa-Update.exe");
            if let Err(err) = install_update(update_path).context("Failed to update app") {
                warn!("{}", err.report())
            }
        }
    });

    api.on_set_maximized({
        let window = main_window.clone_strong();
        move |maximized| {
            let window = window.window();
            window.set_maximized(maximized);
            window.is_maximized()
        }
    });

    api.on_minimize({
        let window = main_window.clone_strong();
        move || {
            let window = window.window();
            window.set_minimized(true);
        }
    });

    api.on_format_money_without_symbol({
        move |value| {
            // Empty strings represent null values
            if value.is_empty() {
                return value;
            }

            let formatter = CurrencyFormatter::new();
            match Money::from_str(&value) {
                Ok(value) => formatter
                    .format_money_without_symbol(value)
                    .to_shared_string(),
                Err(err) => {
                    warn!("Error parsing Money: {err}");
                    formatter
                        .format_money_without_symbol(Money::ZERO)
                        .to_shared_string()
                }
            }
        }
    });

    api.on_window_size({
        let window = main_window.as_weak();
        move || {
            if let Some(window) = window.upgrade() {
                let window = window.window();
                let size = window.size().to_logical(window.scale_factor());
                return (size.height, size.width);
            }
            warn!("Empty window");
            (0.0, 0.0)
        }
    });

    api.on_window_position({
        let window = main_window.as_weak();
        move || {
            if let Some(window) = window.upgrade() {
                let window = window.window();
                let pos = window.position().to_logical(window.scale_factor());
                return (pos.x, pos.y);
            }
            warn!("Empty window");
            (0.0, 0.0)
        }
    });

    api.on_parse_date(|date| {
        let date = Date::strptime("%Y-%m-%d", &date)
            .inspect_err(|err| warn!("{err}"))
            .unwrap_or(Zoned::now().date());

        ui::Date {
            year: date.year() as i32,
            month: date.month() as i32,
            day: date.day() as i32,
        }
    });

    api.on_today(|| Zoned::now().date().to_shared_string());
    api.on_money_to_float(|money| {
        Money::from_str(&money)
            .inspect_err(|err| warn!("{err}"))
            .unwrap_or_default()
            .inner() as f32
    });
}

async fn check_for_update(api: &Api<'_>) -> crate::Result<()> {
    // FIXME: app hanging while writing update file
    // TODO: add arm64 support
    let mut artifacts = HashMap::new();
    artifacts.insert(
        Platform::WindowsX64,
        Artifact {
            download_url: String::from("https://github.com/snubwoody/mukwa/releases/latest/download/Mukwa-x86_64-Setup.exe"),
            digest: String::new(),
        },
    );
    let releases = vec![Release {
        version: Version::new(0, 2, 4),
        artifacts,
        ..Default::default()
    }];
    let manifest = Manifest { releases };

    let update_dir = dirs::data_local_dir()
        .unwrap()
        .join("Mukwa")
        .join("updates");
    // TODO: only check in release mode once done
    smol::fs::create_dir_all(&update_dir).await?;
    api.set_update_state(AutoUpdaterState::Checking);
    info!("Checking for updates");
    match auto_update::check_for_update(Version::parse(env!("CARGO_PKG_VERSION"))?, manifest) {
        Some(release) => {
            info!("New update found, version {}", release.version);
            api.set_update_state(AutoUpdaterState::Downloading);
            info!("Downloading new update...");
            let update_path = download_update(release, &update_dir).await?;
            debug!(path=?update_path,"Successfully downloaded new update");
            api.set_update_state(AutoUpdaterState::Ready);
        }
        None => {
            api.set_update_state(AutoUpdaterState::Idle);
            info!("No new update found");
        }
    }

    Ok(())
}
