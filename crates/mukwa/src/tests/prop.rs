// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// TODO:
// - Maybe prop testing for each page
// - Deterministic seeding
// - Maybe a trait

use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::{ComponentHandle, ToSharedString, platform::PointerEventButton};

use crate::ui::{self, MainWindow, State};

enum Action {
    CreateTransaction,
    EditTransaction,
    DeleteTransaction,
}

// TODO: use the same RNG for seeding
fn create_transaction(app: &MainWindow) {
    let mut rng = fastrand::Rng::new();
    let transaction_page = ElementHandle::find_by_element_type_name(app, "TransactionPage")
        .next()
        .unwrap();
    transaction_page
        .query_descendants()
        .match_accessible_role(AccessibleRole::Button)
        .match_accessible_label("New")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    // TODO: test that it's closed after saving
    assert!(
        transaction_page
            .query_descendants()
            .match_type_name("NewTransactionRow")
            .find_first()
            .is_some()
    );

    let new_transaction_row = transaction_page
        .query_descendants()
        .match_type_name("NewTransactionRow")
        .find_first()
        .unwrap();

    // TODO: fill items here

    // TODO: also test no accounts?
    new_transaction_row
        .query_descendants()
        .match_type_name("AccountCell")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let account_options = new_transaction_row
        .query_descendants()
        .match_accessible_role(AccessibleRole::ListItem)
        .find_all();

    dbg!(account_options.len());
    rng.choice(account_options)
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    new_transaction_row
        .query_descendants()
        .match_accessible_role(AccessibleRole::Button)
        .match_accessible_label("Save")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        transaction_page
            .query_descendants()
            .match_type_name("NewTransactionRow")
            .find_first()
            .is_none()
    );
}

#[test]
fn test() -> crate::Result<()> {
    let (app, _) = super::setup()?;
    let action = Action::CreateTransaction;

    let mut rng = fastrand::Rng::new();
    let state: State = app.global();
    // TODO: account options has len of 7 every time?
    for _ in 0..rng.u8(..) {
        state.invoke_create_account(
            "My account".to_shared_string(),
            ui::AccountType::Cash,
            "".to_shared_string(),
        );
    }

    create_transaction(&app);
    Ok(())
}
