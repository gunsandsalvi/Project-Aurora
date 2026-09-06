//! §17: **`domain`, `world` and `ledger` carry zero third-party dependencies.**
//!
//! Asserted over the **resolved** graph rather than over the manifest, because a manifest says what a
//! crate asked for and the lockfile says what it got. A crate with no direct dependency can still be
//! compiled against one through a workspace feature or a transitive re-export; only the resolved
//! graph knows.
//!
//! Also here, and for the same reason a lockfile is the subject: **no package appears at two
//! versions.** Two versions of one crate is two representations of one thing (§4.3), and in a
//! deterministic engine it is worse than untidy — the two can differ in float behaviour.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

/// The three crates §17 names, whose dependency sets must be empty of anything outside the workspace.
const PURE: [&str; 3] = ["aurora-domain", "aurora-world", "aurora-ledger"];

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, packages, checked) = check(root);
    println!("check-deps");
    println!(
        "  rule: §17 — domain, world and ledger resolve to zero third-party dependencies   crates checked: {checked}"
    );
    println!("  rule: no package resolves to two versions   packages in the lockfile: {packages}");
    println!("  exemptions: 0");
    if findings.is_empty() {
        println!("  violations: 0");
        return ExitCode::SUCCESS;
    }
    println!("  violations: {}", findings.len());
    for f in &findings {
        println!("    {f}");
    }
    ExitCode::FAILURE
}

/// The findings, the lockfile's package count, and how many of the three pure crates were found.
pub fn check(root: &Path) -> (Vec<String>, usize, usize) {
    let mut findings = Vec::new();
    let lock_path = root.join("Cargo.lock");
    let Ok(text) = std::fs::read_to_string(&lock_path) else {
        return (vec![format!("cannot read {}", lock_path.display())], 0, 0);
    };
    let Ok(lock) = text.parse::<toml::Table>() else {
        return (
            vec![format!("{} does not parse", lock_path.display())],
            0,
            0,
        );
    };
    let Some(packages) = lock.get("package").and_then(toml::Value::as_array) else {
        return (
            vec!["the lockfile has no [[package]] array".to_owned()],
            0,
            0,
        );
    };

    // A package is "ours" when it is a workspace member; anything else is third-party.
    let mut versions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut checked = 0usize;
    for pkg in packages {
        let (Some(name), Some(version)) = (
            pkg.get("name").and_then(toml::Value::as_str),
            pkg.get("version").and_then(toml::Value::as_str),
        ) else {
            continue;
        };
        versions
            .entry(name.to_owned())
            .or_default()
            .push(version.to_owned());

        if PURE.contains(&name) {
            checked += 1;
            let deps = pkg.get("dependencies").and_then(toml::Value::as_array);
            for dep in deps.into_iter().flatten() {
                let Some(dep_name) = dep.as_str() else {
                    continue;
                };
                let bare = dep_name.split_whitespace().next().unwrap_or(dep_name);
                if !bare.starts_with("aurora-") {
                    findings.push(format!(
                        "{name} depends on `{bare}`, which is outside the workspace (§17)"
                    ));
                }
            }
        }
    }

    for (name, mut vs) in versions {
        vs.sort();
        vs.dedup();
        if vs.len() > 1 {
            findings.push(format!(
                "`{name}` resolves to {} versions ({}) — two representations of one thing (§4.3)",
                vs.len(),
                vs.join(", ")
            ));
        }
    }

    if checked != PURE.len() {
        findings.push(format!(
            "expected {} pure crates in the lockfile, found {checked}",
            PURE.len()
        ));
    }

    (findings, packages.len(), checked)
}
