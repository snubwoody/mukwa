pub mod error;
mod service;

pub use error::Error;
pub use error::Result;
use std::cell::RefCell;

use crate::service::Service;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use tracing::info;

mod ui {
    slint::include_modules!();
}

#[derive(Clone, Default)]
pub struct AppState {
    service: Rc<RefCell<Service>>,
    accounts: Rc<VecModel<ui::Account>>,
    transactions: Rc<VecModel<ui::Transaction>>,
}

impl AppState {
    /// Creates a new account.
    pub fn create_account(&mut self, name: &str) -> crate::Result<()> {
        let account = self.service.borrow_mut().create_account(name)?;
        info!(id=?account.id,"Created new account");
        self.accounts.push(account.into());
        Ok(())
    }

    pub fn create_transaction(&mut self) -> crate::Result<()> {
        let transaction = self.service.borrow_mut().create_transaction()?;
        info!(id=?transaction.id,"Created new transaction");
        self.transactions.push(transaction.into());
        Ok(())
    }

    pub fn get_account(&self, id: SharedString) -> Option<ui::Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}

// TODO: use skia renderer
pub fn run() -> Result<()> {
    let mut service = Service::open("app.data");
    service.read()?;
    let main_window = ui::MainWindow::new().unwrap();

    let transactions_list: Vec<ui::Transaction> =
        service.transactions().iter().map(|t| t.into()).collect();
    let transactions_model = Rc::new(VecModel::from(transactions_list));
    let transactions_model_rc = ModelRc::new(transactions_model.clone());
    let account_list: Vec<ui::Account> = service.accounts().iter().map(|a| a.into()).collect();
    let accounts_model = Rc::new(VecModel::from(account_list));
    let accounts_model_rc = ModelRc::new(accounts_model.clone());

    let state = AppState {
        service: Rc::new(RefCell::new(service)),
        accounts: accounts_model.clone(),
        transactions: transactions_model.clone(),
    };

    main_window
        .global::<ui::State>()
        .set_transactions(transactions_model_rc.clone());
    main_window
        .global::<ui::State>()
        .set_accounts(accounts_model_rc.clone());

    main_window.global::<ui::State>().on_create_account({
        let mut state = state.clone();
        move |name| state.create_account(name.as_str()).unwrap()
    });

    main_window.global::<ui::State>().on_create_transaction({
        let mut state = state.clone();
        move || state.create_transaction().unwrap()
    });

    main_window.global::<ui::State>().on_get_category_name(|_| {
        // Will support categories in the future
        SharedString::new()
    });

    main_window.global::<ui::State>().on_get_account_name({
        let state = state.clone();
        move |id| match state.get_account(id) {
            Some(account) => account.name,
            None => SharedString::new(),
        }
    });
    main_window.run().unwrap();
    Ok(())
}
