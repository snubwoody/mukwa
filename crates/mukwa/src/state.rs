// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use mukwa_core::Money;
use mukwa_core::service::{AccountType, Service};
use slint::{ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    service: Service,
    accounts: Rc<VecModel<ui::Account>>,
    // We can't map arrays in slint so we have to maintain duplicate arrays for comboboxes
    // see <https://github.com/slint-ui/slint/issues/1328>
    account_options: Rc<VecModel<ui::ComboBoxItem>>,
}

impl AppState {
    pub fn new(service: Service) -> crate::Result<AppState> {
        let accounts = service.fetch_accounts()?;
        let account_list: Vec<ui::Account> = accounts.iter().map(|a| a.into()).collect();
        let account_options: Vec<ui::ComboBoxItem> = accounts.iter().map(|a| a.into()).collect();

        let accounts_model = Rc::new(VecModel::from(account_list));
        let account_options_model = Rc::new(VecModel::from(account_options));

        let mut state = AppState {
            service,
            accounts: accounts_model,
            account_options: account_options_model,
        };

        state.load_accounts()?;
        Ok(state)
    }

    pub fn service(&self) -> Service {
        self.service.clone()
    }

    pub fn accounts(&self) -> Rc<VecModel<ui::Account>> {
        self.accounts.clone()
    }

    pub fn account_options(&self) -> Rc<VecModel<ui::ComboBoxItem>> {
        self.account_options.clone()
    }

    /// Creates a new account.
    pub fn create_account(
        &mut self,
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
                        .submit()?;
                }
                AccountType::Credit => {
                    self.service
                        .create_expense()
                        .account(account.id)
                        .note("Starting balance")
                        .amount(starting_balance)
                        .submit()?;
                }
            }
        }

        info!(id=?account.id,"Created new account");
        self.load_accounts()?;
        self.account_options.push(account.into());
        Ok(())
    }

    pub fn delete_account(&mut self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_account(id)?;
        info!("Deleted account {id}");
        // FIXME
        //self.reset_budgets(self.current_budget_month)?;
        //self.reset_categories()?;
        self.load_accounts()?;
        Ok(())
    }

    pub fn account_balance(&self, id: &str) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        self.service.account_balance(id)
    }

    fn load_accounts(&mut self) -> crate::Result<()> {
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
        self.accounts.set_vec(account_list);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::state::AppState;

    use mukwa_core::service::{AccountType, Service};
    use mukwa_core::{Money, create_test_db};
    use slint::Model;

    #[test]
    fn create_account_adds_to_account_list() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let accounts = service.fetch_accounts()?;
        let mut state = AppState::new(service)?;
        state.create_account("", AccountType::Cash.into(), "")?;
        state.create_account("", AccountType::Cash.into(), "")?;

        assert_eq!(state.accounts().iter().len(), accounts.len() + 2);
        Ok(())
    }

    #[test]
    fn create_cash_account_with_starting_balance() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let mut state = AppState::new(service)?;
        state.create_account("", AccountType::Cash.into(), "250.0")?;

        let transactions = state.service.fetch_transactions()?;
        assert_eq!(
            transactions[0].transaction_type(),
            mukwa_core::service::TransactionType::Income
        );
        assert_eq!(transactions[0].amount, Money::new(250));
        Ok(())
    }

    #[test]
    fn create_credit_account_with_starting_balance() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let mut state = AppState::new(service)?;
        state.create_account("", AccountType::Credit.into(), "150.0")?;

        let transactions = state.service.fetch_transactions()?;
        assert_eq!(
            transactions[0].transaction_type(),
            mukwa_core::service::TransactionType::Expense
        );
        assert_eq!(transactions[0].amount, Money::new(150));
        Ok(())
    }

    #[test]
    fn create_account_adds_to_account_options() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let accounts = service.fetch_accounts()?;

        let mut state = AppState::new(service)?;
        state.create_account("", AccountType::Cash.into(), "")?;
        state.create_account("", AccountType::Cash.into(), "")?;

        assert_eq!(state.account_options().iter().len(), accounts.len() + 2);
        Ok(())
    }

    #[test]
    fn state_loads_data_from_service() -> crate::Result<()> {
        let connection = create_test_db();
        let service = Service::new(connection);

        service.create_account("", AccountType::Cash)?;
        service.create_account("", AccountType::Cash)?;

        let state = AppState::new(service)?;
        assert_eq!(state.accounts().iter().len(), 2);
        Ok(())
    }
}
