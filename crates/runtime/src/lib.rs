//! # `runtime` — layer 9 of the eleven (§4)
//!
//! The loop, the committed order, the period trace.
//!
//! **May depend on: `kernel`, `domain`, `declarations`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
