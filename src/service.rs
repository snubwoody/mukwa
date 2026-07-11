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

use crate::{Error, Money, create_test_db, ui};
use jiff::Zoned;
use jiff::civil::Date;
use rusqlite::{Connection, Row, params};
use slint::{SharedString, ToSharedString};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Mutex, MutexGuard};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Ord, Eq)]
pub struct Account {
    pub id: Uuid,
    pub name: String,
}

impl Account {
    pub fn new(name: &str) -> Account {
        Account {
            id: Uuid::now_v7(),
            name: name.to_string(),
        }
    }
}

impl From<Account> for ui::Account {
    fn from(account: Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.into(),
            balance: SharedString::from("0.00"),
        }
    }
}

impl From<&Account> for ui::Account {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.clone().into(),
            balance: SharedString::from("0.00"),
        }
    }
}

// TODO: could maybe use default struct values
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
}

impl Category {
    pub fn new(title: &str) -> Category {
        Category {
            id: Uuid::now_v7(),
            title: title.to_string(),
        }
    }
}

impl From<Category> for ui::Category {
    fn from(value: Category) -> Self {
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

#[derive(PartialOrd, PartialEq, Debug, Default, Clone)]
pub struct Transaction {
    pub id: Uuid,
    /// The sending account.
    pub sender_id: Option<Uuid>,
    /// The receiving account.
    pub receiver_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub date: Date,
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

        Ok(Category {
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
        let account = Account {
            id: Uuid::parse_str(&id)?,
            name: value.get("name")?,
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

        // FIXME
        let outflow = if transaction_type == TransactionType::Transfer
            || transaction_type == TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        Self {
            id: value.id.to_shared_string(),
            // FIXME
            account_id: value.sender_id.unwrap().to_shared_string(),
            category_id: category_id.to_shared_string(),
            date: value.date.to_shared_string(),
            outflow,
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

        // FIXME
        let outflow = if transaction_type == TransactionType::Transfer
            || transaction_type == TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        Self {
            id: value.id.to_string().into(),
            account_id: value
                .sender_id
                .or(value.receiver_id)
                .unwrap()
                .to_string()
                .into(),
            category_id: category_id.into(),
            date: value.date.to_string().into(),
            outflow,
            inflow,
            transaction_type: transaction_type.into(),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Default, Debug, Copy)]
pub struct UpdateTransactionOpts {
    /// The transaction id.
    pub id: Uuid,
    pub sender_id: Option<Uuid>,
    pub receiver_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub amount: Option<Money>,
    pub date: Option<Date>,
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug, Copy)]
pub struct CreateTransactionOpts {
    /// The sending account
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub date: Date,
    pub amount: Money,
}

impl Default for CreateTransactionOpts {
    fn default() -> Self {
        CreateTransactionOpts {
            account_id: None,
            category_id: None,
            date: Zoned::now().date(),
            amount: Money::ZERO,
        }
    }
}

// TODO: add year and month ints and make them unique
#[derive(Clone)]
pub struct Service {
    connection: Rc<Mutex<Connection>>,
}

// TODO: check if foreign keys are enabled
impl Service {
    pub fn new(connection: Connection) -> Service {
        Service {
            connection: Rc::new(Mutex::new(connection)),
        }
    }

    pub fn connection(&self) -> MutexGuard<'_, Connection> {
        self.connection.lock().unwrap()
    }

    pub fn open(path: impl AsRef<Path>) -> crate::Result<Service> {
        let connection = Connection::open(path)?;
        let service = Service {
            connection: Rc::new(Mutex::new(connection)),
        };

        Ok(service)
    }

    pub fn open_in_memory() -> crate::Result<Service> {
        let connection = create_test_db();
        let service = Service {
            connection: Rc::new(Mutex::new(connection)),
        };

        Ok(service)
    }

    /// Fetches all accounts from the database.
    pub fn fetch_accounts(&self) -> crate::Result<Vec<Account>> {
        let mut accounts = vec![];
        let connection = self.connection.lock().unwrap();
        let sql = "SELECT * FROM accounts";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| Account::try_from(row))?;

        for row in rows {
            accounts.push(row?);
        }
        Ok(accounts)
    }

    /// Fetches all the budgets in a specific month.
    pub fn fetch_budgets_by_month(&self, date: Date) -> crate::Result<Vec<Budget>> {
        let mut budgets = vec![];
        let connection = self.connection.lock().unwrap();
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

    /// Fetches all transactions from the database.
    pub fn fetch_transactions(&self) -> crate::Result<Vec<Transaction>> {
        let mut transactions = vec![];
        let connection = self.connection.lock().unwrap();
        let sql = "SELECT * FROM transactions";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| Transaction::try_from(row))?;

        for row in rows {
            transactions.push(row?);
        }
        Ok(transactions)
    }

    /// Returns the total amount spent in the category in a specific month.
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

    /// Fetches all categories from the database.
    pub fn fetch_categories(&self) -> crate::Result<Vec<Category>> {
        let mut categories = vec![];
        let connection = self.connection.lock().unwrap();
        let sql = "SELECT * FROM categories";
        let mut stmt = connection.prepare_cached(sql)?;
        let rows = stmt.query_and_then([], |row| Category::try_from(row))?;

        for row in rows {
            categories.push(row?);
        }
        Ok(categories)
    }

    /// Creates a new [`Account`].
    pub fn create_account(&self, name: &str) -> crate::Result<Account> {
        let connection = self.connection.lock().unwrap();
        let sql = "INSERT INTO accounts(id,name) VALUES(?1,?2) RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([&Uuid::now_v7().to_string(), name], |row| {
            Account::try_from(row)
        })?;
        let account = rows.next().unwrap()?;
        Ok(account)
    }

    /// Creates a new [`Category`].
    pub fn create_category(&self, title: &str) -> crate::Result<Category> {
        let connection = self.connection.lock().unwrap();
        let sql = "INSERT INTO categories(id,title) VALUES(?1,?2) RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([&Uuid::now_v7().to_string(), title], |row| {
            Category::try_from(row)
        })?;
        let category = rows.next().unwrap()?;
        Ok(category)
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
        let connection = self.connection.lock().unwrap();
        let sql = "UPDATE categories SET title = ?1 WHERE id = ?2 RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then([title, id.to_string().as_str()], |row| {
            Category::try_from(row)
        })?;
        let category = rows.next().unwrap()?;
        Ok(category)
    }

    pub fn get_transaction(&self, id: Uuid) -> crate::Result<Transaction> {
        let connection = self.connection();
        let mut stmt = connection.prepare_cached("SELECT * FROM transactions WHERE id = ?")?;
        let mut rows = stmt.query_and_then([id.to_string()], |row| Transaction::try_from(row))?;
        rows.next().ok_or(Error::new("Transaction not found"))?
    }

    pub fn update_transaction(&self, opts: UpdateTransactionOpts) -> crate::Result<Transaction> {
        let transaction = self.get_transaction(opts.id)?;
        let mut connection = self.connection();

        let tx = connection.transaction()?;
        if let Some(date) = opts.date {
            tx.execute(
                "UPDATE transactions SET transaction_date = ?1 WHERE id = ?2",
                [date.to_string(), opts.id.to_string()],
            )?;
        }

        if let Some(amount) = opts.amount {
            let sql = "UPDATE transactions SET amount = ?1 WHERE id = ?2";
            tx.execute(sql, params![amount.inner(), opts.id.to_string()])?;
        }

        if let Some(id) = opts.sender_id {
            let sql = if transaction.transaction_type() == TransactionType::Income {
                "UPDATE transactions SET sender_id = ?1, receiver_id = NULL WHERE id = ?2"
            } else {
                "UPDATE transactions SET sender_id = ?1 WHERE id = ?2"
            };
            tx.execute(sql, [id.to_string(), opts.id.to_string()])?;
        }

        if let Some(id) = opts.receiver_id {
            let sql = if transaction.transaction_type() == TransactionType::Expense {
                "UPDATE transactions SET receiver_id = ?1, sender_id = NULL WHERE id = ?2"
            } else {
                "UPDATE transactions SET receiver_id = ?1 WHERE id = ?2"
            };
            tx.execute(sql, [id.to_string(), opts.id.to_string()])?;
        }

        tx.commit()?;

        let mut stmt = connection.prepare_cached("SELECT * FROM transactions WHERE id = ?1")?;
        let mut rows =
            stmt.query_and_then([opts.id.to_string()], |row| Transaction::try_from(row))?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
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

    /// Creates a new [`Transaction`].
    pub fn create_transaction(&self, opts: CreateTransactionOpts) -> crate::Result<Transaction> {
        let account_id = match opts.account_id {
            Some(id) => id,
            None => {
                let accounts = self.fetch_accounts()?;
                if accounts.is_empty() {
                    return Err(Error::new("Cannot create a transaction without an account"));
                }
                accounts[0].id
            }
        };

        let category_id = opts.category_id.map(|id| id.to_string());

        let connection = self.connection.lock().unwrap();
        let sql = "INSERT INTO transactions(id,transaction_date,sender_id,category_id,amount) \
            VALUES(?1,?2,?3,?4,?5) \
            RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let params = params![
            Uuid::now_v7().to_string(),
            opts.date.to_string(),
            account_id.to_string(),
            category_id,
            opts.amount.inner()
        ];
        let mut rows = stmt.query_and_then(params, |row| Transaction::try_from(row))?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }
}
