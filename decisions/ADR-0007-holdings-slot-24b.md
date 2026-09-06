---
id: ADR-0007
title: The holdings slot is 24 bytes with its field list published, and encumbrance is derived rather than stored
status: accepted
date: 2026-09-06
register-entry: 22a
registers: PROJECT_AURORA.md §3.4
claim-impact: A1
guard: aurora-tools sizing — the slot width is summed from the field list and the holdings table from the width, so a column added or widened moves a published number; and there is no encumbrance column to add one to
supersedes: the previous edition's 20-byte slot
cost: 172.3 MB of holdings table, and an encumbrance read costs an index lookup rather than a field read
alternatives-rejected: 20 B (cannot hold what §6.11 requires — quantity, integral and asset exhaust it and the tick column has nowhere to live); a per-slot encumbrance flag (57.4 MB, aligned, to serve 100,000 liens against 7,177,280 slots); dropping the integral and walking the journal (that walk is what the integral exists to remove)
re-derivations: none — §3.4's 7,177,280 slots and §12.1's holdings line are unchanged
---

## Decision

The holdings slot is **24 bytes**, in four fields plus two of alignment padding:

| field | type | B |
|---|---|---|
| `quantity` | `i64` | 8 |
| `integral` | `i64` | 8 |
| `asset` | `InstrumentId` (`i32`) | 4 |
| `integralUpdatedAtTick` | `u16` | 2 |
| *(padding)* | | 2 |
| **total** | | **24** |

**Encumbrance is not on the slot.** It is derived from root lien rows through a per-`(holder, asset)`
index.

## Why

**The previous edition's 20 bytes could not hold what another section of the same document required.**
§6.11 needs an asset, a quantity, a balance-tick integral *and* the tick that integral was last
brought forward to. The first three are 20 bytes on their own, so the tick column had nowhere to live.
A width printed without a schema is a number that cannot be checked, and this one had already been
contradicted.

**The tick column is not optional and its absence is not cheap.** §6.11's integral exists so that
accrual is a *read* — value × elapsed ticks — rather than a walk over the journal. Without a column
saying when the integral was last brought forward, there is no elapsed term, and accrual has to
reconstruct it from journal rows. That is exactly the walk the integral was introduced to remove, and
it would run at position 3 over every holding.

**Encumbrance is where the interesting trade is.** A flag on the slot would be one bit, and one bit is
one byte, and alignment makes it eight — **57.4 MB** across 7,177,280 slots, to serve a lien
population of about 100,000. That is 574 bytes of table per lien. Liens are institutional and rare;
the holdings table is neither. So the flag is derived instead: root lien rows carry the encumbrance and
a per-`(holder, asset)` index answers "is this holding pledged", and the index is paid for only where
liens exist.

**Why `i32` for the asset and not `u32`.** The slot is read by four fields at once and the asset column
is the one that can carry a sentinel meaning *empty*, which a signed type expresses without spending a
value from the identifier space. That is a schema convenience rather than a modelling claim, and it
costs nothing: §5.2's census is 9.4 million identifiers against 2.1 billion positive `i32` values.

## What it costs

**172.3 MB of holdings table**, which is the largest single line in the memory derivation after the
journal ring. And an encumbrance read costs an index lookup rather than a field read — paid at
position 16, where collateral is tested, and nowhere else.

## Alternatives rejected

- **20 B.** Cannot hold what §6.11 requires. This is not a preference; it is arithmetic.
- **A per-slot encumbrance flag.** 57.4 MB aligned, for 100,000 liens.
- **Drop the integral and walk the journal at accrual.** The walk is what the integral exists to
  remove, and §6.6's ring holds only two ticks — so the walk could not reach far enough anyway.
- **32 B with room to grow.** 57.4 MB of headroom bought before anything asks for it, on a table whose
  width is already the second-largest line in N4.

## The guard

`aurora-tools sizing` sums the field list to the width and the width to the table, and prints both
beside the encumbrance-flag counterfactual. A column added or widened moves a published number. And
the guard against the flag returning is that there is no column to add it to: the alternative is
priced in the same output, every run.
