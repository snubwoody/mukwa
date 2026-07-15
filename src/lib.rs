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
pub mod migrator;
mod money;
pub mod service;
pub mod state;

pub use error::Error;
pub use error::Result;
pub use money::Money;

use crate::migrator::Migrator;
use crate::service::Service;
use crate::state::AppState;
use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use rusqlite::Connection;
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::warn;

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

    global_state.on_get_category_name(|_| {
        // Will support categories in the future
        SharedString::new()
    });

    global_state.on_update_transaction({
        let mut state = state.clone();
        move |id, account_id, category_id, outflow, inflow, date| {
            if let Err(err) =
                state.update_transaction(&id, &account_id, &category_id, &outflow, &inflow, &date)
            {
                warn!("Failed to update transaction: {err}");
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
        let mut state = state.clone();
        move |id| match state.account_balance(&id) {
            Ok(balance) => balance.to_shared_string(),
            Err(err) => {
                warn!("Failed to calculate account balance: {err}");
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
}

pub fn run() -> Result<()> {
    let connection = Connection::open("data.sqlite")?;
    let mut migrator = Migrator::new();
    migrator.load_from_dir("./migrations")?;
    migrator.migrate(&connection)?;

    let service = Service::new(connection);

    let main_window = ui::MainWindow::new().unwrap();

    let state = AppState::new(service)?;

    setup_global_state(state, &main_window);
    setup_calendar_state(&main_window);

    main_window.run().unwrap();
    Ok(())
}

/// Opens an in memory sqlite database for testing.
pub fn create_test_db() -> Connection {
    let connection = Connection::open_in_memory().expect("Failed to open sqlite connection");
    let mut migrator = Migrator::new();
    migrator.load_from_dir("./migrations").unwrap();
    migrator.migrate(&connection).unwrap();
    connection
}
