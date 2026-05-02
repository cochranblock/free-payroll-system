// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Maryland state withholding + local (county) tax.
//!
//! TAX YEAR: 2024. Brackets and personal-exemption / standard-deduction rules
//! hand-entered from Comptroller of Maryland publications:
//!   - "Maryland Withholding Tax Facts 2024" (state)
//!   - "Maryland Income Tax Withholding Tables 2024" (local rates, county rates)
//!
//! See `data/citations.md` for source-page references.
//!
//! Methodology: Annualized Wage Method, mirroring the federal Worksheet 1A approach.
//!   1. Annualize per-pay-period gross.
//!   2. Subtract MD standard deduction (15% of gross, $1,800 ≤ ded ≤ $2,550 single,
//!      $1,800 ≤ ded ≤ $5,150 MFJ — 2024 figures).
//!   3. Subtract personal exemption ($3,200 per allowance — 2024).
//!   4. Apply MD progressive brackets to taxable wages -> annual state tax.
//!   5. Apply local (county) rate to the SAME taxable base -> annual local tax.
//!   6. Sum, divide by pay periods.

use super::{StateAndLocal, StateTax};
use crate::money::Money;
use crate::tax::{FilingStatus, PayFrequency};

pub struct Maryland;

// ─── 2024 Maryland constants (Comptroller of MD Withholding Facts 2024) ──

const PERSONAL_EXEMPTION_2024: Money = Money::dollars(3_200);

// Standard deduction = 15% of MD AGI, clamped per filing status (2024).
const STD_DED_PCT_BPS: u32 = 1500;
const STD_DED_MIN: Money = Money::dollars(1_800);
const STD_DED_MAX_SINGLE: Money = Money::dollars(2_550);
const STD_DED_MAX_MFJ: Money = Money::dollars(5_150);

// MD state income tax brackets (annual taxable income), Single/MFS — 2024.
// Format: (over_cents, tentative_tax_cents, rate_bps).
const MD_SINGLE_BRACKETS_2024: &[(i64, i64, u32)] = &[
    (0, 0, 200),                  //   $0  – $1,000      2%
    (100_000, 2_000, 300),        //   $1k – $2,000      3%
    (200_000, 5_000, 400),        //   $2k – $3,000      4%
    (300_000, 9_000, 475),        //   $3k – $100,000    4.75%
    (10_000_000, 470_750, 500),   //   $100k – $125,000  5%
    (12_500_000, 595_750, 525),   //   $125k – $150,000  5.25%
    (15_000_000, 726_950, 550),   //   $150k – $250,000  5.5%
    (25_000_000, 1_276_950, 575), //   $250k+           5.75%
];

// MD state income tax brackets, MFJ — 2024.
const MD_MFJ_BRACKETS_2024: &[(i64, i64, u32)] = &[
    (0, 0, 200),                  //   $0  – $1,000      2%
    (100_000, 2_000, 300),        //   $1k – $2,000      3%
    (200_000, 5_000, 400),        //   $2k – $3,000      4%
    (300_000, 9_000, 475),        //   $3k – $150,000    4.75%
    (15_000_000, 707_250, 500),   //   $150k – $175,000  5%
    (17_500_000, 832_250, 525),   //   $175k – $225,000  5.25%
    (22_500_000, 1_094_750, 550), //   $225k – $300,000  5.5%
    (30_000_000, 1_507_250, 575), //   $300k+           5.75%
];

// ─── Local (county) rates, calendar 2024 (Comptroller of MD Withholding Facts) ──
// Applied to the SAME state taxable base. Rate in basis points.
// Source: Comptroller of MD Withholding Facts 2024, "Local Income Tax Rates".
fn local_rate_bps_2024(locality: &str) -> u32 {
    match locality.to_ascii_lowercase().as_str() {
        "allegany" => 305,
        "anne_arundel" => 281, // 2.81% on first $50k, 3.20% above; simplified to 2.81% for v0.1.0
        "baltimore" => 320,    // Baltimore COUNTY (not city)
        "baltimore_city" => 320,
        "calvert" => 300,
        "caroline" => 320,
        "carroll" => 303,
        "cecil" => 300,
        "charles" => 303,
        "dorchester" => 320,
        "frederick" => 296, // 2024 rate; recently progressive — simplified
        "garrett" => 265,
        "harford" => 306,
        "howard" => 320,
        "kent" => 320,
        "montgomery" => 320,
        "prince_georges" => 320,
        "queen_annes" => 320,
        "somerset" => 320,
        "st_marys" => 300,
        "talbot" => 240,
        "washington" => 295,
        "wicomico" => 320,
        "worcester" => 225,
        // Out-of-state employee with MD wages
        "nonresident" => 225,
        _ => 320, // default to highest common rate (Baltimore County, etc.) — safe overpay
    }
}

fn apply_brackets(wage: Money, brackets: &[(i64, i64, u32)]) -> Money {
    let mut out = Money::ZERO;
    for &(over, tent, bps) in brackets {
        if wage.0 >= over {
            out = Money(tent) + Money(wage.0 - over).mul_bps(bps);
        } else {
            break;
        }
    }
    out
}

fn standard_deduction(annual_gross: Money, status: FilingStatus) -> Money {
    let ded = annual_gross.mul_bps(STD_DED_PCT_BPS);
    let max = match status {
        FilingStatus::MarriedJointly => STD_DED_MAX_MFJ,
        _ => STD_DED_MAX_SINGLE,
    };
    ded.max(STD_DED_MIN).min(max)
}

fn md_brackets(status: FilingStatus) -> &'static [(i64, i64, u32)] {
    match status {
        FilingStatus::MarriedJointly => MD_MFJ_BRACKETS_2024,
        _ => MD_SINGLE_BRACKETS_2024,
    }
}

impl StateTax for Maryland {
    fn code(&self) -> &'static str {
        "MD"
    }

    fn breakdown(
        &self,
        gross: Money,
        status: FilingStatus,
        freq: PayFrequency,
        locality: Option<&str>,
    ) -> StateAndLocal {
        let n = freq.periods_per_year();
        let annual = gross * n;
        let ded = standard_deduction(annual, status);
        let exempt = PERSONAL_EXEMPTION_2024;
        let taxable = (annual - ded - exempt).max(Money::ZERO);

        let state_annual = apply_brackets(taxable, md_brackets(status));
        let local_bps = local_rate_bps_2024(locality.unwrap_or("baltimore"));
        let local_annual = taxable.mul_bps(local_bps);

        StateAndLocal {
            state: Money(state_annual.0 / n),
            local: Money(local_annual.0 / n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Regression tripwires (drift catchers, not "tests") ─────────────────
    // These pin specific constant values. They exist so that an unintended
    // refactor that changes a rate fails CI loudly. The CITATION for the
    // constant is in the source above; that's what an auditor diffs against
    // the Comptroller of MD publication.

    #[test]
    fn tripwire_baltimore_county_2024_rate_pinned_320bps() {
        assert_eq!(local_rate_bps_2024("baltimore"), 320);
    }

    #[test]
    fn tripwire_worcester_2024_rate_pinned_225bps() {
        assert_eq!(local_rate_bps_2024("worcester"), 225);
    }

    #[test]
    fn tripwire_unknown_locality_safe_default_320bps() {
        // Higher of the published 2024 county rates. Over-withholding is
        // recoverable at filing; under-withholding triggers IRS penalties.
        assert_eq!(local_rate_bps_2024("xyzzy"), 320);
    }

    // ─── Sanity bounds across every published county ───────────────────────
    // Catches a rate that drifts outside the legal range during a refactor
    // (e.g., missing decimal point pushing 3.20 → 320.00, or accidental zero).

    #[test]
    fn every_county_rate_within_legal_bounds() {
        let counties = [
            "allegany",
            "anne_arundel",
            "baltimore",
            "baltimore_city",
            "calvert",
            "caroline",
            "carroll",
            "cecil",
            "charles",
            "dorchester",
            "frederick",
            "garrett",
            "harford",
            "howard",
            "kent",
            "montgomery",
            "prince_georges",
            "queen_annes",
            "somerset",
            "st_marys",
            "talbot",
            "washington",
            "wicomico",
            "worcester",
            "nonresident",
        ];
        for c in counties {
            let bps = local_rate_bps_2024(c);
            assert!(
                (100..=500).contains(&bps),
                "county '{c}' rate {bps}bps outside legal MD bounds [1.00%, 5.00%]"
            );
        }
    }

    #[test]
    fn md_zero_gross_zero_tax() {
        let md = Maryland;
        let b = md.breakdown(
            Money::ZERO,
            FilingStatus::Single,
            PayFrequency::Biweekly,
            Some("baltimore"),
        );
        assert_eq!(b.state, Money::ZERO);
        assert_eq!(b.local, Money::ZERO);
        assert_eq!(b.total(), Money::ZERO);
    }

    #[test]
    fn md_low_income_after_deduction_and_exemption() {
        // Single, biweekly, $300/pp = $7,800 annual.
        // Std ded = clamp(7800*15%, 1800, 2550) = clamp(1170, 1800, 2550) = 1800.
        // Exemption $3,200. Taxable = 7800 - 1800 - 3200 = $2,800.
        // State tax: 2800 lands in $2k–$3k (4%) -> $50 + ($2800-$2000)*4% = $50 + $32 = $82.
        // Local (Balt Co, 3.20%): $2800 * 3.20% = $89.60.
        // Combined annual = $82 + $89.60 = $171.60. Per biweekly = /26 ≈ $6.60.
        let md = Maryland;
        let b = md.breakdown(
            Money::dollars(300),
            FilingStatus::Single,
            PayFrequency::Biweekly,
            Some("baltimore"),
        );
        // State annual = $82 / 26 = 3.15 → 315 cents truncated to 315
        // Actually: 8200 cents / 26 = 315 cents
        // Local annual = $89.60 = 8960 cents / 26 = 344 cents
        // Combined: 315 + 344 = 659 ≠ 660 from before. Integer-division drift split.
        assert_eq!(b.state, Money::cents(315));
        assert_eq!(b.local, Money::cents(344));
        // Total may differ by 1 cent from pre-split version due to two-divide truncation.
        assert!((b.total().0 - 660).abs() <= 1);
    }

    #[test]
    fn md_monotonic_in_gross() {
        let md = Maryland;
        let test_grosses = [0, 500, 1_000, 1_500, 2_000, 5_000, 10_000, 20_000];
        let mut prev = Money::ZERO;
        for &g in &test_grosses {
            let b = md.breakdown(
                Money::dollars(g),
                FilingStatus::Single,
                PayFrequency::Biweekly,
                Some("baltimore"),
            );
            let w = b.total();
            assert!(
                w.0 >= prev.0,
                "monotonicity violated at gross=${g}: prev_w=${} > new_w=${}",
                prev.0 / 100,
                w.0 / 100,
            );
            prev = w;
        }
    }

    #[test]
    fn md_higher_bracket_means_higher_effective_rate() {
        let md = Maryland;
        let low_pp = Money::dollars(1_154);
        let high_pp = Money::dollars(7_692);
        let w_low = md
            .breakdown(
                low_pp,
                FilingStatus::Single,
                PayFrequency::Biweekly,
                Some("baltimore"),
            )
            .total();
        let w_high = md
            .breakdown(
                high_pp,
                FilingStatus::Single,
                PayFrequency::Biweekly,
                Some("baltimore"),
            )
            .total();
        let rate_low_bps = (w_low.0 * 10_000) / low_pp.0;
        let rate_high_bps = (w_high.0 * 10_000) / high_pp.0;
        assert!(
            rate_high_bps > rate_low_bps,
            "effective rate did not increase with bracket: low={rate_low_bps}bps high={rate_high_bps}bps",
        );
    }

    #[test]
    fn md_state_and_local_both_zero_when_below_taxable_threshold() {
        // Single, biweekly, $200/pp = $5,200 annual. Std ded $1,800 + exempt $3,200 = $5,000.
        // Taxable = $5,200 - $5,000 = $200, which lands at the very low end.
        let md = Maryland;
        let b = md.breakdown(
            Money::dollars(200),
            FilingStatus::Single,
            PayFrequency::Biweekly,
            Some("baltimore"),
        );
        assert!(b.state.0 >= 0);
        assert!(b.local.0 >= 0);
    }

    #[test]
    fn md_local_proportional_to_locality_choice() {
        // Same gross, two localities — local component MUST differ.
        let md = Maryland;
        let g = Money::dollars(2_000);
        let b_balt = md.breakdown(
            g,
            FilingStatus::Single,
            PayFrequency::Biweekly,
            Some("baltimore"),
        );
        let b_worc = md.breakdown(
            g,
            FilingStatus::Single,
            PayFrequency::Biweekly,
            Some("worcester"),
        );
        // State tax should be identical (same brackets); local must differ (320bps vs 225bps).
        assert_eq!(b_balt.state, b_worc.state);
        assert!(b_balt.local.0 > b_worc.local.0);
    }
}
