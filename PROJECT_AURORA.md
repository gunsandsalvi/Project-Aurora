# Project Aurora

## A bottom-up simulation of four national economies

**Version 6.0.** This document is the specification. It is written to be built from, and its
organising rule is **one fact, one place**: every number appears once, in the section that owns it,
and everything else points there. A figure printed twice is a figure that will disagree with itself.

**Version 6.0 records four founding decisions taken by the project owner (D1–D4, Appendix B).** They
change the language, the delivery target, the status of the performance budgets and the status of the
assumption cap. Everything downstream of them is re-derived here rather than left to agree by accident.
**The six model rules of §1.1 are the ends of this project; everything mechanical in this document is an
instrument serving them, and an instrument that stops serving its rule is renegotiable.**

### How to read this

| If you want | Read, in this order |
|---|---|
| **what this project is for** | **§1.1, the six model rules. Read this before anything else** |
| what the model contains | §1, §2, §8, §7, §13.3 |
| the rules you cannot break | §3.2, then §5 and §6 |
| what happens in a tick | §9.4 first, then §9.1, §9.2, §9.3 |
| where a number comes from | §16.1, then §13. **Nowhere else** |
| how correctness is established | §15, then §11 |
| how to build it | `IMPLEMENTATION.md`, then §4, §17, §18 and §19 |
| what changed and why | Appendix B, and D1–D4 at the head of it |
| what is not yet decided | Appendix B's **Owed** list. It is long, and most of it is the economics |

**Sections are ordered by number, not by reading path.** The numbers are stable addresses cited
from source comments and from the construction plan, so a section keeps its number even when it is
rewritten.

### Contents

| | | |
|---|---|---|
| **1** What this is — **M1–M6** | **8** Agents | **15** Correctness |
| **2** Scope | **9** Markets | **16** Cross-cutting concerns |
| **3** Requirements — A1–A4 | **10** Time and capabilities | **17** Standards |
| **4** Layers | **11** Determinism | **18** Extension |
| **5** State | **12** Performance | **19** Delivery |
| **6** The ledger | **13** The seed | **20** Risks |
| **7** Instruments as data | **14** Observation | **A–D** Appendices |

### Conventions

- **A rule is stated imperatively and once.** Where a reason is given it is one sentence. Extended
  argument for a settled value lives in Appendix A's register and nowhere else.
- **M-numbers are model rules and are the ends; A-numbers are architectural claims serving them; N-numbers
  are engineering figures.** An N-number never overrides an M-number (D1).
- **Every value declares its provenance** — `structural`, `derived` or `assumed` — in §13, and the
  build refuses one that does not (§16.1).
- **A cost that a decision accepts is stated where the decision is**, in italics, beginning
  *Accepted cost*. A design with no stated costs is a design whose costs have not been found.
- Region indices are **0–3**. Display labels live in `surface` and carry no information.
- A tick is one **period**; a period is one simulated week; a run is 1,560 periods.

---

## 1. What this is

Four national economies, simulated agent by agent, from primitives. Five hundred thousand
households, fifty thousand small firms, four hundred and forty large ones, and the banks, funds,
insurers, pension funds, governments and central banks they transact with — **550,598 deciding
entities**, each a row, none a representative of anybody else.

The model produces prices; it is not given any. It produces regional difference; it is not told
what the differences are. It produces unemployment, default, insolvency and financial crisis as
outcomes of mechanisms, or not at all.

### 1.1 The six rules

*These are the project's requirements. Everything else in this document is machinery in their service.*

| | Rule |
|---|---|
| **M1** | **Fully bottom-up.** Every deciding entity is a row that decides for itself. No representative agents, no aggregate equations, no scaling factor between a model household and anything outside the model |
| **M2** | **Realistic in mechanism, not fitted to outcomes.** A bank fails because its capital ran out, not because a failure rule fired. The model is never calibrated to an observed economy; realism is that the machinery is right |
| **M3** | **Minimum priors.** The fewest assumed numbers possible. The count is a headline figure, reported on the surface, and every review pushes it downward |
| **M4** | **Closed.** Every flow has a modelled source and a modelled sink. No rest of world, no residual absorber, no counterparty that exists to make the books balance. **An unmodelled counterparty is a prior wearing a costume** |
| **M5** | **Nothing appears or vanishes** except where something in the model creates or destroys it and says so |
| **M6** | **Emergence, not assertion.** Unemployment, default, insolvency, crisis, regional difference and the price level are outputs. A phenomenon that must be built in to appear has not been explained |

**Where a mechanism in this document conflicts with a rule above, the rule wins and the mechanism is
restated.** §12's performance targets are the standing example: they bend, and the model does not (D1).

### 1.2 The four architectural claims

**Each is a discharge of a rule above, and each is mechanical rather than a review question.**

| | Claim | Serves | Discharged by |
|---|---|---|---|
| **A1** | Conservation is structural. A quantity cannot be created or destroyed except by an operation that names where it came from | M5 | the conserved column is private to the `ledger` crate and no mutable borrow of it exists outside it; no mintable `post` handle for it (§6.1a) |
| **A2** | A new instrument type costs one vocabulary entry, one row of intrinsic facts, one row of relational facts per regime, and zero edits to any agent | M1, M6 | the facts tables are total mappings; a branch on instrument type outside them does not compile (§7.2, §7.3) |
| **A3** | No level is seeded. Every number in the opening world is structural, derived, or an assumed dimensionless ratio with a bracket, and the assumed count is published | M2, M3 | the registry build check, and the reservation mint (§16.1) |
| **A4** | The loop closes. Every flow has a modelled source and a modelled sink, and the four regions are the entire world | M4 | no rest-of-world sector exists to be written; the issuer precondition at `register` (§9.7) |

**A4 is why there is no rest of world.** A sector that absorbs whatever is left over is an
unmodelled counterparty wearing the costume of a simplification, and under it every closure test
passes while the model leaks.

## 2. Scope

### 2.1 In scope

Price formation in multiple venues; production, consumption, employment, tenancy and investment;
credit, default and resolution; bank balance sheets and the constraints that bind them; monetary
and fiscal policy as rules over the model's own state; cross-region trade financed on modelled FX
lines. A run is 1,560 weekly periods — thirty simulated years. **The delivery target is an Android
application: a native engine with a thin user interface** (D2).

### 2.2 Not in scope

- **Calibration to an observed economy.** A3 forbids it. The model is compared to itself (§15.3).
- **Forecasting.** Nothing here is fitted, so nothing here predicts.
- **A rest of world.** A4 forbids it. A phenomenon that genuinely needs an outside is either
  modelled as a sector, an agent class or a region primitive, or declared out of scope here and
  said so on the surface. **An invented producer is not a placeholder; it is a mechanism nobody
  specified.**
- **Anything above the tick.** Intra-week timing, order-book microstructure and settlement lags
  shorter than a period are not represented.

## 3. Requirements

### 3.1 Functional

The model must be able to express, without a special case anywhere:

1. A household that loses its job, runs down its deposits, defaults on a mortgage, and is
   foreclosed on by a bank that then holds the dwelling.
2. A bank whose capital ratio falls through its floor, which cures within its window or is
   resolved, with its depositors' claims ranked against its estate.
3. A firm that cannot make payroll, fails to settle a due obligation, and enters resolution
   through the same door as every other insolvency.
4. A region that imports more than it exports and finances the difference by selling claims that
   somebody in another region has to want, at a price, or the trade does not happen.
5. A central bank whose policy rate is an output of the world's own realised return on capital.

### 3.2 Architectural — the four requirements

Stated in §1 and discharged as follows. **Each is a build failure, not a review item.**

**A1 — conservation is structural.** The conserved quantity column is a **private field of the `ledger`
crate**, and no mutable borrow of it is obtainable anywhere outside that crate — not by a lint, by the
compiler. `post` cannot address a conserved column, because the mint is typed and no such overload
exists. Every operation appends exactly one journal row.

*The property is about a module boundary, not a statement count.* Relocation (§5.5), the zeroed-entry
tail shift (§5.6) and slot canonicalisation all write the quantity column, and all three live inside the
ledger. A claim of "exactly one writing statement in the source tree" is false against them, and a check
enforcing it would have to grow the exemption list §17 forbids. What is claimed, and what is
mechanically true, is that **the set of code able to write a conserved quantity is one crate whose entire
public surface is the nine operations.**

**A2 — the instrument model is additive.** Intrinsic facts are a total mapping from type to
thirteen answers; relational facts are a total mapping from (type, regime) to three. An agent
receives *facts*, never a type code, so `if (type === ...)` is not writable inside one.

**A3 — no exogenous calibration.** Every entry in the opening seed carries a provenance, and
`assumed` admits only `ratio`, `count`, `period`, `physical-unit` and `hour` dimensions, never a
level, and never a region scope. A reservation level formed before any price exists is minted in
one module against a declared source.

**There is no cap on the number of entries (D1, D3).** The provenance rules above are build failures and
stay build failures; the *count* is a published figure that every review pushes downward (M3). A cap
decides modelling questions by arithmetic — §13.1.2 records a technology reduced from twenty-seven
sub-units to one because eighty-one exceeded eighty — and a rule that answers a modelling question with a
budget is not serving M2. **What replaces the cap is visibility: the assumed count, its trend, and the
mechanism each entry buys, on the surface beside the placeholder count.**

**A4 — the loop closes.** Every claim asset carries a non-null issuer, enforced at `register` (§9.7). The
entity-to-(region, sector) mapping is total with no `external` member. Counter-accounts may hold
only real units, so none can mint a claim. Sectoral net lending sums to exactly zero, per currency,
every tick — an integer identity, not a tolerance (§9.7).

### 3.3 Non-functional

**Two kinds, and the difference is which way they bend (D1).** A *requirement* is a correctness property:
a breach is a defect and stops the work. A *target* is a performance figure: it is measured every night,
its trend is published, and **a breach never licenses a reduction in agent count or a coarsening of a
cadence** — the target is restated and the delivery bends instead.

| | Requirement | Where |
|---|---|---|
| N1 | The golden digest is identical at one thread and at 64 shards | §11 |
| N3 | No per-tick cost is proportional to stock where it can be proportional to activity | §12.2 |
| N5 | A save round-trips bit-identically | §13.5 |
| N6 | The conformance suite passes before the first economic system exists | §15.1 |

| | Target | Where |
|---|---|---|
| N2a | A tick completes within **680 ms** on the target device | §12.1 |
| N2b | A long run sustains **1.70 ticks/s** | §12.1 |
| N4 | Peak allocation stays under **1,610 MB** | §3.4, §12.1 |

**The target device is a 16 GB Pixel-class phone, and the delivery is an Android application** (D2).
**N3 is a requirement and not a target**, because a cost that grows with stock is a structural defect
rather than a slow program: it says the design has a pass over the world in it.

**`full` is the only scale class that is this model.** `half` and `tenth` exist to make tests and nightly
sweeps affordable, and a result quoted from either is a result about a different world (§12.4).


---

### 3.4 The workload

Every budget in this document is derived from this section, and the arena is sized from it.

**Entities: 550,638 at tick 0** — 550,598 deciding, plus 40 counter-account rows. Counts by class
are in §8.4; the regional split is in §13.3. Identifiers ever issued by tick 1,560: ≈ 971,000, and
none is reused.

**Holdings are blocks, and the table does not grow.** Each party class has a declared slot capacity,
allocated once:

| Class | Slots each | Holders | Slots |
|---|---|---|---|
| Household | 10 | 500,000 | 5,000,000 |
| SME | 16 | 50,000 | 800,000 |
| Large firm | 64 | 440 | 28,160 |
| Bank | 16,384 | 56 | 917,504 |
| Fund | 4,096 | 54 | 221,184 |
| Insurer | 4,096 | 22 | 90,112 |
| Pension fund | 4,096 | 18 | 73,728 |
| Government | 1,024 | 4 | 4,096 |
| Central bank | 1,024 | 4 | 4,096 |
| Counter-account | 64 | 40 | 2,560 |
| | | | **7,141,440 slots, 24 B each, 171.4 MB** |

Blocks are sized for the tail, not the mean, because **exhaustion is a halt**: a mean household
occupies about three of its ten slots once the negative side of what it issued is counted.

**The slot is 24 bytes and its field list is published here**, because a width printed without a schema
is a number that cannot be checked: asset `i32` (4), quantity `i64` (8), balance-tick integral `i64` (8),
`integralUpdatedAtTick` `u16` (2, since 1,560 < 2¹⁶), 2 bytes padding. **The previous edition's 20 bytes
could not hold what §6.11 requires** — asset, quantity and integral alone exhaust it, leaving no tick
column — so 20 B was an unsourced constant that the rest of the document had already contradicted.
Encumbrance is *not* on the slot: it is derived from root lien rows through a per-(holder, asset) index,
because liens are institutional and rare and a per-slot flag would cost 7.1 M bytes to serve tens of
thousands of rows.

*Accepted cost.* 28.6 MB against N4, which is a target and not a requirement (D1).

*Owed.* §6.6's 48-byte journal row is likewise printed without a field list, and an `exchange` row must
carry two parties, two assets, two quantities, a cleared rate, a realised rate, a reason code and an
actor. Publish the packing, or publish the row width the packing needs.

**The flow model: 3,119,665 operation calls per tick**, derived position by position rather than
from stocks. Every operation appends exactly one journal row, so the operation count and the
journal count are the same number. The four largest terms are position 14's 1,671,884 retail and
market exchanges, position 4's 926,246 per-contract payments, position 13's clearing work, and
position 6's 327,886 production moves.

**Cadences.** A system does not run on a tick it has no content for. Household re-planning is 13
ticks, SME 4, everything else 1; monetary policy 13, fiscal 52; depreciation 13; deposit interest
13; dividends and distributions 13; taxes 13; pensions and transfers 4.33; the property and
installed-capital price lines 4; checkpointing 64. **Wages, rent, debt service and consumption fire
every tick from the standing plan**, and that is what keeps A4 intact under staggering.

**Raising or lowering the household count is a change to this section**, not a resolution knob. If
any figure here moves by an order of magnitude, the sections derived from it are re-derived rather
than assumed to still hold. **Under D1 it may never be lowered to meet a performance target**, which is
the only reason anybody would want to.

*Owed.* The 3,119,665 figure is stated as derived position by position and the derivation is not shown.
A twenty-one-row table of operation counts summing to a published total — with clearing's sort and
accumulate cost separated from operation-call cost — is what §12's targets are decomposed against, and
it does not yet exist.

*Owed.* §3.4's ≈ 971,000 identifiers ever issued and §5.2's 47.5 MiB directory at 4 bytes an identifier
imply ≈ 12,450,000. The two differ by a factor of thirteen. The directory's size, the digest's
identifier-order walk and the save all depend on which is right.

## 4. Layers

**Eleven layers, and each is a crate in one Cargo workspace** (D2). `X → Y` means X may depend on Y, and
a crate declares exactly the dependencies its row permits. **A forbidden import is a compile error**, not
a lint with an exemption list to keep empty: the dependency does not exist, so the module cannot be named.

*This is the largest thing D2 bought.* The previous edition enforced the layer graph with a source-tree
check whose exemption list had to be defended forever. A workspace enforces it in the resolver, which
cannot be argued with and cannot be exempted.

```
composition ──► everything                              (leaf: imported by nothing)
runtime     ──► declarations ──► domain ──► kernel
systems ──┬──► agents ──┬──► markets ──► domain
          │             └──► world ──► domain
          ├──► ledger ──► world
          ├──► declarations
          └──► markets
surface     ──► world read views, observation store, domain   (leaf)
```

| Crate | Contents |
|---|---|
| **kernel** | storage primitives, typed columns, identifier machinery, quantity types, code generation. Knows no economics |
| **domain** | vocabulary and pure arithmetic. No state. Also the parallel vocabulary: `RowSpan`, `ShardIndex`, `Selector`, `Cadence`, `Phase`, `Accumulator<T>` |
| **declarations** | one manifest per system: reads, writes, permitted counter-accounts, permitted amendments, owned series, cadence, selector, phase, accumulators. Pure data |
| **world** | one module per table: schema, allocation, generated read views, span arithmetic |
| **markets** | price formation, deliberately independent of `world` so it can be tested and optimised alone |
| **ledger** | the only module that can obtain a writable view of holdings, liens or obligations, and therefore the only one that can mint a handle over one |
| **agents** | policy, not work: one module per agent kind, declaring the five items of §8.1 |
| **systems** | the work of a position |
| **runtime** | the loop, the committed order, the period trace |
| **surface** | named readers and the view model handed to the user interface. **Computes nothing** |
| **composition** | wiring. Generated |

**The user interface is not a layer; it is a separate application** (D2). The engine is a native library
with one boundary — a generated foreign-function interface exposing the readers of `surface` and the
run-control verbs of `runtime`. The interface may compute what a user interface must (layout, formatting,
pixels); it holds no world handle and can write nothing. §4.4's prohibition binds `surface`, which is the
last thing inside the engine, and that is what keeps it enforceable.

**Two relocation prohibitions.** Never move a system *down* a layer for performance — a system that
needs to be fast declares itself fast and stays where it is. Never move a module *up*, or into
`composition`, for convenience: "it needs to see two layers at once" is a missing declaration and a
design finding, not a licence.

### 4.3 General principles

- **Total over partial.** A mapping the compiler can check exhaustively, rather than a lookup with
  a default. A missing case is a build failure.
- **Derive, do not store.** If a fact is computable from another, compute it. A stored copy is a
  second representation with nothing forcing the two to agree.
- **No ambient state.** Everything a system needs is passed to it. No module-scope mutable
  singleton, so two worlds in one process cannot share anything.
- **Make the wrong thing unwritable rather than detected.** A precondition at a door beats a check
  after the fact; a type beats a precondition.

### 4.4 The surface computes nothing

Every displayed number comes from a named reader. A lint forbids arithmetic operators in `surface`.
A quantity computed twice — once in the engine and once in a status bar — is two implementations of
one rule, and the first place anyone looks when they disagree.

### 4.5 The composition root

| # | Guard |
|---|---|
| 1 | **Generated, not written**, from the declaration set and the committed order. **Hand-written source is one host shim — clock, thread construction, capability probe, the foreign-function boundary — capped at 120 lines, and a hand edit anywhere else fails the build** |
| 2 | **It decides nothing**: no conditional over a world-derived value, no arithmetic over a world-derived type, no loop over entities |
| 3 | **It mints nothing** except the clock handle, and holds no writable view of any table |
| 4 | **It runs once.** Anything that must run per period is a system at a named position |
| 5 | **Nothing depends on it**, which is what lets the wall clock, thread spawning and the foreign-function boundary live there |

---

## 5. State

### 5.1 Tables, not an object graph

World state is tables of typed columns: entities, instruments, schedules, schedule rows, schedule
deltas, instrument options and their seven terms tables, holdings, liens, the journal, the
resolution register, `plans`, `intents`, and the observation store. **No object holds fields.** A
firm is a row and its balance sheet is a query.

Columns are typed slices over one owned arena allocation: identifiers and enumerated codes as `i32`,
conserved quantities as `i64` in the asset's smallest unit, prices and rates as `f64`.

**Conserved quantities are `i64`, not floats constrained to the safe-integer range** (D2). The previous
edition carried `|q| < 2^53` because its language had no integer type; with a native engine the
constraint is the type, the range is ±9.22 × 10¹⁸, and an overflow is a panic rather than a silent loss
of the low bit. §6.3's arithmetic is unchanged and its bound is now enforced rather than asserted.

**There is no prices table.** A priced thing's mark lives in its own instrument row, because a
second table keyed by the same identifier must be relocated in lockstep when a row moves.

**`plans` and `intents` are state**, not results. They are pre-allocated, saved, digested and
inspectable, and **no decision system allocates a result object**.

### 5.2 Identity

Every identity space — entity, instrument, venue, lien, schedule, series — is a **newtype over `i32`
with a private field**, minted by a named constructor, with one module per space owning it. Identifiers
are dense integers issued from a deterministic per-space counter at the moment of creation.

**These are nominal at runtime, not only at compile time** (D2). The previous edition's brands were
erased before the program ran, which is what made R18 the register's largest risk; a newtype with a
private field cannot be forged from an integer anywhere outside its module, and there is nothing to
erase.

- **A lookup miss is an error, never a default.** Never a zero, never a blank row.
- **An identifier is never reused.** Reuse silently re-points journal rows, digests and the
  per-(stream, entity) generators at a different subject.
- **A crossing between identity spaces is an explicit named conversion**, so crossings can be
  counted. There is no conversion from an instrument identifier to a holder position: a lien's
  beneficiary is always an entity, and where the economic beneficiary is a class of holders, an
  entity is registered to hold for them.
- **The identifier directory is the only structure whose size may depend on run length** — 4 bytes
  per identifier ever issued, 47.5 MiB over thirty years. A second such structure is a defect.

### 5.3 Quantities, units and money

Quantities carry their unit in the type: face value, share counts, fund units, physical units,
floor area, labour hours, and money in each currency are distinct types.

**The only route from a quantity to a value is `quantity × price`**, with one exception: a claim
with no venue is valued at par through the named reader `parValueOf`, which is `structural` —
definitional arithmetic, not a level.

**The currency is the unit of account, not a holdable asset.** What is held is a claim denominated
in it: `Notes:<cb>` and `Reserves:<cb>`, both liabilities of a region's central bank.

**The numéraire is `S = 2 × 10¹¹` minor units, `structural`, identity `UnitOfAccount`** — never an
`assumed` scale factor, because §6.3's quantization does not commute with scaling and calling it a
scale factor would imply it does.

*Owed: `S` has a lower bound and no stated upper bound, and the upper bound is the tighter one.* §13.2
fixes the floor — one household's weekly wage at or above 10⁴ minor units. The ceiling comes from §6.11:
the balance-tick integral accumulates `balance × ticks` over 1,560 ticks, so with `i64` a sustained
single-holder balance is bounded, and the bound must be computed, registered as a two-sided bracket, and
watched by a declared high-water series. Under the previous edition's `2^53` the headroom was roughly
29× the opening money stock, which inside-money creation could plausibly exceed.

### 5.4 No stored value; one price per thing

**No column records what a holding is worth.** `price.level` and `price.stamp` are posted columns
in the instrument's own row, written by clearing at positions 10 and 13, partitioned by venue into
disjoint row ranges so each range has one writer.

Every read names which price it wants **through the type system**, so a stale mark is a different
type rather than a different value. The stamp yields three answers: never cleared, cleared this
tick, cleared *n* ticks ago.

**An unpriced instrument reads `NotPriced`, never zero, and multiplying it is a compile error.**
Zero multiplies; `NotPriced` does not. Every asset class §6.3 conserves has a line, so no holding
is valued at zero for want of a price — and where something has genuinely never cleared,
`NotPriced` is a modelled outcome every consumer must state what it does with.

### 5.5 The row lifecycle

**An identifier is permanent; a row's residency is not.** The hot store holds only resident rows at
a fixed capacity allocated once at init, and the occupied prefix is contiguous, always. A directory
maps identifier → slot; **slots never cross a module boundary and have no integer-yielding
method**, so no index may be built over them and no relation column may cache one.

**Retirement is gated on quiescence**: a terminal status *and* a zero hold count. The predicate is
evaluated where the relation changed, never by a sweep — **no pass over the world may exist to find
retirable rows.** When an operation drives a terminal row's hold count to zero the ledger pushes
its identifier onto a fixed 65,536-entry queue, which `lifecycle` drains at position 21; overflow
is a defect.

A slot is recovered by relocating the last live row into it. **Relocation is not an operation**: it
changes no quantity, appends no journal row, and happens inside the ledger under its own writable
view.

Each retirable table carries a **256-bit retirement accumulator**, folded in retirement order over
(identifier, terminal status, tick), so what a run destroyed is bound into the digest and a
retirement cannot be lost by being invisible.

**Counter-account rows are never retired.** They have a permanent status and are never quiescent.

---

### 5.6 Holdings are blocks

A holder's rows are a **contiguous block**, holder-major and sorted by asset. A balance-sheet read,
a wind-up walk and settlement's buyer sweep are each one contiguous run over that holder's own
range — which is what makes a constraint expression cost what the agent holds rather than what the
world holds.

Capacities are in §3.4 and are **declared, not grown**. A block with no free slot raises.

A zero-quantity entry is removed by the `move` that zeroes it, by shifting the tail left; there is
no sweep and no deferred cleanup. **An issuer's negative entry is not a zero and is not removed.**

## 6. The ledger

### 6.1 Three doors, nine operations

**A door is distinguished by what it can address, not by which layer calls it.** A handle for one
door has no method belonging to another, and no widening function exists anywhere.

| Door | Addresses | Operations |
|---|---|---|
| **ledger** | the conserved quantity column | `move`, `exchange`, `pledge`, `release`, `amend`, `retire`, `register`, `registerEntity`, `post`¹ |
| **state** | posted columns | `post` |
| **relation** | liens and other relations | `pledge`, `release` |

¹ `post` is listed for completeness; it addresses only `Posted` columns and **cannot address a
conserved one — the mint is typed and no such overload exists**.

- **`exchange` is indivisible at the type level.** Its two legs are not separately callable, and it
  writes one journal row carrying both assets, both quantities, the cleared rate and the realised
  rate.
- **`move` remains one-sided**, for flows that genuinely are: taxes, transfers, dividends,
  endowment, consumption, an estate distribution.
- **A tenth operation is an ADR**, plus a row in the change-cost table, a case in the conformance
  suite, and a restatement of §6.1a.

### 6.1a What A1 actually claims

A1 is a claim about the conserved column, discharged in four parts:

1. Every mutation of a conserved quantity goes through the ledger door.
2. **The conserved column is private to the `ledger` crate.** No mutable borrow of it is obtainable
   outside, so the set of code that can write it is one crate whose public surface is the nine
   operations. Inside the crate the quantity path is one function, and `exchange` is two calls to it in
   one statement.
3. Every `Posted` column has exactly one writing system, named in the registry, checked at build.
4. **No `post` handle is mintable for a `Conserved` column** — a compile error, not a check.

**Part 2 was previously stated as "exactly one writer in the whole source tree, checked in CI", and that
was false** (D2). Relocation copies the quantity column, the zeroed-entry tail shift moves it, and slot
canonicalisation clears it — three writers, all necessary, all inside the ledger. The CI check would have
been unimplementable without exemptions, which is how a check becomes a decoration. **The crate boundary
is the honest form of the claim and is the stronger one**, because it is enforced by the compiler and
covers writers nobody has thought of yet.

### 6.2 What may be created and destroyed

A real thing enters or leaves the world through a **counter-account**: an ordinary holder in the
same table as everybody else, whose balance is the negation of what it has sourced. Four families,
four owners, and **ten permitted (family, unit class) pairs per region — 40 rows in the world**.

| Family | Direction | Unit classes it may touch | Owning system |
|---|---|---|---|
| `Endowment:<asset>` | source | labour hours, land | demography, and the obligation-payment walk for labour |
| `Production:<good>` | source | goods, capital units, dwellings | production |
| `Consumption:<good>` | sink | goods, labour hours | consumption |
| `Wear:<class>` | sink | capital units, dwellings, goods | capital |

Four properties, each mechanical:

1. **Real only.** A counter-account may hold only units with no issuer. A financial claim cannot be
   created by one, because a claim exists only as its issuer's negative balance (R-1).
2. **One owner.** Four families, four systems. Every write carries a minted capability naming the
   family, and no system holds a general one.
3. **Monotone.** A source family's balance is non-increasing and a sink's non-decreasing. A move
   the wrong way raises at the door.
4. **Scoped.** `Production:steel` cannot pay out wheat. The permitted pairs are the whole list.

**Land is endowed and never produced, consumed or worn** — floor area does not depreciate and
nothing manufactures it, so `Production:`, `Consumption:` and `Wear:` carry no floor-area row.
**Dwellings and capital are produced and worn but never endowed**, so the opening stocks §13.1
seeds enter through `Production:` at position 6 on the opening period, written by the system that
owns that row. **Goods are produced, consumed and worn**, the last being spoilage.

*Accepted cost.* The opening tick shows a production flow no producer decided on, and §14 family
1's series for it spikes at period 1. The alternative — widening `Endowment:` to capital — would
be changing a conservation law to avoid a scheduling inconvenience.

**A fifth family is a new conservation law**: one registry row with exactly one owner, one minted
capability, and no new operation.

---

### 6.3 Integer quantities and the three rounding rules

Conserved quantities are **`i64` integers in the asset's smallest meaningful unit** (D2), and an
overflow panics rather than rounding. With floating-point quantities a debit followed by a credit is not
exactly conservative, addition is not associative, and the total depends on summation order —
conservation degrades from a theorem into a measurement with a tolerance, and a tolerance is a judgement
that needs something to enforce it.

**Every `price × quantity` is quantized at the point of write. Discarding a remainder is
forbidden.** There are exactly three cases and **no fourth**:

| | Case | Rule |
|---|---|---|
| 1 | a single move | quantize once, round-half-to-even, debit and credit the same integer. No residue exists, because one number is written twice |
| 2 | a distribution across recipients | quantize the total first, then allocate cumulative-proportionally in **ascending holder identifier**: `Cᵢ = Σⱼ≤ᵢ qⱼ`, `aᵢ = round(N·Cᵢ/C) − round(N·Cᵢ₋₁/C)`. Shard-invariant, which largest-remainder is not |
| 3 | an exchange | designate the primary leg — the non-currency asset, or the lower asset identifier where both are currency — quantize it once, derive the secondary from it. No residue on either leg |

A rounding decision anywhere outside these three is a bug.

### 6.4 Preconditions at the door

An operation raises — never returns a code, never silently no-ops — on any of:

- a non-positive or non-finite quantity; a move from a party to itself;
- a move of an asset the party's class may not hold (Q10); a move of **encumbered** units;
- a move that would take a holder negative in an asset it may not be negative in. The test is total
  over the holder discriminant: `mayGoNegative` is a function of the row, not a list;
- a move **into** a source counter-account or **out of** a sink; a move naming a counter-account
  whose declared asset is not the asset moved;
- a debit from a holder that is `in-resolution`, unless the handle carries `EstateAgency`; a debit
  from or credit to a `resolved` holder, which no authority lifts;
- a move of any asset whose **issuer** is in a resolution state — the issuer freeze, one extra load
  per move and no rows at all;
- a `pledge` beyond depth 3, re-pledging more than the parent's remaining rehypothecable quantity,
  naming an ancestor as beneficiary, or naming a beneficiary who may not hold the asset;
- a `release` of a lien with an open child;
- a holdings block with no free slot;
- a `post` to a column the caller does not own, or a second `post` to the same (column, row) in one
  tick;
- an `amend` whose delta takes effect before the current tick;
- a `retire` transition the caller does not hold, or a final transition on a row that is not
  quiescent.

### 6.5 Encumbrance is a relation

A lien is a row: pledgor, beneficiary, asset, quantity, parent. **Encumbered units cannot move**,
refused at the door rather than detected afterwards.

- **Depth is capped at 3.** A re-pledge charges nothing new — it re-lends a claim the re-pledgor was
  given — and its pledgor is the parent's pledgor, to the root.
- **A cycle is unwritable**: naming an ancestor as beneficiary raises.
- **Over-pledging has no path that produces it**, rather than being detected.
- **Chains unwind leaf-first.** Releasing a lien with an open child raises.
- **A lien's beneficiary is always an entity.** Where the economic beneficiary is a class of holders
  — a bond issue's holders, a cover pool — a **trustee entity** is registered to hold for them. A
  conversion from an instrument identifier would put a second identity space into the `holder`
  column.

### 6.6 The journal is a byproduct

Every operation appends **exactly one row**: who, to whom, what, how much, at what price, under
what reason code, and `actor` — the system, taken from the minted handle, not from an argument.

**The journal is not the authority and state is not rebuilt by replaying it.** Holdings stay
authoritative and the journal derived, so the two are independently produced and their agreement
has content.

Retention is **two ticks**, in a fixed ring of 7,200,000 rows in two segments, 48 bytes each,
345.6 MB. **Exhausting it is a defect that raises, never a wrap that loses history**, and its
high-water mark is a declared series. If the ring must shrink, the lever is the journal's **grain**
— a declared coarsening — never its retention, because retention below two ticks breaks §9.3's
prior-close reads.

### 6.7 Wind-up

**Insolvency is a modelled outcome, not a defect.** The test is declared and fires at position 18.

Status governs what may happen to an entity, and it is a lattice:

| Status | May receive | May initiate | May be debited |
|---|---|---|---|
| `live` | yes | yes | by any system, under the ordinary doors |
| `in-resolution` | yes | **no** | only through a handle carrying `EstateAgency` |
| `resolved` | **no** | no | **never** — no authority lifts it |

**Resolution runs over several ticks, and that is the model rather than a concession.** One row per
entity in the resolution register: entity, state, tick entered, realisation cursor, frozen recovery
numerators.

| State | Entered | Duration | What happens |
|---|---|---|---|
| `open` | the tick default testing fires | 1 tick | status set to `in-resolution`; the issuer freeze takes effect; the escrow entity is registered; steps 1–3 run |
| `realising` | the next tick | up to 13 ticks | step 4: tranches escrowed, cleared, proceeds swept to the estate |
| `distributing` | when the estate is cash, or the window closes | ⌈claimants ÷ 125,000⌉ ticks | recovery numerators frozen; steps 5–7 run, sharded |
| `resolved` | when the entity is quiescent | — | `retire` |

The `resolving` cohort is standing, re-visited every tick at position 18.

**The estate, in order:**

1. **Secured claims resolve against their collateral, off-queue.** Surplus collateral returns to
   the estate; a shortfall joins the queue at the claim's declared unsecured rank, or is
   extinguished where the claim is non-recourse.
2. **Chains unwind from the leaf inward**, and cannot fail to, because the release door refuses a
   lien with an open child.
3. **Schedules are truncated and claims frozen as integers** — principal plus accrued interest to
   the truncation tick.
4. **The estate is realised, not distributed in kind.** Each tick of the window the remaining
   non-cash assets are escrowed, cleared on their venues, and the proceeds swept to the estate.
5. **The ranked queue is paid in cash only.** `Notes:<cb>` answers Q10 with `any`, in every region,
   so no payment can be refused for holder eligibility.
6. **Unpayable claims are extinguished explicitly**, by a move returning the remaining units to
   their issuer. Nothing is written off by omission.
7. **Residual holders take what remains**, normally nothing — and **the move still happens**, so
   that "the equity holder received nothing" is a fact in the journal rather than an absence.

**No asset is left dangling.** Anything the venues will not take goes to the region's government,
**which pays the last mark** — a named modelled holder, not a write-off.

**No instrument-type branch appears anywhere in wind-up.** The waterfall matches on §7.2 Q11's rank
and on the lien rows, both of which are facts.

An entity reaches `resolved` holding **exactly zero of every claim it issued**, which *proves* the
estate distributed exactly rather than asserting it.

### 6.8 Closure by construction

**R-1: a financial claim exists only as its issuer's negative balance.** It comes into existence by
a `move` out of its issuer and leaves by a `move` back. There is no `Issue:` row for it anywhere,
and no counter-account can mint one.

Three consequences, all mechanical:

- **The outstanding amount of a claim is one indexed load**: `outstandingOf(a) = −q(issuer, a)`,
  read off the issuer's own row rather than summed over holders.
- **A claim asset's total across all holders, the issuer included, is exactly zero**, at every
  instant, because issuance moved it out of a row that went equally negative.
- **A bank whose deposits were a counter-account's balance would hold assets and owe nothing** —
  never insolvent, never capital-constrained, incapable of suffering a run. R-1 is what makes bank
  failure expressible at all.

**A claim-asset aggregate is computed asset-outer, holder-inner**: `Σ_a price(a) × (Σ_h q(h,a))`,
with the inner sum in integers. Written holder-outer it is several million floating-point products
whose total depends on summation order, and A1's exactness would be lost — a tolerance, then a
judgement, then something to enforce the judgement.

### 6.9 `post`, and the declared-column registry

A posted column is a fact a system computes and another system reads: a price, a policy rate, a
plan, a submission, an expectation, an index. Six families, each with **one owning system named in
the registry**, each written at a declared position.

- **One `post` per row per tick.** A second raises, so read-modify-write over a posted column is
  unwritable.
- **Every posted column carries a tick stamp.** A reader asking for `thisTick` gets a value only if
  the stamp matches; otherwise it gets the prior tick's, and there is no method that yields this
  tick's value to a reader not entitled to it.
- **No posted column may take a birth value that is a level** in currency or index units unless it
  is derived from primitives.

### 6.10 Status is a lattice; `retire` is monotone

`retire(row, transition, reason)` advances a row one step along its table's declared monotone
terminal path and **has no inverse**. The last step of every path is the end of residency.

### 6.11 Two history columns, maintained by `move`

`move` updates both accrual columns on both legs **before** writing the quantity:

```
integral += balance_before × (t − integralUpdatedAtTick);  integralUpdatedAtTick = t
```

O(1), exact in `i64` over a 1,560-tick run within the bound §5.3 owes, and **proportional to activity,
not to stock** — a dormant deposit costs nothing until it moves or is paid. **No system holds a handle to
either column**, so no system can manufacture interest by setting one.


---

### 6.12 When the issuer of a fungible claim fails

A deposit line held by 300,000 households cannot be resolved in one tick. The distribution is
**sharded**, and two properties make that safe:

- **The recovery numerators are frozen** when `distributing` is entered, so a holder paid in shard 1
  and a holder paid in shard 4 are paid at the same rate.
- **The frozen claim cannot move.** A `move` of it by any holder raises at the door, so the holder
  set cannot change under the walk.

**Units held plus units extinguished equals the issue, at the end of every tick** of the
resolution — not only at the end. That is an A4 assertion, and A1 would have passed without it.

---

## 7. Instruments as data

An agent model normally has to change when an instrument is added, because the instrument's
behaviour is written inside the agents that hold it. Here an instrument is **data**: a row of facts
that every generic mechanism reads. **A branch on instrument type outside the two facts tables is a
bug report about the tables.**

### 7.2 Intrinsic facts: thirteen questions, two levels

**Arity at the type; identity at the instance.** The type answers whether an instrument *has* an
issuer and whether it is dated; the instance row answers *which* entity and *when*. Adding a type
fails to compile until all thirteen are given — there is no default, because a default is a
fourteenth decision made by whoever wrote it.

| # | Question | Answers |
|---|---|---|
| 1 | Unit of measure | one of the nine unit classes |
| 2 | Minimum piece | `SingleUnit` \| `DeclaredPiece` |
| 3 | Denomination | `InCurrency` \| `NotDenominated` |
| 4 | Issuer role | `LiabilityOf` \| `ResidualClaimOn` \| `NoIssuer` |
| 5 | Quoted as | price \| yield \| spread \| rate |
| 6 | Tenor | `Dated` \| `Perpetual` \| `Demand` |
| 7 | Accrual basis | `NoAccrual` \| `PerTickSimple` \| `PerTickCompounded` |
| 8 | Optionality | a set drawn from the seven families of §7.6 |
| 9 | Obligation carrier | `NoCarrier` \| `PerUnit` \| `PerContract` |
| 10 | Eligible holders | any \| institutional \| issuer-restricted \| sovereign |
| 11 | Claim rank | `securedOnPool` boolean × `preferred \| senior \| subordinated \| residual`, plus a shortfall rank where secured |
| 12 | Liquidity tier | tier 1 \| tier 2 \| illiquid |
| 13 | Pricing mode | `VenueCleared` \| `DerivedMark` \| `UnitOfAccount` |

**`Demand` has content that `undated` did not**: the holder may extinguish the claim on any tick,
which is what makes a bank run expressible at weekly ticks.

**Security is a relation, not a rank.** A secured claim's collateral is lien rows whose beneficiary
holds for that instrument (§6.5), so Q11 carries a boolean and an ordinal rather than an ordinal
doing both jobs. Dual recourse is the ordinary answer `{ true, senior }` and the wind-up waterfall
gains no case.

### 7.3 Relational facts: keyed by regime

What risk weight a bond carries, how it is accounted for, and whether it is collateral-eligible and
at what haircut depend on **who holds it and under which regime**. Three questions × **seven
regimes** — `bank-prudential`, `liability-matched`, `fund-unconstrained`, `corporate`, `household`,
`sovereign-fiscal`, `central-bank` — = **21 answers per instrument type**, a total mapping.

An agent calls `riskWeight(instrument, regimeOf(self))` and `haircut(instrument, regimeOf(venue))`.
It never branches on instrument type and never on a regime other than its own. Regimes are named
for the binding, never for the institution: naming one after an institution invites a second regime
for the second institution.

Guarded at build by a declared count, a column-distinctness check and a two-cell separation check.
**A regime exists because a class of agent is bound differently; if two regimes never differ on any
answer they are one regime.**

The cost structure this buys: **a new instrument is cheap** (one intrinsic row plus one relational
row per regime); **a new regime is bounded** (one column across types); **a new question is
expensive and visible** (it breaks every existing type until each is decided).

*Owed.* §7.3 gives the shape and none of the values. They are `assumed` primitives and they are
counted like everything else — written flat they are roughly twenty entries, a quarter of §16.1's
whole budget for one table, so the table is **a base weight per instrument type times a per-regime
severity**: about nine entries for twenty-one answers, and it says something the flat table does
not, that regimes differ in how strictly they read the same instrument.

### 7.4 Schedules

An obligation's cash flows are a **closed type with two constructors** and one interpreter:

| Constructor | Content |
|---|---|
| `Regular` | parameters: principal, rate per period, term, start tick, frequency, and the level payment quantized once at origination |
| `Explicit` | a set of (tick, leg, amount) rows |

**No consumer may branch on which constructor produced a schedule.** One function, `due(schedule,
tick)`, answers what falls due; a caller that needed to know the shape is a missing case in it.
A `Regular` level annuity and an `Explicit` schedule materialised from the same parameters produce
identical due sets at every tick, to the last minor unit.

**`amend` only ever appends.** Base parameters are never edited and no row is deleted; an amendment
is a **delta** on a chain, and the four constructors are `Truncate`, `Defer`, `Reparameterise` and
`Insert`. `Truncate` is absorbing: the earliest truncation wins and no later delta can raise the
horizon.

**Eight mechanisms, five owners, and no system holds a general amendment handle:**

| Mechanism | Delta | Owner |
|---|---|---|
| default truncation | `Truncate` | credit |
| prepayment | `Truncate` if full, `Reparameterise` if partial | credit |
| payment holiday | `Defer` | credit |
| restructure | `Reparameterise` | credit |
| call and early redemption | `Truncate` then `Insert` | primary issuance |
| cancellation on wind-up | `Truncate` | wind-up |
| claim crystallisation | `Insert` | insurance |
| employment termination | `Truncate` | labour |

**Coupon resets and wage revisions are not amendments.** They are index reads at a review tick, and
they write nothing.

**The due-tick index** answers "which instruments pay this tick" in time proportional to what fires,
not to what exists, and is re-bucketed **inside `amend` and inside payment** — an instrument that
paid and was not re-bucketed pays again next tick; one re-bucketed without paying is silently
forgiven. **An obligation that has made its last payment has no next due tick, and that transition
is where its hold count reaches zero** (§5.5).

### 7.5 The instruments table

Eleven columns, written **only by `register`**, so an instrument row is an immutable declaration
except for `status`, which moves only under `retire`.

`type`, `issuer`, `currency`, `venue`, `maturityPeriod`, `minPieceUnits`, `scheduleFirst`,
`optionsFirst`, `issuePeriod`, `status`, `holdCount` — each required exactly when the type's
answers say so, and **a column not so required must be absent**: a perpetual carrying a maturity
period raises.

`register` also raises on: an `IssuerRole` × `UnitClass` pair A4 forbids; an `issuer` naming no
existing entity row (a *dead* issuer is legal — its instruments survive it into the waterfall — a
non-existent one is not); a piece below 1 or not a safe integer; a venue that does not clear the
type's quote basis; a schedule whose bucket family disagrees with the carrier.

**A new column must be either an instance answer to one of the thirteen questions or a provenance
field named in this table. Anything else is a fourteenth question and an ADR.**

*Unresolved.* This table and §3.4.4 describe two different rows — 44 bytes with a schedule
directory, against 148 bytes with the schedule and two price epochs inline, which removes the
schedule identity space and a directory of 8.1 M entries. They have different memory budgets,
different save formats and different relocation costs. **It is settled by measurement on the target
device, and it is an entry criterion for Phase 2.** Until then this section's family counts are not
to be used for sizing.

### 7.6 Option terms: seven typed tables, never a bag

An instrument's optionality is a set at the type level and a chain of rows at the instance level:
`optionsFirst` heads a 16-byte `instrument_options` chain, and each row points into **one of seven
typed terms tables** — callable, puttable, prepayable, convertible, coverPooled, contingent,
extendible.

**Not a JSON blob, not a key-value side table, not `params: number[]`.** An untyped bag declares a
richer world than any code reads, nothing enumerates what is owed, and the first misspelled key
returns a plausible number.

**Every family names the index its event test fires from**, because an event test that scans all
live instruments each tick is an N3 violation. An `instrument_options` row naming a family the type
does not declare raises at `register`.


---


---

### 7.7 Two carriers

**What distinguishes them is the arity of the obligation, not the kind of the asset.**

| Carrier | Obligor | Obligee | Schedule | Paid at |
|---|---|---|---|---|
| `NoCarrier` | — | — | none | — |
| `PerUnit` | the issuer, once | every holder, in proportion to units held | one, for the whole line | position 3 |
| `PerContract` | the issuer (the borrower) | the single holder named on the row | one, individually parameterised | position 4 |

**An amendment to a `PerUnit` schedule reaches every holder at once**, which is correct for a bond
and wrong for a loan. That is why there are two carriers rather than one.

The carrier is consumed in exactly two places — the due-tick index's two bucket families, and the
accrual amount rule — and **no agent reads it**. Whether a coupon arrived by distribution or by
direct payment is invisible to the holder.

*Accepted cost.* Price discrimination across individual depositors within one product is not
representable. That is the direct price of one deposit line per issuer rather than one per holder,
which is a ~200× memory saving at 500,000 households.


---


---

## 8. Agents

### 8.1 An agent is five declarations

What distinguishes a bank from a fund from a household is not plumbing. It is five things, and
**they are a total mapping per agent class: a class declaring four does not compile.**

| | Declaration | Content |
|---|---|---|
| 1 | **Mandate** | which assets it may hold, and in what proportions |
| 2 | **Regime** | which regulatory and accounting regime its relational facts are read under (§7.3) |
| 3 | **Constraints** | the inequalities that bind it, over *facts* and holdings — never over instrument types |
| 4 | **Valuation** | how it forms a reservation level for something it might buy or sell |
| 5 | **Funding policy** | what it does with a surplus or a deficit, its target currency composition (§9.2), and the window within which a breach must be cured before insolvency (§6.7) |

Everything else — settling, holding, collecting, marking, reporting, being wound up — is generic
machinery the agent does not own and does not know about.

**A declaration that is genuinely empty at a phase is declared absent with its reason**, which is a
value the build check reads, not a gap it cannot see. A bank has no valuation before it has a
funding cost, and saying so is a statement; leaving the field out is not.

**Constraint expressions read signed balances.** A claim exists only as its issuer's negative
balance (R-1), so an agent's liabilities are rows on its own block next to its assets, and a
capital ratio is an expression over one contiguous run with no second table and no attribution
step. **The numerator of a capital constraint is regulatory capital, not net worth**: under R-1 an
institution capitalised by issuing equity holds `+X` and `−X`, so its net worth is identically zero
and no positive floor would ever be satisfiable. Regulatory capital is net worth with residual
claims weighted out, and the weight reads §7.2 Q11's rank — a fact, so §8.1's own rule holds.

*Accepted cost.* The weight applies on both sides, so an institution holding another's shares takes
no capital credit for them. That is conservative in the direction real regimes are, and it is
stated once rather than as a deduction schedule §7.3 does not carry.

### 8.2 An agent holds rows, never fields

An agent has no per-class holdings field. Its balance sheet is a query over its own block:

```
totalAssets(a)      = Σ over holdings of a where q > 0 :  q × price(asset)
totalLiabilities(a) = Σ over holdings of a where q < 0 : −q × price(asset)
netWorth(a)         = Σ over holdings of a            :  q × price(asset)
```

One walk, three readings. **No stored aggregates on an agent. Ever.** The moment one caches a
per-class total as a field, the N×M matrix returns through the back door and returns silently.

*Owed.* §8.2 previously named "one permitted accumulator" maintained inside `move`. It cannot have
one writer: the totals are price-weighted, so a mark at position 17 changes them with no `move`
happening. The query is exact and costs one contiguous walk; the accumulator waits on a
price-change rule that does not yet exist.

### 8.3 Adding an agent type

One module declaring the five items above, plus one row in §8.4's mapping. It participates in every
existing line on the day it is written, because participation is expressed through the interfaces
every other agent already uses. **Zero edits to any instrument, any venue, or any other agent.**

### 8.4 The inventory

`AgentClass` is a closed union and `classFacts: AgentClass → (regime, cadence, count)` is a total
mapping. A class naming no regime does not compile. **Counts are in §13.3; nothing is repeated
here.**

| Class | World | Regime | Re-plan cadence | Block |
|---|---|---|---|---|
| Household | 500,000 | `household` | 13 ticks, staggered | 160 B |
| SME | 50,000 | `corporate` | 4 ticks, staggered | 256 B |
| Large firm | 440 | `corporate` | 1 tick | 1,024 B |
| Bank | 56 | `bank-prudential` | 1 tick | 1,024 B |
| Fund | 54 | `fund-unconstrained` | 1 tick | 1,024 B |
| Insurer | 22 | `liability-matched` | 1 tick | 1,024 B |
| Pension fund | 18 | `liability-matched` | 1 tick | 1,024 B |
| Government | 4 | `sovereign-fiscal` | declared meeting cadence | 1,024 B |
| Central bank | 4 | `central-bank` | declared meeting cadence | 1,024 B |
| | **550,598** | | | **95.6 MB** |

Plus **40 counter-account rows**, 10 per region, which are entities but decide nothing and are
never retired: 550,638 rows live at tick 0.

Three things this settles: **class is not regime** (nine classes, seven regimes); **class is not
cadence** (cadence is a cost decision and lives here, so changing it is a one-row diff); **class is
not a block** (block is a schema fact with a hard cap asserted at schema build).

**Representative agents are abolished.** A row is one household. There is no scaling factor between
a model household and anything outside the model, because there is nothing outside the model.

*Accepted cost.* Industrial structure is thin — the smallest region has 64 large firms across the
sub-units it must supply — so concentration is a property of the opening rather than an outcome of
entry, and entry dynamics among large firms are effectively absent at this scale.

*Owed.* One seed primitive covers insurers and pension funds together and there is no rule splitting
40 into 22 and 18. It must be settled before either class is written.

### 8.5 What a row is

A household is **160 bytes**; an SME is **256**; an institution is **1,024**. The caps are asserted
at schema build. **No per-good, per-venue or aggregate column on any agent** — those are the
columns that turn a row into an N×M matrix.

Traits are **drawn, not stored**: a pure function of (world seed, stream, identifier), so they cost
no bytes, survive a save without being saved, and are identical whichever order the population is
walked in. The sampler is uniform because §11 admits no `log`, `exp` or `cos` — those are not
bit-identical across platforms and the golden digest must reproduce on the target device.

*Accepted cost.* Traits are uniformly spread, so a fat-tailed wage distribution cannot come from
traits alone. It has to come from the dynamics, which is where it belongs and is now a testable
claim rather than an assumption inside the draw.

### 8.6 Staggering

Cost proportional to what changes, not to what exists.

**The phase is drawn, never derived from the identifier arithmetically:**

```
phase(a)          = draw(STREAM_REPLAN_PHASE, id(a)) mod cadence(classOf(a))
nextReplanTick(a) = phase(a)
```

**Never `id mod cadence` and never a function of the slot.** A stream draw is uncorrelated with
every other identifier-derived property, so re-planning cannot align with region blocks, cohorts or
creation bursts; an aligned phase produces a real 13-tick cycle in aggregate demand that is an
artefact of the schedule. And because the phase is a function of the identifier, relocation cannot
change it.

*This prohibition is about agents.* §13.4's policy meeting phase **is** `regionIndex mod cadence`,
deliberately: a committee's meeting date is supposed to be a function of the region.

**The pass finds them through the re-plan bucket index** — identifiers grouped by next re-plan tick,
ascending within a bucket, storing identifiers only (550,598 × 4 B = 2.2 MB) and resolved through
the directory. The pass visits bucket `t mod C` and nothing else. There is no scan.

**Every cohort re-plans at tick 0**, so no agent ever lacks a standing plan.

**Off-cadence re-planning is a closed list of six**, each naming the index it fires from and the
system that owns it, each written through **its own capability-minted trigger handle**. There is no
general re-scheduling handle anywhere, and a seventh trigger is an ADR.

| Event | Index it fires from | Owner |
|---|---|---|
| Employment begins or ends | that tick's labour-line match list | labour |
| A loan defaults or enters arrears | the default list | credit |
| An issuer it holds is wound up | the estate's claimant list | wind-up |
| It buys or sells a dwelling | the dwelling-line fill list | housing |
| Household formation or dissolution | the birth/death list | demography |
| Free deposit balance below `bufferTargetMinor` | settlement's watch list | settlement |

**A trigger whose index would be a scan cannot be given a handle**, because the manifest that would
declare it fails the N3 check.

Consumption, wages and debt service fire **every tick** from the standing plan. That is what keeps
A4 intact under staggering: a household that re-plans quarterly still pays its mortgage weekly.


---

## 9. Markets

### 9.1 One clearing interface

**Every line clears the same way**: each participant posts a **reservation level and a size it
scales into**, and the solver solves for the level at which demand meets available supply.

**A venue is an adapter; a line is data.** The solver runs once per line and depends only on
`domain` and `kernel`, so it is testable and optimisable in isolation.

**Termination is provable from the inputs, not capped.** The grid is **64 log-spaced buckets
spanning half to double the line's prior close** — 2.2% wide, so a ceiling is placed within 1.1% of
itself. If the crossing is not interior, the grid is **rebased once** on the failing side and the
accumulation repeats. If it is still not interior, the line moved by more than a factor of four in
one tick and **that is a defect**, because a grid anchored on the prior close cannot be silently
wrong about the current one.

**Two submission shapes, one interface:**

| Shape | Storage | Cost | Used by |
|---|---|---|---|
| `schedule` | a sorted row per participant | O(n log n) | firms, banks, funds, insurers, governments, central banks |
| `priceTaking` | a quantity accumulated into one of the 64 buckets | O(1), no sort | households, and any participant whose plan is standing |

**Rationing inside the marginal bucket needs no sort**: fills proceed in ascending holder identifier
from a start offset of `tick mod n`, so the advantage of a low identifier washes out over ticks
rather than accumulating for thirty years.

**Settlement is a two-pointer walk** down sellers and buyers, both in ascending identifier order:
O(buyers + sellers), no matrix. A price-taking buyer is filled by **at most one seller**; where the
walk would split it, the rest is unfilled — a modelled outcome, reported per line. The cleared
quantity is allocated across each side by rule two **before** the walk begins, so the two sides are
equal integers by construction.

**An issuer's negative balance is not supply.** A claim is issued only at position 19.

### 9.2 Budget allocation, in two stages

An agent's spending is pre-allocated **per line**, in two stages, because with four currencies a
budget is not well-posed until the means of payment exists.

| Stage | Positions | Lines | What it does |
|---|---|---|---|
| 1 — funding | 9, 10, 11 | the 6 FX and the 4 money-market lines | allocate the home-currency balance from a prior-close target currency composition; clear; settle, so the currency exists before it is spent |
| 2 — spending | 12, 13, 14 | every remaining line | per-currency, per-line budgets against balances that are now facts; clear; settle |

Unspent allocations are returned at 15, per currency.

*Accepted cost.* **Intra-stage reallocation is given up deliberately.** An agent that finds a line
cheap cannot move budget into it from a line it has already lost, within the tick. The alternative
is a joint solve across lines, which makes the whole market block one non-decomposable object and
destroys the independent line evaluability the shard unit depends on.

### 9.3 Simultaneity

**The rule is per fact, not per position.** There is no single place in the list where prior-close
becomes this-close, because the market block is two stages and `Spending allocation` is a decision
position that deliberately follows `Funding settlement`.

- A **decision** reads prior-close prices. It cannot see a price formed in its own tick, and the
  tick stamp on every posted cell is what enforces that.
- **Clearing** reads this tick's submissions.
- **Settlement onward** reads this close.
- **The one crossing, named:** stage-2 allocation and clearing read the FX and money-market rates
  formed at position 10, because that is the whole point of settling funding first.

**No system reads a price formed after it.** What is checkable structurally is the stage: each
market stage runs decision → clearing → settlement, in that order.

### 9.4 The committed period order

A hand-written, reviewed, version-controlled list. **Positions are stable names; a system is
inserted at a named position, never appended.** Deriving the order topologically is refused:
nearly every system touches cash so the graph is near-complete; a topological order is not unique,
so adding one system can silently reorder two others and force a digest re-baseline; and "this must
run inside the settlement window" is a fact about the model that should be written down.

**Phase** is the §9.3 family and is what the simultaneity check reads. **Cadence** covers everything
slower than weekly: a system does not run on a tick it has no content for. **Parallel** names the
shard unit where the position is parallelisable, and `—` where it is not.

| # | Position | Phase | Cadence | Reads | Parallel | Owner |
|---|---|---|---|---|---|---|
| 1 | **Demography** — entity birth and solvent exit | endowment | every tick | primitives; its own stream; no marks | — | demography |
| 2 | **Endowment** — land from `Endowment:`. Labour hours are delivered at position 4, not here; goods, capital and dwellings are not endowable at all (§6.2) | endowment | every tick | schedules; no marks | — | endowment |
| 3 | **Accrual and distribution** — every `PerUnit` claim due: deposit interest, coupons, fund distributions, declared dividends | endowment | every tick; the due bucket only | ledger-live balances and the balance-tick integral; `due` | — | accrual |
| 4 | **Obligation payment** — the `PerContract` walk, standing bucket first: wages, rents, amortisation | endowment | every tick | schedules; `plans` prior-run; ledger-live | — | payment |
| 5 | **Depreciation** — into `Wear:` | endowment | every tick | schedules; ledger-live | — | depreciation |
| 6 | **Production** — inputs and hours consumed into `Consumption:`, output issued from `Production:` | endowment | every tick | `plans` prior-run; ledger-live | — | production |
| 7 | **Policy** — the central bank and fiscal rules | decision | monetary 13, fiscal 52, phase `regionIndex mod cadence`; skipped when no region meets | prior-run-close marks | — | monetary policy, fiscal policy |
| 8 | **Valuation and constraints** — the tick's re-planning cohort; all institutions every tick | decision | every tick | prior-run-close marks; `policy.rate` prior-run; own `plans` | row span | valuation |
| 9 | **Funding allocation** — target currency composition; allocation across **the 6 FX and 4 money-market lines, and no others** | decision | every tick | prior-run-close marks; `plans`; ledger-live | row span | funding |
| 10 | **Funding clearing** — those ten lines | clearing | every tick | this tick's `intents` | line | clearing |
| 11 | **Funding settlement** — stage-1 legs move; currency balances become facts | settlement | every tick | this-close fills | — | settlement |
| 12 | **Spending allocation** — per-currency, per-line budgets against known balances | decision | every tick | prior-close marks; **this-run FX and money-market rates from 10**; balances settled at 11 | row span | funding |
| 13 | **Spending clearing** — every remaining line, including all credit lines | clearing | every tick; 28 lines on a 4-tick cadence | this tick's `intents`; this-run FX rate | line | clearing |
| 14 | **Spending settlement** — cash and asset legs move; consumption goods move to `Consumption:` | settlement | every tick | this-close fills | — | settlement |
| 15 | **Budget reconciliation** — unspent allocations returned, per currency | post-clearing | every tick | this-close | — | reconciliation |
| 16 | **Margin and collateral** — pledge, release, substitution | post-clearing | every tick | this-close prices from 10 and 13 | — | collateral |
| 17 | **Mark to market** — position and portfolio marks, NAV | post-clearing | every tick | this-close prices; ledger-live | — | marking |
| 18 | **Default testing and estate resolution** | post-clearing | every tick | this-close valuations from 17; the `resolving` cohort | — | default testing, wind-up |
| 19 | **Primary issuance** — new instruments, calls, redemptions, conversions. **The only door for issuing a claim** | post-clearing | every tick | this-close | — | primary issuance |
| 20 | **Projection** — observation series written; the flow-of-funds matrix read | projection | every tick | everything; writes only observations | — | projection |
| 21 | **Obligation compaction** — closed schedules and their deltas released from the arena | maintenance | every 52nd tick | `closedAtTick` stamps; writes nothing any system reads | — | schedule maintenance |

**Retirement is not a position.** It is issued at 1, at 18 and at 19, and drained at 21. A system
that drains the queue must not take its name from the position it sits in.

**There is no Consumption position.** A household moving its own goods into a counter-account is a
flow whose source arrived from nowhere. Consumption is a purchase from a named seller at a formed
price, settled at 14. *Accepted cost:* no household larder, so consumption cannot be smoothed out
of physical inventory and panic buying is not expressible.

### 9.5 Venues and lines

**V(R) = 9R + 1 = 37 venues** — nine per region plus one FX venue. Every venue belongs to exactly
one region and clears every line in that region's currency; the FX venue is the sole exception and
the only one whose two legs are both currency. **No global venues, and no world price.**

| Venue | Lines per region | Cadence |
|---|---|---|
| Goods | 27 | 1 |
| Labour | 3 | 1 |
| Property | 4 | 4 |
| Installed capital | 3 | 4 |
| Sovereign | 4 tenors | 1 |
| Corporate credit | 2 buckets + single names | 1 |
| Household credit | 2 | 1 |
| Equity | on a listing rule | 1 |
| Money market | 1 | 1 |
| FX (world) | 6 pairs | 1 |

**190 structural lines, 169 clearing in an average tick.** Instantiated at issue level the equity
and bond families expand to **1,276 priced lines, 1,255 clearing** — a derived quantity, not a
second taxonomy. **Hard cap 4,096, checked at venue registration.**

To buy in a region you must hold that region's currency, so **FX demand is what a cross-border
purchase is**, not a separate behavioural assumption. Households participate only in their own
region and never in FX, and transact in **three venue families** — retail consumption, labour,
household credit — never in the 27 inter-firm goods lines.

### 9.6 The venues that are not order books

### 9.6.2 Labour

Hours by class are fungible and integer, so the labour line prices a **class wage** through the
same interface as everything else. What is unlike an order book is that **the stock of employment
does not re-trade: it stands.**

**Employment is an instrument.** One contract, one worker, one employer, one wage; the term is six
years on average. Four properties make the cost proportional to activity:

1. **Hours are endowed on delivery.** No household holds hours, ever. A weekly endowment to 500,000
   households followed by the destruction of what was not worked would be 1,000,000 ledger writes a
   tick to represent leisure that nothing prices. **Unemployment survives as an observable without
   it**: the count of households with no live contract, plus the unfilled quantity on the line,
   both declared series. The bound is enforced at `register` — the sum of contracted hours across a
   household's live contracts may not exceed its endowment — as a precondition over that
   household's own contracts, not a scan.
2. **The settlement is two rows and they are budgeted as two.** Hours are issued from
   `Endowment:labour-<class>` **to the employer, batched per (employer, class)** — 100,000 rows —
   and the wage is paid gross, one row per live contract — 350,000. The payer's ability to pay is
   tested before the batch is issued, so **a firm that cannot make payroll delivers no hours and
   fails to settle**, which is §6.7's insolvency trigger with no labour-specific clause.
3. **Weekly obligations never re-bucket.** A frequency-1 obligation is due every tick, so it lives
   in a standing bucket walked whole.
4. **A wage revision writes nothing.** The contract stores a fixed margin set at hire; the wage in
   force is `wageIndex[region][class][reviewTickOf(contract, t)] × (1 + margin)` where
   `reviewTickOf(c, t) = start(c) + 52 × floor((t − start(c)) / 52)` — arithmetic on the contract's
   own start tick, so roughly a fifty-second of contracts revise on any tick without any of them
   being touched. The index is a 52-tick ring: 3 classes × 4 regions × 52 × 8 B = **5.0 kB**.
   Wage stickiness is structural and free.

**Only the participants submit**: firms with a headcount gap, households not under contract, and
those whose staggered re-plan tick has come. **Employed households under a standing contract submit
nothing.**

**Separation is `amend` under `Termination`, owned by labour**: the schedule is truncated from a
stated tick, no row is deleted, and the reason code distinguishes a quit from a dismissal.

### 9.6.3 Retail

A household posts price-taking on each **active** consumption line, and which lines are active is
staggered by good and identifier: active when `(id + g) mod cadence(g) == 0`, cadence 1 for the two
perishable goods and 4 for the other four. **Expected active lines per household per tick: 3.**

**Households may not hold consumption goods.** This is a party-class eligibility rule using
machinery that already exists. The retail fill is **one `exchange`** whose money leg runs household
→ seller and whose **goods leg is delivered to the buyer's declared sink, `Consumption:<good>` of
the buyer's region**, because eligibility forbids the buyer from holding it. One journal row names
buyer, seller, both assets and both quantities, so the buyer is the named economic recipient and
attribution is complete. It is a rule about where an exchange's primary leg lands, **not a third
party to the trade and not a new verb**.

**Utility is not a conserved asset and has no row.**

### 9.6.4 Property and installed capital

**One dwelling asset per region, one floor-area asset per region, one capital asset per class per
region.** Dwellings are fungible within a region, so a household holds *a quantity of dwellings*
and `q × price` is computable for it.

**Stock and flow are separate lines.** The property venue prices a dwelling and a tenancy; likewise
floor area and its lease. Both are needed: most households do not own, and a model with only a sale
price has no rent and therefore no landlord.

**A tenancy is a contract instrument** structurally identical to employment — one landlord, one
tenant, `Regular { principal = rent, term, start, frequency = 1 }`, in the standing bucket, rent in
force from `rentIndex` at the contract's own review tick — **but its obligation is a plain money
obligation, not an exchange**, because nothing conserved moves: the tenant already has the use of
the dwelling, which never leaves the landlord's holdings.

**No imputed rent.** Owner-occupiers pay none and receive none. Imputed rent is a flow with no
counterparty — a household paying itself is income from nowhere wearing the costume of an
accounting convention. *Accepted cost:* measured consumption and output are lower than a national
account that imputes, and the two are not comparable, which under A3 they were never going to be.

**Installed capital resells on its own line**, distinct from the goods line for new capital of the
same class. A single price would say a five-year-old machine and a new one are the same thing, and
the whole content of an investment decision is that they are not.

*Accepted cost.* There is no idiosyncratic house price, so mortgage collateral revalues for every
borrower at once and the model cannot generate negative equity from house-level dispersion. It
generates it from **loan-level** dispersion — vintage, LTV at origination, amortisation state —
which is where the non-linearity was put deliberately.

### 9.7 No rest of world

**The four regions are the entire world.** There is no rest-of-world sector, no external
counterparty, no residual absorber of net trade or net saving.

**Every claim a deficit region issues must be wanted, at a price, by a modelled holder in one of
the other three. That is not an identity to be satisfied at the end of the tick; it is the FX
line's clearing condition.** If region A's firms import from B, they need currency-B, which they
allocate at 9 and obtain at 10–11. Somebody must sell it: B's importers buying A-goods, B's
investors acquiring A-currency claims, A's exporters repatriating last tick's receipts, or B's
central bank accumulating reserves. **The rate moves until one of them is willing.** If no rate the
grid reaches produces a crossing, the line partly fills, A's importers are rationed, the imports do
not happen, **and the deficit is not run.**

**The current account is bounded by its own financeability, and the exchange rate is the price that
does the bounding.** Nothing anywhere computes a residual and hands it to a sector that must accept
it.

*Accepted cost.* Exchange rates are more volatile than in a model with a rest of world, and trade
can be rationed by financing rather than by price alone. Both are consequences of A4.

**Reserves are a read view, not a table**: a central bank's holdings restricted to assets whose
issuer is outside its region. The world's total holding of currency *c* is exactly what region *c*'s
central bank has issued, readable as `−q(issuerOf(c), c)` — one indexed load. Foreign holdings are a
partition of that total, not an addition to it.

**The closure identity, per currency, declared four times and never converted.** For each of the
28 (region, sector) buckets, each currency, each tick:

```
(C1)  per bucket:    CB(b,c,t) + FB(b,c,t) = Δmoney(b,c,t)
(C2)  per currency:  Σ_b CB(b,c,t) = 0   and   Σ_b FB(b,c,t) = 0
```

so **sectoral net lending is exactly zero every tick** — an integer identity, because every money
leg writes one integer as a debit and the same integer as a credit, partitioned 28 ways. A single
currency-converted world identity would be a real-valued sum, and A1's exactness would be lost in
the conversion: **four exact identities are strictly stronger than one approximate one.**

The class of a money flow comes from a **total mapping from reason code to flow class**, so a new
reason code cannot be introduced without classifying it.

**The flow-of-funds matrix is a report, not a check.** 29 × 29 × 4 × 2 integers, 53.8 kB, one
increment per money move. Both sides of its total come from the same increments, so comparing them
would be verification whose two sides have one source. **Its content is the individual entries**;
the total is not observed, because a series that is identically zero by construction is a check
that cannot fail. If the closure panel ever shows a non-zero world total, **the reader is broken
and not the world**, and the suspect list is three items long: the sector partition, the par-value
join, the sign convention.

**Triangular consistency is deliberately not imposed.** The six FX lines clear independently;
imposing consistency needs a joint solve that destroys independent line evaluability. A cross-rate
inconsistency is a **modelled arbitrage opportunity**, exploitable from the next tick, and the four
triangular residuals are declared series.

---

## 10. Time and capabilities

**A tick is one week.** Everything slower declares a cadence and skips the ticks it has no content
for; nothing is faster.

**Capabilities are minted from the manifest, by the crate that owns the invariant being narrowed.**
A system declares what it needs; the owning crate mints a handle that can do that and nothing else.
**No handle constructor and no widening function is public anywhere**, and a manifest naming a
capability it does not own fails to mint at start-up rather than at first use.

**A handle is a type with a private field and no public constructor** (D2), so it cannot be forged,
widened or reconstructed from its parts anywhere outside the crate that mints it. Under the previous
edition this was a naming convention a lint defended; it is now the compiler's.

**Reads carry their age.** A read is typed by how old it is allowed to be, so a decision that reads
a stale mark is a type error rather than a subtle result.


---


---

## 11. Determinism

Two runs from the same seed produce the same world, bit for bit, at one thread and at 64 shards, **and
between the CI machine and the device**. Everything below is a prohibition. **Most are now enforced by
the type system or the crate graph rather than by a lint** (D2); the three that remain real work are
marked, and they are the ones worth engineering attention:

- **No global random stream.** Randomness is a pure function of (world seed, stream, entity,
  index), so a draw does not depend on how many draws preceded it.
- **⚠ No transcendental on any path reaching a digested value** — not merely in samplers. `ln`, `exp`,
  `sin` and `cos` lower to the platform's math library, so an x86 CI host and an ARM device can disagree
  in the last bit, and the golden digest must reproduce on the device. **The previous edition scoped this
  to samplers, and §13.4's output gap slipped through the letter of a rule it violated in substance**:
  `log(output)` is computed inside the engine and read by a decision. Either §13.4 states the gap as a
  ratio, which removes the transcendental entirely, or the engine carries a correctly-rounded software
  implementation with declared rounding. **This is one of the three prohibitions that is real work.**
  The build-time seed generator (§13.3) is exempt: it runs once, off the device, and ships its output as
  `derived` entries.
- **No ambient state below the two leaf layers.** No module-scope mutable singleton; everything a
  system needs is passed to it.
- **One wall-clock module**, in `composition/host`. Duration is `NonDeterministic<Nanoseconds>`,
  never saved and never digested; **no `NonDeterministic<T>` path reaches a world-derived type**. The
  type has a private field and no arithmetic, so the prohibition is the compiler's.
- **No unordered iteration where order matters.** Iteration order over a map is not a guarantee to
  depend on.
- **⚠ Cross-entity sums go through a declared `Accumulator<T>`, folded left in ascending shard
  order**, never through a variable a shard body closes over. Real-valued addition is not
  associative, so the fold order is part of the answer — and the order *within* a shard is declared too,
  because a deterministic fold over non-deterministically-ordered partials is still non-deterministic.
  **Real work**: the ownership rules stop a shard body writing another shard's lane, but nothing stops a
  fold being written in the wrong order.
- **The shard count is 64**, a saved run parameter and not a device property. Thread count is scheduling
  and carries no semantic content: **one thread executing all 64 shards in ascending order is the
  reference result**, and any other thread count must reproduce it bit for bit.
- **The digest is over the live set in identifier order**, never slot order — rows relocate, so two
  runs that agree on every fact can disagree on every slot — **plus each table's retirement
  accumulator**, so what a run destroyed is bound in.

- **⚠ No unmarked write to a digested column.** The tick digest is taken over dirty regions, so a write
  path that forgets to mark its pages makes the digest agree where the world differs — which turns the
  differ into a generator of false negatives at exactly the moment a determinism bug appears. **Real
  work**: the set of code that can write a digested column and the set that marks pages are both derived
  from the schema and must be equal, with a debug build cross-checking a full-arena hash against the
  accumulated dirty-page hash at end of tick.

**The differ is built before the first optimisation.** A digest alone says *that* something changed
and is nearly useless for finding out *what*, and it is the single instrument that is painful to
add late.

## 12. Performance

### 12.1 Two targets

**These are targets, not requirements (D1).** They are measured nightly, their trend is published, and a
breach is restated rather than paid for out of the model. **No figure in this section may be met by
reducing the agent population or coarsening a cadence** — that is a retreat from M1, and M1 is the point
of the project. What may move: the tick budget itself, the run length, the device, and the amount of
amortised maintenance the design carries.

| | **N2a — interactive** | **N2b — throughput** |
|---|---|---|
| Figure | **680 ms** end to end; engine ≤ 570 ms, surface ≤ 110 ms | **≥ 1.70 ticks/s** sustained |
| Statistic | p95 of a single tick | mean over ticks 260–1,560 |
| Regime | burst, from idle, at peak clock | steady state, throttled |
| Constrains | the **worst tick's work** — the cadence table, the fill rate, the line count. It is what forbids a system that runs every tick over every household | the **whole run's cost** — how much amortised maintenance the design may carry. It caps checkpointing at 1% of a tick |

**N3 — no per-tick cost proportional to stock where it can be proportional to activity.** Two
limbs: the tick at 1,560 periods may not exceed 1.10× the tick at 260, and the fitted slope of tick
time against period index may not exceed 0.02 ms per period. **A breach is a defect, not a
regression**: it means a structure is growing with run length. **N3 stays a requirement under D1**,
because it is a statement about structure rather than about speed.

**N4 — peak allocation ≤ 1,610 MB**, a target. **Capacities are `structural` registry entries carrying
the arithmetic that produced them, and exhaustion raises rather than reallocating.** A capacity change
is an ADR.

*Unresolved, and it is the largest open number in this document.* The 1,488.3 MB was published without
an itemisation and cannot be reproduced from the sections that own its parts. The five named components
— journal 345.6, holdings 171.4 (§3.4, corrected), agent rows 95.6, `plans` and `intents` ≈ 121,
directory 47.5 MiB — total ≈ 783 MB, leaving ≈ 705 MB unaccounted, and the largest term inside that
residue is instrument count × row width, which §7.5 declares unsettled while also saying its own family
counts are not to be used for sizing. **Until a table with one row per world table, capacity × width,
is published and summed, N4 is not a figure anybody can check.** It is a target rather than a
requirement, so this does not stop the work — but it does mean no claim about memory should be made
until the derivation exists.

### 12.2 The derived tick

478.7 ms single-threaded at peak clock, dominated by position 14's 1,671,884 exchanges (161.3 ms),
position 4's 926,246 per-contract payments (87.8 ms) and position 13's clearing (84.8 ms).
**3,119,665 operation calls per tick.**

*This figure is a derivation awaiting a measurement.* 478.7 ms over 3,119,665 calls is 153 ns per
operation, and position 14's share is 96.5 ns per exchange — for an operation that performs two door
precondition sets, two block lookups into a structure far larger than any device's last-level cache,
four accrual read-modify-writes, two quantity writes, one quantization and one journal append. On a
phone's memory system that budget buys roughly one cache miss. **The first work in the build measures
it** (`IMPLEMENTATION.md`, milestone 1). If the real figure is several times larger, §12.1's targets are
restated and the cadence table, the scale classes and this section are re-derived — the model is not
reduced to fit them (D1).

### 12.3 The acceleration seam

**One seam, declared now and fixed now**, because two acceleration strategies against one seam
compose to neither.

- **Two parallel windows: W1 = positions 8, 9, 10 and W2 = 12, 13.** They do not merge, because
  position 11 is a settlement and the ledger must be open for it. Seven barriers a tick.
- **The unit of work is a `(system, shard)` pair; 64 shards**, a saved run parameter.
- **The ledger stays sequential.** A `ShardHandle` carries no verb.
- **Shards are cut by row count, not by region** — the regional populations stand at 2.56 : 1, so
  region-shaped shards would idle three workers in four.
- Folds are by ascending `ShardIndex`, except at the two clearing positions where the shard unit is
  a **line** and the fold is by ascending line identifier.
- **One construction site** for thread spawning and the shared arena. **A second acceleration boundary
  is an ADR at any phase**, because it has no bounded cost.

**The engine must be correct and within budget single-threaded.** Parallelism is a 1.0× assumption;
anything above it is opportunistic and may not be quoted as the model's speed.

### 12.4 Scale classes

| Class | Households | SMEs | Peak MB | Engine | Ticks/s | 30-year run |
|---|---|---|---|---|---|---|
| **`full`** | 500,000 | 50,000 | 1,488.3 | 478.7 ms | 1.78 | 14.6 min |
| `half` | 250,000 | 25,000 | 946 | 274 ms | 3.11 | 8.4 min |
| `tenth` | 50,000 | 5,000 | 508 | 83 ms | 10.2 | 2.6 min |

**A run at a reduced scale class is not this model**, and the surface states which class a run was
performed at. The engine **refuses to start** where the platform cannot carry the requested class,
rather than shedding agents mid-run — which would be the leak A4 forbids wearing the costume of
graceful degradation.

**`half` and `tenth` exist for tests and nightly sweeps and for nothing else** (D1). No result quoted
from either is a result about this model, and the reduced classes may never be used to bring a
performance target into range — under D1 the target moves instead. The three rows' derived figures
inherit §12.2's uncertainty and are re-derived once the measurement exists.

---

## 13. The seed

### 13.1 The opening primitives

**The opening world is generated from primitives, not from levels.** The seed is a closed list, and
the `scope` column is what makes §13.3's asymmetry possible without a single region-scoped
assumption.

| Primitive | Unit | Scope | Provenance | Value |
|---|---|---|---|---|
| Region count | count | world | structural | 4 |
| Sector count | count | world | structural | 7 |
| Household count | count | axis 1 | assumed | 500,000 |
| Large-firm count | count | axis 1 | assumed | 440 |
| Bank / fund / insurer-and-pension / SME counts | count | axis 1 | assumed | 56 / 54 / 40 / 50,000 |
| Cohort shares | ratio | axis 3 | assumed | 0.22, 0.38, 0.27, 0.13 |
| Labour endowment per household | hour | axis 3 | assumed | 40 |
| Capital units | count | axis 1 | assumed | 20,000,000 |
| Dwellings | count | axis 1 | assumed | 450,000 |
| Land | physical-unit | axis 1 | assumed | 4.0 × 10¹⁰ m² |
| Capital service life | period | axis 2 | assumed | 1,040 |
| Hour productivity | ratio | axis 2 | assumed | 1 |
| Preference: reservation ratio, trait dispersion | ratio | axis 3 | assumed | see §13.1.1 |
| Policy slopes `a`, `b`, `c` | ratio | world | assumed | 1.5, 0.5, 0.10 |
| Policy anchor window `K` | period | world | assumed | 104 |
| Asymmetry quadruple `Z` | ratio | world | structural | §13.3 |
| Axis permutations `P₁ P₂ P₃` | ratio | world | assumed | §13.3 |
| Axis log-dispersions `δ₁ δ₂ δ₃` | ratio | world | assumed | §13.3 |
| Numéraire scale `S` | minor-unit | world | structural | 2 × 10¹¹ |
| Per-region currency issue | minor-unit | region | **derived** | `S × labourShare(r)` |
| Per-region physical and institutional vectors | count | region | **derived** | §13.3 |

**Note what is not in the list: no price, no rate, no spread, no index level, no balance-sheet
size, and no second independent nominal quantity.**

#### Three rules

1. **A world total is the settled decision; the split is derived from it.** Each is one world-scope
   entry attached to an axis. Neither carries four numbers.
2. **The four regional values are the generator's output**, rounded by §6.3 rule two so they sum to
   the world total exactly.
3. **No per-region count is written anywhere.** A reviewer wanting a bigger region argues about a
   loading, in dimensionless terms. **There is no line in the source tree where a region's
   population can be typed.**

#### The opening world contains no financial instruments

No bonds, no loans, no deposits, no equity positions seeded with terms. Every instrument in the run
is issued by the model, so no coupon, no spread, no rating threshold and no curve is ever seeded.
**Firms and banks capitalise themselves in-model during the burn-in.**

Under A4 this is more than an A3 convenience: **no seed row names a `Claim` asset**, a build lint,
so every claim asset's total across all holders is zero at period 0 because none has been issued.

The tick-0 monetary position is a **derivation, not a seed entry**, and it runs as a bilateral
expansion rather than a bare transfer:

```
register(Gilt:0r)
move(Gilt:0r,    government_r,  centralBank_r,  face)     face = M_r(0), at par
move(Notes:cb_r, centralBank_r, government_r,   M_r(0))
```

so at the close of tick 0 **every entity's net financial position is exactly zero**, not merely the
world's. A single `move(currency, centralBank, government, S)` would have left the central bank
with a liability and no asset: negative net worth at birth, which is neither realistic nor
necessary.

### 13.1.1 Prices are unseeded, and what that costs

**The first clearing forms reservation levels from primitives directly** — marginal cost from
technology, willingness to pay from preferences — rather than from a prior mark. §9.3's prior-close
rule takes effect from period 1, so the distinction between an opening condition and a term
collapses: **there are no seeded terms at all.**

**Those primitives are dimensionless and a reservation is money.** §16.1 rule 1 requires technology
and preference primitives to be `ratio`-dimensioned, and dimensionless times dimensionless is
dimensionless, so something must carry the unit. The numéraire is the only thing that can, and
§13.2 registers it for exactly that: *the unit the whole world is counted in, against which every
nominal figure is derived.*

**An agent's period-0 reservation is its own dimensionless primitive times a nominal anchor derived
from `S`.** The anchor sets the **unit**, never the level; the price is whatever crosses. For
labour the anchor is `S / Σ_r H_r` = **9,700.36 minor units per hour**, identical in every region
because §13.2 issues `M_r(0) = S × H_r / Σ H` — so a regional wage spread has to be produced by
four regions clearing differently rather than built into the unit they quote in.

*The one economic assumption, stated:* that money turns over once per period. Velocity is an
outcome, so this is wrong in general and does not need to be right — **the anchor's job is to place
§9.6.1's grid, not to set the price.** The grid spans a factor of four and rebases once, so an
anchor wrong by a factor of a few costs one rebase; one wrong by sixteen raises a defect that says
so, rather than clearing quietly at a level nobody chose.

### 13.1.2 The technology, and why there is one sub-unit

§13.1 previously listed the technology coefficients as "input–output ratios, 27 sub-units, per
specification", and gave none. The previous edition then reduced them to one composite sub-unit, and
**gave the budget as the reason**: twenty-seven sub-units at three coefficients each is 81 `assumed`
entries against a cap of 80.

**That reasoning is withdrawn (D3), and the decision is reopened.** The cap no longer exists, and a
modelling question answered by arithmetic about a budget was never answered on its merits. A rule that
decides how much structure an economy has by counting rows is not serving M2, and this section is the
place the previous edition recorded it happening.

**What must now be decided, on the model's terms:** how many production sub-units the opening carries.
The arguments are about the economics and not about the count. In favour of more: with one composite
good there is no relative price between goods, no input–output structure, no sector-specific shock, and
inter-firm trade is trade in a single homogeneous thing — which makes §9.5's twenty-seven goods lines per
region meaningless and much of the trade motive with it. In favour of fewer: each sub-unit costs
coefficients that are genuinely assumed, and M3 counts them.

**§9.5 and this section currently contradict each other** — twenty-seven goods lines per region against
one composite sub-unit — and the contradiction is load-bearing: it changes the line census, the retail
staggering of §9.6.3, observation family 3's declared count, and §3.4's estimate of position 14's
1,671,884 exchanges. **It is settled before the line registry is built**, and whichever way it is settled
the sections derived from it are re-derived.

Whatever the count, input intensities are `derived` from the world's own capital and land endowments
against its labour supply — the technology that uses the endowment fully, which is a stated modelling
claim and costs no assumed entry — plus the assumed hour-productivity ratio §13.3 names on axis 2.

*Accepted cost.* The opening is a full-employment configuration by construction, so the model
cannot **start** with idle capital or idle land. Slack has to arise from dynamics.

### 13.2 The numéraire

`S = 2 × 10¹¹` minor units, `structural`, identity `UnitOfAccount`.

**It is not "a pure scale factor that changes nothing real", and that justification is withdrawn.**
It is false in two ways and both matter. **Quantization does not commute with scaling**: §6.3
quantizes every `price × quantity` to an integer minor unit, so under `S` and `2S` the residues
differ, the sweeps land on different holders, and §8.1's constraint tests bind at different moments
on different agents. And **`S` fixes the model's arithmetic resolution**: the smallest economically
meaningful recurring flow — one household's wage for one period — must be at least **10⁴ minor
units** at the burn-in gate, so that one minor unit is at most 10⁻⁴ of it.

Each region's opening money is `M_r(0) = quantize(S × H_r / Σ_s H_s)`, `derived` — every region
opens with the same quantity of its own money per hour of its own labour. That is an initial
condition and an arbitrary one, but arbitrary in the way A3 requires: an operation on primitives
rather than a resemblance to an observation. *Accepted cost:* the model **cannot represent** a
region that opens monetarily deep relative to its labour force. It can only produce one.

### 13.3 The four regions

*This section owns every per-region figure in the model. No other section prints one.*

The four regions must be deliberately unalike — four equal regions are four clones, and a clone
world has no cause for trade. The difference is **generated**, not asserted: a per-region column on
every primitive would be `4 × K` opportunities to type an observed figure, and §16.1 rule 2 fails
the build on any `assumed` entry with region scope, without exception.

#### The generator

Each asymmetric primitive attaches to exactly one of three axes. Region `r`'s value is

```
p_r = p_base × exp( δ_k · Z[ P_k[r] ] )
```

Counts are then quantized against the world total by §6.3 rule two, ties by ascending region
identifier, so the four sum to the world total exactly.

| | Value | Provenance |
|---|---|---|
| `Z` | `(−3, −1, 1, 3)/√5` = −1.341641, −0.447214, 0.447214, 1.341641 | `structural`, identity `AsymmetryQuadruple` |
| `P₁ P₂ P₃` | `(1,2,3,4)`, `(2,4,1,3)`, `(3,2,1,4)` | `assumed`, under the proved bound max pairwise \|ρ\| ≤ 0.4 |
| `δ₁ δ₂ δ₃` | 0.35, 0.20, 0.15 | `assumed`, `ratio`, world scope |

`Z` is the unique zero-mean, unit-variance, equally-spaced quadruple: zero mean so the world
aggregate is unchanged, unit variance so `δ_k` alone carries magnitude, equal spacing so the
distribution privileges no region. It is arithmetic, not a choice.

**Rank 1 takes the smallest loading.** This is the only reading that reproduces the axis-1 share
row, both multiplier rows and the sign of the stated ρ(P₂,P₃) = +0.4, and it makes region 0 the
smallest region on axis 1.

#### The three axes

| Axis | What it varies | Primitives on it | `δ` | Extreme ratio |
|---|---|---|---|---|
| 1 — scale | how much of the world's productive apparatus a region holds | every institutional and household count, capital units, dwellings, land | 0.35 | 2.56× |
| 2 — technology | how a region turns inputs into output | input–output coefficients, depreciation rate per capital class, hour productivity | 0.20 | 1.71× |
| 3 — preference | what a region's households want and can supply | cohort shares, labour endowment hours, intertemporal share, labour-supply elasticity | 0.15 | 1.49× |

A fourth axis is an ADR under §21.3.

#### The generator's output

**Everything below is `derived` and is computed at build.** It is printed here so a reviewer can
check the generator, not so anyone can copy it: §13.1 rule 3 stands — there is no line in the
source tree where a region's population can be typed.

| | R0 | R1 | R2 | R3 | World |
|---|---|---|---|---|---|
| Axis-1 share | 0.147152 | 0.201244 | 0.275219 | 0.376386 | 1.000000 |
| Households | 73,576 | 100,621 | 137,610 | 188,193 | **500,000** |
| SMEs | 7,357 | 10,062 | 13,761 | 18,820 | **50,000** |
| Large firms | 64 | 89 | 121 | 166 | **440** |
| Banks | 8 | 11 | 15 | 22 | **56** |
| Funds | 7 | 11 | 15 | 21 | **54** |
| Insurers and pension funds | 5 | 8 | 11 | 16 | **40** |
| Capital units | 2,943,040 | 4,024,871 | 5,504,371 | 7,527,718 | **20,000,000** |
| Dwellings | 66,218 | 90,560 | 123,848 | 169,374 | **450,000** |
| Land, m² | 5.886×10⁹ | 8.050×10⁹ | 1.101×10¹⁰ | 1.506×10¹⁰ | **4.0×10¹⁰** |
| Axis-2 multiplier | 0.914441 | 1.307776 | 0.764657 | 1.093565 | — |
| Axis-3 multiplier | 1.069383 | 0.935118 | 0.817711 | 1.222926 | — |
| Hours per household | 42.775 | 37.405 | 32.708 | 48.917 | — |
| Capital service life, periods | 951.0 | 1,360.1 | 795.2 | 1,137.3 | — |
| `H_r` = households × hours | 3,147,238 | 3,763,702 | 4,501,009 | 9,205,842 | **20,617,791** |
| Labour share | 0.152647 | 0.182546 | 0.218307 | 0.446500 | 1.000000 |
| `M_r(0)`, minor units | 3.053×10¹⁰ | 3.651×10¹⁰ | 4.366×10¹⁰ | 8.930×10¹⁰ | **2.0×10¹¹** |

**The world totals are decisions; the splits are not.** A reviewer who wants a larger region 3
argues about `δ₁`, in dimensionless terms, not about a number.

**The population ratio is 2.56 : 1**, which is `exp(2 · 0.35 · 1.341641)` — δ₁'s extreme ratio, and
a check that the table is the generator's output rather than a transcription. It has one
consequence outside this section: **D-7 shards by row count, not by region**, or three shards in
four idle.

#### What is owed

`δ₁ = 0.35` has **no surviving in-model justification**. The one it carried was that the smallest
region still supports eighteen banks, which was computed against an institution total this document
no longer carries; the smallest region has eight. §13.3's own standard applies — *a dispersion is
justified by what it does to a mechanism inside the model, never by what it resembles outside it* —
and it must be met before §15.3.4's sensitivity sweep means anything, because the sweep is what
makes a bracket a claim rather than a comment.

`δ₂ = 0.20` is bounded above by the requirement that no region's technology makes a sub-unit
unprofitable at period 0. `δ₃ = 0.15` is bounded above by the requirement that the smallest labour
endowment still exceeds the largest sub-unit's minimum staffing at period 0. Both stand.

*Accepted cost.* Regional counts stop being memorable or choosable, and four regions can no longer
be tested against one another for equality — a cheap class of regression test, given up.


---

### 13.4 The policy rate

**A difference rule with an internal anchor. No intercept exists anywhere.**

```
i_t = i_{t−1} + a·(π̂_t − π*) + b·ŷ_t + c·( (ρ̄_t + π̂_t) − i_{t−1} )
```

| Term | What it is | Where it comes from |
|---|---|---|
| `π̂_t` | trailing `K`-period growth of the bank's **own published price index** | a posted column: a decision reads it, so it is state |
| `π*` | the target | **0**, `structural`, identity `FixedPointOfPublishedIndex` |
| `ŷ_t` | the output gap. **`log` is not available to the engine (§11)**, so this is `(output_t − μ_K) / μ_K`, a ratio, which is dimensionless and needs no transcendental | a posted column |
| `ρ̄_t` | trailing `K`-period mean **realised real return on the region's capital stock** — value added net of `Wear:` over capital at market value | a posted column |
| `i_{t−1}` | the previous policy rate | a posted column |
| `a, b, c` | 1.5, 0.5, 0.10 | `assumed`, `ratio`, world |
| `K` | 104 periods | `assumed`, `period`, world |

**Where the level comes from: `ρ̄_t + π̂_t`, the model's own realised nominal return on capital**,
averaged over two years. That is a Wicksellian anchor and an **output of the simulation**: the rate
the bank converges to is whatever the world's capital earns. The `c` term is what gives a
difference rule a level at all — without it the rate has a unit root and wanders.

**`π* = 0` is not a seeded level.** Zero is the fixed point of a growth rate: the identity element,
and the only value that is not a choice.

**The meeting cadence is 13 ticks, world scope, phase `regionIndex mod 13`.** Thirteen divides 52,
so a region meets exactly four times a simulated year and the schedule does not drift against the
annual cycle. **A per-region cadence vector is not writable** — it would be a region-scoped
`assumed` value, which rule 2 forbids. What differs by region is the phase, which is `derived` from
the region identifier and carries no economic content.

*This is the one place `mod` on an identifier is correct*, and §8.6 forbids it for agents for the
opposite reason: a committee's meeting date is supposed to be a function of the region, and a
household's re-plan tick is supposed not to be a function of anything.

**The trailing statistics are state columns with one owning system**, written at a declared position
**after position 17's marks**, and read as prior-close by position 7 the following tick.
**No policy rate exists before tick 104.**

*Owed.* §9.4's committed order has twenty-one positions and none of them is the trailing-statistics
system this section requires. It reads position 17's marks and is read by position 7 the following tick,
so it sits after 17; it must be given a stable position name in §9.4 or the order does not describe the
model.

*Accepted cost.* The four central banks cannot meet on different rhythms, so a model in which one
region's policy is structurally more reactive than another's is unavailable. Regional policy
difference is carried by the slopes' **inputs** — each region's own `π̂`, `ŷ` and `ρ̄` — not by how
often the committee sits.

### 13.5 What a save contains

Every column, the directories, the retirement accumulators, the extinguished-stock register,
`plans`, `intents`, the resolution register including a mid-flight shard cursor, and the
observation store. A saved state carries a **schema identifier and a scale-class identity**, and a
build whose schema differs **refuses to load it and says so** rather than coercing: a coerced load
is a world that silently differs from the one that was saved. The schema identifier is a hash of the
generated column schema, so it changes when the layout changes and cannot be forgotten.

Observations are saved with the world but **excluded from the golden digest**, carrying their own
series digest, so a reporting change does not force a state re-baseline.


---


---

## 14. Observation

**Fourteen families, each with one owning system, a declared count and a sub-cap.** 624 series
declared against a **hard cap of 2,048**; the sub-caps sum to exactly 2,048, so the total cannot be
breached without a sub-cap being breached first, and **the sub-cap is the mechanical guard**. A new
series above its family's sub-cap retires one in the same family.

| # | Family | Declared | Sub-cap |
|---|---|---|---|
| 1 | Real creation and destruction — one per (counter-account family, real asset class, region) | 40 | 64 |
| 2 | Output and expenditure | 30 | 64 |
| 3 | Line clearing levels and indices | 206 | 576 |
| 4 | Labour | 20 | 128 |
| 5 | Credit | 24 | 128 |
| 6 | Credit loss | 12 | 32 |
| 7 | Rates and curves | 28 | 128 |
| 8 | Market microstructure — submissions, cleared depth, unfilled quantity, per venue | 111 | 256 |
| 9 | Sectoral balances | 60 | 256 |
| 10 | Issued-claim stock by issuer | 28 | 128 |
| 11 | Lifecycle and resolution | 38 | 128 |
| 12 | Registry health — placeholder count and trend, `assumed` count, total | 4 | 16 |
| 13 | Rows touched, per position and per shard | 21 | 128 |
| 14 | Closure — world net lending, world issued-claim stock | 2 | 16 |
| | | **624** | **2,048** |

**42 of the 624 carry the stationarity flag** for §15.3's gate — a flag on a declaration, not a
fifteenth family.

**The placeholder count is the honest measure of how much model is missing**, and it is on the
dashboard next to the counter-account flows and the closure panel.

### 14.1 The closure panel

Per region and for the world: issued-claim stock by issuer sector, net lending by sector,
revaluation by sector, and **the world net-lending total displayed as a literal `0`** — never as a
computed sum. It is structurally zero, so a non-zero display would be a defect in the reader, and
computing it would be a check that cannot fail.

## 15. Correctness

#### The assurance ladder

| Tier | What | Where |
|---|---|---|
| 1 | properties the compiler enforces | types, branded identifiers, typed doors, total mappings |
| 2 | properties a check over the source tree enforces | single-writer checks, import checks, lints, the registry build check |
| 3 | properties a test enforces | the conformance suite, unit and property tests, the benchmarks |
| **4** | **a pass over the world at runtime** | **empty, by design** |

**Tier 4 is empty and stays empty.** If closure genuinely fails, the failure is in a door
precondition, a total mapping or a single-writer check, and it is found there. **A proposal to add
a world-summing diagnostic pass is an ADR against A1 and A4 and is refused as a whole**, not merged
as a diagnostic.

### 15.1 The conformance suite

**Eight parties across two regions** — two households, two banks, a firm, a fund, a government and
a central bank — **two currencies**, one good, one bond, one secured loan with a two-hop
rehypothecation chain, one fungible deposit line held by all eight, one employment contract, and
one insolvency. **Every operation and every door exercised.** It runs in milliseconds.

**This suite is the acceptance test for the substrate and is written before the first economic
system.** Twenty numbered cases:

| # | Asserts |
|---|---|
| 1 | conservation, as a property test over generated legal sequences including `exchange` and `retire`, not moves alone |
| 2 | issued-claim closure: the sum of a deposit across all holders including the bank is **exactly zero at every intermediate point**, and no counter-account appears in its journal rows |
| 3 | the issuer rule: a household cannot go negative in a bank's deposit; the bank can; the bank cannot go negative in a currency it did not issue; the central bank can |
| 4 | each counter-account's period delta equals the sum of journal rows naming it — independently produced, so the comparison has content |
| 5 | counter-account monotonicity and asset scope |
| 6 | capability and actor: `EstateAgency` gates a debit from an `in-resolution` holder; neither handle touches a `resolved` one; `actor` names the **system**, for a call whose every argument named a different entity |
| 7 | encumbrance over the §6.5 worked example: the depth-3 chain builds, `encumbered` stays at the root quantity at every depth, all six refusals raise, the chain unwinds leaf-first |
| 8 | interior failure: the middle party is wound up, the child lien enforces against the root pledgor's units, the parent reduces by the same quantity |
| 9 | wind-up with an over-collateralised secured creditor, an under-collateralised one with dual recourse, a non-recourse one, an unsecured one and a genuine shortfall — and the entity reaches `terminal` holding **exactly zero of every claim it issued** |
| 10 | holder eligibility: a pledge to an ineligible beneficiary raises; the queue pays in cash; an asset no venue bids for lands on the government, **which pays the last mark** |
| 11 | beneficiary identity: a cover-pool lien names a trustee, which distributes pro rata and ends with zero |
| 12 | fungible resolution, sharded: the per-holder result is **identical** in two shards and one, and units held plus units extinguished equals the issue **at the end of every tick** |
| 13 | fungible distribution across three holders whose quantities do not divide the coupon: quantized **once**, allocated cumulative-proportionally, issuer's debit equal to the sum of credits. **This is the case that fails if anyone quantizes per holder** |
| 14 | two carriers, one walk — and **the payment walk's source contains no `InstrumentTypeCode`** |
| 15 | schedule identity: `Regular` and `Explicit` produce identical due sets at every tick |
| 16 | amendment: `Defer`, then `Reparameterise`, then `Truncate`, then `Insert` — base parameters never edited, delta chain and journal independently produced and agreeing, **the walk's source referencing neither constructor**, and each of the eight mechanisms attempted through a handle not minted for it raising |
| 17 | exchange across two currencies: one row, two assets, a realised rate, no residue on either leg, never observable half-settled, **no counterparty outside the two regions** |
| 18 | posted columns: a second `post` raises; a decision handle yields the **prior** tick and has no method yielding this tick's; an unpriced instrument reads `NotPriced` and multiplying it is a compile error |
| 19 | retirement cannot open the loop: the identifier is not reused, a lookup returns `Retired`, a relocated row still resolves, retiring something still held raises, and the conserved sum **including the extinguished-stock register** is unchanged. **This is the case that decides whether §5.5 is a memory optimisation or a leak** |
| 20 | round trip: save, reload, digest — bit-identical, including a resolution **mid-flight** |

Case 2 carries one further assertion, made **here and nowhere else**: over the deposit's whole life,
net lending computed from the journal equals the tick-over-tick difference of net financial position
at par, for every sector, every tick. Two independent productions compared in a toy world is a test;
the same comparison over the live world every tick would be tier 4.

**The list does not soften to protect an estimate.** It needs a sovereign instrument type to exist
in Phase 1a rather than Phase 2, and the estimate moves with it.

### 15.2 What this design cannot do

When leaks become impossible, **leaks stop being a diagnostic**. A model that cannot leak also
cannot tell you, by leaking, that a mechanism is missing. That lost signal is bought back by the
counter-account flow series and the closure panel — **and only if somebody looks.**

### 15.3 The burn-in gate

A3 forbids comparing the model to an observed economy, which is the comparison that would normally
tell a team the burn-in is over. **The criterion is internal: the model is compared to itself, and
nothing it computes feeds back into any parameter.** A gate that tuned would be the calibration A3
forbids.

**The panel is 42 series, all dimensionless or stationary by construction.** Levels are excluded: a
growing economy's output *should* trend, so testing a level for stationarity tests the wrong thing.

Window `W = 104`, ensemble `E = 16` seeds, all statistics computed **outside the engine** from the
observation store. **No engine handle reads them.**

| Test | Statistic | Passes when | Establishes |
|---|---|---|---|
| **B1** no drift | OLS slope `β̂` on period index over `W` | `\|β̂·W\|/μ_W ≤ 0.05`, or `/σ_W ≤ 0.50` | the series is no longer going anywhere |
| **B1b** no regime shift | split `W` in half | `\|μ₁−μ₂\| ≤ 0.25·σ_pooled` **and** variance ratio ≤ 2.0 | the second half is the same world as the first |
| **B2** the ensemble mixed | `R` = cross-seed sd of the trailing-52 mean, end over start | `0.80 ≤ R ≤ 1.25` | the spread across seeds has stopped changing |
| **B3** the opening is forgotten | Spearman ρ across seeds between period-0 value and gate value | `\|ρ\| ≤ 0.503` | the world's arbitrary opening no longer predicts where it is |

`0.503` is the two-sided 5% critical value of Spearman's ρ at n = 16 — a property of a
distribution, which is arithmetic, registered `structural`.

**`burnInPeriod` is the first period `P` with `260 ≤ P ≤ 520` at which all four pass on all 42
series.** It is in the run manifest, the surface refuses to present any earlier period as a result,
and it is part of the digest's metadata.

- **The floor is 260 and is derived:** no policy rate exists before period 104 and its anchor window
  is a further 104, plus 52 of margin.
- **The ceiling is 520.** A world that has not settled in ten simulated years is not converging
  slowly, it is broken. **Reaching 520 without a pass is a defect**, and it is the first evidence
  this design has ever had that the economics is wrong rather than merely unfamiliar.
- **Which test fails is the diagnosis.** B1 alone is a drifting mechanism; B1b alone a regime change
  mid-window, usually a threshold beginning to bind; B2 high an ensemble still spreading; B3 alone
  path dependence on the opening, which is what an over-large `δ` produces.

**Seven `assumed` entries are declared sensitive** — `a`, `b`, `c`, `K`, `δ₁`, `δ₂`, `δ₃` — and the
gate runs at both ends of each bracket nightly. **That is what makes a bracket a claim rather than
a comment**, and it is why a bracket with no in-model justification is not yet a bracket.

*Accepted cost.* A run's usable output begins somewhere between period 260 and 520 and the team
does not know where until the gate has run.


---


---

## 16. Cross-cutting concerns

### 16.1 The parameter registry

**Every number the model opens with is an entry in one registry, and the build refuses one that is
not.** The schema is eight fields:

| Field | Content |
|---|---|
| `name` | dotted, unique |
| `value` | the number |
| `unit` | drawn from a closed vocabulary; the unit determines the dimension |
| `scope` | `world`, `axis:k`, or `region:r` |
| `owner` | the system that reads it |
| `provenance` | `structural` \| `derived` \| `assumed` \| `placeholder` |
| `basis` | discriminated by provenance: an identity for `structural`, an expression for `derived`, a bracket and axis for `assumed`, a deleting ADR for `placeholder` |
| `justification` | in model terms only |

#### The six rules of the build check

**These six are build failures and remain build failures.** D3 removed the cap on how many entries there
may be; it removed nothing about what an entry must be.

1. **No assumed level.** `assumed` admits only `ratio`, `count`, `period`, `physical-unit` and
   `hour`. A currency or index dimension with `assumed` provenance fails the build.
2. **No assumed region scope.** An `assumed` entry with `region:r` scope fails the build, **with no
   exception form**. "Region 3 is smaller because region 3 is smaller" is unwritable.
3. **A derived expression is evaluated and dimension-checked**, not trusted. An expression that
   does not evaluate, or whose dimensions do not agree, fails the build.
4. **A structural entry names one of sixteen definitional identities.** A seventeenth is an ADR.
5. **An assumed entry carries a bracket**, and the value lies inside it. Widening a bracket is a
   diff a reviewer sees; leaving it is not.
6. **A literal in a derived expression is drawn from `{0, 1, −1, 2}`.** Anything else is a value
   wearing an expression's clothes.

Plus: a justification citing an external source, a proper noun or a URL fails the lint. It catches
the careless, not the determined, and that is what it claims.

Plus a seventh, which the previous edition owed and did not have: **an `Entry` that no declared system
reads fails the build.** This is the other half of the defect recounted below — the capital constraints
that would have forbidden those loans "sat correctly declared and unread", and nothing noticed.

#### The count is published, not capped

**There is no budget (D3).** The previous edition carried ≤ 130 entries of which ≤ 80 `assumed`, and the
cap did real damage: §13.1.2 records a technology reduced from twenty-seven production sub-units to one
because eighty-one exceeded eighty. **A rule that answers a modelling question by counting rows is not
serving M2**, and a cap reached during the economics — where most of the assumptions live — would answer
every remaining question the same way.

What replaces it is **visibility and pressure**:

- the **assumed count, the total, the placeholder count and the trend of each** are published on the
  surface and carried in the run manifest, so no result can be quoted without its assumption count;
- **every entry names the mechanism it buys**, and a review that cannot say what a number buys deletes it;
- **two entries that could be one are one.** §7.3's relational table is the worked example: twenty-one
  answers per instrument type expressed as a base weight times a per-regime severity, about nine entries,
  and the compression says something the flat table does not;
- the counts are a **standing agenda item**, and the direction is downward. M3 is a direction, not a line.

**A capacity is not a model assumption.** §12.1's arena sizes are `structural` entries carrying their
arithmetic, they answer no question about the world, and they are counted separately and are unreadable
by any agent, valuation or economic system. Counting forty engineering sizes against a figure whose
stated purpose is measuring how much of the world was *chosen* was always going to mislead.

#### The check has a subject, and it is the registry

**A function that returns a number is not an `Entry`, so nothing above reaches a valuation.** That
gap is not hypothetical: it is how a lending rate — a drawn trait times a depreciation rate —
passed every check in a build and settled twenty-four thousand loans at a rate no agent had chosen,
while the capital constraints that forbade those loans sat correctly declared and unread.

**A3 therefore has a second subject.** A reservation level is a **branded type minted in one
module**, against a declared source from §13.1's closed set, and a tier-2 check pins the mint to
that module. It cannot make the declaration true — declaring the wrong source compiles — but it
removes the option of not making one, which is how the defect actually happened.

The general rule: **a number that reaches a decision either cites a section or is an `Entry`, and
there is no third case.**

#### A4's structural half

A4 is not enforced by reading anything. It is enforced by four things being unwritable:

- the entity → (region, sector) mapping is **total, with no `external` member**;
- every `Claim` asset carries a **non-null issuer**, checked at `register`, and **there is no issuer
  sentinel meaning "outside the model"**;
- counter-accounts are `Real`-only, so none can mint a claim;
- the reason-code → flow-class mapping is total, so an unclassified flow does not compile.

---

### 16.2 Error policy

| Category | Examples | What happens |
|---|---|---|
| **Defect** | a conserved column written twice, a counter-account reversed, a claim-asset sum written holder-outer, reaching period 520 without a gate pass | **raise immediately with full context.** Never a default value, never a logged warning |
| **Modelled outcome** | insolvency, a failed auction, an unfilled order, a missing price, a rationed trade line | an explicit value in the domain type. Callers must handle it and the compiler enforces that they do |
| **Environmental** | save unreadable, platform capability absent, insufficient memory | reported to `surface`; the engine does not continue in a degraded state |
| **Reader defect** | the closure panel showing a non-zero world total | raise, and name the three candidates. **It is never a report about the world** |

**A bug converted into a plausible number is the failure this design is organised against.**

### 16.3 Observability

The period trace records **rows touched per position**, which is deterministic and is a declared
series, and **duration**, which is `NonDeterministic<Nanoseconds>` in a 128 kB buffer that is never
saved and never digested. The wall clock exists in `composition/host` alone.

The required readings, none of which the surface computes: the closure panel; the counter-account
flow series; the registry's placeholder, `assumed` and total counts; the journal ring's high-water
mark; the retirement census; and the residency series asserted against §3.4 nightly.

### 16.4 Configuration

Configuration is passed in at construction. **Nothing below `composition` reads the environment**, and
nothing below it can: the crates that would do so are not dependencies of the crates that must not.

---

## 17. Standards

**Crates and modules.** One concept per module, one layer per crate (§4). No module-scope mutable state
— there are no statics, so two worlds in one process share nothing. Every crate carries
`#![forbid(unsafe_code)]` except the single named arena seam, which is small, reviewed and has its
safety argument written down. Warnings are errors; the lint set is checked in and a change to it is a
diff a reviewer sees. **`domain`, `world` and `ledger` carry zero third-party dependencies.**

*Most of what this section used to contain is gone, and that is D2's doing.* The previous edition needed
`strict` as a build gate and lints with empty exemption lists against `any`, non-null assertions and
casts to branded types, because its language erased those distinctions before the program ran. A newtype
with a private field is not castable, a private field is not borrowable, and a crate that is not a
dependency cannot be named. **What was a permanent enforcement burden is now the default state of the
program**, and the enforcement effort is freed for the three prohibitions §11 marks as real work.

**Every boundary that leaves the type system has exactly one construction site and one parser** — the
foreign-function interface to the user interface, the save file, the host shim, configuration — with the
parser generated from the same schema as the columns. **A value entering the engine any other way is a
build failure, not a runtime check.**

**The guarantees survive into the running program** (D2). The previous edition had to concede the
opposite — "at runtime a branded identifier is a number and a handle is an object" — and registered it as
R18, the largest risk in the register. A newtype's private field, a crate's absent dependency and an
unconstructable capability type are all present at runtime, so the residue R18 described is gone rather
than managed.

**Naming.** A reader that returns a magnitude is named for the magnitude; a reader that returns a
signed balance is named for the balance. Units are in the name where two units could be confused.

**17.4 Definition of done for a system.** Its specification is written first; its manifest
declares exactly what it reads and writes; its capabilities are minted and no wider; its numbers are
registry entries; its outputs are declared series; it has a conformance case or a stated reason it
needs none; and this document still describes what it does.

## 18. Extension

**An instrument is data; a behaviour is code. New instruments are nearly free; new behaviours cost,
and should.**

The **change-cost table is normative**: it is the contract, the documentation build fails if it and
A2's prose disagree, a change that does not fit it is an architecture gap rather than an exception,
and it may not drop a column to fit a screen. The rows that matter most:

| Change | Cost |
|---|---|
| New instrument **type** | 1 vocabulary entry, 1 intrinsic row of 13 answers, 1 relational row × 7 regimes, 0–1 market adapter. **0 agent edits**, unless it changes what an agent must satisfy |
| New instrument **issue** | 1 `instruments` row, 0–1 `schedules` row, 0–n option rows, all by `register`. **Nothing else, anywhere** |
| New **fungible claim** class | as a new type, plus **0 ledger changes** — issuance is a `move` from the issuer |
| New **line** | **1 registry row and no code** |
| New **venue** | 1 adapter, 1 system, 1 manifest |
| New **agent type** | 1 module in `agents` + 1 row in the §8.4 mapping |
| New **conservation law** | 1 counter-account registry row with exactly one owner, 1 minted capability, **0 new operations** |
| New **option family** | 1 value + 1 bit per type, 1 terms table, 1 system owning the event test **and its index** |
| New **question** about instruments | 1 column × every type, and it breaks every type until each is decided |
| New **regime** | 1 column × every type, and it must clear the two-cell separation check |
| **A tenth ledger operation** | **ADR**, a row in this table, a case in §15.1, a restatement of §6.1a |
| **A fifth region** | **ADR before it is anything else.** It reopens A4 |


---


---

## 19. Phases and gates

**This section states the gates and their evidence. The build plan itself lives in `IMPLEMENTATION.md`**,
which decomposes the work into named milestones with entry and exit criteria, and which supersedes the
phase table below wherever the two differ. The phase table is kept because the gates are cited from
elsewhere in this document; it is a summary, not the plan.

**Phases have entry and exit criteria. A phase is not finished because time has passed.**

**Three gates, two of which carry continue/stop authority.**

| Gate | Sits at | Decides | Evidence it has |
|---|---|---|---|
| **G1** | end of Phase 1a | **continue or stop on the substrate.** Do A1 and A4 hold structurally, and does the write model fit the device? | the conformance suite on a world small enough to reason about; the journalling measurement on the target device; the platform probe |
| **G2** | end of Phase 2 | **scope and workload.** The first real counts; instrument and price scope fixed for Phase 3 | first real row counts, schedule volumes, first clearing. No agents, therefore no economics |
| **G3** | end of Phase 3 | **continue or stop on the economics.** Does a closed loop reach tick 260 from primitives alone, with sectoral net lending exactly zero and a stable digest? | the first end-to-end period, the first agent behaviour, the first long-run benchmark |

**G1 splits in two, because Phase 1a as previously scoped could not pass its own exit criterion.** Of
§15.1's twenty cases, roughly five are reachable with storage, doors and the journal alone; the rest need
an instrument to exist. §15.1 half-admits this — "it needs a sovereign instrument type to exist in
Phase 1a rather than Phase 2" — and the phase table was left unchanged. **G1a** gates on the substrate
cases and carries the continue/stop authority on the write model; **G1b** gates on the remainder once a
minimum instrument kit lands, and carries the authority on the instrument model. Holding one undivided
G1 puts the substrate stop decision behind months of instrument work, which is the ceremony the next
paragraph warns against.

**A gate is worth what it saves, and a gate placed after the saving has already been spent is a
ceremony.** Everything that can falsify the substrate is measurable early; everything after that is
months. G3 sits where it does for the mirror reason: the economics question has no evidence at all
before an end-to-end period exists, and a gate cannot sit earlier than its evidence.

**G3's burn-in limb is taken on the panel that exists.** Most of §15.3's 42 series need mechanisms
that arrive after G3, so the gate is instrumented, run on the series that exist, and the absent ones
are named. Waiting for the full panel would put the gate after the whole spend.

*Residual, admitted:* a project that passes G1 and G2 can still fail at G3 having spent everything
but Phase 4, and no gate protects against that.

#### The phases

| Phase | Contents | Exits when |
|---|---|---|
| **1a** — the substrate experiment | typed column storage, the three doors, the counter-account registry, the identity spaces, the row lifecycle, the journal, the conformance suite, the device probe, the benchmark harness. **Plus the minimum instrument kit** — see the note below | the substrate conformance cases pass; the operation-cost measurement exists on the target device; the probe reports a real ceiling |
| **1b** — the substrate proper | code generation and the generated composition root; the state differ and the row inspector; persistence as a file format with schema identifier and migration; the batch forms of every operation; the remaining posted-column families; the observation store | the root wires the engine with no hand-written line outside the 120-line shim; a save round-trips bit-identically; the batch forms exist and are reviewed alongside the single forms; the standing benchmarks run with published noise floors |
| **2** — instruments, obligations and prices | the two facts tables; the seven option-terms tables; the due-tick index; `amend` for all eight mechanisms; the price table with epoch-typed reads; the clearing interface; the opening world built by ledger operations from primitives | an instrument can be issued, held, priced, pay a full irregular schedule to multiple holder classes with no holder-specific code, and be amended by a default and by a prepayment; **the payment walk contains no `InstrumentTypeCode`**; the opening world generates from primitives alone and reaches tick 260 with no seeded price and no seeded term; the registry instantiates 37 venues under the 4,096-line cap |
| **3** — agents and the period loop | systems and manifests; capability minting; the period order as a committed list; **agents as the five declarations**; decision staggering with every cohort re-planning at tick 0 | a period runs end to end with two agent classes; every position has a system or a stated reason it has none; the digest is stable and equal at 1 worker and 64 shards; the manifest/order consistency check passes; **sectoral net lending is exactly zero every tick**; the burn-in gate is instrumented and a `burnInPeriod` is recorded |
| **4** — economic content | one system at a time, specification first, sequenced by dependency: **money and settlement, then credit, then equity, then the public sector, then the remaining sectors, then cross-region trade and FX** | per system, §17.4's definition of done |

**Phase 1b has a blocking entry decision.** The engine must be correct and within budget
single-threaded, while the memory model was drafted assuming one shared buffer and four workers.
Those cannot both stand. Phase 1a reports the capability; **1b decides it first, or it builds on an
assumption.**

**No duration is claimed beyond Phase 1a.** Later phases exit on criteria.

---

## 20. Risks

The register is twenty-one entries; these are the ones whose mitigations constrain design choices
elsewhere, and each mitigation is a mechanism rather than a promise.

| | Risk | Mitigation |
|---|---|---|
| **R6** | a golden digest re-baselined to make a test pass, hiding a regression | a digest change must be explained by the intended behaviour change, and the explanation must name whether retirement order or shard count moved |
| **R10** | an observed figure typed into the seed because it looked reasonable | §16.1's six rules, the budget, and the placeholder count on the dashboard |
| **R14** | a settled value drifting rather than being superseded — a cap raised, a regime added, inside a commit about something else | every register entry carries a named mechanical guard, and a decision that cannot be given one may not be entered. A diff touching a registered value without an ADR is refused |
| **R15** | the generated composition root becoming a place to put logic | it is generated, decides nothing, mints nothing writable, and a hand edit fails the build |
| **R16** | a capacity raised in a commit about something else, so N4 becomes a preference | capacities are `structural` entries carrying their arithmetic; exhaustion raises; a change is an ADR |
| **R17** | the directory indirection bypassed for speed, so a stale slot re-points at a different subject | slots do not cross a module boundary and have no integer-yielding method |
| **R18** | ~~every guarantee here is compile-time, in a language that erases types at runtime~~ **Largely retired by D2.** Newtypes with private fields, crate-level privacy and unconstructable capability types are present at runtime, so a value cannot be forged from an integer and a layer cannot be reached that is not a dependency | what remains: the foreign-function boundary to the user interface and the save file, each with one construction site and one generated parser, and the single named arena seam with its safety argument written down |
| **R19** | an unmodelled counterparty admitted under scope pressure — a residual holder, an `external` member | A4's four structural parts. **A4 fails and A1 still passes: the books balance perfectly, which is exactly why nothing sees it** |
| **R20** | a phenomenon that genuinely needs an outside faked inside — a firm invented to produce an import, a transfer standing in for migration | the admissible responses are enumerated and neither is a fake: **model the source**, or **declare it out of scope and say so on the surface.** A placeholder is registered and counted; **an invented producer is not a placeholder, it is a mechanism nobody specified** |
| **R21** | closure failing at runtime, and a world-summing pass proposed as the pragmatic fix | **tier 4 is empty by design.** If closure fails, the failure is in a door precondition, a total mapping or a single-writer check. A world-summing pass is an ADR against A1 and A4 and is refused as a whole |

---

# Appendices

---

## Appendix A — The decision register

**Every entry carries a named mechanical guard, and a decision that cannot be given one may not be
entered.** A diff touching a registered value without an ADR is refused. Values here are current;
what they replaced is in Appendix B.

| # | Decision | Current value | Guard |
|---|---|---|---|
| 0 | **The six model rules** | M1–M6 (§1.1). They are the project's requirements; everything mechanical serves them | a mechanism conflicting with a rule is restated, not the rule |
| 1 | Language, runtime and delivery | **Rust, one crate per layer, delivered as an Android application: native engine plus a thin user interface** (D2) | the crate graph; `#![forbid(unsafe_code)]`; private fields on newtypes and conserved columns |
| 2 | Integer width, units, numéraire | **`i64` conserved quantities, overflow panics**; one unit class per asset class; `S = 2 × 10¹¹`, `structural`, upper bracket owed (§5.3) | the type; the unit-class table; the registry check |
| 3 | Rounding | three rules and no fourth | a rounding decision outside them is a review-checklist item |
| 4 | The write model | three doors, nine operations | typed handles; a tenth is an ADR |
| 5 | Rehypothecation depth | 3 | the pledge door |
| 6 | Counter-accounts | four families, four owners, ten pairs per region, `Real` only | the class law at the door; the minted capability |
| 7 | Workload | weekly ticks, 1,560-tick runs, burn-in floor 260 / ceiling 520; 550,638 entities; 37 venues; 3,119,665 calls a tick (derivation owed, §3.4) | N3 as a requirement; N2a, N2b, N4 as nightly targets that bend before the model does (D1) |
| 8 | Journal retention | two ticks, 7,200,000 rows in two segments, 345.6 MB | exhaustion raises; the high-water series |
| 9 | Observation store | fourteen families, 624 declared, hard cap 2,048 under sub-caps | the sub-cap at declaration |
| 10 | Intrinsic questions | thirteen, at two levels | a missing answer does not compile |
| 11 | Regimes | seven, three relational questions, 21 answers per type | declared count, column distinctness, two-cell separation |
| 12 | Amendment | eight mechanisms, five owners, no general handle | per-mechanism minted capabilities |
| 13 | Opening primitives | §13.1's list. **No cap; the assumed count and its trend are published and pushed down** (D3) | the A3 build check's seven rules; the published census; the dead-entry check |
| 14 | Budget allocation | two stages, per line, no intra-stage reallocation | the committed order |
| 15 | Simultaneity | per fact, not per position; one named crossing | the tick stamp; the manifest/order check |
| 16 | Layering | eleven layers, generated composition root, 120-line shim | the import check; a hand edit fails the build |
| 17 | Row lifecycle | permanent identifiers, impermanent residency, quiescence-gated retirement | no retirement sweep may exist; slots have no integer-yielding method |
| 18 | Venues and lines | 37 venues, 190 structural lines, 1,276 instantiated, cap 4,096 | the cap at venue registration |
| 19 | Region and currency | one region per venue, FX the sole exception, no world price | the venue registry |
| 20 | Submission shapes | two: schedule and price-taking, 64 log-spaced buckets | one clearing interface |
| 21 | Agent inventory | nine classes, one total mapping to (regime, cadence, count); **five declarations, total per class** | a class naming no regime, or declaring four items, does not compile |
| 22 | Agent state budget | 160 B household, 256 B SME, 1,024 B institution, 95.6 MB | asserted at schema build |
| 22a | Holdings slot | **24 B, field list published in §3.4**; encumbrance derived from lien rows, not stored | asserted at schema build |
| 23 | Staggering | 13 / 4 / 1; phase from a dedicated stream, never `id mod C`; six triggers, one minted handle each | the manifest fails the N3 check if a trigger's index is a scan |
| 24 | Decision outputs | `plans` and `intents` as world tables, ≈121 MB fixed at init | no decision system allocates a result object |
| 25 | The acceleration seam | W1 = 8–10, W2 = 12–13; 64 shards; sequential ledger; one construction site | a second boundary is an ADR |
| 26 | The period trace | rows touched is a series; duration is `NonDeterministic` and never digested | the type |
| 27 | The A3 build check | eight fields, **seven rules**; `assumed` never a level and never region-scoped; an unread entry fails | the build |
| 28 | Structural asymmetry | three axes, `Z`, three orderings, δ = 0.35 / 0.20 / 0.15; **rank 1 takes the smallest loading**; every per-region value `derived` | no per-region `assumed` entry compiles |
| 29 | Policy rate | difference rule with a Wicksellian anchor; `a, b, c, K` = 1.5, 0.5, 0.10, 104; `π* = 0` structural; no rate before tick 104 | the registry; the posted-column owner |
| 30 | The burn-in gate | 42 series, W = 104, E = 16, four tests, `burnInPeriod ∈ [260, 520]` | reaching 520 without a pass is a defect |
| 31 | A4 enforcement | total mappings, non-null issuers, `Real`-only counter-accounts, no sentinel | four things unwritable |
| 32 | Cross-region closure | no rest of world; four exact per-currency identities; the matrix is a report | the identities are integer; the total is displayed as a literal zero |

### How a decision is superseded

1. **It is named.** The register entry changes, and Appendix B records what it was, what it is, and
   why. A value that changed without being named is the failure this process exists to catch.
2. **Its guard changes with it.** An entry whose guard no longer guards the new value is not
   superseded, it is broken.
3. **Everything derived from it is re-derived**, not assumed to still hold. If a workload figure
   moves, the budgets computed from it move.
4. **A diff touching a registered value without an ADR is refused.**

**Three decisions carry a review point** rather than waiting to be challenged: the workload, at G2,
when the first real counts exist; the agent state budget, at the end of Phase 3, because the
institutional cap is asserted before the institutional systems exist; and cross-region trade, when
it goes live, because the cost of forgoing intra-stage reallocation is a cross-currency cost and is
not observable before then.

---

## Appendix B — Supersession log

**A value that changed without being named is the failure the register exists to catch.** This log
records what this edition changed and why. It does not record what earlier editions changed; that
is what version control is for.

#### The four founding decisions (D1–D4)

*Taken by the project owner, and the reason this is version 6.0. Everything below them in this log is a
consequence rather than a separate decision.*

| | Decision | Supersedes | Why |
|---|---|---|---|
| **D1** | **The model wins; the delivery target bends.** 550,638 deciding entities and weekly ticks are held whatever they cost. N2a, N2b and N4 become measured targets; N1, N3, N5, N6 stay requirements | §3.3's undifferentiated budget table; §12.1's "two budgets" | A performance budget that can stop the project is a budget that will be met by shedding agents or coarsening cadences, and both are retreats from M1. The cheapest way to hit a tick budget is always to have less model. Making the target the flexible side removes the incentive permanently |
| **D2** | **Rust, one crate per layer, delivered as an Android application** — a native engine and a thin user interface | Appendix A #1 (TypeScript, typed columns over `ArrayBuffer`, `strict` as a build gate) and the lint apparatus of §11 and §17 that existed to compensate for it | The previous language erased its own guarantees before the program ran, which the register recorded as R18, its largest risk. Crate privacy, newtypes with private fields and unconstructable capability types are present at runtime. It also makes §12.2's 96.5 ns per exchange plausible rather than optimistic, and an application rather than a page removes the memory ceiling the previous target implied |
| **D3** | **Minimise priors and publish the count; no cap.** The ≤ 130 / ≤ 80 budget is withdrawn. The seven provenance rules stay build failures | §3.2's A3 budget clause; §16.1's budget; Appendix A #13 | The cap decided modelling questions by arithmetic. §13.1.2 is the recorded instance: a technology collapsed from twenty-seven production sub-units to one because 81 > 80. A rule that answers "how much structure does this economy have" by counting rows is not serving M2. The count is now a published figure that review pushes down, which is pressure without a cliff |
| **D4** | **Device measurement is a cross-compiled probe published to releases**; the owner installs it, runs it, and returns one JSON document | the implied device lab | The evidence needed is a handful of numbers from one real device, not a phone farm. The measurement is the input to §12's targets and to the §7.5 row-width question |

**What D3 does not change.** The provenance rules are untouched and remain build failures: no assumed
level, no assumed region scope, a bracket on every assumed entry, dimension-checked derived expressions,
literals drawn from `{0, 1, −1, 2}`, a structural entry naming a definitional identity, and an entry
nothing reads failing the build. **The cap was never what made A3 true; the rules were.**

#### Consequences and other changes in this edition

| What changed | To | Why |
|---|---|---|
| A1's part 2 | **the conserved column is private to the `ledger` crate**, replacing "exactly one writer in the whole source tree, checked in CI" | The old form was false. Relocation, the zeroed-entry tail shift and slot canonicalisation all write the quantity column; a CI check would have needed exemptions on its first run. The crate boundary is true, is the compiler's, and covers writers nobody has thought of |
| The holdings slot | **24 B, with its field list published** | 20 B could not hold what §6.11 requires — asset, quantity and integral exhaust it with no tick column — so it was an unsourced constant the rest of the document contradicted |
| §11's transcendental ban | **widened from samplers to any path reaching a digested value** | §13.4's output gap computed `log(output)` inside the engine and was read by a decision, slipping the letter of a rule it violated in substance. `ŷ` is now a ratio |
| §13.4's `ŷ` | `(output_t − μ_K) / μ_K` | dimensionless, needs no transcendental, and says the same thing about the gap |
| §13.1.2's technology | **reopened** | its stated reason was the assumption cap, and the cap is gone. §9.5's twenty-seven goods lines and one composite sub-unit still contradict each other and the contradiction is now to be settled on the economics |
| N4's 1,488.3 MB | **unresolved, and named as such** | the itemisation was never published and the named components leave roughly 705 MB unaccounted, whose largest term is the instrument row width §7.5 declares unsettled |
| §3.4's identifier census | **unresolved, and named as such** | ≈ 971,000 ever issued against §5.2's implied ≈ 12,450,000 is a factor of thirteen, and it sizes the directory, the digest walk and the save |
| The trailing-statistics system | **owed a position** | §13.4 requires it, §9.4's twenty-one positions do not contain it |
| The institutional census: banks, funds, insurers-and-pension-funds | **56 / 54 / 40** | The seed carried 120 / 240 / 80 while the entity census, the agent inventory and the workload decision carried 56 / 54 / 22 + 18. The layout, the arena sizing, the block widths and the entity budget are all derived from the second set, and it is the set with an argument attached |
| Every published per-region vector | **withdrawn; §13.3's derived table replaces them** | The published household row descended while every other axis-1 primitive in the same table ascended, so one axis was being read in two directions; and no household vector printed anywhere was the formula's output |
| `Z`'s index direction | **rank 1 takes the smallest loading** | The only reading reproducing the axis-1 share row, both multiplier rows and the sign of ρ(P₂,P₃) |
| δ₁'s justification | **withdrawn and owed** | It cited eighteen banks in the smallest region, against a census in which the smallest region has eight. The value stands; the argument must be remade |
| Position 2's endowables | **land only** | The counter-account class law permits `Endowment:` two unit classes, and dwellings and capital are produced, not endowed |
| The employment settlement | **two rows: hours to the employer batched per (employer, class), then the wage per contract** | The one-row form named three parties and `exchange` takes two. The flow model already budgeted it as two |
| The capital-constraint numerator | **regulatory capital, not net worth** | Under R-1 an institution that issued equity holds `+X` and `−X`, so its net worth is identically zero and no positive floor was satisfiable by anyone |
| The technology primitive | **one composite sub-unit**, intensities `derived`, one assumed hour-productivity ratio | Twenty-seven sub-units at three coefficients is 81 `assumed` entries against a cap of 80 |
| The period-0 valuation rule | **a dimensionless primitive × a numéraire-derived anchor** | The sanctioned primitives are `ratio`-dimensioned and a reservation is money; the numéraire is the only thing that can carry the unit |
| A3's subject | **extended from the registry to the reservation mint** | A function that returns a number is not an `Entry`, so nothing in the build check reached a valuation |
| §8.1's five declarations | **a total mapping, compiler-enforced** | The register requires every decision to carry a mechanical guard and this one's guard was review |
| The instrument hold count | **the transition to zero is noticed in the payment walk's re-bucketing** | An obligation that finishes paying is decided by nobody, so nothing stamped it and nothing queued it |
| Position 21's scope | **obligation compaction only** | Retirement is not a position: it is issued at 1, 18 and 19 |

**Unresolved, and named rather than settled:** §7.5's 44-byte instrument row against §3.4.4's
148-byte row. They have different memory budgets, different save formats and different relocation
costs. It is settled by measurement on the target device and is an entry criterion for Phase 2.

**Owed**, and this list is the honest measure of what the document does not yet contain:

*Model content, which is most of it.* A **production section**, of which position 6's row and five
sentences are the whole. The **five declarations of §8.1 for all nine agent classes** — mandate, regime,
constraints, valuation, funding policy — which are a shape with no content for any class: no consumption
rule, no labour supply rule, no firm pricing rule, no credit underwriting rule, no portfolio rule, no
default test, no bank capital values, no cure windows. A **§17.4 specification per committed position**,
of which roughly thirty are owed and one exists. The **relational table's twenty-one values**. The
**instrument type vocabulary**, which §18 charges one entry against a section that does not exist.
**None of this is a detail of implementation. It is the economics, and it is the subject of the title.**

*Numbers and rules.* δ₁'s justification. The split of one insurer-and-pension primitive into two agent
classes. §7.5's 44-byte against 148-byte instrument row. N4's itemisation. §3.4's identifier census and
its operation-count derivation. The numéraire's upper bracket. The trailing-statistics position. The
period-0 grid-placement rule for every venue family other than labour — §13.1.1 supplies one anchor and
the other nine or more lines have none, so a first clearing would raise a grid defect on almost every
line. How a fill at positions 13 and 14 becomes an instrument at position 19. The derivation rule for
Q13 `DerivedMark` instruments, read at position 17.

*Cross-references that resolve to nothing*, each of which is cited as though it carried content:
**§3.4.4** (the 148-byte instrument row), **§9.6.1** (the grid the numéraire argument exists to place),
**§15.3.4** (the sensitivity
sweep), **§17.4** (the definition of done, cited from §8 and Appendix B), **§21.3** (the fourth-axis
ADR) and **D-7** (the shard-by-row-count rule). (§9.7.3 and §9.7.5, cited as A4's discharge, are
repointed to §9.7 in this edition.) A documentation check resolving every reference against the heading
set is a day's work and would have caught all of them.

*Process.* There is **no ADR template, numbering or register**, though this document requires an ADR
about twenty times and Appendix A's supersession procedure depends on one existing.

---

## Appendix C — Failure modes this design is hardened against

When something goes wrong it is usually one of these wearing a costume.

| # | Failure | Addressed by |
|---|---|---|
| 1 | one fact in two representations, with nothing forcing them to agree | one storage location per fact; derived readers rather than stored copies |
| 2 | a plug: a balancing entry with no owner | four counter-account families, four owners, monotone and scoped |
| 3 | a counter-account reversed into a plug | source families non-increasing, sink families non-decreasing, at the door |
| 4 | an agent caching a per-class total, re-creating the N×M matrix silently | no stored aggregates on an agent, ever |
| 5 | a branch on instrument type inside an agent | agents receive facts, never a type code |
| 6 | an iteration cap standing in for a missing mechanism | one bounded rebase; a second failure is a defect |
| 7 | a level typed into the seed because it looked reasonable | the A3 build check and the assumption budget |
| **8** | **a check that cannot fail** | a check whose two sides come from one source is not shipped; the closure total is displayed as a literal zero rather than computed |
| 9 | an untyped bag of parameters | seven typed terms tables; no JSON blob anywhere |
| 10 | state rebuilt from a hand-written field list, silently dropping what was added later | the save is a columnar dump enumerated by the schema |
| 11 | a stale slot re-pointing at a different subject | slots do not cross a module boundary |
| 12 | a structure growing with run length | one such structure is permitted, and a second is a defect |
| 13 | a cost proportional to stock where it could be proportional to activity | N3, with two limbs and a nightly measurement |
| **19** | **an unmodelled counterparty, admitted as plumbing** | A4's four unwritable things. **The books balance perfectly, which is why nothing else would see it** |
| 20 | a phenomenon needing an outside, faked inside | model it, or declare it out of scope. An invented producer is a mechanism nobody specified |
| 21 | a world-summing pass added as a diagnostic | tier 4 is empty by design and a proposal to open it is an ADR |
| 22 | a bug converted into a plausible number | defects raise; modelled outcomes are values in the type |

---

## Appendix D — Glossary

| Term | Meaning |
|---|---|
| **claim** | a financial asset. Exists only as its issuer's negative balance |
| **counter-account** | an ordinary holder whose balance is the negation of what it has sourced. Real units only |
| **conserved column** | the holdings quantity column. One writer in the whole source tree |
| **defect** | a violation of a structural claim. Raises. Distinct from a modelled outcome |
| **derived** | a registry value computed from other entries by a dimension-checked expression |
| **line** | data: one priced thing on a venue |
| **modelled outcome** | insolvency, an unfilled order, a missing price. A value in the type, not an error |
| **par** | a claim's value where no venue prices it. Definitional arithmetic |
| **placeholder** | a registered stand-in for a missing mechanism, counted on the dashboard, with a deleting ADR |
| **position** | one of the 21 stable names in the committed period order |
| **quiescent** | terminal status and zero hold count. The precondition for ending residency |
| **reservation** | the level at which a participant is indifferent. Minted in one module against a declared source |
| **structural** | a registry value that is definitional arithmetic, naming one of sixteen identities |
| **tick** | one period; one simulated week |
| **venue** | an adapter. One region, one currency, except FX |
