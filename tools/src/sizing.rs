//! §12.1's N4 and §5.2's directory, derived rather than quoted.
//!
//! Two numbers in the specification cannot be reproduced from the sections that own their parts, and
//! both size the arena:
//!
//! - **N4's 1,488.3 MB** is published without an itemisation. §12.1 now says so; this computes what
//!   the named components actually come to and how much is unaccounted.
//! - **§3.4's ≈971,000 identifiers ever issued** against **§5.2's 47.5 MiB directory at four bytes an
//!   identifier**, which implies 12,451,840. A factor of thirteen, and the directory, the digest's
//!   identifier-order walk and the save all depend on which is right.
//!
//! Every row states its arithmetic. A row whose width or count is not yet decided is marked and
//! excluded from the total rather than guessed at, because a total that quietly includes a guess is
//! worse than one that is visibly short.

use std::fmt::Write as _;

const MIB: f64 = 1_048_576.0;

/// One arena table: what it holds, how many, how wide, and where the figure comes from.
struct Table {
    name: &'static str,
    count: Option<u64>,
    width: Option<u64>,
    source: &'static str,
}

impl Table {
    fn bytes(&self) -> Option<u64> {
        Some(self.count?.checked_mul(self.width?)?)
    }
}

/// The memory derivation N4 is owed.
fn memory() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "N4 — the memory derivation, one row per table\n");

    let tables = [
        Table {
            name: "agent rows — households",
            count: Some(500_000),
            width: Some(160),
            source: "§8.4",
        },
        Table {
            name: "agent rows — unlisted firms",
            count: Some(50_000),
            width: Some(256),
            source: "§8.4",
        },
        Table {
            name: "agent rows — listed tier",
            count: Some(1_000),
            width: Some(1_024),
            source: "§8.4, ADR-0007",
        },
        Table {
            name: "agent rows — institutions",
            count: Some(182),
            width: Some(1_024),
            source: "§8.4: 56+54+40+24+4+4",
        },
        Table {
            name: "holdings slots",
            count: Some(7_177_280),
            width: Some(24),
            source: "§3.4, corrected slot",
        },
        Table {
            name: "journal ring",
            count: Some(7_200_000),
            width: Some(48),
            source: "§6.6",
        },
        Table {
            name: "re-plan bucket index",
            count: Some(550_622),
            width: Some(4),
            source: "§8.6",
        },
        Table {
            name: "wage index ring",
            count: Some(624),
            width: Some(8),
            source: "§9.6.2: 3×4×52",
        },
        Table {
            name: "flow-of-funds matrix",
            count: Some(6_728),
            width: Some(8),
            source: "§9.7: 29×29×4×2",
        },
        Table {
            name: "identifier directory",
            count: None,
            width: Some(4),
            source: "§5.2 — the count is the contradiction below",
        },
        Table {
            name: "instruments",
            count: None,
            width: None,
            source: "§7.5 unresolved: 44 B or 148 B, and the live count is owed",
        },
        Table {
            name: "schedules + rows + deltas",
            count: None,
            width: None,
            source: "§7.4 — owed",
        },
        Table {
            name: "instrument options + 7 terms tables",
            count: None,
            width: Some(16),
            source: "§7.6 — the chain row; the terms rows are owed",
        },
        Table {
            name: "liens",
            count: None,
            width: None,
            source: "§6.5 — owed",
        },
        Table {
            name: "plans + intents",
            count: None,
            width: None,
            source: "Appendix A #24 quotes ≈121 MB with no derivation",
        },
        Table {
            name: "resolution register",
            count: None,
            width: None,
            source: "§6.7 — owed",
        },
        Table {
            name: "observation store",
            count: None,
            width: None,
            source: "§14 — 2,048 series, retention owed",
        },
    ];

    let _ = writeln!(
        out,
        "  {:<40} {:>12} {:>7} {:>12}  {}",
        "table", "count", "width", "MiB", "source"
    );
    let mut known = 0u64;
    let mut unknown = 0usize;
    for t in &tables {
        match t.bytes() {
            Some(b) => {
                known += b;
                #[allow(clippy::cast_precision_loss)]
                let mib = b as f64 / MIB;
                let _ = writeln!(
                    out,
                    "  {:<40} {:>12} {:>7} {:>12.1}  {}",
                    t.name,
                    t.count.unwrap_or(0),
                    t.width.unwrap_or(0),
                    mib,
                    t.source
                );
            }
            None => {
                unknown += 1;
                let _ = writeln!(
                    out,
                    "  {:<40} {:>12} {:>7} {:>12}  {}",
                    t.name, "—", "—", "OWED", t.source
                );
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    let known_mib = known as f64 / MIB;
    let _ = writeln!(
        out,
        "\n  derivable today: {known_mib:.1} MiB across {} rows. {unknown} rows are OWED and excluded.",
        tables.len() - unknown
    );
    let _ = writeln!(
        out,
        "  §12.1 publishes 1,488.3 MB derived and N4 = 1,610 MB as the target.\n  \
         So {:.1} MiB of the published derivation is in rows that cannot yet be computed — and the\n  \
         largest of them, the instruments table, is the one §7.5 declares unresolved while also saying\n  \
         its own family counts are not to be used for sizing.",
        1_488.3 - known_mib
    );
    out
}

/// One identity space, and how many identifiers it issues over a 1,560-tick run.
struct Space {
    name: &'static str,
    live: u64,
    /// Mean life in ticks. `None` where the population does not turn over.
    life: Option<u64>,
    basis: &'static str,
}

impl Space {
    /// Ever issued over the run: the opening population plus one replacement per mean life.
    fn ever(&self) -> u64 {
        match self.life {
            None => self.live,
            Some(l) if l > 0 => self.live + self.live * (1_560 / l),
            Some(_) => self.live,
        }
    }
}

/// §3.4's identifier census against §5.2's directory.
fn identifiers() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n§5.2 — the identifier census, bottom up\n");

    let spaces = [
        Space {
            name: "entities",
            live: 550_622,
            life: None,
            basis: "§8.4 plus 40 counter-accounts; births and deaths are owed",
        },
        Space {
            name: "employment contracts",
            live: 350_000,
            life: Some(312),
            basis: "§9.6.2: 350,000 live, six-year mean term",
        },
        Space {
            name: "tenancies",
            live: 250_000,
            life: Some(208),
            basis: "§9.6.4; a four-year mean tenancy, assumed here and owed",
        },
        Space {
            name: "household credit",
            live: 300_000,
            life: Some(520),
            basis: "a ten-year mean mortgage or consumer loan, assumed here and owed",
        },
        Space {
            name: "corporate credit",
            live: 60_000,
            life: Some(260),
            basis: "a five-year mean facility, assumed here and owed",
        },
        Space {
            name: "sovereign issues",
            live: 16,
            life: Some(52),
            basis: "§9.5: four tenors, four regions, one tap a year",
        },
        Space {
            name: "deposit lines",
            live: 56,
            life: None,
            basis: "§7.7: one line per issuer, not one per holder",
        },
        Space {
            name: "equity lines",
            live: 1_000,
            life: None,
            basis: "one per listed firm (§8.7)",
        },
        Space {
            name: "venues",
            live: 37,
            life: None,
            basis: "§9.5",
        },
        Space {
            name: "series",
            live: 624,
            life: None,
            basis: "§14",
        },
        Space {
            name: "liens",
            live: 100_000,
            life: Some(52),
            basis: "collateral turns over on the margin cycle; assumed here and owed",
        },
        Space {
            name: "schedules",
            live: 1_060_000,
            life: Some(312),
            basis: "one per obligation-bearing instrument above",
        },
    ];

    let _ = writeln!(
        out,
        "  {:<22} {:>10} {:>8} {:>14}  {}",
        "space", "live", "life", "ever issued", "basis"
    );
    let mut total = 0u64;
    for s in &spaces {
        total += s.ever();
        let _ = writeln!(
            out,
            "  {:<22} {:>10} {:>8} {:>14}  {}",
            s.name,
            s.live,
            s.life.map_or("—".to_owned(), |l| l.to_string()),
            s.ever(),
            s.basis
        );
    }

    #[allow(clippy::cast_precision_loss)]
    let dir_mib = (total * 4) as f64 / MIB;
    let _ = writeln!(
        out,
        "\n  ever issued, bottom up: {total}   directory at 4 B each: {dir_mib:.1} MiB\n  \
         §3.4 says ≈971,000.  §5.2's 47.5 MiB implies 12,451,840.\n\n  \
         FINDING. §3.4's figure is not merely low, it is **below the opening entity count plus one\n  \
         generation of employment contracts alone**. It cannot be right: 350,000 live contracts at a\n  \
         six-year mean term issue about 1.75 M identifiers over thirty years without a single loan,\n  \
         tenancy or bond. §5.2's implied 12.45 M is the plausible one, and this census lands in the\n  \
         same order. The two figures differ by a factor of about thirteen and §3.4's is the wrong one.\n  \
         The rows marked owed above are the mean lives, which are assumed here to get an order of\n  \
         magnitude and must be derived once the instruments exist."
    );
    out
}

/// Both derivations, as one report.
pub fn report() -> String {
    format!("{}{}", memory(), identifiers())
}
