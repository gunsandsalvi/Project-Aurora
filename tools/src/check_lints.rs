//! Every crate in the layer graph carries `#![forbid(unsafe_code)]`, and the only permitted
//! `allow` of `unsafe_code` in the tree is the named arena seam (§17, ADR-0004).
//!
//! **This check tokenises rather than greps, and it learned that the hard way**: the first draft
//! substring-matched, and its first run reported itself, because the needle it searched for appeared
//! in its own source. A grep cannot tell an attribute from a string literal that spells one. The
//! lexer can, it costs nothing, and it is the same rule `check_surface` is built on.

use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The one place `unsafe` is permitted, and the line cap its safety argument must fit inside (ADR-0004).
const ARENA_SEAM: &str = "crates/kernel/src/arena_seam.rs";
const SEAM_LINE_CAP: usize = 60;

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (failures, crates_checked, files, seams) = check(root);
    println!("check-lints");
    println!(
        "  rule: every layer crate carries #![forbid(unsafe_code)]   crates scanned: {crates_checked}"
    );
    println!(
        "  rule: the only allow of unsafe_code is the declared arena seam   files scanned: {files}, seams: {seams}"
    );
    println!("  rule: the release profile keeps overflow checks on (Appendix A #2, ADR-0006)");
    println!(
        "  rule: no shared-mutability primitive in a layer crate — the arena is thread-shareable (ADR-0012)"
    );
    println!("  exemptions: 0");
    report(&failures)
}

/// The findings, and the three counts the report prints.
/// Split from `run` so the negative fixtures
/// in `tools/tests/` can assert that each rule fires on a planted violation: a check that has never
/// seen one is not known to detect one.
pub fn check(root: &Path) -> (Vec<String>, usize, usize, usize) {
    let mut failures = Vec::new();
    let mut crates_checked = 0usize;

    let crates_dir = root.join("crates");
    let Ok(entries) = std::fs::read_dir(&crates_dir) else {
        return (
            vec![format!("cannot read {}", crates_dir.display())],
            0,
            0,
            0,
        );
    };
    let mut dirs: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
    dirs.sort();

    for dir in dirs.iter().filter(|d| d.is_dir()) {
        crates_checked += 1;
        let lib = dir.join("src/lib.rs");
        match std::fs::read_to_string(&lib) {
            Ok(text) => {
                if !attribute_present(&text, "forbid") {
                    failures.push(format!("{}: no #![forbid(unsafe_code)]", lib.display()));
                }
            }
            Err(e) => failures.push(format!("{}: {e}", lib.display())),
        }
    }

    let mut files = Vec::new();
    collect_rs(&root.join("crates"), &mut files);
    collect_rs(&root.join("tools"), &mut files);
    files.sort();

    let mut seams = Vec::new();
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        if attribute_present(&text, "allow")
            && let Ok(rel) = file.strip_prefix(root)
        {
            seams.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    for seam in &seams {
        if seam != ARENA_SEAM {
            failures.push(format!(
                "{seam}: allows unsafe_code and is not the declared arena seam"
            ));
        }
    }
    if seams.iter().any(|s| s == ARENA_SEAM)
        && let Ok(text) = std::fs::read_to_string(root.join(ARENA_SEAM))
    {
        let lines = text.lines().filter(|l| !l.trim().is_empty()).count();
        if lines > SEAM_LINE_CAP {
            failures.push(format!(
                "{ARENA_SEAM}: {lines} non-blank lines, cap is {SEAM_LINE_CAP}"
            ));
        }
    }

    failures.extend(profile(root));
    failures.extend(shared_mutability(&files, root));

    (failures, crates_checked, files.len(), seams.len())
}

/// Appendix A #2 says conserved quantities are `i64` and **overflow panics**. In a release profile
/// that is only true if it is asked for: `overflow-checks` defaults to off there, and the debug build
/// that panics is not the build that ships.
fn profile(root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(root.join("Cargo.toml")) else {
        return vec!["Cargo.toml is unreadable".to_owned()];
    };
    let release: String = text
        .lines()
        .skip_while(|l| l.trim() != "[profile.release]")
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .collect::<Vec<_>>()
        .join("\n");
    if release.is_empty() {
        return vec!["Cargo.toml: no [profile.release]".to_owned()];
    }
    if release.lines().any(|l| {
        l.split_once('=')
            .is_some_and(|(k, v)| k.trim() == "overflow-checks" && v.trim() == "true")
    }) {
        Vec::new()
    } else {
        vec![
            "Cargo.toml [profile.release]: no `overflow-checks = true`. Appendix A #2 says overflow \
             PANICS, and it defaults to off in release — so the build that ships would wrap silently"
                .to_owned(),
        ]
    }
}

/// ADR-0012: the arena is thread-shareable from the start, run single-threaded until M11.
///
/// `Rc`, `RefCell` and `Cell` are the types that make a structure unshareable, and they are cheap to
/// reach for while the engine is single-threaded — which is exactly when the cost of reaching for them
/// is invisible. Refusing them now is what makes "shareable from the start" a fact rather than an
/// intention. `tools` and `probe` are not layers and are not checked.
fn shared_mutability(files: &[std::path::PathBuf], root: &Path) -> Vec<String> {
    let mut findings = Vec::new();
    for file in files {
        let Ok(rel) = file.strip_prefix(root) else {
            continue;
        };
        let path = rel.to_string_lossy().replace('\\', "/");
        if !path.starts_with("crates/") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        // Tokenised, so a comment or a doc-string naming `RefCell` is not a finding — the same
        // discipline the unsafe_code rule needed after its first draft substring-matched itself.
        let Ok(stream) = text.parse::<TokenStream>() else {
            continue;
        };
        let idents = flatten(stream);
        for name in ["Rc", "RefCell", "Cell"] {
            if idents.iter().any(|i| i == name) {
                findings.push(format!(
                    "{path}: names `{name}`, which the arena cannot be shared across threads through (ADR-0012)"
                ));
            }
        }
    }
    findings
}

/// True where the source contains an attribute of the form `#[<word>(… unsafe_code …)]`, inner or
/// outer. Tokenised, so a string or a comment spelling the same thing is not a finding.
fn attribute_present(text: &str, word: &str) -> bool {
    let Ok(stream) = text.parse::<TokenStream>() else {
        return false;
    };
    let mut hash = false;
    for tree in stream {
        match tree {
            TokenTree::Punct(p) if p.as_char() == '#' => hash = true,
            TokenTree::Punct(p) if p.as_char() == '!' && hash => {}
            TokenTree::Group(g) if hash && g.delimiter() == Delimiter::Bracket => {
                hash = false;
                let idents: Vec<String> = flatten(g.stream());
                if idents.iter().any(|i| i == word) && idents.iter().any(|i| i == "unsafe_code") {
                    return true;
                }
            }
            _ => hash = false,
        }
    }
    false
}

/// Every identifier inside a token stream, groups included: enough to ask whether an attribute
/// names both `allow` and `unsafe_code` without parsing Rust.
fn flatten(stream: TokenStream) -> Vec<String> {
    let mut out = Vec::new();
    for tree in stream {
        match tree {
            TokenTree::Ident(i) => out.push(i.to_string()),
            TokenTree::Group(g) => out.extend(flatten(g.stream())),
            _ => {}
        }
    }
    out
}

/// Print the violation count and each finding, and turn that into the process's exit status.
fn report(failures: &[String]) -> ExitCode {
    if failures.is_empty() {
        println!("  violations: 0");
        ExitCode::SUCCESS
    } else {
        println!("  violations: {}", failures.len());
        for f in failures {
            println!("    {f}");
        }
        ExitCode::FAILURE
    }
}

/// Every `.rs` file beneath `dir`, recursively.
fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}
