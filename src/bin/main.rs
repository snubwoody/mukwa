// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
#[cfg(debug_assertions)]
use std::path::PathBuf;

use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    #[cfg(debug_assertions)]
    let log_dir = PathBuf::from(".mukwa/logs");
    #[cfg(not(debug_assertions))]
    let log_dir = mukwa::log_dir();

    fs::create_dir_all(&log_dir).expect("Failed to create directory");

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mukwa")
        .max_log_files(7)
        .filename_suffix("log")
        .build(log_dir)
        .expect("Failed to setup logging");

    // Keep guard in scope
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let std_io_layer = fmt::layer().with_writer(std::io::stdout);

    let file_layer = fmt::layer()
        // .pretty()
        .with_file(false)
        .with_line_number(false)
        .with_writer(file_writer)
        .with_ansi(false);

    let level = if cfg!(debug_assertions) {
        "info,i_slint_core=debug,mukwa=trace"
    } else {
        "info,mukwa=debug"
    };

    tracing_subscriber::registry()
        .with(EnvFilter::new(level))
        .with(std_io_layer)
        .with(file_layer)
        .try_init()
        .expect("Failed to setup logging");

    info!("Launching application");

    if let Err(err) = mukwa::run() {
        error!("{}", err.report());
    }

    info!("Closing application");
}
