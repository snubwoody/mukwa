// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::collections::HashMap;

use crate::Money;
use crate::service::{Account, AccountType, Budget, Transaction, TransactionKind, TransactionType};
use uuid::Uuid;

/// Returns the total amount of money that is available to assign to budgets.
///
/// Calculated by subtracting the total amount assigned to all budgets from the total
/// income into cash accounts.
pub fn available_to_assign(
    accounts: &[Account],
    transactions: &[Transaction],
    budgets: &[Budget],
) -> Money {
    // TODO: should credit accounts be included?
    let cash_accounts: HashMap<Uuid, Account> = accounts
        .into_iter()
        .filter(|account| account.account_type == AccountType::Cash)
        .map(|account| (account.id, account.clone()))
        .collect();

    let mut total_assignable = Money::ZERO;

    for transaction in transactions {
        // Add money deposited into cash accounts
        if transaction.transaction_type() == TransactionType::Income
            && let Some(account_id) = transaction.receiver_id
            && cash_accounts.contains_key(&account_id)
        {
            total_assignable += transaction.amount;
        }
    }

    let total_assigned: Money = budgets.iter().map(|budget| budget.amount).sum();
    total_assignable - total_assigned
}

/// Calculates the account balance.
pub fn account_balance(id: Uuid, transactions: &[Transaction]) -> Money {
    // TODO: test self transfers, account filters, negative balances
    let total_outflow: Money = transactions
        .iter()
        .filter(|t| t.sender_id == Some(id))
        .map(|t| t.amount)
        .sum();
    let total_inflow: Money = transactions
        .iter()
        .filter(|t| t.receiver_id == Some(id))
        .map(|t| t.amount)
        .sum();
    total_inflow - total_outflow
}

// #[cfg(test)]
// impl quickcheck::Arbitrary for jiff::civil::Date {
//     fn arbitrary(g: &mut quickcheck::Gen) -> Self {
//         use jiff::civil::Date;
//         Date::new(i16::arbitrary(g), i8::arbitrary(g), i8::arbitrary(g)).unwrap()
//     }
// }

#[cfg(test)]
impl quickcheck::Arbitrary for TransactionKind {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let values = &[
            TransactionKind::Correction,
            TransactionKind::Regular,
            TransactionKind::StartingBalance,
        ];
        *g.choose(values).unwrap()
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for AccountType {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let values = &[AccountType::Cash, AccountType::Credit];
        *g.choose(values).unwrap()
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Account {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Account {
            id: Uuid::now_v7(),
            name: String::arbitrary(g),
            account_type: AccountType::arbitrary(g),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Budget {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        Budget {
            id: Uuid::now_v7(),
            amount: Money::arbitrary(g),
            month: i64::arbitrary(g),
            year: i64::arbitrary(g),
            category_id: Uuid::now_v7(),
        }
    }
}

#[cfg(test)]
impl quickcheck::Arbitrary for Transaction {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use jiff::ZonedDateTime;

        Transaction {
            id: Uuid::now_v7(),
            sender_id: None,
            receiver_id: None,
            category_id: None,
            date: ZonedDateTime::now().date(),
            note: Option::arbitrary(g),
            amount: Money::arbitrary(g),
            kind: TransactionKind::arbitrary(g),
            audited: bool::arbitrary(g),
        }
    }
}

#[cfg(test)]
mod test {
    use jiff::ZonedDateTime;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn account_balance_filters_by_id() {
        let account1 = Account {
            id: Uuid::now_v7(),
            name: String::new(),
            account_type: AccountType::Cash,
        };
        let account2 = Account {
            id: Uuid::now_v7(),
            name: String::new(),
            account_type: AccountType::Cash,
        };

        let transactions = vec![
            Transaction {
                receiver_id: Some(account1.id),
                amount: Money::new(500),
                ..Default::default()
            },
            Transaction {
                receiver_id: Some(account2.id),
                amount: Money::new(100),
                ..Default::default()
            },
        ];
        assert_eq!(
            super::account_balance(account1.id, &transactions),
            Money::new(500)
        );
    }

    #[test]
    fn account_balance_income_overflow() {
        let id = Uuid::now_v7();
        let transactions = vec![
            Transaction {
                receiver_id: Some(id),
                amount: Money::MAX,
                ..Default::default()
            },
            Transaction {
                receiver_id: Some(id),
                amount: Money::new(100),
                ..Default::default()
            },
        ];
        assert_eq!(super::account_balance(id, &transactions), Money::MAX);
    }

    #[test]
    fn account_balance_expense_overflow() {
        let id = Uuid::now_v7();
        let transactions = vec![
            Transaction {
                sender_id: Some(id),
                amount: Money::MAX,
                ..Default::default()
            },
            Transaction {
                sender_id: Some(id),
                amount: Money::new(100),
                ..Default::default()
            },
        ];
        super::account_balance(id, &transactions);
    }

    #[quickcheck]
    fn prop_available_to_assign(
        accounts: Vec<Account>,
        transactions: Vec<Transaction>,
        budgets: Vec<Budget>,
    ) -> bool {
        let income: Money = transactions
            .iter()
            .filter(|t| t.transaction_type() == TransactionType::Income)
            .map(|t| t.amount)
            .sum();

        let assigned: Money = budgets.iter().map(|b| b.amount).sum();
        // TODO: available_to_assign excludes credit accounts
        let available = available_to_assign(&accounts, &transactions, &budgets);
        available == income - assigned
    }

    #[quickcheck]
    fn prop_account_balance(incomes: Vec<Money>, expenses: Vec<Money>) -> bool {
        let account_id = Uuid::now_v7();
        let mut transactions = vec![];
        for income in &incomes {
            transactions.push(Transaction {
                id: Uuid::now_v7(),
                amount: *income,
                sender_id: None,
                receiver_id: Some(account_id),
                category_id: None,
                date: ZonedDateTime::now().date(),
                note: None,
                kind: crate::service::TransactionKind::Regular,
                audited: false,
            });
        }

        for expense in &expenses {
            transactions.push(Transaction {
                id: Uuid::now_v7(),
                amount: *expense,
                receiver_id: None,
                sender_id: Some(account_id),
                category_id: None,
                date: ZonedDateTime::now().date(),
                note: None,
                kind: crate::service::TransactionKind::Regular,
                audited: false,
            });
        }

        let incomes: Money = incomes.iter().copied().sum();
        let expenses: Money = expenses.iter().copied().sum();
        account_balance(account_id, &transactions) == incomes - expenses
    }
}
