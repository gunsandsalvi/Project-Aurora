---
id: ADR-0006
title: A conserved quantity is i64 and overflow panics, in every profile
status: accepted
date: 2026-09-06
register-entry: 2
registers: Cargo.toml, PROJECT_AURORA.md §5.1
claim-impact: A1
guard: check-lints — a release profile without `overflow-checks = true` fails the build, and the rule found the profile missing it on its first run
supersedes: none — Appendix A #2 already said overflow panics; this makes it true
cost: a checked add on every conserved write; and a run that would have wrapped now halts, which is the point
alternatives-rejected: i128 (doubles the conserved column and the journal row for headroom the numéraire already provides); checked_add returning an Option (a caller who can handle a conservation failure is a caller who can hide one); saturating arithmetic (silently breaks A1 and is undetectable afterwards)
re-derivations: none — the widths in §5.1, §6.6 and §3.4 are unchanged
---

## Decision

A conserved quantity is **`i64`**, and **arithmetic on it panics on overflow in every profile**,
including release. `Cargo.toml`'s release profile carries `overflow-checks = true` and `check-lints`
refuses a profile that does not.

Neither `wrapping_*`, `saturating_*` nor `overflowing_*` may appear on a conserved column. `checked_*`
may not either, and that is the less obvious half: see below.

## Why

**Appendix A #2 has said "overflow panics" since the first edition, and it was not true.** `rustc`
turns overflow checks *off* in release by default, so the panic existed in `cargo test` and in nothing
that would ever ship. A claim that holds only in the build nobody runs is worse than no claim: it is a
guarantee the design leans on — A1 says conservation is structural — resting on a default nobody
looked at. `check-lints` gained the rule and it fired on the first run, against this repository.

**Why `i64` and not something wider.** §5.3 fixes the numéraire at `S = 2 × 10¹¹` minor units. `i64`
holds ±9.22 × 10¹⁸, which is 4.6 × 10⁷ times the entire world money stock. A quantity that overflows
`i64` is not a large number, it is a defect — a runaway loop, a sign error, a missing counterparty —
and the correct response to it is a halt. `i128` would double the conserved column and the journal
row to buy headroom for a value the model cannot produce.

**Why a panic and not a `checked_add` that returns.** This is the part worth stating. A `checked_*`
call site has to decide what to do with `None`, and the honest answer is *there is nothing to do*: A1
says quantities are conserved, so a sum that cannot be represented means the ledger is already wrong
and every subsequent operation compounds it. A caller equipped to handle a conservation failure is a
caller equipped to *continue past one*, and the failure mode that produces is a run that finishes and
reports numbers. **A halt is legible and a wrong answer is not.**

Saturating arithmetic is the same argument at its worst: it breaks conservation *and* leaves no trace,
because the sum is a plausible number.

## What it costs

A checked add on every conserved write. `overflow-checks` costs a compare and a branch per arithmetic
operation, and the branch is perfectly predicted because it is never taken. §12.2's budget is 96.5 ns
against a measured 133.8 ns of memory traffic, and the check is not where that difference lives — it
is in the two cache misses per exchange. The probe measures the arena, and this decision is not
visible in it.

And a run that would have wrapped now halts. That is the cost being bought.

## Alternatives rejected

- **`i128`.** Doubles the conserved column (24 B slot → 32 B) and the journal row, for headroom above
  a bound the numéraire already puts 4.6 × 10⁷ below.
- **`checked_*` returning an `Option`.** Gives every call site the option of continuing past a
  conservation failure, which is the outcome A1 exists to make impossible.
- **`saturating_*`.** Breaks A1 and leaves a plausible number behind.
- **Leaving the release profile as it was.** That is not an alternative, it is the defect.

## The guard

`check-lints`, rule 3: `[profile.release]` must carry `overflow-checks = true`. It fired on its first
run against this repository, which is the only evidence that a check works.

The second half — no `wrapping_*` or `saturating_*` on a conserved column — lands with the conserved
column itself in M1, because there is nothing to scan until a conserved column exists. It is named
here so that the rule arrives with the type rather than after it.
