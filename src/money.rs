// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Money: signed integer cents. No floats. No `f64`-induced wrong paychecks.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Money(pub i64);

impl Money {
    pub const ZERO: Money = Money(0);

    pub const fn cents(c: i64) -> Money {
        Money(c)
    }

    pub const fn dollars(d: i64) -> Money {
        Money(d * 100)
    }

    pub fn from_dollars_str(s: &str) -> Option<Money> {
        let s = s.trim().trim_start_matches('$');
        let neg = s.starts_with('-');
        let s = s.trim_start_matches('-');
        let parts: Vec<&str> = s.split('.').collect();
        let cents: i64 = match parts.as_slice() {
            [whole] => whole.parse::<i64>().ok()? * 100,
            [whole, frac] => {
                let frac = match frac.len() {
                    0 => 0,
                    1 => frac.parse::<i64>().ok()? * 10,
                    2 => frac.parse::<i64>().ok()?,
                    _ => return None,
                };
                whole.parse::<i64>().ok()? * 100 + frac
            }
            _ => return None,
        };
        Some(Money(if neg { -cents } else { cents }))
    }

    /// Multiply by basis points (1 bp = 0.01%). Banker's rounding to nearest cent.
    pub fn mul_bps(self, bps: u32) -> Money {
        let prod = self.0 as i128 * bps as i128;
        let cents = prod / 10_000;
        let rem = prod % 10_000;
        let rounded = if rem.abs() * 2 > 10_000 {
            cents + if prod >= 0 { 1 } else { -1 }
        } else if rem.abs() * 2 == 10_000 {
            // banker's: round to even
            if cents % 2 == 0 {
                cents
            } else {
                cents + if prod >= 0 { 1 } else { -1 }
            }
        } else {
            cents
        };
        Money(rounded as i64)
    }

    pub fn max(self, other: Money) -> Money {
        Money(self.0.max(other.0))
    }
    pub fn min(self, other: Money) -> Money {
        Money(self.0.min(other.0))
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.0 < 0;
        let abs = self.0.unsigned_abs();
        let dollars = abs / 100;
        let cents = abs % 100;
        write!(f, "{}${}.{:02}", if neg { "-" } else { "" }, dollars, cents)
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, o: Money) -> Money {
        Money(self.0 + o.0)
    }
}
impl Sub for Money {
    type Output = Money;
    fn sub(self, o: Money) -> Money {
        Money(self.0 - o.0)
    }
}
impl AddAssign for Money {
    fn add_assign(&mut self, o: Money) {
        self.0 += o.0
    }
}
impl SubAssign for Money {
    fn sub_assign(&mut self, o: Money) {
        self.0 -= o.0
    }
}
impl Mul<i64> for Money {
    type Output = Money;
    fn mul(self, n: i64) -> Money {
        Money(self.0 * n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dollars_to_cents() {
        assert_eq!(Money::dollars(10).0, 1000);
    }
    #[test]
    fn parse_with_dollar_sign() {
        assert_eq!(
            Money::from_dollars_str("$1,234.56".replace(",", "").as_str())
                .unwrap()
                .0,
            123456
        );
    }
    #[test]
    fn parse_negative() {
        assert_eq!(Money::from_dollars_str("-12.34").unwrap().0, -1234);
    }
    #[test]
    fn display_zero() {
        assert_eq!(Money::ZERO.to_string(), "$0.00");
    }
    #[test]
    fn display_pos() {
        assert_eq!(Money::cents(123456).to_string(), "$1234.56");
    }
    #[test]
    fn display_neg() {
        assert_eq!(Money::cents(-50).to_string(), "-$0.50");
    }
    #[test]
    fn mul_bps_625() {
        // 6.20% of $1000 = $62.00 (FICA SS)
        assert_eq!(Money::dollars(1000).mul_bps(620), Money::dollars(62));
    }
    #[test]
    fn mul_bps_145() {
        // 1.45% of $1000 = $14.50 (Medicare)
        assert_eq!(Money::dollars(1000).mul_bps(145), Money::cents(1450));
    }
}
