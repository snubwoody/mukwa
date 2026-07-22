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
    GetCurrencyFormatEx, GetLocaleInfoEx, CURRENCYFMTW, LOCALE_ICURRDIGITS, LOCALE_ICURRENCY,
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

/// Retrieves information about a locale.
///
/// See the [information] about locale constants.
///
/// [information]: https://learn.microsoft.com/en-us/windows/win32/intl/locale-information-constants
fn get_locale_info(locale: &str, locale_info: u32) -> crate::Result<String> {
    let locale = WString::from(locale);
    let info = unsafe {
        let buffer_length = GetLocaleInfoEx(locale.as_ptr(), locale_info, std::ptr::null_mut(), 0);
        if buffer_length == 0 {
            let err = std::io::Error::last_os_error();
            return Err(Error::from(err));
        }
        let mut buffer = WString::zeroed(buffer_length as usize);
        let buffer_ptr = buffer.as_mut_ptr();
        if GetLocaleInfoEx(locale.as_ptr(), locale_info, buffer_ptr, buffer_length) == 0 {
            let err = std::io::Error::last_os_error();
            return Err(Error::from(err));
        }
        String::try_from(buffer)?
    };
    Ok(info)
}

/// Formats a [`Money`] as a currency string.
pub fn format_money(value: Money, locale: &str) -> crate::Result<String> {
    let locale_str = locale;
    let locale = WString::from(locale);
    let value = WString::from(value.to_string().as_str());

    let num_digits = get_locale_info(locale_str, LOCALE_ICURRDIGITS)?.parse::<u32>()?;
    let leading_zero = get_locale_info(locale_str, LOCALE_ILZERO)?.parse::<u32>()?;
    // let grouping = get_locale_info(locale_str, LOCALE_SGROUPING)?;
    let negative_order = get_locale_info(locale_str, LOCALE_INEGCURR)?.parse::<u32>()?;
    let positive_order = get_locale_info(locale_str, LOCALE_ICURRENCY)?.parse::<u32>()?;
    let mut decimal_seperator: WString = get_locale_info(locale_str, LOCALE_SDECIMAL)?.into();
    let mut thousand_seperator: WString = get_locale_info(locale_str, LOCALE_STHOUSAND)?.into();
    // TODO: if I cache this, make sure the pointer is not dangling
    let mut currency_symbol: WString = get_locale_info(locale_str, LOCALE_SCURRENCY)?.into();
    // TODO: cache this
    let currency_format = CURRENCYFMTW {
        NumDigits: num_digits,
        NegativeOrder: negative_order,
        lpThousandSep: thousand_seperator.as_mut_ptr(),
        lpCurrencySymbol: currency_symbol.as_mut_ptr(),
        LeadingZero: leading_zero,
        PositiveOrder: positive_order,
        Grouping: 3, // FIXME: check this
        lpDecimalSep: decimal_seperator.as_mut_ptr(),
    };

    let currency_format_ptr = std::ptr::from_ref(&currency_format);

    let output = unsafe {
        let value_ptr = value.as_ptr();
        let locale_ptr = locale.as_ptr();
        let buffer_length = GetCurrencyFormatEx(
            locale_ptr,
            0,
            value_ptr,
            currency_format_ptr,
            std::ptr::null_mut(),
            0,
        );

        if buffer_length == 0 {
            let err = std::io::Error::last_os_error();
            return Err(Error::from(err));
        }

        let mut formatted_str = WString::zeroed(buffer_length as usize);
        let formatted_str_ptr = formatted_str.as_mut_ptr();

        let return_code = GetCurrencyFormatEx(
            locale_ptr,
            0,
            value_ptr,
            currency_format_ptr,
            formatted_str_ptr,
            buffer_length,
        );

        if return_code == 0 {
            let err = std::io::Error::last_os_error();
            return Err(Error::from(err));
        }

        formatted_str
    };

    Ok(String::try_from(output)?)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fmt() {
        let output = format_money(Money::new(500), "en-US").unwrap();
        assert_eq!(output, "$500.00");
    }

    #[test]
    fn get_locale_info() {
        assert_eq!(
            super::get_locale_info("en-US", LOCALE_SCURRENCY).unwrap(),
            "$"
        );
        assert_eq!(
            super::get_locale_info("en-ZM", LOCALE_SCURRENCY).unwrap(),
            "K"
        );
        assert_eq!(
            super::get_locale_info("en-CA", LOCALE_SCURRENCY).unwrap(),
            "$"
        );
        assert_eq!(
            super::get_locale_info("en-ZA", LOCALE_SCURRENCY).unwrap(),
            "R"
        );
    }

    #[test]
    fn get_locale_info_invalid_locale() {
        let result = super::get_locale_info("does-not-exist", LOCALE_SCURRENCY);
        assert!(result.is_err());
    }
}
