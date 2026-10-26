// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::ops::ControlFlow;

use crate::ui::{CategoryState, CreateTransactionOpts, Settings, State, TransactionState};

use super::*;
use i_slint_backend_testing::ElementHandle;
use jiff::civil::date;
use jiff::{ToSpan, Zoned};
use mukwa_core::fmt::CurrencyFormatter;
use mukwa_core::service::AccountType;
use mukwa_core::{Currency, Money, create_test_db};
use slint::platform::PointerEventButton;
use slint::{Model, SharedString, ToSharedString};
use tempfile::tempdir;

mod analytics_page;
mod spending_page;

fn setup() -> crate::Result<(MainWindow, Service)> {
    i_slint_backend_testing::init_no_event_loop();
    let service = Service::new(create_test_db());
    let app = MainWindow::new()?;
    let state = AppState::new(service.clone())?;
    let settings = SettingsStore::open_in_memory();
    controllers::bind_all(&app, &state, &settings)?;
    Ok((app, service))
}

#[test]
fn total_spent_all_only_includes_expenses() -> Result<()> {
    i_slint_backend_testing::init_no_event_loop();
    let temp = tempdir()?;
    let service = init_service(temp.path())?;
    service.create_account("", AccountType::Cash)?;
    service.create_expense().amount(Money::new(200)).submit()?;
    service.create_expense().amount(Money::new(500)).submit()?;
    service
        .create_income()
        .amount(Money::new(10_000))
        .submit()?;
    let state = AppState::new(service)?;
    let settings = SettingsStore::open(temp.path().join("settings.toml"))?;
    let window = MainWindow::new()?;
    controllers::bind_all(&window, &state, &settings)?;
    let global_state = window.global::<ui::State>();
    let total = global_state.invoke_total_spent_all(Zoned::now().date().into());
    assert_eq!(total, Money::new(700).to_shared_string());
    Ok(())
}

#[test]
fn total_spent_all_filters_by_month() -> Result<()> {
    i_slint_backend_testing::init_no_event_loop();
    let temp = tempdir()?;
    let service = init_service(temp.path())?;
    service.create_account("", AccountType::Cash)?;
    service
        .create_expense()
        .date(date(2020, 2, 1))
        .amount(Money::new(200))
        .submit()?;
    service
        .create_expense()
        .date(date(2020, 1, 1))
        .amount(Money::new(500))
        .submit()?;

    let date = ui::Date {
        year: 2020,
        month: 1,
        day: 1,
    };
    let window = MainWindow::new()?;
    let state = AppState::new(service)?;
    let settings = SettingsStore::open(temp.path().join("settings.toml"))?;
    controllers::bind_all(&window, &state, &settings)?;
    let global_state = window.global::<ui::State>();
    let total = global_state.invoke_total_spent_all(date);
    assert_eq!(total, Money::new(500).to_shared_string());
    Ok(())
}

#[test]
fn draw_pie_chart_filters_by_date() -> Result<()> {
    i_slint_backend_testing::init_no_event_loop();
    let temp = tempdir()?;
    let service = init_service(temp.path())?;
    service.create_account("", AccountType::Cash)?;
    let group = service.create_category_group("")?;
    let category = service.create_category("Groceries", group.id)?;

    service
        .create_expense()
        .category(category.id)
        .date(Zoned::now().date() + 1.month())
        .amount(Money::new(100))
        .submit()?;
    service
        .create_expense()
        .category(category.id)
        .date(Zoned::now().date())
        .amount(Money::new(500))
        .submit()?;
    service
        .create_expense()
        .category(category.id)
        .date(Zoned::now().date() - 1.month())
        .amount(Money::new(200))
        .submit()?;

    let window = MainWindow::new()?;
    let state = AppState::new(service)?;
    let settings = SettingsStore::open(temp.path().join("settings.toml"))?;
    controllers::bind_all(&window, &state, &settings)?;
    let analytics = window.global::<ui::AnalyticsApi>();
    let date = Zoned::now().date() - 1.month();
    let slices = analytics.invoke_draw_pie_chart(500.0, 500.0, date.into());
    let slice = slices.iter().next().unwrap();

    assert_eq!(slices.iter().len(), 1);
    assert_eq!(slice.amount, Money::new(200).to_shared_string());
    Ok(())
}

#[test]
fn draw_pie_chart_sorts_categories() -> Result<()> {
    i_slint_backend_testing::init_no_event_loop();
    let temp = tempdir()?;
    let service = init_service(temp.path())?;
    service.create_account("", AccountType::Cash)?;
    let group = service.create_category_group("")?;
    let groceries = service.create_category("Groceries", group.id)?;
    let electricity = service.create_category("Electricity", group.id)?;
    let water = service.create_category("Water Bill", group.id)?;

    service
        .create_expense()
        .category(groceries.id)
        .amount(Money::new(500))
        .submit()?;
    service
        .create_expense()
        .category(water.id)
        .amount(Money::new(200))
        .submit()?;
    service
        .create_expense()
        .category(electricity.id)
        .amount(Money::new(70))
        .submit()?;

    let window = MainWindow::new()?;
    let state = AppState::new(service)?;
    let settings = SettingsStore::open(temp.path().join("settings.toml"))?;
    controllers::bind_all(&window, &state, &settings)?;
    let analytics = window.global::<ui::AnalyticsApi>();
    let pie_slices = analytics.invoke_draw_pie_chart(500.0, 500.0, Zoned::now().date().into());
    let mut slices = pie_slices.iter();

    assert_eq!(slices.len(), 3);

    let groceries_slice = slices.next().unwrap();
    let water_slice = slices.next().unwrap();
    let electricity_slice = slices.next().unwrap();

    assert_eq!(groceries_slice.amount, Money::new(500).to_shared_string());
    assert_eq!(groceries_slice.label, "Groceries");
    assert_eq!(water_slice.amount, Money::new(200).to_shared_string());
    assert_eq!(water_slice.label, "Water Bill");
    assert_eq!(electricity_slice.amount, Money::new(70).to_shared_string());
    assert_eq!(electricity_slice.label, "Electricity");
    Ok(())
}

#[test]
fn switch_pages() -> Result<()> {
    let (app, _) = setup()?;

    let transaction_page = ElementHandle::find_by_element_type_name(&app, "TransactionPage").next();
    let spending_page = ElementHandle::find_by_element_type_name(&app, "SpendingPage").next();
    let analytics_page = ElementHandle::find_by_element_type_name(&app, "AnalyticsPage").next();
    assert!(transaction_page.is_some());
    assert!(spending_page.is_none());
    assert!(analytics_page.is_none());

    ElementHandle::find_by_accessible_label(&app, "Spending")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let transaction_page = ElementHandle::find_by_element_type_name(&app, "TransactionPage").next();
    let spending_page = ElementHandle::find_by_element_type_name(&app, "SpendingPage").next();
    let analytics_page = ElementHandle::find_by_element_type_name(&app, "AnalyticsPage").next();
    assert!(spending_page.is_some());
    assert!(transaction_page.is_none());
    assert!(analytics_page.is_none());

    ElementHandle::find_by_accessible_label(&app, "Analytics")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let transaction_page = ElementHandle::find_by_element_type_name(&app, "TransactionPage").next();
    let spending_page = ElementHandle::find_by_element_type_name(&app, "SpendingPage").next();
    let analytics_page = ElementHandle::find_by_element_type_name(&app, "AnalyticsPage").next();
    assert!(analytics_page.is_some());
    assert!(transaction_page.is_none());
    assert!(spending_page.is_none());

    ElementHandle::find_by_accessible_label(&app, "All accounts")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let transaction_page = ElementHandle::find_by_element_type_name(&app, "TransactionPage").next();
    let spending_page = ElementHandle::find_by_element_type_name(&app, "SpendingPage").next();
    let analytics_page = ElementHandle::find_by_element_type_name(&app, "AnalyticsPage").next();
    assert!(transaction_page.is_some());
    assert!(spending_page.is_none());
    assert!(analytics_page.is_none());
    Ok(())
}

#[test]
fn create_account_helper_text() -> Result<()> {
    let (app, _) = setup()?;
    let state: State = app.global();

    let helper_text =
        "Create your first account. You can import transactions or add them manually.";
    let accounts: Vec<_> = state.get_accounts().iter().collect();
    assert!(accounts.is_empty());

    let page = ElementHandle::find_by_element_type_name(&app, "TransactionPage")
        .next()
        .unwrap();

    assert!(
        page.query_descendants()
            .match_predicate(|element| {
                element.accessible_label() == Some(helper_text.to_shared_string())
            })
            .find_first()
            .is_some()
    );

    page.query_descendants()
        .match_predicate(|element| {
            element.accessible_label() == Some("Create account".to_shared_string())
        })
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    ElementHandle::find_by_element_id(&app, "CreateAccountDialog::name")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    i_slint_backend_testing::send_keyboard_string_sequence(&app, &SharedString::from("My account"));

    ElementHandle::find_by_element_id(&app, "CreateAccountDialog::create-account")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let accounts: Vec<_> = state.get_accounts().iter().collect();
    assert_eq!(accounts[0].name, "My account".to_shared_string());

    assert!(
        page.query_descendants()
            .match_predicate(|element| {
                element.accessible_label() == Some(helper_text.to_shared_string())
            })
            .find_first()
            .is_none()
    );
    Ok(())
}

#[test]
fn accounts_shown_in_sidebar() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let settings = app.global::<Settings>();
    settings.invoke_set_currency_code("JPY".to_shared_string());

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "100".to_shared_string(),
    );
    state.invoke_create_account(
        "Credit Account".to_shared_string(),
        ui::AccountType::Credit,
        "550".to_shared_string(),
    );

    let account_list_items: Vec<_> =
        ElementHandle::find_by_element_type_name(&app, "AccountListItem").collect();
    assert_eq!(account_list_items.len(), 2);

    let cash_account = &account_list_items[0]
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    let credit_account = &account_list_items[1]
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::JPY);

    assert_eq!(cash_account[0].accessible_label().unwrap(), "Cash Account");
    // For cash accounts starting balance is an income
    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(100))?.to_shared_string()
    );
    assert_eq!(
        credit_account[0].accessible_label().unwrap(),
        "Credit Account"
    );
    // For credit accounts starting balance is an expense
    assert_eq!(
        credit_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-550))?.to_shared_string()
    );
    Ok(())
}

#[test]
fn create_transaction_updates_sidebar_account_balance() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let settings = app.global::<Settings>();
    settings.invoke_set_currency_code("USD".to_shared_string());

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    let account_list_items: Vec<_> =
        ElementHandle::find_by_element_type_name(&app, "AccountListItem").collect();

    let cash_account = &account_list_items[0]
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::ZERO)?.to_shared_string()
    );

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::new(),
        category_id: SharedString::new(),
        date: SharedString::new(),
        inflow: SharedString::new(),
        note: SharedString::new(),
        outflow: SharedString::from("500"),
        payee_id: SharedString::new(),
    });

    let cash_account = ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-500))?.to_shared_string()
    );
    Ok(())
}

#[test]
fn edit_transaction_updates_sidebar_account_balance() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let settings = app.global::<Settings>();
    settings.invoke_set_currency_code("USD".to_shared_string());

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::new(),
        category_id: SharedString::new(),
        date: SharedString::new(),
        inflow: SharedString::new(),
        note: SharedString::new(),
        outflow: SharedString::from("500"),
        payee_id: SharedString::new(),
    });

    let cash_account = ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_all();

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-500))?.to_shared_string()
    );

    let transaction = transaction_state.get_transactions().iter().next().unwrap();
    transaction_state.invoke_set_transaction_outflow(transaction.id, "200".to_shared_string());
    let cash_account = ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-200))?.to_shared_string()
    );
    Ok(())
}

#[test]
fn create_account_popup() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();

    ElementHandle::find_by_accessible_label(&app, "Create account")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    // TODO: maybe test close-on-click-outside

    ElementHandle::find_by_element_id(&app, "CreateAccountDialog::name")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    i_slint_backend_testing::send_keyboard_string_sequence(&app, &SharedString::from("My account"));

    let account_combobox =
        ElementHandle::find_by_element_id(&app, "CreateAccountDialog::account-type")
            .next()
            .unwrap();

    account_combobox.mock_single_click(PointerEventButton::Left);
    account_combobox.visit_descendants(|element| {
        if let Some(value) = element.accessible_value()
            && value == "credit"
        {
            element.mock_single_click(PointerEventButton::Left);
            return ControlFlow::Break(element);
        }
        ControlFlow::<ElementHandle>::Continue(())
    });

    ElementHandle::find_by_element_id(&app, "CreateAccountDialog::create-account")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let accounts: Vec<_> = state.get_accounts().iter().collect();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].name, "My account");
    assert_eq!(accounts[0].account_type, ui::AccountType::Credit);
    Ok(())
}

#[test]
fn edit_transaction_updates_account_balance() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let settings = app.global::<Settings>();
    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    settings.invoke_set_currency_code("USD".to_shared_string());

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::default(),
        category_id: SharedString::default(),
        date: SharedString::default(),
        inflow: SharedString::default(),
        note: SharedString::default(),
        outflow: SharedString::from("0"),
        payee_id: SharedString::default(),
    });

    ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let transaction_page = ElementHandle::find_by_element_type_name(&app, "TransactionPage")
        .next()
        .unwrap();

    let text_elements = transaction_page
        .query_descendants()
        .match_type_name("Text")
        .find_all();

    assert_eq!(
        text_elements[1].accessible_label().unwrap(),
        formatter.format_money(Money::ZERO)?.to_shared_string()
    );

    let transaction = transaction_state.get_transactions().iter().next().unwrap();
    transaction_state
        .invoke_set_transaction_outflow(transaction.id.clone(), "200".to_shared_string());

    assert_eq!(
        text_elements[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-200))?.to_shared_string()
    );

    transaction_state.invoke_set_transaction_inflow(transaction.id, "750".to_shared_string());
    assert_eq!(
        text_elements[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(750))?.to_shared_string()
    );
    Ok(())
}

#[test]
fn create_transaction_updates_account_balance() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let settings = app.global::<Settings>();
    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    settings.invoke_set_currency_code("USD".to_shared_string());

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::default(),
        category_id: SharedString::default(),
        date: SharedString::default(),
        inflow: SharedString::default(),
        note: SharedString::default(),
        outflow: SharedString::from("0"),
        payee_id: SharedString::default(),
    });

    ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let transaction_page = ElementHandle::find_by_element_type_name(&app, "TransactionPage")
        .next()
        .unwrap();

    let text_elements = transaction_page
        .query_descendants()
        .match_type_name("Text")
        .find_all();

    assert_eq!(
        text_elements[1].accessible_label().unwrap(),
        formatter.format_money(Money::ZERO)?.to_shared_string()
    );

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::default(),
        category_id: SharedString::default(),
        date: SharedString::default(),
        inflow: SharedString::default(),
        note: SharedString::default(),
        outflow: SharedString::from("200"),
        payee_id: SharedString::default(),
    });
    assert_eq!(
        text_elements[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-200))?.to_shared_string()
    );
    Ok(())
}

#[test]
fn delete_transaction_updates_sidebar_account_balance() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let settings = app.global::<Settings>();
    settings.invoke_set_currency_code("USD".to_shared_string());

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::new(),
        category_id: SharedString::new(),
        date: SharedString::new(),
        inflow: SharedString::new(),
        note: SharedString::new(),
        outflow: SharedString::from("500"),
        payee_id: SharedString::new(),
    });

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::new(),
        category_id: SharedString::new(),
        date: SharedString::new(),
        inflow: SharedString::new(),
        note: SharedString::new(),
        outflow: SharedString::from("500"),
        payee_id: SharedString::new(),
    });

    let cash_account = ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_all();

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter
            .format_money(Money::new(-1000))?
            .to_shared_string()
    );

    let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
    transaction_state.invoke_delete_transaction(transactions[0].id.clone());

    let cash_account = ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::new(-500))?.to_shared_string()
    );

    transaction_state.invoke_select_transaction(transactions[1].id.clone());
    transaction_state.invoke_delete_selected_transactions();

    let cash_account = ElementHandle::find_by_element_type_name(&app, "AccountListItem")
        .next()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_all();
    assert_eq!(
        cash_account[1].accessible_label().unwrap(),
        formatter.format_money(Money::ZERO)?.to_shared_string()
    );
    Ok(())
}

#[test]
fn transactions_shown_in_table() -> Result<()> {
    let (app, service) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let category_state = app.global::<CategoryState>();
    let settings = app.global::<Settings>();
    settings.invoke_set_currency_code("USD".to_shared_string());

    let group = service.create_category_group("")?;

    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    category_state.invoke_create_category(
        "Entertainment".to_shared_string(),
        group.id.to_shared_string(),
    );

    let category = category_state.get_categories().iter().next().unwrap();

    transaction_state.invoke_create_transaction(CreateTransactionOpts {
        account_id: SharedString::new(),
        category_id: category.id,
        date: SharedString::from("2026-10-12"),
        note: SharedString::from("Note"),
        inflow: SharedString::new(),
        outflow: SharedString::from("500"),
        payee_id: SharedString::new(),
    });

    let transaction_table = ElementHandle::find_by_element_type_name(&app, "Table")
        .next()
        .unwrap();

    let rows = transaction_table
        .query_descendants()
        .match_type_name("TransactionRow")
        .find_all();
    assert_eq!(rows.len(), 1);

    let test_has_text = |type_name: &str, text: &str| {
        let cell = rows[0]
            .query_descendants()
            .match_type_name(type_name)
            .find_first()
            .unwrap()
            .query_descendants()
            .match_type_name("Text")
            .find_first()
            .unwrap();

        assert_eq!(cell.accessible_label().unwrap(), SharedString::from(text));
    };

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    test_has_text("CategoryCell", "Entertainment");
    test_has_text("AccountCell", "Cash Account");
    test_has_text("MoneyCell", &formatter.format_money(Money::new(500))?);
    test_has_text("NoteCell", "Note");
    test_has_text(
        "DateCell",
        &state.invoke_format_date(SharedString::from("2026-10-12")),
    );
    Ok(())
}

#[test]
fn create_simple_transaction() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    let new_transaction_row =
        ElementHandle::find_by_element_type_name(&app, "NewTransactionRow").next();
    assert!(new_transaction_row.is_none());

    let button = ElementHandle::find_by_accessible_label(&app, "New")
        .next()
        .unwrap();

    button.mock_single_click(PointerEventButton::Left);

    let new_transaction_row = ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
        .next()
        .unwrap();

    new_transaction_row
        .query_descendants()
        .match_predicate(|element| element.accessible_label() == Some(SharedString::from("Save")))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let account = state.get_accounts().iter().next().unwrap();
    let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].date, Zoned::now().date().to_shared_string());
    assert_eq!(transactions[0].amount, Money::ZERO.to_shared_string());
    assert_eq!(transactions[0].account_id, account.id);
    assert!(transactions[0].category_id.is_empty());
    assert!(transactions[0].payee_id.is_empty());
    assert!(transactions[0].note.is_empty());

    assert!(
        ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
            .next()
            .is_none()
    );

    button.mock_single_click(PointerEventButton::Left);
    let new_transaction_row = ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
        .next()
        .unwrap();
    new_transaction_row
        .query_descendants()
        .match_predicate(|element| {
            element.accessible_label() == Some(SharedString::from("Save and add another"))
        })
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
    assert_eq!(transactions.len(), 2);

    assert!(
        ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
            .next()
            .is_some()
    );
    Ok(())
}

#[test]
fn new_transaction_row_payee_text() -> Result<()> {
    let (app, service) = setup()?;
    let state = app.global::<State>();
    let category_state = app.global::<CategoryState>();
    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    let group = service.create_category_group("")?;
    category_state.invoke_create_category(
        "Entertainment".to_shared_string(),
        group.id.to_shared_string(),
    );
    category_state
        .invoke_create_category("Groceries".to_shared_string(), group.id.to_shared_string());

    let button = ElementHandle::find_by_accessible_label(&app, "New")
        .next()
        .unwrap();
    button.mock_single_click(PointerEventButton::Left);

    let new_transaction_row = ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
        .next()
        .unwrap();

    let payee_text = new_transaction_row
        .query_descendants()
        .match_type_name("PayeeCell")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();

    assert!(payee_text.accessible_label().unwrap().is_empty());

    let category_cell = new_transaction_row
        .query_descendants()
        .match_type_name("CategoryCell")
        .find_first()
        .unwrap();

    let categories: Vec<_> = category_state.get_categories().iter().collect();

    category_cell.mock_single_click(PointerEventButton::Left);
    category_cell.visit_descendants(|element| {
        if let Some(value) = element.accessible_value()
            && value == categories[1].id
        {
            element.mock_single_click(PointerEventButton::Left);
            return ControlFlow::Break(element);
        }
        ControlFlow::<ElementHandle>::Continue(())
    });

    assert!(payee_text.accessible_label().unwrap().is_empty());
    Ok(())
}

#[test]
fn new_transaction_row_category_text() -> Result<()> {
    let (app, service) = setup()?;
    let state = app.global::<State>();
    let category_state = app.global::<CategoryState>();
    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );

    let group = service.create_category_group("")?;
    category_state.invoke_create_category(
        "Entertainment".to_shared_string(),
        group.id.to_shared_string(),
    );
    category_state
        .invoke_create_category("Groceries".to_shared_string(), group.id.to_shared_string());

    let button = ElementHandle::find_by_accessible_label(&app, "New")
        .next()
        .unwrap();
    button.mock_single_click(PointerEventButton::Left);

    let new_transaction_row = ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
        .next()
        .unwrap();

    let category_text = new_transaction_row
        .query_descendants()
        .match_type_name("CategoryCell")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();

    assert!(category_text.accessible_label().unwrap().is_empty());

    let category_cell = new_transaction_row
        .query_descendants()
        .match_type_name("CategoryCell")
        .find_first()
        .unwrap();

    let categories: Vec<_> = category_state.get_categories().iter().collect();

    category_cell.mock_single_click(PointerEventButton::Left);
    category_cell.visit_descendants(|element| {
        if let Some(value) = element.accessible_value()
            && value == categories[1].id
        {
            element.mock_single_click(PointerEventButton::Left);
            return ControlFlow::Break(element);
        }
        ControlFlow::<ElementHandle>::Continue(())
    });

    let category_text = new_transaction_row
        .query_descendants()
        .match_type_name("CategoryCell")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();
    assert_eq!(category_text.accessible_label().unwrap(), "Groceries");
    Ok(())
}

#[test]
fn new_transaction_row_select_account() -> Result<()> {
    let (app, _) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );
    state.invoke_create_account(
        "Credit Account".to_shared_string(),
        ui::AccountType::Credit,
        "0".to_shared_string(),
    );

    let button = ElementHandle::find_by_accessible_label(&app, "New")
        .next()
        .unwrap();
    button.mock_single_click(PointerEventButton::Left);

    let new_transaction_row = ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
        .next()
        .unwrap();

    let account_text = new_transaction_row
        .query_descendants()
        .match_type_name("AccountCell")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();

    assert_eq!(account_text.accessible_label().unwrap(), "Cash Account");

    let account_cell = new_transaction_row
        .query_descendants()
        .match_type_name("AccountCell")
        .find_first()
        .unwrap();

    let accounts: Vec<_> = state.get_accounts().iter().collect();
    account_cell.mock_single_click(PointerEventButton::Left);
    account_cell.visit_descendants(|element| {
        if let Some(value) = element.accessible_value()
            && value == accounts[1].id
        {
            element.mock_single_click(PointerEventButton::Left);
            return ControlFlow::Break(element);
        }
        ControlFlow::<ElementHandle>::Continue(())
    });

    let account_cell = new_transaction_row
        .query_descendants()
        .match_type_name("AccountCell")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();
    assert_eq!(account_cell.accessible_label().unwrap(), "Credit Account");

    new_transaction_row
        .query_descendants()
        .match_predicate(|element| element.accessible_label() == Some(SharedString::from("Save")))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let accounts: Vec<_> = state.get_accounts().iter().collect();
    let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].account_id, accounts[1].id);
    Ok(())
}

#[test]
fn cancel_new_transaction_row() -> Result<()> {
    let (app, _) = setup()?;

    let new_transaction_row =
        ElementHandle::find_by_element_type_name(&app, "NewTransactionRow").next();
    assert!(new_transaction_row.is_none());

    let button = ElementHandle::find_by_accessible_label(&app, "New")
        .next()
        .unwrap();

    button.mock_single_click(PointerEventButton::Left);

    let new_transaction_row = ElementHandle::find_by_element_type_name(&app, "NewTransactionRow")
        .next()
        .unwrap();

    new_transaction_row
        .query_descendants()
        .match_predicate(|element| element.accessible_label() == Some(SharedString::from("Cancel")))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    let new_transaction_row =
        ElementHandle::find_by_element_type_name(&app, "NewTransactionRow").next();
    assert!(new_transaction_row.is_none());
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[test]
fn title_bar_hidden() -> Result<()> {
    let (app, _) = setup()?;

    assert!(
        ElementHandle::find_by_element_type_name(&app, "TitleBar")
            .next()
            .is_none()
    );
    Ok(())
}

#[cfg(target_os = "windows")]
#[test]
fn title_bar_shows_update_state() -> Result<()> {
    let (app, _) = setup()?;

    let title_bar = ElementHandle::find_by_element_type_name(&app, "TitleBar")
        .next()
        .unwrap();
    assert!(
        title_bar
            .query_descendants()
            .match_accessible_label("Downloading update")
            .find_first()
            .is_none()
    );
    assert!(
        title_bar
            .query_descendants()
            .match_type_name("ProgressBar")
            .find_first()
            .is_none()
    );

    let updater: ui::AutoUpdater = app.global();
    updater.set_update_state(ui::AutoUpdaterState::Downloading);
    assert!(
        title_bar
            .query_descendants()
            .match_accessible_label("Downloading update")
            .find_first()
            .is_some()
    );

    updater.set_download_progress(0.8);
    let progress_bar = title_bar
        .query_descendants()
        .match_type_name("ProgressBar")
        .find_first()
        .unwrap();
    assert_eq!(progress_bar.accessible_value().unwrap(), "0.8");

    updater.set_update_state(ui::AutoUpdaterState::Ready);
    assert!(
        title_bar
            .query_descendants()
            .match_accessible_label("Update & Restart")
            .find_first()
            .is_some()
    );
    Ok(())
}
