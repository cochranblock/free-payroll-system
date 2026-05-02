// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! CLI: clap-based dispatcher. `paystub --employee path.toml` is the only verb in v0.1.0.

use crate::money::Money;
use crate::payroll::{Employee, Paycheck, compute};
use crate::tax::federal::TAX_YEAR;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "free-payroll-system")]
#[command(version)]
#[command(about = "Free, open-source, local-first payroll. No SaaS. No per-employee pricing.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Compute a paystub for an employee config (TOML).
    Paystub {
        /// Path to employee TOML config.
        #[arg(long, short = 'e')]
        employee: PathBuf,
        /// Emit JSON instead of human-readable paystub.
        #[arg(long)]
        json: bool,
    },
    /// Print the federal tax year baked into this build.
    TaxYear,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Paystub { employee, json }) => {
            eprintln!(
                "warning: this build computes against IRS Pub 15-T tax year {TAX_YEAR}. \
                 Verify against current Pub 15-T before live use."
            );
            let body = std::fs::read_to_string(&employee)
                .with_context(|| format!("read employee config: {}", employee.display()))?;
            let emp: Employee = toml::from_str(&body).context("parse employee TOML")?;
            let pc = compute(&emp)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&pc)?);
            } else {
                print_paystub(&pc);
            }
            Ok(())
        }
        Some(Command::TaxYear) => {
            println!("{TAX_YEAR}");
            Ok(())
        }
        None => {
            println!(
                "free-payroll-system {} — payroll without the SaaS tax.",
                env!("CARGO_PKG_VERSION")
            );
            println!("Run with --help to see commands.");
            println!();
            println!(
                "WARNING: this build computes against IRS Publication 15-T (tax year {TAX_YEAR})."
            );
            println!(
                "Verify against the current Pub 15-T before live use. v0.3.0 will fetch automatically."
            );
            Ok(())
        }
    }
}

fn print_paystub(pc: &Paycheck) {
    let line = |label: &str, amt: Money| println!("  {:<28} {:>12}", label, amt.to_string());
    println!();
    println!("  ─── Paystub ─────────────────────────────────");
    println!("  Employee: {}", pc.employee);
    line("Gross wages", pc.gross);
    println!("  ─── Withholding ─────────────────────────────");
    line("Federal income tax", pc.federal_income_tax);
    line("Social Security (6.2%)", pc.social_security);
    line("Medicare (1.45%)", pc.medicare);
    if pc.additional_medicare.0 > 0 {
        line("Additional Medicare (0.9%)", pc.additional_medicare);
    }
    line("State income tax", pc.state_income_tax);
    line("  Total withheld", pc.total_withheld());
    println!("  ─────────────────────────────────────────────");
    line("Net pay", pc.net);
    println!();
    println!("  Tax year: {} (Pub 15-T)", TAX_YEAR);
    println!();
}
