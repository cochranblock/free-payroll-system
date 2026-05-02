// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! End-to-end paycheck computation: gross in, paystub out, no SaaS.

use free_payroll_system::money::Money;
use free_payroll_system::payroll::{Employee, compute};
use free_payroll_system::tax::{FilingStatus, PayFrequency};

fn baltimore_county_single(gross: i64) -> Employee {
    Employee {
        name: "Test Employee".into(),
        state: "MD".into(),
        locality: Some("baltimore".into()),
        filing_status: FilingStatus::Single,
        pay_frequency: PayFrequency::Biweekly,
        gross: Money::dollars(gross),
        ytd_ss_wages: Money::ZERO,
        ytd_medicare_wages: Money::ZERO,
    }
}

#[test]
fn paycheck_balances_to_gross() {
    let pc = compute(&baltimore_county_single(2_000)).unwrap();
    assert_eq!(pc.total_withheld() + pc.net, pc.gross);
}

#[test]
fn paycheck_zero_gross_is_zero() {
    let mut emp = baltimore_county_single(0);
    emp.gross = Money::ZERO;
    let pc = compute(&emp).unwrap();
    assert_eq!(pc.gross, Money::ZERO);
    assert_eq!(pc.total_withheld(), Money::ZERO);
    assert_eq!(pc.net, Money::ZERO);
}

#[test]
fn higher_gross_means_higher_net_but_higher_tax() {
    let low = compute(&baltimore_county_single(1_500)).unwrap();
    let high = compute(&baltimore_county_single(3_000)).unwrap();
    assert!(high.net.0 > low.net.0);
    assert!(high.total_withheld().0 > low.total_withheld().0);
}

#[test]
fn unsupported_state_errors_clearly() {
    let mut emp = baltimore_county_single(2_000);
    emp.state = "CA".into();
    let err = compute(&emp).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("v0.1.0") || msg.contains("MD"),
        "error msg should explain v0.1.0 limit: got '{msg}'"
    );
}

#[test]
fn json_serialization_roundtrips() {
    let pc = compute(&baltimore_county_single(2_000)).unwrap();
    let json = serde_json::to_string(&pc).unwrap();
    let pc2: free_payroll_system::payroll::Paycheck = serde_json::from_str(&json).unwrap();
    assert_eq!(pc, pc2);
}

#[test]
fn employee_toml_roundtrips() {
    let toml_src = r#"
        name = "Roundtrip Test"
        state = "MD"
        locality = "baltimore"
        filing_status = "single"
        pay_frequency = "biweekly"
        gross = 200000
        ytd_ss_wages = 0
        ytd_medicare_wages = 0
    "#;
    let emp: Employee = toml::from_str(toml_src).unwrap();
    let pc = compute(&emp).unwrap();
    assert_eq!(pc.gross, Money::dollars(2_000));
}
