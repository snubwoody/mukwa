use crate::{Error, Money, ui};
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
    pub date: Date,
    pub amount: Money,
}

impl Default for CreateTransactionOpts {
    fn default() -> Self {
        CreateTransactionOpts {
            account_id: None,
            date: Zoned::now().date(),
            amount: Money::ZERO,
        }
    }
}

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

        let connection = self.connection.lock().unwrap();
        let sql = "INSERT INTO transactions(id,transaction_date,sender_id,category_id,amount) \
            VALUES(?1,?2,?3,?4,?5) \
            RETURNING *";
        let mut stmt = connection.prepare_cached(sql)?;
        let category_id: Option<String> = None;
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
