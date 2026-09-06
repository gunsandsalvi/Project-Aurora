---
id: ADR-0008
title: The journal row is 48 bytes, and the realised rate is not one of its fields
status: accepted
date: 2026-09-06
register-entry: 8
registers: PROJECT_AURORA.md §6.4, PROJECT_AURORA.md §6.6
claim-impact: A2
guard: aurora-tools sizing — the row width is summed from the field list and the ring from the width, so a field added without a decision changes a published number
supersedes: §6.4's "the cleared rate and the realised rate"
cost: a reader wanting the realised rate divides two i64 that are already in the row
alternatives-rejected: storing both rates (56 B, and a second copy of a derived value); 32-bit identifiers (15.7M ever issued exceeds 24 bits and leaves no headroom at 32 for the owed mean lives); dropping the tick (the row stops being self-describing when saved)
re-derivations: §6.6's 345.6 MB ring stands; N4's journal line is unchanged
---

## Decision

The journal row is **48 bytes**, in eleven fields plus three of alignment padding:

| field | type | B |
|---|---|---|
| `quantityGiven` | `i64` | 8 |
| `quantityReceived` | `i64` | 8 |
| `clearedRate` | `i64`, fixed point at `S` | 8 |
| `from` | `EntityId` (`u32`) | 4 |
| `to` | `EntityId` (`u32`) | 4 |
| `assetGiven` | `InstrumentId` (`u32`) | 4 |
| `assetReceived` | `InstrumentId` (`u32`) | 4 |
| `tick` | `u16` | 2 |
| `op` | `u8` | 1 |
| `reason` | `u8` | 1 |
| `actor` | `u8` | 1 |
| *(padding)* | | 3 |
| **total** | | **48** |

**The realised rate is not a field.** §6.4's sentence — that an `exchange` row carries *"both assets,
both quantities, the cleared rate and the realised rate"* — is superseded.

## Why

§6.6 printed 48 bytes with no field list, and §2's *Owed* register asked for either the packing or the
width the packing needs. Written out, the row §6.4 describes does not fit: eleven fields plus a second
rate come to 53 B and pad to 56, and the ring goes from 345.6 MB to 403.2 MB.

**The field that does not fit is the one that should never have been stored.** The realised rate — what
this party actually got, as distinct from the price the line cleared at — is `quantityReceived /
quantityGiven`. Exactly, by definition. The pair *is* the realised rate, at the full precision of the
two conserved quantities, while a stored copy is that same value put through §6.3's rounding a second
time. Two copies of one value is the failure §16.1's registry exists to prevent, and nothing was
applying that principle to the journal.

**The cleared rate is not the same kind of thing and is stored.** It is the venue's price for the line,
and when a line rations it is *not* a function of this row's own quantities: the row records what this
party received, the cleared rate records what the line cleared at, and the difference between them is
the rationing. Deriving one from the other would lose it.

The field widths are forced rather than chosen:

- **Identifiers are 32-bit** because the census (`aurora-tools sizing`) counts 15,732,835 ever issued
  over a 1,560-tick run. That exceeds 24 bits, and several of its rows are mean lives still marked
  owed — so the headroom above 15.7 M is not spare, it is the error bar.
- **A rate is 64-bit** because it spans several orders of magnitude either side of one and is held at
  the numéraire's resolution. 32 bits cannot carry that.
- **The tick is 16-bit** and it is kept even though the ring holds two ticks: a row lifted out of the
  ring — into a save, a digest, a closure report — must say which tick it belongs to without its
  position in a buffer being the answer.

Three bytes of padding are left rather than spent. A field packed into them would be a field the
alignment paid for, which is the cheapest possible place to put the next one, and naming the space is
how that stays a decision rather than a discovery.

## What it costs

A reader wanting the realised rate divides two integers that are already in the row. §9.3's prior-close
reads and the closure panel both do this, so it is a division per row read rather than per row written —
and rows are written 3,119,665 times a tick and read far less often.

## Alternatives rejected

- **Store both rates.** 56 B, a 403.2 MB ring, 57.6 MB more, in exchange for a rounded copy of a value
  the row already holds exactly.
- **32-bit identifiers with a 24-bit field.** 15.7 M exceeds 24 bits before the owed mean lives are
  derived, and §5.2's rule that an identifier is never reused means the count only grows.
- **Drop the tick.** It saves nothing — the padding absorbs it — and it makes a saved row depend on its
  buffer position for meaning.

## The guard

`aurora-tools sizing` sums the field list to the width and the width to the ring, and prints both. A
field added or widened without a decision moves a published number, which is the same guard the memory
derivation already runs under. The width is not typed anywhere; it is produced.
