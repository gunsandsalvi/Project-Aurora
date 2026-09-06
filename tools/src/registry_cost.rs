//! # `registry-cost` — what one economic system costs the registry
//!
//! W7.7. D3 withdrew the cap on how many `assumed` entries there may be and replaced it with a
//! published count that review pushes down. That makes one number the important one and nobody has
//! measured it: **the rate at which an economic system consumes assumptions.** M3 needs it to know
//! whether the census is heading for thirty entries or three hundred.
//!
//! So the complete registry for **credit** is written out here, entry by entry, with each one's
//! provenance decided under §16.1's rules rather than assigned by habit. The interesting rows are not
//! the assumptions — they are the entries that a mechanism *removes*, because those are what the rate
//! actually turns on.

use core::fmt::Write as _;

/// One entry the credit system needs, and what §16.1 makes of it.
struct Entry {
    name: &'static str,
    provenance: Provenance,
    note: &'static str,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Provenance {
    /// Definitional arithmetic or a closed decision about the shape of the world.
    Structural,
    /// An expression over other entries, dimension-checked.
    Derived,
    /// Chosen, with a bracket and an axis. The count that matters.
    Assumed,
    /// A mechanism answers it, so there is no entry at all. The count that matters more.
    Absorbed,
}

impl Provenance {
    fn label(self) -> &'static str {
        match self {
            Provenance::Structural => "structural",
            Provenance::Derived => "derived",
            Provenance::Assumed => "ASSUMED",
            Provenance::Absorbed => "— absorbed",
        }
    }
}

/// Everything the credit system needs a number for.
fn credit() -> Vec<Entry> {
    use Provenance::{Absorbed, Assumed, Derived, Structural};
    vec![
        Entry {
            name: "credit.rehypothecation_depth",
            provenance: Structural,
            note: "3 (Appendix A #5). Already an entry; credit does not add it",
        },
        Entry {
            name: "credit.schedule_kinds",
            provenance: Structural,
            note: "the closed set ADR-0009's `kind` field admits: bullet, level, amortising, floating",
        },
        Entry {
            name: "credit.rate",
            provenance: Derived,
            note: "§8.1 declaration 4: funding cost plus the spread the bank's capital headroom implies. Both are reads of its own block",
        },
        Entry {
            name: "credit.spread",
            provenance: Derived,
            note: "a read of funding, capital, risk aversion and inventory (§8.8's dealer rule, and the same shape here)",
        },
        Entry {
            name: "credit.capital_floor",
            provenance: Absorbed,
            note: "§21: a REGULATORY level, so the parliament sets it. Its opening value is the opening parliament's, which is derived from households' own rows. No registry entry",
        },
        Entry {
            name: "credit.ltv_cap",
            provenance: Absorbed,
            note: "as above — a plank, not a parameter",
        },
        Entry {
            name: "credit.risk_weight",
            provenance: Derived,
            note: "a function of §7.2 Q11's shortfall rank, which is a FACT on the instrument. §8.1 forbids a constraint over instrument types, and reading the rank is what makes that possible",
        },
        Entry {
            name: "credit.default_threshold",
            provenance: Absorbed,
            note: "there is none. A borrower defaults when it cannot pay, which is a balance test at position 18, not a level. THIS IS THE LARGEST SINGLE SAVING",
        },
        Entry {
            name: "credit.recovery_rate",
            provenance: Absorbed,
            note: "produced by §6.7's waterfall over what the estate actually holds. A rate here would be a second answer to a question the wind-up already answers",
        },
        Entry {
            name: "credit.maturity_at_origination",
            provenance: Absorbed,
            note: "the borrower asks for the term its own cash-flow profile implies. A distribution over terms would be a prior about borrowers the model is supposed to produce",
        },
        Entry {
            name: "credit.demand",
            provenance: Absorbed,
            note: "firms' plans. A credit demand parameter would be the calibration A3 forbids",
        },
        Entry {
            name: "credit.collateral_haircut",
            provenance: Derived,
            note: "from the trailing volatility of the collateral's own posted price — but see the window below",
        },
        Entry {
            name: "credit.haircut_window",
            provenance: Assumed,
            note: "how many periods of price history the haircut reads. Bracket [13, 104]; axis: none, it is a world constant",
        },
        Entry {
            name: "credit.margin_cycle",
            provenance: Assumed,
            note: "how often collateral is re-tested (§9.4 position 16). Bracket [4, 52]",
        },
        Entry {
            name: "credit.cure_window",
            provenance: Assumed,
            note: "§6.7's window before insolvency. Bracket [1, 13]",
        },
        Entry {
            name: "credit.capital_buffer_target",
            provenance: Assumed,
            note: "how far above the floor a bank aims to sit — §8.1 declaration 5, and a genuine behavioural choice. Bracket [0.00, 0.05]",
        },
        Entry {
            name: "credit.origination_minimum",
            provenance: Assumed,
            note: "the smallest loan that may be written, in minor units. §7.5 requires a piece size; below it the arithmetic of a schedule stops meaning anything. Bracket [1, 10^6]",
        },
        Entry {
            name: "credit.rationing_rule",
            provenance: Assumed,
            note: "when demand exceeds a bank's headroom, who is refused. §6.3's rules cover rounding, not rationing, and NOTHING in the specification covers this",
        },
    ]
}

/// The count, and what it implies for the census.
#[must_use]
pub fn report() -> String {
    let mut out = String::new();
    let entries = credit();
    let _ = writeln!(
        out,
        "\n§16.1 — the registry cost of ONE economic system: credit\n"
    );
    let _ = writeln!(out, "  entry                             provenance   why");
    for e in &entries {
        let _ = writeln!(
            out,
            "  {:<33} {:<12} {}",
            e.name,
            e.provenance.label(),
            e.note
        );
    }

    let count = |p: Provenance| entries.iter().filter(|e| e.provenance == p).count();
    let (structural, derived, assumed, absorbed) = (
        count(Provenance::Structural),
        count(Provenance::Derived),
        count(Provenance::Assumed),
        count(Provenance::Absorbed),
    );
    let _ = writeln!(
        out,
        "\n  {} questions credit has to answer: {structural} structural, {derived} derived, {assumed} ASSUMED, {absorbed} absorbed by a mechanism",
        entries.len()
    );
    out.push_str(&extrapolation(assumed, absorbed, entries.len()));
    out
}

/// What the rate implies, and where it comes from.
fn extrapolation(assumed: usize, absorbed: usize, total: usize) -> String {
    let mut out = String::new();
    // The economic systems §19 and IMPLEMENTATION.md between them name.
    let systems = [
        "credit",
        "labour",
        "goods",
        "housing",
        "equity",
        "money market and FX",
        "insurance and pension",
        "fiscal",
        "monetary",
    ];
    let projected = assumed * systems.len();
    #[allow(clippy::cast_precision_loss)]
    let absorption = absorbed as f64 / total as f64 * 100.0;

    for line in [
        String::new(),
        format!("  THE RATE. Credit costs {assumed} assumed entries. Across the {} economic systems", systems.len()),
        format!("  ({}),", systems.join(", ")),
        format!("  that projects to about {projected} — against the 9 the census carries today."),
        String::new(),
        "  That is the number M3 needs, and the projection is a floor rather than a forecast: labour".to_owned(),
        "  and housing each have a search process credit does not, and the fiscal system has a tax".to_owned(),
        "  schedule. Under D3 there is no cap, so the figure is not a breach — it is the slope.".to_owned(),
        String::new(),
        format!("  THE INTERESTING NUMBER IS THE OTHER ONE. {absorbed} of {total} questions ({absorption:.0}%) have NO"),
        "  registry entry at all, because a mechanism answers them. Those are what the rate actually".to_owned(),
        "  turns on, and three of them are worth naming:".to_owned(),
        String::new(),
        "  - THE DEFAULT THRESHOLD DOES NOT EXIST. A borrower defaults when it cannot pay, which is a".to_owned(),
        "    balance test at position 18. Every model that carries a default probability carries a".to_owned(),
        "    prior about an outcome; this one carries an arithmetic test about a state.".to_owned(),
        "  - THE RECOVERY RATE DOES NOT EXIST. §6.7's waterfall over what the estate actually holds".to_owned(),
        "    produces it. A parameter here would be a second answer to a question already answered.".to_owned(),
        "  - THE REGULATORY LEVELS ARE THE PARLIAMENT'S. §21 makes the capital floor and the LTV cap".to_owned(),
        "    planks rather than parameters, and their opening values are the opening parliament's,".to_owned(),
        "    which is derived from households' own rows. THE POLITICAL SYSTEM IS A REGISTRY SAVING,".to_owned(),
        "    which is not why it was added and is the strongest argument for it.".to_owned(),
        String::new(),
        "  FINDING. One of the six assumed entries has no source at all. `credit.rationing_rule` —".to_owned(),
        "  when demand exceeds a bank's headroom, WHO IS REFUSED — is not in §6.3, which covers".to_owned(),
        "  rounding, nor in §9.2, which allocates against balances, nor in §9.3, which clears. The".to_owned(),
        "  behaviour run (W7.8) reaches it at tick 13 and answers it by lending the headroom to the".to_owned(),
        "  single borrower present, which works only because there is one. With two borrowers the".to_owned(),
        "  model has no rule, and rationing is exactly where a credit model's behaviour lives.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}
