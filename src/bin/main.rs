// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing::error;

fn main() {
    tracing_subscriber::fmt::init();
    if let Err(err) = mukwa::run() {
        error!("{}", err.report());
    }
}
