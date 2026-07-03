pub mod error;
mod money;
mod service;

pub use error::Error;
pub use error::Result;
pub use money::Money;
use std::cell::RefCell;

use crate::service::{Service, UpdateTransactionOpts};
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::rc::Rc;
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
        let opts = UpdateTransactionOpts {
            id: Uuid::parse_str(id)?,
            account_id,
        };
        let transaction = self.service.borrow_mut().update_transaction(opts)?;
        let transactions: Vec<ui::Transaction> = self
            .transactions
            .iter()
            .map(move |mut t| {
                if t.id == transaction.id.to_shared_string() {
                    // return transaction.into();
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

// TODO: use skia renderer
pub fn run() -> Result<()> {
    let mut service = Service::open("app.data");
    service.read()?;
    let main_window = ui::MainWindow::new().unwrap();

    let state = AppState::new(service);
    let transactions_model_rc = ModelRc::new(state.transactions.clone());
    let accounts_model_rc = ModelRc::new(state.accounts.clone());
    let account_options_rc = ModelRc::new(state.account_options.clone());

    main_window
        .global::<ui::State>()
        .set_transactions(transactions_model_rc.clone());
    main_window
        .global::<ui::State>()
        .set_accounts(accounts_model_rc.clone());

    main_window
        .global::<ui::State>()
        .set_account_options(account_options_rc);

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

    main_window.global::<ui::State>().on_update_transaction({
        let mut state = state.clone();
        move |id, account_id, amount| {
            if let Err(err) = state.update_transaction(&id, &account_id, &amount) {
                warn!("Failed to update transaction: {err}");
            }
        }
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
