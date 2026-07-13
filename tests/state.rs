// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use mukwa::service::{Service, TransactionType};
use mukwa::state::AppState;
use mukwa::{Money, create_test_db};
use slint::Model;

#[test]
fn create_transaction_creates_expense() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let mut state = AppState::new(service)?;
    state.create_account("")?;
    state.create_transaction()?;

    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.inflow.as_str(), "");
    assert_eq!(transaction.outflow.as_str(), Money::ZERO.to_string());
    Ok(())
}

#[test]
fn create_account_adds_to_account_list() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let mut state = AppState::new(service)?;
    state.create_account("")?;
    state.create_account("")?;

    assert_eq!(state.accounts().iter().len(), 2);
    Ok(())
}

#[test]
fn create_account_adds_to_account_options() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let mut state = AppState::new(service)?;
    state.create_account("")?;
    state.create_account("")?;

    assert_eq!(state.account_options().iter().len(), 2);
    Ok(())
}

#[test]
fn update_expense() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_transaction(Default::default())?;
    let mut state = AppState::new(service)?;

    state.update_transaction(
        &transaction.id.to_string(),
        &account.id.to_string(),
        "",
        "200",
        "",
        "",
    )?;
    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.outflow.as_str(), Money::new(200).to_string());
    assert_eq!(transaction.inflow.as_str(), "");
    Ok(())
}

#[test]
fn convert_expense_to_income() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_transaction(Default::default())?;
    let mut state = AppState::new(service.clone())?;

    state.update_transaction(
        &transaction.id.to_string(),
        &account.id.to_string(),
        "",
        "",
        "500",
        "",
    )?;
    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.inflow.as_str(), Money::new(500).to_string());
    assert_eq!(transaction.outflow.as_str(), "");

    let transactions = service.fetch_transactions()?;
    let transaction = &transactions[0];
    assert_eq!(transaction.transaction_type(), TransactionType::Income);
    assert!(transaction.sender_id.is_none());
    assert_eq!(transaction.receiver_id.unwrap(), account.id);
    assert_eq!(transaction.amount, Money::new(500));
    Ok(())
}

#[test]
fn state_loads_data_from_service() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    service.create_account("")?;
    service.create_account("")?;
    service.create_transaction(Default::default())?;
    service.create_transaction(Default::default())?;
    service.create_transaction(Default::default())?;
    let state = AppState::new(service)?;
    assert_eq!(state.transactions().iter().len(), 3);
    assert_eq!(state.accounts().iter().len(), 2);
    Ok(())
}

#[test]
fn state_loads_categories_from_service() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_category(Default::default())?;
    service.create_category(Default::default())?;

    let state = AppState::new(service)?;
    assert_eq!(state.categories().iter().len(), 2);
    Ok(())
}
