//! # `workload` — §3.4's 3,119,665 operation calls a tick, position by position
//!
//! W7.4. §3.4 states the figure as *"derived position by position rather than from stocks"* and names
//! four largest terms, and the derivation is not shown. §12's targets are decomposed against this
//! table, and §6.6's journal ring is sized by it: every operation appends exactly one row, so the
//! operation count and the journal count are the same number.
//!
//! **Clearing's cost is separated from operation cost, because they are not the same quantity.** A
//! clearing position sorts and matches; it appends no journal row. The operations it causes are
//! appended by the *settlement* position that follows it. Counting a sort among operation calls is
//! what makes §3.4's four largest terms fail to sum.
//!
//! Rows are derived from declared counts or marked OWED with what they need. A row that is guessed is
//! a row that cannot falsify anything.

use core::fmt::Write as _;

/// What one period position costs, in operations that append a journal row.
struct Position {
    /// §9.4's position number.
    number: u8,
    /// §9.4's name, shortened.
    name: &'static str,
    /// Operations per tick, or `None` when the model does not yet say.
    calls: Option<u64>,
    /// The arithmetic, or what is missing.
    basis: &'static str,
}

/// §8.4's populations, and the live instrument counts §5.2's census carries.
const UNLISTED: u64 = 50_000;
const LISTED_OPENING: u64 = 440;
const INSTITUTIONS: u64 = 56 + 54 + 24 + 40 + 4 + 4;
const ENTITIES: u64 = 550_622;
const EMPLOYMENT: u64 = 350_000;
const TENANCIES: u64 = 250_000;
const HOUSEHOLD_CREDIT: u64 = 300_000;
const CORPORATE_CREDIT: u64 = 60_000;
const LIVE_LIENS: u64 = 100_000;
const INSTANTIATED_LINES: u64 = 1_276;
const FUNDING_LINES: u64 = 10;
/// §2.1's run.
const RUN: u64 = 1_560;
/// §3.4's published total.
const PUBLISHED: u64 = 3_119_665;

/// §9.4's twenty-one positions, each priced from a declared count or marked OWED.
///
/// One literal entry per position, in §9.4's order. Splitting it to satisfy a line count would put
/// half of the period order in one function and half in another, which is exactly the shape §9.4
/// forbids: positions are stable names in one reviewed list.
#[allow(clippy::too_many_lines)]
fn positions() -> Vec<Position> {
    let firms = UNLISTED + LISTED_OPENING;
    // §5.2's census: everything issued over the run, less the entities and the structural lines.
    let issued_per_tick = (2_100_000 + 2_000_000 + 1_200_000 + 420_000 + 496) / RUN;
    vec![
        Position {
            number: 1,
            name: "Demography",
            calls: None,
            basis: "OWED — no birth or exit rate is declared anywhere; §5.2's census owes the same number",
        },
        Position {
            number: 2,
            name: "Endowment (land)",
            calls: None,
            basis: "OWED — who holds land, and whether it is endowed once or per tick, is not stated",
        },
        Position {
            number: 3,
            name: "Accrual and distribution",
            calls: Some(ENTITIES / 4),
            basis: "one accrual per deposit holding, ~1 per entity, on a 4-tick due bucket. THE CADENCE IS ASSUMED: at every tick it is 550,622",
        },
        Position {
            number: 4,
            name: "Obligation payment",
            calls: Some(EMPLOYMENT + TENANCIES / 4 + (HOUSEHOLD_CREDIT + CORPORATE_CREDIT) / 4),
            basis: "wages weekly (350,000); rents and amortisation on a 4-tick bucket. §3.4's 926,246 needs ALL of them weekly",
        },
        Position {
            number: 5,
            name: "Depreciation",
            calls: Some(firms),
            basis: "one Wear: move per capital-holding firm; dwellings are OWED, no count is declared",
        },
        Position {
            number: 6,
            name: "Production",
            calls: Some(327_886),
            basis: "§3.4's own figure: 6.5 moves per firm over 50,440 firms. Inputs-per-firm is OWED, so this is quoted, not derived",
        },
        Position {
            number: 7,
            name: "Policy and election",
            calls: Some(0),
            basis: "writes policy columns and seat counts; moves no conserved quantity",
        },
        Position {
            number: 8,
            name: "Valuation and constraints",
            calls: Some(0),
            basis: "writes `plans`, a world table (Appendix A #24)",
        },
        Position {
            number: 9,
            name: "Funding allocation",
            calls: Some(0),
            basis: "writes `intents`",
        },
        Position {
            number: 10,
            name: "Funding clearing",
            calls: Some(0),
            basis: "A SORT, NOT OPERATIONS. Its cost is separated below",
        },
        Position {
            number: 11,
            name: "Funding settlement",
            calls: Some(INSTITUTIONS * FUNDING_LINES / 2),
            basis: "one exchange per matched pair, over 182 participants on 10 lines",
        },
        Position {
            number: 12,
            name: "Spending allocation",
            calls: Some(0),
            basis: "writes `intents`",
        },
        Position {
            number: 13,
            name: "Spending clearing",
            calls: Some(0),
            basis: "A SORT, NOT OPERATIONS. §3.4 names it a largest TERM, which it cannot be",
        },
        Position {
            number: 14,
            name: "Spending settlement",
            calls: Some(1_671_884),
            basis: "§3.4's own figure. Sectors bought per household per tick is OWED, so this is quoted, not derived",
        },
        Position {
            number: 15,
            name: "Budget reconciliation",
            calls: Some(ENTITIES),
            basis: "one return move per entity per currency where an allocation is unspent. NOT IN §3.4's FOUR TERMS AND NOT VISIBLY ANYWHERE",
        },
        Position {
            number: 16,
            name: "Margin and collateral",
            calls: Some(LIVE_LIENS * 2 / 52),
            basis: "100,000 live liens, pledge and release, over a 52-tick margin cycle",
        },
        Position {
            number: 17,
            name: "Mark to market",
            calls: Some(INSTANTIATED_LINES + 54),
            basis: "one price post per instantiated line, plus one NAV per fund. SEE THE FINDING: per-POSITION marks are not affordable",
        },
        Position {
            number: 18,
            name: "Default testing",
            calls: None,
            basis: "OWED — the default rate is what the model produces, so this is zero until it runs",
        },
        Position {
            number: 19,
            name: "Primary issuance",
            calls: Some(issued_per_tick),
            basis: "§5.2's census: 5,720,496 instruments issued over 1,560 ticks",
        },
        Position {
            number: 20,
            name: "Projection",
            calls: Some(0),
            basis: "writes observations only",
        },
        Position {
            number: 21,
            name: "Obligation compaction",
            calls: Some(0),
            basis: "every 52nd tick; releases arena rows, writes nothing any system reads",
        },
    ]
}

/// The table, and the report.
#[must_use]
pub fn report() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "\n§3.4 — the operation count, position by position\n");

    let positions = positions();

    let _ = writeln!(out, "  #   position                     calls/tick  basis");
    let (mut total, mut owed) = (0u64, 0usize);
    for p in &positions {
        match p.calls {
            Some(n) => total += n,
            None => owed += 1,
        }
        let _ = writeln!(
            out,
            "  {:>2}  {:<26} {:>11}  {}",
            p.number,
            p.name,
            p.calls.map_or_else(|| "OWED".to_owned(), |n| n.to_string()),
            p.basis
        );
    }
    let _ = writeln!(
        out,
        "  {:>2}  {:<26} {total:>11}",
        "", "TOTAL, derivable today"
    );
    out.push_str(&findings(total, owed, positions.len()));
    out
}

/// What the table says that §3.4 does not.
fn findings(total: u64, owed: usize, positions: usize) -> String {
    let mut out = String::new();
    let named = 1_671_884u64 + 926_246 + 327_886;
    let marks_per_tick = 7_177_280u64;
    #[allow(clippy::cast_precision_loss)]
    let ring_ticks = 7_200_000.0 / marks_per_tick as f64;

    for line in [
        String::new(),
        format!("  {owed} of {positions} positions are OWED. The rest come to {total} against §3.4's"),
        format!("  published {PUBLISHED}."),
        String::new(),
        "  FINDING 1. THE FOUR LARGEST TERMS CANNOT SUM INTO THE TOTAL. §3.4 names position 14".to_owned(),
        "  (1,671,884), position 4 (926,246), position 13's clearing work, and position 6 (327,886)."
            .to_owned(),
        format!("  Three of those are {named}, leaving {} for position 13 AND the other", PUBLISHED - named),
        "  seventeen positions together — so position 13 cannot be among the four largest unless the".to_owned(),
        "  rest of the model costs almost nothing. It is not among them, because IT IS NOT AN".to_owned(),
        "  OPERATION COUNT AT ALL: a clearing position sorts and matches and appends no journal row.".to_owned(),
        "  The operations it causes are appended by the settlement that follows it, and those are".to_owned(),
        "  already position 14's figure. Counting a sort among operation calls is the error, and".to_owned(),
        "  separating the two is what W7.4 exists to do.".to_owned(),
        String::new(),
        "  FINDING 2. POSITION 15 IS NOT IN THE PUBLISHED DERIVATION. Budget reconciliation returns".to_owned(),
        "  unspent allocations per currency; §9.4 gives it a position, an owner and a phase. One".to_owned(),
        format!("  return per entity is {ENTITIES} operations — a fifth of the whole published total, and"),
        "  the largest term §3.4 does not mention. Either most allocations are spent to the unit, or".to_owned(),
        "  the figure is short by that much; the model does not yet say which.".to_owned(),
        String::new(),
        "  FINDING 3. A POSITION MARK CANNOT BE AN OPERATION. §6.6 says every operation appends".to_owned(),
        "  exactly one journal row, and `post` is one of the nine. §9.4's position 17 computes".to_owned(),
        format!("  \"position and portfolio marks\": one per holding is {marks_per_tick} posts a tick, against a"),
        format!("  journal ring of 7,200,000 rows that must hold TWO ticks of EVERYTHING — {ring_ticks:.2} ticks"),
        "  of marks and nothing else.".to_owned(),
        "  So a position mark is DERIVED ON READ from a posted line price and the holding's own".to_owned(),
        "  quantity, and never stored. The table above prices position 17 accordingly: one post per".to_owned(),
        format!("  instantiated line ({INSTANTIATED_LINES}) plus one NAV per fund."),
        String::new(),
        "  FINDING 4. POSITION 4's PUBLISHED FIGURE IMPLIES EVERY OBLIGATION PAYS WEEKLY. Wages,".to_owned(),
        format!("  rents and amortisation live are {}, and 926,246 is that number less a few per cent.", EMPLOYMENT + TENANCIES + HOUSEHOLD_CREDIT + CORPORATE_CREDIT),
        "  On a 4-tick bucket for rents and amortisation the position is 502,500 \\u{2014} 46% less. §9.4 says".to_owned(),
        "  the walk takes \"the standing bucket first\", so buckets exist; what is missing is which".to_owned(),
        "  bucket each obligation type falls in, and that is a modelling decision, not a rate.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out.push_str(&clearing_cost(total));
    out.push_str(&retirement());
    out
}

/// The retirement queue against what it has to hold (ADR-0017).
///
/// §5.5 fixes the queue at 65,536 entries and says **overflow is a defect** — a halt, not a wrap. So
/// its capacity is `max retirements per tick x the interval between drains`, and both terms are
/// countable from §5.2's census.
fn retirement() -> String {
    let mut out = String::new();
    // In steady state a space retires what it issues: the census's own flow, less the entities, which
    // are issued once and not replaced within the run.
    let instruments = (2_100_000 + 2_000_000 + 1_200_000 + 420_000 + 496) / RUN;
    let liens = 3_100_000 / RUN;
    let per_tick = instruments + liens;
    let queue = 65_536u64;
    let compaction_interval = 52u64;
    let needed = per_tick * compaction_interval;
    #[allow(clippy::cast_precision_loss)]
    let over = needed as f64 / queue as f64;
    #[allow(clippy::cast_precision_loss)]
    let ticks_to_fill = queue as f64 / per_tick as f64;

    for line in [
        String::new(),
        "  THE RETIREMENT QUEUE, against what it has to hold.".to_owned(),
        String::new(),
        format!("    instruments retired per tick   {instruments}   (§5.2's flow, steady state)"),
        format!("    liens retired per tick         {liens}"),
        format!("    total per tick                 {per_tick}"),
        format!("    §5.5's queue                   {queue}"),
        String::new(),
        "  FINDING. §5.5 pushes a retired identifier onto this queue and says `lifecycle` drains"
            .to_owned(),
        "  it AT POSITION 21 — which §9.4 runs EVERY 52ND TICK. Fifty-two ticks of retirements is"
            .to_owned(),
        format!("  {needed} entries against a capacity of {queue}: {over:.1}x over, and §5.5 says overflow is a"),
        format!("  DEFECT. The queue fills in {ticks_to_fill:.1} ticks and the run halts around tick 12."),
        String::new(),
        "  The capacity is right and the interval is wrong. Draining changes no quantity and appends".to_owned(),
        "  no journal row — §5.5 says relocation is not an operation for exactly that reason — so a".to_owned(),
        "  drain needs no slot in the committed order, and §9.4 already says RETIREMENT IS NOT A".to_owned(),
        "  POSITION. It is drained at the end of every tick by `lifecycle`, and position 21 keeps the".to_owned(),
        "  scope Appendix B gives it: obligation compaction only.".to_owned(),
        String::new(),
        format!("  At an interval of 1 the capacity is {per_tick} x 1, and {queue} carries {ticks_to_fill:.1} ticks of"),
        "  headroom for a burst — a resolution wave at position 18, or a maturity cluster at 19.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// Clearing's cost, in the unit clearing is actually paid in.
///
/// W7.4's second half. A clearing position appends no journal row, so it does not appear in the
/// operation count at all — but it is not free, and §12's targets are stated per *operation*. What it
/// costs is comparisons, and the two are separate quantities that must be budgeted separately.
fn clearing_cost(operations: u64) -> String {
    let mut out = String::new();
    let lines = INSTANTIATED_LINES - FUNDING_LINES;
    // Every fill matches two submissions, so submissions are at least twice the fills. A submission
    // that does not fill still has to be sorted, so this is a floor.
    let submissions = 2 * 1_671_884u64;
    let per_line = submissions / lines;
    #[allow(clippy::cast_precision_loss)]
    let log2 = (per_line as f64).log2();
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let comparisons = (lines as f64 * per_line as f64 * log2) as u64;
    #[allow(clippy::cast_precision_loss)]
    let ratio = comparisons as f64 / operations as f64;

    for line in [
        String::new(),
        "  CLEARING'S COST, SEPARATED. A clearing position appends no journal row, so it contributes".to_owned(),
        "  nothing to the operation count — and it is not free. It is paid in comparisons.".to_owned(),
        String::new(),
        format!("    lines cleared at position 13      {lines}"),
        format!("    submissions, at two per fill      {submissions}   (a floor: an unfilled submission is still sorted)"),
        format!("    per line                          {per_line}"),
        format!("    comparisons, sum of n log2 n      {comparisons}"),
        String::new(),
        format!("  That is {ratio:.1}x the tick's whole operation count, in a different unit. §12.2 budgets"),
        "  96.5 ns per EXCHANGE and says nothing about a comparison. A comparison is far cheaper than".to_owned(),
        "  an operation — no journal row, no conserved column, no identity lookup — but a term this".to_owned(),
        "  size cannot be absorbed into a per-operation budget without being named. §12's targets".to_owned(),
        "  need two numbers, not one.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}
