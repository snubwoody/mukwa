// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::{CategoryState, MainWindow, Settings};
use crate::{AppState, SettingsStore, controllers};
use i_slint_backend_testing::{ElementHandle, ElementRoot};
use jiff::civil::{Date, date};
use jiff::{ToSpan, Zoned};
use mukwa_core::service::{AccountType, CreateBudgetOpts};
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
fn close_sidebar() -> Result<()> {
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
        .match_accessible_label("Close")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        ElementHandle::find_by_element_type_name(&app, "SpendingPageSideBar")
            .next()
            .is_none()
    );
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

#[test]
fn category_helper_text() -> Result<()> {
    // TODO: test
    // - Zero
    // - Overspent
    // - Rollover
    // - No budget?
    // - Budget progress
    // - Hide helper text if budget is 0 and spent is 0
    // - Make similar tests for fully spent and overspent
    // - Try using closures instead

    struct TestCase {
        budgets: Vec<(Date, Money)>,
        transactions: Vec<(Date, Money)>,
        expected: (Date, Money, Money),
    }

    let test_cases = [
        TestCase {
            budgets: vec![(Zoned::now().date(), Money::new(100))],
            transactions: vec![(Zoned::now().date(), Money::new(20))],
            expected: (Zoned::now().date(), Money::new(20), Money::new(100)),
        },
        // Spent 0 of 100
        TestCase {
            budgets: vec![(Zoned::now().date(), Money::new(100))],
            transactions: vec![],
            expected: (Zoned::now().date(), Money::ZERO, Money::new(100)),
        },
        // Spent 0 of 0
        TestCase {
            budgets: vec![(Zoned::now().date(), Money::ZERO)],
            transactions: vec![],
            expected: (Zoned::now().date(), Money::ZERO, Money::ZERO),
        },
        // Rollover into next month
        TestCase {
            budgets: vec![
                (Zoned::now().date(), Money::new(100)),
                (Zoned::now().date() + 1.month(), Money::ZERO),
            ],
            transactions: vec![(Zoned::now().date(), Money::new(20))],
            expected: (Zoned::now().date() + 1.month(), Money::ZERO, Money::new(80)),
        },
    ];

    i_slint_backend_testing::init_no_event_loop();
    for case in test_cases {
        let service = Service::open_in_memory()?;
        service.create_account("", AccountType::Cash)?;

        let group = service.create_category_group("")?;
        let category = service.create_category("Test Category", group.id)?;
        for (date, amount) in case.budgets {
            service.create_budget(CreateBudgetOpts {
                amount: Some(amount),
                month: Some(date),
                category_id: category.id,
            })?;
        }
        for (date, amount) in case.transactions {
            service
                .create_expense()
                .category(category.id)
                .date(date)
                .amount(amount)
                .submit()?;
        }

        let app = MainWindow::new()?;
        let state = AppState::new(service.clone())?;
        let settings = SettingsStore::open_in_memory();
        controllers::bind_all(&app, &state, &settings)?;

        let settings: Settings = app.global();
        settings.set_currency_code("CAD".to_shared_string());
        let category_state: CategoryState = app.global();
        category_state.invoke_set_current_budget_month(case.expected.0.into());
        let formatter = CurrencyFormatter::with_currency(Currency::CAD);

        let root_element = app.root_element();
        find_by_accessible_label(&root_element, "Spending")
            .mock_single_click(PointerEventButton::Left);
        let category_item = find_by_type_name(&root_element, "CategoryItem");
        let helper_text = category_item
            .query_descendants()
            .match_id("CategoryItem::helper-text")
            .find_first()
            .unwrap();
        assert_eq!(
            helper_text.accessible_label().unwrap(),
            format!(
                "Spent {} of {}",
                formatter.format_money(case.expected.1)?,
                formatter.format_money(case.expected.2)?
            ),
        );
    }
    Ok(())
}

#[test]
fn category_helper_text_fully_spent() -> Result<()> {
    // TODO: test
    // - Zero
    // - Overspent
    // - Rollover
    // - No budget?
    // - Budget progress
    // - Hide helper text if budget is 0 and spent is 0
    // - Make similar tests for fully spent and overspent
    // - Try using closures instead

    i_slint_backend_testing::init_no_event_loop();
    let run_test = |budgets: &[(Date, Money)], transactions: &[(Date, Money)], date: Date| {
        let service = Service::open_in_memory().unwrap();
        service.create_account("", AccountType::Cash).unwrap();

        let group = service.create_category_group("").unwrap();
        let category = service.create_category("Test Category", group.id).unwrap();
        for (date, amount) in budgets {
            service
                .create_budget(CreateBudgetOpts {
                    amount: Some(*amount),
                    month: Some(*date),
                    category_id: category.id,
                })
                .unwrap();
        }
        for (date, amount) in transactions {
            service
                .create_expense()
                .category(category.id)
                .date(*date)
                .amount(*amount)
                .submit()
                .unwrap();
        }

        let app = MainWindow::new().unwrap();
        let window = app.window();
        window.set_size(PhysicalSize::new(1200, 1200));
        let state = AppState::new(service.clone()).unwrap();
        let settings = SettingsStore::open_in_memory();
        controllers::bind_all(&app, &state, &settings).unwrap();
        let category_state: CategoryState = app.global();
        category_state.invoke_set_current_budget_month(date.into());

        let root_element = app.root_element();
        find_by_accessible_label(&root_element, "Spending")
            .mock_single_click(PointerEventButton::Left);
        let category_item = find_by_type_name(&root_element, "CategoryItem");
        category_item.visit_descendants(|el| {
            dbg!(el.type_name());
            dbg!(el.accessible_label());
            dbg!(el.id());
            std::ops::ControlFlow::<ElementHandle>::Continue(())
        });
        let helper_text = category_item
            .query_descendants()
            .match_id("CategoryItem::helper-text")
            .find_first()
            .unwrap();
        assert_eq!(helper_text.accessible_label().unwrap(), "Fully spent");
    };

    let now = Zoned::now().date();
    run_test(&[(now, Money::new(100))], &[(now, Money::new(100))], now);
    // Rollover into future months
    run_test(
        &[(now, Money::new(100))],
        &[
            (now + 2.months(), Money::new(50)),
            (now + 7.months(), Money::new(50)),
        ],
        now + 10.months(),
    );
    Ok(())
}

fn find_by_accessible_label(element: &ElementHandle, label: &str) -> ElementHandle {
    element
        .query_descendants()
        .match_accessible_label(label)
        .find_first()
        .unwrap()
}

fn find_by_type_name(element: &ElementHandle, type_name: &str) -> ElementHandle {
    element
        .query_descendants()
        .match_type_name(type_name)
        .find_first()
        .unwrap()
}
