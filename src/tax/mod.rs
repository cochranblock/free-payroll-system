// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! Tax computation. Federal (IRS Pub 15-T, FICA, FUTA) and state (one file per state).
//! Every numeric constant is sourced. See `data/citations.md`.

pub mod federal;
pub mod state;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    /// Single, or Married Filing Separately (Pub 15-T treats these together for withholding).
    Single,
    /// Married Filing Jointly.
    MarriedJointly,
    /// Head of Household.
    HeadOfHousehold,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayFrequency {
    Weekly,
    Biweekly,
    Semimonthly,
    Monthly,
}

impl PayFrequency {
    /// Pay periods per year for annualization (Pub 15-T methodology).
    pub const fn periods_per_year(self) -> i64 {
        match self {
            PayFrequency::Weekly => 52,
            PayFrequency::Biweekly => 26,
            PayFrequency::Semimonthly => 24,
            PayFrequency::Monthly => 12,
        }
    }
}
