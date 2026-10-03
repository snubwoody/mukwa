// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::state::AppState;
use crate::ui;
use crate::ui::MainWindow;
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::fmt::CurrencyFormatter;
use mukwa_core::service::{Service, TransactionType};
use mukwa_core::{Currency, Money, fmt};
use slint::{
    ComponentHandle, DataTransfer, Model, ModelRc, SharedString, ToSharedString, VecModel,
};
use std::collections::HashSet;
use std::rc::Rc;
use std::str::FromStr;
use std::time::Instant;
use tracing::warn;
use uuid::Uuid;

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
    let transactions_model_rc = ModelRc::new(state.transactions());
    let accounts_model_rc = ModelRc::new(state.accounts());
    let categories_model_rc = ModelRc::new(state.categories());
    let category_groups_model_rc = ModelRc::new(state.category_groups());
    let budgets_model_rc = ModelRc::new(state.budgets());
    let account_options_rc = ModelRc::new(state.account_options());

    let global_state = window.global::<ui::State>();

    global_state.set_currency_options(currencies_model_rc);
    global_state.set_font_options(fonts_model_rc);
    global_state.set_transactions(transactions_model_rc);
    global_state.set_accounts(accounts_model_rc);
    global_state.set_categories(categories_model_rc);
    global_state.set_category_groups(category_groups_model_rc);
    global_state.set_budgets(budgets_model_rc);

    global_state.set_account_options(account_options_rc);
    global_state.set_category_options(ModelRc::new(state.category_options()));

    global_state.on_is_transaction_unconfirmed({
        let state = state.clone();
        move |id| {
            is_transaction_unconfirmed(&id, &state.service())
                .inspect_err(|err| warn!("{err}"))
                .unwrap_or_default()
        }
    });

    global_state.on_total_spent_all({
        let state = state.clone();
        move |month| match state.service().fetch_transactions() {
            Ok(transactions) => {
                let date = Date::new(month.year as i16, month.month as i8, month.day as i8)
                    .unwrap_or(Zoned::now().date());
                let total: Money = transactions
                    .iter()
                    .filter(|t| t.transaction_type() == TransactionType::Expense)
                    .filter(|t| t.date.year() == date.year() && t.date.month() == date.month())
                    .map(|t| t.amount)
                    .sum();
                total.to_shared_string()
            }
            Err(err) => {
                warn!("Error while calculating total spent: {err}");
                Money::ZERO.to_shared_string()
            }
        }
    });

    global_state.on_create_account({
        let mut state = state.clone();
        move |name, account_type, balance| {
            if let Err(err) = state.create_account(name.as_str(), account_type, balance.as_str()) {
                warn!("Failed to create account: {err}")
            }
        }
    });

    global_state.on_get_account({
        let state = state.clone();
        move |id| {
            state
                .accounts()
                .iter()
                .find(|a| a.id == id)
                .unwrap_or_default()
        }
    });

    global_state.on_create_transaction({
        let mut state = state.clone();
        move |opts| {
            if let Err(err) = state.create_transaction(opts) {
                warn!("Failed to create transaction: {err}")
            }
        }
    });

    global_state.on_delete_account({
        let mut state = state.clone();
        move |id| {
            if let Err(err) = state.delete_account(&id) {
                warn!("Failed to delete account: {err}")
            }
        }
    });

    global_state.on_set_current_budget_month({
        let mut state = state.clone();
        move |date| {
            let date = Date::new(date.year as i16, date.month as i8, date.day as i8)
                .unwrap_or(Zoned::now().date());
            if let Err(err) = state.set_current_budget_month(date) {
                warn!("{err}")
            }
        }
    });

    global_state.on_set_transaction_category({
        let mut state = state.clone();
        move |id, category_id| {
            if let Err(err) = state.set_transaction_category(&id, &category_id) {
                warn!("{err}");
            }
        }
    });

    global_state.on_set_transaction_date({
        let mut state = state.clone();
        move |id, date| {
            if let Err(err) = state.set_transaction_date(&id, &date) {
                warn!("{err}");
            }
        }
    });

    global_state.on_set_transaction_outflow({
        let mut state = state.clone();
        move |id, amount| {
            if let Err(err) = state.set_transaction_outflow(&id, &amount) {
                warn!("{err}");
            }
        }
    });

    global_state.on_set_transaction_inflow({
        let mut state = state.clone();
        move |id, amount| {
            if let Err(err) = state.set_transaction_inflow(&id, &amount) {
                warn!("{err}");
            }
        }
    });

    global_state.on_set_transaction_account({
        let mut state = state.clone();
        move |id, account_id| {
            if let Err(err) = state.set_transaction_account(&id, &account_id) {
                warn!("{err}");
            }
        }
    });

    global_state.on_set_transaction_payee({
        let mut state = state.clone();
        move |id, account_id| {
            if let Err(err) = state.set_transaction_payee(&id, &account_id) {
                warn!("{err}");
            }
        }
    });

    global_state.on_set_transaction_note({
        let mut state = state.clone();
        move |id, note| {
            if let Err(err) = state.set_transaction_note(&id, &note) {
                warn!("{err}");
            }
        }
    });

    global_state.on_total_balance({
        let state = state.clone();
        move || {
            let mut total = Money::ZERO;
            for transaction in state.transactions().iter() {
                total -= Money::from_str(transaction.outflow.as_str()).unwrap_or_default();
                total += Money::from_str(transaction.inflow.as_str()).unwrap_or_default();
            }
            total.to_shared_string()
        }
    });

    global_state.on_account_balance({
        let state = state.clone();
        move |id| match state.account_balance(&id) {
            Ok(balance) => balance.to_shared_string(),
            Err(err) => {
                warn!("Failed to calculate account balance: {err}");
                Money::ZERO.to_shared_string()
            }
        }
    });

    global_state.on_delete_transaction({
        let mut state = state.clone();
        move |id| {
            if let Err(err) = state.delete_transaction(&id) {
                warn!("Failed to delete transaction: {err}");
            }
        }
    });

    global_state.on_confirm_transaction({
        let mut state = state.clone();
        move |id| {
            if let Err(err) = state.confirm_transaction(&id) {
                warn!("Failed to delete transaction: {err}");
            }
        }
    });

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

    global_state.on_duplicate_transaction({
        let mut state = state.clone();
        move |id| {
            if let Err(err) = state.duplicate_transaction(&id) {
                warn!("Failed to duplicate transaction: {err}");
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

fn is_transaction_unconfirmed(id: &str, service: &Service) -> crate::Result<bool> {
    let unconfirmed_transactions: HashSet<Uuid> = service
        .fetch_unconfirmed_transactions()?
        .iter()
        .copied()
        .collect();
    let id = Uuid::parse_str(id)?;
    Ok(unconfirmed_transactions.contains(&id))
}
