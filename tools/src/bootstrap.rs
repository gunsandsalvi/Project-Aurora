//! # `bootstrap` — §13.6's opening, walked as a ledger rather than argued as prose
//!
//! W7.2. §13.6 derives how a world with no employment and no credit gets started, and asks to be
//! *checked rather than believed*. So this walks it: a minimal double-entry ledger, the smallest cast
//! §13.6 names, and the operations run in §9.4's committed order for ticks 0 to 4.
//!
//! **What it can falsify.** Two invariants, checked after every single operation:
//!
//! 1. **A1, conservation.** Every asset sums to exactly zero across all holders. This is structural
//!    rather than tested — `move` and `exchange` cannot write anything else — which is the point: the
//!    check exists to prove the *operations* are the only writer, not to prove arithmetic.
//! 2. **R-1, the one that can fail.** A holder's balance in an asset it did not issue is never
//!    negative. A step that needs an agent to pay with money it does not have breaks this, and that is
//!    exactly the deadlock §13.6 argues does not occur.
//!
//! Counter-accounts are the declared exception to (2): `Production:`, `Endowment:` and `Consumption:`
//! are where real things come from and go to (§6.2), and their negative side is the model's record
//! that something was produced or consumed rather than a claim on anyone.
//!
//! It is not an engine and does not pretend to be. There is no arena, no shard, no handle, no digest —
//! one `BTreeMap` and two operations. What it establishes is that the *sequence* §13.6 derives can be
//! executed at all.

use core::fmt::Write as _;
use std::collections::{BTreeMap, BTreeSet};

/// Who can hold something. Counter-accounts are entities that decide nothing (§3.4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Party {
    /// The issuer of currency; its negative balance is the money stock (R-1).
    CentralBank,
    /// §13.2: holds the whole money stock at the close of tick 0.
    Government,
    /// The issuer of deposits and of its own equity.
    Bank,
    /// The firm that produces goods.
    Producer,
    /// The firm that produces dwellings.
    Builder,
    /// The class that supplies hours and consumes goods.
    Household,
    /// §6.2's counter-account families. Real things come from and go to these.
    Production,
    /// Where hours arrive from, at position 4.
    Endowment,
    /// Where hours and goods go when they are used up.
    Consumption,
    /// Where depreciation goes (§9.4 position 5).
    Wear,
}

impl Party {
    /// The counter-accounts, which may hold a negative balance in an asset they did not issue.
    ///
    /// This is not an exemption from R-1, it is what R-1 says: a counter-account's negative side is
    /// the record that a real thing was produced or consumed, and there is no claim on anybody.
    fn is_counter_account(self) -> bool {
        matches!(
            self,
            Party::Production | Party::Endowment | Party::Consumption | Party::Wear
        )
    }

    fn name(self) -> &'static str {
        match self {
            Party::CentralBank => "central bank",
            Party::Government => "government",
            Party::Bank => "bank",
            Party::Producer => "producer",
            Party::Builder => "builder",
            Party::Household => "household",
            Party::Production => "Production:",
            Party::Endowment => "Endowment:",
            Party::Consumption => "Consumption:",
            Party::Wear => "Wear:",
        }
    }
}

/// What can be held. Each names its issuer, or `None` when it is a real thing nobody issued.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Asset {
    /// The central bank's claim. §13.2: at the close of tick 0 the governments hold the whole stock.
    Currency,
    /// A bank's liability, and §7.7's one line per issuer.
    Deposit,
    /// A firm's obligation to a bank.
    Loan,
    /// The bank's own equity.
    BankEquity,
    /// An obligation to deliver hours against a wage.
    EmploymentContract,
    /// Hours, which are real and arrive from `Endowment:` at position 4 (§9.4).
    Hours,
    /// Output, real, issued from `Production:`.
    Goods,
    /// A produced means of production, real.
    Capital,
    /// A produced dwelling, real.
    Dwelling,
}

impl Asset {
    /// The issuer, whose negative balance IS the claim (R-1). `None` for a real thing.
    fn issuer(self) -> Option<Party> {
        match self {
            Asset::Currency => Some(Party::CentralBank),
            Asset::Deposit | Asset::BankEquity => Some(Party::Bank),
            Asset::Loan => Some(Party::Producer),
            Asset::EmploymentContract => Some(Party::Household),
            Asset::Hours | Asset::Goods | Asset::Capital | Asset::Dwelling => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Asset::Currency => "currency",
            Asset::Deposit => "deposit",
            Asset::Loan => "loan",
            Asset::BankEquity => "bank equity",
            Asset::EmploymentContract => "employment contract",
            Asset::Hours => "hours",
            Asset::Goods => "goods",
            Asset::Capital => "capital",
            Asset::Dwelling => "dwelling",
        }
    }
}

/// The ledger: holdings, and nothing else. Marks, prices and plans are not state here.
#[derive(Default)]
pub struct Ledger {
    holdings: BTreeMap<(Party, Asset), i64>,
    /// Every operation, in order, as it would appear in the journal.
    pub journal: Vec<String>,
    /// Any R-1 violation, with the step that caused it.
    pub violations: Vec<String>,
}

impl Ledger {
    fn balance(&self, who: Party, what: Asset) -> i64 {
        self.holdings.get(&(who, what)).copied().unwrap_or(0)
    }

    fn credit(&mut self, who: Party, what: Asset, by: i64) {
        *self.holdings.entry((who, what)).or_insert(0) += by;
    }

    /// §6.4's one-sided operation.
    fn r#move(&mut self, tick: u16, position: u8, from: Party, to: Party, what: Asset, qty: i64) {
        self.credit(from, what, -qty);
        self.credit(to, what, qty);
        self.journal.push(format!(
            "  t{tick} p{position:<2} move      {:>12} -> {:<12} {qty:>10} {}",
            from.name(),
            to.name(),
            what.name()
        ));
        self.check(tick, position, "move");
    }

    /// §6.4's indivisible two-sided operation.
    #[allow(clippy::too_many_arguments)]
    fn exchange(
        &mut self,
        tick: u16,
        position: u8,
        a: Party,
        b: Party,
        given: Asset,
        qty_given: i64,
        received: Asset,
        qty_received: i64,
    ) {
        self.credit(a, given, -qty_given);
        self.credit(b, given, qty_given);
        self.credit(b, received, -qty_received);
        self.credit(a, received, qty_received);
        self.journal.push(format!(
            "  t{tick} p{position:<2} exchange  {:>12} <-> {:<12} {qty_given:>10} {} for {qty_received} {}",
            a.name(),
            b.name(),
            given.name(),
            received.name()
        ));
        self.check(tick, position, "exchange");
    }

    /// A1 and R-1, after every operation.
    fn check(&mut self, tick: u16, position: u8, op: &str) {
        let assets: BTreeSet<Asset> = self.holdings.keys().map(|(_, a)| *a).collect();
        for asset in assets {
            // A1: every asset sums to zero across all holders, counter-accounts included.
            let total: i64 = self
                .holdings
                .iter()
                .filter(|((_, a), _)| *a == asset)
                .map(|(_, v)| *v)
                .sum();
            if total != 0 {
                self.violations.push(format!(
                    "t{tick} p{position} {op}: {} sums to {total}, not zero — A1",
                    asset.name()
                ));
            }
            // R-1: only the issuer, or a counter-account, may be negative.
            let offenders: Vec<Party> = self
                .holdings
                .iter()
                .filter(|((_, a), v)| *a == asset && **v < 0)
                .map(|((p, _), _)| *p)
                .filter(|p| !p.is_counter_account() && asset.issuer() != Some(*p))
                .collect();
            for who in offenders {
                self.violations.push(format!(
                    "t{tick} p{position} {op}: {} holds {} of {}, which it did not issue — R-1",
                    who.name(),
                    self.balance(who, asset),
                    asset.name()
                ));
            }
        }
    }
}

/// §13.6's five ticks, walked in §9.4's committed order.
///
/// Quantities are round numbers chosen so the arithmetic is readable. **Nothing here is a parameter**
/// — the question is whether the sequence executes at all, and a sequence that works at 1,000 works at
/// any number, because every step moves what the previous step created.
#[must_use]
pub fn walk() -> Ledger {
    let mut l = Ledger::default();
    open(&mut l);
    ticks_zero_to_two(&mut l);
    ticks_three_and_four(&mut l);
    l
}

/// §13.2's opening state, as one operation rather than as a seeding.
fn open(l: &mut Ledger) {
    // §13.2's opening: the central bank issues, the governments hold the whole stock. This is the one
    // state that exists before tick 0, and R-1 makes it a single operation rather than a seeding.
    l.r#move(
        0,
        0,
        Party::CentralBank,
        Party::Government,
        Asset::Currency,
        10_000,
    );
}

/// Ticks 0 to 2: the government is the only spender, and money reaches households and then the bank.
fn ticks_zero_to_two(l: &mut Ledger) {
    // ── tick 0 ──────────────────────────────────────────────────────────────────────────────────
    // p6: production sources the opening capital and dwellings from `Production:` (§6.2). WHO HOLDS
    // THEM is what §13.6 leaves owed; see `holding_rule` below. The producer of a thing receives its
    // own output, which is what position 6 already does for every later tick.
    l.r#move(
        0,
        6,
        Party::Production,
        Party::Producer,
        Asset::Capital,
        500,
    );
    l.r#move(
        0,
        6,
        Party::Production,
        Party::Builder,
        Asset::Dwelling,
        400,
    );

    // p12/13: only the government has a settled balance, so only the government allocates and bids.
    // The labour line clears; a firm with no balance has a zero budget and does not submit.
    // p19: the fill becomes an employment contract, issued by the household that will deliver.
    l.r#move(
        0,
        19,
        Party::Household,
        Party::Government,
        Asset::EmploymentContract,
        1,
    );

    // ── tick 1 ──────────────────────────────────────────────────────────────────────────────────
    // p4: the contract's obligations both fall due. Hours arrive from `Endowment:` — §9.4 delivers
    // them here, not at position 2 — and the wage is paid.
    l.r#move(1, 4, Party::Endowment, Party::Household, Asset::Hours, 100);
    l.r#move(1, 4, Party::Household, Party::Government, Asset::Hours, 100);
    l.r#move(
        1,
        4,
        Party::Government,
        Party::Consumption,
        Asset::Hours,
        100,
    );
    l.r#move(
        1,
        4,
        Party::Government,
        Party::Household,
        Asset::Currency,
        1_000,
    );
    // The household holds money for the first time. It deposits: the bank takes the currency and
    // issues a deposit, which is its own liability.
    l.exchange(
        1,
        14,
        Party::Household,
        Party::Bank,
        Asset::Currency,
        1_000,
        Asset::Deposit,
        1_000,
    );

    // ── tick 2 ──────────────────────────────────────────────────────────────────────────────────
    // p19 then p14: the bank issues equity and the household buys it, paying with its deposit — which
    // extinguishes the deposit against the bank's own liability. The bank now has capital.
    l.exchange(
        2,
        14,
        Party::Household,
        Party::Bank,
        Asset::Deposit,
        300,
        Asset::BankEquity,
        300,
    );
}

/// Ticks 3 and 4: the bank lends, the firm hires and produces, and the circuit closes.
fn ticks_three_and_four(l: &mut Ledger) {
    // ── tick 3 ──────────────────────────────────────────────────────────────────────────────────
    // p13/14: the bank lends. A loan is a deposit moved out of the bank, which goes equally negative.
    l.exchange(
        3,
        14,
        Party::Bank,
        Party::Producer,
        Asset::Deposit,
        800,
        Asset::Loan,
        800,
    );
    // The firm now holds money, so it can budget for labour at position 12 and bid at 13.
    l.r#move(
        3,
        19,
        Party::Household,
        Party::Producer,
        Asset::EmploymentContract,
        1,
    );

    // ── tick 4 ──────────────────────────────────────────────────────────────────────────────────
    // p4: the firm pays and the hours are delivered and consumed.
    l.r#move(4, 4, Party::Endowment, Party::Household, Asset::Hours, 100);
    l.r#move(4, 4, Party::Household, Party::Producer, Asset::Hours, 100);
    l.r#move(4, 4, Party::Producer, Party::Consumption, Asset::Hours, 100);
    l.r#move(4, 4, Party::Producer, Party::Household, Asset::Deposit, 600);
    // p5: capital wears.
    l.r#move(4, 5, Party::Producer, Party::Wear, Asset::Capital, 10);
    // p6: hours and capital become output, issued from `Production:` to the firm that produced it.
    l.r#move(4, 6, Party::Production, Party::Producer, Asset::Goods, 700);
    // p14: the household buys, and consumes what it bought in the same settlement (§9.4: no larder).
    l.exchange(
        4,
        14,
        Party::Household,
        Party::Producer,
        Asset::Deposit,
        500,
        Asset::Goods,
        500,
    );
    l.r#move(
        4,
        14,
        Party::Household,
        Party::Consumption,
        Asset::Goods,
        500,
    );
}

/// The deadlock §13.6 argues does not occur, run so that the check can be seen to fail.
///
/// §13.6's own summary of the fear: *"at the close of tick 0 the four governments hold the entire
/// money stock, no employment contract exists, and §9.6.2 says a firm that cannot make payroll
/// delivers no hours and fails to settle."* The reading that produces it is that a firm can bid on
/// the labour line at tick 0. Let it, and see what breaks.
///
/// **A check that has never failed is not known to work.** This is that check's failure.
#[must_use]
pub fn walk_naive() -> Ledger {
    let mut l = Ledger::default();
    l.r#move(
        0,
        0,
        Party::CentralBank,
        Party::Government,
        Asset::Currency,
        10_000,
    );
    l.r#move(
        0,
        6,
        Party::Production,
        Party::Producer,
        Asset::Capital,
        500,
    );

    // The step §9.2 forbids: the firm bids on the labour line with no settled balance, and the fill
    // becomes a contract at position 19. Nothing has gone wrong yet — issuing a contract costs
    // nothing, which is exactly why the error is invisible at tick 0.
    l.r#move(
        0,
        19,
        Party::Household,
        Party::Producer,
        Asset::EmploymentContract,
        1,
    );

    // Tick 1, position 4: the wage falls due, and the firm has no money.
    l.r#move(1, 4, Party::Endowment, Party::Household, Asset::Hours, 100);
    l.r#move(1, 4, Party::Household, Party::Producer, Asset::Hours, 100);
    l.r#move(
        1,
        4,
        Party::Producer,
        Party::Household,
        Asset::Currency,
        1_000,
    );
    l
}

/// Run the check and print the trace; the exit status is the process's.
///
/// The guard has two halves and both must hold: §13.6's walk runs clean, and the reading it argues
/// against does not. A pass on the first alone would be a check that cannot fail.
///
/// `verbose` prints the trace; the gate wants only the verdict.
pub fn run(verbose: bool) -> std::process::ExitCode {
    if verbose {
        print!("{}", report());
    }
    let (walked, naive) = (walk(), walk_naive());
    println!("check-bootstrap");
    println!(
        "  rule 1: \u{a7}13.6's opening executes with no A1 or R-1 violation   operations: {}",
        walked.journal.len()
    );
    println!(
        "  rule 2: the reading it argues against does NOT — a check that cannot fail is not a check"
    );
    println!("  exemptions: 0");
    let mut findings = Vec::new();
    for v in &walked.violations {
        findings.push(format!("\u{a7}13.6's own walk: {v}"));
    }
    if naive.violations.is_empty() {
        findings.push(
            "the negative control passed — a firm hiring with no balance no longer breaks R-1, so \
             this check is no longer checking anything"
                .to_owned(),
        );
    }
    if findings.is_empty() {
        println!("  violations: 0");
        return std::process::ExitCode::SUCCESS;
    }
    println!("  violations: {}", findings.len());
    for f in &findings {
        println!("    {f}");
    }
    std::process::ExitCode::FAILURE
}

/// The trace, the closing balances, and the holding rule §13.6 left owed.
#[must_use]
pub fn report() -> String {
    let l = walk();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\n§13.6 — the opening, walked in §9.4's order for ticks 0 to 4\n"
    );
    for row in &l.journal {
        let _ = writeln!(out, "{row}");
    }

    let _ = writeln!(
        out,
        "\n  closing balances (a negative is an issuer's, by R-1)\n"
    );
    let _ = writeln!(out, "  {:<14} {:<22} {:>10}", "holder", "asset", "balance");
    for ((who, what), qty) in &l.holdings {
        if *qty != 0 {
            let _ = writeln!(out, "  {:<14} {:<22} {qty:>10}", who.name(), what.name());
        }
    }

    let _ = writeln!(
        out,
        "\n  operations: {}   A1 and R-1 checked after every one   violations: {}",
        l.journal.len(),
        l.violations.len()
    );
    for v in &l.violations {
        let _ = writeln!(out, "    {v}");
    }
    out.push_str(&negative_control());
    out.push_str(&holding_rule());
    out
}

/// The same check, run against the sequence §13.6 says is impossible.
fn negative_control() -> String {
    let naive = walk_naive();
    let mut out = String::new();
    for line in [
        String::new(),
        "  THE NEGATIVE CONTROL. A check that has never failed is not known to work, so the same".to_owned(),
        "  walk is run against the reading §13.6 argues against: a firm bidding on the labour line at".to_owned(),
        "  tick 0 with no settled balance.".to_owned(),
        String::new(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    for row in &naive.journal {
        let _ = writeln!(out, "{row}");
    }
    let _ = writeln!(out, "\n  violations: {}", naive.violations.len());
    for v in &naive.violations {
        let _ = writeln!(out, "    {v}");
    }
    for line in [
        String::new(),
        "  It fails where §13.6 says it fails, and one tick later than the mistake. Issuing the".to_owned(),
        "  contract at tick 0 costs nothing, so nothing looks wrong until the wage falls due at".to_owned(),
        "  position 4 of tick 1 — which is precisely why §9.2's two-stage allocator has to prevent the".to_owned(),
        "  BID rather than catch the payment. A budget checked at position 12 is a defect refused; a".to_owned(),
        "  balance checked at position 4 is a firm already in resolution.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// The one thing §13.6 leaves open, derived rather than decided.
fn holding_rule() -> String {
    let mut out = String::new();
    for line in [
        String::new(),
        "  THE HOLDING RULE FOR THE OPENING STOCKS, which §13.6 marks \"owed, and still owed\".".to_owned(),
        String::new(),
        "  §13.6 asks who holds the capital and dwellings that position 6 sources from `Production:`".to_owned(),
        "  at tick 0, and says the rule must be DERIVED because M4 requires modelled holders and M3".to_owned(),
        "  forbids a per-region assumed allocation.".to_owned(),
        String::new(),
        "  It is derived, and it needs no new rule at all: THE PRODUCER OF A THING RECEIVES ITS OWN".to_owned(),
        "  OUTPUT. That is what position 6 does at every other tick — inputs and hours consumed into".to_owned(),
        "  `Consumption:`, output issued from `Production:` — and tick 0 is not special. The question".to_owned(),
        "  only looked open because it was read as an allocation problem; asked as \"which mechanism".to_owned(),
        "  puts them there\", the mechanism already exists and names the holder.".to_owned(),
        String::new(),
        "  Three consequences, and each is a claim the model can be held to:".to_owned(),
        String::new(),
        "  1. NO HOUSEHOLD OWNS A DWELLING AT TICK 0. Builders hold every dwelling, because builders".to_owned(),
        "     produced them. Home ownership is an EMERGENT stock, bought once households hold money —".to_owned(),
        "     which is tick 1 at the earliest. Seeding households with dwellings would be a".to_owned(),
        "     per-household assumed allocation, which §16.1 rule 1 refuses.".to_owned(),
        "  2. EVERY HOUSEHOLD IS A TENANT OR HOMELESS AT TICK 0, and tenancies form from tick 1 as".to_owned(),
        "     builders let what they hold. §5.2's 250,000 live tenancies are therefore a state the".to_owned(),
        "     first few ticks REACH, not a state the seed writes.".to_owned(),
        "  3. THE QUANTITY IS STILL A SEED QUESTION. How much capital each firm opens with is §13.3's".to_owned(),
        "     generator's business, derived from the axis loadings; §13.1 rule 3 forbids a per-region".to_owned(),
        "     count being typed. What is settled here is the HOLDER, not the amount.".to_owned(),
        String::new(),
        "  This is the last thing §13.6 named as standing between it and a world that starts.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}
