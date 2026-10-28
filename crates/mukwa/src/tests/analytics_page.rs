// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::{AnalyticsApi, CategoryState, CreateTransactionOpts, Settings, TransactionState};
use i_slint_backend_testing::ElementHandle;
use jiff::{ToSpan, Zoned, civil::date};
use mukwa_core::service::TransactionKind;
use mukwa_core::{Currency, Money, fmt::CurrencyFormatter, service::AccountType};
use slint::{ComponentHandle, Model, SharedString, ToSharedString, platform::PointerEventButton};

use super::Result;

#[test]
fn page_title() -> Result<()> {
    let (app, _) = super::setup()?;
    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        ElementHandle::find_by_accessible_label(&app, "Spending chart")
            .next()
            .is_some()
    );

    Ok(())
}

#[test]
fn sidebar_title() -> Result<()> {
    let (app, _) = super::setup()?;
    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    let sidebar = ElementHandle::find_by_element_type_name(&app, "SideBar")
        .next()
        .unwrap();

    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label("Category")
            .find_first()
            .is_some()
    );
    assert!(
        sidebar
            .query_descendants()
            .match_accessible_label("Total spent")
            .find_first()
            .is_some()
    );
    Ok(())
}

#[test]
fn sidebar_no_transactions_in_current_month() -> Result<()> {
    let (app, service) = super::setup()?;
    service.create_account("", AccountType::Cash)?;
    let group = service.create_category_group("")?;
    let c1 = service.create_category("Rent", group.id)?;
    service
        .create_expense()
        .category(c1.id)
        .date(date(2020, 2, 1))
        .amount(Money::new(25))
        .submit()?;
    service
        .create_expense()
        .category(c1.id)
        .date(date(2019, 12, 31))
        .amount(Money::new(50))
        .submit()?;

    let analytics_api: AnalyticsApi = app.global();
    analytics_api.set_current_date(date(2020, 1, 1).into());

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    let sidebar = ElementHandle::find_by_element_type_name(&app, "SideBar")
        .next()
        .unwrap();

    let category_items = sidebar
        .query_descendants()
        .match_type_name("CategoryItem")
        .find_all();

    assert_eq!(category_items.len(), 0);
    Ok(())
}

#[test]
fn month_switch() -> Result<()> {
    let (app, _) = super::setup()?;
    let analytics: AnalyticsApi = app.global();
    assert_eq!(analytics.get_current_date(), Zoned::now().date().into());

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let month_switch = ElementHandle::find_by_element_type_name(&app, "MonthSwitch")
        .next()
        .unwrap();

    month_switch
        .query_descendants()
        .match_accessible_label("Previous month")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        analytics.get_current_date(),
        Zoned::now().date().saturating_sub(1.month()).into()
    );

    month_switch
        .query_descendants()
        .match_accessible_label("Today")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(analytics.get_current_date(), Zoned::now().date().into());
    assert!(
        month_switch
            .query_descendants()
            .match_accessible_label("Today")
            .find_first()
            .is_none()
    );

    month_switch
        .query_descendants()
        .match_accessible_label("Next month")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert_eq!(
        analytics.get_current_date(),
        Zoned::now().date().saturating_add(1.month()).into()
    );

    analytics.set_current_date(date(2026, 12, 1).into());
    month_switch
        .query_descendants()
        .match_accessible_label("Next month")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(analytics.get_current_date(), date(2027, 1, 1).into());

    analytics.set_current_date(date(2026, 1, 1).into());
    month_switch
        .query_descendants()
        .match_accessible_label("Previous month")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(analytics.get_current_date(), date(2025, 12, 1).into());

    assert!(
        month_switch
            .query_descendants()
            .match_accessible_label("Dec 2025")
            .find_first()
            .is_some()
    );
    Ok(())
}

#[test]
fn total_spent_excludes_starting_balance() -> Result<()> {
    let (app, service) = super::setup()?;
    let settings: Settings = app.global();
    settings.set_currency_code("USD".to_shared_string());
    service.create_account("", AccountType::Cash)?;
    service
        .create_expense()
        .kind(TransactionKind::StartingBalance)
        .amount(Money::new(250))
        .submit()?;
    service.create_expense().amount(Money::new(100)).submit()?;

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let pie_chart = ElementHandle::find_by_element_type_name(&app, "PieChart")
        .next()
        .unwrap();

    let outer_total_spent = pie_chart
        .query_descendants()
        .match_id("PieChart::total-spent")
        .find_first()
        .unwrap();
    let inner_total_spent = pie_chart
        .query_descendants()
        .match_id("PieChart::inner-total-spent")
        .find_first()
        .unwrap();
    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    assert!(
        outer_total_spent
            .query_descendants()
            .match_accessible_label(formatter.format_money(Money::new(100))?)
            .find_first()
            .is_some()
    );
    assert!(
        inner_total_spent
            .query_descendants()
            .match_accessible_label(formatter.format_money(Money::new(100))?)
            .find_first()
            .is_some()
    );
    Ok(())
}

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

    let amount = formater.format_money(Money::new(10))?;
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
fn categories_shown_in_sidebar() -> Result<()> {
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
        outflow: SharedString::from("10"),
        ..Default::default()
    });
    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        category_id: categories[0].id.clone(),
        outflow: SharedString::from("20"),
        ..Default::default()
    });
    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        category_id: categories[1].id.clone(),
        outflow: SharedString::from("70"),
        ..Default::default()
    });

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SideBar")
        .next()
        .unwrap();

    let category_items = sidebar
        .query_descendants()
        .match_type_name("CategoryItem")
        .find_all();
    assert_eq!(category_items.len(), 2);

    let mut formater = CurrencyFormatter::new();
    formater.set_currency(Currency::USD);

    assert_eq!(
        category_items[0]
            .query_descendants()
            .match_id("CategoryItem::label")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        "Transport".to_shared_string()
    );
    assert_eq!(
        category_items[0]
            .query_descendants()
            .match_id("CategoryItem::amount")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        formater.format_money(Money::new(70))?.to_shared_string()
    );
    assert_eq!(
        category_items[0]
            .query_descendants()
            .match_id("CategoryItem::percent")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        "70%".to_shared_string()
    );
    assert_eq!(
        category_items[1]
            .query_descendants()
            .match_id("CategoryItem::label")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        "Entertainment".to_shared_string()
    );
    assert_eq!(
        category_items[1]
            .query_descendants()
            .match_id("CategoryItem::amount")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        formater.format_money(Money::new(30))?.to_shared_string()
    );
    assert_eq!(
        category_items[1]
            .query_descendants()
            .match_id("CategoryItem::percent")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        "30%".to_shared_string()
    );
    Ok(())
}

#[test]
fn less_than_one_percent_label() -> Result<()> {
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
        outflow: SharedString::from("0.5"),
        ..Default::default()
    });
    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        category_id: categories[1].id.clone(),
        outflow: SharedString::from("70"),
        ..Default::default()
    });

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let sidebar = ElementHandle::find_by_element_type_name(&app, "SideBar")
        .next()
        .unwrap();

    let category_items = sidebar
        .query_descendants()
        .match_type_name("CategoryItem")
        .find_all();

    let mut formater = CurrencyFormatter::new();
    formater.set_currency(Currency::USD);
    assert_eq!(
        category_items[1]
            .query_descendants()
            .match_id("CategoryItem::percent")
            .find_first()
            .unwrap()
            .accessible_label()
            .unwrap(),
        "<1%".to_shared_string()
    );
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
