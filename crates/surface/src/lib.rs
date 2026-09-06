//! # `surface` — layer 10 of the eleven (§4)
//!
//! Named readers and the view model. **Computes nothing** (§4.4).
//!
//! **May depend on: `kernel`, `domain`, `world`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
