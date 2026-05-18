<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# Proof of Artifacts — free-payroll-system

*Hard evidence that this project is real, working, and built by humans with AI assistance — not AI hallucination.*

## Project Metrics

| Metric | Value |
|--------|-------|
| Source files (.rs) | 10 |
| Test files (.rs) | 4 |
| Lines of source | 1,172 |
| Lines of tests | 403 |
| Tests | 51 |
| Commits | 3 (v0.1.0 foundation) |
| Binary size (diamond profile) | 860 KB |
| Direct dependencies (default) | 6 |
| Direct dependencies (with `tests` feature) | 8 |
| Edition | 2024 |
| MSRV | 1.85 |
| License | Unlicense |

## Repository

- **GitHub:** https://github.com/cochranblock/free-payroll-system
- **Crates.io:** https://crates.io/crates/free-payroll-system (v0.0.0 stub published 2026-05-01; v0.1.0 publish pending audit completion)
- **CI:** https://github.com/cochranblock/free-payroll-system/actions

## Status

**v0.1.0** — Federal income tax (Pub 15-T 2024 Worksheet 1A annualized method) + FICA + Maryland (state + county local). Biweekly only. One employee at a time, in-memory. Read-only — no DB, no register, no automatic YTD. Useful for: verifying a paystub, learning how withholding actually works, calling out SaaS payroll vendors.

## Roadmap

- **v0.2.0** — All 50 states + DC. Weekly/semimonthly/monthly. W-4 dependents/extra withholding. Multi-employee batch. Append-only payroll register with auto YTD.
- **v0.3.0** — `govfetch` rate-limited HTTP client + eCFR-versioned law store + auto-refreshed tax tables.
- **v0.4.0** — Form 941 quarterly aggregation, Form 940 FUTA, W-2 / W-3 year-end.
- **v0.5.0** — Pre-tax deductions (401k, HSA, Section 125), NACHA direct-deposit file output.

## Quick Start

```bash
cargo install free-payroll-system
free-payroll-system paystub --employee employee.toml
```

Sample output:

```
  ─── Paystub ─────────────────────────────────
  Employee: Jane Doe
  Gross wages                      $2000.00
  ─── Withholding ─────────────────────────────
  Federal income tax                $203.38
  Social Security (6.2%)            $124.00
  Medicare (1.45%)                   $29.00
  State income tax                  $139.39
    Total withheld                  $495.77
  ─────────────────────────────────────────────
  Net pay                          $1504.23

  Tax year: 2024 (Pub 15-T)
```

## Verify the Math in 60 Seconds

The whole point of this crate is that the math is *auditable*, not *trusted*.

```bash
git clone https://github.com/cochranblock/free-payroll-system
cd free-payroll-system
cargo test --features tests   # 51 tests against IRS Pub 15-T 2024 worked examples
```

Every constant in `src/tax/federal.rs` and `src/tax/state/md.rs` cites the IRS publication and page. See [`data/citations.md`](./data/citations.md). Diff brackets against the source PDFs. File an issue if anything is off.

## Tax Year Warning

This build computes withholding against **IRS Publication 15-T (tax year 2024)**. The CLI prints a warning to stderr on every `paystub` invocation. v0.3.0 will fetch the current year automatically via `govfetch`. Until then: verify against the current Pub 15-T before live payroll use.

## Origin

This project exists because of a public LinkedIn comment thread on 2026-05-01. See [`RECEIPTS.md`](./RECEIPTS.md) for the full 33-screenshot timeline (14:47–20:57 EDT).

## Architecture

free-payroll-system is a single-file-input, single-paystub-output CLI binary. Inputs (`Employee` config in TOML) flow through a pure-function compute pipeline; the output is either a human-formatted paystub on stdout or a JSON object for machine consumption. Zero network I/O. Zero database. Zero cloud dependencies. Zero phone-home telemetry. Zero per-employee pricing.

```
TOML Employee config
    │
    ▼
serde Deserialize ── validation (gross >= 0, ytd >= 0)
    │
    ▼
payroll::compute(&Employee) -> anyhow::Result<Paycheck>
    │
    ├─ federal::withhold (Pub 15-T 2024 Annualized Wage Method)
    ├─ federal::social_security (6.2% capped at $168,600 wage base)
    ├─ federal::medicare (1.45%, no cap)
    ├─ federal::additional_medicare (0.9% over $200k YTD)
    └─ State::withhold (per-state module — MD ships in v0.1.0)
            └─ MD: progressive brackets + 24-county local rate
    │
    ▼
Paycheck { gross, federal_income_tax, social_security, medicare,
           additional_medicare, state_income_tax, net }
    │
    ├─ Human format (cli::print_paystub)  → stdout
    └─ JSON (serde_json::to_string_pretty)  → stdout
        + tax-year warning ──────────────── → stderr
```

Money is always `i64` cents. No `f64`, no `rust_decimal`, no money-related dependency at all. `Money::mul_bps(bps)` does the percentage math via `i128` intermediate with banker's rounding.

## Named Techniques

| Technique | Description | Source |
|-----------|-------------|--------|
| Crate-Reservation-as-Public-Callout | Reserve crate name on crates.io live during a public argument with the incumbent vendor's salesperson | crates.io 2026-05-01T20:01:17Z |
| Tax-Year-Stamped Stderr Warning | Every paystub invocation states the IRS Pub 15-T year it computed against | `src/cli.rs` (`Command::Paystub`) |
| Annualized Wage Method | Per-pay-period gross → annual → bracket lookup → divide back | `src/tax/federal.rs::withhold` |
| Banker's-Rounded `mul_bps` | Percentage math via `i128` with round-half-to-even | `src/money.rs::Money::mul_bps` |
| Per-State Module Pattern | One file per state implementing `StateTax` trait; `State` enum dispatches | `src/tax/state/{mod,md}.rs` |
| Citation-Driven Constants | Every numeric tax constant has an inline source-publication citation | `src/tax/federal.rs`, `src/tax/state/md.rs`, `data/citations.md` |
| Regression Tripwires | Tests that pin constant values, framed and named explicitly so the auditor knows they're drift catchers, not algorithmic tests | `src/tax/state/md.rs::tests::tripwire_*` |

*See [`TIMELINE_OF_INVENTION.md`](./TIMELINE_OF_INVENTION.md) for full provenance.*

## Test Coverage

| Category | File | Count |
|----------|------|-------|
| Money (cents math, parsing, display, banker's rounding) | `src/money.rs` | 8 |
| Federal — FICA + brackets (in-source unit tests) | `src/tax/federal.rs` | 12 |
| Maryland (tripwires, sanity bounds, monotonicity, brackets) | `src/tax/state/md.rs` | 8 |
| Payroll compute (negative-input guards, balance) | `src/payroll.rs` | 3 |
| Federal Pub 15-T integration (FICA worked examples + invariants + statute pinning) | `tests/federal_pub15t.rs` | 11 |
| End-to-end (balance, monotonicity, JSON/TOML roundtrip, error paths) | `tests/end_to_end.rs` | 6 |
| Audit fixtures (cross-status/cross-frequency balance + contributor template) | `tests/audit.rs` | 1 |
| Version (`lib::version()` matches `CARGO_PKG_VERSION`) | `tests/version.rs` | 2 |
| **Total** | | **51** |

**TRIPLE SIMS quality gate:** `cargo run --features tests --bin free-payroll-system-test` runs `cargo test` + binary `--help` + `--version` 3 times via `exopack::triple_sims::f60`. Any flake = FAIL. CI runs this on every push.

### What the tests verify

- **Statutory FICA rates pinned to 26 USC § 3101.** SS_RATE_BPS=620, MEDICARE_RATE_BPS=145, ADDL_MEDICARE_RATE_BPS=90, FUTA_RATE_BPS=600, FUTA_WAGE_BASE=$7,000. Refactor that changes any of these = test fails.
- **Balance invariant.** For every paystub computed in tests, `total_withheld + net == gross`. No money created, no money destroyed.
- **Monotonicity (federal + state).** Higher gross → withholding never decreases. Property tested across all 12 (filing_status × pay_frequency) combinations for federal, single+biweekly for MD.
- **Cross-frequency consistency.** Same annual gross at weekly/biweekly/semimonthly/monthly produces ≈ same annual withholding (within 0.05% of target — catches algorithmic bugs without flapping on integer-division drift).
- **Wage-base cap (FICA Social Security).** YTD-aware capping at $168,600 (2024). Tests at the cap, above the cap, and at zero YTD.
- **Threshold crossing (Additional Medicare).** YTD-aware $200k threshold. Tests below, crossing, and fully above the threshold.
- **Annual maximum.** SS withholding for any single paycheck cannot exceed the annual ceiling ($10,453.20 for 2024).
- **County rate sanity.** All 24 published Maryland county rates fall in the legal `[1.00%, 5.00%]` band.
- **Negative-input rejection.** `compute()` rejects negative gross or negative YTD with a clear error.
- **JSON and TOML roundtrip.** Serde schemas are stable; deserializing the example employee.toml produces the expected `Employee` struct; serializing a `Paycheck` to JSON and back recovers the same struct.

### What the tests intentionally DON'T verify (yet)

- **Bracket numerical correctness.** The bracket constants in `src/tax/federal.rs` and `src/tax/state/md.rs` are *hand-entered* against IRS Pub 15-T 2024 and Comptroller of MD Withholding Facts 2024 (citations in `data/citations.md`). The tests verify the bracket-walk *algorithm* against those typed values, but cannot catch a typo in the values themselves. The mitigation: `tests/audit.rs` is a documented scaffold for adding externally-sourced (Pub 15-T worked example, IRS Tax Withholding Estimator, Comptroller of MD calculator) fixtures. Each PR adding such a fixture is a small, citable, audit-able cross-check.
- **W-4 dependents / extra withholding / deductions.** v0.1.0 assumes Step 2 unchecked, no dependents, no extra withholding. v0.2.0 will lift this.

## Compliance

- **SBOM:** generated by `cargo-cyclonedx` on release (post-v0.1.0).
- **SSDF:** aligned with NIST SP 800-218.
- **CISA Secure-by-Design:** memory-safe Rust; no `unsafe` in this crate; no network I/O at runtime in v0.1.0.
- **EO 14028:** aligned.
- **Audit trail:** every numeric tax constant cites its source publication in source comments and in [`data/citations.md`](./data/citations.md).

## Build

```bash
# Default build (release-equivalent for binary distribution)
cargo build --release

# Diamond profile — LTO fat, single codegen unit, panic abort, stripped
cargo build --profile=diamond

# Run a paystub from the example config
cargo run -- paystub --employee examples/employee.toml

# JSON output (stdout-only; warning goes to stderr)
cargo run -- paystub --employee examples/employee.toml --json

# Test suite (51 tests)
cargo test --features tests

# TRIPLE SIMS quality gate (full suite 3x)
cargo run --features tests --bin free-payroll-system-test
```

## Verification

A third party can verify every claim in this document:

1. **Clone and build:** `git clone https://github.com/cochranblock/free-payroll-system && cd free-payroll-system && cargo build` — zero errors, zero warnings (`RUSTFLAGS=-D warnings`).
2. **Run tests:** `cargo test --features tests` — 51 tests pass in <1 second.
3. **Quality gate:** `cargo run --features tests --bin free-payroll-system-test` — TRIPLE SIMS 3/3 pass.
4. **CI history:** https://github.com/cochranblock/free-payroll-system/actions — every push runs build, test, TRIPLE SIMS, paystub smoke, fmt, clippy `-D warnings`, and a diamond LTO build. All green.
5. **Commit history:** `git log --oneline` — every entry in [`TIMELINE_OF_INVENTION.md`](./TIMELINE_OF_INVENTION.md) maps to a real commit hash.
6. **Statute citations:** Open `src/tax/federal.rs` and search for "26 USC". Every FICA/FUTA constant has an inline citation. Click through to https://uscode.house.gov to verify.
7. **Pub 15-T diff:** Open `src/tax/federal.rs::SINGLE_BRACKETS_2024` and diff against IRS Pub 15-T 2024 Worksheet 1A page 10. (`data/citations.md` has the URL.) Open issue at https://github.com/cochranblock/free-payroll-system/issues if anything is off.
8. **Comptroller of MD diff:** Open `src/tax/state/md.rs` and diff brackets + per-county rates against Comptroller of MD Withholding Tax Facts 2024.
9. **Receipts:** Open [`RECEIPTS.md`](./RECEIPTS.md) — full LinkedIn-thread timeline (33 screenshots, 14:47–20:57 EDT 2026-05-01) is the project's origin record.
10. **End-to-end smoke:** `cargo run -- paystub --employee examples/employee.toml` prints a balanced paystub for "Jane Doe" — gross $2,000.00, total withheld $495.77, net $1,504.23, against MD/Baltimore County, biweekly, single. The same numbers appear in this README's quickstart block.

---

*Part of the [CochranBlock](https://cochranblock.org) zero-cloud architecture. All source under the Unlicense.*
<!-- COCHRANBLOCK-BRAND-FOOTER:START - generated by cochranblock/scripts/brand-stamp.sh -->

---

<sub>&#9656; **THE COCHRAN BLOCK, LLC** &#183; CAGE `1CQ66` &#183; UEI `W7X3HAQL9CF9` &#183; UNLICENSE &#183; [cochranblock.org](https://cochranblock.org)</sub>
<!-- COCHRANBLOCK-BRAND-FOOTER:END -->
