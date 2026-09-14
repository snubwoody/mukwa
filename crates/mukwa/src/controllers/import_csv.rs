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
use uuid::Uuid;

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
    let account_id = Uuid::parse_str(&state.get_account_id())?;

    let service = app_state.service();

    // TODO: add header row option
    // TODO: add date-format setting
    // TODO: handle getting index that doesn't exist
    for record in state.get_records().iter() {
        let record: Vec<SharedString> = record.iter().collect();
        let date = Date::strptime("%d/%m/%Y", &record[date_index])?;
        let note = &record[note_index];

        if let Ok(outflow) = Money::from_str(&record[outflow_index]) {
            service.create_expense().account(account_id).date(date).amount(outflow).note(note).submit()?;
            continue;
        }

        if let Ok(inflow) = Money::from_str(&record[inflow_index]) {
            service.create_income().account(account_id).date(date).amount(inflow).note(note).submit()?;
            continue;
        }
    }

    app_state.load_transactions()?;

    Ok(())
}

#[cfg(test)]
mod test{
    use jiff::civil::date;
    use slint::{ModelRc, ToSharedString, VecModel};
    use mukwa_core::service::{AccountType, Service, TransactionType};
    use super::*;

    fn records_to_model(records: Vec<Vec<&str>>) -> ModelRc<ModelRc<SharedString>>{
        let model = VecModel::default();
        for record in records {
            let cells: Vec<_> = record.iter().map(|s| s.to_shared_string()).collect();
            let inner_model = VecModel::from(cells);
            model.push(ModelRc::new(inner_model));
        }
        ModelRc::new(model)
    }

    #[test]
    fn parse_expense() -> crate::Result<()> {
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let account = service.create_account("",AccountType::Cash)?;
        let app_state = AppState::new(service)?;
        let csv_state = window.global::<ImportCsvState>();
        let records = vec![vec!["01/12/2024","50.00",""]];

        csv_state.set_date_column_index(0);
        csv_state.set_outflow_column_index(1);
        csv_state.set_note_column_index(2);
        csv_state.set_account_id(account.id.to_shared_string());
        csv_state.set_records(records_to_model(records));
        import_transactions(csv_state, &app_state)?;

        let transactions = app_state.service().fetch_transactions()?;
        let transaction = &transactions[0];
        assert_eq!(transaction.date,date(2024,12,1));
        assert_eq!(transaction.amount,Money::new(50));
        assert_eq!(transaction.sender_id.unwrap(),account.id);
        assert!(transaction.receiver_id.is_none());
        Ok(())
    }

    #[test]
    fn parse_income() -> crate::Result<()> {
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let account = service.create_account("",AccountType::Cash)?;
        let app_state = AppState::new(service)?;
        let csv_state = window.global::<ImportCsvState>();
        let records = vec![vec!["01/12/2024","500.00",""]];

        csv_state.set_date_column_index(0);
        csv_state.set_inflow_column_index(1);
        csv_state.set_note_column_index(2);
        csv_state.set_account_id(account.id.to_shared_string());
        csv_state.set_records(records_to_model(records));
        import_transactions(csv_state, &app_state)?;

        let transactions = app_state.service().fetch_transactions()?;
        let transaction = &transactions[0];
        assert_eq!(transaction.date,date(2024,12,1));
        assert_eq!(transaction.amount,Money::new(500));
        assert_eq!(transaction.receiver_id.unwrap(),account.id);
        assert!(transaction.sender_id.is_none());
        Ok(())
    }

    #[test]
    fn parse_empty_records() -> crate::Result<()> {
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let account = service.create_account("",AccountType::Cash)?;
        let app_state = AppState::new(service)?;
        let csv_state = window.global::<ui::ImportCsvState>();

        csv_state.set_account_id(account.id.to_shared_string());
        import_transactions(csv_state, &app_state)?;

        assert!(app_state.service().fetch_transactions()?.is_empty());
        Ok(())
    }
}