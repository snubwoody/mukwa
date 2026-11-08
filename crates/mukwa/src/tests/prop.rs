// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

//! Property-based testing.

// TODO:
// - Maybe prop testing for each page
// - Deterministic seeding
// - Maybe a trait

use fastrand::Rng;
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use mukwa_core::Money;
use slint::{
    ComponentHandle, Model, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};

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

        self.create_account();
        self.create_transaction();
        Ok(())
    }

    fn create_account(&mut self) {
        let state: State = self.app.global();
        let accounts_len = state.get_accounts().iter().len();
        let balance = Money::new(self.rng.i64(..));
        let name = "My account";

        ElementHandle::find_by_accessible_label(&self.app, "Create account")
            .next()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        ElementHandle::find_by_element_id(&self.app, "CreateAccountDialog::name")
            .next()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        i_slint_backend_testing::send_keyboard_string_sequence(
            &self.app,
            name,
        );
        i_slint_backend_testing::send_keyboard_string_sequence(
            &self.app,
            &SharedString::from(Key::Tab),
        );
        i_slint_backend_testing::send_keyboard_string_sequence(&self.app, &balance.to_string());

        ElementHandle::find_by_element_id(&self.app, "CreateAccountDialog::create-account")
            .next()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        // TODO: assert that the elements are not visible

        assert!(
            ElementHandle::find_by_element_id(&self.app, "CreateAccountDialog::create-account")
                .next()
                .is_none(),
            "CreateAccountDialog::create-account should not be visible"
        );

        let accounts: Vec<_> = state.get_accounts().iter().collect();
        assert_eq!(
            accounts.len(),
            accounts_len + 1,
            "Test should have created a new account"
        );
        let account = &accounts[accounts.len()-1];
        assert_eq!(account.balance,balance.to_shared_string());
        assert_eq!(account.name,name);
    }

    /// Creates a transaction and verifies its creating using the following steps:
    /// - Click the "New" button and open the transaction form.
    /// - Randomly pick the account.
    /// - Enter a random amount in the outflow textfield.
    /// - Click the "Save" button and submit the form.
    fn create_transaction(&mut self) {
        // FIXME: panicking on unbounded inputs
        let amount = Money::new(self.rng.i64(0..10000));
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

        let outflow_cell = new_transaction_row
            .query_descendants()
            .match_id("NewTransactionRow::outflow-cell")
            .find_first()
            .unwrap();

        outflow_cell.mock_single_click(PointerEventButton::Left);
        // Clear the text field first
        i_slint_backend_testing::send_key_combo(
            &self.app,
            [
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::RightArrow,
                Key::Backspace,
                Key::Backspace,
                Key::Backspace,
            ],
        );

        // TODO: Implement better random number inputs
        i_slint_backend_testing::send_keyboard_string_sequence(&self.app, &amount.to_string());
        i_slint_backend_testing::send_key_combo(&self.app, [Key::Return]);

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
        let assert_transaction = |transaction: &ui::Transaction| {
            assert_eq!(transaction.account_id, account_id);
            assert_eq!(transaction.outflow, amount.to_shared_string());
        };
        assert_transaction(&all_transactions[0]);
        assert_transaction(&transactions[0]);
        dbg!(transactions);
    }
}

#[test]
fn test() -> crate::Result<()> {
    let rng = Rng::new();
    let mut runner = TestRunner::new(rng);
    runner.run()?;
    Ok(())
}
