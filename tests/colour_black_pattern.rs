//! `SPEC-075` — the colour path indexes `BlackLevel` by `BlackLevelRepeatDim`
//! (`DEC-068` §2, `DEC-061` §3).
//!
//! Tier A only: every fixture is written by the SPEC-057 test writer and read
//! back through `Container::sensor`, so the pattern reaches the colour develop
//! exactly as a file's would. No held DNG has a `[1, 2]`, `[2, 1]` or larger
//! pattern.
//!
//! **No expected value comes from calling the crate on the shape under test**
//! (`shared-path-oracle-tautology`). Each equivalence develops two files whose
//! DNG meaning is the same: one uses the new shape, the other the `[2, 2]` shape
//! the held corpus already pins (the Canon). Every fixture has an **odd**
//! `ActiveArea` origin and non-uniform blacks, so a pattern phased from the
//! plane's top-left, or mis-indexed, shows.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag, TagIfd,
};
use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded,
};
use irradiance::develop::output_dimensions;
use irradiance::ifd::{Container, Sensor, TAG_BLACK_LEVEL};
use irradiance::plane::unpack_into;
use irradiance::Error;
use tiff::Order;

const WIDTH: u32 = 14;
const HEIGHT: u32 = 11;
/// `ActiveArea` `[top, left, bottom, right]`: origin (left 3, top 1), odd on
/// both axes, 10×10.
const ACTIVE: [u32; 4] = [1, 3, 11, 13];
const WHITE: u32 = 60_000;
const SENTINEL: u16 = 0xABCD;
const SENTINEL_F32: f32 = -7.0;

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A ramp, distinct at every photosite, from 3000 up, below `WHITE`.
fn ramp() -> Vec<u16> {
    (0..WIDTH * HEIGHT)
        .map(|i| u16::try_from(3000 + (i * 331) % 50_000).unwrap())
        .collect()
}

/// A forward matrix and neutral the colour develop accepts. Both files of a
/// comparison share them, so their values only need to be well-formed.
fn colour_meta() -> DngMeta {
    let row = |a: i32, b: i32, c: i32| [(a, 100_000), (b, 100_000), (c, 100_000)];
    DngMeta {
        white_level: Some(vec![WHITE]),
        active_area: Some(ACTIVE),
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
    }
}

/// An RGGB plane with `BlackLevel` (LONG) `blacks` on `repeat`, or no
/// `BlackLevel` at all when `blacks` is empty.
fn cfa(blacks: &[u32], repeat: Option<[u16; 2]>) -> DngDescription {
    let mut meta = colour_meta();
    meta.black_level = (!blacks.is_empty()).then(|| blacks.to_vec());
    meta.black_level_repeat_dim = repeat;
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width: WIDTH,
        height: HEIGHT,
        bits_per_sample: 16,
        kind: PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        },
        organisation: Organisation::Strips {
            rows_per_strip: HEIGHT,
        },
        payload: Payload::Samples(ramp()),
        meta,
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

fn read(d: &DngDescription) -> (Sensor, Vec<u16>) {
    let file = write_dng(d).expect("write");
    let container = Container::parse(&file).expect("parse");
    let sensor = container.sensor().expect("sensor");
    let mut plane = vec![0u16; (sensor.width * sensor.height) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpack");
    (sensor, plane)
}

fn rgb_len(sensor: &Sensor) -> usize {
    let (w, h) = output_dimensions(sensor).expect("dimensions");
    (w * h * 3) as usize
}

/// The `u16` colour develop of `d`, through the single-thread and the
/// threaded entry, which must agree.
fn develop(d: &DngDescription) -> Vec<u16> {
    let (sensor, plane) = read(d);
    let mut single = vec![SENTINEL; rgb_len(&sensor)];
    develop_color_into(&sensor, &plane, &mut single).expect("develop_color_into");
    let mut threaded = vec![SENTINEL; single.len()];
    develop_color_into_threaded(&sensor, &plane, &mut threaded, 3)
        .expect("develop_color_into_threaded");
    assert_eq!(single, threaded, "the threaded entry disagrees");
    single
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — each pattern shape develops as its 2×2 expansion
// ─────────────────────────────────────────────────────────────────────────────

const A: u32 = 1000;
const B: u32 = 1700;
const C: u32 = 2300;
const D: u32 = 2900;

#[test]
fn one_by_two_pattern_equals_its_two_by_two_expansion() {
    // [1, 2] [a, b]: every row is a b a b ..., so its 2×2 form is [a, b, a, b].
    let got = develop(&cfa(&[A, B], Some([1, 2])));
    let want = develop(&cfa(&[A, B, A, B], Some([2, 2])));
    assert_eq!(got, want, "[1, 2] must develop as [2, 2] [a, b, a, b]");
}

#[test]
fn two_by_one_pattern_equals_its_two_by_two_expansion() {
    // [2, 1] [a, b]: rows alternate a and b, so its 2×2 form is [a, a, b, b].
    let got = develop(&cfa(&[A, B], Some([2, 1])));
    let want = develop(&cfa(&[A, A, B, B], Some([2, 2])));
    assert_eq!(got, want, "[2, 1] must develop as [2, 2] [a, a, b, b]");
}

#[test]
fn four_by_four_pattern_takes_the_general_path() {
    // The 2×2 [[a, b], [c, d]] tiled four times as a 4×4 pattern, row-major.
    #[rustfmt::skip]
    let tiled = [
        A, B, A, B,
        C, D, C, D,
        A, B, A, B,
        C, D, C, D,
    ];
    let got = develop(&cfa(&tiled, Some([4, 4])));
    let want = develop(&cfa(&[A, B, C, D], Some([2, 2])));
    assert_eq!(got, want, "a tiled [4, 4] must develop as its [2, 2] tile");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6 — a count mismatch is refused, everywhere, before any write
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn colour_pattern_count_mismatch_is_refused_on_every_entry_point() {
    let cases: [(&[u32], [u16; 2]); 2] = [(&[A, B, C], [2, 2]), (&[A, B, C, D], [1, 2])];
    for (blacks, repeat) in cases {
        let ctx = format!("{repeat:?} with {} values", blacks.len());
        let (sensor, plane) = read(&cfa(blacks, Some(repeat)));
        let want = |e: Error| match e {
            Error::InvalidBlackLevelPattern {
                repeat_rows,
                repeat_cols,
                count,
            } => {
                assert_eq!(
                    (repeat_rows, repeat_cols, count),
                    (u32::from(repeat[0]), u32::from(repeat[1]), blacks.len()),
                    "{ctx}"
                );
            }
            other => panic!("{ctx}: expected InvalidBlackLevelPattern, got {other:?}"),
        };
        let len = rgb_len(&sensor);

        let mut dst = vec![SENTINEL; len];
        want(develop_color_into(&sensor, &plane, &mut dst).unwrap_err());
        assert!(dst.iter().all(|&v| v == SENTINEL), "{ctx}: u16 dst touched");

        let mut dst = vec![SENTINEL; len];
        want(develop_color_into_threaded(&sensor, &plane, &mut dst, 3).unwrap_err());
        assert!(
            dst.iter().all(|&v| v == SENTINEL),
            "{ctx}: threaded dst touched"
        );

        let mut dst = vec![SENTINEL_F32; len];
        want(develop_color_f32_into(&sensor, &plane, &mut dst).unwrap_err());
        assert!(
            dst.iter().all(|&v| v == SENTINEL_F32),
            "{ctx}: f32 dst touched"
        );

        let mut dst = vec![SENTINEL_F32; len];
        want(develop_color_f32_into_threaded(&sensor, &plane, &mut dst, 3).unwrap_err());
        assert!(
            dst.iter().all(|&v| v == SENTINEL_F32),
            "{ctx}: threaded f32 dst touched"
        );
    }
}

#[test]
fn four_values_with_no_repeat_dim_are_refused() {
    // An absent BlackLevelRepeatDim is the DNG default [1, 1], so four values
    // disagree with it (DEC-068 §2). Before SPEC-075 the colour path read them
    // as a 2×2 slot pattern.
    let (sensor, plane) = read(&cfa(&[A, B, C, D], None));
    assert_eq!(sensor.black_level_repeat_dim, None);
    let mut dst = vec![SENTINEL; rgb_len(&sensor)];
    let err = develop_color_into(&sensor, &plane, &mut dst).unwrap_err();
    assert!(
        matches!(
            err,
            Error::InvalidBlackLevelPattern {
                repeat_rows: 1,
                repeat_cols: 1,
                count: 4
            }
        ),
        "{err:?}"
    );
    assert!(dst.iter().all(|&v| v == SENTINEL), "dst touched");
}

#[test]
fn empty_black_level_still_develops_with_black_zero() {
    // No BlackLevel tag at all, against an explicit single black of 0.
    let absent = develop(&cfa(&[], None));
    let zero = develop(&cfa(&[0], None));
    assert_eq!(absent, zero);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — a fractional black develops as its rounded value, in colour
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fractional_black_develops_as_its_rounded_value_in_colour() {
    // 1024.55, 1024.5, 1024.1, 1023.95 round to 1025, 1025, 1024, 1024.
    let written: [(u32, u32); 4] = [(20_491, 20), (2_049, 2), (10_241, 10), (20_479, 20)];
    let mut bytes = Vec::new();
    for (num, den) in written {
        bytes.extend_from_slice(&num.to_le_bytes());
        bytes.extend_from_slice(&den.to_le_bytes());
    }
    let mut fractional = cfa(&[], Some([2, 2]));
    fractional.extra_tags = vec![RawTag {
        ifd: TagIfd::Sensor,
        tag: TAG_BLACK_LEVEL,
        field_type: 5,
        count: 4,
        bytes,
    }];
    let fractional = develop(&fractional);
    let rounded = develop(&cfa(&[1025, 1025, 1024, 1024], Some([2, 2])));
    let black_zero = develop(&cfa(&[0, 0, 0, 0], Some([2, 2])));
    assert_ne!(
        rounded, black_zero,
        "control: the fixture tells black 1024 from black 0"
    );
    assert_ne!(
        fractional, black_zero,
        "a fractional black must not develop as black 0 (the pre-SPEC-075 read)"
    );
    assert_eq!(
        fractional, rounded,
        "a fractional black develops as its rounded value (DEC-068 §1)"
    );
}
