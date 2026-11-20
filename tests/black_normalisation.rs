//! `SPEC-078` — black normalisation scales by white minus the maximum black
//! (`DEC-070`, DNG 1.7.1.0 Chapter 5, "Rescaling").
//!
//! Tier A only. Every fixture is written by the SPEC-057 test writer and read
//! back through `Container::sensor`, so the levels reach the develop exactly as
//! a file's would.
//!
//! - **AC1, mono `PerPosition`:** the analytic table, each value worked by hand
//!   in the comments, on `develop_with_into` and `develop_with_scratch_into`.
//! - **AC1, colour:** an equivalence that needs no colour oracle. A file with
//!   blacks `[2047, 2047, 2047, 2048]` and a file with black 2048 everywhere and
//!   one more raw code at positions 0–2 have the same normalised plane under
//!   `DEC-070` (both are `clamp(v − 2047, 0, 12952)` over 12952 there), so all
//!   four colour entries must develop them identically. Under the old per-position
//!   rule positions 0–2 of the first file divide by 12953, and they differ. The
//!   analytic values on the colour normaliser itself are `src/color.rs`'s unit
//!   tests (the normaliser is private).
//! - **AC1, uniform:** a uniform-black file gives exactly today's values.
//! - **AC3, refusal:** a pattern whose largest black reaches white is refused
//!   though its first black is below white (`DEC-070` §3).
//!
//! **No expected value comes from calling the crate on the shape under test.**

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded,
};
use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions,
    output_dimensions_with, scratch_len, BlackLevels, DevelopOptions, Framing,
};
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;
use tiff::Order;

const WIDTH: u32 = 8;
const HEIGHT: u32 = 6;
/// `W`.
const WHITE: u32 = 15_000;
/// The AC1 pattern on `[2, 2]`, row-major: `B = 2048`, `W − B = 12952`.
const BLACKS: [u32; 4] = [2047, 2047, 2047, 2048];
const SENTINEL: u16 = 0xABCD;

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// The pattern index of raw `(x, y)`: no `ActiveArea`, so the origin is
/// `(0, 0)` and position 3 is odd row, odd column.
fn position(x: u32, y: u32) -> usize {
    ((y % 2) * 2 + (x % 2)) as usize
}

/// AC1's rows, each placed at a photosite whose black is the row's black:
/// `(x, y, sample, black, today, DEC-070)`.
///
/// `W − B = 12952`, so DEC-070 adds `12952 / 2 = 6476` and divides by 12952.
/// Today divides by `W − b`: 12953 (half 6476) at black 2047, 12952 at 2048.
const AC1_ROWS: [(u32, u32, u16, u32, u16, u16); 6] = [
    // 8523 − 2047 = 6476. DEC-070: (6476·65535 + 6476) / 12952
    //   = 424411136 / 12952 = 32768.0. Today: (6476·65535 + 6476) / 12953 = 32765.4.
    (0, 0, 8523, 2047, 32765, 32768),
    // 8524 − 2048 = 6476 over 12952 on both rules: 32768.
    (1, 1, 8524, 2048, 32768, 32768),
    // 4000 − 2047 = 1953. DEC-070: (1953·65535 + 6476) / 12952 = 127996331 / 12952
    //   = 9882.3. Today: 127996331 / 12953 = 9881.6.
    (2, 0, 4000, 2047, 9881, 9882),
    // 14999 − 2047 = 12952, which is W − B: full scale, (12952·65535 + 6476) /
    //   12952 = 65535.5 → 65535. Today: 12952 of 12953 → 65530.
    (1, 0, 14999, 2047, 65530, 65535),
    // 15000 − 2047 = 12953, clipped to 12952 (DEC-070) → 65535; today it is
    //   white exactly → 65535.
    (0, 1, 15000, 2047, 65535, 65535),
    // 2000 is below its black 2047: clamped to 0 on both rules.
    (3, 0, 2000, 2047, 0, 0),
];

/// Every other photosite: a mid-grey well inside the range at both blacks.
const FILLER: u16 = 5000;

/// The AC1 plane: [`AC1_ROWS`] at their photosites, [`FILLER`] elsewhere.
fn ac1_plane() -> Vec<u16> {
    let mut plane = vec![FILLER; (WIDTH * HEIGHT) as usize];
    for (x, y, sample, black, _, _) in AC1_ROWS {
        assert_eq!(BLACKS[position(x, y)], black, "row placed at ({x}, {y})");
        plane[(y * WIDTH + x) as usize] = sample;
    }
    plane
}

/// A forward matrix and neutral the colour develop accepts.
fn colour_meta() -> DngMeta {
    let row = |a: i32, b: i32, c: i32| [(a, 100_000), (b, 100_000), (c, 100_000)];
    DngMeta {
        white_level: Some(vec![WHITE]),
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

/// An RGGB 16-bit plane of `samples` with `BlackLevel` `blacks` on `[2, 2]`.
fn cfa(blacks: &[u32], samples: Vec<u16>) -> DngDescription {
    let mut meta = colour_meta();
    meta.black_level = Some(blacks.to_vec());
    meta.black_level_repeat_dim = Some([2, 2]);
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
        payload: Payload::Samples(samples),
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

/// The whole plane, stored order, no warp: the output is the normalised plane
/// itself, photosite for photosite.
fn plane_options(black_levels: BlackLevels) -> DevelopOptions {
    let mut o = DevelopOptions::default();
    o.framing = Framing::FullActiveArea;
    o.apply_orientation = false;
    o.warp_alpha = 0.0;
    o.black_levels = black_levels;
    o
}

/// The mono develop through `develop_with_into` and
/// `develop_with_scratch_into`, which must agree.
fn develop_mono(sensor: &Sensor, src: &[u16], options: &DevelopOptions) -> Vec<u16> {
    let (w, h) = output_dimensions_with(sensor, options).expect("dimensions");
    assert_eq!((w, h), (WIDTH, HEIGHT), "the whole plane");
    let mut dst = vec![SENTINEL; (w * h) as usize];
    develop_with_into(sensor, src, &mut dst, options).expect("develop_with_into");
    let mut scratch = vec![0u16; scratch_len(sensor, options).expect("scratch_len")];
    let mut via_scratch = vec![SENTINEL; dst.len()];
    develop_with_scratch_into(sensor, src, &mut via_scratch, &mut scratch, options)
        .expect("develop_with_scratch_into");
    assert_eq!(dst, via_scratch, "the two mono entry points disagree");
    dst
}

/// All four colour entries; threaded equals single-thread. Returns the `u16`
/// image and the `f32` image's bit patterns.
fn develop_colour(d: &DngDescription) -> (Vec<u16>, Vec<u32>) {
    let (sensor, plane) = read(d);
    let (w, h) = output_dimensions(&sensor).expect("dimensions");
    let len = (w * h * 3) as usize;
    let mut single = vec![SENTINEL; len];
    develop_color_into(&sensor, &plane, &mut single).expect("develop_color_into");
    let mut threaded = vec![SENTINEL; len];
    develop_color_into_threaded(&sensor, &plane, &mut threaded, 3)
        .expect("develop_color_into_threaded");
    assert_eq!(single, threaded, "the threaded u16 entry disagrees");
    let mut single_f = vec![-7.0f32; len];
    develop_color_f32_into(&sensor, &plane, &mut single_f).expect("develop_color_f32_into");
    let mut threaded_f = vec![-7.0f32; len];
    develop_color_f32_into_threaded(&sensor, &plane, &mut threaded_f, 3)
        .expect("develop_color_f32_into_threaded");
    let bits = |v: &[f32]| v.iter().map(|s| s.to_bits()).collect::<Vec<u32>>();
    assert_eq!(
        bits(&single_f),
        bits(&threaded_f),
        "the threaded f32 entry disagrees"
    );
    (single, bits(&single_f))
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — mono PerPosition, the analytic table
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn per_position_scales_by_white_minus_the_maximum_black() {
    let (sensor, src) = read(&cfa(&BLACKS, ac1_plane()));
    let got = develop_mono(&sensor, &src, &plane_options(BlackLevels::PerPosition));
    for (x, y, sample, black, today, dec_070) in AC1_ROWS {
        let at = got[(y * WIDTH + x) as usize];
        assert_eq!(
            at, dec_070,
            "sample {sample} at black {black} ({x}, {y}): DEC-070 gives {dec_070} (today {today})"
        );
    }
    // The filler: 5000 − 2047 = 2953 → (2953·65535 + 6476) / 12952 = 14942.2 →
    // 14942; 5000 − 2048 = 2952 → (2952·65535 + 6476) / 12952 = 14937.1 → 14937.
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if AC1_ROWS.iter().any(|r| (r.0, r.1) == (x, y)) {
                continue;
            }
            let want = if position(x, y) == 3 { 14937 } else { 14942 };
            assert_eq!(got[(y * WIDTH + x) as usize], want, "filler at ({x}, {y})");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — colour, all four entries
// ─────────────────────────────────────────────────────────────────────────────

/// The AC1 plane plus one raw code at every position whose black is 2047.
fn shifted_plane() -> Vec<u16> {
    let mut plane = ac1_plane();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if position(x, y) != 3 {
                plane[(y * WIDTH + x) as usize] += 1;
            }
        }
    }
    plane
}

#[test]
fn colour_scales_by_white_minus_the_maximum_black() {
    // Positions 0–2: clamp(v − 2047, 0, 12952) on the pattern file and
    // clamp((v + 1) − 2048, 0, 12952) on the uniform one: the same value,
    // over the same 12952. Position 3 is the same sample and black in both.
    let pattern = develop_colour(&cfa(&BLACKS, ac1_plane()));
    let uniform = develop_colour(&cfa(&[2048; 4], shifted_plane()));
    assert_eq!(pattern.0, uniform.0, "u16 colour: the two files must agree");
    assert_eq!(pattern.1, uniform.1, "f32 colour: the two files must agree");
}

#[test]
fn per_position_mono_equals_the_shifted_uniform_file() {
    // The same equivalence on the mono path, as a second, table-free witness.
    let (pattern, src) = read(&cfa(&BLACKS, ac1_plane()));
    let (uniform, shifted) = read(&cfa(&[2048; 4], shifted_plane()));
    let options = plane_options(BlackLevels::PerPosition);
    assert_eq!(
        develop_mono(&pattern, &src, &options),
        develop_mono(&uniform, &shifted, &options)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — a uniform black gives today's values
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn uniform_black_gives_todays_values() {
    // Black 2047 everywhere: B = b = 2047, W − B = 12953, today's map. The
    // AC1 table's "today" column at black 2047; 8524 → 6477 over 12953:
    // (6477·65535 + 6476) / 12953 = 32770.5 → 32770.
    let (sensor, src) = read(&cfa(&[2047; 4], ac1_plane()));
    for black_levels in [BlackLevels::PerPosition, BlackLevels::FirstValue] {
        let got = develop_mono(&sensor, &src, &plane_options(black_levels));
        for (x, y, sample, black, today, _) in AC1_ROWS {
            let want = if black == 2047 { today } else { 32770 };
            assert_eq!(
                got[(y * WIDTH + x) as usize],
                want,
                "{black_levels:?}: sample {sample} at ({x}, {y})"
            );
        }
    }
    // And `develop_into` is the FirstValue develop on the pattern file too:
    // one black, 2047, so today's map at every photosite.
    let (pattern, src) = read(&cfa(&BLACKS, ac1_plane()));
    let mut first = vec![SENTINEL; (WIDTH * HEIGHT) as usize];
    develop_into(&pattern, &src, &mut first).expect("develop_into");
    let (uniform, src) = read(&cfa(&[2047; 4], ac1_plane()));
    let mut uniform_first = vec![SENTINEL; first.len()];
    develop_into(&uniform, &src, &mut uniform_first).expect("develop_into");
    assert_eq!(first, uniform_first, "FirstValue ignores the pattern");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — the levels check uses B
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn per_position_refuses_a_maximum_black_at_white() {
    // First black 2047 < W; the largest, 15000, reaches W = 15000: no range.
    let (sensor, src) = read(&cfa(&[2047, 2047, 2047, WHITE], ac1_plane()));
    let options = plane_options(BlackLevels::PerPosition);
    let mut dst = vec![SENTINEL; (WIDTH * HEIGHT) as usize];
    let err = develop_with_into(&sensor, &src, &mut dst, &options)
        .expect_err("W ≤ B has no range to scale by");
    assert!(
        matches!(
            err,
            Error::InvalidLevels {
                black_level: WHITE,
                white_level: WHITE
            }
        ),
        "{err:?}"
    );
    assert!(dst.iter().all(|&v| v == SENTINEL), "nothing written");

    // FirstValue applies only the first black, 2047: still a valid develop.
    let first = plane_options(BlackLevels::FirstValue);
    develop_with_into(&sensor, &src, &mut dst, &first).expect("FirstValue is unaffected");
}
