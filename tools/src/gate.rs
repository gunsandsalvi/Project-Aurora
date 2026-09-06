//! # The corrected burn-in gate — ADR-0019
//!
//! `burnin` is the falsification: it measures what §15.3's gate does to worlds whose answers are
//! known, and the answer is that a settled world passes all four tests on 28.7% of series, so the
//! 42-series conjunction passes about once in 10²³. This module is what replaces it.
//!
//! Three changes, and the order matters. **Every critical value is a bootstrap quantile** rather than
//! a chosen number — a stationary block bootstrap of the window under test, whose block length is
//! derived from the series' own autocorrelation. **B2 is replaced** rather than recalibrated, because
//! its statistic is dominated by the spread the seeds opened with and inverts as that spread widens.
//! **The correction runs the other way**: `α = 1 − 0.95^(1/168)`, so that a settled world passes the
//! conjunction 95% of the time, rather than each test passing 95% of the time and the conjunction
//! passing never.
//!
//! §15.3 computes every gate statistic outside the engine from the observation store, and no engine
//! handle reads one. That is why this lives in `tools` and is not engine code: it is where the gate
//! belongs permanently, not a prototype of something `runtime` will own.

use core::fmt::Write as _;

use crate::burnin::{Shape, ensemble_of, normal, unit};

/// §15.3's window.
pub const W: usize = 104;
/// The trailing window B2's mean is taken over.
pub const TRAIL: usize = 52;
/// Flagged series in §15.3's panel.
pub const PANEL: usize = 42;
/// Hypotheses in the conjunction: `PANEL` series times four tests.
pub const HYPOTHESES: usize = PANEL * 4;

/// Bootstrap replicates. 999 resolves a 5% quantile to about ±0.7%, which is the accuracy the
/// calibration check below is measuring against.
const REPLICATES: usize = 999;

/// The per-hypothesis size that makes the **conjunction** a five-per-cent test.
///
/// Šidák, pointed the opposite way from §15.3's instruction: `1 − 0.95^(1/n)`. Computed by bisection
/// rather than with `exp`, so that this module needs no transcendental at all and its result is a
/// property of arithmetic.
#[must_use]
pub fn sidak_alpha(hypotheses: usize, family: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..200 {
        let mid = lo.midpoint(hi);
        // (1 − mid)^n, by repeated multiplication.
        let mut p = 1.0;
        for _ in 0..hypotheses {
            p *= 1.0 - mid;
        }
        if p > family { lo = mid } else { hi = mid }
    }
    lo.midpoint(hi)
}

/// Lag-1 autocorrelation of a window.
#[must_use]
pub fn lag1(series: &[f64]) -> f64 {
    let n = series.len();
    if n < 3 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let nf = n as f64;
    let mean = series.iter().sum::<f64>() / nf;
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, y) in series.iter().enumerate() {
        den += (y - mean).powi(2);
        if let Some(prev) = i.checked_sub(1).and_then(|j| series.get(j)) {
            num += (y - mean) * (prev - mean);
        }
    }
    if den <= 0.0 { 0.0 } else { num / den }
}

/// The mean block length, derived from the series rather than chosen.
///
/// For an AR(1) the integrated autocorrelation time is `τ = (1+ρ)/(1−ρ)`, and the block must be long
/// enough to carry that dependence into the resample. `τ` alone is **not** long enough: measured
/// against a settled AR(1), `L = ⌈τ⌉` gives B1 and B1b a realised size of 0.110 and 0.107 against a
/// nominal 0.05, because a short block breaks the series' persistence and makes the null distribution
/// too narrow. `L = ⌈(2τ²n)^(1/3)⌉` is the standard growth rate for the stationary bootstrap's mean
/// block length, and `BLOCK_RULE` records which of the two is in force so that the choice is a
/// measurement rather than a preference.
///
/// Clamped to `[2, n/4]`: a block longer than a quarter of the window resamples the window as itself.
#[must_use]
pub fn block_length(series: &[f64]) -> usize {
    block_length_for(series, series.len())
}

/// The same rule, where the series the dependence is estimated from is longer than the series being
/// resampled. B2 estimates `τ` from the pooled ensemble and resamples windows of `TRAIL`; putting the
/// pooled length into the rate would give a block half the length of the window it is building.
#[must_use]
pub fn block_length_for(series: &[f64], n: usize) -> usize {
    let r = lag1(series).clamp(-0.99, 0.99);
    let tau = (1.0 + r) / (1.0 - r);
    #[allow(clippy::cast_precision_loss)]
    let nf = n as f64;
    let raw = (2.0 * tau * tau * nf).cbrt();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let l = raw.ceil().max(2.0) as usize;
    l.clamp(2, (n / 4).max(2))
}

/// One stationary-bootstrap resample: geometric block lengths with mean `l`, wrapping at the end.
///
/// The null it draws from is *this window, with its own dependence and its own marginal, and no drift
/// and no break*. That is the least-prior null available: nothing about the shape of the distribution
/// is chosen, because nothing about it is stated.
#[must_use]
pub fn resample(series: &[f64], block: usize, seed: u64, rep: u64) -> Vec<f64> {
    let n = series.len();
    if n == 0 {
        return Vec::new();
    }
    #[allow(clippy::cast_precision_loss)]
    let restart = 1.0 / block as f64;
    let mut out = Vec::with_capacity(n);
    let mut index = 0usize;
    let mut counter = 0u64;
    while out.len() < n {
        let roll = unit(seed, rep, counter);
        counter += 1;
        if out.is_empty() || roll < restart {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let start = (unit(seed, rep ^ 0x5bf0_3635, counter) * n as f64) as usize;
            counter += 1;
            index = start.min(n - 1);
        } else {
            index = (index + 1) % n;
        }
        if let Some(value) = series.get(index) {
            out.push(*value);
        }
    }
    out
}

/// The `q`-quantile of an unsorted sample, by nearest rank.
#[must_use]
pub fn quantile(sample: &mut [f64], q: f64) -> f64 {
    if sample.is_empty() {
        return 0.0;
    }
    sample.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let i = ((q * sample.len() as f64).ceil() as usize).clamp(1, sample.len()) - 1;
    sample.get(i).copied().unwrap_or(0.0)
}

/// B1's statistic: the OLS slope over the window, in units of the window's own standard deviation.
///
/// §15.3's disjunction — against the mean *or* against the standard deviation — is dropped. Two
/// normalisations with two chosen constants is two chosen constants; the σ-relative one is the
/// dimensionless statistic, and the bootstrap supplies its critical value.
#[must_use]
pub fn b1_stat(series: &[f64]) -> f64 {
    let n = series.len();
    if n < 3 {
        return f64::INFINITY;
    }
    #[allow(clippy::cast_precision_loss)]
    let nf = n as f64;
    let mean_x = (nf - 1.0) / 2.0;
    let mean_y = series.iter().sum::<f64>() / nf;
    let (mut num, mut den) = (0.0, 0.0);
    for (i, y) in series.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    let beta = if den <= 0.0 { 0.0 } else { num / den };
    let sd = (series.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>() / nf).sqrt();
    if sd <= 0.0 {
        return 0.0;
    }
    (beta * nf).abs() / sd
}

/// B1b's two statistics: the standardised split-half mean gap, and the split-half variance ratio.
#[must_use]
pub fn b1b_stats(series: &[f64]) -> (f64, f64) {
    let half = series.len() / 2;
    let (a, b) = series.split_at(half);
    if a.is_empty() || b.is_empty() {
        return (f64::INFINITY, f64::INFINITY);
    }
    #[allow(clippy::cast_precision_loss)]
    let (na, nb) = (a.len() as f64, b.len() as f64);
    let (ma, mb) = (a.iter().sum::<f64>() / na, b.iter().sum::<f64>() / nb);
    let va = a.iter().map(|y| (y - ma).powi(2)).sum::<f64>() / na;
    let vb = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>() / nb;
    let pooled = va.midpoint(vb).sqrt();
    let gap = if pooled <= 0.0 {
        0.0
    } else {
        (ma - mb).abs() / pooled
    };
    // The ratio, folded so that either direction of change is the same size of departure. A ratio
    // rather than its logarithm, because no transcendental is available and none is needed.
    let (hi, lo) = (va.max(vb), va.min(vb));
    let ratio = if lo <= 0.0 { f64::INFINITY } else { hi / lo };
    (gap, ratio)
}

/// The verdict on one series, and on one ensemble, under the calibrated gate.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct Verdict {
    /// B1 — no drift.
    pub b1: bool,
    /// B1b — no regime shift, by the smaller of its two bootstrap p-values.
    pub b1b: bool,
    /// B2 — the ensemble mixed.
    pub b2: bool,
    /// B3 — the opening is forgotten.
    pub b3: bool,
}

impl Verdict {
    /// All four, which is what a series must do for its period to be `burnInPeriod`.
    #[must_use]
    pub fn all(self) -> bool {
        self.b1 && self.b1b && self.b2 && self.b3
    }
}

/// The fraction of `sample` at or above `observed` — a bootstrap p-value, never zero.
fn p_value(sample: &[f64], observed: f64) -> f64 {
    let ge = sample.iter().filter(|v| **v >= observed).count();
    #[allow(clippy::cast_precision_loss)]
    let p = (ge as f64 + 1.0) / (sample.len() as f64 + 1.0);
    p
}

/// B1 and B1b on one path, against a bootstrap of that path.
///
/// B1b combines its two statistics by **minimum p-value**, with the critical value for that minimum
/// taken from the same replicates. That keeps B1b one hypothesis, which is what §15.3's count of 168
/// requires, without pretending its two arms are one statistic.
#[must_use]
pub fn path_tests(path: &[f64], seed: u64, alpha: f64) -> (bool, bool) {
    let block = block_length(path);
    let obs_b1 = b1_stat(path);
    let (obs_gap, obs_ratio) = b1b_stats(path);

    let mut null_b1 = Vec::with_capacity(REPLICATES);
    let mut null_gap = Vec::with_capacity(REPLICATES);
    let mut null_ratio = Vec::with_capacity(REPLICATES);
    for r in 0..REPLICATES {
        let boot = resample(path, block, seed, r as u64);
        null_b1.push(b1_stat(&boot));
        let (g, v) = b1b_stats(&boot);
        null_gap.push(g);
        null_ratio.push(v);
    }

    let drift_ok = p_value(&null_b1, obs_b1) > alpha;

    // min-p, calibrated against the replicates' own min-p.
    let obs_minp = p_value(&null_gap, obs_gap).min(p_value(&null_ratio, obs_ratio));
    let mut null_minp: Vec<f64> = (0..REPLICATES)
        .map(|i| {
            let g = null_gap.get(i).copied().unwrap_or(f64::INFINITY);
            let v = null_ratio.get(i).copied().unwrap_or(f64::INFINITY);
            // A small p is an extreme statistic, so the min-p null is built from -p and read at the
            // same tail as everything else here.
            -(p_value(&null_gap, g).min(p_value(&null_ratio, v)))
        })
        .collect();
    let crit = quantile(&mut null_minp, 1.0 - alpha);
    let shift_ok = -obs_minp < crit;
    (drift_ok, shift_ok)
}

/// B2's replacement statistic: between-seed spread of the trailing mean, over the spread each seed's
/// own history says a mean of that length should have.
///
/// The window is `W`. Its first `TRAIL` periods and its last `TRAIL` periods give each seed two means
/// at the same time scale: `h1` and `h2`. The **numerator** is the between-seed variance of `h2`, the
/// trailing mean §15.3 asks about. The **denominator** is the mean over seeds of `(h1 − h2)²/2`,
/// which is what one seed's own variability at that scale looks like.
///
/// Two things follow, and both are the point. Under a mixed ensemble the seeds are exchangeable draws
/// from one law, so numerator and denominator estimate the same quantity and the ratio is 1; while
/// the ensemble is still spreading, the between-seed term is inflated and the ratio exceeds 1. And
/// **no autocorrelation model appears anywhere**: both terms are built from means at the same scale,
/// so whatever dependence the series carries cancels. §15.3's ratio needed the opening spread and got
/// its sign from it; this needs nothing the model did not produce.
#[must_use]
pub fn b2_stat(ensemble: &[Vec<f64>], gate: usize) -> f64 {
    let mut trailing = Vec::with_capacity(ensemble.len());
    let mut within = Vec::with_capacity(ensemble.len());
    for series in ensemble {
        let Some(start) = gate.checked_sub(W - 1) else {
            return f64::INFINITY;
        };
        let (Some(first), Some(last)) = (
            series.get(start..start + TRAIL),
            series.get(gate + 1 - TRAIL..=gate),
        ) else {
            return f64::INFINITY;
        };
        #[allow(clippy::cast_precision_loss)]
        let m = TRAIL as f64;
        let h1 = first.iter().sum::<f64>() / m;
        let h2 = last.iter().sum::<f64>() / m;
        trailing.push(h2);
        within.push((h1 - h2).powi(2) / 2.0);
    }
    #[allow(clippy::cast_precision_loss)]
    let e = trailing.len() as f64;
    if e < 2.0 {
        return f64::INFINITY;
    }
    let grand = trailing.iter().sum::<f64>() / e;
    let between = trailing.iter().map(|m| (m - grand).powi(2)).sum::<f64>() / (e - 1.0);
    let w = within.iter().sum::<f64>() / e;
    if w <= 0.0 { f64::INFINITY } else { between / w }
}

/// B2's critical value: the `1 − α` point of the statistic when the seeds *are* exchangeable.
///
/// Because the statistic is a ratio of two estimates of the same variance, its null distribution
/// depends on nothing but `E`. It is obtained by simulating `E` exchangeable pairs of means and
/// reading the same statistic — arithmetic over a known distribution, computed once per ensemble
/// size, in the same way B3's 0.503 is.
#[must_use]
pub fn b2_critical(seeds: usize, alpha: f64, draws: usize) -> f64 {
    if seeds < 2 {
        return f64::INFINITY;
    }
    let mut sample: Vec<f64> = (0..draws)
        .map(|d| {
            let ensemble: Vec<Vec<f64>> = (0..seeds)
                .map(|e| {
                    // Two means at the same scale, exchangeable across seeds by construction. The
                    // window is laid out so that `b2_stat` reads `h1` and `h2` straight back out.
                    let h1 = normal(0x51ed_2701, d as u64, e as u64);
                    let h2 = normal(0x51ed_2701, d as u64, (e + seeds) as u64);
                    let mut w = vec![h1; TRAIL];
                    w.extend(core::iter::repeat_n(h2, TRAIL));
                    w
                })
                .collect();
            b2_stat(&ensemble, W - 1)
        })
        .filter(|v| v.is_finite())
        .collect();
    quantile(&mut sample, 1.0 - alpha)
}

/// B3's critical value: the `1 − α` point of `|ρ|` under the permutation null at `n` seeds.
///
/// The null does not depend on the data — only on `n` — so it is computed once per ensemble size and
/// reused. That is what makes B3 the one test in §15.3 whose critical value was already derived.
#[must_use]
pub fn spearman_critical(n: usize, alpha: f64, permutations: usize) -> f64 {
    if n < 3 {
        return 1.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let mut sample: Vec<f64> = (0..permutations)
        .map(|p| {
            let base: Vec<f64> = (0..n).map(|i| i as f64).collect();
            let mut shuffled = base.clone();
            // Fisher–Yates, from the same counter-based generator as everything else here.
            for i in (1..n).rev() {
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                let j = (unit(0x9e37_79b9, p as u64, i as u64) * (i + 1) as f64) as usize;
                shuffled.swap(i, j.min(i));
            }
            crate::burnin::spearman_of(&base, &shuffled).abs()
        })
        .collect();
    quantile(&mut sample, 1.0 - alpha)
}

/// The four calibrated tests on one ensemble at one period.
#[must_use]
pub fn verdict(
    ensemble: &[Vec<f64>],
    gate: usize,
    seed: u64,
    alpha: f64,
    b2_crit: f64,
    rho_crit: f64,
) -> Verdict {
    let Some(first) = ensemble.first() else {
        return Verdict {
            b1: false,
            b1b: false,
            b2: false,
            b3: false,
        };
    };
    let start = gate.saturating_sub(W - 1);
    #[allow(clippy::cast_precision_loss)]
    let seeds = ensemble.len() as f64;
    let path: Vec<f64> = (start..=gate.min(first.len().saturating_sub(1)))
        .map(|t| ensemble.iter().filter_map(|s| s.get(t)).sum::<f64>() / seeds)
        .collect();
    let (b1, b1b) = path_tests(&path, seed, alpha);

    let mut opening = Vec::with_capacity(ensemble.len());
    let mut current = Vec::with_capacity(ensemble.len());
    for s in ensemble {
        opening.push(s.first().copied().unwrap_or(0.0));
        current.push(s.get(gate).copied().unwrap_or(0.0));
    }
    Verdict {
        b1,
        b1b,
        b2: b2_stat(ensemble, gate) <= b2_crit,
        b3: crate::burnin::spearman_of(&opening, &current).abs() <= rho_crit,
    }
}

/// The gate period the calibration is measured at.
const GATE: usize = 260;
/// Ensembles per measured rate. 300 resolves a nominal 5% to about ±1.3%, which is what the band
/// below is set against.
const TRIALS: usize = 300;
/// Seeds per ensemble, §15.3's `E`.
const E: usize = 16;

/// What the calibration check requires of a realised size: a bootstrap quantile is not exact, and a
/// band of [0.02, 0.10] around a nominal 0.05 is what `TRIALS` can actually resolve.
const SIZE_BAND: (f64, f64) = (0.02, 0.10);

/// Run the calibration check and report; the exit status is the process's.
pub fn run() -> std::process::ExitCode {
    let (report, failures) = calibrate();
    print!("{report}");
    println!("gate");
    println!(
        "  rule: every critical value is a bootstrap quantile, and each test's realised size on a\n  \
         settled ensemble sits inside [{:.2}, {:.2}] at a nominal 0.05 (ADR-0019's guard)",
        SIZE_BAND.0, SIZE_BAND.1
    );
    println!("  exemptions: 0");
    if failures.is_empty() {
        println!("  violations: 0");
        return std::process::ExitCode::SUCCESS;
    }
    println!("  violations: {}", failures.len());
    for f in &failures {
        println!("    {f}");
    }
    std::process::ExitCode::FAILURE
}

/// The calibration measurement, and any test whose realised size falls outside the band.
///
/// **Measured at a nominal 0.05, not at the operational α.** The operational α is
/// `1 − 0.95^(1/168) ≈ 3.05 × 10⁻⁴`, and resolving a rejection rate that small would take tens of
/// thousands of trials. The bootstrap calibration is α-invariant by construction — the same null
/// sample is read at a different quantile — so measuring it where it *can* be measured is what
/// establishes it everywhere. A guard that could not be afforded would not be run.
#[must_use]
pub fn calibrate() -> (String, Vec<String>) {
    let mut out = String::new();
    let alpha = 0.05;
    let rho_crit = spearman_critical(E, alpha, 20_000);
    let b2_crit = b2_critical(E, alpha, 20_000);
    let operational = sidak_alpha(HYPOTHESES, 0.95);

    let _ = writeln!(
        out,
        "the calibrated gate — ADR-0019, measured at a nominal size of {alpha:.2}\n"
    );
    let _ = writeln!(
        out,
        "  operational alpha, 1 - 0.95^(1/{HYPOTHESES}) = {operational:.3e}\n  critical values at n = {E}, size {alpha:.2} — B2: {b2_crit:.3}   B3 |rho|: {rho_crit:.3}"
    );
    let _ = writeln!(
        out,
        "  block length is derived per window; bootstrap replicates {REPLICATES}; {TRIALS} ensembles per rate\n"
    );
    let _ = writeln!(
        out,
        "  ensemble                      B1      B1b     B2      B3      all four"
    );

    let shapes = [
        ("mixing AR(1), phi 0.5", Shape::Ar1(0.5), true),
        ("slow AR(1), phi 0.98", Shape::Ar1(0.98), false),
        ("random walk", Shape::Walk, false),
        ("frozen at its opening", Shape::Frozen, false),
    ];
    let mut failures = Vec::new();
    for (name, shape, is_settled) in shapes {
        let (mut n1, mut n1b, mut n2, mut n3, mut nall) = (0usize, 0usize, 0usize, 0usize, 0usize);
        for t in 0..TRIALS {
            let base = 400_000 + (t as u64) * 64;
            let ens = ensemble_of(shape, base, E, GATE + 1, 10.0);
            let v = verdict(&ens, GATE, base ^ 0xA17F, alpha, b2_crit, rho_crit);
            n1 += usize::from(v.b1);
            n1b += usize::from(v.b1b);
            n2 += usize::from(v.b2);
            n3 += usize::from(v.b3);
            nall += usize::from(v.all());
        }
        #[allow(clippy::cast_precision_loss)]
        let d = TRIALS as f64;
        #[allow(clippy::cast_precision_loss)]
        let rates = [
            ("B1", n1 as f64 / d),
            ("B1b", n1b as f64 / d),
            ("B2", n2 as f64 / d),
            ("B3", n3 as f64 / d),
        ];
        let _ = writeln!(
            out,
            "  {name:<28}  {:.3}   {:.3}   {:.3}   {:.3}   {:.3}",
            rates.first().map_or(0.0, |r| r.1),
            rates.get(1).map_or(0.0, |r| r.1),
            rates.get(2).map_or(0.0, |r| r.1),
            rates.get(3).map_or(0.0, |r| r.1),
            {
                #[allow(clippy::cast_precision_loss)]
                let a = nall as f64 / d;
                a
            }
        );
        if is_settled {
            for (test, rate) in rates {
                let size = 1.0 - rate;
                if size < SIZE_BAND.0 || size > SIZE_BAND.1 {
                    failures.push(format!(
                        "{test} rejects a settled ensemble at {size:.3}, outside [{:.2}, {:.2}] — its critical value is not a quantile of its null",
                        SIZE_BAND.0, SIZE_BAND.1
                    ));
                }
            }
        }
    }
    let _ = write!(out, "{}", operational_costs(operational));
    let _ = writeln!(out);
    (out, failures)
}

/// What the operational alpha costs, measured rather than assumed.
///
/// The calibration above runs at a size 164 times larger, because that is the size `TRIALS` can
/// resolve. These are the numbers that the operational size implies for the gate as it will run.
fn operational_costs(operational: f64) -> String {
    let mut out = String::new();
    // What the operational alpha costs, measured rather than assumed. The calibration above is at a
    // size 164 times larger, because that is the size 300 trials can resolve; these are the numbers
    // that size implies for the gate as it will actually run.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let min_replicates = (1.0 / operational).ceil() as usize;
    let _ = writeln!(out, "\n  At the operational alpha of {operational:.3e}:\n");
    let _ = writeln!(
        out,
        "  - a bootstrap p-value cannot go below 1/(R+1), so B1 and B1b need R >= {min_replicates}"
    );
    let _ = writeln!(
        out,
        "    replicates to resolve their critical value at all, and about 10x that to resolve it"
    );
    let _ = writeln!(
        out,
        "    stably. {REPLICATES} replicates, which is what the calibration above uses, cannot."
    );
    for seeds in [16usize, 24, 40, 64] {
        let rho = spearman_critical(seeds, operational, 200_000);
        let b2c = b2_critical(seeds, operational, 200_000);
        let _ = writeln!(
            out,
            "  - E = {seeds:>2}:  B3 critical |rho| {rho:.3}   B2 critical ratio {b2c:>6.2}"
        );
    }
    for line in [
        "",
        "  B3 is where E binds. \u{a7}15.3's E = 16 gives a corrected critical |rho| of 0.809 against the",
        "  0.503 it declares uncorrected \u{2014} the correction costs B3 most of its power, and the seeds are",
        "  the only thing that buys it back. At E = 40 the CORRECTED critical value is 0.553, which is",
        "  about what \u{a7}15.3 believed it had at E = 16 before the conjunction was counted. That is the",
        "  derivation: forty seeds, not sixteen, and the cost is 2.5x the ensemble.",
    ] {
        let _ = writeln!(out, "{line}");
    }

    out
}
