//! # `world` — layer 4 of the eleven (§4)
//!
//! One module per table: schema, allocation, generated read views, span arithmetic.
//!
//! **May depend on: `kernel`, `domain`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
