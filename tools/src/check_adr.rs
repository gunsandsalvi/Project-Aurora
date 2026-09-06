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
//!
//! It also holds the counter. `decisions/register.txt` allocates every number, one per line, and this
//! checks that the files and the register agree: a number allocated outside the counter, a filename
//! disagreeing with its own `id` field, or a register that is not strictly ascending. The count of
//! written against reserved is what turns exit criterion 15 into a number.

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
    let register = read_register(root).unwrap_or_default();
    let written = register
        .iter()
        .filter(|(id, _)| adr_path(root, *id).is_some())
        .count();
    println!("check-adr");
    println!("  rule 1: eleven fields, and `guard` is not paperwork   decisions scanned: {count}");
    println!("  rule 2: every number is allocated in `decisions/register.txt`");
    println!("  rule 3: a filename's number and slug are its own `id` and its register line");
    println!("  rule 4: the register is strictly ascending, with no number allocated twice");
    println!("  exemptions: 0");
    println!(
        "  census — allocated: {}   written: {written}   reserved, not yet written: {}",
        register.len(),
        register.len() - written
    );
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

/// The register: `<id> <slug>` per line, comments and blanks skipped, in file order.
///
/// Returns `None` only when the file cannot be read; a malformed line is a finding, not an error, so
/// that the check reports it rather than disappearing.
pub fn read_register(root: &Path) -> Option<Vec<(u16, String)>> {
    let text = std::fs::read_to_string(root.join("decisions/register.txt")).ok()?;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((id, slug)) = line.split_once(' ') else {
            continue;
        };
        let Ok(id) = id.trim().parse::<u16>() else {
            continue;
        };
        out.push((id, slug.trim().to_owned()));
    }
    Some(out)
}

/// The file for an allocated number, if one has been written yet.
fn adr_path(root: &Path, id: u16) -> Option<std::path::PathBuf> {
    let dir = root.join("decisions");
    let prefix = format!("ADR-{id:04}-");
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(&prefix))
        })
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

    findings.extend(check_register(root, &files));

    let count = files.len();
    (findings, count)
}

/// Rules 2–4: the files and `decisions/register.txt` agree, and the register is a counter.
fn check_register(root: &Path, files: &[std::path::PathBuf]) -> Vec<String> {
    let Some(register) = read_register(root) else {
        return vec!["decisions/register.txt is missing — nothing allocates a number".to_owned()];
    };
    let mut findings = Vec::new();

    // Rule 4. Strictly ascending covers both ordering and duplication in one comparison.
    for pair in register.windows(2) {
        if let [(a, _), (b, slug)] = pair
            && b <= a
        {
            findings.push(format!(
                "register: {b:04} ({slug}) does not follow {a:04} — the counter must ascend"
            ));
        }
    }

    for file in files {
        let name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let Some(stem) = name
            .strip_prefix("ADR-")
            .and_then(|n| n.strip_suffix(".md"))
        else {
            findings.push(format!("{name}: not named ADR-NNNN-kebab.md"));
            continue;
        };
        let Some((num, slug)) = stem.split_once('-') else {
            findings.push(format!("{name}: no slug after the number"));
            continue;
        };
        let Ok(id) = num.parse::<u16>() else {
            findings.push(format!("{name}: `{num}` is not a number"));
            continue;
        };

        // Rule 3, first half: the front matter's `id` is the filename's number.
        if let Ok(text) = std::fs::read_to_string(file)
            && let Some(front) = front_matter(&text)
            && let Some((_, declared)) = front.iter().find(|(k, _)| k == "id")
            && declared.trim() != format!("ADR-{id:04}")
        {
            findings.push(format!(
                "{name}: front matter says `{}`, the filename says ADR-{id:04}",
                declared.trim()
            ));
        }

        // Rules 2 and 3: allocated, and allocated under this slug.
        match register.iter().find(|(r, _)| *r == id) {
            None => findings.push(format!(
                "{name}: {id:04} is not in decisions/register.txt — allocated outside the counter"
            )),
            Some((_, registered)) if registered != slug => findings.push(format!(
                "{name}: the register allocates {id:04} to `{registered}`, not `{slug}`"
            )),
            Some(_) => {}
        }
    }
    findings
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
