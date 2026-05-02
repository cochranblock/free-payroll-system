// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! State income tax. One module per state. v0.1.0 ships Maryland.
//! v0.2.0 will add the 9 no-income-tax states (trivial) + the rest.

pub mod md;

use crate::money::Money;
use crate::tax::{FilingStatus, PayFrequency};
use std::str::FromStr;

/// State withholding broken into state and local components. Real paystubs
/// itemize these separately so the local rate can be audited independently.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateAndLocal {
    pub state: Money,
    pub local: Money,
}

impl StateAndLocal {
    pub fn total(&self) -> Money {
        self.state + self.local
    }
}

/// State withholding interface. Each state implements its own rules.
pub trait StateTax {
    /// Two-letter postal code. e.g. "MD".
    fn code(&self) -> &'static str;

    /// State + local income tax withholding, itemized.
    fn breakdown(
        &self,
        gross: Money,
        status: FilingStatus,
        freq: PayFrequency,
        locality: Option<&str>,
    ) -> StateAndLocal;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    MD,
}

impl FromStr for State {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "MD" | "MARYLAND" => Ok(State::MD),
            other => anyhow::bail!(
                "state '{other}' not implemented in v0.1.0 (only MD ships in v0.1.0; v0.2.0 adds all 50)"
            ),
        }
    }
}

impl State {
    pub fn breakdown(
        self,
        gross: Money,
        status: FilingStatus,
        freq: PayFrequency,
        locality: Option<&str>,
    ) -> StateAndLocal {
        match self {
            State::MD => md::Maryland.breakdown(gross, status, freq, locality),
        }
    }
}
