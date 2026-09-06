//! # `tools` — the checks that hold the rules the compiler cannot
//!
//! Not a layer (ADR-0003). Each check exposes `check(root) -> findings` beside the `run` that prints
//! them, so the negative fixtures in `tools/tests/` can assert a planted violation is caught. §17's
//! empty exemption list is a claim, and a check that has never seen a violation cannot support it.
#![forbid(unsafe_code)]

pub mod adr_new;
pub mod appendix;
pub mod behaviour;
pub mod bootstrap;
pub mod burnin;
pub mod check_adr;
pub mod check_coupling;
pub mod check_deps;
pub mod check_instruments;
pub mod check_lints;
pub mod check_refs;
pub mod check_registry;
pub mod check_surface;
pub mod gate;
pub mod registry;
pub mod registry_cost;
pub mod seedgen;
pub mod sizing;
pub mod workload;
