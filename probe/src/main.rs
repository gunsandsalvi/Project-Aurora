//! # `probe` — what the device actually does
//!
//! Founding decision D4: the measurements §12 is derived from, taken on real hardware, printed as one
//! JSON document the owner copies back. It runs on the host too, and a host figure is a **floor**:
//! whatever an x86 desktop costs, a phone costs at least that.
//!
//! The measurement everything rests on is §12.2's **96.5 ns per exchange** — 478.7 ms over 3,119,665
//! calls, of which position 14's 1,671,884 exchanges take 161.3 ms. An exchange performs two block
//! lookups into a structure far larger than any cache, four accrual read-modify-writes, two quantity
//! writes, a quantization and a 48-byte journal append. On a phone's memory system that budget buys
//! roughly one cache miss.
//!
//! It also answers a question M1 would otherwise decide by taste: **columnar or interleaved?** §5.1
//! says columns, and for a walk over one column that is right. A holdings slot is read at a *random*
//! index by four fields at once, so columnar costs four cache misses where interleaved costs one.
//! Both are measured here rather than argued.

use std::time::Instant;

/// §3.4's slot count, at the corrected 24-byte width.
const SLOTS: usize = 7_177_280;
/// How many operations each measurement performs.
const OPS: usize = 2_000_000;

/// A holdings slot, interleaved: one cache line serves the whole read.
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Slot {
    asset: i32,
    tick: u16,
    _pad: u16,
    quantity: i64,
    integral: i64,
}

/// Columnar: §5.1's layout, four arrays touched at one random index.
struct Columns {
    asset: Vec<i32>,
    tick: Vec<u16>,
    quantity: Vec<i64>,
    integral: Vec<i64>,
}

/// A counter-based generator, so an index does not depend on how many preceded it (§11).
fn draw(i: u64) -> u64 {
    let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x.wrapping_mul(0x94D0_49BB_1331_11EB)
}

fn main() {
    println!("aurora probe — schema aurora.probe/1\n");

    let bytes = SLOTS * size_of::<Slot>();
    println!(
        "  arena: {SLOTS} slots x {} B = {:.1} MiB",
        size_of::<Slot>(),
        bytes as f64 / 1_048_576.0
    );

    // ── interleaved ────────────────────────────────────────────────────────────────────────────
    let mut interleaved: Vec<Slot> = vec![Slot::default(); SLOTS];
    let mut journal: Vec<u8> = Vec::with_capacity(OPS * 48);
    let t = Instant::now();
    let mut sink = 0i64;
    for i in 0..OPS as u64 {
        // Two legs, as an `exchange` has.
        for leg in 0..2u64 {
            let k = (draw(i * 2 + leg) as usize) % SLOTS;
            if let Some(s) = interleaved.get_mut(k) {
                // §6.11: both accrual columns updated before the quantity is written.
                s.integral = s
                    .integral
                    .wrapping_add(s.quantity * i64::from(17u16 - s.tick.min(17)));
                s.tick = (i % 1_560) as u16;
                s.quantity = s.quantity.wrapping_add(1);
                sink = sink.wrapping_add(s.quantity);
            }
        }
        journal.extend_from_slice(&[0u8; 48]);
        if journal.len() > OPS * 24 {
            journal.clear();
        }
    }
    let interleaved_ns = t.elapsed().as_nanos() as f64 / OPS as f64;
    drop(interleaved);

    // ── columnar ───────────────────────────────────────────────────────────────────────────────
    let mut cols = Columns {
        asset: vec![0; SLOTS],
        tick: vec![0; SLOTS],
        quantity: vec![0; SLOTS],
        integral: vec![0; SLOTS],
    };
    let t = Instant::now();
    for i in 0..OPS as u64 {
        for leg in 0..2u64 {
            let k = (draw(i * 2 + leg) as usize) % SLOTS;
            let (Some(q), Some(g), Some(tk), Some(a)) = (
                cols.quantity.get(k).copied(),
                cols.integral.get(k).copied(),
                cols.tick.get(k).copied(),
                cols.asset.get(k).copied(),
            ) else {
                continue;
            };
            if let Some(slot) = cols.integral.get_mut(k) {
                *slot = g.wrapping_add(q * i64::from(17u16 - tk.min(17)));
            }
            if let Some(slot) = cols.tick.get_mut(k) {
                *slot = (i % 1_560) as u16;
            }
            if let Some(slot) = cols.quantity.get_mut(k) {
                *slot = q.wrapping_add(1);
            }
            sink = sink.wrapping_add(i64::from(a));
        }
        journal.extend_from_slice(&[0u8; 48]);
        if journal.len() > OPS * 24 {
            journal.clear();
        }
    }
    let columnar_ns = t.elapsed().as_nanos() as f64 / OPS as f64;

    // ── the noise floor: the same loop with no arena touch ─────────────────────────────────────
    let t = Instant::now();
    for i in 0..OPS as u64 {
        sink = sink.wrapping_add(draw(i) as i64);
    }
    let floor_ns = t.elapsed().as_nanos() as f64 / OPS as f64;

    // ── sorted-block insert and remove, at the four widths that matter ─────────────────────────
    let mut block_report = String::new();
    for width in [10usize, 256, 4_096, 16_384] {
        let mut block: Vec<i64> = (0..width as i64).collect();
        let reps = 200_000usize;
        let t = Instant::now();
        for i in 0..reps as u64 {
            let v = (draw(i) as i64).rem_euclid(width as i64);
            let at = block.partition_point(|x| *x < v);
            block.insert(at, v);
            block.remove(at);
        }
        let ns = t.elapsed().as_nanos() as f64 / reps as f64;
        block_report.push_str(&format!(
            "    {width:>6} slots: {ns:>8.1} ns per insert+remove\n"
        ));
    }

    // ── transcendental bit-identity: this host's half of the comparison ────────────────────────
    let mut h_ln: u64 = 0xcbf2_9ce4_8422_2325;
    let mut h_exp: u64 = 0xcbf2_9ce4_8422_2325;
    for i in 1..=4_096u64 {
        let x = i as f64 / 512.0;
        for (h, v) in [(&mut h_ln, x.ln()), (&mut h_exp, (x / 8.0).exp())] {
            for b in v.to_bits().to_be_bytes() {
                *h ^= u64::from(b);
                *h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
    }

    println!(
        "\n  operation cost, per two-leg exchange over a {:.0} MiB arena:",
        bytes as f64 / 1_048_576.0
    );
    println!("    interleaved slot : {interleaved_ns:>8.1} ns");
    println!("    columnar (§5.1)  : {columnar_ns:>8.1} ns");
    println!("    noise floor      : {floor_ns:>8.1} ns  (the same loop, no arena touch)");
    println!("    §12.2's budget   :     96.5 ns");
    println!("\n  sorted-block insert and remove:\n{block_report}");
    println!("  transcendental bit patterns over 4,096 inputs:");
    println!("    ln  : {h_ln:016x}");
    println!("    exp : {h_exp:016x}");
    println!("\n  (sink {sink}, so nothing above is optimised away)");
}
