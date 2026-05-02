<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# free-payroll-system

[![CI](https://github.com/cochranblock/free-payroll-system/actions/workflows/ci.yml/badge.svg)](https://github.com/cochranblock/free-payroll-system/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/free-payroll-system.svg)](https://crates.io/crates/free-payroll-system)
[![Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](./UNLICENSE)

Free, open-source, local-first payroll. **No SaaS. No per-employee pricing. No cloud. Forever.**

```
cargo install free-payroll-system
free-payroll-system paystub --employee employee.toml
```

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

## Verify the math in 60 seconds

You don't trust me. You shouldn't. The whole point of this crate is that the math is *auditable*, not *trusted*.

```bash
git clone https://github.com/cochranblock/free-payroll-system
cd free-payroll-system
cargo test --features tests   # 40+ tests against IRS Pub 15-T 2024 worked examples
```

Every constant in `src/tax/federal.rs` and `src/tax/state/md.rs` cites the IRS publication and page. See [`data/citations.md`](./data/citations.md). Diff our brackets against the source PDFs. File an issue if anything is off.

## Status

**v0.1.0** — Federal income tax (Pub 15-T 2024 Worksheet 1A annualized method) + FICA + Maryland (state + county local). Biweekly only. One employee at a time, in-memory. Read-only — no DB, no register, no automatic YTD. Useful for: verifying a paystub, learning how withholding actually works, calling out SaaS payroll vendors.

**Roadmap:**

- v0.2.0 — All 50 states + DC. Weekly/semimonthly/monthly. W-4 dependents/extra withholding. Multi-employee batch. Append-only payroll register with auto YTD.
- v0.3.0 — `govfetch` rate-limited HTTP client + eCFR-versioned law store + auto-refreshed tax tables.
- v0.4.0 — Form 941 quarterly aggregation, Form 940 FUTA, W-2 / W-3 year-end.
- v0.5.0 — Pre-tax deductions (401k, HSA, Section 125), NACHA direct-deposit file output.

## Tax year warning

This build computes withholding against **IRS Publication 15-T (tax year 2024)**. The CLI prints a warning to stderr on every `paystub` invocation. v0.3.0 will fetch the current year automatically via `govfetch`. Until then: verify against the current Pub 15-T before live payroll use.

## Contributing

See [`CONTRIBUTING.md`](./CONTRIBUTING.md). Adding your state is a single new file in `src/tax/state/`. The MD module is the reference implementation.

## Origin

This project exists because of a public LinkedIn comment thread on 2026-05-01. See [`RECEIPTS.md`](./RECEIPTS.md).

## License

Unlicense (public domain). Use this. Fork it. Sell support around it. Strip the headers if you want — that's the whole point of the Unlicense. The only thing you can't do is stop someone else from doing the same.

[cochranblock.org](https://cochranblock.org)
