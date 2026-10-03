// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui;
use crate::ui::{CategoryState, MainWindow};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::service::Service;
use slint::{ComponentHandle, Global, Model, ModelExt, ModelRc, ToSharedString, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{debug, info, warn};
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
                Ok(_) => {
                    debug!("Created new category")
                }
                Err(err) => {
                    warn!("Failed to create category: {err}")
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

fn load_budgets(state: &CategoryState, service: &Service) -> crate::Result<()> {
    let month = Date::try_from(state.get_current_month())?;
    let budgets: Vec<ui::Budget> = service
        .fetch_or_init_budgets(month)?
        .iter()
        .map(|b| b.into())
        .collect();
    state.set_budgets(ModelRc::new(Rc::new(VecModel::from(budgets))));
    Ok(())
}

fn delete_category(state: &CategoryState, service: &Service, id: &str) -> crate::Result<Uuid> {
    let id = Uuid::parse_str(id)?;
    service.delete_category(id)?;
    //self.reset_budgets(self.current_budget_month)?;
    // self.reset_categories()?;
    load_categories(&state, &service)?;
    // FIXME: find a way to reset transactions
    //self.load_transactions()?;
    Ok(id)
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
    // self.budgets.set_vec(budgets);
    // self.reset_category_groups()?;

    load_budgets(&state, &service)?;
    load_categories(&state, &service)?;
    Ok(())
}

fn create_category(
    state: &CategoryState,
    service: &Service,
    title: &str,
    group_id: &str,
) -> crate::Result<()> {
    let group_id = Uuid::parse_str(group_id)?;
    let category = service.create_category(title, group_id)?;
    info!(id=?category.id,"Created new category");

    state
        .get_category_options()
        .push_row(category.clone().into())?;
    state.get_categories().push_row(category.into())?;
    // FIXME: reset budgets
    // self.reset_budgets(self.current_budget_month)?;
    // self.category_options.push(category.clone().into());
    // self.categories.push(category.into());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup_state() -> crate::Result<(AppState, MainWindow)> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let state = AppState::new(service)?;
        let window = MainWindow::new()?;
        bind(&window, &state)?;

        Ok((state, window))
    }

    #[test]
    fn bind_properties() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
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
}
