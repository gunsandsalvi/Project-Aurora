//! # `shell` — layer 11 of the eleven (§4)
//!
//! The user interface application. May compute; holds no world handle; cannot name `world`.
//!
//! **May depend on: `kernel`, `domain`, `surface`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
