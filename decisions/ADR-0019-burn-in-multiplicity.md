---
id: ADR-0019
title: Every critical value in the burn-in gate is a bootstrap quantile, and the correction controls false failure
status: accepted
date: 2026-09-06
register-entry: 13
claim-impact: A3
guard: aurora-tools gate — re-measures every calibrated test's realised size on a settled ensemble on every run of the build gate and fails outside [0.02, 0.10] at a nominal 0.05; aurora-tools burnin holds the falsification the correction answers
supersedes: §15.3's four threshold rows, its E = 16, the 0.503 registry entry, and ADR-0013's SpearmanCritical5Percent identity
cost: a stationary block bootstrap at every candidate period, ~44M statistic evaluations per gate run, computed outside the engine from the observation store
alternatives-rejected: Bonferroni or Šidák in the direction §15.3 asks for (tightens tests that are already too tight); recalibrating B2's ratio rather than replacing it (the statistic is wrong, not the number); leaving the thresholds and widening the [260, 520] bracket (a mis-specified test does not become specified by being given more periods)
re-derivations: §15.3's burnInPeriod bracket is unchanged; E rises from 16 to 40; §15.3.4's sensitivity sweep runs against the corrected gate
---

## Decision

**Three things, in this order.**

1. **No critical value in the gate is a chosen number.** Each of B1, B1b, B2 and B3 is thresholded at a
   quantile of the null distribution of *its own statistic*, obtained by a **stationary block bootstrap
   of the candidate window itself**. The block length is derived from the series' own autocorrelation,
   not chosen. §15.3's `0.05`, `0.50`, `0.25`, `2.0` and `[0.80, 1.25]` are deleted.

2. **B2 is replaced rather than recalibrated.** Its statistic — the cross-seed standard deviation of the
   trailing-52 mean, end over start — is dominated by the spread the seeds *opened* with, which is not a
   modelled quantity. The replacement is the ratio of the observed between-seed variance of the trailing
   mean to the between-seed variance implied by each seed's own within-window autocorrelation. Under a
   mixed ensemble the seeds are exchangeable draws from one law and the ratio is 1; while the ensemble is
   still spreading it exceeds 1. **The opening does not appear in it.**

3. **The correction controls the conjunction's false-*failure* rate, not its false-pass rate.** Each
   hypothesis is tested at `α = 1 − 0.95^(1/168) ≈ 3.05 × 10⁻⁴`, so that a settled world passes all 168
   at a family-wise rate of 0.95. This is Šidák pointed the other way round from §15.3's instruction, and
   the direction is the whole substance of the change.

4. **`E` rises from 16 to 40**, because the correction takes B3's power and the seeds are the only thing
   that buys it back. This is derived below, not chosen.

`burnInPeriod` is still the first `P ∈ [260, 520]` at which all four pass on all 42 series. Reaching 520
without a pass is still a defect. What changes is that those sentences now mean something.

## Why

**The measurement says §15.3 worries about the wrong failure.** `aurora-tools burnin` runs the four tests
against series and ensembles whose answers are known by construction:

| ensemble | B1 | B1b | B2 | B3 | all four |
|---|---|---|---|---|---|
| mixing AR(1), φ 0.5 — **the settled world** | 1.000 | 0.472 | 0.615 | 0.955 | **0.287** |
| slow AR(1), φ 0.98 | 0.990 | 0.055 | 0.627 | 0.950 | 0.030 |
| random walk | 0.925 | 0.065 | 0.590 | 0.527 | 0.013 |
| frozen at its opening | 1.000 | 0.780 | 1.000 | 0.000 | 0.000 |

A settled series passes all four **28.7%** of the time. §15.3 requires all 42 to pass together, so the
panel passes `0.287⁴² ≈ 1.8 × 10⁻²³` of the time. **The gate as written cannot fire on a world that has
settled**, and §15.3 then calls the resulting arrival at period 520 a defect — so the gate's certain
outcome is to declare a healthy model broken. §15.3's stated worry, that 168 tests at 5% each pass on
noise one time in nine, is the failure that cannot occur.

A correction in the direction §15.3 asks for tightens every test further. That is why the correction is
the *second* problem and the calibration is the first: **a multiplicity correction applied to
mis-specified tests corrects nothing.**

**Each test, separately, and what is wrong with it.**

- **B1** is the only one that behaves. It passes every settled series, and its failures are informative.
  It is still recalibrated, because a threshold that happens to be right is not a derived threshold.
- **B1b** rejects the majority of settled series and no window length fixes it. `|Δμ| ≤ 0.25·σ_pooled` is
  a **fixed effect size with no reference to sampling error**, so it is not a five-per-cent test at any
  `n`. Against a stationary AR(1) the statistic has median 0.231 — the 0.25 band is a 53rd-percentile
  cut.
- **B2 inverts.** Measured against the opening spread it is thresholded through:

  | opening spread | settled | random walk | gap |
  |---|---|---|---|
  | 0.5 | 0.580 | 0.307 | +0.272 |
  | 2.0 | 0.580 | 0.323 | +0.257 |
  | 10.0 | 0.580 | 0.590 | −0.010 |
  | 40.0 | 0.580 | **1.000** | **−0.420** |

  At a wide opening a **random walk passes B2 every single time** while the settled ensemble passes 0.58.
  The test prefers the diverging ensemble to the mixed one, and which way it points is decided by a
  quantity the model never chose. This is not a threshold problem. The statistic is wrong.
- **B3** does what it says: it rejects the frozen path every time. Its coin-toss verdict on the random
  walk is arithmetic rather than chance — a walk opened at spread 10 has a true period-0 correlation near
  0.53 at period 260, which is its critical value of 0.503 almost exactly. B3's 0.503 was the one
  critical value in the gate derived from a distribution rather than chosen, which is why it is the one
  that behaves, and it is the model for what the other three become.

**Why a bootstrap and not a distributional null.** A3 forbids calibrating to an observed economy, and D3
says minimise priors. Choosing AR(1) as the null under which to compute critical values would be a prior
about the model's own series — a chosen family, smuggled in as arithmetic. A stationary block bootstrap
of the window under test assumes only that the window is what it is: it reuses the series' own
autocorrelation and its own marginal, and asks what the statistic would look like if the window carried
no drift and no break. **One derived block length replaces five chosen constants**, which is the trade D3
asks for.

**Why the correction runs the other way.** With 168 conjoined hypotheses at nominal 5%, the gate's actual
size is `0.95¹⁶⁸ ≈ 0.02%` — it is not a 5% test of anything. The quantity worth controlling is the one
that has a consequence: a settled world being called defective. Setting `α = 1 − 0.95^(1/168)` makes the
*conjunction* the 5% test. The 168 are positively dependent — 42 series of one world, over 16 shared
seeds — and positive dependence makes `P(all pass) ≥ ∏P(pass)`, so the realised false-failure rate is at
or below 5%. **The dependence errs in the safe direction**, which is why the independence assumption is
declared rather than estimated.

## What the correction does, measured

`aurora-tools gate` implements the above and re-measures it. At a nominal size of 0.05 — the size 300
trials can resolve — against the same four ensembles:

| ensemble | B1 | B1b | B2 | B3 | all four |
|---|---|---|---|---|---|
| mixing AR(1), φ 0.5 — **the settled world** | 0.907 | 0.940 | 0.930 | 0.927 | **0.747** |
| slow AR(1), φ 0.98 | 0.650 | 0.777 | 0.363 | 0.953 | 0.190 |
| random walk | 0.467 | 0.643 | **0.000** | 0.480 | 0.000 |
| frozen at its opening | 0.940 | 0.953 | **0.000** | **0.000** | 0.000 |

Every test's realised size on the settled ensemble is inside [0.02, 0.10] against its nominal 0.05:
0.093, 0.060, 0.070, 0.073. **Before, B1b's was 0.528 and B2's discrimination pointed the wrong way.**
B2's replacement rejects the random walk and the frozen path every single time while passing the settled
ensemble at its nominal rate, which is what B2 was supposed to do and never did.

Two independent confirmations fell out of the machinery. The permutation null reproduces §15.3's `0.503`
for `|ρ|` at n = 16 and size 0.05 **exactly** — the one derived value in the old gate is derived — and it
is the only one the recalibration leaves where it was.

### Two numbers the correction forces to move

**Bootstrap replicates.** A bootstrap p-value cannot fall below `1/(R+1)`, so an operational α of
`3.05 × 10⁻⁴` needs **R ≥ 3,276** replicates before B1 or B1b can resolve their critical value at all, and
about ten times that to resolve it stably. The 999 replicates the calibration runs at cannot. This is a
cost of the correction, and it is why the guard is measured at a size where it can be measured.

**The ensemble.** B3's corrected critical `|ρ|`, by permutation:

| E | corrected critical &#124;ρ&#124; | B2 critical ratio |
|---|---|---|
| 16 | 0.809 | 4.07 |
| 24 | 0.695 | 2.97 |
| **40** | **0.553** | **2.23** |
| 64 | 0.440 | 1.89 |

At §15.3's `E = 16` the correction pushes B3 from 0.503 to 0.809 — it rejects only a near-perfect rank
correlation, and path dependence on the opening is exactly the failure a bottom-up model with a
hand-built seed is most likely to have. **At E = 40 the corrected critical value is 0.553**, which is
about the power §15.3 believed it had at E = 16 before anyone counted the conjunction. Forty seeds is
therefore the derived figure, and the cost is 2.5× the ensemble.

## What it costs

A stationary block bootstrap at each candidate period, at the replicate floor the correction forces:
261 periods × 42 series × 2 bootstrapped tests × 33,000 replicates ≈ 7.2 × 10⁸ statistic evaluations
over windows of 104. B2 and B3 cost nothing by comparison — their nulls depend only on `E`, so each is
computed once and reused. It is all outside the engine: §15.3 computes every gate statistic from the
observation store and no engine handle reads one, so this changes the cost of a *report*, not of a tick.

**And 2.5× the ensemble**, from `E = 16` to `E = 40`. That is the expensive half of this decision: the
bootstrap is one offline computation, the seeds are 24 more full runs.

The gate can no longer be evaluated by reading four numbers off a page. Someone reproducing a
`burnInPeriod` needs the observation store and the bootstrap seed, both of which are in the run manifest.

## Alternatives rejected

- **Šidák in the direction §15.3 asks for.** Rejected by measurement: it tightens tests whose measured
  failure is that they are already too tight.
- **Recalibrate B2's [0.80, 1.25] band.** No band fixes a statistic whose sign of discrimination is set
  by the opening spread.
- **Keep the thresholds; widen the bracket past 520.** A mis-specified test does not become specified by
  being given more periods, and the ceiling is the only thing in §15.3 that would ever tell the team the
  economics is wrong.
- **Drop B1b entirely** — a regime shift mid-window is a real failure mode and the diagnosis §15.3
  attaches to it (a threshold beginning to bind) is one this model expects to see.

## The guard

`aurora-tools gate`, and it runs in `gate.sh` on every commit. It re-measures each calibrated test's
realised size against a settled ensemble whose answer is known by construction and **exits non-zero if
any of the four falls outside [0.02, 0.10] at a nominal 0.05**. A reintroduced literal, a block-length
rule that stops tracking the series, a resampling scheme that breaks the dependence — each shows up as a
size that moves off its nominal, and the build stops.

The band is what 300 trials can resolve, and the size is measured at 0.05 rather than at the operational
`3.05 × 10⁻⁴` for the same reason: resolving a rejection rate that small takes tens of thousands of
trials, and **a guard nobody can afford stops being run**. The bootstrap calibration is α-invariant by
construction — the same null sample, read at a different quantile — so establishing it where it can be
measured establishes it where it will be used.

`aurora-tools burnin` stays as the falsification record: it is what found this, and it is the thing that
would notice if someone quietly restored §15.3's thresholds.

Second guard: `check-registry`. Every burn-in critical value is a registry entry, and after this ADR none
of them may carry `provenance = assumed` — a chosen literal in the gate is a build failure.
