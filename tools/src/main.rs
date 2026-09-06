//! # `tools` — the checks that hold the rules the compiler cannot
//!
//! Not a layer (ADR-0003). Each subcommand prints its rule inventory and an exemption count, and the
//! exemption count must read zero: §17's empty exemption list is only a claim if something counts it.

use std::process::ExitCode;

use aurora_tools::{check_adr, check_deps, check_lints, check_refs, check_registry, check_surface};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        eprintln!(
            "usage: aurora-tools <verify|check-lints|check-surface|check-deps|check-refs|check-adr|check-registry>"
        );
        return ExitCode::FAILURE;
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tools crate always has a workspace parent")
        .to_path_buf();
    match cmd.as_str() {
        "check-lints" => check_lints::run(&root),
        "check-surface" => check_surface::run(&root),
        "check-deps" => check_deps::run(&root),
        "check-refs" => check_refs::run(&root),
        "check-adr" => check_adr::run(&root),
        "check-registry" => check_registry::run(&root),
        "verify" => verify(&root),
        other => {
            eprintln!("unknown check: {other}");
            ExitCode::FAILURE
        }
    }
}

/// Every check, in one command, reporting how many ran and how many found something.
///
/// §17's empty exemption list is only a claim if something counts it, so each check prints its rule
/// inventory and its exemption count, and this prints the roll-up. A check that is quietly skipped
/// is the failure mode this exists to prevent.
fn verify(root: &std::path::Path) -> ExitCode {
    /// A check: its name, and the function that runs it and reports.
    type Check = (&'static str, fn(&std::path::Path) -> ExitCode);

    let checks: [Check; 6] = [
        ("check-lints", check_lints::run),
        ("check-surface", check_surface::run),
        ("check-deps", check_deps::run),
        ("check-refs", check_refs::run),
        ("check-adr", check_adr::run),
        ("check-registry", check_registry::run),
    ];
    let mut failed = Vec::new();
    for (name, run) in checks {
        if run(root) != ExitCode::SUCCESS {
            failed.push(name);
        }
        println!();
    }
    println!(
        "verify: {} checks ran, {} failed",
        checks.len(),
        failed.len()
    );
    if failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        for name in &failed {
            println!("  {name}");
        }
        ExitCode::FAILURE
    }
}
