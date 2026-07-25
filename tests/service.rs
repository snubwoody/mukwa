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
use mukwa::service::{CreateBudgetOpts, Service, TransactionType};
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
    let group = service.create_category_group("")?;
    let category = service.create_category("Groceries", group.id)?;
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
fn create_category_group() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let group = service.create_category_group("Wants")?;
    assert_eq!(group.title, "Wants");

    service
        .connection()
        .query_one("SELECT * FROM category_groups", [], |row| {
            let deleted_at: Option<i64> = row.get("deleted_at")?;
            let id: String = row.get("id")?;
            let title: String = row.get("title")?;

            assert!(deleted_at.is_none());
            assert_eq!(title, "Wants");
            assert_eq!(group.id.to_string(), id);
            Ok(())
        })?;
    Ok(())
}

#[test]
fn update_category_group() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let group = service.create_category_group("Wants")?;
    service.update_category_group(group.id, "Needs")?;

    service
        .connection()
        .query_one("SELECT * FROM category_groups", [], |row| {
            let title: String = row.get("title")?;
            assert_eq!(title, "Needs");
            Ok(())
        })?;
    Ok(())
}

#[test]
fn create_budget() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let group = service.create_category_group("")?;
    let category = service.create_category("Groceries", group.id)?;

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
    let group = service.create_category_group("")?;
    let category = service.create_category("Groceries", group.id)?;

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
fn create_expense_fails_with_no_accounts() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let result = service.create_expense().submit();
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_income_fails_with_no_accounts() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let result = service.create_expense().submit();
    assert!(result.is_err());
    Ok(())
}

#[test]
fn create_expense_uses_first_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_expense().submit()?;
    assert_eq!(transaction.sender_id.unwrap(), account.id);
    Ok(())
}

#[test]
fn create_income_uses_first_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_expense().submit()?;
    assert_eq!(transaction.sender_id.unwrap(), account.id);
    Ok(())
}

#[test]
fn total_spent() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;

    let date = date(2020, 12, 14);
    let group = service.create_category_group("")?;
    let category = service.create_category("",group.id)?;
    service
        .create_expense()
        .amount(Money::new(500))
        .date(date)
        .category(category.id)
        .submit()?;
    service
        .create_expense()
        .amount(Money::new(150))
        .date(date)
        .category(category.id)
        .submit()?;

    let total = service.total_spent(category.id, date)?;
    assert_eq!(total, Money::new(650));
    Ok(())
}

#[test]
fn total_spent_in_group() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;

    let date = date(2020, 12, 14);
    let group = service.create_category_group("")?;
    let group2 = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;
    let opts = CreateTransactionOpts {
        date,
        category_id: Some(category.id),
        ..Default::default()
    };
    service.create_transaction(CreateTransactionOpts {
        amount: Money::new(500),
        ..opts.clone()
    })?;

    service.create_transaction(CreateTransactionOpts {
        amount: Money::new(150),
        ..opts
    })?;

    let total = service.total_spent_in_group(group.id, date)?;
    assert_eq!(total, Money::new(650));
    let total = service.total_spent_in_group(group2.id, date)?;
    assert_eq!(total, Money::ZERO);
    Ok(())
}

#[test]
fn total_spent_filters_by_date() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;

    let date = date(2020, 12, 14);
    let group = service.create_category_group("")?;
    let category = service.create_category("",group.id)?;
    service
        .create_expense()
        .amount(Money::new(500))
        .category(category.id)
        .date(jiff::civil::date(2020, 11, 1))
        .submit()?;
    service
        .create_expense()
        .amount(Money::new(150))
        .date(date)
        .category(category.id)
        .submit()?;

    let total = service.total_spent(category.id, date)?;
    assert_eq!(total, Money::new(150));
    Ok(())
}

#[test]
fn account_balance() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let group = service.create_category_group("")?;
    let category = service.create_category("",group.id)?;
    service
        .create_expense()
        .amount(Money::new(500))
        .account(account.id)
        .category(category.id)
        .submit()?;
    service
        .create_income()
        .amount(Money::new(700))
        .account(account.id)
        .submit()?;

    let total = service.account_balance(account.id)?;
    assert_eq!(total, Money::new(200));
    Ok(())
}

#[test]
fn account_balance_negative() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let group = service.create_category_group("")?;
    let category = service.create_category("",group.id)?;
    service
        .create_expense()
        .amount(Money::new(900))
        .account(account.id)
        .category(category.id)
        .submit()?;
    service
        .create_income()
        .amount(Money::new(700))
        .account(account.id)
        .submit()?;

    let total = service.account_balance(account.id)?;
    assert_eq!(total, Money::new(-200));
    Ok(())
}

#[test]
fn account_balance_with_no_transactions() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let group = service.create_category_group("")?;
    service.create_category("",group.id)?;
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(700))
        .submit()?;

    let total = service.account_balance(Uuid::now_v7())?;
    assert_eq!(total, Money::ZERO);
    Ok(())
}

#[test]
fn delete_transaction() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    service.create_account("")?;

    let transaction = service.create_expense().submit()?;
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

    let transaction = service.create_expense().submit()?;
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
fn set_transaction_date() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;

    let transaction = service.create_expense().account(account.id).submit()?;
    let date = date(200, 12, 22);
    let transaction = service.set_transaction_date(transaction.id, date)?;
    assert_eq!(transaction.date, date);
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let transaction_date: String = row.get("transaction_date")?;
            assert_eq!(transaction_date, date.to_string());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn set_transaction_account() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;

    let transaction = service.create_expense().account(account.id).submit()?;
    let transaction = service.set_transaction_account(transaction.id, account2.id)?;
    assert_eq!(transaction.sender_id.unwrap(), account2.id);
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let sender_id: String = row.get("sender_id")?;
            assert_eq!(sender_id, account2.id.to_string());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn set_transaction_outflow() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let transaction = service.create_expense().account(account.id).submit()?;
    let transaction = service.set_transaction_outflow(transaction.id, Money::new(500))?;
    assert_eq!(transaction.amount, Money::new(500));
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let amount: i64 = row.get("amount")?;
            assert_eq!(amount, Money::new(500).inner());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn set_transaction_inflow_for_expense() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;

    let transaction = service.create_expense().account(account.id).submit()?;
    let transaction = service.set_transaction_inflow(transaction.id, Money::new(500))?;
    assert_eq!(transaction.amount, Money::new(500));
    assert_eq!(transaction.transaction_type(), TransactionType::Income);
    assert_eq!(transaction.receiver_id.unwrap(), account.id);
    Ok(())
}

#[test]
fn set_transaction_category() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;

    let category = service.create_category("")?;
    let transaction = service.create_expense().submit()?;
    let transaction = service.set_transaction_category(transaction.id, category.id)?;
    assert_eq!(transaction.category_id.unwrap(), category.id);
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let category_id: String = row.get("category_id")?;
            assert_eq!(category_id, category_id.to_string());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn can_only_set_category_for_expenses() -> mukwa::Result<()> {
    let connection = create_test_db();

    let service = Service::new(connection);
    let account = service.create_account("")?;

    let category = service.create_category("")?;
    let transaction = service
        .create_income()
        .amount(Money::new(300))
        .account(account.id)
        .submit()?;
    let result = service.set_transaction_category(transaction.id, category.id);
    assert!(result.is_err());
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let category_id: Option<String> = row.get("category_id")?;
            assert!(category_id.is_none());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn set_transaction_note() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;

    let transaction = service.create_expense().account(account.id).submit()?;
    let transaction = service.set_transaction_note(transaction.id, "Shoprite")?;
    assert_eq!(transaction.note.unwrap(), "Shoprite");
    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let note: String = row.get("note")?;
            assert_eq!(note, "Shoprite");
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

    let t1 = service.create_expense().submit()?;
    let t2 = service.create_expense().submit()?;
    let t3 = service.create_expense().submit()?;

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
    let group = service.create_category_group("")?;
    let c1 = service.create_category("", group.id)?;
    let c2 = service.create_category("", group.id)?;
    let c3 = service.create_category("", group.id)?;

    let categories = service.fetch_categories()?;
    assert_eq!(categories.len(), 3);
    assert!(categories.contains(&c1));
    assert!(categories.contains(&c2));
    assert!(categories.contains(&c3));
    Ok(())
}

#[test]
fn fetch_category_groups() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let g1 = service.create_category_group("Needs")?;
    let g2 = service.create_category_group("Wants")?;
    let g3 = service.create_category_group("Investments & Savings")?;

    let category_groups = service.fetch_category_groups()?;
    assert_eq!(category_groups.len(), 3);
    assert!(category_groups.contains(&g1));
    assert!(category_groups.contains(&g2));
    assert!(category_groups.contains(&g3));
    Ok(())
}

#[test]
fn fetch_or_init_budgets() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let group = service.create_category_group("")?;
    let c1 = service.create_category("", group.id)?;
    service.create_category("", group.id)?;
    service.create_category("", group.id)?;

    service.create_budget(CreateBudgetOpts {
        category_id: c1.id,
        ..Default::default()
    })?;

    let budgets = service.fetch_or_init_budgets(Zoned::now().date())?;

    assert_eq!(budgets.len(), 3);
    Ok(())
}

#[test]
fn fetch_or_init_budgets_in_different_month() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let group = service.create_category_group("")?;
    let c1 = service.create_category("", group.id)?;

    service.create_budget(CreateBudgetOpts {
        category_id: c1.id,
        month: Some(Zoned::now().date()),
        ..Default::default()
    })?;

    let budgets = service.fetch_or_init_budgets(date(2000, 1, 1))?;

    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].month, 1);
    assert_eq!(budgets[0].year, 2000);
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

#[test]
fn cant_create_duplicate_budgets() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let group = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;
    let opts = CreateBudgetOpts {
        category_id: category.id,
        month: Some(date(2020, 1, 1)),
        amount: Some(Money::ZERO),
    };
    service.create_budget(opts)?;
    let result = service.create_budget(opts);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn min_budget_amount_is_zero() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let group = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;
    let opts = CreateBudgetOpts {
        category_id: category.id,
        month: Some(date(2020, 1, 1)),
        amount: Some(Money::new(-200)),
    };
    let result = service.create_budget(opts);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn set_payee_on_an_expense() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;

    let expense = service.create_expense().account(account.id).submit()?;
    let transfer = service.set_transaction_payee(expense.id, account2.id)?;

    assert_eq!(transfer.transaction_type(), TransactionType::Transfer);
    assert_eq!(transfer.sender_id.unwrap(), account.id);
    assert_eq!(transfer.receiver_id.unwrap(), account2.id);

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let sender_id: String = row.get("sender_id")?;
            let receiver_id: String = row.get("receiver_id")?;
            assert_eq!(sender_id, account.id.to_string());
            assert_eq!(receiver_id, account2.id.to_string());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn set_payee_on_an_income() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;

    let expense = service.create_income().account(account.id).submit()?;
    let transfer = service.set_transaction_payee(expense.id, account2.id)?;

    assert_eq!(transfer.transaction_type(), TransactionType::Transfer);
    assert_eq!(transfer.sender_id.unwrap(), account2.id);
    assert_eq!(transfer.receiver_id.unwrap(), account.id);

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let sender_id: String = row.get("sender_id")?;
            let receiver_id: String = row.get("receiver_id")?;
            assert_eq!(sender_id, account2.id.to_string());
            assert_eq!(receiver_id, account.id.to_string());
            Ok(())
        })?;
    Ok(())
}

#[test]
fn set_payee_sets_category_to_null() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let account = service.create_account("")?;
    let account2 = service.create_account("")?;
    let category = service.create_category("")?;

    let expense = service
        .create_expense()
        .account(account.id)
        .category(category.id)
        .submit()?;
    let transfer = service.set_transaction_payee(expense.id, account2.id)?;

    assert!(transfer.category_id.is_none());

    service
        .connection()
        .query_one("SELECT * FROM transactions", [], |row| {
            let category_id: Option<String> = row.get("category_id")?;
            assert!(category_id.is_none());
            Ok(())
        })?;
    Ok(())
}
