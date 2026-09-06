---
id: ADR-0003
title: The layer matrix, its two missing edges, and non-transitivity
status: accepted
date: 2026-09-06
register-entry: 16
claim-impact: none
guard: the Cargo dependency graph; a crate that is not a dependency cannot be named
supersedes: none
cost: a crate that needs a lower layer must name it, so the manifests are longer than the diagram
alternatives-rejected: transitive reachability; a custom import checker over the module graph
re-derivations: none
---

## Decision

The eleven layers of §4 are eleven crates in one Cargo workspace, and the permitted edges are the
matrix below. **The relation is explicit and non-transitive**: a crate may name only what its own row
lists, and depending on `world` does not confer the right to name `kernel`.

| Crate | May depend on |
|---|---|
| `kernel` | — |
| `domain` | `kernel` |
| `declarations` | `kernel`, `domain` |
| `world` | `kernel`, `domain` |
| `markets` | `kernel`, `domain` |
| `ledger` | `kernel`, `domain`, `world` |
| `agents` | `kernel`, `domain`, `declarations`, `world`, `markets` |
| `systems` | `kernel`, `domain`, `declarations`, `world`, `markets`, `ledger`, `agents` |
| `runtime` | `kernel`, `domain`, `declarations` |
| `surface` | `kernel`, `domain`, `world` |
| `shell` | `kernel`, `domain`, `surface` |
| `composition` | all of the above |

`tools` is not in the layer graph. It is the build and check machinery, it ships in no artifact, and it
may name anything it inspects.

## Why

**Two edges §4's diagram omits, and both are forced.** `world → kernel`: `world` allocates typed columns
out of the arena, and both live in `kernel`. `ledger → kernel, domain`: the ledger writes conserved
quantities, which are `domain` types stored in `kernel` columns. The diagram showed `world → domain` and
`ledger → world` and left the rest to be inferred, which is exactly the inference the next clause forbids.

**Non-transitive, because Cargo already is.** A Rust crate cannot name a transitive dependency unless the
intermediate crate re-exports it. The stricter reading is therefore the *default* reading, and adopting it
costs nothing: no custom checker, no configuration, no exemption list. The looser reading would have to be
built deliberately, and §4's "an exemption list that is allowed one entry will have forty" is the argument
against building it.

**`shell` is separate from `surface` and cannot name `world`.** §4.4 forbids arithmetic in `surface`; a
user interface must compute — a label needs `width - 8`. Without the split, the first such line opens the
exemption §17 says has none. `shell` may compute and holds no world handle; `surface` holds the readers
and computes nothing.

## What this replaces

Three independent nets — package manager, type checker, and a custom module-graph walker — which the
predecessor project needed because type-only imports and re-exports slipped past the first two. **Cargo
needs one.** A crate that is not a dependency cannot be named in any form, including through a re-export.
That claim is not assumed here: `tools/tests/layer_refusal.rs` is a compile-fail fixture that proves it.
