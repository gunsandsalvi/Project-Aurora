//! # `appendix` — Appendix A's guard column, generated from the decisions themselves
//!
//! W5.4. Appendix A says *every entry carries a named mechanical guard*, and until now the guard was
//! written twice: once in the ADR that took the decision, once in the register, by hand, in a
//! paraphrase. Two copies of a value is the failure mode §16.1 exists to prevent, applied to prose.
//!
//! So the register's guard cell is **generated** for every row an ADR claims. `aurora-tools appendix`
//! writes it; `check-register` regenerates it and fails if the committed table differs. A row with no
//! ADR keeps its hand-written guard, and the census publishes how many of those remain — which is the
//! honest measure of how much of the register is asserted rather than recorded.
//!
//! Several ADRs may claim one row: ADR-0003, ADR-0004 and ADR-0005 are all entry 16. Their guards are
//! joined in ADR order, so the cell stays a function of the decisions and of nothing else.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

/// Where Appendix A's table starts, and where the register's rows stop.
const TABLE_HEADER: &str = "| # | Decision | Current value | Guard |";

/// The guards each register entry's ADRs name, in ADR order.
#[must_use]
pub fn guards_by_entry(root: &Path) -> BTreeMap<String, Vec<(String, String)>> {
    let mut out: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let Ok(dir) = std::fs::read_dir(root.join("decisions")) else {
        return out;
    };
    let mut files: Vec<_> = dir
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    files.sort();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let field = |name: &str| -> Option<String> {
            text.lines()
                .take_while(|l| !l.starts_with("## "))
                .find_map(|l| l.strip_prefix(&format!("{name}: ")))
                .map(|v| v.trim().to_owned())
        };
        let (Some(id), Some(entry), Some(guard)) =
            (field("id"), field("register-entry"), field("guard"))
        else {
            continue;
        };
        out.entry(entry).or_default().push((id, guard));
    }
    out
}

/// Appendix A with every ADR-claimed guard cell rewritten from its ADRs.
///
/// Returns the whole specification, so that writing it is one `fs::write` and checking it is one
/// comparison — the same text either way, which is what makes the check honest.
///
/// # Errors
///
/// When `PROJECT_AURORA.md` cannot be read.
pub fn render(root: &Path) -> Result<String, String> {
    let path = root.join("PROJECT_AURORA.md");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let guards = guards_by_entry(root);

    let mut out = String::with_capacity(text.len());
    let mut in_table = false;
    for line in text.lines() {
        if line == TABLE_HEADER {
            in_table = true;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_table && !line.starts_with('|') {
            in_table = false;
        }
        if !in_table || line.starts_with("|---") {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        out.push_str(&rewrite_row(line, &guards));
        out.push('\n');
    }
    Ok(out)
}

/// One register row, with its guard cell replaced when an ADR claims its number.
fn rewrite_row(line: &str, guards: &BTreeMap<String, Vec<(String, String)>>) -> String {
    let cells: Vec<&str> = line.split('|').collect();
    // `| a | b | c | d |` splits into six: an empty head, four cells, an empty tail.
    let [head, number, decision, value, _guard, tail] = cells.as_slice() else {
        return line.to_owned();
    };
    let key = number.trim().trim_matches('*').to_owned();
    let Some(claims) = guards.get(&key) else {
        return line.to_owned();
    };
    let generated = claims
        .iter()
        .map(|(id, guard)| format!("{} ({id})", guard.replace('|', "\\|")))
        .collect::<Vec<_>>()
        .join("; ");
    format!("{head}|{number}|{decision}|{value}| {generated} |{tail}")
}

/// Write the generated appendix back into the specification.
pub fn write(root: &Path) -> ExitCode {
    match render(root) {
        Err(why) => {
            eprintln!("{why}");
            ExitCode::FAILURE
        }
        Ok(text) => match std::fs::write(root.join("PROJECT_AURORA.md"), text) {
            Ok(()) => {
                println!("PROJECT_AURORA.md — Appendix A's guard column regenerated");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("cannot write PROJECT_AURORA.md: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, rows, claimed) = check(root);
    println!("check-register");
    println!(
        "  rule 1: an ADR's `register-entry` names a row that exists in Appendix A   rows: {rows}"
    );
    println!(
        "  rule 2: a claimed row's guard cell is generated from its ADRs, verbatim and in order"
    );
    println!("  exemptions: 0");
    println!(
        "  census — register rows: {rows}   recorded by an ADR: {claimed}   asserted without one: {}",
        rows - claimed
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

/// The findings, the number of register rows, and how many of them an ADR claims.
#[must_use]
pub fn check(root: &Path) -> (Vec<String>, usize, usize) {
    let guards = guards_by_entry(root);
    let Ok(text) = std::fs::read_to_string(root.join("PROJECT_AURORA.md")) else {
        return (vec!["PROJECT_AURORA.md is unreadable".to_owned()], 0, 0);
    };
    let numbers: Vec<String> = text
        .lines()
        .skip_while(|l| *l != TABLE_HEADER)
        .skip(2)
        .take_while(|l| l.starts_with('|'))
        .filter_map(|l| l.split('|').nth(1))
        .map(|n| n.trim().trim_matches('*').to_owned())
        .collect();

    let mut findings = Vec::new();
    for (entry, claims) in &guards {
        if !numbers.contains(entry) {
            let ids: Vec<&str> = claims.iter().map(|(id, _)| id.as_str()).collect();
            findings.push(format!(
                "{}: `register-entry: {entry}` names no row in Appendix A",
                ids.join(", ")
            ));
        }
    }
    match render(root) {
        Err(why) => findings.push(why),
        Ok(rendered) if rendered != text => findings.push(
            "Appendix A's guard column differs from what the decisions say. \
             Run `aurora-tools appendix` — a hand edit to a generated cell does not survive."
                .to_owned(),
        ),
        Ok(_) => {}
    }
    let claimed = numbers.iter().filter(|n| guards.contains_key(*n)).count();
    (findings, numbers.len(), claimed)
}
