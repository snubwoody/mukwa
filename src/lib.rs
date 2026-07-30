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
use slint::{DataTransfer, Global, ModelExt};
use std::cell::{Ref, RefCell, RefMut};

use crate::fmt::CurrencyFormatter;
use crate::migrator::Migrator;
use crate::service::Service;
use crate::state::AppState;
use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};
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
    let category_groups_model_rc = ModelRc::new(state.category_groups());
    let budgets_model_rc = ModelRc::new(state.budgets());
    let account_options_rc = ModelRc::new(state.account_options());

    let global_state = window.global::<ui::State>();

    global_state.set_transactions(transactions_model_rc);
    global_state.set_accounts(accounts_model_rc);
    global_state.set_categories(categories_model_rc);
    global_state.set_category_groups(category_groups_model_rc);
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
        move || {
            if let Err(err) = state.create_transaction() {
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

    global_state.on_get_account_name({
        let state = state.clone();
        move |id| match state.get_account(id) {
            Some(account) => account.name,
            None => SharedString::new(),
        }
    });

    let settings = window.global::<ui::Settings>().as_weak();
    global_state.on_format_money({
        let settings = settings.unwrap();
        move |value| {
            static CURRENCY_FORMATTER: OnceLock<CurrencyFormatter> = OnceLock::new();

            // Empty strings represent null values
            if value.is_empty() {
                return value;
            }
            match Money::from_str(&value) {
                Ok(value) => {
                    let formatter = CURRENCY_FORMATTER.get_or_init(|| {
                        let mut formatter = CurrencyFormatter::new().unwrap();
                        formatter.set_symbol(&settings.get_currency_code());
                        formatter
                    });
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
            (size.height as i32, size.width as i32)
        }
    });

    api.on_window_position({
        let window = window.as_weak();
        move || {
            let pos = window.unwrap().window().position();
            (pos.x, pos.y)
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
}

fn setup_settings(window: &ui::MainWindow) -> Result<()> {
    let settings_state = window.global::<ui::Settings>();
    let settings_dir = if cfg!(debug_assertions) {
        PathBuf::from(".mukwa")
    } else {
        config_dir()
    };

    let settings = SettingsStore::load(settings_dir.join("settings.toml"))?;
    settings_state.set_currency_code(settings.currency_code().to_shared_string());

    settings_state.on_set_currency_code({
        let store = settings.clone();
        move |code| {
            if let Err(err) = store.set_currency_code(&code) {
                warn!("Failed to set currency code: {err}");
            }
        }
    });
    Ok(())
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

    connection.pragma_update(None, "journal_mode", "WAL")?;
    let service = Service::new(connection);

    let main_window = ui::MainWindow::new().unwrap();

    let state = AppState::new(service)?;

    setup_global_state(state, &main_window);
    setup_calendar_state(&main_window);
    setup_api(&main_window);
    setup_settings(&main_window)?;

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

    fn currency_code(&self) -> String {
        self.settings().currency_code.clone()
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

    pub fn load(path: impl AsRef<Path>) -> Result<SettingsStore> {
        info!("Loading settings from {:?}", path.as_ref());
        let data = fs::read_to_string(&path)?;
        let settings: Settings = toml::from_str(&data)?;
        let store = SettingsStore {
            path: path.as_ref().to_path_buf(),
            inner: Rc::new(RefCell::new(settings)),
        };
        Ok(store)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    // TODO: add under display or format group
    currency_code: String,
}

impl Settings {
    pub fn load(path: impl AsRef<Path>) -> Result<Settings> {
        let data = fs::read_to_string(path)?;
        let settings: Settings = toml::from_str(&data)?;
        Ok(settings)
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            currency_code: String::from("USD"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Settings;
    use std::fs;

    #[test]
    fn settings() {
        let settings = Settings::default();
        let contents = toml::to_string(&settings).unwrap();
        fs::write(".mukwa/settings.toml", contents).unwrap();
    }
}
