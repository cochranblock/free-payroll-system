// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Payroll: `Employee` config -> `Paycheck` breakdown. Pure function, no I/O.
//!
//! `compute(&Employee) -> Result<Paycheck>` is the main entry point. The
//! returned `Paycheck` is a complete itemization: federal income tax, FICA
//! (Social Security, Medicare, Additional Medicare), state income tax,
//! local income tax, and net pay. Plus self-describing metadata (`tax_year`,
//! `currency`, `unit`, `version`) so JSON consumers don't have to guess
//! whether `200000` is cents or dollars or what tax year it was computed
//! against.

use crate::money::Money;
use crate::tax::federal::TAX_YEAR;
use crate::tax::{FilingStatus, PayFrequency, federal, state::State};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Employee {
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub locality: Option<String>,
    pub filing_status: FilingStatus,
    pub pay_frequency: PayFrequency,
    /// Gross wages this pay period.
    pub gross: Money,
    /// Year-to-date Social Security wages BEFORE this paycheck.
    #[serde(default)]
    pub ytd_ss_wages: Money,
    /// Year-to-date Medicare wages BEFORE this paycheck.
    #[serde(default)]
    pub ytd_medicare_wages: Money,
}

/// A computed paycheck. All money values are signed integer CENTS.
/// The `unit` field documents this for JSON consumers explicitly.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paycheck {
    pub employee: String,
    pub state: String,
    pub locality: Option<String>,
    pub pay_frequency: PayFrequency,
    pub filing_status: FilingStatus,

    pub gross: Money,
    pub federal_income_tax: Money,
    pub social_security: Money,
    pub medicare: Money,
    pub additional_medicare: Money,
    /// State income tax (state-level only; local broken out separately).
    pub state_income_tax: Money,
    /// Local (county/city) income tax. `Money::ZERO` if locality has no income tax.
    pub local_income_tax: Money,
    pub net: Money,

    // ── Self-describing metadata so JSON consumers don't have to guess ──
    /// IRS Pub 15-T tax year baked into this build.
    pub tax_year: u32,
    /// ISO 4217 code. Always "USD" in v0.x; future internationalization will vary.
    pub currency: String,
    /// "cents" — every Money field is signed i64 cents. `200000` = $2,000.00.
    pub unit: String,
    /// `CARGO_PKG_VERSION` of the binary that produced this paycheck.
    pub version: String,
}

impl Paycheck {
    pub fn total_withheld(&self) -> Money {
        self.federal_income_tax
            + self.social_security
            + self.medicare
            + self.additional_medicare
            + self.state_income_tax
            + self.local_income_tax
    }
}

pub fn compute(emp: &Employee) -> anyhow::Result<Paycheck> {
    if emp.gross.0 < 0 {
        anyhow::bail!("gross wages must be non-negative; got {}", emp.gross);
    }
    if emp.ytd_ss_wages.0 < 0 {
        anyhow::bail!(
            "ytd_ss_wages must be non-negative; got {}",
            emp.ytd_ss_wages
        );
    }
    if emp.ytd_medicare_wages.0 < 0 {
        anyhow::bail!(
            "ytd_medicare_wages must be non-negative; got {}",
            emp.ytd_medicare_wages
        );
    }
    let state: State = emp.state.parse()?;
    let fit = federal::withhold(emp.gross, emp.filing_status, emp.pay_frequency);
    let ss = federal::social_security(emp.gross, emp.ytd_ss_wages);
    let med = federal::medicare(emp.gross);
    let addl = federal::additional_medicare(emp.gross, emp.ytd_medicare_wages);
    let breakdown = state.breakdown(
        emp.gross,
        emp.filing_status,
        emp.pay_frequency,
        emp.locality.as_deref(),
    );
    let net = emp.gross - fit - ss - med - addl - breakdown.state - breakdown.local;
    Ok(Paycheck {
        employee: emp.name.clone(),
        state: emp.state.clone(),
        locality: emp.locality.clone(),
        pay_frequency: emp.pay_frequency,
        filing_status: emp.filing_status,
        gross: emp.gross,
        federal_income_tax: fit,
        social_security: ss,
        medicare: med,
        additional_medicare: addl,
        state_income_tax: breakdown.state,
        local_income_tax: breakdown.local,
        net,
        tax_year: TAX_YEAR,
        currency: "USD".into(),
        unit: "cents".into(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Employee {
        Employee {
            name: "Test Employee".into(),
            state: "MD".into(),
            locality: Some("baltimore".into()),
            filing_status: FilingStatus::Single,
            pay_frequency: PayFrequency::Biweekly,
            gross: Money::dollars(2_000),
            ytd_ss_wages: Money::ZERO,
            ytd_medicare_wages: Money::ZERO,
        }
    }

    #[test]
    fn negative_gross_rejected() {
        let mut emp = baseline();
        emp.gross = Money::cents(-1);
        let err = compute(&emp).unwrap_err();
        assert!(format!("{err}").contains("non-negative"));
    }

    #[test]
    fn negative_ytd_rejected() {
        let mut emp = baseline();
        emp.ytd_ss_wages = Money::cents(-1);
        let err = compute(&emp).unwrap_err();
        assert!(format!("{err}").contains("non-negative"));
    }

    #[test]
    fn balanced_paycheck() {
        let pc = compute(&baseline()).unwrap();
        // Total withheld + net = gross, exactly. No rounding leak.
        assert_eq!(pc.total_withheld() + pc.net, pc.gross);
    }

    #[test]
    fn paycheck_metadata_set_correctly() {
        let pc = compute(&baseline()).unwrap();
        assert_eq!(pc.tax_year, TAX_YEAR);
        assert_eq!(pc.currency, "USD");
        assert_eq!(pc.unit, "cents");
        assert_eq!(pc.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(pc.state, "MD");
        assert_eq!(pc.locality.as_deref(), Some("baltimore"));
    }

    #[test]
    fn local_income_tax_broken_out() {
        let pc = compute(&baseline()).unwrap();
        assert!(pc.state_income_tax.0 > 0);
        assert!(pc.local_income_tax.0 > 0);
        // State + local should approximately equal what v0.1.0 reported as a single field.
        let combined = pc.state_income_tax + pc.local_income_tax;
        assert!(combined.0 > pc.state_income_tax.0); // local is non-zero
    }
}
