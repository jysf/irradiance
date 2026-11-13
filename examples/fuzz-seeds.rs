//! Write the fuzz seed corpus for `fuzz/fuzz_targets/ifd.rs`,
//! `fuzz/fuzz_targets/plane.rs`, `fuzz/fuzz_targets/develop.rs`,
//! `fuzz/fuzz_targets/warp_opcode.rs`, `fuzz/fuzz_targets/opcode.rs`
//! (`SPEC-017`'s own target — `OpcodeList1`'s `FixBadPixelsConstant`, real
//! bytes plus adversarial variants from `tests/support/opcode.rs::
//! fix_bad_pixels_fuzz_seed_corpus`), `fuzz/fuzz_targets/demosaic.rs`
//! (`SPEC-032`'s own target — the Bayer demosaic, seeded in [`demosaic_seeds`]
//! from `SPEC-027`'s CFA builders), and `fuzz/fuzz_targets/ljpeg.rs`
//! (`SPEC-038`'s own target — the single-component SOF-3 lossless-JPEG decoder,
//! seeded in [`ljpeg_seeds`] from a local LJ92 builder mirroring the oracle's
//! `encode_lj92`).
//!
//! `SPEC-003`, extended by `SPEC-012`, `SPEC-014` and `SPEC-052` (the
//! `aliased-ifd-tables` seed for `DEC-046`'s aggregate table-bytes guard,
//! added to the shared `tiff::all()` list). The `ifd` seeds are the
//! hand-built tier-A fixtures in `tests/support/tiff.rs` — the same list
//! `tests/ifd_reader.rs` asserts against, so a fixture cannot be added to the
//! test lane and forgotten by the fuzz lane. The `plane` seeds are built
//! locally in [`plane_seeds`] from the strip bytes `tests/plane_unpack.rs`
//! also uses — `DEC-008`/`SPEC-012`'s measured `## Implementation Context`,
//! not any corpus file. The `develop` seeds are built locally in
//! [`develop_seeds`], covering the geometry surface `SPEC-014` `AC6` names
//! (an out-of-range crop, an inverted `ActiveArea`, `Orientation` outside
//! `1..=8`, `BlackLevel >= WhiteLevel`) plus the two shapes that develop
//! successfully. Everything here is **own work built from TIFF 6.0 §2**:
//! tier B is never committed (`DEC-003`), and a truncated 86 MB Leica frame
//! would still be a Leica frame.
//!
//! ```text
//! cargo run --example fuzz-seeds     # rewrite every fuzz/seeds/<target>/
//! ```
//!
//! The output directories are committed, and `fuzz/corpus/*` — where
//! libFuzzer writes what it discovers — is not. Run each target against both:
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
//!     fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run plane \
//!     fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=60
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run develop \
//!     fuzz/corpus/develop fuzz/seeds/develop -- -max_total_time=60
//! ```

#[path = "../tests/support/dng_writer.rs"]
mod dng_writer;
#[path = "../tests/support/opcode.rs"]
mod opcode;
#[path = "../tests/support/tiff.rs"]
mod tiff;

use std::path::PathBuf;

/// A minimal sensor-only TIFF (IFD0 *is* the sensor plane) with a strip
/// payload planted at `strip_offset`. Mirrors `tests/plane_unpack.rs`'s
/// `Fixture::build`, which cannot be imported here (it is test-only code in a
/// different compilation unit) — kept in sync by hand; both are exercised by
/// `just test` (the test) and `just fuzz-plane` (the seed), so drift shows up
/// as a fuzz crash or a test failure, not silently.
fn plane_fixture(
    order: tiff::Order,
    width: u32,
    height: u32,
    bits: u16,
    strip_offset: u32,
    strip_byte_count: u32,
    strip_bytes: &[u8],
) -> Vec<u8> {
    let entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, width),
        tiff::long(tiff::IMAGE_LENGTH, height),
        tiff::short(tiff::BITS_PER_SAMPLE, bits, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        tiff::long(tiff::STRIP_OFFSETS, strip_offset),
        tiff::long(tiff::STRIP_BYTE_COUNTS, strip_byte_count),
    ];
    let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
    let at = strip_offset as usize;
    let end = at + strip_bytes.len();
    if data.len() < end {
        data.resize(end, 0);
    }
    data[at..end].copy_from_slice(strip_bytes);
    data
}

/// A VALID synthetic DNG `Compression = 7` plane (`SPEC-045`): own-work SOF-3
/// (`tests/support/tiff.rs::compression7_dng`, predictor 1, 14-bit,
/// little-endian), so the plane / develop / demosaic targets start from a file
/// that decodes through the PUBLIC `unpack_into` -> `ljpeg` route and mutate
/// from there. Not derived from any corpus file (`DEC-003`).
fn compression7_seed() -> Vec<u8> {
    let (width, height, precision) = (8u32, 4u32, 14u8);
    let samples: Vec<u16> = (0..width * height)
        .map(|i| ((i * 977 + 4000) % (1 << precision)) as u16)
        .collect();
    tiff::compression7_dng(tiff::Order::Little, width, height, precision, &samples)
}

/// `SPEC-058` (`DEC-056`): TILED sensor planes from the SPEC-057 writer, so
/// every plane-reaching target starts inside the tile path. A 13 x 9 Bayer
/// plane in 8 x 4 tiles (2 x 3 = 6 tiles: partial right AND bottom tiles, an
/// odd visible width of 5), each tile coded at full size with a `0xBEEF`-style
/// padding sentinel: uncompressed at 12 and 16 bits and Compression-7 at
/// `Nf=1` and `Nf=2`, in both byte orders, plus one hostile table (a tile
/// count one short of the grid). Own work, no corpus file (`DEC-003`).
fn tiled_seeds() -> Vec<(&'static str, Vec<u8>)> {
    use dng_writer::{
        write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag,
        TagIfd, TAG_TILE_BYTE_COUNTS,
    };
    let (width, height, tile_width, tile_length) = (13u32, 9u32, 8u32, 4u32);
    let plane = |bits: u32| -> Vec<u16> {
        (0..width * height)
            .map(|i| ((i.wrapping_mul(2_654_435_761) >> 7) % (1 << bits)) as u16)
            .collect()
    };
    let lj92 = |samples: &[u16], precision: u8, nf: u8| -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        for ty in 0..height.div_ceil(tile_length) {
            for tx in 0..width.div_ceil(tile_width) {
                let mut tile = Vec::new();
                for r in 0..tile_length {
                    for c in 0..tile_width {
                        let (x, y) = (tx * tile_width + c, ty * tile_length + r);
                        tile.push(if x < width && y < height {
                            samples[(y * width + x) as usize]
                        } else {
                            0x0EEF
                        });
                    }
                }
                chunks.push(tiff::lj92_strip_components(
                    &tile,
                    tile_width as usize,
                    tile_length as usize,
                    precision,
                    nf,
                    0x11,
                ));
            }
        }
        chunks
    };
    let describe = |order: tiff::Order, bits: u32, payload: Payload| DngDescription {
        order,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width,
        height,
        bits_per_sample: bits,
        kind: PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        },
        organisation: Organisation::Tiles {
            tile_width,
            tile_length,
            edge_fill: 0x0EEF,
        },
        payload,
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    };
    let write = |d: &DngDescription| write_dng(d).expect("tiled seed description is valid");

    let mut hostile = describe(tiff::Order::Little, 16, Payload::Samples(plane(16)));
    hostile.extra_tags = vec![RawTag {
        ifd: TagIfd::Sensor,
        tag: TAG_TILE_BYTE_COUNTS,
        field_type: 4,
        count: 5,
        bytes: [64u32 * 4 * 2; 5]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect(),
    }];

    vec![
        (
            "tiled-uncompressed-16-ii",
            write(&describe(
                tiff::Order::Little,
                16,
                Payload::Samples(plane(16)),
            )),
        ),
        (
            "tiled-uncompressed-12-mm",
            write(&describe(tiff::Order::Big, 12, Payload::Samples(plane(12)))),
        ),
        (
            "tiled-lj92-nf1-ii",
            write(&describe(
                tiff::Order::Little,
                14,
                Payload::Encoded {
                    compression: 7,
                    chunks: lj92(&plane(14), 14, 1),
                },
            )),
        ),
        (
            "tiled-lj92-nf2-mm",
            write(&describe(
                tiff::Order::Big,
                16,
                Payload::Encoded {
                    compression: 7,
                    chunks: lj92(&plane(16), 16, 2),
                },
            )),
        ),
        ("tiled-count-mismatch", write(&hostile)),
    ]
}

/// Seeds for `fuzz/fuzz_targets/plane.rs` — reaching BOTH of `DEC-008`'s
/// paths is the point (a target that only ever drives one recreates
/// `SPIKE-001`'s exact blind spot), plus the hostile shapes `SPEC-012`
/// `AC7` names.
fn plane_seeds() -> Vec<(&'static str, Vec<u8>)> {
    // L1021223.DNG's strip head (DEC-008/SPEC-012 Implementation Context).
    const Q2M_STRIP: [u8; 14] = [
        0x0b, 0xa8, 0x2d, 0x50, 0xb1, 0xc2, 0xf0, 0x0a, 0x18, 0x2c, 0x10, 0xc1, 0x02, 0xae,
    ];
    // L1000622.DNG's strip head.
    const M_MONO_STRIP: [u8; 16] = [
        0x99, 0x12, 0xef, 0x11, 0x0e, 0x12, 0x0b, 0x11, 0xbe, 0x11, 0x1f, 0x11, 0x00, 0x12, 0xbe,
        0x10,
    ];

    let compressed = {
        let entries = vec![
            tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
            tiff::long(tiff::IMAGE_WIDTH, 4),
            tiff::long(tiff::IMAGE_LENGTH, 2),
            tiff::short(tiff::BITS_PER_SAMPLE, 14, tiff::Order::Little),
            tiff::short(tiff::COMPRESSION, 7, tiff::Order::Little),
            tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, tiff::Order::Little),
            tiff::short(tiff::SAMPLES_PER_PIXEL, 1, tiff::Order::Little),
            tiff::long(tiff::STRIP_OFFSETS, 512),
            tiff::long(tiff::STRIP_BYTE_COUNTS, 14),
        ];
        tiff::tiff(tiff::Order::Little, 8, &[tiff::Ifd::new(8, entries, 0)])
    };

    let mut seeds = vec![
        (
            // The sub-byte path (bits % 8 != 0).
            "valid-fourteen-bit",
            plane_fixture(tiff::Order::Little, 4, 2, 14, 512, 14, &Q2M_STRIP),
        ),
        (
            // The byte-aligned path (bits % 8 == 0).
            "valid-sixteen-bit",
            plane_fixture(tiff::Order::Little, 8, 1, 16, 512, 16, &M_MONO_STRIP),
        ),
        (
            // AC3/AC4: the 14-bit strip misdeclared as 16-bit-aligned —
            // decodes without error unless something checks the values.
            "wrong-bits-as-sixteen",
            plane_fixture(tiff::Order::Little, 7, 1, 16, 512, 14, &Q2M_STRIP),
        ),
        (
            // AC7: StripByteCounts promises bytes the file does not hold.
            "truncated-strip",
            plane_fixture(tiff::Order::Little, 4, 2, 14, 512, 14, &[]),
        ),
        (
            // AC7: bits_per_sample outside {8, 12, 14, 16}.
            "odd-bit-depth",
            plane_fixture(tiff::Order::Little, 4, 2, 10, 512, 10, &[0u8; 10]),
        ),
        (
            // SPEC-012 AC6 named this as the Compression-7 gate seed. Since
            // SPEC-043 the plane reaches the SOF-3 decoder, and its strip (14
            // zero bytes, no SOI) is a typed decode error — still a useful
            // hostile shape.
            "compressed-plane",
            compressed,
        ),
        (
            // SPEC-045: a VALID synthetic Compression-7 DNG (own-work SOF-3,
            // predictor 1, 14-bit, little-endian) so the fuzzer starts from a
            // plane that decodes through the PUBLIC route and mutates from
            // there. Reused by the develop and demosaic targets (a mono plane
            // is a rejected CFA there, after a full Compression-7 unpack).
            "compression7-sof3",
            compression7_seed(),
        ),
        (
            // SPEC-053 (DEC-047): a Compression-7 DNG declaring 6 x 4 whose
            // SOF-3 frame is the equal-area transpose (4 wide x 6 lines). The
            // pre-decode shape binding refuses it with LjpegPlaneShapeMismatch
            // where it used to decode to a mis-strided Ok.
            "compression7-transposed-frame",
            {
                let samples: Vec<u16> = (0..24u32).map(|i| (i * 131 % 4096) as u16).collect();
                let strip = tiff::lj92_strip(&samples, 4, 6, 12);
                tiff::compression7_dng_with_strip(tiff::Order::Little, 6, 4, 12, &strip)
            },
        ),
        (
            // SPEC-039 (DEC-050): a forged 65535 x 65535 uncompressed plane
            // over a 14-byte strip. `validate` refuses it with PlaneTooLarge
            // under the harness budget, before the harness allocates.
            "forged-dims-uncompressed",
            plane_fixture(tiff::Order::Little, 65_535, 65_535, 14, 512, 14, &Q2M_STRIP),
        ),
        (
            // SPEC-039: the same forgery on the Compression-7 path, where the
            // file-bytes bound does not apply and only the budget bounds n.
            "forged-dims-compression7",
            {
                let samples: Vec<u16> = (0..4u32).map(|i| (i * 977 % 4096) as u16).collect();
                let strip = tiff::lj92_strip(&samples, 2, 2, 12);
                tiff::compression7_dng_with_strip(tiff::Order::Little, 65_535, 65_535, 12, &strip)
            },
        ),
        (
            // SPEC-039: sixteen one-row strips aliasing the same 64 bytes —
            // each strip in bounds, together claiming more than the file.
            // `validate` refuses it with StripBytesExceedFile.
            "aliased-strips",
            aliased_strips_seed(),
        ),
    ];
    // SPEC-058: the tile path.
    seeds.extend(tiled_seeds());
    seeds
}

/// Sixteen one-row 16-bit strips of a 32-wide plane, every `StripOffsets`
/// value pointing at the same 64 payload bytes (`SPEC-039`): 1024 bytes
/// declared in a 576-byte file. The strip arrays live at 256 and 320.
fn aliased_strips_seed() -> Vec<u8> {
    const STRIPS: u32 = 16;
    const WIDTH: u32 = 32;
    const PAYLOAD_AT: u32 = 512;
    let order = tiff::Order::Little;
    let strip_bytes = WIDTH * 2;
    let mut entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, WIDTH),
        tiff::long(tiff::IMAGE_LENGTH, STRIPS),
        tiff::short(tiff::BITS_PER_SAMPLE, 16, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        tiff::at_offset(tiff::STRIP_OFFSETS, 4, STRIPS, 256),
        tiff::long(TAG_ROWS_PER_STRIP, 1),
        tiff::at_offset(tiff::STRIP_BYTE_COUNTS, 4, STRIPS, 320),
    ];
    entries.sort_by_key(|e| e.tag);
    let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);
    data.resize((PAYLOAD_AT + strip_bytes) as usize, 0);
    for i in 0..STRIPS as usize {
        data[256 + 4 * i..260 + 4 * i].copy_from_slice(&PAYLOAD_AT.to_le_bytes());
        data[320 + 4 * i..324 + 4 * i].copy_from_slice(&strip_bytes.to_le_bytes());
    }
    for (i, b) in data[PAYLOAD_AT as usize..].iter_mut().enumerate() {
        *b = (i * 37) as u8;
    }
    data
}

/// `RowsPerStrip` (TIFF 6.0), restated as a raw number like the tags below.
const TAG_ROWS_PER_STRIP: u16 = 278;

// ─────────────────────────────────────────────────────────────────────────────
// Seeds for `fuzz/fuzz_targets/develop.rs` (SPEC-014)
// ─────────────────────────────────────────────────────────────────────────────

/// DNG tags `tests/support/tiff.rs` does not name — `ActiveArea`,
/// `DefaultCropOrigin`/`Size`, `Orientation`, `BlackLevel`/`WhiteLevel`
/// (`irradiance::ifd::TAG_*`, restated here as raw numbers so this file does
/// not need to depend on the library crate for constants alone).
const TAG_ORIENTATION: u16 = 274;
const TAG_BLACK_LEVEL: u16 = 50714;
const TAG_WHITE_LEVEL: u16 = 50717;
const TAG_DEFAULT_CROP_ORIGIN: u16 = 50719;
const TAG_DEFAULT_CROP_SIZE: u16 = 50720;
const TAG_ACTIVE_AREA: u16 = 50829;

/// A single-strip sensor IFD (byte-aligned, 16-bit — the simplest path,
/// since this seed set is exercising GEOMETRY, not `DEC-008`'s bit-packing)
/// with an optional geometry/levels tag set, each array written as an
/// explicit-offset LONG array after the strip — the same technique
/// `tests/support/tiff.rs`'s `rational_default_crop_size` uses for a
/// multi-value tag that cannot fit inline.
#[allow(clippy::too_many_arguments)]
fn develop_fixture(
    width: u32,
    height: u32,
    active_area: Option<[u32; 4]>,
    crop_origin: Option<[u32; 2]>,
    crop_size: Option<[u32; 2]>,
    orientation: Option<u32>,
    black_level: Option<u32>,
    white_level: Option<u32>,
) -> Vec<u8> {
    let order = tiff::Order::Little;
    let strip_offset: u32 = 512;
    let pixel_count = usize::try_from(width).unwrap_or(0) * usize::try_from(height).unwrap_or(0);
    let strip_bytes: Vec<u8> = (0..pixel_count * 2).map(|i| (i % 256) as u8).collect();
    let strip_byte_count = u32::try_from(strip_bytes.len()).unwrap_or(0);

    let mut entries = vec![
        tiff::long(tiff::NEW_SUBFILE_TYPE, 0),
        tiff::long(tiff::IMAGE_WIDTH, width),
        tiff::long(tiff::IMAGE_LENGTH, height),
        tiff::short(tiff::BITS_PER_SAMPLE, 16, order),
        tiff::short(tiff::COMPRESSION, 1, order),
        tiff::short(tiff::PHOTOMETRIC, tiff::LINEAR_RAW, order),
        tiff::short(tiff::SAMPLES_PER_PIXEL, 1, order),
        tiff::long(tiff::STRIP_OFFSETS, strip_offset),
        tiff::long(tiff::STRIP_BYTE_COUNTS, strip_byte_count),
    ];
    if let Some(v) = orientation {
        entries.push(tiff::short(
            TAG_ORIENTATION,
            u16::try_from(v).unwrap_or(u16::MAX),
            order,
        ));
    }
    if let Some(v) = black_level {
        entries.push(tiff::long(TAG_BLACK_LEVEL, v));
    }
    if let Some(v) = white_level {
        entries.push(tiff::long(TAG_WHITE_LEVEL, v));
    }

    // Arrays go after the strip; each reserves 16 bytes (4 LONGs) whether or
    // not it uses all of them, so offsets stay easy to compute.
    let mut next_array_offset = strip_offset as usize + strip_bytes.len();
    // Align up to a 4-byte boundary — TIFF offsets need not be aligned, but
    // keeping them so makes this file easier to read.
    next_array_offset = next_array_offset.div_ceil(4) * 4;

    let mut arrays: Vec<(u16, u32, Vec<u32>)> = Vec::new(); // (tag, offset, values)
    if let Some(values) = active_area {
        arrays.push((TAG_ACTIVE_AREA, next_array_offset as u32, values.to_vec()));
        entries.push(tiff::at_offset(
            TAG_ACTIVE_AREA,
            4, // LONG
            4,
            next_array_offset as u32,
        ));
        next_array_offset += 16;
    }
    if let Some(values) = crop_origin {
        arrays.push((
            TAG_DEFAULT_CROP_ORIGIN,
            next_array_offset as u32,
            values.to_vec(),
        ));
        entries.push(tiff::at_offset(
            TAG_DEFAULT_CROP_ORIGIN,
            4,
            2,
            next_array_offset as u32,
        ));
        next_array_offset += 8;
    }
    if let Some(values) = crop_size {
        arrays.push((
            TAG_DEFAULT_CROP_SIZE,
            next_array_offset as u32,
            values.to_vec(),
        ));
        entries.push(tiff::at_offset(
            TAG_DEFAULT_CROP_SIZE,
            4,
            2,
            next_array_offset as u32,
        ));
        // No further array follows `DefaultCropSize` in this fixture builder,
        // so `next_array_offset` need not advance again.
    }

    let mut data = tiff::tiff(order, 8, &[tiff::Ifd::new(8, entries, 0)]);

    let strip_at = strip_offset as usize;
    let strip_end = strip_at + strip_bytes.len();
    if data.len() < strip_end {
        data.resize(strip_end, 0);
    }
    data[strip_at..strip_end].copy_from_slice(&strip_bytes);

    for (_tag, offset, values) in arrays {
        let at = offset as usize;
        let end = at + values.len() * 4;
        if data.len() < end {
            data.resize(end, 0);
        }
        for (i, v) in values.iter().enumerate() {
            let slot = at + i * 4;
            data[slot..slot + 4].copy_from_slice(&v.to_le_bytes());
        }
    }

    data
}

/// Seeds for `fuzz/fuzz_targets/develop.rs` — reaching every geometry
/// rejection `SPEC-014` `AC6` names, plus the two shapes that actually
/// develop successfully (unrotated and rotated), so the target exercises the
/// happy path as well as the hostile one.
fn develop_seeds() -> Vec<(&'static str, Vec<u8>)> {
    let mut seeds = vec![
        (
            "unrotated-full-plane",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([1, 1]),
                Some([6, 4]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            "rotated-six",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([1, 1]),
                Some([6, 4]),
                Some(6),
                None,
                None,
            ),
        ),
        (
            // AC4: a NON-ZERO ActiveArea origin — the shape no decodable
            // corpus file carries.
            "nonzero-active-area-origin",
            develop_fixture(
                8,
                6,
                Some([2, 3, 6, 8]),
                Some([1, 1]),
                Some([3, 2]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            "absent-geometry-tags",
            develop_fixture(8, 6, None, None, None, None, None, None),
        ),
        (
            // AC6: DefaultCropSize larger than ActiveArea.
            "crop-exceeds-active-area",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([0, 0]),
                Some([100, 100]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            // AC6: crop origin leaves no room for its size.
            "crop-origin-out-of-plane",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([50, 50]),
                Some([2, 2]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            // AC6: zero-width crop.
            "zero-size-crop",
            develop_fixture(
                8,
                6,
                Some([0, 0, 6, 8]),
                Some([0, 0]),
                Some([0, 4]),
                Some(1),
                None,
                None,
            ),
        ),
        (
            // AC6: inverted ActiveArea (bottom < top).
            "inverted-active-area",
            develop_fixture(8, 6, Some([5, 0, 2, 4]), None, None, Some(1), None, None),
        ),
        (
            // AC6: Orientation outside 1..=8.
            "orientation-out-of-range",
            develop_fixture(8, 6, None, None, None, Some(9), None, None),
        ),
        (
            // AC6: BlackLevel >= WhiteLevel. `FU-2`: the plane's max sample
            // is 24,414 (this fixture's fixed `(i % 256)` byte pattern over
            // 8x6 pixels) — both levels must sit above that, or
            // `plane::unpack_into` rejects the sample as exceeding
            // `WhiteLevel` before `develop_into` is ever reached, and the
            // seed never exercises the branch it names.
            "black-level-at-white-level",
            develop_fixture(8, 6, None, None, None, None, Some(30_000), Some(30_000)),
        ),
    ];
    // SPEC-063: the options surface. Each fixture once per trailing option
    // byte (the target's last-byte convention).
    seeds.extend(develop_option_seeds());
    // SPEC-045: a valid Compression-7 plane, so develop is fuzzed past the
    // public SOF-3 unpack too.
    seeds.push(("compression7-sof3", compression7_seed()));
    // SPEC-058: tiled planes, so develop is fuzzed past the tile unpack.
    seeds.extend(tiled_seeds());
    seeds
}

/// `SPEC-063` seeds for the `develop` target's options byte: an 8×6 plane
/// with an `ActiveArea` (rows 1..6, cols 1..7) and a 4×3 `DefaultCrop` inside
/// it at Orientation 6, without and with a non-identity `WarpRectilinear`
/// `OpcodeList3` (whose scale stays in `[0.9, 1]`, so since `SPEC-064` the
/// `FullActiveArea` bounds rule admits it), each ending in each of the four
/// option bytes.
///
/// `SPEC-064` (`DEC-059`) adds two more warps, each ending in the option
/// bytes for `FullActiveArea` at `warp_alpha` 1, 0 and 0.5 (`1`, `5`, `9`;
/// the target's header has the bit layout): the Pixel 3a's real over-unity
/// polynomial (refused by the bounds rule at α > 0) and Q2M `L1021223`'s
/// (admitted). Their coefficients are numbers in source, read by the
/// `SPEC-064` design probe; no corpus byte is used. Built with
/// `tests/support/dng_writer.rs`; no corpus file is used (`DEC-003`).
fn develop_option_seeds() -> Vec<(&'static str, Vec<u8>)> {
    let (width, height) = (8u32, 6u32);
    let describe = |warp: Option<Vec<u8>>| dng_writer::DngDescription {
        order: tiff::Order::Little,
        layout: dng_writer::IfdLayout::SensorInIfd0,
        width,
        height,
        bits_per_sample: 16,
        kind: dng_writer::PlaneKind::LinearRaw,
        organisation: dng_writer::Organisation::Strips {
            rows_per_strip: height,
        },
        payload: dng_writer::Payload::Samples((0..48u16).map(|i| 100 + i * 37).collect()),
        meta: dng_writer::DngMeta {
            black_level: Some(vec![0]),
            white_level: Some(vec![65535]),
            active_area: Some([1, 1, 6, 7]),
            default_crop_origin: Some([1, 1]),
            default_crop_size: Some([4, 3]),
            orientation: Some(6),
            ..dng_writer::DngMeta::default()
        },
        opcode_lists: [None, None, warp],
        extra_tags: Vec::new(),
        fault: None,
    };
    let write = |d: &dng_writer::DngDescription| {
        dng_writer::write_dng(d).expect("option seed description is valid")
    };
    let bordered = write(&describe(None));
    let warped = write(&describe(Some(opcode::single_warp_rectilinear_list(
        [1.0, -0.1, 0.0, 0.0],
        [0.0, 0.0],
        0.5,
        0.5,
        1,
        0,
    ))));

    let names: [(&'static str, &'static str); 4] = [
        ("options-bordered-0-default", "options-warped-0-default"),
        (
            "options-bordered-1-full-active-area",
            "options-warped-1-full-active-area",
        ),
        (
            "options-bordered-2-unoriented",
            "options-warped-2-unoriented",
        ),
        (
            "options-bordered-3-full-unoriented",
            "options-warped-3-full-unoriented",
        ),
    ];
    let mut seeds = Vec::new();
    for (byte, (bordered_name, warped_name)) in (0u8..4).zip(names) {
        let mut b = bordered.clone();
        b.push(byte);
        seeds.push((bordered_name, b));
        let mut w = warped.clone();
        w.push(byte);
        seeds.push((warped_name, w));
    }

    // SPEC-064: the lerped warp and the FullActiveArea bounds rule.
    let pixel_3a = write(&describe(Some(opcode::single_warp_rectilinear_list(
        [
            0.980_158_686_637_878_4,
            0.091_988_243_162_631_99,
            -0.177_878_528_833_389_28,
            0.107_260_853_052_139_28,
        ],
        [0.0, 0.0],
        0.501_746_677_217_029_4,
        0.502_773_708_767_361_2,
        1,
        0,
    ))));
    let l1021223 = write(&describe(Some(opcode::single_warp_rectilinear_list(
        [
            0.999_251_106,
            -0.061_376_512_877_263_58,
            -0.093_915_541_393_360_16,
            0.055_882_009_215_291_75,
        ],
        [0.0, 0.0],
        0.5,
        0.5,
        1,
        0,
    ))));
    let alpha_seeds: [(&'static str, &Vec<u8>, u8); 6] = [
        ("options-pixel3a-full-alpha-1", &pixel_3a, 0b0_0001),
        ("options-pixel3a-full-alpha-0", &pixel_3a, 0b0_0101),
        ("options-pixel3a-full-alpha-half", &pixel_3a, 0b0_1001),
        ("options-l1021223-full-alpha-1", &l1021223, 0b0_0001),
        ("options-l1021223-full-alpha-0", &l1021223, 0b0_0101),
        ("options-l1021223-full-alpha-half", &l1021223, 0b0_1001),
    ];
    for (name, base, byte) in alpha_seeds {
        let mut seed = base.clone();
        seed.push(byte);
        seeds.push((name, seed));
    }

    // SPEC-065 (DEC-060): the two scratch layouts no seed above reaches, a
    // `FixBadPixelsConstant` alone and with a warp. Every seed above leaves
    // `OpcodeList1` absent, so before these the scratch differential only
    // saw the warp-free and warp-only cases. Every seventh sample is 0, the
    // opcode's `Constant`, so the fix has pixels to replace.
    let with_fix = |warp: Option<Vec<u8>>| {
        let mut d = describe(warp);
        d.opcode_lists[0] = Some(opcode::single_fix_bad_pixels_constant_list(0, 0, 0));
        d.payload = dng_writer::Payload::Samples(
            (0..48u16)
                .map(|i| if i % 7 == 3 { 0 } else { 100 + i * 37 })
                .collect(),
        );
        write(&d)
    };
    let fix_only = with_fix(None);
    let fix_and_warp = with_fix(Some(opcode::single_warp_rectilinear_list(
        [
            0.999_251_106,
            -0.061_376_512_877_263_58,
            -0.093_915_541_393_360_16,
            0.055_882_009_215_291_75,
        ],
        [0.0, 0.0],
        0.5,
        0.5,
        1,
        0,
    )));
    let scratch_seeds: [(&'static str, &Vec<u8>, u8); 5] = [
        ("scratch-fix-only-default", &fix_only, 0b0_0000),
        ("scratch-fix-only-full-unoriented", &fix_only, 0b0_0011),
        ("scratch-fix-and-warp-default", &fix_and_warp, 0b0_0000),
        (
            "scratch-fix-and-warp-full-alpha-half",
            &fix_and_warp,
            0b0_1001,
        ),
        ("scratch-fix-and-warp-alpha-0", &fix_and_warp, 0b0_0100),
    ];
    for (name, base, byte) in scratch_seeds {
        let mut seed = base.clone();
        seed.push(byte);
        seeds.push((name, seed));
    }

    // SPEC-066 (DEC-061): per-position black levels, option bit 5. A CFA
    // plane whose ActiveArea origin is odd (rows 1..6, cols 1..7) with a
    // non-uniform 2x2 pattern; a [1, 2] pattern; and four values against a
    // [1, 1] dimension, which reaches InvalidBlackLevelPattern. The values
    // are numbers in source (the Pixel 3a/4a pattern and hand-picked ones);
    // no corpus byte is used (DEC-003).
    let black_pattern = |blacks: Vec<u32>, repeat: [u16; 2]| {
        let mut d = describe(None);
        d.kind = dng_writer::PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        };
        d.meta.black_level = Some(blacks);
        d.meta.black_level_repeat_dim = Some(repeat);
        d.meta.white_level = Some(vec![16368]);
        d.payload = dng_writer::Payload::Samples((0..48u16).map(|i| 1000 + i * 41).collect());
        write(&d)
    };
    let black_seeds: [(&'static str, Vec<u8>, u8); 3] = [
        (
            "black-levels-2x2-odd-active-area",
            black_pattern(vec![1023, 1023, 1022, 1022], [2, 2]),
            0b10_0000,
        ),
        (
            "black-levels-1x2",
            black_pattern(vec![1000, 1100], [1, 2]),
            0b10_0001,
        ),
        (
            "black-levels-count-mismatch",
            black_pattern(vec![1000, 1001, 1002, 1003], [1, 1]),
            0b10_0000,
        ),
    ];
    for (name, mut seed, byte) in black_seeds {
        seed.push(byte);
        seeds.push((name, seed));
    }
    seeds
}

// ─────────────────────────────────────────────────────────────────────────────
// Seeds for `fuzz/fuzz_targets/demosaic.rs` (SPEC-032)
// ─────────────────────────────────────────────────────────────────────────────

/// Seeds for `fuzz/fuzz_targets/demosaic.rs` — one byte string that survives
/// every stage and actually reaches `demosaic_into` (a real-strip CFA DNG),
/// `SPEC-027`'s canonical valid CFA container (both byte orders), and the
/// hostile CFA shapes the target must reject without panicking: a degenerate
/// CFA whose pattern payload points past EOF (so `sensor().color_filter`
/// resolves to `None` and the demosaic is a typed `NotAColorFilterArray`), and
/// a real-strip DNG truncated mid-strip (so the unpack ahead of the demosaic
/// rejects a short plane). All built from `SPEC-027`'s `tests/support/tiff.rs`
/// CFA builders — own work, no corpus file is used as a seed (`DEC-003`).
fn demosaic_seeds() -> Vec<(&'static str, Vec<u8>)> {
    // A 6×6 RGGB mosaic with a real strip — big enough that interior pixels
    // have genuine offset-2 neighbours, so this seed exercises the full HA
    // kernel, not just its border policy. Values stay well inside 16 bits.
    let samples: Vec<u16> = (0..36u16).map(|i| i * 100).collect();
    let valid_cfa_dng = tiff::cfa_dng_16bit(tiff::Order::Little, 6, 6, &samples);

    // The same real-strip DNG cut off mid-strip: the IFD survives (it sits near
    // the 8-byte header), the strip does not, so the plane unpack ahead of the
    // demosaic rejects it with a typed error.
    let mut truncated_cfa_dng = valid_cfa_dng.clone();
    truncated_cfa_dng.truncate(260);

    vec![
        // Reaches `demosaic_into` on a valid Bayer plane — the good seed a
        // mutation-only fuzzer would otherwise never synthesize.
        ("cfa-16bit-rggb", valid_cfa_dng),
        // SPEC-027's canonical valid CFA container, both byte orders. Its strip
        // is declared past EOF, so the unpack rejects — still a valid parse →
        // sensor → CFA-read seed in each endianness.
        (
            "valid-cfa-sensor-ii",
            tiff::valid_cfa_sensor(tiff::Order::Little),
        ),
        (
            "valid-cfa-sensor-mm",
            tiff::valid_cfa_sensor(tiff::Order::Big),
        ),
        // Degenerate CFA: `CFAPattern` points past EOF, so `color_filter` is
        // `None` and the demosaic is a typed `NotAColorFilterArray`.
        (
            "cfa-pattern-past-eof",
            tiff::cfa_pattern_past_eof(tiff::Order::Little),
        ),
        // A real-strip DNG truncated mid-strip.
        ("truncated-cfa-strip", truncated_cfa_dng),
        // SPEC-045: a valid Compression-7 (mono) plane — the public SOF-3
        // unpack runs in full, then the demosaic is a typed rejection.
        ("compression7-sof3", compression7_seed()),
    ]
    .into_iter()
    // SPEC-058: tiled Bayer planes, which unpack AND demosaic.
    .chain(tiled_seeds())
    .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Seeds for `fuzz/fuzz_targets/ljpeg.rs` (SPEC-038)
// ─────────────────────────────────────────────────────────────────────────────
//
// A local SOF-3 (lossless JPEG) builder mirroring `src/ljpeg.rs mod tests`'s
// in-harness `encode_lj92` round-trip oracle — kept in sync BY HAND, the same
// way `plane_fixture` above mirrors `tests/plane_unpack.rs::Fixture::build`
// (the test module's encoder cannot be imported here — it is test-only code in a
// different compilation unit). Both are exercised by `just test` (the in-crate
// panic-free replay) and `just fuzz-*` / the manual `cargo +nightly fuzz run`
// (the seed), so drift surfaces as a test failure or a fuzz crash, not silently.
//
// It is the INVERSE of the decoder: given the final `u16` samples it emits a
// complete single-component SOF-3 stream (SOI / SOF3 / DHT / optional DRI / SOS /
// a Huffman-coded, EXTENDed, 0xFF-stuffed scan / EOI) that `ljpeg::decode_into`
// reconstructs bit-for-bit. Like the oracle it uses a flat 17-symbol / 5-bit DC
// Huffman table (canonical code of symbol `s` is the 5-bit value `s`), so the
// encoder is a trivially-correct inverse. Everything here is own work built from
// ITU-T T.81 Annex B/C/F/H (`DEC-003`, `DEC-035`); no corpus file is a seed.

// Markers (ITU-T T.81 Annex B.1).
const LJ_MARKER_SOI: u8 = 0xD8;
const LJ_MARKER_EOI: u8 = 0xD9;
const LJ_MARKER_SOF3: u8 = 0xC3;
const LJ_MARKER_DHT: u8 = 0xC4;
const LJ_MARKER_SOS: u8 = 0xDA;
const LJ_MARKER_DRI: u8 = 0xDD;
const LJ_MARKER_RST0: u8 = 0xD0;

/// Canonical code length of the flat table's codes (and the symbol count).
const LJ_SYMBOL_BITS: u32 = 5;

/// A SOF3 frame header (ITU-T T.81 B.2.2) for a single component: `Lf P Y X Nf`
/// then the one 3-byte component spec. `Lf = 8 + 3*1 = 11`.
fn lj_sof3_segment(precision: u8, lines: u16, samples: u16) -> Vec<u8> {
    let mut seg = vec![0xFF, LJ_MARKER_SOF3];
    seg.extend_from_slice(&11u16.to_be_bytes());
    seg.push(precision);
    seg.extend_from_slice(&lines.to_be_bytes());
    seg.extend_from_slice(&samples.to_be_bytes());
    seg.push(1); // Nf
    seg.extend_from_slice(&[0x00, 0x11, 0x00]); // Ci, Hi/Vi, Tqi
    seg
}

/// A DHT segment (ITU-T T.81 B.2.4.2) carrying one DC (class 0) table.
fn lj_dht_segment(th: u8, bits: &[u8; 16], huffval: &[u8]) -> Vec<u8> {
    let lh = (2 + 1 + 16 + huffval.len()) as u16;
    let mut seg = vec![0xFF, LJ_MARKER_DHT];
    seg.extend_from_slice(&lh.to_be_bytes());
    seg.push(th & 0x0F); // Tc = 0 (DC/lossless) high nibble, Th = th
    seg.extend_from_slice(bits);
    seg.extend_from_slice(huffval);
    seg
}

/// A DRI segment (ITU-T T.81 B.2.4.4): `Lr` (= 4) then the restart interval.
fn lj_dri_segment(ri: u16) -> Vec<u8> {
    let mut seg = vec![0xFF, LJ_MARKER_DRI];
    seg.extend_from_slice(&4u16.to_be_bytes());
    seg.extend_from_slice(&ri.to_be_bytes());
    seg
}

/// A SOS scan header (ITU-T T.81 B.2.3) for lossless: `Ss` is the predictor,
/// `Al` the point transform, `Td` the DC table destination. `Ls = 6 + 2*1 = 8`.
fn lj_sos_segment(ss: u8, al: u8, td: u8) -> Vec<u8> {
    let mut seg = vec![0xFF, LJ_MARKER_SOS];
    seg.extend_from_slice(&8u16.to_be_bytes());
    seg.push(1); // Ns
    seg.push(0x00); // Cs
    seg.push(td << 4); // Td high nibble, Ta = 0
    seg.push(ss); // Ss (predictor)
    seg.push(0x00); // Se
    seg.push(al & 0x0F); // Ah = 0 high nibble, Al (point transform) low
    seg
}

/// The flat DC table's `BITS`/`HUFFVAL`: 17 codes all of length 5, symbols
/// `0..=16` in order (so the canonical code of symbol `s` equals `s`).
fn lj_flat_table_bytes() -> ([u8; 16], Vec<u8>) {
    let mut bits = [0u8; 16];
    bits[4] = 17; // 17 codes of length 5 (index 4 == length 5)
    let huffval: Vec<u8> = (0u8..=16).collect();
    (bits, huffval)
}

/// Floor division by two (the sign-preserving `>> 1` of T.81 Table H.1
/// predictors 5/6/7). `>>` on `i32` is an arithmetic shift, i.e. a floor toward
/// negative infinity — matching production's `floor_div2`.
fn lj_floor_div2(x: i32) -> i32 {
    x >> 1
}

/// T.81 Table H.1 lossless predictor `Px` for selector `sel` (`1..=7`), from
/// the point-transformed neighbours `Ra`/`Rb`/`Rc`. Mirrors production's
/// `predict` exactly, so encode→decode round-trips.
fn lj_predict(sel: u8, ra: i32, rb: i32, rc: i32) -> i32 {
    match sel {
        1 => ra,
        2 => rb,
        3 => rc,
        4 => ra + rb - rc,
        5 => ra + lj_floor_div2(rb - rc),
        6 => rb + lj_floor_div2(ra - rc),
        7 => lj_floor_div2(ra + rb),
        _ => ra,
    }
}

/// MSB-first bit sink with JPEG `0xFF 0x00` stuffing (the encoder's counterpart
/// to `ScanReader`), padding the final partial byte with 1-bits.
struct LjBitWriter {
    bytes: Vec<u8>,
    acc: u32,
    nbits: u32,
}

impl LjBitWriter {
    fn new() -> LjBitWriter {
        LjBitWriter {
            bytes: Vec::new(),
            acc: 0,
            nbits: 0,
        }
    }

    fn put_bits(&mut self, value: u32, count: u32) {
        let mut i = count;
        while i > 0 {
            i -= 1;
            let bit = (value >> i) & 1;
            self.acc = (self.acc << 1) | bit;
            self.nbits += 1;
            if self.nbits == 8 {
                self.emit_acc();
            }
        }
    }

    fn emit_acc(&mut self) {
        let b = (self.acc & 0xFF) as u8;
        self.bytes.push(b);
        if b == 0xFF {
            self.bytes.push(0x00); // stuff a literal 0xFF data byte
        }
        self.acc = 0;
        self.nbits = 0;
    }

    fn flush_to_byte(&mut self) {
        if self.nbits > 0 {
            let pad = 8 - self.nbits;
            self.acc = (self.acc << pad) | ((1u32 << pad) - 1);
            self.nbits = 8;
            self.emit_acc();
        }
    }

    fn emit_marker(&mut self, code: u8) {
        self.bytes.push(0xFF);
        self.bytes.push(code);
    }
}

/// Inverse of `RECEIVE`+`EXTEND` (ITU-T T.81 §F.1.2.1 / Annex H.1.2.1): map a
/// signed difference `d` to its category `SSSS` and the `SSSS`-bit value.
fn lj_encode_diff(d: i32) -> (u32, u32) {
    if d == 0 {
        return (0, 0);
    }
    if d == -32768 {
        return (16, 0);
    }
    let mag = d.unsigned_abs(); // 1..=32767
    let ssss = 32 - mag.leading_zeros(); // bit length, 1..=15
    let mask = (1u32 << ssss) - 1;
    let raw = if d > 0 { d as u32 } else { (d - 1) as u32 };
    (ssss, raw & mask)
}

/// Encode an ordered difference list into a Huffman-coded, `0xFF`-stuffed
/// entropy scan, inserting an `RSTm` marker (cycling `D0..D7`) before every
/// `restart_interval` samples, byte-aligning first.
fn lj_encode_scan(diffs: &[i32], restart_interval: usize) -> Vec<u8> {
    let mut w = LjBitWriter::new();
    let mut rst: u8 = 0;
    for (i, &d) in diffs.iter().enumerate() {
        if restart_interval > 0 && i > 0 && i % restart_interval == 0 {
            w.flush_to_byte();
            w.emit_marker(LJ_MARKER_RST0 + rst);
            rst = (rst + 1) % 8;
        }
        let (ssss, value) = lj_encode_diff(d);
        w.put_bits(ssss, LJ_SYMBOL_BITS); // flat table: code == symbol
        if (1..=15).contains(&ssss) {
            w.put_bits(value, ssss);
        }
    }
    w.flush_to_byte();
    w.bytes
}

/// The round-trip LJ92 encoder: final `u16` samples (row-major, `cols*rows`) in,
/// a complete single-component SOF-3 stream out. Prediction mirrors
/// `reconstruct` exactly, computing each difference in the point-transformed
/// domain.
fn lj_encode(
    samples: &[u16],
    cols: usize,
    rows: usize,
    precision: u8,
    predictor: u8,
    point_transform: u8,
    restart_interval: usize,
) -> Vec<u8> {
    assert_eq!(samples.len(), cols * rows, "samples must be cols*rows");
    let pt = point_transform;
    let v: Vec<i32> = samples.iter().map(|&s| i32::from(s >> pt)).collect();
    let initial = {
        let e = i32::from(precision) - i32::from(pt) - 1;
        if e < 0 {
            0
        } else {
            1i32 << e
        }
    };
    let mut diffs: Vec<i32> = Vec::with_capacity(samples.len());
    let mut idx = 0usize;
    let mut count = 0usize;
    for r in 0..rows {
        for c in 0..cols {
            if restart_interval > 0 && count == restart_interval {
                count = 0;
            }
            let px = if count == 0 {
                initial
            } else if r == 0 {
                v[idx - 1]
            } else if c == 0 {
                v[idx - cols]
            } else {
                lj_predict(predictor, v[idx - 1], v[idx - cols], v[idx - cols - 1])
            };
            let raw = (v[idx] - px).rem_euclid(65536);
            let d = if raw >= 32768 { raw - 65536 } else { raw };
            diffs.push(d);
            idx += 1;
            count += 1;
        }
    }

    let entropy = lj_encode_scan(&diffs, restart_interval);
    let (bits, huffval) = lj_flat_table_bytes();
    let mut data = vec![0xFF, LJ_MARKER_SOI];
    data.extend(lj_sof3_segment(precision, rows as u16, cols as u16));
    if restart_interval > 0 {
        data.extend(lj_dri_segment(restart_interval as u16));
    }
    data.extend(lj_dht_segment(0, &bits, &huffval));
    data.extend(lj_sos_segment(predictor, point_transform, 0));
    data.extend(entropy);
    data.extend_from_slice(&[0xFF, LJ_MARKER_EOI]);
    data
}

/// A valid sample plane in `[0, 2^P)`, aligned to `Pt` (stored = reconstructed
/// `<< Pt`, so `reconstructed < 2^(P-Pt)`), from a deterministic LCG — the same
/// shape the oracle's round-trip test draws, so each plane decodes cleanly.
fn lj_gen_samples(count: usize, precision: u8, pt: u8, seed: u32) -> Vec<u16> {
    let modulo: u32 = 1u32 << (precision - pt); // 2^(P - Pt)
    let mut state = seed;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let recon = (state >> 8) % modulo;
        out.push((recon << pt) as u16);
    }
    out
}

/// Seeds for `fuzz/fuzz_targets/ljpeg.rs` — valid single-component SOF-3 streams
/// across the full predictor set (`1..=7`), precisions {8, 12, 16}, a point
/// transform, and a restart interval, plus the hostile shapes `SPEC-038` names:
/// a truncated header, a valid header then a truncated scan, an over-subscribed
/// DHT, `Nf != 1`, and a bad marker; and (`SPEC-053`) a valid two-component
/// stream and a subsampled (`H = V = 2`) component. Every one must `decode_into` to a typed
/// result (the valid ones to `Ok`, the hostile ones to a typed `Error`) and
/// never panic — the in-crate replay test in `src/ljpeg.rs` asserts exactly that
/// over this directory.
fn ljpeg_seeds() -> Vec<(&'static str, Vec<u8>)> {
    let mut seeds: Vec<(&'static str, Vec<u8>)> = Vec::new();

    // Valid: every predictor 1..=7, spread across the three precisions.
    let valid: &[(&'static str, usize, usize, u8, u8, u8, usize)] = &[
        // (name, cols, rows, precision, predictor, point_transform, restart)
        ("valid-pred1-p8", 4, 3, 8, 1, 0, 0),
        ("valid-pred2-p8", 4, 3, 8, 2, 0, 0),
        ("valid-pred3-p8", 4, 3, 8, 3, 0, 0),
        ("valid-pred4-p12", 4, 3, 12, 4, 0, 0),
        ("valid-pred5-p12", 4, 3, 12, 5, 0, 0),
        ("valid-pred6-p16", 4, 3, 16, 6, 0, 0),
        ("valid-pred7-p16", 4, 3, 16, 7, 0, 0),
        ("valid-pt2-p12", 4, 3, 12, 1, 2, 0),
        ("valid-restart-p8", 4, 4, 8, 1, 0, 4),
    ];
    for &(name, cols, rows, precision, predictor, pt, ri) in valid {
        let samples = lj_gen_samples(cols * rows, precision, pt, 0x1357 ^ u32::from(predictor));
        seeds.push((
            name,
            lj_encode(&samples, cols, rows, precision, predictor, pt, ri),
        ));
    }

    let (flat_bits, flat_huffval) = lj_flat_table_bytes();

    // Hostile: a SOF3 header cut mid-segment (ends after the Y field — X never
    // reads). parse_headers → Truncated.
    seeds.push((
        "truncated-header",
        vec![
            0xFF,
            LJ_MARKER_SOI,
            0xFF,
            LJ_MARKER_SOF3,
            0x00,
            0x0B, // Lf = 11
            0x08, // P = 8
            0x00,
            0x02, // Y = 2
        ],
    ));

    // Hostile: a complete, valid header then a single entropy byte — the
    // 16-sample scan runs out mid-stream. decode_into → Truncated.
    seeds.push(("truncated-scan", {
        let mut d = vec![0xFF, LJ_MARKER_SOI];
        d.extend(lj_sof3_segment(8, 4, 4));
        d.extend(lj_dht_segment(0, &flat_bits, &flat_huffval));
        d.extend(lj_sos_segment(1, 0, 0));
        d.push(0x00); // one entropy byte, then EOF
        d
    }));

    // Hostile: a DHT whose BITS over-subscribe the code space — three 1-bit
    // codes cannot exist. HuffTable::build → LjpegInvalidHuffmanTable.
    seeds.push(("oversized-dht", {
        let mut d = vec![0xFF, LJ_MARKER_SOI];
        d.extend(lj_sof3_segment(8, 2, 2));
        let oversub_bits = [3u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        d.extend(lj_dht_segment(0, &oversub_bits, &[1u8, 2, 3]));
        d
    }));

    // Hostile: a SOF3 frame declaring three components. parse_frame_header →
    // LjpegUnsupportedComponentCount { nf: 3 }.
    seeds.push(("nf-not-one", {
        let mut d = vec![0xFF, LJ_MARKER_SOI, 0xFF, LJ_MARKER_SOF3];
        d.extend_from_slice(&17u16.to_be_bytes()); // Lf = 8 + 3*3
        d.push(0x08); // P
        d.extend_from_slice(&2u16.to_be_bytes()); // Y
        d.extend_from_slice(&2u16.to_be_bytes()); // X
        d.push(0x03); // Nf = 3
        d.extend_from_slice(&[0x00, 0x11, 0x00, 0x01, 0x11, 0x00, 0x02, 0x11, 0x00]);
        d
    }));

    // Hostile: a reserved/unexpected marker where a header segment belongs.
    // next_marker reads it; parse_headers → LjpegUnexpectedMarker { marker: 1 }.
    seeds.push(("bad-marker", vec![0xFF, LJ_MARKER_SOI, 0xFF, 0x01]));

    // SPEC-053 (DEC-047). Valid: a two-component (Nf = 2) column-interleave
    // stream, the layout both held Compression-7 files use, so the fuzzer
    // reaches the Nf = 2 shape (2 x samples_per_line wide) from a decoding seed.
    let two_component: Vec<u16> = (0..8u32 * 3).map(|i| (i * 211 % 4096) as u16).collect();
    seeds.push((
        "valid-two-component-p12",
        tiff::lj92_strip_components(&two_component, 8, 3, 12, 2, 0x11),
    ));

    // Hostile: an otherwise valid stream whose component declares H = V = 2.
    // parse_frame_header → LjpegUnsupportedSampling { factor: 0x22 }.
    let subsampled: Vec<u16> = (0..4u32 * 3).map(|i| (i * 97 % 4096) as u16).collect();
    seeds.push((
        "subsampled-hv22",
        tiff::lj92_strip_components(&subsampled, 4, 3, 12, 1, 0x22),
    ));

    seeds
}

fn write_seeds(
    subdir: &str,
    extension: &str,
    seeds: Vec<(&'static str, Vec<u8>)>,
) -> std::io::Result<usize> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("seeds")
        .join(subdir);
    std::fs::create_dir_all(&dir)?;

    let mut written = 0usize;
    for (name, bytes) in seeds {
        let path = dir.join(format!("{name}.{extension}"));
        std::fs::write(&path, &bytes)?;
        println!("{:>6} bytes  {}", bytes.len(), path.display());
        written += 1;
    }
    println!("fuzz-seeds: {written} seed(s) in {}", dir.display());
    Ok(written)
}

fn main() {
    if let Err(e) = write_seeds("ifd", "tiff", tiff::all()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    // SPEC-062: the preview seeds share the `ifd` target's seed directory but
    // not `tiff::all()`, which tests/ifd_reader.rs sweeps shape by shape.
    if let Err(e) = write_seeds("ifd", "tiff", tiff::preview_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("plane", "tiff", plane_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("develop", "tiff", develop_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("warp_opcode", "bin", opcode::fuzz_seed_corpus()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("opcode", "bin", opcode::fix_bad_pixels_fuzz_seed_corpus()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("demosaic", "tiff", demosaic_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
    if let Err(e) = write_seeds("ljpeg", "lj92", ljpeg_seeds()) {
        eprintln!("fuzz-seeds: {e}");
        std::process::exit(1);
    }
}
