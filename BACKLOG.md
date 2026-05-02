<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# Backlog

Prioritized work stack. Most important at top. Max 20 items.

Cross-project deps: **exopack** (TRIPLE SIMS, integrated v0.2.1), **govfetch** (rate-limited gov-API client, planned standalone crate, blocks v0.3.0), **chromiumoxide** (PDF/captcha scraping + OpenAPI Swagger-UI scrape, planned), **pdfium-render** (PDF table extraction — decision Q7).

---

1. [ux] Money: thousands separator in `Display` — `$2,000.00` not `$2000.00`. One-line fix in `src/money.rs::Display`.
2. [ux] Money: serde `Deserialize` accepts EITHER `200000` (cents) OR `"$2,000.00"` (string). Eliminates the cents-footgun in `examples/employee.toml`. Custom deserializer using existing `Money::from_dollars_str`. **Add `Money::Banded { default, alt, threshold }`** to handle Nebraska/Rhode Island SUI band ranges (decision Q4).
3. [ux] Paystub: break out `state_income_tax` into separate `state_income_tax` + `local_income_tax` fields in both human and JSON output. Real paystubs do this; auditors of a Baltimore County paystub need to verify the local rate independently.
4. [ux] Paystub JSON: add `tax_year`, `currency: "USD"`, `unit: "cents"`, `state`, `locality`, `pay_frequency`, `version`. Eliminates the "is `200000` cents or dollars" ambiguity.
5. [ux] `paystub --example` flag prints starter `employee.toml` to stdout. One less "where do I start" step.
6. [docs] Crate-level rustdoc on `lib.rs` and item-level rustdoc on `payroll::compute` + `Employee` + `Paycheck`. docs.rs is the auditor's first stop.
7. [feature] W-4 fields on `Employee` — `dependents`, `other_income`, `deductions`, `extra_withholding`. Pub 15-T Worksheet 1A requires these; v0.1.0 approximates as zero.
8. [feature] Multi-employee batch — `paystub-batch --employees roster.toml --pay-date YYYY-MM-DD --register register.jsonl`. Append-only register auto-increments YTD. Eliminates the "user must update ytd_ss_wages by hand" footgun.
9. [feature] All 50 states + DC + PR + VI. Start with the 9 no-income-tax states (TX/FL/WA/NV/SD/WY/AK/TN/NH) — each ≤10 lines. Refactor `State` enum → `HashMap<&str, Box<dyn StateTax>>` registry once count > 20. **Loader for `data/state-sui-wage-base/2026.json` already shipped (decision Q4); plug into per-state SUI calc.**
10. [feature] Pay frequency family — `Daily`, `Quarterly`. ~5 lines in `PayFrequency`.
11. [test] External-authority audit fixtures — populate `tests/audit.rs` with at least 5 `(input, expected)` pairs sourced from IRS Pub 15-T 2024 worked examples and the IRS Tax Withholding Estimator. Each cites source page/URL.
12. [build] `govfetch` standalone crate — token-bucket per host, `Retry-After` aware, `If-None-Match` / `If-Modified-Since` SQLite cache, `robots.txt` enforced, append-only JSONL audit log, per-source TOML config. **Type-gated GET-only (decision Q11) — POST is not in the type system.** Smoke target: regulations.gov + sam.gov. Blocks v0.3.0.
13. [feature] eCFR-versioned law store — `data/law/{usc|cfr}/title-26/section-3101/YYYY-MM-DD.xml` via eCFR Versioner API (`/api/versioner/v1/full/{date}/title-26.xml?part=31`). `fps law show 26usc3101 --as-of 2024-01-01` time-travel query. **Floor advertised: 2017-01-01 (decision Q2); pre-2017 errors with link to GovInfo USCODE bulk archive.** Phil cannot match this at any subscription price.
14. [feature] Auto-refresh tax tables via `govfetch` from IRS Pub 15-T URL on `make refresh-tax-tables`. **PDF extraction via `pdfium-render` (decision Q7) in build-time crate `free-payroll-system-extract`; end-user binary stays <1MB.** Two-source verify against Federal Register announcement + last year's tables + announced delta; fail loud on mismatch.
15. [build] `cargo xtask vendor-openapi` task — chromiumoxide headless Chrome scrapes Swagger UI pages (eCFR, Federal Register, Regulations.gov, BLS, Congress, SAM), extracts `window.ui.specSelectors.specStr()`, validates against OpenAPI 3.x, writes to `data/openapi/<source>-YYYY-MM-DD.json`. Nightly CI; PR on diff. **(Decision Q3.)**
16. [feature] Form 941 quarterly aggregation, Form 940 FUTA annual, W-2 / W-3 year-end. Reuses the YTD register from item #8.
17. [feature] Pre-tax deductions — 401(k) traditional + Roth, HSA, Section 125 (FSA, premiums). Order-of-operations matters: 401(k) reduces FIT base but not FICA base; HSA reduces both.
18. [feature] PA Act-32 PSD-code dataset (~2,500 municipalities × 2 EIT rates resident/non-resident + LST flat fees) from `dced.pa.gov`. **(Decision Q10 — PA wins us the press cycle: every PA small business currently pays ~$50/mo per employee specifically because PSD-code lookups are hellish manually.)** v0.4.0 target. Followed by OH RITA+CCA (~600 cities), MI city tax (24), AL city tax (~30), KY county+city occupational (~120). Each as a `data/local-tax/<state>/<year>.json` fixture with `pdfium-render` or `csv` extraction.
19. [security] `SECURITY.md` + NanoSign key custody — YubiKey 5C Nano holds production BLAKE3 keyed-hash key in PIV slot; `~/.config/cochranblock/nanosign-dev.key` for local dev with `dev: true` flag in manifest; CI rejects `dev: true` commits. 12-month key rotation; prior keys archived to sealed envelope; pubkey index in `data/nanosign-pubkeys/`. **(Decision Q12.)** Required before v0.3.0 cuts the first signed generation.
20. [feature] NACHA direct-deposit file output. Significant compliance footprint; v0.5.0+.

---

*Last updated: 2026-05-02. P23 triple lens applied: technical (51/51 tests pass + CI green), product (every roadmap item maps to a SaaS-killer outcome — PA PSD wins press, eCFR floor enables time-travel, type-gated POST prevents identity misuse, multi-tier custody prevents key compromise), honest (every URL hit during research probes is in `docs/data-sources.md`; SSA bot-block is acknowledged not papered over). All 12 open questions from data-sources.md have been answered as of this revision.*
