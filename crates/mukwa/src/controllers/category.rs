// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui;
use crate::ui::{CategoryState, MainWindow};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::service::{CreateBudgetOpts, Service};
use slint::{
    ComponentHandle, DataTransfer, Global, Model, ModelExt, ModelRc, SharedString, ToSharedString,
    VecModel,
};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{info, warn};
use uuid::Uuid;

pub fn bind(window: &MainWindow, state: &AppState) -> crate::Result<()> {
    let category_state = window.global::<CategoryState>();

    let service = state.service();
    let categories = service.fetch_categories()?;
    let category_list: Vec<ui::Category> = categories.iter().map(|c| c.into()).collect();
    let category_options: Vec<ui::ComboBoxItem> = categories.iter().map(|c| c.into()).collect();
    let category_groups = service.fetch_category_groups()?;
    let category_group_list: Vec<ui::CategoryGroup> =
        category_groups.iter().map(|c| c.into()).collect();

    let category_model = Rc::new(VecModel::from(category_list));
    let category_group_model = Rc::new(VecModel::from(category_group_list));
    let category_options_model = Rc::new(VecModel::from(category_options));

    let budgets_list: Vec<ui::Budget> = service
        .fetch_budgets_by_month(Zoned::now().date())?
        .iter()
        .map(|b| b.into())
        .collect();
    let budget_model = Rc::new(VecModel::from(budgets_list));

    category_state.set_categories(ModelRc::new(category_model));
    category_state.set_category_groups(ModelRc::new(category_group_model));
    category_state.set_budgets(ModelRc::new(budget_model));
    // We can't map arrays in slint so we have to maintain duplicate arrays for comboboxes
    // see <https://github.com/slint-ui/slint/issues/1328>
    category_state.set_category_options(ModelRc::new(category_options_model));
    category_state.set_current_month(Zoned::now().date().into());

    category_state.on_left_to_assign({
        let service = service.clone();
        move || {
            service
                .left_to_assign()
                .inspect_err(|err| warn!("Error calculating left to assign: {err}"))
                .unwrap_or_default()
                .to_shared_string()
        }
    });

    category_state.on_get_category({
        let category_state = category_state.as_weak();
        move |id| {
            category_state
                .unwrap()
                .get_categories()
                .iter()
                .find(|c| c.id == id)
                .unwrap_or_default()
        }
    });

    category_state.on_categories_in_group({
        let category_state = category_state.as_weak();
        move |group_id| {
            let filtered_categories = category_state
                .unwrap()
                .get_categories()
                .filter(move |category| category.group_id == group_id);

            ModelRc::new(filtered_categories)
        }
    });

    category_state.on_get_budget({
        let category_state = category_state.as_weak();
        move |category_id, date| {
            category_state
                .unwrap()
                .get_budgets()
                .iter()
                .find(|budget| {
                    budget.category_id == category_id
                        && budget.month == date.month
                        && budget.year == date.year
                })
                .unwrap_or_default()
        }
    });

    category_state.on_create_category({
        let category_state = category_state.as_weak();
        let service = service.clone();
        move |title, group_id| {
            let category_state = category_state.unwrap();
            match create_category(&category_state, &service, &title, &group_id) {
                Ok(id) => {
                    info!("Created new category ({id})")
                }
                Err(err) => {
                    warn!("Failed to create category: {err}")
                }
            }
        }
    });

    category_state.on_create_category_group({
        let category_state = category_state.as_weak();
        let service = service.clone();
        move |title| {
            let category_state = category_state.unwrap();
            match create_category_group(&category_state, &service, &title) {
                Ok(id) => {
                    info!("Created new category group ({id})")
                }
                Err(err) => {
                    warn!("Failed to create category group: {err}")
                }
            }
        }
    });

    category_state.on_delete_category({
        let service = service.clone();
        let category_state = category_state.as_weak();
        move |id| {
            let category_state = category_state.unwrap();
            match delete_category(&category_state, &service, &id) {
                Ok(id) => {
                    info!("Deleted category {id}");
                }
                Err(err) => {
                    warn!("Failed to delete category: {err}")
                }
            }
        }
    });

    category_state.on_delete_category_group({
        let service = service.clone();
        let category_state = category_state.as_weak();
        move |id| {
            let category_state = category_state.unwrap();
            match delete_category_group(&category_state, &service, &id) {
                Ok(id) => {
                    info!("Deleted category group {id}");
                }
                Err(err) => {
                    warn!("Failed to delete category group: {err}")
                }
            }
        }
    });

    category_state.on_edit_budget({
        let category_state = category_state.as_weak();
        let service = service.clone();

        move |id, amount| {
            let category_state = category_state.unwrap();
            if let Err(err) = update_budget(&category_state, &service, &id, &amount) {
                warn!("Failed to update budget: {err}");
            }
        }
    });

    category_state.on_update_category({
        let category_state = category_state.as_weak();
        let service = service.clone();
        move |id, title| {
            let category_state = category_state.unwrap();
            if let Err(err) = update_category(&category_state, &service, &id, &title) {
                warn!("Failed to update category: {err}");
            }
        }
    });

    category_state.on_update_category_group({
        let category_state = category_state.as_weak();
        let service = service.clone();
        move |id, title| {
            let category_state = category_state.unwrap();
            if let Err(err) = update_category_group(&category_state, &service, &id, &title) {
                warn!("Failed to update category group: {err}");
            }
        }
    });

    category_state.on_move_category({
        let category_state = category_state.as_weak();
        let service = service.clone();
        move |id, group_id| {
            let category_state = category_state.unwrap();
            if let Err(err) = move_category(&category_state, &service, &id, &group_id) {
                warn!("Failed to move category: {err}");
            }
        }
    });

    category_state.on_total_spent_in_group({
        let service = service.clone();
        move |id, date| {
            total_spent_in_group(&service, &id, date)
                .unwrap_or_else(|err| {
                    warn!("Failed to calculate total spent in category group: {err}");
                    Money::ZERO
                })
                .to_shared_string()
        }
    });

    category_state.on_left_to_spend_in_group({
        let service = service.clone();
        move |id, date| {
            left_to_spend_in_group(&service, &id, date)
                .unwrap_or_else(|err| {
                    warn!("Failed to calculate left to spend in category group: {err}");
                    Money::ZERO
                })
                .to_shared_string()
        }
    });

    category_state.on_total_assigned_in_group({
        let service = service.clone();
        move |id, date| {
            total_assigned_in_group(&service, &id, date)
                .unwrap_or_else(|err| {
                    warn!("Failed to calculate total assigned in category group: {err}");
                    Money::ZERO
                })
                .to_shared_string()
        }
    });

    category_state.on_total_spent({
        let service = service.clone();
        move |id| {
            total_spent(&service, &id)
                .unwrap_or_else(|err| {
                    warn!("Failed to calculate total spent: {err}");
                    Money::ZERO
                })
                .to_shared_string()
        }
    });

    category_state.on_left_to_spend({
        let service = service.clone();
        move |id| {
            left_to_spend(&service, &id)
                .unwrap_or_else(|err| {
                    warn!("Failed to calculate left to spend: {err}");
                    Money::ZERO
                })
                .to_shared_string()
        }
    });

    category_state.on_category_to_transfer(DataTransfer::from);

    category_state.on_transfer_to_category(|data| {
        data.plain_text().unwrap_or_else(|err| {
            warn!("{err}");
            SharedString::new()
        })
    });

    Ok(())
}

fn load_categories(state: &CategoryState, service: &Service) -> crate::Result<()> {
    let categories: Vec<ui::Category> = service
        .fetch_categories()?
        .iter()
        .map(|c| c.into())
        .collect();

    state.set_categories(ModelRc::new(Rc::new(VecModel::from(categories))));
    Ok(())
}

fn load_category_groups(state: &CategoryState, service: &Service) -> crate::Result<()> {
    let groups: Vec<ui::CategoryGroup> = service
        .fetch_category_groups()?
        .iter()
        .map(|c| c.into())
        .collect();

    state.set_category_groups(ModelRc::new(Rc::new(VecModel::from(groups))));
    Ok(())
}

fn load_budgets(state: &CategoryState, service: &Service) -> crate::Result<()> {
    let month = Date::try_from(state.get_current_month())?;
    let budgets: Vec<ui::Budget> = service
        .fetch_budgets_by_month(month)?
        .iter()
        .map(|b| b.into())
        .collect();
    state.set_budgets(ModelRc::new(Rc::new(VecModel::from(budgets))));
    Ok(())
}

fn delete_category(state: &CategoryState, service: &Service, id: &str) -> crate::Result<Uuid> {
    let id = Uuid::parse_str(id)?;
    service.delete_category(id)?;
    load_categories(state, service)?;
    load_budgets(state, service)?;
    Ok(id)
}

fn delete_category_group(
    state: &CategoryState,
    service: &Service,
    id: &str,
) -> crate::Result<Uuid> {
    let id = Uuid::parse_str(id)?;
    service.delete_category_group(id)?;
    load_budgets(state, service)?;
    load_category_groups(state, service)?;
    load_categories(state, service)?;
    Ok(id)
}

fn left_to_spend_in_group(service: &Service, id: &str, date: ui::Date) -> crate::Result<Money> {
    let total = total_spent_in_group(service, id, date.clone())?;
    let id = Uuid::parse_str(id)?;
    let date = Date::try_from(date)?;
    let assigned = service.total_assigned_in_group(id, date)?;
    let available = assigned - total;
    Ok(available.max(Money::ZERO))
}

fn total_spent_in_group(service: &Service, id: &str, date: ui::Date) -> crate::Result<Money> {
    let id = Uuid::parse_str(id)?;
    let date = Date::try_from(date)?;
    let total = service.total_spent_in_group(id, date)?;
    Ok(total)
}

fn total_spent(service: &Service, id: &str) -> crate::Result<Money> {
    let id = Uuid::parse_str(id)?;
    let budget = service.get_budget(id)?;
    let date = Date::new(budget.year as i16, budget.month as i8, 1)?;
    let total = service.total_spent(budget.category_id, date)?;
    Ok(total)
}

fn left_to_spend(service: &Service, id: &str) -> crate::Result<Money> {
    let total = total_spent(service, id)?;
    let id = Uuid::parse_str(id)?;
    let budget = service.get_budget(id)?;
    let available = budget.amount - total;
    Ok(available.max(Money::ZERO))
}

fn total_assigned_in_group(service: &Service, id: &str, date: ui::Date) -> crate::Result<Money> {
    let id = Uuid::parse_str(id)?;
    let date = Date::try_from(date)?;
    let total = service.total_assigned_in_group(id, date)?;
    Ok(total)
}

fn update_budget(
    state: &CategoryState,
    service: &Service,
    id: &str,
    amount: &str,
) -> crate::Result<()> {
    let budget_id = Uuid::parse_str(id)?;
    let amount = Money::from_str(amount)?;
    service.update_budget(budget_id, amount)?;
    load_budgets(state, service)?;
    load_categories(state, service)?;
    load_category_groups(state, service)?;
    Ok(())
}

fn move_category(
    state: &CategoryState,
    service: &Service,
    id: &str,
    group_id: &str,
) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let group_id = Uuid::parse_str(group_id)?;
    service.move_category(id, group_id)?;
    load_categories(state, service)?;
    load_category_groups(state, service)?;
    load_budgets(state, service)?;
    Ok(())
}

fn create_category(
    state: &CategoryState,
    service: &Service,
    title: &str,
    group_id: &str,
) -> crate::Result<Uuid> {
    let group_id = Uuid::parse_str(group_id)?;
    let category = service.create_category(title, group_id)?;

    let id = category.id;
    let date = Date::try_from(state.get_current_month())?;
    service.create_budget(CreateBudgetOpts {
        amount: Some(Money::ZERO),
        month: Some(date),
        category_id: category.id,
    })?;
    state
        .get_category_options()
        .push_row(category.clone().into())?;
    state.get_categories().push_row(category.into())?;
    load_budgets(state, service)?;
    Ok(id)
}

fn update_category(
    state: &CategoryState,
    service: &Service,
    id: &str,
    title: &str,
) -> crate::Result<Uuid> {
    let id = Uuid::parse_str(id)?;
    service.update_category(id, title)?;
    load_categories(state, service)?;
    load_budgets(state, service)?;
    Ok(id)
}

fn update_category_group(
    state: &CategoryState,
    service: &Service,
    id: &str,
    title: &str,
) -> crate::Result<Uuid> {
    let id = Uuid::parse_str(id)?;
    service.update_category_group(id, title)?;
    service.update_category(id, title)?;
    load_categories(state, service)?;
    load_category_groups(state, service)?;
    load_budgets(state, service)?;
    Ok(id)
}

fn create_category_group(
    state: &CategoryState,
    service: &Service,
    title: &str,
) -> crate::Result<Uuid> {
    let category_group = service.create_category_group(title)?;
    let id = category_group.id;
    state
        .get_category_groups()
        .push_row(category_group.into())?;
    Ok(id)
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::civil::date;
    use mukwa_core::service::{AccountType, CreateBudgetOpts};

    #[test]
    fn bind_properties() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let state = AppState::new(service)?;
        let window = MainWindow::new()?;
        bind(&window, &state)?;

        let category_state = window.global::<CategoryState>();
        let current_month = category_state.get_current_month();
        assert_eq!(current_month, Zoned::now().date().into());
        Ok(())
    }

    #[test]
    fn create_category_adds_to_combobox_list() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let state = AppState::new(service)?;
        let window = MainWindow::new()?;
        bind(&window, &state)?;
        let category_state = window.global::<CategoryState>();
        category_state.invoke_create_category("Entertainment".into(), group.id.to_shared_string());
        let combobox_options = category_state.get_category_options();
        assert_eq!(combobox_options.iter().len(), 1);
        let option = combobox_options.iter().next().unwrap();
        assert_eq!(option.text, "Entertainment");
        Ok(())
    }

    #[test]
    fn delete_category_group() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        service.create_category("", group.id)?;
        let state = AppState::new(service)?;
        let window = MainWindow::new()?;
        bind(&window, &state)?;

        let category_state = window.global::<CategoryState>();
        let groups = category_state.get_category_groups();
        let categories = category_state.get_categories();
        assert_eq!(groups.iter().len(), 2);
        assert_eq!(categories.iter().len(), 1);

        category_state.invoke_delete_category_group(group.id.to_shared_string());

        let categories = category_state.get_categories();
        let groups = category_state.get_category_groups();
        assert_eq!(groups.iter().len(), 1);
        assert_eq!(categories.iter().len(), 0);
        Ok(())
    }

    #[test]
    fn create_category_creates_a_budget_in_current_month() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let state = AppState::new(service.clone())?;
        let window = MainWindow::new()?;
        bind(&window, &state)?;

        let category_state = window.global::<CategoryState>();

        category_state.set_current_month(date(2020, 1, 1).into());
        category_state.invoke_create_category("".to_shared_string(), group.id.to_shared_string());
        let categories = service.fetch_categories()?;
        let budgets = service.fetch_budgets_by_month(date(2020, 1, 1))?;
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].year, 2020);
        assert_eq!(budgets[0].month, 1);
        assert_eq!(budgets[0].category_id, categories[0].id);
        Ok(())
    }

    #[test]
    fn edit_budget() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;
        let budget = service.create_budget(CreateBudgetOpts {
            amount: Some(Money::new(50)),
            category_id: category.id,
            month: Some(date(2020, 1, 1)),
        })?;
        let state = AppState::new(service)?;
        let window = MainWindow::new()?;
        bind(&window, &state)?;

        let category_state = window.global::<CategoryState>();
        category_state.set_current_month(date(2020, 1, 1).into());
        category_state.invoke_edit_budget(budget.id.to_shared_string(), "500".to_shared_string());
        let budgets = category_state.get_budgets();
        let budget = budgets.iter().next().unwrap();
        assert_eq!(budget.amount, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn calculate_total_spent() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        service.create_account("", AccountType::Cash)?;
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
        let window = MainWindow::new()?;
        bind(&window, &state)?;
        let category_state: CategoryState = window.global();
        let total = category_state.invoke_total_spent(budget.id.to_shared_string());
        assert_eq!(total, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn calculate_total_spent_only_includes_current_month() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        service.create_account("", AccountType::Cash)?;
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
        let window = MainWindow::new()?;
        bind(&window, &state)?;
        let category_state: CategoryState = window.global();
        let total = category_state.invoke_total_spent(budget.id.to_shared_string());
        assert_eq!(total, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn left_to_spend_caps_at_zero() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        service.create_account("", AccountType::Cash)?;
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
        let window = MainWindow::new()?;
        bind(&window, &state)?;
        let category_state: CategoryState = window.global();
        let total = category_state.invoke_left_to_spend(budget.id.to_shared_string());
        assert_eq!(total, Money::ZERO.to_shared_string());
        Ok(())
    }
}
