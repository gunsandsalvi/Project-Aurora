# Project Aurora
## Economic Simulation Engine — Architecture, Standards and Settled Decisions

**Version 3.0** · 5 September 2026 · Status: **Baselined. Approved for Phase 1a.**

---

### Document control

| Field | Value |
|---|---|
| **Purpose** | The single governing document for Project Aurora: an agent-based economic simulation engine built from scratch. It defines the architecture, the engineering standards, the delivery plan, and every decision that had to be settled before work could start. |
| **Audience** | Engineers who will build it, and reviewers who will judge whether the plan is sound. No prior familiarity with any existing system is assumed or required. |
| **Status** | Baselined. Sections 3–20 are the design. Section 21 records the settled decisions with their rationale. Nothing in this document is open; changing any of it is a supersession under §21.3. |
| **Scope of authority** | This document governs structure, standards, and the settled parameters in §21. It fixes how mechanisms are expressed and where numbers may come from (A3). It does not specify which economic mechanisms exist; that belongs in per-system specifications (§17.4). |
| **Independence** | Greenfield. No code is carried over from any predecessor, by copy, port or reference. The reasoning, and the conditions under which it was the wrong call, are in Appendix D. |
| **Revision history** | Appendix C. |

---

### Contents

1. Executive summary
2. Scope
3. Requirements — functional, architectural (A1–A3), non-functional, target workload
4. Architecture overview — layers, the relocation rule, general principles, the surface rule
5. The state model — tables, identity, quantities, prices
6. The ledger — one table, one verb, counter-accounts, encumbrance, journal, wind-up
7. Instruments as data — intrinsic facts, relational facts, obligations and amendment
8. Agents as policy
9. Markets — clearing, budgets, simultaneity
10. Time, sequencing and capabilities
11. Determinism
12. Performance and concurrency
13. Persistence and the seed
14. Observation and reporting
15. Correctness strategy
16. Cross-cutting concerns — parameter provenance and A3, errors, observability, configuration
17. Engineering standards
18. Extension playbooks
19. Delivery plan
20. Risks
21. Settled decisions — the fifteen, with rationale, and how to supersede one
- Appendix A — failure modes this design is hardened against
- Appendix B — glossary
- Appendix C — revision history
- Appendix D — decision record: greenfield rather than continued remediation

---

## 1. Executive summary

We are building a discrete-period simulation of an economy: firms that produce and invest, banks
that lend, funds and insurers that allocate, households that work and consume, governments that tax
and spend, and the instruments and markets that connect them. It must run interactively on consumer
hardware, produce reproducible results, and — the point of this document — remain extensible for
years without periodic structural rewrites.

Three architectural requirements drive every decision:

- **A1 — Conservation is structural.** Value cannot be created or destroyed accidentally, because no
  code path exists that could do it. There is no reconciliation pass, no periodic identity suite, no
  audit harness. Correctness of the books is a property of the design.
- **A2 — Coupling is additive, not multiplicative.** With **N** instrument types and **M** agent
  types, introducing either costs **N + M** knowledge, never **N × M**. Adding a covered bond must
  not require edits to the asset manager, the insurer, the household or the wind-up waterfall.
  Adding a new class of agent must not require edits to any instrument.
- **A3 — No exogenous calibration.** Every level in the world is an outcome of the simulation's own
  evolution from declared primitives. No price, aggregate, spread, revenue or index is seeded from,
  fitted to, or pinned against an observed real-world figure. This is the requirement most easily
  eroded one literal at a time, so it is stated as an architectural requirement with structural
  enforcement (§16.1), not as a modelling preference.

Everything else — the layering, the storage model, the type discipline, the delivery order — is
justified by one of those three, and each section states which.

The plan is four phases, with Phase 1 split into an experiment and a build. Phase 1a proves A1 on a
toy world in a fortnight and carries a stated kill criterion. Phase 1b completes the substrate.
Phases 2–4 build economic content one system at a time, each preceded by a written specification.
The economics is the long pole and is deliberately sequenced last.

The fifteen decisions that had to be made before any of that could start — language, integer widths,
rounding, the verb set, the counter-account registry, the workload, the opening primitives, and the
rest — are settled in §21 and quoted where they bite throughout the document. There are no `TBD`s.
A number in this document is either a settled decision with a rationale or an output of the model.

---

## 2. Scope

### 2.1 In scope

A simulation engine covering: entities and their balance sheets; instruments and their obligations;
price formation in multiple markets; production, consumption, employment and investment; credit,
default and wind-up; government and central-bank policy; a declared set of observed series; and a
read-only presentation surface.

### 2.2 Non-goals

- **Not a general-purpose framework.** It is one engine for one model. Generality is added when a
  second use case actually appears, not in anticipation.
- **Not a distributed system.** It runs in one process on one device. Parallelism, if any, is
  within-process and is a performance decision (§12), not an architectural one.
- **Not economically prescriptive.** This document fixes how mechanisms are expressed, not which
  mechanisms exist.
- **Not a forecasting tool, and not a calibrated one.** Reproducibility and internal consistency are
  requirements. Correspondence to any real economy is neither a guarantee nor, under A3, a
  permissible target of parameter choice.

---

## 3. Requirements

### 3.1 Functional

| ID | Requirement |
|---|---|
| F1 | Advance the world by one discrete period, deterministically, from a seed. |
| F2 | Hold arbitrary quantities of arbitrary asset classes against arbitrary holders. |
| F3 | Form prices in venues where multiple participants meet, for every priced thing. |
| F4 | Represent obligations with arbitrary schedules — regular, irregular, amortising, indexed, and contingent on a declared event — including schedules that change during the instrument's life. |
| F5 | Represent claims over assets held by another party (pledge, lien, collateral), including chains. |
| F6 | Resolve insolvency: rank claims, resolve collateral, distribute an estate, extinguish what cannot be paid. |
| F7 | Persist and restore world state exactly. |
| F8 | Explain any change: for a given quantity and period, what moved it, from whom, and why. |
| F9 | Produce a declared set of period-indexed series for reporting, without any engine decision reading them back. |

### 3.2 Architectural requirements

Stated so they can be verified mechanically, not admired.

**A1 — Conservation is structural.**
For every conserved asset, the sum of holdings across all holders, counter-accounts included (§6.2),
is invariant under every operation the engine can perform.

*Structural argument:* one writable handle to the quantity column, obtainable only inside the ledger;
one verb that debits and credits in the same statement.
*Mechanical discharge, three parts:* (i) a CI check that the quantity column has exactly one writer
in the whole source tree; (ii) a property test over generated sequences of legal verb calls,
asserting the invariant per asset; (iii) write-door preconditions (§6.4).
*Review protocol, retained:* "name the function that could unbalance it." If one can be named, A1 is
not met. This is a review question, not the verification.

**A2 — Coupling is additive.**
Introducing a new instrument type must require changes confined to: one vocabulary entry, one row of
intrinsic facts, one row of relational facts per regime, its obligation rows, and — only if it trades
somewhere new — one market adapter. Zero edits to any agent model, except where the instrument
genuinely changes what an agent *does*, which is a constraint edit and is declared as such (§18.1).
Introducing a new agent type must require one new module and zero edits to any instrument.
*Verified by:* the change-cost table in §18.4. If a proposed feature breaks it, the architecture has
failed, not the feature.

**A3 — No exogenous calibration.**
No number on a behavioural path may be chosen because it resembles an observed real-world figure.
Every level in the world — prices, output, wages, spreads, index levels, balance-sheet sizes — must
be traceable to either a registered structural primitive or an operation on prior state.

*Test:* pick any number in the world and ask where it came from. Two answers are acceptable: "it is a
registered primitive with a written derivation" and "it is what the model produced." The answer "it
is roughly what it is in reality" is a violation.
*Mechanical discharge:* the parameter registry (§16.1) admits no `measured` or `calibrated`
provenance; registry entries citing a real-world source fail the build; the opening world is
generated by ledger operations from primitives, never assembled from a table of levels (§13).
*Honest limit:* nothing stops someone choosing an `assumed` value because it looks familiar. What A3
buys is that every such choice is a named, owned, reviewable registry entry rather than an anonymous
literal in a constructor. The registry makes the population of such choices countable; that is the
enforcement.
*Consequence to accept:* an uncalibrated opening world is arbitrary, so no output is meaningful until
the model has run a burn-in. The burn-in length is a declared parameter and the surface must not
present pre-burn-in periods as results.

### 3.3 Non-functional

| ID | Requirement | How it is verified |
|---|---|---|
| N1 | **Determinism.** Same seed and same build ⇒ bit-identical state. | Golden-digest test in CI (§11). |
| N2 | **Period budget.** One period completes within the stated wall-clock budget at target scale on the target device (§3.4). | Standing benchmark from Phase 1b (§12.1). |
| N3 | **Flat cost.** Cost per period must not grow more than the stated percentage across a full-length run (§3.4). | Long-run benchmark, nightly. |
| N4 | **Portability.** Runs on the target platform (§3.4) with no capability that platform lacks, within its memory budget. | Platform probe in CI from Phase 1b (§12.4). |
| N5 | **Explainability.** Every change to a conserved quantity is attributable to a recorded instruction. | Structural: the ledger records it (§6.6). |
| N6 | **Reproducible builds of state.** A saved world reloads to a bit-identical state. | Round-trip test (§13). |

N3 deserves emphasis because it is the requirement most often discovered too late. A model whose
books accumulate — contracts, positions, obligations — will get slower every period unless the
per-period work is proportional to *what changes*, not to *what exists*. That is a design decision
(§7.4), not a tuning exercise, and it must be made before the first book grows.

### 3.4 Target workload

The storage model (§5.1) and the performance posture (§12) are justified against a workload, and are
unjustifiable without one. This is the workload. It is settled (§21, decision 7); every performance
claim in this document is a claim about these numbers and no others.

| Parameter | Value | Notes |
|---|---|---|
| Period length | **1 week** | 13 periods to a quarter, 52 to a year. Fine enough for market dynamics, coarse enough that a year fast-forwards in seconds. |
| Full run length | **1,560 periods** (30 years) | N3 is measured over this. |
| Burn-in | **260 periods** (5 years) | Required by A3; no output before period 260 is presented as a result. |
| Regions | **4** | Each with its own currency, government and central bank. |
| Households | **10,000** | Representative agents, not literal persons. |
| Firms | **2,000** | Across 27 production sub-units. |
| Banks / funds / insurers | **40 / 60 / 20** | The balance-sheet-heavy holders. |
| Governments / central banks | **4 / 4** | |
| **Entities, total, steady state** | **≈ 12,130** | Entity-poor, mechanism-rich (§12.2). |
| Instruments live, steady state | **60,000** | Deposits, loans, bonds, equities, fund units. |
| Holdings rows live | **400,000** | Institutions dominate the count, not households. |
| Obligation rows live | **1,200,000** | Drives the §7.4 due-period index. |
| Lien rows live | **80,000** | Depth ≤ 3 (§6.5). |
| Venues cleared per period | **40** | 27 goods, 4 sovereign, 1 corporate credit, 1 equity, 6 FX pairs, 1 money market. |
| Observed series | **≤ 512** | Hard cap (§14). |
| Target device | **A 4 GB-RAM mobile device of the slowest supported generation**, named in the CI device matrix and benchmarked on hardware, not a simulator. | N4. |
| **Period budget (N2)** | **250 ms** at the above scale on the target device | A simulated year in ≈ 13 s. |
| **Flat-cost tolerance (N3)** | **≤ 15%** — period 1,560 may cost no more than 1.15 × period 260 | Measured post-burn-in so growth, not warm-up, is what is being measured. |
| **Peak memory (N4)** | **900 MB**, of which engine state ≤ 600 MB, journal ≤ 60 MB (§21 decision 8), observations ≤ 20 MB (§14) | |

Two consequences worth stating, since they are what the numbers were chosen to produce:

- **Obligations dominate row count**, at roughly three rows for every two holdings. That is why the
  due-period index (§7.4) is a requirement rather than an optimisation: a design that walks the
  obligation stock each period does 1.2 M row visits to pay a few thousand flows.
- **Household count is a modelling decision, not a resolution knob.** Ten thousand representative
  households is chosen so the per-period decision pass fits the budget; raising it is a change to
  §3.4 under §21.3, not a configuration value someone sets at launch.

If any figure here changes by an order of magnitude, §5.1 and §12 are re-argued rather than assumed
to still hold.

---

## 4. Architecture overview

### 4.1 Layers and the dependency rule

`X → Y` means *X may import Y*.

```
runtime  →  system declarations only

systems  →  ledger  →  world  →  domain  →  kernel
   │                     │         ▲          ▲
   └────→  markets ──────┼─────────┘          │
                         └────────────────────┘

surface  →  world read views, observation store, domain      (leaf; imported by nothing)
```

| Layer | Contents | May import |
|---|---|---|
| **kernel** | Storage primitives, typed columns, identifier machinery, quantity types, code generation. Knows no economics. | nothing |
| **domain** | Vocabulary and pure arithmetic. No state. One declaration per concept. | kernel |
| **world** | One module per table: entities, instruments, obligations, holdings, liens, prices, journal, observations. Schema plus generated read views. | kernel, domain |
| **ledger** | The only module that can obtain a writable view of holdings, liens or obligations. Exposes verbs. | kernel, domain, world |
| **markets** | Price formation. Deliberately independent of `world` so it can be tested and optimised alone. | kernel, domain |
| **systems** | The model's work, one module each. Pure over (read views, ledger handle, row range). | kernel, domain, world, ledger, markets |
| **runtime** | Period loop, ordering, capability minting, sharding, profiling. | system *declarations* only |
| **surface** | Presentation, reporting, export. Read-only, and computes nothing (§4.4). | world read views, observations, domain |

The rule is enforced by an import-boundary check with an **empty exemption list**. An exemption
list that is allowed to have one entry will have forty.

### 4.2 The rule that protects the layering

*A system is never relocated into a lower layer for performance.*

This is the single most common way a clean layering degrades. Something needs fast access to
storage, so it moves into the storage package; it brings its callers' dependencies with it; the
storage package acquires an upward import; and within a year the two packages are one package with a
cycle. When a system needs to be fast, it **declares** itself fast (§12.3) and stays where it is.

A corollary for naming: **packages are named for their role, never for their representation.** A
package named for a technique attracts anything using that technique, and becomes a second system.

### 4.3 General principles applied throughout

- **One source of truth per fact.** If a value can be derived, it is derived, not stored.
- **Make illegal states unrepresentable** where the type system allows, and fail fast where it does
  not.
- **Pure core, imperative shell.** Decision logic is pure functions over read views; all mutation
  happens through one narrow layer.
- **No ambient state.** No module-scope mutable singletons, no global registries mutated at runtime,
  no hidden clock, no hidden random source. Everything a system needs is passed to it. Shared static
  configuration is immutable at load and is never written per-entity or per-region.
- **Explicit module interfaces.** Each module exports a deliberate surface; internals are
  unreachable from outside. "Exported because a test needed it" is not a reason.
- **Generate rather than duplicate.** If a fact must appear in two shapes, one is generated from the
  other and the generated output is committed and reviewed.
- **Total over partial.** Prefer exhaustive mappings the compiler can check to lookups that can miss.

### 4.4 The surface computes nothing

Every quantity the surface displays is produced by a **named reader** in `domain`, `world` or the
observation store (§14). The surface may lay out, format and paginate. It may not compute a
world-derived quantity, and specifically may not sum, net, ratio, fee, mark or aggregate.

This is not fussiness. A presentation layer sits outside the ledger's reach, nobody thinks of a
display as a second implementation of a rule, and the result is two numbers for one fact with nothing
forcing agreement — the first entry in Appendix A, arriving through the one door the rest of this
design does not cover. Net asset value computed once in a portfolio screen and once in a status bar
is the canonical example, and it is the canonical example because it happens every time.

Enforcement: a lint rule forbidding arithmetic operators over world-derived types inside `surface`
modules, plus review checklist item 10 (§17.3). If a screen needs a number that no reader produces,
the fix is a new reader with a name, not an expression in a component.

---

## 5. The state model

### 5.1 Tables, not an object graph

World state is a set of **tables of typed columns**: entities, instruments, obligations, holdings,
liens, prices, journal, observations. There is no `Firm` object holding fields. A firm is a row; its
balance sheet is a query.

Three reasons, in order of importance:

1. **A2.** An object with fields invites per-class fields (`corporateBondHoldings`), and per-class
   fields are the N×M matrix in physical form. A holdings table has no place to put one.
2. **A1.** One storage location per fact means the ledger can be the only writer. A field on an
   object is a second location that someone will eventually assign.
3. **Performance.** In simulations of this shape the arithmetic is a small fraction of the runtime
   and the *representation* — allocation, pointer chasing, string keying, rebuilding views — is most
   of it. This is a hypothesis to validate against §3.4 in Phase 1b (§12.1), not an assumption to
   build on blindly.

Consequence to accept deliberately: there is no object to inspect in a debugger. A row inspector
that prints any row by name must exist in Phase 1b, or the project is unpleasant to work on. Budget it.

### 5.2 Identity

Every identity space — entity, instrument, venue, lien, series — is a **distinct nominal type**,
minted only by a named constructor, with one module per space owning the constructors. In TypeScript
(§21.1) that is `number & { readonly __space: unique symbol }`: a plain integer at runtime, a
distinct type at every boundary, and the brand survives array subscripting because the index type is
declared on the column reader rather than on the array.

**A lookup miss is an error, never a default.** "There is none of it" and "you asked with the wrong
kind of key" must not be the same answer. A store that returns zero for an unrecognised key converts
a bug into a plausible number, and the plausible number propagates.

Where one space legitimately derives from another — a company's equity is identified from the
company — the crossing is an explicit named conversion, so crossings can be counted and reviewed.

Identifiers are **dense integers assigned in a deterministic order**, not strings interned on first
encounter. Dense integers make lookups indexed loads rather than hash probes, make sharding a
contiguous range, and make results independent of encounter order (§11).

**Birth and death.** Identifiers are issued from a deterministic per-space counter at the moment of
creation, in the declared order of the creating system. **An identifier is never reused.** A dead
entity keeps its row and acquires a terminal status; it holds nothing and initiates nothing.
Reuse would silently re-point journal rows, digests and the per-(stream, entity) generators of §11 at
a different subject. The identifier space therefore grows monotonically over a run, and its size is
part of the §3.4 memory budget.

### 5.3 Quantities, units and money

Quantities carry their unit in the type: face value, share counts, physical units, floor area,
and money in each currency are distinct types. **The only route from a quantity to a value is
`quantity × price`.**

State the limit honestly. TypeScript has no value types (§21.1), so a branded number protects
**field and parameter boundaries**, not arithmetic inside a function body, where everything degrades
to `number`. A lint rule over mixed-unit binary arithmetic recovers part of the remainder. The rest
is review. Nobody should claim that subtracting a price from a face value "cannot compile"; the
accurate claim is that it cannot cross a boundary undetected. This is the largest single weakness
that the language decision buys, it was weighed against the alternatives in §21.1, and it is the
reason §17.2's naming rules are mandatory rather than advisory.

### 5.4 No stored value; one price per thing

There is no column recording what a holding is worth. Value is `units × price(asset)`, computed at
read. Storing units and value side by side creates two numbers that are equal only at the instant
they are written, and every reader between two writes gets a stale product with no indication.

Prices live in **one table indexed by instrument, for every priced thing** — including equity and
physical goods, which in most designs keep their price on the issuer or the good because they had one
before a price store existed. Every read names *which* price it wants (this period's close, or the
prior one) through the type system, so a stale mark is a different type rather than a different value.

An unpriced instrument returns "not priced" and the caller must handle it. It does not return zero.
Zero multiplies.

---

## 6. The ledger — conservation by construction

*Satisfies A1.*

### 6.1 One table, one verb

Every conserved thing — currency, security, physical good, capital, dwelling — is stored in exactly
one table:

```
holdings(holder, asset, quantity)
```

There is no separate accounts table and no separate positions table. Cash is a holding of a currency
asset; a bond position is a holding of an instrument asset; a tonne of steel is a holding of a good.
Two tables would be two representations of one concept sitting at the centre of the ledger, which is
Appendix A #1 inside the module built to prevent it. *Position* survives as a **read view**: holdings
restricted to instrument assets. *Account* survives as a word for a holder, not for a table.

No agent has a holdings field. No aggregate total is stored anywhere.

The only operation that changes a quantity is:

```
move(asset, from, to, quantity, reason)
```

It debits and credits in the same statement. There is no setter, no one-sided credit, no writable
field. The writable view of the holdings table is obtainable only inside the ledger module; every
other layer receives a read view. Not a convention — a type.

Therefore the sum of holdings for an asset is invariant **because no expressible program changes
it**, and the claim is discharged mechanically by the single-writer check and the verb property test
of §3.2.

**The closed verb set** is six operations and nothing else (§21, decision 4):

| Verb | Effect | Conserved? |
|---|---|---|
| `move(asset, from, to, quantity, reason)` | Debit and credit in one statement. | yes |
| `pledge(pledgor, beneficiary, asset, quantity, parent?, reason)` | Inserts a lien row (§6.5). | no — relations are not conserved |
| `release(lien, quantity, reason)` | Reduces or closes a lien row. | no |
| `amend(obligation, change, reason)` | The only way a scheduled row changes (§7.4). | no |
| `register(table, row, reason)` | Inserts an immutable declaration row: an entity, an instrument, an obligation. **Cannot touch a quantity column.** | n/a |
| `observe(series, period, value)` | Appends one observation (§14). Append-only, never read by the engine. | n/a |

Each has a batch form with identical preconditions, written and reviewed alongside the single form
(§19, Phase 1b), because batching is where a bypass gets added later. **Adding a seventh verb is an
ADR under §21.3**, a row in the change-cost table, and a new case in the toy suite. The set is small
enough to hold in mind, which is the property that makes "name the function that could unbalance it"
answerable at all.

### 6.2 Creation and destruction are moves, and every counter-account has an owner

Some things genuinely appear and disappear: a central bank creates money, a factory creates output, a
household consumes it, capital wears out. Most designs make these one-sided operations, and one-sided
operations are exactly where conservation stops being provable.

Instead, creation and destruction are moves to and from **named counter-accounts that are ordinary
holders in the same table**. Each counter-account is declared once, in a registry, with the mechanism
it represents and **exactly one owning system**:

This is the registry, settled as the complete initial set (§21, decision 6). Seven families, seven
owners. A new family is a new conservation law: an ADR, a row in the change-cost table (§18.4), and a
case in the toy suite.

| Family | Move | Owning system |
|---|---|---|
| `Issue:<currency>` | → central bank | monetary operations |
| `Issue:<instrument>` | → subscriber | primary issuance |
| `Endowment:<asset>` | → holder, once per period | demography (labour hours, land) |
| `Production:<good>` | → producer | production |
| `Consumption:<good>` | holder → | consumption |
| `Wear:<class>` | owner → | capital |
| `Writeoff:<instrument>` | holder → | wind-up |

Labour needs no family of its own: hours are endowed to households each period from
`Endowment:labour-<class>` and destroyed on use into `Consumption:labour-<class>`, which is the same
pair of mechanisms as any other good and requires no special case.

Write capability for a counter-account is **minted only to its owning system** (§10.2). No other
system can create money, produce output or write off a debt, because no other system holds a handle
that names those rows. This is what separates a counter-account from a plug.

That distinction deserves to be stated plainly, because Appendix A #7 warns against exactly the thing
a counter-account superficially resembles: a residual holder that always balances and therefore can
never report that the sum is wrong. Four properties separate them:

1. **One declared meaning.** A counter-account represents a specific mechanism, named in the registry.
2. **One owner.** Exactly one system can write it; every other system physically cannot.
3. **Visible flow.** Its per-period flow is a first-class output on the dashboard (§16.3) and a
   declared series (§14).
4. **No catch-all.** There is no `Misc:`, no `Adjustment:`, no `Plug:`. Adding a counter-account
   requires naming its mechanism and its owner, and is a reviewed change.

A plug has none of the four. Remove any one of them and this design has a plug.

The consequence is that the question a conservation audit exists to answer — *how much appeared from
nowhere this period?* — becomes **a balance you read**, not a sum you compute and compare. "The
economy minted 4.2bn this period" is the change in `Issue:<currency>`, on a named row owned by a
named system. If that is wrong, it is wrong visibly.

### 6.3 Integer quantities

Conserved quantities are **integers in the asset's smallest meaningful unit** — minor currency units,
whole shares, whole physical units. Prices, rates, elasticities and index levels remain real-valued.

This is load-bearing for A1. With floating-point quantities, a debit followed by a credit is not
exactly conservative, addition is not associative, and the total depends on summation order.
Conservation then degrades from a theorem into a measurement with a tolerance; a tolerance is a
judgement; and a judgement needs something to enforce it — which is the audit harness we are refusing.

**Representation** (§21, decision 2). Quantities are stored in `Float64Array` columns and constrained
to the safe-integer range, |q| < 2⁵³ ≈ 9.007 × 10¹⁵. Every verb asserts `Number.isSafeInteger` on both
the argument and the resulting balance. Float64 is chosen over `BigInt64Array` because integer
arithmetic below 2⁵³ is exact in float64, and BigInt allocates on every operation, which at §3.4
volumes is the difference between meeting N2 and not. It is chosen over `Int32Array` because 2³¹ is
too small for a currency total by six orders of magnitude.

**Units and range, per class:**

| Class | Unit | Ceiling before 2⁵³ | Margin against §3.4 |
|---|---|---|---|
| Currency | 1/100 of the base unit (a "cent") | 9.0 × 10¹³ base units | The design ceiling for total money in one currency is 10¹² base units; three orders of headroom. |
| Debt instruments | 1 cent of face, in the denomination currency | as above | Face totals are bounded by money and credit multipliers. |
| Equity | 1 whole share | 9.0 × 10¹⁵ shares | Issuance is in-model; a share count is bounded by the split rule in the equity specification. |
| Physical goods | 1 unit as declared in the good's specification, with a scaling exponent in the parameter registry | 9.0 × 10¹⁵ units | The exponent exists so a good measured in grammes and a good measured in vehicles both sit mid-range. |
| Capital | 1 whole capital unit of its class | 9.0 × 10¹⁵ | |
| Dwellings | 1 dwelling | 9.0 × 10¹⁵ | |
| Floor area | 1 whole square metre | 9.0 × 10¹⁵ | |
| Labour | 1 whole hour | 9.0 × 10¹⁵ | 10,000 households × a weekly endowment is ~10⁶ per period. |

A quantity or balance that leaves the safe range is a **defect** (§16.2), raised at the write door,
never wrapped and never saturated. Range is re-checked whenever §3.4 changes.

**Rounding and residues** (§21, decision 3). Every `price × quantity` yields a real number that must
be quantized at the point of write. Rounding and discarding the remainder is forbidden; it is a leak.
Two cases, and only two:

- **A single move.** Quantize the amount once, **round-half-to-even**, then debit and credit that same
  integer. No residue exists, because one number is written twice. Half-to-even rather than
  half-up because a model that rounds several million small flows per period should not drift in a
  known direction.
- **A distribution across many recipients** — a coupon across holders, an estate across claimants, a
  dividend. Quantize the **total** first, then allocate by **largest remainder, ties broken by
  ascending holder identifier**. The allocation sums to the total exactly by construction, so the
  residue is assigned to specific named parties rather than lost, and the assignment is deterministic
  under §11 because identifiers are dense and ordered. The journal records the residue recipients.

There is no third case. A rounding decision anywhere outside these two rules is a bug.

**What order-independence does and does not buy.** Because integer sums are order-independent,
"run a period with entities processed in reverse order and require identical quantities" is a usable
check on any reordering or parallelism. It covers **conserved quantities only**. Values, ratios and
every constraint test in §8.1 are real-valued sums of `quantity × price`, so a reordering can change
which constraints bind, and therefore behaviour, without moving a single quantity. Determinism of the
whole state rests on §11's accumulation rules, not on this check. Presenting it as a whole-state
determinism test would be false.

### 6.4 Preconditions at the door

The verbs reject, at the call site, by raising:

- non-positive or non-finite quantity;
- a move from a party to itself;
- a move of an asset a party class may not hold;
- a move of encumbered units (§6.5);
- a move initiated by an entity in a terminal status (§5.2, §6.7);
- an overdraft on a holder not declared overdraftable.

This is the distinction that makes "no harness" coherent. A **precondition inside the write door** is
part of the mutation: always on, fires at the moment and place of the mistake, names the caller. A
**periodic pass over the whole world** is an external audit: separated in time and space from the
cause, expensive enough to be made optional, and liable to rot into a check that cannot fail. The
first is kept. The second does not exist in this design.

### 6.5 Encumbrance is a relation, and it is built first

The hardest test of a conservation design is not a bond. It is **collateral**: an asset pledged
against a loan, where the pledgor still owns it, the pledgee has a claim on it, and — with
rehypothecation — the same asset can back more than one obligation at once.

An encumbrance is two-party, quantity-bearing, and not conserved. It fits no move verb. Recorded as
a note on the owner's row, over-pledging is structurally invisible. Recorded as a transfer, the
collateral leaves the pledgor's balance sheet, which destroys the thing you wanted to count.

So liens are **rows in their own table** — `(pledgor, beneficiary, asset, quantity, parent-lien)` —
with two derived reads:

```
encumbered(holder, asset) = Σ liens where pledgor = holder
free(holder, asset)       = held(holder, asset) − encumbered(holder, asset)
```

`move` transfers only free units. Over-pledging is unwritable, not detected. `pledge` and `release`
are verbs; a lien is never edited in place.

Rehypothecation is a chain through `parent-lien`, with a **maximum depth of 3** (§21, decision 5):
the original pledge plus two re-pledges. One would forbid the mechanism outright and make §6.5
untestable; two allows a single hop, which is enough to demonstrate the shape but not enough for
collateral scarcity to bite; three is the shortest chain in which an unwind has an interior. The
limit exists so that unwinding terminates and its cost is bounded at depth × liens, which is what
keeps wind-up (§6.7) inside the period budget. A pledge that would exceed depth 3 raises at the door.

**Build this in Phase 1a, before the first instrument exists**, and demonstrate a secured loan
end-to-end on three toy parties including one rehypothecation hop. It is the case that decides
whether a lien is a row or a field, and that decision cannot be revisited cheaply.

### 6.6 The journal is a byproduct

Every verb appends a row: who, to whom, what, how much, at what price, under what reason code. This
satisfies F8 and N5. Obligation amendments (§7.4) append too, with their triggering event.

The journal is **not the authority**, and state is **not** rebuilt by replaying it. That shape is
tempting and should be refused for a specific reason: if state is a fold of the log, then comparing
the fold to the state proves nothing, because both are the same function of the same rows. It is a
check that cannot fail — the failure mode this whole design is organised against.

Keeping holdings authoritative and the journal derived means the two are independently produced: the
verb writes the quantity from its arguments and records the instruction separately. The toy suite
uses exactly that independence (§15.1).

Journal retention is **two periods — the current one and the prior one** — held in a fixed-capacity
ring buffer of 1,200,000 rows, about 60 MB at §3.4 volumes (§21, decision 8). The capacity is fixed,
not elastic: exhausting it is a **defect** that raises, not a wrap that silently loses history, since
a journal that quietly drops its oldest rows is a record you cannot trust and therefore not a record.

Anything worth keeping beyond that window is projected into the observation store before the window
closes (§14), which is where the tension between short retention and long-horizon reporting is
resolved. History is otherwise reproduced by re-running from the seed, which N1 makes exact.

### 6.7 Wind-up: resolving an insolvent estate

*Satisfies F6.* Insolvency is the point where liens, claim ranks, counter-accounts and holder
eligibility all meet, so it is specified here rather than left to a system to invent.

**Trigger.** Insolvency is a *modelled outcome* (§16.2), not a defect. The test is declared and lives
in a system, never in the agent: failure to settle a due obligation, or a constraint breach the
agent's funding policy could not cure within its declared window. The agent does not decide it has
failed.

**Status.** On trigger the entity takes a terminal status. It may receive; it may not initiate. The
write-door precondition (§6.4) enforces this, so no code path can let a failed firm keep trading.

**The estate**, in resolution order:

1. **Secured claims resolve against their collateral first.** Each lien is released and the
   collateral moved to the beneficiary, up to the claim. Only the shortfall joins the ranked queue.
2. **Rehypothecation chains unwind from the leaf inward.** A chain that cannot unwind is a *defect*,
   not a modelled outcome: it means the depth limit or the free-unit precondition failed earlier.
3. **Remaining free holdings** form the distributable estate.
4. **The ranked queue** is ordered by the `claimRank` intrinsic fact (§7.2) and paid in rank order,
   pro rata within a rank, with the quantization residue assigned by the §6.3 rule.
5. **Unpayable claims are extinguished explicitly**, by a move to `Writeoff:<instrument>`. Nothing is
   dropped, deleted or silently zeroed. The loss is a readable balance owned by a named system.
6. **Residual holders** — equity — take what remains, normally nothing. The move still happens.

**Obligations** of the wound-up entity are cancelled by amendment (§7.4), never by row deletion, so
the reason code and the schedule's history survive in the journal.

**No instrument-type branch appears anywhere in the wind-up system.** If one is needed, a question is
missing from the intrinsic facts table and the fix is to add the question (§7.2).

The toy conformance suite must include an insolvency with one secured creditor, one unsecured
creditor and a genuine shortfall (§15.1).

---

## 7. Instruments as data

*Satisfies A2, first half.*

### 7.1 Why agent models normally have to change, and the fix

An agent model normally contains an enumeration of what it can hold: the bank sums its bond types,
the fund allocates across its known classes, the insurer reserves against known liabilities, the
wind-up ranks known claims. Each enumeration must gain a line for every new instrument. That is the
N×M matrix, and no amount of tidy layering removes it, because the coupling is semantic.

The fix has four parts: agents hold rows rather than fields (§8.2); obligations are paid generically
(§7.4); **there is a closed set of questions an agent may ask about any instrument** (§7.2); and the
questions whose answers depend on *who is asking* are separated out and keyed by regime (§7.3),
rather than being forced to a single global answer on the instrument row.

### 7.2 Intrinsic facts: the closed question set

Facts that are true of the instrument itself, regardless of who holds it. Declared once, in one
place, as a **total mapping** from instrument type to answers. Missing an answer is a compile error.

| Question | Example answers |
|---|---|
| Unit of measure and divisibility | face value, shares, physical units, floor area |
| Minimum piece | one minor unit, one share, a stated denomination |
| Currency of denomination | one currency identifier, or none for a physical asset |
| Issuer | entity identifier, or none |
| How it is quoted | price, yield, spread, rate |
| Tenor | dated (maturity period), perpetual, undated |
| Accrual basis | per-period, stated day-count family |
| Optionality | none, callable, puttable, prepayable, convertible — each naming the event that amends the schedule (§7.4) |
| Obligation schedule | rows in the obligations table |
| Which party classes may hold it | institutional only, any, issuer-restricted |
| Claim rank in a wind-up | secured, senior, subordinated, residual |
| Declared liquidity tier | tier 1, tier 2, illiquid |
| Where it prices | venue identifier |

Currency, issuer, tenor, accrual and optionality are on this list because a four-currency model with
callable and prepayable instruments cannot express itself without them, and because a fact added
after agents exist is far more expensive than a fact added before.

**This list is frozen as v1** (§21, decision 10). Thirteen questions, every one of them answerable
for every instrument type in the Phase 4 sequence. Adding a fourteenth is a reviewed change under
§21.3 with a column added across every type in the same commit; the review asks not "is this useful"
but "is this a question an agent must ask about *any* instrument", because a question only some
agents ask about some instruments is a relational fact (§7.3) or a system's own state.

### 7.3 Relational facts: keyed by regime, not forced to one answer

Some questions have no single answer. What risk weight a bond carries, how it is accounted for, and
whether it is collateral-eligible and at what haircut depend on **who holds it and under which
regime** — a bank under its capital rules, an insurer under its solvency rules, a fund under none of
them. Putting these on the instrument row forces one global answer, and the first time that answer is
wrong for somebody, a branch on holder type appears inside an agent. That is the N×M matrix arriving
from the other direction.

So they live in a **second total mapping, keyed by (instrument type, regime)**:

| Relational question | Depends on |
|---|---|
| Risk-weight family and weight | regulatory regime |
| Accounting treatment (cost / market / amortised) | accounting regime and declared intent |
| Collateral eligibility and haircut | the accepting venue or counterparty's regime |

The regime list is settled at **five** (§21, decision 11), one per distinct rulebook the model
contains: `bank-prudential`, `insurer-solvency`, `fund-unconstrained`, `household`, `sovereign`. So
the relational table is 3 questions × 5 regimes = 15 answers per instrument type, all compiler-
enforced. Five is the number of genuinely different rulebooks, not a placeholder for "more later": a
regime exists because a class of agent is bound differently, and if two regimes never differ on any
answer they are one regime.

Each agent declares **its own regime** as part of its constraints (§8.1). Agents call
`riskWeight(instrument, regimeOf(self))`, `haircut(instrument, regimeOf(venue))`. An agent never
branches on instrument type, and never branches on a regime other than its own.

The cost asymmetry this produces is the one a long-lived model needs:

- **A new instrument is cheap** — one intrinsic row plus one relational row per regime. The compiler
  enumerates every answer owed, and once the rows are complete, every agent already handles it.
- **A new regime is bounded** — one column across instrument types, zero agent edits beyond the
  agent that declares it.
- **A new *question* is expensive and visible** — it breaks every existing type until each is
  decided. Correct: a new question is a new dimension of behaviour and deserves a reviewed,
  everywhere change.

**Diagnostic rule, to be written on the wall:** *a branch on instrument type outside these two tables
is a bug report about the tables.* It means a question is missing. The fix is to add the question,
not the branch. Enforced by lint; exceptions are design debt with a named owner.

### 7.4 Obligations are rows, one system pays them, and amendment is a verb

An instrument's cash flows are **rows in an obligations table**, indexed by due period:

```
(instrument, kind, due-period, basis, amount | rate + index + margin, next-in-chain, status)
```

Not a periodicity formula. A formula cannot express an irregular schedule, a step-up coupon, a
sinking fund, an amortiser, a make-whole or a payment holiday — and the first time one is needed, the
formula acquires a special case, and the special case is where a second implementation of the same
rule is born.

**One generic system** walks the rows due this period and pays whoever holds the instrument. No agent
contains coupon-collection code. A new instrument's cash flows reach banks, funds, insurers, pension
funds and households with zero lines written in any of them.

**Schedules change during an instrument's life, and that is a verb, not an edit.** Default truncates
a schedule; prepayment reshapes it; a call extinguishes it; an insurance claim crystallises one that
did not exist. Pre-generating rows at issuance is correct for what is known at issuance, and F4
requires the rest. So:

```
amend(obligation, change, reason)
```

- **It is the only way an obligation row changes.** Rows are never edited in place and never deleted;
  cancellation is a status change with a reason code, so the journal keeps the history (§6.6).
- **Capability is narrow.** Amendment handles are minted per triggering mechanism (§10.2), and this
  is the settled and complete list (§21, decision 12). No system holds a general amendment handle,
  which is what keeps `amend` from becoming a second write path (R12).

  | Amendment | Permitted system | Effect on rows |
  |---|---|---|
  | Default truncation | credit | Remaining rows → cancelled, with the default event as reason. |
  | Prepayment | credit | Future principal rows rescheduled or cancelled; no row deleted. |
  | Call / early redemption | primary issuance | Rows after the call date cancelled; a redemption row inserted. |
  | Cancellation on wind-up | wind-up | All open rows cancelled (§6.7). |
  | Claim crystallisation | insurance | A contingent obligation materialises as a row. |

  Coupon resets are **not** amendments. A floating row stores `rate + index + margin` and is
  evaluated against the index at payment; nothing is rewritten, so a reset needs no capability and
  leaves no journal entry beyond the payment itself.
- **Contingent obligations** are declared as optionality on the instrument (§7.2) and materialise as
  rows only when the declared event fires. Until then no row exists and the payment walk does not see
  it. The event test lives in the owning system, never in the payment walk.

**N3 constraints on contingency.** Indexing obligations by due period makes the payment pass
proportional to what fires, not to the accumulated stock. Amendment must obey the same rule: **an
event test that scans all live instruments each period is a violation of N3** and requires an index
over the state it watches. Any new contingency must state its index in its specification.

---

## 8. Agents as policy

*Satisfies A2, second half.*

### 8.1 An agent is five declarations

What distinguishes a bank from a hedge fund from a pension fund from a household is not plumbing:

1. **Mandate** — which assets it may hold, and in what proportions.
2. **Regime** — which regulatory and accounting regime its relational facts are read under (§7.3).
3. **Constraints** — the inequalities that bind it, each expressed over *facts* and holdings: capital
   adequacy over risk weights, liquidity coverage over liquidity tiers, leverage, duration targets.
   **Never over instrument types.**
4. **Valuation** — how it forms a reservation level for something it might buy or sell.
5. **Funding policy** — what it does with a surplus or a deficit: issue, borrow, distribute,
   deleverage; and the window within which a breach must be cured before insolvency is triggered
   (§6.7).

Everything else — settling, holding, collecting, marking, reporting, being wound up — is generic
machinery the agent does not own and does not know about.

### 8.2 An agent holds rows, never fields

An agent has no per-class holdings field. Its balance sheet is a query:

```
totalAssets(agent) = Σ over holdings of agent:  units × price(asset)
```

A new instrument enters that sum the moment someone holds one, with no edit anywhere — **provided
the discipline is absolute**. The moment any agent caches a per-class total as a field, the N×M
matrix returns through the back door, and it returns silently. No stored aggregates. Ever. If a total
is expensive to compute, that is a storage-layout problem (§12), not a licence to cache it on the
agent.

### 8.3 Adding an agent type

One module declaring the five items above. It can participate in every existing market on the day it
is written, because participation is expressed through the same interfaces every other agent uses.
Zero edits to any instrument, any market, or any other agent.

---

## 9. Markets

### 9.1 One clearing interface

Every venue clears the same way: each participant posts a **reservation level and a size it scales
into**, and the venue solves for the level at which demand meets available supply.

This matters more for extensibility than for realism. Price formation expressed as *"here is the
quantity I want"* has no floor mechanism, produces a shape without a level, and forces every venue to
invent its own rule — which means every new instrument brings a new pricing code path, which is the
N×M matrix again in a different costume. Price formation expressed as a **schedule** is
asset-class-agnostic: one solver, one participant interface, and a new venue is an adapter naming who
participates and how they value, not a new engine.

The solver depends only on `domain` and `kernel` (§4.1) so it can be tested and optimised in
isolation. Its termination must be provable from its own inputs: a solver that needs an iteration cap
to stop is a solver whose convergence condition is unstated, and the cap hides the reason.

### 9.2 Budget allocation across venues

The question is whether buyers spend from a single wallet across venues opened in sequence, or
whether a budget is allocated per venue before any venue opens. It is a decision about economics that
cannot be changed later without rewriting behaviour, so it is settled here rather than discovered
after the loop exists.

**Decision: pre-allocated budgets** (§21, decision 14). Each agent's funding policy allocates its
spendable balance across the venues it will participate in, before the first venue opens. Unspent
allocations are returned by an **explicit reconciliation step** at a named position at the end of the
market block. An agent cannot overspend, because its allocation is checked at submission and its
cash moves at settlement.

Why, given that the single-wallet alternative is the richer economics:

- With **40 venues per period** (§3.4), venue order under a single wallet is a large, hidden, and
  economically load-bearing model parameter. Nobody would choose forty ordering assumptions
  deliberately; under single-wallet you acquire them by default.
- Pre-allocation keeps venues independently evaluable, which is the precondition for testing a venue
  in isolation and for any future sharding (§12.3). Single-wallet forecloses both permanently.
- It makes N1 easier to hold: no cross-venue path dependence means a venue's result is a function of
  its own submissions.

The cost, accepted explicitly: **agents cannot opportunistically move funds between venues within a
period.** A bank that sees a bargain in the corporate venue cannot pull cash it allocated to the
sovereign venue an hour earlier. At a one-week period this is a small distortion, and the funding
policy re-allocates every period. If it later proves material, the fix is a richer allocation rule
inside the funding policy, not a return to sequential wallets.

### 9.3 Simultaneity: what a decision may read

Agents form reservation levels from prices; prices form from reservation levels. The epoch types of
§10.3 make a lagged read distinguishable from a stale one, but they do not decide the model rule, and
leaving it undecided means it gets decided per system by whoever writes each one. The rule:

- **Decision systems read prior-close.** Valuation, constraints and funding policy (§8.1) take the
  previous period's marks. Their manifests mint prior-close handles only, so a decision system
  physically cannot read a price formed later in its own period.
- **Only clearing and settlement read this-close.** The solver sees this period's submissions;
  settlement, margin and mark-to-market run after the clearing that produces the price they need, at
  a named position in the order (§10.4).
- **No system reads a price whose formation is scheduled after it.** This is checkable from the
  manifests and the committed order, and it is checked in CI.

Applied to system families (§21, decision 15):

| Family | Reads | Examples |
|---|---|---|
| **Endowment and accrual** | neither; they act on schedules | demography, obligation payment, depreciation |
| **Decision** | prior-close only | valuation, constraints, funding policy, budget allocation, production planning, consumption planning, policy rules |
| **Clearing** | this period's submissions | the solver, per venue |
| **Post-clearing** | this-close | settlement, margin, mark-to-market, default testing, wind-up |
| **Projection** | everything, writes only observations | series writers, period trace |

### 9.4 The committed period order

This is the initial list required by §10.4, committed here so that §9.3 is checkable from day one.
Positions are stable names; systems are inserted at a named position, never appended.

1. Endowment — labour and land issued from `Endowment:`
2. Obligation payment — the due-period walk (§7.4)
3. Depreciation — `Wear:`
4. Production — inputs consumed, output issued from `Production:`
5. Policy — central bank and fiscal rules, on their declared meeting periods only
6. Valuation and constraints — every agent, prior-close
7. Funding policy and budget allocation (§9.2)
8. Clearing — the 40 venues, in a fixed list order that is not economically load-bearing under §9.2
9. Settlement — cash and asset legs move
10. Budget reconciliation — unspent allocations returned
11. Margin and collateral — pledge, release, substitution
12. Mark-to-market and accrual
13. Default testing and wind-up (§6.7)
14. Primary issuance — new instruments, calls, redemptions
15. Consumption — households consume into `Consumption:`
16. Projection — observation series written (§14)

Everything between positions 6 and 8 reads prior-close; everything from 9 onward reads this-close.
The boundary is at one place in the list, which is what makes the CI check trivial and the model rule
memorable.

---

## 10. Time, sequencing and capabilities

### 10.1 Periods

The world advances in discrete periods of the length fixed in §3.4. Within a period, work is done by
**systems**: modules with a single responsibility, each of which may not import another system.

### 10.2 Capabilities are minted from the manifest

Each system declares what it reads and what it writes, at the granularity of named facts — including
which counter-accounts it may write (§6.2) and which obligation amendments it may make (§7.4). The
runtime **mints the system's access handles from that declaration**. A system that did not declare a
row family has no handle for it and physically cannot touch it.

This is deliberately stronger than declaring and then verifying. Verification of declarations is a
runtime audit — the thing this design refuses — and it can only see what a given run exercises.
Minting makes undeclared access **unrepresentable**, at zero per-access cost, with the check
happening once when the handle is created.

Residual weakness, stated honestly: nothing prevents a system from over-declaring. The counter is a
per-system declaration-width budget reviewed at merge — a social control, not a compiler one. Under
schedule pressure the temptation is to widen the declaration rather than split the system, and
reviewers should watch for exactly that (R4).

### 10.3 Prior-period reads

Reading a previous period's value requires requesting it by a different name, which returns a
**different type** with no method that yields a current figure. A legitimate lagged read and an
accidental stale read are then distinguishable by a compiler rather than by reasoning about
execution order. The model rule that says which one a given system is entitled to is §9.3.

**No default parameter may supply a world-derived quantity.** Making such arguments required is what
prevents readers from silently taking the stale path, and it costs nothing.

### 10.4 The order of systems is declared, reviewed and committed

The order within a period is a **hand-written, reviewed, version-controlled list**.

It is tempting to derive it topologically from the read/write declarations. Do not, for three
reasons:

1. **It is degenerate.** At any granularity you can realistically declare, most systems read and
   write the same few things — nearly everything touches cash — so the graph is close to complete and
   the derived order collapses back to declaration order anyway, with the manifest as pure overhead.
2. **It is not unique.** A topological order is one of many valid orders, so adding one system can
   legally reorder two unrelated ones. With price-sensitive dynamics that changes results and forces
   a re-baseline of the golden digest (§11) — and re-baselining is precisely the move under which
   regressions hide.
3. **It hides intent.** "This must run inside the settlement window" is a fact about the model. It
   should be written down, not inferred.

An explicit list is reviewable in a diff, which is the property that matters.

---

## 11. Determinism

*Satisfies N1, and is a precondition for every performance change.*

- **One seeded generator per (stream, entity)**, derived from the world seed and the entity's
  identifier. Never one global stream consumed in iteration order — that makes every draw dependent
  on processing order and makes reordering impossible. Because identifiers are never reused (§5.2),
  a generator is bound to one subject for the life of the run.
- **No wall-clock, no ambient randomness, no environment reads** in any layer below `surface`. Time
  is the period counter, passed in. There is no other calendar; anything date-shaped that the surface
  displays is derived from the period counter and the declared epoch.
- **No iteration over unordered containers.** Where order matters, iterate dense integer ranges or
  explicitly sorted keys with a total, stable comparator.
- **Accumulation order is part of the design.** Any sum across entities is either a per-row column or
  a declared accumulator with a stated combination rule; parallel work is combined in shard order,
  never in completion order. This, not §6.3's reverse-order check, is what makes real-valued results
  reproducible.
- **A canonical state digest** over all tables, computed on demand, and a **golden-digest test** in
  CI. Alongside it, a **state differ** that reports which fields diverged and by how much — a digest
  alone tells you *that* something changed and is nearly useless for finding out *what*.

Build the differ before the first optimisation, not after. It is the single instrument that is
painful to add late.

One limit: transcendental functions are not guaranteed bit-identical across platforms and runtime
versions. Define the determinism contract as *identical results for a given build*, not as a
universal constant, and pin the comparison baseline to the build that produced it.

---

## 12. Performance and concurrency

### 12.1 Budgets are requirements, and measurement starts in Phase 1b

N2, N3 and the workload they are measured against are numbers in §3.4, fixed before the systems are
written. Stand up the measurement at the same time:

- an **interleaved A/B benchmark** with a **published noise floor** — differences below the floor are
  reported as unresolved, never as wins;
- a **long-horizon run** exercising N3, because a short profile is structurally incapable of seeing
  the cost term that dominates a long run;
- **profiling hooks as a permanent part of the system interface**, zero-cost when disabled, rather
  than instrumentation added and removed per investigation.

And state, in advance, the number that would **kill** a design decision — for example: *if
journalling every move costs more than X% of the period budget at §3.4 volume, the journal grain
must be coarser.* A forecast with no falsifying measurement is an opinion.

### 12.2 What to expect, and what to verify

The prior for this class of model — entity-poor, mechanism-rich — is that no single hot loop exists,
arithmetic is a small fraction of runtime, and cost is spread across allocation, indirection and key
lookups. That prior is only meaningful against the entity counts in §3.4; confirm it with the Phase
1b benchmark. If confirmed, the columnar decision (§5.1) is already collecting the benefit, and
further optimisation should target allocation and lookups before arithmetic.

### 12.3 Concurrency

Default: **single-threaded**. Parallelism is added only against a measured need, and only in one
shape: **contiguous row ranges over shared storage, combined in shard order**. No transport, no
serialisation of entities across a boundary, no per-entity crossing structures.

The rule that matters: **choose one acceleration boundary and put everything through it.** Two
acceleration strategies built against the same seam cannot compose, and a system with a fast
arithmetic path and a fast throughput path that exclude each other ends up shipping with neither
enabled.

If a native or compiled component is ever introduced, its data layout must be **generated from the
same schema as the primary implementation**, with a compatibility identifier checked at load. A
hand-maintained second layout held in step by a comment is a fault waiting for a schedule crunch. If
both sides of a boundary cannot be generated, do not create the boundary.

### 12.4 Platform

The target is a mobile device of the slowest supported generation with 4 GB of RAM (§3.4), running the
engine as a single ES module under the app's JavaScript runtime (§21.1). A **platform probe runs in
CI from Phase 1b**, so that a capability the target lacks cannot be depended on silently. The probe
covers at minimum: `Float64Array` and `SharedArrayBuffer` availability, `Number.isSafeInteger`
behaviour, structured-clone limits for save payloads, and the absence of any Node-only API in the
engine bundle. Two specific traps: capabilities that
exist in the development runtime but not the deployment one, and imports that break the target build
while all tests continue to pass in the development environment. Benchmark on target hardware early;
optimising against the wrong device optimises against nothing.

---

## 13. Persistence and the seed

*Satisfies F7, N6.*

Because state is tables rather than an object graph, a save is a **dump of columns**, not a traversal:
fast, exact, and free of the "which fields did we forget" class of bug.

- A saved state carries a **schema identifier**. A build whose schema differs refuses to load it and
  says so. During development this is correct behaviour and cheap; a save is regenerated from the
  seed.
- **Round-trip is a CI test**: save, reload, digest, compare.
- Any state that cannot live in a column — variable-length text, nested structures — lives in a
  **declared** secondary region, enumerated explicitly, so that "we compared the whole state" is a
  true statement rather than one that quietly excludes what it could not reach.
- Observations (§14) are saved with the world but are excluded from the golden digest and carry their
  own series digest, so that a reporting change does not force a state re-baseline.

**The opening world is built by ledger operations, not by assignment.** Every initial holding is an
issue or a transfer from a counter-account. A world assembled by writing quantities directly is a
world whose opening books were never proved, and A1 would hold only from period one onward — which
is to say, not at all.

**The opening world is generated from primitives, not from levels (A3).** The seed is a closed list
(§21, decision 13):

| Primitive | Form | Why it is a primitive and not an outcome |
|---|---|---|
| Population by region and cohort | counts | A model of an economy needs people; how many is a scenario choice, not an observation to match. |
| Labour endowment per household per period | hours | Dimensionless in the sense that matters: it is a capacity, not a price. |
| Physical endowments | counts of goods, capital units, dwellings, square metres of land | Something must exist at period 0 or nothing can be produced. |
| Technology coefficients | input–output ratios per sub-unit, dimensionless | Ratios, not levels. |
| Preference parameters | shares and elasticities, dimensionless | |
| Policy rule coefficients | dimensionless responses, **never rate levels** | A Taylor-type rule's slope is a primitive; the rate it produces is an outcome. |
| Institutional counts | how many banks, funds, insurers | |
| Numéraire scale | one quantity of currency minor units issued to each government at period 0 | The only level in the seed, and it is a pure scale factor: doubling it doubles every nominal quantity in the run and changes nothing real. Registered `structural` on exactly that justification. |

**The opening world contains no financial instruments.** No bonds, no loans, no deposits, no equity
positions seeded with terms. Every instrument in the run is issued by the model, which means no coupon,
no spread, no rating threshold and no curve is ever seeded — the whole class of A3 violation that
begins "we need something reasonable to start from" has nowhere to enter. Firms and banks capitalise
themselves in-model during the burn-in.

Prices are likewise unseeded. The first clearing (period 0) forms reservation levels from primitives
directly — marginal cost from technology, willingness to pay from preferences — rather than from a
prior mark. §9.3's prior-close rule takes effect from period 1. The distinction between an **opening
condition** (a price, re-cleared next period) and **terms** (fixed for the life of an instrument)
therefore collapses: Aurora has no seeded terms at all.

The cost, accepted: the model is unrecognisable as an economy for a long time after period 0, and the
260-period burn-in (§3.4) exists to absorb that. A short burn-in and a plausible-looking period 0 was
the alternative, and it is exactly the trade that puts real-world levels into the seed.

Because nothing is calibrated, the opening world is arbitrary and its early periods are transient.
The declared burn-in (§3.4) is part of the run contract, and the surface marks pre-burn-in periods as
such.

---

## 14. Observation and reporting

*Satisfies F9, and resolves the tension between short journal retention (§6.6), no stored aggregates
(§8.2) and a surface that must chart history.*

Derived quantities that must persist across periods — macro aggregates, index levels, counter-account
flows, trailing statistics, anything the surface plots — live in an **append-only observation store**:
one row per (series, period), written once, never updated.

The rules that keep it from becoming a second state:

- **Nothing in the engine reads it back.** The observation store is write-only from the engine's side
  and read-only from the surface's. There is no handle that lets a system read an observation.
- **If a decision needs it, it is not an observation.** A trailing average that agents actually
  respond to is *state*: a declared column with an owning system and an update rule, subject to every
  rule in §5 and §6. It may additionally be observed. The distinction is which side of the engine
  boundary the number is consumed on, and it is enforced by capability, not by convention.
- **One writer per series.** Each series declares: name, unit, whether it is a stock or a flow, the
  reader that computes it, and the single system that writes it, at a named position at the end of
  the period order.
- **Flows are projected before the journal window closes.** A flow series is computed from
  counter-account deltas and journal rows within the period in which they occur. This is what lets
  journal retention stay at a period or two while long-horizon reporting still works.
- **Budgeted.** Size is periods × series, fixed-width float64: 512 × 1,560 × 8 bytes ≈ 6.4 MB, held
  against a 20 MB allowance for headroom (§3.4). The **512-series cap is hard** (§21, decision 9); a
  new series above the cap requires retiring one, which is what stops the store growing into a
  second, unreviewed model.

The initial families, each with one owning system:

| Family | Series | Owner |
|---|---|---|
| Counter-account flows | one per counter-account family per asset | the owning system of each (§6.2) |
| Output and expenditure | production, consumption, investment, government, net external, by region | projection |
| Prices | venue clearing levels and the derived indices, by region | projection |
| Labour | employment, hours, wage distribution moments | projection |
| Credit | lending, defaults, write-offs, spreads by rating bucket | projection |
| Rates and curves | policy rate, the sovereign curve by tenor, per region | projection |
| Market microstructure | submissions, clearing depth, unfilled quantity per venue | projection |
| Sectoral balances | net lending/borrowing by sector, per region | projection |
| Placeholder register | count and trend (§16.1) | projection |
| Period trace | duration and rows touched per system (§16.3) | runtime |

Counter-account flows are observations of the first importance and are always present: they are the
mint-and-destroy readings that replace a conservation report (§6.2).

---

## 15. Correctness strategy

The requirement is no external audit harness. That is achievable, but only if "correctness" is
decomposed properly. This is the assurance ladder, strongest first. **Every invariant in the system
must be assigned to a tier, and tier 4 is empty by design.**

| Tier | Mechanism | Cost | Catches |
|---|---|---|---|
| **1. Structural** | Impossible to express: single write path, capability-minted handles, branded types, total mappings. | Zero at runtime. | Conservation, over-pledging, undeclared access, unauthorised issuance or amendment, unit confusion at boundaries, missing facts. |
| **2. Build-time** | Compile errors and lint over source: import boundaries, exhaustive matches, mixed-unit arithmetic, single-writer check, vocabulary with no writer, branches on instrument type, arithmetic in `surface`, registry entries citing external sources. | Seconds, every build. | Structural rules a type cannot state. |
| **3. Write-door precondition** | Raises at the call site, always on, in production. | Nanoseconds. | Bad arguments, illegal transfers, encumbered moves, moves by terminal entities, overdrafts. |
| **4. Runtime audit over the whole world** | — | — | **None. This tier does not exist.** |
| **5. Tests** | Fast, deterministic, in CI: unit tests of pure functions; property tests over the ledger verbs; the toy-world conformance suite; the golden digest; the manifest/order consistency check (§9.3). | Seconds. | Behavioural regressions, arithmetic mistakes, ordering changes. |

Two clarifications, because they are where this constraint is usually violated in spirit:

**Tests are not a harness.** A harness is a runtime pass over the whole world that measures whether
invariants held during a real run. A test is a deterministic exercise of a small world with a known
answer, run in CI. The first is what we are eliminating; the second is ordinary engineering and we
are doing plenty of it.

**A benchmark is not a harness either.** Measuring speed is not checking correctness.

### 15.1 The toy-world conformance suite

A handful of parties, one currency, one good, one bond, one secured loan with a rehypothecation hop,
and one insolvency. Every ledger verb exercised. It runs in milliseconds and asserts:

- **Conservation, as a property test.** Over generated sequences of legal verb calls, the sum per
  asset across all holders including counter-accounts is invariant. (The structural claim — that no
  such function exists — is discharged by the single-writer check in tier 2, not asserted here. An
  assertion that "no verb can be found" is not a thing a test can evaluate, and writing one would be
  the failure mode this design is organised against.)
- **Counter-account flows equal the journal.** Each counter-account's period delta matches the sum of
  journal rows naming it. The two are independently produced (§6.6), so the comparison has content.
- **Encumbrance.** An attempt to move encumbered units raises; over-pledging cannot be written; a
  rehypothecation chain unwinds from the leaf.
- **Wind-up.** An insolvency with one secured creditor, one unsecured creditor and a shortfall
  distributes the estate exactly, extinguishes the remainder to `Writeoff:`, and leaves the equity
  holder with nothing by an explicit move.
- **Amendment.** A default truncates a schedule by `amend`, the cancelled rows survive with a reason
  code, and the payment walk stops paying them.
- **Round trip.** Save, reload, digest: bit-identical.

This suite is written in Phase 1a, **before** the first economic system, and it is the acceptance
test for the substrate.

### 15.2 What this design cannot do

An architecture document that omits this is selling something.

- **It cannot tell you the economics is wrong.** A demand schedule with the wrong shape, a policy
  rule with the wrong sign, an implausible elasticity — all structurally perfect, all nonsense.
- **It cannot tell you the counterparty was wrong.** Conservation holds when you pay the wrong party.
  Holder-eligibility types catch a class of these; not all.
- **It cannot tell you a mechanism is missing.** An absence has no location: nothing in a codebase
  says "and no borrower ever defaults" or "nothing is ever forced to sell." Any review derived *from
  the code* reproduces the code's blind spots exactly, looks thorough, and finds nothing.

  The only instrument that finds an absence is a **specification written from the domain with the
  code shut**: for each system, what must exist for it to be that system, what must be true, and
  explicitly what must *not* exist. Then compare. This is a document read by people, not a harness.
  Two disciplines make it work: it is updated in the same change as the code, and a clause is never
  deleted to make the comparison look better.

- **It cannot enforce A3 against a determined author.** §16.1 makes every parameter choice named and
  owned; it cannot read the author's mind about why the number was chosen.
- **One loss to accept deliberately:** when leaks become impossible, leaks stop being a diagnostic.
  In a system where value can escape, an imbalance is a cheap and powerful hint that a mechanism is
  wrong. Removing the possibility removes the hint. The counter-account design (§6.2) and the
  observation store (§14) buy it back — every creation and destruction is a named, plotted series —
  but only if someone actually looks. Hence §16.3.

---

## 16. Cross-cutting concerns

### 16.1 Parameter provenance, and the enforcement of A3

Every numeric constant on a behavioural path is declared in a **parameter registry** carrying: name,
value, unit, owning system, provenance, and a written justification.

Provenance is one of four values. There is deliberately no `measured` and no `calibrated`:

| Provenance | Meaning | Justification required |
|---|---|---|
| `structural` | Definitional or arithmetic. Periods in a year; the number of ranks in a waterfall. | A sentence. |
| `derived` | Computed at initialisation from other registry entries or from state. | The derivation. |
| `assumed` | A modelling choice. Preferably dimensionless or in model units. | Why this shape of behaviour, in domain terms, with no reference to an observed figure. |
| `placeholder` | Stands in for a mechanism not yet built. | The absent mechanism, its owner, and the change that deletes it. |

Rules that follow:

- **No entry may cite an external source.** A registry entry whose justification is "this is roughly
  the real figure" fails review, and a mechanical check fails the build on source-citation fields.
  This is A3's teeth (§3.2).
- **Any `assumed` entry that is a level** in currency or index units carries a heightened burden: it
  must be derived from primitives, or it is a placeholder. Levels are outputs.
- **A bound is not a mechanism.** Clamping a value where a mechanism should be converts an absence
  into a plausible number, and the number propagates. Worse, it destroys the evidence that would have
  named the absence. Where a clamp genuinely is arithmetic impossibility (a price cannot be
  negative), name it as such. Where it stands in for something unbuilt, register it as a placeholder
  with an owner, and **delete it in the same change that builds the mechanism.**
- **An iteration cap is a placeholder**, not a fix. A loop that needs a cap to terminate has an
  unstated convergence condition; the cap conceals it. Register the cap, name the loop, and treat its
  presence as an open defect (§9.1).
- **The placeholder count is the honest measure of how much model is missing**, and it is on the
  dashboard next to the counter-account flows.

### 16.2 Error policy

| Category | Example | Handling |
|---|---|---|
| **Defect** | Lookup miss, illegal transfer, unit mismatch, non-finite value, un-unwindable lien chain. | Raise immediately with full context. Never a default value, never a logged warning. |
| **Modelled outcome** | Insolvency, failed auction, unfilled order, missing price. | An explicit value in the domain type. Callers must handle it; the compiler enforces that they do. |
| **Environmental** | Save file unreadable, platform capability absent. | Reported to the surface layer; the engine does not continue in a degraded state. |

A non-finite value is a **defect at the point of comparison**, not a falsy result. A comparison
against a non-finite number silently answers "false", which is how a check stops checking without
anyone noticing.

### 16.3 Observability

- **Counter-account flows on the dashboard.** The mint-and-destroy readings, and the closest thing to
  a conservation report this design has — by design.
- **The placeholder register**, with its count and its trend.
- **A period trace**: which system ran, how long it took, how many rows it touched. Off by default,
  zero cost when off, permanent.
- **Query-anything row inspector** (§5.1), from Phase 1b.
- **Burn-in state**, so no reader mistakes a transient for a result.

### 16.4 Configuration

Configuration is a value passed in at construction, not read from the environment inside the engine.
Anything read from the environment is a determinism hazard and a portability hazard at once.

---

## 17. Engineering standards

### 17.1 Modules

- One responsibility per module; if the name needs "and", split it.
- An explicit exported surface. Internals are unreachable. "Exported for a test" is not a reason —
  test through the public surface or move the logic.
- No circular imports at any granularity, enforced in CI.
- No module-scope mutable state, and no mutation of shared static configuration at any time,
  including during initialisation.
- No file remains in the tree that is not imported by something. Migration scripts, one-off patches
  and abandoned refactors are deleted in the change that finishes them, and CI fails on unreferenced
  source files.

### 17.2 Naming

Names carry the things that mistakes are made of:

- **Unit** — `principalMinorUnits`, not `principal`.
- **Periodicity** — `wageAnnual` / `wagePerPeriod`, never a bare `wage`.
- **Currency or basis** — `valueLocal` vs `valueBase`, never ambiguous.
- **Freshness** — `priceThisClose` / `pricePriorClose`.
- **Derivation** — a computed reader is a verb (`totalAssetsOf`), never a noun that reads like storage.

### 17.3 Review checklist

A change is not approved unless a reviewer can answer yes to all of these:

1. Does every new fact have exactly one storage location?
2. Does every new conserved quantity move only through a verb?
3. Does any new code branch on instrument type outside the facts tables? (If yes: which question is
   missing?)
4. Does any agent gain a per-class field or a cached aggregate?
5. Are new constants in the parameter registry with provenance, and does any of them encode an
   observed real-world figure (A3)?
6. Does any new clamp or iteration cap stand in for a missing mechanism? Is it registered?
7. Are new reads of world state declared in the manifest, and is the manifest no wider than
   necessary?
8. Does the specification for this system still describe what the code does?
9. Is the golden digest change, if any, explained by the intended behaviour change?
10. Does the surface compute anything, or does every displayed number come from a named reader?
11. Does any new counter-account or amendment capability have exactly one owning system?

### 17.4 Definition of done for a system

1. **Specification written first**, from the domain, with no code open: what must exist, what must be
   true, what must not exist.
2. Tables and vocabulary entries added; intrinsic and relational fact rows completed.
3. Ledger verbs used — no new write path.
4. The system, with its manifest, inserted at a named position in the period order.
5. Any series it owns declared in the observation store.
6. Tests: unit tests of the pure logic, and a toy-world case if it touches conservation.
7. Specification and code reconciled, in the same change.

---

## 18. Extension playbooks

### 18.1 A new instrument — worked example: covered bonds

A covered bond is a bank's bond secured on a ring-fenced pool of its own loans, where the pool must
be replenished to maintain a cover ratio, and holders have recourse to both the pool and the bank. It
is the right test case because it exercises every joint at once: a new instrument class, a new
schedule shape, a two-party non-conserved relation, a new venue, and a claim-rank interaction.

The complete change, run to its honest cost:

1. **Specification**, written from the domain, committed first — including what must not exist ("no
   pool substitution without releasing the outgoing lien").
2. **One vocabulary entry.** Every exhaustive match now fails to compile, enumerating the decisions
   owed.
3. **One row of intrinsic facts:** unit is face value; currency; issuer; quoted as price; dated;
   accrual basis; optionality none; institutional holders; claim rank secured-on-pool; liquidity tier
   2; venue identifier.
4. **One row of relational facts per regime:** risk-weight family, accounting treatment, collateral
   eligibility and haircut, each answered per regulatory regime rather than once globally.
5. **Zero new tables.** The bond is a row in instruments; its coupons and principal are rows in
   obligations; holdings are rows in holdings; **the cover pool is rows in liens** — pledgor is the
   bank, beneficiary is the bond, no parent lien.
6. **One market adapter**, if it trades somewhere new. The solver is untouched.
7. **One system**, for pool maintenance: substitution is `release` then `pledge`, and an attempt to
   release below the ratio raises at the call.
8. **One constraint added to the bank's declaration** — and this is a genuine agent edit. The cover
   ratio is something a bank must satisfy, so it belongs in the bank's constraint list (§8.1). A
   design that claimed zero here would be claiming that a new obligation on a bank costs the bank
   nothing, which is false.

**What the architecture actually buys**, and it is still the whole point: **zero edits to the asset
manager, insurer, pension fund, household, wind-up, settlement, payments, pricing and reporting.**
They price it, hold it, collect its cash flows, count it toward their ratios and rank it correctly,
because none of them was ever written in terms of instrument types. One agent changes, because one
agent's behaviour genuinely changed. That is A2 working, not A2 failing.

For contrast, the case that genuinely costs nothing: **a senior unsecured corporate bond at a tenor
no existing instrument uses** is one intrinsic row, one relational row per regime, its obligation
rows, and nothing else anywhere.

**The remaining honest exceptions:**

- A **new question** agents must answer is a new column, breaking every existing type until each is
  decided. Deliberate and correct.
- A **new regime** is a new column across instrument types plus the agent that declares it.
- A **new conservation law** — a new way for something to be created or destroyed — touches the
  ledger and adds a counter-account with an owner. Covered bonds do not. Something eventually will,
  and that change should be hard and reviewed.

The rule underneath: **an instrument is data; a behaviour is code.** New instruments are nearly free.
New behaviours cost, and should.

### 18.2 A new agent type

One module: mandate, regime, constraints, valuation, funding policy (§8.1). It trades everything that
exists on the day it is written. Zero edits elsewhere.

### 18.3 A new venue

One adapter naming participants and how they value, plus a venue identifier on the intrinsic facts of
anything that trades there, plus a named position in the period order. The solver is untouched.

### 18.4 Change-cost table

This is the contract. If a proposed change does not fit here, the architecture has a gap and the gap
is the finding.

| Change | Vocabulary | Intrinsic facts | Relational facts | Tables | Ledger | Agents | Markets | Systems |
|---|---|---|---|---|---|---|---|---|
| New instrument type | 1 entry | 1 row | 1 row × regimes | 0 | 0 | **0**, unless it changes what an agent must satisfy (§18.1 step 8) | 0–1 adapter | 0–1 |
| New agent type | 0–1 entry | 0 | 0 | 0 | 0 | 1 new module | 0 | 0 |
| New venue | 0 | field on affected rows | 0 | 0 | 0 | **0** | 1 adapter | 1 |
| New obligation shape | 0 | 0 | 0 | 0 | 0 | **0** | 0 | 0 |
| New contingency / amendment trigger | 0 | 1 field (optionality) | 0 | 0 | 0 | **0** | 0 | 1 (owns the event test + index) |
| New *question* about instruments | 0 | **1 column × every type** | 0 | 0 | 0 | consumers of it | 0 | 0 |
| New regime | 0 | 0 | **1 column × every type** | 0 | 0 | 1 declaration | 0 | 0 |
| New observed series | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 declaration |
| New conservation law | 1 entry | 1 row | 0 | 0–1 | **1 verb + 1 counter-account with owner** | **0** | 0 | 1 |

---

## 19. Delivery plan

Phases have **entry and exit criteria**. A phase is not finished because time has passed.

### Phase 1a — The substrate experiment (target: one fortnight)

The purpose is to falsify or confirm A1 and the no-escape-hatch claim as fast as possible, on the
smallest thing that can carry the argument. Scope is deliberately minimal:

Branded identifiers; safe-integer quantities with the §6.3 unit table and both rounding rules; the
holdings table; the six verbs of §6.1 with write-door preconditions; liens to depth 3; the seven
counter-account families with their declared owners; the journal ring buffer; the toy-world
conformance suite (§15.1) including the insolvency case; a first measurement of journalling cost
against the 250 ms budget.

*Not in 1a:* code generation, typed column storage, the state differ, the row inspector, persistence,
the platform probe, the benchmark harness, the observation store. Those are 1b. The fortnight is
credible only because they are excluded.

*Exit criteria:* the toy suite passes, including wind-up and amendment; no escape hatch was needed;
journalling cost measured against the §3.4 budget.

**Kill criterion, stated in advance:** if the ledger requires an escape hatch for an ordinary
operation, or if journalling costs more than the stated share of the period budget at §3.4 volume,
the design's central claim has failed and the plan must be revised before Phase 1b.

### Phase 1b — The substrate proper

*Exit criteria: typed column storage in place, the state differ and row inspector work, save/reload
round-trips bit-identically, the platform probe runs in CI, the benchmark exists with a published
noise floor.*

Code generation; typed column storage; the state differ and row inspector; persistence with schema
identifier; the observation store skeleton; the platform probe; the benchmark harness; the batch
forms of every verb — written now and reviewed as carefully as the single forms, because batching is
where a bypass gets added later.

No fortnight is claimed for 1b. It exits on criteria.

### Phase 2 — Instruments, obligations and prices

*Exit criteria: an instrument can be issued, held, priced, pay a full irregular schedule to multiple
holder classes with no holder-specific code, and have that schedule amended by a default and by a
prepayment. The opening world generates from primitives alone (§13) and reaches period 260 without a
seeded price or a seeded term.*

The intrinsic and relational fact tables as total mappings; obligations as rows with due-period
indexing; `amend` with per-mechanism capabilities; the price table with epoch-typed reads; the market
solver and one adapter; the opening world built by ledger operations from primitives (§13).

**A scope review sits at the end of Phase 2**, with an explicit continue/stop decision (R8).

### Phase 3 — First agents and the period loop

*Exit criteria: a period runs end to end with two agent classes; the golden digest is stable; the
manifest/order consistency check passes (§9.3); the budget-allocation decision (§9.2) is recorded.*

Systems and manifests; capability minting; the period order as a committed list; two agent types
(one intermediary, one allocator) implemented purely as the five declarations.

### Phase 4 — Economic content

*Exit criteria: per system, the §17.4 definition of done.*

One system at a time, specification first. Sequence by dependency: money and settlement, then credit,
then equity, then the public sector, then the remaining sectors. Throughout, the standing benchmark
(N2) and the long-run benchmark (N3) run nightly.

**Native or parallel acceleration is not in the plan.** It is considered only if the standing
benchmark demands it, and only under §12.3.

---

## 20. Risks

| # | Risk | Impact | Mitigation |
|---|---|---|---|
| R1 | Integer quantities prove awkward for some asset class; team is tempted to make an exception. | A1 fails silently; the audit harness returns. | Units and ranges settled per class (§21, decision 2) and asserted at the write door. An exception is an ADR under §21.3 naming what replaces A1 for that class. |
| R2 | Phase 1 over-engineers ahead of need. | Schedule loss; complexity with no consumer. | 1a/1b split; exit criteria are behavioural. Anything not exercised by the toy suite is deferred to 1b or later. |
| R3 | Agents acquire cached aggregates for performance. | N×M returns invisibly. | Checklist item 4; a lint rule on agent modules; treat as a storage-layout problem instead. |
| R4 | Manifests are over-declared under deadline pressure. | Ordering hazards return; counter-account ownership dilutes. | Declaration-width budget reviewed at merge; visible in the diff; checklist item 11. |
| R5 | Specifications drift from code. | The only instrument that finds absences stops working. | §17.4 requires reconciliation in the same change; a stale specification blocks approval. |
| R6 | The golden digest is re-baselined casually. | Regressions hide inside intended changes. | A digest change must name which field families moved and why; reviewer signs off on that list, not on the digest. |
| R7 | Performance target missed late, after the model exists. | Expensive rework of storage decisions. | §3.4 is settled, not discovered; benchmarks from Phase 1b; N3 nightly from Phase 3; workload re-examined at the Phase 2 scope review. |
| R8 | Greenfield never reaches useful scope. | Total loss. | Phase exit criteria; explicit continue/stop scope review at the end of Phase 2. |
| R9 | Economic knowledge is re-derived ad hoc rather than specified. | Known problems are re-encountered as new ones. | Specifications written before each system; see ADR-001 §3. |
| R10 | A3 erodes one `assumed` value at a time, each chosen to look like reality. | The predecessor's defining failure returns with better plumbing. | No `measured` provenance; source citations fail the build; levels require derivation; checklist item 5; placeholder count on the dashboard. |
| R11 | The observation store becomes shadow state that decisions read. | Stored aggregates return through the reporting door. | No engine-side read handle exists; anything a decision needs is a state column with an owner (§14). |
| R12 | `amend` becomes a general-purpose obligation write path. | A second write path; F8 and N3 both degrade. | Amendment capability minted per triggering mechanism only; no general handle; rows are never deleted. |
| R13 | The surface re-implements a derived number. | One fact, two representations, in the layer the user actually reads. | §4.4; lint on arithmetic in `surface`; checklist item 10. |
| R14 | A settled decision drifts rather than being superseded — a cap raised, a regime added, a series list grown, inside a commit about something else. | The document stops describing the system, which is how §15.2's only absence-finding instrument stops working. | §21.3; each pressured decision has a mechanical guard (assertion, precondition, benchmark, hard cap, compile error); a diff touching §21 values without an ADR is refused. |

---

## 21. Settled decisions

Fifteen decisions had to be made before work could start. They are made. Each is stated here with
what was chosen, why, and what it costs, and each is applied in the body of the document at the point
where it bites. Nothing in Aurora is `TBD`.

### 21.1 Language and runtime

**Decision: TypeScript, strict, compiled to a single ES module, with typed-column storage over
`ArrayBuffer`.** Quantities in `Float64Array`, identifiers and enumerated codes in `Int32Array`,
prices and rates in `Float64Array`.

*Why.* The presentation surface is a TypeScript application, and a second language would create
exactly the boundary §12.3 forbids unless both sides are generated from one schema. TypeScript
supplies what the design needs: branded identifiers (§5.2), compiler-checked exhaustive matching over
discriminated unions, which is what makes the total mappings of §7.2 and §7.3 enforceable, and
`ArrayBuffer`-backed columns with predictable layout. It runs on the target device without a
toolchain the platform lacks (N4).

*What it costs, stated plainly.* No value types, so §5.3's protection stops at boundaries; no integer
type, which forces decision 2; and a garbage collector, which makes allocation the thing to watch in
§12.2 rather than arithmetic. A compiled language would strengthen §5.3 and weaken §12.3. The
boundary cost was judged larger than the type-safety gain, and §17.2's naming rules are the
compensating control.

### 21.2 The register

| # | Decision | Settled as | Where it is applied |
|---|---|---|---|
| 1 | Language and runtime | TypeScript, strict, typed columns over `ArrayBuffer` | §21.1, §5.2, §5.3, §12.4 |
| 2 | Integer width and unit per asset class | Safe-integer float64, magnitudes below 2⁵³; currency in 1/100 base units; shares, goods, capital, dwellings, hours in whole units; goods carry a scaling exponent. Three orders of headroom against §3.4. | §6.3 |
| 3 | Rounding and residues | Half-to-even on a single move, so no residue exists; largest-remainder with ascending-identifier tie-break on any distribution, so the residue is assigned to named parties. No third case. | §6.3 |
| 4 | The verb set | Six: `move`, `pledge`, `release`, `amend`, `register`, `observe`. Batch forms of each. A seventh is an ADR. | §6.1 |
| 5 | Rehypothecation depth | 3 — the shortest chain whose unwind has an interior. Breach raises at the door. | §6.5 |
| 6 | Counter-account registry | Seven families with seven owners: `Issue:<currency>`, `Issue:<instrument>`, `Endowment:`, `Production:`, `Consumption:`, `Wear:`, `Writeoff:`. Labour uses `Endowment:`/`Consumption:`, not a family of its own. | §6.2 |
| 7 | Target workload | Weekly periods; 1,560-period runs; 260-period burn-in; ≈12,130 entities; 60k instruments; 1.2 M obligation rows; 40 venues; 250 ms period budget; ≤15% flat-cost drift; 900 MB peak. | §3.4 |
| 8 | Journal retention | Two periods, fixed 1.2 M-row ring buffer, ≈60 MB. Exhaustion is a defect, not a wrap. | §6.6 |
| 9 | Observation store | Ten families, hard cap of 512 series, ≈6.4 MB against a 20 MB allowance. A new series above the cap retires one. | §14 |
| 10 | Intrinsic question set | The thirteen in §7.2, frozen as v1. A fourteenth is a reviewed change with a column added across every type in the same commit. | §7.2 |
| 11 | Regimes and relational facts | Five regimes — `bank-prudential`, `insurer-solvency`, `fund-unconstrained`, `household`, `sovereign` — × three relational questions. | §7.3 |
| 12 | Who may amend obligations | Five mechanisms, five owners: default and prepayment (credit), call (primary issuance), cancellation (wind-up), crystallisation (insurance). Coupon resets are not amendments. | §7.4 |
| 13 | Opening primitives | Populations, endowments, technology and preference coefficients, policy slopes, institutional counts, and one numéraire scale per currency. **No financial instruments and no prices in the seed.** | §13 |
| 14 | Budget allocation across venues | Pre-allocated per venue, with an explicit reconciliation step. Intra-period reallocation is given up deliberately. | §9.2 |
| 15 | Simultaneity | Decisions read prior-close; clearing reads submissions; everything from settlement onward reads this-close. The boundary sits at one place in the committed period order. | §9.3, §9.4 |

### 21.3 How a settled decision is superseded

None of the above is permanent, and all of it is closed. The difference matters: a closed decision is
one that engineers build against without reopening it in a code review.

To change one: write an ADR that names the decision, states what has been learned that the original
rationale did not account for, lists every section of this document the change touches, and states
the migration cost for work already done. It is approved or refused as a whole. What is not permitted
is drift — a value quietly widened, a cap raised in a commit that was about something else, a sixth
regime appearing because one instrument was awkward. Decisions 2, 5, 7, 9 and 10 are the ones most
likely to be pressured that way, and each has a mechanical guard: a range assertion, a door
precondition, a benchmark, a hard cap, and a compile error respectively.

Two decisions carry an explicit review point rather than waiting to be challenged:

- **Decision 7 (workload)** is re-examined at the end of Phase 2, when the first real row counts
  exist. The numbers here are design targets, and the honest expectation is that obligation-row
  volume is the one that moves.
- **Decision 14 (budgets)** is re-examined once two agent classes are live in Phase 3, since that is
  the first point at which the cost of forgoing intra-period reallocation is observable rather than
  argued.

---

## Appendix A — Failure modes this design is hardened against

Recurring structural failures in large simulation systems, and the section that addresses each. When
something goes wrong, it is usually one of these wearing a costume.

| # | Failure mode | Addressed by |
|---|---|---|
| 1 | One fact in two representations, with nothing forcing agreement. | §5.1, §5.4, §6.1 |
| 2 | A derived view stored beside its source and read as current. | §5.4, §10.3 |
| 3 | Identity as a plain string, so a wrong key returns a plausible default. | §5.2 |
| 4 | Quantity and value sharing a numeric type — equal exactly while price is par, which is when getting it wrong is free. | §5.3 |
| 5 | A bound or an iteration cap standing in for a missing mechanism, hiding the absence it covers. | §16.1, §9.1 |
| 6 | Value moved by assigning a field; a write is indistinguishable from a transfer. | §6.1 |
| 7 | A residual used as a holder — always balances, so it can never report that the sum is wrong. | §6.2 (the four properties that separate a counter-account from a plug) |
| 8 | Verification that cannot fail: both sides from one source; a check in a language that cannot see its subject; a comparison against an undefined value. | §6.6, §15.1 |
| 9 | Declared vocabulary no writer ever produces, so the type describes a richer world than the code. | §15, tier 2 |
| 10 | State rebuilt from a fixed field list, silently dropping whatever was added later. | §5.1, §13 |
| 11 | The same read written in many places, drifting apart; the dangerous copy is the one missing one clause. | §4.3, §4.4, §7.4 |
| 12 | Positional coupling across a boundary, with names living only in comments. | §12.3 |
| 13 | Per-period work proportional to accumulated stock rather than to activity. | §7.4, N3 |
| 14 | Two acceleration strategies against one seam, so neither can be enabled. | §12.3 |
| 15 | Shared static configuration mutated as per-entity state, so the last writer wins globally. | §4.3, §17.1 |
| 16 | A number chosen because it resembles an observed figure, becoming permanent structure. | A3, §13, §16.1 |
| 17 | A derived quantity computed a second time in the presentation layer. | §4.4 |
| 18 | Dead scripts and abandoned refactors left in the tree, indistinguishable from live code. | §17.1 |

---

## Appendix B — Glossary

| Term | Meaning here |
|---|---|
| **Agent** | An entity that makes decisions: a firm, bank, fund, insurer, household, government. |
| **Asset** | Anything that can be held in a quantity: currency, security, physical good, capital, dwelling. |
| **Burn-in** | The declared number of opening periods whose output is not treated as a result, required because nothing is calibrated (A3). |
| **Counter-account** | A named, singly-owned holder representing a source or sink, so creation and destruction are transfers (§6.2). |
| **Conserved** | A quantity whose total across all holders cannot change except by moving it. |
| **Encumbrance / lien** | A claim by one party over units held by another, without transfer of holding. |
| **Holding** | A row in the single quantity table: (holder, asset, quantity). |
| **Instrument** | A specific issued thing that can be held and priced, with declared facts and obligations. In Aurora none exists at period 0; all are issued in-model (§13). |
| **Intrinsic fact** | An answer that is true of an instrument regardless of who holds it (§7.2). |
| **Manifest** | A system's declaration of what it reads and writes, from which its access handles are minted. |
| **Obligation** | A scheduled future payment or delivery arising from an instrument, stored as a row. |
| **Observation** | A period-indexed derived series, written by one system, never read by the engine (§14). |
| **Numéraire scale** | The single currency quantity issued at period 0. A pure scale factor: doubling it doubles every nominal quantity and changes nothing real (§13). |
| **Period** | One discrete step of the simulation. One week (§3.4). |
| **Placeholder** | A registered constant standing in for a mechanism not yet built (§16.1). |
| **Plug** | An anonymous, universally writable residual. This design has none; see §6.2. |
| **Position** | A read view over holdings restricted to instrument assets. Not a table. |
| **Regime** | The regulatory or accounting context under which relational facts are answered (§7.3). |
| **Relational fact** | An answer that depends on the holder's regime as well as the instrument (§7.3). |
| **Reservation level** | The price at which a participant is willing to transact, and the size it scales into. |
| **System** | A module doing one part of a period's work, over declared reads and writes. |
| **Verb** | One of the closed set of ledger operations; the only way state changes. |

---

## Appendix C — Revision history

| Version | Date | Change |
|---|---|---|
| 2.0 | — | Initial architecture and project preparation document. |
| 2.1 | 5 Sep 2026 | Review response. **Structural:** merged `accounts` and `positions` into one `holdings` table (§6.1); bound every counter-account to exactly one owning system with minted write capability, and stated the four properties separating a counter-account from a plug (§6.2); split instrument facts into intrinsic (§7.2) and regime-keyed relational (§7.3), and added currency, issuer, tenor, accrual and optionality; made obligation amendment an explicit capability-scoped verb covering default, prepayment, call and cancellation (§7.4); added §6.7 wind-up, satisfying F6; added A3 (no exogenous calibration) with registry-level enforcement (§3.2, §13, §16.1); added §14 observation store, resolving short journal retention against long-horizon reporting (F9). **Corrections:** A1 given a mechanical discharge rather than a review question alone (§3.2); removed the unassertable "no verb can be found" assertion from the toy suite (§15.1); scoped the order-independence claim to conserved quantities only (§6.3); ran the covered-bond example to its honest cost, including one bank constraint edit (§18.1). **Additions:** §3.4 target workload; §4.4 surface computes nothing; §5.2 identifier birth/death policy; §9.3 simultaneity rule; identifier and language assumptions made explicit (§21.1). **Document:** corrected the layer diagram to match the layer table; moved the greenfield rationale to ADR-001; added contents, version, revision history; extended the change-cost table, review checklist, risk register (R10–R13), failure-mode appendix and decision list. |
| **3.0** | 5 Sep 2026 | **Baselined as Project Aurora.** All fifteen open decisions closed and recorded in §21 with rationale and cost: language and runtime (TypeScript, typed columns over `ArrayBuffer`); safe-integer float64 quantities with a per-class unit and range table; two rounding rules and no third; a six-verb closed set; rehypothecation depth 3; a seven-family counter-account registry with owners; the full §3.4 workload (weekly periods, 1,560-period runs, 260-period burn-in, ≈12,130 entities, 40 venues, 250 ms, ≤15% drift, 900 MB); two-period journal retention in a fixed ring buffer; ten observation families under a hard 512-series cap; the intrinsic question set frozen at thirteen; five regimes × three relational facts; five amendment mechanisms; the opening primitive set, with **no financial instruments and no prices in the seed**; pre-allocated venue budgets with explicit reconciliation; and the simultaneity rule mapped to system families. Added §9.4, the committed period order. Added §21.3, the supersession process, and R14 against decision drift. ADR-001 folded in as Appendix D. |

---

## Appendix D — Decision record: greenfield rather than continued remediation

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 5 September 2026 |
| **Decision** | Build Aurora from scratch against this document. Port no code from the predecessor. Transfer knowledge as written specifications. |
| **Supersedes** | Continued incremental remediation of the existing system. |

Recorded so the choice is kept with its reasoning, and so the conditions under which it would have
been the wrong choice are written down in advance rather than argued afterwards.

### D.1 The test that decides it

Sort the outstanding work on any existing system into three categories:

- **(a) Missing mechanism** — something that should be modelled and is not.
- **(b) Wrong representation** — the right thing is done, but expressed so that it can be read
  wrongly; the fix is "find the N places that do this and change each one."
- **(c) Wrong number** — a mechanism exists and is calibrated badly.

A rebuild pays only when **(b) dominates**. It builds no mechanisms and calibrates nothing; those
costs are identical either way. What it buys is the permanent elimination of a class of work.

### D.2 Why (b) does not converge

The properties Aurora is built on — *exactly one way to write a balance*, *no agent enumerates
instruments*, *no fact has two representations*, *no number is chosen to resemble an observed figure*
— are **global invariants**. A global invariant holds only at zero violations.

Incremental cleanup reduces the count; it cannot hold it at zero, because nothing structural prevents
the next violation being added tomorrow. So the work has a shape that is recognisable but rarely
named: *converging to zero on a quantity with no mechanism holding it at zero.* Each step is real
progress and none of it compounds, because the entire value of "there is only one way to do this" is
that you can rely on it without checking. Meanwhile the system grows, so the number of sites per
property grows with it, and each property costs more than the last.

Three observable signals say the same thing.

1. **Being required to build an audit apparatus at all.** Every reconciliation pass is an admission
   that an invariant is maintained by discipline rather than by construction, and the count of such
   passes measures how much correctness the architecture failed to supply.
2. **Checks that cannot fail** — comparing a value with itself, testing a condition that has become
   undefined, matching text patterns that no longer match anything. These appear specifically once
   the checking apparatus is too large to hold in mind. Finding one is not a bug to fix; it is a
   reading on the instrument panel.
3. **Containment in place of diagnosis** — an iteration cap added to stop a hang, a clamp added to
   stop a divergence, a guard added to stop a non-finite value propagating. Each is a mechanism whose
   absence has been made survivable rather than found.

### D.3 Why nothing is ported

The instinct is to carry the economics across and rebuild only the substrate. It is rejected for
architectural rather than aesthetic reasons.

**Ported code carries the assumptions it was written under.** A system written against mutable
objects, stored aggregates, per-class fields and seeded real-world levels arrives expressing exactly
the shapes Aurora exists to eliminate, and the path of least resistance is to add an accommodation
for it. Since the entire benefit is a *global* property (§D.2), a single accommodation costs more
than the port saves. Code is the main vector by which old invariants return.

What transfers instead is **knowledge, as specification**. Every mechanism worth keeping is written
down as a specification — from the domain, in domain language, with no implementation open — and
built fresh from that. This is the same artefact §15.2 requires anyway as the only instrument capable
of finding absences, so the cost is not additional: it is work that must happen regardless, sequenced
first.

This is the more expensive path in raw effort and it carries the highest schedule risk. R8 and R9 are
its risks, and the phase exit criteria are their mitigation.

### D.4 The precondition

Before committing beyond Phase 1a: **demonstrate the no-audit property on a toy world in the first
fortnight** (§15.1). A handful of parties, one currency, one good, one bond, one secured loan with a
rehypothecation hop, one insolvency with a shortfall. Show that value cannot leak because no verb
leaks it; that over-pledging cannot be written; that an estate distributes exactly and the remainder
is extinguished visibly; that creation and destruction are readable balances on named, singly-owned
accounts.

If that works, the rest is engineering and this document is a plan.

If it does not — if the ledger needs an escape hatch on day one, if the write door must be opened for
something ordinary, if conservation turns out to need a tolerance — then the central claim has failed
and the plan must be revised before the model is built on top of it. That is the Phase 1a kill
criterion, and it is why the substrate is proved before a single economic system exists.

### D.5 Conditions under which this decision was wrong

Recorded now, so the judgement can be checked later rather than defended:

- **If (a) turns out to dominate (b).** If most of the outstanding work is missing mechanisms rather
  than wrong representations, the rebuild bought nothing and cost a substrate.
- **If Phase 1a needs an escape hatch.** Then A1 is not achievable in TypeScript, and the
  predecessor's audit apparatus was not a symptom of bad architecture but a necessity.
- **If Phase 2's scope review shows the specification effort exceeding the remediation effort it
  replaced.** Specification-first is the transfer mechanism and also the largest untimed cost line in
  the plan; if it dominates, the economics of the decision inverts.
- **If the same failure modes reappear in the new codebase.** Appendix A is the checklist. Recurrence
  would mean the failures were a property of how the work is executed, not of what it was executed
  against, and no amount of architecture fixes that.
