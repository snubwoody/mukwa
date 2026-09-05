// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

//! Slint auto generated code.
use mukwa_core::Money;
use slint::{SharedString, ToSharedString};

slint::include_modules!();

impl From<jiff::civil::Date> for Date {
    fn from(value: jiff::civil::Date) -> Self {
        Date {
            year: value.year().into(),
            month: value.month().into(),
            day: value.day().into(),
        }
    }
}

impl From<mukwa_core::service::Account> for Account {
    fn from(account: mukwa_core::service::Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.into(),
            account_type: account.account_type.into(),
            balance: Money::ZERO.to_shared_string(),
        }
    }
}

impl From<mukwa_core::service::Account> for ComboBoxItem {
    fn from(account: mukwa_core::service::Account) -> Self {
        Self {
            value: account.id.to_string().into(),
            text: account.name.to_string().into(),
        }
    }
}

impl From<&mukwa_core::service::Account> for ComboBoxItem {
    fn from(account: &mukwa_core::service::Account) -> Self {
        Self {
            value: account.id.to_string().into(),
            text: account.name.to_string().into(),
        }
    }
}

impl From<&mukwa_core::service::Account> for Account {
    fn from(account: &mukwa_core::service::Account) -> Self {
        Self {
            id: account.id.to_string().into(),
            name: account.name.clone().into(),
            account_type: account.account_type.into(),
            balance: Money::ZERO.to_shared_string(),
        }
    }
}

impl From<&mukwa_core::service::AccountType> for AccountType {
    fn from(value: &mukwa_core::service::AccountType) -> Self {
        match value {
            mukwa_core::service::AccountType::Cash => Self::Cash,
            mukwa_core::service::AccountType::Credit => Self::Credit,
        }
    }
}

impl From<mukwa_core::service::AccountType> for AccountType {
    fn from(value: mukwa_core::service::AccountType) -> Self {
        match value {
            mukwa_core::service::AccountType::Cash => Self::Cash,
            mukwa_core::service::AccountType::Credit => Self::Credit,
        }
    }
}

impl From<mukwa_core::service::Budget> for Budget {
    fn from(value: mukwa_core::service::Budget) -> Self {
        Self {
            id: value.id.to_shared_string(),
            amount: value.amount.to_shared_string(),
            year: value.year as i32,
            month: value.month as i32,
            category_id: value.category_id.to_shared_string(),
        }
    }
}

impl From<&mukwa_core::service::Budget> for Budget {
    fn from(value: &mukwa_core::service::Budget) -> Self {
        Self {
            id: value.id.to_shared_string(),
            amount: value.amount.to_shared_string(),
            year: value.year as i32,
            month: value.month as i32,
            category_id: value.category_id.to_shared_string(),
        }
    }
}

impl From<mukwa_core::service::Category> for Category {
    fn from(value: mukwa_core::service::Category) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            group_id: value.group_id.to_shared_string(),
        }
    }
}

impl From<mukwa_core::service::CategoryGroup> for CategoryGroup {
    fn from(value: mukwa_core::service::CategoryGroup) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            is_meta: value.is_meta
        }
    }
}

impl From<&mukwa_core::service::CategoryGroup> for CategoryGroup {
    fn from(value: &mukwa_core::service::CategoryGroup) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            is_meta: value.is_meta
        }
    }
}

impl From<&mukwa_core::service::Category> for Category {
    fn from(value: &mukwa_core::service::Category) -> Self {
        Self {
            id: value.id.to_shared_string(),
            title: value.title.to_shared_string(),
            group_id: value.group_id.to_shared_string(),
        }
    }
}

impl From<mukwa_core::service::Category> for ComboBoxItem {
    fn from(value: mukwa_core::service::Category) -> Self {
        Self {
            value: value.id.to_string().into(),
            text: value.title.to_string().into(),
        }
    }
}

impl From<&mukwa_core::service::Category> for ComboBoxItem {
    fn from(value: &mukwa_core::service::Category) -> Self {
        Self {
            value: value.id.to_string().into(),
            text: value.title.to_string().into(),
        }
    }
}

impl From<mukwa_core::service::TransactionType> for TransactionType {
    fn from(value: mukwa_core::service::TransactionType) -> Self {
        match value {
            mukwa_core::service::TransactionType::Expense => TransactionType::Expense,
            mukwa_core::service::TransactionType::Income => TransactionType::Income,
            mukwa_core::service::TransactionType::Transfer => TransactionType::Transfer,
        }
    }
}

impl From<mukwa_core::service::Transaction> for Transaction {
    fn from(value: mukwa_core::service::Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        let transaction_type = value.transaction_type();
        let inflow = if transaction_type == mukwa_core::service::TransactionType::Income {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let outflow = if transaction_type == mukwa_core::service::TransactionType::Transfer
            || transaction_type == mukwa_core::service::TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let note = value.note.unwrap_or_default().to_shared_string();

        let account_id = match transaction_type {
            mukwa_core::service::TransactionType::Income => {
                value.receiver_id.unwrap().to_shared_string()
            }
            _ => value.sender_id.unwrap().to_shared_string(),
        };

        let payee_id = match transaction_type {
            mukwa_core::service::TransactionType::Transfer => {
                value.receiver_id.unwrap().to_shared_string()
            }
            _ => SharedString::new(),
        };

        Self {
            id: value.id.to_shared_string(),
            account_id,
            payee_id,
            category_id: category_id.to_shared_string(),
            date: value.date.to_shared_string(),
            outflow,
            note,
            inflow,
            amount: value.amount.to_shared_string(),
            transaction_type: transaction_type.into(),
        }
    }
}

impl From<&mukwa_core::service::Transaction> for Transaction {
    fn from(value: &mukwa_core::service::Transaction) -> Self {
        let category_id = match value.category_id {
            Some(id) => id.to_string(),
            None => String::new(),
        };

        let transaction_type = value.transaction_type();
        let inflow = if transaction_type == mukwa_core::service::TransactionType::Income {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let outflow = if transaction_type == mukwa_core::service::TransactionType::Transfer
            || transaction_type == mukwa_core::service::TransactionType::Expense
        {
            value.amount.to_shared_string()
        } else {
            SharedString::new()
        };

        let note = value.note.clone().unwrap_or_default().to_shared_string();

        let account_id = match transaction_type {
            mukwa_core::service::TransactionType::Income => {
                value.receiver_id.unwrap().to_shared_string()
            }
            _ => value.sender_id.unwrap().to_shared_string(),
        };

        let payee_id = match transaction_type {
            mukwa_core::service::TransactionType::Transfer => {
                value.receiver_id.unwrap().to_shared_string()
            }
            _ => SharedString::new(),
        };

        Self {
            id: value.id.to_string().into(),
            account_id,
            payee_id,
            category_id: category_id.into(),
            note,
            date: value.date.to_string().into(),
            outflow,
            inflow,
            amount: value.amount.to_shared_string(),
            transaction_type: transaction_type.into(),
        }
    }
}
