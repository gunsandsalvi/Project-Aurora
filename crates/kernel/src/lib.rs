//! # `kernel` — layer 1 of the eleven (§4)
//!
//! Storage primitives, typed columns, identifier machinery, quantity types, code generation. Knows no economics.
//!
//! **May depend on: nothing.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]

// Compiled only under `--cfg aurora_layer_probe`, by `tools/tests/layer_refusal.rs`, which asserts
// that this module FAILS to compile. See ADR-0003.
#[cfg(aurora_layer_probe)]
mod layer_probe;
