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
| W3.10 | **The device run** | The owner installs the APK from the Release link, taps Run, waits about twenty minutes, taps Copy, and pastes the JSON back. **Everything else in W3 is done; this is the only step that needs hardware** | — | one `aurora.probe/1` document with no field null |

**The output the owner copies back**, as the host emits it today (`cargo run --release -p
aurora-probe`). Every field is written before the document is printed, so a zero is a measurement that
ran and found zero, and a missing key would be a bug rather than a blank:

```json
{ "schema": "aurora.probe/1", "commit": "<sha>", "device": "<manufacturer model, android, sdk>",
  "slots": 7177280, "ops": 2000000,
  "allocation":     { "ceilingMiB": 2048, "heldSeconds": 10 },
  "operationCost":  { "interleavedNs": 133.8, "columnarNs": 213.0, "noiseFloorNs": 0.8, "budgetNs": 96.5 },
  "blockOps":       { "10": 31.1, "256": 70.7, "4096": 405.6, "16384": 2036.8 },
  "storage":        { "seqWriteMiBs": 145.3, "chunkedWriteMiBs": 461.5 },
  "transcendental": { "lnHash": "b5d414b87dd05ab7", "expHash": "a493d8dc7d53c03e" },
  "thermal":        { "burstOpsPerSec": 37059878, "soakOpsPerSec": 37217543, "ratio": 1.004, "soakSeconds": 5 },
  "sink": 8501787225403862321 }
```

*The host's `ratio` of 1.004 is not a result, it is the absence of one:* a desktop with a fan does not
throttle over five seconds. On the device the soak is fifteen minutes and the ratio is the number N2b
is derived from.

**How the owner runs it.** The `probe` workflow cross-compiles for `aarch64-linux-android`, packages
the binary as `lib/arm64-v8a/libaurora_probe.so`, builds the shell APK and attaches it to a Release.
Install from the link, tap **Run**, keep the screen on for about twenty minutes, tap **Copy**, paste.

*Two deviations from this plan as drafted, both deliberate.* **There is no JNI**: the probe is a PIE
executable run as a subprocess, which keeps `forbid(unsafe_code)` in the delivered artifact and keeps
a foreign-function boundary out of the thing being measured. And **there is no foreground service**:
the run is a child of the app with the screen held on, so the app stays foreground and the child stays
alive — a service defends against the app being *backgrounded*, which is not this case. If a soak is
ever actually killed, that is the evidence for adding one. `largeHeap` is declared and does nothing:
it raises the JVM ceiling and every byte the probe allocates is in another process, which is precisely
why the allocation ceiling is worth measuring at all.

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

### W7 — Falsifiers that need paper

*Owner M, with E4. No dependencies. These are the cheapest evidence in the project.*

| | Task | Days | Done when |
|---|---|---|---|
| W7.1 | **The memory derivation.** One row per world table, capacity × width, summed against N4. Currently ~705 MB of the 1,488.3 MB is unaccounted and its largest term is the instrument row width §7.5 leaves unsettled | 4 | a table that sums, with every capacity a `capacity` registry entry carrying its arithmetic |
| W7.3 | **The identifier census.** Per identity space, live and ever-issued over 1,560 ticks. §3.4's ≈971,000 against §5.2's implied ≈12,450,000 is a factor of thirteen and it sizes the directory, the digest walk and the save | 2 | one reconciled figure per space |

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

**Four are met and stay met**, each by a check in `./gate.sh` with a negative fixture behind it.
**Two are blocked on the device.** **Two were rewritten** because what was built is not what they
described, and §1a says the constraint wins over the draft. **One is blocked on M1**, which is where
the thing it polices arrives.

| | Criterion | State |
|---|---|---|
| 5 | `check-generated` fails on a hand edit to any generated file | **blocked on M1.** There is no generator until M1's column schema; the check is written with the first one |
| 7 | An ADR without a `guard` fails `check-adr`, and a parsed-value change to a *ratified* registered file is refused where the same change to a draft file is free | **met** — `check-adr` and `check-coupling`, six fixtures. **Rewritten**: the coupling is to an ADR that `registers:` the file, not to a `Decision:` commit trailer. A trailer is prose, and prose is what this machinery exists to stop a value from being |
| 8 | Appendix A's guard column regenerates from ADR front matter and the build fails when the committed table differs | **met** — `aurora-tools appendix` and `check-register`, two fixtures. **Rewritten**: Appendix B is a narrative supersession log for an *edition*, not a per-decision table, and generating it would replace an argument with a dump. Appendix A's guard column is the part that was written twice by hand, and it had already drifted |
| 9 | The probe has returned a complete `aurora.probe/1` document from the owner's device, no field null | **blocked on the device.** The document is emitted, the APK is built and the pipeline is green end to end |
| 10 | The transcendental comparison is decided and ADR-0015 records what follows | **blocked on the device.** The host half is measured: `ln` hashes to `b5d414b87dd05ab7`, `exp` to `a493d8dc7d53c03e` |
| 11 | The memory derivation sums table by table, and every capacity in it is a registry entry carrying its arithmetic | **partly met.** Eleven of sixteen rows are derived and the directory and instruments rows are now computed rather than owed; five remain owed and each names what it needs. The capacity entries exist and are counted separately, but they are `structural` rather than expressions — see the register |
| 12 | The seed generator's output is committed beside §13.3's table, every row marked reproduces or does not | **met** — `aurora-tools seedgen`, and the answer is that the continuous half reproduces exactly and the integer half reproduces under no rule |
| 13 | The burn-in conjunction's false-pass rate is measured and ADR-0019 specifies the correction | **met**, and the measurement inverted the question: the risk is false *failure*, and ADR-0019's correction runs the other way |
| 14 | Every opening type answers all thirteen questions, and removing one fails the build | **met** — `check-instruments`, eight types × thirteen questions, each answer from its own closed set. **Rewritten**: the table is data checked by a tool rather than a type that compiles, because M0 writes no engine code (§1a, and W4's same correction) |
| 15 | All nineteen ADRs are accepted, each naming a mechanical guard | **seventeen of nineteen.** 0015 and 0016 are the two the device decides |
| 17 | No file under an engine crate's `src/` but `lib.rs` and the layer probe — §1's constraint, as a check | **met** — `check-lints` rule 5, three fixtures. It was the last criterion without a check, on the milestone's defining constraint |

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
| **W2** build machinery and CI | **done but for one**: `check-generated` needs a generator, and the first one is M1's column schema |
| **W3** the probe and the way results get back | **built, released, and waiting on hardware.** The pipeline is green end to end — the cross-compile produces a real `ELF 64-bit LSB pie executable, ARM aarch64`, Gradle packages a 1.03 MB APK, and it is attached to **[probe-v0.1](https://github.com/gunsandsalvi/Project-Aurora/releases/tag/probe-v0.1)**. **The only step left is the owner installing it and pasting the JSON back** |
| **W4** the parameter registry | **done but for two**, both deferred with a reason: the generated unit vocabulary needs `domain`'s quantity types (M1), and the `capacity` read rule needs systems to police |
| **W5** ADR machinery | **done.** Format and `check-adr`; the counter (`register.txt` + `adr new`); the coupling (`coupling.toml` + `check-coupling`, ratified against draft); Appendix A's guard column generated from the decisions (`aurora-tools appendix` + `check-register`). Ten negative fixtures across the three |
| **W6** falsifiers that need code | **done.** The seed generator is red and pinned; all four burn-in tests are measured, and the gate they falsified is recalibrated and guarded (ADR-0019, `aurora-tools gate`); the intrinsic table and the amendment matrix are filled, total, and checked |
| **W7** falsifiers that need paper | **done, all nine**, and not one of them is prose: five in `aurora-tools sizing` (memory, identifier census, journal row ADR-0008, household block, instrument row ADR-0009), the position workload in `aurora-tools workload`, §13.6's opening in `aurora-tools bootstrap`, §8.1's two declarations in `aurora-tools behaviour`, and credit's registry cost in `aurora-tools registry-cost` |

**Ten checks, behind one `./gate.sh`:** `check-lints` · `check-surface` · `check-deps` · `check-refs` ·
`check-adr` · `check-registry` · `check-instruments` · `check-coupling` · `check-register` ·
`check-bootstrap`, all behind `aurora-tools verify`, then `aurora-tools gate` — ADR-0019's guard, which
re-measures the burn-in tests' realised size on a settled ensemble and fails if one has drifted off its
nominal.

**Two of the ten carry their own negative control**, and they are the two whose subject is a claim
rather than a shape: `check-bootstrap` fails if §13.6's opening stops executing *or* if the deadlock it
argues against stops breaking R-1, and `gate` fails if a calibrated test's realised size drifts off its
nominal in either direction. A check that cannot fail is not a check.

**The census, published on every build:** 23 model entries — 9 assumed, 12 structural, 2 derived,
0 placeholder — and 4 capacity entries counted separately. No cap (D3); the direction is down.

**The gate is one command:** `./gate.sh` — format, build, clippy `-D warnings`, test,
`aurora-tools verify`, `aurora-tools gate`. Ten seconds warm. CI runs that same file and nothing else,
so the thing checked before a commit and the thing checked after a push cannot drift.

**ADRs: nineteen allocated, seventeen written, two reserved** — the counter says so on every build,
and the two that remain are the two the device decides: **0015** (the transcendental comparison) and
**0016** (checkpoint cadence, which turns on the measured write throughput).

0001 Rust/crates/Android · 0002 the model wins · 0003 the layer matrix · 0004 the arena seam ·
0005 the surface/shell split · 0006 `i64` and overflow panics · 0007 the 24-byte holdings slot ·
0008 the 48-byte journal row · 0009 the 80-byte instrument row · 0010 the identifier census ·
0011 sixty-four shards · 0012 the thread-shareable arena · 0013 the definitional identities ·
0014 the registry's two namespaces · 0017 the retirement queue · 0018 amendment handles ·
0019 the burn-in gate's calibration and correction.

**Thirty-three findings so far, every one measured rather than reviewed.** `check-lints`' first draft substring-matched and its first run reported *itself*.
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
healthy model as defective. Adding B2 and B3 made that worse and more precise: a settled ensemble
passes all four on 0.287 of series, so the panel passes 1.8 × 10⁻²³ of the time. **B2 does not merely
lack power, it inverts** — at an opening spread of 0.5 it separates a settled ensemble from a random
walk by +0.27, and at 40 a random walk passes it *every time* while the settled ensemble passes 0.58.
Its direction of discrimination is set by a quantity the model never chose. Writing ADR-0019's registry
entries then found that **§16.1's four provenances have no slot for a value produced by a simulation**:
a permutation quantile is not `assumed`, not `derived`, and `structural` only by charity. And §15.3
never says which of the `E` series B1 and B1b are evaluated on — its own count of 168 hypotheses is
what forces the answer. Generating Appendix A's guard column found that **the register had already
drifted from the decisions**: entry 12 read "per-mechanism minted capabilities" after ADR-0018 decided
the handle is minted per (mechanism, type), and entry 1 named "conserved columns" where ADR-0001 names
the single private column. The guard was written twice, by hand, and two copies of a value is what
§16.1 exists to prevent — applied to prose, nothing was checking it.
Deriving the journal row found that §6.6's 48 bytes hold **only because the realised rate is not
stored**: §6.4 asks for both rates, both come to 53 B and pad to 56, and the ring goes from 345.6 MB
to 403.2 MB — while the realised rate is `quantityReceived / quantityGiven` exactly, so the pair in
the row already is it, at better precision than a stored copy. And enumerating what a household can
hold found that **§3.4's ten slots do not cover its own tail**, which is eleven. The goods term was
settled by derivation rather than left open — §9.4 has no Consumption position and no larder, so one
goods line is live at a time provided position 14 pairs each purchase with its move, which is a
constraint on the settlement order worth 72.0 MB. What survives is worse than a capacity being wrong:
**the tail is unbounded**. Nothing stops a household holding fifty equity lines, so a fixed block plus
"exhaustion is a halt" is a halt waiting at every capacity, and what the block model needs is a
modelled rule bounding what a class member may hold. Ten is also the only capacity in §3.4's table
that is not a power of two.
Writing out both arms of §7.5's instrument row found that **neither published width is derived** —
the eleven columns come to 40 B, not 44, and the inline row to 80 B, not 148 — and that **the
comparison could not have been made at all**, because the 148-byte arm is specified in §3.4.4, one of
the eight dangling references. The two arms then came out 8.5 MB apart, inside the census's own error
bar, so what actually decided it was that **a schedule is not a claim**: nothing holds one, nothing
pledges one, no journal row names one. Its 6,360,000 identifiers were 40% of the whole census, for
rows that were never addressable.
Deriving §3.4's operation count position by position produced four more. **The four largest terms
cannot sum into the total**, because one of them — position 13's clearing work — is a sort and appends
no journal row, so it is not an operation count at all; separating the two is the whole of W7.4.
**Clearing's own cost is 38.0 million comparisons a tick**, 11.7× the operation count, in a unit §12
never budgets. **Position 15 is missing from the published derivation** at 550,622 operations, a fifth
of the total. And **a position mark cannot be an operation**: per-holding marks would be 7,177,280
journal rows a tick against a ring of 7,200,000 that must hold two ticks of everything, so a mark is
derived on read and never stored. The eighteen priceable positions come to 3,250,739 against a
published 3,119,665.
And walking §13.6's opening as a ledger settled the one thing it marked *"owed, and still owed"* —
who holds the capital and dwellings position 6 sources at tick 0 — **without a new rule**: the
producer of a thing receives its own output, which is what position 6 does at every other tick. The
question was open only because it was read as an allocation problem. It follows that **no household
owns a dwelling at tick 0** and home ownership is an emergent stock, because seeding one would be the
per-household assumed allocation §16.1 rule 1 refuses.
Writing §8.1's five declarations for a household and a bank and running them for ten ticks produced
five more. **§8.1's claim holds**: the bank's capital ratio reads two rows on its own block and
nothing else, which is a consequence of R-1 rather than of the constraint's design — a model holding
liabilities in a second table would need exactly the attribution step §8.1 says it does not have.
**The floor never binds under the declarations as written**, and that is a result: an equity flow of
20% of the wage against an 8% floor supports 2.5× the wage bill in new lending a tick, so the binding
constraint on credit in this cast is the household's saving. Stop the equity buying and it binds at
tick 13 — and *the ratio never goes below the floor*, because the constraint refuses the lending that
would take it there; a constraint that shows up as a breach was checked too late. **The bank's capital
arrives as a liability swap, not an inflow** — equity bought with a deposit moves no cash, so lending
capacity rises while the bank's currency does not. **The committed order caught an error in the
walk's own first draft**: position 6 written after position 14 made the producer sell what it had not
yet produced, reported on all ten ticks, and it was not caught by reading the code. And **five
declarations were enough, with one qualification**: declaration 3 has to bind at position 12, not at
the payment, because R-1 already makes an unbacked payment impossible — §8.1 says what the constraints
are and never says where they bind, §9.2 and §9.4 do, and neither says the two must be read together.
Writing credit's complete registry produced the last two. **The rate is six assumed entries per
economic system**, projecting to about 54 across nine against the 9 the census carries — but the
number that matters is the other one: **six of credit's eighteen questions have no registry entry at
all**, because a mechanism answers them. There is no default threshold (a borrower defaults when it
cannot pay, which is a balance test), no recovery rate (§6.7's waterfall produces it), and the
regulatory levels are the parliament's — **the political system is a registry saving**, which is not
why it was added. And one of the six assumed entries has no source anywhere: **nothing says who is
refused when credit is rationed.** §6.3 covers rounding, §9.2 allocates, §9.3 clears; rationing is
where a credit model's behaviour lives.
Three more came from writing the last ADRs, and each was found by the guard the ADR needed rather
than by reading. **Appendix A #2's "overflow panics" was not true**: `overflow-checks` defaults to off
in release, so the panic A1 leans on existed only in the build nobody ships. **§5.5's retirement queue
overflows at about tick 12** — 65,536 entries, drained at a position that runs every fifty-second
tick, against 5,653 retirements a tick; the capacity is right and the interval was wrong, and §5.5 and
Appendix B had already contradicted each other about which position drains it. And **exit criterion
17, the milestone's defining constraint, had no check at all** — "no engine code" was a rule everyone
was keeping and nothing was enforcing.
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
