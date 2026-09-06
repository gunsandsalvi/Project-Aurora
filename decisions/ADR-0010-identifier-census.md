---
id: ADR-0010
title: The identifier census is counted bottom up per space, and it is 9.4 million, not 971,000
status: accepted
date: 2026-09-06
register-entry: 7
registers: PROJECT_AURORA.md §3.4, PROJECT_AURORA.md §5.2
claim-impact: A4
guard: aurora-tools sizing — the census is summed per identity space on every run and the directory row is computed from that sum, so a space added or a mean life changed moves a published number
supersedes: §3.4's ≈ 971,000; §5.2's 47.5 MiB directory and its implied ≈ 12,450,000
cost: five of the census's rows are mean lives standing in for mechanisms that do not exist, so the figure is an order of magnitude rather than a number
alternatives-rejected: keeping ≈ 971,000 (below the opening entity count plus one generation of employment contracts); keeping §5.2's implied 12.45 M (the right order, but no derivation); deferring the census to M1 (it sizes the directory, the digest walk and the save, all of which M1 builds)
re-derivations: the directory is 35.8 MiB, not 47.5; §6.6's journal row takes 32-bit identifiers from it (ADR-0008); ADR-0009 removed the schedules space and 6.36 M identifiers with it
---

## Decision

**The identifier census is counted bottom up, one row per identity space**, in `aurora-tools sizing`.
It stands at **9,372,835 ever issued** over a 1,560-tick run, and the directory at 4 bytes each is
**35.8 MiB**.

§3.4's ≈ 971,000 and §5.2's implied ≈ 12,450,000 are both superseded.

## Why

**971,000 is below the opening entity count plus one generation of employment contracts.** There are
550,622 entities at tick 0, and 350,000 live employment contracts at a six-year mean term issue about
1.75 million identifiers over thirty years — without a single loan, tenancy or bond. The figure could
not have been produced by counting anything.

**§5.2's number was the right order and had no derivation either.** It was inferred backwards from a
47.5 MiB directory at 4 bytes an identifier, which is a size dividing into a count rather than a count
producing a size. The two published figures differed by a factor of thirteen and neither was computed.

**A census has to be per space, because the spaces do not behave alike.** Entities are issued once and
never replaced within the run; employment contracts turn over every six years; liens turn over on the
margin cycle. A single "identifiers per tick" figure cannot represent that, which is probably how the
original number was reached.

**And it is not a static count — it is a flow.** §5.2 says an identifier is never reused, so the
directory holds everything ever issued, not everything live. That distinction is the whole of the
factor of ten: 1,060,000 live obligation-bearing instruments against 9.4 million identifiers.

## What it costs

**Five of the census's twelve rows are mean lives assumed here**: six years for an employment
contract, four for a tenancy, ten for household credit, five for a corporate facility, one margin
cycle for a lien. Each is an assumption standing in for a mechanism that does not exist yet, and each
is replaced by the milestone that builds the instrument. **The figure is an order of magnitude rather
than a number**, and every decision that reads it is taken with that in mind — ADR-0008's 32-bit
identifier is chosen because 9.4 million against a 4.29 billion ceiling leaves the error bar room.

## Alternatives rejected

- **Keep ≈ 971,000.** It is below a lower bound that one instrument type establishes on its own.
- **Keep §5.2's implied 12.45 M.** The right order, and still a size divided by a width.
- **Defer to M1.** The census sizes the directory, the digest's identifier-order walk and the save —
  all of which M1 builds, so deferring it means M1 building three things against a number known to be
  wrong.

## The guard

`aurora-tools sizing` sums the census on every run and the memory derivation's directory row is
computed from that sum rather than typed beside it. A space added, a space removed, or a mean life
changed moves a published number in the same output. ADR-0009 is the demonstration: removing the
schedules space took 6,360,000 identifiers and 24.2 MiB out of two tables at once, and neither figure
had to be edited.
