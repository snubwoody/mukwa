// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui;
use crate::ui::{ImportCsvState, MainWindow};
use jiff::civil::Date;
use mukwa_core::Money;
use slint::{ComponentHandle, Global, Model, SharedString};
use std::str::FromStr;
use tracing::{info, warn};

pub fn bind(window: &MainWindow, app_state: &AppState) {
    let csv_state = window.global::<ui::ImportCsvState>();

    csv_state.on_import_transactions({
        let csv_state = csv_state.as_weak();
        let app_state = app_state.clone();
        move || {
            let csv_state = csv_state.unwrap();
            match import_transactions(csv_state, &app_state) {
                Ok(_) => info!("Successfully imported transactions"),
                Err(err) => warn!("Failed to import transactions: {err}"),
            }
        }
    });
}

fn import_transactions(state: ImportCsvState, app_state: &AppState) -> crate::Result<()> {
    // TODO: ignore failures?
    let state = state.as_weak();
    let state = state.unwrap();
    let date_index = state.get_date_column_index() as usize;
    let inflow_index = state.get_inflow_column_index() as usize;
    let outflow_index = state.get_outflow_column_index() as usize;
    let note_index = state.get_note_column_index() as usize;
    let payee_index = state.get_payee_column_index() as usize;

    let service = app_state.service();

    // TODO: add header row option
    // TODO: add account-id
    // TODO: add date-format setting
    // TODO: handle getting index that doesn't exist
    for record in state.get_records().iter() {
        let record: Vec<SharedString> = record.iter().collect();
        let date = Date::strptime("%m/%d/%Y", &record[date_index])?;
        let note = &record[note_index];

        // dbg!(date, note, outflow, note);
        if let Ok(outflow) = Money::from_str(&record[outflow_index]) {
            service.create_expense().date(date).amount(outflow).note(note).submit()?;
            continue;
        }

        if let Ok(inflow) = Money::from_str(&record[inflow_index]) {
            service.create_income().date(date).amount(inflow).note(note).submit()?;
            continue;
        }
    }

    app_state.load_transactions()?;

    Ok(())
}
