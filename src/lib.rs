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
//! `SPEC-001` scaffolded the crate; `SPEC-003` added [`ifd`], the TIFF/IFD
//! container reader and its fuzz target; `SPEC-004` added the develop
//! pipeline's remaining DNG integer tags (`BlackLevel`, `WhiteLevel`,
//! `ActiveArea`, `DefaultCropOrigin`/`Size`, `Orientation`, opcode-list
//! presence) as typed extraction. Still absent, by design: `RATIONAL`,
//! `ASCII` and the signed field types (no DNG tag PROJ-001 reads needs them
//! yet), and any pixel decode or unpack (STAGE-002).

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]

pub mod ifd;

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

    /// The first two bytes were neither `II` nor `MM`, so this is not a TIFF
    /// container (TIFF 6.0 §2, "Image File Header").
    NotTiff {
        /// The two bytes actually found.
        mark: [u8; 2],
    },

    /// The byte-order mark was valid but the version word was not 42.
    UnsupportedTiffVersion {
        /// The version word actually found.
        version: u16,
    },

    /// An IFD offset was reached twice. The cycle guard — a `SubIFD` chain
    /// that points at itself terminates here instead of recursing forever.
    CyclicIfd {
        /// The offset that was about to be visited a second time.
        offset: u32,
    },

    /// `SubIFD` recursion went deeper than [`ifd::MAX_IFD_DEPTH`].
    IfdDepthExceeded {
        /// The depth limit that was exceeded.
        limit: u32,
    },

    /// The walk reached [`ifd::MAX_IFDS`] directories. A chain long enough to
    /// hit this is hostile even when it is acyclic.
    TooManyIfds {
        /// The IFD-count limit that was reached.
        limit: usize,
    },

    /// A tag was present but carried a field type this reader does not read as
    /// an integer. The wider type model is `SPEC-004`'s.
    UnexpectedFieldType {
        /// The tag whose type was unexpected.
        tag: u16,
        /// The TIFF field type found (TIFF 6.0 §2, "Types").
        field_type: u16,
    },

    /// `count × sizeof(type)` overflowed, or a value did not fit the type it
    /// was being converted into.
    ValueOverflow {
        /// The tag whose size arithmetic overflowed.
        tag: u16,
    },

    /// A tag declared more values than [`ifd::MAX_TAG_VALUES`]. Bounds
    /// allocation from a hostile `count` independently of the file's length.
    TagTooLarge {
        /// The tag with the outsized count.
        tag: u16,
        /// The count the file declared.
        count: u32,
        /// The limit that was exceeded.
        limit: usize,
    },

    /// A tag the caller required is not present in the IFD.
    MissingTag {
        /// The absent tag.
        tag: u16,
    },

    /// An offset in the file is larger than this platform can address.
    OffsetOutOfRange {
        /// The offset that could not be represented as a `usize`.
        offset: u32,
    },

    /// No IFD matched the sensor-plane selection rule
    /// (`NewSubfileType == 0 && PhotometricInterpretation == 34892 &&
    /// SamplesPerPixel == 1`).
    NoSensorIfd,

    /// No IFD matched the sensor-plane rule outright, **and** at least one
    /// IFD's identifying tag (`NewSubfileType`, `PhotometricInterpretation`,
    /// or `SamplesPerPixel`) could not be read at all — that IFD's identity
    /// is *unknown*, not "not the sensor". Returned instead of a bare
    /// [`Error::NoSensorIfd`] so a malformed tag on the true sensor IFD does
    /// not silently masquerade as "this file has no raw plane" (`DEC-012`;
    /// `SPEC-004` FU-11).
    NoSensorIfdCandidatesMalformed {
        /// `(ifd_index, tag)` for every IFD whose identity could not be
        /// determined — `tag` is whichever identifying tag failed to read.
        candidates: Vec<(usize, u16)>,
    },

    /// The sensor plane is compressed and this library does not decode that
    /// compression. Three of the seven corpus files land here, and landing
    /// here **is** the clean rejection.
    UnsupportedCompression {
        /// The TIFF `Compression` code found.
        compression: u32,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Truncated { at, len } => {
                write!(f, "truncated input: needed {len} byte(s) at offset {at}")
            }
            Error::NotTiff { mark } => {
                let (a, b) = (mark.first().copied(), mark.last().copied());
                write!(
                    f,
                    "not a TIFF container: byte-order mark is {:#04x}{:#04x}, expected II or MM",
                    a.unwrap_or(0),
                    b.unwrap_or(0)
                )
            }
            Error::UnsupportedTiffVersion { version } => {
                write!(f, "unsupported TIFF version {version}, expected 42")
            }
            Error::CyclicIfd { offset } => {
                write!(f, "cyclic IFD: offset {offset} was already visited")
            }
            Error::IfdDepthExceeded { limit } => {
                write!(f, "SubIFD recursion exceeded the depth limit of {limit}")
            }
            Error::TooManyIfds { limit } => {
                write!(f, "IFD walk reached the limit of {limit} directories")
            }
            Error::UnexpectedFieldType { tag, field_type } => {
                write!(f, "tag {tag} has unreadable field type {field_type}")
            }
            Error::ValueOverflow { tag } => {
                write!(f, "tag {tag}: value size arithmetic overflowed")
            }
            Error::TagTooLarge { tag, count, limit } => {
                write!(
                    f,
                    "tag {tag} declares {count} values, over the limit of {limit}"
                )
            }
            Error::MissingTag { tag } => write!(f, "required tag {tag} is absent"),
            Error::OffsetOutOfRange { offset } => {
                write!(f, "offset {offset} is not addressable on this platform")
            }
            Error::NoSensorIfd => f.write_str(
                "no IFD matched the sensor-plane rule (NewSubfileType 0, LinearRaw, 1 sample)",
            ),
            Error::NoSensorIfdCandidatesMalformed { candidates } => {
                write!(
                    f,
                    "no IFD matched the sensor-plane rule, and {} candidate(s) could not be \
                     evaluated because an identifying tag was malformed: {candidates:?}",
                    candidates.len()
                )
            }
            Error::UnsupportedCompression { compression } => {
                write!(
                    f,
                    "compression {compression} is not supported by this library"
                )
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
