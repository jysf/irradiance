//! `irradiance` — a permissively-licensed, pure-Rust library that reads
//! camera RAW files and develops them into images: sensor data in, pixels
//! and metadata out.
//!
//! This crate does **no I/O**, ships **no CLI**, and depends on **no async
//! runtime** (constraint `library-not-application`, `guidance/constraints.yaml`).
//! The `irr` binary alongside this crate is an internal dev/oracle tool only
//! and is never part of the public API — see `src/bin/irr.rs`.
//!
//! # Panic policy
//!
//! RAW is attacker-influenced binary input (vendor-supplied offsets, tile
//! tables, Huffman tables). Every fallible read on a parse/decode path
//! returns a typed [`Error`] instead of panicking. This is enforced
//! mechanically, not only by review: the five lints below are `deny`-level
//! here and are only relaxed in `#[cfg(test)]` and in `src/bin/irr.rs`,
//! neither of which is a library path.
//!
//! That claim is itself checked. `scripts/lint-red-proof.sh` (CI job
//! `lint-policy-red-proof`, `DEC-009`) runs clippy over a temp-dir copy of
//! this crate three times: once **unmutated**, which must pass; once with one
//! violation per lint injected after the attribute prologue, which must fail
//! with all five lints firing *at the injected lines*; and once more without
//! CI's blanket `-D warnings`, which must also fail — that last run is what
//! pins the block below at `deny` rather than `warn`.
//!
//! The unmutated run is the **control**, and it is the load-bearing part: it
//! is what makes a failure of the mutated run attributable to the injection
//! rather than to a syntax error, a broken copy, or a toolchain fault. Delete
//! the block below, weaken it to `allow`, downgrade it to `warn`, or drop a
//! single lint from it, and the proof fails — which is the only reason the
//! sentence above is allowed to say "mechanically".
//!
//! What the proof does **not** establish: that code in a module carrying its
//! own `#[allow(...)]` is covered. It pins the policy at the crate root.
//!
//! See constraint `no-panics-on-untrusted-input` in `guidance/constraints.yaml`
//! and AGENTS.md §11/§12.
//!
//! This spec (`SPEC-001`) scaffolds the crate only — no TIFF walk, no tag
//! model, no unpack. Those land in SPEC-003/004.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

use core::fmt;

/// Errors produced while reading or developing a RAW file.
///
/// Every fallible path in this crate returns this type rather than
/// panicking (constraint `no-panics-on-untrusted-input`). `#[non_exhaustive]`
/// because later specs (SPEC-003 onward: the IFD reader, the tag model, the
/// plane unpacker) will add variants, and a match against this enum outside
/// the crate must not be able to assume today's variant set is complete.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The input ended before the expected data could be read.
    ///
    /// `at` is the byte offset the read started at; `len` is the number of
    /// bytes that were needed from that offset.
    Truncated {
        /// Byte offset the truncated read started at.
        at: usize,
        /// Number of bytes the read needed but did not have.
        len: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Truncated { at, len } => {
                write!(f, "truncated input: needed {len} byte(s) at offset {at}")
            }
        }
    }
}

impl core::error::Error for Error {}

#[cfg(test)]
#[allow(
    // Test code is not a parse/decode path exercised on untrusted input
    // (AGENTS.md §11, guidance/toolchain-brief.md); the same five lints are
    // allowed here as in src/bin/irr.rs.
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    #[test]
    fn error_type_is_public_and_non_exhaustive() {
        // Error is constructible from within the crate and Debug-printable.
        let e = crate::Error::Truncated { at: 0, len: 0 };
        assert!(format!("{e:?}").contains("Truncated"));
    }

    #[test]
    fn error_display_carries_context() {
        let e = crate::Error::Truncated { at: 4, len: 2 };
        let msg = e.to_string();
        assert!(msg.contains('4'));
        assert!(msg.contains('2'));
    }
}
