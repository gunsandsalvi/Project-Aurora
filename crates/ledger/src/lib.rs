//! # `ledger` — layer 6 of the eleven (§4)
//!
//! The only crate that can obtain a writable view of holdings, liens or obligations, and therefore the only one that can mint a handle over one. Owns the conserved quantity column as a private field (A1).
//!
//! **May depend on: `kernel`, `domain`, `world`.** The matrix is ADR-0003 and it is enforced by Cargo: a crate that is
//! not a dependency cannot be named, in any form, including through a re-export.
#![forbid(unsafe_code)]
#![no_std]
