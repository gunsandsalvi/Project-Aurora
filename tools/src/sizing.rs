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
        self.count?.checked_mul(self.width?)
    }
}

/// The memory derivation N4 is owed.
#[allow(clippy::too_many_lines)]
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
            count: Some(ever_issued()),
            width: Some(4),
            source: "§5.2, counted bottom up below",
        },
        Table {
            name: "instruments",
            count: Some(1_060_000),
            width: Some(80),
            source: "ADR-0009: schedule and both price epochs inline",
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
        "  {:<40} {:>12} {:>7} {:>12}  source",
        "table", "count", "width", "MiB"
    );
    let mut known = 0u64;
    let mut unknown = 0usize;
    for t in &tables {
        if let Some(b) = t.bytes() {
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
        } else {
            unknown += 1;
            let _ = writeln!(
                out,
                "  {:<40} {:>12} {:>7} {:>12}  {}",
                t.name, "\u{2014}", "\u{2014}", "OWED", t.source
            );
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
         So {:.1} MiB of the published derivation is in rows that cannot yet be computed. The\n  \
         instruments table is no longer among them: ADR-0009 settled the row at 80 B with the\n  \
         schedule inline, and §7.5's family counts are usable for sizing.",
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
        if let Some(l) = self.life
            && l > 0
        {
            self.live + self.live * (1_560 / l)
        } else {
            self.live
        }
    }
}

/// The identity spaces, and how many identifiers each issues over a 1,560-tick run.
fn spaces() -> Vec<Space> {
    vec![
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
        // ADR-0009 removed the schedules space: a schedule is a field of its instrument, not a
        // claim, so nothing holds one and nothing can address one. It was 6,360,000 identifiers —
        // 40% of this census — for rows that were never addressable.
    ]
}

/// Every identifier issued over the run, across all spaces.
fn ever_issued() -> u64 {
    spaces().iter().map(Space::ever).sum()
}

/// §3.4's identifier census against §5.2's directory.
#[allow(clippy::too_many_lines)]
fn identifiers() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n§5.2 — the identifier census, bottom up\n");

    let spaces = spaces();

    let _ = writeln!(
        out,
        "  {:<22} {:>10} {:>8} {:>14}  basis",
        "space", "live", "life", "ever issued"
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
    for line in [
        String::new(),
        format!("  ever issued, bottom up: {total}   directory at 4 B each: {dir_mib:.1} MiB"),
        "  ADR-0009 removed the schedules space: 6,360,000 identifiers, 40% of the former census."
            .to_owned(),
        String::new(),
        "  RESOLVED. \u{a7}3.4's \u{2248} 971,000 was below the opening entity count plus one generation of".to_owned(),
        "  employment contracts alone: 350,000 live at a six-year mean term issue about 1.75 M".to_owned(),
        "  identifiers over thirty years without a single loan, tenancy or bond. \u{a7}3.4 now carries this".to_owned(),
        "  census's figure, and \u{a7}5.2's 47.5 MiB directory (implying \u{2248} 12,451,840) is superseded by the".to_owned(),
        "  bottom-up count above.".to_owned(),
        String::new(),
        "  STILL OWED: the mean lives. Six years for an employment contract, four for a tenancy, ten".to_owned(),
        "  for household credit, five for a corporate facility, one margin cycle for a lien \u{2014} every one".to_owned(),
        "  of them an assumption standing in for a mechanism that does not exist yet, and each replaced".to_owned(),
        "  by the milestone that builds the instrument. The figure is an order of magnitude, not a".to_owned(),
        "  number, and the 32-bit identifier is chosen with that in mind.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// One field of the journal row: its name, its type, its width, and why it is that wide.
struct Field {
    name: &'static str,
    kind: &'static str,
    bytes: usize,
    why: &'static str,
}

/// §6.6's 48-byte journal row, derived field by field (W7.5).
///
/// The row must accommodate `exchange`, which is the widest of the nine operations: §6.4 requires it
/// to name two parties, two assets, two quantities, the cleared rate, the realised rate, a reason code
/// and an actor. Written out, that does not fit — and the field that does not fit is the one that
/// should never have been stored.
fn journal() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n§6.6 — the journal row, derived field by field\n");

    let fields = [
        Field {
            name: "quantityGiven",
            kind: "i64",
            bytes: 8,
            why: "a conserved quantity is i64 and overflow panics (Appendix A #2)",
        },
        Field {
            name: "quantityReceived",
            kind: "i64",
            bytes: 8,
            why: "zero on a one-sided operation; equal and opposite is not assumed",
        },
        Field {
            name: "clearedRate",
            kind: "i64, fixed point at S",
            bytes: 8,
            why: "a rate spans several orders either side of one; 32 bits cannot carry it at S",
        },
        Field {
            name: "from",
            kind: "EntityId (u32)",
            bytes: 4,
            why: "15,732,835 identifiers ever issued needs more than 24 bits",
        },
        Field {
            name: "to",
            kind: "EntityId (u32)",
            bytes: 4,
            why: "as above",
        },
        Field {
            name: "assetGiven",
            kind: "InstrumentId (u32)",
            bytes: 4,
            why: "as above",
        },
        Field {
            name: "assetReceived",
            kind: "InstrumentId (u32)",
            bytes: 4,
            why: "equal to assetGiven on a move; the row has one shape",
        },
        Field {
            name: "tick",
            kind: "u16",
            bytes: 2,
            why: "1,560 < 65,536; the row is self-describing when saved, and §9.3 reads a prior close",
        },
        Field {
            name: "op",
            kind: "u8",
            bytes: 1,
            why: "nine operations (Appendix A #4)",
        },
        Field {
            name: "reason",
            kind: "u8",
            bytes: 1,
            why: "§6.6",
        },
        Field {
            name: "actor",
            kind: "u8",
            bytes: 1,
            why: "from the minted handle, never an argument (§6.6)",
        },
    ];
    let _ = writeln!(
        out,
        "  field                type                     B  why"
    );
    let mut total = 0usize;
    for f in &fields {
        total += f.bytes;
        let _ = writeln!(
            out,
            "  {:<19}  {:<22} {:>3}  {}",
            f.name, f.kind, f.bytes, f.why
        );
    }
    // The widest field is i64, so the struct aligns to 8.
    let align = 8;
    let padding = (align - total % align) % align;
    let width = total + padding;
    let _ = writeln!(
        out,
        "  {:<19}  {:<22} {:>3}  alignment to the widest field, i64",
        "(padding)", "", padding
    );
    let _ = writeln!(out, "  {:<19}  {:<22} {:>3}", "TOTAL", "", width);

    out.push_str(&journal_finding(width));
    out
}

/// One line a household can hold at once: the instrument type, how many lines of it, and why.
struct Occupancy {
    kind: &'static str,
    mean: usize,
    tail: usize,
    why: &'static str,
}

/// §3.4's ten-slot household block, against what a household can simultaneously hold (W7.6).
///
/// A slot is one `(holder, asset)` holding, and under R-1 a claim exists only as its issuer's negative
/// balance — so a household's own borrowing occupies its slots as surely as its deposits do. The
/// question is therefore not how many *types* a household touches but how many *lines*, and the answer
/// is a sum over the eight opening types rather than a count of them.
fn household_block() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\n\u{a7}3.4 — the household block, against what one household can hold at once\n"
    );

    let lines = [
        Occupancy {
            kind: "CurrencyClaim",
            mean: 1,
            tail: 1,
            why: "its own region's currency",
        },
        Occupancy {
            kind: "DepositLine",
            mean: 1,
            tail: 2,
            why: "\u{a7}7.7: one line per ISSUER, so an account at a second bank is a second slot",
        },
        Occupancy {
            kind: "SecuredTermLoan",
            mean: 1,
            tail: 2,
            why: "negative, by R-1. 300,000 household credit lines over 500,000 households (\u{a7}5.2), so two is inside the tail",
        },
        Occupancy {
            kind: "EmploymentContract",
            mean: 1,
            tail: 2,
            why: "350,000 live over 500,000 households; a two-earner household holds two",
        },
        Occupancy {
            kind: "SovereignBond",
            mean: 0,
            tail: 1,
            why: "four tenors per region are directly holdable; a household holding one is not excluded",
        },
        Occupancy {
            kind: "ListedEquity",
            mean: 0,
            tail: 1,
            why: "1,000 lines, held directly rather than through a fund",
        },
        Occupancy {
            kind: "PrivateEquity",
            mean: 0,
            tail: 1,
            why: "50,000 unlisted firms over 500,000 households: one in ten owns one",
        },
        Occupancy {
            kind: "GoodsUnit",
            mean: 1,
            tail: 7,
            why: "ONE PER SECTOR, and \u{a7}9.5 has seven. This is the term that decides the block",
        },
    ];
    let _ = writeln!(out, "  type                  mean   tail  why");
    let (mut mean, mut tail) = (0usize, 0usize);
    for l in &lines {
        mean += l.mean;
        tail += l.tail;
        let _ = writeln!(
            out,
            "  {:<20} {:>5}  {:>5}  {}",
            l.kind, l.mean, l.tail, l.why
        );
    }
    let _ = writeln!(out, "  {:<20} {mean:>5}  {tail:>5}", "TOTAL");
    out.push_str(&household_finding(mean, tail));
    out
}

/// What the enumeration costs, at the capacities that could hold it.
fn household_finding(mean: usize, tail: usize) -> String {
    let mut out = String::new();
    let households = 500_000usize;
    let slot = 24usize;
    let declared = 10usize;
    #[allow(clippy::cast_precision_loss)]
    let mb = |slots: usize| (households * slots * slot) as f64 / 1e6;
    let without_goods = tail - 7;

    for line in [
        String::new(),
        format!("  \u{a7}3.4 declares {declared} slots. The mean household needs {mean}, which is \u{a7}3.4's own"),
        "  \"about three of its ten\" and is not the question: \u{a7}3.4 says blocks are sized for the TAIL,".to_owned(),
        "  not the mean, BECAUSE EXHAUSTION IS A HALT.".to_owned(),
        String::new(),
        format!("  The tail needs {tail}. Without a goods stock it needs {without_goods} \u{2014} which is {declared}"),
        "  exactly, with nothing spare, on a block whose whole justification is headroom.".to_owned(),
        String::new(),
        "  capacity   slots      MB   holds".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    for (capacity, holds) in [
        (10usize, "the mean; not the tail, with or without goods"),
        (16, "the tail without a goods stock, with 6 spare"),
        (20, "the tail with goods in 3 sectors"),
        (
            32,
            "the full tail, and the next two instrument types M9 adds",
        ),
    ] {
        let _ = writeln!(
            out,
            "  {capacity:>8}  {:>6}  {:>6.1}  {holds}",
            households * capacity,
            mb(capacity)
        );
    }

    for line in [
        String::new(),
        "  FINDING. The block turns on ONE UNSETTLED QUESTION: does a household hold a goods stock".to_owned(),
        "  between operations, or is a purchase consumed inside the same tick that bought it? Seven".to_owned(),
        "  sectors means the answer is worth 7 slots per household, and 7 slots per household is".to_owned(),
        format!("  {:.1} MB \u{2014} more than the entire holdings table costs today. Nothing in \u{a7}9 answers it.", mb(7)),
        String::new(),
        "  And the enumeration is over the EIGHT OPENING TYPES ONLY. \u{a7}5.2's identifier census counts".to_owned(),
        "  250,000 live tenancies, and there is no tenancy instrument type; the pension and insurance".to_owned(),
        "  claims \u{a7}8.4's liability-matched institution issues have no type either. Each is at least one".to_owned(),
        format!("  more slot on the households that hold one, and at {:.1} MB per slot the block cannot", mb(1)),
        "  absorb them silently.".to_owned(),
        String::new(),
        "  Ten is not a derived figure. It is the only block capacity in \u{a7}3.4's table that is not a".to_owned(),
        "  power of two, which is what a number reached by looking at a mean looks like.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// §7.5's unresolved instrument row: 44 B with a schedule directory, against 148 B inline (W7.9).
///
/// §7.5 defers this to "measurement on the target device" and calls it a Phase 2 entry criterion. It
/// is neither. It decides whether the **schedule identity space exists at all**, and an identity space
/// is M1's; and it is decided by arithmetic, because the two arms differ by less than either arm's own
/// error bar while differing by 6.36 M identifiers.
fn instrument_row() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\n\u{a7}7.5 — the instrument row, both arms written out\n"
    );

    // §7.5's eleven columns, at the widths the rest of the specification forces.
    let core = [
        ("minPieceUnits", "i64", 8usize),
        ("issuer", "EntityId (u32)", 4),
        ("optionsFirst", "OptionTermsId (u32)", 4),
        ("holdCount", "u32", 4),
        ("venue", "u16", 2),
        ("maturityPeriod", "u16", 2),
        ("issuePeriod", "u16", 2),
        ("type", "u8", 1),
        ("currency", "u8", 1),
        ("status", "u8", 1),
    ];
    // A schedule as a GENERATING RULE rather than a list of dated rows.
    let schedule = [
        ("amount", "i64", 8usize),
        ("rateBasis", "SeriesId (u32)", 4),
        ("firstPeriod", "u16", 2),
        ("intervalPeriods", "u16", 2),
        ("count", "u16", 2),
        ("kind", "u8", 1),
    ];
    // Two price epochs: when the price last changed, and what it changed to.
    let epochs = [("price", "i64", 8usize), ("period", "u16", 2)];

    let sum = |fields: &[(&str, &str, usize)]| fields.iter().map(|f| f.2).sum::<usize>();
    let align8 = |n: usize| n.div_ceil(8) * 8;

    let _ = writeln!(
        out,
        "  core columns (\u{a7}7.5's eleven, less scheduleFirst)"
    );
    for (name, kind, bytes) in &core {
        let _ = writeln!(out, "    {name:<18} {kind:<20} {bytes:>3}");
    }
    let _ = writeln!(out, "    {:<18} {:<20} {:>3}", "subtotal", "", sum(&core));

    let _ = writeln!(out, "\n  a schedule, as a generating rule");
    for (name, kind, bytes) in &schedule {
        let _ = writeln!(out, "    {name:<18} {kind:<20} {bytes:>3}");
    }
    let _ = writeln!(
        out,
        "    {:<18} {:<20} {:>3}",
        "subtotal",
        "",
        sum(&schedule)
    );

    let _ = writeln!(
        out,
        "\n  a price epoch, of which the inline row carries two"
    );
    for (name, kind, bytes) in &epochs {
        let _ = writeln!(out, "    {name:<18} {kind:<20} {bytes:>3}");
    }
    let _ = writeln!(
        out,
        "    {:<18} {:<20} {:>3}  (x2, each aligned to 8)",
        "subtotal",
        "",
        align8(sum(&epochs))
    );

    let a_row = align8(sum(&core) + 4); // + scheduleFirst
    let a_schedule = align8(sum(&schedule));
    let b_row = align8(sum(&core) + sum(&schedule) + 2 * align8(sum(&epochs)));
    out.push_str(&instrument_finding(a_row, a_schedule, b_row));
    out
}

/// The two arms priced against the census, and what actually decides between them.
fn instrument_finding(a_row: usize, a_schedule: usize, b_row: usize) -> String {
    let mut out = String::new();
    let live = 1_060_000usize;
    let ever = 6_360_000usize;
    #[allow(clippy::cast_precision_loss)]
    let mb = |b: usize| b as f64 / 1e6;
    #[allow(clippy::cast_precision_loss)]
    let share = ever as f64 / 15_732_835.0 * 100.0;

    let a_total = live * a_row + live * a_schedule + ever * 4;
    let b_total = live * b_row;

    for line in [
        String::new(),
        format!("  A \u{2014} row {a_row} B + schedule {a_schedule} B out of line + a schedule identity space"),
        format!("      rows       {live} x {a_row:>3} B = {:>6.1} MB", mb(live * a_row)),
        format!("      schedules  {live} x {a_schedule:>3} B = {:>6.1} MB", mb(live * a_schedule)),
        format!("      directory  {ever} x   4 B = {:>6.1} MB   (\u{a7}5.2's ever-issued)", mb(ever * 4)),
        format!("      TOTAL                        {:>6.1} MB", mb(a_total)),
        String::new(),
        format!("  B \u{2014} row {b_row} B, schedule and two price epochs inline, NO schedule identity space"),
        format!("      rows       {live} x {b_row:>3} B = {:>6.1} MB", mb(live * b_row)),
        format!("      TOTAL                        {:>6.1} MB", mb(b_total)),
        String::new(),
        format!(
            "  Inline is {:.1} MB cheaper and removes {ever} identifiers \u{2014} {share:.0}% of the whole",
            mb(a_total - b_total)
        ),
        "  identifier census.".to_owned(),
        String::new(),
        "  FINDING 1. NEITHER PUBLISHED NUMBER IS THE ANSWER. \u{a7}7.5's eleven columns come to 33 B and".to_owned(),
        format!("  pad to {a_row}, not 44. And the inline row comes to {b_row}, not 148 \u{2014} the 68 B of difference is"),
        "  what an ENUMERATED schedule costs: five dated rows and no more, which cannot express a".to_owned(),
        "  ten-year mortgage at weekly ticks. A schedule enumerated rather than generated is not a".to_owned(),
        format!("  148 B row at all: {live} instruments x 120 payments x 16 B is {:.1} GB.", mb(live * 120 * 16) / 1000.0),
        String::new(),
        "  FINDING 2. THE COMPARISON \u{a7}7.5 DEFERS CANNOT BE MADE AS WRITTEN. It says \"this table and".to_owned(),
        "  \u{a7}3.4.4 describe two different rows\" \u{2014} and \u{a7}3.4.4 is one of the eight dangling cross-".to_owned(),
        "  references M0 already found. The 148 B arm is specified in a section that does not exist, so".to_owned(),
        "  no measurement on any device could have settled it.".to_owned(),
        String::new(),
        "  FINDING 3. AND MEMORY IS NOT WHAT DECIDES IT. The two arms are within ten per cent of each".to_owned(),
        "  other, which is inside the error bar on the census's owed mean lives. What decides it is".to_owned(),
        "  whether a schedule needs an IDENTITY: under A2 an instrument is data, and under R-1 a claim".to_owned(),
        "  exists as its issuer's negative balance. NOTHING HOLDS A SCHEDULE. It is a field of the".to_owned(),
        "  instrument, not a claim on anyone, so it has no holder, no balance and no reason to be".to_owned(),
        "  addressable. \u{a7}7.4's prepayment and payment-holiday amend the INSTRUMENT, which already has".to_owned(),
        "  an identity. The 6.36 M schedule identifiers are an identity space for a thing that is not".to_owned(),
        "  a claim.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// Both derivations, as one report.
pub fn report() -> String {
    format!(
        "{}{}{}{}",
        memory(),
        identifiers(),
        journal(),
        household_block()
    ) + &instrument_row()
}

/// What the 48 buys, and what §6.4 asked for that does not fit.
fn journal_finding(width: usize) -> String {
    let mut out = String::new();
    let ring = 7_200_000usize;
    #[allow(clippy::cast_precision_loss)]
    let mb = |b: usize| b as f64 / 1e6;
    let with_realised = width + 8;
    for line in [
        String::new(),
        format!("  The ring is {ring} rows: {:.1} MB at {width} B, which is \u{a7}6.6's figure.", mb(ring * width)),
        String::new(),
        "  FINDING. It fits only because the REALISED RATE IS NOT STORED. \u{a7}6.4 says the row carries".to_owned(),
        "  \"the cleared rate and the realised rate\"; both, written out, come to 53 B and pad to".to_owned(),
        format!("  {with_realised} B \u{2014} a ring of {:.1} MB, {:.1} MB more.", mb(ring * with_realised), mb(ring * (with_realised - width))),
        String::new(),
        "  And it should not be stored, on the specification's own principle. The realised rate is".to_owned(),
        "  quantityReceived / quantityGiven, exactly, by definition \u{2014} the pair IS the realised rate,".to_owned(),
        "  at full precision, while a stored copy is that value rounded once more under \u{a7}6.3. Two".to_owned(),
        "  copies of one value is the failure \u{a7}16.1 exists to prevent; nothing was applying it to the".to_owned(),
        "  journal. The CLEARED rate is different and is stored: it is the venue's price, which is not".to_owned(),
        "  a function of this row's own quantities when a line rations.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}
