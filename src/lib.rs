pub mod error;
mod money;
mod service;

pub use error::Error;
pub use error::Result;
pub use money::Money;
use std::cell::RefCell;

use crate::service::{Service, UpdateTransactionOpts};
use jiff::civil::{Date, Weekday};
use jiff::{ToSpan, Zoned};
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{info, warn};
use uuid::Uuid;

mod ui {
    slint::include_modules!();
}

#[derive(Clone, Default)]
pub struct AppState {
    service: Rc<RefCell<Service>>,
    accounts: Rc<VecModel<ui::Account>>,
    // We can't map arrays in slint so we have to maintain duplicate arrays for comboboxes
    // see <https://github.com/slint-ui/slint/issues/1328>
    account_options: Rc<VecModel<(SharedString, SharedString)>>,
    transactions: Rc<VecModel<ui::Transaction>>,
}

impl AppState {
    pub fn new(service: Service) -> AppState {
        let transactions_list: Vec<ui::Transaction> =
            service.transactions().iter().map(|t| t.into()).collect();

        let transactions_model = Rc::new(VecModel::from(transactions_list));
        let account_list: Vec<ui::Account> = service.accounts().iter().map(|a| a.into()).collect();
        let account_options: Vec<_> = account_list
            .iter()
            .map(|a| (a.name.clone(), a.id.clone()))
            .collect();
        let accounts_model = Rc::new(VecModel::from(account_list));
        let account_options_model = Rc::new(VecModel::from(account_options));

        AppState {
            service: Rc::new(RefCell::new(service)),
            accounts: accounts_model,
            account_options: account_options_model,
            transactions: transactions_model,
        }
    }
    /// Creates a new account.
    pub fn create_account(&mut self, name: &str) -> Result<()> {
        let account = self.service.borrow_mut().create_account(name)?;
        info!(id=?account.id,"Created new account");
        self.accounts.push(account.clone().into());
        self.account_options.push((
            SharedString::from(account.id.to_string()),
            account.name.into(),
        ));
        Ok(())
    }

    pub fn create_transaction(&mut self) -> Result<()> {
        let transaction = self.service.borrow_mut().create_transaction()?;
        info!(id=?transaction.id,"Created new transaction");
        self.transactions.push(transaction.into());
        Ok(())
    }

    pub fn update_transaction(&mut self, id: &str, account_id: &str, amount: &str) -> Result<()> {
        let account_id = Uuid::parse_str(account_id).ok();
        let amount = Money::from_str(amount).ok();
        let opts = UpdateTransactionOpts {
            id: Uuid::parse_str(id)?,
            account_id,
            amount,
        };
        let transaction = self.service.borrow_mut().update_transaction(opts)?;
        let transactions: Vec<ui::Transaction> = self
            .transactions
            .iter()
            .map(move |mut t| {
                if t.id == transaction.id.to_shared_string() {
                    t.account_id = transaction.account_id.to_shared_string()
                }
                t
            })
            .collect();
        self.transactions.set_vec(transactions);
        info!(id=?id,"Updated transaction");
        Ok(())
    }

    pub fn get_account(&self, id: SharedString) -> Option<ui::Account> {
        self.accounts.iter().find(|a| a.id == id)
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
                let mut days: Vec<i32> = vec![];
                // Pad with 0 for the out of month days to align the calendar grid
                let offset = date.first_of_month().weekday().to_sunday_zero_offset();
                for _ in 0..offset {
                    days.push(0);
                }

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

pub fn run() -> Result<()> {
    let mut service = Service::open("app.data");
    service.read()?;
    let main_window = ui::MainWindow::new().unwrap();

    let state = AppState::new(service);
    let transactions_model_rc = ModelRc::new(state.transactions.clone());
    let accounts_model_rc = ModelRc::new(state.accounts.clone());
    let account_options_rc = ModelRc::new(state.account_options.clone());

    let global_state = main_window.global::<ui::State>();
    global_state.set_transactions(transactions_model_rc.clone());
    global_state.set_accounts(accounts_model_rc.clone());

    global_state.set_account_options(account_options_rc);

    global_state.on_create_account({
        let mut state = state.clone();
        move |name| state.create_account(name.as_str()).unwrap()
    });

    global_state.on_create_transaction({
        let mut state = state.clone();
        move || state.create_transaction().unwrap()
    });

    global_state.on_get_category_name(|_| {
        // Will support categories in the future
        SharedString::new()
    });

    global_state.on_update_transaction({
        let mut state = state.clone();
        move |id, account_id, amount| {
            if let Err(err) = state.update_transaction(&id, &account_id, &amount) {
                warn!("Failed to update transaction: {err}");
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

    setup_calendar_state(&main_window);

    main_window.run().unwrap();
    Ok(())
}
