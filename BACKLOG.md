<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# Backlog

Prioritized work stack. Most important at top. Max 20 items.

Cross-project deps: **exopack** (TRIPLE SIMS, integrated v0.2.1), **govfetch** (rate-limited gov-API client, planned standalone crate, blocks v0.3.0), **chromiumoxide** (PDF/captcha scraping for tax-table refresh, planned).

---

1. [ux] Money: thousands separator in `Display` — `$2,000.00` not `$2000.00`. Cosmetic but every output reads better. One-line fix in `src/money.rs::Display`.
2. [ux] Money: serde `Deserialize` accepts EITHER `200000` (cents) OR `"$2,000.00"` (string). Eliminates the cents-footgun in `examples/employee.toml`. Custom deserializer using existing `Money::from_dollars_str`.
3. [ux] Paystub: break out `state_income_tax` into separate `state_income_tax` + `local_income_tax` fields in both human and JSON output. Real paystubs do this; auditors of a Baltimore County paystub need to verify the local rate independently.
4. [ux] Paystub JSON: add `tax_year`, `currency: "USD"`, `unit: "cents"`, `state`, `locality`, `pay_frequency`, `version`. Eliminates the "is `200000` cents or dollars" ambiguity for machine consumers.
5. [ux] `paystub --example` flag prints starter `employee.toml` to stdout. One less "where do I start" friction step.
6. [docs] Crate-level rustdoc on `lib.rs` and item-level rustdoc on `payroll::compute` + `Employee` + `Paycheck`. docs.rs page is the auditor-persona's first stop.
7. [feature] W-4 fields on `Employee` — `dependents`, `other_income`, `deductions`, `extra_withholding`. Pub 15-T Worksheet 1A actually requires these; v0.1.0 approximates as zero. Required for accuracy.
8. [feature] Multi-employee batch — `paystub-batch --employees roster.toml --pay-date YYYY-MM-DD --register register.jsonl`. Append-only register auto-increments YTD. Eliminates the "user must update ytd_ss_wages by hand" footgun.
9. [feature] All 50 states + DC + PR. Start with the 9 no-income-tax states (TX/FL/WA/NV/SD/WY/AK/TN/NH) — each ≤10 lines. Refactor `State` enum → `HashMap<&str, Box<dyn StateTax>>` registry once count > 20.
10. [feature] Pay frequency family — `Daily`, `Quarterly` in addition to existing weekly/biweekly/semimonthly/monthly. ~5 lines in `PayFrequency`.
11. [test] External-authority audit fixtures — populate `tests/audit.rs` with at least 5 `(input, expected)` pairs sourced from IRS Pub 15-T 2024 worked examples and the IRS Tax Withholding Estimator. Required for v0.2.0. Each fixture cites the source page/URL.
12. [build] `govfetch` standalone crate — token-bucket per host, `Retry-After` aware, `If-None-Match` / `If-Modified-Since` SQLite cache, `robots.txt` enforced, append-only JSONL audit log, configurable per-source TOML. Smoke target: regulations.gov + sam.gov. Blocks v0.3.0.
12. [feature] eCFR-versioned law store — `data/law/{usc|cfr}/title-26/section-3101/YYYY-MM-DD.xml` append-only via eCFR API (`/api/versioner/v1/full/{date}/title-26.xml`). `fps law show 26usc3101 --as-of 2024-01-01` time-travel query. Phil cannot match this at any subscription price.
13. [feature] Auto-refresh tax tables via `govfetch` from IRS Pub 15-T URL on `make refresh-tax-tables`. Two-source verify against last year's tables + announced delta; fail loud on mismatch.
14. [security] Pin exopack to commit hash — currently `version = "0.2"` resolves to `0.2.1` but a bad release of exopack to 0.2.x would silently affect this crate. `exopack = { git = "...", rev = "<hash>" }` once the use-case is post-v0.1.0 mature.
15. [feature] Form 941 quarterly aggregation, Form 940 FUTA annual, W-2 / W-3 year-end. Reuses the YTD register from item #8.
16. [feature] Pre-tax deductions — 401(k) traditional + Roth, HSA, Section 125 (FSA, premiums). Order-of-operations matters: 401(k) reduces FIT base but not FICA base; HSA reduces both.
17. [feature] NACHA direct-deposit file output. Significant compliance footprint; v0.5.0+.
18. [docs] Asciinema recording of paystub command in README. Visual converts harder than a code block.
19. [research] State-tax-table scrape strategy — most state DORs publish PDFs, not APIs. PDF-table parsing via `pdfium-render` or `lopdf` with table-extraction heuristics. Two-source verify against last year + announced delta.
20. [build] Cargo-cyclonedx SBOM generation step in CI; embed in release binary metadata.

---

*Last updated: 2026-05-02. P23 triple lens applied: technical (does it compile/test/run on real hardware? — yes, 51/51 + CI green), product (does it solve a real problem? — yes, free local payroll for small businesses, no SaaS dependency), honest (are claims verifiable? — yes, every constant cited, every commit linked, statute pinned to 26 USC).*
