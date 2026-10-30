// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::{self, Settings, State, TransactionState};
use i_slint_backend_testing::{AccessibleRole, ElementHandle, mock_elapsed_time};
use mukwa_core::{Currency, Money, fmt::CurrencyFormatter, service::AccountType};
use slint::{ComponentHandle, Model, ToSharedString, platform::PointerEventButton};

use super::Result;

#[test]
fn open_and_close_searchbox() -> Result<()> {
    let (app, _) = super::setup()?;

    let search = ElementHandle::find_by_element_type_name(&app, "Search")
        .next()
        .unwrap();

    assert!(
        search
            .query_descendants()
            .match_accessible_role(AccessibleRole::TextInput)
            .find_first()
            .is_none()
    );

    assert!(
        search
            .query_descendants()
            .match_accessible_label("Close")
            .find_first()
            .is_none()
    );

    search
        .query_descendants()
        .match_accessible_label("Search")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    assert!(
        search
            .query_descendants()
            .match_accessible_role(AccessibleRole::TextInput)
            .find_first()
            .is_some()
    );

    search
        .query_descendants()
        .match_accessible_label("Clear")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        search
            .query_descendants()
            .match_type_name("TextField")
            .find_first()
            .is_none()
    );

    assert!(
        search
            .query_descendants()
            .match_accessible_label("Clear")
            .find_first()
            .is_none()
    );
    Ok(())
}

#[test]
fn type_in_searchbox() -> Result<()> {
    let (app, service) = super::setup()?;
    let transaction_state: TransactionState = app.global();
    service.create_account("", AccountType::Cash)?;
    service.create_expense().note("Burger").submit()?;
    service.create_expense().note("Burger king").submit()?;
    service.create_income().submit()?;
    transaction_state.invoke_load_transactions();

    let search = ElementHandle::find_by_element_type_name(&app, "Search")
        .next()
        .unwrap();

    search
        .query_descendants()
        .match_accessible_label("Search")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    i_slint_backend_testing::send_keyboard_string_sequence(&app, "Burger");
    assert_eq!(transaction_state.get_transactions().iter().len(), 2);
    i_slint_backend_testing::send_keyboard_string_sequence(&app, " king");
    assert_eq!(transaction_state.get_transactions().iter().len(), 1);

    search
        .query_descendants()
        .match_accessible_label("Clear")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(transaction_state.get_transactions().iter().len(), 3);
    Ok(())
}

#[test]
fn total_balance_does_not_update_with_search() -> Result<()> {
    let (app, service) = super::setup()?;
    let transaction_state: TransactionState = app.global();
    let settings: Settings = app.global();
    settings.set_currency_code("USD".to_shared_string());
    service.create_account("", AccountType::Cash)?;
    service
        .create_income()
        .amount(Money::new(25))
        .note("Burger")
        .submit()?;
    service
        .create_income()
        .amount(Money::new(75))
        .note("Burger king")
        .submit()?;
    service.create_income().amount(Money::new(200)).submit()?;
    transaction_state.invoke_load_transactions();

    let search = ElementHandle::find_by_element_type_name(&app, "Search")
        .next()
        .unwrap();

    search
        .query_descendants()
        .match_accessible_label("Search")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    let total_spent = ElementHandle::find_by_element_id(&app, "TransactionPage::total-spent")
        .next()
        .unwrap();

    i_slint_backend_testing::send_keyboard_string_sequence(&app, "Burger");
    assert_eq!(
        total_spent.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    i_slint_backend_testing::send_keyboard_string_sequence(&app, " king");
    assert_eq!(
        total_spent.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );

    search
        .query_descendants()
        .match_accessible_label("Clear")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        total_spent.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    Ok(())
}

#[test]
fn account_balance_does_not_update_with_search() -> Result<()> {
    let (app, service) = super::setup()?;
    let state: State = app.global();
    let transaction_state: TransactionState = app.global();
    let settings: Settings = app.global();

    settings.set_currency_code("USD".to_shared_string());
    state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );
    let accounts = service.fetch_accounts()?;
    let account = &accounts[0];
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(25))
        .note("Burger")
        .submit()?;
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(75))
        .note("Burger king")
        .submit()?;
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(200))
        .submit()?;
    state.invoke_load_accounts();
    transaction_state.invoke_load_transactions();

    ElementHandle::find_by_accessible_label(&app, "My account")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let search = ElementHandle::find_by_element_type_name(&app, "Search")
        .next()
        .unwrap();

    search
        .query_descendants()
        .match_accessible_label("Search")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    let total_spent = ElementHandle::find_by_element_id(&app, "TransactionPage::total-spent")
        .next()
        .unwrap();

    i_slint_backend_testing::send_keyboard_string_sequence(&app, "Burger");
    assert_eq!(
        total_spent.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    i_slint_backend_testing::send_keyboard_string_sequence(&app, " king");
    assert_eq!(
        total_spent.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );

    search
        .query_descendants()
        .match_accessible_label("Clear")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        total_spent.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    Ok(())
}

#[test]
fn editing_transaction_does_not_reload_all_transactions() -> Result<()> {
    let (app, service) = super::setup()?;
    let transaction_state: TransactionState = app.global();
    let settings: Settings = app.global();
    settings.set_currency_code("USD".to_shared_string());
    service.create_account("", AccountType::Cash)?;

    service.create_expense().amount(Money::new(50)).submit()?;
    service.create_income().amount(Money::new(50)).submit()?;
    transaction_state.invoke_load_transactions();

    service.create_income().amount(Money::new(50)).submit()?;
    Ok(())
}
