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

use std::string::FromUtf16Error;

use windows_sys::Win32::Globalization::{
    CURRENCYFMTW, GetCurrencyFormatEx, GetLocaleInfoEx, LOCALE_ICURRDIGITS, LOCALE_ICURRENCY,
    LOCALE_ILZERO, LOCALE_INEGCURR, LOCALE_SCURRENCY, LOCALE_SDECIMAL, LOCALE_STHOUSAND,
};

use crate::{Error, Money};

/// A UTF-16 null terminated string.
pub struct WString {
    vec: Vec<u16>,
}

impl WString {
    pub fn with_capacity(capacity: usize) -> Self {
        WString {
            vec: Vec::with_capacity(capacity),
        }
    }

    pub fn zeroed(size: usize) -> Self {
        WString { vec: vec![0; size] }
    }

    pub const fn as_ptr(&self) -> *const u16 {
        self.vec.as_ptr()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut u16 {
        self.vec.as_mut_ptr()
    }
}

impl From<&str> for WString {
    fn from(value: &str) -> Self {
        let mut buf: Vec<u16> = value.encode_utf16().collect();
        buf.push(0);
        Self { vec: buf }
    }
}

impl From<String> for WString {
    fn from(value: String) -> Self {
        let mut buf: Vec<u16> = value.encode_utf16().collect();
        buf.push(0);
        Self { vec: buf }
    }
}

impl TryFrom<WString> for String {
    type Error = FromUtf16Error;

    fn try_from(mut value: WString) -> Result<Self, Self::Error> {
        // Remove the null terminator
        value.vec.pop();
        String::from_utf16(&value.vec)
    }
}

pub struct CurrencyFormatOptions {
    pub num_digits: u32,
    /// Specifier for leading zeros in decimal fields:
    /// - `0` for no leading zeros
    /// - `1` for leading zeros.
    pub leading_zero: u32,
    /// Position of the monetary symbol in the positive currency mode,
    /// corresponds to [LOCALE_ICURRENCY].
    ///
    /// [LOCALE_ICURRENCY]: https://learn.microsoft.com/en-us/windows/win32/intl/locale-icurrency
    pub positive_order: u32,
    /// Negative currency mode, corresponds to [LOCALE_INEGCURR].
    ///
    /// [LOCALE_INEGCURR]:https://learn.microsoft.com/en-us/windows/win32/intl/locale-ineg-constants
    pub negative_order: u32,
    pub decimal_separator: String,
    pub thousand_separator: String,
    pub currency_symbol: String,
    pub grouping: u32,
}

impl CurrencyFormatOptions {
    pub fn load_from_sys(locale: &str) -> crate::Result<Self> {
        let num_digits = get_locale_info(locale, LOCALE_ICURRDIGITS)?.parse::<u32>()?;
        let leading_zero = get_locale_info(locale, LOCALE_ILZERO)?.parse::<u32>()?;
        // FIXME: seems like the wrong type
        // let grouping = get_locale_info(locale, LOCALE_SGROUPING)?;
        let negative_order = get_locale_info(locale, LOCALE_INEGCURR)?.parse::<u32>()?;
        let positive_order = get_locale_info(locale, LOCALE_ICURRENCY)?.parse::<u32>()?;
        let decimal_separator = get_locale_info(locale, LOCALE_SDECIMAL)?;
        let thousand_separator = get_locale_info(locale, LOCALE_STHOUSAND)?;
        let currency_symbol = get_locale_info(locale, LOCALE_SCURRENCY)?;

        let opt = CurrencyFormatOptions {
            negative_order,
            num_digits,
            leading_zero,
            positive_order,
            decimal_separator,
            thousand_separator,
            currency_symbol,
            grouping: 3, // FIXME
        };
        Ok(opt)
    }
}

/// Retrieves information about a locale.
///
/// See the [Microsoft docs] for information about locale constants.
///
/// [Microsoft docs]: https://learn.microsoft.com/en-us/windows/win32/intl/locale-information-constants
fn get_locale_info(locale: &str, locale_info: u32) -> crate::Result<String> {
    let locale = WString::from(locale);
    let buffer_length =
        unsafe { GetLocaleInfoEx(locale.as_ptr(), locale_info, std::ptr::null_mut(), 0) };

    if buffer_length == 0 {
        let err = std::io::Error::last_os_error();
        return Err(Error::from(err));
    }

    let mut buffer = WString::zeroed(buffer_length as usize);
    let buffer_ptr = buffer.as_mut_ptr();

    let return_code =
        unsafe { GetLocaleInfoEx(locale.as_ptr(), locale_info, buffer_ptr, buffer_length) };

    if return_code == 0 {
        let err = std::io::Error::last_os_error();
        return Err(Error::from(err));
    }

    Ok(String::try_from(buffer)?)
}

/// Formats a [`Money`] as a currency string.
pub fn format_money(
    value: Money,
    locale: &str,
    opt: &CurrencyFormatOptions,
) -> crate::Result<String> {
    let locale = WString::from(locale);
    let value = WString::from(value.to_string().as_str());

    let mut currency_symbol = WString::from(opt.currency_symbol.as_str());
    let mut thousand_separator = WString::from(opt.thousand_separator.as_str());
    let mut decimal_separator = WString::from(opt.decimal_separator.as_str());

    let currency_format = CURRENCYFMTW {
        NumDigits: opt.num_digits,
        NegativeOrder: opt.negative_order,
        lpThousandSep: thousand_separator.as_mut_ptr(),
        lpCurrencySymbol: currency_symbol.as_mut_ptr(),
        LeadingZero: opt.leading_zero,
        PositiveOrder: opt.positive_order,
        Grouping: opt.grouping,
        lpDecimalSep: decimal_separator.as_mut_ptr(),
    };

    let buffer_length = unsafe {
        GetCurrencyFormatEx(
            locale.as_ptr(),
            0,
            value.as_ptr(),
            std::ptr::from_ref(&currency_format),
            std::ptr::null_mut(),
            0,
        )
    };

    if buffer_length == 0 {
        let err = std::io::Error::last_os_error();
        return Err(Error::from(err));
    }

    let mut buffer = WString::zeroed(buffer_length as usize);

    let return_code = unsafe {
        GetCurrencyFormatEx(
            locale.as_ptr(),
            0,
            value.as_ptr(),
            std::ptr::from_ref(&currency_format),
            buffer.as_mut_ptr(),
            buffer_length,
        )
    };

    if return_code == 0 {
        let err = std::io::Error::last_os_error();
        return Err(Error::from(err));
    }

    Ok(String::try_from(buffer)?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fmt() {
        let opts = CurrencyFormatOptions::load_from_sys("en-US").unwrap();
        let output = format_money(Money::new(500), "en-US", &opts).unwrap();
        assert_eq!(output, "$500.00");
    }

    #[test]
    fn get_locale_info() {
        let symbol = super::get_locale_info("en-US", LOCALE_SCURRENCY).unwrap();
        assert_eq!(symbol, "$");
    }

    #[test]
    fn get_locale_info_invalid_locale() {
        let result = super::get_locale_info("does-not-exist", LOCALE_SCURRENCY);
        assert!(result.is_err());
    }
}
