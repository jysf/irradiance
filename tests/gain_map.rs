//! `SPEC-059` — `OpcodeList2` `GainMap` (OpcodeID 9) on both develop paths,
//! proven against a hand-written oracle (`DEC-069`).
//!
//! **The oracle never calls the crate.** [`oracle`] is a direct transcription
//! of `DEC-069` §2, written here and shaped differently from `src/` on
//! purpose: it walks each map's area by stepping the pitch from the area's
//! top-left, one whole-image pass per map, where the crate tests every sample
//! against every map with a remainder. A helper shared with `src/` would make
//! the oracle unable to go red (`shared-path-oracle-tautology`).
//!
//! - **AC1**: the parser round-trips writer-built parameters, and each
//!   `DEC-069` §3 class is `MalformedOpcodeParams { id: 9, .. }`.
//! - **AC2**: [`irradiance::develop::apply_gain_maps`] equals the oracle bit
//!   for bit on every shape the rule distinguishes.
//! - **AC3**: hand values, each with its arithmetic.
//! - **AC4** (mono) and **AC5** (colour): a file with `GainMap`s develops to
//!   exactly what a **pre-gained twin** develops to. The twin has black 0 and
//!   white 65535, which makes level normalisation the identity, so its raw
//!   `ActiveArea` window can hold the oracle's gained stage 2 exactly, and
//!   the existing warp, demosaic, crop and orientation run on it unchanged.
//!
//! Tier A only: every fixture is written by the `SPEC-057` writer and read
//! back through `Container::sensor`.

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
use irradiance::develop::{
    apply_gain_maps, develop_into, develop_with_into, develop_with_scratch_into,
    output_dimensions_with, scratch_len, DevelopOptions, Framing,
};
use irradiance::ifd::{Container, Sensor};
use irradiance::opcode::{parse_opcode_list, Opcode};
use irradiance::plane::unpack_into;
use irradiance::Error;
use opcode_support::{opcode_list, raw_opcode, single_warp_rectilinear_list};
use tiff::Order;

const ID_GAIN_MAP: u32 = 9;
const MANDATORY: u32 = 0;
const OPTIONAL: u32 = 1;

// ─────────────────────────────────────────────────────────────────────────────
// The map, its encoder, and the oracle
// ─────────────────────────────────────────────────────────────────────────────

/// One `GainMap`'s parameters, as the test writes them.
#[derive(Debug, Clone)]
struct Map {
    top: u32,
    left: u32,
    bottom: u32,
    right: u32,
    plane: u32,
    planes: u32,
    row_pitch: u32,
    col_pitch: u32,
    points_v: u32,
    points_h: u32,
    spacing_v: f64,
    spacing_h: f64,
    origin_v: f64,
    origin_h: f64,
    map_planes: u32,
    /// `points_v · points_h · map_planes`, row-major, map planes innermost.
    gains: Vec<f32>,
}

impl Map {
    /// A whole-image (`area` given), pitch-1 map spanning the image:
    /// origin 0, spacing `1 / (points − 1)` (or 1 for a single point).
    fn spanning(area: [u32; 4], points_v: u32, points_h: u32, gains: Vec<f32>) -> Map {
        let span = |n: u32| if n > 1 { 1.0 / f64::from(n - 1) } else { 1.0 };
        Map {
            top: area[0],
            left: area[1],
            bottom: area[2],
            right: area[3],
            plane: 0,
            planes: 1,
            row_pitch: 1,
            col_pitch: 1,
            points_v,
            points_h,
            spacing_v: span(points_v),
            spacing_h: span(points_h),
            origin_v: 0.0,
            origin_h: 0.0,
            map_planes: 1,
            gains,
        }
    }

    /// A 1×1 map of constant `gain` over `area`.
    fn constant(area: [u32; 4], gain: f32) -> Map {
        Map::spanning(area, 1, 1, vec![gain])
    }

    fn pitch(mut self, row_pitch: u32, col_pitch: u32) -> Map {
        self.row_pitch = row_pitch;
        self.col_pitch = col_pitch;
        self
    }
}

/// `DEC-069`'s parameter block, big-endian: ten `u32`, four `f64`,
/// `MapPlanes`, then the gains as `f32`.
fn encode(m: &Map) -> Vec<u8> {
    let mut p = Vec::new();
    for v in [
        m.top,
        m.left,
        m.bottom,
        m.right,
        m.plane,
        m.planes,
        m.row_pitch,
        m.col_pitch,
        m.points_v,
        m.points_h,
    ] {
        p.extend_from_slice(&v.to_be_bytes());
    }
    for v in [m.spacing_v, m.spacing_h, m.origin_v, m.origin_h] {
        p.extend_from_slice(&v.to_be_bytes());
    }
    p.extend_from_slice(&m.map_planes.to_be_bytes());
    for g in &m.gains {
        p.extend_from_slice(&g.to_be_bytes());
    }
    p
}

fn gain_map_op(m: &Map, flags: u32) -> Vec<u8> {
    raw_opcode(ID_GAIN_MAP, 0x0103_0000, flags, &encode(m))
}

/// A list of `maps`, all mandatory.
fn list_of(maps: &[Map]) -> Vec<u8> {
    let ops: Vec<Vec<u8>> = maps.iter().map(|m| gain_map_op(m, MANDATORY)).collect();
    opcode_list(&ops)
}

/// One grid axis, `DEC-069` §2: at or below 0 → index 0, at or above the
/// last point → the last index, each with weight 0 on the next; otherwise the
/// floor and the point after it.
fn oracle_axis(f: f64, points: u32) -> (usize, usize, f64) {
    let last = points as usize - 1;
    if f <= 0.0 {
        (0, 0, 0.0)
    } else if f >= last as f64 {
        (last, last, 0.0)
    } else {
        let i = f.floor();
        (i as usize, i as usize + 1, f - i)
    }
}

/// `DEC-069` §2 applied to an `h × w` normalised stage-2 image, written from
/// the decision's text. One pass per map, in list order, rounding to `u16`
/// after each.
fn oracle(stage2: &[u16], w: u32, h: u32, maps: &[Map]) -> Vec<u16> {
    let mut out = stage2.to_vec();
    for m in maps {
        // Plane: this image has one plane, plane 0.
        if m.plane != 0 {
            continue;
        }
        // Area: empty means the whole image, else intersected. The pitch
        // phase runs from the area's own top-left.
        let (r0, c0, r1, c1) = if m.top >= m.bottom || m.left >= m.right {
            (0, 0, h, w)
        } else {
            (m.top, m.left, m.bottom.min(h), m.right.min(w))
        };
        let p = m.plane.min(m.map_planes - 1) as usize;
        let at = |i: usize, j: usize| {
            f64::from(m.gains[(i * m.points_h as usize + j) * m.map_planes as usize + p])
        };
        let mut r = r0;
        while r < r1 {
            let mut c = c0;
            while c < c1 {
                // Pixel centres over the stage-2 bounds.
                let fr = ((r as f64 + 0.5) / h as f64 - m.origin_v) / m.spacing_v;
                let fc = ((c as f64 + 0.5) / w as f64 - m.origin_h) / m.spacing_h;
                let (i0, i1, tr) = oracle_axis(fr, m.points_v);
                let (j0, j1, tc) = oracle_axis(fc, m.points_h);
                // Rows first, then columns.
                let col0 = at(i0, j0) * (1.0 - tr) + at(i1, j0) * tr;
                let col1 = at(i0, j1) * (1.0 - tr) + at(i1, j1) * tr;
                let g = col0 * (1.0 - tc) + col1 * tc;
                let i = (r * w + c) as usize;
                let x = (f64::from(out[i]) / 65535.0 * g).clamp(0.0, 1.0);
                out[i] = (x * 65535.0 + 0.5).floor() as u16;
                c += m.col_pitch;
            }
            r += m.row_pitch;
        }
    }
    out
}

/// `apply_gain_maps` over a copy of `stage2`, with the count it returns.
fn crate_apply(stage2: &[u16], w: u32, h: u32, list: &[u8]) -> (Vec<u16>, usize) {
    let mut img = stage2.to_vec();
    let n = apply_gain_maps(&mut img, w, h, list).expect("apply_gain_maps");
    (img, n)
}

/// A stage-2 ramp, distinct values from small to near white.
fn ramp(w: u32, h: u32) -> Vec<u16> {
    (0..w * h)
        .map(|i| u16::try_from(1 + (i * 7919) % 65_000).unwrap())
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — the parser
// ─────────────────────────────────────────────────────────────────────────────

fn rich_map() -> Map {
    Map {
        top: 1,
        left: 2,
        bottom: 300,
        right: 400,
        plane: 0,
        planes: 1,
        row_pitch: 2,
        col_pitch: 3,
        points_v: 2,
        points_h: 3,
        spacing_v: 0.3,
        spacing_h: 0.25,
        origin_v: 0.1,
        origin_h: -0.2,
        map_planes: 2,
        gains: vec![
            1.0, 1.5, 2.25, 0.75, 1.125, 3.5, 0.5, 1.0625, 2.0, 1.25, 0.875, 1.75,
        ],
    }
}

#[test]
fn ac1_gain_map_parses_to_the_fields_written() {
    let m = rich_map();
    let ops = parse_opcode_list(&opcode_list(&[gain_map_op(&m, OPTIONAL)])).expect("parses");
    assert_eq!(
        ops,
        vec![Opcode::GainMap {
            top: 1,
            left: 2,
            bottom: 300,
            right: 400,
            plane: 0,
            planes: 1,
            row_pitch: 2,
            col_pitch: 3,
            points_v: 2,
            points_h: 3,
            spacing_v: 0.3,
            spacing_h: 0.25,
            origin_v: 0.1,
            origin_h: -0.2,
            map_planes: 2,
            gains: m.gains.clone(),
        }]
    );
    // A mandatory GainMap is recognised too: no UnsupportedMandatoryOpcode.
    assert!(parse_opcode_list(&opcode_list(&[gain_map_op(&m, MANDATORY)])).is_ok());
}

/// Every `DEC-069` §3 class, each from a valid map changed in one field.
fn malformed_cases() -> Vec<(&'static str, Vec<u8>)> {
    let base = Map::spanning([0, 0, 4, 4], 2, 2, vec![1.0, 1.0, 1.0, 1.0]);
    let with = |f: &dyn Fn(&mut Map)| {
        let mut m = base.clone();
        f(&mut m);
        encode(&m)
    };
    let mut short = encode(&base);
    short.pop();
    let mut long = encode(&base);
    long.push(0);
    vec![
        ("one byte short", short),
        ("one byte long", long),
        ("RowPitch 0", with(&|m| m.row_pitch = 0)),
        ("ColPitch 0", with(&|m| m.col_pitch = 0)),
        // MapPointsV 0 with no gains: the length is then consistent (76).
        (
            "MapPointsV 0",
            with(&|m| {
                m.points_v = 0;
                m.gains.clear();
            }),
        ),
        (
            "MapPointsH 0",
            with(&|m| {
                m.points_h = 0;
                m.gains.clear();
            }),
        ),
        (
            "MapPlanes 0",
            with(&|m| {
                m.map_planes = 0;
                m.gains.clear();
            }),
        ),
        ("Planes 0", with(&|m| m.planes = 0)),
        ("spacing 0", with(&|m| m.spacing_v = 0.0)),
        ("spacing negative", with(&|m| m.spacing_h = -0.5)),
        ("spacing NaN", with(&|m| m.spacing_v = f64::NAN)),
        ("spacing infinite", with(&|m| m.spacing_h = f64::INFINITY)),
        ("origin infinite", with(&|m| m.origin_v = f64::INFINITY)),
        ("origin NaN", with(&|m| m.origin_h = f64::NAN)),
        ("gain NaN", with(&|m| m.gains[3] = f32::NAN)),
        ("gain infinite", with(&|m| m.gains[0] = f32::NEG_INFINITY)),
        // A grid whose declared size overflows: 2^32-1 squared points.
        (
            "huge grid against a short block",
            with(&|m| {
                m.points_v = u32::MAX;
                m.points_h = u32::MAX;
            }),
        ),
    ]
}

#[test]
fn ac1_every_malformed_class_is_a_typed_error() {
    let cases = malformed_cases();
    assert!(cases.len() >= 12);
    for (name, params) in cases {
        let size = u32::try_from(params.len()).unwrap();
        let list = opcode_list(&[raw_opcode(ID_GAIN_MAP, 0x0103_0000, OPTIONAL, &params)]);
        let expected = format!(
            "{:?}",
            Error::MalformedOpcodeParams {
                id: 9,
                declared_size: size
            }
        );
        let err = parse_opcode_list(&list).expect_err(name);
        assert_eq!(format!("{err:?}"), expected, "{name}: parse_opcode_list");
        let mut img = vec![7u16; 16];
        let err = apply_gain_maps(&mut img, 4, 4, &list).expect_err(name);
        assert_eq!(format!("{err:?}"), expected, "{name}: apply_gain_maps");
        assert_eq!(img, vec![7u16; 16], "{name}: nothing written");
    }
}

#[test]
fn ac1_apply_gain_maps_refuses_a_wrong_length_image_and_too_many_maps() {
    let list = list_of(&[Map::constant([0, 0, 0, 0], 2.0)]);
    let mut img = vec![1u16; 11];
    let err = apply_gain_maps(&mut img, 3, 4, &list).unwrap_err();
    assert!(
        matches!(
            err,
            Error::GainMapImageWrongLength {
                expected: 12,
                actual: 11
            }
        ),
        "{err:?}"
    );
    let seventeen = list_of(&vec![Map::constant([0, 0, 0, 0], 2.0); 17]);
    let mut img = vec![1u16; 12];
    let err = apply_gain_maps(&mut img, 3, 4, &seventeen).unwrap_err();
    assert!(
        matches!(err, Error::TooManyGainMaps { count: 17 }),
        "{err:?}"
    );
    assert_eq!(img, vec![1u16; 12], "nothing written");
    // Sixteen is the cap, not over it.
    let sixteen = list_of(&vec![Map::constant([0, 0, 0, 0], 1.0); 16]);
    assert_eq!(apply_gain_maps(&mut img, 3, 4, &sixteen).unwrap(), 16);
    // Other opcodes are ignored; an empty list applies nothing.
    let other = opcode_list(&[raw_opcode(999, 0, OPTIONAL, &[1, 2, 3])]);
    assert_eq!(apply_gain_maps(&mut img, 3, 4, &other).unwrap(), 0);
    assert_eq!(img, vec![1u16; 12]);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — bit for bit against the oracle
// ─────────────────────────────────────────────────────────────────────────────

const W2: u32 = 9;
const H2: u32 = 7;

/// The AC2 cases: a name and a list of maps, each over the 9×7 ramp.
fn ac2_cases() -> Vec<(&'static str, Vec<Map>)> {
    let mut grid = Map::spanning([0, 0, H2, W2], 2, 3, vec![0.6, 1.3, 2.4, 1.7, 0.9, 3.1]);
    // Non-zero origin and spacing: the grid covers v in [0.1, 0.4] and h in
    // [0.2, 0.7], so indices clamp on both sides and interpolate between.
    grid.origin_v = 0.1;
    grid.spacing_v = 0.3;
    grid.origin_h = 0.2;
    grid.spacing_h = 0.25;
    let phase = |top: u32, left: u32, gain: f32| {
        Map::spanning(
            [top, left, H2, W2],
            3,
            3,
            vec![gain, 1.1, 0.8, 1.2, gain, 1.4, 0.7, 1.05, gain],
        )
        .pitch(2, 2)
    };
    let mut edge = grid.clone();
    (edge.top, edge.left, edge.bottom, edge.right) = (2, 5, 100, 100);
    let mut empty = grid.clone().pitch(2, 3);
    (empty.top, empty.left, empty.bottom, empty.right) = (5, 7, 5, 2);
    let mut outside = grid.clone();
    (outside.top, outside.left, outside.bottom, outside.right) = (50, 0, 60, W2);
    let mut plane_1 = grid.clone();
    plane_1.plane = 1;
    plane_1.map_planes = 2;
    plane_1.gains = grid.gains.iter().flat_map(|&g| [g, 2.0 * g]).collect();
    let mut plane_0_of_2 = plane_1.clone();
    plane_0_of_2.plane = 0;
    vec![
        ("2x3 grid, clamped and interpolated", vec![grid.clone()]),
        ("pitch 2, phase (1, 1)", vec![phase(1, 1, 1.6)]),
        ("pitch 2, phase (1, 2)", vec![phase(1, 2, 1.7)]),
        ("pitch 2, phase (2, 1)", vec![phase(2, 1, 0.6)]),
        ("pitch 2, phase (2, 2)", vec![phase(2, 2, 2.2)]),
        (
            "four phase maps in one list",
            vec![
                phase(1, 1, 1.6),
                phase(1, 2, 1.7),
                phase(2, 1, 0.6),
                phase(2, 2, 2.2),
            ],
        ),
        ("area intersected by the image edge", vec![edge]),
        ("empty area is the whole image, pitch from 0", vec![empty]),
        ("area wholly outside the image", vec![outside]),
        ("Plane 1 applies to nothing", vec![plane_1]),
        ("Plane 0 of a two-plane map", vec![plane_0_of_2]),
        (
            "1x1 map, a constant gain",
            vec![Map::constant([0, 0, H2, W2], 1.37)],
        ),
        (
            "gain above 1 clips at white",
            vec![Map::constant([0, 0, H2, W2], 3.9)],
        ),
        ("gain below 1", vec![Map::constant([0, 0, H2, W2], 0.43)]),
        (
            "two maps on the same samples, rounded after each",
            vec![
                Map::constant([0, 0, H2, W2], 1.4),
                Map::constant([0, 0, H2, W2], 1.4),
            ],
        ),
    ]
}

#[test]
fn ac2_apply_gain_maps_equals_the_oracle_bit_for_bit() {
    let stage2 = ramp(W2, H2);
    for (name, maps) in ac2_cases() {
        let expected = oracle(&stage2, W2, H2, &maps);
        let (got, count) = crate_apply(&stage2, W2, H2, &list_of(&maps));
        assert_eq!(count, maps.len(), "{name}: every GainMap counts as applied");
        assert_eq!(
            got, expected,
            "{name}: apply_gain_maps differs from the oracle"
        );
    }
}

/// The cases must be able to tell the rule's parts apart, or the comparison
/// above proves less than it claims: each changes the image (or, where it
/// must not, leaves it), and the two-map case differs from one rounding.
#[test]
fn ac2_cases_are_sensitive() {
    let stage2 = ramp(W2, H2);
    for (name, maps) in ac2_cases() {
        let out = oracle(&stage2, W2, H2, &maps);
        let unchanged = name.contains("outside") || name.starts_with("Plane 1");
        assert_eq!(out == stage2, unchanged, "{name}: changes the image?");
    }
    // One rounding over both 1.4 gains differs from two somewhere.
    let once = oracle(&stage2, W2, H2, &[Map::constant([0, 0, H2, W2], 1.4 * 1.4)]);
    let twice = oracle(
        &stage2,
        W2,
        H2,
        &[
            Map::constant([0, 0, H2, W2], 1.4),
            Map::constant([0, 0, H2, W2], 1.4),
        ],
    );
    assert_ne!(once, twice);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — hand values
// ─────────────────────────────────────────────────────────────────────────────

/// A 4×4 stage 2. The map is a 2×2 grid, origin (0.25, 0.25), spacing
/// (0.5, 0.5), gains `[[1.25, 1.5], [0.5, 2.0]]` (rows first index), over the
/// whole image at pitch 1.
#[test]
fn ac3_hand_values() {
    let mut m = Map::spanning([0, 0, 4, 4], 2, 2, vec![1.25, 1.5, 0.5, 2.0]);
    m.origin_v = 0.25;
    m.origin_h = 0.25;
    m.spacing_v = 0.5;
    m.spacing_h = 0.5;
    let mut img = vec![1000u16; 16];
    img[0] = 40_000; // (0, 0)
    img[6] = 32_768; // (1, 2)
    img[15] = 40_000; // (3, 3)
    let (out, _) = crate_apply(&img, 4, 4, &list_of(&[m]));

    // Corner clamp, (0, 0): fr = ((0 + 0.5)/4 − 0.25)/0.5 = −0.25 ≤ 0 → row 0;
    // fc likewise → col 0. g = 1.25; 40000 · 1.25 = 50000.
    assert_eq!(out[0], 50_000);
    // Interior bilinear, (1, 2): fr = (1.5/4 − 0.25)/0.5 = 0.25,
    // fc = (2.5/4 − 0.25)/0.5 = 0.75. Rows first: col 0 = 1.25·0.75 + 0.5·0.25
    // = 1.0625, col 1 = 1.5·0.75 + 2.0·0.25 = 1.625; then g = 1.0625·0.25 +
    // 1.625·0.75 = 1.484375. 32768 · 1.484375 = 48640.
    assert_eq!(out[6], 48_640);
    // Clip at white, (3, 3): fr = fc = (3.5/4 − 0.25)/0.5 = 1.25 ≥ 1 → last
    // row and column, g = 2.0. 40000 · 2 = 80000 > 65535 → 65535.
    assert_eq!(out[15], 65_535);

    // Pitch: a constant 2.0 over rows 1..4 at row pitch 2 from Top = 1
    // touches rows 1 and 3 only. 1000 · 2 = 2000 there; row 2 is skipped and
    // keeps 1000.
    let skip = Map::constant([1, 0, 4, 4], 2.0).pitch(2, 1);
    let (out, _) = crate_apply(&[1000u16; 16], 4, 4, &list_of(&[skip]));
    assert_eq!(out[4], 2000); // (1, 0)
    assert_eq!(out[8], 1000); // (2, 0), pitch-skipped
    assert_eq!(out[12], 2000); // (3, 0)
    assert_eq!(out[0], 1000); // (0, 0), above the area

    // Rounded after each map: v = 1 under two 1.4 maps is ⌊1.4 + 0.5⌋ = 1,
    // then 1 again; one rounding of 1 · 1.96 would give 2.
    let both = [
        Map::constant([0, 0, 0, 0], 1.4),
        Map::constant([0, 0, 0, 0], 1.4),
    ];
    let (out, _) = crate_apply(&[1u16; 4], 2, 2, &list_of(&both));
    assert_eq!(out, vec![1u16; 4]);

    // An empty area is the whole image: Top = Bottom, every sample doubled.
    let (out, _) = crate_apply(
        &[300u16; 4],
        2,
        2,
        &list_of(&[Map::constant([3, 0, 3, 9], 2.0)]),
    );
    assert_eq!(out, vec![600u16; 4]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures for the develop paths
// ─────────────────────────────────────────────────────────────────────────────

const BLACK: u32 = 512;
const WHITE: u32 = 60_000;

/// `normalize` per `DEC-018`, transcribed: clamp to `[black, white]`, scale
/// to `[0, 65535]`, round half up in integers.
fn normalize(s: u16, black: u32, white: u32) -> u16 {
    let v = u32::from(s).clamp(black, white) - black;
    let d = u64::from(white - black);
    ((u64::from(v) * 65535 + d / 2) / d) as u16
}

struct Plane {
    width: u32,
    height: u32,
    /// `[top, left, bottom, right]`.
    active: [u32; 4],
    raw: Vec<u16>,
}

impl Plane {
    fn new(width: u32, height: u32, active: [u32; 4]) -> Plane {
        let raw = (0..width * height)
            .map(|i| u16::try_from(300 + (i * 4099) % 62_000).unwrap())
            .collect();
        Plane {
            width,
            height,
            active,
            raw,
        }
    }

    fn active_dims(&self) -> (u32, u32) {
        (
            self.active[3] - self.active[1],
            self.active[2] - self.active[0],
        )
    }

    /// The normalised stage 2 (the `ActiveArea` window, levels applied).
    fn stage2(&self) -> Vec<u16> {
        let (aw, ah) = self.active_dims();
        let mut out = Vec::new();
        for r in 0..ah {
            for c in 0..aw {
                let x = self.active[1] + c;
                let y = self.active[0] + r;
                out.push(normalize(
                    self.raw[(y * self.width + x) as usize],
                    BLACK,
                    WHITE,
                ));
            }
        }
        out
    }

    /// The pre-gained twin's raw plane (black 0, white 65535, so levels are
    /// the identity): every photosite normalised, and the `ActiveArea`
    /// window replaced by the oracle's gained stage 2.
    fn pre_gained(&self, maps: &[Map]) -> Vec<u16> {
        let (aw, ah) = self.active_dims();
        let gained = oracle(&self.stage2(), aw, ah, maps);
        let mut out: Vec<u16> = self
            .raw
            .iter()
            .map(|&s| normalize(s, BLACK, WHITE))
            .collect();
        for r in 0..ah {
            for c in 0..aw {
                let x = self.active[1] + c;
                let y = self.active[0] + r;
                out[(y * self.width + x) as usize] = gained[(r * aw + c) as usize];
            }
        }
        out
    }
}

/// A DNG of `samples` on `plane`'s geometry. `levels` is `Some((black,
/// white))`, or `None` for the twin (no `BlackLevel`, white 65535).
fn description(
    plane: &Plane,
    kind: PlaneKind,
    samples: Vec<u16>,
    levels: Option<(u32, u32)>,
    orientation: u16,
    lists: [Option<Vec<u8>>; 3],
) -> DngDescription {
    let (black, white) = match levels {
        Some((b, w)) => (Some(vec![b]), w),
        None => (None, 65_535),
    };
    let row = |a: i32, b: i32, c: i32| [(a, 100_000), (b, 100_000), (c, 100_000)];
    let meta = DngMeta {
        black_level: black,
        white_level: Some(vec![white]),
        active_area: Some(plane.active),
        default_crop_origin: Some([1, 1]),
        default_crop_size: Some([plane.active_dims().0 - 3, plane.active_dims().1 - 2]),
        orientation: Some(orientation),
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
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width: plane.width,
        height: plane.height,
        bits_per_sample: 16,
        kind,
        organisation: Organisation::Strips {
            rows_per_strip: plane.height,
        },
        payload: Payload::Samples(samples),
        meta,
        opcode_lists: lists,
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

/// A mild barrel warp the `FullActiveArea` bounds rule admits.
fn warp_list() -> Vec<u8> {
    single_warp_rectilinear_list([1.0, -0.04, 0.0, 0.0], [0.0, 0.0], 0.47, 0.53, 1, MANDATORY)
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4 — the mono path
// ─────────────────────────────────────────────────────────────────────────────

/// Every mono develop entry on `(sensor, src)`: `develop_into`,
/// `develop_with_into` under both framings (oriented and not), and
/// `develop_with_scratch_into` under the same options.
fn mono_outputs(sensor: &Sensor, src: &[u16]) -> Vec<(String, Vec<u16>)> {
    let mut outs = Vec::new();
    let default = DevelopOptions::default();
    let len = |o: &DevelopOptions| {
        let (w, h) = output_dimensions_with(sensor, o).expect("dims");
        (w * h) as usize
    };
    let mut dst = vec![0xABCDu16; len(&default)];
    develop_into(sensor, src, &mut dst).expect("develop_into");
    outs.push(("develop_into".to_string(), dst));
    for framing in [Framing::StandardCrop, Framing::FullActiveArea] {
        for apply_orientation in [true, false] {
            let mut o = DevelopOptions::default();
            o.framing = framing;
            o.apply_orientation = apply_orientation;
            let mut dst = vec![0xABCDu16; len(&o)];
            develop_with_into(sensor, src, &mut dst, &o).expect("develop_with_into");
            outs.push((
                format!("develop_with_into {framing:?} {apply_orientation}"),
                dst,
            ));
            let mut scratch = vec![0x5A5Au16; scratch_len(sensor, &o).expect("scratch_len")];
            let mut dst = vec![0xABCDu16; len(&o)];
            develop_with_scratch_into(sensor, src, &mut dst, &mut scratch, &o)
                .expect("develop_with_scratch_into");
            outs.push((
                format!("develop_with_scratch_into {framing:?} {apply_orientation}"),
                dst,
            ));
        }
    }
    outs
}

fn mono_maps() -> Vec<Map> {
    let mut grid = Map::spanning(
        [0, 0, 0, 0],
        3,
        4,
        vec![
            1.9, 1.2, 1.1, 2.6, 1.4, 1.0, 0.95, 1.3, 2.2, 1.15, 1.05, 3.0,
        ],
    );
    grid.origin_h = 0.1;
    grid.spacing_h = 0.3;
    vec![grid, Map::constant([1, 2, 6, 9], 1.6).pitch(2, 3)]
}

#[test]
fn ac4_mono_develop_equals_the_oracle_gain_then_the_existing_pipeline() {
    // 13×10 plane, ActiveArea origin (left 3, top 1): odd on both axes.
    let plane = Plane::new(13, 10, [1, 3, 9, 12]);
    let maps = mono_maps();
    let gain_list = list_of(&maps);
    for orientation in [1u16, 6] {
        for warp in [false, true] {
            let l3 = warp.then(warp_list);
            let with_gain = description(
                &plane,
                PlaneKind::LinearRaw,
                plane.raw.clone(),
                Some((BLACK, WHITE)),
                orientation,
                [None, Some(gain_list.clone()), l3.clone()],
            );
            let twin = description(
                &plane,
                PlaneKind::LinearRaw,
                plane.pre_gained(&maps),
                None,
                orientation,
                [None, None, l3],
            );
            let (sensor, src) = read(&with_gain);
            let (twin_sensor, twin_src) = read(&twin);
            let got = mono_outputs(&sensor, &src);
            let want = mono_outputs(&twin_sensor, &twin_src);
            for ((label, g), (_, w)) in got.iter().zip(&want) {
                assert_eq!(g, w, "orientation {orientation} warp {warp}: {label}");
            }
            // The gain really did something to these outputs.
            let (plain_sensor, plain_src) = read(&description(
                &plane,
                PlaneKind::LinearRaw,
                plane.raw.clone(),
                Some((BLACK, WHITE)),
                orientation,
                [None, None, warp.then(warp_list)],
            ));
            let plain = mono_outputs(&plain_sensor, &plain_src);
            assert_ne!(got[0].1, plain[0].1, "the GainMap must change the image");
        }
    }
}

/// An optional `GainMap` applies exactly like a mandatory one (`DEC-069` §2).
#[test]
fn ac4_optional_gain_maps_apply_too() {
    let plane = Plane::new(13, 10, [1, 3, 9, 12]);
    let maps = mono_maps();
    let optional = opcode_list(
        &maps
            .iter()
            .map(|m| gain_map_op(m, OPTIONAL))
            .collect::<Vec<_>>(),
    );
    let develop = |list: Vec<u8>| {
        let (sensor, src) = read(&description(
            &plane,
            PlaneKind::LinearRaw,
            plane.raw.clone(),
            Some((BLACK, WHITE)),
            1,
            [None, Some(list), None],
        ));
        mono_outputs(&sensor, &src)
    };
    assert_eq!(develop(optional), develop(list_of(&maps)));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5 — the colour path
// ─────────────────────────────────────────────────────────────────────────────

fn rggb() -> PlaneKind {
    PlaneKind::Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 1, 1, 2],
        plane_color: vec![0, 1, 2],
    }
}

/// The Pixel shape: four phase maps at pitch 2 over the whole image, one
/// per Bayer phase, each a 3×4 grid.
fn phase_maps() -> Vec<Map> {
    let mut maps = Vec::new();
    for (k, (top, left)) in [(0u32, 0u32), (0, 1), (1, 0), (1, 1)]
        .into_iter()
        .enumerate()
    {
        let base = 1.0 + 0.2 * k as f32;
        let gains = (0..12).map(|i| base + 0.15 * (i % 5) as f32).collect();
        let mut m = Map::spanning([top, left, 10, 10], 3, 4, gains).pitch(2, 2);
        m.origin_v = 0.05;
        m.spacing_v = 0.4;
        maps.push(m);
    }
    maps
}

/// The four colour entries on `(sensor, src)`, the `f32` ones carried as
/// bit patterns; the threaded entries must equal the single-thread ones.
fn colour_outputs(sensor: &Sensor, src: &[u16]) -> (Vec<u16>, Vec<u32>) {
    let o = DevelopOptions::default();
    let (w, h) = output_dimensions_with(sensor, &o).expect("dims");
    let n = (w * h * 3) as usize;
    let mut u = vec![0xABCDu16; n];
    develop_color_into(sensor, src, &mut u).expect("develop_color_into");
    let mut ut = vec![0u16; n];
    develop_color_into_threaded(sensor, src, &mut ut, 3).expect("u16 threaded");
    assert_eq!(u, ut, "threaded u16 differs from single-thread");
    let mut f = vec![-1.0f32; n];
    develop_color_f32_into(sensor, src, &mut f).expect("develop_color_f32_into");
    let mut ft = vec![0f32; n];
    develop_color_f32_into_threaded(sensor, src, &mut ft, 3).expect("f32 threaded");
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(
        bits(&f),
        bits(&ft),
        "threaded f32 differs from single-thread"
    );
    (u, bits(&f))
}

#[test]
fn ac5_colour_develop_equals_the_pre_gained_twin_on_every_entry() {
    // 14×11 plane, ActiveArea origin (left 3, top 1): odd on both axes.
    let plane = Plane::new(14, 11, [1, 3, 11, 13]);
    let maps = phase_maps();
    for orientation in [1u16, 3] {
        let with_gain = description(
            &plane,
            rggb(),
            plane.raw.clone(),
            Some((BLACK, WHITE)),
            orientation,
            [None, Some(list_of(&maps)), None],
        );
        let twin = description(
            &plane,
            rggb(),
            plane.pre_gained(&maps),
            None,
            orientation,
            [None, None, None],
        );
        let (sensor, src) = read(&with_gain);
        let (twin_sensor, twin_src) = read(&twin);
        let got = colour_outputs(&sensor, &src);
        assert_eq!(
            got,
            colour_outputs(&twin_sensor, &twin_src),
            "orientation {orientation}"
        );
        let (plain_sensor, plain_src) = read(&description(
            &plane,
            rggb(),
            plane.raw.clone(),
            Some((BLACK, WHITE)),
            orientation,
            [None, None, None],
        ));
        assert_ne!(
            got,
            colour_outputs(&plain_sensor, &plain_src),
            "the GainMap must change the image"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fuzz seeds — the `opcode` target's GainMap corpus, run through every entry
// ─────────────────────────────────────────────────────────────────────────────

/// The committed `GainMap` fuzz seeds (`fuzz/seeds/opcode/gainmap-*`) through
/// the parser and the applier: typed answers, no panic, and the applier's
/// count equals the parsed `GainMap` count whenever the list parses.
#[test]
fn gain_map_fuzz_seeds_do_not_panic() {
    let seeds = opcode_support::gain_map_fuzz_seed_corpus();
    assert_eq!(seeds.len(), 13);
    for (name, bytes) in seeds {
        let parsed = parse_opcode_list(&bytes);
        let mut img = vec![1000u16; 8 * 6];
        let applied = apply_gain_maps(&mut img, 8, 6, &bytes);
        match (&parsed, &applied) {
            (Ok(ops), Ok(n)) => {
                let maps = ops
                    .iter()
                    .filter(|op| matches!(op, Opcode::GainMap { .. }))
                    .count();
                assert_eq!(*n, maps, "{name}");
            }
            (Ok(_), Err(Error::TooManyGainMaps { count: 17 })) => {}
            (Err(a), Err(b)) => assert_eq!(format!("{a:?}"), format!("{b:?}"), "{name}"),
            other => panic!("{name}: parser and applier disagree: {other:?}"),
        }
    }
}
