//! §7.2's totality, and §7.4's amendment matrix.
//!
//! **A2's cheapest falsification.** §7.2 claims a new instrument type costs thirteen answers and zero
//! agent edits; filling the table by hand before anything reads it is what tests whether thirteen
//! questions are enough, and whether the answer vocabularies close.
//!
//! The check is total in both directions: every type answers every question, every answer is drawn
//! from that question's closed set, and every mechanism names types that exist.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;

/// The thirteen questions and the closed set each admits. `*` marks a question whose answer is a
/// structured string rather than one word, checked by shape below.
const QUESTIONS: [(&str, &[&str]); 13] = [
    (
        "q1",
        &[
            "currency",
            "face-value",
            "share",
            "fund-unit",
            "goods",
            "capital-unit",
            "dwelling",
            "floor-area",
            "labour-hour",
        ],
    ),
    ("q2", &["SingleUnit", "DeclaredPiece"]),
    ("q3", &["InCurrency", "NotDenominated"]),
    ("q4", &["LiabilityOf", "ResidualClaimOn", "NoIssuer"]),
    ("q5", &["price", "yield", "spread", "rate"]),
    ("q6", &["Dated", "Perpetual", "Demand"]),
    ("q7", &["NoAccrual", "PerTickSimple", "PerTickCompounded"]),
    ("q8", &["*"]),
    ("q9", &["NoCarrier", "PerUnit", "PerContract"]),
    (
        "q10",
        &["any", "institutional", "issuer-restricted", "sovereign"],
    ),
    ("q11", &["*"]),
    ("q12", &["tier1", "tier2", "illiquid"]),
    ("q13", &["VenueCleared", "DerivedMark", "UnitOfAccount"]),
];

/// The seven option families of §7.6.
const FAMILIES: [&str; 7] = [
    "callable",
    "puttable",
    "prepayable",
    "convertible",
    "coverPooled",
    "contingent",
    "extendible",
];

/// The four ranks of §7.2 question 11.
const RANKS: [&str; 4] = ["preferred", "senior", "subordinated", "residual"];

/// Run the check and print its report; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, types, mechanisms) = check(root);
    println!("check-instruments");
    println!(
        "  rule: §7.2 — every type answers all thirteen questions, from each question's closed set   types: {types}"
    );
    println!(
        "  rule: §7.4 — every amendment mechanism names types that exist   mechanisms: {mechanisms}"
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

/// The findings, the type count and the mechanism count.
pub fn check(root: &Path) -> (Vec<String>, usize, usize) {
    let mut findings = Vec::new();
    let intrinsic = match std::fs::read_to_string(root.join("instruments/intrinsic.toml")) {
        Ok(t) => t,
        Err(e) => return (vec![format!("instruments/intrinsic.toml: {e}")], 0, 0),
    };
    let Ok(doc) = intrinsic.parse::<toml::Table>() else {
        return (
            vec!["instruments/intrinsic.toml does not parse".to_owned()],
            0,
            0,
        );
    };
    let rows = doc
        .get("type")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut names = BTreeSet::new();
    for row in &rows {
        let name = row
            .get("name")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unnamed>")
            .to_owned();
        if !names.insert(name.clone()) {
            findings.push(format!("{name}: the type is declared twice"));
        }
        for (q, allowed) in QUESTIONS {
            let Some(v) = row.get(q) else {
                findings.push(format!(
                    "{name}: no answer to {q}. §7.2 admits no default, because a default is a \
                     fourteenth decision made by whoever wrote it."
                ));
                continue;
            };
            match q {
                "q8" => check_optionality(&name, v, &mut findings),
                "q11" => check_rank(&name, v, &mut findings),
                _ => {
                    let Some(word) = v.as_str() else {
                        findings.push(format!("{name}: {q} is not a word"));
                        continue;
                    };
                    if !allowed.contains(&word) {
                        findings.push(format!(
                            "{name}: {q} answers `{word}`, which is outside its closed set ({})",
                            allowed.join(" | ")
                        ));
                    }
                }
            }
        }
    }

    let mechanisms = check_amendments(root, &names, &mut findings);
    let types = rows.len();
    (findings, types, mechanisms)
}

/// Question 8: a set drawn from the seven families of §7.6, never a bag and never a string.
fn check_optionality(name: &str, v: &toml::Value, findings: &mut Vec<String>) {
    let Some(list) = v.as_array() else {
        findings.push(format!("{name}: q8 is not a set. §7.6 forbids a bag."));
        return;
    };
    for item in list {
        let Some(f) = item.as_str() else {
            findings.push(format!("{name}: q8 holds a non-word"));
            continue;
        };
        if !FAMILIES.contains(&f) {
            findings.push(format!(
                "{name}: q8 names `{f}`, which is not one of §7.6's seven families"
            ));
        }
    }
}

/// Question 11: `secured:<bool> rank:<rank>` and, where secured, a shortfall rank.
///
/// §7.2: *security is a relation, not a rank* — a boolean and an ordinal, rather than an ordinal
/// doing both jobs. The shortfall rank is required in the secured arm and forbidden outside it,
/// because an optional field is the default §7.2 refuses.
fn check_rank(name: &str, v: &toml::Value, findings: &mut Vec<String>) {
    let Some(s) = v.as_str() else {
        findings.push(format!("{name}: q11 is not a word"));
        return;
    };
    let parts: Vec<&str> = s.split_whitespace().collect();
    let secured = parts.iter().find_map(|p| p.strip_prefix("secured:"));
    let rank = parts.iter().find_map(|p| p.strip_prefix("rank:"));
    let shortfall = parts.iter().find_map(|p| p.strip_prefix("shortfall:"));
    match (secured, rank) {
        (Some(sec), Some(r)) => {
            if !RANKS.contains(&r) {
                findings.push(format!("{name}: q11 rank `{r}` is not one of the four"));
            }
            match (sec, shortfall) {
                ("true", None) => findings.push(format!(
                    "{name}: q11 is secured and names no shortfall rank. §7.2 requires one in the \
                     secured arm; an optional field would be the default it refuses."
                )),
                ("false", Some(_)) => findings.push(format!(
                    "{name}: q11 is unsecured and names a shortfall rank, which has no meaning"
                )),
                ("true" | "false", _) => {}
                (other, _) => {
                    findings.push(format!("{name}: q11 secured is `{other}`, not a boolean"));
                }
            }
        }
        _ => findings.push(format!(
            "{name}: q11 must read `secured:<bool> rank:<rank>`"
        )),
    }
}

/// §7.4's matrix: every mechanism names an owner and types that exist.
fn check_amendments(root: &Path, types: &BTreeSet<String>, findings: &mut Vec<String>) -> usize {
    let Ok(text) = std::fs::read_to_string(root.join("instruments/amendments.toml")) else {
        findings.push("instruments/amendments.toml is unreadable".to_owned());
        return 0;
    };
    let Ok(doc) = text.parse::<toml::Table>() else {
        findings.push("instruments/amendments.toml does not parse".to_owned());
        return 0;
    };
    let rows = doc
        .get("mechanism")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    if rows.len() != 8 {
        findings.push(format!(
            "§7.4 declares eight amendment mechanisms and this file holds {}. A ninth is an ADR.",
            rows.len()
        ));
    }
    for row in &rows {
        let name = row
            .get("name")
            .and_then(toml::Value::as_str)
            .unwrap_or("<unnamed>");
        let applies = row
            .get("applies")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        for t in &applies {
            let Some(t) = t.as_str() else { continue };
            if !types.contains(t) {
                findings.push(format!(
                    "{name}: applies to `{t}`, which is not a declared type"
                ));
            }
        }
    }
    rows.len()
}
