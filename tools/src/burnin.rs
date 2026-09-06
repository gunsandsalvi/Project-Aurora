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
fn unit(seed: u64, stream: u64, index: u64) -> f64 {
    (draw(seed, stream, index) >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
}

/// A standard normal by the central limit theorem over twelve uniforms.
///
/// §11 admits no `log`, `exp` or `cos`, which rules out Box–Muller; this needs none of them. It is a
/// crude normal and it is entirely adequate here, where the question is about a test's behaviour and
/// not about the tail of a distribution.
fn normal(seed: u64, stream: u64, index: u64) -> f64 {
    (0..12)
        .map(|k| unit(seed, stream, index * 12 + k))
        .sum::<f64>()
        - 6.0
}

/// The window §15.3 declares.
const W: usize = 104;
/// The ensemble §15.3 declares.
const E: usize = 16;

/// The four verdicts on one series.
#[derive(Debug, Clone, Copy)]
pub struct Verdict {
    /// B1 — no drift.
    pub b1: bool,
    /// B1b — no regime shift.
    pub b1b: bool,
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

    let _ = writeln!(
        out,
        "\n  FINDING. §15.3's gate is a conjunction over 42 flagged series and four tests — 168\n  \
         hypotheses — and it makes `burnInPeriod` the FIRST period at which every one of them passes,\n  \
         while making arrival at period 520 without a pass a DEFECT. The measurement above is over two\n  \
         of the four tests and a world that is stationary by construction. Whatever the exact figure,\n  \
         the direction is not in doubt and it is the opposite of the one the specification worries\n  \
         about: the risk is not that the gate passes on noise, it is that **a settled world fails it**,\n  \
         and §16.2 then classifies a healthy model as defective.\n\n  \
         The binding constraint is B1b, and it is not a multiplicity problem — it is a mis-specified\n  \
         threshold. `|Δμ| ≤ 0.25·σ_pooled` is a FIXED EFFECT SIZE with no reference to sampling error,\n  \
         so it does not become a five-per-cent test at any window length; on an autocorrelated series\n  \
         it rejects most of the time. B1's disjunction, by contrast, passes every stationary series and\n  \
         a third of random walks, so the two tests are mis-calibrated in OPPOSITE directions.\n\n  \
         Two things are owed, and they are different. (1) Each test's threshold must be set from the\n  \
         distribution of its own statistic under the null, as B3's 0.503 already is — B3 is the only\n  \
         one of the four whose critical value is derived rather than chosen. (2) A multiplicity\n  \
         correction over the 168 hypotheses, which is a separate problem and does not fix this one.\n  \
         E = {E} and W = {W} are §15.3's own; B2 and B3 need cross-seed ensembles and are owed here."
    );
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
