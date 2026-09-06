//! §4.4: **the surface computes nothing.** Every displayed number comes from a named reader, and a
//! quantity computed twice — once in the engine and once in a status bar — is two implementations of
//! one rule, and the first place anyone looks when they disagree.
//!
//! The check tokenises rather than greps, so a `+` inside a comment, a string or a doc comment is not
//! a finding. It flags the five binary arithmetic puncts. It cannot tell a unary minus on a literal
//! from a subtraction, and it does not try: `surface` has no reason to write either.
//!
//! `shell` is deliberately outside this check (ADR-0003). A user interface must compute — a label
//! needs `width - 8` — and without the split the first such line opens the exemption §17 says has none.

use proc_macro2::{Spacing, TokenStream, TokenTree};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const ARITHMETIC: [char; 5] = ['+', '-', '*', '/', '%'];

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, scanned) = check(root);
    println!("check-surface");
    println!(
        "  rule: §4.4 — no binary arithmetic operator in `surface`   files scanned: {scanned}"
    );
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

/// The findings and the file count. Split from `run` so a planted violation can be asserted on.
pub fn check(root: &Path) -> (Vec<String>, usize) {
    let mut findings = Vec::new();
    let mut files = Vec::new();
    collect_rs(&root.join("crates/surface"), &mut files);
    files.sort();

    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let Ok(stream) = text.parse::<TokenStream>() else {
            findings.push(format!("{}: does not tokenise", file.display()));
            continue;
        };
        walk(stream, file, &mut findings);
    }

    (findings, files.len())
}

/// Walk a token stream, groups included, reporting each binary arithmetic punct.
fn walk(stream: TokenStream, file: &Path, out: &mut Vec<String>) {
    let mut trees = stream.into_iter().peekable();
    while let Some(tree) = trees.next() {
        match tree {
            TokenTree::Group(g) => walk(g.stream(), file, out),
            TokenTree::Punct(p) if ARITHMETIC.contains(&p.as_char()) => {
                // `->` is a `-` punct joined to a `>`, and it is in the signature of every function
                // that returns anything. The first negative fixture caught this: one subtraction in
                // one line reported twice, because the return arrow counted as the second.
                if p.as_char() == '-'
                    && p.spacing() == Spacing::Joint
                    && matches!(trees.peek(), Some(TokenTree::Punct(n)) if n.as_char() == '>')
                {
                    trees.next();
                    continue;
                }
                let line = p.span().start().line;
                out.push(format!(
                    "{}:{line}: `{}` — the surface computes nothing (§4.4)",
                    file.display(),
                    p.as_char()
                ));
            }
            _ => {}
        }
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
