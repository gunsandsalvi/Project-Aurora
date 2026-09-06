//! # `declarations` — layer 3 of the eleven (§4)
//!
//! One manifest per system: reads, writes, permitted counter-accounts, permitted amendments, owned series, cadence, selector, phase, accumulators. Pure data.
//!
//! **May depend on: `kernel`, `domain`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
