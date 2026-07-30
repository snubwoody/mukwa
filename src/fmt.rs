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

use crate::Money;
use jiff::civil::Date;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone, PartialEq, Debug)]
pub struct CurrencyFormatter {
    /// The currency symbol
    symbol: String,
}

impl CurrencyFormatter {
    /// Creates a new currency formatter.
    pub fn new() -> crate::Result<Self> {
        let formatter = CurrencyFormatter {
            symbol: String::from("$"),
        };

        Ok(formatter)
    }

    /// Set the currency symbol
    pub fn set_symbol(&mut self, symbol: &str) {
        self.symbol = symbol.to_owned();
    }

    /// Returns the currency symbol.
    pub fn symbol(&self) -> &str {
        self.symbol.as_str()
    }

    /// Formats [`Money`] as a currency string.
    pub fn format_currency(&self, value: Money) -> crate::Result<String> {
        #[cfg(target_os = "windows")]
        {
            use std::sync::OnceLock;
            use windows::CurrencyFormatOptions;

            static CURRENCY_FORMAT_OPTS: OnceLock<CurrencyFormatOptions> = OnceLock::new();
            let currency_symbol = self.symbol.clone();
            // FIXME: invalidate when editing
            // TODO: maybe just a function with options?
            let opts = CURRENCY_FORMAT_OPTS.get_or_init(move || {
                let mut opts = CurrencyFormatOptions::load_from_sys().unwrap();
                opts.currency_symbol = currency_symbol;
                opts
            });
            windows::format_money(value, opts)
        }

        #[cfg(not(target_os = "windows"))]
        Ok(format!("{}{}", self.symbol, value))
    }

    pub fn format_currency_no_cache(&self, value: Money) -> crate::Result<String> {
        #[cfg(target_os = "windows")]
        {
            use std::sync::OnceLock;
            use windows::CurrencyFormatOptions;

            let currency_symbol = self.symbol.clone();
            let mut opts = CurrencyFormatOptions::load_from_sys().unwrap();
            opts.currency_symbol = currency_symbol;
            windows::format_money(value, &opts)
        }

        #[cfg(not(target_os = "windows"))]
        Ok(format!("{}{}", self.symbol, value))
    }
}

pub fn format_date(date: Date) -> crate::Result<String> {
    #[cfg(target_os = "windows")]
    return windows::format_date(date);

    #[cfg(not(target_os = "windows"))]
    Ok(date.strftime("%d/%m/%Y").to_string())
}
