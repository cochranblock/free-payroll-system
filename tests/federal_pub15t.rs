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
