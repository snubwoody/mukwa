use crate::{ui, Error};
use slint::SharedString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::str::Lines;
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

pub struct Service {
    path: PathBuf,
    accounts: Vec<Account>,
    categories: Vec<Category>,
}

// TODO: add backup
// TODO: parser struct
impl Service {
    pub fn open(path: impl AsRef<Path>) -> Service {
        Service {
            path: path.as_ref().to_path_buf(),
            accounts: Vec::new(),
            categories: Vec::new(),
        }
    }

    pub fn accounts(&self) -> &[Account] {
        self.accounts.as_ref()
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

    pub fn read(&mut self) -> crate::Result<()> {
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

    fn is_section_header(&self, value: &str) -> bool {
        match value {
            "[Accounts]" | "[Categories]" | "[Transactions]" => true,
            _ => false,
        }
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
            .open(&self.path)?;

        // TODO: maybe store strings as ""
        write!(file, "[Accounts]\n")?;
        for account in &self.accounts {
            write!(file, "{}|{}\n", account.id, account.name)?;
        }

        write!(file, "[Categories]\n")?;
        for category in &self.categories {
            write!(file, "{}|{}\n", category.id, category.title)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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
}
