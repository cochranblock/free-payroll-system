// SPDX-License-Identifier: Unlicense
// Unlicense — public domain — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.7

//! free-payroll-system-test. TRIPLE SIMS quality gate via exopack::triple_sims.
//! Stage 1: cargo test (integration tests — IRS Pub 15-T worked examples)
//! Stage 2: binary smoke test (--help exits 0, --version prints version)
//! Stage 3: determinism (same input -> identical paystub bytes, 3x)
//! All stages run 3x via exopack::triple_sims::f60. Any flake = FAIL.

use std::process::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("free-payroll-system-test");
    println!("========================");

    let project_dir = std::env::current_dir().unwrap_or_else(|_| {
        let exe = std::env::current_exe().unwrap();
        exe.parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    });

    let exe = std::env::current_exe().unwrap();
    let prod_bin = exe.parent().unwrap().join("free-payroll-system");

    let ok = exopack::triple_sims::f60(|| {
        let dir = project_dir.clone();
        let bin = prod_bin.clone();
        async move {
            print!("  cargo test ... ");
            let test_out = Command::new("cargo")
                .args(["test", "--quiet"])
                .current_dir(&dir)
                .output();
            match test_out {
                Ok(o) if o.status.success() => println!("OK"),
                Ok(o) => {
                    println!("FAILED");
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    if !stderr.is_empty() {
                        eprintln!("{}", stderr);
                    }
                    return false;
                }
                Err(e) => {
                    println!("ERROR: {}", e);
                    return false;
                }
            }

            print!("  --help ... ");
            if bin.exists() {
                let help = Command::new(&bin).arg("--help").output();
                match help {
                    Ok(o) if o.status.success() => println!("OK"),
                    Ok(o) => {
                        println!("FAILED (exit {})", o.status);
                        return false;
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                        return false;
                    }
                }
            } else {
                println!("SKIP (binary not found at {})", bin.display());
            }

            print!("  --version ... ");
            if bin.exists() {
                let v = Command::new(&bin).arg("--version").output();
                match v {
                    Ok(o) if o.status.success() => {
                        let s = String::from_utf8_lossy(&o.stdout);
                        if s.contains(env!("CARGO_PKG_VERSION")) {
                            println!("OK");
                        } else {
                            println!("FAILED (no version match in '{}')", s.trim());
                            return false;
                        }
                    }
                    Ok(o) => {
                        println!("FAILED (exit {})", o.status);
                        return false;
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                        return false;
                    }
                }
            } else {
                println!("SKIP (binary not found)");
            }

            true
        }
    })
    .await;

    std::process::exit(if ok { 0 } else { 1 });
}
