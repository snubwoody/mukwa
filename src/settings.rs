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

// enum CurrencyCode {
//     SystemDefault,
//     Custom(String),
// }

// TODO: get config dir, .config/ on Linux
// TODO: try setting system locale - en-CA

use crate::Money;
use fixed_decimal::Decimal;
use icu::locale::Locale;
use icu_experimental::dimension::currency::CurrencyCode;
use icu_experimental::dimension::currency::formatter::CurrencyFormatter;
use tinystr::TinyAsciiStr;

#[derive(Clone, PartialEq, Eq)]
enum UserLocale {
    /// Uses the system's locale
    System,
    Custom(Locale),
}

// TODO: use buffered writer
// TODO: SettingsFile struct?
// TODO: store settings as toml
// TODO: rename to Config
// TODO: store currency code as TinyAsciiStr
struct AppSettings {
    locale: UserLocale,
    currency_code: CurrencyCode,
}

impl AppSettings {
    pub fn new() -> AppSettings {
        let locale: Locale = "en-CA".parse().unwrap();
        let currency_code = CurrencyCode(TinyAsciiStr::try_from_str("CAD").unwrap());

        AppSettings {
            locale: UserLocale::Custom(locale),
            currency_code,
        }
    }

    fn locale(&self) -> Locale {
        match &self.locale {
            UserLocale::Custom(locale) => locale.clone(),
            UserLocale::System => {
                let locale = sys_locale::get_locale().unwrap();
                locale.parse().unwrap()
            }
        }
    }

    pub fn format_money(&self, value: Money) -> String {
        // TODO: lock version
        let locale = self.locale();
        let fmt = CurrencyFormatter::try_new(locale.into(), Default::default()).unwrap();
        let mut dec = Decimal::from(value.inner());
        // dec.multiply_pow10(-6);
        dbg!(dec.to_string());
        // let mut value = Decimal::from(value.inner());
        // let value = value.clone().multiplied_pow10(-(Money::SCALE as i16));
        // let a = Decimal::from(value.inner()).clone().multiplied_pow10(-(Money::SCALE as i16));
        // let value = value.to_string().parse().unwrap();
        fmt.format_fixed_decimal(&dec, &self.currency_code)
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::Money;
    use crate::settings::AppSettings;
    use icu::locale::locale;
    use icu_experimental::dimension::currency::CurrencyCode;
    use icu_experimental::dimension::currency::compact_formatter::CompactCurrencyFormatter;
    use icu_experimental::dimension::currency::formatter::CurrencyFormatter;
    use tinystr::*;

    #[test]
    fn format_money() {
        let settings = AppSettings::new();
        let value = settings.format_money(Money::new(600));
        dbg!(value);
    }

    #[test]
    fn format_currency() {
        let locale = sys_locale::get_locale().unwrap();
        dbg!(locale);
        // TODO: try LongCompactCurrencyFormatter
        // let locale = locale!("en-US").into();
        let currency_code = CurrencyCode(tinystr!(3, "CAD"));
        let compact_fmt =
            CompactCurrencyFormatter::try_new(locale!("en-US").into(), Default::default()).unwrap();
        let fmt = CurrencyFormatter::try_new(locale!("en-CA").into(), Default::default()).unwrap();
        let value = "1234".parse().unwrap();
        let a: String = compact_fmt
            .format_fixed_decimal(&value, &currency_code)
            .to_string();
        let b: String = fmt.format_fixed_decimal(&value, &currency_code).to_string();
        dbg!(a);
        dbg!(b);
    }
}
