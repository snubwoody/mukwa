// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::PointerEventButton;
use slint::{ComponentHandle, ToSharedString};

use crate::Result;
use crate::tests::setup;
use crate::ui::{AccountType, ImportCsvState, State};

// TODO:
// - test that transactions are reloaded
#[test]
fn close_import_csv_dialog() -> Result<()> {
    let (app, _) = setup()?;
    let state: State = app.global();
    let import_state: ImportCsvState = app.global();
    state.invoke_create_account(
        "My account".to_shared_string(),
        AccountType::Cash,
        "".to_shared_string(),
    );

    import_state.set_has_data(true);

    ElementHandle::find_by_accessible_label(&app, "My account")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_accessible_label(&app, "Import CSV file")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    for element in ElementQuery::from_root(&app)
        .match_inherits("PopupWindow")
        .find_all()
    {
        dbg!(element.type_name());
        if element.id().is_some() {
            dbg!(element.id());
        }
        if element.type_name() == Some("TransactionTable".to_shared_string()) {
            dbg!(element.type_name());
        }
    }

    let import_dialog = ElementHandle::find_by_element_type_name(&app, "ImportCsv")
        .next()
        .unwrap();

    import_dialog
        .query_descendants()
        .match_accessible_label("Close")
        .find_first()
        .unwrap();

    assert!(
        ElementHandle::find_by_element_type_name(&app, "ImportCsv")
            .next()
            .is_none()
    );
    Ok(())
}
