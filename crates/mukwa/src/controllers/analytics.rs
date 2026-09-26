// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui;
use crate::ui::{AnalyticsApi, PieChartSlice};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::Money;
use mukwa_core::plot::PieChart;
use mukwa_core::service::Category;
use slint::{ComponentHandle, Global, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::collections::{HashMap, HashSet};
use tracing::warn;
use uuid::Uuid;

pub fn bind(window: &ui::MainWindow, state: &AppState) {
    let analytics = window.global::<AnalyticsApi>();

    analytics.on_is_category_filtered({
        let analytics = analytics.as_weak();
        move |id| {
            !analytics
                .unwrap()
                .get_filtered_categories()
                .iter()
                .find(|category_id| category_id == &id)
                .is_some()
        }
    });

    analytics.on_filter_category({
        let analytics = analytics.as_weak();
        move |id| {
            if let Err(err) = filter_category(&analytics.unwrap(), id) {
                warn!("Failed to filter category: {err}");
            }
        }
    });

    analytics.on_draw_pie_chart({
        // TODO: draw gray no data donut chart if empty
        let state = state.clone();
        let analytics = analytics.as_weak();

        move |width, height, date| {
            draw_pie_chart(&analytics.unwrap(), &state, width, height, date)
                .inspect_err(|err| warn!("Failed to draw analytics: {err}"))
                .unwrap_or_default()
        }
    });
}

fn draw_pie_chart(
    analytics: &AnalyticsApi,
    state: &AppState,
    width: f32,
    height: f32,
    date: ui::Date,
) -> crate::Result<ModelRc<PieChartSlice>> {
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
    let categories = state.service().fetch_categories()?;
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
    let transactions = state.service().fetch_transactions()?;
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

fn filter_category(analytics: &AnalyticsApi, id: SharedString) -> crate::Result<()> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::ui::MainWindow;
    use mukwa_core::service::Service;

    #[test]
    fn filter_category_adds_category() -> crate::Result<()> {
        let window = MainWindow::new()?;
        let analytics = window.global::<AnalyticsApi>();
        filter_category(&analytics, SharedString::from("C1"))?;
        let category = analytics.get_filtered_categories().iter().next().unwrap();
        assert_eq!(category, "C1");
        Ok(())
    }

    #[test]
    fn filter_category_removes_category() -> crate::Result<()> {
        let window = MainWindow::new()?;
        let analytics = window.global::<AnalyticsApi>();

        filter_category(&analytics, SharedString::from("C1"))?;
        filter_category(&analytics, SharedString::from("C2"))?;
        filter_category(&analytics, SharedString::from("C2"))?;
        filter_category(&analytics, SharedString::from("C1"))?;
        filter_category(&analytics, SharedString::from("C3"))?;
        let categories = analytics
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
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let entertainment = service.create_category("Entertainment", group.id)?;

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

        let state = AppState::new(service)?;
        let analytics = window.global::<AnalyticsApi>();
        let slices: Vec<_> = draw_pie_chart(&analytics, &state, 500.0, 500.0, ui::Date::default())?
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
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let entertainment = service.create_category("Entertainment", group.id)?;

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

        let state = AppState::new(service)?;
        let analytics = window.global::<AnalyticsApi>();
        filter_category(&analytics, entertainment.id.to_shared_string())?;

        let slices: Vec<_> = draw_pie_chart(&analytics, &state, 500.0, 500.0, ui::Date::default())?
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
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let group = service.create_category_group("")?;
        let groceries = service.create_category("Groceries", group.id)?;
        let entertainment = service.create_category("Entertainment", group.id)?;

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

        let state = AppState::new(service)?;
        let analytics = window.global::<AnalyticsApi>();
        filter_category(&analytics, entertainment.id.to_shared_string())?;
        filter_category(&analytics, groceries.id.to_shared_string())?;
        let slices: Vec<_> = draw_pie_chart(&analytics, &state, 500.0, 500.0, ui::Date::default())?
            .iter()
            .collect();
        assert_eq!(slices.len(), 0);
        Ok(())
    }
}
