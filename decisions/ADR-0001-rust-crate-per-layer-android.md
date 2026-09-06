---
id: ADR-0001
title: Rust, one crate per layer, delivered as an Android application
status: accepted
date: 2026-09-06
register-entry: 1
claim-impact: A1
guard: the crate graph; #![forbid(unsafe_code)]; private fields on newtypes and on the conserved column
supersedes: Appendix A entry 1 (TypeScript, typed columns over ArrayBuffer, strict as a build gate)
cost: the specification's language decision is reopened, and a less familiar toolchain
alternatives-rejected: TypeScript as specified; deciding by measurement first
re-derivations: §4 (layers become crates), §5.1 (i64 quantities), §5.2 (newtypes), §11 and §17 (most of the lint apparatus), §20 R18 (largely retired)
---

## Decision

The engine is Rust, one crate per layer of §4, delivered as an Android application: a native core and a
thin user interface. **This records founding decision D2** (Appendix B).

## Why

The previous language erased its own guarantees before the program ran, which the register carried as
**R18 — the largest risk in it**: *every guarantee here is compile-time, in a language that erases types
at runtime.* Crate privacy, newtypes with private fields and unconstructable capability types are all
present at runtime, so the residue R18 described is gone rather than managed.

Three consequences, and each removes work rather than adding it:

- **The layer graph is the dependency graph.** A forbidden import is a compile error, not a lint with an
  exemption list to keep empty. ADR-0003 has the matrix and `tools/tests/layer_refusal.rs` proves it.
- **A1's single-writer property becomes module privacy.** The conserved column is a private field of the
  `ledger` crate. This also dissolves the false form of the claim — "exactly one writing statement in the
  source tree" was untrue against relocation, the tail shift and slot canonicalisation, all of which live
  inside the ledger.
- **§12.2's 96.5 ns per exchange becomes plausible** rather than optimistic, and an application rather
  than a page removes the allocation ceiling a browser tab implied.

**What is not claimed.** The performance question is still open and is still measured (W3). D1 means it
cannot stop the project either way; this decision changes what the answer is likely to be, not whether it
is needed.
