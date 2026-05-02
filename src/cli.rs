// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! CLI: clap-based dispatcher. `paystub --employee path.toml` is the primary verb.

use crate::money::Money;
use crate::payroll::{Employee, Paycheck, compute};
use crate::tax::federal::TAX_YEAR;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const STARTER_EMPLOYEE_TOML: &str = include_str!("../examples/employee.toml");

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
        /// Emit JSON instead of human-readable paystub. JSON is fully self-describing
        /// (tax_year, currency, unit, version, state, locality, pay_frequency).
        #[arg(long)]
        json: bool,
    },
    /// Print a starter `employee.toml` to stdout. Pipe to a file:
    /// `free-payroll-system example > me.toml`
    Example,
    /// Print the federal tax year baked into this build.
    TaxYear,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Paystub { employee, json }) => run_paystub(&employee, json),
        Some(Command::Example) => {
            print!("{STARTER_EMPLOYEE_TOML}");
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
            println!();
            println!("Quick start:");
            println!("  free-payroll-system example > me.toml      # write a starter config");
            println!("  free-payroll-system paystub -e me.toml     # compute paystub");
            println!("  free-payroll-system paystub -e me.toml --json   # for jq / scripts");
            println!();
            println!("Tax year: IRS Pub 15-T {TAX_YEAR}.");
            println!("Source: https://github.com/cochranblock/free-payroll-system");
            Ok(())
        }
    }
}

fn run_paystub(employee_path: &std::path::Path, json: bool) -> Result<()> {
    let body = std::fs::read_to_string(employee_path)
        .with_context(|| format!("read employee config: {}", employee_path.display()))?;
    let emp: Employee = toml::from_str(&body).with_context(|| {
        format!(
            "parse employee TOML at {}\n  Hint: see https://github.com/cochranblock/free-payroll-system/blob/master/examples/employee.toml or run `free-payroll-system example > {}`",
            employee_path.display(),
            employee_path.display(),
        )
    })?;
    let pc = compute(&emp)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&pc)?);
    } else {
        print_paystub(&pc);
    }
    // Tax-year warning AFTER successful paystub. Stderr so JSON pipes are unaffected.
    // Skipped on error paths — the user already has bigger problems than the year.
    eprintln!(
        "note: computed against IRS Pub 15-T {TAX_YEAR}. \
         Cross-check current Pub 15-T before live payroll use. \
         v0.3.0 auto-refreshes via govfetch."
    );
    Ok(())
}

fn print_paystub(pc: &Paycheck) {
    let line = |label: &str, amt: Money| println!("  {:<30} {:>14}", label, amt.to_string());
    println!();
    println!("  ─── Paystub ─────────────────────────────────────");
    println!("  Employee:        {}", pc.employee);
    println!(
        "  Pay period:      {:?}  ({:?})",
        pc.pay_frequency, pc.filing_status
    );
    println!(
        "  Jurisdiction:    {} / {}",
        pc.state,
        pc.locality.as_deref().unwrap_or("(no locality)")
    );
    line("Gross wages", pc.gross);
    println!("  ─── Withholding ─────────────────────────────────");
    line("Federal income tax", pc.federal_income_tax);
    line("Social Security", pc.social_security);
    line("Medicare", pc.medicare);
    if pc.additional_medicare.0 > 0 {
        line("Additional Medicare", pc.additional_medicare);
    }
    line(
        &format!("{} state income tax", pc.state),
        pc.state_income_tax,
    );
    if pc.local_income_tax.0 > 0 {
        line(
            &format!(
                "{} local income tax",
                pc.locality.as_deref().unwrap_or("Local")
            ),
            pc.local_income_tax,
        );
    }
    line("Total withheld", pc.total_withheld());
    println!("  ─────────────────────────────────────────────────");
    line("Net pay", pc.net);
    println!();
    println!(
        "  Tax year: {}  ·  free-payroll-system {}",
        pc.tax_year, pc.version
    );
    println!();
}
