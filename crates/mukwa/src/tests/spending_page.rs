// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::{CategoryState, MainWindow, Settings};
use i_slint_backend_testing::ElementHandle;
use mukwa_core::service::AccountType;
use mukwa_core::{Currency, Money, fmt::CurrencyFormatter, service::Service};
use slint::{
    ComponentHandle, Model, PhysicalSize, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};

use super::Result;

fn setup() -> Result<(MainWindow, Service)> {
    let (app, service) = super::setup()?;

    ElementHandle::find_by_accessible_label(&app, "Spending")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    Ok((app, service))
}

/// Asserts that an element with the accessible label exists.
fn assert_exists(handle: &ElementHandle, accessible_label: &str) {
    let element = handle
        .query_descendants()
        .match_accessible_label(accessible_label)
        .find_first();
    assert!(
        element.is_some(),
        "No element with accessible label '{}' was not found",
        accessible_label
    );
}

/// Asserts that an element with the accessible label does not exist.
fn assert_not_exists(handle: &ElementHandle, accessible_label: &str) {
    let element = handle
        .query_descendants()
        .match_accessible_label(accessible_label)
        .find_first();
    assert!(
        element.is_none(),
        "Element with accessible label '{}' exists",
        accessible_label
    );
}

/// Opens the budget target form, enters the `amount` in the text field and creates a new
/// budget target.
///
/// Only works if the `SpendingPageSideBar` is already open.
fn create_budget_target(app: &MainWindow, amount: Money) {
    ElementHandle::find_by_accessible_label(app, "Create target")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    ElementHandle::find_by_element_type_name(app, "TextField")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    i_slint_backend_testing::send_keyboard_string_sequence(app, &amount.to_string());
    ElementHandle::find_by_accessible_label(app, "Save target")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
}

#[test]
fn edit_budget_target() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    create_budget_target(&app, Money::new(50));

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    sidebar
        .query_descendants()
        .match_accessible_label("Edit target")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let edit_form = sidebar
        .query_descendants()
        .match_type_name("EditBudgetTarget")
        .find_first()
        .unwrap();

    edit_form
        .query_descendants()
        .match_type_name("TextField")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    i_slint_backend_testing::send_keyboard_string_sequence(&app, "750");

    sidebar
        .query_descendants()
        .match_accessible_label("Save target")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        sidebar
            .query_descendants()
            .match_type_name("EditBudgetTarget")
            .find_first()
            .is_none()
    );

    let category = category_state.get_categories().iter().next().unwrap();
    let target = category_state.invoke_get_budget_target(category.id);
    assert_eq!(target, Money::new(750).to_shared_string());
    Ok(())
}

#[test]
fn delete_budget_target() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    create_budget_target(&app, Money::new(50));

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    sidebar
        .query_descendants()
        .match_accessible_label("Edit target")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let edit_form = sidebar
        .query_descendants()
        .match_type_name("EditBudgetTarget")
        .find_first()
        .unwrap();

    edit_form
        .query_descendants()
        .match_accessible_label("Delete target")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        sidebar
            .query_descendants()
            .match_type_name("EditBudgetTarget")
            .find_first()
            .is_none()
    );

    let category = category_state.get_categories().iter().next().unwrap();
    assert!(!category_state.invoke_has_budget_target(category.id));

    let helper_text =
        "A budget target is an estimate of how much you want to assign to a category each month.";
    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label(helper_text)
            .find_first()
            .is_some()
    );
    Ok(())
}

#[test]
fn cancel_edit_budget_target_form() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    create_budget_target(&app, Money::new(50));

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    sidebar
        .query_descendants()
        .match_accessible_label("Edit target")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let edit_form = sidebar
        .query_descendants()
        .match_type_name("EditBudgetTarget")
        .find_first()
        .unwrap();

    edit_form
        .query_descendants()
        .match_accessible_label("Cancel")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        sidebar
            .query_descendants()
            .match_type_name("EditBudgetTarget")
            .find_first()
            .is_none()
    );

    let category = category_state.get_categories().iter().next().unwrap();
    let target = category_state.invoke_get_budget_target(category.id);
    assert_eq!(target, Money::new(50).to_shared_string());
    Ok(())
}

#[test]
fn cancel_create_budget_target_form() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_accessible_label(&app, "Create target")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_accessible_label(&app, "Cancel")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    assert!(
        sidebar
            .query_descendants()
            .match_type_name("CreateBudgetTarget")
            .find_first()
            .is_none()
    );

    let category = category_state.get_categories().iter().next().unwrap();
    assert!(!category_state.invoke_has_budget_target(category.id));
    Ok(())
}

#[test]
fn switch_after_creating_budget_target() -> Result<()> {
    let (app, _) = setup()?;
    let window = app.window();
    window.set_size(PhysicalSize::new(1200, 1200));

    let settings: Settings = app.global();
    settings.set_currency_code("CAD".to_shared_string());

    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());
    category_state
        .invoke_create_category(SharedString::from("Entertainment"), groups[1].id.clone());

    let category_items: Vec<_> =
        ElementHandle::find_by_element_type_name(&app, "CategoryItem").collect();

    category_items[0].mock_single_click(PointerEventButton::Left);

    create_budget_target(&app, Money::new(50));

    let category_items: Vec<_> =
        ElementHandle::find_by_element_type_name(&app, "CategoryItem").collect();

    category_items[1]
        .query_descendants()
        .match_type_name("TouchArea")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::CAD);

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    assert_exists(&sidebar, "Create target");
    let amount = formatter.format_money(Money::new(50))?;
    let zero_amount = formatter.format_money(Money::new(0))?;
    assert_not_exists(
        &sidebar,
        &format!("Assign {amount} more to reach your target."),
    );
    assert_not_exists(&sidebar, &format!("Assigned {zero_amount} of {amount}."));
    Ok(())
}

#[test]
fn budget_target_helper_text() -> Result<()> {
    let (app, _) = setup()?;
    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    create_budget_target(&app, Money::new(200));

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    let formatter = CurrencyFormatter::new();
    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label(format!(
                "Assign {} more to reach your target.",
                formatter.format_money(Money::new(200)).unwrap()
            ))
            .find_first()
            .is_some()
    );
    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label(format!(
                "Assigned {} of {}",
                formatter.format_money(Money::ZERO).unwrap(),
                formatter.format_money(Money::new(200)).unwrap()
            ))
            .find_first()
            .is_some()
    );

    let budget = category_state.get_budgets().iter().next().unwrap();
    category_state.invoke_edit_budget(budget.id.clone(), Money::new(50).to_shared_string());

    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label(format!(
                "Assign {} more to reach your target.",
                formatter.format_money(Money::new(150)).unwrap()
            ))
            .find_first()
            .is_some()
    );
    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label(format!(
                "Assigned {} of {}",
                formatter.format_money(Money::new(50)).unwrap(),
                formatter.format_money(Money::new(200)).unwrap()
            ))
            .find_first()
            .is_some()
    );

    category_state.invoke_edit_budget(budget.id, Money::new(200).to_shared_string());
    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label("You've reached your target.".to_string())
            .find_first()
            .is_some()
    );
    Ok(())
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

    let budget = category_state.get_budgets().iter().next().unwrap();
    assert_eq!(budget.amount, Money::new(200).to_shared_string());
    Ok(())
}

#[test]
fn create_budget_target_form() -> Result<()> {
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

    create_budget_target(&app, Money::new(200));

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
        .match_accessible_label("Rent")
        .find_first();
    assert!(title.is_some());
    Ok(())
}

#[test]
fn assign_button_reloads_left_to_budget() -> Result<()> {
    let (app, service) = super::setup()?;
    let settings: Settings = app.global();
    settings.set_currency_code("CAD".to_shared_string());

    let account = service.create_account("", AccountType::Cash)?;
    service
        .create_income()
        .amount(Money::new(750))
        .account(account.id)
        .submit()?;

    let category_state: CategoryState = app.global();
    category_state.invoke_create_category_group(SharedString::new());
    let groups: Vec<_> = category_state.get_category_groups().iter().collect();
    category_state.invoke_create_category(SharedString::from("Rent"), groups[1].id.clone());

    ElementHandle::find_by_accessible_label(&app, "Spending")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let formatter = CurrencyFormatter::with_currency(Currency::CAD);

    let left_to_budget =
        ElementHandle::find_by_element_id(&app, "SpendingPageBody::left-to-budget-amount")
            .next()
            .unwrap();
    assert_eq!(
        left_to_budget.accessible_label().unwrap(),
        formatter.format_money(Money::new(750))?
    );

    ElementHandle::find_by_element_type_name(&app, "CategoryItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
        .next()
        .unwrap();

    create_budget_target(&app, Money::new(500));

    sidebar
        .query_descendants()
        .match_accessible_label("Assign")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        left_to_budget.accessible_label().unwrap(),
        formatter.format_money(Money::new(250))?
    );
    Ok(())
}
