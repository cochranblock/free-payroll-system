// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! # free-payroll-system
//!
//! Free, open-source, **local-first** payroll. No SaaS. No per-employee pricing. Forever.
//!
//! This crate ships a deterministic, audit-friendly US payroll-withholding
//! engine. Federal income tax (IRS Pub 15-T 2024 Worksheet 1A — Annualized
//! Wage Method), FICA (Social Security 6.2% capped at the SSA wage base,
//! Medicare 1.45%, Additional Medicare 0.9% over $200k YTD), and Maryland
//! state + county-local income tax are implemented in v0.1.x. All 50 states
//! land in v0.2.0.
//!
//! ## Usage as a library
//!
//! ```no_run
//! use free_payroll_system::money::Money;
//! use free_payroll_system::payroll::{compute, Employee};
//! use free_payroll_system::tax::{FilingStatus, PayFrequency};
//!
//! let emp = Employee {
//!     name: "Jane Doe".into(),
//!     state: "MD".into(),
//!     locality: Some("baltimore".into()),
//!     filing_status: FilingStatus::Single,
//!     pay_frequency: PayFrequency::Biweekly,
//!     gross: Money::dollars(2_000),
//!     ytd_ss_wages: Money::ZERO,
//!     ytd_medicare_wages: Money::ZERO,
//! };
//! let paycheck = compute(&emp).unwrap();
//! println!("Net pay: {}", paycheck.net);
//! ```
//!
//! ## Usage as a CLI
//!
//! ```text
//! cargo install free-payroll-system
//! free-payroll-system example > me.toml
//! free-payroll-system paystub --employee me.toml
//! free-payroll-system paystub --employee me.toml --json   # for jq / scripts
//! ```
//!
//! ## Design principles
//!
//! - **Money is `i64` cents.** No floats, no `rust_decimal`. `Money::mul_bps`
//!   does percentage math via `i128` intermediate with banker's rounding.
//! - **Every numeric tax constant cites its source.** Open `src/tax/federal.rs`
//!   or `src/tax/state/md.rs` and search for `26 USC` / `Pub 15-T` /
//!   `Comptroller of MD`. Audit trail in [`data/citations.md`](https://github.com/cochranblock/free-payroll-system/blob/master/data/citations.md).
//! - **Pure function over an `Employee`.** No I/O in the library; the CLI is a
//!   thin wrapper that reads TOML and prints JSON or human output.
//! - **Self-describing JSON.** `Paycheck` includes `tax_year`, `currency`,
//!   `unit`, `version`, and the full `state` / `locality` / `pay_frequency` /
//!   `filing_status` so JSON consumers cannot misread a value.
//! - **Tax-year stamped in stderr** on every `paystub` invocation so users
//!   in 2028 don't accidentally compute against 2024 brackets without warning.
//!
//! ## See also
//!
//! - [`payroll::compute`] — main entry point.
//! - [`money::Money`] — i64-cents wrapper with custom serde Deserialize that
//!   accepts both `200000` (integer cents) and `"$2,000.00"` (dollar string).
//! - [`tax::federal`] — federal withholding + FICA constants.
//! - [`tax::state::md`] — Maryland state and county-local withholding.

pub mod cli;
pub mod money;
pub mod payroll;
pub mod tax;

/// `CARGO_PKG_VERSION` of this build. Useful for embedding in paystub
/// metadata so the consumer knows which version produced the numbers.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
