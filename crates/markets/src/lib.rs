//! # `markets` — layer 5 of the eleven (§4)
//!
//! Price formation, deliberately independent of `world` so it can be tested and optimised alone.
//!
//! **May depend on: `kernel`, `domain`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
