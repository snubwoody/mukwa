// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::ElementHandle;
use mukwa::{Result, ui};

#[test]
fn button_old() {
    // i_slint_backend_testing::init_no_event_loop();
    // slint::slint! {
    //     import { ComboBox } from "../ui/widgets/combobox.slint";

    //     export component TestCase inherits Window {
    //         combobox := ComboBox{
    //         }
    //     }
    // };

    // let app = ui::MainWindow

    // let testcase = TestCase::new().unwrap();
    // let element = ElementHandle::find_by_element_id(&testcase, "combobox")
    //     .next()
    //     .unwrap();
}

#[test]
fn button() -> Result<()> {
    i_slint_backend_testing::init_no_event_loop();

    let app = ui::MainWindow::new()?;

    let sidebar = ElementHandle::find_by_element_type_name(&app, "Sidebar")
        .next()
        .unwrap();

    Ok(())
}
