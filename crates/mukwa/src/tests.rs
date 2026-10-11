use std::time::Duration;

use crate::ui::{Settings, State};

use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery, ElementRoot};
use jiff::civil::date;
use jiff::{ToSpan, Zoned};
use mukwa_core::fmt::CurrencyFormatter;
use mukwa_core::service::AccountType;
use mukwa_core::{Currency, Money, create_test_db};
use slint::platform::PointerEventButton;
use slint::{Model, ToSharedString};
use tempfile::tempdir;

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
fn accounts_shown_in_sidebar() -> Result<()> {
    let (app, _) = setup()?;

    let state = app.global::<State>();
    let settings = app.global::<Settings>();
    settings.invoke_set_currency_code("JPY".to_shared_string());
    // TODO: set currency code
    dbg!(settings.get_currency_code());
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
    //i_slint_backend_testing::mock_elapsed_time(Duration::from_nanos(10));
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
