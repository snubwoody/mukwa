// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::service::{AccountType, Service, Transaction};
use slint::{Model, SharedString, ToSharedString, VecModel};
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
    transactions: Rc<VecModel<ui::Transaction>>,
}

impl AppState {
    pub fn new(service: Service) -> crate::Result<AppState> {
        let mut transactions = service.fetch_transactions()?;
        transactions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        let transactions_list: Vec<ui::Transaction> =
            transactions.iter().map(|t| t.into()).collect();

        let transactions_model = Rc::new(VecModel::from(transactions_list));

        let accounts = service.fetch_accounts()?;
        let account_list: Vec<ui::Account> = accounts.iter().map(|a| a.into()).collect();
        let account_options: Vec<ui::ComboBoxItem> = accounts.iter().map(|a| a.into()).collect();

        let accounts_model = Rc::new(VecModel::from(account_list));
        let account_options_model = Rc::new(VecModel::from(account_options));

        let mut state = AppState {
            service,
            accounts: accounts_model,
            account_options: account_options_model,
            transactions: transactions_model,
        };

        state.load_accounts()?;
        Ok(state)
    }

    pub fn transactions(&self) -> Rc<VecModel<ui::Transaction>> {
        self.transactions.clone()
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
        self.load_transactions()?;
        self.load_accounts()?;
        self.account_options.push(account.into());
        Ok(())
    }

    /// Creates a new transaction.
    pub fn create_transaction(&mut self, opts: ui::CreateTransactionOpts) -> crate::Result<()> {
        let date = Date::strptime("%Y-%m-%d", &opts.date)?;

        let transaction =
            if !opts.outflow.is_empty() && opts.inflow.is_empty() && opts.payee_id.is_empty() {
                let amount = Money::from_str(&opts.outflow)?;
                let mut builder = self.service.create_expense().amount(amount).date(date);

                if !opts.account_id.is_empty() {
                    builder = builder.account(Uuid::parse_str(&opts.account_id)?);
                }

                if !opts.category_id.is_empty() {
                    builder = builder.category(Uuid::parse_str(&opts.category_id)?);
                }

                if !opts.note.is_empty() {
                    builder = builder.note(&opts.note);
                }

                builder.submit()?
            } else if !opts.inflow.is_empty() {
                let amount = Money::from_str(&opts.inflow)?;
                let mut builder = self.service.create_income().amount(amount).date(date);

                if !opts.account_id.is_empty() {
                    builder = builder.account(Uuid::parse_str(&opts.account_id)?);
                }

                if !opts.note.is_empty() {
                    builder = builder.note(&opts.note);
                }

                builder.submit()?
            } else {
                let amount = Money::from_str(&opts.outflow)?;
                let account_id = Uuid::parse_str(&opts.account_id)?;
                let payee_id = Uuid::parse_str(&opts.payee_id)?;
                let mut builder = self
                    .service
                    .create_transfer()
                    .accounts(account_id, payee_id)
                    .amount(amount)
                    .date(date);

                if !opts.note.is_empty() {
                    builder = builder.note(&opts.note);
                }

                builder.submit()?
            };

        info!(id=?transaction.id,"Created new transaction");
        self.transactions.insert(0, transaction.into());
        self.load_accounts()?;
        Ok(())
    }

    pub fn delete_transaction(&mut self, id: &str) -> crate::Result<()> {
        let tid = Uuid::parse_str(id)?;
        self.service.delete_transaction(tid)?;
        info!("Deleted transaction {id}");
        let transactions = self
            .transactions
            .iter()
            .filter(|t| t.id.as_str() != id)
            .collect::<Vec<_>>();
        self.transactions.set_vec(transactions);
        self.load_accounts()?;
        Ok(())
    }
    pub fn confirm_transaction(&mut self, id: &str) -> crate::Result<()> {
        let tid = Uuid::parse_str(id)?;
        self.service.confirm_transaction(tid)?;
        info!("Confirmed transaction {id}");
        self.load_transactions()?;
        self.load_accounts()?;
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
        self.load_transactions()?;
        Ok(())
    }

    pub fn duplicate_transaction(&mut self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let transaction = self.service.duplicate_transaction(id)?;
        self.transactions.push(transaction.into());
        info!("Duplicated transaction {id}");
        self.load_accounts()?;
        Ok(())
    }

    pub fn account_balance(&self, id: &str) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        self.service.account_balance(id)
    }

    pub fn set_transaction_date(&mut self, id: &str, date: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let date = Date::strptime("%Y-%m-%d", date)?;
        let transaction = self.service.set_transaction_date(id, date)?;
        info!(id=?id,"Updated transaction date");
        self.replace_transaction(transaction);
        Ok(())
    }

    pub fn set_transaction_note(&mut self, id: &str, note: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let transaction = self.service.set_transaction_note(id, note)?;
        info!(id=?id,"Updated transaction note");
        self.replace_transaction(transaction);
        Ok(())
    }

    pub fn set_transaction_account(&mut self, id: &str, account_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let account_id = Uuid::parse_str(account_id)?;
        let transaction = self.service.set_transaction_account(id, account_id)?;
        info!(id=?id,"Updated transaction account");
        self.replace_transaction(transaction);
        self.load_accounts()?;
        Ok(())
    }

    pub fn set_transaction_payee(&mut self, id: &str, account_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let account_id = Uuid::parse_str(account_id)?;
        let transaction = self.service.set_transaction_payee(id, account_id)?;
        info!(id=?id,"Updated transaction payee");
        self.replace_transaction(transaction);
        self.load_accounts()?;
        Ok(())
    }

    pub fn set_transaction_outflow(&mut self, id: &str, amount: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        let transaction = self.service.set_transaction_outflow(id, amount)?;
        info!(id=?id,"Updated transaction outflow");
        self.replace_transaction(transaction);
        self.load_accounts()?;
        Ok(())
    }

    pub fn set_transaction_inflow(&mut self, id: &str, amount: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        let transaction = self.service.set_transaction_inflow(id, amount)?;
        info!(id=?id,"Updated transaction inflow");
        self.replace_transaction(transaction);
        self.load_accounts()?;
        Ok(())
    }

    pub fn set_transaction_category(&mut self, id: &str, category_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let category_id = Uuid::parse_str(category_id)?;
        let transaction = self.service.set_transaction_category(id, category_id)?;
        info!(id=?id,"Updated transaction category");
        self.replace_transaction(transaction);
        self.load_accounts()?;
        Ok(())
    }

    fn replace_transaction(&mut self, transaction: Transaction) {
        let transactions: Vec<ui::Transaction> = self
            .transactions
            .iter()
            .map(|t| {
                if t.id == transaction.id.to_shared_string() {
                    transaction.clone().into()
                } else {
                    t
                }
            })
            .collect();
        self.transactions.set_vec(transactions);
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

    pub(crate) fn load_transactions(&self) -> crate::Result<()> {
        let mut transactions = self.service.fetch_transactions()?;
        transactions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        let transactions: Vec<ui::Transaction> = transactions.iter().map(|t| t.into()).collect();
        self.transactions.set_vec(transactions);
        Ok(())
    }

    #[allow(unused)]
    pub fn get_account(&self, id: SharedString) -> Option<ui::Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}

#[cfg(test)]
mod test {
    use crate::state::AppState;
    use crate::ui::CreateTransactionOpts;
    use jiff::Zoned;

    use mukwa_core::service::{AccountType, Service};
    use mukwa_core::{Money, create_test_db};
    use slint::{Model, SharedString, ToSharedString};

    #[test]
    fn set_outflow_reloads_accounts() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("", AccountType::Cash)?;
        let transaction = service
            .create_income()
            .account(account.id)
            .amount(Money::new(500))
            .submit()?;
        let mut state = AppState::new(service)?;
        state.set_transaction_outflow(&transaction.id.to_shared_string(), "400")?;
        let account = state.get_account(account.id.to_shared_string()).unwrap();
        assert_eq!(account.balance, Money::new(-400).to_shared_string());
        Ok(())
    }

    #[test]
    fn set_inflow_reloads_accounts() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("", AccountType::Cash)?;
        let transaction = service
            .create_income()
            .account(account.id)
            .amount(Money::new(50))
            .submit()?;
        let mut state = AppState::new(service)?;
        state.set_transaction_inflow(&transaction.id.to_shared_string(), "100")?;
        let account = state.get_account(account.id.to_shared_string()).unwrap();
        assert_eq!(account.balance, Money::new(100).to_shared_string());
        Ok(())
    }

    #[test]
    fn duplicate_transaction_reloads_accounts() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("", AccountType::Cash)?;
        let transaction = service
            .create_income()
            .account(account.id)
            .amount(Money::new(50))
            .submit()?;
        let mut state = AppState::new(service)?;
        state.duplicate_transaction(&transaction.id.to_shared_string())?;
        let account = state.get_account(account.id.to_shared_string()).unwrap();
        assert_eq!(account.balance, Money::new(100).to_shared_string());
        Ok(())
    }

    #[test]
    fn delete_transaction_reloads_accounts() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("", AccountType::Cash)?;
        let transaction = service
            .create_income()
            .account(account.id)
            .amount(Money::new(50))
            .submit()?;
        let mut state = AppState::new(service)?;
        state.delete_transaction(&transaction.id.to_shared_string())?;
        let account = state.get_account(account.id.to_shared_string()).unwrap();
        assert_eq!(account.balance, Money::new(0).to_shared_string());
        Ok(())
    }

    #[test]
    fn create_expense() -> crate::Result<()> {
        let connection = create_test_db();
        let service = Service::new(connection);
        let account = service.create_account("", AccountType::Cash)?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;

        let mut state = AppState::new(service)?;
        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            outflow: "0.00".to_shared_string(),
            category_id: category.id.to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            note: SharedString::from("Pick n Pay"),
            ..Default::default()
        };
        state.create_transaction(opts)?;

        let transaction = state.transactions().remove(0);
        assert_eq!(transaction.account_id, account.id.to_shared_string());
        assert_eq!(transaction.category_id, category.id.to_shared_string());
        assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
        assert_eq!(transaction.note.as_str(), "Pick n Pay");
        assert_eq!(transaction.inflow.as_str(), "");
        assert_eq!(transaction.payee_id.as_str(), "");
        assert_eq!(transaction.outflow.as_str(), Money::ZERO.to_string());
        Ok(())
    }

    #[test]
    fn create_income() -> crate::Result<()> {
        let connection = create_test_db();
        let service = Service::new(connection);
        let account = service.create_account("", AccountType::Cash)?;

        let mut state = AppState::new(service)?;
        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            inflow: "0.00".to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            note: SharedString::from("Pick n Pay"),
            ..Default::default()
        };
        state.create_transaction(opts)?;

        let transaction = state.transactions().remove(0);
        assert_eq!(transaction.account_id, account.id.to_shared_string());
        assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
        assert_eq!(transaction.note.as_str(), "Pick n Pay");
        assert_eq!(transaction.outflow.as_str(), "");
        assert_eq!(transaction.payee_id.as_str(), "");
        assert_eq!(transaction.inflow.as_str(), Money::ZERO.to_string());
        Ok(())
    }

    #[test]
    fn create_expense_uses_account() -> crate::Result<()> {
        let connection = create_test_db();
        let service = Service::new(connection);
        service.create_account("", AccountType::Cash)?;
        service.create_account("", AccountType::Cash)?;
        service.create_account("", AccountType::Cash)?;
        let account = service.create_account("", AccountType::Cash)?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;

        let mut state = AppState::new(service)?;
        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            outflow: "0.00".to_shared_string(),
            category_id: category.id.to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            note: SharedString::from("Pick n Pay"),
            ..Default::default()
        };
        state.create_transaction(opts)?;

        let transaction = state.transactions().remove(0);
        assert_eq!(transaction.account_id, account.id.to_shared_string());
        Ok(())
    }

    #[test]
    fn create_expense_empty_category() -> crate::Result<()> {
        let connection = create_test_db();
        let service = Service::new(connection);
        service.create_account("", AccountType::Cash)?;

        let mut state = AppState::new(service)?;
        let opts = CreateTransactionOpts {
            outflow: "0.00".to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            ..Default::default()
        };
        state.create_transaction(opts)?;
        Ok(())
    }

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

        service.create_expense().submit()?;
        service.create_expense().submit()?;
        service.create_expense().submit()?;

        let state = AppState::new(service)?;
        assert_eq!(state.transactions().iter().len(), 3);
        assert_eq!(state.accounts().iter().len(), 2);
        Ok(())
    }
}
