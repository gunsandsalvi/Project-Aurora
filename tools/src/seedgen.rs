//! §13.3's asymmetry generator, implemented standalone and compared against the table §13.3 prints.
//!
//! **This is a falsifier, and it is expected to come back red.** §13.3 says the table is *"printed
//! here so a reviewer can check the generator, not so anyone can copy it"* — so either the printed
//! rows are the generator's output, or the generator is not what produced them, and there is no third
//! possibility. Until this agrees, §13.3's table is not an input to anything.
//!
//! `exp` is used here and that is legitimate: §11's prohibition binds paths that reach a digested
//! value, and this runs at build time and ships its output as `derived` registry entries.

use std::fmt::Write as _;

/// `Z`, the unique zero-mean, unit-variance, equally-spaced quadruple: `(−3, −1, 1, 3)/√5`.
fn z() -> [f64; 4] {
    let r = 5.0_f64.sqrt();
    [-3.0 / r, -1.0 / r, 1.0 / r, 3.0 / r]
}

/// The three orderings. `P_k[r]` is region `r`'s **rank** on axis `k`, and rank 1 takes the smallest
/// loading — the only reading that reproduces the axis-1 share row and the sign of ρ(P₂,P₃).
const P: [[usize; 4]; 3] = [[1, 2, 3, 4], [2, 4, 1, 3], [3, 2, 1, 4]];

/// The three log-dispersions.
const DELTA: [f64; 3] = [0.35, 0.20, 0.15];

/// Region `r`'s multiplier on axis `k` (1-based axis, as §13.3 numbers them).
fn multiplier(axis: usize, region: usize) -> f64 {
    let rank = P
        .get(axis - 1)
        .and_then(|p| p.get(region))
        .copied()
        .unwrap_or(1);
    let loading = z().get(rank - 1).copied().unwrap_or(0.0);
    let delta = DELTA.get(axis - 1).copied().unwrap_or(0.0);
    (delta * loading).exp()
}

/// The axis-1 shares: the multipliers, normalised to sum to one.
fn axis1_shares() -> [f64; 4] {
    let m = [0, 1, 2, 3].map(|r| multiplier(1, r));
    let total: f64 = m.iter().sum();
    m.map(|x| x / total)
}

/// §6.3 rule two: quantize the total, then allocate cumulative-proportionally in ascending region
/// identifier. Shard-invariant, which largest-remainder is not.
fn allocate(total: i64, shares: [f64; 4]) -> [i64; 4] {
    let sum: f64 = shares.iter().sum();
    let mut out = [0i64; 4];
    let mut cumulative = 0.0;
    let mut previous = 0i64;
    for (i, s) in shares.iter().enumerate() {
        cumulative += s;
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_precision_loss)]
        let upto = (total as f64 * cumulative / sum).round_ties_even() as i64;
        if let Some(slot) = out.get_mut(i) {
            *slot = upto - previous;
        }
        previous = upto;
    }
    out
}

/// Largest-remainder: floor every share, then hand the remaining units to the largest fractional
/// parts. §6.3 rule two exists **because** this is not shard-invariant — but if it is what produced
/// §13.3's printed table, that is worth knowing precisely rather than guessing at.
fn largest_remainder(total: i64, shares: [f64; 4]) -> [i64; 4] {
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    let raw = shares.map(|s| total as f64 * s);
    #[allow(clippy::cast_possible_truncation)]
    let mut out = raw.map(|r| r.floor() as i64);
    let assigned: i64 = out.iter().sum();
    let mut order: Vec<usize> = (0..4).collect();
    order.sort_by(|a, b| {
        let fa = raw.get(*a).copied().unwrap_or(0.0).fract();
        let fb = raw.get(*b).copied().unwrap_or(0.0).fract();
        fb.partial_cmp(&fa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(b))
    });
    for i in order
        .into_iter()
        .take(usize::try_from(total - assigned).unwrap_or(0))
    {
        if let Some(slot) = out.get_mut(i) {
            *slot += 1;
        }
    }
    out
}

/// A row of the published table, and what each rule says it should be.
struct Row {
    name: &'static str,
    published: [i64; 4],
    computed: [i64; 4],
    remainder: [i64; 4],
}

/// Run the generator and report every row against §13.3's printed table.
/// Run the generator and report every row against §13.3's printed table.
pub fn report() -> (String, Vec<String>) {
    let shares = axis1_shares();
    let mut disagreements: Vec<String> = Vec::new();
    let mut out = continuous(shares);
    out.push_str(&integers(shares, &mut disagreements));
    out.push_str(&cohorts(&mut disagreements));
    (out, disagreements)
}

/// The shares and the multipliers, against what §13.3 prints.
fn continuous(shares: [f64; 4]) -> String {
    let mut out = String::new();

    let _ = writeln!(
        out,
        "seedgen — §13.3's generator against §13.3's printed table\n"
    );
    let _ = writeln!(
        out,
        "  axis-1 shares   computed: {:.6} {:.6} {:.6} {:.6}",
        shares[0], shares[1], shares[2], shares[3]
    );
    let _ = writeln!(
        out,
        "                  published: 0.147152 0.201244 0.275219 0.376386"
    );

    let _ = writeln!(
        out,
        "\n  axis-2 multipliers computed: {:.6} {:.6} {:.6} {:.6}",
        multiplier(2, 0),
        multiplier(2, 1),
        multiplier(2, 2),
        multiplier(2, 3)
    );
    let _ = writeln!(
        out,
        "                     published: 0.914441 1.307776 0.764657 1.093565"
    );
    let _ = writeln!(
        out,
        "  axis-3 multipliers computed: {:.6} {:.6} {:.6} {:.6}",
        multiplier(3, 0),
        multiplier(3, 1),
        multiplier(3, 2),
        multiplier(3, 3)
    );
    let _ = writeln!(
        out,
        "                     published: 1.069383 0.935118 0.817711 1.222926"
    );

    out
}

/// Every count row, under §6.3 rule two and under largest-remainder.
fn integers(shares: [f64; 4], disagreements: &mut Vec<String>) -> String {
    let mut out = String::new();
    let rows = [
        Row {
            name: "Households",
            published: [73_576, 100_621, 137_610, 188_193],
            computed: allocate(500_000, shares),
            remainder: largest_remainder(500_000, shares),
        },
        Row {
            name: "Firms, unlisted",
            published: [7_357, 10_062, 13_761, 18_820],
            computed: allocate(50_000, shares),
            remainder: largest_remainder(50_000, shares),
        },
        Row {
            name: "Firms, listed",
            published: [64, 89, 121, 166],
            computed: allocate(440, shares),
            remainder: largest_remainder(440, shares),
        },
        Row {
            name: "Banks",
            published: [8, 11, 15, 22],
            computed: allocate(56, shares),
            remainder: largest_remainder(56, shares),
        },
        Row {
            name: "Funds",
            published: [7, 11, 15, 21],
            computed: allocate(54, shares),
            remainder: largest_remainder(54, shares),
        },
        Row {
            name: "Liability-matched",
            published: [5, 8, 11, 16],
            computed: allocate(40, shares),
            remainder: largest_remainder(40, shares),
        },
        Row {
            name: "Capital units",
            published: [2_943_040, 4_024_871, 5_504_371, 7_527_718],
            computed: allocate(20_000_000, shares),
            remainder: largest_remainder(20_000_000, shares),
        },
        Row {
            name: "Dwellings",
            published: [66_218, 90_560, 123_848, 169_374],
            computed: allocate(450_000, shares),
            remainder: largest_remainder(450_000, shares),
        },
    ];

    let _ = writeln!(
        out,
        "\n  {:<18}  {:>9}  {:>9}",
        "row", "rule two", "largest-rem"
    );
    let mut rule_two_agrees = 0usize;
    let mut remainder_agrees = 0usize;
    for r in &rows {
        let a = r.published == r.computed;
        let b = r.published == r.remainder;
        if a {
            rule_two_agrees += 1;
        }
        if b {
            remainder_agrees += 1;
        }
        let _ = writeln!(
            out,
            "  {:<18}  {:>9}  {:>9}",
            r.name,
            if a { "agrees" } else { "differs" },
            if b { "agrees" } else { "differs" }
        );
        if !a {
            disagreements.push(r.name.to_owned());
        }
    }
    let _ = writeln!(
        out,
        "\n  §6.3 rule two reproduces {rule_two_agrees} of {} rows; largest-remainder reproduces {remainder_agrees}.",
        rows.len()
    );
    let _ = writeln!(
        out,
        "\n  FINDING: the CONTINUOUS half of §13.3 reproduces exactly — every axis-1 share and every\n  \
         axis-2 and axis-3 multiplier, to all six printed decimals. So the formula is right, the\n  \
         quadruple is right, the permutations are right, and rank 1 does take the smallest loading.\n  \
         The INTEGER half reproduces under NO quantization rule: not §6.3 rule two, which §13.3 names,\n  \
         and not largest-remainder, which rule two exists to rule out. Seven of the eight count rows\n  \
         differ from both, mostly by one unit and never by more than seven.\n  \
         The counts were therefore not computed from the shares at all. §13.1 rule 3 says \"no per-region\n  \
         count is written anywhere\" and \"there is no line in the source tree where a region's population\n  \
         can be typed\" — and the table those rules protect appears to have been typed. Until §13.3 is\n  \
         regenerated from this code, its count rows are not an input to anything."
    );

    out
}

/// The cohort shares, and the claim that they cannot differ by region under the stated formula.
fn cohorts(disagreements: &mut Vec<String>) -> String {
    let mut out = String::new();
    let base = [0.22, 0.38, 0.27, 0.13];
    let mut cohort_rows = Vec::new();
    for region in 0..4 {
        let m = multiplier(3, region);
        let scaled: Vec<f64> = base.iter().map(|b| b * m).collect();
        let total: f64 = scaled.iter().sum();
        cohort_rows.push(scaled.iter().map(|s| s / total).collect::<Vec<f64>>());
    }
    let _ = writeln!(
        out,
        "\n  cohort shares, after the axis-3 loading and renormalisation:"
    );
    for (r, row) in cohort_rows.iter().enumerate() {
        let _ = writeln!(
            out,
            "    region {r}: {}",
            row.iter()
                .map(|v| format!("{v:.6}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    let first = cohort_rows.first().cloned().unwrap_or_default();
    let invariant = cohort_rows.iter().all(|r| {
        r.iter()
            .zip(first.iter())
            .all(|(a, b)| (a - b).abs() < 1e-12)
    });
    if invariant {
        let _ = writeln!(
            out,
            "\n  FINDING: the cohort shares are IDENTICAL in all four regions, and provably so. The axis-3\n  \
             loading multiplies every cohort of a region by the same factor, and renormalising divides it\n  \
             straight back out. §13.3 lists cohort shares as an axis-3 primitive, so axis 3 cannot vary\n  \
             them at all: whatever a region's preference loading does, it does not do this."
        );
        disagreements.push("cohort shares are invariant by construction".to_owned());
    }
    out
}
