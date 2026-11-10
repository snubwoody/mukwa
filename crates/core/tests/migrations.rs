// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// Tests to make sure the migrations don't break the user's database.

use jiff::Zoned;
use mukwa_core::service::{AccountType, Service, TransactionKind};
use mukwa_core::{Error, Money, Result, migrator::Migrator};
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

fn create_expense(service: &Service, note: &str) -> crate::Result<()> {
    let accounts = service.fetch_accounts()?;
    if accounts.is_empty() {
        return Err(Error::new("Cannot create a transaction without an account"));
    }

    let account_id = accounts[0].id;
    let amount = Money::new(0);
    let date = Zoned::now().date();
    let connection = service.connection();
    let sql = "INSERT INTO transactions(id,transaction_date,sender_id,amount,note) \
            VALUES(?1,?2,?3,?4,?5)";
    let mut stmt = connection.prepare_cached(sql)?;
    let params = params![
        Uuid::now_v7().to_string(),
        date.to_string(),
        account_id.to_string(),
        amount.inner(),
        note
    ];
    stmt.execute(params)?;
    Ok(())
}

#[test]
fn old_accounts_default_to_cash_accounts() -> Result<()> {
    // All accounts that were added before account types
    // were supported will become Cash accounts.

    let mut connection = Connection::open_in_memory()?;
    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate_until(&mut connection, 20260815192720)?;

    connection.execute(
        "INSERT INTO accounts(id,name) VALUES(?,'')",
        [Uuid::now_v7().to_string()],
    )?;
    migrator.migrate(&mut connection)?;

    connection.query_one("SELECT * FROM accounts", [], |row| {
        let account_type_id: i64 = row.get("account_type_id")?;
        assert_eq!(account_type_id, 1);
        Ok(())
    })?;

    let service = Service::new(connection);
    let accounts = service.fetch_accounts()?;
    assert_eq!(accounts[0].account_type, AccountType::Cash);
    Ok(())
}

#[test]
fn add_credit_payments_meta_group() -> Result<()> {
    let mut connection = Connection::open_in_memory()?;

    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate_until(&mut connection, 20260819184648)?;
    migrator.migrate(&mut connection)?;

    let service = Service::new(connection);
    let groups = service.fetch_category_groups()?;
    let group = &groups[0];

    assert_eq!(group.title, "Credit payments");
    assert!(group.is_meta);
    Ok(())
}

#[test]
fn transactions_default_to_regular() -> Result<()> {
    let mut connection = Connection::open_in_memory()?;
    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate(&mut connection)?;
    let service = Service::new(connection);
    service.create_account("", AccountType::Cash)?;
    let transaction = service.create_expense().submit()?;
    assert_eq!(transaction.kind, TransactionKind::Regular);
    Ok(())
}

#[test]
fn mark_starting_balance_transactions() -> Result<()> {
    let temp = tempfile::NamedTempFile::new()?;
    let mut connection = Connection::open(temp.path())?;

    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate_until(&mut connection, 20261023100557)?;
    let service = Service::new(Connection::open(temp.path())?);
    service.create_account("", AccountType::Cash)?;
    create_expense(&service, "Starting balance")?;
    create_expense(&service, "Not Starting balance")?;
    migrator.migrate(&mut connection)?;

    let transaction_kinds = {
        let connection = service.connection();
        let mut stmt = connection.prepare("SELECT id,kind FROM transactions")?;
        let mut transaction_kinds: Vec<_> = stmt
            .query_and_then([], |row| {
                let id = Uuid::from_str(row.get::<_, String>("id")?.as_str())?;
                let kind = TransactionKind::try_from(row.get::<_, i64>("kind")?)?;
                Ok::<_, Error>((id, kind))
            })?
            .map(|r| r.unwrap())
            .collect();
        transaction_kinds.sort_by(|a, b| a.0.cmp(&b.0));
        transaction_kinds
    };

    assert_eq!(transaction_kinds[0].1, TransactionKind::StartingBalance);
    assert_eq!(transaction_kinds[1].1, TransactionKind::Regular);
    Ok(())
}

#[test]
fn old_transactions_are_not_audited() -> Result<()> {
    let temp = tempfile::NamedTempFile::new()?;
    let mut connection = Connection::open(temp.path())?;

    let mut migrator = Migrator::new();
    migrator.load_embedded()?;
    migrator.migrate_until(&mut connection, 20261023104439)?;

    let service = Service::new(Connection::open(temp.path())?);
    service.create_account("", AccountType::Cash)?;

    create_expense(&service, "")?;
    migrator.migrate(&mut connection)?;

    for transaction in service.fetch_transactions()? {
        assert!(!transaction.audited);
    }
    Ok(())
}
