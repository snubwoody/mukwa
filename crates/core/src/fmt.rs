// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

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

        // We aim to use the system APIs here so that it follows the user's
        // currency format system settings.
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
            Ok(format!("{}{1:.2$}", symbol, value, precision))
        }
    }

    pub fn format_money_without_symbol(&self, value: Money) -> String {
        let precision = self.currency.precision().unwrap_or(2) as usize;
        format!("{0:.1$}", value, precision)
    }
}

impl Default for CurrencyFormatter {
    fn default() -> Self {
        CurrencyFormatter::new()
    }
}

pub fn format_date(date: Date) -> crate::Result<String> {
    // We aim to use the system APIs here so that it follows the user's
    // date format system settings.

    #[cfg(target_os = "windows")]
    return windows::format_date(date);

    #[cfg(target_os = "macos")]
    return macos::format_date(date);

    #[cfg(all(unix, not(target_os = "macos")))]
    return linux_like::format_date(date);

    #[cfg(not(any(unix, windows)))]
    Ok(date.strftime("%d/%m/%Y").to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
mod linux_like {
    use super::*;
    use std::ffi::CString;
    use std::ops::Sub;

    /// Formats the `Date` using libc's `strftime` function.
    pub fn format_date(date: Date) -> crate::Result<String> {
        let time = libc::tm {
            tm_sec: 0,
            tm_min: 0,
            tm_hour: 0,
            tm_mday: date.day().into(),
            tm_mon: date.month().sub(1).into(),
            tm_year: date.year().sub(1900).into(),
            tm_wday: date.weekday().to_sunday_zero_offset().into(),
            tm_yday: 0,
            tm_isdst: -1,
            tm_gmtoff: 0,
            tm_zone: std::ptr::null(),
        };
        let formatted_string = unsafe {
            let locale = CString::new("").unwrap();
            let locale = libc::newlocale(libc::LC_TIME_MASK, locale.as_ptr(), std::ptr::null_mut());

            if locale.is_null() {
                return Ok(date.strftime("%d/%m/%Y").to_string());
            }

            let format = libc::nl_langinfo_l(libc::D_FMT, locale);

            // TODO: maybe stack allocated string
            let formatted_date = CString::default().into_raw();
            let length = libc::strftime_l(
                formatted_date,
                256,
                format,
                std::ptr::from_ref(&time),
                locale,
            );
            libc::freelocale(locale);

            if length == 0 {
                return Ok(date.strftime("%d/%m/%Y").to_string());
            }
            let output = CString::from_raw(formatted_date);
            output.to_str()?.to_string()
        };
        Ok(formatted_string)
    }

    #[cfg(test)]
    #[test]
    fn test_format_date() {
        use jiff::Zoned;

        let date = Zoned::now().date();
        let result = format_date(date);
        assert!(result.is_ok());
    }

    #[cfg(test)]
    #[test]
    fn format_date_invalid_locale_fallback() {
        use jiff::Zoned;
        unsafe {
            std::env::set_var("LC_TIME", "invalid-locale");
        }
        let date = Zoned::now().date();
        let result = format_date(date);
        assert_eq!(result.unwrap(), date.strftime("%d/%m/%Y").to_string());
    }
}

#[cfg(not(windows))]
#[cfg(test)]
mod test {
    use super::*;
    use crate::{Currency, Money};

    #[test]
    fn format_money_uses_currency_precision() -> crate::Result<()> {
        let test_fmt = |currency: Currency, amount: Money, expected: &str| {
            let mut formatter = CurrencyFormatter::new();
            formatter.set_currency(currency);
            assert_eq!(formatter.format_money(amount).unwrap(), expected);
        };
        test_fmt(Currency::CAD, Money::from_f64(10.5), "$10.50");
        test_fmt(Currency::LYD, Money::from_f64(242.2424), "ل.د242.242");
        test_fmt(Currency::JPY, Money::from_f64(500.2), "¥500");
        Ok(())
    }
}
