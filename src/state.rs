// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::service::{CreateBudgetOpts, Service, Transaction};
use crate::{Money, ui};
use jiff::Zoned;
use jiff::civil::Date;
use slint::{Model, SharedString, ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::info;
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

        Ok(AppState {
            service,
            accounts: accounts_model,
            categories: category_model,
            category_groups: category_group_model,
            current_budget_month: Zoned::now().date(),
            account_options: account_options_model,
            transactions: transactions_model,
            category_options: category_options_model,
            budgets: budget_model,
        })
    }

    pub fn transactions(&self) -> Rc<VecModel<ui::Transaction>> {
        self.transactions.clone()
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
    pub fn create_account(&mut self, name: &str) -> crate::Result<()> {
        let account = self.service.create_account(name)?;
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

    /// Creates a new expense.
    pub fn create_transaction(&mut self) -> crate::Result<()> {
        let transaction = self.service.create_expense().submit()?;
        info!(id=?transaction.id,"Created new transaction");
        self.transactions.insert(0, transaction.into());
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
        Ok(())
    }

    pub fn duplicate_transaction(&mut self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let transaction = self.service.duplicate_transaction(id)?;
        self.transactions.push(transaction.into());
        info!("Duplicated transaction {id}");
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
        Ok(())
    }

    pub fn update_category(&mut self, id: &str, title: &str) -> crate::Result<()> {
        let budget_id = Uuid::parse_str(id)?;
        self.service.update_category(budget_id, title)?;
        info!(id=?id,"Updated category");
        self.reset_categories()?;
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
        Ok(())
    }

    pub fn set_transaction_payee(&mut self, id: &str, account_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let account_id = Uuid::parse_str(account_id)?;
        let transaction = self.service.set_transaction_payee(id, account_id)?;
        info!(id=?id,"Updated transaction payee");
        self.replace_transaction(transaction);
        Ok(())
    }

    pub fn set_transaction_outflow(&mut self, id: &str, amount: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        let transaction = self.service.set_transaction_outflow(id, amount)?;
        info!(id=?id,"Updated transaction outflow");
        self.replace_transaction(transaction);
        Ok(())
    }

    pub fn set_transaction_inflow(&mut self, id: &str, amount: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        let transaction = self.service.set_transaction_inflow(id, amount)?;
        info!(id=?id,"Updated transaction inflow");
        self.replace_transaction(transaction);
        Ok(())
    }

    pub fn set_transaction_category(&mut self, id: &str, category_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let category_id = Uuid::parse_str(category_id)?;
        let transaction = self.service.set_transaction_category(id, category_id)?;
        info!(id=?id,"Updated transaction category");
        self.replace_transaction(transaction);
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

    fn reset_categories(&mut self) -> crate::Result<()> {
        let categories: Vec<ui::Category> = self
            .service
            .fetch_categories()?
            .iter()
            .map(|b| b.into())
            .collect();
        self.categories.set_vec(categories);
        Ok(())
    }

    pub fn get_account(&self, id: SharedString) -> Option<ui::Account> {
        self.accounts.iter().find(|a| a.id == id)
    }
}
