//! **Every check has a negative fixture.** A check that has never seen a violation is not known to
//! detect one, and a green run over an empty tree is evidence of nothing. Each test below plants one
//! violation and asserts the check finds it — **and finds it for its own stated reason, and no other**.

use std::fs;
use std::path::PathBuf;

/// A throwaway workspace root under `target/`, shaped like the real one.
fn scratch(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tools crate always has a workspace parent")
        .join("target/fixtures")
        .join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("crates")).expect("scratch root is writable");
    root
}

fn crate_at(root: &std::path::Path, name: &str, lib: &str) {
    let dir = root.join("crates").join(name).join("src");
    fs::create_dir_all(&dir).expect("scratch crate is writable");
    fs::write(dir.join("lib.rs"), lib).expect("scratch lib.rs is writable");
}

#[test]
fn a_crate_without_forbid_unsafe_is_caught() {
    let root = scratch("no-forbid");
    crate_at(&root, "kernel", "//! fine\n#![forbid(unsafe_code)]\n");
    crate_at(&root, "domain", "//! no forbid attribute here\n");

    let (findings, crates, _, _) = aurora_tools::check_lints::check(&root);
    assert_eq!(crates, 2, "both scratch crates should be scanned");
    let [only] = findings.as_slice() else {
        panic!("exactly one crate is missing the attribute: {findings:?}")
    };
    assert!(
        only.contains("domain"),
        "the finding names the offender: {only}"
    );
    assert!(
        only.contains("forbid"),
        "and names the rule it broke: {only}"
    );
}

#[test]
fn a_string_that_spells_the_attribute_is_not_a_finding() {
    // The defect the first draft of check-lints had: it substring-matched, and its first run
    // reported itself, because its own needle appeared in its own source.
    let root = scratch("needle-in-a-string");
    crate_at(
        &root,
        "kernel",
        "//! fine\n#![forbid(unsafe_code)]\npub const NEEDLE: &str = \"#![allow(unsafe_code)]\";\n",
    );

    let (findings, _, _, seams) = aurora_tools::check_lints::check(&root);
    assert_eq!(seams, 0, "a string literal is not an attribute");
    assert!(findings.is_empty(), "and so is not a finding: {findings:?}");
}

#[test]
fn an_undeclared_unsafe_seam_is_caught() {
    let root = scratch("stray-seam");
    crate_at(&root, "world", "//! fine\n#![forbid(unsafe_code)]\n");
    fs::write(
        root.join("crates/world/src/sneaky.rs"),
        "#[allow(unsafe_code)]\npub fn f() {}\n",
    )
    .expect("scratch file is writable");

    let (findings, _, _, seams) = aurora_tools::check_lints::check(&root);
    assert_eq!(seams, 1, "the attribute is seen");
    let [only] = findings.as_slice() else {
        panic!("one stray seam, one finding: {findings:?}")
    };
    assert!(
        only.contains("sneaky.rs"),
        "the finding names the file: {only}"
    );
    assert!(only.contains("arena seam"), "and names the rule: {only}");
}

#[test]
fn arithmetic_in_surface_is_caught() {
    let root = scratch("surface-arithmetic");
    let dir = root.join("crates/surface/src");
    fs::create_dir_all(&dir).expect("scratch crate is writable");
    fs::write(
        dir.join("lib.rs"),
        "//! §4.4 says this crate computes nothing.\n#![forbid(unsafe_code)]\npub fn width(w: i32) -> i32 { w - 8 }\n",
    )
    .expect("scratch lib.rs is writable");

    let (findings, scanned) = aurora_tools::check_surface::check(&root);
    assert_eq!(scanned, 1);
    let [only] = findings.as_slice() else {
        panic!("the one subtraction is found once: {findings:?}")
    };
    assert!(only.contains("`-`"), "the operator is named: {only}");
    assert!(only.contains("§4.4"), "with the rule it broke: {only}");
}

#[test]
fn a_return_arrow_is_not_arithmetic() {
    // Caught by the fixture above on its first run: `->` is a `-` joined to a `>`, and it is in the
    // signature of every function that returns anything, so a naive check flags the whole tree.
    let root = scratch("surface-arrow");
    let dir = root.join("crates/surface/src");
    fs::create_dir_all(&dir).expect("scratch crate is writable");
    fs::write(
        dir.join("lib.rs"),
        "//! A named reader, and nothing computed.\n#![forbid(unsafe_code)]\npub fn units(n: i32) -> i32 { n }\n",
    )
    .expect("scratch lib.rs is writable");

    let (findings, _) = aurora_tools::check_surface::check(&root);
    assert!(
        findings.is_empty(),
        "the return arrow is not a subtraction: {findings:?}"
    );
}

#[test]
fn arithmetic_in_a_comment_or_string_is_not_a_finding() {
    let root = scratch("surface-comment");
    let dir = root.join("crates/surface/src");
    fs::create_dir_all(&dir).expect("scratch crate is writable");
    fs::write(
        dir.join("lib.rs"),
        "//! A reader is `units × price`, never a - b.\n#![forbid(unsafe_code)]\npub const LABEL: &str = \"3 + 4\";\n",
    )
    .expect("scratch lib.rs is writable");

    let (findings, _) = aurora_tools::check_surface::check(&root);
    assert!(
        findings.is_empty(),
        "a grep would flag both of these; the lexer does not: {findings:?}"
    );
}

#[test]
fn a_new_dangling_citation_is_caught() {
    let root = scratch("dangling-new");
    fs::write(
        root.join("PROJECT_AURORA.md"),
        "## 1. What this is\n\nSee §1 for the claims, and §99.9 for the thing nobody wrote.\n",
    )
    .expect("scratch doc is writable");
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    fs::write(root.join("decisions/dangling-refs.baseline"), "# empty\n")
        .expect("scratch baseline is writable");

    let (findings, _, _, owed) = aurora_tools::check_refs::check(&root);
    assert_eq!(owed, 0, "the baseline is empty");
    let [only] = findings.as_slice() else {
        panic!("§1 resolves and §99.9 does not: {findings:?}")
    };
    assert!(
        only.contains("99.9"),
        "the finding names the citation: {only}"
    );
    assert!(
        only.contains("baseline"),
        "and says why it is a defect now: {only}"
    );
}

#[test]
fn a_baseline_entry_that_now_resolves_must_come_off() {
    // The other half of the ratchet. Without it the list goes stale and stops meaning anything.
    let root = scratch("dangling-fixed");
    fs::write(
        root.join("PROJECT_AURORA.md"),
        "## 7. Instruments\n\n### 7.1 The vocabulary\n\nCited at §7.1, and it now exists.\n",
    )
    .expect("scratch doc is writable");
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    fs::write(
        root.join("decisions/dangling-refs.baseline"),
        "7.1  the vocabulary. M3\n",
    )
    .expect("scratch baseline is writable");

    let (findings, _, _, owed) = aurora_tools::check_refs::check(&root);
    assert_eq!(owed, 1);
    let [only] = findings.as_slice() else {
        panic!("the resolved baseline entry is reported: {findings:?}")
    };
    assert!(only.contains("now resolves"), "{only}");
    assert!(
        only.contains("the vocabulary. M3"),
        "and names who owed it: {only}"
    );
}

#[test]
fn an_adr_without_a_guard_is_caught() {
    let root = scratch("adr-no-guard");
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    let mut front = String::from("---\n");
    for f in [
        "id: ADR-0001",
        "title: t",
        "status: accepted",
        "date: 2026-01-01",
        "register-entry: 1",
        "claim-impact: none",
        "guard:",
        "supersedes: none",
        "cost: none",
        "alternatives-rejected: none",
        "re-derivations: none",
    ] {
        front.push_str(f);
        front.push('\n');
    }
    front.push_str("---\n\n## Decision\n\nSomething.\n");
    fs::write(root.join("decisions/ADR-0001-x.md"), front).expect("scratch adr is writable");

    let (findings, count) = aurora_tools::check_adr::check(&root);
    assert_eq!(count, 1);
    let [only] = findings.as_slice() else {
        panic!("the empty guard is the one finding: {findings:?}")
    };
    assert!(only.contains("guard"), "{only}");
    assert!(
        only.contains("same as absent"),
        "and says why an empty field is not a field: {only}"
    );
}
