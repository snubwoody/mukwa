// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// Tests to make sure the migrations don't break the user's database.

use mukwa_core::service::{AccountType, Service, TransactionKind};
use mukwa_core::{Result, migrator::Migrator};
use rusqlite::Connection;
use uuid::Uuid;

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
    let transaction1 = service.create_expense().note("Starting balance").submit()?;
    let transaction2 = service
        .create_expense()
        .note("Not Starting balance")
        .submit()?;
    migrator.migrate(&mut connection)?;

    let transaction1 = service.get_transaction(transaction1.id)?;
    let transaction2 = service.get_transaction(transaction2.id)?;
    assert_eq!(transaction1.kind, TransactionKind::StartingBalance);
    assert_eq!(transaction2.kind, TransactionKind::Regular);
    Ok(())
}
