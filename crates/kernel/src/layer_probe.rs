//! The layer matrix, proved rather than assumed (ADR-0003, §4).
//!
//! `kernel` is the bottom of the eleven layers and depends on nothing. This module names `world`,
//! which is four layers above it. It is compiled **only** under `--cfg aurora_layer_probe`, and
//! `tools/tests/layer_refusal.rs` builds it under that flag and asserts the build fails.
//!
//! It is committed because a check that has never seen a violation is not known to detect one. The
//! predecessor project needed three independent nets here — package manager, type checker, and a
//! custom module-graph walker — because type-only imports and re-exports slipped past the first two.
//! Cargo needs one, and this file is the evidence for that claim.

use aurora_world::AnythingAtAll;

pub fn this_must_not_compile() -> AnythingAtAll {
    unimplemented!()
}
