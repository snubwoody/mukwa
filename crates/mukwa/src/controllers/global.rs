// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui;
use crate::ui::MainWindow;
use jiff::civil::Date;
use mukwa_core::fmt::CurrencyFormatter;
use mukwa_core::{Currency, Money, fmt};
use slint::{ComponentHandle, ModelRc, SharedString, ToSharedString, VecModel};
use std::collections::HashSet;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Instant;
use tracing::warn;

pub fn bind(window: &MainWindow, state: &AppState) {
    let instant = Instant::now();
    let mut database = fontdb::Database::new();
    database.load_system_fonts();

    let mut families = HashSet::new();
    for face in database.faces() {
        for (family, _) in &face.families {
            families.insert(family);
        }
    }

    let mut families: Vec<_> = families.iter().collect();
    families.sort();

    let fonts: Vec<_> = families
        .iter()
        .map(|family| ui::ComboBoxItem {
            value: family.to_shared_string(),
            text: family.to_shared_string(),
        })
        .collect();
    let elapsed = instant.elapsed().as_millis();
    tracing::trace!("Loaded system fonts in {elapsed}ms");

    let currencies: Vec<_> = Currency::ALL_CURRENCIES
        .iter()
        .map(|currency| ui::ComboBoxItem {
            value: currency.code().to_shared_string(),
            text: currency.name().to_shared_string(),
        })
        .collect();

    let currencies_model = Rc::new(VecModel::from(currencies));
    let currencies_model_rc = ModelRc::new(currencies_model);
    let fonts_model = Rc::new(VecModel::from(fonts));
    let fonts_model_rc = ModelRc::new(fonts_model);

    let global_state = window.global::<ui::State>();

    global_state.set_currency_options(currencies_model_rc);
    global_state.set_font_options(fonts_model_rc);

    global_state.on_format_dateym({
        |date| match Date::new(date.year as i16, date.month as i8, date.day as i8) {
            Ok(date) => date.strftime("%b %Y").to_shared_string(),
            Err(err) => {
                warn!("Invalid date: {err}");
                SharedString::new()
            }
        }
    });

    global_state.on_format_date({
        |date| match Date::strptime("%Y-%m-%d", date) {
            Ok(date) => match fmt::format_date(date) {
                Ok(value) => value.to_shared_string(),
                Err(err) => {
                    warn!("{err}");
                    SharedString::new()
                }
            },
            Err(err) => {
                warn!("Invalid date: {err}");
                SharedString::new()
            }
        }
    });

    global_state.on_format_money({
        move |value, currency_code| {
            // Empty strings represent null values
            if value.is_empty() {
                return value;
            }

            match Money::from_str(&value) {
                Ok(value) => {
                    let mut formatter = CurrencyFormatter::new();
                    let currency = Currency::from_str(&currency_code).unwrap();
                    formatter.set_currency(currency);
                    match formatter.format_money(value) {
                        Ok(result) => result.to_shared_string(),
                        Err(err) => {
                            warn!("Error occurred while formatting Money: {err}");
                            Money::ZERO.to_shared_string()
                        }
                    }
                }
                Err(err) => {
                    warn!("Error parsing Money: {err}");
                    Money::ZERO.to_shared_string()
                }
            }
        }
    })
}
