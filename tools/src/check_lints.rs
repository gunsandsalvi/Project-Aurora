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

    (failures, crates_checked, files.len(), seams.len())
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
