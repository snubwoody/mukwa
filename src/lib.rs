// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

pub mod error;
pub mod fmt;
pub mod migrator;
mod money;
pub mod plot;
pub mod service;
pub mod state;

pub use error::Error;
pub use error::Result;
pub use money::{Currency, Money};
use slint::{DataTransfer, Global, ModelExt};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io;
use std::io::Read;
use std::time::Instant;
use tempfile::tempdir;

use crate::fmt::CurrencyFormatter;
use crate::migrator::Migrator;
use crate::plot::PieChart;
use crate::service::Service;
use crate::service::TransactionType;
use crate::state::AppState;
use crate::ui::MainWindow;
use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{info, warn};

/// Slint auto generated code.
pub mod ui {
    slint::include_modules!();
}

impl From<Date> for ui::Date {
    fn from(value: Date) -> Self {
        Self {
            year: value.year() as i32,
            month: value.month() as i32,
            day: value.day() as i32,
        }
    }
}

pub struct App {
    state: AppState,
    main_window: MainWindow,
    settings: SettingsStore,
}

impl App {
    pub fn new() -> Result<Self> {
        let data_dir = if cfg!(debug_assertions) {
            PathBuf::from(".mukwa")
        } else {
            data_dir()
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
        let main_window = ui::MainWindow::new()?;

        let settings_dir = if cfg!(debug_assertions) {
            PathBuf::from(".mukwa")
        } else {
            config_dir()
        };

        fs::create_dir_all(&settings_dir)?;
        let settings = SettingsStore::open(settings_dir.join("settings.toml"))?;
        let state = AppState::new(service)?;

        #[cfg(target_os = "linux")]
        slint::set_xdg_app_id("com.wakunguma.Mukwa")?;

        let app = App {
            state,
            main_window,
            settings,
        };

        app.init_settings();
        app.init_api();
        app.init_combobox_api();
        app.init_global_state();
        app.init_calendar_state();
        app.init_analytics();
        Ok(app)
    }

    /// Creates a new `App` for testing.
    pub fn new_test() -> Result<Self> {
        let temp = tempdir()?;
        let service = Service::open_in_memory()?;
        let main_window = ui::MainWindow::new()?;

        let settings = SettingsStore::open(temp.path().join("settings.toml"))?;
        let state = AppState::new(service)?;

        #[cfg(target_os = "linux")]
        slint::set_xdg_app_id("com.wakunguma.Mukwa")?;

        let app = App {
            state,
            main_window,
            settings,
        };

        app.init_settings();
        app.init_api();
        app.init_global_state();
        app.init_calendar_state();
        app.init_analytics();

        Ok(app)
    }

    pub fn window(&self) -> &MainWindow {
        &self.main_window
    }

    fn init_analytics(&self) {
        let window = &self.main_window;
        let analytics = window.global::<ui::AnalyticsApi>();

        analytics.on_draw_pie_chart({
            let state = self.state.clone();

            move |width, height| {
                // TODO: collect categories below a threshold into 'Other'
                // TODO: order categories consistently
                let transactions = state.service().fetch_transactions().unwrap_or_default();
                let mut map = HashMap::new();
                for transaction in transactions {
                    if let Some(category_id) = transaction.category_id {
                        match map.get(&category_id) {
                            Some(value) => {
                                map.insert(category_id, transaction.amount.inner() as f32 + value);
                            }
                            None => {
                                map.insert(category_id, transaction.amount.inner() as f32);
                            }
                        }
                    }
                }
                let colors = vec![
                    tiny_skia::Color::from_rgba8(0, 117, 222, 255),
                    tiny_skia::Color::from_rgba8(0, 94, 180, 255),
                    tiny_skia::Color::from_rgba8(0, 70, 138, 255),
                    tiny_skia::Color::from_rgba8(0, 48, 98, 255),
                    tiny_skia::Color::from_rgba8(61, 144, 255, 255),
                    tiny_skia::Color::from_rgba8(127, 171, 255, 255),
                    tiny_skia::Color::from_rgba8(236, 241, 255, 255),
                ];

                let series: Vec<f32> = map.values().copied().collect();
                let mut pixmap =
                    tiny_skia::Pixmap::new(width.max(1.0) as u32, height.max(1.0) as u32).unwrap();
                let radius = width.min(height) / 2.0;
                let mut chart = PieChart::new(width / 2.0, height / 2.0, series, radius);
                chart.set_colors(colors);
                chart.set_label_line_length(50.0);
                chart.set_hole_radius(radius - 150.0);
                chart.draw(&mut pixmap);
                let segments = chart.segments();

                let slices = VecModel::default();
                for segment in segments {
                    let color = segment.color().to_color_u8();
                    let slice = ui::PieChartSlice {
                        arc_path: segment.arc_svg().to_shared_string(),
                        line_path: segment.label_line_svg().to_shared_string(),
                        fill: slint::Color::from_rgb_u8(color.red(), color.green(), color.blue()),
                    };
                    slices.push(slice);
                }
                ModelRc::new(slices)
            }
        });
    }

    fn init_calendar_state(&self) {
        let window = &self.main_window;
        let calendar_state = window.global::<ui::CalendarState>();
        calendar_state.on_today(|| Zoned::now().date().into());

        calendar_state.on_increment_month(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            if let Ok(date) = result {
                let date = date.saturating_add(1.month());
                return date.into();
            }

            Zoned::now().date().into()
        });

        calendar_state.on_increment_week(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            if let Ok(date) = result {
                let date = date.saturating_add(1.week());
                return date.into();
            }

            Zoned::now().date().into()
        });

        calendar_state.on_increment_day(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            if let Ok(date) = result {
                let date = date.saturating_add(1.day());
                return date.into();
            }

            Zoned::now().date().into()
        });

        calendar_state.on_decrement_month(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            if let Ok(date) = result {
                let date = date.saturating_sub(1.month());
                return date.into();
            }

            Zoned::now().date().into()
        });

        calendar_state.on_decrement_week(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            if let Ok(date) = result {
                let date = date.saturating_sub(1.week());
                return date.into();
            }

            Zoned::now().date().into()
        });

        calendar_state.on_decrement_day(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            if let Ok(date) = result {
                let date = date.saturating_sub(1.day());
                return date.into();
            }

            Zoned::now().date().into()
        });

        calendar_state.on_days_in_month(|date| {
            let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
            match result {
                Ok(date) => {
                    // Pad with 0 for the out of month days to align the calendar grid
                    let offset = date.first_of_month().weekday().to_sunday_zero_offset();
                    let mut days: Vec<i32> = vec![0; offset as usize];

                    for d in 1..=date.days_in_month() {
                        days.push(d as i32);
                    }

                    let weeks: Vec<_> = days
                        .chunks(7)
                        .map(|chunk| ModelRc::new(Rc::new(VecModel::from(chunk.to_vec()))))
                        .collect();
                    ModelRc::new(Rc::new(VecModel::from(weeks)))
                }
                Err(_) => {
                    warn!("Invalid date: {:?}", date);
                    Default::default()
                }
            }
        });
    }

    fn init_settings(&self) {
        let window = &self.main_window;
        let settings_state = window.global::<ui::Settings>();

        let settings = &self.settings;
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

    fn init_combobox_api(&self) {
        let window = &self.main_window;
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

    fn init_api(&self) {
        let window = &self.main_window;
        let api = window.global::<ui::Api>();

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
            let window = window.as_weak();
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
            let window = window.as_weak();
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

    fn init_global_state(&self) {
        let instant = Instant::now();
        let mut database = fontdb::Database::new();
        database.load_system_fonts();

        let mut families = HashSet::new();
        for face in database.faces() {
            for (family, _) in &face.families {
                families.insert(family);
            }
        }

        let mut families: Vec<_> = families.iter().collect();
        families.sort();

        let fonts: Vec<_> = families
            .iter()
            .map(|family| ui::ComboBoxItem {
                value: family.to_shared_string(),
                text: family.to_shared_string(),
            })
            .collect();
        let elapsed = instant.elapsed().as_millis();
        tracing::trace!("Loaded system fonts in {elapsed}ms");

        let currencies: Vec<_> = Currency::ALL_CURRENCIES
            .iter()
            .map(|currency| ui::ComboBoxItem {
                value: currency.code().to_shared_string(),
                text: currency.name().to_shared_string(),
            })
            .collect();

        let window = &self.main_window;
        let state = &self.state;
        let currencies_model = Rc::new(VecModel::from(currencies));
        let currencies_model_rc = ModelRc::new(currencies_model);
        let fonts_model = Rc::new(VecModel::from(fonts));
        let fonts_model_rc = ModelRc::new(fonts_model);
        let transactions_model_rc = ModelRc::new(state.transactions());
        let accounts_model_rc = ModelRc::new(state.accounts());
        let categories_model_rc = ModelRc::new(state.categories());
        let category_groups_model_rc = ModelRc::new(state.category_groups());
        let budgets_model_rc = ModelRc::new(state.budgets());
        let account_options_rc = ModelRc::new(state.account_options());

        let global_state = window.global::<ui::State>();

        global_state.set_currency_options(currencies_model_rc);
        global_state.set_font_options(fonts_model_rc);
        global_state.set_transactions(transactions_model_rc);
        global_state.set_accounts(accounts_model_rc);
        global_state.set_categories(categories_model_rc);
        global_state.set_category_groups(category_groups_model_rc);
        global_state.set_budgets(budgets_model_rc);

        global_state.set_account_options(account_options_rc);
        global_state.set_category_options(ModelRc::new(state.category_options()));

        global_state.on_total_spent_all({
            let state = state.clone();
            move || match state.service().fetch_transactions() {
                Ok(transactions) => {
                    let total: Money = transactions
                        .iter()
                        .filter(|t| t.transaction_type() == TransactionType::Expense)
                        .map(|t| t.amount)
                        .sum();
                    total.to_shared_string()
                }
                Err(err) => {
                    warn!("Error while calculating total spent: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_create_account({
            let mut state = state.clone();
            move |name, account_type| {
                if let Err(err) = state.create_account(name.as_str(), account_type) {
                    warn!("Failed to create account: {err}")
                }
            }
        });

        global_state.on_get_category({
            let state = state.clone();
            move |id| {
                state
                    .categories()
                    .iter()
                    .find(|c| c.id == id)
                    .unwrap_or_default()
            }
        });

        global_state.on_categories_in_group({
            let state = state.clone();
            move |group_id| {
                let filtered_categories = state
                    .categories()
                    .filter(move |category| category.group_id == group_id);

                ModelRc::new(filtered_categories)
            }
        });

        global_state.on_get_budget({
            let state = state.clone();
            move |category_id, date| {
                state
                    .budgets()
                    .iter()
                    .find(|budget| {
                        budget.category_id == category_id
                            && budget.month == date.month
                            && budget.year == date.year
                    })
                    .unwrap_or_default()
            }
        });

        global_state.on_get_account({
            let state = state.clone();
            move |id| {
                state
                    .accounts()
                    .iter()
                    .find(|a| a.id == id)
                    .unwrap_or_default()
            }
        });

        global_state.on_create_transaction({
            let mut state = state.clone();
            move |opts| {
                if let Err(err) = state.create_transaction(opts) {
                    warn!("Failed to create transaction: {err}")
                }
            }
        });

        global_state.on_create_category({
            let mut state = state.clone();
            move |title, group_id| {
                if let Err(err) = state.create_category(&title, &group_id) {
                    warn!("Failed to create category: {err}")
                }
            }
        });

        global_state.on_delete_category({
            let mut state = state.clone();
            move |id| {
                if let Err(err) = state.delete_category(&id) {
                    warn!("Failed to delete category: {err}")
                }
            }
        });

        global_state.on_delete_category_group({
            let mut state = state.clone();
            move |id| {
                if let Err(err) = state.delete_category_group(&id) {
                    warn!("Failed to delete category group: {err}")
                }
            }
        });

        global_state.on_create_category_group({
            let mut state = state.clone();
            move |title| {
                if let Err(err) = state.create_category_group(&title) {
                    warn!("{err}")
                }
            }
        });

        global_state.on_set_current_budget_month({
            let mut state = state.clone();
            move |date| {
                let date = Date::new(date.year as i16, date.month as i8, date.day as i8)
                    .unwrap_or(Zoned::now().date());
                if let Err(err) = state.set_current_budget_month(date) {
                    warn!("{err}")
                }
            }
        });

        global_state.on_set_transaction_category({
            let mut state = state.clone();
            move |id, category_id| {
                if let Err(err) = state.set_transaction_category(&id, &category_id) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_set_transaction_date({
            let mut state = state.clone();
            move |id, date| {
                if let Err(err) = state.set_transaction_date(&id, &date) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_set_transaction_outflow({
            let mut state = state.clone();
            move |id, amount| {
                if let Err(err) = state.set_transaction_outflow(&id, &amount) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_set_transaction_inflow({
            let mut state = state.clone();
            move |id, amount| {
                if let Err(err) = state.set_transaction_inflow(&id, &amount) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_set_transaction_account({
            let mut state = state.clone();
            move |id, account_id| {
                if let Err(err) = state.set_transaction_account(&id, &account_id) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_set_transaction_payee({
            let mut state = state.clone();
            move |id, account_id| {
                if let Err(err) = state.set_transaction_payee(&id, &account_id) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_set_transaction_note({
            let mut state = state.clone();
            move |id, note| {
                if let Err(err) = state.set_transaction_note(&id, &note) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_total_balance({
            let state = state.clone();
            move || {
                let mut total = Money::ZERO;
                for transaction in state.transactions().iter() {
                    total -= Money::from_str(transaction.outflow.as_str()).unwrap_or_default();
                    total += Money::from_str(transaction.inflow.as_str()).unwrap_or_default();
                }
                total.to_shared_string()
            }
        });

        global_state.on_account_balance({
            let state = state.clone();
            move |id| match state.account_balance(&id) {
                Ok(balance) => balance.to_shared_string(),
                Err(err) => {
                    warn!("Failed to calculate account balance: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_total_spent({
            let state = state.clone();
            move |id| match state.total_spent(&id) {
                Ok(total) => total.to_shared_string(),
                Err(err) => {
                    warn!("Failed to calculate total spent: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_total_spent_in_group({
            let state = state.clone();
            move |id, date| match state.total_spent_in_group(&id, date) {
                Ok(total) => total.to_shared_string(),
                Err(err) => {
                    warn!("Failed to calculate total spent: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_delete_transaction({
            let mut state = state.clone();
            move |id| {
                if let Err(err) = state.delete_transaction(&id) {
                    warn!("Failed to delete transaction: {err}");
                }
            }
        });

        global_state.on_edit_budget({
            let mut state = state.clone();
            move |id, amount| {
                if let Err(err) = state.update_budget(&id, &amount) {
                    warn!("Failed to update budget: {err}");
                }
            }
        });

        global_state.on_format_dateym({
            |date| match Date::new(date.year as i16, date.month as i8, date.day as i8) {
                Ok(date) => date.strftime("%b %Y").to_shared_string(),
                Err(err) => {
                    warn!("Invalid date: {err}");
                    SharedString::new()
                }
            }
        });

        global_state.on_format_date({
            |date| match Date::strptime("%Y-%m-%d", date) {
                Ok(date) => match fmt::format_date(date) {
                    Ok(value) => value.to_shared_string(),
                    Err(err) => {
                        warn!("{err}");
                        SharedString::new()
                    }
                },
                Err(err) => {
                    warn!("Invalid date: {err}");
                    SharedString::new()
                }
            }
        });

        global_state.on_update_category({
            let mut state = state.clone();
            move |id, title| {
                if let Err(err) = state.update_category(&id, &title) {
                    warn!("Failed to update category: {err}");
                }
            }
        });

        global_state.on_update_category_group({
            let mut state = state.clone();
            move |id, title| {
                if let Err(err) = state.update_category_group(&id, &title) {
                    warn!("{err}");
                }
            }
        });

        global_state.on_left_to_spend({
            let state = state.clone();
            move |id| match state.left_to_spend(&id) {
                Ok(value) => value.to_shared_string(),
                Err(err) => {
                    warn!("{err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_category_to_transfer(DataTransfer::from);

        global_state.on_transfer_to_category(|data| {
            data.plain_text().unwrap_or_else(|err| {
                warn!("{err}");
                SharedString::new()
            })
        });

        global_state.on_move_category({
            let mut state = state.clone();
            move |id, group_id| {
                if let Err(err) = state.move_category(&id, &group_id) {
                    warn!("Failed to move category: {err}");
                }
            }
        });

        global_state.on_left_to_spend_in_group({
            let state = state.clone();
            move |id, date| match state.left_to_spend_in_group(&id, date) {
                Ok(value) => value.to_shared_string(),
                Err(err) => {
                    warn!("Failed to calculate the amount left to spend in group {id}: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_total_assigned_in_group({
            let state = state.clone();
            move |id, date| match state.total_assigned_in_group(&id, date) {
                Ok(value) => value.to_shared_string(),
                Err(err) => {
                    warn!(group_id=?id,"Failed to calculate total assigned in group: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_total_spent_in_group({
            let state = state.clone();
            move |id, date| match state.total_spent_in_group(&id, date) {
                Ok(value) => value.to_shared_string(),
                Err(err) => {
                    warn!(group_id=?id,"Failed to calculate total spent in group: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        });

        global_state.on_duplicate_transaction({
            let mut state = state.clone();
            move |id| {
                if let Err(err) = state.duplicate_transaction(&id) {
                    warn!("Failed to duplicate transaction: {err}");
                }
            }
        });

        global_state.on_format_money({
            move |value, currency_code| {
                // Empty strings represent null values
                if value.is_empty() {
                    return value;
                }

                match Money::from_str(&value) {
                    Ok(value) => {
                        let mut formatter = CurrencyFormatter::new();
                        let currency = Currency::from_str(&currency_code).unwrap();
                        formatter.set_currency(currency);
                        match formatter.format_money(value) {
                            Ok(result) => result.to_shared_string(),
                            Err(err) => {
                                warn!("Error occurred while formatting Money: {err}");
                                Money::ZERO.to_shared_string()
                            }
                        }
                    }
                    Err(err) => {
                        warn!("Error parsing Money: {err}");
                        Money::ZERO.to_shared_string()
                    }
                }
            }
        })
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

#[derive(Clone)]
pub struct SettingsStore {
    path: PathBuf,
    inner: Rc<RefCell<Settings>>,
}

impl SettingsStore {
    fn set_currency_code(&self, code: &str) -> Result<()> {
        self.settings_mut().currency_code = code.to_owned();
        self.write()?;
        info!("Updated currency code to {code}");
        Ok(())
    }

    fn set_font_family(&self, family: &str) -> Result<()> {
        self.settings_mut().appearance.font_family = family.to_owned();
        self.write()?;
        info!("Updated font family to {family}");
        Ok(())
    }

    fn currency_code(&self) -> String {
        self.settings().currency_code.clone()
    }

    fn font_family(&self) -> String {
        self.settings().appearance.font_family.clone()
    }

    fn settings(&self) -> Ref<'_, Settings> {
        self.inner.borrow()
    }

    fn settings_mut(&self) -> RefMut<'_, Settings> {
        self.inner.borrow_mut()
    }

    fn write(&self) -> Result<()> {
        let settings = self.settings();
        let contents = toml::to_string(&*settings)?;
        fs::write(&self.path, contents)?;
        Ok(())
    }

    pub fn open(path: impl AsRef<Path>) -> Result<SettingsStore> {
        match File::open(&path) {
            Ok(mut file) => {
                info!("Loading settings from {:?}", path.as_ref());

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                let settings: Settings = toml::from_slice(&buffer)?;
                let store = SettingsStore {
                    path: path.as_ref().to_path_buf(),
                    inner: Rc::new(RefCell::new(settings)),
                };
                Ok(store)
            }
            Err(err) => {
                if err.kind() != io::ErrorKind::NotFound {
                    return Err(err.into());
                }

                let settings = Settings::default();
                let contents = toml::to_string(&settings)?;
                fs::write(&path, contents)?;

                info!("Initialised settings at {:?}", path.as_ref());
                let store = SettingsStore {
                    path: path.as_ref().to_path_buf(),
                    inner: Rc::new(RefCell::new(settings)),
                };
                Ok(store)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    currency_code: String,
    #[serde(default)]
    appearance: Appearance,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Appearance {
    font_family: String,
}

impl Default for Appearance {
    fn default() -> Self {
        let default_font = if cfg!(target_os = "windows") {
            "Segoe UI"
        } else if cfg!(target_os = "macos") {
            "SF Pro"
        } else {
            // Slint will use the default font
            ""
        };

        Self {
            font_family: String::from(default_font),
        }
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            currency_code: String::from("USD"),
            appearance: Appearance::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::tempdir;

    use super::*;
    use crate::SettingsStore;
    use std::fs;

    #[test]
    fn init_settings_if_not_found() -> Result<()> {
        let temp = tempdir()?;
        let path = temp.path().join("settings.toml");
        SettingsStore::open(&path)?;
        assert!(fs::exists(path)?);
        Ok(())
    }

    #[test]
    fn total_spent_all_only_includes_expenses() -> Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let mut app = App::new_test()?;
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
        let total = global_state.invoke_total_spent_all();
        assert_eq!(total, Money::new(700).to_shared_string());
        Ok(())
    }
}
