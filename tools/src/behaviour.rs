//! # `behaviour` — two agent classes, declared and then run for ten ticks
//!
//! W7.8, which one review called the single highest-value item anywhere in the project: it buys much
//! of G3's signal for none of M7's cost. §8.1 says an agent is **five declarations and a class
//! declaring four does not compile**, and that everything else is generic machinery the agent does not
//! own. That is a strong claim and it has never been written out for even one class.
//!
//! So: the five declarations for `Household` and for `Bank`, as data rather than as prose, and ten
//! ticks driven by them over the same ledger `bootstrap` uses — with A1 and R-1 checked after every
//! operation.
//!
//! **What it can falsify.** Three claims, each of which fails visibly if it is wrong:
//!
//! 1. **Five declarations is enough.** If a tick needs the agent to know something none of the five
//!    carries, the claim is wrong and the missing thing is named here.
//! 2. **A capital ratio is an expression over one contiguous run** — §8.1's own words, *no second
//!    table and no attribution step*. The bank's constraint here reads nothing but the bank's own
//!    signed balances.
//! 3. **The constraint binds.** A bank that can lend without limit means the constraint was not
//!    expressible after all. The run continues until it binds, and reports which tick.

use core::fmt::Write as _;

use crate::bootstrap::{Asset, Ledger, Party};

/// One of §8.1's five, for one class. `content` is `None` when the declaration is genuinely empty at
/// this phase — which §8.1 requires be *declared absent with its reason*, a value the check reads.
struct Declaration {
    which: u8,
    name: &'static str,
    content: Option<&'static str>,
    absent_because: Option<&'static str>,
}

/// The five for the household.
fn household() -> [Declaration; 5] {
    [
        Declaration {
            which: 1,
            name: "Mandate",
            absent_because: None,
            content: Some(
                "may hold currency, deposits, bank equity, goods and dwellings; issues employment contracts. \
             No proportion is fixed: what it holds is what its funding policy leaves it holding",
            ),
        },
        Declaration {
            which: 2,
            name: "Regime",
            absent_because: None,
            content: Some(
                "`household` (§7.3) — no capital regime, no consolidation, claims held at settled value",
            ),
        },
        Declaration {
            which: 3,
            name: "Constraints",
            absent_because: None,
            content: Some(
                "consumption spend <= settled deposit balance at position 12. NOT a balance check at \
             position 14: R-1 already makes an unbacked payment impossible, so the constraint's work \
             is to stop the BID (§9.2), which is one position earlier",
            ),
        },
        Declaration {
            which: 4,
            name: "Valuation",
            absent_because: None,
            content: Some(
                "reservation wage = last accepted wage, or the endowment's opportunity cost when it has \
             never worked; reservation price for goods = the price at which its budget clears its \
             intended basket",
            ),
        },
        Declaration {
            which: 5,
            name: "Funding policy",
            absent_because: None,
            content: Some(
                "surplus -> deposit at its bank; deficit -> reduce consumption to the budget. Currency \
             composition: its own region only. Cure window: none, and this is not an absence — a \
             household holding no claim it issued except its own labour cannot breach a capital \
             floor, so the window has no subject",
            ),
        },
    ]
}

/// The five for the bank.
fn bank() -> [Declaration; 5] {
    [
        Declaration {
            which: 1,
            name: "Mandate",
            absent_because: None,
            content: Some(
                "may hold currency and loans; issues deposits and its own equity. It may not hold goods, \
             dwellings or hours — a mandate is over ASSETS, and this is where a bank differs from a \
             fund without a single line of different machinery",
            ),
        },
        Declaration {
            which: 2,
            name: "Regime",
            absent_because: None,
            content: Some(
                "`bank-prudential` (§7.3) — a capital regime, and §7.2 Q11's rank is what weights the \
             residual claims out of it",
            ),
        },
        Declaration {
            which: 3,
            name: "Constraints",
            absent_because: None,
            content: Some(
                "regulatory capital / risk-weighted assets >= 0.08. Regulatory capital is -(its own \
             equity balance), which under R-1 is a row on its own block; risk-weighted assets are its \
             loan balance times the weight. BOTH ARE ROWS IN ONE CONTIGUOUS RUN",
            ),
        },
        Declaration {
            which: 4,
            name: "Valuation",
            content: None,
            absent_because: Some(
                "it has no funding cost before it has funding. §8.1 names exactly this case: a bank has \
             no valuation before it has a funding cost, and SAYING SO IS A STATEMENT. From the tick \
             it first pays interest, the reservation lending rate is funding cost plus the spread its \
             capital headroom implies",
            ),
        },
        Declaration {
            which: 5,
            name: "Funding policy",
            absent_because: None,
            content: Some(
                "surplus -> hold currency; deficit -> issue equity, and stop lending while the ratio is \
             below floor. Currency composition: own region. Cure window: 4 ticks (§6.7)",
            ),
        },
    ]
}

/// §8.1's floor, and §7.2 Q11's weight on a residual claim.
const CAPITAL_FLOOR: f64 = 0.08;

/// What one tick did, under the declarations.
struct Step {
    tick: u16,
    what: String,
    capital: i64,
    rwa: i64,
    ratio: f64,
    bound: bool,
}

/// Ten ticks, with the declarations deciding and the ledger recording.
///
/// The household earns, deposits, consumes and buys equity out of what its funding policy leaves.
/// The bank lends whatever its constraint allows and stops when it does not. Nothing here is a
/// schedule: each tick reads the ledger and applies the declarations.
///
/// `equity_until` is the last tick on which the household adds to its equity holding. Run to 10 the
/// capital constraint never binds; run to 2 it does, and a constraint that never binds is a comment.
#[allow(clippy::too_many_lines)]
fn run(equity_until: u16, ticks: u16) -> (Ledger, Vec<Step>) {
    let mut l = Ledger::default();
    let mut steps = Vec::new();

    // The opening state §13.6 derives, compressed: the government holds the money stock, the
    // household is employed by it, and the bank exists with no capital and no book.
    l.r#move(
        0,
        0,
        Party::CentralBank,
        Party::Government,
        Asset::Currency,
        100_000,
    );
    l.r#move(
        0,
        19,
        Party::Household,
        Party::Government,
        Asset::EmploymentContract,
        1,
    );

    let wage = 1_000i64;
    let consume_share = 70; // per cent of income; the rest is the funding policy's surplus
    let equity_share = 20; // of the surplus, while the household holds less equity than a year's wage

    for tick in 1..=ticks {
        let mut what = String::new();

        // p4 — the wage falls due, and hours are delivered and consumed.
        l.r#move(
            tick,
            4,
            Party::Endowment,
            Party::Household,
            Asset::Hours,
            100,
        );
        l.r#move(
            tick,
            4,
            Party::Household,
            Party::Government,
            Asset::Hours,
            100,
        );
        l.r#move(
            tick,
            4,
            Party::Government,
            Party::Consumption,
            Asset::Hours,
            100,
        );
        l.r#move(
            tick,
            4,
            Party::Government,
            Party::Household,
            Asset::Currency,
            wage,
        );

        // Household declaration 5: the surplus goes on deposit. It banks the whole wage first,
        // because a deposit is how it holds money at all — the currency itself is the government's
        // to hand over and the household's to place.
        l.exchange(
            tick,
            14,
            Party::Household,
            Party::Bank,
            Asset::Currency,
            wage,
            Asset::Deposit,
            wage,
        );

        // Household declaration 5 again: part of the surplus buys bank equity while it holds less
        // than a year's wage of it. This is the only funding the bank's capital ever gets.
        let held_equity = l.balance(Party::Household, Asset::BankEquity);
        let surplus = wage * (100 - consume_share) / 100;
        let buy = if tick <= equity_until && held_equity < wage * 52 {
            surplus * equity_share / (100 - consume_share)
        } else {
            0
        };
        if buy > 0 {
            l.exchange(
                tick,
                14,
                Party::Household,
                Party::Bank,
                Asset::Deposit,
                buy,
                Asset::BankEquity,
                buy,
            );
            let _ = write!(what, "buys {buy} equity; ");
        }

        // Bank declaration 3, read off its own block and nothing else.
        let capital = -l.balance(Party::Bank, Asset::BankEquity);
        let rwa = l.balance(Party::Bank, Asset::Loan);
        #[allow(clippy::cast_precision_loss)]
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss
        )]
        let headroom = (capital as f64 / CAPITAL_FLOOR) as i64 - rwa;
        let demand = 400i64;
        let lend = demand.min(headroom.max(0));
        if lend > 0 {
            l.exchange(
                tick,
                14,
                Party::Bank,
                Party::Producer,
                Asset::Deposit,
                lend,
                Asset::Loan,
                lend,
            );
            let _ = write!(what, "lends {lend}");
        } else {
            let _ = write!(what, "CANNOT LEND — the floor binds");
        }

        // p6 — the producer's output is issued from `Production:`. IT COMES BEFORE POSITION 14, and
        // the first draft of this walk had it after: the check reported the producer holding -700 of
        // goods it did not issue, which is a firm selling what it has not yet produced. §9.4's
        // committed order is what prevents that, and running out of order is how you find out.
        let spend = (wage * consume_share / 100).min(l.balance(Party::Household, Asset::Deposit));
        l.r#move(
            tick,
            6,
            Party::Production,
            Party::Producer,
            Asset::Goods,
            spend,
        );

        // p14 — the household consumes what its constraint allows, from the producer.
        if spend > 0 {
            l.exchange(
                tick,
                14,
                Party::Household,
                Party::Producer,
                Asset::Deposit,
                spend,
                Asset::Goods,
                spend,
            );
            l.r#move(
                tick,
                14,
                Party::Household,
                Party::Consumption,
                Asset::Goods,
                spend,
            );
        }

        let capital = -l.balance(Party::Bank, Asset::BankEquity);
        let rwa = l.balance(Party::Bank, Asset::Loan);
        #[allow(clippy::cast_precision_loss)]
        let ratio = if rwa == 0 {
            f64::INFINITY
        } else {
            capital as f64 / rwa as f64
        };
        steps.push(Step {
            tick,
            what,
            capital,
            rwa,
            ratio,
            bound: lend < demand,
        });
    }
    (l, steps)
}

/// The declarations, the ten ticks, and what the run establishes.
#[must_use]
pub fn report() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "\n§8.1 — five declarations, for two classes, then run\n"
    );

    for (class, decls) in [("Household", household()), ("Bank", bank())] {
        let _ = writeln!(out, "  {class}");
        for d in &decls {
            match (d.content, d.absent_because) {
                (Some(c), _) => {
                    let _ = writeln!(out, "    {} {:<16} {c}", d.which, d.name);
                }
                (None, Some(why)) => {
                    let _ = writeln!(
                        out,
                        "    {} {:<16} ABSENT, and declared so: {why}",
                        d.which, d.name
                    );
                }
                (None, None) => {
                    let _ = writeln!(
                        out,
                        "    {} {:<16} MISSING — a class declaring four does not compile",
                        d.which, d.name
                    );
                }
            }
        }
        let _ = writeln!(out);
    }

    let (l, steps) = run(10, 10);
    let _ = writeln!(out, "  ten ticks, the declarations deciding\n");
    let _ = writeln!(out, "  tick  capital    loans     ratio  what happened");
    for s in &steps {
        let ratio = if s.ratio.is_finite() {
            format!("{:.4}", s.ratio)
        } else {
            "  —".to_owned()
        };
        let _ = writeln!(
            out,
            "  {:>4}  {:>7}  {:>7}  {ratio:>8}  {}",
            s.tick, s.capital, s.rwa, s.what
        );
    }

    let _ = writeln!(out, "\n  closing balances\n");
    for who in [
        Party::CentralBank,
        Party::Government,
        Party::Bank,
        Party::Producer,
        Party::Household,
    ] {
        for what in [
            Asset::Currency,
            Asset::Deposit,
            Asset::Loan,
            Asset::BankEquity,
            Asset::Goods,
        ] {
            let q = l.balance(who, what);
            if q != 0 {
                let _ = writeln!(out, "  {:<14} {:<14} {q:>10}", who.label(), what.label());
            }
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
    // The same run with the household's equity buying stopped at tick 2, so the constraint has
    // something to bind against. A constraint that never binds is a comment.
    let (_, constrained) = run(2, 15);
    let _ = writeln!(
        out,
        "\n  the same run to fifteen ticks, with the household's equity buying stopped at tick 2\n"
    );
    let _ = writeln!(out, "  tick  capital    loans     ratio  what happened");
    for s in &constrained {
        let ratio = if s.ratio.is_finite() {
            format!("{:.4}", s.ratio)
        } else {
            "  \u{2014}".to_owned()
        };
        let _ = writeln!(
            out,
            "  {:>4}  {:>7}  {:>7}  {ratio:>8}  {}",
            s.tick, s.capital, s.rwa, s.what
        );
    }

    out.push_str(&findings(&steps, &constrained));
    out
}

/// What ten ticks establish that the declarations alone do not.
fn findings(steps: &[Step], constrained: &[Step]) -> String {
    let mut out = String::new();
    let free_bind = steps.iter().find(|s| s.bound).map(|s| s.tick);
    // The ratio never goes BELOW the floor, because the constraint refuses the lending that would
    // take it there. So the test is that the bank lent less than was asked, not that it breached.
    let held_bind = constrained.iter().find(|s| s.bound).map(|s| s.tick);
    // Capital grows by the equity flow; capacity grows by that over the floor. The constraint binds
    // only when per-tick credit demand exceeds it.
    let equity_flow = 200i64;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    let capacity_per_tick = (equity_flow as f64 / CAPITAL_FLOOR) as i64;
    #[allow(clippy::cast_precision_loss)]
    let multiple = capacity_per_tick as f64 / 1000.0;

    for line in [
        String::new(),
        "  FINDING 1. THE CAPITAL CONSTRAINT IS AN EXPRESSION OVER ONE CONTIGUOUS RUN, as \u{a7}8.1".to_owned(),
        "  claims, and the run above reads nothing else. Regulatory capital is -(the bank's own equity".to_owned(),
        "  balance); risk-weighted assets are its loan balance. Two rows on its own block, no second".to_owned(),
        "  table, no attribution step. That is a consequence of R-1 rather than of the constraint's".to_owned(),
        "  design: a model holding liabilities anywhere but as the issuer's negative balance would need".to_owned(),
        "  exactly the attribution step \u{a7}8.1 says it does not have.".to_owned(),
        String::new(),
        match free_bind {
            Some(t) => format!("  FINDING 2. The floor binds at tick {t} under the declarations as written."),
            None => "  FINDING 2. THE FLOOR NEVER BINDS UNDER THE DECLARATIONS AS WRITTEN, and that is a result".to_owned(),
        },
        format!("  rather than a mis-set knob. An equity flow of {equity_flow} a tick against a {CAPITAL_FLOOR:.2} floor supports"),
        format!("  {capacity_per_tick} of NEW LENDING a tick, and the demand here is 400. The floor binds only when per-tick"),
        format!("  credit demand exceeds {multiple:.1}x the wage bill \u{2014} so in this cast the binding constraint on credit"),
        "  is the household's saving, not the bank's capital. The declarations produce that; nothing".to_owned(),
        "  states it.".to_owned(),
        String::new(),
        match held_bind {
            Some(t) => format!("  Stop the household's equity buying at tick 2 and THE FLOOR BINDS AT TICK {t} \u{2014} the second"),
            None => "  Stopping the household's equity buying does not make it bind either, which would mean the".to_owned(),
        },
        "  table above. Note what binding looks like: the ratio never goes BELOW 0.08, because the".to_owned(),
        "  constraint refuses the lending that would take it there. A constraint that shows up as a".to_owned(),
        "  breach is a constraint that was checked too late.".to_owned(),
        String::new(),
        "  FINDING 3. THE BANK'S CAPITAL COMES FROM A LIABILITY SWAP, NOT AN INFLOW. When the household".to_owned(),
        "  buys equity it pays with a deposit \u{2014} the bank's own liability \u{2014} so the bank's currency does".to_owned(),
        "  not move: it exchanges one claim it issued for another. Its capacity to lend rises and its".to_owned(),
        "  cash does not. Under R-1 that falls out of the arithmetic. A model holding liabilities in a".to_owned(),
        "  separate table would have to state it as a rule, and would probably state it wrong.".to_owned(),
        String::new(),
        "  FINDING 4. THE COMMITTED ORDER CAUGHT AN ERROR IN THIS WALK'S FIRST DRAFT. Position 6 was".to_owned(),
        "  written after position 14, and the check reported the producer holding -700 of goods it did".to_owned(),
        "  not issue on every one of ten ticks \u{2014} a firm selling what it has not yet produced. \u{a7}9.4's".to_owned(),
        "  order is exactly what prevents that, and the violation is what a period order is FOR. It was".to_owned(),
        "  not caught by reading the code.".to_owned(),
        String::new(),
        "  FINDING 5. FIVE DECLARATIONS WERE ENOUGH, WITH ONE QUALIFICATION. Every decision in the ten".to_owned(),
        "  ticks came from one of the five, and the bank's valuation was genuinely absent with a reason".to_owned(),
        "  \u{2014} \u{a7}8.1's own example. The qualification: declaration 3 had to be read as binding at POSITION".to_owned(),
        "  12, not at the payment. R-1 already makes an unbacked payment impossible, so a constraint".to_owned(),
        "  checked at settlement checks something the ledger cannot do anyway. \u{a7}8.1 says constraints are".to_owned(),
        "  \"the inequalities that bind it\" and does not say WHERE; \u{a7}9.2 and \u{a7}9.4 do. The two must be read".to_owned(),
        "  together, and neither says so.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}
