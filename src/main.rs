// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tracing_subscriber::fmt::init();
    finance_app::run().unwrap()
}
