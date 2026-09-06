# M0 — The Refusing Workspace

**The first milestone.** Executable detail for `IMPLEMENTATION.md` §3's M0.

> **Question: can this project's rules be made mechanical before there is any code to break them, and
> what does the device actually do?**

**Size:** 55–65 engineer-weeks, 8–10 calendar weeks.
**Team:** 3–4 engineers plus a part-time economist/modeller.
**Deliverable:** a workspace that refuses, a probe that has run on the owner's phone, nine falsifiers
with written results, and nineteen ADRs.

**Landed so far:** the workspace and its twelve crates; the layer matrix, proved by a committed fixture;
the lint floor; the `surface`/`shell` split; five checks — `check-lints`, `check-surface`, `check-deps`,
`check-refs`, `check-adr` — each with negative fixtures, behind one `verify` command; the dangling-ref
ratchet; CI. ADRs 0001–0005. The git log says what each commit did.

---

## 1. The one constraint: no engine code

**Not one line of `kernel`, `domain`, `world`, `ledger`, `markets`, `agents`, `systems` or `runtime` is
written in this milestone.** Everything here is rules, generators, probes and paper.

The reason is mechanical, not stylistic. Roughly fourteen guards discharge A1–A4, and every one is cheap
on an empty tree and expensive on a written one: a guard introduced after the code it polices is
negotiated down to fit what already exists, which is how an exemption list acquires its first entry.
D2 makes most of those guards properties of the crate graph and the type system rather than lints — which
does not remove the ordering constraint, it sharpens it. **A crate graph is shaped once. Reshaping one
with forty modules already inside it is a refactor nobody schedules and everybody defers.**

The second reason is evidential. Nine of this project's load-bearing numbers can be attacked with paper
arithmetic or a few hundred lines of throwaway code. Two are already known to be wrong. Attacking them
now costs days; attacking them in month nine costs the same days plus everything built against them.

---

## 1a. How this file is maintained

**This file lists remaining work.** A completed work item is deleted in the commit that completes it, and
the commit message names it. When every table below is empty and §5's exit criteria pass, M0 is done.

**A defect found while working M0** is classified and written up as a new numbered step rather than fixed
in passing — into this milestone if it blocks G0, into a later milestone if that milestone owns the code,
or into `IMPLEMENTATION.md` §4's register if it is a defect in the specification. `IMPLEMENTATION.md` §8
has the rule and the reasoning.

---

## 2. Team and the first Monday

| Who | Owns | Monday morning |
|---|---|---|
| **E1** — engineer, systems | W1 workspace, W2 build machinery | `cargo new --lib` twelve times; write the dependency matrix |
| **E2** — engineer, Android/native | W3 probe and delivery | `cargo-ndk` toolchain up; empty APK to Releases by Friday |
| **E3** — engineer, tooling | W4 registry, W5 ADR machinery | the `Entry` type and its first compile-fail fixture |
| **E4** — engineer, floating | W6 falsifiers (code half) | the seed generator, standalone, against §13.3's table |
| **M** — modeller, part-time | W7 falsifiers (paper half) | the tick-0 hand trace through the committed order |

**Nothing blocks on hardware.** The owner already has the device; E2's pipeline is the long pole, and
the paper falsifiers need no device at all.

---

## 3. Workstreams

### W2 — Build machinery and CI

*Owner E1. Two checks remain, and each is blocked on the thing it would police.*

| | Task | Detail | Days | Done when |
|---|---|---|---|---|
| W2.4 | `check-generated` | every generator runs into a temp directory and the output is compared byte-for-byte with what is committed; each generated file carries an `@generated` header with its input hash. **Blocked: there are no generators until M1's column schema.** Written when the first one is | 2 | a hand edit to a generated file fails CI |
| W2.7 | `check-registry` | the seven rules of §16.1, each with its own compile-fail fixture. **Belongs with W4**, which builds the registry it checks | — | folded into W4.5 |

---

### W3 — The probe, and the way results get back

*Owner E2. Depends on W1.1. This is the long pole; start it Monday.*

| | Task | Detail | Days | Done when |
|---|---|---|---|---|
| W3.1 | `cargo-ndk` cross-compile | `aarch64-linux-android`, release profile with the shipping codegen settings, into a `cdylib` | 3 | an `.so` builds in CI |
| W3.2 | The shell APK | Minimal Kotlin activity: one **Run** button, a scrolling text view, a **Copy** button. `largeHeap="true"`, a foreground service so a long run is not killed. It calls one JNI entry point and displays what comes back | 5 | the APK installs on the owner's device and prints `{}` |
| W3.3 | Release delivery | The `probe` workflow attaches a signed debug APK to a GitHub Release with the commit SHA in its name | 2 | the owner can install from a link with no toolchain |
| W3.4 | Allocation ceiling | Allocate in 128 MB steps, **commit every page by touching it**, hold 60 s under a churn workload, record where it fails. Reserved-but-uncommitted address space tells you nothing on Android. **Host-meaningless; needs the device** | 2 | the ceiling is in the JSON |
| W3.8 | Transcendental bit-identity | The **host half is measured**: `ln` hashes to `b5d414b87dd05ab7` and `exp` to `a493d8dc7d53c03e` over the committed 4,096 inputs. What remains is the device half and the comparison | 1 | two hashes from the device, equal to the host's or not |
| W3.9 | Thermal soak | 15 minutes at full load, throughput per minute, battery temperature. N2b is a sustained figure and every other measurement here is a burst. **Needs the device** | 1 | the throttled/burst ratio is in the JSON |

**The output the owner copies back** — one object, printed once, with a Copy button beside it:

```json
{ "schema": "aurora.probe/1", "commit": "<sha>", "device": {...}, "runAt": "<iso8601>",
  "allocation":     { "ceilingMiB": 0, "heldSeconds": 0, "splitCeilingMiB": 0 },
  "operationCost":  { "moveNs": 0.0, "exchangeNs": 0.0, "journalAppendNs": 0.0, "noiseFloorNs": 0.0 },
  "blockOps":       { "insert": {"10":0.0,"256":0.0,"4096":0.0,"16384":0.0}, "remove": {...} },
  "storage":        { "seqWriteMiBs": 0.0, "chunkedWriteMiBs": 0.0 },
  "transcendental": { "lnHash": "", "expHash": "", "ciLnHash": "", "ciExpHash": "", "identical": false },
  "thermal":        { "burstOpsPerSec": 0.0, "soakOpsPerSec": 0.0, "ratio": 0.0, "peakBatteryC": 0.0 } }
```

---

### W4 — The parameter registry

*Owner E3. Depends on W1.1, W1.6.*

Built now, against a hand-drafted census, before the seed exists — because §12.1 makes every capacity a
registry entry and M1 writes every capacity there is. A registry arriving later arrives in debt.

**W4.1 and W4.3 as drafted broke §1's constraint, and the constraint wins.** Both named engine code —
an `Entry` type in `declarations`, and a unit vocabulary generated from `domain`'s quantity types — in a
milestone whose defining rule is that no engine code is written. Rewritten below rather than worked
around, per §1a. The registry in M0 is **data plus a `tools` checker**; the engine-side type and the
generated vocabulary land in M1, when the quantity types they depend on exist.

*Nothing of the point is lost.* What M0 is for is that the rules are mechanical before anything can
break them, and a checker over a data file achieves that as completely as a type would. The fixtures
become data fixtures the checker must reject **for their own stated reason**, which is a stronger test
than a compile failure: a compile error proves something was refused, and a checked message proves it
was refused for the right reason.

| | Task | Detail | Days | Done when |
|---|---|---|---|---|
| W4.3b | The unit vocabulary, generated from `domain` | The closed list M0 hard-codes becomes generated from the quantity types, so there is one dimension system and not two — Appendix C failure mode 1, occurring in §16.1 itself. **Deferred to M1** with the types | 2 | a registry unit naming no quantity type fails the build |
| W4.6 | The `capacity` namespace's read rule | `capacity` entries are unreadable by any agent, valuation or economic system. The split lands in M0; **the read rule needs systems to police**, so its check lands with the manifests | 2 | a `capacity` entry read from `agents` fails a check |

---

### W5 — ADR machinery

*Owner E3. Depends on W1.1.*

The specification requires an ADR about twenty times and defines none. Nothing else in this milestone can
be *recorded* until this exists.

| | Task | Detail | Days | Done when |
|---|---|---|---|---|
| W5.1 | Format and numbering | `decisions/ADR-NNNN-kebab.md`; `tools adr new` allocating from a committed counter, so a collision is a merge conflict and a one-line fix | 2 | two ADRs created on branches collide visibly |
| W5.3 | Coupling, scoped by maturity | Registered files require an ADR and a `Decision:` trailer when a *parsed value* changes. **While a schema file is under initial construction the coupling is off**, declared per file per milestone, and the milestone exit ratifies the whole file. Otherwise M1 produces one ADR per column and the process becomes the throughput limit | 3 | a value change without an ADR is refused on a ratified file and free on an unratified one |
| W5.4 | Appendix generation | Appendices A and B generated from front matter; the doc build fails if the committed appendix differs | 2 | a hand edit to Appendix A fails CI |

---

### W6 — Falsifiers that need code

*Owner E4. No dependencies beyond W1.1.*

| | Task | Days | Done when |
|---|---|---|---|
| W6.2 | **The burn-in conjunction.** B1/B1b/B2/B3 against synthetic AR(1), random walk, mid-window break, diverging ensemble, frozen path; then Monte Carlo ~1,000 synthetic 42-series panels and report the empirical false-pass rate of the conjunction. `0.95^42 ≈ 11.6%` under the true null | 5 | the false-pass rate is measured and a multiplicity correction is specified |

---

### W7 — Falsifiers that need paper

*Owner M, with E4. No dependencies. These are the cheapest evidence in the project.*

| | Task | Days | Done when |
|---|---|---|---|
| W7.1 | **The memory derivation.** One row per world table, capacity × width, summed against N4. Currently ~705 MB of the 1,488.3 MB is unaccounted and its largest term is the instrument row width §7.5 leaves unsettled | 4 | a table that sums, with every capacity a `capacity` registry entry carrying its arithmetic |
| W7.2 | **The tick-0 hand trace.** Walk the committed order for ticks 0–4 against **§13.6**, which now derives the bootstrap: no firm bids at tick 0 because §9.2 budgets against settled balances, the government is the only agent with one, and it spends by hiring on the labour line. The trace confirms the derivation position by position and settles the one thing §13.6 leaves open — who holds the capital and dwellings position 6 sources at tick 0 | 2 | a written trace for ticks 0–4, and a derived holding rule for the opening stocks |
| W7.3 | **The identifier census.** Per identity space, live and ever-issued over 1,560 ticks. §3.4's ≈971,000 against §5.2's implied ≈12,450,000 is a factor of thirteen and it sizes the directory, the digest walk and the save | 2 | one reconciled figure per space |
| W7.4 | **The position workload table.** Twenty-one rows of operation counts summing to a published total, clearing's sort cost separated from operation-call cost. §12's targets are decomposed against this and it does not exist | 3 | the table sums to a published total |
| W7.5 | **The journal row layout.** Field list for `move` and for `exchange` against 48 B. An `exchange` needs two parties, two assets, two quantities, two rates, a reason code and an actor | 1 | a published packing, or a published width |
| W7.6 | **Household block occupancy.** Enumerate what one household can simultaneously hold — deposit, mortgage negative, consumer credit, employment contract, tenancy, dwelling, pension claim, insurance claim, fund units — against the declared ten slots | 1 | a count, and a slot capacity that survives it or is changed now |
| W7.7 | **The registry cost of one economic system.** Write the complete registry for *credit* on paper and count the assumed entries. Under D3 this no longer breaches a cap; it establishes the *rate*, which is what M3 needs to know | 3 | a count, and an extrapolation to seven agent classes |
| W7.8 | **Household and bank behaviour, hand-simulated.** The five declarations for two classes, written out, and ten ticks simulated by hand. Called by one study the single highest-value item available anywhere in the project: it buys much of G3's signal for none of M7's cost | 5 | two written declarations and a ten-tick trace |
| W7.9 | **The instrument row A/B, on paper.** 44 B with a schedule directory against 148 B inline, against W7.1's completed derivation. §7.5 calls this a Phase 2 entry criterion; it is not, because it decides whether the schedule identity space exists at all, and identity spaces are M1 | 2 | an ADR, confirmed by measurement in M3 |

---

## 4. The nineteen ADRs

Enumerated, because "roughly eighteen ADRs" in an exit criterion is not a criterion.

| | Decision |
|---|---|
| 0006 | `i64` conserved quantities; overflow panics |
| 0007 | Holdings slot at 24 B; encumbrance derived from lien rows, not stored |
| 0008 | Journal row layout and rate precision |
| 0009 | Instrument row: 44 B with a directory, or 148 B inline |
| 0010 | Identifier census, and the directory sizing that follows |
| 0011 | Shard count always 64, a saved run parameter; thread count carries no semantics |
| 0012 | Arena is thread-shareable from the start, run single-threaded until M11 |
| 0015 | The output gap as a ratio; the transcendental ban widened to any digested path |
| 0016 | Digest cadence decoupled from checkpoint cadence |
| 0017 | Retirement queue drained every tick; capacity derived from max-per-tick × interval |
| 0019 | The burn-in multiplicity correction |

---

## 5. Exit criteria

Mechanically checkable. No "or restate by ADR"; no "reported" where "passes" is meant; nothing that
passes because the tree is empty.

5. `check-generated` fails on a hand edit to any generated file.
7. An ADR without a `guard` field fails `check-adr`. A parsed-value change to a *ratified* registered
   file without a `Decision:` trailer is refused; the same change to an unratified file is not.
8. Appendices A and B regenerate from ADR front matter, and the doc build fails when the committed
   appendix differs.
9. **The probe has run on the owner's device and returned a complete JSON document conforming to
   `aurora.probe/1`**, with no field null.
10. The transcendental comparison is decided: `identical` is `true` or `false`, and ADR-0015 records
    which and what follows.
11. W7.1's memory derivation sums, table by table, and every capacity in it is a `capacity` registry
    entry carrying its arithmetic.
12. The seed generator's output is committed beside §13.3's published table with every row marked
    reproduces or does not.
13. The burn-in conjunction's empirical false-pass rate is a measured number, and ADR-0019 specifies the
    correction.
14. The intrinsic facts table compiles with all thirteen answers for all seven opening types, and
    deleting one fails to compile.
15. All nineteen ADRs are accepted, each naming a mechanical guard.
17. **No file exists under the eight engine crates' `src/` other than `lib.rs`** — and
    `kernel/src/layer_probe.rs`, which is compiled only under `--cfg aurora_layer_probe` and must fail.
    The constraint of §1, as a check.

---

## 6. Gate G0

**Decides:** the delivery envelope, and nineteen ADRs.

**It has rescope authority, not stop authority, and this plan says so rather than inventing thresholds.**
Under D1 the model wins and the target bends, so no measurement expressed in nanoseconds or megabytes can
stop this project — it can only change what is being delivered. A gate whose stop conditions are all
performance figures would be a gate that cannot fire, which is worse than a gate that admits its scope.

**Rescope thresholds, published before the measurements:**

| Finding | Consequence |
|---|---|
| Committed-and-held allocation ceiling < 1,700 MiB | The Android target cannot carry the full class. The *delivery* changes — desktop first, Android when the memory derivation supports it. The model does not change |
| `exchangeNs` > 300 ns | §12.2's 478.7 ms tick is restated. N2a and N2b are re-derived from the measurement, and the run takes longer. **No cadence is coarsened and no agent is removed** |
| `exchangeNs` > 1,000 ns | As above, and the acceleration seam moves from M11 into the critical path, because a 3.1 s tick makes an interactive product implausible without it |
| `transcendental.identical` is false | ADR-0015 takes the software-implementation branch and `kernel` gains a deterministic math module. If true, the branch is dropped and the widened §11 ban is free |
| Storage `seqWriteMiBs` < 200 | Checkpointing at cadence 64 is not affordable; ADR-0016 takes dirty-region incremental checkpointing |
| W7.1's derivation exceeds the measured ceiling | The instrument row A/B (ADR-0009) is decided by memory rather than by speed, and §3.4's slot capacities are re-derived before M1 allocates anything |

**G0 no longer carries a stop finding, and that is a result rather than a gap.** The candidate stop —
that no modelled sequence moves the first unit of money from a government to a household — was resolved
by derivation from the model rules before this milestone starts: §13.6 now carries it. W7.2 confirms the
derivation rather than testing whether one exists. **The first gate with stop authority is G1a.**

---

## 7. Where the work stands

| Workstream | State |
|---|---|
| **W1** the workspace that refuses | **done** — 12 crates, the layer matrix proved by a committed fixture, the lint floor, the `surface`/`shell` split |
| **W2** build machinery and CI | **done but for two**, each blocked on the thing it would police: `check-generated` needs a generator (M1), `check-registry` is W4's |
| **W3** the probe and the way results get back | **the measurements are written and run** (`cargo run --release -p aurora-probe`), and three are already red on the host, which is a floor. What remains is the Android packaging — `cargo-ndk`, the Kotlin shell, the release — and the device run |
| **W4** the parameter registry | **done but for two**, both deferred with a reason: the generated unit vocabulary needs `domain`'s quantity types (M1), and the `capacity` read rule needs systems to police |
| **W5** ADR machinery | **part done** — the format and `check-adr` landed with W2; numbering, coupling and appendix generation remain |
| **W6** falsifiers that need code | **done but for B2/B3's ensemble half.** The seed generator and the burn-in tests are red and pinned; the intrinsic table and the amendment matrix are filled, total, and checked |
| **W7** falsifiers that need paper | **two of nine done**, and both were computations rather than prose: the memory derivation and the identifier census (`aurora-tools sizing`) |

**Checks running, behind one `aurora-tools verify`:**
`check-lints` · `check-surface` · `check-deps` · `check-refs` · `check-adr` · `check-registry` ·
`check-instruments`

**The census, published on every build:** 21 model entries — 9 assumed, 10 structural, 2 derived,
0 placeholder — and 4 capacity entries counted separately. No cap (D3); the direction is down.

**The gate is one command:** `./gate.sh` — format, build, clippy `-D warnings`, test, then
`aurora-tools verify`. CI runs that same file and nothing else, so the thing checked before a commit
and the thing checked after a push cannot drift.

**ADRs taken:** 0001 Rust/crates/Android · 0002 the model wins · 0003 the layer matrix ·
0004 the arena seam · 0005 the surface/shell split · 0013 the sixteen identities ·
0014 the registry's two namespaces.

**Ten findings so far, every one measured rather than reviewed.** `check-lints`' first draft substring-matched and its first run reported *itself*.
`check-surface`'s first run flagged one subtraction twice, because `->` is a `-` punct.
`check-refs` found §17.4 demoted from a heading to bold text by an earlier edit, while three
citations still pointed at it. `check-registry` rule 3 rejected the first derived entry written
against it — a labour endowment declared in `hour` where the arithmetic says `hour/count` — and in
doing so showed that §16.1's rule 1, stated over unit *names*, cannot express a compound unit at all.
`aurora-tools seedgen` found that §13.3's continuous half reproduces exactly and its integer half
reproduces under **no** quantization rule, so its count rows were not computed from its own generator —
and that cohort shares are invariant across regions by construction, so an axis-3 primitive cannot
vary on axis 3. `aurora-tools burnin` found that B1b rejects 56% of genuinely stationary series and
the 42-series conjunction passed **0 of 2,000 panels**, so §15.3's gate as written would classify a
healthy model as defective.
The tenth was found by the process rather than by a check, and it is about the checks: **`verify`
printed "7 checks ran, 0 failed" on a tree where clippy was reporting a finding, twice**, because
`verify` never ran clippy and clippy returns zero on a warning. The gate existed as a habit — a list
of five commands in a document — and a habit is skippable in a way an exit code is not. `gate.sh` is
the repair: one command, non-zero on any stage, and the CI workflow now runs that file rather than
its own copy of the list.
**A check is not known to work until it has caught something, and each of these caught something on
its first run.**

## 8. Explicitly out of scope

Not in this milestone, and naming them is how they stay out: any `kernel`, `domain`, `world`, `ledger`,
`markets`, `agents`, `systems` or `runtime` implementation; any world table; the doors; the conformance
suite; the digest; the differ; the save format; the composition root generator; the committed period
order; any economic content; any agent declaration beyond W7.8's two on paper; threads.

**W7.8 writes two behaviour specifications and simulates them by hand. It does not implement them.** The
line is that paper is in scope and code in the eight engine crates is not.

---

## 9. Risks to this milestone

| Risk | Mitigation |
|---|---|
| The APK pipeline eats the milestone — signing, JNI, foreground service, device quirks | E2 starts it Monday and it is the only task with a hard external dependency. An empty APK reaching Releases by end of week 4 is the checkpoint; if it slips, the probe's benchmarks are run as a plain `adb`-pushed binary and the shell APK moves to M1 |
| The team writes engine code "just to test the checks" | Exit criterion 17 is a check, run in CI from week 1 |
| ADR machinery becomes the milestone | W5.3's maturity scoping exists precisely to stop it. If ADR count passes forty in this milestone, the coupling is too tight and W5.3 is wrong |
| The paper falsifiers slip because they have no compiler to say they are done | Each has a named artefact and a date; W7.2 and W7.8 are the two that must not slip, because they are the only economic evidence available before M6 |
| The probe measures the wrong thing — a benchmark that fits in cache | W3.5 specifies a realistic arena at the corrected 24 B slot and random access; the noise floor is published beside every figure, and a result without one is not a result |
