---
id: ADR-0017
title: The retirement queue is drained every tick, not at position 21, and its capacity is max-per-tick times the interval
status: accepted
date: 2026-09-06
register-entry: 17
registers: PROJECT_AURORA.md §5.5, PROJECT_AURORA.md §9.4
claim-impact: A4
guard: aurora-tools workload — the queue's capacity is priced against the census's own retirement flow on every run, so a change to either the flow or the interval moves a published number and the 4.5x overflow is what the rule found
supersedes: §5.5's "which `lifecycle` drains at position 21"
cost: a drain at the end of every tick rather than one in fifty-two, and the drain is not in the committed order — so its cost is not in §3.4's operation count and has to be named separately
alternatives-rejected: a queue five times larger (holds the flow and hides the burst); draining at position 21 every tick (position 21 is obligation compaction, on a 52-tick cadence for a reason); a wrapping queue (§5.5 says overflow is a defect, and a lost retirement is a leaked slot the digest cannot see)
re-derivations: §5.5's 65,536 stands and is now derived rather than stated; position 21 keeps Appendix B's scope, obligation compaction only
---

## Decision

**The retirement queue is drained at the end of every tick**, by `lifecycle`, and **not at position
21**. Its capacity is `max retirements per tick × the interval between drains`; at an interval of one
tick, §5.5's 65,536 entries is right and carries 11.6 ticks of burst headroom.

Position 21 keeps the scope Appendix B gives it: **obligation compaction only.**

## Why

**§5.5 as written halts the run at about tick 12.** It pushes a retired identifier onto a fixed
65,536-entry queue, says `lifecycle` drains it *at position 21*, and says **overflow is a defect**.
§9.4 runs position 21 every fifty-second tick. Priced against §5.2's own census, steady-state
retirement is 5,653 identifiers a tick — 3,666 instruments and 1,987 liens — so fifty-two ticks is
**293,956 entries against a capacity of 65,536**. Four and a half times over, and the queue fills in
11.6 ticks.

**The capacity is right and the interval is wrong.** That is worth stating in that order, because the
obvious repair is to make the queue five times bigger and it is the wrong one: a queue sized to hold
fifty-two ticks of steady-state flow has *no* headroom for a burst, and bursts are exactly what
retirement has — a resolution wave at position 18, a maturity cluster at 19. Sizing for the flow and
sizing for the tail are different problems, and the interval is what lets one capacity do both.

**Draining every tick costs nothing that has to be scheduled.** §5.5 says relocation *is not an
operation*: it changes no quantity and appends no journal row. A drain is the same thing, so it needs
no slot in the committed order — and §9.4 already says, in its own words, **retirement is not a
position**. The drain belongs at the end of the tick with the rest of the machinery that is not
modelled behaviour.

**This also removes an inconsistency inside the specification.** §5.5 says `lifecycle` drains at
position 21; Appendix B says position 21's scope is *obligation compaction only* and that retirement
is not a position. Both cannot be true. Appendix B is the one that survives.

## What it costs

A drain at the end of every tick rather than one in fifty-two. The work is the same total — every
retired identifier is processed once either way — but it is spread rather than batched, which is worse
for cache locality and better for the tail: a 293,956-entry drain every fifty-two ticks is a latency
spike in one tick out of fifty-two, and N2b is a sustained figure.

**And the drain is not in §3.4's operation count**, because it appends no journal row. Its cost has to
be named separately, in the same way clearing's sort cost does (W7.4). Two kinds of work that a
per-operation budget does not see.

## Alternatives rejected

- **A queue five times larger, drained at 21.** Holds the steady-state flow and has no headroom left
  for the burst, which is the thing a fixed queue is for.
- **Position 21 every tick.** Position 21 is obligation compaction, which is on a 52-tick cadence
  because releasing closed schedules is exactly the work that benefits from batching. Moving it to
  every tick to serve the queue would trade a cheap fix for an expensive one.
- **A wrapping queue.** §5.5 says overflow is a defect, and it is right: a lost retirement is a slot
  that is never recovered and a row the retirement accumulator never folds in, so the digest cannot
  see it. It would be a leak that reproduces.

## The guard

`aurora-tools workload` prices the queue against the census's own retirement flow on every run and
prints both the interval and the multiple. It is what found the 4.5× overflow. A change to the flow —
a new instrument type, a different mean life — or a change to the interval moves a published number
in the same output.
