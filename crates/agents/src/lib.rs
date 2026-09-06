//! # `agents` — layer 7 of the eleven (§4)
//!
//! Policy, not work: one module per agent kind, declaring the five items of §8.1.
//!
//! **May depend on: `kernel`, `domain`, `declarations`, `world`, `markets`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
