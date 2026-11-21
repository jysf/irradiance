//! `SPEC-080` — the colour develop refuses `WhiteLevel` at or below the
//! maximum applied black (`DEC-070` §3, second amendment), as the mono path's
//! `PerPosition` option already did.
//!
//! Tier A only: every fixture is written by the SPEC-057 test writer and read
//! back through `Container::sensor`. The scalar branch (an empty `BlackLevel`
//! list with a scalar `black_level`) and the colour white rule (the first
//! `white_levels` value wins over `white_level`) are reached by editing the
//! parsed `Sensor`'s public fields, because no file the writer can produce
//! separates them.
//!
//! Every refusal is checked on all four colour entries with a sentinel-filled
//! `dst`, so "before any write" is observed, not assumed. Every refusing
//! fixture with more than one black has its maximum black at neither the first
//! nor the last pattern index and its first black below `W`, so a check made
//! against the first (or last) black does not refuse it.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/tiff.rs"]
mod tiff;

use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded,
};
use irradiance::develop::output_dimensions;
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;
use opcode_support::{opcode_list, raw_opcode};
use tiff::Order;

const WIDTH: u32 = 14;
const HEIGHT: u32 = 11;
/// `ActiveArea` `[top, left, bottom, right]`: an odd origin, as in
/// `tests/colour_black_pattern.rs`.
const ACTIVE: [u32; 4] = [1, 3, 11, 13];
const SENTINEL: u16 = 0xABCD;
const SENTINEL_F32: f32 = -7.0;
const THREADS: usize = 3;

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A forward matrix and neutral the colour develop accepts.
fn colour_meta(white: u32) -> DngMeta {
    let row = |a: i32, b: i32, c: i32| [(a, 100_000), (b, 100_000), (c, 100_000)];
    DngMeta {
        white_level: Some(vec![white]),
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

/// An RGGB plane holding `samples` (or a ramp), `BlackLevel` (LONG) `blacks`
/// on `repeat`, no `BlackLevel` when `blacks` is empty, and one `WhiteLevel`.
fn cfa(blacks: &[u32], repeat: Option<[u16; 2]>, white: u32, samples: Vec<u16>) -> DngDescription {
    let mut meta = colour_meta(white);
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
        payload: Payload::Samples(samples),
        meta,
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

/// A ramp, distinct at every photosite.
fn ramp() -> Vec<u16> {
    (0..WIDTH * HEIGHT)
        .map(|i| u16::try_from(3000 + (i * 331) % 50_000).unwrap())
        .collect()
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

/// One refusing (or boundary) case: a branch of `resolve_colour_levels`, its
/// blacks, and the `B` it must report.
struct Case {
    name: &'static str,
    /// `BlackLevel` values; empty means the scalar branch (built by editing
    /// the sensor, see [`sensor_for`]).
    blacks: &'static [u32],
    repeat: Option<[u16; 2]>,
    /// The maximum black the branch applies.
    max_black: u32,
}

/// The scalar branch's black, set on the sensor directly.
const SCALAR_BLACK: u32 = 5000;

/// Each branch, with `B = 5000` and the first black below every `W` used.
const CASES: [Case; 4] = [
    Case {
        name: "scalar BlackLevel, no pattern",
        blacks: &[],
        repeat: None,
        max_black: SCALAR_BLACK,
    },
    Case {
        // B at index 1 of 4: neither first nor last.
        name: "2x2 slot pattern",
        blacks: &[1000, 5000, 1200, 1100],
        repeat: Some([2, 2]),
        max_black: 5000,
    },
    Case {
        // Two values: B is the last, the first is below W.
        name: "[1, 2] pattern",
        blacks: &[1000, 5000],
        repeat: Some([1, 2]),
        max_black: 5000,
    },
    Case {
        // B at index 6 of 16: neither first nor last; the first is the lowest.
        name: "general 4x4 pattern",
        blacks: &[
            900, 1000, 1100, 1200, 1300, 1400, 5000, 1500, 1600, 1700, 1800, 1900, 2000, 2100,
            2200, 2300,
        ],
        repeat: Some([4, 4]),
        max_black: 5000,
    },
];

/// The parsed sensor for `case` at colour white `white`, over `samples`.
/// The scalar branch is the file with no `BlackLevel`, then a scalar black
/// set on the sensor: `black_levels` stays empty.
fn sensor_for(case: &Case, white: u32, samples: Vec<u16>) -> (Sensor, Vec<u16>) {
    let (mut sensor, plane) = read(&cfa(case.blacks, case.repeat, white, samples));
    if case.blacks.is_empty() {
        assert!(sensor.black_levels.is_empty(), "{}", case.name);
        sensor.black_level = Some(SCALAR_BLACK);
    }
    (sensor, plane)
}

/// Run all four colour entries on `sensor` with sentinel-filled buffers,
/// assert each returns exactly `InvalidLevels { black_level, white_level }`,
/// and that no buffer was touched.
fn assert_refused_everywhere(sensor: &Sensor, plane: &[u16], black: u32, white: u32, ctx: &str) {
    let want = format!(
        "{:?}",
        Error::InvalidLevels {
            black_level: black,
            white_level: white,
        }
    );
    let len = rgb_len(sensor);

    let mut dst = vec![SENTINEL; len];
    let err = develop_color_into(sensor, plane, &mut dst).expect_err(ctx);
    assert_eq!(format!("{err:?}"), want, "{ctx}: develop_color_into");
    assert!(dst.iter().all(|&v| v == SENTINEL), "{ctx}: u16 dst touched");

    let mut dst = vec![SENTINEL_F32; len];
    let err = develop_color_f32_into(sensor, plane, &mut dst).expect_err(ctx);
    assert_eq!(format!("{err:?}"), want, "{ctx}: develop_color_f32_into");
    assert!(
        dst.iter().all(|&v| v == SENTINEL_F32),
        "{ctx}: f32 dst touched"
    );

    let mut dst = vec![SENTINEL; len];
    let err = develop_color_into_threaded(sensor, plane, &mut dst, THREADS).expect_err(ctx);
    assert_eq!(
        format!("{err:?}"),
        want,
        "{ctx}: develop_color_into_threaded"
    );
    assert!(
        dst.iter().all(|&v| v == SENTINEL),
        "{ctx}: threaded u16 dst touched"
    );

    let mut dst = vec![SENTINEL_F32; len];
    let err = develop_color_f32_into_threaded(sensor, plane, &mut dst, THREADS).expect_err(ctx);
    assert_eq!(
        format!("{err:?}"),
        want,
        "{ctx}: develop_color_f32_into_threaded"
    );
    assert!(
        dst.iter().all(|&v| v == SENTINEL_F32),
        "{ctx}: threaded f32 dst touched"
    );
}

/// The four entries' outputs on `sensor`, each required `Ok` and fully
/// written; the u16 single and threaded outputs must agree, and so must the
/// f32 pair.
fn develop_everywhere(sensor: &Sensor, plane: &[u16], ctx: &str) -> (Vec<u16>, Vec<f32>) {
    let len = rgb_len(sensor);
    let mut single = vec![SENTINEL; len];
    develop_color_into(sensor, plane, &mut single)
        .unwrap_or_else(|e| panic!("{ctx}: develop_color_into: {e:?}"));
    let mut threaded = vec![SENTINEL; len];
    develop_color_into_threaded(sensor, plane, &mut threaded, THREADS)
        .unwrap_or_else(|e| panic!("{ctx}: develop_color_into_threaded: {e:?}"));
    assert_eq!(single, threaded, "{ctx}: threaded u16 disagrees");

    let mut single_f = vec![SENTINEL_F32; len];
    develop_color_f32_into(sensor, plane, &mut single_f)
        .unwrap_or_else(|e| panic!("{ctx}: develop_color_f32_into: {e:?}"));
    let mut threaded_f = vec![SENTINEL_F32; len];
    develop_color_f32_into_threaded(sensor, plane, &mut threaded_f, THREADS)
        .unwrap_or_else(|e| panic!("{ctx}: develop_color_f32_into_threaded: {e:?}"));
    assert_eq!(single_f, threaded_f, "{ctx}: threaded f32 disagrees");
    assert!(
        single_f.iter().all(|&v| v != SENTINEL_F32),
        "{ctx}: f32 dst not fully written"
    );
    (single, single_f)
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — W ≤ B is refused on every entry and branch, reporting B
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn white_equal_to_the_maximum_black_is_refused_on_every_entry_and_branch() {
    for case in &CASES {
        let white = case.max_black;
        let (sensor, plane) = sensor_for(case, white, ramp());
        let ctx = format!("{}: W = B = {white}", case.name);
        assert_refused_everywhere(&sensor, &plane, case.max_black, white, &ctx);
    }
}

#[test]
fn white_below_the_maximum_black_is_refused_on_every_entry_and_branch() {
    for case in &CASES {
        // Above every black but B, so only the maximum is at or above W.
        let white = case.max_black - 1500;
        let (sensor, plane) = sensor_for(case, white, ramp());
        let ctx = format!("{}: W = {white} < B", case.name);
        assert_refused_everywhere(&sensor, &plane, case.max_black, white, &ctx);
    }
}

#[test]
fn a_whitelevel_of_zero_with_no_blacklevel_is_refused() {
    // The scalar branch straight from a file: no BlackLevel (B = 0) and
    // WhiteLevel 0, so W = B.
    let (sensor, plane) = read(&cfa(&[], None, 0, ramp()));
    assert!(sensor.black_levels.is_empty());
    assert_refused_everywhere(&sensor, &plane, 0, 0, "no BlackLevel, WhiteLevel 0");
}

#[test]
fn the_colour_white_is_the_first_whitelevel_value() {
    // DEC-061 §3: the colour white is white_levels' first value, then
    // white_level. Here they differ; only the first is compared with B.
    for case in &CASES {
        // white_levels says 4000 (≤ B), white_level says 60000: refused,
        // reporting 4000.
        let (mut sensor, plane) = sensor_for(case, 60_000, ramp());
        sensor.white_levels = vec![4000];
        sensor.white_level = Some(60_000);
        let ctx = format!("{}: white_levels [4000], white_level 60000", case.name);
        assert_refused_everywhere(&sensor, &plane, case.max_black, 4000, &ctx);

        // The reverse: white_levels says 60000, white_level 4000. Not refused.
        let (mut sensor, plane) = sensor_for(case, 60_000, ramp());
        sensor.white_levels = vec![60_000];
        sensor.white_level = Some(4000);
        let ctx = format!("{}: white_levels [60000], white_level 4000", case.name);
        develop_everywhere(&sensor, &plane, &ctx);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — the boundary W = B + 1 develops, and a sample at or above W is 65535
// ─────────────────────────────────────────────────────────────────────────────

/// A plane of one value everywhere.
fn flat(value: u16) -> Vec<u16> {
    vec![value; (WIDTH * HEIGHT) as usize]
}

#[test]
fn white_one_above_the_maximum_black_develops_on_every_entry_and_branch() {
    for case in &CASES {
        let white = case.max_black + 1;
        let w16 = u16::try_from(white).unwrap();
        let ctx = format!("{}: W = B + 1 = {white}", case.name);

        // The ramp develops (every sample is above every black here, most far
        // above W, so they all clip).
        let (sensor, plane) = sensor_for(case, white, ramp());
        develop_everywhere(&sensor, &plane, &ctx);

        // Every sample at or above W normalises to 65535: a plane of W and a
        // plane of 65535 develop identically, on every entry.
        let (sensor, at_white) = sensor_for(case, white, flat(w16));
        let (_, at_max) = sensor_for(case, white, flat(u16::MAX));
        let (_, above) = sensor_for(case, white, flat(w16 + 7));
        let reference = develop_everywhere(&sensor, &at_max, &ctx);
        assert_eq!(
            develop_everywhere(&sensor, &at_white, &ctx),
            reference,
            "{ctx}: a sample at W is not 65535"
        );
        assert_eq!(
            develop_everywhere(&sensor, &above, &ctx),
            reference,
            "{ctx}: a sample above W is not 65535"
        );
        // And the range is not empty: a plane of 0 (below every black)
        // develops to something different from the clipped plane.
        let (_, zero) = sensor_for(case, white, flat(0));
        assert_ne!(
            develop_everywhere(&sensor, &zero, &ctx).0,
            reference.0,
            "{ctx}: black and white develop alike"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — order: earlier checks win; levels come before the GainMaps
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn a_wrong_dst_length_is_reported_before_the_levels() {
    for case in &CASES {
        let (sensor, plane) = sensor_for(case, case.max_black, ramp());
        let len = rgb_len(&sensor) + 1;
        let ctx = case.name;
        let want = |e: Error| {
            assert!(
                matches!(e, Error::RgbBufferWrongLength { .. }),
                "{ctx}: {e:?}"
            );
        };
        let mut dst = vec![SENTINEL; len];
        want(develop_color_into(&sensor, &plane, &mut dst).unwrap_err());
        let mut dst = vec![SENTINEL_F32; len];
        want(develop_color_f32_into(&sensor, &plane, &mut dst).unwrap_err());
        let mut dst = vec![SENTINEL; len];
        want(develop_color_into_threaded(&sensor, &plane, &mut dst, THREADS).unwrap_err());
        let mut dst = vec![SENTINEL_F32; len];
        want(develop_color_f32_into_threaded(&sensor, &plane, &mut dst, THREADS).unwrap_err());
    }
}

#[test]
fn a_wrong_blacklevel_count_is_reported_before_the_levels() {
    // Three values on a [2, 2] repeat, all above W: the pattern refusal wins.
    let (sensor, plane) = read(&cfa(&[5000, 6000, 7000], Some([2, 2]), 4000, ramp()));
    let len = rgb_len(&sensor);
    let want = |e: Error| {
        assert!(
            matches!(
                e,
                Error::InvalidBlackLevelPattern {
                    repeat_rows: 2,
                    repeat_cols: 2,
                    count: 3
                }
            ),
            "{e:?}"
        );
    };
    let mut dst = vec![SENTINEL; len];
    want(develop_color_into(&sensor, &plane, &mut dst).unwrap_err());
    let mut dst = vec![SENTINEL_F32; len];
    want(develop_color_f32_into(&sensor, &plane, &mut dst).unwrap_err());
    let mut dst = vec![SENTINEL; len];
    want(develop_color_into_threaded(&sensor, &plane, &mut dst, THREADS).unwrap_err());
    let mut dst = vec![SENTINEL_F32; len];
    want(develop_color_f32_into_threaded(&sensor, &plane, &mut dst, THREADS).unwrap_err());
}

/// A well-framed `OpcodeList2` holding one optional `GainMap` (id 9) whose
/// three parameter bytes cannot be a `GainMap`: the list itself parses, so
/// the required-processing check passes it, and the `GainMap` step refuses it.
fn malformed_gain_map_list() -> Vec<u8> {
    const ID_GAIN_MAP: u32 = 9;
    const OPTIONAL: u32 = 1;
    opcode_list(&[raw_opcode(ID_GAIN_MAP, 0x0103_0000, OPTIONAL, &[0, 0, 0])])
}

#[test]
fn the_levels_are_refused_before_a_malformed_gain_map() {
    for case in &CASES {
        // Control: with valid levels the malformed list is itself an error,
        // and not InvalidLevels, so the next half's answer means something.
        let (mut sensor, plane) = sensor_for(case, 60_000, ramp());
        sensor.opcode_list_2 = Some(malformed_gain_map_list());
        let mut dst = vec![SENTINEL; rgb_len(&sensor)];
        let err = develop_color_into(&sensor, &plane, &mut dst)
            .expect_err("a malformed OpcodeList2 is refused");
        assert!(
            matches!(err, Error::MalformedOpcodeParams { id: 9, .. }),
            "{}: control: {err:?}",
            case.name
        );

        // W ≤ B and the same malformed list: the levels are reported.
        let (mut sensor, plane) = sensor_for(case, case.max_black, ramp());
        sensor.opcode_list_2 = Some(malformed_gain_map_list());
        let ctx = format!("{}: W = B and a malformed OpcodeList2", case.name);
        assert_refused_everywhere(&sensor, &plane, case.max_black, case.max_black, &ctx);
    }
}
