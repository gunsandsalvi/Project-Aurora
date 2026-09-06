//! # `systems` — layer 8 of the eleven (§4)
//!
//! The work of a position.
//!
//! **May depend on: `kernel`, `domain`, `declarations`, `world`, `markets`, `ledger`, `agents`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
