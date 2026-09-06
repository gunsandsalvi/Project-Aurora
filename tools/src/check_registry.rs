//! §16.1's rules, made mechanical. **Seven, not six**: D3 withdrew the cap on how many entries there
//! may be and withdrew nothing about what an entry must be, and it added the seventh — an entry
//! nothing reads.
//!
//! The seventh matters more than its late arrival suggests. §16.1 recounts a real defect: a lending
//! rate that was never an `Entry` settled twenty-four thousand loans at a rate no agent had chosen,
//! *"while the capital constraints that forbade those loans sat correctly declared and unread."* The
//! first six rules catch a number that is not an entry. Only the seventh catches an entry nothing reads.

use crate::registry::{self, Entry, Units};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

/// The systems that may own an entry. In M0 this is a declared list; when the manifests exist it
/// becomes a read of them, and rule 7 becomes what §16.1 describes.
const OWNERS: [&str; 8] = [
    "seed",
    "monetary-policy",
    "fiscal-policy",
    "ledger",
    "world",
    "runtime",
    "reservation-mint",
    "burn-in-gate",
];

/// Marks of a justification that cites the world outside the model rather than a mechanism in it.
const EXTERNAL: [&str; 6] = [
    "http://",
    "https://",
    "www.",
    "according to",
    "in reality",
    "observed in",
];

/// Run the check and print its report, including the census; the exit status is the process's.
pub fn run(root: &Path) -> ExitCode {
    let (findings, census) = check(root);
    println!("check-registry");
    println!("  rule 1: no assumed level          rule 5: an assumed entry carries a bracket");
    println!("  rule 2: no assumed region scope   rule 6: a derived literal is one of 0, 1, -1, 2");
    println!("  rule 3: a derived expression evaluates and its dimensions agree");
    println!("  rule 4: a structural entry names one of the sixteen identities");
    println!("  rule 7: an entry no declared system reads fails the build");
    println!("  exemptions: 0");
    println!("{census}");
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

/// The findings and the published census.
pub fn check(root: &Path) -> (Vec<String>, String) {
    let units = match Units::load(root) {
        Ok(u) => u,
        Err(e) => return (vec![e], String::new()),
    };
    let identities = match registry::identities(root) {
        Ok(i) => i,
        Err(e) => return (vec![e], String::new()),
    };
    let entries = match registry::entries(root) {
        Ok(e) => e,
        Err(e) => return (vec![e], String::new()),
    };
    let findings = rules(&entries, &units, &identities);
    let census = census(&entries);
    (findings, census)
}

/// Apply the seven rules. Split so a fixture can be checked from a parsed string.
pub fn rules(entries: &[Entry], units: &Units, identities: &[String]) -> Vec<String> {
    let mut findings = Vec::new();
    let by_name: BTreeMap<String, &Entry> = entries.iter().map(|e| (e.name.clone(), e)).collect();
    let mut seen = BTreeSet::new();
    for e in entries {
        if !seen.insert(e.name.clone()) {
            findings.push(format!("{}: the name is not unique", e.name));
        }
        one(e, units, identities, &by_name, &mut findings);
    }
    findings
}

/// The rules that apply to a single entry.
fn one(
    e: &Entry,
    units: &Units,
    identities: &[String],
    by_name: &BTreeMap<String, &Entry>,
    findings: &mut Vec<String>,
) {
    {
        if units.dimension_of(&e.unit).is_none() {
            findings.push(format!(
                "{}: unit `{}` is outside the closed vocabulary",
                e.name, e.unit
            ));
        }

        match e.provenance.as_str() {
            "assumed" => {
                // Rule 1 — no assumed level.
                //
                // §16.1 states this over unit NAMES: `assumed` admits ratio, count, period,
                // physical-unit and hour. That list cannot express a compound unit, and the first
                // derived entry written against a per-household endowment (`hour/count`) proved it:
                // the name is not in the list and the entry is plainly not a level. So the test is
                // over the DIMENSION, which is what A3 actually claims — a level is money or an index.
                if let Some(d) = units.dimension_of(&e.unit)
                    && let Some(level) = ["money", "index"].iter().find(|k| d.contains_key(**k))
                {
                    findings.push(format!(
                            "rule 1 — {}: `assumed` with unit `{}`, whose dimension includes `{level}`. \
                             A3 admits {} and their compounds for an assumed entry; a level is what it forbids.",
                            e.name,
                            e.unit,
                        units.assumable.join(", ")
                    ));
                }
                // Rule 2 — no assumed region scope, with no exception form.
                if e.scope.starts_with("region") {
                    findings.push(format!(
                        "rule 2 — {}: `assumed` with region scope. \"Region 3 is smaller because region 3 \
                         is smaller\" is unwritable, and there is no exception form.",
                        e.name
                    ));
                }
                // Rule 5 — a bracket, and the value inside it.
                match e.bracket {
                    None => {
                        findings.push(format!("rule 5 — {}: `assumed` with no bracket", e.name));
                    }
                    Some((lo, hi)) if e.value < lo || e.value > hi => findings.push(format!(
                        "rule 5 — {}: value {} lies outside its bracket [{lo}, {hi}]",
                        e.name, e.value
                    )),
                    Some(_) => {}
                }
            }
            "structural" => {
                // Rule 4 — one of the sixteen.
                match &e.identity {
                    None => findings.push(format!(
                        "rule 4 — {}: `structural` names no identity",
                        e.name
                    )),
                    Some(id) if !identities.contains(id) => findings.push(format!(
                        "rule 4 — {}: `{id}` is not one of the sixteen definitional identities. \
                         A seventeenth is an ADR.",
                        e.name
                    )),
                    Some(_) => {}
                }
            }
            "derived" => {
                // Rules 3 and 6 — it evaluates, its literals are legal, and its dimensions agree.
                match &e.expression {
                    None => {
                        findings.push(format!("rule 3 — {}: `derived` with no expression", e.name));
                    }
                    Some(expr) => match registry::evaluate(expr, by_name, units) {
                        Err(why) => findings.push(format!("rule 3/6 — {}: {why}", e.name)),
                        Ok(v) => {
                            let declared = units.dimension_of(&e.unit).unwrap_or_default();
                            if v.dimension != declared {
                                findings.push(format!(
                                    "rule 3 — {}: the expression is {}, the declared unit `{}` is {}",
                                    e.name,
                                    registry::show(&v.dimension),
                                    e.unit,
                                    registry::show(&declared)
                                ));
                            }
                        }
                    },
                }
            }
            "placeholder" => {}
            other => findings.push(format!("{}: `{other}` is not a provenance", e.name)),
        }

        // Rule 7 — an entry no declared system reads.
        if !OWNERS.contains(&e.owner.as_str()) {
            findings.push(format!(
                "rule 7 — {}: owner `{}` is not a declared system, so nothing reads it. \
                 An entry that sits correctly declared and unread is the other half of the defect \
                 §16.1 recounts.",
                e.name, e.owner
            ));
        }

        // The justification lint: it catches the careless, not the determined, and claims no more.
        let lower = e.justification.to_lowercase();
        if let Some(mark) = EXTERNAL.iter().find(|m| lower.contains(**m)) {
            findings.push(format!(
                "{}: the justification cites the world outside the model (`{mark}`). \
                 A3 asks for a mechanism inside it.",
                e.name
            ));
        }
    }
}

/// The published census (M3, D3). **There is no cap**; the count is the figure every review pushes
/// down, and it is printed on every build so that no result can be quoted without its assumption count.
pub fn census(entries: &[Entry]) -> String {
    let model: Vec<&Entry> = entries.iter().filter(|e| e.namespace == "model").collect();
    let capacity = entries.len() - model.len();
    let count = |p: &str| model.iter().filter(|e| e.provenance == p).count();
    let assumed = count("assumed");
    let placeholder = count("placeholder");
    format!(
        "  census — model entries: {}   assumed: {assumed}   structural: {}   derived: {}   placeholder: {placeholder}\n  \
         census — capacity entries: {capacity} (engineering sizes; no agent, valuation or economic system reads one)\n  \
         census — the assumed count is the honest measure of how much of the world was chosen rather than produced. \
It has no cap (D3) and the direction is down.",
        model.len(),
        count("structural"),
        count("derived"),
    )
}
