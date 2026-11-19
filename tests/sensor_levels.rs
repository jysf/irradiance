//! `SPEC-075` — the `BlackLevel` read and `MaskedAreas` (`DEC-068` §1, §3).
//!
//! Tier A only: every fixture is written by the SPEC-057 test writer, with
//! `extra_tags` carrying a `RATIONAL` `BlackLevel` and every `MaskedAreas`
//! shape. No held DNG carries `MaskedAreas`, and only the Pixel 7 Pro (which
//! does not develop yet) has a fractional black, so the writer is the only
//! source of both.
//!
//! **No expected value comes from calling the crate.** Rounded blacks are
//! written out by hand (`2049/2` is 1024.5, so 1025). The develop test
//! compares two files whose DNG meaning is the same, a fractional black and
//! the SHORT black it rounds to, and also checks that the fractional file
//! does not develop like the black-0 file it builds itself (the pre-SPEC-075
//! behaviour), so the test is red on the old read.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag, TagIfd,
};
use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::{ActiveArea, Container, Rational, Sensor, TAG_BLACK_LEVEL, TAG_MASKED_AREAS};
use irradiance::plane::unpack_into;
use tiff::Order;

const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;

/// The fixture plane: 16 wide, 12 high, 16-bit.
const WIDTH: u32 = 16;
const HEIGHT: u32 = 12;

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures
// ─────────────────────────────────────────────────────────────────────────────

/// A ramp, distinct at every photosite, from 900 up, below `WhiteLevel`.
fn ramp() -> Vec<u16> {
    (0..WIDTH * HEIGHT)
        .map(|i| u16::try_from(900 + (i * 37) % 3000).unwrap())
        .collect()
}

/// A little-endian, one-IFD strip DNG of [`ramp`], `kind`, white 4095.
fn desc(kind: PlaneKind) -> DngDescription {
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width: WIDTH,
        height: HEIGHT,
        bits_per_sample: 16,
        kind,
        organisation: Organisation::Strips {
            rows_per_strip: HEIGHT,
        },
        payload: Payload::Samples(ramp()),
        meta: DngMeta {
            white_level: Some(vec![4095]),
            ..DngMeta::default()
        },
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

fn mono() -> DngDescription {
    desc(PlaneKind::LinearRaw)
}

fn rggb() -> DngDescription {
    desc(PlaneKind::Cfa {
        repeat_dim: [2, 2],
        pattern: vec![0, 1, 1, 2],
        plane_color: vec![0, 1, 2],
    })
}

/// A sensor-IFD tag of `field_type` holding `bytes` (already little-endian).
fn raw(tag: u16, field_type: u16, count: u32, bytes: Vec<u8>) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type,
        count,
        bytes,
    }
}

/// `BlackLevel` as `RATIONAL`, `(num, den)` written verbatim.
fn rational_black(values: &[(u32, u32)]) -> RawTag {
    let mut bytes = Vec::new();
    for (num, den) in values {
        bytes.extend_from_slice(&num.to_le_bytes());
        bytes.extend_from_slice(&den.to_le_bytes());
    }
    raw(TAG_BLACK_LEVEL, TYPE_RATIONAL, values.len() as u32, bytes)
}

/// `BlackLevel` as `SHORT`.
fn short_black(values: &[u16]) -> RawTag {
    let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    raw(TAG_BLACK_LEVEL, TYPE_SHORT, values.len() as u32, bytes)
}

fn with_tags(mut d: DngDescription, tags: Vec<RawTag>) -> DngDescription {
    d.extra_tags = tags;
    d
}

fn read(d: &DngDescription) -> (Sensor, Vec<u16>) {
    let file = write_dng(d).expect("write");
    let container = Container::parse(&file).expect("parse");
    let sensor = container.sensor().expect("sensor");
    let mut plane = vec![0u16; (sensor.width * sensor.height) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpack");
    (sensor, plane)
}

fn sensor_of(d: &DngDescription) -> Sensor {
    read(d).0
}

fn r(num: i32, den: i32) -> Rational {
    Rational { num, den }
}

fn count_of(tags: &[u16], tag: u16) -> usize {
    tags.iter().filter(|&&t| t == tag).count()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1 — a fractional black reads rounded, the exact values kept
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn fractional_black_level_rounds_to_nearest() {
    // 2049/2 = 1024.5: an exact half rounds up.
    let s = sensor_of(&with_tags(mono(), vec![rational_black(&[(2049, 2)])]));
    assert_eq!(s.black_level, Some(1025));
    assert_eq!(s.black_levels, vec![1025]);
    assert_eq!(s.black_levels_exact, vec![r(2049, 2)]);
    assert!(s.malformed_tags.is_empty(), "{:?}", s.malformed_tags);

    // 20491/20 = 1024.55 rounds up; 20489/20 = 1024.45 rounds down.
    let s = sensor_of(&with_tags(mono(), vec![rational_black(&[(20491, 20)])]));
    assert_eq!(s.black_level, Some(1025));
    let s = sensor_of(&with_tags(mono(), vec![rational_black(&[(20489, 20)])]));
    assert_eq!(s.black_level, Some(1024));
}

#[test]
fn pixel_7_pro_black_levels_read_exact_and_rounded() {
    // The held Pixel 7 Pro's BlackLevel, as written (little-endian RATIONAL at
    // offset 3200): 1024.109375, 1024.3125, 1024.308594, 1024.132812.
    let written = [
        (262_172, 256),
        (262_224, 256),
        (262_223, 256),
        (262_178, 256),
    ];
    let mut d = with_tags(rggb(), vec![rational_black(&written)]);
    d.meta.black_level_repeat_dim = Some([2, 2]);
    let s = sensor_of(&d);
    assert_eq!(s.black_levels, vec![1024, 1024, 1024, 1024]);
    assert_eq!(s.black_level, Some(1024));
    assert_eq!(
        s.black_levels_exact,
        vec![
            r(262_172, 256),
            r(262_224, 256),
            r(262_223, 256),
            r(262_178, 256)
        ]
    );
    assert!(
        !s.malformed_tags.contains(&TAG_BLACK_LEVEL),
        "{:?}",
        s.malformed_tags
    );
}

#[test]
fn zero_denominator_black_level_is_malformed() {
    let s = sensor_of(&with_tags(
        mono(),
        vec![rational_black(&[(1024, 1), (5, 0)])],
    ));
    assert_eq!(s.black_level, None);
    assert!(s.black_levels.is_empty());
    assert!(s.black_levels_exact.is_empty());
    assert_eq!(count_of(&s.malformed_tags, TAG_BLACK_LEVEL), 1);
    // The tag costs only itself: the rest of the read stands.
    assert_eq!(s.white_level, Some(4095));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2 — integer types and integral rationals are unchanged
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn integer_black_level_has_no_exact_values() {
    let s = sensor_of(&with_tags(mono(), vec![short_black(&[512])]));
    assert_eq!(s.black_level, Some(512));
    assert_eq!(s.black_levels, vec![512]);
    assert!(s.black_levels_exact.is_empty());

    // The writer's own LONG BlackLevel reads the same way.
    let mut d = rggb();
    d.meta.black_level = Some(vec![256, 257, 258, 259]);
    d.meta.black_level_repeat_dim = Some([2, 2]);
    let s = sensor_of(&d);
    assert_eq!(s.black_levels, vec![256, 257, 258, 259]);
    assert!(s.black_levels_exact.is_empty());
    assert!(s.malformed_tags.is_empty(), "{:?}", s.malformed_tags);
}

#[test]
fn integral_rational_black_level_keeps_exact_values() {
    // The Canon 5D3's shape: 524032/256 = 2047 exactly.
    let s = sensor_of(&with_tags(mono(), vec![rational_black(&[(524_032, 256)])]));
    assert_eq!(s.black_level, Some(2047));
    assert_eq!(s.black_levels, vec![2047]);
    assert_eq!(s.black_levels_exact, vec![r(524_032, 256)]);
    assert!(s.malformed_tags.is_empty(), "{:?}", s.malformed_tags);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3 — a fractional black develops as its rounded value (mono)
// ─────────────────────────────────────────────────────────────────────────────

fn develop_mono(d: &DngDescription) -> Vec<u16> {
    let (s, plane) = read(d);
    let (w, h) = output_dimensions(&s).unwrap();
    let mut dst = vec![0xABCD; (w * h) as usize];
    develop_into(&s, &plane, &mut dst).expect("develop_into");
    dst
}

#[test]
fn fractional_black_develops_as_its_rounded_value() {
    // 10249/10 = 1024.9, which rounds to 1025.
    let fractional = develop_mono(&with_tags(mono(), vec![rational_black(&[(10_249, 10)])]));
    let rounded = develop_mono(&with_tags(mono(), vec![short_black(&[1025])]));
    let black_zero = develop_mono(&with_tags(mono(), vec![short_black(&[0])]));
    assert_ne!(
        rounded, black_zero,
        "control: the fixture tells black 1025 from black 0"
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

// ─────────────────────────────────────────────────────────────────────────────
// AC7 — MaskedAreas passes through
// ─────────────────────────────────────────────────────────────────────────────

/// `MaskedAreas` as LONG (or SHORT) rectangles `[top, left, bottom, right]`.
fn masked(field_type: u16, rects: &[[u32; 4]]) -> RawTag {
    let mut bytes = Vec::new();
    for v in rects.iter().flatten() {
        match field_type {
            TYPE_SHORT => bytes.extend_from_slice(&u16::try_from(*v).unwrap().to_le_bytes()),
            _ => bytes.extend_from_slice(&v.to_le_bytes()),
        }
    }
    raw(
        TAG_MASKED_AREAS,
        field_type,
        (rects.len() * 4) as u32,
        bytes,
    )
}

fn area([top, left, bottom, right]: [u32; 4]) -> ActiveArea {
    ActiveArea {
        top,
        left,
        bottom,
        right,
    }
}

#[test]
fn masked_areas_read_verbatim_in_order() {
    // Absent: empty, and not malformed.
    let s = sensor_of(&mono());
    assert!(s.masked_areas.is_empty());
    assert!(s.malformed_tags.is_empty());

    // One rectangle: a left strip.
    let one = [[0, 0, 12, 2]];
    let s = sensor_of(&with_tags(mono(), vec![masked(TYPE_LONG, &one)]));
    assert_eq!(s.masked_areas, vec![area(one[0])]);
    assert!(s.malformed_tags.is_empty(), "{:?}", s.malformed_tags);

    // Four rectangles, in an order that is not sorted any way, and one that
    // overlaps another: verbatim, in file order, unpoliced.
    let four = [
        [10, 0, 12, 16],
        [0, 0, 12, 2],
        [0, 14, 12, 16],
        [0, 1, 2, 16],
    ];
    let s = sensor_of(&with_tags(mono(), vec![masked(TYPE_LONG, &four)]));
    assert_eq!(
        s.masked_areas,
        four.iter().map(|&a| area(a)).collect::<Vec<_>>()
    );
    // A rectangle reaching exactly the plane's edge is in range.
    let edge = [[0, 0, HEIGHT, WIDTH]];
    let s = sensor_of(&with_tags(mono(), vec![masked(TYPE_LONG, &edge)]));
    assert_eq!(s.masked_areas, vec![area(edge[0])]);
}

#[test]
fn masked_areas_short_and_long() {
    let rects = [[0, 0, 12, 2], [0, 14, 12, 16]];
    let want: Vec<ActiveArea> = rects.iter().map(|&a| area(a)).collect();
    for field_type in [TYPE_SHORT, TYPE_LONG] {
        let s = sensor_of(&with_tags(mono(), vec![masked(field_type, &rects)]));
        assert_eq!(s.masked_areas, want, "type {field_type}");
        assert!(s.malformed_tags.is_empty(), "type {field_type}");
    }
}

/// The sensor read with the fields a `MaskedAreas` tag may touch blanked:
/// `masked_areas` and `malformed_tags` themselves, and `strip_offsets`, which
/// moves when the writer adds an out-of-line tag.
fn rest_of(s: &Sensor) -> String {
    let mut s = s.clone();
    s.masked_areas.clear();
    s.malformed_tags.clear();
    s.strip_offsets.clear();
    format!("{s:?}")
}

#[test]
fn malformed_masked_areas_cost_only_the_tag() {
    let (control, control_plane) = read(&mono());
    let seventeen: Vec<[u32; 4]> = (0..17).map(|_| [0, 0, 12, 2]).collect();
    let cases: Vec<(&str, RawTag)> = vec![
        (
            "count 5",
            raw(
                TAG_MASKED_AREAS,
                TYPE_LONG,
                5,
                [0u32, 0, 12, 2, 7]
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
            ),
        ),
        ("count 0", raw(TAG_MASKED_AREAS, TYPE_LONG, 0, Vec::new())),
        ("17 rectangles", masked(TYPE_LONG, &seventeen)),
        (
            "top == bottom",
            masked(TYPE_LONG, &[[0, 0, 12, 2], [5, 0, 5, 2]]),
        ),
        ("right > width", masked(TYPE_LONG, &[[0, 0, 12, WIDTH + 1]])),
        // SPEC-075 verify: the other two DEC-068 §3 classes, each in its own
        // case so dropping either check from the read shows red.
        (
            "bottom > height",
            masked(TYPE_LONG, &[[0, 0, HEIGHT + 1, 2]]),
        ),
        (
            "left == right",
            masked(TYPE_LONG, &[[0, 0, 12, 2], [0, 4, 12, 4]]),
        ),
        (
            "RATIONAL type",
            raw(
                TAG_MASKED_AREAS,
                TYPE_RATIONAL,
                4,
                [0u32, 1, 0, 1, 12, 1, 2, 1]
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect(),
            ),
        ),
    ];
    for (name, tag) in cases {
        let (s, plane) = read(&with_tags(mono(), vec![tag]));
        assert!(s.masked_areas.is_empty(), "{name}: {:?}", s.masked_areas);
        assert_eq!(
            s.malformed_tags,
            vec![TAG_MASKED_AREAS],
            "{name}: costed once, and nothing else"
        );
        assert_eq!(rest_of(&s), rest_of(&control), "{name}: every other field");
        assert_eq!(plane, control_plane, "{name}: the plane");
    }

    // 16 rectangles is the cap, and reads.
    let sixteen: Vec<[u32; 4]> = (0..16).map(|i| [0, i, 12, i + 1]).collect();
    let s = sensor_of(&with_tags(mono(), vec![masked(TYPE_LONG, &sixteen)]));
    assert_eq!(s.masked_areas.len(), 16);
    assert!(s.malformed_tags.is_empty());
}
