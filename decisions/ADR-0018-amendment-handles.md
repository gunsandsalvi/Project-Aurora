---
id: ADR-0018
title: An amendment handle is minted per (mechanism, type), not per mechanism
status: accepted
date: 2026-09-06
register-entry: 12
claim-impact: A2
guard: check-instruments — every mechanism declares the types it applies to, and a mechanism naming an undeclared type fails; the mint takes the pair
supersedes: none
cost: one extra column in the amendment table, and a handle set that grows with the type count
alternatives-rejected: one handle per mechanism; one general amendment handle, which §7.4 already forbids
re-derivations: none
---

## Decision

An amendment handle is minted for a **(mechanism, type)** pair. `instruments/amendments.toml` declares
the types each mechanism may amend, and a mechanism reaching a type outside its list does not mint.

## Why

**The matrix is sparse, and filling it is what showed that.** Of eight mechanisms against eight opening
types, most cells are empty: `prepayment` and `payment-holiday` apply to one type, `employment-termination`
to one, and `claim-crystallisation` to none yet.

A per-mechanism handle would therefore be far wider than any mechanism needs. Concretely: **credit owns
`default-truncation`, and a per-mechanism handle would let credit truncate an employment contract** —
which labour owns, and which §9.6.2 says is amended under `Termination` with a reason code distinguishing
a quit from a dismissal. §7.4's rule that *no system holds a general amendment handle* is aimed at exactly
that, and a handle wide enough to reach every type is a general handle with extra steps.

**The cost is one column, decided now.** §7.4's own note is that this is *"one extra column if decided
early and a rewrite of `amend` if decided late"*, which is why it was worth filling the table by hand in
M0 rather than discovering the shape in M3.

*Noted, not resolved here:* `claim-crystallisation` applies to no opening type at all, because insurance
has no instrument yet. §7.4's eight mechanisms anticipate instruments the opening set does not contain,
which is fine — but it means the mechanism count and the type count are independent, and a ninth
mechanism is an ADR while a ninth type is not.
