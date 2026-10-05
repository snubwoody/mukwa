// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::collections::HashSet;
use crate::ui;
use crate::ui::{CategoryState, MainWindow, TransactionState};
use mukwa_core::service::{Service, Transaction};
use slint::{ComponentHandle, Global, Model, ModelRc, VecModel};
use std::rc::Rc;
use std::str::FromStr;
use jiff::civil::Date;
use tracing::{info, warn};
use uuid::Uuid;
use mukwa_core::Money;

pub fn bind(window: &MainWindow, service: Service) -> crate::Result<()> {
    let transaction_state: TransactionState = window.global();
    let mut transactions = service.fetch_transactions()?;
    transactions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    let transactions_list: Vec<ui::Transaction> = transactions.iter().map(|t| t.into()).collect();

    transaction_state.set_transactions(ModelRc::new(VecModel::from(transactions_list)));

    transaction_state.on_delete_transaction({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id| {
            if let Err(err) = delete_transaction(&id, &transaction_state.unwrap(), &service) {
                warn!("Failed to delete transaction: {err}");
            }
        }
    });

    transaction_state.on_confirm_transaction({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id| {
            if let Err(err) = confirm_transaction(&id,&transaction_state.unwrap(),&service) {
                warn!("Failed to delete transaction: {err}");
            }
        }
    });


    transaction_state.on_set_transaction_category({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, category_id| {
            if let Err(err) = set_transaction_category(&id, &category_id,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_set_transaction_date({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, date| {
            if let Err(err) = set_transaction_date(&id, &date,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_set_transaction_outflow({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, amount| {
            if let Err(err) = set_transaction_outflow(&id, &amount,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_set_transaction_inflow({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, amount| {
            if let Err(err) = set_transaction_inflow(&id, &amount,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_set_transaction_account({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, account_id| {
            if let Err(err) = set_transaction_account(&id, &account_id,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_set_transaction_payee({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, account_id| {
            if let Err(err) = set_transaction_payee(&id, &account_id,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_set_transaction_note({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id, note| {
            if let Err(err) = set_transaction_note(&id, &note,&transaction_state.unwrap(),&service) {
                warn!("{err}");
            }
        }
    });

    transaction_state.on_create_transaction({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |opts| {
            if let Err(err) = create_transaction(opts,&transaction_state.unwrap(),&service) {
                warn!("Failed to create transaction: {err}")
            }
        }
    });

    transaction_state.on_is_transaction_unconfirmed({
        let service = service.clone();
        move |id| {
            is_transaction_unconfirmed(&id, &service)
                .inspect_err(|err| warn!("{err}"))
                .unwrap_or_default()
        }
    });

    transaction_state.on_duplicate_transaction({
        let transaction_state = transaction_state.as_weak();
        let service = service.clone();
        move |id| {
            if let Err(err) = duplicate_transaction(&id,&transaction_state.unwrap(),&service) {
                warn!("Failed to duplicate transaction: {err}");
            }
        }
    });

    Ok(())
}

fn confirm_transaction(id: &str,state: &TransactionState,service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    service.confirm_transaction(id)?;
    load_transactions(state,service)?;
    Ok(())
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

fn delete_transaction(id: &str, state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    service.delete_transaction(id)?;
    load_transactions(state, service)?;
    Ok(())
}


fn create_transaction(opts: ui::CreateTransactionOpts, state: &TransactionState, service: &Service) -> crate::Result<()> {
    let date = Date::strptime("%Y-%m-%d", &opts.date)?;

    let transaction =
        if !opts.outflow.is_empty() && opts.inflow.is_empty() && opts.payee_id.is_empty() {
            let amount = Money::from_str(&opts.outflow)?;
            let mut builder = service.create_expense().amount(amount).date(date);

            if !opts.account_id.is_empty() {
                builder = builder.account(Uuid::parse_str(&opts.account_id)?);
            }

            if !opts.category_id.is_empty() {
                builder = builder.category(Uuid::parse_str(&opts.category_id)?);
            }

            if !opts.note.is_empty() {
                builder = builder.note(&opts.note);
            }

            builder.submit()?
        } else if !opts.inflow.is_empty() {
            let amount = Money::from_str(&opts.inflow)?;
            let mut builder = service.create_income().amount(amount).date(date);

            if !opts.account_id.is_empty() {
                builder = builder.account(Uuid::parse_str(&opts.account_id)?);
            }

            if !opts.note.is_empty() {
                builder = builder.note(&opts.note);
            }

            builder.submit()?
        } else {
            let amount = Money::from_str(&opts.outflow)?;
            let account_id = Uuid::parse_str(&opts.account_id)?;
            let payee_id = Uuid::parse_str(&opts.payee_id)?;
            let mut builder = service
                .create_transfer()
                .accounts(account_id, payee_id)
                .amount(amount)
                .date(date);

            if !opts.note.is_empty() {
                builder = builder.note(&opts.note);
            }

            builder.submit()?
        };

    state.get_transactions().insert_row(0,transaction.into())?;
    Ok(())
}

fn duplicate_transaction(id: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let transaction = service.duplicate_transaction(id)?;
    state.get_transactions().push_row(transaction.into())?;
    Ok(())
}

fn load_transactions(state: &TransactionState, service: &Service) -> crate::Result<()> {
    let transactions: Vec<ui::Transaction> = service
        .fetch_transactions()?
        .iter()
        .map(|t| t.into())
        .collect();

    state.set_transactions(ModelRc::new(Rc::new(VecModel::from(transactions))));
    Ok(())
}

fn set_transaction_date(id: &str, date: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let date = Date::strptime("%Y-%m-%d", date)?;
    let transaction = service.set_transaction_date(id, date)?;
    load_transactions(state,service)?;
    Ok(())
}

fn set_transaction_note(id: &str, note: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let transaction = service.set_transaction_note(id, note)?;
    load_transactions(state,service)?;
    Ok(())
}

fn set_transaction_account(id: &str, account_id: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let account_id = Uuid::parse_str(account_id)?;
    let transaction = service.set_transaction_account(id, account_id)?;
    load_transactions(state,service)?;
    Ok(())
}

fn set_transaction_payee(id: &str, account_id: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let account_id = Uuid::parse_str(account_id)?;
    let transaction = service.set_transaction_payee(id, account_id)?;
    load_transactions(state,service)?;
    Ok(())
}

fn set_transaction_outflow(id: &str, amount: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let amount = Money::from_str(amount)?;
    let transaction = service.set_transaction_outflow(id, amount)?;
    load_transactions(state,service)?;
    Ok(())
}

fn set_transaction_inflow(id: &str, amount: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let amount = Money::from_str(amount)?;
    let transaction = service.set_transaction_inflow(id, amount)?;
    load_transactions(state,service)?;
    Ok(())
}

fn set_transaction_category(id: &str, category_id: &str,state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    let category_id = Uuid::parse_str(category_id)?;
    let transaction = service.set_transaction_category(id, category_id)?;
    load_transactions(state,service)?;
    Ok(())
}


#[cfg(test)]
mod test {
    use super::*;
    use jiff::civil::date;
    use mukwa_core::Money;
    use slint::{Model, ToSharedString};

    #[test]
    fn bind_loads_transactions_from_service() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let expense = service
            .create_expense()
            .amount(Money::new(50))
            .date(date(2021, 10, 24))
            .submit()?;
        bind(&window, service)?;

        let transaction_state: TransactionState = window.global();
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction, expense.into());
        Ok(())
    }

    #[test]
    fn delete_transaction() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let expense = service
            .create_expense()
            .amount(Money::new(50))
            .date(date(2021, 10, 24))
            .submit()?;
        let expense2 = service
            .create_expense()
            .amount(Money::new(500))
            .date(date(2021, 10, 24))
            .submit()?;
        bind(&window, service)?;

        let transaction_state: TransactionState = window.global();
        transaction_state.invoke_delete_transaction(expense2.id.to_shared_string());
        let transactions = transaction_state.get_transactions();
        assert_eq!(transactions.iter().len(), 1);
        let transaction = transactions.iter().next().unwrap();
        assert_eq!(transaction, expense.into());
        Ok(())
    }
}
