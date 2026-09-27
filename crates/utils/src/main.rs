// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use clap::{Parser, Subcommand};
use mukwa_core::auto_update::{fetch_releases, gen_release_manifest};
use mukwa_core::migrator::Migrator;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Migrate {
        #[command(subcommand)]
        command: MigrateCommand,
        /// The directory containing the migration files
        #[arg(short = 'd', long, default_value = "./migrations")]
        migrations_dir: PathBuf,
        /// The path to the sqlite database file
        #[arg(short, long, default_value = "data.sqlite")]
        path: PathBuf,
    },
    Manifest {
        #[command(subcommand)]
        command: ManifestCommand,
    },
}

#[derive(Subcommand)]
enum ManifestCommand {
    /// Create a release manifest
    Create,
}

#[derive(Subcommand)]
enum MigrateCommand {
    /// Create a new migration
    New { name: String },
    /// Apply all migrations
    Up,
    /// Revert the most recently applied migration
    Rollback,
}

fn run() -> mukwa_core::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Migrate {
            command,
            migrations_dir,
            path,
        } => match command {
            MigrateCommand::New { name } => {
                let path = mukwa_core::migrator::create_migration_file(&migrations_dir, &name)?;
                info!("Created migration: {:?}", path)
            }
            MigrateCommand::Up => {
                let mut connection = Connection::open(path)?;
                let mut migrator = Migrator::new();
                migrator.load_from_dir(&migrations_dir)?;
                migrator.migrate(&mut connection)?;
            }
            MigrateCommand::Rollback => {
                let mut connection = Connection::open(path)?;
                let mut migrator = Migrator::new();
                migrator.load_from_dir(&migrations_dir)?;
                migrator.rollback(&mut connection)?;
            }
        },
        Command::Manifest { command } => match command {
            ManifestCommand::Create => smol::block_on(async { create_manifest().await })?,
        },
    }
    Ok(())
}

async fn create_manifest() -> mukwa_core::Result<()> {
    let releases = fetch_releases().await?;
    let manifest = gen_release_manifest(&releases)?;
    let json = serde_json::to_string_pretty(&manifest).unwrap();
    let path = Path::new("release-manifest.json");
    std::fs::write(&path, &json)?;
    info!("Created release manifest at {}", path.display());
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .init();

    if let Err(err) = run() {
        warn!("{}", err.report());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn gen_release_info() -> mukwa_core::Result<()> {
        smol::block_on(async {
            let release = octocrab::instance()
                .repos("snubwoody", "mukwa")
                .releases()
                .get_latest()
                .await
                .unwrap();
            dbg!(release);
        });
        Ok(())
    }
}
