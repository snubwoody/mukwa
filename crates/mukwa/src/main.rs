// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod controllers;
mod settings;
mod state;
mod ui;

pub use mukwa_core::error::{Error, Result};
use settings::SettingsStore;
use tempfile::tempdir;

use crate::state::AppState;
use crate::ui::MainWindow;
use mukwa_core::migrator::Migrator;
use mukwa_core::service::Service;
use rusqlite::Connection;
use slint::ComponentHandle;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    #[cfg(debug_assertions)]
    let log_dir = PathBuf::from(".mukwa/logs");
    #[cfg(not(debug_assertions))]
    let log_dir = mukwa_core::log_dir();

    fs::create_dir_all(&log_dir).expect("Failed to create directory");

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mukwa")
        .max_log_files(7)
        .filename_suffix("log")
        .build(log_dir)
        .expect("Failed to setup logging");

    // Keep guard in scope
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let std_io_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_file(false)
        .with_line_number(false)
        .with_writer(file_writer)
        .with_ansi(false);

    let level = if cfg!(debug_assertions) {
        "info,i_slint_core=debug,mukwa=trace"
    } else {
        "info,mukwa=debug"
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(level))
        .with(std_io_layer)
        .with(file_layer)
        .try_init()
        .expect("Failed to setup logging");

    info!("Launching application");

    if let Err(err) = run() {
        error!("{}", err.report());
    }

    info!("Closing application");
}

pub struct App {
    #[allow(unused)]
    state: AppState,
    main_window: MainWindow,
    #[allow(unused)]
    settings: SettingsStore,
}

impl App {
    pub fn new() -> Result<Self> {
        let data_dir = if cfg!(debug_assertions) {
            PathBuf::from(".mukwa")
        } else {
            mukwa_core::data_dir()
        };

        fs::create_dir_all(&data_dir)?;

        let path = data_dir.join("data.sqlite");
        info!("Opening sqlite database at {:?}", &path);
        let mut connection = Connection::open(&path)?;
        let mut migrator = Migrator::new();
        migrator.load_embedded()?;
        migrator.migrate(&mut connection)?;

        connection.pragma_update(None, "journal_mode", "WAL")?;
        let service = Service::new(connection);
        let main_window = MainWindow::new()?;

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

        service.check_credit_account_categories()?;

        let settings_dir = if cfg!(debug_assertions) {
            PathBuf::from(".mukwa")
        } else {
            mukwa_core::config_dir()
        };

        fs::create_dir_all(&settings_dir)?;
        let settings = SettingsStore::open(settings_dir.join("settings.toml"))?;
        let state = AppState::new(service)?;

        controllers::bind_all(&main_window, &state, &settings);

        #[cfg(target_os = "linux")]
        slint::set_xdg_app_id("com.wakunguma.Mukwa")?;

        let app = App {
            state,
            main_window,
            settings,
        };

        Ok(app)
    }

    /// Creates a new `App` for testing.
    pub fn new_test() -> Result<Self> {
        let temp = tempdir()?;
        let service = Service::open_in_memory()?;
        let main_window = MainWindow::new()?;

        let settings = SettingsStore::open(temp.path().join("settings.toml"))?;
        let state = AppState::new(service)?;

        #[cfg(target_os = "linux")]
        slint::set_xdg_app_id("com.wakunguma.Mukwa")?;

        controllers::bind_all(&main_window, &state, &settings);

        let app = App {
            state,
            main_window,
            settings,
        };

        Ok(app)
    }

    pub fn window(&self) -> &MainWindow {
        &self.main_window
    }

    pub fn run(&self) -> Result<()> {
        self.main_window.run()?;
        Ok(())
    }
}

pub fn run() -> Result<()> {
    let app = App::new()?;
    app.run()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::civil::date;
    use jiff::{ToSpan, Zoned};
    use mukwa_core::Money;
    use slint::{Model, ToSharedString};

    #[test]
    fn total_spent_all_only_includes_expenses() -> Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = App::new_test()?;
        app.state
            .service()
            .create_expense()
            .amount(Money::new(200))
            .submit()?;
        app.state
            .service()
            .create_expense()
            .amount(Money::new(500))
            .submit()?;
        app.state
            .service()
            .create_income()
            .amount(Money::new(10_000))
            .submit()?;
        app.state.load_transactions()?;
        let window = app.window();
        let global_state = window.global::<ui::State>();
        let total = global_state.invoke_total_spent_all(Zoned::now().date().into());
        assert_eq!(total, Money::new(700).to_shared_string());
        Ok(())
    }

    #[test]
    fn total_spent_all_filters_by_month() -> Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = App::new_test()?;
        app.state
            .service()
            .create_expense()
            .date(date(2020, 2, 1))
            .amount(Money::new(200))
            .submit()?;
        app.state
            .service()
            .create_expense()
            .date(date(2020, 1, 1))
            .amount(Money::new(500))
            .submit()?;
        app.state.load_transactions()?;

        let date = ui::Date {
            year: 2020,
            month: 1,
            day: 1,
        };
        let window = app.window();
        let global_state = window.global::<ui::State>();
        let total = global_state.invoke_total_spent_all(date);
        assert_eq!(total, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn draw_pie_chart_filters_by_date() -> Result<()> {
        i_slint_backend_testing::init_no_event_loop();

        let app = App::new_test()?;
        let service = app.state.service();
        let group = service.create_category_group("")?;
        let category = service.create_category("Groceries", group.id)?;

        service
            .create_expense()
            .category(category.id)
            .date(Zoned::now().date() + 1.month())
            .amount(Money::new(100))
            .submit()?;
        service
            .create_expense()
            .category(category.id)
            .date(Zoned::now().date())
            .amount(Money::new(500))
            .submit()?;
        service
            .create_expense()
            .category(category.id)
            .date(Zoned::now().date() - 1.month())
            .amount(Money::new(200))
            .submit()?;
        app.state.load_transactions()?;

        let window = app.window();
        let analytics = window.global::<ui::AnalyticsApi>();
        let date = Zoned::now().date() - 1.month();
        let slices = analytics.invoke_draw_pie_chart(500.0, 500.0, date.into());
        let slice = slices.iter().next().unwrap();

        assert_eq!(slices.iter().len(), 1);
        assert_eq!(slice.amount, Money::new(200).to_shared_string());
        Ok(())
    }

    #[test]
    fn draw_pie_chart_sorts_categories() -> Result<()> {
        i_slint_backend_testing::init_no_event_loop();

        let app = App::new_test()?;
        let service = app.state.service();
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let electricity = service.create_category("Electricity", group.id)?;
        let water = service.create_category("Water Bill", group.id)?;

        service
            .create_expense()
            .category(groceries.id)
            .amount(Money::new(500))
            .submit()?;
        service
            .create_expense()
            .category(water.id)
            .amount(Money::new(200))
            .submit()?;
        service
            .create_expense()
            .category(electricity.id)
            .amount(Money::new(70))
            .submit()?;
        app.state.load_transactions()?;

        let window = app.window();
        let analytics = window.global::<ui::AnalyticsApi>();
        let pie_slices = analytics.invoke_draw_pie_chart(500.0, 500.0, Zoned::now().date().into());
        let mut slices = pie_slices.iter();

        assert_eq!(slices.len(), 3);

        let groceries_slice = slices.next().unwrap();
        let water_slice = slices.next().unwrap();
        let electricity_slice = slices.next().unwrap();

        assert_eq!(groceries_slice.amount, Money::new(500).to_shared_string());
        assert_eq!(groceries_slice.label, "Groceries");
        assert_eq!(water_slice.amount, Money::new(200).to_shared_string());
        assert_eq!(water_slice.label, "Water Bill");
        assert_eq!(electricity_slice.amount, Money::new(70).to_shared_string());
        assert_eq!(electricity_slice.label, "Electricity");
        Ok(())
    }
}
