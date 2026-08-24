// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::{Error, Money, create_test_db, ui};
use jiff::Zoned;
use jiff::civil::Date;
use rusqlite::{Connection, Row, params};
use slint::{SharedString, ToSharedString};
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Default)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
}

impl From<Account> for ui::Account {
    fn from(account: Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.into(),
            account_type: account.account_type.into(),
            balance: Money::ZERO.to_shared_string(),
        }
    }
}

impl From<Account> for ui::ComboBoxItem {
    fn from(account: Account) -> Self {
        Self {
            value: account.id.to_string().into(),
            text: account.name.to_string().into(),
        }
    }
}

impl From<&Account> for ui::ComboBoxItem {
    fn from(account: &Account) -> Self {
        Self {
            value: account.id.to_string().into(),
            text: account.name.to_string().into(),
        }
    }
}

impl From<&Account> for ui::Account {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.clone().into(),
            account_type: account.account_type.into(),
            balance: Money::ZERO.to_shared_string(),
        }
    }
}

impl From<&AccountType> for ui::AccountType {
    fn from(value: &AccountType) -> Self {
        match value {
            AccountType::Cash => Self::Cash,
            AccountType::Credit => Self::Credit,
        }
    }
}

impl From<AccountType> for ui::AccountType {
    fn from(value: AccountType) -> Self {
        match value {
            AccountType::Cash => Self::Cash,
            AccountType::Credit => Self::Credit,
        }
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy, Ord, Eq, Default)]
pub enum AccountType {
    #[default]
    Cash = 1,
    Credit = 2,
}

#[derive(PartialOrd, PartialEq, Debug, Default, Clone, Copy)]
pub struct CreateBudgetOpts {
    pub amount: Option<Money>,
    /// The budget month, defaults to the current month.
    pub month: Option<Date>,
    pub category_id: Uuid,
}

#[derive(PartialOrd, PartialEq, Debug, Default, Clone, Copy)]
pub struct Budget {
    pub id: Uuid,
    pub amount: Money,
    pub month: i64,
    pub year: i64,
    pub category_id: Uuid,
}

impl From<Budget> for ui::Budget {
    fn from(value: Budget) -> Self {
        Self {
            id: value.id.to_shared_string(),
            amount: value.amount.to_shared_string(),
            year: value.year as i32,
            month: value.month as i32,
            category_id: value.category_id.to_shared_string(),
        }
    }
}

impl From<&Budget> for ui::Budget {
    fn from(value: &Budget) -> Self {
        Self {
            id: value.id.to_shared_string(),
            amount: value.amount.to_shared_string(),
            year: value.year as i32,
            month: value.month as i32,
            category_id: value.category_id.to_shared_string(),
        }
    }
}

impl<'a> TryFrom<&Row<'a>> for Budget {
    type Error = Error;

    fn try_from(value: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = value.get("id")?;
        let category_id: String = value.get("category_id")?;
        let amount: i64 = value.get("amount")?;
        let year: i64 = value.get("year")?;
        let month: i64 = value.get("month")?;

        Ok(Budget {
            id: Uuid::parse_str(&id)?,
            year,
            month,
            amount: Money::from_scaled(amount),
            category_id: Uuid::parse_str(&category_id)?,
        })
    }
}

#[derive(PartialOrd, PartialEq, Debug, Default, Clone)]
pub struct Category {
    pub id: Uuid,
    pub title: String,
    pub group_id: Uuid,
}

#[derive(PartialOrd, PartialEq, Debug, Default, Clone)]
pub struct CategoryGroup {
    pub id: Uuid,
    pub title: String,
}

impl From<Category> for ui::Category {
    fn from(value: Category) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            group_id: value.group_id.to_shared_string(),
        }
    }
}

impl From<CategoryGroup> for ui::CategoryGroup {
    fn from(value: CategoryGroup) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
        }
    }
}

impl From<&CategoryGroup> for ui::CategoryGroup {
    fn from(value: &CategoryGroup) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
        }
    }
}

impl From<&Category> for ui::Category {
    fn from(value: &Category) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            group_id: value.group_id.to_shared_string(),
        }
    }
}

impl From<Category> for ui::ComboBoxItem {
    fn from(value: Category) -> Self {
        Self {
            value: value.id.to_string().into(),
            text: value.title.to_string().into(),
        }
    }
}

impl From<&Category> for ui::ComboBoxItem {
    fn from(value: &Category) -> Self {
        Self {
            value: value.id.to_string().into(),
            text: value.title.to_string().into(),
        }
    }
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy, Eq, Ord)]
pub enum TransactionType {
    Expense,
    Income,
    Transfer,
}

impl From<TransactionType> for ui::TransactionType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::Expense => ui::TransactionType::Expense,
            TransactionType::Income => ui::TransactionType::Income,
            TransactionType::Transfer => ui::TransactionType::Transfer,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct Transaction {
    pub id: Uuid,
    /// The sending account.
    pub sender_id: Option<Uuid>,
    /// The receiving account.
    pub receiver_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub date: Date,
    pub note: Option<String>,
    pub amount: Money,
}

impl Transaction {
    pub fn transaction_type(&self) -> TransactionType {
        if self.sender_id.is_some() && self.receiver_id.is_some() {
            return TransactionType::Transfer;
        }

        if self.receiver_id.is_some() {
            return TransactionType::Income;
        }

        TransactionType::Expense
    }
}

impl<'a> TryFrom<&Row<'a>> for Category {
    type Error = Error;

    fn try_from(value: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = value.get("id")?;
        let title: String = value.get("title")?;
        let group_id: String = value.get("group_id")?;

        Ok(Category {
            id: Uuid::parse_str(&id)?,
            title: title.to_string(),
            group_id: Uuid::parse_str(&group_id)?,
        })
    }
}

impl<'a> TryFrom<&Row<'a>> for CategoryGroup {
    type Error = Error;

    fn try_from(value: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = value.get("id")?;
        let title: String = value.get("title")?;

        Ok(CategoryGroup {
            id: Uuid::parse_str(&id)?,
            title: title.to_string(),
        })
    }
}

impl<'a> TryFrom<&Row<'a>> for Transaction {
    type Error = Error;

    fn try_from(value: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = value.get("id")?;
        let sender_id: Option<String> = value.get("sender_id")?;
        let receiver_id: Option<String> = value.get("receiver_id")?;
        let category_id: Option<String> = value.get("category_id")?;
        let note: Option<String> = value.get("note")?;
        let transaction_date: String = value.get("transaction_date")?;
        let amount: i64 = value.get("amount")?;

        let category_id = match category_id {
            Some(id) => Some(Uuid::parse_str(&id)?),
            None => None,
        };

        let sender_id = match sender_id {
            Some(id) => Some(Uuid::parse_str(&id)?),
            None => None,
        };

        let receiver_id = match receiver_id {
            Some(id) => Some(Uuid::parse_str(&id)?),
            None => None,
        };

        let transaction = Transaction {
            id: Uuid::parse_str(&id)?,
            amount: Money::from_scaled(amount),
            date: Date::strptime("%Y-%m-%d", transaction_date)?,
            note,
            sender_id,
            receiver_id,
            category_id,
        };

        Ok(transaction)
    }
}

impl<'a> TryFrom<&Row<'a>> for Account {
    type Error = Error;

    fn try_from(value: &Row<'a>) -> Result<Self, Self::Error> {
        let id: String = value.get("id")?;
        let account_type = match value.get::<_, i64>("account_type_id")? {
            1 => Ok(AccountType::Cash),
            2 => Ok(AccountType::Credit),
            _ => Err(Error::new("Failed to parse account type")),
        };

        let account = Account {
            id: Uuid::parse_str(&id)?,
            name: value.get("name")?,
            account_type: account_type?,
        };

        Ok(account)
    }
}

impl From<Transaction> for ui::Transaction {
    fn from(value: Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        let transaction_type = value.transaction_type();
        let inflow = if transaction_type == TransactionType::Income {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let outflow = if transaction_type == TransactionType::Transfer
            || transaction_type == TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let note = value.note.unwrap_or_default().to_shared_string();

        let account_id = match transaction_type {
            TransactionType::Income => value.receiver_id.unwrap().to_shared_string(),
            _ => value.sender_id.unwrap().to_shared_string(),
        };

        let payee_id = match transaction_type {
            TransactionType::Transfer => value.receiver_id.unwrap().to_shared_string(),
            _ => SharedString::new(),
        };

        Self {
            id: value.id.to_shared_string(),
            account_id,
            payee_id,
            category_id: category_id.to_shared_string(),
            date: value.date.to_shared_string(),
            outflow,
            note,
            inflow,
            transaction_type: transaction_type.into(),
        }
    }
}

impl From<&Transaction> for ui::Transaction {
    fn from(value: &Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        let transaction_type = value.transaction_type();
        let inflow = if transaction_type == TransactionType::Income {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let outflow = if transaction_type == TransactionType::Transfer
            || transaction_type == TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let note = value.note.clone().unwrap_or_default().to_shared_string();

        let account_id = match transaction_type {
            TransactionType::Income => value.receiver_id.unwrap().to_shared_string(),
            _ => value.sender_id.unwrap().to_shared_string(),
        };

        let payee_id = match transaction_type {
            TransactionType::Transfer => value.receiver_id.unwrap().to_shared_string(),
            _ => SharedString::new(),
        };

        Self {
            id: value.id.to_string().into(),
            account_id,
            payee_id,
            category_id: category_id.into(),
            note,
            date: value.date.to_string().into(),
            outflow,
            inflow,
            transaction_type: transaction_type.into(),
        }
    }
}

pub struct Income;
pub struct Expense;
pub struct Transfer;

pub struct TransactionBuilder<T> {
    service: Service,
    amount: Money,
    sender_id: Option<Uuid>,
    receiver_id: Option<Uuid>,
    date: Date,
    category_id: Option<Uuid>,
    note: Option<String>,
    marker: PhantomData<T>,
}

impl<T> TransactionBuilder<T> {
    fn new(service: Service) -> TransactionBuilder<T> {
        Self {
            service,
            amount: Money::ZERO,
            sender_id: None,
            receiver_id: None,
            category_id: None,
            note: None,
            date: Zoned::now().date(),
            marker: PhantomData,
        }
    }
}

impl<T> TransactionBuilder<T> {
    /// Sets the transaction amount.
    pub fn amount(mut self, amount: Money) -> TransactionBuilder<T> {
        self.amount = amount;
        self
    }

    /// Sets the transaction note.
    pub fn note(mut self, note: &str) -> TransactionBuilder<T> {
        self.note = Some(note.to_owned());
        self
    }

    /// Sets the transaction date.
    pub fn date(mut self, date: Date) -> TransactionBuilder<T> {
        self.date = date;
        self
    }
}

impl TransactionBuilder<Income> {
    pub fn account(mut self, id: Uuid) -> Self {
        self.receiver_id = Some(id);
        self
    }

    /// Submits the query.
    pub fn submit(self) -> crate::Result<Transaction> {
        let account_id = match self.receiver_id {
            Some(id) => id,
            None => {
                let accounts = self.service.fetch_accounts()?;
                if accounts.is_empty() {
                    return Err(Error::new("Cannot create a transaction without an account"));
                }
                accounts[0].id
            }
        };

        let connection = self.service.connection();
        let sql = "INSERT INTO transactions(id,transaction_date,receiver_id,amount,note) \
            VALUES(?1,?2,?3,?4,?5) \
            RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let params = params![
            Uuid::now_v7().to_string(),
            self.date.to_string(),
            account_id.to_string(),
            self.amount.inner(),
            self.note,
        ];
        let mut rows = stmt.query_and_then(params, |row| Transaction::try_from(row))?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }
}

impl TransactionBuilder<Expense> {
    pub fn account(mut self, id: Uuid) -> Self {
        self.sender_id = Some(id);
        self
    }

    pub fn category(mut self, id: Uuid) -> Self {
        self.category_id = Some(id);
        self
    }

    /// Submits the query.
    pub fn submit(self) -> crate::Result<Transaction> {
        let account_id = match self.sender_id {
            Some(id) => id,
            None => {
                let accounts = self.service.fetch_accounts()?;
                if accounts.is_empty() {
                    return Err(Error::new("Cannot create a transaction without an account"));
                }
                accounts[0].id
            }
        };

        let connection = self.service.connection();
        let sql = "INSERT INTO transactions(id,transaction_date,sender_id,category_id,amount,note) \
            VALUES(?1,?2,?3,?4,?5,?6) \
            RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let params = params![
            Uuid::now_v7().to_string(),
            self.date.to_string(),
            account_id.to_string(),
            self.category_id.map(|c| c.to_string()),
            self.amount.inner(),
            self.note,
        ];
        let mut rows = stmt.query_and_then(params, |row| Transaction::try_from(row))?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }
}
impl TransactionBuilder<Transfer> {
    pub fn accounts(mut self, from: Uuid, to: Uuid) -> Self {
        self.sender_id = Some(from);
        self.receiver_id = Some(to);
        self
    }

    /// Submits the query.
    pub fn submit(self) -> crate::Result<Transaction> {
        let receiver_id = self.receiver_id.ok_or(Error::new("Missing receiver id"))?;
        let sender_id = self.sender_id.ok_or(Error::new("Missing sender id"))?;

        let connection = self.service.connection();
        let sql = "INSERT INTO transactions(id,transaction_date,receiver_id,sender_id,amount,note) \
            VALUES(?1,?2,?3,?4,?5,?6) \
            RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let params = params![
            Uuid::now_v7().to_string(),
            self.date.to_string(),
            receiver_id.to_string(),
            sender_id.to_string(),
            self.amount.inner(),
            self.note,
        ];
        let mut rows = stmt.query_and_then(params, |row| Transaction::try_from(row))?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }
}

#[derive(Clone)]
pub struct Service {
    connection: Rc<Connection>,
}

impl Service {
    pub fn new(connection: Connection) -> Service {
        Service {
            connection: Rc::new(connection),
        }
    }

    pub fn connection(&self) -> Rc<Connection> {
        self.connection.clone()
    }

    pub fn open(path: impl AsRef<Path>) -> crate::Result<Service> {
        let connection = Connection::open(path)?;
        let service = Service {
            connection: Rc::new(connection),
        };

        Ok(service)
    }

    /// Opens an in-memory service for testing.
    pub fn open_in_memory() -> crate::Result<Service> {
        let connection = create_test_db();
        let service = Service {
            connection: Rc::new(connection),
        };

        service.create_account("Test account", AccountType::Cash)?;

        Ok(service)
    }

    /// Fetches all accounts from the database.
    pub fn fetch_accounts(&self) -> crate::Result<Vec<Account>> {
        let mut accounts = vec![];
        let connection = self.connection();
        let sql = "SELECT * FROM accounts";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| Account::try_from(row))?;

        for row in rows {
            accounts.push(row?);
        }
        Ok(accounts)
    }

    /// Creates a new expense builder.
    ///
    /// ## Example
    /// ```
    /// use mukwa::Money;
    /// use mukwa::service::{TransactionType,Service,AccountType};
    ///
    /// fn main() -> mukwa::Result<()>{
    ///     let service = Service::open_in_memory()?;
    ///
    ///     let account = service.create_account("Credit card",AccountType::Credit)?;
    ///     let transaction = service
    ///         .create_expense()
    ///         .amount(Money::new(5))
    ///         .account(account.id)
    ///         .submit()?;
    ///
    ///     assert_eq!(transaction.amount,Money::new(5));
    ///     assert_eq!(transaction.transaction_type(),TransactionType::Expense);
    ///     Ok(())
    /// }
    /// ```
    pub fn create_expense(&self) -> TransactionBuilder<Expense> {
        TransactionBuilder::new(self.clone())
    }

    /// Creates a new income builder.
    ///
    /// ## Example
    /// ```
    /// use mukwa::Money;
    /// use jiff::civil::date;
    /// use mukwa::service::{TransactionType,AccountType,Service};
    ///
    /// fn main() -> mukwa::Result<()>{
    ///     let service = Service::open_in_memory()?;
    ///
    ///     let account = service.create_account("Chequing",AccountType::Cash)?;
    ///     let transaction = service
    ///         .create_income()
    ///         .amount(Money::new(12_500))
    ///         .account(account.id)
    ///         .date(date(2020,1,1))
    ///         .submit()?;
    ///
    ///     assert_eq!(transaction.amount,Money::new(12_500));
    ///     assert_eq!(transaction.date,date(2020,1,1));
    ///     assert_eq!(transaction.transaction_type(),TransactionType::Income);
    ///     Ok(())
    /// }
    /// ```
    pub fn create_income(&self) -> TransactionBuilder<Income> {
        TransactionBuilder::new(self.clone())
    }

    /// Creates a new transfer builder.
    ///
    /// ## Example
    /// ```
    /// use mukwa::Money;
    /// use mukwa::service::{TransactionType,AccountType,Service};
    ///
    /// fn main() -> mukwa::Result<()>{
    ///     let service = Service::open_in_memory()?;
    ///
    ///     let account = service.create_account("Chequing",AccountType::Cash)?;
    ///     let account2 = service.create_account("Savings",AccountType::Cash)?;
    ///     let transaction = service
    ///         .create_transfer()
    ///         .amount(Money::new(4000))
    ///         .accounts(account.id,account2.id)
    ///         .submit()?;
    ///
    ///     assert_eq!(transaction.transaction_type(),TransactionType::Transfer);
    ///     Ok(())
    /// }
    /// ```
    pub fn create_transfer(&self) -> TransactionBuilder<Transfer> {
        TransactionBuilder::new(self.clone())
    }

    /// Fetches all the budgets in a specific month.
    pub fn fetch_budgets_by_month(&self, date: Date) -> crate::Result<Vec<Budget>> {
        let mut budgets = vec![];
        let connection = self.connection();
        let sql = "SELECT * FROM budgets WHERE month = ?1 AND year = ?2";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then(params![date.month(), date.year()], |row| {
            Budget::try_from(row)
        })?;

        for row in rows {
            budgets.push(row?);
        }
        Ok(budgets)
    }

    pub fn fetch_or_init_budgets(&self, date: Date) -> crate::Result<Vec<Budget>> {
        let categories = self.fetch_categories()?;
        let budgets = self.fetch_budgets_by_month(date)?;

        for category in categories {
            if budgets
                .iter()
                .find(|budget| budget.category_id == category.id)
                .is_some()
            {
                continue;
            }
            self.create_budget(CreateBudgetOpts {
                category_id: category.id,
                month: Some(date),
                ..Default::default()
            })?;
        }

        self.fetch_budgets_by_month(date)
    }

    /// Deletes a category from the database
    pub fn delete_category(&self, id: Uuid) -> crate::Result<()> {
        let connection = self.connection();
        let sql = "DELETE FROM categories WHERE id = ?";
        let mut stmt = connection.prepare_cached(sql)?;
        stmt.execute([id.to_string()])?;
        Ok(())
    }

    /// Deletes a category group from the database
    pub fn delete_category_group(&self, id: Uuid) -> crate::Result<()> {
        let connection = self.connection();
        let sql = "DELETE FROM category_groups WHERE id = ?";
        let mut stmt = connection.prepare_cached(sql)?;
        stmt.execute([id.to_string()])?;
        Ok(())
    }

    /// Fetches all transactions from the database.
    pub fn fetch_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let mut transactions = vec![];
        let connection = self.connection();
        let sql = "SELECT * FROM transactions";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| Transaction::try_from(row))?;

        for row in rows {
            transactions.push(row?);
        }
        Ok(transactions)
    }

    /// Calculates the total amount spent in the category in a specific month.
    pub fn total_spent(&self, category_id: Uuid, month: Date) -> crate::Result<Money> {
        let connection = self.connection();
        let sql = "SELECT * FROM transactions WHERE category_id = ?1";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows =
            stmt.query_and_then([category_id.to_string()], |row| Transaction::try_from(row))?;

        let mut total = Money::ZERO;
        for row in rows {
            let transaction = row?;
            let is_same_month = transaction.date.month() == month.month()
                && transaction.date.year() == month.year();

            if !is_same_month {
                continue;
            }

            total += transaction.amount;
        }
        Ok(total)
    }

    /// Calculates the total amount spent in the category group.
    pub fn total_spent_in_group(&self, group_id: Uuid, month: Date) -> crate::Result<Money> {
        let connection = self.connection();

        let sql = "SELECT \
            t.id,t.amount,t.transaction_date,t.sender_id,t.receiver_id,t.category_id,t.note \
            FROM transactions t \
            LEFT JOIN categories c ON t.category_id = c.id \
            WHERE c.group_id = ?";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([group_id.to_string()], |row| Transaction::try_from(row))?;

        let mut total = Money::ZERO;
        for row in rows {
            let transaction = row?;

            let is_same_month = transaction.date.month() == month.month()
                && transaction.date.year() == month.year();

            if !is_same_month {
                continue;
            }

            total += transaction.amount;
        }
        Ok(total)
    }

    pub fn total_assigned_in_group(&self, group_id: Uuid, month: Date) -> crate::Result<Money> {
        let connection = self.connection();

        let sql = "SELECT b.* FROM budgets b \
            LEFT JOIN categories c ON b.category_id = c.id \
            WHERE c.group_id = ?";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([group_id.to_string()], |row| Budget::try_from(row))?;

        let mut total = Money::ZERO;
        for row in rows {
            let budget = row?;

            let is_same_month =
                budget.month == month.month().into() && budget.year == month.year().into();

            if !is_same_month {
                continue;
            }

            total += budget.amount;
        }
        Ok(total)
    }

    /// Calculates the account balance.
    pub fn account_balance(&self, account_id: Uuid) -> crate::Result<Money> {
        let connection = self.connection();
        let mut expense_stmt = connection.prepare_cached(
            "SELECT coalesce(sum(amount),0) FROM transactions WHERE sender_id = ?1",
        )?;
        let mut income_stmt = connection.prepare_cached(
            "SELECT coalesce(sum(amount),0) FROM transactions WHERE receiver_id = ?1",
        )?;
        let total_expenses = expense_stmt.query_one([account_id.to_string()], |row| {
            Ok(Money::from_scaled(row.get::<_, i64>(0)?))
        })?;
        let total_incomes = income_stmt.query_one([account_id.to_string()], |row| {
            Ok(Money::from_scaled(row.get::<_, i64>(0)?))
        })?;

        let total = total_incomes - total_expenses;

        Ok(total)
    }

    /// Fetches all categories from the database.
    pub fn fetch_categories(&self) -> crate::Result<Vec<Category>> {
        let mut categories = vec![];
        let connection = self.connection();
        let sql = "SELECT * FROM categories";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| Category::try_from(row))?;

        for row in rows {
            categories.push(row?);
        }
        Ok(categories)
    }

    /// Fetches all category groups from the database.
    pub fn fetch_category_groups(&self) -> crate::Result<Vec<CategoryGroup>> {
        let mut category_groups = vec![];
        let connection = self.connection();
        let sql = "SELECT * FROM category_groups";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| CategoryGroup::try_from(row))?;

        for row in rows {
            category_groups.push(row?);
        }
        Ok(category_groups)
    }

    /// Creates a new [`Account`].
    pub fn create_account(&self, name: &str, account_type: AccountType) -> crate::Result<Account> {
        let account_type = match account_type {
            AccountType::Cash => 1,
            AccountType::Credit => 2,
        };
        let connection = self.connection();
        let sql = "INSERT INTO accounts(id,name,account_type_id) VALUES(?1,?2,?3) RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then(
            params![&Uuid::now_v7().to_string(), name, account_type],
            |row| Account::try_from(row),
        )?;
        let account = rows.next().unwrap()?;
        Ok(account)
    }

    /// Creates a new [`Category`].
    pub fn create_category(&self, title: &str, group_id: Uuid) -> crate::Result<Category> {
        let connection = self.connection();
        let sql = "INSERT INTO categories(id,title,group_id) VALUES(?1,?2,?3) RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;

        let mut rows = stmt.query_and_then(
            params![&Uuid::now_v7().to_string(), title, group_id.to_string()],
            |row| Category::try_from(row),
        )?;
        let category = rows.next().unwrap()?;
        Ok(category)
    }

    /// Creates a new [`CategoryGroup`].
    pub fn create_category_group(&self, title: &str) -> crate::Result<CategoryGroup> {
        let connection = self.connection();
        let sql = "INSERT INTO category_groups(id,title) VALUES(?1,?2) RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([&Uuid::now_v7().to_string(), title], |row| {
            CategoryGroup::try_from(row)
        })?;
        let group = rows.next().unwrap()?;
        Ok(group)
    }

    /// Creates a new [`Budget`].
    pub fn create_budget(&self, opts: CreateBudgetOpts) -> crate::Result<Budget> {
        let connection = self.connection();
        let amount = opts.amount.unwrap_or_default().inner();
        let date = opts.month.unwrap_or(Zoned::now().date());

        let sql = "INSERT INTO budgets(id,amount,category_id,month,year) \
        VALUES(?1,?2,?3,?4,?5) \
        RETURNING *";

        let mut stmt = connection.prepare_cached(sql)?;
        let params = params![
            Uuid::now_v7().to_string(),
            amount,
            opts.category_id.to_string(),
            date.month(),
            date.year()
        ];
        let mut rows = stmt.query_and_then(params, |row| Budget::try_from(row))?;
        let budget = rows.next().unwrap()?;

        Ok(budget)
    }

    pub fn update_category(&self, id: Uuid, title: &str) -> crate::Result<Category> {
        let connection = self.connection();
        let sql = "UPDATE categories SET title = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([title, id.to_string().as_str()], |row| {
            Category::try_from(row)
        })?;
        let category = rows.next().unwrap()?;
        Ok(category)
    }

    /// Moves the category into the category group.
    pub fn move_category(&self, id: Uuid, group_id: Uuid) -> crate::Result<Category> {
        let connection = self.connection();
        let sql = "UPDATE categories SET group_id = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then(
            [group_id.to_string().as_str(), id.to_string().as_str()],
            |row| Category::try_from(row),
        )?;
        let category = rows.next().unwrap()?;
        Ok(category)
    }

    pub fn update_category_group(&self, id: Uuid, title: &str) -> crate::Result<CategoryGroup> {
        let connection = self.connection();
        let sql = "UPDATE category_groups SET title = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([title, id.to_string().as_str()], |row| {
            CategoryGroup::try_from(row)
        })?;
        let group = rows.next().unwrap()?;
        Ok(group)
    }

    pub fn update_budget(&self, id: Uuid, amount: Money) -> crate::Result<Budget> {
        let connection = self.connection();
        let sql = "UPDATE budgets SET amount = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt
            .query_and_then(params![amount.inner(), id.to_string().as_str()], |row| {
                Budget::try_from(row)
            })?;
        let budget = rows.next().unwrap()?;
        Ok(budget)
    }

    pub fn get_transaction(&self, id: Uuid) -> crate::Result<Transaction> {
        let connection = self.connection();
        let mut stmt = connection.prepare_cached("SELECT * FROM transactions WHERE id = ?")?;
        let mut rows = stmt.query_and_then([id.to_string()], |row| Transaction::try_from(row))?;
        rows.next().ok_or(Error::new("Transaction not found"))?
    }

    pub fn get_budget(&self, id: Uuid) -> crate::Result<Budget> {
        let connection = self.connection();
        let mut stmt = connection.prepare_cached("SELECT * FROM budgets WHERE id = ?")?;
        let mut rows = stmt.query_and_then([id.to_string()], |row| Budget::try_from(row))?;
        rows.next().ok_or(Error::new("Budget not found"))?
    }

    pub fn set_transaction_note(&self, id: Uuid, note: &str) -> crate::Result<Transaction> {
        let connection = self.connection();
        let sql = "UPDATE transactions SET note = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then(params![note, id.to_string().as_str()], |row| {
            Transaction::try_from(row)
        })?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }

    pub fn set_transaction_date(&self, id: Uuid, date: Date) -> crate::Result<Transaction> {
        let connection = self.connection();
        let sql = "UPDATE transactions SET transaction_date = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt
            .query_and_then(params![date.to_string(), id.to_string().as_str()], |row| {
                Transaction::try_from(row)
            })?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }

    pub fn set_transaction_category(
        &self,
        id: Uuid,
        category_id: Uuid,
    ) -> crate::Result<Transaction> {
        let transaction = self.get_transaction(id)?;
        if transaction.transaction_type() != TransactionType::Expense {
            let error = Error::new("Invalid transaction type (only expenses can have a category)");
            return Err(error);
        }
        let connection = self.connection();
        let sql = "UPDATE transactions SET category_id = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([category_id.to_string(), id.to_string()], |row| {
            Transaction::try_from(row)
        })?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }

    pub fn set_transaction_account(
        &self,
        id: Uuid,
        account_id: Uuid,
    ) -> crate::Result<Transaction> {
        let transaction = self.get_transaction(id)?;
        let sql = match transaction.transaction_type() {
            TransactionType::Income => {
                "UPDATE transactions SET receiver_id = ?1, sender_id = NULL WHERE id = ?2 RETURNING *"
            }
            TransactionType::Expense => {
                "UPDATE transactions SET sender_id = ?1, receiver_id = NULL WHERE id = ?2 RETURNING *"
            }
            TransactionType::Transfer => {
                "UPDATE transactions SET sender_id = ?1 WHERE id = ?2 RETURNING *"
            }
        };
        let connection = self.connection();
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([account_id.to_string(), id.to_string()], |row| {
            Transaction::try_from(row)
        })?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }

    pub fn set_transaction_payee(&self, id: Uuid, account_id: Uuid) -> crate::Result<Transaction> {
        let transaction = self.get_transaction(id)?;
        let sql = match transaction.transaction_type() {
            TransactionType::Income => {
                "UPDATE transactions SET sender_id = ?1, category_id = NULL WHERE id = ?2 RETURNING *"
            }
            TransactionType::Expense => {
                "UPDATE transactions SET receiver_id = ?1, category_id = NULL WHERE id = ?2 RETURNING *"
            }
            TransactionType::Transfer => {
                "UPDATE transactions SET receiver_id = ?1, category_id = NULL WHERE id = ?2 RETURNING *"
            }
        };
        let connection = self.connection();
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([account_id.to_string(), id.to_string()], |row| {
            Transaction::try_from(row)
        })?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }

    pub fn set_transaction_outflow(&self, id: Uuid, amount: Money) -> crate::Result<Transaction> {
        let transaction = self.get_transaction(id)?;
        let connection = self.connection();
        match transaction.transaction_type() {
            TransactionType::Income => {
                let sql = "UPDATE transactions SET amount = ?1, sender_id = ?2, receiver_id = null WHERE id = ?3 RETURNING *";
                let mut stmt = connection.prepare_cached(sql)?;
                let mut rows = stmt.query_and_then(
                    params![
                        amount.inner(),
                        transaction.receiver_id.unwrap().to_string(),
                        id.to_string()
                    ],
                    |row| Transaction::try_from(row),
                )?;
                let transaction = rows.next().unwrap()?;
                Ok(transaction)
            }
            _ => {
                let sql = "UPDATE transactions SET amount = ?1 WHERE id = ?2 RETURNING *";
                let mut stmt = connection.prepare_cached(sql)?;
                let mut rows = stmt
                    .query_and_then(params![amount.inner(), id.to_string()], |row| {
                        Transaction::try_from(row)
                    })?;
                let transaction = rows.next().unwrap()?;
                Ok(transaction)
            }
        }
    }

    pub fn set_transaction_inflow(&self, id: Uuid, amount: Money) -> crate::Result<Transaction> {
        let transaction = self.get_transaction(id)?;
        let connection = self.connection();
        match transaction.transaction_type() {
            TransactionType::Expense => {
                let sql = "UPDATE transactions SET amount = ?1, receiver_id = ?2, sender_id = null WHERE id = ?3 RETURNING *";
                let mut stmt = connection.prepare_cached(sql)?;
                let mut rows = stmt.query_and_then(
                    params![
                        amount.inner(),
                        transaction.sender_id.unwrap().to_string(),
                        id.to_string()
                    ],
                    |row| Transaction::try_from(row),
                )?;
                let transaction = rows.next().unwrap()?;
                Ok(transaction)
            }
            TransactionType::Income => {
                let sql = "UPDATE transactions SET amount = ?1 WHERE id = ?2 RETURNING *";
                let mut stmt = connection.prepare_cached(sql)?;
                let mut rows = stmt
                    .query_and_then(params![amount.inner(), id.to_string()], |row| {
                        Transaction::try_from(row)
                    })?;
                let transaction = rows.next().unwrap()?;
                Ok(transaction)
            }
            TransactionType::Transfer => Err(Error::new("Invalid transaction type")),
        }
    }

    pub fn delete_transaction(&self, id: Uuid) -> crate::Result<()> {
        let connection = self.connection();
        let mut stmt = connection.prepare_cached("DELETE FROM transactions WHERE id = ?")?;
        stmt.execute([id.to_string()])?;
        Ok(())
    }

    /// Duplicates a transaction, the new transaction will have all the same values as the old transaction
    /// except the `id`.
    pub fn duplicate_transaction(&self, id: Uuid) -> crate::Result<Transaction> {
        let connection = self.connection();
        let sql = "INSERT INTO transactions(id,sender_id,receiver_id,category_id,transaction_date,amount) \
        SELECT ?1,sender_id,receiver_id,category_id,transaction_date,amount FROM transactions \
        WHERE id = ?2 RETURNING *";

        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt
            .query_and_then([Uuid::now_v7().to_string(), id.to_string()], |row| {
                Transaction::try_from(row)
            })?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn create_expense() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("My account", AccountType::Cash)?;
        let group = service.create_category_group("")?;
        let category = service.create_category("Movies", group.id)?;
        let expense = service
            .create_expense()
            .amount(Money::from_f64(25.24))
            .date(date(20, 1, 1))
            .category(category.id)
            .account(account.id)
            .note("The odyssey")
            .submit()?;

        assert_eq!(expense.amount, Money::from_f64(25.24));
        assert_eq!(expense.sender_id.unwrap(), account.id);
        assert_eq!(expense.category_id.unwrap(), category.id);
        assert_eq!(expense.date, date(20, 1, 1));
        assert_eq!(expense.note.unwrap(), "The odyssey");
        assert!(expense.receiver_id.is_none());
        Ok(())
    }

    #[test]
    fn create_income() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("My account", AccountType::Cash)?;
        let income = service
            .create_income()
            .date(date(20, 1, 1))
            .account(account.id)
            .amount(Money::from_f64(25.24))
            .note("The odyssey")
            .submit()?;

        assert_eq!(income.amount, Money::from_f64(25.24));
        assert_eq!(income.receiver_id.unwrap(), account.id);
        assert!(income.category_id.is_none());
        assert_eq!(income.date, date(20, 1, 1));
        assert_eq!(income.note.unwrap(), "The odyssey");
        assert!(income.sender_id.is_none());
        Ok(())
    }

    #[test]
    fn create_transfer() -> crate::Result<()> {
        let service = Service::open_in_memory()?;
        let account = service.create_account("My account", AccountType::Cash)?;
        let account2 = service.create_account("My account 2", AccountType::Cash)?;
        let transfer = service
            .create_transfer()
            .date(date(2100, 12, 1))
            .accounts(account.id, account2.id)
            .amount(Money::from_f64(25.24))
            .note("Transfer to savings")
            .submit()?;

        assert_eq!(transfer.amount, Money::from_f64(25.24));
        assert_eq!(transfer.sender_id.unwrap(), account.id);
        assert_eq!(transfer.receiver_id.unwrap(), account2.id);
        assert!(transfer.category_id.is_none());
        assert_eq!(transfer.date, date(2100, 12, 1));
        assert_eq!(transfer.note.unwrap(), "Transfer to savings");
        Ok(())
    }
}
