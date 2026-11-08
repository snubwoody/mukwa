// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

//! Property based fuzz testing

// TODO:
// - Maybe prop testing for each page
// - Deterministic seeding
// - Maybe a trait
// - maybe save the database to a file on failure

use std::{fs, path::PathBuf};

use fastrand::Rng;
use i_slint_backend_testing::{AccessibleRole, ElementHandle};
use mukwa_core::Money;
use slint::{
    ComponentHandle, Model, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};

use crate::ui::{self, MainWindow, State, TransactionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Action {
    CreateTransaction = 0,
    CreateAccount = 1,
}

impl Action {
    const MAX: u8 = Action::CreateAccount as u8;
}

impl From<u8> for Action {
    fn from(value: u8) -> Self {
        match value {
            x if x == Action::CreateTransaction as u8 => Action::CreateTransaction,
            x if x == Action::CreateAccount as u8 => Action::CreateAccount,
            _ => panic!("Cannot convert {value} into Action"),
        }
    }
}

pub struct TestRunner {
    rng: Rng,
    app: MainWindow,
    succeeded: bool,
    current_run: u64,
    seed: u64,
}

impl TestRunner {
    fn new(seed: u64) -> TestRunner {
        let (app, _) = super::setup().unwrap();
        TestRunner {
            rng: Rng::with_seed(seed),
            app,
            succeeded: false,
            current_run: 0,
            seed,
        }
    }

    fn run(&mut self) -> crate::Result<()> {
        let state: State = self.app.global();
        // TODO: account options has len of 7 every time?
        for _ in 0..self.rng.u8(..) {
            state.invoke_create_account(
                "My account".to_shared_string(),
                ui::AccountType::Cash,
                "".to_shared_string(),
            );
        }

        for run in 1..3 {
            self.current_run = run;
            // TODO: maybe add enum weights so that certain things are more likely,
            // to prevent things from being too uniform
            // TODO: pick a random subset of actions to choose from?
            // Pick a random combination of actions
            let actions: Vec<_> = (1..self.rng.u8(2..10))
                .map(|_| Action::from(self.rng.u8(0..=Action::MAX)))
                .collect();
            for action in actions {
                match action {
                    Action::CreateAccount => {
                        self.create_account();
                    }
                    Action::CreateTransaction => {
                        self.create_transaction();
                    }
                }
            }
        }

        self.succeeded = true;
        Ok(())
    }

    fn create_account(&mut self) {
        let state: State = self.app.global();
        let accounts_len = state.get_accounts().iter().len();
        let balance = Money::new(self.rng.i64(0..));
        let name = "My account";

        ElementHandle::find_by_accessible_label(&self.app, "Create account")
            .next()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        ElementHandle::find_by_element_id(&self.app, "CreateAccountDialog::name")
            .next()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

        i_slint_backend_testing::send_keyboard_string_sequence(&self.app, name);
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
        let account = &accounts[accounts.len() - 1];
        assert_eq!(account.balance, balance.to_shared_string());
        assert_eq!(account.name, name);
    }

    /// Creates a transaction and verifies its creating using the following steps:
    /// - Click the "New" button and open the transaction form.
    /// - Randomly pick the account.
    /// - Enter a random amount in the outflow textfield.
    /// - Click the "Save" button and submit the form.
    fn create_transaction(&mut self) {
        let amount = Money::new(self.rng.i64(0..));
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
        let account_cell = new_transaction_row
            .query_descendants()
            .match_type_name("AccountCell")
            .find_first()
            .unwrap();

        account_cell.mock_single_click(PointerEventButton::Left);

        let account_options = account_cell
            .query_descendants()
            .match_accessible_role(AccessibleRole::ListItem)
            .find_all();

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
        // dbg!(transactions);
    }
}

impl Drop for TestRunner {
    fn drop(&mut self) {
        if self.succeeded {
            return;
        }

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-output");
        fs::create_dir_all(&dir).unwrap();
        let mut content = format!("Seed: {}\n", self.seed);
        content += &format!("Failed on run: {}", self.current_run);
        let path = dir.join(format!("test-{}.txt", self.seed));
        fs::write(&path, content).unwrap();
        eprintln!("Saved test state to {}", path.display());
    }
}

#[test]
fn test() -> crate::Result<()> {
    let seed = if let Ok(seed) = std::env::var("TEST_SEED")
        && let Ok(seed) = seed.parse::<u64>()
    {
        seed
    } else {
        fastrand::u64(..)
    };

    eprintln!("Running test runner with seed {seed}");
    eprintln!("To re-run this test use the command:");
    eprintln!("TEST_SEED={seed} cargo test");

    let mut runner = TestRunner::new(seed);
    runner.run()?;
    Ok(())
}
