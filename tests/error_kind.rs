//! `SPEC-040` — `Error::kind()` / `ErrorKind`, the three-way classification
//! decided in `DEC-051`.
//!
//! The compiler already guarantees `kind()` is *total*: it is a wildcard-free
//! `match` inside the defining crate, so a new `Error` variant fails to compile
//! until it is classified. These tests pin the *assignment* — which kind each
//! variant gets — so a re-classification is a visible diff against `DEC-051`'s
//! table rather than a silent change.
//!
//! An integration test cannot itself match `Error` exhaustively (it is
//! `#[non_exhaustive]` outside the crate), so `every_variant_classified` also
//! reads the variant names out of `src/lib.rs` and requires the table below to
//! cover exactly that set: a variant added to the enum but not to this table
//! fails here by name.
//!
//! Tier A only: every representative is constructed in-test. No corpus, no
//! tools.

use std::collections::BTreeSet;

use irradiance::dng::write::{Framing, WriteInputError};
use irradiance::{Error, ErrorKind};

/// One representative of every `Error` variant, paired with `DEC-051`'s kind.
/// Grouped in `DEC-051`'s order: 15 `CallerError`, 21 `Unsupported`,
/// 30 `Corrupt` (25 at 0.2.0, plus `UnreadableOpcodeList`, SPEC-056/DEC-053;
/// SPEC-058/DEC-056 added `UnsupportedTileRowBits` (Unsupported) and
/// `InvalidTileLayout` + `StripsAndTiles` (Corrupt); SPEC-062/DEC-057 added
/// `UnsupportedPreviewLayout` (Unsupported) and `InvalidPreviewLayout`
/// (Corrupt); SPEC-063/DEC-058 added `UnsupportedWarpedFullActiveArea`
/// (Unsupported); SPEC-064/DEC-059 added `InvalidWarpAlpha` (CallerError);
/// SPEC-065/DEC-060 added `DevelopScratchWrongLength` (CallerError);
/// SPEC-066/DEC-061 added `InvalidBlackLevelPattern` (Corrupt);
/// SPEC-068/DEC-063 added `PlaneUnitMismatch` (CallerError); the 2026-11
/// review fixes added `UnsupportedCfaLayout`, `UnsupportedCfaPlaneColor`,
/// `UnsupportedOpcodeVersion` (Unsupported) and `NonFiniteWarpCoefficients`
/// (CallerError); SPEC-086/DEC-077 added `UnsupportedDngVersion` and
/// `UnsupportedTag` (Unsupported) and `LjpegPrecisionMismatch` (Corrupt);
/// SPEC-087/DEC-078 added `MalformedTag` (Corrupt); SPEC-061/DEC-075 added
/// `InvalidWriteInput` (CallerError) and `UnsupportedOpcodeForFraming`
/// (Unsupported); SPEC-091/DEC-079 added `InvalidRowRange` (CallerError);
/// SPEC-092/DEC-083 added `InvalidMixWeights` and `MonoBufferWrongLength`
/// (CallerError) and `MixRowOutOfRange` (Unsupported)).
fn dec_051_table() -> Vec<(Error, ErrorKind)> {
    use ErrorKind::{CallerError, Corrupt, Unsupported};
    vec![
        // CallerError (15).
        (
            Error::PlaneBufferWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::PlaneTooLarge {
                pixels: 1 << 32,
                limit: 1 << 20,
            },
            CallerError,
        ),
        (
            Error::SourcePlaneWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::DevelopBufferWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::WarpSourceWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::WarpDestWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::FixBadPixelsPlaneWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::GainMapImageWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::OutputBufferWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::DemosaicSourceWrongLength {
                expected: 16,
                actual: 15,
            },
            CallerError,
        ),
        (
            Error::RgbBufferWrongLength {
                expected: 48,
                actual: 47,
            },
            CallerError,
        ),
        (
            Error::ColorSourceWrongLength {
                expected: 48,
                actual: 47,
            },
            CallerError,
        ),
        (
            Error::LjpegPlaneWrongLength {
                expected: 16,
                got: 15,
            },
            CallerError,
        ),
        // SPEC-064, DEC-059: a warp_alpha outside [0, 1] is the caller's
        // argument, whatever the file holds.
        (Error::InvalidWarpAlpha { warp_alpha: 1.5 }, CallerError),
        // A hand-built warp with a NaN/inf coefficient; the parser refuses
        // the same bytes as MalformedOpcodeParams (Corrupt).
        (Error::NonFiniteWarpCoefficients, CallerError),
        // SPEC-065, DEC-060: a develop scratch of the wrong length is the
        // caller's buffer.
        (
            Error::DevelopScratchWrongLength {
                expected: 122,
                actual: 121,
            },
            CallerError,
        ),
        // SPEC-068, DEC-063: a plane unit that is not a unit of this
        // sensor's layout is the caller's argument.
        (
            Error::PlaneUnitMismatch {
                row_start: 4928,
                row_count: 704,
            },
            CallerError,
        ),
        // Unsupported (21).
        (Error::NotTiff { mark: *b"PK" }, Unsupported),
        (Error::UnsupportedTiffVersion { version: 43 }, Unsupported),
        (Error::NoSensorIfd, Unsupported),
        (
            Error::UnsupportedCompression { compression: 34713 },
            Unsupported,
        ),
        (Error::UnsupportedBitDepth { bits: 10 }, Unsupported),
        (
            Error::UnsupportedStripLayout {
                strip_offsets: 2,
                strip_byte_counts: 3,
            },
            Unsupported,
        ),
        (
            Error::UnsupportedTileRowBits {
                tile_width: 3,
                bits: 12,
            },
            Unsupported,
        ),
        (
            Error::UnsupportedOrientation { orientation: 9 },
            Unsupported,
        ),
        (Error::UnsupportedMandatoryOpcode { id: 99 }, Unsupported),
        (
            Error::UnsupportedRequiredOpcode { list: 2, id: 1 },
            Unsupported,
        ),
        (
            Error::UnsupportedWarpTangentialTerms { kt0: 0.5, kt1: 0.0 },
            Unsupported,
        ),
        (Error::NotAColorFilterArray, Unsupported),
        (
            Error::UnsupportedCfaPattern {
                repeat_dim: [6, 6],
                pattern: vec![1; 36],
            },
            Unsupported,
        ),
        (Error::UnsupportedCfaLayout { layout: 2 }, Unsupported),
        (
            Error::UnsupportedOpcodeVersion {
                id: 4,
                version: 0x0200_0000,
            },
            Unsupported,
        ),
        (
            Error::UnsupportedCfaPlaneColor {
                plane_color: vec![2, 1, 0],
            },
            Unsupported,
        ),
        // SPEC-086, DEC-077: a DNG backward version newer than 1.7.1.0, and a
        // tag that changes what the samples mean with a value not honoured.
        (
            Error::UnsupportedDngVersion {
                backward_version: [1, 8, 0, 0],
            },
            Unsupported,
        ),
        (Error::UnsupportedTag { tag: 50712 }, Unsupported),
        (Error::MissingColorCalibration, Unsupported),
        (Error::LjpegUnsupportedProcess { sof: 0xC0 }, Unsupported),
        (Error::LjpegUnsupportedComponentCount { nf: 3 }, Unsupported),
        (
            Error::LjpegUnsupportedPrecision { precision: 1 },
            Unsupported,
        ),
        (
            Error::LjpegUnsupportedPredictor { predictor: 0 },
            Unsupported,
        ),
        (
            Error::LjpegUnsupportedSampling { factor: 0x21 },
            Unsupported,
        ),
        // SPEC-062, DEC-057: a preview layout this library does not return as
        // one zero-copy slice (tiled, old-style JPEG, non-contiguous strips).
        (
            Error::UnsupportedPreviewLayout {
                ifd_index: 0,
                layout: irradiance::UnsupportedPreview::Tiled,
            },
            Unsupported,
        ),
        // SPEC-063, DEC-058: FullActiveArea framing with a non-identity
        // WarpRectilinear; since SPEC-064/DEC-059 only one whose lerped
        // radial scale leaves [0, 1] or whose centre is outside the image.
        (
            Error::UnsupportedWarpedFullActiveArea {
                active_width: 8392,
                active_height: 5632,
                warp_alpha: 1.0,
                scale_min: 0.98,
                scale_max: 1.0015,
            },
            Unsupported,
        ),
        // Corrupt (30).
        // SPEC-062, DEC-057: a preview IFD with no usable strip table.
        (
            Error::InvalidPreviewLayout {
                ifd_index: 0,
                strip_offsets: 2,
                strip_byte_counts: 1,
            },
            Corrupt,
        ),
        (Error::Truncated { at: 8, len: 12 }, Corrupt),
        // SPEC-056, DEC-053: a present-but-unreadable OpcodeList.
        (Error::UnreadableOpcodeList { list: 1 }, Corrupt),
        (Error::CyclicIfd { offset: 8 }, Corrupt),
        (Error::IfdDepthExceeded { limit: 8 }, Corrupt),
        (Error::TooManyIfds { limit: 256 }, Corrupt),
        (Error::TooManyGainMaps { count: 17 }, Corrupt),
        (
            Error::IfdTableBytesExceedFile {
                declared: 12_288,
                file_len: 770,
            },
            Corrupt,
        ),
        (
            Error::UnexpectedFieldType {
                tag: 256,
                field_type: 2,
            },
            Corrupt,
        ),
        (Error::ValueOverflow { tag: 273 }, Corrupt),
        (
            Error::MalformedRationalValue {
                tag: 50717,
                numerator: 1,
                denominator: 0,
            },
            Corrupt,
        ),
        (
            Error::RationalValueOutOfRange {
                tag: 50721,
                value: u32::MAX,
            },
            Corrupt,
        ),
        (
            Error::TagTooLarge {
                tag: 273,
                count: u32::MAX,
                limit: 1 << 20,
            },
            Corrupt,
        ),
        (Error::MissingTag { tag: 256 }, Corrupt),
        (Error::OffsetOutOfRange { offset: u32::MAX }, Corrupt),
        (
            Error::NoSensorIfdCandidatesMalformed {
                candidates: vec![(1, 254)],
            },
            Corrupt,
        ),
        (Error::MissingRowsPerStrip { strips: 2 }, Corrupt),
        (
            Error::PackedSizeMismatch {
                expected_bits: 256,
                strip_bits: 248,
            },
            Corrupt,
        ),
        (
            Error::StripBytesExceedFile {
                declared: 1 << 20,
                file_len: 4096,
            },
            Corrupt,
        ),
        (
            Error::InvalidTileLayout {
                tile_width: 256,
                tile_length: 256,
                tile_offsets: 191,
                tile_byte_counts: 192,
                expected_tiles: 192,
            },
            Corrupt,
        ),
        (
            Error::StripsAndTiles {
                strip_offsets: 1,
                tile_offsets: 192,
            },
            Corrupt,
        ),
        (
            Error::InvalidActiveArea {
                top: 0,
                left: 0,
                bottom: 0,
                right: 4,
                plane_width: 4,
                plane_height: 4,
            },
            Corrupt,
        ),
        (
            Error::InvalidDefaultCrop {
                origin_x: 2,
                origin_y: 2,
                crop_width: 4,
                crop_height: 4,
                active_width: 4,
                active_height: 4,
            },
            Corrupt,
        ),
        (
            Error::InvalidLevels {
                black_level: 4095,
                white_level: 1023,
            },
            Corrupt,
        ),
        (
            Error::InvalidBlackLevelPattern {
                repeat_rows: 2,
                repeat_cols: 2,
                count: 3,
            },
            Corrupt,
        ),
        (
            Error::MalformedOpcodeParams {
                id: 1,
                declared_size: 7,
            },
            Corrupt,
        ),
        (
            Error::LjpegInvalidPointTransform {
                point_transform: 14,
                precision: 14,
            },
            Corrupt,
        ),
        (Error::LjpegInvalidHuffmanTable, Corrupt),
        (Error::LjpegMissingMarker { marker: 0xD8 }, Corrupt),
        (Error::LjpegUnexpectedMarker { marker: 0xD9 }, Corrupt),
        (
            Error::LjpegPlaneShapeMismatch {
                expected_width: 8,
                expected_height: 2,
                decoded_width: 2,
                decoded_height: 8,
            },
            Corrupt,
        ),
        // SPEC-086, DEC-077 §3: a SOF-3 precision other than BitsPerSample.
        (
            Error::LjpegPrecisionMismatch {
                precision: 16,
                bits_per_sample: 12,
            },
            Corrupt,
        ),
        // SPEC-087, DEC-078 §3: a tag the develop path reads is present but
        // malformed.
        (Error::MalformedTag { tag: 50717 }, Corrupt),
        // SPEC-061, DEC-075 §7: the writer refused the caller's input.
        (
            Error::InvalidWriteInput(WriteInputError::MissingUniqueCameraModel),
            CallerError,
        ),
        // SPEC-061, DEC-075 §5: an OpcodeList1 opcode a cut would shift.
        (
            Error::UnsupportedOpcodeForFraming {
                list: 1,
                opcode_id: 5,
                framing: Framing::StandardCrop,
            },
            Unsupported,
        ),
        // SPEC-091, DEC-079: a band form's row range is empty, reversed or
        // past the plane's height.
        (
            Error::InvalidRowRange {
                start: 6,
                end: 5,
                height: 17,
            },
            CallerError,
        ),
        // SPEC-092, DEC-083: the mixer's weights are ill-formed or do not sum
        // to one; its dst is not one sample per pixel; its fixed-point row
        // does not fit i32.
        (Error::InvalidMixWeights { channel: Some(1) }, CallerError),
        (
            Error::MonoBufferWrongLength {
                expected: 340,
                actual: 339,
            },
            CallerError,
        ),
        (Error::MixRowOutOfRange { channel: 0 }, Unsupported),
    ]
}

/// The variant name of an `Error` value: the identifier its `Debug` output
/// starts with (`Truncated { .. }` -> `Truncated`, `NoSensorIfd` ->
/// `NoSensorIfd`).
fn variant_name(error: &Error) -> String {
    format!("{error:?}")
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect()
}

/// Every variant name declared in `src/lib.rs`'s `pub enum Error { .. }`.
///
/// Reads the source text, so a variant added to the enum is seen here without
/// this test being edited. A variant is a line indented exactly four spaces
/// that starts with an uppercase identifier and ends in ` {` (struct variant),
/// `,` (unit variant) or `(..),` (tuple variant, SPEC-061's
/// `InvalidWriteInput`) — the shape `cargo fmt` gives every variant today.
fn declared_variants() -> BTreeSet<String> {
    let source = include_str!("../src/lib.rs");
    let body = source
        .split_once("\npub enum Error {\n")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split_once("\n}\n").map(|(body, _)| body))
        .unwrap_or_else(|| panic!("src/lib.rs has no `pub enum Error {{ .. }}` block"));
    body.lines()
        .filter_map(|line| line.strip_prefix("    "))
        .filter(|line| line.starts_with(|c: char| c.is_ascii_uppercase()))
        .filter_map(|line| {
            line.strip_suffix(" {")
                .or_else(|| line.strip_suffix(','))
                .map(|name| name.split_once('(').map_or(name, |(head, _)| head))
                .filter(|name| name.chars().all(|c| c.is_ascii_alphanumeric()))
                .map(str::to_owned)
        })
        .collect()
}

#[test]
fn every_variant_classified() {
    let table = dec_051_table();

    // Each variant appears once — a duplicate would hide a missing one.
    let tested: BTreeSet<String> = table.iter().map(|(e, _)| variant_name(e)).collect();
    assert_eq!(
        tested.len(),
        table.len(),
        "the DEC-051 table lists some variant more than once"
    );

    // The table covers exactly the enum as declared in src/lib.rs.
    let declared = declared_variants();
    let untested: Vec<_> = declared.difference(&tested).collect();
    let unknown: Vec<_> = tested.difference(&declared).collect();
    assert!(
        untested.is_empty() && unknown.is_empty(),
        "Error variants missing from the DEC-051 table: {untested:?}; \
         table entries not in src/lib.rs: {unknown:?}"
    );
    // DEC-051's own count. Adding a variant means updating the table above,
    // this number, and DEC-051 together.
    assert_eq!(
        declared.len(),
        82,
        "parsed {} variants from src/lib.rs; DEC-051 (+ DEC-053, DEC-056, DEC-057, DEC-058, DEC-059, DEC-060, DEC-061, DEC-063, DEC-069, the 2026-11 review fixes, DEC-077, DEC-078, DEC-075, DEC-079 and DEC-083) classified 82",
        declared.len()
    );

    // Every representative carries DEC-051's kind.
    let wrong: Vec<_> = table
        .iter()
        .filter(|(e, want)| e.kind() != *want)
        .map(|(e, want)| {
            format!(
                "{}: got {:?}, DEC-051 says {want:?}",
                variant_name(e),
                e.kind()
            )
        })
        .collect();
    assert!(wrong.is_empty(), "misclassified: {wrong:#?}");

    // DEC-051's bucket sizes: 17 / 26 / 33 (DEC-053 added one Corrupt;
    // DEC-056 one Unsupported and two Corrupt; DEC-057 one of each; DEC-058
    // one Unsupported; DEC-059 and DEC-060 one CallerError each; DEC-061 one
    // Corrupt; DEC-063 one CallerError; DEC-069 one CallerError and one
    // Corrupt; the 2026-11 review fixes three Unsupported and one
    // CallerError; DEC-077 two Unsupported and one Corrupt; DEC-078 one
    // Corrupt; DEC-075 one CallerError and one Unsupported; DEC-079 one
    // CallerError).
    let count = |kind| table.iter().filter(|(_, k)| *k == kind).count();
    assert_eq!(count(ErrorKind::CallerError), 21);
    assert_eq!(count(ErrorKind::Unsupported), 28);
    assert_eq!(count(ErrorKind::Corrupt), 33);
}

#[test]
fn the_three_buckets_are_distinct() {
    let unsupported = Error::UnsupportedCompression { compression: 34713 }.kind();
    let corrupt = Error::Truncated { at: 0, len: 8 }.kind();
    let caller = Error::PlaneBufferWrongLength {
        expected: 16,
        actual: 15,
    }
    .kind();

    assert_eq!(unsupported, ErrorKind::Unsupported);
    assert_eq!(corrupt, ErrorKind::Corrupt);
    assert_eq!(caller, ErrorKind::CallerError);
    assert_ne!(unsupported, corrupt);
    assert_ne!(unsupported, caller);
    assert_ne!(corrupt, caller);
}

#[test]
fn spec_039_additions_classified() {
    // DEC-051 judgment call: the budget is the caller's own argument (DEC-050),
    // so exceeding it is a caller-side remedy, not a corrupt file.
    assert_eq!(
        Error::PlaneTooLarge {
            pixels: u64::from(u32::MAX) * u64::from(u32::MAX),
            limit: 100_000_000,
        }
        .kind(),
        ErrorKind::CallerError
    );
    // Aliased strips are a forged layout in the file's own bytes.
    assert_eq!(
        Error::StripBytesExceedFile {
            declared: u64::MAX,
            file_len: 1024,
        }
        .kind(),
        ErrorKind::Corrupt
    );
}

#[test]
fn dec_051_judgment_calls() {
    // Not a file this library decodes, rather than bad bytes.
    assert_eq!(
        Error::NotTiff { mark: *b"\x89P" }.kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(Error::NoSensorIfd.kind(), ErrorKind::Unsupported);
    // ...but candidates that existed and could not be read are bad data.
    assert_eq!(
        Error::NoSensorIfdCandidatesMalformed {
            candidates: vec![(0, 262)],
        }
        .kind(),
        ErrorKind::Corrupt
    );
    // A missing capability, unlike a missing low-level TIFF tag.
    assert_eq!(
        Error::MissingColorCalibration.kind(),
        ErrorKind::Unsupported
    );
    assert_eq!(Error::MissingTag { tag: 256 }.kind(), ErrorKind::Corrupt);
}

#[test]
fn error_kind_is_copy_and_comparable() {
    let kind = Error::NoSensorIfd.kind();
    let copied = kind;
    assert_eq!(kind, copied);
    assert_eq!(format!("{kind:?}"), "Unsupported");
}
