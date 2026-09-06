---
id: ADR-0004
title: One unsafe seam, declared before it is written
status: accepted
date: 2026-09-06
register-entry: 16
claim-impact: A1
guard: tools check-lints — the only `allow` of `unsafe_code` in the tree is the declared seam, and it is capped at 60 non-blank lines
supersedes: none
cost: a typed column view that needs unsafe must be routed through one module rather than written where it is needed
alternatives-rejected: unsafe wherever the arena needs it; no unsafe at all, decided before the arena exists
re-derivations: none
---

## Decision

`crates/kernel/src/arena_seam.rs` is the only file in the tree that may `allow(unsafe_code)`. It is
capped at **60 non-blank lines** and must carry a written safety argument. Every other crate carries
`#![forbid(unsafe_code)]`.

**The file does not exist yet, and that is deliberate.** The check that governs it is written first and
passes with zero seams; M1 decides whether the arena needs one at all.

## Why

§17 requires that every boundary leaving the type system have exactly one construction site. The arena is
the one place where a typed column view over an owned allocation may need to bypass the borrow checker,
and if that need arises it should arise **in a place already named**, under a cap already agreed, rather
than in whichever module first found it convenient.

Declaring it before it is needed costs nothing and removes the argument later. Discovering the need first
and choosing the location afterwards is how a project acquires four seams and an exemption list.
