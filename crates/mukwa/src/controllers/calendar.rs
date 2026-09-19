// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use jiff::civil::Date;
use jiff::{ToSpan, Zoned};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;
use tracing::warn;

pub fn bind(window: &ui::MainWindow) {
    let calendar_state = window.global::<ui::CalendarState>();
    calendar_state.on_today(|| Zoned::now().date().into());

    calendar_state.on_increment_month(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        if let Ok(date) = result {
            let date = date.saturating_add(1.month());
            return date.into();
        }

        Zoned::now().date().into()
    });

    calendar_state.on_increment_week(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        if let Ok(date) = result {
            let date = date.saturating_add(1.week());
            return date.into();
        }

        Zoned::now().date().into()
    });

    calendar_state.on_increment_day(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        if let Ok(date) = result {
            let date = date.saturating_add(1.day());
            return date.into();
        }

        Zoned::now().date().into()
    });

    calendar_state.on_decrement_month(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        if let Ok(date) = result {
            let date = date.saturating_sub(1.month());
            return date.into();
        }

        Zoned::now().date().into()
    });

    calendar_state.on_decrement_week(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        if let Ok(date) = result {
            let date = date.saturating_sub(1.week());
            return date.into();
        }

        Zoned::now().date().into()
    });

    calendar_state.on_decrement_day(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        if let Ok(date) = result {
            let date = date.saturating_sub(1.day());
            return date.into();
        }

        Zoned::now().date().into()
    });

    calendar_state.on_days_in_month(|date| {
        let result = Date::new(date.year as i16, date.month as i8, date.day as i8);
        match result {
            Ok(date) => {
                // Pad with 0 for the out of month days to align the calendar grid
                let offset = date.first_of_month().weekday().to_sunday_zero_offset();
                let mut days: Vec<i32> = vec![0; offset as usize];

                for d in 1..=date.days_in_month() {
                    days.push(d as i32);
                }

                let weeks: Vec<_> = days
                    .chunks(7)
                    .map(|chunk| ModelRc::new(Rc::new(VecModel::from(chunk.to_vec()))))
                    .collect();
                ModelRc::new(Rc::new(VecModel::from(weeks)))
            }
            Err(_) => {
                warn!("Invalid date: {:?}", date);
                Default::default()
            }
        }
    });
}
