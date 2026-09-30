// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use clap::{Parser, Subcommand};
use mukwa_core::Error;
use mukwa_core::migrator::Migrator;
use rusqlite::Connection;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tracing::{error, info};

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
    CheckFormat {
        /// Path to .slint files
        files: Vec<PathBuf>,
    },
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
        Command::CheckFormat { files } => {
            check_slint_fmt(&files)?;
        }
    }
    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .init();

    if let Err(err) = run() {
        error!("{}", err.report());
        std::process::exit(1)
    }
}

/// Returns an error if the `.slint` files are not formatted.
///
/// The Slint LSP has no `format --check` command, so to emulate that this function
/// hashes the contents of `files`, runs the `slint-lsp format` command against each of the files and
/// compares the two hashes.
fn check_slint_fmt(files: &[PathBuf]) -> mukwa_core::Result<()> {
    let mut contents = Vec::new();
    for path in files {
        if !(path.extension().unwrap().to_str().unwrap() == "slint") {
            return Err(Error::new(&format!(
                "Unexpected file type: {}",
                path.display()
            )));
        }
        let mut file = File::open(path)?;
        file.read_to_end(&mut contents)?;
    }

    let output = std::process::Command::new("slint-lsp")
        .arg("format")
        .args(files)
        .output()
        .expect("Failed to run process");

    if !output.status.success() {
        let err = String::from_utf8(output.stderr).unwrap();
        return Err(Error::new(&err));
    }

    if contents != output.stdout {
        return Err(Error::new("Slint files are not formatted"));
    }
    Ok(())
}
