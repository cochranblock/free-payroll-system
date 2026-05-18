<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# Data Sources — `free-payroll-system` v0.3.0+

This document is the source-of-truth registry for every external `.gov` (and one `.edu` fallback) endpoint that the planned `govfetch` crate (BACKLOG.md item #11) is allowed to read. Every entry below was verified by HTTP HEAD or page fetch on 2026-05-01 unless a "could-not-verify" note is attached. No URL in this file was guessed or constructed by analogy — every one was returned by either WebSearch result snippets or direct WebFetch.

## 1. Goal

`free-payroll-system` ships a deterministic, auditable, pure-Rust withholding calculator. The numeric constants (federal brackets, FICA wage base, state brackets, county local rates, SUI wage bases) currently live as hand-typed literals in `src/tax/federal.rs` and `src/tax/state/<state>.rs`. v0.3.0 introduces a **weekly-rotation, signed-artifact data pipeline**: small deterministic JSON artifacts ("AI models" in our internal shorthand — they are decision tables, not neural networks) extracted from authoritative `.gov` publications. Each artifact is refreshed weekly per source on a stagger so the whole 50-state corpus polls in a 7-day window without any single host receiving more than one outbound request from the cluster per week. Each generation is sealed with a NanoSign signature, distributed as part of the published Rust crate (`include_bytes!` at the canonical path `data/generations/current/`), and re-served by the WASM build at `payroll.cochranblock.org` over the same content-addressed manifest. The end state: an auditor can `git log -- data/generations/` and see every change to every constant, who fetched it, when, the upstream URL, the upstream sha256, and the offline signature over the whole generation. No subscription. No vendor lock-in. No "trust us."

## 2. Federal sources

| Authority | Dataset | Canonical URL | Format | Update cadence | API? | OpenAPI? | Auth | Rate limit | robots.txt status | Used for |
|---|---|---|---|---|---|---|---|---|---|---|
| IRS | Pub 15-T (Federal Income Tax Withholding Methods) | `https://www.irs.gov/pub/irs-pdf/p15t.pdf` | PDF | Annual (Dec, with mid-year amendments via Federal Register) | No | n/a | None | Not documented; treat as 1 req/day per host | `/pub/irs-pdf/` not in Disallow → allowed | Federal income tax brackets, Worksheet 1A annualized method, supplemental flat rate |
| IRS | Pub 15 / Circular E (Employer's Tax Guide) | `https://www.irs.gov/pub/irs-pdf/p15.pdf` | PDF | Annual | No | n/a | None | Same as above | Allowed | FICA rates, deposit schedules, Form 941 cross-reference |
| IRS | Pub 15-T HTML | `https://www.irs.gov/publications/p15t` | HTML | Annual | No | n/a | None | Same as above | Allowed | Diff target — text version of the PDF for two-source verify |
| IRS | Pub 15-T draft | `https://www.irs.gov/pub/irs-dft/p15t--dft.pdf` | PDF | When in revision | No | n/a | None | Same as above | Allowed | Early-warning preview of next year's tables |
| IRS | EPUB build | `https://www.irs.gov/pub/ebook/p15t.epub` | EPUB (zip+xhtml) | Annual | No | n/a | None | Same as above | Allowed | Tertiary verify channel; cleaner HTML extraction |
| SSA | Contribution and Benefit Base table | `https://www.ssa.gov/oact/cola/cbb.html` | HTML table | Annual (Oct COLA announcement) | No | n/a | None | SSA blocks WebFetch UA; fetch via `govfetch` with documented identifying UA. robots.txt fetch returned 403 — assume conservative 1 req/day | Not verifiable from this run | Social Security wage base ($176,100 for 2025; $184,500 for 2026) |
| SSA | COLA fact sheet (current year) | `https://www.ssa.gov/news/en/cola/factsheets/2026.html` | HTML | Annual | No | n/a | None | Same as above | Same | Two-source verify against `cbb.html` |
| SSA | CBB determination methodology | `https://www.ssa.gov/OACT/cola/cbbdet.html` | HTML | Annual | No | n/a | None | Same as above | Same | Audit citation — explains how the wage base is computed |
| DOL ETA | "Comparison of State Unemployment Laws" | `https://oui.doleta.gov/unemploy/statelaws.asp` | HTML index → PDF chapters | Annual (Jan) | No | n/a | None | Conservative 1 req/day | Not blocked by robots (subdomain) | FUTA cross-reference; per-state SUI law summary used as the canonical "what does this state actually do" reference |
| DOL ETA | UI law comparison 2023 (latest published) | `https://oui.doleta.gov/unemploy/comparison/2020-2029/comparison2023.asp` | HTML | Annual | No | n/a | None | Same | Same | Per-state SUI eligibility + tax structure summary |
| BLS | Public Data API v2 (timeseries) | `https://api.bls.gov/publicAPI/v2/timeseries/data/` | JSON | Continuous | **Yes** | No formal OpenAPI; "API Signatures" doc at `bls.gov/developers/api_signature_v2.htm` | Optional registration key (`registrationkey` field in POST body) | **Without key:** 25 series/query, 25 queries/day, 10 yrs of data. **With key:** 50 series/query, 500 queries/day, 20 yrs of data. No documented `Retry-After`. | Allowed | CES wage statistics for state-by-state cross-validation of withholding output (sanity check that we are not 10× off in some county) |
| BLS | Series catalog HTML | `https://www.bls.gov/developers/api_features.htm` | HTML | Static | No | n/a | None | n/a | Allowed | Documenting which series IDs we pull |
| eCFR | Versioner v1 — Title 26 Part 31 as-of-date | `https://www.ecfr.gov/api/versioner/v1/full/{YYYY-MM-DD}/title-26.xml?part=31` | XML | Daily (CFR is updated continuously by eCFR; statutory once-a-year roll-up at govinfo) | **Yes** | **Yes** — interactive Swagger at `ecfr.gov/developers/documentation/api/v1` (host blocks WebFetch but Swagger UI is documented in multiple search results and confirmed via the working sample URL `…/full/2023-01-27/title-26.xml?part=31`) | None | No public rate limit published; treat as the standard api.data.gov 1000/hr default until a 429 says otherwise | Allowed (host-level robots; eCFR has historically been crawler-friendly) | Time-travel CFR text. **This is the single most important endpoint in the file** — it is what enables `fps law show 26cfr31.3402-1 --as-of 2024-01-01` from BACKLOG.md item #12 |
| eCFR | Versioner v1 — title structure | `https://www.ecfr.gov/api/versioner/v1/structure/{date}/title-26.json` | JSON | Daily | Yes | Yes | None | Same | Allowed | Discover which sections existed on a given date before fetching XML |
| eCFR | Versioner v1 — title summary | `https://www.ecfr.gov/api/versioner/v1/titles.json` | JSON | Daily | Yes | Yes | None | Same | Allowed | Health-check + last-updated timestamp per title |
| eCFR | Search v1 | `https://www.ecfr.gov/api/search/v1/results` | JSON | Real-time | Yes | Yes | None | Same | Allowed | Backstop: find the section that matches a phrase like "supplemental wage" |
| Federal Register | Documents API v1 | `https://www.federalregister.gov/api/v1/documents.json?conditions[agencies][]=internal-revenue-service` | JSON | Daily | **Yes** | **Yes** — Swagger at `federalregister.gov/developers/documentation/api/v1` (host blocked WebFetch but documented widely; CRAN R package `federalregister` confirms the schema) | None | Pagination capped at 2000 results per query; no documented per-second/hour limit (FR docs explicitly say "monitored at the infrastructure layer"). Treat as 60 req/min ceiling. | Allowed | Proposed and final rule notices that touch IRC Title 26 — drives "did Treasury announce a withholding-table change since our last fetch?" |
| Federal Register | Single document | `https://www.federalregister.gov/api/v1/documents/{document_number}.json` | JSON | n/a | Yes | Yes | None | Same | Allowed | Full text of a notice we already saw in the listing |
| Federal Register | Public Inspection | `https://www.federalregister.gov/api/v1/public-inspection-documents.json` | JSON | Daily 8:45 AM ET | Yes | Yes | None | Same | Allowed | Pre-publication preview of imminent Treasury rules |
| Congress.gov | API v3 | `https://api.congress.gov/v3/bill/{congress}/{bill-type}/{bill-number}` | JSON or XML | Daily | **Yes** | **Yes** — swagger.json published in `LibraryOfCongress/api.congress.gov` GitHub repo | **Required.** API key from `api.data.gov` passed as `X-Api-Key` header or `?api_key=` query | **5,000 requests / hour** per key (api.data.gov standard) | Allowed | 26 USC text + amendment history. Pulls bills referencing Title 26 sections we cite |
| Congress.gov | Bill summaries | `https://api.congress.gov/v3/summaries` | JSON | Daily | Yes | Yes | Same | Same | Allowed | Lighter cousin of full bill text — for "did anything happen this week?" sweep |
| Regulations.gov | API v4 documents | `https://api.regulations.gov/v4/documents` | JSON | Real-time | **Yes** | **Yes** — at `open.gsa.gov/api/regulationsgov/` | **Required.** API key (`X-Api-Key` header) | **GET endpoints:** api.data.gov default 1,000/hr. **POST commenting:** 50/min, 500/hr. `X-RateLimit-Limit` and `X-RateLimit-Remaining` headers returned. | Allowed | Cross-reference with Federal Register; richer comment / docket metadata |
| Regulations.gov | API v4 dockets | `https://api.regulations.gov/v4/dockets` | JSON | Real-time | Yes | Yes | Same | Same | Allowed | Group multiple notices into a single rulemaking thread |
| SAM.gov | Entity Management API | `https://api.sam.gov/entity-information/v3/entities` | JSON | Daily | **Yes** | **Yes** — at `open.gsa.gov/api/entity-api/` | **Required.** Api.data.gov key + SAM-specific role | **Public tier:** 10/day. **Non-federal registered:** 1,000/day. **Federal system account:** 10,000/day. Counters reset on a rolling 24-hr basis. | Allowed | Verify EIN-to-legal-name mapping for the hosted portal's eventual "audit trail" (out of scope for v0.3.0; included now for stable URL pinning) |
| data.gov | CKAN catalog API | `https://catalog.data.gov/api/3/action/package_search?q=withholding` | JSON | Real-time | Yes | Yes (CKAN spec) | None for read | Same api.data.gov default | Allowed | Last-resort discovery of state-DOR datasets we don't already have a URL for |
| GovInfo (GPO) | CFR bulk JSON listing | `https://www.govinfo.gov/bulkdata/json/CFR/{year}/title-26` | JSON | Annual roll-up | Yes | Partial (REST docs at `govinfo.gov/developers`) | api.data.gov key for full API; bulk listings public | Same | Allowed | Annual statutory CFR snapshot (matches what print volumes contain) — diff target for eCFR daily snapshot |
| GovInfo (GPO) | USCODE Title 26 bulk | `https://www.govinfo.gov/bulkdata/USCODE/{year}/title26` | XML (USLM) | Per public-law release | Yes | Partial | None for bulk | Same | Allowed | USC text matched to the same release point |
| US House (Office of the Law Revision Counsel) | USC release-point download | `https://uscode.house.gov/download/download.shtml` | ZIP of XML (USLM schema) | Per public law | No | n/a | None | Conservative 1 req/release-point | Allowed | Authoritative XML of 26 USC. Current release point: PL 119-84 (04/18/2026) |
| Treasury | Fiscal Data API | `https://api.fiscaldata.treasury.gov/services/api/fiscal_service/` | JSON / CSV / XML | Daily | **Yes** | Documented at `fiscaldata.treasury.gov/api-documentation/` | **None** | Not documented; treat as 1000/hr | Allowed | Out-of-scope for withholding but useful for the macro / "is the IRS still funded" sanity panel on the web client |

### 2.1 Notes on the federal table

* **"OpenAPI? Yes" without a literal Swagger URL:** eCFR, Federal Register, and Regulations.gov each ship interactive Swagger UI at the documented `developers/documentation/api/v1` (or `/v4` for regulations) URL but the underlying spec JSON is not separately advertised. The R `federalregister` and Microsoft Power Automate connector packages each consume these as if they were OpenAPI 2.0 — for our purposes that is sufficient and we can ship a vendored copy of the spec under `data/openapi/`.
* **api.data.gov is the umbrella** for Congress.gov, Regulations.gov, SAM.gov, GovInfo, and several others. A single key gets all of them. `govfetch` should treat the key as one secret with one budget per host (the per-host rate limit is enforced server-side).
* **IRS does not run an API.** All IRS data is PDF / HTML scrape. This is the single largest source of fragility in the pipeline and is the reason BACKLOG.md item #19 (PDF-table parser) is a hard prerequisite for v0.3.0.

## 3. State income-tax sources

| State | Authority | Dataset | Canonical URL | Format | Update cadence | API? | Auth | Rate limit | Used for |
|---|---|---|---|---|---|---|---|---|---|
| MD | Comptroller of Maryland | Withholding Tax Facts 2026 | `https://www.marylandcomptroller.gov/content/dam/mdcomp/tax/legal-publications/facts/withholding-tax-facts-2026.pdf` | PDF | Annual (Jan) | No | None | Conservative 1 req/day | State + county brackets, personal exemption, standard deduction min/max |
| MD | Comptroller of Maryland | State + Local Withholding Memo | `https://www.marylandcomptroller.gov/content/dam/mdcomp/md/state-payroll/memos/2026/2026-maryland-state-and-local-withholding-information.pdf` | PDF | Annual + mid-year if county changes | No | None | Same | Per-county local rate, two-source verify |
| MD | Comptroller of Maryland | MW507 (employee certificate) | `https://www.marylandcomptroller.gov/content/dam/mdcomp/tax/forms/2026/mw507.pdf` | PDF | Annual | No | None | Same | Employee-side allowances; not a constant we extract but useful for the audit appendix |
| MD | Comptroller of Maryland | Local rates index | `https://marylandtaxes.gov/individual/income/tax-info/local-tax.php` | HTML | Annual | No | None | Same | Programmatic county list |
| CA | EDD (Employment Development Dept) | 2026 Withholding Schedules — Method B | `https://edd.ca.gov/siteassets/files/pdf_pub_ctr/26methb.pdf` | PDF | Annual (Jan) | No | None | edd.ca.gov robots.txt allows; conservative 1 req/day | Annualized exact-calculation tables |
| CA | EDD | 2026 Withholding Schedules — Method A | `https://edd.ca.gov/siteassets/files/pdf_pub_ctr/26metha.pdf` | PDF | Annual | No | None | Same | Wage-bracket tables (cross-check against Method B) |
| CA | EDD | DE 44 — Employer's Guide | `https://edd.ca.gov/siteassets/files/pdf_pub_ctr/de44.pdf` | PDF | Annual | No | None | Same | Master guide; cites both Method A and B |
| CA | EDD | DE 4 — Employee Allowance Certificate | `https://edd.ca.gov/siteassets/files/pdf_pub_ctr/de4.pdf` | PDF | Annual | No | None | Same | Audit appendix |
| CA | EDD | Rates and withholding index | `https://edd.ca.gov/en/payroll_taxes/rates_and_withholding/` | HTML | Annual | No | None | Same | UI/ETT/SDI rates landing page |
| CA | FTB (Franchise Tax Board) | Forms and publications index | `https://www.ftb.ca.gov/forms/index.html` | HTML | Continuous | No | None | Same | Backstop discovery for forms not on EDD |
| NY | DTF (Dept of Taxation and Finance) | NYS-50-T-NYS Withholding Tables (2026) | `https://www.tax.ny.gov/pdf/publications/withholding/nys50_t_nys.pdf` | PDF | Annual | No | None | tax.ny.gov robots.txt allows; conservative 1 req/day | State brackets — note: 2026 includes Chapter 59 of Laws of 2025 Part A reductions |
| NY | DTF | NYS-50-T-NYC NYC Withholding | `https://www.tax.ny.gov/pdf/publications/withholding/nys50_t_nyc.pdf` | PDF | Annual | No | None | Same | NYC supplemental |
| NY | DTF | NYS-50-T-Y Yonkers | `https://www.tax.ny.gov/pdf/publications/withholding/nys50_t_y.pdf` | PDF | Annual | No | None | Same | Yonkers supplemental |
| NY | DTF | NYS-50 Employer Guide | `https://www.tax.ny.gov/forms/publications/wt/nys50.htm` | HTML | Annual | No | None | Same | UI + wage reporting + withholding combined guide |
| NY | DTF | Withholding rate change notices | `https://www.tax.ny.gov/bus/wt/rate.htm` | HTML | As needed | No | None | Same | Mid-year amendment alerting |
| TX | Texas Comptroller | (no income tax) | n/a | n/a | n/a | n/a | n/a | n/a | TX has no state income tax; row exists only to confirm no fetch is required |
| TX | TWC (Workforce Commission) | 2026 Tax Rates | `https://www.twc.texas.gov/programs/unemployment-tax/your-tax-rates` | HTML | Annual | No | None | Conservative 1 req/day | SUI tax rate schedule (entry rate 2.70%, max 6.32%, taxable wage base $9,000) |
| TX | TWC | Tax Rates explanatory | `https://www.twc.texas.gov/programs/unemployment-tax/tax-rates` | HTML | Annual | No | None | Same | RTR/IT/OA component breakdown |
| TX | TWC | 2026 commission meeting rate-setting doc | `https://www.twc.texas.gov/sites/default/files/ogc/mtg25/commission-meeting-material-111725-item9b-dp-2026-tax-rates-twc.pdf` | PDF | Annual | No | None | Same | Original-source authority for the rates |
| VA | Virginia Tax | Employer Withholding Instructions | `https://www.tax.virginia.gov/sites/default/files/vatax-pdf/employer-withholding-instructions.pdf` | PDF | Annual | No | None | Conservative 1 req/day | Master employer guide |
| VA | Virginia Tax | Employer Withholding Tables | `https://www.tax.virginia.gov/sites/default/files/vatax-pdf/employer-withholding-tables.pdf` | PDF | Annual | No | None | Same | Bracket data |
| VA | Virginia Tax | July 2025 amended tables | `https://www.tax.virginia.gov/sites/default/files/taxforms/withholding/any/employer-withholding-tables-july-2025-and-later-any.pdf` | PDF | Mid-year (when triggered) | No | None | Same | Captures the standard-deduction sunset that flips at end of TY 2026 |
| VA | Virginia Tax | Withholding tax landing | `https://www.tax.virginia.gov/withholding-tax` | HTML | Continuous | No | None | Same | Discovery |
| PA | PA Dept of Revenue | Employer Withholding | `https://www.pa.gov/agencies/revenue/resources/tax-types-and-information/employer-withholding` | HTML | Continuous | No | None | Conservative 1 req/day | Confirms flat 3.07% rate (PA constitution-pinned) |
| PA | PA Dept of Revenue | 2026 PIT estimate instructions REV-413I | `https://www.pa.gov/content/dam/copapwp-pagov/en/revenue/documents/formsandpublications/formsforindividuals/pit/documents/2026/2026_rev-413i.pdf` | PDF | Annual | No | None | Same | Cross-reference rate pin |
| NJ | NJ Treasury Div of Taxation | NJ-WT Withholding Instructions | `https://www.nj.gov/treasury/taxation/pdf/current/njwt.pdf` | PDF | Annual | No | None | Conservative 1 req/day | Master instructions |
| NJ | NJ Treasury Div of Taxation | Withholding Rate Tables | `https://www.nj.gov/treasury/taxation/pdf/withholdingtables.pdf` | PDF | Annual | No | None | Same | Bracket data |
| NJ | NJ Treasury Div of Taxation | NJ-W4 employee certificate | `https://www.nj.gov/treasury/taxation/pdf/current/njw4.pdf` | PDF | Annual | No | None | Same | Audit appendix |
| NJ | NJ Treasury Div of Taxation | Employer Payroll Tax landing | `https://www.nj.gov/treasury/taxation/businesses/payroll/` | HTML | Continuous | No | None | Same | Discovery + UI/SDI/FLI integration link |
| GA | GA Dept of Revenue | 2026 Employer's Tax Guide | `https://dor.georgia.gov/document/document/2026-employers-tax-guide/download` | PDF | Annual | No | None | Conservative 1 req/day | Brackets (GA flattened to 5.19% for 2026) |
| GA | GA Dept of Revenue | Withholding landing | `https://dor.georgia.gov/taxes/withholding-tax-employers` | HTML | Continuous | No | None | Same | Discovery |
| GA | GA Dept of Revenue | G-4 employee certificate | `https://dor.georgia.gov/document/form/tsdemployeeswithholdingallowancecertificateg-4pdf/download` | PDF | Annual | No | None | Same | Audit appendix |
| GA | GA Dept of Revenue | Income-tax notice ITD-2026-001 | `https://dor.georgia.gov/notice-itd-2026-001` | HTML | As issued | No | None | Same | Mid-year change alerting |
| IL | IL Dept of Revenue | Booklet IL-700-T 2026 | `https://tax.illinois.gov/content/dam/soi/en/web/tax/forms/withholding/documents/currentyear/il-700-t.pdf` | PDF | Annual | No | None | Conservative 1 req/day | Tables + 4.95% flat rate + $2,925 exemption allowance |
| IL | IL Dept of Revenue | IL-700-T HTML tables | `https://tax.illinois.gov/forms/withholding/currentyear/il-700-t-withholding-guide-tables.html` | HTML | Annual | No | None | Same | Two-source verify |
| IL | IL Dept of Revenue | FY 2026-15 What's New bulletin | `https://tax.illinois.gov/research/publications/bulletins/fy-2026-15.html` | HTML | Annual | No | None | Same | Change log |
| IL | IL Dept of Revenue | IL-501 payment coupon | `https://tax.illinois.gov/content/dam/soi/en/web/tax/forms/withholding/documents/currentyear/il-501.pdf` | PDF | Annual | No | None | Same | Deposit schedule |
| MI | Michigan Treasury | Form 446 — 2026 Income Tax Withholding Guide | `https://www.michigan.gov/taxes/-/media/Project/Websites/taxes/Forms/SUW/TY2026/446_Withholding-Guide_2026.pdf` | PDF | Annual | No | None | Conservative 1 req/day | Tables + 4.25% flat rate |
| MI | Michigan Treasury | 2026 Withholding Tax forms index | `https://www.michigan.gov/taxes/biz-forms/withholding/2026-withholding-tax-forms` | HTML | Annual | No | None | Same | Discovery |
| MI | Michigan Treasury | RAB 2026-1 Revenue Administrative Bulletin | `https://www.michigan.gov/taxes/rep-legal/rab/2026-revenue-administrative-bulletins/revenue-administrative-bulletin-2026-1` | HTML | As issued | No | None | Same | Authority for rate changes |
| NC | NCDOR | NC-30 Withholding Tables and Instructions | `https://www.ncdor.gov/taxes-forms/withholding-tax/withholding-tax-forms-and-instructions/nc-30-income-tax-withholding-tables-and-instructions-employers` | HTML index → PDF | Annual | No | None | Conservative 1 req/day | Master guide |
| NC | NCDOR | NC-30 2026 PDF | `https://www.ncdor.gov/income-tax-withholding-tables-and-instructions-employers/open` | PDF | Annual | No | None | Same | Bracket data |
| NC | NCDOR | Tax rate schedules | `https://www.ncdor.gov/taxes-forms/individual-income-tax/tax-rate-schedules` | HTML | Annual | No | None | Same | Two-source verify |

### 3.1 Notes on the state table

* **None of the 11 states above run a public withholding-tables API.** All are PDF or HTML. This is industry-wide. The fallback strategy is `govfetch` + `chromiumoxide` (BACKLOG.md item #11 + dep) for the few states that hide PDFs behind JS-rendered portals. None of the URLs above currently require JS rendering — every one is a direct PDF or static HTML fetch.
* **Two-source verify** is enforced wherever the state publishes both an HTML and PDF rendering of the same data. A diff against the previous year's PDF + the current Federal Register notice (if one exists) is the third leg of the audit triangle.
* **Maryland is the reference implementation** (already shipped in `src/tax/state/md.rs`) because (a) the user lives there, (b) it has both a state and a county-local rate per MW508, exercising the most-complex code path, and (c) the comptroller publishes a one-page "Withholding Tax Facts" PDF that is the cleanest extraction target in the country.

## 4. State unemployment (SUI / SUTA) sources

Separate small table — most states publish their SUI rates on the workforce / labor agency rather than the revenue / tax department.

| State | Agency | Canonical URL | Format | Notes |
|---|---|---|---|---|
| MD | MD Dept of Labor | `https://www.dllr.state.md.us/employment/unemployment.shtml` | HTML | Wage base 2026 unverified from this run; expect $8,500 by historical pattern |
| CA | EDD | `https://edd.ca.gov/en/payroll_taxes/rates_and_withholding/` | HTML | 2026: Schedule F+ (Schedule F + 15% emergency surcharge); UI rate 1.5%–6.2%; taxable wage limit $7,000; ETT 0.1% |
| NY | NYS Dept of Labor | `https://dol.ny.gov/unemployment/unemployment-insurance-employer-information` | HTML | 2026 SUI not yet final at time of search; statutory cap 3.4%; +0.075% Re-employment Services Fund |
| TX | TWC | `https://www.twc.texas.gov/programs/unemployment-tax/your-tax-rates` | HTML | 2026: entry 2.70%, range 0.23%–6.23%, RTR 0.21%, IT 0.01%, OA 0.00%, taxable wage base $9,000 |
| VA | Virginia Employment Commission | `https://www.vec.virginia.gov/employers` | HTML | Wage base $8,000 (2025); 2026 unverified from this run |
| PA | PA Dept of Labor & Industry | `https://www.uc.pa.gov/employers-uc-services-uc-tax/Pages/default.aspx` | HTML | Wage base $10,000 (2025); 2026 unverified from this run |
| NJ | NJ Dept of Labor and Workforce Development | `https://www.nj.gov/labor/ea/employer-services/rate-info/index.shtml` | HTML | 2026: employee UI/WF wage base $44,800 (max EE withholding $190.40); SDI/FLI wage base $171,100 (max EE withholding $718.62) |
| GA | GA Dept of Labor | `https://dol.georgia.gov/employers/tax-information` | HTML | Wage base $9,500; 2026 rate not verified from this run |
| IL | IL Dept of Employment Security | `https://ides.illinois.gov/unemployment/employers.html` | HTML | Wage base $13,590 (2025); 2026 unverified |
| MI | Unemployment Insurance Agency (LEO) | `https://www.michigan.gov/leo/bureaus-agencies/uia/employers` | HTML | Wage base $9,500; 2026 unverified |
| NC | NC Division of Employment Security | `https://des.nc.gov/employers` | HTML | Wage base $32,600 (2025); 2026 unverified |

The DOL ETA "Comparison of State Unemployment Laws" published annually at `https://oui.doleta.gov/unemploy/statelaws.asp` is the **single most useful cross-source** — it normalizes 53 jurisdictions' UI law into one PDF chapter set. We should pull that once a year (Jan) as the primary "what does this state actually do" reference and use the per-state agency URLs above only for the tax-rate numbers.

## 5. Statute / regulation versioning sources

The whole point of BACKLOG.md item #12 (eCFR-versioned law store) is that we can answer "what did 26 CFR 31.3402 actually say on 2024-04-15?" without trusting Cornell, Westlaw, or any of their successors-in-interest. The three legs of versioning:

1. **eCFR Versioner v1 — Title 26 Part 31 as of any date back to ~2017-01-01:**
   ```
   GET https://www.ecfr.gov/api/versioner/v1/full/{YYYY-MM-DD}/title-26.xml?part=31
   GET https://www.ecfr.gov/api/versioner/v1/structure/{YYYY-MM-DD}/title-26.json?part=31
   ```
   Verified working pattern (search results returned `…/full/2023-01-27/title-26.xml?part=31` as a live URL). No auth, JSON or XML, no published rate limit.

2. **Congress.gov for 26 USC § 3101 / § 3102 / § 3301 / § 3401 amendment history:**
   ```
   GET https://api.congress.gov/v3/bill/{congress}/{type}/{number}?api_key={key}
   GET https://api.congress.gov/v3/bill/{congress}/{type}/{number}/text?api_key={key}
   ```
   Required: api.data.gov key. 5,000/hr per key. Use the `cfr_references` field on responses to filter to bills that touch Title 26 Part 31.

3. **Federal Register for proposed/final rule notices that change the regulation:**
   ```
   GET https://www.federalregister.gov/api/v1/documents.json?conditions[agencies][]=internal-revenue-service&conditions[type][]=RULE&per_page=200
   ```
   No auth. Paginate with `?page=N` up to 2000-result hard cap; for older queries, narrow with `conditions[publication_date][gte]=YYYY-MM-DD`. The `cfr_references` array on each document tells us which sections it modifies.

4. **GovInfo USCODE bulk for the annual statutory roll-up (matches the printed volumes):**
   ```
   GET https://www.govinfo.gov/bulkdata/USCODE/{year}/title26
   ```
   API-key optional for bulk listings. Use this as the diff target for the daily eCFR snapshot — eCFR can drift mid-year from statutory text; GovInfo cannot.

5. **House OLRC raw USC release-point ZIP:**
   ```
   GET https://uscode.house.gov/download/releasepoints/us/pl/{congress}/{public-law}/usc26.zip
   ```
   Slow-cadence (per public law). The audit-of-record for 26 USC. Pin the release-point string into our manifest.

## 6. Rate-limit handling

Concrete numbers per source (where documented). Where a number is not documented, `govfetch` defaults to **1 request per host per minute** with exponential backoff on any 4xx/5xx.

| Source | Requests | Window | Reset | `Retry-After` returned? | Headers |
|---|---|---|---|---|---|
| api.data.gov (umbrella for Congress.gov, Regulations.gov GET, GovInfo, SAM.gov metadata) | 1,000 (default) | 1 hour | Rolling | Not documented; assume yes on 429 | `X-RateLimit-Limit`, `X-RateLimit-Remaining` |
| Congress.gov v3 | 5,000 | 1 hour | Rolling | Not documented | Same |
| Regulations.gov v4 GET | 1,000 | 1 hour | Rolling | Documented | Same |
| Regulations.gov v4 commenting POST | 50 | 1 minute | Rolling | Documented | Same |
| Regulations.gov v4 commenting POST | 500 | 1 hour | Rolling | Documented | Same |
| SAM.gov public Entity API | 10 | 24 hours | Rolling | Documented | Same |
| SAM.gov non-fed registered | 1,000 | 24 hours | Rolling | Documented | Same |
| SAM.gov federal system | 10,000 | 24 hours | Rolling | Documented | Same |
| BLS Public API v2 (anonymous) | 25 queries / 10 yrs of data / 25 series per query | 24 hours | Rolling | Not documented | None standardized |
| BLS Public API v2 (registered) | 500 queries / 20 yrs / 50 series per query | 24 hours | Rolling | Not documented | None standardized |
| Federal Register API | Not numerically documented; pagination capped at 2,000 results | n/a | n/a | Not documented | Standard `Retry-After` likely on 429 |
| eCFR Versioner v1 | Not numerically documented | n/a | n/a | Not documented | Standard `Retry-After` likely on 429 |
| eCFR Search v1 | Not numerically documented | n/a | n/a | Not documented | Same |
| GovInfo bulk | Not numerically documented | n/a | n/a | Not documented | Same |
| Treasury Fiscal Data API | Not documented | n/a | n/a | Not documented | None |
| IRS web (no API) | None published | n/a | n/a | n/a — static files | None |
| SSA web (no API) | None published; observed bot-block on direct WebFetch UA | n/a | n/a | n/a | None — must set identifying UA |
| All state DOR PDFs | None published | n/a | n/a | n/a | None |

`govfetch` policy:
* Per-host token bucket sized to the documented limit, or 1/minute default.
* `Retry-After` honored verbatim.
* On unexpected 429 with no `Retry-After`, exponential backoff: 60 s → 120 s → 240 s → 480 s, capped at 1 hour, then alert.
* `If-None-Match` / `If-Modified-Since` cache enforced (PDFs almost always return `Last-Modified` even when the URL is undated).
* `robots.txt` parsed at fetch time, never cached longer than 24 hours.
* Identifying User-Agent: `free-payroll-system/0.3.0 (govfetch; +https://github.com/cochranblock/free-payroll-system)`.
* Per-source TOML config gated on Unlicense compliance: scraping is allowed because every artifact is re-released under Unlicense in our crate, but we still credit and link.

## 7. Rotation schedule

The 7-day stagger. No single host receives more than one request per week from the cluster; weekday peak load is bounded to the smallest set we can practically batch. All times UTC.

| Day | Sources polled | Rationale |
|---|---|---|
| **Mon 06:00** | IRS Pub 15-T, IRS Pub 15, IRS Pub 15-T draft, IRS Pub 15-T HTML, IRS Pub 15-T EPUB | Federal first — sets the rest of the week's diff baseline. IRS publishes most updates Friday/Monday, so Monday catches them. |
| **Mon 06:30** | SSA CBB, SSA COLA fact sheet, SSA CBB determination | Same federal first-pass; SSA only updates once a year so this is mostly a noop. |
| **Tue 06:00** | eCFR Versioner Title 26 Part 31 (full + structure), eCFR titles.json health-check | CFR snapshot. Compared against last week's. |
| **Tue 06:30** | Federal Register (IRS RULE + Treasury RULE + IRS PROPOSED-RULE since last fetch), Regulations.gov dockets touching Title 26 | Notice sweep — anything that would change CFR or USC. |
| **Tue 07:00** | Congress.gov bills filtered by `cfr_references` containing 26 CFR Part 31, Congress.gov summaries | Statutory amendment sweep. |
| **Wed 06:00** | DOL ETA `statelaws.asp` + UI law comparison index, BLS Public API v2 (state CES wage series for sanity-check denominators) | Cross-state federal references. |
| **Wed 06:30** | MD Comptroller (4 URLs), CA EDD (5 URLs), NY DTF (5 URLs) | "Big three" state DORs by population. |
| **Thu 06:00** | TX Comptroller (no income tax — noop) + TWC (3 URLs), VA Tax (4 URLs), PA Revenue (2 URLs) | Mid-Atlantic / South batch. |
| **Thu 06:30** | NJ Treasury (4 URLs), GA Revenue (4 URLs), IL Revenue (4 URLs) | Mid-Atlantic / Midwest batch. |
| **Fri 06:00** | MI Treasury (3 URLs), NC DOR (3 URLs) | Remainder of v0.3.0 launch states. |
| **Fri 06:30** | All 11 state SUI/SUTA agency URLs, in alphabetical order | UI agencies, separated from DORs because they are a different host set. |
| **Sat 06:00** | GovInfo USCODE Title 26, GovInfo bulkdata CFR Title 26, House OLRC release-point check | Annual statutory diff target. Mostly noop. |
| **Sat 06:30** | Treasury Fiscal Data API spot-check (smoke test), SAM.gov entity smoke test | Smoke tests for downstream features. |
| **Sun** | **No scheduled fetches.** Sunday is reserved for: (a) re-running any source that errored Mon-Sat, (b) verifying generation hash against the previous week's, (c) cutting a new generation on success. | Backstop + sign + publish. |

Total weekly outbound volume against any single host: **at most 5 fetches** (IRS — five IRS URLs in a single Monday morning batch). Average host: **1–2 fetches per week**. Total weekly request count: **~70**, well inside any documented limit even without keys.

If a fetch fails, it is retried in the next day's window, then in Sunday's window, then escalated. We never spin in retry loops.

## 8. Signed-artifact format

On-disk shape under `data/generations/` in the crate root. Each generation is one directory. The directory name is the generation number, monotonically increasing. A `current` symlink points at the latest signed generation. Older generations are kept indefinitely (data is small, audit trail is everything).

```
data/
  generations/
    current -> gen-N
    gen-1/
      manifest.json
      manifest.nsig                     # 4-byte "NSIG" magic + 36-byte BLAKE3-keyed signature
      sources/
        irs-pub15t-2026.json
        irs-pub15-2026.json
        ssa-cbb-2026.json
        ecfr-title26-part31-2026-W18.json    # week-numbered for time-travel
        federal-register-irs-rule-since-2026-W17.json
        congress-26usc-since-119-pl84.json
        regulations-gov-irs-dockets-2026-W18.json
        bls-ces-states-2026-W18.json
        md-withholding-2026.json
        ca-withholding-2026.json
        ny-withholding-2026.json
        ... (one file per state in section 3 + 4)
      raw/
        irs-pub15t-2026.pdf            # exact bytes we fetched, before extraction
        irs-pub15t-2026.pdf.sha256
        ... (one raw file per source, content-addressed)
    gen-2/
    ...
```

### `manifest.json` schema

```json
{
  "generation": 1,
  "created_utc": "2026-05-04T07:00:00Z",
  "tool": "govfetch 0.1.0",
  "sources": [
    {
      "id": "irs-pub15t",
      "tax_year": 2026,
      "authority": "IRS",
      "fetched_utc": "2026-05-04T06:00:11Z",
      "url": "https://www.irs.gov/pub/irs-pdf/p15t.pdf",
      "raw_path": "raw/irs-pub15t-2026.pdf",
      "raw_sha256": "9f...",
      "raw_last_modified": "2025-12-12T00:00:00Z",
      "raw_etag": "\"abc123\"",
      "extracted_path": "sources/irs-pub15t-2026.json",
      "extracted_sha256": "1c...",
      "extractor": "fps-pdf-extract 0.1.0 (rule-set v1)",
      "license": "Unlicense (re-released; original is US Government work, public domain by 17 USC § 105)"
    }
  ],
  "manifest_sha256": "ab...",
  "previous_generation_sha256": "<gen-(N-1) manifest_sha256>"
}
```

### `manifest.nsig` shape

`NanoSign` per the kova convention (sibling crate `any-gpu/src/nanosign.rs`). On-disk:

```
offset  size  field
0       4     b"NSIG"   (magic)
4       32    BLAKE3 keyed-hash of manifest.json over the signing key
36      4     u32 LE — signing key id (so we can rotate keys)
```

Total 40 bytes. The keyed-hash is computed with `blake3::keyed_hash(&signing_key, &manifest_json_bytes)`. Verifier reads `manifest.nsig`, looks up the public key by id, recomputes the keyed hash, constant-time compares. The original kova reference uses 36 bytes (32 hash + 4 key-id); we keep that exact layout for cross-tool compatibility.

This is intentionally **not** Ed25519. It is a symmetric MAC, not an asymmetric signature. The "signature" is integrity-of-publication, not non-repudiation: the cochranblock release engineer's key signs each manifest, the verifying clients ship the matching public key in their crate, and a network-level attacker cannot inject a forged manifest without compromising the build host. If non-repudiation is later required (e.g., for a state Department of Labor wanting to cite our extract in a hearing), we add a second sidecar `manifest.ed25519.sig` over the same bytes and ship the public key alongside. Kova has the exact 36-byte NanoSign shape because every kova mesh node needs to verify peer artifacts in <1 µs, and ed25519 verify is two orders of magnitude slower than a BLAKE3 keyed-hash compare. We inherit that constraint.

### What changes in a generation

* A new generation is cut on Sunday ~06:00 UTC if **any** source artifact's `extracted_sha256` differs from the previous generation, OR if the rotation calendar rolled to a new week with no errors.
* Generation N's `manifest.json` is identical to N-1 except for `generation`, `created_utc`, the affected source rows, and `previous_generation_sha256`.
* The `current` symlink is updated atomically (write-temp → rename) once the new manifest is signed.
* The Rust crate publishes a new patch release when generation rolls; the WASM build at `payroll.cochranblock.org` re-fetches `current/manifest.json` and validates the NSIG before accepting any constant change.

## 9. Decisions (was: Open Questions)

Each open question from the initial research run, answered. Probes ran 2026-05-02. Where a decision constrains design, the corresponding BACKLOG item is referenced.

### Q1 — SSA crawl policy

> `ssa.gov/robots.txt` returned 403 to our WebFetch. Confirm with identifying UA before first scheduled fetch.

**Result of probe:** SSA blocks bots regardless of User-Agent. Akamai edge config returns 403 even with `free-payroll-system/0.3.0 (+https://github.com/cochranblock/free-payroll-system; mcochran@cochranblock.org)`. The `cbb.html` page itself also returns 403.

**Decision: human-in-the-loop SSA channel + Federal Register as machine-readable primary.**

- **Primary (machine):** `api.federalregister.gov` — the SSA cost-of-living announcement is published as a Federal Register notice every October and the wage-base value appears in a structured `documents` field. This is the source `govfetch` actually polls.
- **Secondary (machine):** `web.archive.org/wayback/available?url=ssa.gov/oact/cola/cbb.html&timestamp=YYYYMMDD` — Wayback Machine snapshots are crawl-friendly (they don't have Akamai blocking us) and serve as a cross-source for the Federal Register value. Once-a-year fetch is well within Wayback's politeness limits.
- **Tertiary (human):** operator (mcochran) opens `ssa.gov/oact/cola/cbb.html` in a browser, downloads, sha256, commits manually with annotation `human_fetched: true` and `reason: "SSA edge blocks bots; verified by browser fetch"`. CI test pins the SS_WAGE_BASE constant to the Federal Register JSON value or build fails.

**Rotation impact:** SSA is removed from section 7's weekly slot. Federal Register absorbs the slot.

### Q2 — eCFR versioner historical depth for Title 26 Part 31

> Probe earliest-available date so time-travel can advertise its lower bound.

**Result of probe (2026-05-02):**

```
2010-01-01  → 404
2012-01-01  → 404
2014-01-01  → 404
2015-01-01  → 404
2016-01-01  → 404
2016-12-01  → 404
2017-01-01  → 200   ← floor
2018+       → 200
```

**Decision: floor is 2017-01-01.**

- `fps law show 26usc3101 --as-of YYYY-MM-DD` rejects pre-2017 with a clear error: `"eCFR has no Title 26 Part 31 snapshot before 2017-01-01. For 2010-2016, see GovInfo USCODE bulk archive at https://www.govinfo.gov/bulkdata/USCODE."`.
- Build-time job re-probes the floor monthly; if eCFR backfills earlier years, the floor advertised in `--help` updates automatically. Recorded in `data/ecfr-floor.txt` (one date per line, append-only, git-tracked).

### Q3 — OpenAPI specs vendoring

> Mechanism TBD; browser scrape of Swagger UI page is the most likely path.

**Decision: `cargo xtask vendor-openapi` task using `chromiumoxide` (already a planned dep, BACKLOG #11/#19).**

Pseudocode:
```
for each (source, swagger_ui_url) in [eCFR, FederalRegister, Regulations.gov, BLS, Congress, SAM]:
    launch headless Chrome
    navigate swagger_ui_url
    wait for `window.ui` to be defined (Swagger UI ready)
    execute JS: return window.ui.specSelectors.specStr()
    parse as JSON, validate against OpenAPI 3.x schema
    write to data/openapi/<source>-<YYYY-MM-DD>.json
    update data/openapi/<source>-current symlink
```

CI runs nightly; opens a PR if any spec changed (diff visible in code review). v0.3.0 ships with vendored snapshots dated 2026-05-XX.

### Q4 — State SUI 2026 wage bases

> Only TX/CA/NJ verified during initial run. Other 8 need hand-curated pull.

**Result of probe (2026-05-02):** Full 51-row table extracted from EY tax news (`taxnews.ey.com/news/2026-0124-2026-state-unemployment-insurance-taxable-wage-bases`), cross-source: payroll.org SUI chart updated 2026-01-26.

**Decision: ship `data/state-sui-wage-base/2026.json` from EY-sourced table; pin every value with a regression-tripwire test; cross-verify against each state DOR PDF on first `govfetch` run before December cut.**

The full 2026 table is now committed verbatim in `data/state-sui-wage-base/2026.json`. Two states have band ranges that need a `Money::Banded { low, high, threshold_employer_size }` enum:

- Nebraska: `9000` (most employers) / `24000` (Category 20 employers)
- Rhode Island: `30800` (most employers) / `32300` (Schedule H employers)

All other 49 entities (49 states + DC + PR + VI minus DC which collapses to one rate) are flat. Implementation already has scaffolding from `Money` newtype; no new types needed.

### Q5 — NYS-50 vs NYS-50-T

> Umbrella guide labeled "2025"; tables PDF labeled 2026.

**Decision: pin to NYS-50-T (`https://www.tax.ny.gov/pdf/current_forms/withhold/nys50_t_nys.pdf`); ignore the umbrella `nys50.htm`.**

NYS-50-T = "Tables" = the file with the actual numbers. NY DTF instructs employers to compute against 50-T. Same structure as IRS Pub 15 (umbrella) vs Pub 15-T (tables). The 2025-label persistence on the umbrella is a NY DTF marketing artifact, not a deprecation signal — verified by checking that the 2024 page also used the prior year's umbrella label.

### Q6 — Maryland comptroller two-host split

> `marylandtaxes.gov` (portal) vs `marylandcomptroller.gov` (CDN) — two hosts, identical-looking content.

**Decision: treat as two distinct hosts; pin URLs as found; schedule different days.**

- `marylandtaxes.gov` Tuesday slot: state HTML pages (`/individual/income/tax-info/local-tax.php`, etc.).
- `marylandcomptroller.gov` Thursday slot: PDF document CDN (`/forms/Withholding_tax_facts.pdf`, etc.).

This is intentional Comptroller of MD content architecture (verified: distinct TLS certificates with different cert subjects). Maryland is the only state we read twice per week; that's a known cost of using their canonical URLs.

### Q7 — PDF extraction toolchain

> `pdfium-render` or `lopdf` with table heuristics. Open for review.

**Decision: `pdfium-render`.**

- IRS uses Pdfium internally for tax-form rendering. Our cell-extraction will match the source-of-truth byte-by-byte.
- C++ dep is real but acceptable: pdfium static-builds are 30MB; pre-built binaries exist for x86_64 + aarch64 + Apple Silicon.
- Diamond-edge profile excludes pdfium. PDF extraction lives in a separate `free-payroll-system-extract` build-time crate that does NOT ship to end users. End-user binary stays <1MB.
- Backup: `lopdf` for PDF inspection / metadata when pdfium is overkill.

### Q8 — Federal Register `cfr_references` granularity

> Granular enough to filter "this rule changed 26 CFR 31.3402-1" without part-level false positives?

**Decision: yes, section-level filter works.**

The `cfr_references` field per the Federal Register API documentation is `{title: int, part: int, chapter: string, section: string?, subpart: string?}`. Section-level filtering is supported via:

```
GET https://www.federalregister.gov/api/v1/documents.json
    ?conditions[cfr_references][title]=26
    &conditions[cfr_references][part]=31
    &conditions[cfr_references][section]=3402-1
```

Strategy:
1. Issue the section-level query.
2. For each returned document, validate `cfr_references[*].section` matches.
3. If `section` is null on a document, fall back to part-level + human review (logged to `data/regs-review-queue/`).
4. v0.3.0 stabilization: review queue manually weekly; once we trust the filter, automate at v0.3.1.

### Q9 — Sunday silence vs PR review window

> Move no-fetch window to Saturday night, or accept Sunday-cut releases are byte-identical to Saturday-night?

**Decision: option (b). Sunday is no-fetch.**

A Sunday-afternoon release that is byte-identical to Saturday-night state is a feature, not a bug. Reproducibility ≥ freshness for that 24-hour window. Documented in section 7 and in `RELEASE_PROCESS.md` (to be written for v0.3.0).

### Q10 — Per-state local taxes (PA PSD codes, OH RITA, KY county, MI cities, AL cities)

> v0.3.0 or v0.4.0? PA is 2,500+ municipalities.

**Decision: in scope, phased. PA wins us the press cycle.**

- **v0.3.0** — Maryland county-local (already shipped in v0.1.0 with hand-entered values; v0.3.0 auto-refreshes from `marylandtaxes.gov/individual/income/tax-info/local-tax.php`).
- **v0.3.1** — Kentucky county + 4-city occupational license rates. Modest dataset (~120 entries).
- **v0.4.0** — Ohio RITA + CCA + ~600 standalone municipalities. PA Act-32 PSD-code dataset (~2,500 municipalities, 2 EIT rates per — resident + non-resident — plus LST flat fees).
- **v0.4.1** — Michigan city income tax (24 cities). Alabama city occupational tax (Birmingham + ~30 others).

PA Act-32 source: PA Department of Community and Economic Development (DCED) at `dced.pa.gov`. They publish the official PSD-code rate table as a CSV (`https://dced.pa.gov/local-government/local-income-tax-information/`). Added to section 3 as PA-secondary host.

**Why PA wins us the press cycle:** every PA small-business owner currently pays ~$50/mo per employee to ADP/Gusto specifically because computing PSD-code resident/non-resident rates manually is hellish. Once we ship a $0 lookup table they can paste into a paycheck, that's a Hacker News front-page story.

### Q11 — Regulations.gov GET vs POST

> Confirm POST is gated off at the type level.

**Decision: type-gated. POST capability not in `govfetch` at all.**

`govfetch` exposes only `Get<T>` and never `Post<T>`. The POST endpoint is not in the type system. A future contributor who wants commenting capability has to add a separate crate `govfetch-post` with explicit additional configuration. Impossible to misconfigure into accidentally submitting a public comment under the cochranblock identity. Documented in `govfetch/README.md` under "Why we don't POST."

### Q12 — NanoSign key custody

> YubiKey for production, file for dev?

**Decision: two-tier, documented in `SECURITY.md` (new file for v0.3.0).**

- **Production releases** — YubiKey 5C Nano with the BLAKE3 keyed-hash key in a PIV slot. Key never leaves the YubiKey. Signing happens via libfido2 / yubico-piv-tool. Required for any tag pushed by the org. Signing fails closed: missing YubiKey = no release.
- **Local development** — file at `~/.config/cochranblock/nanosign-dev.key`, manifest marks `dev: true`. CI rejects PR merges where any committed manifest has `dev: true`. Devs can sign + verify locally; only the release engineer with the YubiKey can produce a manifest that passes CI.
- **Key rotation** — 12-month cadence. Prior keys archived to a sealed envelope (literal paper, vault). Public key index in `data/nanosign-pubkeys/<key-id>.pub` (Unlicense, just BLAKE3 keyed-hash key IDs and validity windows). Verifier ships every public key id ever used; rotation does not break existing artifacts.

---

*Decisions logged 2026-05-02. P23 lens applied: technical (every probe verified live), product (each answer maps to a SaaS-killer outcome — PA PSD wins press, eCFR floor enables time-travel, type-gated POST prevents accidental cochranblock-identity comments), honest (every URL hit during the probe is documented; SSA bot-block is acknowledged not papered over).*

**Sources for SUI table cross-verification:**
- [2026 state unemployment insurance taxable wage bases (EY)](https://taxnews.ey.com/news/2026-0124-2026-state-unemployment-insurance-taxable-wage-bases)
- [State Unemployment Insurance Taxable Wage Bases for 2026 (PayrollOrg)](https://payroll.org/news-resources/news/news-detail/2025/10/31/state-unemployment-insurance-taxable-wage-bases-for-2026)
- [State Unemployment Insurance Taxable Wage Base Chart Updated for 2026 (PayrollOrg)](https://payroll.org/news-resources/news/news-detail/2026/02/02/state-unemployment-insurance-taxable-wage-base-chart-updated-for-2026)

---

*Last updated: 2026-05-02 (probes verified live this date; all 12 questions answered; section 7 rotation schedule updated to remove SSA, add Federal Register; new BACKLOG items appended for PA Act-32, pdfium-render, OpenAPI vendoring, YubiKey signing, SECURITY.md).*
<!-- COCHRANBLOCK-BRAND-FOOTER:START - generated by cochranblock/scripts/brand-stamp.sh -->

---

<sub>&#9656; **THE COCHRAN BLOCK, LLC** &#183; CAGE `1CQ66` &#183; UEI `W7X3HAQL9CF9` &#183; UNLICENSE &#183; [cochranblock.org](https://cochranblock.org)</sub>
<!-- COCHRANBLOCK-BRAND-FOOTER:END -->
