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

// Tests to make sure the migrations don't break the user's database.

use mukwa::service::{AccountType, Service};
use mukwa::{Result, migrator::Migrator};
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
