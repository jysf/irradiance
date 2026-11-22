//! `SPEC-086` AC5 (`DEC-077` §3): a Compression 7 strip or tile whose SOF-3
//! frame precision `P` differs from the sensor's `BitsPerSample` is refused at
//! decode with `Error::LjpegPrecisionMismatch`.
//!
//! The develop never sees the frame header, and white defaults from
//! `BitsPerSample` when `WhiteLevel` is absent, so a P = 16 frame tagged 12-bit
//! would otherwise clip at 4095 with `Ok`. The check sits beside the shape
//! binding (`DEC-047`) on every path that decodes a Compression 7 unit:
//! `unpack_into`, `unpack_unit_into`, `PlaneDecoder::unpack_unit_into`, and the
//! tile path.
//!
//! **Tier-A only**: the frames are encoded in-test (`tests/support/tiff.rs`'s
//! `lj92_strip`) and wrapped by the test-only DNG writer.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use std::num::NonZeroU32;

use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
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
    let chunk = tiff::lj92_strip(plane, W as usize, H as usize, precision);
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
        meta: DngMeta::default(),
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

#[test]
fn ac5_precision_must_equal_bits_per_sample_on_every_decode_path() {
    for order in [Order::Little, Order::Big] {
        for tiled in [false, true] {
            // P = 16 tagged 12-bit: would clip at 4095 with Ok.
            let file = comp7(order, tiled, 12, 16, &samples(16));
            for path in PATHS {
                let ctx = format!("{order:?} tiled {tiled} P16/BPS12 {path:?}");
                assert_mismatch(decode(path, &file), 16, 12, &ctx);
            }
            // P = 12 tagged 16-bit.
            let file = comp7(order, tiled, 16, 12, &samples(12));
            for path in PATHS {
                let ctx = format!("{order:?} tiled {tiled} P12/BPS16 {path:?}");
                assert_mismatch(decode(path, &file), 12, 16, &ctx);
            }
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
