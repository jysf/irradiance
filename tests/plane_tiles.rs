//! `SPEC-058` — tiled sensor planes (`DEC-056`): uncompressed and
//! Compression-7 (SOF-3 lossless JPEG) tiles, with partial right and bottom
//! edge tiles, decoded allocation-free into the caller's plane.
//!
//! Tier A (every case but the two Pixel tests): each fixture is written by the
//! SPEC-057 test-only DNG writer (`tests/support/dng_writer.rs`), uncompressed
//! tiles from `Payload::Samples` and Compression-7 tiles from
//! `Payload::Encoded` with one `tiff::lj92_strip_components` chunk per tile.
//! Every tile is coded at its FULL size, padding included, exactly as the Pixel
//! phones code their edge tiles (SPEC-058 design probe), and the padding is a
//! distinctive `edge_fill`, so a reader that leaks padding into the plane shows
//! a sentinel rather than a plausible value.
//!
//! Tier B (`pixel_files_*`): the three Pixel phone DNGs, bit-exact against the
//! manifest's pinned `dnglab --raw-checksum`, and still refused at develop for
//! their mandatory `OpcodeList2` GainMaps (`DEC-049`). Each skips loudly by name
//! when the corpus is absent (`DEC-003`).
//!
//! The window red-proof (AC7) lives with the copy-and-rebuild apparatus in
//! `tests/plane_oracle.rs` (`window_clip_red_proof`).

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/md5.rs"]
mod md5;
#[path = "support/tiff.rs"]
mod tiff;

use corpus::{CorpusRoot, Manifest};
use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag,
    TagIfd, TAG_TILE_BYTE_COUNTS, TAG_TILE_LENGTH, TAG_TILE_OFFSETS, TAG_TILE_WIDTH,
};
use irradiance::ifd::{
    Container, TAG_BITS_PER_SAMPLE, TAG_IMAGE_LENGTH, TAG_IMAGE_WIDTH, TAG_STRIP_BYTE_COUNTS,
    TAG_STRIP_OFFSETS,
};
use irradiance::plane::{unpack_into, validate};
use irradiance::{Error, ErrorKind};
use tiff::Order;

const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// A deterministic sample ramp below `2^bits`, distinct from its neighbours
/// almost everywhere, so a misplaced tile or a transposed row cannot hide.
fn samples(width: u32, height: u32, bits: u32) -> Vec<u16> {
    let max = (1u32 << bits) - 1;
    (0..width * height)
        .map(|i| ((i.wrapping_mul(2_654_435_761) >> 7) % (max + 1)) as u16)
        .collect()
}

/// A padding sentinel below `2^bits`: `0xBEEF` masked to the depth, never a
/// value [`samples`] is likely to land on at the same position.
fn fill(bits: u32) -> u16 {
    (0xBEEF & ((1u32 << bits) - 1)) as u16
}

fn rggb() -> PlaneKind {
    PlaneKind::Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 1, 1, 2],
        plane_color: vec![0, 1, 2],
    }
}

/// A tiled description, sensor in a SubIFD under a preview IFD0 (the shape the
/// Pixel files have).
#[allow(clippy::too_many_arguments)]
fn tiled(
    order: Order,
    kind: PlaneKind,
    width: u32,
    height: u32,
    bits: u32,
    tile_width: u32,
    tile_length: u32,
    payload: Payload,
) -> DngDescription {
    DngDescription {
        order,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width,
        height,
        bits_per_sample: bits,
        kind,
        organisation: Organisation::Tiles {
            tile_width,
            tile_length,
            edge_fill: fill(bits.min(16)),
        },
        payload,
        meta: DngMeta::default(),
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

/// Cut `plane` (`width × height`) into full-size tiles, padding past the
/// plane's right and bottom edges with `pad`, and LJ92-encode each tile as one
/// SOF-3 datastream: `nf = 1` gives a `tile_width × tile_length` frame,
/// `nf = 2` a `tile_width/2`-sample, two-component column interleave.
#[allow(clippy::too_many_arguments)]
fn lj92_tiles(
    plane: &[u16],
    width: u32,
    height: u32,
    tile_width: u32,
    tile_length: u32,
    pad: u16,
    precision: u8,
    nf: u8,
) -> Vec<Vec<u8>> {
    let across = width.div_ceil(tile_width);
    let down = height.div_ceil(tile_length);
    let mut chunks = Vec::new();
    for ty in 0..down {
        for tx in 0..across {
            let mut tile = Vec::with_capacity((tile_width * tile_length) as usize);
            for r in 0..tile_length {
                for c in 0..tile_width {
                    let (x, y) = (tx * tile_width + c, ty * tile_length + r);
                    tile.push(if x < width && y < height {
                        plane[(y * width + x) as usize]
                    } else {
                        pad
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
}

/// `validate`, allocate, `unpack_into` — the documented public sequence.
fn decode(file: &[u8]) -> Result<Vec<u16>, Error> {
    let container = Container::parse(file)?;
    let sensor = container.sensor()?;
    let n = validate(&sensor, container.byte_order(), file, usize::MAX)?;
    let mut plane = vec![0u16; n];
    unpack_into(&sensor, container.byte_order(), file, &mut plane)?;
    Ok(plane)
}

/// Both public entry points on the same file: `validate`'s verdict, and
/// `unpack_into`'s into a correctly sized buffer.
fn both(file: &[u8]) -> (Result<usize, Error>, Result<(), Error>) {
    let container = Container::parse(file).expect("fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor");
    let order = container.byte_order();
    let v = validate(&sensor, order, file, usize::MAX);
    let n = (u64::from(sensor.width) * u64::from(sensor.height)) as usize;
    let mut plane = vec![0u16; n];
    let u = unpack_into(&sensor, order, file, &mut plane);
    (v, u)
}

fn le_longs(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn long_tag(tag: u16, values: &[u32]) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type: TYPE_LONG,
        count: values.len() as u32,
        bytes: le_longs(values),
    }
}

/// Geometry cases every tiled round trip runs: partial right AND bottom tiles,
/// a single tile larger than the plane, an exact multiple of the tile size, and
/// a plane only partial at the right (and one only at the bottom).
/// `(width, height, tile_width, tile_length)`; tile widths are multiples of 4
/// so every depth (14-bit included) has whole-byte tile rows.
const GEOMETRIES: [(u32, u32, u32, u32); 5] = [
    (21, 13, 8, 4),  // partial right and bottom
    (5, 3, 16, 8),   // one tile, larger than the plane
    (32, 16, 8, 8),  // exact multiple
    (20, 16, 16, 4), // partial right only
    (16, 18, 8, 8),  // partial bottom only
];

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — uncompressed tiles
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn uncompressed_tiles_round_trip_edges_and_orders() {
    let mut cases = 0;
    for order in [Order::Little, Order::Big] {
        for bits in [8u32, 12, 14, 16] {
            for kind in [PlaneKind::LinearRaw, rggb()] {
                for (w, h, tw, tl) in GEOMETRIES {
                    let input = samples(w, h, bits);
                    let d = tiled(
                        order,
                        kind.clone(),
                        w,
                        h,
                        bits,
                        tw,
                        tl,
                        Payload::Samples(input.clone()),
                    );
                    let file = write_dng(&d).expect("writer accepts the description");
                    let plane = decode(&file).unwrap_or_else(|e| {
                        panic!("{order:?} {bits}-bit {w}x{h} tiles {tw}x{tl}: {e}")
                    });
                    assert_eq!(
                        plane, input,
                        "{order:?} {bits}-bit {kind:?} {w}x{h} tiles {tw}x{tl} must round-trip"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 2 * 4 * 2 * GEOMETRIES.len());
}

#[test]
fn uncompressed_tile_padding_never_reaches_dst() {
    // The padding sentinel must appear NOWHERE in the decoded plane, at every
    // depth, for a plane partial on the right, at the bottom, and both. The
    // input is built so it never contains the sentinel itself, so any
    // occurrence in the output is leaked padding — including the bottom-edge
    // and 12-bit fill SPEC-057's verify flagged as only indirectly asserted.
    for order in [Order::Little, Order::Big] {
        for bits in [8u32, 12, 14, 16] {
            let pad = fill(bits);
            for (w, h, tw, tl) in [(21u32, 13u32, 8u32, 4u32), (16, 18, 8, 8), (20, 16, 16, 4)] {
                let input: Vec<u16> = samples(w, h, bits)
                    .into_iter()
                    .map(|s| if s == pad { s ^ 1 } else { s })
                    .collect();
                assert!(!input.contains(&pad));
                let d = tiled(
                    order,
                    PlaneKind::LinearRaw,
                    w,
                    h,
                    bits,
                    tw,
                    tl,
                    Payload::Samples(input.clone()),
                );
                let file = write_dng(&d).unwrap();
                let plane = decode(&file).unwrap();
                assert!(
                    !plane.contains(&pad),
                    "{order:?} {bits}-bit {w}x{h}: padding {pad:#x} leaked into the plane"
                );
                assert_eq!(plane, input);
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — Compression-7 tiles
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn lj92_tiles_round_trip_nf1_nf2_edges() {
    let mut cases = 0;
    for order in [Order::Little, Order::Big] {
        for precision in [8u8, 12, 14, 16] {
            for nf in [1u8, 2] {
                for (w, h, tw, tl) in GEOMETRIES {
                    let bits = u32::from(precision);
                    let input = samples(w, h, bits);
                    let chunks = lj92_tiles(&input, w, h, tw, tl, fill(bits), precision, nf);
                    let d = tiled(
                        order,
                        rggb(),
                        w,
                        h,
                        bits,
                        tw,
                        tl,
                        Payload::Encoded {
                            compression: 7,
                            chunks,
                        },
                    );
                    let file = write_dng(&d).expect("writer accepts the chunks");
                    let plane = decode(&file).unwrap_or_else(|e| {
                        panic!("{order:?} P={precision} Nf={nf} {w}x{h} tiles {tw}x{tl}: {e}")
                    });
                    assert_eq!(
                        plane, input,
                        "{order:?} P={precision} Nf={nf} {w}x{h} tiles {tw}x{tl} must round-trip"
                    );
                    assert!(
                        !plane.contains(&fill(bits)) || input.contains(&fill(bits)),
                        "padding leaked"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 2 * 4 * 2 * GEOMETRIES.len());
}

#[test]
fn lj92_tiles_odd_visible_width_nf2() {
    // Under Nf=2 a tile's two column phases are two components; an ODD visible
    // width at the right edge leaves component 0 visible and component 1
    // discarded in the last visible MCU. Widths 1..=7 over a 4-wide tile cover
    // every residue, with partial bottom tiles too.
    for order in [Order::Little, Order::Big] {
        for w in 1u32..=11 {
            for h in [1u32, 3, 5] {
                let (tw, tl) = (4u32, 2u32);
                let input = samples(w, h, 16);
                let chunks = lj92_tiles(&input, w, h, tw, tl, 0xBEEF, 16, 2);
                let d = tiled(
                    order,
                    PlaneKind::LinearRaw,
                    w,
                    h,
                    16,
                    tw,
                    tl,
                    Payload::Encoded {
                        compression: 7,
                        chunks,
                    },
                );
                let file = write_dng(&d).unwrap();
                let plane = decode(&file).unwrap_or_else(|e| panic!("{w}x{h}: {e}"));
                assert_eq!(plane, input, "{order:?} {w}x{h} (odd visible width) Nf=2");
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — hostile tile tables
// ─────────────────────────────────────────────────────────────────────────────

/// A valid 16-bit uncompressed tiled fixture, little-endian, sensor in a
/// SubIFD: 16×16 in 8×8 tiles (four tiles of 128 bytes each).
fn hostile_base() -> DngDescription {
    tiled(
        Order::Little,
        PlaneKind::LinearRaw,
        16,
        16,
        16,
        8,
        8,
        Payload::Samples(samples(16, 16, 16)),
    )
}

/// Assert `validate` and `unpack_into` both refuse `file` with the same variant
/// (by `matches`), and that the variant is the one `want` names.
///
/// Every table refusal comes from the shared layout pass, which runs before
/// the first tile is written, so `dst` must come back exactly as it went in:
/// it is pre-filled with a sentinel and checked after the refusal (SPEC-058
/// verify). A layout check that ran per tile inside the write loop would
/// return the same variant after earlier tiles were written, and fail here.
fn assert_both_refuse(label: &str, file: &[u8], want: fn(&Error) -> bool) {
    const UNTOUCHED: u16 = 0xA5A5;
    let container = Container::parse(file).expect("fixture parses");
    let sensor = container.sensor().expect("fixture has a sensor");
    let order = container.byte_order();
    let v = validate(&sensor, order, file, usize::MAX);
    let n = (u64::from(sensor.width) * u64::from(sensor.height)) as usize;
    let mut plane = vec![UNTOUCHED; n];
    let u = unpack_into(&sensor, order, file, &mut plane);
    let v = v.expect_err(&format!("{label}: validate must refuse"));
    let u = u.expect_err(&format!("{label}: unpack_into must refuse"));
    assert!(
        plane.iter().all(|&s| s == UNTOUCHED),
        "{label}: unpack_into wrote to dst before refusing"
    );
    assert!(want(&v), "{label}: validate gave {v:?}");
    assert!(want(&u), "{label}: unpack_into gave {u:?}");
    assert_eq!(
        std::mem::discriminant(&v),
        std::mem::discriminant(&u),
        "{label}: validate and unpack_into must agree"
    );
}

#[test]
fn hostile_tile_tables_are_typed_from_validate_and_unpack() {
    let base = hostile_base();
    // The honest base decodes — the cases below are refusals of THIS file with
    // one table entry changed, not of a fixture that never worked.
    assert_eq!(
        decode(&write_dng(&base).unwrap()).unwrap(),
        samples(16, 16, 16)
    );

    let with = |tags: Vec<RawTag>| {
        let mut d = base.clone();
        d.extra_tags = tags;
        write_dng(&d).unwrap()
    };

    // TileWidth = 0, TileLength = 0.
    for tag in [TAG_TILE_WIDTH, TAG_TILE_LENGTH] {
        assert_both_refuse(
            &format!("tag {tag} = 0"),
            &with(vec![long_tag(tag, &[0])]),
            |e| matches!(e, Error::InvalidTileLayout { .. }),
        );
    }

    // TileOffsets and TileByteCounts disagree in count.
    assert_both_refuse(
        "offsets/counts length mismatch",
        &with(vec![long_tag(TAG_TILE_BYTE_COUNTS, &[128, 128, 128])]),
        |e| matches!(e, Error::InvalidTileLayout { .. }),
    );

    // Both agree, but not with ceil(W/TW) x ceil(H/TL) = 4.
    let file = write_dng(&base).unwrap();
    let sensor = Container::parse(&file).unwrap().sensor().unwrap();
    let offsets = sensor.tile_offsets.clone();
    assert_eq!(offsets.len(), 4);
    assert_both_refuse(
        "three tiles for a four-tile grid",
        &with(vec![
            long_tag(TAG_TILE_OFFSETS, &offsets[..3]),
            long_tag(TAG_TILE_BYTE_COUNTS, &[128, 128, 128]),
        ]),
        |e| {
            matches!(
                e,
                Error::InvalidTileLayout {
                    tile_offsets: 3,
                    tile_byte_counts: 3,
                    expected_tiles: 4,
                    ..
                }
            )
        },
    );

    // A tile that starts past the end of the file.
    let mut past = offsets.clone();
    past[3] = u32::MAX - 64;
    assert_both_refuse(
        "tile past EOF",
        &with(vec![long_tag(TAG_TILE_OFFSETS, &past)]),
        |e| matches!(e, Error::Truncated { .. }),
    );

    // A tile whose END runs past the file: the last tile starts inside it but
    // its count is a huge number. (Uncompressed would refuse the count first,
    // so this is the Compression-7 route, whose counts are free.)
    let lj = {
        let input = samples(16, 16, 16);
        let chunks = lj92_tiles(&input, 16, 16, 8, 8, 0, 16, 1);
        let mut d = base.clone();
        d.payload = Payload::Encoded {
            compression: 7,
            chunks,
        };
        d
    };
    let lj_file = write_dng(&lj).unwrap();
    let lj_sensor = Container::parse(&lj_file).unwrap().sensor().unwrap();
    let mut long_counts = lj_sensor.tile_byte_counts.clone();
    long_counts[2] = u32::MAX;
    let mut d = lj.clone();
    d.extra_tags = vec![long_tag(TAG_TILE_BYTE_COUNTS, &long_counts)];
    assert_both_refuse(
        "Compression-7 tile end past EOF",
        &write_dng(&d).unwrap(),
        |e| matches!(e, Error::Truncated { .. }),
    );

    // Uncompressed: a tile byte count that is not the packed tile size.
    assert_both_refuse(
        "uncompressed tile byte count",
        &with(vec![long_tag(TAG_TILE_BYTE_COUNTS, &[128, 128, 127, 128])]),
        |e| {
            matches!(
                e,
                Error::PackedSizeMismatch {
                    expected_bits: 1024,
                    strip_bits: 1016,
                }
            )
        },
    );

    // Uncompressed: a tile row that is not a whole number of bytes (a 3-wide,
    // 12-bit tile row is 36 bits). The 16-bit fixture is re-labelled 12-bit.
    let mut d = tiled(
        Order::Little,
        PlaneKind::LinearRaw,
        3,
        2,
        16,
        3,
        2,
        Payload::Samples(samples(3, 2, 12)),
    );
    d.extra_tags = vec![RawTag {
        ifd: TagIfd::Sensor,
        tag: TAG_BITS_PER_SAMPLE,
        field_type: TYPE_SHORT,
        count: 1,
        bytes: 12u16.to_le_bytes().to_vec(),
    }];
    assert_both_refuse("12-bit, 3-wide tile row", &write_dng(&d).unwrap(), |e| {
        matches!(
            e,
            Error::UnsupportedTileRowBits {
                tile_width: 3,
                bits: 12
            }
        ) && e.kind() == ErrorKind::Unsupported
    });

    // Uncompressed: every tile aliases ONE tile's bytes, so the declared bytes
    // sum past the file. Built from Compression-1 chunks where only tile 0
    // carries bytes, so the file is far smaller than 4 x 128.
    let marker: Vec<u8> = (0u8..128).map(|i| i.wrapping_mul(37) ^ 0x5A).collect();
    let mut d = base.clone();
    d.payload = Payload::Encoded {
        compression: 1,
        chunks: vec![marker.clone(), vec![], vec![], vec![]],
    };
    let probe = write_dng(&d).unwrap();
    let at = probe
        .windows(marker.len())
        .position(|w| w == marker.as_slice())
        .expect("tile 0's bytes are in the file") as u32;
    d.extra_tags = vec![
        long_tag(TAG_TILE_OFFSETS, &[at; 4]),
        long_tag(TAG_TILE_BYTE_COUNTS, &[128; 4]),
    ];
    let aliased = write_dng(&d).unwrap();
    assert!(
        aliased.len() < 4 * 128,
        "the fixture must be smaller than 512 bytes"
    );
    assert_both_refuse("aliased tiles", &aliased, |e| {
        matches!(e, Error::StripBytesExceedFile { declared: 512, .. })
    });

    // Strip AND tile tags on one plane.
    assert_both_refuse(
        "strips and tiles",
        &with(vec![
            long_tag(TAG_STRIP_OFFSETS, &[offsets[0]]),
            long_tag(TAG_STRIP_BYTE_COUNTS, &[512]),
        ]),
        |e| matches!(e, Error::StripsAndTiles { .. }) && e.kind() == ErrorKind::Corrupt,
    );
}

#[test]
fn tile_count_is_computed_without_u32_wraparound() {
    // 641 x 6700417 one-pixel tiles is 2^32 + 1 tiles: a u32 product wraps to
    // exactly 1, which would MATCH this file's single tile. validate computes
    // the count in u64 and refuses. (unpack_into is not run: a correctly-sized
    // buffer would be ~8 GiB, and its dst-length check precedes layout anyway.)
    let mut d = hostile_base();
    d.extra_tags = vec![
        long_tag(TAG_IMAGE_WIDTH, &[641]),
        long_tag(TAG_IMAGE_LENGTH, &[6_700_417]),
        long_tag(TAG_TILE_WIDTH, &[1]),
        long_tag(TAG_TILE_LENGTH, &[1]),
        long_tag(TAG_TILE_OFFSETS, &[64]),
        long_tag(TAG_TILE_BYTE_COUNTS, &[2]),
    ];
    let file = write_dng(&d).unwrap();
    let container = Container::parse(&file).unwrap();
    let sensor = container.sensor().unwrap();
    let err = validate(&sensor, container.byte_order(), &file, usize::MAX).unwrap_err();
    assert!(
        matches!(
            err,
            Error::InvalidTileLayout {
                expected_tiles: 4_294_967_297,
                tile_offsets: 1,
                ..
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn tile_frame_shape_is_bound_to_the_tile() {
    // An 8x4 tile grid whose tiles are coded as EQUAL-AREA 4x8 frames: the
    // area check alone would accept it and mis-stride the plane. The per-tile
    // shape binding (DEC-047 applied to tiles, DEC-056 §3) refuses it before
    // any sample is decoded. validate does not decode, so it accepts the table
    // (its documented contract, the same as for strips).
    let (w, h, tw, tl) = (16u32, 8u32, 8u32, 4u32);
    let input = samples(w, h, 12);
    // Encode each tile's 32 samples as a 4-wide, 8-tall frame.
    let mut chunks = Vec::new();
    for _ in 0..(w / tw) * (h / tl) {
        chunks.push(tiff::lj92_strip_components(
            &input[..32],
            tl as usize,
            tw as usize,
            12,
            1,
            0x11,
        ));
    }
    let d = tiled(
        Order::Little,
        PlaneKind::LinearRaw,
        w,
        h,
        12,
        tw,
        tl,
        Payload::Encoded {
            compression: 7,
            chunks,
        },
    );
    let (v, u) = both(&write_dng(&d).unwrap());
    assert_eq!(v.unwrap(), (w * h) as usize);
    let err = u.unwrap_err();
    assert!(
        matches!(
            err,
            Error::LjpegPlaneShapeMismatch {
                expected_width: 8,
                expected_height: 4,
                decoded_width: 4,
                decoded_height: 8,
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn clipped_size_edge_frame_is_refused() {
    // Real edge tiles are coded at FULL tile size (SPEC-058 probe), and
    // DEC-056 §3 refuses an edge frame coded at only its visible size rather
    // than guessing. Without the per-tile shape binding the window would
    // accept such a frame (its visible extent fits it) and decode it.
    //
    // Two shapes. (a) A 5x3 plane in one 16x8 tile, the tile coded as a 5x3
    // frame: refused before any sample is written, so dst is untouched.
    // (b) A 21x13 plane in 8x4 tiles whose right-edge and bottom-edge tiles
    // are coded at their visible size (5 wide, 1 tall): refused at the first
    // such tile.
    const UNTOUCHED: u16 = 0xA5A5;
    let input = samples(5, 3, 12);
    let chunk = tiff::lj92_strip_components(&input, 5, 3, 12, 1, 0x11);
    let d = tiled(
        Order::Big,
        PlaneKind::LinearRaw,
        5,
        3,
        12,
        16,
        8,
        Payload::Encoded {
            compression: 7,
            chunks: vec![chunk],
        },
    );
    let file = write_dng(&d).unwrap();
    let container = Container::parse(&file).unwrap();
    let sensor = container.sensor().unwrap();
    let order = container.byte_order();
    assert_eq!(validate(&sensor, order, &file, usize::MAX).unwrap(), 15);
    let mut plane = vec![UNTOUCHED; 15];
    let err = unpack_into(&sensor, order, &file, &mut plane).unwrap_err();
    assert!(
        matches!(
            err,
            Error::LjpegPlaneShapeMismatch {
                expected_width: 16,
                expected_height: 8,
                decoded_width: 5,
                decoded_height: 3,
            }
        ),
        "single clipped-size tile: got {err:?}"
    );
    assert!(
        plane.iter().all(|&s| s == UNTOUCHED),
        "the shape binding must refuse before the tile is decoded"
    );

    let (w, h, tw, tl) = (21u32, 13u32, 8u32, 4u32);
    let input = samples(w, h, 12);
    let mut chunks = Vec::new();
    for ty in 0..h.div_ceil(tl) {
        for tx in 0..w.div_ceil(tw) {
            let (x0, y0) = (tx * tw, ty * tl);
            let (vw, vh) = (tw.min(w - x0), tl.min(h - y0));
            let mut tile = Vec::new();
            for r in 0..vh {
                for c in 0..vw {
                    tile.push(input[((y0 + r) * w + x0 + c) as usize]);
                }
            }
            chunks.push(tiff::lj92_strip_components(
                &tile,
                vw as usize,
                vh as usize,
                12,
                1,
                0x11,
            ));
        }
    }
    let d = tiled(
        Order::Little,
        PlaneKind::LinearRaw,
        w,
        h,
        12,
        tw,
        tl,
        Payload::Encoded {
            compression: 7,
            chunks,
        },
    );
    let (v, u) = both(&write_dng(&d).unwrap());
    assert_eq!(v.unwrap(), (w * h) as usize);
    let err = u.unwrap_err();
    assert!(
        matches!(
            err,
            Error::LjpegPlaneShapeMismatch {
                expected_width: 8,
                expected_height: 4,
                decoded_width: 5,
                decoded_height: 4,
            }
        ),
        "first right-edge tile coded at its visible size: got {err:?}"
    );
}

#[test]
fn tile_error_display_names_the_numbers() {
    // AC10: each new variant's Display text names the values a caller needs.
    let msg = Error::InvalidTileLayout {
        tile_width: 256,
        tile_length: 128,
        tile_offsets: 191,
        tile_byte_counts: 192,
        expected_tiles: 192,
    }
    .to_string();
    for needle in [
        "256x128",
        "191 TileOffsets",
        "192 TileByteCounts",
        "need 192",
    ] {
        assert!(
            msg.contains(needle),
            "InvalidTileLayout: {msg:?} lacks {needle:?}"
        );
    }
    let msg = Error::StripsAndTiles {
        strip_offsets: 3,
        tile_offsets: 12,
    }
    .to_string();
    for needle in ["3 StripOffsets", "12 TileOffsets"] {
        assert!(
            msg.contains(needle),
            "StripsAndTiles: {msg:?} lacks {needle:?}"
        );
    }
    let msg = Error::UnsupportedTileRowBits {
        tile_width: 3,
        bits: 12,
    }
    .to_string();
    for needle in ["3-wide", "12-bit", "whole number of bytes"] {
        assert!(
            msg.contains(needle),
            "UnsupportedTileRowBits: {msg:?} lacks {needle:?}"
        );
    }
}

#[test]
fn validate_reads_no_tile_bytes() {
    // validate's Ok must not depend on tile CONTENT: corrupting every
    // Compression-7 tile's bytes (keeping the table) leaves validate's answer
    // unchanged, while unpack_into now fails in the decode.
    let input = samples(16, 16, 16);
    let chunks: Vec<Vec<u8>> = lj92_tiles(&input, 16, 16, 8, 8, 0, 16, 2)
        .into_iter()
        .map(|c| vec![0u8; c.len()])
        .collect();
    let d = tiled(
        Order::Big,
        PlaneKind::LinearRaw,
        16,
        16,
        16,
        8,
        8,
        Payload::Encoded {
            compression: 7,
            chunks,
        },
    );
    let (v, u) = both(&write_dng(&d).unwrap());
    assert_eq!(v.unwrap(), 256);
    assert!(u.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 / AC8 — the Pixel phone DNGs (tier B)
// ─────────────────────────────────────────────────────────────────────────────

const PIXEL_FILES: [&str; 3] = [
    "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng",
    "GOOGLE-PIXEL-3A/IMG_20190918_164153.dng",
    "GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng",
];

/// The Pixel files present in this corpus, with their manifest
/// `raw_checksum`. Each absent one is announced by name by `require`.
fn present_pixel_files() -> Vec<(&'static str, Vec<u8>, String)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    PIXEL_FILES
        .iter()
        .filter_map(|path| {
            let entry = manifest
                .get(path)
                .unwrap_or_else(|| panic!("{path} must be in the manifest"));
            entry.require(&root).map(|p| {
                let bytes = std::fs::read(&p).unwrap_or_else(|e| panic!("read {path}: {e}"));
                (*path, bytes, entry.oracle.raw_checksum.clone())
            })
        })
        .collect()
}

fn plane_md5(plane: &[u16]) -> String {
    let bytes: Vec<u8> = plane.iter().flat_map(|s| s.to_le_bytes()).collect();
    md5::to_hex(&md5::hash(&bytes))
}

#[test]
fn pixel_files_unpack_bit_exact() {
    for (path, bytes, pinned) in present_pixel_files() {
        let plane = decode(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
        let digest = plane_md5(&plane);
        assert_eq!(
            digest, pinned,
            "{path}: plane MD5 must equal dnglab's raw_checksum"
        );
        eprintln!("PIXEL BIT-EXACT ({path}): {digest}");
    }
}

#[test]
fn pixel_files_develop_refuses_mandatory_gainmap() {
    use irradiance::color::develop_color_into;
    use irradiance::develop::develop_into;

    for (path, bytes, _) in present_pixel_files() {
        let container = Container::parse(&bytes).unwrap();
        let sensor = container.sensor().unwrap();
        // The plane decodes now (AC3); develop must still refuse (DEC-049):
        // OpcodeList2 carries mandatory GainMaps no develop path applies.
        let plane = decode(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut out: Vec<u16> = Vec::new();
        for (label, result) in [
            ("colour", develop_color_into(&sensor, &plane, &mut out)),
            ("mono", develop_into(&sensor, &plane, &mut out)),
        ] {
            let err = result.expect_err(&format!("{path}: {label} develop must refuse"));
            // GainMap is opcode 9 (DNG 1.7 §6). This reader does not model it,
            // so the mandatory-flag check refuses it as an unrecognized
            // mandatory opcode (DEC-049), Unsupported-kind.
            assert!(
                matches!(err, Error::UnsupportedMandatoryOpcode { id: 9 }),
                "{path}: {label} develop gave {err:?}, expected the mandatory GainMap refusal"
            );
            assert_eq!(err.kind(), ErrorKind::Unsupported);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — no allocation on the plane path
// ─────────────────────────────────────────────────────────────────────────────

/// Allocation spellings a grep can see. Not proof against every allocation,
/// but every one this crate's code style would write.
const ALLOCATING: [&str; 7] = [
    "vec!",
    "Vec::",
    "Box::",
    ".to_vec()",
    ".collect",
    "String::",
    "format!",
];

/// The non-test source of a `src/` file (everything before its `mod tests {`).
fn production_source(file: &str) -> String {
    let path = corpus::crate_root().join("src").join(file);
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {file}: {e}"));
    let cut = src.find("\nmod tests {").unwrap_or(src.len());
    src[..cut].to_string()
}

#[test]
fn the_tile_path_allocates_nothing() {
    // DEC-016 / DEC-056: src/plane.rs's production code allocates nothing at
    // all, strips or tiles. Comments are skipped (they may name `Vec`).
    let plane = production_source("plane.rs");
    for (n, line) in plane.lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        for token in ALLOCATING {
            assert!(
                !code.contains(token),
                "src/plane.rs:{}: `{token}` in production code -- DEC-016 forbids allocation \
                 in unpack_into",
                n + 1
            );
        }
    }

    // src/ljpeg.rs parses its Huffman tables into a small Vec (pre-existing,
    // header-sized). The reconstruction it gained for tiles — the window,
    // both reconstruct loops and both decode entry points — allocates nothing.
    let ljpeg = production_source("ljpeg.rs");
    let start = ljpeg
        .find("pub(crate) struct Window")
        .expect("src/ljpeg.rs has the Window");
    let end = ljpeg
        .find("fn open_scan")
        .expect("src/ljpeg.rs has open_scan");
    for (n, line) in ljpeg[start..end].lines().enumerate() {
        let code = line.split("//").next().unwrap_or("");
        for token in ALLOCATING {
            assert!(
                !code.contains(token),
                "src/ljpeg.rs window section line {}: `{token}` -- DEC-056 decodes in place",
                n + 1
            );
        }
    }
}
