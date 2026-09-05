// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// Tests to make sure the migrations don't break the user's database.

use mukwa_core::service::{AccountType, Service};
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
