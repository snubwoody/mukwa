// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// TODO:
// - Maybe prop testing for each page
// - Deterministic seeding
// - Maybe a trait

use fastrand::Rng;
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use slint::{ComponentHandle, Model, ToSharedString, platform::PointerEventButton};

use crate::ui::{self, MainWindow, State, TransactionState};

enum Action {
    CreateTransaction,
    EditTransaction,
    DeleteTransaction,
}

pub struct TestRunner {
    rng: Rng,
    app: MainWindow,
}

impl TestRunner {
    fn new(rng: Rng) -> TestRunner {
        let (app, _) = super::setup().unwrap();
        TestRunner { rng, app }
    }

    fn run(&mut self) -> crate::Result<()> {
        let action = Action::CreateTransaction;
        let state: State = self.app.global();
        // TODO: account options has len of 7 every time?
        for _ in 0..self.rng.u8(..) {
            state.invoke_create_account(
                "My account".to_shared_string(),
                ui::AccountType::Cash,
                "".to_shared_string(),
            );
        }

        self.create_transaction();
        Ok(())
    }

    // TODO: use the same RNG for seeding
    fn create_transaction(&mut self) {
        let transaction_page =
            ElementHandle::find_by_element_type_name(&self.app, "TransactionPage")
                .next()
                .unwrap();
        transaction_page
            .query_descendants()
            .match_accessible_role(AccessibleRole::Button)
            .match_accessible_label("New")
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        // TODO: test that it's closed after saving
        assert!(
            transaction_page
                .query_descendants()
                .match_type_name("NewTransactionRow")
                .find_first()
                .is_some()
        );

        let new_transaction_row = transaction_page
            .query_descendants()
            .match_type_name("NewTransactionRow")
            .find_first()
            .unwrap();

        // TODO: fill items here

        // TODO: also test no accounts?
        new_transaction_row
            .query_descendants()
            .match_type_name("AccountCell")
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        let account_options = new_transaction_row
            .query_descendants()
            .match_accessible_role(AccessibleRole::ListItem)
            .find_all();

        dbg!(account_options.len());
        let account_option = self.rng.choice(account_options).unwrap();
        let account_id = account_option.accessible_id().unwrap();
        account_option.mock_single_click(PointerEventButton::Left);

        new_transaction_row
            .query_descendants()
            .match_accessible_role(AccessibleRole::Button)
            .match_accessible_label("Save")
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        assert!(
            transaction_page
                .query_descendants()
                .match_type_name("NewTransactionRow")
                .find_first()
                .is_none()
        );

        let transaction_state: TransactionState = self.app.global();
        let transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
        let all_transactions: Vec<_> = transaction_state.get_all_transactions().iter().collect();
        assert_eq!(transactions.len(), 1);
        assert_eq!(all_transactions.len(), 1);
        assert_eq!(transactions[0].account_id, account_id);
        assert_eq!(all_transactions[0].account_id, account_id);
    }
}

#[test]
fn test() -> crate::Result<()> {
    let rng = Rng::new();
    let mut runner = TestRunner::new(rng);
    runner.run()?;
    Ok(())
}
