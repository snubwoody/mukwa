use crate::{ui, Error, Money};
use jiff::civil::Date;
use jiff::Zoned;
use slint::{SharedString, ToSharedString};
use std::fmt::Display;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::str::{FromStr, Lines};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
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

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let category_id = match self.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };
        write!(
            f,
            "{}|{}|{}|{}|{}",
            self.id,
            self.date,
            self.account_id,
            category_id,
            self.amount.inner()
        )
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
            amount: value.to_shared_string(),
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

#[derive(Clone, PartialEq, PartialOrd, Default, Debug)]
pub struct UpdateTransactionOpts {
    /// The transaction id.
    pub id: Uuid,
    pub account_id: Option<Uuid>,
    pub amount: Option<Money>,
    pub date: Option<Date>,
}

// TODO: use buffered writer <https://doc.rust-lang.org/std/io/struct.BufWriter.html>
#[derive(Clone, Default)]
pub struct Service {
    path: PathBuf,
    accounts: Vec<Account>,
    categories: Vec<Category>,
    transactions: Vec<Transaction>,
}

impl Service {
    pub fn open(path: impl AsRef<Path>) -> Service {
        Service {
            path: path.as_ref().to_path_buf(),
            accounts: Vec::new(),
            categories: Vec::new(),
            transactions: Vec::new(),
        }
    }

    pub fn accounts(&self) -> &[Account] {
        self.accounts.as_ref()
    }
    pub fn transactions(&self) -> &[Transaction] {
        self.transactions.as_ref()
    }

    pub fn categories(&self) -> &[Category] {
        self.categories.as_ref()
    }

    pub fn create_account(&mut self, name: &str) -> crate::Result<Account> {
        let account = Account {
            id: Uuid::now_v7(),
            name: name.to_string(),
        };

        self.accounts.push(account.clone());
        self.write()?;
        Ok(account)
    }

    pub fn update_transaction(
        &mut self,
        opts: UpdateTransactionOpts,
    ) -> crate::Result<Transaction> {
        let (index, transaction) = self
            .transactions
            .iter()
            .enumerate()
            .find(|(_, t)| t.id == opts.id)
            .ok_or(Error::not_found(&format!(
                "Transaction ({}) not found",
                opts.id
            )))?;

        let mut new_transaction = transaction.clone();
        if let Some(account_id) = opts.account_id {
            new_transaction.account_id = account_id;
        }

        if let Some(amount) = opts.amount {
            new_transaction.amount = amount;
        }

        if let Some(date) = opts.date {
            new_transaction.date = date;
        }

        self.transactions[index] = new_transaction;
        self.write()?;
        Ok(self.transactions[index].clone())
    }

    pub fn create_transaction(&mut self) -> crate::Result<Transaction> {
        let account_id = match self.accounts.first() {
            Some(account) => account.id,
            None => Uuid::now_v7(),
        };

        let transaction = Transaction {
            date: Zoned::now().date(),
            account_id,
            id: Uuid::now_v7(),
            category_id: None,
            amount: Money::ZERO,
        };
        self.transactions.push(transaction.clone());
        self.write()?;
        Ok(transaction)
    }

    pub fn get_account(&self, id: Uuid) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn get_transaction(&self, id: Uuid) -> Option<&Transaction> {
        self.transactions.iter().find(|t| t.id == id)
    }

    pub fn read(&mut self) -> crate::Result<()> {
        info!("Loading data from {:?}", self.path);
        let data = fs::read_to_string(&self.path)?;

        let mut lines = data.lines().peekable();

        while let Some(line) = lines.next() {
            match line {
                "[Accounts]" => {
                    self.parse_accounts(&mut lines)?;
                }
                "[Categories]" => {
                    self.parse_categories(&mut lines)?;
                }
                "[Transactions]" => {
                    self.parse_transactions(&mut lines)?;
                }
                " " => {}
                _ => return Err(Error::ParseError(format!("Invalid section: {line}"))),
            }
        }

        Ok(())
    }

    fn parse_accounts(&mut self, lines: &mut Peekable<Lines>) -> crate::Result<()> {
        while let Some(line) = lines.next() {
            let parts: Vec<_> = line.split("|").collect();
            let id = parts[0];
            let name = parts[1];
            let account = Account {
                id: Uuid::parse_str(id)?,
                name: name.to_owned(),
            };
            self.accounts.push(account);
            match lines.peek() {
                Some(val) if self.is_section_header(val) => break,
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_transactions(&mut self, lines: &mut Peekable<Lines>) -> crate::Result<()> {
        while let Some(line) = lines.next() {
            let transaction = Transaction::parse(line)?;
            self.transactions.push(transaction);
            match lines.peek() {
                Some(val) if self.is_section_header(val) => break,
                _ => {}
            }
        }
        Ok(())
    }

    fn is_section_header(&self, value: &str) -> bool {
        matches!(value, "[Accounts]" | "[Categories]" | "[Transactions]")
    }

    fn parse_categories(&mut self, lines: &mut Peekable<Lines>) -> crate::Result<()> {
        while let Some(line) = lines.next() {
            let parts: Vec<_> = line.split("|").collect();
            let id = parts[0];
            let title = parts[1];
            let category = Category {
                id: Uuid::parse_str(id)?,
                title: title.to_owned(),
            };
            self.categories.push(category);
            match lines.peek() {
                Some(val) if self.is_section_header(val) => break,
                _ => {}
            }
        }
        Ok(())
    }

    pub fn write(&self) -> crate::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .truncate(true)
            .open(&self.path)?;

        writeln!(file, "[Accounts]")?;
        for account in &self.accounts {
            writeln!(file, "{}|{}", account.id, account.name)?;
        }

        writeln!(file, "[Categories]")?;
        for category in &self.categories {
            writeln!(file, "{}|{}", category.id, category.title)?;
        }

        writeln!(file, "[Transactions]")?;
        for transaction in &self.transactions {
            writeln!(file, "{transaction}",)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::civil::date;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_transaction() -> crate::Result<()> {
        let id = Uuid::now_v7();
        let account_id = Uuid::now_v7();
        let category_id = Uuid::now_v7();
        let date = date(1999, 1, 1);
        let amount = Money::new(200);
        let value = format!("{id}|{date}|{account_id}|{category_id}|{}", amount.inner());
        let transaction = Transaction::parse(&value)?;
        assert_eq!(transaction.id, id);
        assert_eq!(transaction.account_id, account_id);
        assert_eq!(transaction.category_id.unwrap(), category_id);
        assert_eq!(transaction.date, date);
        assert_eq!(transaction.amount, amount);
        Ok(())
    }

    #[test]
    fn parse_transaction_empty_category() -> crate::Result<()> {
        let id = Uuid::now_v7();
        let account_id = Uuid::now_v7();
        let date = date(1999, 1, 1);
        let value = format!("{id}|{date}|{account_id}||0");
        let transaction = Transaction::parse(&value)?;
        assert_eq!(transaction.id, id);
        assert_eq!(transaction.account_id, account_id);
        assert!(transaction.category_id.is_none());
        assert_eq!(transaction.date, date);
        Ok(())
    }

    #[test]
    fn transaction_to_string() -> crate::Result<()> {
        let category_id = Uuid::now_v7();
        let amount = Money::new(600);
        let transaction = Transaction {
            category_id: Some(category_id),
            amount,
            ..Default::default()
        };
        let value = format!(
            "{}|{}|{}|{}|{}",
            transaction.id,
            transaction.date,
            transaction.account_id,
            category_id,
            amount.inner()
        );
        assert_eq!(transaction.to_string(), value);
        Ok(())
    }

    #[test]
    fn save_to_file() -> crate::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("app.data");
        let mut service = Service::open(&path);
        let category = Category::new("Groceries");
        let account = Account::new("Savings");
        service.categories.push(category.clone());
        service.accounts.push(account.clone());
        service.write()?;

        let content = fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines[0], "[Accounts]");
        assert_eq!(lines[1], format!("{}|Savings", account.id));
        assert_eq!(lines[2], "[Categories]");
        assert_eq!(lines[3], format!("{}|Groceries", category.id));

        Ok(())
    }

    #[test]
    fn parse_file() -> crate::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("app.data");
        let mut service = Service::open(&path);

        let mut file = OpenOptions::new().create(true).write(true).open(&path)?;

        write!(file, "[Accounts]\n")?;
        write!(file, "{}|Savings\n", Uuid::now_v7())?;
        write!(file, "[Categories]\n")?;
        write!(file, "{}|Transport\n", Uuid::now_v7())?;
        write!(file, "{}|Groceries\n", Uuid::now_v7())?;

        service.read()?;

        let account = &service.accounts[0];
        let category1 = &service.categories[0];
        let category2 = &service.categories[1];

        assert_eq!(account.name, "Savings");
        assert_eq!(category1.title, "Transport");
        assert_eq!(category2.title, "Groceries");
        Ok(())
    }

    #[test]
    fn create_transaction_picks_first_account() -> crate::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("app.data");
        let mut service = Service::open(&path);

        let account = service.create_account("Account")?;
        let transaction = service.create_transaction()?;

        assert_eq!(transaction.account_id, account.id);
        Ok(())
    }

    #[test]
    fn create_transaction_with_no_accounts() -> crate::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("app.data");
        let mut service = Service::open(&path);

        let _ = service.create_transaction()?;
        Ok(())
    }
}
