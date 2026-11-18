// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use crate::Money;
use crate::service::Transaction;
use uuid::Uuid;

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

#[cfg(test)]
mod test {
    use jiff::ZonedDateTime;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[quickcheck]
    fn prop_account_balance(incomes: Vec<Money>, expenses: Vec<Money>) {
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
        assert_eq!(
            account_balance(account_id, &transactions),
            incomes - expenses
        )
    }
}
