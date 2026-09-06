---
id: ADR-0014
title: Two registry namespaces — model and capacity
status: accepted
date: 2026-09-06
register-entry: 13
claim-impact: A3
guard: the census counts them separately; the read rule (a capacity entry is unreadable by any agent, valuation or economic system) lands with the manifests
supersedes: none
cost: an entry must be classified when it is written, and the boundary is occasionally arguable
alternatives-rejected: one namespace and one count; keeping capacities out of the registry entirely
re-derivations: §16.1's census
---

## Decision

Registry entries are `model` or `capacity`.

**`model`** entries are claims about the world. Their count — and the `assumed` count within it — is the
figure M3 pushes down and the surface publishes.

**`capacity`** entries are engineering sizes: arena widths, ring lengths, queue depths, tier capacities.
They carry the arithmetic that produced them, exhaustion raises rather than reallocating, and **no
agent, valuation or economic system may read one.**

## Why

D3 removed the cap and made the count the mechanism: *the assumed count is the honest measure of how
much of the world was chosen rather than produced.* That measure is only honest if it counts the right
things. Forty entries of pure engineering sizing — how many slots a holdings block has, how many rows the
journal ring holds — say nothing about how much of the *economy* was chosen, and counting them against a
figure whose stated purpose is measuring exactly that was always going to mislead.

Keeping capacities out of the registry entirely was the other option and it is worse: §12.1 requires
every capacity to be a `structural` entry carrying its arithmetic, precisely so that a capacity cannot be
raised in a commit about something else (R16). Two namespaces keep that guard and fix the count.

**The read rule is the half that is not yet mechanical.** Nothing today can read anything, so the check
that a `capacity` entry never reaches an agent lands with the manifests that would declare such a read.
Recorded as owed rather than claimed.
