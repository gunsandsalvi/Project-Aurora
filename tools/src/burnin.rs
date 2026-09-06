//! §15.3's four tests, characterised against synthetic series with known properties, and then run as
//! a Monte Carlo over whole panels.
//!
//! **This is the cheapest falsification anywhere in the project.** It needs no engine, no world and no
//! device — only a random number generator and the four pass conditions as §15.3 writes them — and it
//! asks the one question §15.3 never asks of itself: *on a world that has genuinely settled, does this
//! gate say so?*
//!
//! The gate is a conjunction over 42 flagged series and four tests. §15.3 makes `burnInPeriod` the
//! first period at which all of them pass, and makes reaching period 520 without a pass **a defect**.
//! So a gate that is too strict does not merely delay a result: it declares a healthy model broken.

use std::fmt::Write as _;

/// A counter-based generator: a pure function of (seed, stream, index), so a draw does not depend on
/// how many draws preceded it. This is a prototype of what §11 requires of the engine's own `draw`.
fn draw(seed: u64, stream: u64, index: u64) -> u64 {
    let mut x = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(stream.wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .wrapping_add(index.wrapping_mul(0x94D0_49BB_1331_11EB));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// A uniform in [0, 1), exactly representable.
#[allow(clippy::cast_precision_loss)]
pub fn unit(seed: u64, stream: u64, index: u64) -> f64 {
    (draw(seed, stream, index) >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
}

/// A standard normal by the central limit theorem over twelve uniforms.
///
/// §11 admits no `log`, `exp` or `cos`, which rules out Box–Muller; this needs none of them. It is a
/// crude normal and it is entirely adequate here, where the question is about a test's behaviour and
/// not about the tail of a distribution.
pub fn normal(seed: u64, stream: u64, index: u64) -> f64 {
    (0..12)
        .map(|k| unit(seed, stream, index * 12 + k))
        .sum::<f64>()
        - 6.0
}

/// The window §15.3 declares.
const W: usize = 104;
/// The ensemble §15.3 declares.
const E: usize = 16;

/// The trailing window B2 takes its mean over.
const TRAIL: usize = 52;
/// B3's critical value: the two-sided 5% point of Spearman's rho at n = 16, registered `structural`.
const RHO_CRIT: f64 = 0.503;

/// The four verdicts on one series.
///
/// Four booleans is exactly what §15.3 defines, so the shape is the specification's rather than a
/// missing enumeration: the tests are conjoined, not alternative.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct Verdict {
    /// B1 — no drift.
    pub b1: bool,
    /// B1b — no regime shift.
    pub b1b: bool,
    /// B2 — the ensemble mixed.
    pub b2: bool,
    /// B3 — the opening is forgotten.
    pub b3: bool,
}

impl Verdict {
    /// All four, which is what §15.3 requires of every series at `burnInPeriod`.
    #[must_use]
    pub fn all(self) -> bool {
        self.b1 && self.b1b && self.b2 && self.b3
    }
}

/// B1: the OLS slope over the window, judged against the mean or the standard deviation.
fn b1(series: &[f64]) -> bool {
    let n = series.len();
    if n < 3 {
        return false;
    }
    #[allow(clippy::cast_precision_loss)]
    let nf = n as f64;
    #[allow(clippy::cast_precision_loss)]
    let mean_x = (nf - 1.0) / 2.0;
    let mean_y = series.iter().sum::<f64>() / nf;
    let mut num = 0.0;
    let mut den = 0.0;
    for (i, y) in series.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let dx = i as f64 - mean_x;
        num += dx * (y - mean_y);
        den += dx * dx;
    }
    let beta = if den == 0.0 { 0.0 } else { num / den };
    let sd = (series.iter().map(|y| (y - mean_y).powi(2)).sum::<f64>() / nf).sqrt();
    let by_mean = if mean_y.abs() > 0.0 {
        (beta * nf).abs() / mean_y.abs() <= 0.05
    } else {
        false
    };
    let by_sd = if sd > 0.0 {
        (beta * nf).abs() / sd <= 0.50
    } else {
        true
    };
    by_mean || by_sd
}

/// B1b: split the window in half; the means must be close and the variances within a factor of two.
fn b1b(series: &[f64]) -> bool {
    let half = series.len() / 2;
    let (a, b) = series.split_at(half);
    if a.is_empty() || b.is_empty() {
        return false;
    }
    #[allow(clippy::cast_precision_loss)]
    let (na, nb) = (a.len() as f64, b.len() as f64);
    let (ma, mb) = (a.iter().sum::<f64>() / na, b.iter().sum::<f64>() / nb);
    let va = a.iter().map(|y| (y - ma).powi(2)).sum::<f64>() / na;
    let vb = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>() / nb;
    let pooled = va.midpoint(vb).sqrt();
    let means_close = if pooled > 0.0 {
        (ma - mb).abs() <= 0.25 * pooled
    } else {
        (ma - mb).abs() == 0.0
    };
    let ratio = if va.min(vb) > 0.0 {
        va.max(vb) / va.min(vb)
    } else {
        1.0
    };
    means_close && ratio <= 2.0
}

/// The standard deviation of a set of values, over `n` rather than `n − 1`.
///
/// B2 is a ratio of two of these at the same `n`, so the choice of denominator cancels; over `n` is
/// used because it is what the rest of this module uses.
fn sd(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n).sqrt()
}

/// The mean of the `TRAIL` periods ending at `at`, inclusive; `None` if the history is too short.
fn trailing_mean(series: &[f64], at: usize) -> Option<f64> {
    let start = at.checked_sub(TRAIL - 1)?;
    let slice = series.get(start..=at)?;
    #[allow(clippy::cast_precision_loss)]
    let n = slice.len() as f64;
    Some(slice.iter().sum::<f64>() / n)
}

/// B2: the cross-seed spread of the trailing-52 mean, at the end of the window over at its start.
///
/// This is the one test of the four that reads the ensemble rather than a path, and it is the reason
/// `E = 16` exists at all: a single seed cannot say whether the spread across seeds has stopped
/// changing.
fn b2(ensemble: &[Vec<f64>], gate: usize) -> bool {
    let Some(start) = gate.checked_sub(W - 1) else {
        return false;
    };
    let (mut at_start, mut at_end) = (Vec::new(), Vec::new());
    for series in ensemble {
        let (Some(a), Some(b)) = (trailing_mean(series, start), trailing_mean(series, gate)) else {
            return false;
        };
        at_start.push(a);
        at_end.push(b);
    }
    let (s0, s1) = (sd(&at_start), sd(&at_end));
    if s0 <= 0.0 {
        // No spread at the opening is not a settled ensemble, it is one seed repeated.
        return false;
    }
    let r = s1 / s0;
    (0.80..=1.25).contains(&r)
}

/// B3: Spearman's rho across seeds between the period-0 value and the value at the gate.
fn b3(ensemble: &[Vec<f64>], gate: usize) -> bool {
    let (mut opening, mut current) = (Vec::new(), Vec::new());
    for series in ensemble {
        let (Some(a), Some(b)) = (series.first(), series.get(gate)) else {
            return false;
        };
        opening.push(*a);
        current.push(*b);
    }
    spearman_of(&opening, &current).abs() <= RHO_CRIT
}

/// Spearman's rho: Pearson's correlation on ranks, with ties averaged.
pub fn spearman_of(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.len() < 2 {
        return 0.0;
    }
    let (ra, rb) = (ranks(a), ranks(b));
    #[allow(clippy::cast_precision_loss)]
    let n = ra.len() as f64;
    let (ma, mb) = (ra.iter().sum::<f64>() / n, rb.iter().sum::<f64>() / n);
    let mut num = 0.0;
    let (mut da, mut db) = (0.0, 0.0);
    for (x, y) in ra.iter().zip(rb.iter()) {
        num += (x - ma) * (y - mb);
        da += (x - ma).powi(2);
        db += (y - mb).powi(2);
    }
    if da <= 0.0 || db <= 0.0 {
        return 0.0;
    }
    num / (da * db).sqrt()
}

/// Ranks, one-based, with tied values sharing the average of the ranks they span.
fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&i, &j| match (values.get(i), values.get(j)) {
        (Some(x), Some(y)) => x.partial_cmp(y).unwrap_or(core::cmp::Ordering::Equal),
        _ => core::cmp::Ordering::Equal,
    });
    let mut out = vec![0.0; values.len()];
    let mut i = 0;
    while i < order.len() {
        let mut j = i + 1;
        while let (Some(&oi), Some(&oj)) = (order.get(i), order.get(j))
            && let (Some(x), Some(y)) = (values.get(oi), values.get(oj))
            && (x - y).abs() <= 0.0
        {
            j += 1;
        }
        #[allow(clippy::cast_precision_loss)]
        let shared = ((i + j - 1) as f64) / 2.0 + 1.0;
        for &o in order.get(i..j).unwrap_or_default() {
            if let Some(slot) = out.get_mut(o) {
                *slot = shared;
            }
        }
        i = j;
    }
    out
}

/// One stationary AR(1) series: the shape §15.3's flagged series are supposed to have by the gate.
fn ar1(seed: u64, stream: u64, n: usize, phi: f64) -> Vec<f64> {
    let mut out = Vec::with_capacity(n);
    let mut x = 0.0;
    for i in 0..n {
        #[allow(clippy::cast_possible_truncation)]
        let e = normal(seed, stream, i as u64);
        x = phi * x + e;
        out.push(x + 100.0);
    }
    out
}

/// A random walk: the shape B1 exists to reject.
fn walk(seed: u64, stream: u64, n: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(n);
    let mut x = 100.0;
    for i in 0..n {
        #[allow(clippy::cast_possible_truncation)]
        let e = normal(seed, stream, i as u64);
        x += e;
        out.push(x);
    }
    out
}

/// A mid-window level break: the shape B1b exists to reject.
fn broken(seed: u64, stream: u64, n: usize) -> Vec<f64> {
    ar1(seed, stream, n, 0.5)
        .into_iter()
        .enumerate()
        .map(|(i, x)| if i >= n / 2 { x + 5.0 } else { x })
        .collect()
}

/// How wide the ensemble's openings are spread, in the headline table. B2 and B3 are both about the
/// opening, so an ensemble whose seeds all start in the same place cannot exercise either.
const OPENING_SPREAD: f64 = 10.0;

/// The shapes an ensemble can have, and what §15.3 says each should be caught by.
#[derive(Clone, Copy)]
pub enum Shape {
    /// AR(1) with the given `phi`, from dispersed openings. At `phi = 0.5` it mixes within a few
    /// periods; at `phi = 0.98` it is still carrying its opening a hundred periods later.
    Ar1(f64),
    /// A random walk from dispersed openings: the cross-seed spread grows without bound, which is
    /// what B2 exists to catch.
    Walk,
    /// A path that never leaves its opening: B3's target, and B2 sees nothing wrong with it.
    Frozen,
}

/// `E` series of length `n`, one per seed, all of the same shape, from openings `spread` wide.
fn ensemble(shape: Shape, base: u64, n: usize, spread: f64) -> Vec<Vec<f64>> {
    ensemble_of(shape, base, E, n, spread)
}

/// The same, with the ensemble size given: ADR-0019's calibration needs to vary `E`.
#[must_use]
pub fn ensemble_of(shape: Shape, base: u64, seeds: usize, n: usize, spread: f64) -> Vec<Vec<f64>> {
    (0..seeds)
        .map(|e| {
            let seed = base + e as u64;
            let x0 = 100.0 + spread * normal(seed, 7, 0);
            let mut out = Vec::with_capacity(n);
            let mut x = x0;
            for i in 0..n {
                #[allow(clippy::cast_possible_truncation)]
                let noise = normal(seed, 3, i as u64);
                x = match shape {
                    Shape::Ar1(phi) => 100.0 + phi * (x - 100.0) + noise,
                    Shape::Walk => x + noise,
                    Shape::Frozen => x0 + 0.01 * noise,
                };
                out.push(x);
            }
            out
        })
        .collect()
}

/// B2 and B3 against ensembles whose answers are known, and the four-test conjunction.
fn ensemble_report() -> String {
    let mut out = String::new();
    for line in [
        String::new(),
        format!("  B2 and B3 need an ensemble, so these are over {E} seeds with openings spread"),
        format!(
            "  {OPENING_SPREAD:.0} wide. Rates are over {ENSEMBLE_TRIALS} independent ensembles at the gate period {GATE}."
        ),
    ] {
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(
        out,
        "  ensemble                      B1      B1b     B2      B3      all four"
    );

    let shapes = [
        ("mixing AR(1), phi 0.5", Shape::Ar1(0.5)),
        ("slow AR(1), phi 0.98", Shape::Ar1(0.98)),
        ("random walk", Shape::Walk),
        ("frozen at its opening", Shape::Frozen),
    ];
    let mut settled_all = 0.0;
    for (name, shape) in shapes {
        let (mut n1, mut n1b, mut n2, mut n3, mut nall) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for panel in 0..ENSEMBLE_TRIALS {
            let ens = ensemble(
                shape,
                900_000 + panel * u64::try_from(E).unwrap_or(16),
                GATE + 1,
                OPENING_SPREAD,
            );
            let v = verdict(&ens, GATE);
            n1 += u32::from(v.b1);
            n1b += u32::from(v.b1b);
            n2 += u32::from(v.b2);
            n3 += u32::from(v.b3);
            nall += u32::from(v.all());
        }
        let d = f64::from(u32::try_from(ENSEMBLE_TRIALS).unwrap_or(400));
        let all = f64::from(nall) / d;
        if matches!(shape, Shape::Ar1(phi) if phi <= 0.5) {
            settled_all = all;
        }
        let _ = writeln!(
            out,
            "  {name:<28}  {:.3}   {:.3}   {:.3}   {:.3}   {all:.3}",
            f64::from(n1) / d,
            f64::from(n1b) / d,
            f64::from(n2) / d,
            f64::from(n3) / d,
        );
    }

    // B2's ratio is a function of the opening spread, and the opening spread is not modelled. If
    // that is what drives it, widening the spread must close the gap between a settled ensemble and
    // a diverging one — so measure the gap at three spreads rather than assert it at one.
    for line in [
        String::new(),
        "  B2 alone, against the spread the seeds opened with:".to_owned(),
        String::new(),
        "  opening spread    settled    random walk    gap".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    for spread in [0.5_f64, 2.0, 10.0, 40.0] {
        let rate = |shape: Shape| {
            let mut n = 0u32;
            for panel in 0..ENSEMBLE_TRIALS {
                let ens = ensemble(
                    shape,
                    700_000 + panel * u64::try_from(E).unwrap_or(16),
                    GATE + 1,
                    spread,
                );
                n += u32::from(b2(&ens, GATE));
            }
            f64::from(n) / f64::from(u32::try_from(ENSEMBLE_TRIALS).unwrap_or(400))
        };
        let (settled, walk) = (rate(Shape::Ar1(0.5)), rate(Shape::Walk));
        let _ = writeln!(
            out,
            "  {spread:>13.1}    {settled:>7.3}    {walk:>11.3}    {:>+.3}",
            settled - walk
        );
    }

    // The whole gate, on the world it is supposed to pass: 42 series, four tests, all at once.
    let flagged = 42i32;
    let panel = settled_all.powi(flagged);
    for line in [
        String::new(),
        format!("  The settled ensemble passes all four on {settled_all:.3} of series. \u{a7}15.3 requires all 42"),
        format!("  flagged series to pass at the same period, so the whole panel passes {panel:.2e} of the time."),
        "  **The gate as specified cannot fire on a world that has settled.** Reaching period 520 without".to_owned(),
        "  a pass is what \u{a7}15.3 calls a defect; on these numbers it is the certain outcome.".to_owned(),
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// The gate period the ensemble tests are evaluated at: inside §15.3's [260, 520] bracket, and far
/// enough in that B2's start-of-window trailing mean has its 52 periods of history.
const GATE: usize = 260;
/// How many independent ensembles each rate is measured over.
const ENSEMBLE_TRIALS: u64 = 400;

/// All four tests on one ensemble.
///
/// **§15.3 does not say which series B1 and B1b are evaluated on**, and there are `E` of them per
/// flagged quantity. It says the panel is 42 series and that the gate is 168 hypotheses, which is
/// 42 x 4 — so B1 and B1b are one hypothesis per flagged quantity, not one per seed. The only
/// reading that gives 168 is that the path tests run on the ensemble mean, and that is what this
/// does. It is an inference, not a quotation, and it is recorded as such.
fn verdict(ensemble: &[Vec<f64>], gate: usize) -> Verdict {
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
    let mean_path: Vec<f64> = (start..=gate.min(first.len().saturating_sub(1)))
        .map(|t| ensemble.iter().filter_map(|s| s.get(t)).sum::<f64>() / seeds)
        .collect();
    Verdict {
        b1: b1(&mean_path),
        b1b: b1b(&mean_path),
        b2: b2(ensemble, gate),
        b3: b3(ensemble, gate),
    }
}

/// Characterise the tests against known shapes, then measure the conjunction's behaviour.
pub fn report() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "burn-in — §15.3's tests against series whose answer is known\n"
    );

    // Known answers, over an ensemble so a single unlucky draw is not the finding.
    let rate = |f: &dyn Fn(u64) -> Vec<f64>, test: &dyn Fn(&[f64]) -> bool| {
        #[allow(clippy::cast_precision_loss)]
        let passes = (0..200u64).filter(|s| test(&f(*s))).count() as f64 / 200.0;
        passes
    };
    let stationary = |s: u64| ar1(s, 1, W, 0.5);
    let rw = |s: u64| walk(s, 2, W);
    let brk = |s: u64| broken(s, 3, W);

    let _ = writeln!(
        out,
        "  {:<26} {:>10} {:>10}",
        "series", "B1 pass", "B1b pass"
    );
    let _ = writeln!(
        out,
        "  {:<26} {:>10.3} {:>10.3}",
        "stationary AR(1)",
        rate(&stationary, &b1),
        rate(&stationary, &b1b)
    );
    let _ = writeln!(
        out,
        "  {:<26} {:>10.3} {:>10.3}",
        "random walk",
        rate(&rw, &b1),
        rate(&rw, &b1b)
    );
    let _ = writeln!(
        out,
        "  {:<26} {:>10.3} {:>10.3}",
        "mid-window level break",
        rate(&brk, &b1),
        rate(&brk, &b1b)
    );

    // The conjunction. §15.3 requires ALL of the flagged series to pass ALL of the tests, at one
    // period, before that period can be `burnInPeriod`.
    let panels = 2000u64;
    let flagged = 42usize;
    let mut all_pass = 0usize;
    for p in 0..panels {
        let ok = (0..flagged).all(|k| {
            #[allow(clippy::cast_possible_truncation)]
            let s = ar1(p * 1000 + k as u64, 7, W, 0.5);
            b1(&s) && b1b(&s)
        });
        if ok {
            all_pass += 1;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let conj = all_pass as f64 / panels as f64;

    let _ = writeln!(
        out,
        "\n  the conjunction, over {panels} panels of {flagged} genuinely stationary series:\n  \
         all {flagged} series pass both B1 and B1b together in {all_pass} panels — {:.1}%",
        conj * 100.0
    );

    let _ = write!(out, "{}", calibration());
    let _ = write!(out, "{}", ensemble_report());

    for line in [
        "",
        "  FINDING. \u{a7}15.3's gate is a conjunction over 42 flagged series and four tests \u{2014} 168",
        "  hypotheses \u{2014} and it makes `burnInPeriod` the FIRST period at which every one of them passes,",
        "  while making arrival at period 520 without a pass a DEFECT. Measured against worlds whose",
        "  answers are known, the risk is the opposite of the one the specification worries about: not",
        "  that the gate fires on noise, but that **a settled world never passes it**, after which",
        "  \u{a7}16.2 classifies a healthy model as defective.",
        "",
        "  Where each test actually stands, measured rather than assumed:",
        "",
        "  - B1 is the only one of the four that behaves. It passes every settled series and its",
        "    failures are informative.",
        "  - B1b rejects the majority of settled series, and no window length fixes it. `|d-mu| <=",
        "    0.25 sigma_pooled` is a FIXED EFFECT SIZE with no reference to sampling error, so it is not",
        "    a five-per-cent test at any n; on an autocorrelated series it rejects most of the time.",
        "  - B2 does not merely lack power, it INVERTS. Its ratio is dominated by the spread the seeds",
        "    OPENED with, which is not a modelled quantity at all: at a spread of 0.5 it separates the",
        "    settled ensemble from a random walk by +0.27, at 10 the gap is gone, and at 40 a random",
        "    walk passes B2 every time while the settled ensemble passes 0.58 \u{2014} a gap of -0.42, the",
        "    wrong way round. A test whose verdict is decided by an arbitrary opening is not a test.",
        "    The settled rate of 0.58 is besides too low for a gate: a ratio of two standard deviations",
        "    at E = 16 is far too noisy for a [0.80, 1.25] band.",
        "  - B3 does what it says. It rejects the frozen path every time. On the random walk it lands on",
        "    its own critical value, because a walk from an opening spread of 10 has a true period-0",
        "    correlation near 0.53 at period 260 \u{2014} so B3's verdict there is arithmetic, not chance.",
        "",
        "  So the multiplicity correction \u{a7}15.3 asks for is the SECOND problem, not the first. Three of",
        "  the four tests have no calibrated null \u{2014} B3's 0.503 is the only critical value in the gate",
        "  that was derived from a distribution rather than chosen \u{2014} and a correction applied to",
        "  mis-specified tests corrects nothing. ADR-0019 takes both, in that order.",
    ] {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// What threshold would make B1b a five-per-cent test? Measure the statistic it thresholds.
fn calibration() -> String {
    let mut out = String::new();
    let mut ratios: Vec<f64> = (0..4000u64)
        .map(|s| {
            let series = ar1(s, 11, W, 0.5);
            let half = series.len() / 2;
            let (a, b) = series.split_at(half);
            #[allow(clippy::cast_precision_loss)]
            let (na, nb) = (a.len() as f64, b.len() as f64);
            let (ma, mb) = (a.iter().sum::<f64>() / na, b.iter().sum::<f64>() / nb);
            let va = a.iter().map(|y| (y - ma).powi(2)).sum::<f64>() / na;
            let vb = b.iter().map(|y| (y - mb).powi(2)).sum::<f64>() / nb;
            let pooled = va.midpoint(vb).sqrt();
            if pooled > 0.0 {
                (ma - mb).abs() / pooled
            } else {
                0.0
            }
        })
        .collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let pct = |q: f64| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let i = ((ratios.len() as f64 - 1.0) * q) as usize;
        ratios.get(i).copied().unwrap_or(0.0)
    };
    let _ = writeln!(
        out,
        "\n  B1b thresholds |Δμ| / σ_pooled at 0.25. On a stationary AR(1) that statistic has\n  \
         median {:.3}, 95th percentile {:.3}, 99th percentile {:.3}.\n  \
         A 0.25 band is therefore roughly a {:.0}th-percentile cut, not a five-per-cent test:\n  \
         **it rejects the majority of genuinely stationary series.**",
        pct(0.50),
        pct(0.95),
        pct(0.99),
        {
            #[allow(clippy::cast_precision_loss)]
            let below = ratios.iter().filter(|r| **r <= 0.25).count() as f64 / ratios.len() as f64;
            below * 100.0
        }
    );

    out
}
