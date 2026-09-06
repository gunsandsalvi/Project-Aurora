---
id: ADR-0012
title: The arena is thread-shareable from the first line, and runs single-threaded until M11
status: accepted
date: 2026-09-06
register-entry: 25
registers: PROJECT_AURORA.md §11
claim-impact: A1
guard: check-lints rule 4 — no `Rc`, `RefCell` or `Cell` may be named in a layer crate, tokenised so that prose naming one is not a finding
supersedes: none
cost: an interior-mutability shortcut is unavailable for ten milestones before anything needs it to be
alternatives-rejected: single-threaded types now and a conversion at M11 (the conversion is the whole cost, deferred and compounded); threads from M1 (a determinism surface with no measurement asking for it)
re-derivations: none
---

## Decision

Every structure the engine holds is **`Send + Sync`-shaped from the first line**, and the engine
**runs single-threaded until M11**. `Rc`, `RefCell` and `Cell` may not be named in a layer crate.

## Why

**These are two decisions and they are usually confused.** Whether a structure *can* be shared across
threads is a property of its types; whether it *is* is a property of the runtime. Deciding the first
late is expensive and deciding the second early is a determinism surface nobody has asked for.

**The cost of deciding it late is not distributed, it is concentrated.** `Rc` and `RefCell` are cheap
to reach for while the engine is single-threaded, and cheap in exactly the way that matters: nothing
goes wrong, no test fails, and the shortcut is invisible until the day the arena is shared — at which
point every one of them is a compile error in a codebase that has grown around it. §11 makes the arena
the whole of the engine's mutable state, so "convert later" means converting everything at once,
under the deadline that made it urgent.

**And the second decision has no evidence yet.** §12's targets are single-threaded numbers; the probe
measures a single thread; the acceleration seam is M11's, *if the measurements say it is needed*. A
model that ran threads from M1 would be paying a determinism cost — §11's `draw` is a pure function of
(seed, stream, index) precisely so that it does not depend on scheduling — against no measurement.

**Why these three types and not a trait bound.** `Send`/`Sync` cannot be asserted on a codebase with no
types in it, and M0 has none. A name check can be run today, on an empty tree, and stays exactly as
valid when the tree fills. It is a weaker rule than the bound and it arrives ten milestones earlier;
the bound lands with the arena in M1 and this rule stays as the thing that catches a reach for the
shortcut before the bound exists to refuse it.

## What it costs

An interior-mutability shortcut is unavailable for ten milestones before anything needs it to be. In
practice that is a design constraint on `world` and `ledger`: state is mutated through the arena's own
handles rather than through a shared cell, which is what §11 already requires and this makes
mechanical.

## Alternatives rejected

- **Single-threaded types now, convert at M11.** The conversion *is* the cost, and deferring it
  compounds it against a codebase that has grown around the shortcut.
- **Threads from M1.** A determinism surface with no measurement asking for it, ten milestones before
  the seam that would use it.
- **A `Send + Sync` bound instead of a name check.** Correct, and unavailable: there are no types yet.
  It lands in M1 and this rule keeps working underneath it.

## The guard

`check-lints`, rule 4. `Rc`, `RefCell` and `Cell` are refused in any crate under `crates/`, tokenised
with `proc-macro2`, so a comment or a doc-string naming one is not a finding — the same discipline the
`unsafe_code` rule needed after its first draft substring-matched itself. Two fixtures: a `RefCell` in
a layer crate is caught, and a comment naming it is not.
