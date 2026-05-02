// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

use free_payroll_system::version;

#[test]
fn version_matches_cargo_pkg_version() {
    assert_eq!(version(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn version_is_not_empty() {
    assert!(!version().is_empty());
}
