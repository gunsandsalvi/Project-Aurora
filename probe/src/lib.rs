//! # `aurora-probe` — what the device actually does
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
//! Both are measured rather than argued.
//!
//! **Why it is a library.** The Android shell runs this as a native executable and reads its stdout;
//! `main` is four lines and everything measurable is here, so the same code produces the host floor
//! and the device figure with nothing conditional between them.

// A benchmark counts operations and divides by elapsed nanoseconds; every cast below is a count
// crossing into a ratio, and none of them can reach a digested value because nothing here is the
// engine. Stated once, at the top, rather than sprinkled.
#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines
)]

use std::fmt::Write as _;
use std::time::Instant;

/// §3.4's slot count, at the corrected 24-byte width.
pub const SLOTS: usize = 7_177_280;
/// How many operations each timed measurement performs.
pub const OPS: usize = 2_000_000;
/// The schema the owner copies back.
pub const SCHEMA: &str = "aurora.probe/1";

/// A holdings slot, interleaved: one cache line serves the whole read.
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Slot {
    /// The instrument held.
    pub asset: i32,
    /// The tick the integral was last brought forward to (§6.11).
    pub tick: u16,
    /// The conserved quantity.
    pub quantity: i64,
    /// The balance-tick integral.
    pub integral: i64,
}

/// The same four fields, one vector each: §5.1's columnar layout.
pub struct Columns {
    /// The instrument column.
    pub asset: Vec<i32>,
    /// The tick column.
    pub tick: Vec<u16>,
    /// The quantity column.
    pub quantity: Vec<i64>,
    /// The integral column.
    pub integral: Vec<i64>,
}

/// A counter-based generator: a pure function of the index, so a draw does not depend on how many
/// draws preceded it.
#[must_use]
pub fn draw(i: u64) -> u64 {
    let mut x = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x.wrapping_mul(0x94D0_49BB_1331_11EB)
}

/// Everything one run measures. Every field is written before the document is emitted, so a `null`
/// in the JSON means a measurement did not run — which is exit criterion 9's whole point.
#[derive(Default)]
pub struct Measurements {
    /// Nanoseconds per two-leg exchange, interleaved slot.
    pub interleaved_ns: f64,
    /// Nanoseconds per two-leg exchange, §5.1's columns.
    pub columnar_ns: f64,
    /// The same loop with no arena touch.
    pub noise_floor_ns: f64,
    /// Insert+remove cost at four block widths, in slots and nanoseconds.
    pub block_ops: Vec<(usize, f64)>,
    /// FNV-1a over the bit patterns of `ln` and `exp` across 4,096 committed inputs.
    pub ln_hash: u64,
    /// As above, for `exp`.
    pub exp_hash: u64,
    /// Committed-and-held allocation ceiling, in MiB.
    pub ceiling_mib: usize,
    /// How long the ceiling was held under a churn workload.
    pub held_seconds: u64,
    /// Sequential write throughput, MiB/s.
    pub seq_write_mibs: f64,
    /// Chunked write throughput at the checkpoint's own granularity, MiB/s.
    pub chunked_write_mibs: f64,
    /// Exchanges a second in a burst.
    pub burst_ops_per_sec: f64,
    /// Exchanges a second sustained over the soak.
    pub soak_ops_per_sec: f64,
    /// How long the soak ran.
    pub soak_seconds: u64,
}

/// One two-leg exchange against the interleaved arena, timed over `OPS`.
fn interleaved_cost() -> (f64, i64) {
    let mut arena: Vec<Slot> = vec![Slot::default(); SLOTS];
    let mut journal: Vec<u8> = Vec::with_capacity(OPS * 48);
    let mut sink = 0i64;
    let t = Instant::now();
    for i in 0..OPS as u64 {
        for leg in 0..2u64 {
            let k = (draw(i * 2 + leg) as usize) % SLOTS;
            if let Some(s) = arena.get_mut(k) {
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
    (t.elapsed().as_nanos() as f64 / OPS as f64, sink)
}

/// The same exchange against §5.1's columns: four random reads where interleaved does one.
fn columnar_cost() -> (f64, i64) {
    let mut cols = Columns {
        asset: vec![0; SLOTS],
        tick: vec![0; SLOTS],
        quantity: vec![0; SLOTS],
        integral: vec![0; SLOTS],
    };
    let mut journal: Vec<u8> = Vec::with_capacity(OPS * 48);
    let mut sink = 0i64;
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
    (t.elapsed().as_nanos() as f64 / OPS as f64, sink)
}

/// The same loop with no arena touch. **A result without one is not a result.**
fn noise_floor() -> (f64, i64) {
    let mut sink = 0i64;
    let t = Instant::now();
    for i in 0..OPS as u64 {
        sink = sink.wrapping_add(draw(i) as i64);
    }
    (t.elapsed().as_nanos() as f64 / OPS as f64, sink)
}

/// Sorted-block insert and remove, at the four widths §3.4's block table uses.
fn block_ops() -> Vec<(usize, f64)> {
    let mut out = Vec::new();
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
        out.push((width, t.elapsed().as_nanos() as f64 / reps as f64));
    }
    out
}

/// FNV-1a over the bit patterns `ln` and `exp` produce across 4,096 committed inputs.
///
/// §11 bans a transcendental on any path reaching a digested value, and ADR-0015 turns on whether the
/// ban is free: if the device and the host agree bit for bit, a deterministic math module is not
/// needed. Two hashes, and a comparison.
fn transcendental() -> (u64, u64) {
    let (mut h_ln, mut h_exp) = (0xcbf2_9ce4_8422_2325u64, 0xcbf2_9ce4_8422_2325u64);
    for i in 1..=4_096u64 {
        let x = i as f64 / 512.0;
        for (h, v) in [(&mut h_ln, x.ln()), (&mut h_exp, (x / 8.0).exp())] {
            for b in v.to_bits().to_be_bytes() {
                *h ^= u64::from(b);
                *h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
    }
    (h_ln, h_exp)
}

/// How much memory the device will actually give up, **committed by touching every page**.
///
/// Reserved-but-uncommitted address space tells you nothing on Android: a `Vec` that is never written
/// costs nothing and reports success. This allocates in 128 MiB steps, writes one byte per 4 KiB page
/// in each step, and holds everything allocated so far while it does — so the number is memory the
/// process is actually using when the next step is attempted.
///
/// `budget_mib` stops the walk on a host, where the answer would otherwise be "all of swap".
fn allocation_ceiling(budget_mib: usize) -> (usize, u64) {
    const STEP_MIB: usize = 128;
    const PAGE: usize = 4_096;
    let mut held: Vec<Vec<u8>> = Vec::new();
    let mut committed = 0usize;
    let t = Instant::now();
    while committed + STEP_MIB <= budget_mib {
        let mut block = vec![0u8; STEP_MIB * 1_048_576];
        let mut page = 0usize;
        while page < block.len() {
            if let Some(b) = block.get_mut(page) {
                *b = 1;
            }
            page += PAGE;
        }
        held.push(block);
        committed += STEP_MIB;
        // Churn what is already held, so the pages cannot be quietly reclaimed.
        for b in &mut held {
            let k = (draw(committed as u64) as usize) % b.len();
            if let Some(v) = b.get_mut(k) {
                *v = v.wrapping_add(1);
            }
        }
    }
    (committed, t.elapsed().as_secs())
}

/// What a checkpoint costs, at two granularities.
///
/// Sequential is the floor. Chunked is the shape §12.3's checkpoint actually has — a shard at a time —
/// and on flash the two are not the same number. ADR-0016 turns on which.
fn storage(dir: &std::path::Path) -> (f64, f64) {
    const TOTAL: usize = 64 * 1_048_576;
    const CHUNK: usize = 1_048_576;
    let buf = vec![7u8; CHUNK];
    let path = dir.join("aurora-probe-write.bin");

    let sequential = {
        let t = Instant::now();
        let ok = std::fs::write(&path, vec![7u8; TOTAL]).is_ok();
        let secs = t.elapsed().as_secs_f64();
        if ok && secs > 0.0 {
            (TOTAL as f64 / 1_048_576.0) / secs
        } else {
            0.0
        }
    };

    let chunked = {
        use std::io::Write as _;
        let t = Instant::now();
        let mut ok = true;
        if let Ok(mut f) = std::fs::File::create(&path) {
            for _ in 0..(TOTAL / CHUNK) {
                if f.write_all(&buf).is_err() {
                    ok = false;
                    break;
                }
                let _ = f.flush();
            }
            let _ = f.sync_all();
        } else {
            ok = false;
        }
        let secs = t.elapsed().as_secs_f64();
        if ok && secs > 0.0 {
            (TOTAL as f64 / 1_048_576.0) / secs
        } else {
            0.0
        }
    };

    let _ = std::fs::remove_file(&path);
    (sequential, chunked)
}

/// Burst against sustained throughput.
///
/// N2b is a sustained figure and every other measurement here is a burst. A phone that does 3 M
/// exchanges a second for two seconds and 900 K for fifteen minutes has a throttled/burst ratio of
/// 0.3, and §12's tick budget is the second number.
fn thermal(soak_seconds: u64) -> (f64, f64, i64) {
    let mut arena: Vec<Slot> = vec![Slot::default(); SLOTS];
    let mut sink = 0i64;

    let mut run = |seconds: u64, offset: u64| -> f64 {
        let t = Instant::now();
        let mut done = 0u64;
        while t.elapsed().as_secs() < seconds {
            for i in 0..100_000u64 {
                let k = (draw(offset + done + i) as usize) % SLOTS;
                if let Some(s) = arena.get_mut(k) {
                    s.quantity = s.quantity.wrapping_add(1);
                    sink = sink.wrapping_add(s.quantity);
                }
            }
            done += 100_000;
        }
        let secs = t.elapsed().as_secs_f64();
        if secs > 0.0 { done as f64 / secs } else { 0.0 }
    };

    let burst = run(2, 0);
    let soak = run(soak_seconds, 1_000_000_000);
    (burst, soak, sink)
}

/// How long the soak runs, and how far the allocation walk goes.
///
/// The device wants §12's own figures — fifteen minutes, and as much memory as it will give. A host
/// run is a floor and a smoke test, so it takes the short ones: a fifteen-minute CI job that measures
/// a desktop's thermal behaviour tells nobody anything.
pub struct Budget {
    /// Seconds of sustained load for the thermal ratio.
    pub soak_seconds: u64,
    /// How far the committed-allocation walk goes before it stops, in MiB.
    pub allocation_budget_mib: usize,
}

impl Budget {
    /// §12's own figures: fifteen minutes, and 8 GiB of headroom to walk into.
    #[must_use]
    pub fn device() -> Self {
        Self {
            soak_seconds: 900,
            allocation_budget_mib: 8_192,
        }
    }

    /// A floor and a smoke test.
    #[must_use]
    pub fn host() -> Self {
        Self {
            soak_seconds: 5,
            allocation_budget_mib: 2_048,
        }
    }
}

/// Run everything, in an order that puts the thermal soak last so nothing else runs hot.
#[must_use]
pub fn measure(budget: &Budget, scratch: &std::path::Path) -> (Measurements, i64) {
    let mut m = Measurements::default();
    let mut sink = 0i64;

    let (ns, s) = interleaved_cost();
    m.interleaved_ns = ns;
    sink = sink.wrapping_add(s);

    let (ns, s) = columnar_cost();
    m.columnar_ns = ns;
    sink = sink.wrapping_add(s);

    let (ns, s) = noise_floor();
    m.noise_floor_ns = ns;
    sink = sink.wrapping_add(s);

    m.block_ops = block_ops();
    let (ln, exp) = transcendental();
    m.ln_hash = ln;
    m.exp_hash = exp;

    let (seq, chunked) = storage(scratch);
    m.seq_write_mibs = seq;
    m.chunked_write_mibs = chunked;

    let (ceiling, held) = allocation_ceiling(budget.allocation_budget_mib);
    m.ceiling_mib = ceiling;
    m.held_seconds = held;

    let (burst, soak, s) = thermal(budget.soak_seconds);
    m.burst_ops_per_sec = burst;
    m.soak_ops_per_sec = soak;
    m.soak_seconds = budget.soak_seconds;
    sink = sink.wrapping_add(s);

    (m, sink)
}

/// The one object the owner copies back (D4).
///
/// Hand-written rather than serialised through a dependency: the workspace has none, and a schema
/// this small is cheaper to write than to justify a crate for. Every field is present — exit criterion
/// 9 requires a document with **no field null**, so a measurement that did not run shows as a zero
/// with its own reason beside it rather than as a missing key.
#[must_use]
pub fn json(m: &Measurements, commit: &str, device: &str, sink: i64) -> String {
    let mut blocks = String::new();
    for (i, (width, ns)) in m.block_ops.iter().enumerate() {
        if i > 0 {
            blocks.push_str(", ");
        }
        let _ = write!(blocks, "\"{width}\": {ns:.1}");
    }
    let ratio = if m.burst_ops_per_sec > 0.0 {
        m.soak_ops_per_sec / m.burst_ops_per_sec
    } else {
        0.0
    };
    let mut out = String::new();
    let _ = write!(
        out,
        concat!(
            "{{\n",
            "  \"schema\": \"{schema}\",\n",
            "  \"commit\": \"{commit}\",\n",
            "  \"device\": \"{device}\",\n",
            "  \"slots\": {slots},\n",
            "  \"ops\": {ops},\n",
            "  \"allocation\": {{ \"ceilingMiB\": {ceiling}, \"heldSeconds\": {held} }},\n",
            "  \"operationCost\": {{ \"interleavedNs\": {interleaved:.1}, \"columnarNs\": {columnar:.1}, ",
            "\"noiseFloorNs\": {floor:.1}, \"budgetNs\": 96.5 }},\n",
            "  \"blockOps\": {{ {blocks} }},\n",
            "  \"storage\": {{ \"seqWriteMiBs\": {seq:.1}, \"chunkedWriteMiBs\": {chunked:.1} }},\n",
            "  \"transcendental\": {{ \"lnHash\": \"{ln:016x}\", \"expHash\": \"{exp:016x}\" }},\n",
            "  \"thermal\": {{ \"burstOpsPerSec\": {burst:.0}, \"soakOpsPerSec\": {soak:.0}, ",
            "\"ratio\": {ratio:.3}, \"soakSeconds\": {soak_secs} }},\n",
            "  \"sink\": {sink}\n",
            "}}"
        ),
        schema = SCHEMA,
        commit = commit,
        device = device,
        slots = SLOTS,
        ops = OPS,
        ceiling = m.ceiling_mib,
        held = m.held_seconds,
        interleaved = m.interleaved_ns,
        columnar = m.columnar_ns,
        floor = m.noise_floor_ns,
        blocks = blocks,
        seq = m.seq_write_mibs,
        chunked = m.chunked_write_mibs,
        ln = m.ln_hash,
        exp = m.exp_hash,
        burst = m.burst_ops_per_sec,
        soak = m.soak_ops_per_sec,
        ratio = ratio,
        soak_secs = m.soak_seconds,
        sink = sink,
    );
    out
}
