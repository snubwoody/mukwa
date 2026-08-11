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
use mukwa::ui::CreateTransactionOpts;
use mukwa::{Money, create_test_db};
use slint::{Model, SharedString, ToSharedString};

#[test]
fn create_expense() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;
    let group = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;

    let mut state = AppState::new(service)?;
    let opts = CreateTransactionOpts {
        account_id: account.id.to_shared_string(),
        outflow: "0.00".to_shared_string(),
        category_id: category.id.to_shared_string(),
        date: Zoned::now().date().to_shared_string(),
        note: SharedString::from("Pick n Pay"),
        ..Default::default()
    };
    state.create_transaction(opts)?;

    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.account_id, account.id.to_shared_string());
    assert_eq!(transaction.category_id, category.id.to_shared_string());
    assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
    assert_eq!(transaction.note.as_str(), "Pick n Pay");
    assert_eq!(transaction.inflow.as_str(), "");
    assert_eq!(transaction.payee_id.as_str(), "");
    assert_eq!(transaction.outflow.as_str(), Money::ZERO.to_string());
    Ok(())
}

#[test]
fn create_income() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let account = service.create_account("")?;

    let mut state = AppState::new(service)?;
    let opts = CreateTransactionOpts {
        account_id: account.id.to_shared_string(),
        inflow: "0.00".to_shared_string(),
        date: Zoned::now().date().to_shared_string(),
        note: SharedString::from("Pick n Pay"),
        ..Default::default()
    };
    state.create_transaction(opts)?;

    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.account_id, account.id.to_shared_string());
    assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
    assert_eq!(transaction.note.as_str(), "Pick n Pay");
    assert_eq!(transaction.outflow.as_str(), "");
    assert_eq!(transaction.payee_id.as_str(), "");
    assert_eq!(transaction.inflow.as_str(), Money::ZERO.to_string());
    Ok(())
}

#[test]
fn create_expense_uses_account() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    service.create_account("")?;
    service.create_account("")?;
    service.create_account("")?;
    let account = service.create_account("")?;
    let group = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;

    let mut state = AppState::new(service)?;
    let opts = CreateTransactionOpts {
        account_id: account.id.to_shared_string(),
        outflow: "0.00".to_shared_string(),
        category_id: category.id.to_shared_string(),
        date: Zoned::now().date().to_shared_string(),
        note: SharedString::from("Pick n Pay"),
        ..Default::default()
    };
    state.create_transaction(opts)?;

    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.account_id, account.id.to_shared_string());
    Ok(())
}

#[test]
fn create_expense_empty_category() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    service.create_account("")?;

    let mut state = AppState::new(service)?;
    let opts = CreateTransactionOpts {
        outflow: "0.00".to_shared_string(),
        date: Zoned::now().date().to_shared_string(),
        ..Default::default()
    };
    state.create_transaction(opts)?;
    Ok(())
}

#[test]
fn create_category_creates_a_budget() -> mukwa::Result<()> {
    let connection = create_test_db();
    let service = Service::new(connection);
    let group = service.create_category_group("")?;
    let mut state = AppState::new(service.clone())?;

    state.create_category("Groceries", &group.id.to_string())?;

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
    let group = service.create_category_group("")?;
    let mut state = AppState::new(service.clone())?;

    state.set_current_budget_month(date(2020, 1, 1))?;
    state.create_category("Groceries", &group.id.to_string())?;
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
fn create_category_group_adds_to_list() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    let mut state = AppState::new(service)?;
    state.create_category_group("")?;
    state.create_category_group("")?;

    assert_eq!(state.category_groups().iter().len(), 2);
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
    let group = service.create_category_group("")?;
    service.create_category(Default::default(), group.id)?;
    service.create_category(Default::default(), group.id)?;

    let state = AppState::new(service)?;
    assert_eq!(state.categories().iter().len(), 2);
    Ok(())
}

#[test]
fn calculate_total_spent() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;
    let group = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;
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
    let group = service.create_category_group("")?;
    let category = service.create_category(Default::default(), group.id)?;
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
fn delete_categories_resets_transactions() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;
    let group = service.create_category_group("")?;
    let category = service.create_category("", group.id)?;
    let expense = service.create_expense().category(category.id).submit()?;

    let mut state = AppState::new(service)?;
    state.delete_category(&category.id.to_string())?;

    dbg!(state.transactions().iter().len());
    let transaction = state.transactions().remove(0);
    assert_eq!(transaction.category_id, category.id.to_shared_string());
    Ok(())
}

#[test]
fn left_to_spend_caps_at_zero() -> mukwa::Result<()> {
    let service = Service::open_in_memory()?;
    service.create_account("")?;
    let group = service.create_category_group("")?;
    let category = service.create_category(Default::default(), group.id)?;
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
