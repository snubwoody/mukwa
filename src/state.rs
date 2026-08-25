// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::service::{AccountType, CreateBudgetOpts, Service, Transaction};
use crate::{Money, ui};
use jiff::Zoned;
use jiff::civil::Date;
use slint::{Model, SharedString, ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{debug, info};
use uuid::Uuid;

// TODO: set the new transaction to editing
#[derive(Clone)]
pub struct AppState {
    service: Service,
    accounts: Rc<VecModel<ui::Account>>,
    categories: Rc<VecModel<ui::Category>>,
    category_groups: Rc<VecModel<ui::CategoryGroup>>,
    current_budget_month: Date,
    /// The budgets of the active month.
    budgets: Rc<VecModel<ui::Budget>>,
    // We can't map arrays in slint so we have to maintain duplicate arrays for comboboxes
    // see <https://github.com/slint-ui/slint/issues/1328>
    account_options: Rc<VecModel<ui::ComboBoxItem>>,
    category_options: Rc<VecModel<ui::ComboBoxItem>>,
    transactions: Rc<VecModel<ui::Transaction>>,
}

impl AppState {
    pub fn new(service: Service) -> crate::Result<AppState> {
        let mut transactions = service.fetch_transactions()?;
        transactions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        let transactions_list: Vec<ui::Transaction> =
            transactions.iter().map(|t| t.into()).collect();

        let transactions_model = Rc::new(VecModel::from(transactions_list));

        let categories = service.fetch_categories()?;
        let category_list: Vec<ui::Category> = categories.iter().map(|c| c.into()).collect();
        let category_options: Vec<ui::ComboBoxItem> = categories.iter().map(|c| c.into()).collect();
        let category_groups = service.fetch_category_groups()?;
        let category_group_list: Vec<ui::CategoryGroup> =
            category_groups.iter().map(|c| c.into()).collect();

        let category_model = Rc::new(VecModel::from(category_list));
        let category_group_model = Rc::new(VecModel::from(category_group_list));
        let category_options_model = Rc::new(VecModel::from(category_options));

        let budgets_list: Vec<ui::Budget> = service
            .fetch_budgets_by_month(Zoned::now().date())?
            .iter()
            .map(|b| b.into())
            .collect();
        let budget_model = Rc::new(VecModel::from(budgets_list));

        let accounts = service.fetch_accounts()?;
        let account_list: Vec<ui::Account> = accounts.iter().map(|a| a.into()).collect();
        let account_options: Vec<ui::ComboBoxItem> = accounts.iter().map(|a| a.into()).collect();

        let accounts_model = Rc::new(VecModel::from(account_list));
        let account_options_model = Rc::new(VecModel::from(account_options));

        let mut state = AppState {
            service,
            accounts: accounts_model,
            categories: category_model,
            category_groups: category_group_model,
            current_budget_month: Zoned::now().date(),
            account_options: account_options_model,
            transactions: transactions_model,
            category_options: category_options_model,
            budgets: budget_model,
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

    pub fn categories(&self) -> Rc<VecModel<ui::Category>> {
        self.categories.clone()
    }

    pub fn category_groups(&self) -> Rc<VecModel<ui::CategoryGroup>> {
        self.category_groups.clone()
    }

    pub fn budgets(&self) -> Rc<VecModel<ui::Budget>> {
        self.budgets.clone()
    }

    pub fn account_options(&self) -> Rc<VecModel<ui::ComboBoxItem>> {
        self.account_options.clone()
    }

    pub fn category_options(&self) -> Rc<VecModel<ui::ComboBoxItem>> {
        self.category_options.clone()
    }

    /// Creates a new account.
    pub fn create_account(
        &mut self,
        name: &str,
        account_type: ui::AccountType,
    ) -> crate::Result<()> {
        let account_type = match account_type {
            ui::AccountType::Cash => AccountType::Cash,
            ui::AccountType::Credit => AccountType::Credit,
        };
        let account = self.service.create_account(name, account_type)?;
        info!(id=?account.id,"Created new account");
        self.accounts.push(account.clone().into());
        self.account_options.push(account.into());
        Ok(())
    }

    pub fn set_current_budget_month(&mut self, date: Date) -> crate::Result<()> {
        self.current_budget_month = date;
        self.reset_budgets(date)?;
        Ok(())
    }

    /// Creates a new category.
    pub fn create_category(&mut self, title: &str, group_id: &str) -> crate::Result<()> {
        let group_id = Uuid::parse_str(group_id)?;
        let category = self.service.create_category(title, group_id)?;
        info!(id=?category.id,"Created new category");

        self.reset_budgets(self.current_budget_month)?;
        self.category_options.push(category.clone().into());
        self.categories.push(category.into());
        Ok(())
    }

    /// Creates a new category group.
    pub fn create_category_group(&mut self, title: &str) -> crate::Result<()> {
        let category_group = self.service.create_category_group(title)?;
        info!(id=?category_group.id,"Created new category group");

        self.reset_budgets(self.current_budget_month)?;
        self.category_groups.push(category_group.into());
        Ok(())
    }

    /// Creates a new budget.
    pub fn create_budget(&mut self, opts: CreateBudgetOpts) -> crate::Result<()> {
        let budget = self.service.create_budget(opts)?;
        info!(id=?budget.id,"Created new budget");
        self.budgets.push(budget.into());
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

    pub fn delete_category(&mut self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_category(id)?;
        info!("Deleted category {id}");
        self.reset_budgets(self.current_budget_month)?;
        self.reset_categories()?;
        self.load_transactions()?;
        Ok(())
    }

    pub fn delete_category_group(&mut self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_category_group(id)?;
        info!("Deleted category group {id}");
        self.reset_budgets(self.current_budget_month)?;
        self.reset_category_groups()?;
        self.reset_categories()?;
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

    pub fn total_spent(&self, id: &str) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        let budget = self.service.get_budget(id)?;
        let date = Date::new(budget.year as i16, budget.month as i8, 1)?;
        self.service.total_spent(budget.category_id, date)
    }

    pub fn total_spent_in_group(&self, id: &str, date: ui::Date) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        let date = Date::new(date.year as i16, date.month as i8, date.day as i8)?;
        let total = self.service.total_spent_in_group(id, date)?;
        Ok(total)
    }

    pub fn total_assigned_in_group(&self, id: &str, date: ui::Date) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        let date = Date::new(date.year as i16, date.month as i8, date.day as i8)?;
        let total = self.service.total_assigned_in_group(id, date)?;
        Ok(total)
    }

    pub fn fetch_or_init_budgets(&self, date: Date) -> crate::Result<Vec<ui::Budget>> {
        let budgets: Vec<ui::Budget> = self
            .service
            .fetch_or_init_budgets(date)?
            .iter()
            .map(|b| b.into())
            .collect();
        Ok(budgets)
    }

    pub fn left_to_spend(&self, id: &str) -> crate::Result<Money> {
        let total = self.total_spent(id)?;
        let id = Uuid::parse_str(id)?;
        let budget = self.service.get_budget(id)?;
        let available = budget.amount - total;
        Ok(available.max(Money::ZERO))
    }

    pub fn left_to_spend_in_group(&self, id: &str, date: ui::Date) -> crate::Result<Money> {
        let total = self.total_spent_in_group(id, date.clone())?;
        let id = Uuid::parse_str(id)?;
        let date = Date::new(date.year as i16, date.month as i8, date.day as i8)?;
        let assigned = self.service.total_assigned_in_group(id, date)?;
        let available = assigned - total;
        Ok(available.max(Money::ZERO))
    }

    pub fn move_category(&mut self, id: &str, group_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let group_id = Uuid::parse_str(group_id)?;
        self.service.move_category(id, group_id)?;
        self.reset_budgets(self.current_budget_month)?;
        self.reset_category_groups()?;
        self.reset_categories()?;
        debug!("Moved category {id} into group {group_id}");
        Ok(())
    }

    pub fn update_budget(&mut self, id: &str, amount: &str) -> crate::Result<()> {
        let budget_id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        let new_budget = self.service.update_budget(budget_id, amount)?;
        info!(id=?id,"Updated budget");
        let budgets: Vec<ui::Budget> = self
            .budgets
            .iter()
            .map(|budget| {
                if budget.id == new_budget.id.to_shared_string() {
                    new_budget.into()
                } else {
                    budget
                }
            })
            .collect();
        self.budgets.set_vec(budgets);
        self.reset_categories()?;
        self.reset_category_groups()?;
        Ok(())
    }

    pub fn update_category(&mut self, id: &str, title: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.update_category(id, title)?;
        info!(id=?id,"Updated category");
        self.reset_categories()?;
        self.reset_budgets(self.current_budget_month)?;
        Ok(())
    }

    pub fn update_category_group(&mut self, id: &str, title: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.update_category_group(id, title)?;
        info!(id=?id,"Updated category group");
        self.reset_categories()?;
        self.reset_category_groups()?;
        self.reset_budgets(self.current_budget_month)?;
        Ok(())
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

    fn reset_budgets(&mut self, month: Date) -> crate::Result<()> {
        let budgets_list: Vec<ui::Budget> = self
            .service
            .fetch_or_init_budgets(month)?
            .iter()
            .map(|b| b.into())
            .collect();
        self.budgets.set_vec(budgets_list);
        Ok(())
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

    fn reset_categories(&mut self) -> crate::Result<()> {
        let categories: Vec<ui::Category> = self
            .service
            .fetch_categories()?
            .iter()
            .map(|c| c.into())
            .collect();
        self.categories.set_vec(categories);
        Ok(())
    }

    pub(crate) fn load_transactions(&mut self) -> crate::Result<()> {
        let transactions: Vec<ui::Transaction> = self
            .service
            .fetch_transactions()?
            .iter()
            .map(|t| t.into())
            .collect();
        self.transactions.set_vec(transactions);
        Ok(())
    }

    fn reset_category_groups(&mut self) -> crate::Result<()> {
        let groups: Vec<ui::CategoryGroup> = self
            .service
            .fetch_category_groups()?
            .iter()
            .map(|b| b.into())
            .collect();
        self.category_groups.set_vec(groups);
        Ok(())
    }

    pub fn get_account(&self, id: SharedString) -> Option<ui::Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
}
