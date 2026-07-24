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

use crate::Error;
use crate::error::ErrorExt;
use jiff::Zoned;
use rusqlite::Connection;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use std::{
    fs::{self},
    iter::Peekable,
    path::Path,
    str,
};
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
struct Migration {
    version: i64,
    up: String,
    down: String,
}

impl Migration {
    pub fn new(up: &str, down: &str, version: i64) -> Migration {
        Migration {
            version,
            up: up.to_owned(),
            down: down.to_owned(),
        }
    }

    /// Creates a new up migration.
    #[allow(unused)]
    pub fn up(query: &str, version: i64) -> Migration {
        Migration::new(query, "", version)
    }

    /// Creates a new down migration.
    #[allow(unused)]
    pub fn down(query: &str, version: i64) -> Migration {
        Migration::new("", query, version)
    }
}

/// Creates the `schema_migrations` table if it does not exist.
fn create_migrations_table(conn: &Connection) -> crate::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations(version INT PRIMARY KEY NOT NULL)",
        (),
    )?;

    Ok(())
}

#[derive(rust_embed::Embed)]
#[folder = "migrations"]
struct Migrations;

// TODO: Run migrations in a transaction

/// Sql migrator.
pub struct Migrator {
    migrations: Vec<Migration>,
}

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrator {
    pub fn new() -> Migrator {
        Migrator {
            migrations: Vec::new(),
        }
    }

    pub fn load_embedded(&mut self) -> crate::Result<()> {
        for file_name in Migrations::iter() {
            let split = file_name.split("_").collect::<Vec<_>>();
            let version = split[0]
                .parse::<i64>()
                .map_err(|_| Error::new("Failed to parse migration version"))?;
            let file = Migrations::get(&file_name).ok_or(Error::new("Migration not found"))?;
            let data = std::str::from_utf8(&file.data)?;
            let (up, down) = parse_migration(data);
            if down.is_none() {
                return Err(Error::new("Missing down migration"));
            }

            if up.is_none() {
                return Err(Error::new("Missing up migration"));
            }

            let migration = Migration::new(up.unwrap().as_str(), down.unwrap().as_str(), version);
            self.add_migration(migration);
        }

        Ok(())
    }

    /// Returns a list of the applied migration versions.
    pub fn applied_migrations(&self, conn: &Connection) -> crate::Result<Vec<i64>> {
        let mut stmt = conn.prepare_cached("SELECT version FROM schema_migrations")?;
        let mut versions = vec![];
        let rows = stmt.query_map((), |row| row.get::<_, i64>(0))?;

        for row in rows {
            versions.push(row?);
        }
        Ok(versions)
    }

    /// Adds a migration.
    fn add_migration(&mut self, migration: Migration) -> &mut Self {
        self.migrations.push(migration);
        self
    }

    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        let file_name = path
            .as_ref()
            .file_name()
            .ok_or(Error::new("Failed to read file name"))?;
        let split = file_name
            .to_str()
            .ok_or(Error::new("Failed to read file name"))?
            .split("_")
            .collect::<Vec<_>>();
        let version = split[0]
            .parse::<i64>()
            .map_err(|_| Error::new("Failed to parse migration version"))?;

        let buffer = fs::read_to_string(&path)?;

        let (up, down) = parse_migration(&buffer);
        if down.is_none() {
            return Err(Error::new("Missing down migration"));
        }

        if up.is_none() {
            return Err(Error::new("Missing up migration"));
        }

        let migration = Migration::new(up.unwrap().as_str(), down.unwrap().as_str(), version);
        self.add_migration(migration);
        Ok(())
    }

    /// Loads migrations from a directory
    pub fn load_from_dir(&mut self, path: impl AsRef<Path>) -> crate::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            self.load_from_file(entry.path())?;
        }
        Ok(())
    }

    /// Run all the migrations.
    pub fn migrate(&self, conn: &mut Connection) -> crate::Result<()> {
        create_migrations_table(conn)?;
        let applied_migrations = self.applied_migrations(conn)?;

        let mut migrations = self.migrations.clone();
        migrations.sort_by_key(|a| a.version);

        for migration in &migrations {
            if applied_migrations.contains(&migration.version) {
                continue;
            }

            let instant = Instant::now();
            conn.pragma_update(None, "foreign_keys", "OFF")?;

            let tx = conn.transaction()?;
            tx.execute_batch(&migration.up)
                .context(format!("Failed to run migration {}", migration.version))?;
            tx.execute(
                "INSERT INTO schema_migrations(version) VALUES(?)",
                [migration.version],
            )?;
            tx.commit()?;

            conn.pragma_update(None, "foreign_keys", "ON")?;

            let duration = instant.elapsed().as_millis();
            info!("Applied migration {} in {duration}ms", migration.version);
        }

        Ok(())
    }

    /// Reverts the most recently applied migration.
    pub fn rollback(&self, conn: &mut Connection) -> crate::Result<()> {
        if !conn.table_exists(Some("main"), "schema_migrations")? {
            warn!("No migrations to roll back");
        }

        let applied_migrations = self.applied_migrations(conn)?;

        let mut migrations = self.migrations.clone();
        migrations.sort_by_key(|a| a.version);

        if let Some(migration) = migrations.last()
            && applied_migrations.contains(&migration.version)
        {
            let instant = Instant::now();
            conn.pragma_update(None, "foreign_keys", "OFF")?;

            let tx = conn.transaction()?;
            tx.execute_batch(&migration.down)?;

            tx.execute(
                "DELETE FROM schema_migrations WHERE version = ?",
                [migration.version],
            )?;

            tx.commit()?;
            conn.pragma_update(None, "foreign_keys", "ON")?;

            let duration = instant.elapsed().as_millis();
            info!("Reverted migration {} in {duration}ms", migration.version);
        } else {
            warn!("No migrations to roll back")
        }

        Ok(())
    }
}

pub fn create_migration_file(dir: impl AsRef<Path>, name: &str) -> crate::Result<PathBuf> {
    let version = Zoned::now().strftime("%Y%m%d%H%M%S").to_string();
    let file_name = format!("{version}_{name}");
    let mut buffer = String::new();
    buffer += "--migrate:up\n\n";
    buffer += "--migrate:down\n";

    fs::create_dir_all(&dir)?;
    let mut path = dir.as_ref().join(file_name);
    path.set_extension("sql");
    let mut file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)?;
    let _ = file.write(buffer.as_bytes())?;

    Ok(path)
}

/// Parses a sql migration and returns the up and down migrations.
fn parse_migration(sql: &str) -> (Option<String>, Option<String>) {
    let mut lines = sql.lines().peekable();
    let mut up = None;
    let mut down = None;

    while let Some(line) = lines.next() {
        let stripped = line.replace(" ", "");
        if stripped == "--migrate:up" {
            up = Some(seek_block(&mut lines));
        }
        if stripped == "--migrate:down" {
            down = Some(seek_block(&mut lines));
        }
    }

    (up, down)
}

fn seek_block(iter: &mut Peekable<str::Lines>) -> String {
    let mut block = String::new();
    while let Some(line) = iter.peek() {
        let stripped = line.replace(" ", "");
        if stripped == "--migrate:up" || stripped == "--migrate:down" {
            break;
        }

        block.push_str(iter.next().unwrap());
        block.push('\n');
    }

    block
}

#[cfg(test)]
mod tests {
    use rusqlite::ErrorCode;

    use super::*;

    #[test]
    fn load_migration_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("000_migration.sql");
        let sql = "--migrate:up\nCREATE TABLE schemas(name TEXT PRIMARY KEY);\n--migrate:down\nDROP TABLE schemas;";
        fs::write(&path, sql).unwrap();

        let mut migrator = Migrator::new();
        migrator.load_from_file(path).unwrap();
        let migration = &migrator.migrations[0];
        assert_eq!(
            migration.up,
            "CREATE TABLE schemas(name TEXT PRIMARY KEY);\n"
        );
        assert_eq!(migration.down, "DROP TABLE schemas;\n")
    }

    #[test]
    fn load_migrations_from_dir() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("2024_migration.sql");
        let p2 = dir.path().join("240_migration.sql");
        let sql1 = "--migrate:up\nCREATE TABLE schemas(name TEXT PRIMARY KEY);\n--migrate:down\nDROP TABLE schemas;";
        let sql2 = "--migrate:up\nCREATE TABLE users(id TEXT PRIMARY KEY);\n--migrate:down\nDROP TABLE users;";
        fs::write(&p1, sql1).unwrap();
        fs::write(&p2, sql2).unwrap();

        let mut migrator = Migrator::new();
        migrator.load_from_dir(dir.path()).unwrap();
        assert_eq!(migrator.migrations.len(), 2);
    }

    #[test]
    fn parse_file_name_as_version() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("2026_migration.sql");
        let sql = "--migrate:up\nCREATE TABLE schemas(name TEXT PRIMARY KEY);\n--migrate:down\nDROP TABLE schemas;";
        fs::write(&path, sql).unwrap();

        let mut migrator = Migrator::new();
        migrator.load_from_file(path).unwrap();
        let migration = &migrator.migrations[0];
        assert_eq!(migration.version, 2026);
    }

    #[test]
    fn parse_up_migration() {
        let sql = "--migrate:up\nCREATE TABLE schemas(name TEXT PRIMARY KEY);\n--migrate:down\nDROP TABLE schemas;";
        let (up, down) = parse_migration(sql);
        assert_eq!(
            up.unwrap(),
            "CREATE TABLE schemas(name TEXT PRIMARY KEY);\n"
        );
        assert_eq!(down.unwrap(), "DROP TABLE schemas;\n");
    }

    #[test]
    fn run_migration() -> crate::Result<()> {
        let mut connection = Connection::open_in_memory()?;
        let mut migrator = Migrator::new();
        let migration = Migration::up("CREATE TABLE users(id TEXT PRIMARY KEY)", 0);
        migrator.add_migration(migration);
        migrator.migrate(&mut connection)?;

        let row = connection.query_row(
            "INSERT INTO users(id) VALUES ('Player 1') RETURNING *",
            (),
            |row| row.get::<_, String>(0),
        )?;

        assert_eq!(row, "Player 1");
        Ok(())
    }

    #[test]
    fn rollback_migration() -> crate::Result<()> {
        let mut connection = Connection::open_in_memory()?;
        let mut migrator = Migrator::new();
        let migration = Migration::new(
            "CREATE TABLE users(id TEXT PRIMARY KEY)",
            "DROP TABLE users",
            0,
        );
        migrator.add_migration(migration);
        migrator.migrate(&mut connection)?;
        migrator.rollback(&mut connection)?;

        let table_exists = connection.table_exists(Some("main"), "users")?;
        assert!(!table_exists);
        Ok(())
    }

    #[test]
    fn skip_applied_migrations() {
        let mut connection = Connection::open_in_memory().unwrap();
        create_migrations_table(&connection).unwrap();
        connection
            .execute("INSERT INTO schema_migrations(version) VALUES (1010)", ())
            .expect("Failed to run query");
        let mut migrator = Migrator::new();
        let m1 = Migration::up("CREATE TABLE users(id TEXT PRIMARY KEY)", 1);
        let m2 = Migration::up("CREATE TABLE organisations(id TEXT PRIMARY KEY)", 1010);
        migrator.add_migration(m1);
        migrator.add_migration(m2);
        migrator.migrate(&mut connection).unwrap();

        connection
            .execute("INSERT INTO users(id) VALUES ('Player 1')", ())
            .unwrap();
        let result = connection.execute("INSERT INTO organisations(id) VALUES ('Player 1')", ());
        assert!(result.is_err());
    }

    #[test]
    fn update_migrations_table_after_migrating() {
        let mut connection = Connection::open_in_memory().unwrap();
        let mut migrator = Migrator::new();
        let migration = Migration::up("CREATE TABLE users(id TEXT PRIMARY KEY)", 100);
        migrator.add_migration(migration);
        migrator.migrate(&mut connection).unwrap();

        let version = connection
            .query_row("SELECT * FROM schema_migrations", (), |row| {
                row.get::<_, i64>("version")
            })
            .unwrap();
        assert_eq!(version, 100);
    }

    #[test]
    fn init_migrations_table() {
        let connection = Connection::open_in_memory().unwrap();
        create_migrations_table(&connection).unwrap();

        connection
            .execute(
                "INSERT INTO schema_migrations(version) VALUES($1)",
                [29492424],
            )
            .unwrap();
    }

    #[test]
    fn get_applied_migrations() {
        let connection = Connection::open_in_memory().unwrap();
        create_migrations_table(&connection).unwrap();

        connection
            .execute(
                "INSERT INTO schema_migrations(version) VALUES (5),(10),(15),(20)",
                (),
            )
            .unwrap();
        let migrator = Migrator::new();
        let migrations = migrator.applied_migrations(&connection).unwrap();
        assert_eq!(migrations, [5, 10, 15, 20]);
    }

    #[test]
    fn schema_migrations_unique_version() {
        let connection = Connection::open_in_memory().unwrap();
        create_migrations_table(&connection).unwrap();

        connection
            .execute("INSERT INTO schema_migrations(version) VALUES($1)", [0])
            .unwrap();

        let result = connection.execute("INSERT INTO schema_migrations(version) VALUES($1)", [0]);
        let err = result.expect_err("Expected duplicate insert to fail");
        match err {
            rusqlite::Error::SqliteFailure(a, _) => {
                assert!(matches!(a.code, ErrorCode::ConstraintViolation))
            }
            _ => panic!("Invalid error"),
        }
    }

    #[test]
    fn schema_migrations_not_null() {
        let connection = Connection::open_in_memory().unwrap();
        create_migrations_table(&connection).unwrap();
        let result = connection.execute("INSERT INTO schema_migrations(version) VALUES(null)", ());
        let err = result.expect_err("Expected null insert to fail");
        match err {
            rusqlite::Error::SqliteFailure(a, _) => {
                assert!(matches!(a.code, ErrorCode::ConstraintViolation))
            }
            _ => panic!("Invalid error"),
        }
    }

    #[test]
    fn init_migrations_table_already_exists() {
        let connection = Connection::open_in_memory().unwrap();
        create_migrations_table(&connection).unwrap();
        create_migrations_table(&connection).unwrap();
        create_migrations_table(&connection).unwrap();
    }
}
