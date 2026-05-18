<!-- Unlicense — public domain — cochranblock.org -->
<!-- Contributors: GotEmCoach, KOVA, Claude Opus 4.7 -->

# Timeline of Invention — free-payroll-system

*Dated, commit-level record of what was built, when, and why. Proves human-piloted AI development — not generated spaghetti.*

> Every entry below maps to real commits. Run `git log --oneline` to verify.

## How to Read This Document

Each entry follows this format:

- **Date**: When the work shipped (not when it was started)
- **What**: Concrete deliverable — binary, feature, fix, architecture change
- **Why**: Business or technical reason driving the decision
- **Commit**: Short hash(es) for traceability
- **AI Role**: What the AI did vs. what the human directed
- **Proof**: Link to artifact, screenshot, test output, or live URL

This document exists because AI-assisted code has a trust problem. Anyone can generate 10,000 lines of spaghetti. This timeline proves that a human pilot directed every decision, verified every output, and shipped working software.

---

## Human Revelations — Invented Techniques

*Novel ideas that came from human insight, not AI suggestion. These are original contributions to the field.*

### Crate-Reservation-as-Public-Callout (2026-05-01)

**Invention:** Reserve a crate name on crates.io live during a public argument with the incumbent vendor's salesperson, screenshot it, and use the reservation as a commitment device.

**The Problem:** Open-source vaporware claims are cheap on LinkedIn. "I'll build a free version of X" is 10x more credible when there's a public artifact you can't take back.

**The Insight:** `cargo publish` of a v0.0.0 stub is irreversible (yank ≠ delete). Doing it WHILE the salesperson is mocking you ("I can't wait for your system to f up all the taxes") makes the receipt timestamped, immutable, and embarrassing for the salesperson if you actually ship.

**The Technique:** While Phil P. ("Payroll Rebel | Sales GOAT") was escalating in a public LinkedIn comment thread on a post calling out the SaaS payroll cabal, the user reserved `free-payroll-system` v0.0.0 on crates.io at 16:01 EDT and posted the screenshot in the same thread. The reservation timestamp + Phil's "Aww yes crate name reserved baby! I'm surprised you haven't blocked me yet" reply are both archived in `RECEIPTS.md`.

**Result:** Project's existence is bound to a public commitment. Every commit from this point forward is part of the receipt.

**Named:** Crate-Reservation-as-Public-Callout
**Commit:** crates.io v0.0.0, 2026-05-01T20:01:17Z
**Origin:** Real LinkedIn argument. The user spent 6 hours that day responding to Phil and Channing Tipton ("Are you mentally stable or have you gone through shock therapy??"); the crate reservation made it impossible to walk away clean.

### Tax-Year-Stamped Stderr Warning (2026-05-01)

**Invention:** Every paystub invocation prints a stderr-only warning naming the IRS Pub 15-T tax year baked into this build.

**The Problem:** Hand-entered tax tables age. A user who installs free-payroll-system in 2028 and computes paychecks against 2024 tables ends up under-withholding and owing the IRS at filing time. Worse, they didn't know.

**The Insight:** Honest software acknowledges what it doesn't know. The warning is on stderr (so JSON consumers piping `stdout` to `jq` are unaffected), prints AFTER the paystub on success (so it's the LAST thing the user sees), and never prints on errors (the user has bigger problems).

**The Technique:** `eprintln!("warning: this build computes against IRS Pub 15-T tax year {TAX_YEAR}. Verify against current Pub 15-T before live use.")` printed after a successful paystub. `TAX_YEAR` constant in `src/tax/federal.rs` is the source of truth and is referenced in `data/citations.md` for audit.

**Result:** A user who hits this warning has been told, on every single invocation, what tax year their paystub came from. v0.3.0 will replace the warning with a `govfetch`-fetched current-year table on demand.

**Named:** Tax-Year-Stamped Stderr Warning
**Commit:** `050daea`
**Origin:** Mid-flight UI/UX analysis surfaced that "v0.3.0 will fetch automatically" was forward-looking marketing text, not a useful warning. The warning was rewritten to be present-tense and action-oriented.

---

## Entries

*Reverse chronological. Most recent first.*

### 2026-05-02 — Audit-Bullshit Pass + Provenance Docs

**What:** (1) Audited every test for tautology — reframed three constant-pinning tests as explicit "regression tripwires" (with citation comments); (2) Added a county-rate sanity-bounds test that catches any Maryland county rate drifting outside the legal `[100bps, 500bps]` range; (3) Added MD bracket monotonicity property test (higher gross ⇒ higher withholding for arbitrary inputs); (4) Added MD progressive-rate property test (effective rate at $200k > effective rate at $30k); (5) Added federal monotonicity test across all four pay frequencies × all three filing statuses; (6) Added federal zero-gross zero-tax test across the same matrix; (7) Added federal cross-frequency consistency test with percentage-of-target tolerance bound (catches algorithmic bugs without flapping on legitimate sub-cent rounding); (8) Added FICA-rates-match-statute test pinned to 26 USC § 3101 with citation; (9) Added FICA SS annual-cap ceiling test ($10,453.20 max for 2024); (10) Added `tests/audit.rs` with `audit_self_balances` cross-status/cross-frequency balance test plus a documented contributor template for adding externally-verified fixtures from Pub 15-T worked examples or the IRS Tax Withholding Estimator. Wrote `TIMELINE_OF_INVENTION.md`, `PROOF_OF_ARTIFACTS.md`, and `BACKLOG.md` per CochranBlock canonical templates. Total tests: 51 (was 42 at v0.1.0 foundation commit).
**Why:** User explicitly called out: "make sure those tests ain't bullshit." Pre-publish audit pass. Tautological tests provide false confidence — they pass even if the bracket constants are wrong. Property tests + invariants + statute citations + cross-validation scaffold replace that false confidence with a real one.
**Commit:** `pending`
**AI Role:** AI re-read every test, identified tautologies, wrote replacements + new property tests + the audit fixture scaffold. Human directed the audit (specifically calling tests "bullshit"), the standard for what's NOT bullshit (statute citations + invariants + monotonicity + external-source fixtures), and the pre-publish gate (no publish until audit lands + provenance docs ship).
**Proof:** `cargo test --features tests` — 51 tests pass. `cargo run --features tests --bin free-payroll-system-test` — TRIPLE SIMS 3/3 pass. Every numeric constant in `src/tax/federal.rs` and `src/tax/state/md.rs` cites its source publication URL. `tests/federal_pub15t.rs::fica_constants_match_statute` pins SS/Medicare/Additional Medicare/FUTA rates to their 26 USC § 3101 / § 3301 statutory values.

### 2026-05-01 — v0.1.0 Foundation: Federal + Maryland Paystub

**What:** Federal income tax withholding (IRS Pub 15-T 2024 Worksheet 1A Annualized Wage Method, Single + MFJ + HoH), FICA (Social Security 6.2% w/ $168,600 wage base, Medicare 1.45%, Additional Medicare 0.9% over $200k), FUTA constants (employer-side). Maryland state withholding with progressive brackets (Single + MFJ) + 24-county local rate table + nonresident rate. `Money(i64 cents)` newtype with banker's-rounding `mul_bps`, custom `Display`, JSON serialization. `Employee` → `Paycheck` pure function. clap-based CLI with `paystub --employee path.toml --json`, `tax-year` subcommand. Negative-input rejection on gross / YTD wages. Stderr tax-year warning on every paystub invocation. README "Verify the math in 60 seconds" section. `data/citations.md` with every numeric constant cited to its source publication. `examples/employee.toml` starter config. `CONTRIBUTING.md` with a 5-step recipe for adding a state. GitHub Actions CI: build + test + TRIPLE SIMS + paystub smoke + fmt + clippy `-D warnings` + `--profile=diamond` LTO build (5 jobs). 42 tests at this commit.
**Why:** Phil P. and Channing Tipton needed receipts. v0.0.0 was a name-only stub; v0.1.0 is the math. Federal + Maryland because that's the user's home state (Baltimore County, MSDE/BCPS context).
**Commit:** [`050daea`](https://github.com/cochranblock/free-payroll-system/commit/050daea)
**AI Role:** AI wrote all source files, derived crate conventions from sibling Cochran Block crates (exopack, illbethejudgeofthat, deglaze, any-gpu), and proposed UI/UX deltas after a mid-flight user-story analysis. Human directed: standalone-not-kova-workspace, federal+MD scope for v0.1.0, biweekly-only, no-pre-tax-deductions-yet (defer to v0.2.0), "build a good foundation first" (don't rush publish), and approved each phase before next.
**Proof:** GitHub Actions run [25243096326](https://github.com/cochranblock/free-payroll-system/actions/runs/25243096326) — 5/5 jobs green in <1 min. `cargo publish --dry-run` ✅ (18 files, 20KiB compressed).

### 2026-05-01 — exopack TRIPLE SIMS Gate

**What:** Added `[features] tests = ["exopack", "tokio"]` to `Cargo.toml`. Created `src/bin/free-payroll-system-test.rs` mirroring the `illbethejudgeofthat` pattern: runs `cargo test` + `--help` + `--version` 3x via `exopack::triple_sims::f60`. First integration test `tests/version.rs` (`version()` matches `CARGO_PKG_VERSION`). Compiles clean against published `exopack v0.2.1` from crates.io.
**Why:** CLAUDE.md mandates the TRIPLE SIMS gate before marking work done. Wired immediately so every subsequent change runs through it.
**Commit:** [`1b01b02`](https://github.com/cochranblock/free-payroll-system/commit/1b01b02)
**AI Role:** AI mirrored the illbethejudgeofthat/src/bin/illbethejudgeofthat-test.rs pattern, swapping the project name. Human directed: use exopack, mirror the sibling pattern.
**Proof:** `cargo run --features tests --bin free-payroll-system-test` — TRIPLE SIMS 3/3 pass.

### 2026-05-01 — Initial Commit + GitHub Repo + Receipts

**What:** Created `cochranblock/free-payroll-system` GitHub repo via REST API (PAT pulled cross-node from Mac Mini's `~/.config/gh/hosts.yml` and persisted to this node's `~/.secrets/.env` as `GITHUB_TOKEN` for future use). First commit: `Cargo.toml`, `README.md`, `RECEIPTS.md` (full LinkedIn timeline + 33 screenshot manifest, 14:47–20:57 EDT 2026-05-01), `UNLICENSE`, `src/lib.rs`, `src/main.rs`, `.gitignore`. Author configured to `Michael Cochran <mcochran@cochranblock.org>`.
**Why:** crates.io v0.0.0 reservation was already public; the GitHub repo at the URL listed in Cargo.toml's `repository` field needed to exist or the listing was a 404 dead-link.
**Commit:** [`f1b83c7`](https://github.com/cochranblock/free-payroll-system/commit/f1b83c7)
**AI Role:** AI located the missing PAT (none in any local .env on n2/bt), pulled it from the Mac via SSH, persisted it, hit the GitHub API to create the repo, configured git author/email per project memory rule, drafted the LinkedIn-origin first-commit message, and pushed. Human directed: pull the token over (`mclarkfyrue@gmail.com` is private, `mcochran@cochranblock.org` is the project email).
**Proof:** Repo at https://github.com/cochranblock/free-payroll-system. `RECEIPTS.md` lists 33 screenshot files with timestamps.

---

*Part of the [CochranBlock](https://cochranblock.org) zero-cloud architecture. All source under the Unlicense.*
<!-- COCHRANBLOCK-BRAND-FOOTER:START - generated by cochranblock/scripts/brand-stamp.sh -->

---

<sub>&#9656; **THE COCHRAN BLOCK, LLC** &#183; CAGE `1CQ66` &#183; UEI `W7X3HAQL9CF9` &#183; UNLICENSE &#183; [cochranblock.org](https://cochranblock.org)</sub>
<!-- COCHRANBLOCK-BRAND-FOOTER:END -->
