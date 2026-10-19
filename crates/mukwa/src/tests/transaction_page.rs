// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::ElementHandle;
use slint::{
    ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel,
    platform::PointerEventButton,
};

use crate::ui::{AccountType, ImportCsvState, State};

use super::Result;

#[test]
fn import_csv() -> Result<()> {
    let (app, _) = super::setup()?;
    let state: State = app.global();
    let csv_state: ImportCsvState = app.global();
    state.invoke_create_account(
        "Account".to_shared_string(),
        AccountType::Cash,
        "".to_shared_string(),
    );

    let buffer = vec![];
    let writer = csv::Writer::from_writer(&mut buffer);
    writer.write_record(["31/12/2026", "100.00", "", "500.00"])?;
    writer.write_record(["31/12/2026", "50.00", "", "450.00"])?;
    writer.write_record(["31/12/2026", "", "200", "650.00"])?;
    writer.flush();

    let records = [
        ["31/12/2026", "100.00", "", "500.00"],
        ["31/12/2026", "50.00", "", "450.00"],
        ["31/12/2026", "", "200", "650.00"],
    ];
    let model = VecModel::from(records);
    // let records: Vec<Vec<String>> = writer.into_inner().unwrap();
    // csv_state.set_records(ModelRc::new(VecModel::from_slice(records)));

    ElementHandle::find_by_accessible_label(&app, "Account")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_accessible_label(&app, "Import CSV file")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    Ok(())
}
