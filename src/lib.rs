pub mod error;
mod service;

pub use error::Error;
pub use error::Result;

use std::rc::Rc;

use crate::service::Service;
use slint::{ComponentHandle, ModelRc, VecModel};
use tracing::info;

mod ui {
    slint::include_modules!();
}

pub fn run() -> Result<()> {
    let mut service = Service::open("app.data");
    service.read()?;
    let main_window = ui::MainWindow::new().unwrap();

    let account_list: Vec<ui::Account> = service.accounts().iter().map(|a| a.into()).collect();

    let accounts_model = Rc::new(VecModel::from(account_list));
    let accounts_model_rc = ModelRc::new(accounts_model.clone());

    main_window.set_accounts(accounts_model_rc);
    main_window
        .global::<ui::State>()
        .on_create_account(move |name| {
            let account = service
                .create_account(name.clone().to_string().as_ref())
                .unwrap();
            accounts_model.push(account.into());
            info!("Created new account");
        });
    main_window.run().unwrap();
    Ok(())
}
