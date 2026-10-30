//! `SPEC-012` — strip location and sample unpack, `DEC-008`'s two paths.
//!
//! Two lanes, as `tests/ifd_reader.rs` establishes the pattern:
//!
//! - **Tier A** (`each_path_produces_impossible_values_on_the_others_data`,
//!   `a_sample_above_white_level_unpacks_and_is_preserved` /
//!   `a_plane_within_white_level_is_unchanged` (SPEC-029),
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
use irradiance::ifd::{ByteOrder, Container, TAG_ROWS_PER_STRIP, TAG_WHITE_LEVEL};
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

    // Both wrong-path values (43019, 39186) exceed WhiteLevel 16383. DEC-008's
    // `max <= WhiteLevel` assertion caught exactly this byte-swap; DEC-029
    // retired it (a sample above WhiteLevel is legal — see SPEC-029's tests
    // above), so the guard against this mis-selected path is now the bit-exact
    // oracle (SPEC-025), not an in-unpack assertion.
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-029 — a sample above WhiteLevel is legal (DEC-029 amends DEC-008)
// ─────────────────────────────────────────────────────────────────────────────
//
// DEC-008 pinned `max <= WhiteLevel` as a "permanent invariant" (the free
// assertion that caught SPIKE-002's byte-swap). SPEC-029 reverses that one
// clause: a real sensor records specular highlights above the nominal white
// point, so a sample above WhiteLevel is NOT an error, and the bit-exact plane
// must preserve the raw value — clamping is develop's saturation step, not
// unpack's, and would break SPEC-025's `dnglab --raw-checksum`. The byte-swap
// guard moves to the bit-exact oracle (SPEC-025) + the per-strip layer-0 check.

/// A plane whose max sample is ABOVE WhiteLevel unpacks `Ok`, and the raw
/// above-white value is preserved in `dst` — not clamped to WhiteLevel. The
/// value `15464 > WhiteLevel 15000` is the Canon 5D-III's measured clip.
#[test]
fn a_sample_above_white_level_unpacks_and_is_preserved() {
    const WHITE_LEVEL: u32 = 15000;
    const ABOVE_WHITE: u16 = 15464; // the Canon 5D-III's measured above-white clip
    let samples = [100u16, 4096, ABOVE_WHITE, 200]; // 2 x 2, one sample above white
    let strip = pack_u16(&samples, Order::Little);
    let data = Fixture {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        strip_offset: 512,
        strip_byte_count: u32::try_from(strip.len()).expect("fixture strip fits u32"),
        strip_bytes: &strip,
        white_level: Some(WHITE_LEVEL),
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect("a sample above WhiteLevel is legal (DEC-029), not an error");
    assert_eq!(
        dst, samples,
        "the above-white sample must be preserved raw, not clamped to WhiteLevel"
    );
    assert!(
        u32::from(dst[2]) > WHITE_LEVEL,
        "dst[2] = {} must still exceed WhiteLevel {WHITE_LEVEL} — unchanged, not clamped",
        dst[2]
    );
}

/// Regression: removing the assertion must not touch the within-white path. A
/// plane whose every sample is `<= WhiteLevel` (including one `== WhiteLevel`)
/// unpacks to exactly its samples, byte-identical to before. The corpus half of
/// this regression is the tier-B `unpacks_fourteen_bit_msb_first_samples` (Q2M,
/// 14-bit) and `unpacks_sixteen_bit_in_file_byte_order` (M Monochrom, 16-bit),
/// which must also stay green byte-for-byte.
#[test]
fn a_plane_within_white_level_is_unchanged() {
    const WHITE_LEVEL: u32 = 15000;
    let samples = [100u16, 4096, 14999, 15000]; // all <= WhiteLevel (incl. the boundary)
    let strip = pack_u16(&samples, Order::Little);
    let data = Fixture {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        strip_offset: 512,
        strip_byte_count: u32::try_from(strip.len()).expect("fixture strip fits u32"),
        strip_bytes: &strip,
        white_level: Some(WHITE_LEVEL),
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect("a within-WhiteLevel plane unpacks unchanged");
    assert_eq!(
        dst, samples,
        "within-WhiteLevel samples (including == WhiteLevel) are preserved unchanged"
    );
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

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-028 — multi-strip mosaic plane assembly
// ─────────────────────────────────────────────────────────────────────────────
//
// `unpack_into` assembles N strips (StripOffsets/StripByteCounts + RowsPerStrip)
// into the plane; the single strip stays a degenerate one-iteration case, byte
// for byte unchanged. These live here, beside the SPEC-012 unpack tests, rather
// than in `src/ifd.rs mod tests` (which drives the IFD reader, never
// `unpack_into`): AGENTS.md §12 puts integration tests of a public function in
// `tests/<area>.rs`, and this is that file. The TIFF DSL the plane tests use
// (the `Fixture` builder above, on `tests/support/tiff.rs`) is extended for N
// strips by `MultiStrip` below. A build-note divergence from the spec's literal
// "`src/ifd.rs mod tests`" wording, recorded in HANDOFF-066.

/// A multi-strip (or single-strip) sensor-only TIFF: `IFD0` *is* the plane, with
/// `N` strips planted contiguously from a fixed base. Each strip carries a
/// DECLARED `StripByteCounts` kept separate from its actual payload length, so a
/// fixture can promise bytes the file does not hold (the truncated case) or a
/// wrong count (the mismatch case). `rows_per_strip = None` omits `RowsPerStrip`
/// from the IFD — the "missing RowsPerStrip on a multi-strip plane" case.
struct MultiStrip<'a> {
    order: Order,
    width: u32,
    height: u32,
    bits: u16,
    rows_per_strip: Option<u32>,
    /// One `(declared StripByteCounts, payload bytes)` per strip.
    strips: &'a [(u32, Vec<u8>)],
    white_level: Option<u32>,
}

impl MultiStrip<'_> {
    fn build(&self) -> Vec<u8> {
        // Well clear of the IFD block (which ends well under 200 for this many
        // entries) and of each other, for the small N these fixtures use.
        const OFFSETS_AT: u32 = 200;
        const COUNTS_AT: u32 = 320;
        const PAYLOAD_BASE: usize = 1024;

        // Strip payloads are contiguous from PAYLOAD_BASE, exactly as a real
        // file lays them out; StripOffsets point at each payload's start.
        let mut strip_offsets: Vec<u32> = Vec::new();
        let mut cursor = PAYLOAD_BASE;
        for (_, bytes) in self.strips {
            strip_offsets.push(u32::try_from(cursor).expect("fixture offset fits u32"));
            cursor += bytes.len();
        }
        let strip_counts: Vec<u32> = self.strips.iter().map(|(count, _)| *count).collect();
        let n = u32::try_from(self.strips.len()).expect("fixture strip count fits u32");

        let mut entries = vec![
            tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
            tiff::long(tiff::IMAGE_WIDTH, self.width),
            tiff::long(tiff::IMAGE_LENGTH, self.height),
            tiff::short(tiff::BITS_PER_SAMPLE, self.bits, self.order),
            tiff::short(tiff::COMPRESSION, 1, self.order),
            tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, self.order),
            tiff::short(tiff::SAMPLES_PER_PIXEL, 1, self.order),
        ];
        if let Some(rows) = self.rows_per_strip {
            entries.push(tiff::long(TAG_ROWS_PER_STRIP, rows));
        }
        // TIFF packs a value word of <= 4 bytes inline, so a single strip's
        // LONG offset/count is inline; N >= 2 longs live in an external array.
        if n == 1 {
            entries.push(tiff::long(tiff::STRIP_OFFSETS, strip_offsets[0]));
            entries.push(tiff::long(tiff::STRIP_BYTE_COUNTS, strip_counts[0]));
        } else {
            entries.push(tiff::at_offset(tiff::STRIP_OFFSETS, 4, n, OFFSETS_AT));
            entries.push(tiff::at_offset(tiff::STRIP_BYTE_COUNTS, 4, n, COUNTS_AT));
        }
        if let Some(wl) = self.white_level {
            entries.push(tiff::long(TAG_WHITE_LEVEL, wl));
        }

        let mut data = tiff::tiff(self.order, 8, &[tiff::Ifd::new(8, entries, 0)]);

        // Plant the external arrays (read only when N >= 2) and the payloads.
        if n != 1 {
            put_u32_array(&mut data, OFFSETS_AT as usize, &strip_offsets, self.order);
            put_u32_array(&mut data, COUNTS_AT as usize, &strip_counts, self.order);
        }
        for (offset, (_, bytes)) in strip_offsets.iter().zip(self.strips.iter()) {
            let at = *offset as usize;
            let end = at + bytes.len();
            if data.len() < end {
                data.resize(end, 0);
            }
            data[at..end].copy_from_slice(bytes);
        }
        data
    }
}

/// Write a `u32` array into `data` at `at`, in `order` — resizing to fit.
fn put_u32_array(data: &mut Vec<u8>, at: usize, vals: &[u32], order: Order) {
    let end = at + vals.len() * 4;
    if data.len() < end {
        data.resize(end, 0);
    }
    for (i, v) in vals.iter().enumerate() {
        let o = at + i * 4;
        let bytes = match order {
            Order::Little => v.to_le_bytes(),
            Order::Big => v.to_be_bytes(),
        };
        data[o..o + 4].copy_from_slice(&bytes);
    }
}

/// Pack `samples` MSB-first at `bits` per sample, padding the final byte with
/// zero bits — the exact inverse of [`irradiance::plane`]'s `BitReader`, and the
/// per-strip byte-boundary padding `DEC-008` describes.
fn pack_msb(samples: &[u16], bits: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut acc: u32 = 0;
    let mut nbits: u32 = 0;
    let mask = (1u32 << bits) - 1;
    for &s in samples {
        acc = (acc << bits) | (u32::from(s) & mask);
        nbits += bits;
        while nbits >= 8 {
            nbits -= 8;
            out.push(((acc >> nbits) & 0xFF) as u8);
        }
    }
    if nbits > 0 {
        out.push(((acc << (8 - nbits)) & 0xFF) as u8);
    }
    out
}

/// Pack `samples` as 16-bit integers in `order` — the byte-aligned path's wire
/// form.
fn pack_u16(samples: &[u16], order: Order) -> Vec<u8> {
    let mut out = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let bytes = match order {
            Order::Little => s.to_le_bytes(),
            Order::Big => s.to_be_bytes(),
        };
        out.extend_from_slice(&bytes);
    }
    out
}

/// `pack_msb` must be the inverse of the unpacker: the Q2M head (8 x 14-bit,
/// tightly packed, no padding) round-trips to its measured bytes.
#[test]
fn pack_msb_is_the_inverse_of_the_bitstream_reader() {
    let samples = [746u16, 725, 711, 752, 646, 705, 772, 686];
    assert_eq!(pack_msb(&samples, 14), Q2M_STRIP_HEAD.to_vec());
}

/// AC1 — a 2-strip 16-bit plane unpacks to the same samples as the single-strip
/// equivalent (byte-aligned, per-strip assembly).
#[test]
fn a_two_strip_16bit_plane_unpacks_like_one_strip() {
    // 2 x 2, 16-bit; one row per strip.
    let samples = [1000u16, 2000, 3000, 4000];

    let two = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        rows_per_strip: Some(1),
        strips: &[
            (4, pack_u16(&samples[0..2], Order::Little)),
            (4, pack_u16(&samples[2..4], Order::Little)),
        ],
        white_level: None,
    }
    .build();
    let container = Container::parse(&two).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    unpack_into(&sensor, container.byte_order(), &two, &mut dst).expect("two-strip unpack");

    // The single-strip equivalent: the same four samples as one 8-byte strip.
    let one = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        rows_per_strip: None, // a single strip is the whole plane
        strips: &[(8, pack_u16(&samples, Order::Little))],
        white_level: None,
    }
    .build();
    let container = Container::parse(&one).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut one_dst = [0u16; 4];
    unpack_into(&sensor, container.byte_order(), &one, &mut one_dst).expect("one-strip unpack");

    assert_eq!(
        dst, samples,
        "two-strip assembly must reproduce the samples"
    );
    assert_eq!(
        dst, one_dst,
        "two strips must assemble to exactly the single-strip plane"
    );
}

/// AC2 — a 2-strip 14-bit plane: each strip is byte-padded INDEPENDENTLY, so
/// strip 2's first sample is not shifted by strip 1's tail padding. With 2 x 2
/// samples per strip, each strip is 28 bits = 4 bytes (4 padding bits); naive
/// concatenation would read sample 2 across strip 1's padding.
#[test]
fn a_two_strip_14bit_plane_respects_per_strip_byte_padding() {
    let samples = [100u16, 200, 300, 400];
    let strip0 = pack_msb(&samples[0..2], 14); // 28 bits -> 4 bytes, 4 pad bits
    let strip1 = pack_msb(&samples[2..4], 14);
    assert_eq!(strip0.len(), 4, "a padded 14-bit strip is 4 bytes, not 3.5");

    let data = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 14,
        rows_per_strip: Some(1),
        strips: &[(4, strip0), (4, strip1)],
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    unpack_into(&sensor, container.byte_order(), &data, &mut dst).expect("per-strip 14-bit unpack");
    assert_eq!(
        dst, samples,
        "strip 2 must start at its own byte offset, not across strip 1's padding"
    );
}

/// AC3 — an uneven last strip (`height % rows_per_strip != 0`) unpacks the right
/// remainder. 3 rows, 2 rows per strip: strip 0 is rows 0-1, strip 1 is row 2.
#[test]
fn an_uneven_last_strip_unpacks_the_remainder() {
    let samples = [11u16, 12, 21, 22, 31, 32]; // 3 rows x 2 cols
    let data = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 3,
        bits: 16,
        rows_per_strip: Some(2),
        strips: &[
            (8, pack_u16(&samples[0..4], Order::Little)), // rows 0-1
            (4, pack_u16(&samples[4..6], Order::Little)), // row 2 (remainder)
        ],
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 6];
    unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect("uneven-last-strip unpack");
    assert_eq!(
        dst, samples,
        "the short last strip must fill exactly its remainder rows"
    );
}

/// AC4 (regression) — a single strip stays the SPEC-012 path, byte for byte.
/// The real-file half of this regression is the tier-B
/// `unpacks_fourteen_bit_msb_first_samples` / `unpacks_sixteen_bit_in_file_byte_order`
/// (Q2M 14-bit, M Monochrom 16-bit), which must also stay green.
#[test]
fn a_single_strip_plane_is_unchanged() {
    let samples = [5u16, 6, 7, 8, 9, 10, 11, 12]; // 4 x 2, 16-bit
    let data = MultiStrip {
        order: Order::Little,
        width: 4,
        height: 2,
        bits: 16,
        rows_per_strip: None,
        strips: &[(16, pack_u16(&samples, Order::Little))],
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 8];
    unpack_into(&sensor, container.byte_order(), &data, &mut dst).expect("single-strip unpack");
    assert_eq!(dst, samples, "the single-strip path must be unchanged");
}

/// AC5 — a multi-strip plane with no `RowsPerStrip` is a typed error, not a
/// panic: without it the per-strip row partition is undefined.
#[test]
fn a_multi_strip_plane_without_rows_per_strip_is_a_typed_error() {
    let samples = [1u16, 2, 3, 4];
    let data = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        rows_per_strip: None, // omitted on a 2-strip plane: the error under test
        strips: &[
            (4, pack_u16(&samples[0..2], Order::Little)),
            (4, pack_u16(&samples[2..4], Order::Little)),
        ],
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    let err = unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect_err("a 2-strip plane without RowsPerStrip has no row partition");
    assert!(
        matches!(err, Error::MissingRowsPerStrip { strips: 2 }),
        "expected MissingRowsPerStrip{{2}}, got {err:?}"
    );
}

/// AC6a — a strip whose declared bytes run past EOF is `Truncated`, not a panic.
/// Strip 1's declared count is correct for the layer-0 check (so the read is
/// reached), but the file does not actually hold its bytes.
#[test]
fn a_truncated_strip_is_truncated_not_a_panic() {
    let samples = [1u16, 2, 3, 4];
    let data = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        rows_per_strip: Some(1),
        strips: &[
            (4, pack_u16(&samples[0..2], Order::Little)), // present
            (4, Vec::new()),                              // declares 4 bytes the file does not hold
        ],
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    let err = unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect_err("strip 1 promises bytes past the end of the file");
    assert!(
        matches!(err, Error::Truncated { .. }),
        "expected Truncated, got {err:?}"
    );
}

/// AC6b — a strip whose declared byte count is not its rows' packed size is
/// `PackedSizeMismatch`, not a panic. Strip 0 declares 6 bytes where 2 x 16-bit
/// rows need 4.
#[test]
fn a_strip_byte_count_mismatch_is_packed_size_mismatch() {
    let samples = [1u16, 2, 3, 4];
    let data = MultiStrip {
        order: Order::Little,
        width: 2,
        height: 2,
        bits: 16,
        rows_per_strip: Some(1),
        strips: &[
            (6, vec![0u8; 6]), // wrong: 1 row x 2 x 16 = 32 bits = 4 bytes, not 6
            (4, pack_u16(&samples[2..4], Order::Little)),
        ],
        white_level: None,
    }
    .build();
    let container = Container::parse(&data).expect("parses");
    let sensor = container.sensor().expect("has a sensor plane");
    let mut dst = [0u16; 4];
    let err = unpack_into(&sensor, container.byte_order(), &data, &mut dst)
        .expect_err("strip 0's declared byte count does not match its rows");
    assert!(
        matches!(
            err,
            Error::PackedSizeMismatch {
                expected_bits: 32,
                strip_bits: 48,
            }
        ),
        "expected PackedSizeMismatch{{32, 48}}, got {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-028 (corpus) — the Nikon D300S Bayer plane is 12 strips
// ─────────────────────────────────────────────────────────────────────────────
//
// Tier B. The file is NOT yet in `tests/corpus/manifest.toml` — registering it
// (with its sha256 and the `dnglab` raw-checksum oracle) is SPEC-027 — so this
// resolves it through `CorpusRoot` (never a hardcoded path) and skips loudly
// when the corpus is absent. SPEC-028's deliverable is narrow: assembly runs,
// so `UnsupportedStripLayout` is GONE. SPEC-029 then retired the
// `max <= WhiteLevel` assertion, so the assembled plane now unpacks FULLY — the
// Nikon carries a sample above WhiteLevel (3835 > 3808) that used to trip the
// (now-removed) assertion after assembly, and is legal.

#[test]
fn the_nikon_d300s_twelve_strip_plane_no_longer_rejects_the_strip_layout() {
    const NIKON: &str = "NIKON-D300S/_DSC4615.DNG";
    let root = CorpusRoot::resolve();
    let path = root.path.join(NIKON);
    if !root.from_env || !path.exists() {
        eprintln!("SKIP {NIKON}: set $IRRADIANCE_CORPUS_DIR to the corpus (SPEC-027 registers it)");
        return;
    }

    let bytes = std::fs::read(&path).expect("read the Nikon corpus file");
    let container = Container::parse(&bytes).expect("Nikon DNG parses");
    let sensor = container.sensor().expect("Nikon has a sensor plane");
    assert!(
        sensor.strip_offsets.len() >= 2,
        "the Nikon D300S plane is stored in multiple strips (measured: 12)"
    );
    let pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .expect("Nikon plane fits this host");
    let mut plane = vec![0u16; pixel_count];
    let result = unpack_into(&sensor, container.byte_order(), &bytes, &mut plane);

    // SPEC-028's deliverable: assembly runs, so the multi-strip layout is no
    // longer rejected. SPEC-029's: the assembled plane now unpacks FULLY — the
    // above-white sample (3835 > WhiteLevel 3808) that used to trip the retired
    // assertion is legal and preserved. Both are proven by `Ok(())` over the
    // whole width*height plane with no length, layout, or WhiteLevel error.
    assert!(
        !matches!(result, Err(Error::UnsupportedStripLayout { .. })),
        "SPEC-028: the 12-strip plane must assemble, not return UnsupportedStripLayout"
    );
    result.expect(
        "SPEC-029: the assembled 12-strip plane unpacks fully; its above-white \
         sample (3835 > WhiteLevel 3808) is legal, not an error",
    );
    let max = plane.iter().copied().max().unwrap_or(0);
    assert!(
        max > 3808,
        "the Nikon plane carries a sample above WhiteLevel 3808 (measured max 3835), preserved"
    );
    eprintln!("NIKON {NIKON}: assembled and fully unpacked (above-white max {max} preserved)");
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-029 (corpus) — the Canon 5D-III single-strip Bayer plane clips above white
// ─────────────────────────────────────────────────────────────────────────────
//
// Tier B, the companion of the Nikon check above and SPEC-029's own corpus AC.
// The Canon is a SINGLE strip (unlike the Nikon's 12), 16-bit, with WhiteLevel
// 15000 and a sample 15464 above it — the real falsifier for DEC-008's
// "permanent invariant". Before SPEC-029 this tripped `SampleExceedsWhiteLevel`
// (sample 2016443 = 15464); now it unpacks fully with the raw value preserved.
// NOT yet in the manifest (SPEC-027 registers it), so resolved via `CorpusRoot`
// and skipped loudly when the corpus is absent.

#[test]
fn the_canon_5d_iii_plane_unpacks_with_its_above_white_sample_preserved() {
    const CANON: &str = "CANON-EOS-5D-MARK-III/5G4A9394.DNG";
    const WHITE_LEVEL: u32 = 15000;
    let root = CorpusRoot::resolve();
    let path = root.path.join(CANON);
    if !root.from_env || !path.exists() {
        eprintln!("SKIP {CANON}: set $IRRADIANCE_CORPUS_DIR to the corpus (SPEC-027 registers it)");
        return;
    }

    let bytes = std::fs::read(&path).expect("read the Canon corpus file");
    let container = Container::parse(&bytes).expect("Canon DNG parses");
    let sensor = container.sensor().expect("Canon has a sensor plane");
    assert_eq!(
        sensor.white_level,
        Some(WHITE_LEVEL),
        "the Canon 5D-III declares WhiteLevel 15000"
    );
    let pixel_count = usize::try_from(u64::from(sensor.width) * u64::from(sensor.height))
        .expect("Canon plane fits this host");
    let mut plane = vec![0u16; pixel_count];

    // SPEC-029: a sample above WhiteLevel is legal. Before this spec the unpack
    // returned `SampleExceedsWhiteLevel`; now it is `Ok` with the raw value kept.
    unpack_into(&sensor, container.byte_order(), &bytes, &mut plane)
        .expect("SPEC-029: the Canon's above-white sample (15464 > WhiteLevel 15000) is legal");
    let max = plane.iter().copied().max().unwrap_or(0);
    assert!(
        u32::from(max) > WHITE_LEVEL,
        "the Canon plane's max sample must exceed WhiteLevel {WHITE_LEVEL} (measured 15464), \
         preserved raw — found {max}"
    );
    eprintln!("CANON {CANON}: unpacked with above-white max {max} > WhiteLevel {WHITE_LEVEL}");
}
