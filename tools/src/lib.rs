//! # `tools` — the checks that hold the rules the compiler cannot
//!
//! Not a layer (ADR-0003). Each check exposes `check(root) -> findings` beside the `run` that prints
//! them, so the negative fixtures in `tools/tests/` can assert a planted violation is caught. §17's
//! empty exemption list is a claim, and a check that has never seen a violation cannot support it.
#![forbid(unsafe_code)]

pub mod check_lints;
pub mod check_surface;
