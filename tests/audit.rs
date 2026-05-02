// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Audit fixture tests: paystub inputs and expected outputs sourced from
//! external authorities (IRS Pub 15-T worked examples, Comptroller of MD
//! example calculations, IRS Tax Withholding Estimator, etc.).
//!
//! v0.1.0 ships this file with the property tests below and an empty fixture
//! roster. As contributors verify specific paychecks against an external
//! authority, they add a `#[test]` here citing the authority. PRs without a
//! citation comment will be rejected.
//!
//! WHY THIS FILE EXISTS:
//! Every other test in this crate (federal_pub15t.rs, end_to_end.rs, the
//! per-module unit tests) verifies that the bracket-walk algorithm works
//! against the bracket VALUES typed into the source. Those tests cannot
//! catch a typo in the bracket VALUES. This file is the cross-check.
//!
//! HOW TO ADD A FIXTURE:
//!   1. Pick a public-authority worked example (e.g., Pub 15-T 2024 Example 1
//!      on page X, or the Comptroller of MD calculator at marylandtaxes.gov).
//!   2. Run our `paystub` command with the same inputs.
//!   3. If our output matches the authority's: add a #[test] below with a
//!      doc comment citing the page/URL.
//!   4. If our output disagrees: open an issue. Don't paper over it.

use free_payroll_system::money::Money;
use free_payroll_system::payroll::{Employee, compute};
use free_payroll_system::tax::{FilingStatus, PayFrequency};

#[test]
fn audit_self_balances() {
    // Even with no external fixtures, every computed paycheck must balance.
    // Total withheld + net == gross. No money created, no money destroyed.
    let cases = [
        (1_000i64, "single", "biweekly"),
        (2_000, "single", "biweekly"),
        (5_000, "married", "monthly"),
        (10_000, "single", "weekly"),
        (200, "single", "biweekly"), // below all thresholds
    ];
    for &(gross, status_str, freq_str) in &cases {
        let status = match status_str {
            "single" => FilingStatus::Single,
            "married" => FilingStatus::MarriedJointly,
            _ => unreachable!(),
        };
        let freq = match freq_str {
            "weekly" => PayFrequency::Weekly,
            "biweekly" => PayFrequency::Biweekly,
            "monthly" => PayFrequency::Monthly,
            _ => unreachable!(),
        };
        let emp = Employee {
            name: format!("audit-{status_str}-{freq_str}-{gross}"),
            state: "MD".into(),
            locality: Some("baltimore".into()),
            filing_status: status,
            pay_frequency: freq,
            gross: Money::dollars(gross),
            ytd_ss_wages: Money::ZERO,
            ytd_medicare_wages: Money::ZERO,
        };
        let pc = compute(&emp).unwrap();
        assert_eq!(
            pc.total_withheld() + pc.net,
            pc.gross,
            "balance violated for ${gross} {status_str} {freq_str}"
        );
        assert!(
            pc.net.0 >= 0,
            "negative net for ${gross} {status_str} {freq_str}"
        );
        assert!(
            pc.net.0 <= pc.gross.0,
            "net > gross for ${gross} {status_str} {freq_str}"
        );
    }
}

// ─── External-authority fixtures (PRs welcome) ─────────────────────────────
//
// Add #[test] entries below as verified. Required header:
//
//   /// Source: <url or "Pub 15-T 2024 Example N, page X">
//   /// Verified by <handle> on <YYYY-MM-DD>
//
// Example skeleton (DO NOT enable until the value is independently sourced):
//
//   /// Source: IRS Pub 15-T (2024), Worksheet 1A Example 1, page 11
//   /// Verified by GotEmCoach on 2026-05-XX
//   #[test]
//   fn pub15t_2024_w1a_example_1() {
//       let emp = Employee { ... };
//       let pc = compute(&emp).unwrap();
//       assert_eq!(pc.federal_income_tax, Money::cents(<from authority>));
//   }
