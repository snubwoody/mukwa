pub mod error;
mod service;

pub use error::Error;
pub use error::Result;

use crate::service::{Service, Transaction};
use jiff::Zoned;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;
use tracing::info;
use uuid::Uuid;

mod ui {
    slint::include_modules!();
}

// TODO: use skia renderer
pub fn run() -> Result<()> {
    let mut service = Service::open("app.data");
    service.read()?;
    let main_window = ui::MainWindow::new().unwrap();

    // TODO: impl From for slices and arrays []
    let transactions_list: Vec<ui::Transaction> =
        service.transactions().iter().map(|t| t.into()).collect();
    let transactions_model = Rc::new(VecModel::from(transactions_list));
    let transactions_model_rc = ModelRc::new(transactions_model.clone());
    let account_list: Vec<ui::Account> = service.accounts().iter().map(|a| a.into()).collect();

    let accounts_model = Rc::new(VecModel::from(account_list));
    let accounts_model_rc = ModelRc::new(accounts_model.clone());

    main_window
        .global::<ui::State>()
        .set_transactions(transactions_model_rc.clone());
    main_window
        .global::<ui::State>()
        .set_accounts(accounts_model_rc.clone());

    let mut new_service = service.clone();
    main_window
        .global::<ui::State>()
        .on_create_account(move |name| {
            let account = new_service
                .create_account(name.clone().to_string().as_ref())
                .unwrap();
            info!(id=?account.id,"Created new account");
            accounts_model.push(account.into());
        });

    main_window
        .global::<ui::State>()
        .on_create_transaction(move || {
            let transaction = Transaction {
                date: Zoned::now().date(),
                account_id: Uuid::now_v7(),
                id: Uuid::now_v7(),
                category_id: None,
            };
            info!(id=?transaction.id,"Created new transaction");
            transactions_model.push(transaction.into());
        });

    main_window
        .global::<ui::State>()
        .on_get_account_name(move |id| {
            // let id = Uuid::parse_str(id.as_str()).unwrap();
            if let Ok(id) = Uuid::parse_str(id.as_str())
                && let Some(account) = service.get_account(id)
            {
                return SharedString::from(&account.name);
            }
            SharedString::new()
        });
    main_window.run().unwrap();
    Ok(())
}
