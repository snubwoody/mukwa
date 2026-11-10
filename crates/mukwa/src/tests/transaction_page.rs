// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::ops::ControlFlow;

use crate::{
    tests::{
        find_all_by_type_name, find_by_accessible_label, find_by_id, find_by_type_name, setup,
    },
    ui::{
        self, AccountState, CategoryState, CreateTransactionOpts, Settings, State, TransactionState,
    },
};
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementRoot, mock_elapsed_time};
use jiff::Zoned;
use mukwa_core::{Currency, Money, fmt::CurrencyFormatter, service::AccountType};
use slint::{
    ComponentHandle, Model, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};

use super::Result;

#[test]
fn open_and_close_searchbox() -> Result<()> {
    let (app, _) = super::setup()?;

    let search = ElementHandle::find_by_element_type_name(&app, "Search")
        .next()
        .unwrap();

    assert!(
        search
            .query_descendants()
            .match_accessible_role(AccessibleRole::TextInput)
            .find_first()
            .is_none()
    );

    assert!(
        search
            .query_descendants()
            .match_accessible_label("Close")
            .find_first()
            .is_none()
    );

    search
        .query_descendants()
        .match_accessible_label("Search")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    assert!(
        search
            .query_descendants()
            .match_accessible_role(AccessibleRole::TextInput)
            .find_first()
            .is_some()
    );

    search
        .query_descendants()
        .match_accessible_label("Clear")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    assert!(
        search
            .query_descendants()
            .match_type_name("TextField")
            .find_first()
            .is_none()
    );

    assert!(
        search
            .query_descendants()
            .match_accessible_label("Clear")
            .find_first()
            .is_none()
    );
    Ok(())
}

#[test]
fn type_in_searchbox() -> Result<()> {
    let (app, service) = super::setup()?;
    let transaction_state: TransactionState = app.global();
    service.create_account("", AccountType::Cash)?;
    service.create_expense().note("Burger").submit()?;
    service.create_expense().note("Burger king").submit()?;
    service.create_income().submit()?;
    transaction_state.invoke_load_transactions();

    let search = ElementHandle::find_by_element_type_name(&app, "Search")
        .next()
        .unwrap();

    search
        .query_descendants()
        .match_accessible_label("Search")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    i_slint_backend_testing::send_keyboard_string_sequence(&app, "Burger");
    assert_eq!(transaction_state.get_transactions().iter().len(), 2);
    i_slint_backend_testing::send_keyboard_string_sequence(&app, " king");
    assert_eq!(transaction_state.get_transactions().iter().len(), 1);

    search
        .query_descendants()
        .match_accessible_label("Clear")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(transaction_state.get_transactions().iter().len(), 3);
    Ok(())
}

#[test]
fn total_balance_does_not_update_with_search() -> Result<()> {
    let (app, service) = super::setup()?;
    let transaction_state: TransactionState = app.global();
    let account_state: AccountState = app.global();
    let settings: Settings = app.global();
    settings.set_currency_code("USD".to_shared_string());
    service.create_account("", AccountType::Cash)?;
    service
        .create_income()
        .amount(Money::new(25))
        .note("Burger")
        .submit()?;
    service
        .create_income()
        .amount(Money::new(75))
        .note("Burger king")
        .submit()?;
    service.create_income().amount(Money::new(200)).submit()?;
    let root_element = app.root_element();
    transaction_state.invoke_load_transactions();
    account_state.invoke_load_accounts();

    let search = find_by_type_name(&root_element, "Search");
    find_by_accessible_label(&search, "Search").mock_single_click(PointerEventButton::Left);

    // Animation
    mock_elapsed_time(1000);

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    let root_element = app.root_element();
    let balance = find_by_id(&root_element, "TransactionPage::balance");

    i_slint_backend_testing::send_keyboard_string_sequence(&app, "Burger");
    assert_eq!(
        balance.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    i_slint_backend_testing::send_keyboard_string_sequence(&app, " king");
    assert_eq!(
        balance.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );

    find_by_accessible_label(&search, "Clear").mock_single_click(PointerEventButton::Left);
    assert_eq!(
        balance.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    Ok(())
}

#[test]
fn account_balance_does_not_update_with_search() -> Result<()> {
    let (app, service) = super::setup()?;
    let account_state: AccountState = app.global();
    let transaction_state: TransactionState = app.global();
    let settings: Settings = app.global();

    settings.set_currency_code("USD".to_shared_string());
    account_state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );
    let accounts = service.fetch_accounts()?;
    let account = &accounts[0];
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(25))
        .note("Burger")
        .submit()?;
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(75))
        .note("Burger king")
        .submit()?;
    service
        .create_income()
        .account(account.id)
        .amount(Money::new(200))
        .submit()?;

    let root_element = app.root_element();
    account_state.invoke_load_accounts();
    transaction_state.invoke_load_transactions();

    find_by_accessible_label(&root_element, "My account")
        .mock_single_click(PointerEventButton::Left);

    let search = find_by_type_name(&root_element, "Search");
    find_by_accessible_label(&search, "Search").mock_single_click(PointerEventButton::Left);

    mock_elapsed_time(1000);

    let mut formatter = CurrencyFormatter::new();
    formatter.set_currency(Currency::USD);

    let balance = find_by_id(&root_element, "TransactionPage::balance");

    i_slint_backend_testing::send_keyboard_string_sequence(&app, "Burger");
    assert_eq!(
        balance.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    i_slint_backend_testing::send_keyboard_string_sequence(&app, " king");
    assert_eq!(
        balance.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );

    find_by_accessible_label(&search, "Clear").mock_single_click(PointerEventButton::Left);
    assert_eq!(
        balance.accessible_label().unwrap(),
        formatter.format_money(Money::new(300))?
    );
    Ok(())
}

#[test]
fn editing_transaction_does_not_reload_all_transactions() -> Result<()> {
    let (app, service) = super::setup()?;
    let transaction_state: TransactionState = app.global();
    let settings: Settings = app.global();
    settings.set_currency_code("USD".to_shared_string());
    service.create_account("", AccountType::Cash)?;

    service.create_expense().amount(Money::new(50)).submit()?;
    service.create_income().amount(Money::new(50)).submit()?;
    transaction_state.invoke_load_transactions();

    service.create_income().amount(Money::new(50)).submit()?;
    Ok(())
}

#[test]
fn transactions_shown_in_table() -> Result<()> {
    let (app, service) = setup()?;
    let state = app.global::<State>();
    let transaction_state = app.global::<TransactionState>();
    let category_state = app.global::<CategoryState>();
    let settings = app.global::<Settings>();
    let account_state: AccountState = app.global();
    settings.invoke_set_currency_code("USD".to_shared_string());

    let group = service.create_category_group("")?;

    account_state.invoke_create_account(
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
    let transaction_state = app.global::<TransactionState>();
    let account_state: AccountState = app.global();
    account_state.invoke_create_account(
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

    let account = account_state.get_accounts().iter().next().unwrap();
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
    let account_state: AccountState = app.global();
    let category_state = app.global::<CategoryState>();
    account_state.invoke_create_account(
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
    let account_state: AccountState = app.global();

    let category_state = app.global::<CategoryState>();
    account_state.invoke_create_account(
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
    let account_state: AccountState = app.global();
    let transaction_state = app.global::<TransactionState>();
    account_state.invoke_create_account(
        "Cash Account".to_shared_string(),
        ui::AccountType::Cash,
        "0".to_shared_string(),
    );
    account_state.invoke_create_account(
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

    let accounts: Vec<_> = account_state.get_accounts().iter().collect();
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

    let accounts: Vec<_> = account_state.get_accounts().iter().collect();
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

#[test]
fn create_transaction_updates_account_balance() -> crate::Result<()> {
    let (app, _) = setup()?;
    let account_state: AccountState = app.global();
    app.global::<Settings>()
        .set_currency_code("CAD".to_shared_string());
    let formatter = CurrencyFormatter::with_currency(Currency::CAD);
    let root_element = app.root_element();

    account_state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );

    let page = find_by_type_name(&root_element, "TransactionPage");
    let create_transaction = |text: &str| {
        find_by_accessible_label(&page, "New").mock_single_click(PointerEventButton::Left);
        find_by_id(&page, "NewTransactionRow::outflow-cell")
            .mock_single_click(PointerEventButton::Left);
        // Clear the text field
        i_slint_backend_testing::send_key_combo(
            &app,
            [
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
            ],
        );
        i_slint_backend_testing::send_keyboard_string_sequence(&app, text);
        i_slint_backend_testing::send_keyboard_string_sequence(
            &app,
            &SharedString::from(Key::Return),
        );
        find_by_accessible_label(&page, "Save").mock_single_click(PointerEventButton::Left);
    };

    create_transaction("10");
    assert_eq!(
        find_by_id(&root_element, "AccountListItem::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-10))?
    );
    assert_eq!(
        find_by_id(&root_element, "TransactionPage::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-10))?
    );

    find_by_type_name(&root_element, "AccountListItem").mock_single_click(PointerEventButton::Left);
    create_transaction("50");

    assert_eq!(
        find_by_id(&root_element, "TransactionPage::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-60))?
    );
    Ok(())
}

#[test]
fn delete_transaction_updates_account_balance() -> crate::Result<()> {
    let (app, _) = setup()?;
    let account_state: AccountState = app.global();
    app.global::<Settings>()
        .set_currency_code("CAD".to_shared_string());
    let formatter = CurrencyFormatter::with_currency(Currency::CAD);
    let root_element = app.root_element();

    account_state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );

    let page = find_by_type_name(&root_element, "TransactionPage");
    let create_transaction = |text: &str| {
        find_by_accessible_label(&page, "New").mock_single_click(PointerEventButton::Left);
        find_by_id(&page, "NewTransactionRow::outflow-cell")
            .mock_single_click(PointerEventButton::Left);
        // Clear the text field
        i_slint_backend_testing::send_key_combo(
            &app,
            [
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
            ],
        );
        i_slint_backend_testing::send_keyboard_string_sequence(&app, text);
        i_slint_backend_testing::send_keyboard_string_sequence(
            &app,
            &SharedString::from(Key::Return),
        );
        find_by_accessible_label(&page, "Save").mock_single_click(PointerEventButton::Left);
    };

    create_transaction("10");
    create_transaction("40");
    create_transaction("60");

    let transactions = find_all_by_type_name(&page, "TransactionRow");
    find_by_type_name(&transactions[2], "Checkbox").mock_single_click(PointerEventButton::Left);
    find_by_accessible_label(&page, "Delete").mock_single_click(PointerEventButton::Left);
    assert_eq!(
        find_by_id(&root_element, "AccountListItem::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-100))?
    );
    assert_eq!(
        find_by_id(&root_element, "TransactionPage::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-100))?
    );

    find_by_type_name(&root_element, "AccountListItem").mock_single_click(PointerEventButton::Left);
    let transactions = find_all_by_type_name(&page, "TransactionRow");
    find_by_type_name(&transactions[0], "Checkbox").mock_single_click(PointerEventButton::Left);
    find_by_accessible_label(&page, "Delete").mock_single_click(PointerEventButton::Left);

    assert_eq!(
        find_by_id(&root_element, "TransactionPage::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-40))?
    );
    Ok(())
}

#[test]
fn edit_outflow_updates_account_balance() -> crate::Result<()> {
    let (app, service) = setup()?;
    let account_state: AccountState = app.global();
    let transaction_state: TransactionState = app.global();
    app.global::<Settings>()
        .set_currency_code("CAD".to_shared_string());
    let formatter = CurrencyFormatter::with_currency(Currency::CAD);
    let root_element = app.root_element();

    account_state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );

    service.create_expense().amount(Money::new(20)).submit()?;
    transaction_state.invoke_load_transactions();
    account_state.invoke_load_accounts();

    let edit_transaction = |text: &str| {
        find_by_id(&root_element, "TransactionRow::outflow-cell")
            .mock_single_click(PointerEventButton::Left);
        // Clear the text field
        i_slint_backend_testing::send_key_combo(
            &app,
            [
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
            ],
        );
        i_slint_backend_testing::send_keyboard_string_sequence(&app, text);
        i_slint_backend_testing::send_keyboard_string_sequence(
            &app,
            &SharedString::from(Key::Return),
        );
    };

    edit_transaction("10");
    assert_eq!(
        find_by_id(&root_element, "TransactionPage::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-10))?
    );
    assert_eq!(
        find_by_id(&root_element, "AccountListItem::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-10))?
    );

    find_by_type_name(&root_element, "AccountListItem").mock_single_click(PointerEventButton::Left);
    edit_transaction("50");

    assert_eq!(
        find_by_id(&root_element, "TransactionPage::balance")
            .accessible_label()
            .unwrap(),
        formatter.format_money(Money::new(-50))?
    );
    Ok(())
}

#[test]
fn audit_account_button_visibility() -> Result<()> {
    let (app, _) = setup()?;
    let root_element = app.root_element();
    let account_state: AccountState = app.global();
    account_state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );

    assert!(
        root_element
            .query_descendants()
            .match_accessible_role(AccessibleRole::Button)
            .match_accessible_label("Audit")
            .find_first()
            .is_none()
    );

    find_by_accessible_label(&root_element, "My account")
        .mock_single_click(PointerEventButton::Left);

    assert!(
        root_element
            .query_descendants()
            .match_accessible_role(AccessibleRole::Button)
            .match_accessible_label("Audit")
            .find_first()
            .is_some()
    );

    Ok(())
}

#[test]
fn audit_account_with_matching_balance() -> Result<()> {
    let (app, service) = setup()?;
    let settings: Settings = app.global();
    settings.set_currency_code("CAD".to_shared_string());

    let root_element = app.root_element();
    let account_state: AccountState = app.global();
    account_state.invoke_create_account(
        "My account".to_shared_string(),
        ui::AccountType::Cash,
        "".to_shared_string(),
    );
    service.create_expense().amount(Money::new(50)).submit()?;
    account_state.invoke_load_accounts();

    find_by_accessible_label(&root_element, "My account")
        .mock_single_click(PointerEventButton::Left);
    find_by_accessible_label(&root_element, "Audit").mock_single_click(PointerEventButton::Left);

    let audit_element = find_by_type_name(&root_element, "Audit");
    assert!(
        audit_element
            .query_descendants()
            .match_accessible_label("Is your bank account balance:")
            .find_first()
            .is_some()
    );

    let formatter = CurrencyFormatter::with_currency(Currency::CAD);
    let balance = formatter.format_money(Money::new(-50))?;
    assert!(
        audit_element
            .query_descendants()
            .match_accessible_label(balance)
            .find_first()
            .is_some()
    );

    find_by_accessible_label(&audit_element, "Yes").mock_single_click(PointerEventButton::Left);
    let transactions = service.fetch_transactions()?;
    assert!(transactions[0].audited);
    Ok(())
}
