<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# Citations

Every numeric constant in this crate is sourced. v0.1.0 is hand-entered against the publications below. v0.3.0 fetches them via `govfetch` from the canonical URLs.

## Federal — Tax year 2024

| Constant | Source | Notes |
|---|---|---|
| Single / MFS annual brackets | IRS Pub 15-T (2024) — Worksheet 1A "Percentage Method Tables for Automated Payroll Systems", Standard Withholding Schedule (W-4 Step 2 unchecked) | Annualized Wage Method. |
| MFJ annual brackets | IRS Pub 15-T (2024) — Worksheet 1A, Standard MFJ | |
| HOH annual brackets | IRS Pub 15-T (2024) — Worksheet 1A, Standard HOH | |
| Social Security wage base ($168,600) | SSA Cost-of-Living Announcement, 2023-10-12 | https://www.ssa.gov/oact/cola/cbb.html |
| Social Security rate (6.2%) | 26 USC § 3101(a) | Stable. |
| Medicare rate (1.45%) | 26 USC § 3101(b)(1) | Stable. |
| Additional Medicare (0.9% over $200k) | 26 USC § 3101(b)(2) — ACA, employer threshold | Stable since 2013. |
| FUTA rate / wage base | 26 USC § 3301; IRS Form 940 | Employer side; not on paystub. |

Authoritative URLs:
- Pub 15-T: https://www.irs.gov/pub/irs-pdf/p15t.pdf
- 26 USC: https://uscode.house.gov/view.xhtml?path=/prelim@title26/subtitleC&edition=prelim
- 26 CFR Part 31: https://www.ecfr.gov/current/title-26/chapter-I/subchapter-C/part-31

## Maryland — Tax year 2024

| Constant | Source |
|---|---|
| Personal exemption ($3,200) | Comptroller of Maryland — "Withholding Tax Facts 2024" |
| Standard deduction (15%, $1,800–$2,550 single, $1,800–$5,150 MFJ) | Comptroller of Maryland — "Withholding Tax Facts 2024" |
| State brackets (Single + MFJ, 2%–5.75%) | Comptroller of Maryland — "Withholding Tax Facts 2024" |
| Local rates per county (calendar 2024) | Comptroller of Maryland — "Local Income Tax Rates" |

Authoritative URLs:
- Withholding Facts: https://marylandtaxes.gov/forms/current_forms/Withholding_tax_facts.pdf
- Local rates: https://marylandtaxes.gov/individual/income/tax-info/local-tax.php

## Audit invitation

Every constant in this crate is a literal in source. There are no remote fetches at runtime in v0.1.0. To verify the math:

1. Open the publication URL above.
2. Open `src/tax/federal.rs` or `src/tax/state/md.rs`.
3. Diff. File issues at https://github.com/cochranblock/free-payroll-system/issues if anything is off.

This file is the moat melting in public.
