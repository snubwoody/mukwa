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

use core_foundation::base::TCFType;
use core_foundation::date::CFDate;
use core_foundation::string::CFString;
use core_foundation_sys::base::kCFAllocatorDefault;
use core_foundation_sys::date_formatter::{
    CFDateFormatterCreate, CFDateFormatterCreateDateFromString,
    CFDateFormatterCreateStringWithDate, CFDateFormatterRef, CFDateFormatterSetFormat,
    kCFDateFormatterNoStyle, kCFDateFormatterShortStyle,
};
use core_foundation_sys::locale::{CFLocaleCopyCurrent, CFLocaleRef};
use jiff::civil::Date;

// CoreFoundation types are not thread safe.
thread_local! {
    static CURRENT_LOCALE: CFLocaleRef = unsafe { CFLocaleCopyCurrent() };

     static DATE_FORMATTER: CFDateFormatterRef = unsafe {
        CFDateFormatterCreate(
            kCFAllocatorDefault,
            CURRENT_LOCALE.with(|locale| *locale),
            kCFDateFormatterShortStyle,
            kCFDateFormatterNoStyle,
        )
    };

    static PARSE_DATE_FORMATTER: CFDateFormatterRef = unsafe {
        let formatter = CFDateFormatterCreate(
            kCFAllocatorDefault,
            CURRENT_LOCALE.with(|locale| *locale),
            kCFDateFormatterNoStyle,
            kCFDateFormatterNoStyle,
        );

        let date_format = CFString::new("yyyy-MM-dd");
        CFDateFormatterSetFormat(formatter, date_format.as_concrete_TypeRef());
        formatter
    };
}

fn parse_cf_date(date: Date) -> CFDate {
    let date = CFString::new(&date.to_string());

    unsafe {
        let date = CFDateFormatterCreateDateFromString(
            kCFAllocatorDefault,
            PARSE_DATE_FORMATTER.with(|formatter| *formatter),
            date.as_concrete_TypeRef(),
            std::ptr::null_mut(),
        );
        CFDate::wrap_under_create_rule(date)
    }
}

pub fn format_date(date: Date) -> crate::Result<String> {
    let date = parse_cf_date(date);
    let formatted_date = unsafe {
        let date_string = CFDateFormatterCreateStringWithDate(
            kCFAllocatorDefault,
            DATE_FORMATTER.with(|formatter| *formatter),
            date.as_concrete_TypeRef(),
        );

        CFString::wrap_under_create_rule(date_string).to_string()
    };

    Ok(formatted_date)
}
