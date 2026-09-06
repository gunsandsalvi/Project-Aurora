//! **A cross-reference resolves to a section, or it is not a reference.**
//!
//! The specification cites `§x.y` from its own prose, from `IMPLEMENTATION.md`, and eventually from
//! source comments. Eight of those citations resolved to nothing when this project started, three of
//! them naming what discharges A1 and A4 — a reader following one would find the argument missing
//! and could not tell whether the section was deleted or never written.
//!
//! **The worse failure is the one this check exists for**: a citation that used to dangle and now
//! resolves *to the wrong section*, because a later edition added a heading with that number. A
//! dangling reference announces itself; a stale one does not.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

/// The documents that own headings, and the documents that cite them.
const DOCS: [&str; 3] = ["PROJECT_AURORA.md", "IMPLEMENTATION.md", "MILESTONE_0.md"];

/// The ratchet: sections cited but not yet written, each with who owes it.
const BASELINE: &str = "decisions/dangling-refs.baseline";

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, headings, refs, owed) = check(root);
    println!("check-refs");
    println!(
        "  rule: every §x.y citation resolves to a heading   headings: {headings}, citations: {refs}"
    );
    println!(
        "  rule: the dangling baseline may shrink and may never grow   sections still owed: {owed}"
    );
    println!("  exemptions: 0 — the baseline is a ratchet, and it is counted below, not exempted");
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

/// The findings, the headings collected, the citations checked, and the size of the baseline.
pub fn check(root: &Path) -> (Vec<String>, usize, usize, usize) {
    let mut headings = BTreeSet::new();
    for doc in DOCS {
        let Ok(text) = std::fs::read_to_string(root.join(doc)) else {
            continue;
        };
        for line in text.lines() {
            if let Some(id) = heading_id(line) {
                // A heading `### 13.4 …` also makes `§13` resolvable, since §13 is its parent.
                let mut parts: Vec<&str> = id.split('.').collect();
                while !parts.is_empty() {
                    headings.insert(parts.join("."));
                    parts.pop();
                }
            }
        }
    }

    let baseline = read_baseline(root);
    let mut findings = Vec::new();
    let mut refs = 0usize;
    let mut dangled: BTreeSet<String> = BTreeSet::new();

    for doc in DOCS {
        let Ok(text) = std::fs::read_to_string(root.join(doc)) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            for cited in citations(line) {
                refs += 1;
                if headings.contains(&cited) {
                    continue;
                }
                dangled.insert(cited.clone());
                if !baseline.contains_key(&cited) {
                    findings.push(format!(
                        "{doc}:{}: §{cited} resolves to no heading, and is not in the baseline. \
                         A new dangling citation is a defect at the point it is written.",
                        n + 1
                    ));
                }
            }
        }
    }

    // The ratchet's other half: a baseline entry that now resolves must come off the list, or the
    // list goes stale and stops meaning anything.
    for (id, owed) in &baseline {
        if !dangled.contains(id) {
            findings.push(format!(
                "§{id} now resolves — remove it from {BASELINE}. It was owed by: {owed}"
            ));
        }
    }

    let count = headings.len();
    (findings, count, refs, baseline.len())
}

/// The baseline, as `id -> who owes it`. Comment lines and blanks are skipped.
fn read_baseline(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(root.join(BASELINE)) else {
        return out;
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        if let Some(id) = parts.next() {
            out.insert(id.to_owned(), parts.next().unwrap_or("").trim().to_owned());
        }
    }
    out
}

/// `## 13. The seed` and `### 13.4 The policy rate` both yield their number; anything else yields none.
fn heading_id(line: &str) -> Option<String> {
    let rest = line
        .strip_prefix("## ")
        .or_else(|| line.strip_prefix("### "))?;
    let token = rest.split_whitespace().next()?;
    let id = token.trim_end_matches('.');
    if id.is_empty() || !id.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    // `6.1a` is a heading. A trailing letter is part of the id, not a reason to reject it.
    if id
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c.is_ascii_lowercase())
    {
        Some(id.to_owned())
    } else {
        None
    }
}

/// Every `§x`, `§x.y` or `§x.y.z` on a line. A trailing letter (`§6.1a`) is part of the id.
fn citations(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(pos) = rest.find('§') {
        let after = &rest[pos + '§'.len_utf8()..];
        let id: String = after
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.' || c.is_ascii_lowercase())
            .collect();
        let id = id.trim_end_matches('.').to_owned();
        if !id.is_empty() && id.starts_with(|c: char| c.is_ascii_digit()) {
            out.push(id);
        }
        rest = after;
    }
    out
}
