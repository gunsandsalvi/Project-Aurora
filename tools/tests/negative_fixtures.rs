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
    // A compliant baseline, so every fixture below breaks exactly one thing. A fixture that trips two
    // rules proves neither, and a scratch root with no manifest trips the profile rule for free.
    fs::write(
        root.join("Cargo.toml"),
        "[profile.release]\noverflow-checks = true\n",
    )
    .expect("scratch manifest is writable");
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
    fs::write(root.join("decisions/register.txt"), "0001 x\n")
        .expect("scratch register is writable");

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

// ── the counter's three rules, one fixture each ─────────────────────────────────────────────────
//
// W5.1's claim is that a collision becomes a merge conflict rather than a duplicate decision. That
// only holds if the register and the directory are checked against each other, so each way they can
// disagree fails here for its own reason.

/// A complete ADR under `id`, named `ADR-<file_id>-<slug>.md`, in a fresh scratch root.
fn adr_fixture(
    name: &str,
    id: &str,
    file_id: &str,
    slug: &str,
    register: &str,
) -> std::path::PathBuf {
    let root = scratch(name);
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    let mut front = String::from("---\n");
    for f in [
        format!("id: {id}"),
        "title: t".to_owned(),
        "status: accepted".to_owned(),
        "date: 2026-01-01".to_owned(),
        "register-entry: 1".to_owned(),
        "claim-impact: none".to_owned(),
        "guard: check-adr".to_owned(),
        "supersedes: none".to_owned(),
        "cost: none".to_owned(),
        "alternatives-rejected: none".to_owned(),
        "re-derivations: none".to_owned(),
    ] {
        front.push_str(&f);
        front.push('\n');
    }
    front.push_str("---\n\n## Decision\n\nSomething.\n");
    fs::write(
        root.join(format!("decisions/ADR-{file_id}-{slug}.md")),
        front,
    )
    .expect("scratch adr is writable");
    fs::write(root.join("decisions/register.txt"), register).expect("scratch register is writable");
    root
}

#[test]
fn a_number_allocated_outside_the_counter_is_caught() {
    let root = adr_fixture("adr-unallocated", "ADR-0007", "0007", "x", "0001 a\n");
    let (findings, _) = aurora_tools::check_adr::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the unallocated number is the one finding: {findings:?}")
    };
    assert!(only.contains("allocated outside the counter"), "{only}");
}

#[test]
fn a_slug_disagreeing_with_the_register_is_caught() {
    let root = adr_fixture(
        "adr-wrong-slug",
        "ADR-0001",
        "0001",
        "elsewhere",
        "0001 here\n",
    );
    let (findings, _) = aurora_tools::check_adr::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the slug disagreement is the one finding: {findings:?}")
    };
    assert!(only.contains("`here`, not `elsewhere`"), "{only}");
}

#[test]
fn a_filename_disagreeing_with_its_own_id_is_caught() {
    let root = adr_fixture("adr-wrong-id", "ADR-0002", "0001", "x", "0001 x\n");
    let (findings, _) = aurora_tools::check_adr::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the id disagreement is the one finding: {findings:?}")
    };
    assert!(only.contains("front matter says `ADR-0002`"), "{only}");
    assert!(only.contains("filename says ADR-0001"), "{only}");
}

#[test]
fn a_number_allocated_twice_is_caught() {
    let root = adr_fixture("adr-twice", "ADR-0001", "0001", "x", "0001 x\n0001 y\n");
    let (findings, _) = aurora_tools::check_adr::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the duplicate allocation is the one finding: {findings:?}")
    };
    assert!(only.contains("the counter must ascend"), "{only}");
}

// ── the registry's seven rules, one fixture each ────────────────────────────────────────────────
//
// §16.1's rules are the whole of A3's first subject, and each must fail **for its own stated reason
// and no other**. A fixture that trips two rules proves neither.

/// A well-formed entry, which each fixture then breaks in exactly one way.
fn entry(extra: &str) -> String {
    format!(
        "[[entry]]\nname = \"t.x\"\nnamespace = \"model\"\nvalue = 1.0\n\
         scope = \"world\"\nowner = \"seed\"\njustification = \"a mechanism inside the model\"\n{extra}\n"
    )
}

fn findings_for(doc: &str) -> Vec<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the tools crate always has a workspace parent");
    let units = aurora_tools::registry::Units::load(root).expect("units.txt is committed");
    let ids = aurora_tools::registry::identities(root).expect("identities.txt is committed");
    let entries = aurora_tools::registry::parse_entries(doc).expect("the fixture parses");
    aurora_tools::check_registry::rules(&entries, &units, &ids)
}

fn only_finding(doc: &str) -> String {
    let f = findings_for(doc);
    let [only] = f.as_slice() else {
        panic!(
            "a fixture must break exactly one rule, and this broke {}: {f:?}",
            f.len()
        )
    };
    only.clone()
}

#[test]
fn rule_1_an_assumed_level_is_caught() {
    let f = only_finding(&entry(
        "unit = \"minor-unit\"\nprovenance = \"assumed\"\nbracket = [0.0, 2.0]\naxis = \"none\"",
    ));
    assert!(f.starts_with("rule 1"), "{f}");
    assert!(
        f.contains("money"),
        "the finding names the dimension that makes it a level: {f}"
    );
}

#[test]
fn rule_2_an_assumed_region_scope_is_caught() {
    let doc =
        entry("unit = \"ratio\"\nprovenance = \"assumed\"\nbracket = [0.0, 2.0]\naxis = \"none\"")
            .replace("scope = \"world\"", "scope = \"region:3\"");
    let f = only_finding(&doc);
    assert!(f.starts_with("rule 2"), "{f}");
    assert!(f.contains("no exception form"), "{f}");
}

#[test]
fn rule_3_a_dimension_mismatch_names_both_sides() {
    let doc = format!(
        "{}{}",
        entry("unit = \"count\"\nprovenance = \"structural\"\nidentity = \"RegionCount\""),
        "[[entry]]\nname = \"t.y\"\nnamespace = \"model\"\nvalue = 1.0\nunit = \"hour\"\n\
         scope = \"world\"\nowner = \"seed\"\nprovenance = \"derived\"\n\
         expression = \"t.x\"\njustification = \"a mechanism inside the model\"\n"
    );
    let f = only_finding(&doc);
    assert!(f.starts_with("rule 3"), "{f}");
    assert!(
        f.contains("count") && f.contains("hour"),
        "both sides are named: {f}"
    );
}

#[test]
fn rule_4_an_unknown_identity_is_caught() {
    let f = only_finding(&entry(
        "unit = \"count\"\nprovenance = \"structural\"\nidentity = \"Invented\"",
    ));
    assert!(f.starts_with("rule 4"), "{f}");
    assert!(f.contains("seventeenth is an ADR"), "{f}");
}

#[test]
fn rule_5_a_value_outside_its_bracket_is_caught() {
    let f = only_finding(&entry(
        "unit = \"ratio\"\nprovenance = \"assumed\"\nbracket = [2.0, 3.0]\naxis = \"none\"",
    ));
    assert!(f.starts_with("rule 5"), "{f}");
    assert!(f.contains("outside its bracket"), "{f}");
}

#[test]
fn rule_6_a_literal_outside_the_closed_set_is_caught() {
    let doc = entry("unit = \"ratio\"\nprovenance = \"derived\"\nexpression = \"1 * 7\"");
    let f = only_finding(&doc);
    assert!(f.contains("rule 3/6"), "{f}");
    assert!(
        f.contains("wearing an expression's clothes"),
        "the finding says why: {f}"
    );
}

#[test]
fn rule_7_an_entry_nothing_reads_is_caught() {
    let doc = entry("unit = \"count\"\nprovenance = \"structural\"\nidentity = \"RegionCount\"")
        .replace("owner = \"seed\"", "owner = \"nobody\"");
    let f = only_finding(&doc);
    assert!(f.starts_with("rule 7"), "{f}");
    assert!(f.contains("correctly declared and unread"), "{f}");
}

#[test]
fn a_justification_citing_the_outside_world_is_caught() {
    let doc = entry("unit = \"ratio\"\nprovenance = \"structural\"\nidentity = \"RegionCount\"")
        .replace(
            "a mechanism inside the model",
            "observed in the euro area, see https://example.org",
        );
    let f = only_finding(&doc);
    assert!(f.contains("outside the model"), "{f}");
}

// ── §13.3's generator, pinned ───────────────────────────────────────────────────────────────────

#[test]
fn the_seed_generators_continuous_half_reproduces_and_its_integer_half_does_not() {
    // A committed finding, not a passing test dressed as one. It asserts the state of §13.3 as
    // measured, so that regenerating the table breaks this test rather than passing silently — which
    // is the failure mode a "known red" note in a document always eventually has.
    let (report, disagreements) = aurora_tools::seedgen::report();

    assert!(
        report.contains("computed: 0.147152 0.201244 0.275219 0.376386"),
        "the axis-1 shares must still reproduce exactly; if they do not, the formula changed:\n{report}"
    );
    assert!(
        report.contains("§6.3 rule two reproduces 1 of 8 rows; largest-remainder reproduces 1."),
        "the integer half still reproduces under neither rule. If this count has MOVED, §13.3 was \
         regenerated and this test is what should be updated:\n{report}"
    );
    assert!(
        disagreements
            .iter()
            .any(|d| d.contains("cohort shares are invariant")),
        "the cohort-share invariance is structural and cannot be fixed by regenerating: {disagreements:?}"
    );
}

#[test]
fn b1b_rejects_most_stationary_series_and_the_conjunction_never_passes() {
    // A committed finding. §15.3's gate makes reaching period 520 without a pass a DEFECT, so a gate
    // that a settled world cannot pass does not delay a result — it condemns a healthy model.
    let report = aurora_tools::burnin::report();
    assert!(
        report.contains("all 42 series pass both B1 and B1b together in 0 panels"),
        "the conjunction still never passes on stationary series; if it now does, §15.3's thresholds \
         were changed and this test is what should be updated:\n{report}"
    );
    assert!(
        report.contains("rejects the majority of genuinely stationary series"),
        "B1b's 0.25 band is still not a five-per-cent test:\n{report}"
    );
}

// ── W5.3's coupling, both halves ────────────────────────────────────────────────────────────────
//
// The claim is that a value change is refused on a ratified file and free on an unratified one, so
// both halves are fixtures. Each builds a real git repository, because the check reads history — a
// mock would be testing the mock.

/// A scratch repository with `path` committed at `first`, then rewritten to `second` in the working
/// tree. `state` is what `coupling.toml` declares the file to be.
fn coupled(name: &str, path: &str, state: &str, first: &str, second: &str) -> std::path::PathBuf {
    let root = scratch(name);
    let git = |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(args)
            .output()
            .expect("git runs")
            .status
            .success();
        assert!(ok, "git {args:?} failed");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "fixture@example.invalid"]);
    git(&["config", "user.name", "fixture"]);
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    fs::create_dir_all(root.join(path).parent().expect("path has a parent"))
        .expect("scratch target dir");
    fs::write(
        root.join("decisions/coupling.toml"),
        format!("[[file]]\npath = \"{path}\"\nstate = \"{state}\"\nmilestone = \"M0\"\n"),
    )
    .expect("scratch coupling is writable");
    fs::write(root.join(path), first).expect("scratch target is writable");
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "first"]);
    fs::write(root.join(path), second).expect("scratch target is rewritable");
    root
}

#[test]
fn a_value_change_on_a_ratified_file_without_an_adr_is_refused() {
    let root = coupled(
        "coupling-ratified",
        "registry/entries.toml",
        "ratified",
        "[[entry]]\nname = \"a\"\nvalue = 1.0\n",
        "[[entry]]\nname = \"a\"\nvalue = 2.0\n",
    );
    let files = aurora_tools::check_coupling::registered(&root);
    let findings = aurora_tools::check_coupling::check(&root, &files);
    let [only] = findings.as_slice() else {
        panic!("the unauthorised value change is the one finding: {findings:?}")
    };
    assert!(
        only.contains("value=2.0") || only.contains("value=1.0"),
        "{only}"
    );
    assert!(only.contains("no ADR in this tree"), "{only}");
}

#[test]
fn the_same_change_on_a_draft_file_is_free() {
    let root = coupled(
        "coupling-draft",
        "registry/entries.toml",
        "draft",
        "[[entry]]\nname = \"a\"\nvalue = 1.0\n",
        "[[entry]]\nname = \"a\"\nvalue = 2.0\n",
    );
    let files = aurora_tools::check_coupling::registered(&root);
    assert!(
        aurora_tools::check_coupling::check(&root, &files).is_empty(),
        "a draft file's values move without a decision — that is what draft means"
    );
}

#[test]
fn prose_on_a_ratified_file_is_never_a_value() {
    let root = coupled(
        "coupling-prose",
        "registry/entries.toml",
        "ratified",
        "# a comment\n[[entry]]\nname = \"a\"\nvalue = 1.0\njustification = \"because\"\n",
        "# a different comment\n[[entry]]\nname = \"a\"\nvalue = 1.0\njustification = \"because, at length\"\n",
    );
    let files = aurora_tools::check_coupling::registered(&root);
    assert!(
        aurora_tools::check_coupling::check(&root, &files).is_empty(),
        "a justification is how a ratified value gets explained without being changed"
    );
}

#[test]
fn an_adr_naming_the_file_authorises_the_change() {
    let root = coupled(
        "coupling-authorised",
        "registry/entries.toml",
        "ratified",
        "[[entry]]\nname = \"a\"\nvalue = 1.0\n",
        "[[entry]]\nname = \"a\"\nvalue = 2.0\n",
    );
    fs::write(
        root.join("decisions/ADR-0001-x.md"),
        "---\nid: ADR-0001\nregisters: registry/entries.toml\n---\n\n## Decision\n\nTwo.\n",
    )
    .expect("scratch adr is writable");
    let files = aurora_tools::check_coupling::registered(&root);
    assert!(
        aurora_tools::check_coupling::check(&root, &files).is_empty(),
        "an ADR that names the file and is new in this tree is what the rule asks for"
    );
}

// ── W5.4: a hand edit to a generated guard cell does not survive ────────────────────────────────

#[test]
fn a_hand_edited_guard_cell_is_caught() {
    let root = scratch("appendix-hand-edit");
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    fs::write(
        root.join("decisions/ADR-0001-x.md"),
        "---\nid: ADR-0001\nregister-entry: 4\nguard: the typed handle\n---\n\n## Decision\n\nFour.\n",
    )
    .expect("scratch adr is writable");
    let table = "| # | Decision | Current value | Guard |\n|---|---|---|---|\n\
                 | 4 | The write model | three doors | the typed handle (ADR-0001) |\n";
    fs::write(root.join("PROJECT_AURORA.md"), table).expect("scratch spec is writable");

    let (findings, rows, claimed) = aurora_tools::appendix::check(&root);
    assert!(
        findings.is_empty(),
        "the generated table agrees: {findings:?}"
    );
    assert_eq!((rows, claimed), (1, 1));

    // Now edit the cell by hand, the way someone tidying prose would.
    let edited = table.replace("the typed handle (ADR-0001)", "handles, basically");
    fs::write(root.join("PROJECT_AURORA.md"), edited).expect("scratch spec is rewritable");
    let (findings, _, _) = aurora_tools::appendix::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the hand edit is the one finding: {findings:?}")
    };
    assert!(
        only.contains("differs from what the decisions say"),
        "{only}"
    );
}

#[test]
fn an_adr_claiming_a_row_that_does_not_exist_is_caught() {
    let root = scratch("appendix-missing-row");
    fs::create_dir_all(root.join("decisions")).expect("scratch decisions dir");
    fs::write(
        root.join("decisions/ADR-0001-x.md"),
        "---\nid: ADR-0001\nregister-entry: 99\nguard: the typed handle\n---\n\n## Decision\n\n.\n",
    )
    .expect("scratch adr is writable");
    fs::write(
        root.join("PROJECT_AURORA.md"),
        "| # | Decision | Current value | Guard |\n|---|---|---|---|\n| 4 | m | v | g |\n",
    )
    .expect("scratch spec is writable");

    let (findings, _, _) = aurora_tools::appendix::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the dangling register-entry is the one finding: {findings:?}")
    };
    assert!(only.contains("names no row in Appendix A"), "{only}");
}

// ── the two rules ADR-0006 and ADR-0012 added ───────────────────────────────────────────────────

#[test]
fn a_release_profile_without_overflow_checks_is_caught() {
    let root = scratch("lints-no-overflow-checks");
    crate_at(&root, "kernel", "#![forbid(unsafe_code)]\n");
    fs::write(
        root.join("Cargo.toml"),
        "[profile.release]\nlto = \"fat\"\npanic = \"abort\"\n",
    )
    .expect("scratch manifest is rewritable");

    let (findings, _, _, _) = aurora_tools::check_lints::check(&root);
    let [only] = findings.as_slice() else {
        panic!("the missing overflow-checks is the one finding: {findings:?}")
    };
    assert!(only.contains("overflow-checks = true"), "{only}");
    assert!(
        only.contains("wrap silently"),
        "and says what the consequence is: {only}"
    );
}

#[test]
fn a_shared_mutability_primitive_in_a_layer_crate_is_caught() {
    let root = scratch("lints-refcell");
    crate_at(
        &root,
        "world",
        "#![forbid(unsafe_code)]\nuse std::cell::RefCell;\npub struct A(RefCell<u8>);\n",
    );

    let (findings, _, _, _) = aurora_tools::check_lints::check(&root);
    assert!(
        findings
            .iter()
            .any(|f| f.contains("`RefCell`") && f.contains("ADR-0012")),
        "the arena must stay thread-shareable: {findings:?}"
    );
}

#[test]
fn a_comment_naming_refcell_is_not_a_finding() {
    let root = scratch("lints-refcell-comment");
    crate_at(
        &root,
        "world",
        "#![forbid(unsafe_code)]\n//! Deliberately no RefCell here.\n/// Not a RefCell either.\npub struct A;\n",
    );

    let (findings, _, _, _) = aurora_tools::check_lints::check(&root);
    assert!(
        findings.is_empty(),
        "prose naming the type is not a use of it: {findings:?}"
    );
}
