---
id: ADR-0013
title: The sixteen definitional identities, enumerated
status: accepted
date: 2026-09-06
register-entry: 27
claim-impact: A3
guard: check-registry rule 4 — a `structural` entry naming anything else fails the build
supersedes: none
cost: a structural value that is genuinely definitional but unlisted must have an ADR before it can be entered
alternatives-rejected: leaving the list implicit; allowing any string as an identity
re-derivations: none
---

## Decision

§16.1 rule 4 requires a `structural` entry to name one of **sixteen** definitional identities, and the
specification never lists them. They are enumerated in `registry/identities.txt`, and a seventeenth is
an ADR. A `structural` entry naming anything else fails the build.

## Why

An unenumerated closed set is not closed. "Sixteen" was a claim nothing could check, and the rule it
supports — *a structural entry is definitional arithmetic, not a choice* — is exactly the rule that
decays first, because every value looks definitional to the person entering it.

**What qualifies.** Definitional arithmetic (the numéraire, the asymmetry quadruple, zero as the fixed
point of a growth rate, a critical value of a distribution) or a closed decision about the shape of the
world (four regions, seven sectors, sixty-four shards, twenty-one positions, thirteen questions). What
does not: anything measured, and anything that is a claim about how the economy behaves.

**The list is deliberately not padded.** Several capacities currently name an identity that is only
loosely theirs, because the capacity namespace (ADR-0014) exists to carry engineering sizes and the
identity field is a poor fit for them. That is recorded here rather than hidden: **capacities should
carry their derivation, not an identity**, and the field is owed a discriminant when M1 writes the
arena's real capacities.
