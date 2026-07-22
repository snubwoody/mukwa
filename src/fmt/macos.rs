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

use crate::Money;
use core_foundation_sys::base::kCFAllocatorDefault;
use core_foundation_sys::locale::CFLocaleCopyCurrent;
use core_foundation_sys::number::kCFNumberFloat64Type;
use core_foundation_sys::number_formatter::{
    kCFNumberFormatterCurrencyStyle, CFNumberFormatterCreate,
    CFNumberFormatterCreateStringWithValue,
};

pub fn format_money(value: Money) -> crate::Result<String> {
    // TODO: use thread_local!
    // TODO: release using CFRelease?
    // static CURRENT_LOCALE: CFLocaleRef = unsafe { CFLocaleCopyCurrent() };
    unsafe {
        let locale = CFLocaleCopyCurrent();
        let formatter =
            CFNumberFormatterCreate(kCFAllocatorDefault, locale, kCFNumberFormatterCurrencyStyle);

        let str_ptr = CFNumberFormatterCreateStringWithValue(
            kCFAllocatorDefault,
            formatter,
            kCFNumberFloat64Type,
            std::ptr::null(),
        );
    }
    Ok(String::new())
}

#[cfg(test)]
mod test {
    use crate::Money;

    #[test]
    fn format() {
        super::format_money(Money::new(500)).unwrap();
        panic!()
    }
}
