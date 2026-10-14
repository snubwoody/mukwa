// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui::{self, MainWindow};
use crate::ui::{AnalyticsApi, PieChartSlice};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::error::ErrorExt;
use mukwa_core::plot::PieChart;
use mukwa_core::service::{Category, Service, TransactionType};
use slint::{ComponentHandle, Global, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use tracing::warn;
use uuid::Uuid;

pub fn bind(window: &ui::MainWindow, state: &AppState) {
    AnalyticsController::bind(window, &state.service());
}

#[derive(Clone)]
pub struct AnalyticsController {
    analytics_state: Rc<AnalyticsApi<'static>>,
    service: Service,
}

impl AnalyticsController {
    pub fn bind(app: &MainWindow, service: &Service) -> Self {
        let analytics: AnalyticsApi = app.global();
        let controller = Self {
            analytics_state: Rc::new(analytics.as_weak().unwrap()),
            service: service.clone(),
        };

        analytics.on_draw_pie_chart({
            let controller = controller.clone();
            move |width, height, date| {
                controller
                    .draw_pie_chart(width, height, date)
                    .context("Failed to draw pie chart")
                    .inspect_err(|err| warn!("{}", err.report()))
                    .unwrap_or_default()
            }
        });

        analytics.on_filter_category({
            let controller = controller.clone();
            move |id| {
                if let Err(err) = controller
                    .filter_category(id)
                    .context("Failed to filter category")
                {
                    warn!("{}", err.report());
                }
            }
        });

        analytics.on_total_spent({
            let controller = controller.clone();
            move || {
                controller
                    .total_spent()
                    .context("Failed to calculate total spent")
                    .inspect_err(|err| warn!("{}", err.report()))
                    .unwrap_or_default()
                    .to_shared_string()
            }
        });

        analytics.on_is_category_filtered({
            let controller = controller.clone();
            move |id| controller.is_category_filtered(&id)
        });

        controller
    }

    fn total_spent(&self) -> crate::Result<Money> {
        let date = Date::try_from(self.analytics_state.get_current_date())?;
        let filtered_categories: HashSet<SharedString> = self
            .analytics_state
            .get_filtered_categories()
            .iter()
            .collect();
        let total: Money = self
            .service
            .fetch_transactions()?
            .iter()
            .filter(|t| t.transaction_type() == TransactionType::Expense)
            .filter(|t| t.date.year() == date.year() && t.date.month() == date.month())
            .filter(|t| {
                !filtered_categories.contains(&t.category_id.unwrap_or_default().to_shared_string())
            })
            .map(|t| t.amount)
            .sum();
        Ok(total)
    }

    fn is_category_filtered(&self, id: &str) -> bool {
        !self
            .analytics_state
            .get_filtered_categories()
            .iter()
            .find(|category_id| category_id == id)
            .is_some()
    }

    fn draw_pie_chart(
        &self,
        width: f32,
        height: f32,
        date: ui::Date,
    ) -> crate::Result<ModelRc<PieChartSlice>> {
        let analytics = &self.analytics_state;
        let colors = [
            tiny_skia::Color::from_rgba8(0, 117, 222, 255),
            tiny_skia::Color::from_rgba8(0, 94, 180, 255),
            tiny_skia::Color::from_rgba8(0, 70, 138, 255),
            tiny_skia::Color::from_rgba8(0, 48, 98, 255),
            tiny_skia::Color::from_rgba8(61, 144, 255, 255),
            tiny_skia::Color::from_rgba8(127, 171, 255, 255),
            tiny_skia::Color::from_rgba8(236, 241, 255, 255),
        ];

        let filtered_categories: HashSet<SharedString> =
            analytics.get_filtered_categories().iter().collect();
        let categories = self.service.fetch_categories()?;
        let categories: Vec<Category> = categories
            .into_iter()
            .filter(|category| !filtered_categories.contains(&category.id.to_shared_string()))
            .collect();

        // TODO: collect categories below a threshold into 'Other'
        struct Analytic {
            category: Category,
            total: Money,
        }
        let date = Date::try_from(date).unwrap_or(Zoned::now().date());
        let transactions = self.service.fetch_transactions()?;
        let mut analytics: HashMap<Uuid, Analytic> = HashMap::new();

        for transaction in transactions {
            if transaction.date.year() != date.year() || transaction.date.month() != date.month() {
                continue;
            }

            if let Some(category_id) = transaction.category_id {
                if filtered_categories.contains(&category_id.to_shared_string()) {
                    continue;
                }
                match analytics.get(&category_id) {
                    Some(value) => {
                        analytics.insert(
                            category_id,
                            Analytic {
                                total: transaction.amount + value.total,
                                category: value.category.clone(),
                            },
                        );
                    }
                    None => {
                        let category = categories
                            .iter()
                            .find(|c| c.id == category_id)
                            .cloned()
                            .unwrap_or_default();
                        let analytic = Analytic {
                            category,
                            total: transaction.amount,
                        };
                        analytics.insert(category_id, analytic);
                    }
                }
            }
        }

        let mut analytics = analytics.values().collect::<Vec<_>>();
        analytics.sort_by(|a, b| a.total.cmp(&b.total).reverse());

        let series: Vec<f32> = analytics.iter().map(|a| a.total.inner() as f32).collect();
        let labels: Vec<String> = analytics.iter().map(|a| a.category.title.clone()).collect();
        let radius = width.min(height) / 2.0;
        let chart = PieChart::new(width / 2.0, height / 2.0, series, radius)
            .with_colors(colors.to_vec())
            .with_label_line_length(50.0)
            .with_labels(labels)
            .with_hole_radius(radius - 100.0);

        let segments = chart.segments();
        let slices = VecModel::default();

        for (index, segment) in segments.iter().enumerate() {
            let color = segment.color().to_color_u8();
            let (label_x, label_y) = segment.label_position();
            let amount = analytics[index].total;

            let slice = PieChartSlice {
                arc_path: segment.arc_svg().to_shared_string(),
                line_path: segment.label_line_svg().to_shared_string(),
                fill: slint::Color::from_rgb_u8(color.red(), color.green(), color.blue()),
                label: segment.label().to_shared_string(),
                label_x,
                amount: amount.to_shared_string(),
                label_y,
                ratio: segment.ratio(),
            };

            slices.push(slice);
        }

        Ok(ModelRc::new(slices))
    }

    fn filter_category(&self, id: SharedString) -> crate::Result<()> {
        let analytics = &self.analytics_state;
        let filtered_categories = analytics.get_filtered_categories();
        match filtered_categories
            .iter()
            .find(|category_id| category_id == &id)
        {
            Some(_) => {
                let new_categories = filtered_categories
                    .iter()
                    .filter(|category_id| category_id != &id)
                    .collect::<VecModel<_>>();

                analytics.set_filtered_categories(ModelRc::new(new_categories));
            }
            None => {
                let new_categories = filtered_categories.iter().collect::<VecModel<_>>();
                new_categories.push(id);
                analytics.set_filtered_categories(ModelRc::new(new_categories));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ui::MainWindow;
    use mukwa_core::service::Service;

    #[test]
    fn filter_category_adds_category() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let controller = AnalyticsController::bind(&app, &Service::open_in_memory()?);
        controller.filter_category(SharedString::from("C1"))?;
        let category = controller
            .analytics_state
            .get_filtered_categories()
            .iter()
            .next()
            .unwrap();
        assert_eq!(category, "C1");
        Ok(())
    }

    #[test]
    fn filter_category_removes_category() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let controller = AnalyticsController::bind(&app, &Service::open_in_memory()?);

        controller.filter_category(SharedString::from("C1"))?;
        controller.filter_category(SharedString::from("C2"))?;
        controller.filter_category(SharedString::from("C2"))?;
        controller.filter_category(SharedString::from("C1"))?;
        controller.filter_category(SharedString::from("C3"))?;

        let categories = controller
            .analytics_state
            .get_filtered_categories()
            .iter()
            .collect::<Vec<_>>();
        assert!(!categories.contains(&SharedString::from("C1")));
        assert!(!categories.contains(&SharedString::from("C2")));
        assert!(categories.contains(&SharedString::from("C3")));
        Ok(())
    }

    #[test]
    fn draw_simple_pie_chart() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let entertainment = service.create_category("Entertainment", group.id)?;
        let controller = AnalyticsController::bind(&app, &service);

        service
            .create_expense()
            .category(groceries.id)
            .amount(Money::new(25))
            .submit()?;
        service
            .create_expense()
            .category(entertainment.id)
            .amount(Money::new(75))
            .submit()?;

        let slices: Vec<_> = controller
            .draw_pie_chart(500.0, 500.0, ui::Date::default())?
            .iter()
            .collect();
        assert_eq!(slices.len(), 2);

        let entertainment_slice = &slices[0];
        let groceries_slice = &slices[1];
        assert_eq!(
            entertainment_slice.amount,
            Money::new(75).to_shared_string()
        );
        assert_eq!(
            entertainment_slice.label,
            SharedString::from("Entertainment")
        );
        assert_eq!(entertainment_slice.ratio, 0.75);
        assert_eq!(groceries_slice.amount, Money::new(25).to_shared_string());
        assert_eq!(groceries_slice.label, SharedString::from("Groceries"));
        assert_eq!(groceries_slice.ratio, 0.25);
        Ok(())
    }

    #[test]
    fn draw_pie_chart_with_filter() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let entertainment = service.create_category("Entertainment", group.id)?;
        let controller = AnalyticsController::bind(&app, &service);

        service
            .create_expense()
            .category(groceries.id)
            .amount(Money::new(25))
            .submit()?;
        service
            .create_expense()
            .category(entertainment.id)
            .amount(Money::new(75))
            .submit()?;

        controller.filter_category(entertainment.id.to_shared_string())?;

        let slices: Vec<_> = controller
            .draw_pie_chart(500.0, 500.0, ui::Date::default())?
            .iter()
            .collect();
        assert_eq!(slices.len(), 1);

        let groceries_slice = &slices[0];
        assert_eq!(groceries_slice.amount, Money::new(25).to_shared_string());
        assert_eq!(groceries_slice.label, SharedString::from("Groceries"));
        assert_eq!(groceries_slice.ratio, 1.0);
        Ok(())
    }

    #[test]
    fn draw_pie_chart_with_all_categories_filtered() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let app = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let entertainment = service.create_category("Entertainment", group.id)?;
        let controller = AnalyticsController::bind(&app, &service);

        service
            .create_expense()
            .category(groceries.id)
            .amount(Money::new(25))
            .submit()?;
        service
            .create_expense()
            .category(entertainment.id)
            .amount(Money::new(75))
            .submit()?;

        controller.filter_category(entertainment.id.to_shared_string())?;
        controller.filter_category(groceries.id.to_shared_string())?;
        let slices: Vec<_> = controller
            .draw_pie_chart(500.0, 500.0, ui::Date::default())?
            .iter()
            .collect();
        assert_eq!(slices.len(), 0);
        Ok(())
    }
}
