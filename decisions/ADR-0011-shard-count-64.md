---
id: ADR-0011
title: The shard count is always 64 and is a saved run parameter; the thread count carries no semantics
status: accepted
date: 2026-09-06
register-entry: 25
registers: registry/entries.toml, PROJECT_AURORA.md §11
claim-impact: A1
guard: check-registry rule 4 — `world.shard_count` is `structural` and names the `ShardCount` identity, so it cannot be redeclared as an assumption or made to depend on a device
supersedes: none
cost: 64 shards on a device with 4 cores, and on a device with 16; the number is not a tuning knob and cannot become one
alternatives-rejected: shards = threads (makes the digest depend on the hardware, which breaks N1); shards as a device measurement (same, one level of indirection away); a power of two chosen at load (a saved run cannot then be resumed on another device)
re-derivations: none
---

## Decision

**The shard count is 64, always, in every run and on every device.** It is a saved run parameter: it
goes in the manifest, it is part of what a resumed run resumes, and a run saved at 64 shards cannot be
loaded at any other number.

**The thread count carries no semantic content.** Nothing the model computes may depend on it, and no
digested value may be reached by a path that reads it.

## Why

**N1 says a run reproduces bit for bit.** A sharded computation reproduces only if the shard boundaries
are the same, because the boundaries decide the association order of every per-shard accumulation, and
floating-point association is not commutative — and even in integers, they decide which rows a
per-shard sort sees. So the shard count is part of the *result*, not part of the schedule.

**That is the whole argument, and it forbids the obvious design.** Setting shards from the core count
makes the digest a function of the hardware: the same seed on a four-core phone and a sixteen-core
desktop produces two different runs, both correct, neither reproducible from the other. Measuring the
device and choosing a shard count is the same defect one level of indirection away.

**Threads are a schedule and the schedule is free.** Sixty-four shards can be executed on one thread,
four, or sixty-four; the association order within a shard is fixed by the shard, so the answer does not
move. This is why the two numbers must be separated: the shard count is a modelled quantity and the
thread count is a resource decision, and conflating them makes a resource decision into a modelling
one.

**Why 64 and not another number.** It is not derived and does not pretend to be — it is a closed
decision about the shape of the world, which is what `structural` means and why `ShardCount` is one of
the definitional identities (ADR-0013). What it has to satisfy is that it divides the work finely
enough for any thread count the delivery target might have, and coarsely enough that per-shard
overhead is not the dominant cost. 64 is comfortably above any phone's core count and comfortably
below the row counts in §3.4's tables.

## What it costs

64 shards on a four-core device is 16 shards per core, and 64 on a sixteen-core desktop is 4. Neither
is optimal for its hardware, and that is accepted: the alternative is a number that is optimal and
non-reproducible. A run saved at 64 cannot be loaded at another value, which is a constraint on the
save format rather than a cost at runtime.

## Alternatives rejected

- **Shards = threads.** Makes the digest a function of the hardware and breaks N1.
- **Shards from a device measurement.** The same defect, one level of indirection away.
- **A power of two chosen at load.** A saved run could not then be resumed on another device, which is
  most of what saving is for.

## The guard

`check-registry` rule 4: `world.shard_count` is a `structural` entry naming the `ShardCount`
definitional identity. A `structural` entry cannot carry a bracket, cannot be region-scoped, and
cannot be redeclared `assumed` — so the number cannot quietly become a tuning knob, and it cannot be
made to depend on anything the device reports.

The second half — that no digested path reads a thread count — needs a `runtime` to scan and lands
with it in M11. It is named here so the rule arrives with the threads rather than after them.
