# Project Aurora — Implementation

**Version 1.0.** This document is the build plan. `PROJECT_AURORA.md` says what the model is; this says
in what order it gets made, what each stage proves, and what would make us stop.

Its organising rule is the specification's own, applied to work rather than to facts: **one milestone,
one question.** A milestone that answers two questions is two milestones, and a milestone that answers
none is a schedule with a name on it.

---

## 0. How to read this

| If you want | Read |
|---|---|
| why the order is this order | §1 |
| the milestones themselves | §3 |
| what is not yet decided, and when it must be | §4 |
| what runs continuously rather than in a milestone | §5 |
| what would make us stop | §6 |
| what this costs | §7 |
| the first milestone in executable detail | `MILESTONE_0.md` |
| what has to be in the model at all | `PROJECT_AURORA.md` §2.3, the surface |

**This plan supersedes §19's phase table** wherever the two differ. §19 is kept in the specification
because its gates are cited from elsewhere; it is a summary of this.

### The four founding decisions this plan is built on

Recorded in `PROJECT_AURORA.md` Appendix B and binding here:

- **D1 — the model wins; the delivery target bends.** N2a, N2b and N4 are targets. No milestone may be
  brought into budget by reducing the agent population or coarsening a cadence.
- **D2 — Rust, one crate per layer, delivered as an Android application.** Native engine, thin UI.
- **D3 — minimise priors and publish the count; no cap.**
- **D4 — device measurement is a cross-compiled probe published to GitHub Releases**, installed and run
  by the project owner, returning one JSON document.

---

## 1. The five principles behind the ordering

**1. A check must precede the code it polices.** A guard retrofitted onto a violating tree is negotiated
down to fit what is already written. Under D2 most of the specification's guards became properties of
the crate graph and the type system, which is exactly why the workspace must be shaped before anything
is written into it. This is why M0 exists and why it contains no engine code.

**2. Buy the cheap evidence first.** Roughly a dozen numbers carry this design — 96.5 ns per exchange,
24 B per holding slot, 48 B per journal row, 1,488.3 MB, 0.503, the region generator's output — and most
were printed without a derivation. Several can be attacked with paper arithmetic or three hundred lines
of throwaway code. Two are already known to be wrong. Spend days on them before spending months against
them.

**3. Build the instruments that cannot be added late.** The differ, the digest, the identity and
relocation model, the reason-code vocabulary and the committed period order all get monotonically more
expensive with every line written on top of them. The specification says this about the differ; it is
true of all five.

**4. Reach a closed circuit early, then widen it.** "Sectoral net lending is exactly zero, per currency,
per tick" is the project's central assertion. It should be a green test running every tick from roughly
a third of the way in — not a promise redeemed at the end. Everything after that point is a substitution
into a system that is already running and already closed.

**5. Decompose the economics; do not schedule it as one bar.** §2.3's surface is what it consists of, and
reading it is the fastest way to see why one bar was never going to hold. The specification contains the machine
and not the model: one production section of five sentences, no content for any of the thirty-five agent
declarations, roughly thirty per-position specifications owed and one written. Every candidate plan that
treated this as a single milestone under-sized it by two to three times. It is four milestones here, each
with its own gate and its own draw on the assumption count.

---

## 2. The shape of the road

```
M0  The Refusing Workspace        ── G0 ──►  delivery envelope settled, 19 ADRs taken
M1  One Arena, One Digest
M2  One Crate Writes Money        ── G1a ─►  CONTINUE/STOP on the write model
M3  Seven Types That Pay          ── G1b ─►  CONTINUE/STOP on the instrument model
M4  Clearing Without A World                (runs parallel to M2–M3)
M5  Twenty-One Positions, Empty
M6  The First Franc               ── G2 ──►  the circuit closes; workload re-derived
M7  The Deciding Tick
M8  Credit, Default And The Estate
M8b The Elected Parliament
M9  Property, Equity And The Rest
M10 Four Regions Trading
M11 Thirty Years At Full Class    ── G3 ──►  CONTINUE/STOP on the economics
```

**M4 is the one parallel track.** `markets` depends only on `domain` and `kernel`, so the grid, the
solver, rule-two allocation and the two-pointer walk are buildable and property-testable against
synthetic submissions with no world, no instruments and no agents. It is the largest de-risking
opportunity in the build and it costs nothing on the critical path.

---

## 3. The milestones

### M0 — The Refusing Workspace

> **Question: can this project's rules be made mechanical before there is any code to break them, and
> what does the device actually do?**

A workspace whose layering, determinism and registry rules hold from the first commit, and a probe on a
real phone replacing the design's unsourced constants with measurements. **No engine code is written.**

**Contents** — Cargo workspace, one crate per layer, the dependency graph as the mechanical form of §4;
`#![forbid(unsafe_code)]` and warnings-as-errors; the ADR machinery the specification requires twenty
times and never defines; the parameter registry with its seven rules and a compile-fail fixture per rule;
the probe APK and its delivery pipeline; and nine paper falsifiers costing days each.

**Exits when** the workspace refuses a forbidden import at `cargo build`; every registry rule has a
compile-fail fixture that fails for its own reason and no other; the probe has run on the owner's device
and returned its JSON; and all nine paper falsifiers have run, with their results written down whether
green or red.

**Gate G0** — settles the delivery envelope and nineteen ADRs. **Rescope authority, not stop authority**, and saying so matters: under D1 no performance finding can stop this project, so a gate whose
thresholds are all in nanoseconds decides nothing. What G0 can do is change the delivery target.

**Size** — 55–65 engineer-weeks, 8–10 calendar weeks at 3–4 engineers plus a part-time modeller.

Scoped in full in **`MILESTONE_0.md`**.

---

### M1 — One Arena, One Digest

> **Question: can a world be stored, walked, saved, digested and diffed, deterministically, with
> identifiers that outlive the rows they name?**

The substrate under everything: the arena, the column schema and its generator, the identity spaces and
the directory, row residency and relocation, the digest, the differ, the save.

**Contents** — the column schema as data and the code generator over it (read views, serializer, parser,
digest walker, differ, relocation, arena layout); the arena, allocated once, exhaustion panics; six
identity spaces as newtypes with private fields; the directory with its three-valued lookup; residency,
relocation and the quiescence queue; the retirement accumulators and the extinguished-stock register; the
tick digest over dirty regions with the mark-set check §11 now requires; the differ and the row
inspector; the save format with its schema hash and its refusal to coerce; the deterministic draw
function and the ordered containers that replace hash maps.

**Why here** — every later milestone writes tables. The digest and the differ arrive before the first
optimisation because a digest mismatch with no differ is a week of print statements, and the
specification says so about the differ specifically.

**Exits when** a world of sixty-four rows per table round-trips bit-identically including a relocation;
the digest is invariant to slot permutation and changes under any state change; the differ localises a
single-cell change to the cell; measured peak allocation equals the generated layout total exactly at
the `test` and `tenth` classes; and the generated artefacts regenerate byte-identically in CI.

**Defers** — economics, instruments, agents, positions, threads.

**Size** — 50–70 engineer-weeks.

---

### M2 — One Crate Writes Money

> **Question: does A1 hold — can a quantity be created or destroyed without an operation naming where it
> came from?**

The ledger: three doors, nine operations, counter-accounts, liens, the journal, and wind-up.

**Contents** — the nine operations behind the door types; every precondition of §6.4 as a raise with a
named reason; the four counter-account families with their minted capabilities and the ten permitted
pairs per region; the three rounding rules and no fourth; liens to depth three with leaf-first
unwinding; the journal ring with its tagged rows and generated decoder; the two accrual columns
maintained inside `move`; the resolution state machine, then the seven-step estate waterfall; and the
substrate half of the conformance suite.

**Why here** — A1 is the claim the whole design is organised around, and under D2 its discharge is the
crate boundary this milestone establishes. **Conformance case 19 is written third**, not nineteenth: the
specification calls it the case that decides whether §5.5 is a memory optimisation or a leak, and if
relocation can open the loop then the holdings layout, the identity spaces and the digest all change.

**Exits when** conformance cases 1–12 and 17–20 pass; no mutable borrow of a conserved column is
obtainable outside the `ledger` crate, demonstrated by a compile-fail fixture; the property test over
generated legal operation sequences finds no conservation violation in ten million operations; and an
entity reaches `resolved` holding exactly zero of every claim it issued.

**Gate G1a — continue or stop on the write model.** The first gate with real authority.

**Size** — 45–60 engineer-weeks.

---

### M3 — Seven Types That Pay

> **Question: does A2 hold — can an instrument be added without touching an agent?**

Instruments as data, and enough of them to finish the conformance suite.

**Contents** — the instrument vocabulary §18 charges for and §7 never writes; the thirteen intrinsic
answers as a total mapping over the opening types (currency claim, deposit line, sovereign bond, secured
term loan, employment contract, goods unit, **listed and private equity** — §8.7 makes those two types,
not one); the three relational questions across seven regimes, compressed to a base weight times a
per-regime severity; `register` and every raise it owes; schedules as two constructors and one
interpreter; `amend` with four delta constructors, eight mechanisms and eight minted handles; the
due-tick index and its two bucket families; the seven option-terms tables, each naming the index its
event test fires from.

**The vocabulary is sized against §2.3, not against the conformance suite.** The suite needs seven types;
the model needs bills, commercial paper, repo, covered bonds, convertibles, contingent claims and the
rest, and the facts tables must be shaped so those are rows rather than edits. **§18.1's two worked
examples are this milestone's real acceptance test**: a covered bond and a convertible must each cost one
vocabulary entry, one intrinsic row, one relational row per regime and one option row against a terms
table that already exists — with **zero agent edits**. If either costs more, the facts tables are wrong
and this is where that is found, not at M9.

**Decides** — covenants and acceleration: an option family under §7.6, a ninth `amend` mechanism, or a
fourteenth intrinsic question (§2.3, and §18 prices the third as the expensive one).

**Why here** — the specification puts instruments in Phase 2 and then says in §15.1 that the conformance
suite needs a sovereign instrument type in Phase 1a. It noticed the contradiction and left the phase
table alone. This milestone is where that is resolved: the minimum kit rides with the ledger.

**Exits when** conformance cases 13–16 pass; the payment walk's crate closure contains no reference to
an instrument type code, checked over the dependency graph rather than by grep; `Regular` and `Explicit`
schedules produce identical due sets at every tick to the last minor unit; and adding a fourteenth
question fails to compile for every existing type until each is decided.

**Gate G1b — continue or stop on the instrument model.**

**Size** — 40–55 engineer-weeks.

---

### M4 — Clearing Without A World

> **Question: does one clearing interface really price everything, and does it terminate?**

*Runs in parallel with M2–M3. Not on the critical path.*

**Contents** — the sixty-four-bucket log-spaced grid and its single rebase; the two submission shapes;
the crossing solver; rule-two allocation before the walk; rationing from an offset of `tick mod n`; the
two-pointer settlement walk; and the period-0 reservation anchor for **every** venue family, not only
labour.

**The line taxonomy is sized against §2.3.** Repo, interbank unsecured, bills and commercial paper,
commodities as a read of the goods line, securities lending, insurance and trade credit are all *lines*
in the registry — a registry row and no code, under §18 — and the 4,096-line cap is checked against the
real count rather than against §9.5's current census. **A dealer submits on both sides of a line**
(§8.8), so the walk is exercised with a participant in both ordered lists from the first property test.

**Why here** — `markets` depends only on `domain` and `kernel`. It can be built, property-tested and
optimised against synthetic submissions before a world exists, and it is the largest block of economic
mechanism available that early.

**Exits when** a crossing is interior on a hundred thousand generated submission sets, or rebases exactly
once, or raises; both sides of every fill are equal integers by construction; the result is bit-identical
under permutation of submission order and under shard count; and every venue family has a dimension-checked
derived anchor entry.

**Decides** — the §9.5-versus-§13.1.2 contradiction. One composite good or twenty-seven cannot both be
right, and the line registry is built here.

**Size** — 25–35 engineer-weeks.

---

### M5 — Twenty-One Positions, Empty

> **Question: does the committed order, the manifest machinery and the generated composition root work,
> before any of it carries content?**

The loop runs. Every position is present. None of them does anything.

**Contents** — the position vocabulary as twenty-one stable names; the manifest schema as parsed data
(reads, writes, permitted counter-accounts, permitted amendments, owned series, cadence, selector,
phase, accumulators); the manifest-versus-order consistency check; the N3 check that refuses to mint a
trigger whose index would be a scan; the generated composition root and the 120-line host shim cap; the
period trace; the re-plan bucket index and the six trigger handles; the reason-code vocabulary and its
total mapping to flow classes; the observation store and its sub-caps; `plans` and `intents` as
pre-allocated tables.

**Why here** — §19 makes the composition root a Phase 1b exit criterion and the committed order a Phase 3
artefact, and §4.5 says the root is generated *from* the order. You cannot generate a root from an order
that does not exist. Doing the order first also converts every later system from an integration into an
insertion at a named position.

**Exits when** the loop runs 1,560 ticks with all twenty-one positions declared `NoContent{reason}`; the
digest is identical at one thread and at sixty-four shards and *differs* at a deliberately mis-ordered
fold, so the test is not vacuous; the root is generated with no hand-written line outside the shim; and
a manifest naming a capability it does not own fails to mint at start-up.

**Size** — 40–55 engineer-weeks.

---

### M6 — The First Franc

> **Question: can money reach a household, and does the loop close when it does?**

The smallest world that closes, driven by scripted intents rather than by deciding agents.

**Contents** — the opening world built by ledger operations from primitives; the tick-0 bilateral
monetary expansion; **the fiscal path that lets a government spend**, which the specification does not
have and without which nothing can start; bank deposit creation at position 19; the labour line and its
two-row settlement; production from the technology M4 settled; the retail exchange with its declared
sink; positions 2, 3, 4, 6, 12, 13, 14, 15, 17, 19 and 20 live and the rest declared empty; the C1/C2
closure identities as accumulators; the flow-of-funds report.

**Why here, and why this order** — §19 sequences Phase 4 as money, then credit, then equity, then the
public sector. **That order deadlocks at tick 1.** At tick 0 the four governments hold the entire money
stock and no employment contract exists, so §9.6.2's payroll precondition sends every firm into
resolution on the first tick. The public sector is not fourth; it is the only source from which the
first franc can reach a firm. The corrected order is government spending, bank deposit creation, labour,
production, retail — and everything else after.

**Exits when** a wage is paid by tick 5 and no firm enters resolution at tick 1; sectoral net lending is
exactly zero, per currency, every tick, for a thousand ticks; every counter-account's period delta equals
the sum of the journal rows naming it; and the world total of every claim asset is exactly zero at every
intermediate point.

**Gate G2 — the circuit closes.** Also where §3.4's workload is re-derived against the first real counts,
which the specification already schedules for G2 and which now has genuine content.

**Size** — 45–60 engineer-weeks.

---

### M7 — The Deciding Tick

> **Question: can an agent be five declarations and nothing else?**

The scripted intents of M6 are replaced by agents that decide.

**Contents** — the five-declaration interface as a total mapping per class; **declaration 4 returning a
reservation per side**, so a dealer's two-sided quote needs no interface change later (§8.8); traits drawn
rather than stored, including the political disposition §21 needs; the single-walk constraint evaluator;
staggering with the phase drawn from its own stream; the two-stage budget allocator; and the first five
agent classes written in full — **household, firm, bank, government, dealer** — with the rest declared
absent with their reasons.

**The dealer is written here and not later**, because it is the class that makes a price move for a
reason other than a binding constraint, and every clearing measurement taken after this point is
different depending on whether it exists. Its width is a read of its own funding, capital, risk aversion
and inventory — never a stated spread.

**Why here** — this is the first milestone whose content the specification does not contain. §8.1 gives
the shape of an agent and no content for any class: no consumption rule, no labour supply rule, no firm
pricing rule, no capital regime values. Writing four classes first, and writing them as specifications
before code per §17.4, is what turns the remaining five into repetitions rather than discoveries.

**Exits when** four classes carry full content and each of the other five is `Absent{reason}`; a class
declaring four of five items fails to compile; the reservation audit finds no scalar reaching a
submission that is neither an `Entry` nor a minted reservation; and re-planning is invisible in
aggregate demand, tested by a spectrum with no peak at the cadence.

**Size** — 70–100 engineer-weeks. *The first genuinely uncertain estimate in this plan.*

---

### M8 — Credit, Default And The Estate

> **Question: can a household lose its job, run down its deposits, default, and be foreclosed on — with
> no special case anywhere?**

§3.1's first functional requirement, end to end.

**Contents** — the household and corporate credit lines; underwriting as a constraint rather than a
rule; the bank capital regime **as a read of §21's parliament rather than a constant**; arrears, default
and the three amendment mechanisms credit owns; covenants and acceleration, in whichever shape M3 chose;
provisions distinct from realised loss; default testing at position 18 against real valuations; the
estate waterfall exercised by real failures, with **trade creditors ranked**; foreclosure returning a
dwelling to supply; lending standards that tighten as reads; bank equity issuance and a subordinated
layer to bail in; deposit pricing against a fund alternative; the issuer freeze and the payment-continuity
problem it creates.

**Why here** — credit is where the specification's machinery was designed to be tested and where the
assumption count grows fastest, so it is the first honest reading of M3's cost.

**Exits when** §3.1 requirements 1, 2 and 3 run without a special case; a bank whose capital ratio falls
through its floor cures or is resolved with depositors ranked against its estate; and the sharded
distribution of a fungible claim is per-holder identical at one shard and at four.

**Names a problem the specification does not have.** A bank resolution freezes every claim it issued for
up to fifteen ticks, during which every household holding that deposit fails its wages, rent and debt
service — each failure itself an insolvency trigger. Either a bridge mechanism is specified here, or a
cascading region is declared an accepted modelled outcome. It must not be discovered at M11.

**Size** — 60–85 engineer-weeks.

---

### M8b — The Elected Parliament

> **Question: can fiscal and regulatory policy be produced by the model rather than set in it?**

*Inserted after M8, and it keeps the number it was born with rather than renumbering what follows —
a step's id is its identity, not its rank.*

**Contents** — the parties as platforms on three axes (§21.1); the household's vote as a read of its own
rows plus its drawn disposition (§21.2); the founding election at tick 0 and the 208-period cadence
thereafter; seat allocation by §6.3 rule two; the median seat as the policy in force; and the conversion
of the government's declarations from stated parameters to **reads of the parliament** — the tax rates and
their progressivity, the spending level and composition, the bank capital floor and cure window, and the
loan-to-value and debt-service limits.

**Why here, and why not earlier.** It needs households that decide (M7) and the regulatory constants it
destroys (M8). **It must not come earlier**, and the reason is a rule the predecessor project paid to
learn: *a shape parameter stands in for a missing mechanism, and deleting it before the mechanism exists
makes the model wrong rather than more bottom-up.* M6's government spends on stated fiscal parameters
because it has to spend before anything else can happen; this milestone is where those parameters are
deleted **and** their mechanism built, in one step. That pairing is the only sequencing the design allows.

**Why it is worth its own bar.** It is the only milestone that **removes more priors than it adds.** The
tax rate, the capital floor, the loan-to-value limit and the transfer parameters were each going to be an
`assumed` entry standing where a mechanism belongs; a parliament that households elected replaces the lot
with three platform vectors and a seat count. Under M3 that is the best trade in the plan.

**Exits when** a household's vote is a function of no state it does not already hold — a stored confidence
index fails the check; no `assumed` entry with a `region:` scope exists among the platforms; seat
allocation is bit-identical at one shard and at sixty-four; **the four regions' parliaments differ**, and
the difference is traceable to their households rather than to any per-region value; and the fiscal and
regulatory constants M6 and M8 carried have no writer other than §21's policy vector.

**Defers** — coalition bargaining, confidence votes, early elections, party entry and exit, and
differential turnout. A government cannot fall in this edition, and §21 says so.

**Size** — 25–35 engineer-weeks.

---

### M9 — Property, Equity And The Rest

> **Question: do the remaining sectors add as declared insertions, as A2 and §18 promise?**

**Contents** — property and tenancy; installed capital and its separate resale line; plant vintages and
depreciation by kind; inventory at cost with lower-of-cost-and-market; product line entry and exit; M&A
with funding and acceptance; equity, dividends and distributions; funds and liability-matched
institutions; insurance; trade credit and invoices; demography at position 1; collateral and margin at 16;
the remaining three agent classes; the mark rule for `DerivedMark` instruments, which nothing currently
specifies and which private equity is the flagship consumer of.

**Plus the three rows §2.3 marks `new`, which are this milestone's real weight:** the **derivative
contract** — a bilateral instrument with a reference and variation margin, which §7.6's embedded options
are not — and the **clearing house** that novates and holds it; and the **securitisation vehicle**, which
*issues* against a pool where §6.5's trustee only *holds*. **Margin must be a read of the reference's own
realised move**: a stated margin rate cannot rise when it matters, which deletes procyclicality, and
procyclicality is the contagion mechanism.

**Why here** — if A2 and §18's change-cost table are true, this milestone is repetitive rather than
inventive. If it is not repetitive, the change-cost table is wrong, and that is worth finding out under
controlled conditions rather than at the end.

**Exits when** all seven agent classes carry content; the change-cost table's claims are measured against
the diffs this milestone actually produced and the table is corrected where it was wrong; and §3.1's
requirements 1–3 still pass.

**Size** — 70–100 engineer-weeks.

---

### M10 — Four Regions Trading

> **Question: does A4 hold when it is finally load-bearing?**

**Contents** — the six FX lines and the funding stage they clear in; cross-region trade; **FX forwards,
swaps and the cross-currency basis** — and a forward whose rate carries an interest differential, since
one that is spot moved by a basis is not a forward and cannot be checked against parity; reserves as a
read view; the four exact per-currency closure identities; **the balance of payments as a read of the
transactions, never a stored field**; the triangular residuals as declared series.

**Why here** — A4 is structurally true from M6, but trivially so while nothing crosses a border. This is
where it is first tested against a mechanism that could break it, and where §9.2's forgone intra-stage
reallocation first has a cost worth observing — which is what Appendix A's review point for cross-region
trade says.

**Exits when** §3.1 requirement 4 runs: a region imports more than it exports and finances the difference
by selling claims somebody in another region wanted at a price; net lending is exactly zero per currency
with FX live; and a rationed import is visible as a modelled outcome rather than a failure.

**Never** — the four regions and four currencies are always fully instantiated. This is the one scope cut
that would falsify A4 while leaving every book balanced, so it is a build failure rather than a schedule
decision.

**Size** — 45–65 engineer-weeks.

---

### M11 — Thirty Years At Full Class

> **Question: does a world built from primitives alone settle, and when?**

**Contents** — the full scale class on the device; the acceleration seam, if the measurements say it is
needed; the out-of-engine analysis harness; sixteen-seed ensembles; the burn-in gate's four tests with
the multiplicity correction §15.3 now requires; the nightly sensitivity sweep; the surface and its named
readers; the closure panel; **a separate PPI**, and the curves beyond the sovereign — secured,
swap-spread, credit by rating, commodity, cross-currency basis.

**Why here, and why the seam is last** — §12.3 says parallelism is a 1.0× assumption and the engine must
be correct and within budget single-threaded. A seam built earlier has no sequential baseline to prove
N1 against.

**Exits when** a `full`-class run reaches 1,560 periods on the device; the golden digest is identical at
one thread and sixty-four shards and between CI and the device; a `burnInPeriod` in [260, 520] is
recorded, or the failure is diagnosed by which of B1/B1b/B2/B3 failed on which series; and N2a, N2b and
N4 are reported against their targets, with any breach restating the target rather than the model (D1).

**Gate G3 — continue or stop on the economics.** The gate the specification admits nothing protects
against.

**Size** — 60–90 engineer-weeks.

---

## 4. The open decisions register

*Every admitted-owed, unresolved and silently-missing item, with the milestone that must settle it.
Sources are `PROJECT_AURORA.md` unless stated.*

| Item | Source | Settle by |
|---|---|---|
| N4's itemisation — ~705 MB unaccounted | §12.1 | **M0** (paper) |
| Identifier census: 971,000 against an implied 12.45M | §3.4, §5.2 | **M0** (paper) |
| The 3,119,665 operation count, per position | §3.4 | **M0** (paper) |
| The journal row's field list against 48 B | §6.6 | **M0** (paper) |
| The 44 B versus 148 B instrument row | §7.5 *Unresolved* | **M0** decides on paper; M3 confirms |
| Transcendental bit-identity, CI against device | §11 | **M0** (probe) |
| Household block occupancy against ten slots | §3.4 | **M0** (paper) |
| ~~The tick-0 bootstrap~~ **Derived: §13.6** | was *silently missing* | **M6** builds it; M0 no longer traces it |
| ADR template, numbering and register | *silently missing* | **M0** |
| The numéraire's upper bracket | §5.3 *Owed* | **M1** |
| Retirement queue capacity against drain interval | §5.5 | **M1** |
| Digest cadence decoupled from checkpoint cadence | §13.5 | **M1** |
| The per-asset holder index — five mechanisms need it | *silently missing* | **M1** |
| Counter-account ownership split by unit class | §6.2 | **M2** |
| Escrow entity retirement at resolution close | *silently missing* | **M2** |
| The instrument type vocabulary (§7.1 does not exist) | §18 charges for it | **M3** |
| The relational table's 21 values | §7.3 *Owed* | **M3** |
| ~~Q10 cannot express "not a household"~~ **Derived: no rule needed** — the goods leg lands in the sink | §9.6.3 | closed |
| The amendment permission matrix, 7 types × 8 mechanisms | *silently missing* | **M3** |
| ~~Technology: one or twenty-seven~~ **Derived: a floor of three** | §13.1.2 | **M4** confirms the count above the floor |
| §9.5's 27 goods lines — **withdrawn**; the census re-derives from the sub-unit count | resolved by the above | **M4** |
| The period-0 grid anchor for every non-labour venue | §13.1.1, cites a missing §9.6.1 | **M4** |
| §13.3's four non-reproducing count vectors | measured, red | **M4** |
| The trailing-statistics system has no position | §13.4 | **M5** |
| How a fill at 13/14 becomes an instrument at 19 | *silently missing* | **M5** |
| `regimeOf(venue)` is undefined for 33 of 37 venues | §7.3 | **M5** |
| The fiscal rule, including the tick-0 headcount decision §13.6 requires | *silently missing* | **M6** |
| **Who holds the opening capital and dwellings** sourced at position 6 of tick 0 | *silently missing*, surfaced by §13.6 | **M6** |
| The production specification | Appendix B *Owed* | **M6** |
| The five declarations for seven agent classes | §8.1 — a shape with no content | **M7**, **M9** |
| ~~SME and Large firm: two classes or one?~~ **Decided: one `Firm` class; tier by block pressure, listing by decision, the two independent (§8.7).** What remains is economics: the listing requirement's ratios and their brackets, and the funding-policy rule that makes a firm want to list | §8.4, §8.7 *Owed* | **M7**, with the firm declarations |
| ~~Is a dealer an agent class?~~ **Decided: yes (§8.8).** It differs on mandate and on valuation — two reservations, not one — and needs no change to §9.1, since a dealer is a participant twice. What remains is its width's content | §8.8 *Owed* | **M7** |
| The party platforms, the party and seat counts, the disposition dispersion | §21 *Owed* | **M8b** |
| Whether a household's vote reads a forward expectation — the model has no expectations mechanism, so "outlook" is currently recent experience | §21.5 *Owed* | **M8b** |
| **Covenants and acceleration**: an option family under §7.6, a ninth `amend` mechanism, or a fourteenth intrinsic question | §2.3 | **M3** |
| **The derivative contract shape and a clearing house.** §7.6 gives options embedded in an instrument, not a bilateral contract with a reference and variation margin. Margin must respond to the reference's own realised move, or procyclicality — the contagion mechanism — is deleted | §2.3, Appendix C 25 | **M9** |
| **The securitisation vehicle.** §6.5's trustee *holds* for a class of holders; an SPV *issues* against a pool | §2.3 | **M9** |
| **Is freight in scope at all?** A network with capacity per route is genuinely new structure. Under R20 the options are to model it or declare it out of scope on the surface | §2.3 | **M9** |
| ~30 per-position §17.4 specifications, 1 written | *silently missing* | **M6**–**M10** |
| The trait declaration set | *silently missing* | **M7** |
| The bank capital regime's values and cure window | *silently missing* | **M8** |
| Issuer-freeze payment continuity | *silently missing* | **M8** |
| ~~The insurer / pension split of 40~~ **Derived: not split.** One `Liability-matched institution` class of 40 | §8.4 *Owed* | closed |
| §8.2's withdrawn accumulator and the price-change rule | §8.2 *Owed* | **M9** |
| The `DerivedMark` derivation rule | §7.2 Q13 | **M9** |
| δ₁ = 0.35 — **derived: justified by a named in-model mechanism or not at all** | §13.3 *What is owed* | **M11** names it |
| **B1b is mis-specified, and it is not a multiplicity problem.** `\|Δμ\| ≤ 0.25·σ_pooled` is a fixed effect size with no reference to sampling error, so it is a ~53rd-percentile cut on a stationary AR(1) and rejects the majority of genuinely settled series. **Measured**: the 42-series conjunction passed 0 of 2,000 panels. §16.2 would then classify a healthy model as defective at period 520 | measured by `aurora-tools burnin` | **M0** states it; **M11** sets every threshold from its own null distribution, as B3's 0.503 already is |
| Burn-in multiplicity over 168 hypotheses — a **separate** problem from the above, and fixing one does not fix the other | §15.3 | **M11** |
| **§13.3's count rows were not produced by its own generator.** The continuous half reproduces exactly — every share and multiplier to six decimals — and the integer half reproduces under neither §6.3 rule two nor largest-remainder. §13.1 rule 3 forbids a per-region count being typed anywhere, and these appear to have been | measured by `aurora-tools seedgen` | **M0** records it; §13.3 is regenerated before its counts are an input to anything |
| **Cohort shares cannot vary by region under §13.3's formula.** The axis-3 loading multiplies every cohort of a region by the same factor and renormalisation divides it straight back out — so a primitive §13.3 lists on axis 3 is invariant across regions by construction | measured by `aurora-tools seedgen` | **M4**, with the seed |
| Observation family 13: 21 × 64 exceeds its sub-cap | §14 | **M5** |
| Observation family 9: 60 series against 224 identities | §14 | **M6** |
| Eight dangling cross-references | §3.4.4, §9.6.1, §15.3.4, §21.3, D-7 | **M0** |

---

## 5. Standing practices

Things that run continuously rather than sitting in a milestone.

- **Specification before code.** §17.4 is the definition of done and applies from M2 onward: a system's
  specification is written first, its manifest declares exactly what it reads and writes, its
  capabilities are minted no wider, its numbers are registry entries, its outputs are declared series, it
  has a conformance case or a stated reason it needs none, and the specification still describes it.
- **The assumption census is published every build** — total, assumed, structural, placeholder, and the
  trend of each — and carried in every run manifest, so no result is quotable without its prior count
  (D3). Every review asks what each new entry buys.
- **Nightly**: the golden digest on CI and on the device; N3's two limbs; the benchmark suite against
  published noise floors; from M11, the sensitivity sweep at `tenth` and weekly at `full`.
- **A digest re-baseline requires a written explanation** naming the intended behaviour change and
  whether retirement order or shard count moved (R6). A re-baseline without one is refused.
- **An ADR carries a named mechanical guard or it is not accepted** (Appendix A). A diff touching a
  registered value without one is refused. While a schema file is under initial construction the
  coupling is off and the milestone exit ratifies the whole file; after ratification it is per-value.
- **Every check has a negative fixture.** A check that has never seen a violation is not known to detect
  one.

---

## 6. Stop conditions

Real ones. Under D1 no performance finding is among them, which is a deliberate consequence of D1 and
narrows this list considerably.

1. **A1 cannot be discharged (G1a).** If a conserved quantity can be created without an operation naming
   its source, and the crate boundary cannot be made to prevent it, M5 is not a rule this design can
   keep and the design is wrong rather than late.
2. **A2 is false in practice (G1b).** If adding the seventh instrument type requires touching an agent,
   the facts tables are not doing their job and every later milestone's cost estimate is wrong by an
   unknown factor.
3. ~~**The world cannot start.**~~ **Retired.** The model rules settled it: §9.2 budgets stage-2
   spending against balances that are facts, so no firm bids at tick 0 and no firm fails at tick 1. The
   government is the only agent with a balance and therefore the first spender, and it spends through
   the ordinary labour line. **`PROJECT_AURORA.md` §13.6 now carries the derivation.** What survives of
   this condition is narrower and belongs to M6: *if the circuit does not close within the burn-in floor
   of 260 periods*, the opening is not merely slow but wrong.
4. **The loop does not close once it carries content (G2, M10).** Sectoral net lending not exactly zero,
   with the reader exonerated, means a flow has an unmodelled counterparty.
5. **The burn-in gate cannot pass (G3).** Reaching period 520 without a pass is the specification's own
   defect condition and the first evidence this design has ever had that the economics is wrong rather
   than merely unfamiliar.
6. **The assumption count grows without bound (M7–M9).** D3 removed the cap, not the rule. If each new
   mechanism costs five priors to buy one behaviour, M3 is not being served and the model is being
   assumed into existence rather than produced.

**The residual the specification admits, and this plan does not remove:** a project can pass G0, G1a,
G1b and G2 and still fail at G3, having spent everything but the last milestone. G3 sits where it does
because the economics has no evidence at all before an end-to-end period exists, and a gate cannot sit
earlier than its evidence. What this plan does about it is get the first closed circuit to M6 rather
than to the end, and write four agent classes at M7 rather than nine — so the first real economic signal
arrives at roughly the halfway point rather than at the finish.

---

## 7. What this costs

| Milestone | Engineer-weeks |
|---|---|
| M0 The Refusing Workspace | 55–65 |
| M1 One Arena, One Digest | 50–70 |
| M2 One Crate Writes Money | 45–60 |
| M3 Seven Types That Pay | 40–55 |
| M4 Clearing Without A World *(parallel)* | 25–35 |
| M5 Twenty-One Positions, Empty | 40–55 |
| M6 The First Franc | 45–60 |
| M7 The Deciding Tick | 70–100 |
| M8 Credit, Default And The Estate | 60–85 |
| M8b The Elected Parliament | 25–35 |
| M9 Property, Equity And The Rest | 70–100 |
| M10 Four Regions Trading | 45–65 |
| M11 Thirty Years At Full Class | 60–90 |
| **Total** | **630–875** |

**That is 13–18 engineer-years; at four engineers, roughly three to four calendar years.**

**§2.3's surface changes this table and the change is not yet fully made.** M7–M10 were sized against
"the economics" as an idea; §2.3 is what the economics consists of, and it is larger than those bars were
estimated against — the three rows it marks `new` (derivatives with a clearing house, securitisation
vehicles, freight) are each a real build inside M9. **The re-derivation is owed at G2**, where the first
real counts exist, and until then these figures are a floor rather than an estimate.

**Where the uncertainty is.** M0 through M6 are estimated against a specification that describes them,
and the range there is the ordinary one. M7 through M10 are estimated against a specification that does
*not* contain them — no production section, no content for thirty-five declarations, roughly thirty
per-position specifications owed and one written — and those four milestones are 245–350 of the total.
**The honest statement is that the second half of this plan is an estimate of work that has not been
specified**, and the first thing M7 does is find out how wrong it is by writing four agent classes and
measuring what they actually cost.

Three earlier candidate plans sized the economics at a third to a half of what is above. Each was
independently judged to be under by 1.5–3×, for the same reason: they priced a milestone by its unit
cost after A2 makes each system an insertion, and never counted the units.

---

## 8. Working the plan

**A milestone file lists remaining work, never completed work.** When an item is done it is **deleted**,
in the same commit that completes it, with the commit message naming what it was. Git history is the
record; the file is the state. A milestone is finished when its work-item tables are empty and its exit
criteria pass — which means progress is legible at a glance rather than inferred from a column of ticks.

**A defect found mid-flight is never fixed opportunistically.** It is classified, written up as a new
numbered step, and resolved at the point in the plan that owns it. Three buckets:

| The defect | Where it goes |
|---|---|
| Blocks the current milestone's exit | A new step in the current milestone, inserted at the right place in its dependency order |
| Is in code a later milestone owns | A new step in that milestone. It waits, and the waiting is visible |
| Is a defect in the specification rather than the build | A row in §4's open decisions register, assigned to the milestone that must settle it, and an ADR if it touches a registered value |

**Why not just fix it.** An opportunistic fix is a change nobody scheduled, made under the pressure of
whatever else was being done at the time, in a milestone whose exit criteria do not mention it. The
project's own standard already covers this: Appendix A refuses a diff that touches a registered value
without an ADR, for the same reason. **Work that disappeared without being named is the same failure as a
value that changed without being named** — and a bug fixed silently in the wrong milestone is both.

The one exception is a defect that makes the current work unbuildable — a broken build, a check that
cannot run. That is repaired immediately, and the repair still gets its numbered step, written
afterwards.

---

## 9. Honest weaknesses of this plan

- **M7–M10 are four boxes around a hole.** They are the subject of the title and they are the part
  nobody has specified. Decomposing them into four gated bars with their own assumption draw is better
  than one bar, and it is not the same as knowing what is in them.
- **G0 has no stop authority and this plan says so** rather than inventing thresholds. D1 removed
  performance as a reason to stop, which is right, and the cost is that the first eight to ten weeks
  produce decisions and evidence but no decision point.
- **M4's independence is real but its output is thin.** Clearing correct against synthetic submissions
  proves termination, shard-invariance and integer equality. It proves nothing about whether a price
  formed this way is an economically sensible price, and it will feel like more progress than it is.
- **The closure assertion is trivially true before M6 and weakly true until M10.** A world where nothing
  can leak cannot demonstrate that it does not leak. The assertion acquires content in stages, and the
  plan should not be read as though M5's green closure test means anything.
- **Nothing here protects the burn-in gate.** If the economics does not settle, that is discovered at
  M11 with almost everything spent. The mitigation — an earlier closed circuit, four classes before
  nine, partial gate instrumentation from M6 — reduces the exposure and does not remove it.
