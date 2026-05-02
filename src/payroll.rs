// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Payroll: Employee config -> Paycheck breakdown. Pure function, no I/O.

use crate::money::Money;
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paycheck {
    pub employee: String,
    pub gross: Money,
    pub federal_income_tax: Money,
    pub social_security: Money,
    pub medicare: Money,
    pub additional_medicare: Money,
    pub state_income_tax: Money,
    pub net: Money,
}

impl Paycheck {
    pub fn total_withheld(&self) -> Money {
        self.federal_income_tax
            + self.social_security
            + self.medicare
            + self.additional_medicare
            + self.state_income_tax
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
    let sit = state.withhold(
        emp.gross,
        emp.filing_status,
        emp.pay_frequency,
        emp.locality.as_deref(),
    );
    let net = emp.gross - fit - ss - med - addl - sit;
    Ok(Paycheck {
        employee: emp.name.clone(),
        gross: emp.gross,
        federal_income_tax: fit,
        social_security: ss,
        medicare: med,
        additional_medicare: addl,
        state_income_tax: sit,
        net,
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
}
