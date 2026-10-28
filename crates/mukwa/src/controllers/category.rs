// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use crate::ui::{CategoryState, MainWindow};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::error::ErrorExt;
use mukwa_core::service::{CreateBudgetOpts, Service};
use slint::{
    ComponentHandle, DataTransfer, Global, Model, ModelExt, ModelRc, SharedString, ToSharedString,
    VecModel,
};
use std::rc::Rc;
use std::str::FromStr;
use tracing::{info, warn};
use uuid::Uuid;

/// Binds the Slint callbacks to Rust methods.
pub fn bind(window: &MainWindow, service: &Service) -> crate::Result<()> {
    CategoryController::bind(window, service)?;
    Ok(())
}

#[derive(Clone)]
pub struct CategoryController {
    state: Rc<CategoryState<'static>>,
    service: Service,
}

impl CategoryController {
    pub fn bind(window: &MainWindow, service: &Service) -> crate::Result<Self> {
        let category_state = window.global::<CategoryState>();

        let controller = CategoryController {
            service: service.clone(),
            state: Rc::new(category_state.as_weak().unwrap()),
        };

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

        category_state.on_edit_budget_target({
            let controller = controller.clone();
            move |category_id, amount| {
                if let Err(err) = controller
                    .edit_budget_target(&category_id, &amount)
                    .context("Failed to edit budget target")
                {
                    warn!("{}", err.report());
                }
            }
        });

        category_state.on_create_budget_target({
            let controller = controller.clone();
            move |category_id, amount| {
                if let Err(err) = controller
                    .create_budget_target(&category_id, &amount)
                    .context("Failed to create budget target")
                {
                    warn!("{}", err.report());
                }
            }
        });

        category_state.on_delete_budget_target({
            let controller = controller.clone();
            move |category_id| {
                if let Err(err) = controller
                    .delete_budget_target(&category_id)
                    .context("Failed to delete budget target")
                {
                    warn!("{}", err.report());
                }
            }
        });

        category_state.on_has_budget_target({
            let controller = controller.clone();
            move |category_id| {
                controller
                    .has_budget_target(&category_id)
                    .inspect_err(|err| warn!("{}", err.report()))
                    .unwrap_or(false)
            }
        });

        category_state.on_get_budget_target({
            let controller = controller.clone();
            move |category_id| {
                controller
                    .get_budget_target(&category_id)
                    .inspect_err(|err| warn!("{}", err.report()))
                    .unwrap_or_default()
                    .to_shared_string()
            }
        });

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

        category_state.on_set_current_budget_month({
            let category_state = category_state.as_weak();
            let controller = controller.clone();
            move |date| {
                let category_state = category_state.unwrap();
                category_state.set_current_month(date);
                if let Err(err) = controller.load_budgets() {
                    warn!("Failed to load budgets: {err}")
                }
            }
        });

        category_state.on_load_state({
            let controller = controller.clone();
            move || {
                if let Err(err) = controller
                    .load_category_groups()
                    .context("Failed to load category groups")
                {
                    warn!("{}", err.report());
                }
                if let Err(err) = controller
                    .load_categories()
                    .context("Failed to load categories")
                {
                    warn!("{}", err.report());
                }
                if let Err(err) = controller.load_budgets().context("Failed to load budgets") {
                    warn!("{}", err.report());
                }
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
            let controller = controller.clone();
            move |title, group_id| match controller.create_category(&title, &group_id) {
                Ok(id) => {
                    info!("Created new category ({id})")
                }
                Err(err) => {
                    warn!("Failed to create category: {err}")
                }
            }
        });

        category_state.on_create_category_group({
            let controller = controller.clone();
            move |title| match controller.create_category_group(&title) {
                Ok(id) => {
                    info!("Created new category group ({id})")
                }
                Err(err) => {
                    warn!("Failed to create category group: {err}")
                }
            }
        });

        category_state.on_delete_category({
            let controller = controller.clone();
            move |id| match controller.delete_category(&id) {
                Ok(id) => {
                    info!("Deleted category {id}");
                }
                Err(err) => {
                    warn!("Failed to delete category: {err}")
                }
            }
        });

        category_state.on_delete_category_group({
            let controller = controller.clone();
            move |id| match controller.delete_category_group(&id) {
                Ok(id) => {
                    info!("Deleted category group {id}");
                }
                Err(err) => {
                    warn!("Failed to delete category group: {err}")
                }
            }
        });

        category_state.on_edit_budget({
            let controller = controller.clone();
            move |id, amount| {
                if let Err(err) = controller.update_budget(&id, &amount) {
                    warn!("Failed to update budget: {err}");
                }
            }
        });

        category_state.on_update_category({
            let controller = controller.clone();
            move |id, title| {
                if let Err(err) = controller.update_category(&id, &title) {
                    warn!("Failed to update category: {err}");
                }
            }
        });

        category_state.on_update_category_group({
            let controller = controller.clone();
            move |id, title| {
                if let Err(err) = controller.update_category_group(&id, &title) {
                    warn!("Failed to update category group: {err}");
                }
            }
        });

        category_state.on_move_category({
            let controller = controller.clone();
            move |id, group_id| {
                if let Err(err) = controller.move_category(&id, &group_id) {
                    warn!("Failed to move category: {err}");
                }
            }
        });

        category_state.on_total_spent_in_group({
            let controller = controller.clone();
            move |id, date| {
                controller
                    .total_spent_in_group(&id, date)
                    .unwrap_or_else(|err| {
                        warn!("Failed to calculate total spent in category group: {err}");
                        Money::ZERO
                    })
                    .to_shared_string()
            }
        });

        category_state.on_left_to_spend_in_group({
            let controller = controller.clone();
            move |id, date| {
                controller
                    .left_to_spend_in_group(&id, date)
                    .unwrap_or_else(|err| {
                        warn!("Failed to calculate left to spend in category group: {err}");
                        Money::ZERO
                    })
                    .to_shared_string()
            }
        });

        category_state.on_total_assigned_in_group({
            let controller = controller.clone();
            move |id, date| {
                controller
                    .total_assigned_in_group(&id, date)
                    .unwrap_or_else(|err| {
                        warn!("Failed to calculate total assigned in category group: {err}");
                        Money::ZERO
                    })
                    .to_shared_string()
            }
        });

        category_state.on_total_spent({
            let controller = controller.clone();
            move |id| {
                controller
                    .total_spent(&id)
                    .unwrap_or_else(|err| {
                        warn!("Failed to calculate total spent: {err}");
                        Money::ZERO
                    })
                    .to_shared_string()
            }
        });

        category_state.on_left_to_spend({
            let controller = controller.clone();
            move |id| {
                controller
                    .left_to_spend(&id)
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

        Ok(controller)
    }

    fn load_categories(&self) -> crate::Result<()> {
        let categories: Vec<ui::Category> = self
            .service
            .fetch_categories()?
            .iter()
            .map(|c| c.into())
            .collect();
        self.state
            .set_categories(ModelRc::new(Rc::new(VecModel::from(categories))));
        Ok(())
    }

    fn load_category_groups(&self) -> crate::Result<()> {
        let groups: Vec<ui::CategoryGroup> = self
            .service
            .fetch_category_groups()?
            .iter()
            .map(|c| c.into())
            .collect();
        self.state
            .set_category_groups(ModelRc::new(Rc::new(VecModel::from(groups))));
        Ok(())
    }

    fn load_budgets(&self) -> crate::Result<()> {
        let month = Date::try_from(self.state.get_current_month())?;
        let budgets: Vec<ui::Budget> = self
            .service
            .fetch_or_init_budgets(month)?
            .iter()
            .map(|b| b.into())
            .collect();
        self.state
            .set_budgets(ModelRc::new(Rc::new(VecModel::from(budgets))));
        Ok(())
    }

    fn edit_budget_target(&self, category_id: &str, amount: &str) -> crate::Result<()> {
        let category_id = Uuid::parse_str(category_id)?;
        let amount = Money::from_str(amount)?;
        self.service.update_budget_target(category_id, amount)?;
        Ok(())
    }

    fn create_budget_target(&self, category_id: &str, amount: &str) -> crate::Result<()> {
        let category_id = Uuid::parse_str(category_id)?;
        let amount = Money::from_str(amount)?;
        self.service.create_budget_target(category_id, amount)?;
        Ok(())
    }

    fn delete_budget_target(&self, category_id: &str) -> crate::Result<()> {
        let category_id = Uuid::parse_str(category_id)?;
        self.service.delete_budget_target(category_id)?;
        Ok(())
    }

    fn get_budget_target(&self, category_id: &str) -> crate::Result<Money> {
        let category_id = Uuid::parse_str(category_id)?;
        self.service.get_budget_target(category_id)
    }

    fn has_budget_target(&self, category_id: &str) -> crate::Result<bool> {
        let category_id = Uuid::parse_str(category_id)?;
        let result = self.service.get_budget_target(category_id);
        Ok(result.is_ok())
    }

    fn delete_category(&self, id: &str) -> crate::Result<Uuid> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_category(id)?;
        self.load_categories()?;
        self.load_budgets()?;
        Ok(id)
    }

    fn delete_category_group(&self, id: &str) -> crate::Result<Uuid> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_category_group(id)?;
        self.load_budgets()?;
        self.load_category_groups()?;
        self.load_categories()?;
        Ok(id)
    }

    fn left_to_spend_in_group(&self, id: &str, date: ui::Date) -> crate::Result<Money> {
        let total = self.total_spent_in_group(id, date.clone())?;
        let id = Uuid::parse_str(id)?;
        let date = Date::try_from(date)?;
        let assigned = self.service.total_assigned_in_group(id, date)?;
        let available = assigned - total;
        Ok(available.max(Money::ZERO))
    }

    fn total_spent_in_group(&self, id: &str, date: ui::Date) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        let date = Date::try_from(date)?;
        let total = self.service.total_spent_in_group(id, date)?;
        Ok(total)
    }

    fn total_spent(&self, id: &str) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        let budget = self.service.get_budget(id)?;
        let date = Date::new(budget.year as i16, budget.month as i8, 1)?;
        let total = self.service.total_spent(budget.category_id, date)?;
        Ok(total)
    }

    fn left_to_spend(&self, id: &str) -> crate::Result<Money> {
        let total = self.total_spent(id)?;
        let id = Uuid::parse_str(id)?;
        let budget = self.service.get_budget(id)?;
        let available = budget.amount - total;
        Ok(available.max(Money::ZERO))
    }

    fn total_assigned_in_group(&self, id: &str, date: ui::Date) -> crate::Result<Money> {
        let id = Uuid::parse_str(id)?;
        let date = Date::try_from(date)?;
        let total = self.service.total_assigned_in_group(id, date)?;
        Ok(total)
    }

    fn update_budget(&self, id: &str, amount: &str) -> crate::Result<()> {
        let budget_id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        self.service.update_budget(budget_id, amount)?;
        self.load_budgets()?;
        self.load_categories()?;
        self.load_category_groups()?;
        Ok(())
    }

    fn move_category(&self, id: &str, group_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let group_id = Uuid::parse_str(group_id)?;
        self.service.move_category(id, group_id)?;
        self.load_categories()?;
        self.load_category_groups()?;
        self.load_budgets()?;
        Ok(())
    }

    fn create_category(&self, title: &str, group_id: &str) -> crate::Result<Uuid> {
        let group_id = Uuid::parse_str(group_id)?;
        let category = self.service.create_category(title, group_id)?;

        let id = category.id;
        let date = Date::try_from(self.state.get_current_month())?;
        self.service.create_budget(CreateBudgetOpts {
            amount: Some(Money::ZERO),
            month: Some(date),
            category_id: category.id,
        })?;
        self.state
            .get_category_options()
            .push_row(category.clone().into())?;
        self.state.get_categories().push_row(category.into())?;
        self.load_budgets()?;
        Ok(id)
    }

    fn update_category(&self, id: &str, title: &str) -> crate::Result<Uuid> {
        let id = Uuid::parse_str(id)?;
        self.service.update_category(id, title)?;
        self.load_categories()?;
        self.load_budgets()?;
        Ok(id)
    }

    fn update_category_group(&self, id: &str, title: &str) -> crate::Result<Uuid> {
        let id = Uuid::parse_str(id)?;
        self.service.update_category_group(id, title)?;
        self.service.update_category(id, title)?;
        self.load_categories()?;
        self.load_category_groups()?;
        self.load_budgets()?;
        Ok(id)
    }

    fn create_category_group(&self, title: &str) -> crate::Result<Uuid> {
        let category_group = self.service.create_category_group(title)?;
        let id = category_group.id;
        self.state
            .get_category_groups()
            .push_row(category_group.into())?;
        Ok(id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::civil::date;
    use mukwa_core::service::{AccountType, CreateBudgetOpts};

    fn setup() -> crate::Result<(Service, CategoryState<'static>, MainWindow)> {
        i_slint_backend_testing::init_no_event_loop();
        let service = Service::open_in_memory()?;
        let window = MainWindow::new()?;
        bind(&window, &service)?;
        let category_state: CategoryState = window.global();
        let category_state = category_state.as_weak().unwrap();
        Ok((service, category_state, window))
    }

    #[test]
    fn bind_properties() -> crate::Result<()> {
        let (_, _, window) = setup()?;

        let category_state = window.global::<CategoryState>();
        let current_month = category_state.get_current_month();
        assert_eq!(current_month, Zoned::now().date().into());
        Ok(())
    }

    #[test]
    fn create_category_adds_to_combobox_list() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
        let group = service.create_category_group("")?;
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
        let window = MainWindow::new()?;
        bind(&window, &service)?;

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
        let window = MainWindow::new()?;
        bind(&window, &service)?;

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
        let window = MainWindow::new()?;
        bind(&window, &service)?;

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
        let (service, category_state, _) = setup()?;
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
        let total = category_state.invoke_total_spent(budget.id.to_shared_string());
        assert_eq!(total, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn calculate_total_spent_only_includes_current_month() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
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
        let total = category_state.invoke_total_spent(budget.id.to_shared_string());
        assert_eq!(total, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn left_to_spend_caps_at_zero() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
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
        let total = category_state.invoke_left_to_spend(budget.id.to_shared_string());
        assert_eq!(total, Money::ZERO.to_shared_string());
        Ok(())
    }

    #[test]
    fn get_budget_target() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;

        // Default value when there is no target
        let target = category_state.invoke_get_budget_target(category.id.to_shared_string());
        assert_eq!(target, Money::new(0).to_shared_string());

        service.create_budget_target(category.id, Money::new(10))?;
        let target = category_state.invoke_get_budget_target(category.id.to_shared_string());
        assert_eq!(target, Money::new(10).to_shared_string());
        Ok(())
    }

    #[test]
    fn create_budget_target() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;
        category_state
            .invoke_create_budget_target(category.id.to_shared_string(), "500".to_shared_string());
        // Should not create duplicate targets
        category_state
            .invoke_create_budget_target(category.id.to_shared_string(), "0".to_shared_string());
        let target = category_state.invoke_get_budget_target(category.id.to_shared_string());
        assert_eq!(target, Money::new(500).to_shared_string());
        Ok(())
    }

    #[test]
    fn has_budget_target() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;
        let has_target = category_state.invoke_has_budget_target(category.id.to_shared_string());
        assert!(!has_target);
        category_state
            .invoke_create_budget_target(category.id.to_shared_string(), "500".to_shared_string());
        let has_target = category_state.invoke_has_budget_target(category.id.to_shared_string());
        assert!(has_target);
        Ok(())
    }

    #[test]
    fn delete_budget_target() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;
        category_state
            .invoke_create_budget_target(category.id.to_shared_string(), "500".to_shared_string());
        category_state.invoke_delete_budget_target(category.id.to_shared_string());
        let has_target = category_state.invoke_has_budget_target(category.id.to_shared_string());
        assert!(!has_target);
        Ok(())
    }

    #[test]
    fn edit_budget_target() -> crate::Result<()> {
        let (service, category_state, _) = setup()?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;
        category_state
            .invoke_create_budget_target(category.id.to_shared_string(), "500".to_shared_string());
        category_state
            .invoke_edit_budget_target(category.id.to_shared_string(), "100".to_shared_string());
        let target = category_state.invoke_get_budget_target(category.id.to_shared_string());
        assert_eq!(target, Money::new(100).to_shared_string());
        Ok(())
    }
}
