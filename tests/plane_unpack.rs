//! `SPEC-012` — strip location and sample unpack, `DEC-008`'s two paths.
//!
//! Two lanes, as `tests/ifd_reader.rs` establishes the pattern:
//!
//! - **Tier A** (`each_path_produces_impossible_values_on_the_others_data`,
//!   `a_plane_whose_max_exceeds_white_level_is_an_error`,
//!   `layer0_arithmetic_is_enforced`'s hand-built half,
//!   `hostile_strip_bounds_do_not_panic`) run everywhere. `Fixture` below
//!   builds minimal sensor-only TIFFs with a real strip payload planted at a
//!   chosen offset — the measured strip heads from `DEC-008`/`SPEC-012`'s
//!   `## Implementation Context`, not tier-B files.
//! - **Tier B** (`unpacks_fourteen_bit_msb_first_samples`,
//!   `unpacks_sixteen_bit_in_file_byte_order`,
//!   `compressed_files_are_rejected_without_decoding`,
//!   `layer0_arithmetic_is_enforced`'s corpus half) need real files under
//!   `$IRRADIANCE_CORPUS_DIR` and skip loudly, per-entry, when absent.
//!
//! The tier-B expected sample values are **not** re-derived from `dnglab` at
//! test time — they are pinned constants, measured two independent ways
//! (hand-unpacked from the strip bytes, and read from `dnglab --raw-pixel`'s
//! own plane) during design and recorded in the spec. Reproducing the dnglab
//! side again here would only re-run the same oracle SPEC-013 owns.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/tiff.rs"]
mod tiff;

use corpus::{CorpusRoot, Manifest};
use irradiance::ifd::{ByteOrder, Container, TAG_WHITE_LEVEL};
use irradiance::plane::unpack_into;
use irradiance::Error;
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Tier-A fixture builder
// ─────────────────────────────────────────────────────────────────────────────

/// A minimal sensor-only TIFF (IFD0 *is* the sensor plane) with a strip
/// payload planted at `strip_offset`. `strip_byte_count` is the DECLARED
/// `StripByteCounts` value — deliberately a separate field from
/// `strip_bytes.len()` so a fixture can say "the file promises more bytes
/// than it actually holds" (`hostile_strip_bounds_do_not_panic`) or "the
/// declared count does not match width x height x bits"
/// (`layer0_arithmetic_is_enforced`).
struct Fixture<'a> {
    order: Order,
    width: u32,
    height: u32,
    bits: u16,
    strip_offset: u32,
    strip_byte_count: u32,
    strip_bytes: &'a [u8],
    white_level: Option<u32>,
}

impl Fixture<'_> {
    fn build(&self) -> Vec<u8> {
        let mut entries = vec![
            tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
            tiff::long(tiff::IMAGE_WIDTH, self.width),
            tiff::long(tiff::IMAGE_LENGTH, self.height),
            tiff::short(tiff::BITS_PER_SAMPLE, self.bits, self.order),
            tiff::short(tiff::COMPRESSION, 1, self.order),
            tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, self.order),
            tiff::short(tiff::SAMPLES_PER_PIXEL, 1, self.order),
            tiff::long(tiff::STRIP_OFFSETS, self.strip_offset),
            tiff::long(tiff::STRIP_BYTE_COUNTS, self.strip_byte_count),
        ];
        if let Some(wl) = self.white_level {
            entries.push(tiff::long(TAG_WHITE_LEVEL, wl));
        }
        let mut data = tiff::tiff(self.order, 8, &[tiff::Ifd::new(8, entries, 0)]);

        let at = self.strip_offset as usize;
        let end = at + self.strip_bytes.len();
        if data.len() < end {
            data.resize(end, 0);
        }
        data[at..end].copy_from_slice(self.strip_bytes);
        data
    }
}

/// `L1021223.DNG`'s strip head — `DEC-008`/`SPEC-012` Implementation Context.
/// 14-bit, MSB-first: samples 0-7 are `[746, 725, 711, 752, 646, 705, 772,
/// 686]`.
const Q2M_STRIP_HEAD: [u8; 14] = [
    0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
];

/// `L1000622.DNG`'s strip head. 16-bit, file byte order (`II`): samples 0-7
/// are `[4761, 4591, 4622, 4363, 4542, 4383, 4608, 4286]`.
const M_MONO_STRIP_HEAD: [u8; 16] = [
    0x99, 0x12, 0xef, 0x11, 0x0e, 0x12, 0x0b, 0x11, 0xbe, 0x11, 0x1f, 0x11, 0x00, 0x12, 0xbe, 0x10,
];

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — each path produces the OTHER path's data as impossible values
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn each_path_produces_impossible_values_on_the_others_data() {
    // Case 1: the Q2M's 14-bit-packed strip, declared as 16-bit-aligned.
    // 7 pixels x 16 bits = 112 bits = 14 bytes, matching the strip exactly —
    // a plausible-looking (wrong) declaration, not a truncated file.
    let wrong_as_sixteen = Fixture {
        order: Order::Little,
        width: 7,
        height: 1,
        bits: 16,
        strip_offset: 512,
        strip_byte_count: 14,
        strip_bytes: &Q2M_STRIP_HEAD,
        white_level: None, // no assertion to trip — this fixture's whole point
    }
    .build();
    let container = Container::parse(&wrong_as_sixteen).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 7];
    unpack_into(&sensor, container.byte_order(), &wrong_as_sixteen, &mut dst)
        .expect("no WhiteLevel is present to trip on the impossible values");
    assert_eq!(
        &dst[..4],
        &[43019, 20525, 49841, 2800],
        "14-bit strip misread as 16-bit LE must reproduce the measured wrong values"
    );

    // Case 2: the M Monochrom's 16-bit strip, declared big-endian (`MM`) —
    // the file's real byte order is `II`; this is what reading it the wrong
    // way produces.
    let wrong_as_big_endian = Fixture {
        order: Order::Big,
        width: 8,
        height: 1,
        bits: 16,
        strip_offset: 512,
        strip_byte_count: 16,
        strip_bytes: &M_MONO_STRIP_HEAD,
        white_level: None,
    }
    .build();
    let container = Container::parse(&wrong_as_big_endian).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    assert_eq!(container.byte_order(), ByteOrder::Big);
    let mut dst = [0u16; 8];
    unpack_into(
        &sensor,
        container.byte_order(),
        &wrong_as_big_endian,
        &mut dst,
    )
    .expect("no WhiteLevel is present to trip on the impossible values");
    assert_eq!(
        &dst[..4],
        &[39186, 61201, 3602, 2833],
        "16-bit LE strip misread as big-endian must reproduce the measured wrong values"
    );

    // Both wrong-path values (43019, 39186) exceed WhiteLevel 16383 — the
    // assertion that actually catches this in practice is AC4, tested below.
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — max > WhiteLevel is a loud, unconditional error
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn a_plane_whose_max_exceeds_white_level_is_an_error() {
    let data = Fixture {
        order: Order::Little,
        width: 7,
        height: 1,
        bits: 16, // wrong on purpose — see AC3's first case
        strip_offset: 512,
        strip_byte_count: 14,
        strip_bytes: &Q2M_STRIP_HEAD,
        white_level: Some(16383), // now there IS an assertion to trip
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 7];
    let err = unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect_err("sample 0 (43019) exceeds WhiteLevel 16383");
    match err {
        Error::SampleExceedsWhiteLevel {
            index,
            sample,
            white_level,
        } => {
            assert_eq!(index, 0);
            assert_eq!(sample, 43019);
            assert_eq!(white_level, 16383);
        }
        other => panic!("expected SampleExceedsWhiteLevel, got {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — layer-0: width x height x bits == StripByteCounts x 8
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn layer0_arithmetic_is_enforced() {
    // Tier A: a declared StripByteCounts that does not match width x height
    // x bits, independent of whether the file even holds that many bytes.
    let data = Fixture {
        order: Order::Little,
        width: 5, // wrong: 5 x 2 x 14 = 140 bits, not 112
        height: 2,
        bits: 14,
        strip_offset: 512,
        strip_byte_count: 14,
        strip_bytes: &Q2M_STRIP_HEAD,
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 10];
    let err = unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect_err("5 x 2 x 14 bits != 14 bytes x 8");
    assert!(
        matches!(
            err,
            Error::PackedSizeMismatch {
                expected_bits: 140,
                strip_bits: 112,
            }
        ),
        "expected PackedSizeMismatch{{140, 112}}, got {err:?}"
    );

    // Tier B: the layer-0 invariant on the two DECODABLE real files —
    // free, no oracle tooling, per AGENTS.md §12 bar 3.
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    for path in [
        "LEICA-Q2-MONO/L1021223.DNG",
        "LEICA-M-MONOCHROM/L1000622.DNG",
    ] {
        let Some(entry) = manifest.get(path) else {
            panic!("{path} must be in the manifest");
        };
        let Some(file_path) = entry.require(&root) else {
            continue; // SKIP already announced by CorpusFile::require
        };
        let bytes = std::fs::read(&file_path).expect("read corpus file");
        let sensor = Container::parse(&bytes)
            .expect("parses")
            .sensor()
            .expect("has a sensor plane");
        let expected = sensor.packed_bits().expect("no overflow on a real file");
        let declared = u64::from(
            *sensor
                .strip_byte_counts
                .first()
                .expect("real files carry one strip"),
        ) * 8;
        assert_eq!(expected, declared, "{path}: layer-0 must close");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — panic-free on hostile input
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn hostile_strip_bounds_do_not_panic() {
    // A strip that promises more bytes than the file actually holds.
    let truncated = Fixture {
        order: Order::Little,
        width: 4,
        height: 2,
        bits: 14,
        strip_offset: 512,
        strip_byte_count: 14, // consistent with width x height x bits...
        strip_bytes: &[],     // ...but none of it is actually in the file
        white_level: None,
    }
    .build();
    let container = Container::parse(&truncated).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 8];
    let err = unpack_into(&sensor, container.byte_order(), &truncated, &mut dst)
        .expect_err("StripByteCounts promises bytes the file does not hold");
    assert!(
        matches!(err, Error::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );

    // Zero dimensions: a degenerate but legitimate empty plane, not a panic.
    let empty = Fixture {
        order: Order::Little,
        width: 0,
        height: 0,
        bits: 14,
        strip_offset: 8, // inside the header/IFD area, which always exists
        strip_byte_count: 0,
        strip_bytes: &[],
        white_level: None,
    }
    .build();
    let container = Container::parse(&empty).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst: [u16; 0] = [];
    unpack_into(&sensor, container.byte_order(), &empty, &mut dst)
        .expect("zero-by-zero is a legitimate, if useless, empty plane");

    // Absurd dimensions: width x height alone doesn't overflow u64, but it
    // will never match any buffer this test is willing to allocate — a
    // typed length-mismatch error, not an attempt to honour the request.
    let absurd = Fixture {
        order: Order::Little,
        width: u32::MAX,
        height: u32::MAX,
        bits: 14,
        strip_offset: 8,
        strip_byte_count: 0,
        strip_bytes: &[],
        white_level: None,
    }
    .build();
    let container = Container::parse(&absurd).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 1];
    assert!(
        unpack_into(&sensor, container.byte_order(), &absurd, &mut dst).is_err(),
        "absurd dimensions must be a typed error, not a panic or a silent truncation"
    );

    // bits outside {8, 12, 14, 16}.
    let odd_bits = Fixture {
        order: Order::Little,
        width: 4,
        height: 2,
        bits: 10,
        strip_offset: 512,
        strip_byte_count: 10, // 4*2*10 = 80 bits = 10 bytes — layer-0 closes
        strip_bytes: &[0u8; 10],
        white_level: None,
    }
    .build();
    let container = Container::parse(&odd_bits).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 8];
    let err = unpack_into(&sensor, container.byte_order(), &odd_bits, &mut dst)
        .expect_err("bits_per_sample 10 is not a supported width");
    assert!(
        matches!(err, Error::UnsupportedBitDepth { bits: 10 }),
        "expected UnsupportedBitDepth{{10}}, got {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 / AC2 — bit-exact against the two real, decodable shapes
// ─────────────────────────────────────────────────────────────────────────────

fn unpack_corpus_file(path: &str) -> Option<(irradiance::ifd::Sensor, Vec<u16>, ByteOrder)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let file_path = entry.require(&root)?;
    let bytes = std::fs::read(&file_path).expect("read corpus file");
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("{path}: {e}"));
    sensor
        .require_uncompressed()
        .unwrap_or_else(|e| panic!("{path}: {e}"));
    let pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .unwrap_or_else(|_| panic!("{path}: plane too large for this host"));
    let mut dst = vec![0u16; pixel_count];
    unpack_into(&sensor, container.byte_order(), &bytes, &mut dst)
        .unwrap_or_else(|e| panic!("{path}: {e}"));
    let byte_order = container.byte_order();
    Some((sensor, dst, byte_order))
}

#[test]
fn unpacks_fourteen_bit_msb_first_samples() {
    let Some((sensor, dst, _)) = unpack_corpus_file("LEICA-Q2-MONO/L1021223.DNG") else {
        return; // SKIP already announced by CorpusFile::require
    };
    assert_eq!(sensor.bits_per_sample, 14);
    assert_eq!(
        &dst[..8],
        &[746, 725, 711, 752, 646, 705, 772, 686],
        "measured two independent ways in SPEC-012's Implementation Context"
    );
}

#[test]
fn unpacks_sixteen_bit_in_file_byte_order() {
    let Some((sensor, dst, byte_order)) = unpack_corpus_file("LEICA-M-MONOCHROM/L1000622.DNG")
    else {
        return; // SKIP already announced by CorpusFile::require
    };
    assert_eq!(sensor.bits_per_sample, 16);
    assert_eq!(byte_order, ByteOrder::Little, "L1000622.DNG's header is II");
    assert_eq!(
        &dst[..8],
        &[4761, 4591, 4622, 4363, 4542, 4383, 4608, 4286],
        "measured two independent ways in SPEC-012's Implementation Context"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — the three compressed files are rejected without allocating a plane
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn compressed_files_are_rejected_without_decoding() {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();

    for path in [
        "LEICA-M-MONOCHROM-TYP246/M2462362.DNG", // Compression 7 (JPEG)
        "PENTAX-K3III-MONO/K3III.DNG",           // Compression 7 (JPEG)
        "PENTAX-K3III-MONO/K3III.PEF",           // Compression 65535
    ] {
        let entry = manifest
            .get(path)
            .unwrap_or_else(|| panic!("{path} must be in the manifest"));
        let Some(file_path) = entry.require(&root) else {
            continue; // SKIP already announced by CorpusFile::require
        };
        let bytes = std::fs::read(&file_path).expect("read corpus file");
        let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
        let sensor = container.sensor().unwrap_or_else(|e| panic!("{path}: {e}"));

        // An EMPTY destination: if this ever reached the length check (or
        // beyond) instead of the compression check, it would fail on
        // PlaneBufferWrongLength instead — proving compression is rejected
        // BEFORE any plane-sized buffer would even need to exist.
        let mut dst: [u16; 0] = [];
        let err = unpack_into(&sensor, container.byte_order(), &bytes, &mut dst)
            .expect_err("compressed data must never reach the unpack path");
        assert!(
            matches!(err, Error::UnsupportedCompression { .. }),
            "{path}: expected UnsupportedCompression before any length check, got {err:?}"
        );
    }
}
