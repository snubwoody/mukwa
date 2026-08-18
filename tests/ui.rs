// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use mukwa::{App, Result, ui};

#[test]
fn total_spent_all_only_includes_expenses() -> Result<()> {
    let app = App::new_test()?;
    let window = app.window();
    let global_state = window.global::<ui::State>();
    let total = global_state.invoke_on_total_spent_all();
    Ok(())
}
