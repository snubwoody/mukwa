// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::AccountState;
use crate::ui::{self, MainWindow};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::error::ErrorExt;
use mukwa_core::service::{AccountType, Service, TransactionKind, TransactionType};
use slint::{ComponentHandle, Global, Model, ModelRc, ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{info, warn};
use uuid::Uuid;

/// Binds the `AccountState` global callbacks and properties.
pub fn bind(app: &MainWindow, service: &Service) -> crate::Result<()> {
    AccountController::bind(app, service);
    Ok(())
}

#[derive(Clone)]
pub struct AccountController {
    state: Rc<AccountState<'static>>,
    service: Service,
}

impl AccountController {
    pub fn bind(app: &MainWindow, service: &Service) -> Self {
        let state: AccountState = app.global();
        let controller = AccountController {
            state: Rc::new(state.as_weak().unwrap()),
            service: service.clone(),
        };

        state.on_account_balance({
            let controller = controller.clone();
            move |id| {
                controller
                    .account_balance(&id)
                    .context("Failed to calculate account balance")
                    .inspect_err(|err| warn!("{}", err.report()))
                    .unwrap_or_default()
                    .to_shared_string()
            }
        });

        state.on_create_account({
            let controller = controller.clone();
            move |name, account_type, balance| {
                if let Err(err) = controller
                    .create_account(&name, account_type, &balance)
                    .context("Failed to create account")
                {
                    warn!("{}", err.report());
                }
            }
        });

        state.on_delete_account({
            let controller = controller.clone();
            move |id| {
                if let Err(err) = controller
                    .delete_account(&id)
                    .context("Failed to delete account")
                {
                    warn!("{}", err.report());
                }
            }
        });

        state.on_load_accounts({
            let controller = controller.clone();
            move || {
                if let Err(err) = controller
                    .load_accounts()
                    .context("Failed to load accounts")
                {
                    warn!("{}", err.report());
                }
            }
        });

        state.on_total_balance({
            let state = state.as_weak();
            move || {
                let state = state.unwrap();
                let mut total = Money::ZERO;
                for account in state.get_accounts().iter() {
                    total += Money::from_str(&account.balance).unwrap_or_default();
                }
                total.to_shared_string()
            }
        });

        state.on_get_account({
            let state = state.as_weak();
            move |id| {
                let state = state.unwrap();
                state
                    .get_accounts()
                    .iter()
                    .find(|a| a.id == id)
                    .unwrap_or_default()
            }
        });

        state.on_total_spent_all({
            let service = service.clone();
            move |month| match service.fetch_transactions() {
                Ok(transactions) => {
                    let date = Date::new(month.year as i16, month.month as i8, month.day as i8)
                        .unwrap_or(Zoned::now().date());
                    let total: Money = transactions
                        .iter()
                        .filter(|t| t.transaction_type() == TransactionType::Expense)
                        .filter(|t| t.date.year() == date.year() && t.date.month() == date.month())
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

        controller.state.invoke_load_accounts();
        controller
    }

    /// Creates a new account.
    fn create_account(
        &self,
        name: &str,
        account_type: ui::AccountType,
        balance: &str,
    ) -> crate::Result<()> {
        let account_type = match account_type {
            ui::AccountType::Cash => AccountType::Cash,
            ui::AccountType::Credit => AccountType::Credit,
        };
        let account = self.service.create_account(name, account_type)?;

        if let Ok(starting_balance) = Money::from_str(balance) {
            match account_type {
                AccountType::Cash => {
                    self.service
                        .create_income()
                        .account(account.id)
                        .amount(starting_balance)
                        .note("Starting balance")
                        .kind(TransactionKind::StartingBalance)
                        .submit()?;
                }
                AccountType::Credit => {
                    self.service
                        .create_expense()
                        .account(account.id)
                        .note("Starting balance")
                        .amount(starting_balance)
                        .kind(TransactionKind::StartingBalance)
                        .submit()?;
                }
            }
        }

        info!(id=?account.id,"Created new account");
        self.load_accounts()?;
        let mut account_options: Vec<_> = self.state.get_account_options().iter().collect();
        account_options.push(account.into());
        self.state
            .set_account_options(ModelRc::new(VecModel::from(account_options)));
        Ok(())
    }

    /// Deletes an account and all associated transactions.
    fn delete_account(&self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_account(id)?;
        info!("Deleted account {id}");
        self.load_accounts()?;
        // FIXME: delete account doesn't remove from list
        Ok(())
    }

    fn account_balance(&self, id: &str) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        self.service.account_balance(id)
    }

    fn load_accounts(&self) -> crate::Result<()> {
        let accounts = self.service.fetch_accounts()?;
        let mut account_list = vec![];
        for account in accounts {
            let balance = self.service.account_balance(account.id)?;
            account_list.push(ui::Account {
                id: account.id.to_shared_string(),
                name: account.name.to_shared_string(),
                account_type: account.account_type.into(),
                balance: balance.to_shared_string(),
            })
        }
        self.state
            .set_accounts(ModelRc::new(VecModel::from(account_list)));
        // TODO: load account options?
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use mukwa_core::service::{AccountType, Service};
    use mukwa_core::{Money, create_test_db};
    use slint::Model;

    fn setup() -> crate::Result<(MainWindow, AccountController)> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let service = Service::new(create_test_db());
        let controller = AccountController::bind(&app, &service);
        Ok((app, controller))
    }

    #[test]
    fn create_account_adds_to_account_options() -> crate::Result<()> {
        let (app, _) = setup()?;
        let state: AccountState = app.global();
        state.invoke_create_account(
            "Account 1".to_shared_string(),
            ui::AccountType::Cash,
            "".to_shared_string(),
        );
        state.invoke_create_account(
            "Account 2".to_shared_string(),
            ui::AccountType::Credit,
            "".to_shared_string(),
        );

        let accounts: Vec<_> = state.get_accounts().iter().collect();
        let account_options: Vec<_> = state.get_account_options().iter().collect();
        assert_eq!(account_options.len(), 2);
        assert_eq!(account_options[0].text, "Account 1");
        assert_eq!(account_options[1].text, "Account 2");
        assert_eq!(account_options[0].value, accounts[0].id);
        assert_eq!(account_options[1].value, accounts[1].id);
        Ok(())
    }

    #[test]
    fn create_account_adds_to_account_list() -> crate::Result<()> {
        let (app, _) = setup()?;
        let state: AccountState = app.global();
        state.invoke_create_account(
            "Account 1".to_shared_string(),
            ui::AccountType::Cash,
            "".to_shared_string(),
        );
        state.invoke_create_account(
            "Account 2".to_shared_string(),
            ui::AccountType::Credit,
            "".to_shared_string(),
        );

        let accounts: Vec<_> = state.get_accounts().iter().collect();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].name, "Account 1");
        assert_eq!(accounts[1].name, "Account 2");
        Ok(())
    }

    #[test]
    fn create_cash_account_with_starting_balance() -> crate::Result<()> {
        let (app, controller) = setup()?;
        let state: AccountState = app.global();
        state.invoke_create_account(
            "".to_shared_string(),
            AccountType::Cash.into(),
            "250.0".to_shared_string(),
        );

        let transactions = controller.service.fetch_transactions()?;
        assert_eq!(
            transactions[0].transaction_type(),
            mukwa_core::service::TransactionType::Income
        );
        assert_eq!(transactions[0].amount, Money::new(250));
        Ok(())
    }

    #[test]
    fn create_credit_account_with_starting_balance() -> crate::Result<()> {
        let (app, controller) = setup()?;
        let state: AccountState = app.global();
        state.invoke_create_account(
            "".to_shared_string(),
            AccountType::Credit.into(),
            "150.0".to_shared_string(),
        );

        let transactions = controller.service.fetch_transactions()?;
        assert_eq!(
            transactions[0].transaction_type(),
            mukwa_core::service::TransactionType::Expense
        );
        assert_eq!(transactions[0].amount, Money::new(150));
        Ok(())
    }

    #[test]
    fn load_data_on_init() -> crate::Result<()> {
        let (app, controller) = setup()?;
        let state: AccountState = app.global();

        controller.state.invoke_create_account(
            "".to_shared_string(),
            ui::AccountType::Cash,
            "".to_shared_string(),
        );
        controller.state.invoke_create_account(
            "".to_shared_string(),
            ui::AccountType::Cash,
            "".to_shared_string(),
        );

        assert_eq!(state.get_accounts().iter().len(), 2);
        Ok(())
    }

    #[test]
    fn starting_balance_transaction_kind() -> crate::Result<()> {
        let (app, controller) = setup()?;
        let account_state: AccountState = app.global();

        account_state.invoke_create_account(
            "".to_shared_string(),
            ui::AccountType::Cash,
            "500".to_shared_string(),
        );
        account_state.invoke_create_account(
            "".to_shared_string(),
            ui::AccountType::Credit,
            "500".to_shared_string(),
        );
        let transactions = controller.service.fetch_transactions()?;
        let t1 = &transactions[0];
        let t2 = &transactions[1];
        assert_eq!(t1.kind, TransactionKind::StartingBalance);
        assert_eq!(t2.kind, TransactionKind::StartingBalance);
        Ok(())
    }
}
