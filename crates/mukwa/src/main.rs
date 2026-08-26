// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod settings;
mod state;

pub use mukwa_core::error::{Error, Result};
use mukwa_core::{Currency, Money};
use settings::SettingsStore;
use slint::{DataTransfer, Global, ModelExt};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tempfile::tempdir;

use crate::state::AppState;
use crate::ui::MainWindow;
use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use mukwa_core::fmt;
use mukwa_core::fmt::CurrencyFormatter;
use mukwa_core::migrator::Migrator;
use mukwa_core::plot::PieChart;
use mukwa_core::service::TransactionType;
use mukwa_core::service::{
    Account, AccountType, Budget, Category, CategoryGroup, Service, Transaction,
};
use rusqlite::Connection;
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use tiny_skia::Pixmap;

use tracing::{error, info, warn};
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

/// Slint auto generated code.
pub mod ui {
    slint::include_modules!();
}

impl From<Account> for ui::Account {
    fn from(account: Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.into(),
            account_type: account.account_type.into(),
            balance: Money::ZERO.to_shared_string(),
        }
    }
}

impl From<Account> for ui::ComboBoxItem {
    fn from(account: Account) -> Self {
        Self {
            value: account.id.to_string().into(),
            text: account.name.to_string().into(),
        }
    }
}

impl From<&Account> for ui::ComboBoxItem {
    fn from(account: &Account) -> Self {
        Self {
            value: account.id.to_string().into(),
            text: account.name.to_string().into(),
        }
    }
}

impl From<&Account> for ui::Account {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.clone().into(),
            account_type: account.account_type.into(),
            balance: Money::ZERO.to_shared_string(),
        }
    }
}

impl From<&AccountType> for ui::AccountType {
    fn from(value: &AccountType) -> Self {
        match value {
            AccountType::Cash => Self::Cash,
            AccountType::Credit => Self::Credit,
        }
    }
}

impl From<AccountType> for ui::AccountType {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Cash => Self::Cash,
            AccountType::Credit => Self::Credit,
        }
    }
}

impl From<Budget> for ui::Budget {
    fn from(value: Budget) -> Self {
        Self {
            id: value.id.to_shared_string(),
            amount: value.amount.to_shared_string(),
            year: value.year as i32,
            month: value.month as i32,
            category_id: value.category_id.to_shared_string(),
        }
    }
}

impl From<&Budget> for ui::Budget {
    fn from(value: &Budget) -> Self {
        Self {
            id: value.id.to_shared_string(),
            amount: value.amount.to_shared_string(),
            year: value.year as i32,
            month: value.month as i32,
            category_id: value.category_id.to_shared_string(),
        }
    }
}

impl From<Category> for ui::Category {
    fn from(value: Category) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            group_id: value.group_id.to_shared_string(),
        }
    }
}

impl From<CategoryGroup> for ui::CategoryGroup {
    fn from(value: CategoryGroup) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
        }
    }
}

impl From<&CategoryGroup> for ui::CategoryGroup {
    fn from(value: &CategoryGroup) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
        }
    }
}

impl From<&Category> for ui::Category {
    fn from(value: &Category) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            group_id: value.group_id.to_shared_string(),
        }
    }
}

impl From<Category> for ui::ComboBoxItem {
    fn from(value: Category) -> Self {
        Self {
            value: value.id.to_string().into(),
            text: value.title.to_string().into(),
        }
    }
}

impl From<&Category> for ui::ComboBoxItem {
    fn from(value: &Category) -> Self {
        Self {
            value: value.id.to_string().into(),
            text: value.title.to_string().into(),
        }
    }
}

impl From<TransactionType> for ui::TransactionType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Expense => ui::TransactionType::Expense,
            TransactionType::Income => ui::TransactionType::Income,
            TransactionType::Transfer => ui::TransactionType::Transfer,
        }
    }
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

impl From<Transaction> for ui::Transaction {
    fn from(value: Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        let transaction_type = value.transaction_type();
        let inflow = if transaction_type == TransactionType::Income {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let outflow = if transaction_type == TransactionType::Transfer
            || transaction_type == TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let note = value.note.unwrap_or_default().to_shared_string();

        let account_id = match transaction_type {
            TransactionType::Income => value.receiver_id.unwrap().to_shared_string(),
            _ => value.sender_id.unwrap().to_shared_string(),
        };

        let payee_id = match transaction_type {
            TransactionType::Transfer => value.receiver_id.unwrap().to_shared_string(),
            _ => SharedString::new(),
        };

        Self {
            id: value.id.to_shared_string(),
            account_id,
            payee_id,
            category_id: category_id.to_shared_string(),
            date: value.date.to_shared_string(),
            outflow,
            note,
            inflow,
            transaction_type: transaction_type.into(),
        }
    }
}

impl From<&Transaction> for ui::Transaction {
    fn from(value: &Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        let transaction_type = value.transaction_type();
        let inflow = if transaction_type == TransactionType::Income {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let outflow = if transaction_type == TransactionType::Transfer
            || transaction_type == TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let note = value.note.clone().unwrap_or_default().to_shared_string();

        let account_id = match transaction_type {
            TransactionType::Income => value.receiver_id.unwrap().to_shared_string(),
            _ => value.sender_id.unwrap().to_shared_string(),
        };

        let payee_id = match transaction_type {
            TransactionType::Transfer => value.receiver_id.unwrap().to_shared_string(),
            _ => SharedString::new(),
        };

        Self {
            id: value.id.to_string().into(),
            account_id,
            payee_id,
            category_id: category_id.into(),
            note,
            date: value.date.to_string().into(),
            outflow,
            inflow,
            transaction_type: transaction_type.into(),
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
        let main_window = ui::MainWindow::new()?;

        let settings_dir = if cfg!(debug_assertions) {
            PathBuf::from(".mukwa")
        } else {
            mukwa_core::config_dir()
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
            // TODO: draw gray no data donut chart if empty
            let state = self.state.clone();

            move |width, height| {
                // TODO: collect categories below a threshold into 'Other'
                // TODO: order categories by total spent
                let transactions = state.service().fetch_transactions().unwrap_or_default();
                let mut map = HashMap::new();
                let mut labels = vec![];
                for transaction in transactions {
                    if let Some(category_id) = transaction.category_id {
                        match map.get(&category_id) {
                            Some(value) => {
                                map.insert(category_id, transaction.amount + *value);
                            }
                            None => {
                                let category = state
                                    .categories()
                                    .iter()
                                    .find(|c| c.id == category_id.to_shared_string())
                                    .map(|c| c.title.to_string())
                                    .unwrap_or_default();
                                labels.push(category);
                                map.insert(category_id, transaction.amount);
                            }
                        }
                    }
                }
                let colors = [
                    tiny_skia::Color::from_rgba8(0, 117, 222, 255),
                    tiny_skia::Color::from_rgba8(0, 94, 180, 255),
                    tiny_skia::Color::from_rgba8(0, 70, 138, 255),
                    tiny_skia::Color::from_rgba8(0, 48, 98, 255),
                    tiny_skia::Color::from_rgba8(61, 144, 255, 255),
                    tiny_skia::Color::from_rgba8(127, 171, 255, 255),
                    tiny_skia::Color::from_rgba8(236, 241, 255, 255),
                ];

                let series: Vec<f32> = map.values().map(|amount| amount.inner() as f32).collect();
                let mut pixmap =
                    Pixmap::new(width.max(1.0) as u32, height.max(1.0) as u32).unwrap();
                let radius = width.min(height) / 2.0;

                let chart = PieChart::new(width / 2.0, height / 2.0, series, radius)
                    .with_colors(colors.to_vec())
                    .with_label_line_length(50.0)
                    .with_labels(labels)
                    .with_hole_radius(radius - 150.0);

                chart.draw(&mut pixmap);
                let segments = chart.segments();

                let slices = VecModel::default();
                let values = map.values().copied().collect::<Vec<_>>();

                for (index, segment) in segments.iter().enumerate() {
                    // FIXME: already scaled money
                    let color = segment.color().to_color_u8();
                    let (label_x, label_y) = segment.label_position();
                    let amount = values[index];

                    let slice = ui::PieChartSlice {
                        arc_path: segment.arc_svg().to_shared_string(),
                        line_path: segment.label_line_svg().to_shared_string(),
                        fill: slint::Color::from_rgb_u8(color.red(), color.green(), color.blue()),
                        label: segment.label().to_shared_string(),
                        label_x,
                        amount: amount.to_shared_string(),
                        label_y,
                        ratio: segment.ratio(),
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

#[cfg(test)]
mod test {
    use super::*;

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
