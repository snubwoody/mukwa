// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use jiff::Zoned;
use jiff::civil::date;
use mukwa::service::{CreateBudgetOpts, Service};
use mukwa::state::AppState;
use mukwa::{Money, create_test_db};
use slint::Model;

#[test]
fn create_transaction_creates_expense() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let mut state = AppState::new(service)?;
    state.create_account("")?;
    state.create_transaction()?;

    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.inflow.as_str(), "");
    assert_eq!(transaction.outflow.as_str(), Money::ZERO.to_string());
    Ok(())
}

#[test]
fn create_category_creates_a_budget() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let mut state = AppState::new(service.clone())?;

    state.create_category("Groceries")?;

    let categories = service.fetch_categories()?;
    let budgets = service.fetch_budgets_by_month(Zoned::now().date())?;
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].category_id, categories[0].id);
    Ok(())
}

#[test]
fn create_category_creates_a_budget_in_current_month() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let mut state = AppState::new(service.clone())?;

    state.set_current_budget_month(date(2020, 1, 1))?;
    state.create_category("Groceries")?;
    let categories = service.fetch_categories()?;
    let budgets = service.fetch_budgets_by_month(date(2020, 1, 1))?;
    assert_eq!(budgets.len(), 1);
    assert_eq!(budgets[0].year, 2020);
    assert_eq!(budgets[0].month, 1);
    assert_eq!(budgets[0].category_id, categories[0].id);
    Ok(())
}

#[test]
fn create_account_adds_to_account_list() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let mut state = AppState::new(service)?;
    state.create_account("")?;
    state.create_account("")?;

    assert_eq!(state.accounts().iter().len(), 2);
    Ok(())
}

#[test]
fn create_account_adds_to_account_options() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let mut state = AppState::new(service)?;
    state.create_account("")?;
    state.create_account("")?;

    assert_eq!(state.account_options().iter().len(), 2);
    Ok(())
}

#[test]
fn state_loads_data_from_service() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);

    service.create_account("")?;
    service.create_account("")?;

    service.create_expense().submit()?;
    service.create_expense().submit()?;
    service.create_expense().submit()?;

    let state = AppState::new(service)?;
    assert_eq!(state.transactions().iter().len(), 3);
    assert_eq!(state.accounts().iter().len(), 2);
    Ok(())
}

#[test]
fn state_loads_categories_from_service() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_category(Default::default())?;
    service.create_category(Default::default())?;

    let state = AppState::new(service)?;
    assert_eq!(state.categories().iter().len(), 2);
    Ok(())
}

#[test]
fn calculate_total_spent() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;
    let category = service.create_category(Default::default())?;
    service
        .create_expense()
        .amount(Money::new(500))
        .category(category.id)
        .submit()?;

    let budget = service.create_budget(CreateBudgetOpts {
        category_id: category.id,
        ..Default::default()
    })?;
    let state = AppState::new(service)?;
    let total = state.total_spent(budget.id.to_string().as_str())?;
    assert_eq!(total, Money::new(500));
    Ok(())
}

#[test]
fn calculate_total_spent_only_includes_current_month() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;
    let category = service.create_category(Default::default())?;
    service
        .create_expense()
        .amount(Money::new(500))
        .category(category.id)
        .submit()?;
    service
        .create_expense()
        .amount(Money::new(500))
        .category(category.id)
        .date(date(1990, 1, 1))
        .submit()?;
    let budget = service.create_budget(CreateBudgetOpts {
        category_id: category.id,
        ..Default::default()
    })?;
    let state = AppState::new(service)?;
    let total = state.total_spent(budget.id.to_string().as_str())?;
    assert_eq!(total, Money::new(500));
    Ok(())
}

#[test]
fn left_to_spend_caps_at_zero() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;
    let category = service.create_category(Default::default())?;
    let budget = service.create_budget(CreateBudgetOpts {
        category_id: category.id,
        amount: Some(Money::new(200)),
        month: Some(Zoned::now().date()),
    })?;
    service
        .create_expense()
        .amount(Money::new(500))
        .date(Zoned::now().date())
        .category(category.id)
        .submit()?;
    let state = AppState::new(service)?;
    let total = state.left_to_spend(budget.id.to_string().as_str())?;
    assert_eq!(total, Money::ZERO);
    Ok(())
}
