// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Federal tax: income tax withholding, FICA (Social Security + Medicare), FUTA.
//!
//! TAX YEAR: 2024. Brackets and FICA wage base hand-entered from
//! IRS Publication 15-T (2024) — Worksheet 1A "Percentage Method Tables for
//! Automated Payroll Systems". Annualized Wage Method is used: convert pay-period
//! gross to annual, look up annual tax, divide back.
//!
//! See `data/citations.md` for source-page references for every constant.
//!
//! v0.2.0 will add 2025 + 2026 tables. v0.3.0 will fetch them via `govfetch` from
//! the IRS publication URL on demand.

use crate::money::Money;
use crate::tax::{FilingStatus, PayFrequency};

/// Tax year baked into this build. Display this in CLI banner so users know
/// what they're computing against.
pub const TAX_YEAR: u32 = 2024;

// ─── Federal income tax withholding (Pub 15-T 2024, Worksheet 1A) ───────
// Standard Withholding Schedules (W-4 Step 2 unchecked).
// Annual wage brackets. Format: (over_cents, tentative_tax_cents, rate_bps).

/// Single or Married Filing Separately — Pub 15-T 2024 p.10 Worksheet 1A Std.
const SINGLE_BRACKETS_2024: &[(i64, i64, u32)] = &[
    (0, 0, 0),                      //   $0 –  $6,000   0%
    (600_000, 0, 1000),             //   $6k – $17,600  10%
    (1_760_000, 116_000, 1200),     //  $17.6k – $53,150 12%
    (5_315_000, 542_600, 2200),     //  $53.15k – $106,150 22%
    (10_615_000, 1_708_600, 2400),  //  $106.15k – $200,750 24%
    (20_075_000, 3_979_000, 3200),  //  $200.75k – $251,150 32%
    (25_115_000, 5_591_800, 3500),  //  $251.15k – $615,400 35%
    (61_540_000, 18_340_550, 3700), //  $615.4k+        37%
];

/// Married Filing Jointly — Pub 15-T 2024 p.10 Worksheet 1A Std.
const MFJ_BRACKETS_2024: &[(i64, i64, u32)] = &[
    (0, 0, 0),                      //   $0 – $16,300   0%
    (1_630_000, 0, 1000),           //  $16.3k – $39,500  10%
    (3_950_000, 232_000, 1200),     //  $39.5k – $110,600 12%
    (11_060_000, 1_085_200, 2200),  //  $110.6k – $217,350 22%
    (21_735_000, 3_433_700, 2400),  //  $217.35k – $400,200 24%
    (40_020_000, 7_822_100, 3200),  //  $400.2k – $501,050 32%
    (50_105_000, 11_049_300, 3500), //  $501.05k – $747,500 35%
    (74_750_000, 19_675_050, 3700), //  $747.5k+         37%
];

/// Head of Household — Pub 15-T 2024 p.10 Worksheet 1A Std.
const HOH_BRACKETS_2024: &[(i64, i64, u32)] = &[
    (0, 0, 0),                      //   $0 – $13,300   0%
    (1_330_000, 0, 1000),           //  $13.3k – $30,800  10%
    (3_080_000, 175_000, 1200),     //  $30.8k – $79,050 12%
    (7_905_000, 754_000, 2200),     //  $79.05k – $117,250 22%
    (11_725_000, 1_594_400, 2400),  //  $117.25k – $211,950 24%
    (21_195_000, 3_867_200, 3200),  //  $211.95k – $262,350 32%
    (26_235_000, 5_480_000, 3500),  //  $262.35k – $626,350 35%
    (62_635_000, 18_220_000, 3700), //  $626.35k+        37%
];

/// Apply a bracket table: walk down, find the highest bracket where wage >= over,
/// return tentative + (wage - over) * rate_bps / 10000.
fn apply_brackets(wage: Money, brackets: &[(i64, i64, u32)]) -> Money {
    let cents = wage.0;
    let mut out = Money::ZERO;
    for &(over, tent, bps) in brackets {
        if cents >= over {
            let excess = Money(cents - over);
            out = Money(tent) + excess.mul_bps(bps);
        } else {
            break;
        }
    }
    out
}

fn annual_brackets_2024(status: FilingStatus) -> &'static [(i64, i64, u32)] {
    match status {
        FilingStatus::Single => SINGLE_BRACKETS_2024,
        FilingStatus::MarriedJointly => MFJ_BRACKETS_2024,
        FilingStatus::HeadOfHousehold => HOH_BRACKETS_2024,
    }
}

/// Federal income tax withholding, Annualized Wage Method (Pub 15-T 2024 Worksheet 1A).
/// Returns the per-pay-period withholding amount.
pub fn withhold(gross: Money, status: FilingStatus, freq: PayFrequency) -> Money {
    let n = freq.periods_per_year();
    let annual = gross * n;
    let annual_tax = apply_brackets(annual, annual_brackets_2024(status));
    Money(annual_tax.0 / n)
}

// ─── FICA: Social Security + Medicare (FICA = Federal Insurance Contributions Act) ───
// 26 USC §§ 3101 (employee), 3111 (employer), 3121 (definitions).

/// Social Security wage base for 2024 ($168,600). SSA cost-of-living announcement
/// 2023-10-12. Update annually from ssa.gov/oact/cola/cbb.html.
pub const SS_WAGE_BASE_2024: Money = Money::dollars(168_600);

/// Social Security tax rate, employee share — 6.2% (620 bps).
/// Codified: 26 USC § 3101(a). https://uscode.house.gov/view.xhtml?req=granuleid:USC-prelim-title26-section3101
/// Stable since 1990 (last amendment to the rate floor).
pub const SS_RATE_BPS: u32 = 620;

/// Medicare tax rate, employee share — 1.45% (145 bps).
/// Codified: 26 USC § 3101(b)(1). Stable since 1986.
pub const MEDICARE_RATE_BPS: u32 = 145;

/// Additional Medicare tax — 0.9% (90 bps) on wages above $200,000 (employer
/// withholding threshold, regardless of filing status — employee-side reconciliation
/// at filing time accounts for joint thresholds).
/// Codified: 26 USC § 3101(b)(2). Set by ACA (Pub. L. 111-148, § 9015). Stable since 2013.
pub const ADDL_MEDICARE_RATE_BPS: u32 = 90;
pub const ADDL_MEDICARE_THRESHOLD: Money = Money::dollars(200_000);

/// Compute employee Social Security withholding for this paycheck.
/// `ytd_ss_wages` = year-to-date Social Security wages BEFORE this paycheck.
pub fn social_security(gross: Money, ytd_ss_wages: Money) -> Money {
    let cap = SS_WAGE_BASE_2024;
    let remaining = (cap - ytd_ss_wages).max(Money::ZERO);
    let taxable = gross.min(remaining);
    taxable.mul_bps(SS_RATE_BPS)
}

/// Compute employee Medicare withholding (the regular 1.45% share, no cap).
pub fn medicare(gross: Money) -> Money {
    gross.mul_bps(MEDICARE_RATE_BPS)
}

/// Additional Medicare tax (0.9%) on wages above $200k YTD.
pub fn additional_medicare(gross: Money, ytd_medicare_wages: Money) -> Money {
    let after = ytd_medicare_wages + gross;
    let above = (after - ADDL_MEDICARE_THRESHOLD)
        .max(Money::ZERO)
        .min(gross);
    above.mul_bps(ADDL_MEDICARE_RATE_BPS)
}

// ─── FUTA (Federal Unemployment Tax Act) — employer side, NOT on paystub ───
// 26 USC § 3301. Included for completeness / employer reports. Withholding for
// paychecks is handled above.

/// FUTA rate — 6.0% on first $7,000 of wages (26 USC § 3301).
/// Most employers get a credit reducing this to 0.6% effective. Employer side only.
pub const FUTA_RATE_BPS: u32 = 600;
pub const FUTA_WAGE_BASE: Money = Money::dollars(7_000);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ss_no_ytd_under_cap() {
        // $1,000 paycheck * 6.2% = $62.00
        let ss = social_security(Money::dollars(1_000), Money::ZERO);
        assert_eq!(ss, Money::dollars(62));
    }

    #[test]
    fn ss_caps_at_wage_base() {
        // YTD already $168,000; paycheck $2,000; only $600 of it is taxable.
        let ss = social_security(Money::dollars(2_000), Money::dollars(168_000));
        assert_eq!(ss, Money::dollars(600).mul_bps(620));
    }

    #[test]
    fn ss_zero_above_cap() {
        let ss = social_security(Money::dollars(5_000), Money::dollars(170_000));
        assert_eq!(ss, Money::ZERO);
    }

    #[test]
    fn medicare_basic() {
        // $1,000 * 1.45% = $14.50
        let m = medicare(Money::dollars(1_000));
        assert_eq!(m, Money::cents(1450));
    }

    #[test]
    fn additional_medicare_below_threshold() {
        let am = additional_medicare(Money::dollars(5_000), Money::dollars(100_000));
        assert_eq!(am, Money::ZERO);
    }

    #[test]
    fn additional_medicare_crosses_threshold() {
        // YTD $198k, paycheck $5k: $3k of the $5k is above $200k, taxed at 0.9% = $27.00
        let am = additional_medicare(Money::dollars(5_000), Money::dollars(198_000));
        assert_eq!(am, Money::dollars(3_000).mul_bps(90));
    }

    #[test]
    fn additional_medicare_full_paycheck_above() {
        let am = additional_medicare(Money::dollars(5_000), Money::dollars(250_000));
        assert_eq!(am, Money::dollars(5_000).mul_bps(90));
    }

    // ─── Federal income tax (Annualized Wage Method, Pub 15-T 2024 W1A) ───

    #[test]
    fn withhold_single_zero_below_threshold() {
        // Single, biweekly, $230/pp = $5,980 annual. Below the $6,000 0% bracket end.
        let w = withhold(
            Money::dollars(230),
            FilingStatus::Single,
            PayFrequency::Biweekly,
        );
        assert_eq!(w, Money::ZERO);
    }

    #[test]
    fn withhold_single_in_10pct_band() {
        // Single, biweekly, $500/pp = $13,000 annual.
        // Annual tax = ($13,000 - $6,000) * 10% = $700 -> per pp = $700/26 ≈ $26.92
        let w = withhold(
            Money::dollars(500),
            FilingStatus::Single,
            PayFrequency::Biweekly,
        );
        // $13,000 - $6,000 = $7,000 -> $700 -> /26 = $26.92 (floor int div) actually integer
        // div in cents: 70000 / 26 = 2692 cents = $26.92
        assert_eq!(w, Money::cents(2692));
    }

    #[test]
    fn withhold_single_in_12pct_band() {
        // Single, biweekly, $2,000/pp = $52,000 annual.
        // Falls in $17,600–$53,150 (12%).
        // tax = $1,160 + ($52,000 - $17,600) * 12% = $1,160 + $4,128 = $5,288
        // per pp = $5,288 / 26 = $203.38 (203.3846...)
        let w = withhold(
            Money::dollars(2_000),
            FilingStatus::Single,
            PayFrequency::Biweekly,
        );
        // 528800 cents / 26 = 20338 cents = $203.38
        assert_eq!(w, Money::cents(20338));
    }

    #[test]
    fn withhold_mfj_below_threshold() {
        // MFJ, biweekly, $600/pp = $15,600 annual. Below $16,300 0% bracket end.
        let w = withhold(
            Money::dollars(600),
            FilingStatus::MarriedJointly,
            PayFrequency::Biweekly,
        );
        assert_eq!(w, Money::ZERO);
    }

    #[test]
    fn withhold_mfj_in_12pct_band() {
        // MFJ, biweekly, $3,000/pp = $78,000 annual.
        // Falls in $39,500–$110,600 (12%).
        // tax = $2,320 + ($78,000 - $39,500) * 12% = $2,320 + $4,620 = $6,940
        // per pp = $6,940 / 26 = $266.92
        let w = withhold(
            Money::dollars(3_000),
            FilingStatus::MarriedJointly,
            PayFrequency::Biweekly,
        );
        // 694000 / 26 = 26692 cents
        assert_eq!(w, Money::cents(26692));
    }
}
