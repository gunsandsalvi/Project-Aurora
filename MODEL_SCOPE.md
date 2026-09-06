# Model scope — the feature baseline

**Version 1.0.** This document is the **breadth** target. `PROJECT_AURORA.md` says how the model is
built; `IMPLEMENTATION.md` says in what order; this says **what has to be in it**.

It is derived from the predecessor project's master plan — a working model of ~230 files and ~55k lines
with ~45 mapped systems. **Nothing about that project's implementation is carried over**: its own audit
found that money and ownership did not close, that price was not universal, that value was a stored field
in ten of eleven asset classes, and that the instrument measuring all of it was itself broken. Those are
the defects Aurora's architecture exists to make unwritable.

**What is carried over is the surface.** That model reached breadth this specification does not yet
describe, and reaching it again is the target. Read this as a checklist of *what must eventually be
expressible*, not as a design.

---

## 0. How to use this

- **A row here is not a work item.** It is a capability the model must reach. The milestone column says
  where it lands; `IMPLEMENTATION.md` owns the sequencing.
- **Most rows cost nothing structural.** §18's change-cost table is the whole point: a new instrument is
  one intrinsic row plus one relational row per regime and **zero agent edits**; a new line is one
  registry row and no code. The rows that cost more are marked, and they are the interesting ones.
- **A row Aurora already does better is marked ✓✓.** The predecessor is a scope baseline, not a
  quality one, and there are places where copying it would be a regression.

---

## 1. What the predecessor got structurally wrong, and Aurora already forbids

*Recorded first, because the temptation when reading a feature list is to import the shape that carried it.*

| The predecessor's defect | What Aurora does instead |
|---|---|
| Value stored as a field in ten of eleven asset classes, so nothing could be re-marked | `value = units × price(asset)`, a function, never a field (§5.4) |
| Money and ownership did not close; residuals with no holder | A4, R-1, counter-accounts with four owners (§6.2, §6.8) |
| Credit traded at par; commodity spot was a drift formula | One clearing interface, every line (§9.1) |
| Loss *rates* instead of default *events* — four systems ran on `principal × PD × LGD / 52` | Default is a modelled outcome with a date, an estate and a recovery (§6.7) |
| Bounds, clamps and `Math.max(0, …)` standing in for missing mechanisms | Modelled outcomes are values in the type; defects raise (§16.2) |
| Employment was an integer headcount; a worker was a fraction of five occupations | ✓✓ Employment is a contract instrument, one worker, one wage, a term (§9.6.2) |
| SME pools where entry was the accounting identity of exit, so population was constant by construction | ✓✓ Representative agents abolished; a row is one firm (§8.4) |
| One credit rating, a property of the firm, held by nobody | §8.1 declaration 4 — valuation is the agent's own |
| Two representations of one fact, everywhere (the master plan's rule 4 and rule 19 exist to fight this) | One fact, one place; one writer per column (§4.3, §6.1a) |

**The single most useful sentence in the predecessor's audit**, and the one to keep in view for the whole
build: *"Nothing in this model is ever forced to sell anything, which is why no shock has ever propagated
through a price."* Aurora's forced-seller path is §6.4's preconditions plus §6.7's resolution, and it must
actually run.

---

## 2. The feature surface

Status: **structural** = Aurora's current text already carries it · **additive** = instrument rows, line
registry rows, or an agent declaration, at §18's stated cost · **new structure** = needs a new agent
class, venue family, counter-account family, or a change to a settled decision.

### 2.1 Markets and venues

| Capability | Status | Notes |
|---|---|---|
| Goods auction per sub-unit, per-lot settlement, contracts | structural | §9.5 goods venue; the sub-unit count is §13.1.2's reopened question |
| Retail consumption | structural | §9.6.3 |
| Labour, by occupation, with vacancies and matching | structural | §9.6.2 ✓✓; occupational *mismatch and retraining* is additive |
| Property: dwellings, tenancies, floor area, leases | structural | §9.6.4 |
| Installed capital resale, distinct from new | structural | §9.6.4 |
| Sovereign issuance across tenors, with a curve | structural | §9.5 |
| Corporate credit — buckets and single names | structural | §9.5; the bucket/name split follows listing (§8.7) |
| Household credit | structural | §9.5 |
| Equity, on a listing rule | structural | §8.7 |
| Money market / overnight | structural | §9.5 |
| FX spot, six pairs, no world price | structural | §9.5, §9.7 |
| **Repo and secured funding** | additive | Aurora's liens carry rehypothecation to depth 3 (§6.5) — repo is a lien plus a contract, and the machinery exists |
| **Interbank unsecured** | additive | a line on the money-market venue |
| **Commodities spot** | additive | the predecessor made commodity spot a *read* of the goods auction; that is Aurora-shaped. A commodity needs a stock, a location and a holder — the predecessor had none of the three |
| **Bills and commercial paper** | additive | discount paper; one instrument type |
| **Securities lending** | additive | a lien plus a manufactured payment; the predecessor inverted the economics |
| **Derivatives: IRS, CDS, futures, options** | new structure | see §3.1 |
| **Securitisation: pools, tranches, waterfalls** | new structure | see §3.2 |
| **Insurance** | additive | §7.4 already names insurance as an amendment owner (claim crystallisation) |
| **Trade credit, invoices, factoring** | additive | a receivable is an instrument with terms |
| **M&A** | additive-plus | an exchange of equity for cash is expressible; a *bid with acceptance and rival bidders* needs a venue shape §9.1 does not obviously have |
| **Freight and shipping lanes** | new structure | a network with capacity per lane. Aurora has no concept of a route |

### 2.2 Agents and institutions

| Capability | Status | Notes |
|---|---|---|
| Households | structural | |
| Firms, listed and unlisted, with promotion | structural | §8.7 |
| Banks | structural | |
| Central banks | structural | |
| Governments / treasuries | structural | |
| Funds — mutual, money-market, hedge, ETF, private equity | structural | one `Fund` class; what distinguishes them is the units they issue (redeemable on demand vs dated) and what they hold. Same test that merged insurer and pension fund |
| Insurers and pension funds | structural | one `Liability-matched institution` class |
| **Dealers / market makers** | **new structure** | see §3.3 — the most important missing class |
| **Clearing house (CCP)** | new structure | a party with a balance sheet that holds margin, novates, and runs a default waterfall |
| **Prime brokers** | additive | a bank's activity, not a class |
| **Securitisation vehicles (SPVs)** | new structure | §6.5's trustee *holds* for a class of holders; an SPV *issues* against a pool |
| **Rating agencies** | additive | or not a party at all — see §3.4 |

### 2.3 Mechanisms — credit and default

| Capability | Status |
|---|---|
| Origination, underwriting as a constraint | structural (§8.1 declaration 3) |
| Default as a dated event, not a rate | structural (§6.7) |
| Estate, waterfall, ranked claims, recovery | structural (§6.7) |
| Seniority that changes the *payout*, not only the price | structural (§7.2 Q11) |
| **Covenants** — a term of the instrument an issuer can breach | additive (an option family under §7.6, or a fourteenth question — decide at M3) |
| **Acceleration** — default makes principal due | additive (an `amend` mechanism; §7.4 has eight and a ninth is an ADR) |
| **Restructuring / exchange offers / holdout creditors** | additive-plus (a ninth amendment mechanism plus a negotiation the predecessor never had) |
| **Provisions** distinct from realised losses | additive |
| **Foreclosure** — a seized dwelling returning to supply | structural (a `move` at resolution) |
| **Lending standards that tighten** — LTV and DSTI as reads, not constants | structural (§8.1 declaration 3) |
| **Trade creditors ranked in the estate** | structural — and the predecessor's bias here is a worked warning: its estate *collected* the dead firm's receivables as an asset while trade creditors ranked nowhere, biasing every recovery high |

### 2.4 Mechanisms — banks and the official sector

| Capability | Status |
|---|---|
| Capital ratio, cure window, resolution | structural (§8.1 declaration 5, §6.7) |
| **A cost of funds that enters the loan price** | structural — and the predecessor's absence of it is why "no price could reach a decision" |
| **Bank equity issuance, and a subordinated layer to bail in** | additive |
| **Deposit pricing against a money-fund alternative** | additive |
| Liquidity, lender of last resort, swap lines | additive |
| **Large-exposure limits** | additive |
| Policy rate as a rule over the model's own state | structural (§13.4) |
| **Administered rates with a real quantity response** (standing facility, reverse repo) | additive — the predecessor's rule 3 allowed exactly one exception for these, and it is a good one |
| Sovereign issuance calendar, auctions that can fail | additive |
| **A treasury that cannot overdraw its central bank** | structural — A4 forbids the plug the predecessor had |
| **Sovereign default, exchange offers, market exclusion** | additive |
| Central-bank remittance, and a deferred asset when it loses money | additive |

### 2.5 Mechanisms — firms and production

| Capability | Status |
|---|---|
| Production from a recipe | structural — **but see §3.5: the predecessor's recipe was a value share, not a physical one** |
| Plant as dated vintages, with depreciation by kind | structural (§9.6.4, §6.2 `Wear:`) |
| Capex with a commissioning lag | additive |
| **Product line entry and exit** | additive (§18: a new line is one registry row) |
| Inventory at cost, with a basis; lower of cost and net realisable value | additive — the predecessor reached this and it was hard-won |
| COGS as the units that left; idle capacity as a period cost | additive |
| **Investment against a hurdle, from the firm's own cost of capital** | structural (§8.1 declarations 4 and 5) |
| Firm birth, death, and age that enters an assessment | structural (§9.4 position 1) |
| M&A with funding, acceptance, rival bidders, and real synergies | additive-plus |
| Guidance as the management's own published expectation | additive |

### 2.6 Mechanisms — households and labour

| Capability | Status |
|---|---|
| Consumption, labour supply, tenure, portfolio | structural (§8.1) |
| Deposit / money-fund / bill substitution | additive |
| Mortgages, arrears, foreclosure | structural |
| **Inheritance** | additive |
| Participation as a response to the wage | additive |
| Quits, vacancies withdrawn, separation with a cost | structural (§9.6.2's `Termination`) |
| **Occupational mismatch and retraining** | additive |
| Wage stickiness as a property of the contract | structural ✓✓ (§9.6.2's fixed margin and review tick) |

### 2.7 Cross-border

| Capability | Status |
|---|---|
| Trade financed on modelled FX lines | structural (§9.7) |
| **Balance of payments as a read, never a stored field** | structural (§9.7's flow-of-funds is most of it) |
| **FX forwards, swaps, cross-currency basis** | additive — and the predecessor's warning: a forward whose rate is spot moved by a basis, carrying no interest differential, is not a forward and cannot be checked against parity |
| **No vehicle currency by construction** | structural (§9.7: no world price, six pairs clear independently) |
| Triangular arbitrage as an outcome, not an identity | structural (§9.7) |

### 2.8 Measurement

| Capability | Status |
|---|---|
| CPI | structural (§13.4's published index) |
| **A separate PPI** | additive — the predecessor had one index wearing both names, so it had no margin story |
| Equity indices, and betas measured against something real | additive — the predecessor measured every beta against a *random walk* for a year |
| Closure panel, sectoral balances, counter-account flows | structural (§14) |
| **Curves beyond the sovereign** — secured, swap-spread, credit by rating, commodity, basis | additive |

---

## 3. The rows that cost real structure

### 3.1 Derivatives, margin and a clearing house

§7.6 gives seven *option-terms* tables for options embedded in an instrument. It does not give a
**bilateral contract with a reference, a mark, and variation margin** — which is what a swap, a CDS or a
future is. Aurora has position 16 (margin and collateral) and liens, so the pieces exist; what is missing
is the contract shape and a CCP that novates and holds margin.

**The predecessor's hardest-won lesson here is worth more than its code:** it built margin, novation, a
default fund and a waterfall, and then found that *margin was a stated rate, so it could not rise when it
mattered* — which deletes procyclicality, which **is** the contagion mechanism. Margin that responds to
the reference's own realised move is the requirement.

### 3.2 Securitisation

Needs a named vehicle party that **issues** tranches against a pool, a stated waterfall, and named
holders of each tranche. §6.5's trustee entity holds for a class of holders and is close but not the same
thing. Blocked on default being an event (it is, in Aurora) and on a loan being a row rather than a field
(it is, under R-1).

The event this system exists to produce — correlation worse than the tranching assumed, every holder hit
at once — is unreachable without real holders, which is exactly A4's point.

### 3.3 Dealers, and why this is the most important gap

**Aurora has no market-maker.** Every §8.1 agent is an end investor with a mandate. The predecessor's
atlas found the consequence and stated it precisely: three derivative books had two participants and
**both were hedgers**, so the cleared price was a function of regulatory gaps and never of a view, and a
week in which neither gap bound did not open the book at all.

Under **M6** that is disqualifying: a price that cannot move because somebody thinks it is wrong is not a
formed price. Something in the model must take a position because it disagrees.

Two ways, and the choice is owed at M7:
- **A dealer agent class** whose mandate is inventory and whose valuation is a two-sided quote. Costs one
  class and one module (§8.3).
- **No new class** — any agent may quote two-sided on a venue it participates in, and "dealer" becomes a
  behaviour rather than a kind. Cheaper, and more consistent with §8.4's rule that class is not an
  institutional label.

*The second is more Aurora-shaped and I would take it, but it needs the valuation declaration to be able
to express a spread rather than a single reservation, which §9.1's clearing interface currently does not
ask for.*

### 3.4 Ratings as opinions

The predecessor had one rating, a property of the firm, computed centrally, held by nobody — so every
participant agreed about credit by construction, which removes a source of the demand dispersion an
auction needs.

Aurora has no rating at all, which is *better* but not sufficient: §8.1 declaration 4 makes valuation the
agent's own, so a credit opinion is already the right shape. What is owed is that assessments **differ**,
and that the disagreement is what puts two sides in a book. Published agency ratings, if wanted, are one
more opinion among many and never the model's own.

### 3.5 The recipe, and the sharpest single warning in the predecessor's plan

Its `recipeInputs` was **cents per dollar of revenue**, and the physical draw was `neededUnits =
neededUSD / inputUnitPrice` — so **a price doubling halved the physical draw.** That is the strongest
possible substitution assumption, sitting where the modeller believed they had chosen Leontief, and it
was invisible from the code because it read as an ordinary units calculation.

Aurora's §13.1.2 is where this is decided, and the technology must be **physical**: what a process takes
in units, not what it costs in money. Anything else re-imports the defect.

### 3.6 Freight and lanes

A network with capacity per route is genuinely new: Aurora prices things, not distances. The predecessor
had lanes and found capacity set the *price* of distance and never the *quantity*. Whether Aurora needs
freight at all is a scope decision — under R20 the honest options are to model it or to declare it out of
scope on the surface, and there is no third.

---

## 4. Where the breadth lands

| Milestone | What of this scope it must carry |
|---|---|
| **M3** Seven Types That Pay | The instrument *vocabulary* must be sized against this document, not against the conformance suite. Covenants and acceleration are decided here — as an option family, an amendment mechanism, or a fourteenth question |
| **M4** Clearing Without A World | The venue and line taxonomy is sized against §2.1 — repo, interbank, commodities, bills, derivatives and insurance are lines, and the 4,096 cap is checked against the real count |
| **M7** The Deciding Tick | The dealer question (§3.3). Whether a two-sided quote is expressible in §8.1 declaration 4 and §9.1's submission shapes |
| **M8** Credit, Default And The Estate | §2.3 whole: covenants, acceleration, restructuring, provisions, foreclosure, tightening standards, trade creditors ranked |
| **M9** Property, Equity And The Rest | §2.5, §2.6, insurance, funds, securitisation (§3.2), derivatives and the CCP (§3.1) |
| **M10** Four Regions Trading | §2.7 whole: forwards, swaps, basis, balance of payments |
| **M11** Thirty Years At Full Class | §2.8: PPI, indices, curves beyond the sovereign |

**The honest consequence:** §2 is substantially larger than `IMPLEMENTATION.md`'s M7–M10 were sized
against. Those four milestones were estimated at 245–350 engineer-weeks for "the economics"; this document
is what the economics *is*, and the estimate should be re-derived against it at G2 rather than defended.

---

## 5. What this document deliberately does not carry over

- **The 45-system decomposition.** It was that model's file layout, and Aurora's layering is different.
- **Every number.** Ratios, shares, rates and thresholds from the predecessor are `assumed` values from a
  model that was calibrated in places; importing one would breach M2 and A3.
- **Its parked projects.** The columnar scale-up and the game layer were that project's engineering.
- **Its taxonomy of parties, ids and keys.** Aurora settles this in §5.2 and §13-BOOK's equivalent is M1.
- **Any mechanism the predecessor's own audit called absent, as though it were present.** Roughly a third
  of §2 is on this list — the predecessor *planned* these and had not built them. They are scope, not
  precedent, and this document marks no row as done.
