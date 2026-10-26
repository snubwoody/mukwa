// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::ui::{self, State};
use crate::ui::{MainWindow, TransactionState};
use jiff::Zoned;
use jiff::civil::Date;
use mukwa_core::error::ErrorExt;
use mukwa_core::service::Service;
use mukwa_core::{Error, Money};
use slint::{ComponentHandle, Global, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::str::FromStr;
use tracing::warn;
use uuid::Uuid;

pub fn bind(app: &MainWindow, service: Service) -> crate::Result<()> {
    TransactionController::bind(app, &service)?;
    Ok(())
}

#[derive(Clone)]
struct TransactionController {
    transaction_state: Rc<TransactionState<'static>>,
    service: Service,
}

impl TransactionController {
    pub fn bind(app: &MainWindow, service: &Service) -> crate::Result<Self> {
        let transaction_state: TransactionState = app.global();
        let global_state: State = app.global();

        let controller = Self {
            transaction_state: Rc::new(transaction_state.as_weak().unwrap()),
            service: service.clone(),
        };

        let mut transactions = service.fetch_transactions()?;
        transactions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
        let transactions_list: Vec<ui::Transaction> =
            transactions.iter().map(|t| t.into()).collect();

        transaction_state.set_transactions(ModelRc::new(VecModel::from(transactions_list.clone())));
        transaction_state.set_all_transactions(ModelRc::new(VecModel::from(transactions_list)));
        transaction_state.set_selected_transactions(ModelRc::new(VecModel::default()));

        transaction_state.on_select_transaction({
            let transaction_state = transaction_state.as_weak();
            move |id| {
                let transaction_state = transaction_state.unwrap();
                if transaction_state
                    .get_selected_transactions()
                    .iter()
                    .find(|transaction_id| *transaction_id == id)
                    .is_some()
                {
                    let selected_transactions: VecModel<_> = transaction_state
                        .get_selected_transactions()
                        .iter()
                        .filter(|transaction_id| *transaction_id != id)
                        .collect();
                    transaction_state
                        .set_selected_transactions(ModelRc::new(selected_transactions));
                    return;
                }
                if let Err(err) = transaction_state.get_selected_transactions().push_row(id) {
                    warn!("Failed to select transaction: {err}");
                }
            }
        });

        transaction_state.on_delete_selected_transactions({
            let global_state = global_state.as_weak();
            let controller = controller.clone();
            move || {
                if let Err(err) = controller.delete_selected_transactions() {
                    warn!("Failed to delete transactions: {err}");
                }
                global_state.unwrap().invoke_load_accounts();
            }
        });

        transaction_state.on_delete_transaction({
            let controller = controller.clone();
            let global_state = global_state.as_weak();
            move |id| {
                if let Err(err) = controller.delete_transaction(&id) {
                    warn!("Failed to delete transaction: {err}");
                }
                global_state.unwrap().invoke_load_accounts();
            }
        });

        transaction_state.on_search({
            let controller = controller.clone();
            move |query| {
                if let Err(err) = controller.search(&query) {
                    warn!("Failed to search transactions: {err}");
                }
            }
        });

        transaction_state.on_clear_search({
            let controller = controller.clone();
            move || {
                controller
                    .transaction_state
                    .set_transactions(controller.transaction_state.get_all_transactions());
            }
        });

        transaction_state.on_confirm_transaction({
            let controller = controller.clone();
            move |id| {
                if let Err(err) = controller.confirm_transaction(&id) {
                    warn!("Failed to confirm transaction: {err}");
                }
            }
        });

        transaction_state.on_set_transaction_category({
            let controller = controller.clone();
            move |id, category_id| {
                if let Err(err) = controller.set_transaction_category(&id, &category_id) {
                    warn!(
                        "{}",
                        Error::with_source("Failed to set transaction category", err).report()
                    );
                }
            }
        });

        transaction_state.on_set_transaction_date({
            let controller = controller.clone();
            move |id, date| {
                if let Err(err) = controller.set_transaction_date(&id, &date) {
                    warn!("Failed to set transaction date: {err}");
                }
            }
        });

        transaction_state.on_set_transaction_outflow({
            let global_state = global_state.as_weak();
            let controller = controller.clone();
            move |id, amount| {
                if let Err(err) = controller.set_transaction_outflow(&id, &amount) {
                    warn!("Failed to set transaction outflow: {err}");
                }
                global_state.unwrap().invoke_load_accounts();
            }
        });

        transaction_state.on_set_transaction_inflow({
            let global_state = global_state.as_weak();
            let controller = controller.clone();
            move |id, amount| {
                if let Err(err) = controller.set_transaction_inflow(&id, &amount) {
                    warn!("Failed to set transaction inflow: {err}");
                }
                global_state.unwrap().invoke_load_accounts();
            }
        });

        transaction_state.on_set_transaction_account({
            let controller = controller.clone();
            move |id, account_id| {
                if let Err(err) = controller.set_transaction_account(&id, &account_id) {
                    warn!("Failed to set transaction account: {err}");
                }
            }
        });

        transaction_state.on_set_transaction_payee({
            let controller = controller.clone();
            move |id, account_id| {
                if let Err(err) = controller.set_transaction_payee(&id, &account_id) {
                    warn!("Failed to set transaction payee: {err}");
                }
            }
        });

        transaction_state.on_set_transaction_note({
            let controller = controller.clone();
            move |id, note| {
                if let Err(err) = controller.set_transaction_note(&id, &note) {
                    warn!("Failed to set transaction note: {err}");
                }
            }
        });

        transaction_state.on_create_transaction({
            let global_state = global_state.as_weak();
            let controller = controller.clone();
            move |opts| {
                if let Err(err) = controller
                    .create_transaction(opts)
                    .context("Failed to create transaction")
                {
                    warn!("{}", err.report())
                }
                global_state.unwrap().invoke_load_accounts();
            }
        });

        transaction_state.on_is_transaction_unconfirmed({
            let controller = controller.clone();
            move |id| {
                // Special case for the new transaction row
                if id == "placeholder-transaction" {
                    return false;
                }

                match controller.is_transaction_unconfirmed(&id) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            "{}",
                            Error::with_source(
                                "Failed to check if transaction is unconfirmed",
                                err
                            )
                            .report()
                        );
                        false
                    }
                }
            }
        });

        transaction_state.on_duplicate_transaction({
            let controller = controller.clone();
            move |id| {
                if let Err(err) = controller
                    .duplicate_transaction(&id)
                    .context("Failed to duplicate transaction")
                {
                    warn!("{}", err.report());
                }
            }
        });

        transaction_state.on_total_balance({
            let transaction_state = transaction_state.as_weak();
            move || {
                let transaction_state = transaction_state.unwrap();
                let mut total = Money::ZERO;
                for transaction in transaction_state.get_all_transactions().iter() {
                    total -= Money::from_str(transaction.outflow.as_str()).unwrap_or_default();
                    total += Money::from_str(transaction.inflow.as_str()).unwrap_or_default();
                }
                total.to_shared_string()
            }
        });

        transaction_state.on_load_transactions({
            let controller = controller.clone();
            move || {
                if let Err(err) = controller
                    .load_transactions()
                    .context("Failed to load transactions")
                {
                    warn!("{}", err.report())
                }
            }
        });

        Ok(controller)
    }

    fn confirm_transaction(&self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.confirm_transaction(id)?;
        self.load_transactions()?;
        Ok(())
    }

    fn is_transaction_unconfirmed(&self, id: &str) -> crate::Result<bool> {
        let unconfirmed_transactions: HashSet<Uuid> = self
            .service
            .fetch_unconfirmed_transactions()?
            .iter()
            .copied()
            .collect();
        let id = Uuid::parse_str(id)?;
        Ok(unconfirmed_transactions.contains(&id))
    }

    fn delete_transaction(&self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.delete_transaction(id)?;
        self.load_transactions()?;
        Ok(())
    }

    fn delete_selected_transactions(&self) -> crate::Result<()> {
        let state = &self.transaction_state;
        let service = &self.service;
        if state.get_all_transactions_selected() {
            for id in state
                .get_transactions()
                .iter()
                .map(|transaction| transaction.id)
            {
                let id = Uuid::parse_str(&id)?;
                service.delete_transaction(id)?;
            }
        }

        for id in state.get_selected_transactions().iter() {
            let id = Uuid::parse_str(&id)?;
            service.delete_transaction(id)?;
        }
        self.load_transactions()?;
        state.set_all_transactions_selected(false);
        state.set_selected_transactions(ModelRc::new(VecModel::default()));
        Ok(())
    }

    fn create_transaction(&self, opts: ui::CreateTransactionOpts) -> crate::Result<()> {
        let date = if opts.date.is_empty() {
            Zoned::now().date()
        } else {
            Date::strptime("%Y-%m-%d", &opts.date)?
        };

        let service = &self.service;
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

        self.transaction_state
            .get_transactions()
            .insert_row(0, transaction.into())?;
        Ok(())
    }

    fn search(&self, query: &str) -> crate::Result<()> {
        // TODO: test that it doesnt load transactions
        // TODO: if it's empty then just add all transactions
        // TODO: for some reason / matches zero transactions
        let categories = self.service.fetch_categories()?;
        let accounts = self.service.fetch_accounts()?;
        let category_map: HashMap<SharedString, String> = categories
            .into_iter()
            .map(|category| (category.id.to_shared_string(), category.title))
            .collect();
        let account_map: HashMap<SharedString, String> = accounts
            .into_iter()
            .map(|category| (category.id.to_shared_string(), category.name))
            .collect();

        let query = query.to_ascii_lowercase();
        let mut matched_transactions = vec![];
        for transaction in self.transaction_state.get_all_transactions().iter() {
            if transaction.amount.contains(&query)
                || transaction.note.to_ascii_lowercase().contains(&query)
                || transaction.date.contains(&query)
                || category_map
                    .get(&transaction.category_id)
                    .unwrap_or(&String::new())
                    .to_ascii_lowercase()
                    .contains(&query)
                || account_map
                    .get(&transaction.account_id)
                    .unwrap_or(&String::new())
                    .to_ascii_lowercase()
                    .contains(&query)
                || account_map
                    .get(&transaction.payee_id)
                    .unwrap_or(&String::new())
                    .to_ascii_lowercase()
                    .contains(&query)
            {
                matched_transactions.push(transaction);
            }
        }
        self.transaction_state
            .set_transactions(ModelRc::new(VecModel::from(matched_transactions)));
        Ok(())
    }

    fn duplicate_transaction(&self, id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let transaction = self.service.duplicate_transaction(id)?;
        self.transaction_state
            .get_transactions()
            .push_row(transaction.into())?;
        Ok(())
    }

    fn load_transactions(&self) -> crate::Result<()> {
        let transactions: Vec<ui::Transaction> = self
            .service
            .fetch_transactions()?
            .iter()
            .map(|t| t.into())
            .collect();
        self.transaction_state
            .set_transactions(ModelRc::new(Rc::new(VecModel::from(transactions.clone()))));
        self.transaction_state
            .set_all_transactions(ModelRc::new(Rc::new(VecModel::from(transactions))));
        Ok(())
    }

    fn set_transaction_date(&self, id: &str, date: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let date = Date::strptime("%Y-%m-%d", date)?;
        self.service.set_transaction_date(id, date)?;
        self.load_transactions()?;
        Ok(())
    }

    fn set_transaction_note(&self, id: &str, note: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        self.service.set_transaction_note(id, note)?;
        self.load_transactions()?;
        Ok(())
    }

    fn set_transaction_account(&self, id: &str, account_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let account_id = Uuid::parse_str(account_id)?;
        self.service.set_transaction_account(id, account_id)?;
        self.load_transactions()?;
        Ok(())
    }

    fn set_transaction_payee(&self, id: &str, account_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let account_id = Uuid::parse_str(account_id)?;
        self.service.set_transaction_payee(id, account_id)?;
        self.load_transactions()?;
        Ok(())
    }

    fn set_transaction_outflow(&self, id: &str, amount: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        self.service.set_transaction_outflow(id, amount)?;
        self.load_transactions()?;
        Ok(())
    }

    fn set_transaction_inflow(&self, id: &str, amount: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id)?;
        let amount = Money::from_str(amount)?;
        self.service.set_transaction_inflow(id, amount)?;
        self.load_transactions()?;
        Ok(())
    }

    fn set_transaction_category(&self, id: &str, category_id: &str) -> crate::Result<()> {
        let id = Uuid::parse_str(id).context("Failed to parse transaction id")?;
        let category_id = Uuid::parse_str(category_id).context("Failed to parse category id")?;
        self.service.set_transaction_category(id, category_id)?;
        self.load_transactions()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use jiff::Zoned;
    use jiff::civil::date;
    use mukwa_core::Money;
    use mukwa_core::service::AccountType;
    use slint::{Model, SharedString, ToSharedString};

    use crate::ui::CreateTransactionOpts;

    fn setup(f: impl FnOnce(&Service)) -> crate::Result<TransactionController> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        f(&service);
        let controller = TransactionController::bind(&window, &service)?;
        Ok(controller)
    }

    // TODO: test empty string
    #[test]
    fn search_matches_by_amount() -> crate::Result<()> {
        let controller = setup(|service| {
            let amounts = [3000.0, 300.0, 150.0, 1050.0, 555.0, 0.5, 90.97];
            for amount in amounts {
                service
                    .create_expense()
                    .date(date(9999, 9, 9)) // Should not contain 1s
                    .amount(Money::from_f64(amount))
                    .submit()
                    .unwrap();
            }
        })?;

        let state = controller.transaction_state.clone();
        let test_search = |query: &str, len: usize| {
            state.invoke_search(query.to_shared_string());
            assert_eq!(
                state.get_transactions().iter().len(),
                len,
                "Query: '{}' failed",
                query
            );
        };

        test_search("150", 1);
        test_search("15", 1);
        test_search("5", 4);
        test_search("300", 2);
        test_search("3", 2);
        test_search("1", 2);
        test_search("3000", 1);
        test_search("1050", 1);
        test_search(".9", 1);
        test_search("0.5", 1);
        test_search("0", 7);
        Ok(())
    }

    #[test]
    fn search_matches_by_note() -> crate::Result<()> {
        let controller = setup(|service| {
            let notes = ["PAYMENT", "Youtube Premium", "Burger King"];
            for note in notes {
                service.create_expense().note(note).submit().unwrap();
            }
        })?;

        let state = controller.transaction_state.clone();
        let test_search = |query: &str, len: usize| {
            state.invoke_search(query.to_shared_string());
            assert_eq!(
                state.get_transactions().iter().len(),
                len,
                "Query: '{}' failed",
                query
            );
        };

        test_search("PAY", 1);
        test_search("P", 2);
        test_search("Y", 2);
        test_search("you", 1);
        test_search("premium", 1);
        test_search("p", 2);
        test_search("r", 2);
        test_search("e", 3);
        test_search("ki", 1);
        Ok(())
    }

    #[test]
    fn search_matches_by_account() -> crate::Result<()> {
        let controller = setup(|service| {
            let credit = service
                .create_account("Credit card", AccountType::Credit)
                .unwrap();
            let savings = service
                .create_account("Savings account", AccountType::Cash)
                .unwrap();
            service
                .create_expense()
                .account(credit.id)
                .submit()
                .unwrap();
            service
                .create_expense()
                .account(savings.id)
                .submit()
                .unwrap();
        })?;

        let state = controller.transaction_state.clone();
        let test_search = |query: &str, len: usize| {
            state.invoke_search(query.to_shared_string());
            assert_eq!(
                state.get_transactions().iter().len(),
                len,
                "Query: '{}' failed",
                query
            );
        };

        test_search("i", 2);
        test_search("Credit", 1);
        test_search("car", 1);
        test_search("s", 1);
        test_search("a", 2);
        test_search("A", 2);
        Ok(())
    }

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
        TransactionController::bind(&window, &service)?;

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
        TransactionController::bind(&window, &service)?;

        let transaction_state: TransactionState = window.global();
        transaction_state.invoke_delete_transaction(expense2.id.to_shared_string());
        let transactions = transaction_state.get_transactions();
        assert_eq!(transactions.iter().len(), 1);
        let transaction = transactions.iter().next().unwrap();
        assert_eq!(transaction, expense.into());
        Ok(())
    }

    #[test]
    fn create_expense() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        let account = service.create_account("", AccountType::Cash)?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;

        TransactionController::bind(&window, &service)?;

        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            outflow: "0.00".to_shared_string(),
            category_id: category.id.to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            note: SharedString::from("Pick n Pay"),
            ..Default::default()
        };

        transaction_state.invoke_create_transaction(opts);

        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction.account_id, account.id.to_shared_string());
        assert_eq!(transaction.category_id, category.id.to_shared_string());
        assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
        assert_eq!(transaction.note.as_str(), "Pick n Pay");
        assert_eq!(transaction.inflow.as_str(), "");
        assert_eq!(transaction.payee_id.as_str(), "");
        assert_eq!(transaction.outflow.as_str(), Money::ZERO.to_string());
        Ok(())
    }

    #[test]
    fn create_income() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        let account = service.create_account("", AccountType::Cash)?;
        TransactionController::bind(&window, &service)?;
        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            inflow: "0.00".to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            note: SharedString::from("Pick n Pay"),
            ..Default::default()
        };
        transaction_state.invoke_create_transaction(opts);

        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction.account_id, account.id.to_shared_string());
        assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
        assert_eq!(transaction.note.as_str(), "Pick n Pay");
        assert_eq!(transaction.outflow.as_str(), "");
        assert_eq!(transaction.payee_id.as_str(), "");
        assert_eq!(transaction.inflow.as_str(), Money::ZERO.to_string());
        Ok(())
    }

    #[test]
    fn create_income_ignores_category() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        let account = service.create_account("", AccountType::Cash)?;
        TransactionController::bind(&window, &service)?;
        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            inflow: "0.00".to_shared_string(),
            date: Zoned::now().date().to_shared_string(),
            category_id: SharedString::from("does-not-exist"),
            ..Default::default()
        };
        transaction_state.invoke_create_transaction(opts);

        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction.account_id, account.id.to_shared_string());
        assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
        assert!(transaction.category_id.is_empty());
        assert!(transaction.outflow.is_empty());
        assert!(transaction.payee_id.is_empty());
        assert_eq!(transaction.inflow.as_str(), Money::ZERO.to_string());
        Ok(())
    }

    #[test]
    fn create_transaction_with_default_date() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        let account = service.create_account("", AccountType::Cash)?;
        TransactionController::bind(&window, &service)?;
        let opts = CreateTransactionOpts {
            outflow: "50.00".to_shared_string(),
            account_id: account.id.to_shared_string(),
            ..Default::default()
        };

        transaction_state.invoke_create_transaction(opts);
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction.date, Zoned::now().date().to_shared_string());
        Ok(())
    }

    #[test]
    fn set_outflow_on_an_expense() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        let account = service.create_account("", AccountType::Cash)?;
        TransactionController::bind(&window, &service)?;
        let opts = CreateTransactionOpts {
            outflow: "50.00".to_shared_string(),
            account_id: account.id.to_shared_string(),
            ..Default::default()
        };

        transaction_state.invoke_create_transaction(opts);
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        transaction_state.invoke_set_transaction_outflow(transaction.id, "300".to_shared_string());
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction.outflow.as_str(), Money::new(300).to_string());
        Ok(())
    }

    #[test]
    fn set_outflow_on_an_income() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        let account = service.create_account("", AccountType::Cash)?;
        let group = service.create_category_group("")?;
        let category = service.create_category("", group.id)?;
        TransactionController::bind(&window, &service)?;
        let opts = CreateTransactionOpts {
            account_id: account.id.to_shared_string(),
            inflow: "50.00".to_shared_string(),
            category_id: category.id.to_shared_string(),
            ..Default::default()
        };

        transaction_state.invoke_create_transaction(opts);
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        transaction_state.invoke_set_transaction_outflow(transaction.id, "300".to_shared_string());
        let transaction = transaction_state.get_transactions().iter().next().unwrap();
        assert_eq!(transaction.outflow.as_str(), Money::new(300).to_string());
        assert!(transaction.inflow.is_empty());
        assert!(transaction.category_id.is_empty());
        Ok(())
    }

    #[test]
    fn select_transaction() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        let transaction_state: TransactionState = window.global();
        TransactionController::bind(&window, &service)?;

        transaction_state.invoke_select_transaction("1".to_shared_string());
        transaction_state.invoke_select_transaction("2".to_shared_string());
        let ids: Vec<_> = transaction_state
            .get_selected_transactions()
            .iter()
            .collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"1".to_shared_string()));
        assert!(ids.contains(&"2".to_shared_string()));
        transaction_state.invoke_select_transaction("1".to_shared_string());
        let ids: Vec<_> = transaction_state
            .get_selected_transactions()
            .iter()
            .collect();
        assert_eq!(ids.len(), 1);
        assert!(ids.contains(&"2".to_shared_string()));
        Ok(())
    }

    #[test]
    fn delete_selected_transactions() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        service.create_account("", AccountType::Cash)?;
        let t1 = service.create_expense().submit()?;
        let t2 = service.create_expense().submit()?;
        let transaction_state: TransactionState = window.global();
        TransactionController::bind(&window, &service)?;

        let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
        assert_eq!(transactions.len(), 2);
        transaction_state.invoke_select_transaction(t1.id.to_shared_string());
        transaction_state.invoke_delete_selected_transactions();
        let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, t2.id.to_shared_string());
        let ids: Vec<_> = transaction_state
            .get_selected_transactions()
            .iter()
            .collect();
        assert_eq!(ids.len(), 0);
        assert!(!transaction_state.get_all_transactions_selected());
        Ok(())
    }

    #[test]
    fn delete_all_selected_transactions() -> crate::Result<()> {
        i_slint_backend_testing::init_no_event_loop();
        let window = MainWindow::new()?;
        let service = Service::open_in_memory()?;
        service.create_account("", AccountType::Cash)?;
        let t1 = service.create_expense().submit()?;
        service.create_expense().submit()?;
        let transaction_state: TransactionState = window.global();
        TransactionController::bind(&window, &service)?;

        let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
        assert_eq!(transactions.len(), 2);
        transaction_state.invoke_select_transaction(t1.id.to_shared_string());
        transaction_state.set_all_transactions_selected(true);
        transaction_state.invoke_delete_selected_transactions();
        let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
        assert_eq!(transactions.len(), 0);
        let ids: Vec<_> = transaction_state
            .get_selected_transactions()
            .iter()
            .collect();
        assert_eq!(ids.len(), 0);
        assert!(!transaction_state.get_all_transactions_selected());
        Ok(())
    }
}
