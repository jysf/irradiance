//! `SPEC-086` / `SPEC-097` (`DEC-077` §3 as amended by `DEC-085`): a
//! Compression 7 strip or tile decodes when its SOF-3 frame precision `P` is at
//! most the sensor's `BitsPerSample`, and `BitsPerSample` is at most 16. A
//! precision above `BitsPerSample`, or a `BitsPerSample` above 16, is refused
//! at decode with `Error::LjpegPrecisionMismatch`.
//!
//! The develop never sees the frame header, and white defaults from
//! `BitsPerSample` when `WhiteLevel` is absent, so a P = 16 frame tagged 12-bit
//! would otherwise clip at 4095 with `Ok`. A P below `BitsPerSample` fits the
//! wider container, decodes unshifted, and white still defaults from
//! `BitsPerSample` (AC4). The check sits beside the shape binding (`DEC-047`)
//! on every path that decodes a Compression 7 unit: `unpack_into`,
//! `unpack_unit_into`, `PlaneDecoder::unpack_unit_into`, and the tile path.
//!
//! **Tier-A only**: the frames are encoded in-test (`tests/support/tiff.rs`'s
//! `lj92_strip`) and wrapped by the test-only DNG writer.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use std::num::NonZeroU32;

use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::Container;
use irradiance::plane::{units, unpack_into, unpack_unit_into, PlaneDecoder};
use irradiance::{Error, ErrorKind};
use tiff::Order;

const W: u32 = 16;
const H: u32 = 16;

/// Deterministic samples below `2^precision`.
fn samples(precision: u8) -> Vec<u16> {
    let modulo = 1u32 << precision;
    let mut state = 0x086_u32 ^ u32::from(precision);
    (0..W * H)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            ((state >> 9) % modulo) as u16
        })
        .collect()
}

/// A `W × H` LinearRaw Compression 7 DNG tagged `bits_per_sample`, whose one
/// strip (or one tile) is a P = `precision` SOF-3 frame of `plane`.
fn comp7(order: Order, tiled: bool, bits_per_sample: u32, precision: u8, plane: &[u16]) -> Vec<u8> {
    comp7_with(
        order,
        tiled,
        bits_per_sample,
        precision,
        plane,
        DngMeta::default(),
    )
}

/// [`comp7`] with caller-chosen DNG metadata (AC4's `WhiteLevel`).
fn comp7_with(
    order: Order,
    tiled: bool,
    bits_per_sample: u32,
    precision: u8,
    plane: &[u16],
    meta: DngMeta,
) -> Vec<u8> {
    comp7_components(order, tiled, bits_per_sample, precision, plane, meta, 1)
}

/// [`comp7_with`] with `nf` frame components: 1, or the two-component
/// (`Nf = 2`) column interleave both Leica files use (`SPEC-046`). `nf = 1` is
/// byte-identical to `tiff::lj92_strip`.
fn comp7_components(
    order: Order,
    tiled: bool,
    bits_per_sample: u32,
    precision: u8,
    plane: &[u16],
    meta: DngMeta,
    nf: u8,
) -> Vec<u8> {
    let chunk = tiff::lj92_strip_components(plane, W as usize, H as usize, precision, nf, 0x11);
    let organisation = if tiled {
        Organisation::Tiles {
            tile_width: W,
            tile_length: H,
            edge_fill: 0,
        }
    } else {
        Organisation::Strips { rows_per_strip: H }
    };
    write_dng(&DngDescription {
        order,
        layout: IfdLayout::SensorInIfd0,
        width: W,
        height: H,
        bits_per_sample,
        kind: PlaneKind::LinearRaw,
        organisation,
        payload: Payload::Encoded {
            compression: 7,
            chunks: vec![chunk],
        },
        meta,
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    })
    .expect("the Compression 7 fixture writes")
}

/// The three decode paths, each its own door.
#[derive(Debug, Clone, Copy)]
enum Path {
    UnpackInto,
    UnpackUnitInto,
    PlaneDecoder,
}

const PATHS: [Path; 3] = [Path::UnpackInto, Path::UnpackUnitInto, Path::PlaneDecoder];

fn decode(path: Path, file: &[u8]) -> Result<Vec<u16>, Error> {
    let container = Container::parse(file).expect("the fixture parses");
    let sensor = container.sensor().expect("the fixture has a sensor IFD");
    let order = container.byte_order();
    let mut dst = vec![0u16; (W * H) as usize];
    let whole = NonZeroU32::new(H).expect("H is not zero");
    match path {
        Path::UnpackInto => unpack_into(&sensor, order, file, &mut dst)?,
        Path::UnpackUnitInto => {
            for unit in units(&sensor, file, whole)? {
                unpack_unit_into(&sensor, order, file, &unit, &mut dst)?;
            }
        }
        Path::PlaneDecoder => {
            let decoder = PlaneDecoder::new(&sensor, order, file)?;
            for unit in decoder.units(whole) {
                decoder.unpack_unit_into(&unit, &mut dst)?;
            }
        }
    }
    Ok(dst)
}

#[track_caller]
fn assert_mismatch(r: Result<Vec<u16>, Error>, precision: u8, bits_per_sample: u32, ctx: &str) {
    match r {
        Err(e @ Error::LjpegPrecisionMismatch { .. }) => {
            assert!(
                matches!(e, Error::LjpegPrecisionMismatch { precision: p, bits_per_sample: b }
                    if p == precision && b == bits_per_sample),
                "{ctx}: {e:?}"
            );
            assert_eq!(e.kind(), ErrorKind::Corrupt, "{ctx}");
        }
        other => panic!(
            "{ctx}: expected LjpegPrecisionMismatch {{ precision: {precision}, \
             bits_per_sample: {bits_per_sample} }}, got {other:?}"
        ),
    }
}

const ORDERS: [Order; 2] = [Order::Little, Order::Big];
const LAYOUTS: [bool; 2] = [false, true];

/// Every (order, layout, path) cell of the grid must refuse `(P, BPS)` with
/// `LjpegPrecisionMismatch`, `ErrorKind::Corrupt`, and leave `dst` untouched.
#[track_caller]
fn assert_refused_everywhere(precision: u8, bits_per_sample: u32) {
    for order in ORDERS {
        for tiled in LAYOUTS {
            let file = comp7(
                order,
                tiled,
                bits_per_sample,
                precision,
                &samples(precision),
            );
            for path in PATHS {
                let ctx =
                    format!("{order:?} tiled {tiled} P{precision}/BPS{bits_per_sample} {path:?}");
                assert_mismatch(decode(path, &file), precision, bits_per_sample, &ctx);
                assert_dst_untouched(path, &file, &ctx);
            }
        }
    }
}

/// A refused decode writes nothing: `dst` keeps its sentinel fill. The check
/// runs before any sample is decoded, on every path.
#[track_caller]
fn assert_dst_untouched(path: Path, file: &[u8], ctx: &str) {
    const SENTINEL: u16 = 0xA5A5;
    let container = Container::parse(file).expect("the fixture parses");
    let sensor = container.sensor().expect("the fixture has a sensor IFD");
    let order = container.byte_order();
    let mut dst = vec![SENTINEL; (W * H) as usize];
    let whole = NonZeroU32::new(H).expect("H is not zero");
    let r = match path {
        Path::UnpackInto => unpack_into(&sensor, order, file, &mut dst),
        Path::UnpackUnitInto => units(&sensor, file, whole).and_then(|mut us| {
            us.try_for_each(|unit| unpack_unit_into(&sensor, order, file, &unit, &mut dst))
        }),
        Path::PlaneDecoder => PlaneDecoder::new(&sensor, order, file).and_then(|decoder| {
            decoder
                .units(whole)
                .try_for_each(|unit| decoder.unpack_unit_into(&unit, &mut dst))
        }),
    };
    assert!(r.is_err(), "{ctx}: expected a refusal");
    assert!(dst.iter().all(|&v| v == SENTINEL), "{ctx}: dst was written");
}

#[test]
fn ac1_precision_below_bits_per_sample_decodes_unchanged() {
    // P = 2 is the parser's floor; the encoder writes it (DEC-085 §1: any gap).
    let grid: [(u8, u32); 7] = [
        (2, 16),
        (8, 16),
        (12, 16),
        (14, 16),
        (15, 16),
        (8, 12),
        (12, 14),
    ];
    for (precision, bits_per_sample) in grid {
        let plane = samples(precision);
        for order in ORDERS {
            for tiled in LAYOUTS {
                let file = comp7(order, tiled, bits_per_sample, precision, &plane);
                for path in PATHS {
                    let ctx = format!(
                        "{order:?} tiled {tiled} P{precision}/BPS{bits_per_sample} {path:?}"
                    );
                    let got = decode(path, &file).unwrap_or_else(|e| panic!("{ctx}: {e}"));
                    // Exact equality: the samples land unshifted (DEC-085 §2).
                    assert_eq!(got, plane, "{ctx}");
                }
            }
        }
    }
}

/// `SPEC-097` verify: AC1 on the two-component (`Nf = 2`) column interleave
/// the Leica M10 and M11 use (P = 14 in 16). AC1's grid is single-component,
/// so without this a decode fault confined to `Nf = 2` frames below
/// `BitsPerSample` is caught only by the Tier-B dnglab checksum and the AC7
/// pins, which never run in CI.
#[test]
fn ac1_two_component_precision_below_bits_per_sample_decodes_unchanged() {
    for (precision, bits_per_sample) in [(14u8, 16u32), (12, 16), (8, 12)] {
        let plane = samples(precision);
        for order in ORDERS {
            for tiled in LAYOUTS {
                let file = comp7_components(
                    order,
                    tiled,
                    bits_per_sample,
                    precision,
                    &plane,
                    DngMeta::default(),
                    2,
                );
                for path in PATHS {
                    let ctx = format!(
                        "Nf2 {order:?} tiled {tiled} P{precision}/BPS{bits_per_sample} {path:?}"
                    );
                    let got = decode(path, &file).unwrap_or_else(|e| panic!("{ctx}: {e}"));
                    assert_eq!(got, plane, "{ctx}");
                }
            }
        }
    }
}

#[test]
fn ac2_precision_above_bits_per_sample_is_refused() {
    for (precision, bits_per_sample) in [(16u8, 12u32), (14, 12), (13, 12), (16, 14), (16, 15)] {
        assert_refused_everywhere(precision, bits_per_sample);
    }
}

#[test]
fn ac3_bits_per_sample_above_16_is_refused() {
    for (precision, bits_per_sample) in [(14u8, 17u32), (14, 20), (16, 17), (16, 32), (8, 32)] {
        assert_refused_everywhere(precision, bits_per_sample);
    }
}

#[test]
fn ac4_white_defaults_from_bits_per_sample_not_precision() {
    let plane = samples(12);
    for order in ORDERS {
        for tiled in LAYOUTS {
            // No WhiteLevel, no BlackLevel, no crop: white is 2^16 - 1, black
            // 0, so every sample develops to itself.
            let file = comp7(order, tiled, 16, 12, &plane);
            let container = Container::parse(&file).expect("the fixture parses");
            let sensor = container.sensor().expect("the fixture has a sensor IFD");
            let (w, h) = output_dimensions(&sensor).expect("dimensions");
            assert_eq!((w, h), (W, H), "no crop");
            let src = decode(Path::UnpackInto, &file).expect("P12 in BPS16 unpacks");
            let mut dst = vec![0u16; (w * h) as usize];
            develop_into(&sensor, &src, &mut dst).expect("develops");
            assert_eq!(
                dst, plane,
                "{order:?} tiled {tiled}: white defaults from BitsPerSample"
            );

            // The same frame with WhiteLevel 4095: round(min(v, 4095) * 65535 / 4095).
            let meta = DngMeta {
                white_level: Some(vec![4095]),
                ..DngMeta::default()
            };
            let file = comp7_with(order, tiled, 16, 12, &plane, meta);
            let container = Container::parse(&file).expect("the fixture parses");
            let sensor = container.sensor().expect("the fixture has a sensor IFD");
            let src = decode(Path::UnpackInto, &file).expect("P12 in BPS16 unpacks");
            let mut dst = vec![0u16; (W * H) as usize];
            develop_into(&sensor, &src, &mut dst).expect("develops");
            let want: Vec<u16> = plane
                .iter()
                .map(|&v| {
                    let v = u64::from(v.min(4095));
                    u16::try_from((v * 65535 + 4095 / 2) / 4095).expect("fits u16")
                })
                .collect();
            assert_eq!(
                dst, want,
                "{order:?} tiled {tiled}: an explicit WhiteLevel 4095"
            );
        }
    }
}

#[test]
fn ac5_matching_precision_still_decodes() {
    for order in [Order::Little, Order::Big] {
        for tiled in [false, true] {
            for precision in [12u8, 14, 16] {
                let plane = samples(precision);
                let file = comp7(order, tiled, u32::from(precision), precision, &plane);
                for path in PATHS {
                    let got = decode(path, &file).unwrap_or_else(|e| {
                        panic!("{order:?} tiled {tiled} P{precision} {path:?}: {e}")
                    });
                    assert_eq!(got, plane, "{order:?} tiled {tiled} P{precision} {path:?}");
                }
            }
        }
    }
}

#[test]
fn precision_mismatch_display_names_both_values() {
    let msg = Error::LjpegPrecisionMismatch {
        precision: 16,
        bits_per_sample: 12,
    }
    .to_string();
    assert!(msg.contains("16"), "{msg}");
    assert!(msg.contains("12"), "{msg}");
}

/// `SPEC-097` verify: DEC-085 §3 on both develop paths.
///
/// AC4 above pins the mono path at `BitsPerSample` 16 only, where "white is
/// `2^BitsPerSample − 1`" and "white is always 65535" agree. This adds a
/// 12-bit container (P = 8) on the mono path, checked analytically, and the
/// colour path (`develop_color_into`, `resolve_colour_levels`) at P = 12 in 16,
/// which AC4 does not reach. The colour half is checked against the tag
/// rather than the colour arithmetic: with no `WhiteLevel` the frame must
/// develop exactly as with an explicit `WhiteLevel` of `2^16 − 1`, and
/// differently from one of `2^P − 1`.
#[test]
fn ac4_white_default_is_bits_per_sample_on_mono_and_colour() {
    use irradiance::color::develop_color_into;

    let develop_mono = |bits_per_sample: u32, precision: u8, white: Option<u32>| -> Vec<u16> {
        let meta = DngMeta {
            white_level: white.map(|w| vec![w]),
            ..DngMeta::default()
        };
        let file = comp7_with(
            Order::Little,
            false,
            bits_per_sample,
            precision,
            &samples(precision),
            meta,
        );
        let container = Container::parse(&file).expect("the fixture parses");
        let sensor = container.sensor().expect("the fixture has a sensor IFD");
        let src = decode(Path::UnpackInto, &file).expect("P below BPS unpacks");
        let mut dst = vec![0u16; (W * H) as usize];
        develop_into(&sensor, &src, &mut dst).expect("develops");
        dst
    };
    // Mono, P = 8 in a 12-bit container, analytically (DEC-004): white is
    // 2^12 - 1 = 4095, black 0, so v develops to round(v * 65535 / 4095).
    // White 2^P - 1 = 255 or a fixed 65535 would each give other values.
    let want: Vec<u16> = samples(8)
        .iter()
        .map(|&v| u16::try_from((u64::from(v) * 65535 + 4095 / 2) / 4095).expect("fits u16"))
        .collect();
    assert_eq!(
        develop_mono(12, 8, None),
        want,
        "mono P8/BPS12: no WhiteLevel must develop against white 2^12 - 1"
    );

    // Colour: an RGGB Compression 7 plane, P = 12 in a 16-bit container.
    let develop_colour = |white: Option<u32>| -> Vec<u16> {
        let row = |a: i32, b: i32, c: i32| [(a, 100_000), (b, 100_000), (c, 100_000)];
        let meta = DngMeta {
            white_level: white.map(|w| vec![w]),
            forward_matrix_1: Some(
                [
                    row(60_000, 30_000, 6_422),
                    row(20_000, 70_000, 10_000),
                    row(10_000, 10_000, 62_521),
                ]
                .concat(),
            ),
            as_shot_neutral: Some(vec![(1, 2), (1, 1), (1, 2)]),
            ..DngMeta::default()
        };
        let chunk = tiff::lj92_strip(&samples(12), W as usize, H as usize, 12);
        let file = write_dng(&DngDescription {
            order: Order::Little,
            layout: IfdLayout::SensorInIfd0,
            width: W,
            height: H,
            bits_per_sample: 16,
            kind: PlaneKind::Cfa {
                repeat_dim: [2, 2],
                pattern: vec![0, 1, 1, 2],
                plane_color: vec![0, 1, 2],
            },
            organisation: Organisation::Strips { rows_per_strip: H },
            payload: Payload::Encoded {
                compression: 7,
                chunks: vec![chunk],
            },
            meta,
            opcode_lists: [None, None, None],
            extra_tags: Vec::new(),
            fault: None,
        })
        .expect("the colour Compression 7 fixture writes");
        let container = Container::parse(&file).expect("the fixture parses");
        let sensor = container.sensor().expect("the fixture has a sensor IFD");
        let mut src = vec![0u16; (W * H) as usize];
        unpack_into(&sensor, container.byte_order(), &file, &mut src)
            .expect("P12 in BPS16 unpacks");
        let (w, h) = output_dimensions(&sensor).expect("dimensions");
        let mut rgb = vec![0u16; (w * h * 3) as usize];
        develop_color_into(&sensor, &src, &mut rgb).expect("the colour develop runs");
        rgb
    };
    let default = develop_colour(None);
    assert_eq!(
        default,
        develop_colour(Some(65535)),
        "colour P12/BPS16: no WhiteLevel must develop as WhiteLevel 2^16 - 1"
    );
    assert_ne!(
        default,
        develop_colour(Some(4095)),
        "colour P12/BPS16: the default must not be 2^P - 1"
    );
}
