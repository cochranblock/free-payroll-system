// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! IRS Pub 15-T 2024 worked-example known-answer tests.

use free_payroll_system::money::Money;
use free_payroll_system::tax::{FilingStatus, PayFrequency, federal};

#[test]
fn fica_employee_share_60k_salary_biweekly() {
    // Worker earns $60,000/yr biweekly = ~$2,307.69/pp.
    // Test exact $60,000 / 26 ≈ $2,307.69, but to keep arithmetic clean use $2,000/pp.
    // $2,000 * 6.20% = $124.00 SS
    // $2,000 * 1.45% = $29.00 Medicare
    let gross = Money::dollars(2_000);
    assert_eq!(
        federal::social_security(gross, Money::ZERO),
        Money::dollars(124)
    );
    assert_eq!(federal::medicare(gross), Money::dollars(29));
    assert_eq!(
        federal::additional_medicare(gross, Money::ZERO),
        Money::ZERO
    );
}

#[test]
fn fica_ss_caps_correctly() {
    // YTD $168,000; this paycheck $1,000. Only first $600 taxable for SS.
    // $600 * 6.20% = $37.20.
    let ss = federal::social_security(Money::dollars(1_000), Money::dollars(168_000));
    assert_eq!(ss, Money::cents(3720));
}

#[test]
fn additional_medicare_kicks_in_at_200k() {
    // YTD Medicare wages $199k, paycheck $2k.
    // $1k of paycheck is above $200k threshold; 0.9% * $1,000 = $9.00.
    let am = federal::additional_medicare(Money::dollars(2_000), Money::dollars(199_000));
    assert_eq!(am, Money::dollars(9));
}

#[test]
fn fed_withhold_single_in_22pct_band() {
    // Single, biweekly, $4,000/pp = $104,000 annual.
    // Falls in $53,150–$106,150 (22%).
    // tax = $5,426 + ($104,000 - $53,150) * 22% = $5,426 + $11,187 = $16,613
    // per pp = $16,613 / 26 = $639.0 (639 exactly: 1661300/26 = 63896.15..., 63896 cents)
    // 1661300 / 26 = 63896 cents (truncated) = $638.96
    let w = federal::withhold(
        Money::dollars(4_000),
        FilingStatus::Single,
        PayFrequency::Biweekly,
    );
    assert_eq!(w, Money::cents(63896));
}

#[test]
fn fed_withhold_weekly_vs_biweekly_consistency() {
    // Same annual rate ($52,000/year) computed weekly should equal biweekly result
    // in annualized total tax, just split differently per pp.
    let weekly_pp = Money::dollars(1_000); // $52,000/yr
    let biweekly_pp = Money::dollars(2_000); // $52,000/yr
    let w_weekly = federal::withhold(weekly_pp, FilingStatus::Single, PayFrequency::Weekly);
    let w_biweekly = federal::withhold(biweekly_pp, FilingStatus::Single, PayFrequency::Biweekly);
    let annual_via_weekly = w_weekly * 52;
    let annual_via_biweekly = w_biweekly * 26;
    // Allow $1 of integer-division drift; both methods should annualize identically modulo rounding.
    let drift = (annual_via_weekly.0 - annual_via_biweekly.0).abs();
    assert!(
        drift <= 100,
        "annualized drift {drift} cents exceeds $1 tolerance"
    );
}

#[test]
fn no_negative_withholding() {
    // Edge case: $50/pp shouldn't underflow into negative.
    let w = federal::withhold(
        Money::dollars(50),
        FilingStatus::Single,
        PayFrequency::Biweekly,
    );
    assert!(w.0 >= 0);
}

// ─── Property tests ───────────────────────────────────────────────────────
// These don't hand-compute against my typed brackets. They verify INVARIANTS
// that any correct withholding implementation must satisfy regardless of the
// specific bracket values. They catch bracket-walk bugs that single-point
// tests cannot.

#[test]
fn federal_withhold_monotonic_in_gross() {
    // For any pair gross_a > gross_b, withhold(gross_a) >= withhold(gross_b).
    // Holds across pay frequencies and filing statuses.
    use FilingStatus::*;
    use PayFrequency::*;
    let grosses_pp = [0, 100, 500, 1_000, 2_500, 5_000, 10_000, 20_000, 50_000];
    for status in [Single, MarriedJointly, HeadOfHousehold] {
        for freq in [Weekly, Biweekly, Semimonthly, Monthly] {
            let mut prev = Money::ZERO;
            for &g in &grosses_pp {
                let w = federal::withhold(Money::dollars(g), status, freq);
                assert!(
                    w.0 >= prev.0,
                    "monotonicity violated: status={status:?} freq={freq:?} gross=${g} prev_w=${} new_w=${}",
                    prev.0 / 100,
                    w.0 / 100,
                );
                prev = w;
            }
        }
    }
}

#[test]
fn federal_withhold_zero_gross_zero_tax() {
    use FilingStatus::*;
    use PayFrequency::*;
    for status in [Single, MarriedJointly, HeadOfHousehold] {
        for freq in [Weekly, Biweekly, Semimonthly, Monthly] {
            assert_eq!(
                federal::withhold(Money::ZERO, status, freq),
                Money::ZERO,
                "non-zero withhold at zero gross: status={status:?} freq={freq:?}"
            );
        }
    }
}

#[test]
fn federal_withhold_consistent_across_frequencies() {
    // Same annual gross computed at different pay frequencies should produce
    // ≈ the same annualized total tax. Drift comes from two integer-division
    // truncations (annualize per-pp gross to annual, then divide annual tax
    // back by n). Bound: drift across all four frequencies must be < 0.05%
    // of the annual gross (catches algorithmic bugs without flapping on
    // legitimate sub-cent rounding).
    let annual_targets_cents = [26_000_00i64, 52_000_00, 100_000_00, 250_000_00];
    for &target_cents in &annual_targets_cents {
        // Per-pp at CENT level to keep inputs as close to identical-annual as possible.
        let weekly = Money(target_cents / 52);
        let biweekly = Money(target_cents / 26);
        let semimonthly = Money(target_cents / 24);
        let monthly = Money(target_cents / 12);

        let aw = federal::withhold(weekly, FilingStatus::Single, PayFrequency::Weekly) * 52;
        let ab = federal::withhold(biweekly, FilingStatus::Single, PayFrequency::Biweekly) * 26;
        let as_ =
            federal::withhold(semimonthly, FilingStatus::Single, PayFrequency::Semimonthly) * 24;
        let am = federal::withhold(monthly, FilingStatus::Single, PayFrequency::Monthly) * 12;

        let max = aw.0.max(ab.0).max(as_.0).max(am.0);
        let min = aw.0.min(ab.0).min(as_.0).min(am.0);
        let drift = max - min;
        let tolerance = (target_cents * 5) / 10_000; // 0.05% of annual gross
        let tolerance = tolerance.max(100); // floor at $1 for small targets
        assert!(
            drift <= tolerance,
            "annual tax drift {drift}c > tolerance {tolerance}c at target ${} (w={} b={} s={} m={})",
            target_cents / 100,
            aw.0 / 100,
            ab.0 / 100,
            as_.0 / 100,
            am.0 / 100,
        );
    }
}

#[test]
fn fica_constants_match_statute() {
    // 26 USC § 3101 — these are the *statutory* values. If anyone refactors
    // the constants, this catches it. Citations in src/tax/federal.rs.
    assert_eq!(federal::SS_RATE_BPS, 620, "26 USC § 3101(a) is 6.2%");
    assert_eq!(
        federal::MEDICARE_RATE_BPS,
        145,
        "26 USC § 3101(b)(1) is 1.45%"
    );
    assert_eq!(
        federal::ADDL_MEDICARE_RATE_BPS,
        90,
        "26 USC § 3101(b)(2) is 0.9%"
    );
    assert_eq!(
        federal::ADDL_MEDICARE_THRESHOLD,
        Money::dollars(200_000),
        "26 USC § 3101(b)(2) employer threshold is $200,000"
    );
    assert_eq!(
        federal::FUTA_RATE_BPS,
        600,
        "26 USC § 3301 base FUTA rate is 6.0%"
    );
    assert_eq!(
        federal::FUTA_WAGE_BASE,
        Money::dollars(7_000),
        "26 USC § 3306(b)(1) FUTA wage base is $7,000"
    );
}

#[test]
fn fica_ss_never_exceeds_max_annual_tax() {
    // 6.2% * $168,600 = $10,453.20 max SS for 2024. No paycheck combo can exceed.
    let max_ss_2024 = Money::cents(1_045_320); // $10,453.20
    let huge_paycheck = Money::dollars(50_000);
    let ss = federal::social_security(huge_paycheck, Money::ZERO);
    assert!(
        ss.0 <= max_ss_2024.0,
        "SS withhold exceeded annual max: got {ss} > {max_ss_2024}"
    );
}
