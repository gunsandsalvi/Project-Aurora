//! # `tools` — the checks that hold the rules the compiler cannot
//!
//! Not a layer (ADR-0003). Each subcommand prints its rule inventory and an exemption count, and the
//! exemption count must read zero: §17's empty exemption list is only a claim if something counts it.

use std::process::ExitCode;

use aurora_tools::{check_lints, check_surface};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        eprintln!("usage: aurora-tools <check-lints|check-surface>");
        return ExitCode::FAILURE;
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tools crate always has a workspace parent")
        .to_path_buf();
    match cmd.as_str() {
        "check-lints" => check_lints::run(&root),
        "check-surface" => check_surface::run(&root),
        other => {
            eprintln!("unknown check: {other}");
            ExitCode::FAILURE
        }
    }
}
