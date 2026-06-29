use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, VecModel};

mod ui {
    slint::include_modules!();
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub balance: f32,
}

fn main() -> Result<(), slint::PlatformError> {
    let main_window = ui::MainWindow::new()?;

    let accounts = [
        Account {
            id: String::from("A1"),
            name: String::from("Absa savings"),
            balance: 200.0,
        },
        Account {
            id: String::from("A2"),
            name: String::from("FNB savings"),
            balance: 200.0,
        },
    ];
    let account_list = accounts.map(|a| ui::Account {
        id: a.id.into(),
        name: a.name.into(),
        balance: a.balance.to_string().into(),
    }).to_vec();
    let model = VecModel::from(account_list);
    let accounts_model = ModelRc::new(Rc::new(model));

    main_window.set_accounts(accounts_model);

    main_window.run()
}
