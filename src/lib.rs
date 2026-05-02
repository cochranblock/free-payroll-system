// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7
// free-payroll-system — Free, open-source, local-first payroll.

pub mod cli;
pub mod money;
pub mod payroll;
pub mod tax;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
