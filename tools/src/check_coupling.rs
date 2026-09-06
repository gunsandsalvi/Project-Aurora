//! # `check-coupling` — a registered value moves only alongside the decision that moved it
//!
//! W5.3. `check-adr` makes a decision record well-formed; this makes a *value* reach for one. A file
//! declared `ratified` in `decisions/coupling.toml` may not have a parsed value changed unless the
//! same tree carries an ADR that names the file in its `registers` field.
//!
//! **Prose is free.** A `justification`, a `note`, a `why`, a comment, a blank line — none of these is
//! a parsed value, and none is coupled. That is the point: a ratified value gets *explained* without
//! an ADR and gets *changed* only with one.
//!
//! **`draft` turns the rule off, and that is a decision rather than a gap.** A schema under initial
//! construction would produce one ADR per column, and a process that costs more than the thing it
//! records stops being followed. A draft file names the milestone building it; that milestone's exit
//! ratifies the file, once, for everything in it.
//!
//! What this compares: the working tree against `HEAD` when the tree carries changes to a registered
//! file, and `HEAD` against its parent when it does not. So the check sees a change while it is still
//! uncommitted, and sees it again in CI after it is pushed.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;

/// Keys whose content is prose. A change to one of these is not a value change.
const PROSE: [&str; 5] = ["justification", "note", "why", "detail", "description"];

/// A registered file and the state its values are in.
pub struct Registered {
    /// Repository-relative path.
    pub path: String,
    /// `ratified` or `draft`.
    pub state: String,
    /// The milestone building it, when `draft`; the milestone that ratified it otherwise.
    pub milestone: String,
}

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let files = registered(root);
    let ratified = files.iter().filter(|f| f.state == "ratified").count();
    let findings = check(root, &files);

    println!("check-coupling");
    println!(
        "  rule: a parsed value in a `ratified` file changes only in a tree that also carries an ADR\n  \
         naming the file; prose is never a parsed value; `draft` files are free until their milestone exits"
    );
    println!("  exemptions: 0");
    println!(
        "  census — registered: {}   ratified: {ratified}   draft: {}",
        files.len(),
        files.len() - ratified
    );
    for f in &files {
        if f.state == "draft" {
            println!("    draft, until {} exits: {}", f.milestone, f.path);
        }
    }
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

/// The registered files, read from `decisions/coupling.toml`.
#[must_use]
pub fn registered(root: &Path) -> Vec<Registered> {
    let Ok(text) = std::fs::read_to_string(root.join("decisions/coupling.toml")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut current: Option<Registered> = None;
    for line in text.lines() {
        let line = line.trim();
        if line == "[[file]]" {
            if let Some(f) = current.take() {
                out.push(f);
            }
            current = Some(Registered {
                path: String::new(),
                state: String::new(),
                milestone: String::new(),
            });
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_owned();
        if let Some(f) = current.as_mut() {
            match key.trim() {
                "path" => f.path = value,
                "state" => f.state = value,
                "milestone" => f.milestone = value,
                _ => {}
            }
        }
    }
    if let Some(f) = current {
        out.push(f);
    }
    out
}

/// The findings: every ratified file whose values moved without an ADR naming it.
pub fn check(root: &Path, files: &[Registered]) -> Vec<String> {
    let mut findings = Vec::new();
    for file in files.iter().filter(|f| f.state == "ratified") {
        let Some((base, moved)) = value_change(root, &file.path) else {
            continue;
        };
        if moved.is_empty() {
            continue;
        }
        if authorising_adr(root, &file.path, &base).is_none() {
            let shown: Vec<&str> = moved.iter().take(4).map(String::as_str).collect();
            findings.push(format!(
                "{}: {} parsed value(s) moved ({}) with no ADR in this tree naming the file in `registers`",
                file.path,
                moved.len(),
                shown.join(", ")
            ));
        }
    }
    findings
}

/// The comparison base, and the parsed values that differ between it and the current content.
///
/// The base is `HEAD` when the working tree has changed the file, and `HEAD~1` when it has not — so a
/// change is seen while uncommitted, and seen again in CI once it is pushed.
fn value_change(root: &Path, path: &str) -> Option<(String, Vec<String>)> {
    let current = std::fs::read_to_string(root.join(path)).ok()?;
    let head = git_show(root, "HEAD", path)?;
    let toml = std::path::Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
    let (base, previous) = if values(&current, toml) == values(&head, toml) {
        ("HEAD~1".to_owned(), git_show(root, "HEAD~1", path)?)
    } else {
        ("HEAD".to_owned(), head)
    };
    let before = values(&previous, toml);
    let after = values(&current, toml);
    let moved: Vec<String> = before.symmetric_difference(&after).cloned().collect();
    Some((base, moved))
}

/// An ADR in this tree that names `path` in its `registers` field, and that itself changed against
/// `base`. Naming the file is not enough — the ADR that authorised last year's change is still there.
fn authorising_adr(root: &Path, path: &str, base: &str) -> Option<String> {
    let dir = root.join("decisions");
    for entry in std::fs::read_dir(dir).ok()?.filter_map(Result::ok) {
        let file = entry.path();
        if file.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        if !text
            .lines()
            .any(|l| l.trim_start().starts_with("registers:") && l.contains(path))
        {
            continue;
        }
        let name = file.file_name()?.to_string_lossy().to_string();
        let rel = format!("decisions/{name}");
        match git_show(root, base, &rel) {
            None => return Some(rel), // new in this tree
            Some(old) if old != text => return Some(rel),
            Some(_) => {}
        }
    }
    None
}

/// `git show <rev>:<path>`, or `None` when the path does not exist at that revision.
fn git_show(root: &Path, rev: &str, path: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("show")
        .arg(format!("{rev}:{path}"))
        .output()
        .ok()?;
    if out.status.success() {
        String::from_utf8(out.stdout).ok()
    } else {
        None
    }
}

/// The parsed values of a registry file: `key=value` when `toml`, and the leading name of each line
/// when not. Comments, blank lines and prose keys are excluded.
///
/// The shape has to be told rather than guessed. `registry/identities.txt` describes an identity
/// called `SpearmanCritical5Percent` as *"the 5% critical value at n = 16"*, and a guess based on the
/// presence of an `=` reads that sentence as an assignment.
#[must_use]
pub fn values(text: &str, toml: bool) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("[[")
            || line.starts_with('[')
        {
            continue;
        }
        match line.split_once('=').filter(|_| toml) {
            Some((key, value)) => {
                let key = key.trim();
                if PROSE.contains(&key) {
                    continue;
                }
                out.insert(format!("{key}={}", value.trim()));
            }
            // A bare list — `registry/identities.txt` and `units.txt`. The leading name is the value;
            // its position must not matter, and the prose after it is prose.
            None => {
                if let Some(name) = line.split_whitespace().next() {
                    out.insert(name.to_owned());
                }
            }
        }
    }
    out
}
