use clap::{Parser, Subcommand};
use mukwa::migrator::Migrator;
use rusqlite::Connection;
use std::path::PathBuf;
use tracing::info;

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
}

#[derive(Subcommand)]
enum MigrateCommand {
    /// Create a new migration
    New { name: String },
    /// Apply all migrations
    Up,
}
fn main() -> mukwa::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .init();
    let cli = Cli::parse();
    match cli.command {
        Command::Migrate {
            command,
            migrations_dir,
            path,
        } => match command {
            MigrateCommand::New { name } => {
                let path = mukwa::migrator::create_migration_file(&migrations_dir, &name)?;
                info!("Created migration: {:?}", path)
            }
            MigrateCommand::Up => {
                let mut connection = Connection::open(path)?;
                let mut migrator = Migrator::new();
                migrator.load_from_dir(&migrations_dir)?;
                migrator.migrate(&mut connection)?;
            }
        },
    }

    Ok(())
}
