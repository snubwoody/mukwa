// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::{AccessibleRole, ElementHandle, mock_elapsed_time};
use mukwa_core::{
    Money,
    fmt::CurrencyFormatter,
    service::{AccountType, Service},
};
use slint::{
    ComponentHandle, Model, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};

use crate::ui::{CategoryState, MainWindow, TransactionState};

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
