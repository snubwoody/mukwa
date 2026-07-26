// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub mod error;
pub mod fmt;
pub mod migrator;
mod money;
pub mod service;
pub mod state;

pub use error::Error;
pub use error::Result;
pub use money::Money;

use crate::fmt::CurrencyFormatter;
use crate::migrator::Migrator;
use crate::service::Service;
use crate::state::AppState;
use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use rusqlite::Connection;
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::OnceLock;
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

fn setup_calendar_state(window: &ui::MainWindow) {
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

fn setup_global_state(state: AppState, window: &ui::MainWindow) {
    let transactions_model_rc = ModelRc::new(state.transactions());
    let accounts_model_rc = ModelRc::new(state.accounts());
    let categories_model_rc = ModelRc::new(state.categories());
    let budgets_model_rc = ModelRc::new(state.budgets());
    let account_options_rc = ModelRc::new(state.account_options());

    let global_state = window.global::<ui::State>();

    global_state.set_transactions(transactions_model_rc);
    global_state.set_accounts(accounts_model_rc);
    global_state.set_categories(categories_model_rc);
    global_state.set_budgets(budgets_model_rc);

    global_state.set_account_options(account_options_rc);
    global_state.set_category_options(ModelRc::new(state.category_options()));

    global_state.on_create_account({
        let mut state = state.clone();
        move |name| state.create_account(name.as_str()).unwrap()
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

    global_state.on_create_transaction({
        let mut state = state.clone();
        move || {
            if let Err(err) = state.create_transaction() {
                warn!("Failed to create transaction: {err}")
            }
        }
    });

    global_state.on_create_category({
        let mut state = state.clone();
        move |title| {
            if let Err(err) = state.create_category(&title) {
                warn!("Failed to create category: {err}")
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

    global_state.on_update_transaction({
        let mut state = state.clone();
        move |id, account_id, category_id, outflow, inflow, date, note| {
            if let Err(err) = state.update_transaction(
                &id,
                &account_id,
                &category_id,
                &outflow,
                &inflow,
                &date,
                &note,
            ) {
                warn!("Failed to update transaction: {err}");
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

    global_state.on_set_transaction_account({
        let mut state = state.clone();
        move |id, account_id| {
            if let Err(err) = state.set_transaction_account(&id, &account_id) {
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

    global_state.on_duplicate_transaction({
        let mut state = state.clone();
        move |id| {
            if let Err(err) = state.duplicate_transaction(&id) {
                warn!("Failed to duplicate transaction: {err}");
            }
        }
    });

    global_state.on_get_account_name({
        let state = state.clone();
        move |id| match state.get_account(id) {
            Some(account) => account.name,
            None => SharedString::new(),
        }
    });

    global_state.on_format_money({
        move |value| {
            static CURRENCY_FORMATTER: OnceLock<CurrencyFormatter> = OnceLock::new();

            // Empty strings represent null values
            if value.is_empty() {
                return value;
            }
            match Money::from_str(&value) {
                Ok(value) => {
                    let formatter =
                        CURRENCY_FORMATTER.get_or_init(|| CurrencyFormatter::new().unwrap());
                    let result = formatter.format_currency(value);
                    match result {
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

fn setup_api(window: &ui::MainWindow) {
    let api = window.global::<ui::Api>();

    api.on_window_size({
        let window = window.as_weak();
        move || {
            let size = window.unwrap().window().size();
            (size.width as i32, size.height as i32)
        }
    });

    api.on_window_position({
        let window = window.as_weak();
        move || {
            let pos = window.unwrap().window().position();
            (pos.x, pos.y)
        }
    });
}

pub fn run() -> Result<()> {
    let data_dir = if cfg!(debug_assertions) {
        PathBuf::from(".mukwa")
    } else {
        data_dir()
    };

    fs::create_dir_all(&data_dir)?;

    let database_path = data_dir.join("data.sqlite");
    info!("Opening sqlite database at {:?}", database_path);
    let mut connection = Connection::open(&database_path)?;
    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate(&mut connection)?;

    let service = Service::new(connection);

    let main_window = ui::MainWindow::new().unwrap();

    let state = AppState::new(service)?;

    setup_global_state(state, &main_window);
    setup_calendar_state(&main_window);
    setup_api(&main_window);

    main_window.run().unwrap();
    Ok(())
}

/// Opens an in memory sqlite database for testing.
pub fn create_test_db() -> Connection {
    let mut connection = Connection::open_in_memory().expect("Failed to open sqlite connection");
    let mut migrator = Migrator::new();
    migrator.load_from_dir("./migrations").unwrap();
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
