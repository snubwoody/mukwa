// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui;
use crate::ui::{CategoryState, MainWindow, TransactionState};
use mukwa_core::service::{Service, Transaction};
use slint::{ComponentHandle, Global, ModelRc, VecModel};
use std::rc::Rc;
use tracing::{info, warn};
use uuid::Uuid;

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

    Ok(())
}

fn delete_transaction(id: &str, state: &TransactionState, service: &Service) -> crate::Result<()> {
    let id = Uuid::parse_str(id)?;
    service.delete_transaction(id)?;
    load_transactions(state, service)?;
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
