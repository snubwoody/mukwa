// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use clap::{Parser, Subcommand};
use mukwa_core::migrator::Migrator;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
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
    let config = std::fs::read_to_string("crates/mukwa/ui/iconlist")?;
    let icon_list = config.split('\n').collect::<HashSet<&str>>();
    let dest_path = PathBuf::from("crates/mukwa/ui/icons.slint");

    let url =
        "https://github.com/lucide-icons/lucide/releases/download/1.52.0/lucide-icons-1.52.0.zip";
    info!("Fetching icons from {url}");
    let mut response = ureq::get(url).call().unwrap();
    let mut buffer = vec![];
    response.body_mut().as_reader().read_to_end(&mut buffer)?;

    let mut archive = ZipArchive::new(Cursor::new(buffer)).unwrap();
    info!("Parsing icons...");

    let mut dest = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&dest_path)?;
    dest.write_all(b"// SPDX-License-Identifier: GPL-3.0-or-later\n")?;
    dest.write_all(b"// Copyright (C) 2026 Wakunguma Kalimukwa\n")?;
    dest.write_all(b"\n")?;

    // TODO: add icon component that they all inherit from
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
        let icon = svg_to_icon(&data?, &kebab_to_pascal(&name))?;
        dest.write_all(format!("{icon}\n").as_bytes())?;
    }

    std::process::Command::new("slint-lsp")
        .args(["format", "--inline"])
        .arg(&dest_path)
        .output()?;
    info!("Generated icons at {}", dest_path.display());
    Ok(())
}

/// Converts a kebab-case string to a PascalCase string.
fn kebab_to_pascal(value: &str) -> String {
    let mut s = String::new();
    for word in value.split('-') {
        let mut chars = word.chars();
        if let Some(c) = chars.next() {
            s.push_str(c.to_uppercase().to_string().as_str());
        }
        for c in chars {
            s.push(c)
        }
    }
    s
}

fn svg_to_icon(data: &[u8], name: &str) -> mukwa_core::Result<String> {
    use std::fmt::Write;
    let content = usvg::Tree::from_data(data, &Default::default()).unwrap();
    let root = content.root();
    let mut icon = String::new();
    writeln!(icon, "export component {name} inherits Rectangle {{").unwrap();
    writeln!(
        icon,
        "in-out property <length> size: 24px;\nwidth: size;\nheight: size;"
    )
    .unwrap();
    writeln!(icon, "in-out property <color> stroke: black;").unwrap();
    writeln!(icon, "in-out property <length> stroke-width: 1px;").unwrap();
    for node in root.children() {
        match node {
            Node::Path(path) => {
                // TODO: not every path has fill
                let commands = path_segments_to_string(path.data().segments()).unwrap();
                let mut component = String::new();
                write!(component, "Path {{\ncommands:\"{commands}\";\n").unwrap();
                writeln!(component, "stroke: parent.stroke;").unwrap();
                writeln!(component, "stroke-width: parent.stroke-width;").unwrap();
                writeln!(component, "viewbox-width: 24;").unwrap();
                writeln!(component, "viewbox-height: 24;").unwrap();
                writeln!(component, "viewbox-x: 0;").unwrap();
                writeln!(component, "viewbox-y: 0;").unwrap();
                writeln!(component, "stroke-line-cap: round;").unwrap();
                writeln!(component, "stroke-line-join: round;").unwrap();
                write!(component, "}}").unwrap();
                write!(icon, "\n{component}\n").unwrap();
            }
            _ => {
                panic!("Unsupported element")
            }
        }
    }
    write!(icon, "}}").unwrap();
    Ok(icon)
}

fn path_segments_to_string(segments: PathSegmentsIter) -> Result<String, std::fmt::Error> {
    use std::fmt::Write;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn convert_kebab_to_pascal_case() {
        let test_case = |value: &str, expected: &str| {
            let pascal_string = kebab_to_pascal(value);
            assert_eq!(pascal_string, expected);
        };

        test_case("scan-line", "ScanLine");
        test_case("my-little-pony", "MyLittlePony");
        test_case("noend-", "Noend");
        test_case("-nostart", "Nostart");
    }
}
