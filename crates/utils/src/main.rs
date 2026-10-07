// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use clap::{Parser, Subcommand};
use mukwa_core::migrator::Migrator;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fmt::Write;
use std::fs;
use std::io::{BufReader, Cursor, Read};
use std::path::PathBuf;
use tracing::{info, warn};
use usvg::Node;
use usvg::tiny_skia_path::{PathSegment, PathSegmentsIter};
use zip::ZipArchive;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    GenerateIcons,
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
        Command::GenerateIcons => generate_icons()?,
    }
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

fn generate_icons() -> mukwa_core::Result<()> {
    let config = fs::read_to_string("crates/mukwa/ui/iconlist")?;
    let icon_list = config.split('\n').collect::<HashSet<&str>>();
    let dest_path = PathBuf::from("crates/mukwa/ui/icons.slint");

    let url =
        "https://github.com/lucide-icons/lucide/releases/download/1.52.0/lucide-icons-1.52.0.zip";
    info!("Fetching icons from {url}");
    let mut response = ureq::get(url).call().unwrap();
    let mut buffer = vec![];
    response.body_mut().as_reader().read_to_end(&mut buffer)?;

    let mut archive = ZipArchive::new(Cursor::new(buffer)).unwrap();

    let mut buffer = String::new();
    writeln!(buffer, "// SPDX-License-Identifier: GPL-3.0-or-later")?;
    writeln!(buffer, "// Copyright (C) 2026 Wakunguma Kalimukwa")?;
    writeln!(buffer, "\n// Auto generated file, do not edit\n")?;

    writeln!(buffer, "export struct IconData {{\npaths: [string],}}\n")?;

    writeln!(buffer, "export component Icon {{")?;
    writeln!(buffer, "in-out property <length> size: 16px;")?;
    writeln!(buffer, "in-out property <color> stroke: black;")?;
    writeln!(buffer, "in-out property <length> stroke-width: 1px;")?;
    writeln!(buffer, "in-out property <IconData> icon;")?;
    writeln!(buffer, "width: self.size;")?;
    writeln!(buffer, "height: self.size;")?;

    writeln!(buffer, "\nfor path in icon.paths: Path {{")?;
    writeln!(buffer, "stroke: parent.stroke;")?;
    writeln!(buffer, "stroke-width: parent.stroke-width;")?;
    writeln!(buffer, "stroke-line-cap: round;")?;
    writeln!(buffer, "stroke-line-join: round;")?;
    writeln!(buffer, "commands: path;")?;
    writeln!(buffer, "viewbox-x: 0;")?;
    writeln!(buffer, "viewbox-y: 0;")?;
    writeln!(buffer, "viewbox-width: 24;")?;
    writeln!(buffer, "viewbox-height: 24;")?;
    writeln!(buffer, "}}")?;

    writeln!(buffer, "}}")?;

    writeln!(buffer, "export global Icons {{")?;

    info!("Parsing icons...");
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        if !file.name().ends_with(".svg") {
            continue;
        }

        let name = file.name().replace(".svg", "").replace("icons/", "");
        if !icon_list.contains(name.as_str()) {
            continue;
        }

        let reader = BufReader::new(file);
        let data: std::io::Result<Vec<u8>> = reader.bytes().collect();
        let icon = svg_to_icon(&data?, &name)?;
        write!(buffer, "{icon}")?;
    }

    writeln!(buffer, "}}")?;
    fs::write(&dest_path, buffer.as_bytes())?;
    std::process::Command::new("slint-lsp")
        .args(["format", "--inline"])
        .arg(&dest_path)
        .output()?;
    info!("Generated icons at {}", dest_path.display());
    Ok(())
}

fn svg_to_icon(data: &[u8], name: &str) -> mukwa_core::Result<String> {
    let content = usvg::Tree::from_data(data, &Default::default()).unwrap();
    let root = content.root();
    let mut icon = String::new();
    writeln!(icon, "out property <IconData> {name}: {{")?;
    writeln!(icon, "paths: [")?;
    for node in root.children() {
        match node {
            Node::Path(path) => {
                // TODO: not every path has fill
                let commands = path_segments_to_string(path.data().segments())?;
                writeln!(icon, "\"{commands}\",")?;
            }
            _ => {
                panic!("Unsupported element")
            }
        }
    }
    writeln!(icon, "],")?;
    write!(icon, "}};")?;
    Ok(icon)
}

fn path_segments_to_string(segments: PathSegmentsIter) -> Result<String, std::fmt::Error> {
    let mut s = String::new();
    for segment in segments {
        match segment {
            PathSegment::MoveTo(p) => write!(s, "M {} {} ", p.x, p.y)?,
            PathSegment::LineTo(p) => write!(s, "L {} {} ", p.x, p.y)?,
            PathSegment::QuadTo(p0, p1) => write!(s, "Q {} {} {} {} ", p0.x, p0.y, p1.x, p1.y)?,
            PathSegment::CubicTo(p0, p1, p2) => write!(
                s,
                "C {} {} {} {} {} {} ",
                p0.x, p0.y, p1.x, p1.y, p2.x, p2.y
            )?,
            PathSegment::Close => write!(s, "Z ")?,
        }
    }
    Ok(s)
}
