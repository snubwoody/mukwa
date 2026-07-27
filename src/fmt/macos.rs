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
    kCFDateFormatterNoStyle, kCFDateFormatterShortStyle,
    CFDateFormatterCreate, CFDateFormatterCreateDateFromString, CFDateFormatterCreateStringWithDate,
    CFDateFormatterSetFormat,
};
use core_foundation_sys::locale::CFLocaleCopyCurrent;
use jiff::civil::Date;

fn parse_cf_date(date: Date) -> CFDate {
    let date = CFString::new(&date.to_string());

    unsafe {
        let formatter = CFDateFormatterCreate(
            kCFAllocatorDefault,
            // TODO: use static thread_local
            CFLocaleCopyCurrent(),
            kCFDateFormatterNoStyle,
            kCFDateFormatterNoStyle,
        );

        let date_format = CFString::new("yyyy-MM-dd");
        CFDateFormatterSetFormat(formatter, date_format.as_concrete_TypeRef());

        let date = CFDateFormatterCreateDateFromString(
            kCFAllocatorDefault,
            formatter,
            date.as_concrete_TypeRef(),
            std::ptr::null_mut(),
        );
        CFDate::wrap_under_create_rule(date)
    }
}

pub fn format_date(date: Date) -> crate::Result<String> {
    // TODO: check zed repo for formatting
    // TODO: use thread_local!
    // TODO: release using CFRelease?
    // static CURRENT_LOCALE: CFLocaleRef = unsafe { CFLocaleCopyCurrent() };

    let date = parse_cf_date(date);
    let formatted_date = unsafe {
        let locale = CFLocaleCopyCurrent();
        let formatter = CFDateFormatterCreate(
            kCFAllocatorDefault,
            locale,
            kCFDateFormatterShortStyle,
            kCFDateFormatterNoStyle,
        );

        let date_string = CFDateFormatterCreateStringWithDate(
            kCFAllocatorDefault,
            formatter,
            date.as_concrete_TypeRef(),
        );

        CFString::wrap_under_create_rule(date_string).to_string()
    };

    Ok(formatted_date)
}

#[cfg(test)]
mod test {
    use jiff::civil::date;

    #[test]
    fn format() {
        let date = super::format_date(date(2026, 12, 31)).unwrap();
        dbg!(date);
        panic!()
    }
}
