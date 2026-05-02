// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Money: signed integer cents. No floats. No `f64`-induced wrong paychecks.

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Money(pub i64);

// Custom Deserialize: accept EITHER an integer (cents) OR a string ("$2,000.00", "2000.00", "-$1.50").
// Eliminates the cents-footgun in employee TOML configs.
impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Money, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MoneyVisitor;

        impl Visitor<'_> for MoneyVisitor {
            type Value = Money;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("integer cents (e.g. 200000) or a dollar string (e.g. \"$2,000.00\")")
            }

            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Money, E> {
                Ok(Money(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Money, E> {
                Ok(Money(v as i64))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Money, E> {
                // Strip thousands commas and any leading whitespace before parsing.
                let cleaned: String = v.chars().filter(|c| *c != ',' && *c != ' ').collect();
                Money::from_dollars_str(&cleaned).ok_or_else(|| {
                    de::Error::custom(format!(
                        "invalid money string: {v:?} (expected like \"$2,000.00\", \"2000.00\", or \"-$1.50\")"
                    ))
                })
            }
        }

        deserializer.deserialize_any(MoneyVisitor)
    }
}

impl Money {
    pub const ZERO: Money = Money(0);

    pub const fn cents(c: i64) -> Money {
        Money(c)
    }

    pub const fn dollars(d: i64) -> Money {
        Money(d * 100)
    }

    pub fn from_dollars_str(s: &str) -> Option<Money> {
        // Accept "-$50.00", "$-50.00", "-50.00", "$50.00", "1,234.56", etc.
        let cleaned: String = s.chars().filter(|c| *c != '$' && *c != ',').collect();
        let s = cleaned.trim();
        let neg = s.starts_with('-');
        let s = s.trim_start_matches('-').trim_start_matches('+');
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
        // Thousands separator. $2,000.00 not $2000.00.
        let dollars_str = {
            let s = dollars.to_string();
            let mut out = String::with_capacity(s.len() + s.len() / 3);
            for (i, c) in s.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    out.push(',');
                }
                out.push(c);
            }
            out.chars().rev().collect::<String>()
        };
        write!(
            f,
            "{}${}.{:02}",
            if neg { "-" } else { "" },
            dollars_str,
            cents
        )
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
        assert_eq!(Money::cents(123456).to_string(), "$1,234.56");
    }
    #[test]
    fn display_million() {
        // SaaS payroll vendor at scale: $1.5M annual gross. Read it.
        assert_eq!(Money::cents(150_000_000).to_string(), "$1,500,000.00");
    }
    #[test]
    fn display_under_thousand_no_separator() {
        assert_eq!(Money::cents(99_99).to_string(), "$99.99");
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

    // ── Custom serde Deserialize: accept BOTH int (cents) and string (dollars) ──

    #[test]
    fn deserialize_from_integer_cents() {
        let m: Money = serde_json::from_str("200000").unwrap();
        assert_eq!(m, Money::dollars(2_000));
    }

    #[test]
    fn deserialize_from_dollar_string_with_sign_and_commas() {
        let m: Money = serde_json::from_str(r#""$2,000.00""#).unwrap();
        assert_eq!(m, Money::dollars(2_000));
    }

    #[test]
    fn deserialize_from_dollar_string_no_sign() {
        let m: Money = serde_json::from_str(r#""1234.56""#).unwrap();
        assert_eq!(m, Money::cents(123_456));
    }

    #[test]
    fn deserialize_from_negative_string() {
        let m: Money = serde_json::from_str(r#""-$50.00""#).unwrap();
        assert_eq!(m, Money::dollars(-50));
    }

    #[test]
    fn deserialize_invalid_string_errors_clearly() {
        let r: Result<Money, _> = serde_json::from_str(r#""not money""#);
        assert!(r.is_err());
        assert!(format!("{}", r.unwrap_err()).contains("invalid money string"));
    }

    #[test]
    fn deserialize_via_toml_integer() {
        let s = "amount = 200000";
        #[derive(serde::Deserialize)]
        struct T {
            amount: Money,
        }
        let t: T = toml::from_str(s).unwrap();
        assert_eq!(t.amount, Money::dollars(2_000));
    }

    #[test]
    fn deserialize_via_toml_string() {
        let s = r#"amount = "$2,000.00""#;
        #[derive(serde::Deserialize)]
        struct T {
            amount: Money,
        }
        let t: T = toml::from_str(s).unwrap();
        assert_eq!(t.amount, Money::dollars(2_000));
    }
}
