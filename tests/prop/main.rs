// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

//! Property based fuzz testing

#![allow(unused)]
use clap::Parser;
use fastrand::Rng;
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementRoot};
use mukwa::testing::*;
use mukwa::ui::{self, MainWindow, TransactionState};
use mukwa::ui::{AccountState, Settings};
use mukwa_core::service::Service;
use mukwa_core::{Currency, Money, fmt::CurrencyFormatter};
use slint::{
    ComponentHandle, Model, SharedString, ToSharedString,
    platform::{Key, PointerEventButton},
};
use std::fmt::Debug;
use std::{fs, path::PathBuf};
use tracing::{info, span, trace, trace_span};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

#[derive(Debug)]
enum Action {
    CreateTransaction(CreateTransaction),
    CreateAccount(CreateAccount),
    EditOutflow(EditOutflow),
}

/// Creates a transaction and verifies its creating using the following steps:
/// - Click the "New" button and open the transaction form.
/// - Randomly pick the account.
/// - Enter a random amount in the outflow textfield.
/// - Click the "Save" button and submit the form.
#[derive(Debug)]
struct CreateTransaction;

impl CreateTransaction {
    fn execute(&self, app: &MainWindow, rng: &mut Rng) {
        // TODO: test that the new transaction is visible
        let _span = trace_span!("CreateTransaction").entered();
        let transaction_state: TransactionState = app.global();
        let amount = Money::new(rng.i64(0..));
        let transaction_page = ElementHandle::find_by_element_type_name(app, "TransactionPage")
            .next()
            .unwrap();
        transaction_page
            .query_descendants()
            .match_accessible_role(AccessibleRole::Button)
            .match_accessible_label("New")
            .find_first()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);

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

        let account_state: AccountState = app.global();
        let accounts: Vec<_> = account_state.get_accounts().iter().collect();

        if accounts.is_empty() {
            let transactions_len = transaction_state.get_transactions().iter().len();
            let all_transactions_len = transaction_state.get_all_transactions().iter().len();
            new_transaction_row
                .query_descendants()
                .match_accessible_role(AccessibleRole::Button)
                .match_accessible_label("Save")
                .find_first()
                .unwrap()
                .mock_single_click(PointerEventButton::Left);

            assert_eq!(
                transaction_state.get_transactions().iter().len(),
                transactions_len
            );
            assert_eq!(
                transaction_state.get_all_transactions().iter().len(),
                all_transactions_len
            );
            return;
        }

        let account_id = if accounts.len() > 1 {
            let account_cell = new_transaction_row
                .query_descendants()
                .match_type_name("AccountCell")
                .find_first()
                .unwrap();
            account_cell.mock_single_click(PointerEventButton::Left);

            let combobox = find_by_type_name(&account_cell, "ComboBox");
            assert!(
                combobox.accessible_expanded().unwrap(),
                "Account cell ComboBox did not open"
            );

            let account_options = account_cell
                .query_descendants()
                .match_accessible_role(AccessibleRole::ListItem)
                .find_all();

            let account_option = rng.choice(account_options).unwrap();
            let account_id = account_option.accessible_id().unwrap();
            account_option.mock_single_click(PointerEventButton::Left);
            account_id
        } else {
            accounts[0].id.to_shared_string()
        };

        let outflow_cell = new_transaction_row
            .query_descendants()
            .match_id("NewTransactionRow::outflow-cell")
            .find_first()
            .unwrap();

        outflow_cell.mock_single_click(PointerEventButton::Left);
        // Clear the text field first
        i_slint_backend_testing::send_key_combo(
            app,
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
        i_slint_backend_testing::send_keyboard_string_sequence(app, &amount.to_string());
        i_slint_backend_testing::send_key_combo(app, [Key::Return]);

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

        trace!(amount=?amount,account_id=?account_id,"Created new transaction");

        let mut transactions: Vec<_> = transaction_state.get_transactions().iter().collect();
        let mut all_transactions: Vec<_> =
            transaction_state.get_all_transactions().iter().collect();

        transactions.sort_by(|a, b| {
            a.id.parse::<Uuid>()
                .unwrap()
                .cmp(&b.id.parse().unwrap())
                .reverse()
        });
        all_transactions.sort_by(|a, b| {
            a.id.parse::<Uuid>()
                .unwrap()
                .cmp(&b.id.parse().unwrap())
                .reverse()
        });

        let assert_transaction = |transaction: &ui::Transaction| {
            assert_eq!(transaction.account_id, account_id);
            assert_eq!(transaction.outflow, amount.to_shared_string());
        };
        assert_transaction(&all_transactions[0]);
        assert_transaction(&transactions[0]);
    }
}

#[derive(Debug)]
struct EditOutflow;

impl EditOutflow {
    fn execute(&self, app: &MainWindow, rng: &mut Rng) {
        let _span = trace_span!("EditOutflow").entered();
        // TODO: check negative values
        // TODO: test that the balance in the transaction changes
        // TODO: negative outflow converts to expense
        // FIXME: is it generating the same amount?
        let amount = Money::new(rng.i64(0..));
        let root_element = app.root_element();

        let transactions = find_all_by_type_name(&root_element, "TransactionRow");
        if transactions.is_empty() {
            return;
        }

        let transaction = rng.choice(&transactions).unwrap();
        let transaction_id = transaction.accessible_id().unwrap();

        let outflow_cell = find_by_id(transaction, "TransactionRow::outflow-cell");
        outflow_cell.mock_single_click(PointerEventButton::Left);
        // Clear the text field
        outflow_cell.mock_single_click(PointerEventButton::Left);
        outflow_cell.mock_single_click(PointerEventButton::Left);
        i_slint_backend_testing::send_key_combo(app, [Key::Backspace]);
        i_slint_backend_testing::send_keyboard_string_sequence(app, &amount.to_string());
        i_slint_backend_testing::send_key_combo(app, [Key::Return]);

        let state: TransactionState = app.global();
        let transactions: Vec<_> = state.get_all_transactions().iter().collect();
        let transaction = transactions
            .iter()
            .find(|t| t.id == transaction_id.to_shared_string())
            .unwrap();

        trace!(amount=?amount,transaction_id=?transaction_id,"Edited outflow");
        assert_eq!(transaction.amount, amount.to_shared_string());

        let settings: Settings = app.global();
        settings.set_font_family("CAD".to_shared_string());
        let formatter = CurrencyFormatter::with_currency(Currency::CAD);
    }
}

#[derive(Debug)]
struct CreateAccount;

impl CreateAccount {
    fn execute(&self, app: &MainWindow, rng: &mut Rng) {
        let _span = trace_span!("CreateAccount").entered();
        let state: AccountState = app.global();
        let accounts_len = state.get_accounts().iter().len();
        let balance = Money::new(rng.i64(0..));
        let name = gen_string(rng);

        let root_element = app.root_element();
        find_by_accessible_label(&root_element, "Create account")
            .mock_single_click(PointerEventButton::Left);
        find_by_id(&root_element, "CreateAccountDialog::name")
            .mock_single_click(PointerEventButton::Left);

        i_slint_backend_testing::send_keyboard_string_sequence(app, &name);
        i_slint_backend_testing::send_keyboard_string_sequence(app, &SharedString::from(Key::Tab));
        i_slint_backend_testing::send_keyboard_string_sequence(app, &balance.to_string());

        find_by_id(&root_element, "CreateAccountDialog::create-account")
            .mock_single_click(PointerEventButton::Left);

        trace!(name=name,balance=?balance,"Created account");
        // TODO: assert that the elements are not visible

        assert!(
            ElementHandle::find_by_element_id(app, "CreateAccountDialog::create-account")
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
}

pub struct TestRunner {
    rng: Rng,
    app: MainWindow,
    service: Service,
    succeeded: bool,
    current_run: u64,
    seed: u64,
}

impl TestRunner {
    fn new(seed: u64) -> TestRunner {
        let (app, service) = setup().unwrap();
        TestRunner {
            rng: Rng::with_seed(seed),
            app,
            service,
            succeeded: false,
            current_run: 0,
            seed,
        }
    }

    fn run(&mut self) -> mukwa::Result<()> {
        // TODO: use drop guard instead
        let state: AccountState = self.app.global();
        // TODO: account options has len of 7 every time?
        // TODO: don't create default accounts
        state.invoke_create_account(
            "My account".to_shared_string(),
            ui::AccountType::Cash,
            "".to_shared_string(),
        );
        state.invoke_load_accounts();

        let all_actions = [
            Action::CreateAccount(CreateAccount),
            Action::EditOutflow(EditOutflow),
            Action::CreateTransaction(CreateTransaction),
        ];

        // Pick a random set of actions to run
        let len = self.rng.usize(1..=all_actions.len());
        let actions = self.rng.choose_multiple(all_actions, len);

        for run in 1..=3 {
            self.current_run = run;

            for i in 0..self.rng.u8(1..=10) {
                let action = self.rng.choice(&actions).unwrap();
                match action {
                    Action::CreateTransaction(act) => act.execute(&self.app, &mut self.rng),
                    Action::EditOutflow(act) => act.execute(&self.app, &mut self.rng),
                    Action::CreateAccount(act) => act.execute(&self.app, &mut self.rng),
                }
            }
        }

        self.succeeded = true;
        Ok(())
    }
}

impl Drop for TestRunner {
    fn drop(&mut self) {
        if self.succeeded {
            return;
        }

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("test-output")
            .join(format!("test-{}", self.seed));
        let _ = fs::create_dir_all(&dir);

        let mut content = format!("Seed: {}\n", self.seed);
        content += &format!("Failed on run: {}", self.current_run);

        let path = dir.join("state.txt");
        fs::write(&path, content).unwrap();

        let _ = self.service.vacuum(dir.join("data.sqlite"));
        eprintln!("Saved test state to {}", dir.display());
    }
}

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    seed: Option<u64>,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

fn gen_string(rng: &mut Rng) -> String {
    let mut alphabet = b"abcdefg ".to_vec();
    rng.shuffle(&mut alphabet);

    let count = rng.usize(1..=alphabet.len());
    alphabet.truncate(count);

    let mut result = vec![];
    let count = rng.usize(0..8);
    for _ in 0..count {
        result.push(alphabet[rng.usize(0..alphabet.len())]);
    }
    String::from_utf8(result).unwrap()
}

fn main() -> mukwa_core::Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or(fastrand::u64(..));
    if cli.verbose {
        let std_io_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_writer(std::io::stdout);

        let level = "warn,simulacrum=trace";
        tracing_subscriber::registry()
            .with(EnvFilter::new(level))
            .with(std_io_layer)
            .init();
    }

    println!("Running test runner with seed {seed}");
    println!("To re-run this test use the command:");
    println!("cargo run --bin simulacrum -- --seed {seed}");

    let mut runner = TestRunner::new(seed);
    runner.run()?;
    println!("No errors found");
    Ok(())
}
