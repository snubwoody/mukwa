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

use jiff::Zoned;
use jiff::civil::date;
use mukwa::service::{CreateBudgetOpts, CreateTransactionOpts, Service, UpdateTransactionOpts};
use mukwa::{Money, create_test_db};
use uuid::Uuid;

#[test]
fn create_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("My account")?;
    assert_eq!(account.name, "My account");

    let connection = service.connection();
    let name = connection.query_one(
        "SELECT name FROM accounts WHERE id=?",
        [account.id.to_string()],
        |row| row.get::<_, String>(0),
    )?;
    assert_eq!(name, account.name);
    Ok(())
}

#[test]
fn create_category() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let category = service.create_category("Groceries")?;
    assert_eq!(category.title, "Groceries");

    service
        .connection()
        .query_one("SELECT * FROM categories", [], |row| {
            let deleted_at: Option<i64> = row.get("deleted_at")?;
            let title: String = row.get("title")?;
            assert!(deleted_at.is_none());
            assert_eq!(title, "Groceries");
            Ok(())
        })?;
    Ok(())
}

#[test]
fn create_budget() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let category = service.create_category("Groceries")?;

    let opts = CreateBudgetOpts {
        category_id: category.id,
        month: Some(date(1990, 12, 31)),
        amount: Some(Money::new(200)),
    };
    let budget = service.create_budget(opts)?;

    assert_eq!(budget.month, 12);
    assert_eq!(budget.year, 1990);
    assert_eq!(budget.amount, Money::new(200));
    assert_eq!(budget.category_id, category.id);

    service
        .connection()
        .query_one("SELECT * FROM budgets", [], |row| {
            let category_id: String = row.get("category_id")?;
            let amount: i64 = row.get("amount")?;
            let year: i64 = row.get("year")?;
            let month: i64 = row.get("month")?;
            assert_eq!(category_id, category.id.to_string());
            assert_eq!(amount, Money::new(200).inner());
            assert_eq!(year, 1990);
            assert_eq!(month, 12);
            Ok(())
        })?;
    Ok(())
}

#[test]
fn fetch_budgets_by_month() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let category = service.create_category("Groceries")?;

    let opts = CreateBudgetOpts {
        category_id: category.id,
        month: Some(date(1990, 12, 31)),
        amount: Some(Money::new(200)),
    };
    let budget1 = service.create_budget(opts)?;
    let budget2 = service.create_budget(CreateBudgetOpts {
        category_id: category.id,
        ..Default::default()
    })?;

    let budgets = service.fetch_budgets_by_month(Zoned::now().date())?;
    assert!(budgets.contains(&budget2));
    assert!(!budgets.contains(&budget1));
    Ok(())
}

#[test]
fn create_transaction_fails_with_no_accounts() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let result = service.create_transaction(Default::default());
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_transaction_selects_first_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_transaction(Default::default())?;
    assert_eq!(transaction.sender_id.unwrap(), account.id);
    Ok(())
}

#[test]
fn create_transaction() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;

    let opts = CreateTransactionOpts {
        amount: Money::new(200),
        date: date(2020, 10, 20),
        ..Default::default()
    };
    let transaction = service.create_transaction(opts)?;
    assert_eq!(transaction.sender_id.unwrap(), account.id);
    assert_eq!(transaction.category_id, None);
    assert_eq!(transaction.amount, Money::new(200));
    assert_eq!(transaction.date, date(2020, 10, 20));

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let amount: i64 = row.get("amount")?;
            let date: String = row.get("transaction_date")?;
            assert_eq!(amount, Money::new(200).inner());
            assert_eq!(date, "2020-10-20");
            Ok(())
        })?;

    Ok(())
}

#[test]
fn total_spent() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;

    let date = date(2020, 12, 14);
    let category = service.create_category("")?;
    let opts = CreateTransactionOpts {
        date,
        category_id: Some(category.id),
        ..Default::default()
    };
    service.create_transaction(CreateTransactionOpts {
        amount: Money::new(500),
        ..opts
    })?;

    service.create_transaction(CreateTransactionOpts {
        amount: Money::new(150),
        ..opts
    })?;

    let total = service.total_spent(category.id, date)?;
    assert_eq!(total, Money::new(650));
    Ok(())
}

#[test]
fn account_balance() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let category = service.create_category("")?;
    let opts = CreateTransactionOpts {
        account_id: Some(account.id),
        category_id: Some(category.id),
        ..Default::default()
    };
    service.create_transaction(CreateTransactionOpts {
        amount: Money::new(500),
        ..opts
    })?;
    service.create_income(Money::new(700), account.id)?;

    let total = service.account_balance(account.id)?;
    assert_eq!(total, Money::new(200));
    Ok(())
}

#[test]
fn account_balance_negative() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let category = service.create_category("")?;
    let opts = CreateTransactionOpts {
        account_id: Some(account.id),
        category_id: Some(category.id),
        ..Default::default()
    };
    service.create_transaction(CreateTransactionOpts {
        amount: Money::new(900),
        ..opts
    })?;
    service.create_income(Money::new(700), account.id)?;

    let total = service.account_balance(account.id)?;
    assert_eq!(total, Money::new(-200));
    Ok(())
}

#[test]
fn account_balance_with_no_transactions() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    service.create_category("")?;
    service.create_income(Money::new(700), account.id)?;

    let total = service.account_balance(Uuid::now_v7())?;
    assert_eq!(total, Money::ZERO);
    Ok(())
}

#[test]
fn delete_transaction() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    service.create_account("")?;

    let transaction = service.create_transaction(Default::default())?;
    service.delete_transaction(transaction.id)?;
    let transactions = service.fetch_transactions()?;
    assert!(transactions.is_empty());

    Ok(())
}

#[test]
fn duplicate_transaction() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    service.create_account("")?;

    let transaction = service.create_transaction(Default::default())?;
    service.duplicate_transaction(transaction.id)?;
    let transactions = service.fetch_transactions()?;
    assert_eq!(transactions.len(), 2);
    assert_eq!(transactions[0].date, transactions[1].date);
    assert_eq!(transactions[0].amount, transactions[1].amount);
    assert_eq!(
        transactions[0].sender_id.unwrap(),
        transactions[1].sender_id.unwrap()
    );
    assert_eq!(transactions[0].category_id, transactions[1].category_id);

    Ok(())
}

#[test]
fn update_expense_amount() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;

    let create_opts = CreateTransactionOpts {
        account_id: Some(account.id),
        ..Default::default()
    };

    let transaction = service.create_transaction(create_opts)?;
    let update_opts = UpdateTransactionOpts {
        id: transaction.id,
        amount: Some(Money::new(500)),
        ..Default::default()
    };

    let transaction = service.update_transaction(update_opts)?;
    assert_eq!(transaction.amount, Money::new(500));
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let amount: i64 = row.get("amount")?;
            let sender_id: String = row.get("sender_id")?;
            let receiver_id: Option<String> = row.get("receiver_id")?;
            assert_eq!(amount, Money::new(500).inner());
            assert_eq!(sender_id, account.id.to_string());
            assert!(receiver_id.is_none());
            Ok(())
        })?;

    Ok(())
}

#[test]
fn update_expense_account() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;

    let create_opts = CreateTransactionOpts {
        account_id: Some(account.id),
        ..Default::default()
    };

    let transaction = service.create_transaction(create_opts)?;
    let update_opts = UpdateTransactionOpts {
        id: transaction.id,
        sender_id: Some(account2.id),
        ..Default::default()
    };

    let transaction = service.update_transaction(update_opts)?;
    assert_eq!(transaction.sender_id, Some(account2.id));
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let sender_id: String = row.get("sender_id")?;
            let receiver_id: Option<String> = row.get("receiver_id")?;
            assert_eq!(sender_id, account2.id.to_string());
            assert!(receiver_id.is_none());
            Ok(())
        })?;

    Ok(())
}

#[test]
fn update_transaction_date() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;

    let create_opts = CreateTransactionOpts {
        account_id: Some(account.id),
        ..Default::default()
    };

    let transaction = service.create_transaction(create_opts)?;
    let update_opts = UpdateTransactionOpts {
        id: transaction.id,
        date: Some(date(1990, 1, 1)),
        ..Default::default()
    };

    let transaction = service.update_transaction(update_opts)?;
    assert_eq!(transaction.date, date(1990, 1, 1));
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let date: String = row.get("transaction_date")?;
            assert_eq!(date, "1990-01-01");
            Ok(())
        })?;

    Ok(())
}

#[test]
fn convert_expense_to_income() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;

    let create_opts = CreateTransactionOpts {
        account_id: Some(account.id),
        ..Default::default()
    };

    let transaction = service.create_transaction(create_opts)?;
    let update_opts = UpdateTransactionOpts {
        id: transaction.id,
        amount: Some(Money::new(500)),
        receiver_id: Some(account2.id),
        date: Some(date(1990, 1, 1)),
        ..Default::default()
    };

    service.update_transaction(update_opts)?;

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let amount: i64 = row.get("amount")?;
            let date: String = row.get("transaction_date")?;
            let sender_id: Option<String> = row.get("sender_id")?;
            let receiver_id: Option<String> = row.get("receiver_id")?;
            assert_eq!(amount, Money::new(500).inner());
            assert_eq!(date, "1990-01-01");
            assert_eq!(receiver_id, Some(account2.id.to_string()));
            assert!(sender_id.is_none());
            Ok(())
        })?;

    Ok(())
}

#[test]
fn fetch_accounts() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let a1 = service.create_account("My account")?;
    let a2 = service.create_account("My account")?;

    let accounts = service.fetch_accounts()?;
    assert_eq!(accounts.len(), 2);
    assert!(accounts.contains(&a1));
    assert!(accounts.contains(&a2));
    Ok(())
}

#[test]
fn fetch_transactions() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    service.create_account("My account")?;
    let t1 = service.create_transaction(Default::default())?;
    let t2 = service.create_transaction(Default::default())?;
    let t3 = service.create_transaction(Default::default())?;

    let transactions = service.fetch_transactions()?;
    assert_eq!(transactions.len(), 3);
    assert!(transactions.contains(&t1));
    assert!(transactions.contains(&t2));
    assert!(transactions.contains(&t3));
    Ok(())
}

#[test]
fn fetch_categories() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let c1 = service.create_category("")?;
    let c2 = service.create_category("")?;
    let c3 = service.create_category("")?;

    let categories = service.fetch_categories()?;
    assert_eq!(categories.len(), 3);
    assert!(categories.contains(&c1));
    assert!(categories.contains(&c2));
    assert!(categories.contains(&c3));
    Ok(())
}

#[test]
fn fetch_empty_accounts() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let accounts = service.fetch_accounts()?;
    assert!(accounts.is_empty());
    Ok(())
}
