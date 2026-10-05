// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, VecModel};
use crate::ui::{MainWindow, TransactionState};
use mukwa_core::service::Service;
use crate::ui;

pub fn bind(window: &MainWindow, service: Service) -> crate::Result<()> {
    let transaction_state: TransactionState = window.global();
    let mut transactions = service.fetch_transactions()?;
    transactions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
    let transactions_list: Vec<ui::Transaction> =
        transactions.iter().map(|t| t.into()).collect();

    transaction_state.set_transactions(ModelRc::new(VecModel::from(transactions_list)));

    Ok(())
}

#[cfg(test)]
mod test {
    use jiff::civil::date;
    use slint::{Model, ToSharedString};
    use mukwa_core::Money;
    use super::*;

    #[test]
    fn bind_loads_transactions_from_service() -> crate::Result<()>{
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let expense = service.create_expense().amount(Money::new(50)).date(date(2021,10,24)).submit()?;
        bind(&window,service)?;

        let transaction_state: TransactionState = window.global();
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction,expense.into());
        Ok(())
    }
}