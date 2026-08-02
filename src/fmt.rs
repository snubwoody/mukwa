// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Utilities for formatting dates and money.

use crate::{Currency, Money};
use jiff::civil::Date;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone, PartialEq, Debug)]
pub struct CurrencyFormatter {
    currency: Currency,
}

impl CurrencyFormatter {
    /// Creates a new currency formatter.
    pub fn new() -> Self {
        CurrencyFormatter {
            currency: Currency::USD,
        }
    }

    /// Set the currency symbol
    pub fn set_currency(&mut self, currency: Currency) {
        self.currency = currency;
    }

    /// Returns the currency symbol.
    pub fn currency(&self) -> Currency {
        self.currency
    }

    /// Formats [`Money`] as a currency string.
    pub fn format_money(&self, value: Money) -> crate::Result<String> {
        let symbol = self.currency.symbol().to_owned();
        #[cfg(target_os = "windows")]
        {
            use windows::CurrencyFormatOptions;
            let mut opts = CurrencyFormatOptions::load_from_sys()?;
            opts.currency_symbol = symbol;
            windows::format_money(value, &opts)
        }

        #[cfg(not(target_os = "windows"))]
        {
            let precision = self.currency.precision().unwrap_or(2) as usize;
            Ok(format!("{}{:0precision$}", symbol, value))
        }
    }
}

impl Default for CurrencyFormatter {
    fn default() -> Self {
        CurrencyFormatter::new()
    }
}

pub fn format_date(date: Date) -> crate::Result<String> {
    #[cfg(target_os = "windows")]
    return windows::format_date(date);

    #[cfg(target_os = "macos")]
    return macos::format_date(date);

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    Ok(date.strftime("%d/%m/%Y").to_string())
}
