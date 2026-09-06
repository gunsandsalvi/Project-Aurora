//! # `domain` — layer 2 of the eleven (§4)
//!
//! Vocabulary and pure arithmetic. No state. Also the parallel vocabulary: `RowSpan`, `ShardIndex`, `Selector`, `Cadence`, `Phase`, `Accumulator<T>`.
//!
//! **May depend on: `kernel`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
