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
fn update_expense() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let transaction = service.create_transaction(Default::default())?;
    let mut state = AppState::new(service)?;

    state.update_transaction(
        &transaction.id.to_string(),
        &account.id.to_string(),
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
