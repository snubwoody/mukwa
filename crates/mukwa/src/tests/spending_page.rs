// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::ElementHandle;
use mukwa_core::{Money, service::Service};
use slint::{
    ComponentHandle, Model, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};

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
fn assign_button_assigns_target_amount() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    let category = category_state.get_categories().iter().next().unwrap();
    category_state.invoke_create_budget_target(category.id.clone(), "200".to_shared_string());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_accessible_label(&app, "Assign")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let budgets: Vec<_> = category_state.get_budgets().iter().collect();
    let budget = category_state.get_budgets().iter().next().unwrap();
    assert_eq!(budget.amount, Money::new(200).to_shared_string());
    Ok(())
}

#[test]
fn create_budget_target() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let helper_text =
        "A budget target is an estimate of how much you want to assign to a category each month.";

    assert!(
        ElementHandle::find_by_accessible_label(&app, helper_text)
            .next()
            .is_some()
    );

    ElementHandle::find_by_accessible_label(&app, "Create target")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_element_type_name(&app, "TextField")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    i_slint_backend_testing::send_keyboard_string_sequence(&app, &SharedString::from("200"));

    ElementHandle::find_by_accessible_label(&app, "Save target")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let category = category_state.get_categories().iter().next().unwrap();
    let target = category_state.invoke_get_budget_target(category.id);
    assert_eq!(target, Money::new(200).to_shared_string());

    assert!(
        ElementHandle::find_by_accessible_label(&app, helper_text)
            .next()
            .is_none()
    );
    Ok(())
}

#[test]
fn return_key_submits_target_form() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let helper_text =
        "A budget target is an estimate of how much you want to assign to a category each month.";

    assert!(
        ElementHandle::find_by_accessible_label(&app, helper_text)
            .next()
            .is_some()
    );

    ElementHandle::find_by_accessible_label(&app, "Create target")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_element_type_name(&app, "TextField")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    i_slint_backend_testing::send_keyboard_string_sequence(&app, &SharedString::from("200"));
    i_slint_backend_testing::send_keyboard_string_sequence(&app, &SharedString::from(Key::Return));

    let category = category_state.get_categories().iter().next().unwrap();
    let target = category_state.invoke_get_budget_target(category.id);
    assert_eq!(target, Money::new(200).to_shared_string());

    assert!(
        ElementHandle::find_by_accessible_label(&app, helper_text)
            .next()
            .is_none()
    );
    Ok(())
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
