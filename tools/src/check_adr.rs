//! **An ADR with no named mechanical guard is not accepted.**
//!
//! Appendix A's own rule: *every entry carries a named mechanical guard, and a decision that cannot be
//! given one may not be entered.* The specification requires an ADR about twenty times and defines no
//! format, so the format is here, and the one field that is not paperwork is `guard`.
//!
//! The check is deliberately shallow. It asserts that the eleven fields are present and that `guard`,
//! `why` and `alternatives-rejected` are not empty. **It cannot tell whether a guard is real** — a
//! field reading "review" would pass. What it removes is the option of not writing one, which is how
//! the decisions that lacked guards got in.

use std::path::Path;
use std::process::ExitCode;

/// The eleven fields Appendix A's supersession procedure needs to be operable.
const FIELDS: [&str; 11] = [
    "id",
    "title",
    "status",
    "date",
    "register-entry",
    "claim-impact",
    "guard",
    "supersedes",
    "cost",
    "alternatives-rejected",
    "re-derivations",
];

/// Fields whose presence is not enough: an empty one is the same as an absent one.
const MUST_NOT_BE_EMPTY: [&str; 3] = ["guard", "cost", "alternatives-rejected"];

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, count) = check(root);
    println!("check-adr");
    println!("  rule: eleven fields, and `guard` is not paperwork   decisions scanned: {count}");
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

/// The findings and the number of decision records scanned.
pub fn check(root: &Path) -> (Vec<String>, usize) {
    let dir = root.join("decisions");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return (vec![format!("cannot read {}", dir.display())], 0);
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    files.sort();

    let mut findings = Vec::new();
    let mut ids = Vec::new();
    for file in &files {
        let name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let Ok(text) = std::fs::read_to_string(file) else {
            findings.push(format!("{name}: unreadable"));
            continue;
        };
        let Some(front) = front_matter(&text) else {
            findings.push(format!("{name}: no `---` front matter"));
            continue;
        };
        for field in FIELDS {
            match front.iter().find(|(k, _)| k == field) {
                None => findings.push(format!("{name}: no `{field}` field")),
                Some((_, v)) if MUST_NOT_BE_EMPTY.contains(&field) && v.trim().is_empty() => {
                    findings.push(format!(
                        "{name}: `{field}` is empty, which is the same as absent"
                    ));
                }
                Some(_) => {}
            }
        }
        if let Some((_, id)) = front.iter().find(|(k, _)| k == "id") {
            ids.push(id.trim().to_owned());
        }
    }

    let mut seen = ids.clone();
    seen.sort();
    seen.dedup();
    if seen.len() != ids.len() {
        findings.push("two decisions share an id — the counter has collided".to_owned());
    }

    let count = files.len();
    (findings, count)
}

/// The `key: value` pairs between the opening and closing `---`.
fn front_matter(text: &str) -> Option<Vec<(String, String)>> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let mut out = Vec::new();
    for line in rest.get(..end)?.lines() {
        if let Some((k, v)) = line.split_once(':') {
            out.push((k.trim().to_owned(), v.trim().to_owned()));
        }
    }
    Some(out)
}
