// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use i_slint_backend_testing::ElementHandle;
use mukwa_core::{Currency, Money, fmt::CurrencyFormatter, service::AccountType};
use slint::{ComponentHandle, Model, SharedString, ToSharedString, platform::PointerEventButton};

use crate::ui::{CategoryState, CreateTransactionOpts, Settings, TransactionState};

use super::Result;

#[test]
fn total_spent_updates_with_category_filter() -> Result<()> {
    let (app, service) = super::setup()?;
    let category_state: CategoryState = app.global();
    let transaction_state: TransactionState = app.global();
    let settings: Settings = app.global();
    settings.set_currency_code("USD".to_shared_string());
    service.create_account("", AccountType::Cash)?;
    let group = service.create_category_group("")?;
    category_state.invoke_create_category(
        "Entertainment".to_shared_string(),
        group.id.to_shared_string(),
    );
    category_state
        .invoke_create_category("Transport".to_shared_string(), group.id.to_shared_string());

    let categories: Vec<_> = category_state.get_categories().iter().collect();
    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        category_id: categories[0].id.clone(),
        outflow: SharedString::from("500"),
        ..Default::default()
    });
    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        category_id: categories[1].id.clone(),
        outflow: SharedString::from("10"),
        ..Default::default()
    });

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let pie_chart = ElementHandle::find_by_element_type_name(&app, "PieChart")
        .next()
        .unwrap();

    let mut formater = CurrencyFormatter::new();
    formater.set_currency(Currency::USD);

    let amount = formater.format_money(Money::new(510)).unwrap();
    let text_elements = pie_chart
        .query_descendants()
        .match_type_name("Text")
        .match_predicate(move |element| {
            element.accessible_label() == Some(amount.to_shared_string())
        })
        .find_all();

    assert_eq!(text_elements.len(), 2);

    ElementHandle::find_by_accessible_label(&app, "All categories")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_element_type_name(&app, "Checkbox")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let amount = formater.format_money(Money::new(10)).unwrap();
    let text_elements = pie_chart
        .query_descendants()
        .match_type_name("Text")
        .match_predicate(move |element| {
            element.accessible_label() == Some(amount.to_shared_string())
        })
        .find_all();
    assert_eq!(text_elements.len(), 2);
    Ok(())
}

#[test]
fn category_filter_helper_text() -> Result<()> {
    let (app, service) = super::setup()?;
    let category_state: CategoryState = app.global();
    service.create_account("", AccountType::Cash)?;
    let group = service.create_category_group("")?;
    category_state.invoke_create_category(
        "Entertainment".to_shared_string(),
        group.id.to_shared_string(),
    );
    category_state.invoke_create_category("Gaming".to_shared_string(), group.id.to_shared_string());
    category_state
        .invoke_create_category("Transport".to_shared_string(), group.id.to_shared_string());

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let category_filter = ElementHandle::find_by_element_type_name(&app, "CategoryFilter")
        .next()
        .unwrap();

    category_filter.mock_single_click(PointerEventButton::Left);

    let checkboxes = category_filter
        .query_descendants()
        .match_type_name("Checkbox")
        .find_all();

    assert_eq!(checkboxes.len(), 3);

    assert!(
        category_filter
            .query_descendants()
            .match_predicate(move |element| {
                element.accessible_label() == Some("All categories".to_shared_string())
            })
            .find_first()
            .is_some()
    );

    checkboxes[0].mock_single_click(PointerEventButton::Left);
    assert!(
        category_filter
            .query_descendants()
            .match_predicate(move |element| {
                element.accessible_label() == Some("2 categories".to_shared_string())
            })
            .find_first()
            .is_some()
    );

    checkboxes[1].mock_single_click(PointerEventButton::Left);
    assert!(
        category_filter
            .query_descendants()
            .match_predicate(move |element| {
                element.accessible_label() == Some("1 category".to_shared_string())
            })
            .find_first()
            .is_some()
    );

    checkboxes[2].mock_single_click(PointerEventButton::Left);
    assert!(
        category_filter
            .query_descendants()
            .match_predicate(move |element| {
                element.accessible_label() == Some("0 categories".to_shared_string())
            })
            .find_first()
            .is_some()
    );
    Ok(())
}
