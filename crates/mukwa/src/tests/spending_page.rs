// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::ElementHandle;
use mukwa_core::service::Service;
use slint::{ComponentHandle, Model, SharedString, platform::PointerEventButton};

use crate::ui::{CategoryState, MainWindow};

use super::Result;

fn setup() -> crate::Result<(MainWindow, Service)> {
    let (app, service) = super::setup()?;

    ElementHandle::find_by_accessible_label(&app, "Spending")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    Ok((app, service))
}

#[test]
fn category_shown_in_sidebar() -> Result<()> {
    let (app, _) = setup()?;

    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());
    category_state.invoke_create_category(SharedString::from("Groceries"), groups[1].id.clone());

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar").next();
    assert!(sidebar.is_none());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    let title = sidebar
        .query_descendants()
        .match_type_name("Text")
        .match_predicate(|element| element.accessible_label() == Some(SharedString::from("Rent")))
        .find_first();
    assert!(title.is_some());

    let category_items: Vec<_> =
        ElementHandle::find_by_element_type_name(&app, "CategoryItem").collect();
    category_items[1].mock_single_click(PointerEventButton::Left);

    let title = sidebar
        .query_descendants()
        .match_type_name("Text")
        .match_predicate(|element| {
            element.accessible_label() == Some(SharedString::from("Groceries"))
        })
        .find_first();
    assert!(title.is_some());
    Ok(())
}
