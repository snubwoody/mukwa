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

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
    str::FromStr,
};

/// `Money` represents a fixed point monetary value, with 6 digits after the decimal point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Money(i64);

impl Money {
    /// The number of digits after the decimal.
    pub const SCALE: u32 = 6;
    /// The multiplication factor for scaling values.
    const FACTOR: u32 = 10u32.pow(Self::SCALE);

    pub const ZERO: Money = Money::new(0);

    /// The largest value that can be represented by this type.
    pub const MAX: Money = Money::new(i64::MAX);
    pub const MIN: Money = Money::new(i64::MIN);

    /// Creates a new `Money`.
    ///
    /// ## Examples
    /// ```
    /// use mukwa::Money;
    ///
    /// let money = Money::new(200);
    /// assert_eq!(money.inner(),200_000_000);
    /// ```
    pub const fn new(value: i64) -> Self {
        let scaled = value.saturating_mul(Self::FACTOR as i64);
        Self(scaled)
    }

    /// Returns the inner `i64`.
    pub fn inner(&self) -> i64 {
        self.0
    }

    /// Computes the absolute value of `Self`
    ///
    /// ## Example
    /// ```
    /// use mukwa::Money;
    ///
    /// assert_eq!(Money::new(-100).abs(),Money::new(100))
    /// ```
    pub fn abs(&self) -> Self {
        Money(self.0.abs())
    }

    pub const fn from_scaled(value: i64) -> Self {
        Self(value)
    }

    /// Creates a `Money` from an `i64`.
    ///
    /// There is possible loss of precision when parsing floats.
    pub fn from_f64(value: f64) -> Self {
        let scaled = (value * 10f64.powi(Self::SCALE as i32)).round() as i64;
        Self(scaled)
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Self) -> Self::Output {
        Money(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Self) -> Self::Output {
        Money(self.0.saturating_sub(rhs.0))
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let abs = self.abs().0;
        let whole = abs / Money::FACTOR as i64;
        let frac = abs % Money::FACTOR as i64;

        if self.0 < 0 {
            write!(f, "-")?;
        }
        write!(
            f,
            "{}.{:0scale$}",
            whole,
            frac,
            scale = Self::SCALE as usize
        )
    }
}

impl FromStr for Money {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse::<i64>() {
            return Ok(Self::new(value));
        }

        // This is lossy but the risk is acceptable for now
        let value: f64 = s.parse()?;
        Ok(Self::from_f64(value))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn add() {
        let a = Money::new(100);
        let b = Money::new(100);
        let c = a + b;
        assert_eq!(c, Money::new(200));
    }

    #[test]
    fn saturate_add() {
        let a = Money::MAX;
        let b = Money::new(100);
        let c = a + b;
        assert_eq!(c, Money::MAX);
    }

    #[test]
    fn saturate_add_assign() {
        assert_eq!(Money::MAX + Money::new(100), Money::MAX);
    }

    #[test]
    fn saturate_sub_assign() {
        assert_eq!(Money::MIN - Money::new(100), Money::MIN);
    }

    #[test]
    fn saturate_sub() {
        let a = Money::MIN;
        let b = Money::new(100);
        let c = a - b;
        assert_eq!(c, Money::MIN);
    }

    #[test]
    fn subtract() {
        let a = Money::new(100);
        let b = Money::new(50);
        let c = a - b;
        assert_eq!(c, Money::new(50));
    }

    #[test]
    fn saturate_unscaled_overflow() {
        let max_i64 = i64::MAX;
        let money = Money::new(max_i64);
        assert_eq!(money.inner(), i64::MAX);
    }

    #[test]
    fn saturate_f64_overflow() {
        let money = Money::from_f64(f64::MAX);
        assert_eq!(money.inner(), i64::MAX);
    }

    #[test]
    fn non_finite() {
        assert_eq!(Money::from_f64(f64::NAN), Money::ZERO,);
        assert_eq!(Money::from_f64(f64::INFINITY), Money::MAX,);
        assert_eq!(Money::from_f64(f64::NEG_INFINITY), Money::MIN,);
    }

    #[test]
    fn new() {
        let money = Money::new(20);
        assert_eq!(money.0, 20_000000);
    }

    #[test]
    fn from_scaled() {
        let money = Money::from_scaled(20);
        assert_eq!(money.0, 20);
    }

    #[test]
    fn from_f64() {
        let money = Money::from_f64(999.999_999);
        assert_eq!(money.0, 999_999_999);
    }

    #[test]
    fn from_str_int() -> crate::Result<()> {
        let money = Money::from_str("150")?;
        assert_eq!(money.0, 150_000_000);
        Ok(())
    }

    #[test]
    fn from_str_float() -> crate::Result<()> {
        let money = Money::from_str("150.24706")?;
        assert_eq!(money.0, 150_247_060);
        Ok(())
    }

    #[test]
    fn clip_float_string() -> crate::Result<()> {
        let money = Money::from_str("150.2470650935093059305930593095")?;
        assert_eq!(money.0, 150_247_065);
        Ok(())
    }

    #[test]
    fn to_string() {
        let money = Money::new(20);
        assert_eq!(money.to_string(), "20.000000");
    }
}
