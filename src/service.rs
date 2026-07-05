use crate::{Error, Money, ui};
use jiff::Zoned;
use jiff::civil::Date;
use rusqlite::{Connection, Row, params};
use slint::{SharedString, ToSharedString};
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
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

#[derive(PartialOrd, PartialEq, Debug, Default, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub category_id: Option<Uuid>,
    pub date: Date,
    pub amount: Money,
}

impl Transaction {
    /// Parses a `Transaction` from a `&str`.
    pub fn parse(value: &str) -> crate::Result<Transaction> {
        let parts: Vec<_> = value.split("|").collect();
        let id = Uuid::parse_str(parts[0])
            .map_err(|_| Error::ParseError("Invalid transaction id".to_string()))?;
        let date =
            Date::from_str(parts[1]).map_err(|_| Error::ParseError("Invalid date".to_string()))?;
        let account_id = Uuid::parse_str(parts[2])
            .map_err(|_| Error::ParseError("Invalid account id".to_string()))?;
        let category_id = Uuid::parse_str(parts[3]).ok();
        let amount = Money::from_scaled(parts[4].parse::<i64>()?);

        let transaction = Transaction {
            id,
            account_id,
            date,
            category_id,
            amount,
        };

        Ok(transaction)
    }
}

impl<'a> TryFrom<&rusqlite::Row<'a>> for Transaction {
    type Error = Error;
    fn try_from(value: &Row<'a>) -> Result<Self, Self::Error> {
        // FIXME
        let id: String = value.get("id")?;
        let account_id: String = value.get("account_id")?;
        let category_id: Option<String> = value.get("category_id")?;
        let transaction_date: String = value.get("transaction_date")?;
        let amount: i64 = value.get("amount")?;

        let category_id = match category_id {
            Some(id) => Some(Uuid::parse_str(&id)?),
            None => None,
        };

        let transaction = Transaction {
            id: Uuid::parse_str(&id)?,
            amount: Money::from_scaled(amount),
            date: Date::strptime("%Y-%m-%d", transaction_date)?,
            account_id: Uuid::parse_str(&account_id)?,
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

        Self {
            id: value.id.to_shared_string(),
            account_id: value.account_id.to_shared_string(),
            category_id: category_id.to_shared_string(),
            date: value.date.to_shared_string(),
            amount: value.amount.to_shared_string(),
        }
    }
}

impl From<&Transaction> for ui::Transaction {
    fn from(value: &Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        Self {
            id: value.id.to_string().into(),
            account_id: value.account_id.to_string().into(),
            category_id: category_id.into(),
            date: value.date.to_string().into(),
            amount: value.amount.to_shared_string(),
        }
    }
}

#[derive(Clone, PartialEq, PartialOrd, Default, Debug, Copy)]
pub struct UpdateTransactionOpts {
    /// The transaction id.
    pub id: Uuid,
    pub account_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub amount: Option<Money>,
    pub date: Option<Date>,
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug, Copy)]
pub struct CreateTransactionOpts {
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

    pub fn update_transaction(&self, opts: UpdateTransactionOpts) -> crate::Result<Transaction> {
        let sql = "UPDATE transactions \
        SET \
            account_id = COALESCE(?1,account_id), \
            category_id = COALESCE(?2,category_id), \
            amount = COALESCE(?3,amount), \
            transaction_date = COALESCE(?4,transaction_date) \
        WHERE id = ?5 \
        RETURNING *";

        let params = params![
            opts.account_id.map(|id| id.to_string()),
            opts.category_id.map(|id| id.to_string()),
            opts.amount.map(|a| a.inner()),
            opts.date.map(|d| d.to_string()),
            opts.id.to_string()
        ];

        let connection = self.connection();
        let mut stmt = connection.prepare_cached(sql)?;
        let mut rows = stmt.query_and_then(params, |row| Transaction::try_from(row))?;
        let transaction = rows.next().unwrap()?;
        Ok(transaction)
    }

    pub fn create_transaction(&self, opts: CreateTransactionOpts) -> crate::Result<Transaction> {
        let account_id = match opts.account_id {
            Some(id) => id,
            None => {
                let accounts = self.fetch_accounts()?;
                if accounts.is_empty() {
                    return Err(Error::generic(
                        "Cannot create a transaction without an account",
                    ));
                }
                accounts[0].id
            }
        };

        let connection = self.connection.lock().unwrap();
        let sql = "INSERT INTO transactions(id,transaction_date,account_id,category_id,amount) \
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
