//! `SPEC-092` — the mono mixer at sensor geometry (`DEC-079`, `DEC-083`):
//! `mono_mix_row`, `mono_mix_plane_into` and its row-range form
//! `mono_mix_plane_rows_into`.
//!
//! - **Tier A** (always runs): SPEC-091's synthetic RGGB plane with an **odd**
//!   `ActiveArea` top and left, a masked border on every side and identity
//!   levels (AC1, AC2, AC3, AC10), and a ramp of flat colour patches through
//!   clipping (AC7).
//! - **Tier B** (`$IRRADIANCE_CORPUS_DIR`, skips per file by name): the five held
//!   colour files (AC2, AC4, AC5).
//!
//! The oracles here never run through the mixer's private code: the kernel's
//! is an `i128` division written below, and the camera triples come from
//! `demosaic_into`, which phases the CFA on its own (the SPEC-091 FU-2
//! lesson).

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/tiff.rs"]
mod tiff;

use core::ops::Range;
use std::path::Path;

use corpus::{CorpusRoot, Manifest};
use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::color::{
    mono_mix_plane_into, mono_mix_plane_rows_into, mono_mix_row, srgb_linear_plane_f32_into,
    srgb_linear_plane_into, MONO_MIX_VERSION,
};
use irradiance::demosaic::demosaic_into;
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;
use opcode_support::{opcode_list, raw_opcode};
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Weights and the independent kernel oracle
// ─────────────────────────────────────────────────────────────────────────────

const fn r(num: i32, den: i32) -> Rational {
    Rational { num, den }
}

/// Rec. 709 luminance on linear sRGB.
const LUMINANCE: [Rational; 3] = [r(2126, 10000), r(7152, 10000), r(722, 10000)];
/// AC5's illustrative red filter and orthochromatic (no red) sets.
const RED: [Rational; 3] = [r(4, 5), r(1, 5), r(0, 1)];
const ORTHO: [Rational; 3] = [r(0, 1), r(1, 2), r(1, 2)];
/// The one-hot sets: each row is `M`'s row for that linear-sRGB channel.
const ONE_HOT: [[Rational; 3]; 3] = [
    [r(1, 1), r(0, 1), r(0, 1)],
    [r(0, 1), r(1, 1), r(0, 1)],
    [r(0, 1), r(0, 1), r(1, 1)],
];

/// The kernel's oracle (AC1): `i128`, a floor division by `2^37` with
/// `div_euclid` rather than a shift, so it shares no code with the mixer.
fn oracle(q: &[i32; 3], x: [u16; 3]) -> u16 {
    let (mut p, mut n) = (0i128, 0i128);
    for (&qi, &xi) in q.iter().zip(&x) {
        let t = i128::from(qi) * i128::from(xi);
        if qi >= 0 {
            p += t;
        } else {
            n += t;
        }
    }
    let m = i128::from(*x.iter().max().unwrap());
    let u = (65535 - m).min(8192);
    let num = p * 8192 + n * u + (1i128 << 36);
    num.div_euclid(1i128 << 37).clamp(0, 65535) as u16
}

/// The plain fold rounded half up, `clamp(⌊(Σ q_i·x_i + 2^23) / 2^24⌋)`.
fn plain_fold(q: &[i32; 3], x: [u16; 3]) -> u16 {
    let acc: i128 = q
        .iter()
        .zip(&x)
        .map(|(&qi, &xi)| i128::from(qi) * i128::from(xi))
        .sum();
    (acc + (1i128 << 23))
        .div_euclid(1i128 << 24)
        .clamp(0, 65535) as u16
}

fn has_negative(q: &[i32; 3]) -> bool {
    q.iter().any(|&v| v < 0)
}

// ─────────────────────────────────────────────────────────────────────────────
// The tier-A fixture: SPEC-091's odd-ActiveArea plane (tests/linear_plane.rs)
// ─────────────────────────────────────────────────────────────────────────────

const W: u32 = 20;
const H: u32 = 17;
/// `ActiveArea` `[top, left, bottom, right]`: odd top and left, a masked border
/// on all four sides.
const ACTIVE: [u32; 4] = [3, 1, 15, 19];

const SENTINEL: u16 = 0xA5A5;

/// xorshift32: integer-only, so the samples are the same everywhere.
fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

/// Full-range noise with a near-full-scale column band, so camera channels
/// reach the fade band and white.
fn samples() -> Vec<u16> {
    let mut state = 0x5091_0001;
    (0..W * H)
        .map(|i| {
            let n = xorshift32(&mut state);
            if i % W >= 14 {
                (65535 - (n & 0x3ff)) as u16
            } else {
                (n & 0xffff) as u16
            }
        })
        .collect()
}

/// No `BlackLevel` and no `WhiteLevel` at 16 bits: the level normaliser is the
/// identity, so `demosaic_into`'s triples are the traversal's camera values.
fn description() -> DngDescription {
    let row = |a: i32, b: i32, c: i32| [(a, 100_000), (b, 100_000), (c, 100_000)];
    DngDescription {
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
        payload: Payload::Samples(samples()),
        meta: DngMeta {
            active_area: Some(ACTIVE),
            default_crop_origin: Some([2, 1]),
            default_crop_size: Some([14, 9]),
            orientation: Some(1),
            forward_matrix_1: Some(
                [
                    row(60_000, 30_000, 6_422),
                    row(20_000, 70_000, 10_000),
                    row(10_000, 10_000, 62_521),
                ]
                .concat(),
            ),
            as_shot_neutral: Some(vec![(1, 2), (1, 1), (2, 3)]),
            ..DngMeta::default()
        },
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

fn fixture() -> (Sensor, Vec<u16>) {
    read(&description())
}

fn pixels(sensor: &Sensor) -> usize {
    sensor.width as usize * sensor.height as usize
}

fn mono(sensor: &Sensor, src: &[u16], weights: &[Rational; 3]) -> Vec<u16> {
    let mut dst = vec![SENTINEL; pixels(sensor)];
    mono_mix_plane_into(sensor, src, weights, &mut dst).expect("the mono plane mixes");
    dst
}

/// `demosaic_into`'s camera triples, one per pixel.
fn triples(sensor: &Sensor, src: &[u16]) -> Vec<[u16; 3]> {
    let mut camera = vec![0u16; pixels(sensor) * 3];
    demosaic_into(sensor, src, &mut camera).expect("demosaics");
    camera.as_chunks::<3>().0.to_vec()
}

/// The fixture's first one-hot weight set whose row has a negative entry.
fn negative_one_hot(sensor: &Sensor) -> (usize, [Rational; 3], [i32; 3]) {
    ONE_HOT
        .iter()
        .enumerate()
        .find_map(|(i, w)| {
            let q = mono_mix_row(sensor, w).expect("row");
            has_negative(&q).then_some((i, *w, q))
        })
        .expect("a one-hot row of the fixture has a negative entry")
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1: the plane equals the i128 oracle on demosaic_into's triples.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac1_the_plane_equals_the_i128_oracle_on_demosaic_triples() {
    let (sensor, src) = fixture();
    let cam = triples(&sensor, &src);
    let (which, negative, q_negative) = negative_one_hot(&sensor);
    eprintln!("AC1: one-hot set {which} has the row {q_negative:?}");
    let mut faded = 0;
    for (name, weights) in [("luminance", LUMINANCE), ("negative one-hot", negative)] {
        let q = mono_mix_row(&sensor, &weights).expect("row");
        let plane = mono(&sensor, &src, &weights);
        for (p, (&f, &x)) in plane.iter().zip(&cam).enumerate() {
            assert_eq!(
                f,
                oracle(&q, x),
                "{name}: pixel ({}, {}), x {x:?}, row {q:?}",
                p % W as usize,
                p / W as usize
            );
            faded += usize::from(oracle(&q, x) != plain_fold(&q, x));
        }
    }
    assert!(faded > 0, "the fade acts on some pixel of the fixture");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: every pixel written; strips tile the plane.
// ─────────────────────────────────────────────────────────────────────────────

/// Strips of `step` rows from `0`, the last holding the remainder, after an
/// optional leading strip of `lead` rows.
fn strips(height: u32, lead: u32, step: u32) -> Vec<Range<u32>> {
    let mut out = Vec::new();
    let mut y = 0;
    if lead > 0 {
        out.push(0..lead);
        y = lead;
    }
    while y < height {
        let end = (y + step).min(height);
        out.push(y..end);
        y = end;
    }
    out
}

fn assert_strips_tile(
    sensor: &Sensor,
    src: &[u16],
    weights: &[Rational; 3],
    partition: &[Range<u32>],
    context: &str,
) {
    let whole = mono(sensor, src, weights);
    let width = sensor.width as usize;
    let mut tiled = Vec::with_capacity(whole.len());
    let mut covered = 0;
    for rows in partition {
        assert_eq!(rows.start, covered, "{context}: contiguous");
        covered = rows.end;
        let mut band = vec![SENTINEL; (rows.end - rows.start) as usize * width];
        mono_mix_plane_rows_into(sensor, src, weights, rows.clone(), &mut band)
            .unwrap_or_else(|e| panic!("{context}: rows {rows:?}: {e}"));
        tiled.extend_from_slice(&band);
    }
    assert_eq!(covered, sensor.height, "{context}: covers 0..height");
    if let Some(i) = (0..whole.len()).find(|&i| tiled[i] != whole[i]) {
        panic!(
            "{context}: strips differ from the whole plane first at ({}, {})",
            i % width,
            i / width
        );
    }
}

#[test]
fn ac2_every_pixel_written_and_strips_tile() {
    let (sensor, src) = fixture();
    let (_, negative, _) = negative_one_hot(&sensor);
    for weights in [LUMINANCE, negative] {
        let plane = mono(&sensor, &src, &weights);
        let stale = plane.iter().filter(|&&v| v == SENTINEL).count();
        assert_eq!(stale, 0, "{stale} sentinel samples left");
        assert_eq!(plane, mono(&sensor, &src, &weights), "repeatable");
        let partitions: [(&str, Vec<Range<u32>>); 4] = [
            ("one strip", strips(H, 0, H)),
            ("strips of 1 row", strips(H, 0, 1)),
            ("strips of 7 rows", strips(H, 0, 7)),
            // Row 1 is inside the masked top border (ActiveArea top is 3), and odd.
            ("two strips split at row 1", vec![0..1, 1..H]),
        ];
        for (name, partition) in &partitions {
            assert_strips_tile(&sensor, &src, &weights, partition, name);
        }
    }
    // The sentinel can be a real mixed value only by chance; the masked border
    // is written too: rows above the ActiveArea top differ from the sentinel.
    let plane = mono(&sensor, &src, &LUMINANCE);
    assert!(plane[..(ACTIVE[0] * W) as usize]
        .iter()
        .all(|&v| v != SENTINEL));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: a row with no negative entry is the plain fold.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac3_no_negative_row_plane_is_the_plain_fold() {
    let (sensor, src) = fixture();
    let cam = triples(&sensor, &src);
    let q = mono_mix_row(&sensor, &LUMINANCE).expect("row");
    assert!(
        !has_negative(&q),
        "the fixture's luminance row {q:?} has no negative entry"
    );
    let above = cam
        .iter()
        .filter(|x| *x.iter().max().unwrap() >= 57344)
        .count();
    assert!(above > 0, "some pixel has m at or above 57344");
    let plane = mono(&sensor, &src, &LUMINANCE);
    for (p, (&f, &x)) in plane.iter().zip(&cam).enumerate() {
        assert_eq!(f, plain_fold(&q, x), "pixel {p}, x {x:?}");
    }
    eprintln!("AC3: row {q:?}; {above} pixels with m >= 57344");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7: a ramp of flat patches through clipping stays monotone.
// ─────────────────────────────────────────────────────────────────────────────

/// Patch edge, in photosites; centres are well inside, so Hamilton-Adams
/// sees only the patch's own colour there.
const PATCH: u32 = 12;
const PATCHES: u32 = 48;
/// The patch colour's camera ratio, in 1/1000: red clips first, then green,
/// and blue keeps rising after both (the plain fold's darkening case when the
/// row's blue entry is negative).
const RATIO: [u32; 3] = [1000, 600, 550];

fn ramp_description() -> DngDescription {
    let row = |a: i32, b: i32, c: i32| [(a, 10_000), (b, 10_000), (c, 10_000)];
    let width = PATCH * PATCHES;
    let mut samples = Vec::with_capacity((width * PATCH) as usize);
    for y in 0..PATCH {
        for x in 0..width {
            // RGGB: (0,0) R, (1,0)/(0,1) G, (1,1) B.
            let channel = match (y % 2, x % 2) {
                (0, 0) => 0,
                (1, 1) => 2,
                _ => 1,
            };
            // Exposure of the brightest channel rises from 0.8 to 2.21 of
            // white in steps of 0.03: red clips first, then green, and blue
            // clips near the end.
            let k = 800 + 30 * (x / PATCH);
            let v = u64::from(RATIO[channel]) * u64::from(k) * 65535 / 1_000_000;
            samples.push(v.min(65535) as u16);
        }
    }
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width,
        height: PATCH,
        bits_per_sample: 16,
        kind: PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        },
        organisation: Organisation::Strips {
            rows_per_strip: PATCH,
        },
        payload: Payload::Samples(samples),
        meta: DngMeta {
            // A ForwardMatrix whose Y row carries a negative blue term, so the
            // luminance row has a negative blue entry.
            forward_matrix_1: Some(
                [
                    row(5000, 3500, 1464),
                    row(3500, 9000, -2500),
                    row(500, -1500, 9249),
                ]
                .concat(),
            ),
            as_shot_neutral: Some(vec![(1, 1), (1, 1), (1, 1)]),
            ..DngMeta::default()
        },
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

#[test]
fn ac7_a_ramp_through_clipping_stays_monotone() {
    let (sensor, src) = read(&ramp_description());
    let q = mono_mix_row(&sensor, &LUMINANCE).expect("row");
    assert!(
        has_negative(&q),
        "the ramp's luminance row {q:?} has a negative entry"
    );
    let plane = mono(&sensor, &src, &LUMINANCE);
    let cam = triples(&sensor, &src);
    let width = sensor.width as usize;
    let centre = |patch: u32| ((PATCH / 2) as usize) * width + (patch * PATCH + PATCH / 2) as usize;
    let mut prev = 0u16;
    let mut clipped_and_faded = 0;
    let mut plain_decreases = 0;
    let mut prev_plain = 0u16;
    for patch in 0..PATCHES {
        let p = centre(patch);
        let (f, x) = (plane[p], cam[p]);
        let plain = plain_fold(&q, x);
        eprintln!("AC7 patch {patch:2}: x {x:?} mono {f} plain {plain}");
        assert!(f >= prev, "patch {patch}: {f} after {prev}");
        clipped_and_faded += usize::from(x.contains(&65535) && f != plain);
        plain_decreases += usize::from(plain < prev_plain);
        prev = f;
        prev_plain = plain;
    }
    assert!(
        clipped_and_faded > 0,
        "some patch centre has a clipped channel and differs from the plain fold"
    );
    eprintln!("AC7: the plain fold decreases {plain_decreases} times along the ramp");
    // The control: without the fade this ramp darkens, so the check can fail.
    assert!(
        plain_decreases > 0,
        "the plain fold decreases along the ramp"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC10: refusals, in order, with dst untouched.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum Entry {
    Plane,
    Rows,
}

/// Run `entry` over `rows` into a sentinel `dst` of `len`; return the result
/// and whether `dst` is still exactly its sentinel fill.
fn refuse(
    entry: Entry,
    sensor: &Sensor,
    src: &[u16],
    weights: &[Rational; 3],
    rows: Range<u32>,
    len: usize,
) -> (Result<(), Error>, bool) {
    let mut dst = vec![SENTINEL; len];
    let res = match entry {
        Entry::Plane => mono_mix_plane_into(sensor, src, weights, &mut dst),
        Entry::Rows => mono_mix_plane_rows_into(sensor, src, weights, rows, &mut dst),
    };
    (res, dst.iter().all(|&v| v == SENTINEL))
}

#[test]
fn ac10_refusals_in_order_leave_dst_untouched() {
    let (sensor, src) = fixture();
    let n = pixels(&sensor);
    let whole = 0..H;
    let invalid: [([Rational; 3], Option<u8>); 4] = [
        ([r(1, 2), r(1, 0), r(1, 2)], Some(1)),
        ([r(-1, 2), r(1, 1), r(1, 2)], Some(0)),
        ([r(1, 2), r(1, 2), r(0, -1)], Some(2)),
        ([r(1, 3), r(1, 3), r(1, 4)], None),
    ];
    // An unsupported mandatory opcode in OpcodeList1: the colour path refuses it
    // first of the file checks, but the weights come first of all.
    let mut opcode_file = sensor.clone();
    opcode_file.opcode_list_1 = Some(opcode_list(&[raw_opcode(4, 0x0103_0000, 0, &[0u8; 8])]));
    opcode_file.opcode_lists[0] = true;
    for entry in [Entry::Plane, Entry::Rows] {
        for (weights, channel) in invalid {
            for s in [&sensor, &opcode_file] {
                let (res, untouched) = refuse(entry, s, &src, &weights, whole.clone(), n);
                assert!(
                    matches!(res, Err(Error::InvalidMixWeights { channel: c }) if c == channel),
                    "{entry:?} {weights:?}: {res:?}"
                );
                assert!(untouched, "{entry:?} {weights:?}: dst written");
            }
        }
        // The opcode file refuses with valid weights (control for the above).
        let (res, untouched) = refuse(entry, &opcode_file, &src, &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::UnsupportedRequiredOpcode { .. })),
            "{entry:?} opcode file: {res:?}"
        );
        assert!(untouched);

        // 2/10 + 7/10 + 1/10 is exactly one (0.9999999999999999 in f64).
        let tenths = [r(2, 10), r(7, 10), r(1, 10)];
        let (res, _) = refuse(entry, &sensor, &src, &tenths, whole.clone(), n);
        assert!(res.is_ok(), "{entry:?} tenths: {res:?}");

        // A wrong src length.
        let (res, untouched) = refuse(entry, &sensor, &src[1..], &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::ColorSourceWrongLength { expected, actual })
                if expected == u64::from(W * H) && actual == src.len() - 1),
            "{entry:?} short src: {res:?}"
        );
        assert!(untouched);

        // dst one short and one long.
        for len in [n - 1, n + 1] {
            let (res, untouched) = refuse(entry, &sensor, &src, &LUMINANCE, whole.clone(), len);
            assert!(
                matches!(res, Err(Error::MonoBufferWrongLength { expected, actual })
                    if expected == u64::from(W * H) && actual == len),
                "{entry:?} dst {len}: {res:?}"
            );
            assert!(untouched, "{entry:?} dst {len}: dst written");
        }

        // A non-CFA plane.
        let mut no_cfa = sensor.clone();
        no_cfa.color_filter = None;
        let (res, untouched) = refuse(entry, &no_cfa, &src, &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::NotAColorFilterArray)),
            "{entry:?}: {res:?}"
        );
        assert!(untouched);

        // No AsShotNeutral.
        let mut no_neutral = sensor.clone();
        no_neutral.as_shot_neutral = None;
        let (res, untouched) = refuse(entry, &no_neutral, &src, &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::MissingColorCalibration)),
            "{entry:?}: {res:?}"
        );
        assert!(untouched);

        // A tiny red neutral: the row's red entry is far past 2^31 / 2^24.
        let mut tiny = sensor.clone();
        tiny.as_shot_neutral = Some(vec![r(1, 100_000), r(1, 1), r(1, 1)]);
        let (res, untouched) = refuse(entry, &tiny, &src, &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::MixRowOutOfRange { channel: 0 })),
            "{entry:?} tiny neutral: {res:?}"
        );
        assert!(untouched, "{entry:?} tiny neutral: dst written");
    }

    // mono_mix_row's own order: weights, calibration, range.
    assert!(matches!(
        mono_mix_row(&sensor, &invalid[0].0),
        Err(Error::InvalidMixWeights { channel: Some(1) })
    ));
    let mut no_neutral = sensor.clone();
    no_neutral.as_shot_neutral = None;
    assert!(matches!(
        mono_mix_row(&no_neutral, &invalid[3].0),
        Err(Error::InvalidMixWeights { channel: None })
    ));
    assert!(matches!(
        mono_mix_row(&no_neutral, &LUMINANCE),
        Err(Error::MissingColorCalibration)
    ));
    let mut tiny = sensor.clone();
    tiny.as_shot_neutral = Some(vec![r(1, 100_000), r(1, 1), r(1, 1)]);
    assert!(matches!(
        mono_mix_row(&tiny, &LUMINANCE),
        Err(Error::MixRowOutOfRange { channel: 0 })
    ));
    // mono_mix_row runs no CFA or opcode check.
    let mut no_cfa = sensor.clone();
    no_cfa.color_filter = None;
    assert!(mono_mix_row(&no_cfa, &LUMINANCE).is_ok());
    assert!(mono_mix_row(&opcode_file, &LUMINANCE).is_ok());

    // The band forms' own refusals.
    let bad = [
        Range { start: 5, end: 5 },
        Range { start: 6, end: 5 },
        Range {
            start: 0,
            end: H + 1,
        },
    ];
    for rows in bad {
        let (start, end) = (rows.start, rows.end);
        let (res, untouched) = refuse(Entry::Rows, &sensor, &src, &LUMINANCE, rows, n);
        assert!(
            matches!(res, Err(Error::InvalidRowRange { start: s, end: e, height })
                if s == start && e == end && height == H),
            "rows {start}..{end}: {res:?}"
        );
        assert!(untouched);
    }
    let band = 4..11;
    let band_len = ((band.end - band.start) * W) as usize;
    let (res, untouched) = refuse(Entry::Rows, &sensor, &src, &LUMINANCE, band, band_len - 1);
    assert!(
        matches!(res, Err(Error::MonoBufferWrongLength { expected, actual })
            if expected == band_len as u64 && actual == band_len - 1),
        "short band dst: {res:?}"
    );
    assert!(untouched);
}

/// Verify (`SPEC-092`, HANDOFF-203): `MixRowOutOfRange` is the **last**
/// refusal (Design §6). AC10 shows it only on an otherwise valid file, where a
/// row check moved ahead of the file checks also passes. Here the tiny-neutral
/// file carries a second fault, and the earlier refusal must win, `dst` intact.
#[test]
fn the_row_range_refusal_comes_after_every_file_check() {
    let (sensor, src) = fixture();
    let n = pixels(&sensor);
    let whole = 0..H;
    let mut tiny = sensor.clone();
    tiny.as_shot_neutral = Some(vec![r(1, 100_000), r(1, 1), r(1, 1)]);
    let mut tiny_opcode = tiny.clone();
    tiny_opcode.opcode_list_1 = Some(opcode_list(&[raw_opcode(4, 0x0103_0000, 0, &[0u8; 8])]));
    tiny_opcode.opcode_lists[0] = true;
    let mut tiny_no_cfa = tiny.clone();
    tiny_no_cfa.color_filter = None;
    for entry in [Entry::Plane, Entry::Rows] {
        let (res, untouched) = refuse(entry, &tiny, &src, &LUMINANCE, whole.clone(), n - 1);
        assert!(
            matches!(res, Err(Error::MonoBufferWrongLength { .. })),
            "{entry:?} tiny neutral, short dst: {res:?}"
        );
        assert!(untouched);
        let (res, untouched) = refuse(entry, &tiny, &src[1..], &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::ColorSourceWrongLength { .. })),
            "{entry:?} tiny neutral, short src: {res:?}"
        );
        assert!(untouched);
        let (res, untouched) = refuse(entry, &tiny_opcode, &src, &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::UnsupportedRequiredOpcode { .. })),
            "{entry:?} tiny neutral, opcode file: {res:?}"
        );
        assert!(untouched);
        let (res, untouched) = refuse(entry, &tiny_no_cfa, &src, &LUMINANCE, whole.clone(), n);
        assert!(
            matches!(res, Err(Error::NotAColorFilterArray)),
            "{entry:?} tiny neutral, no CFA: {res:?}"
        );
        assert!(untouched);
    }
    let (res, untouched) = refuse(
        Entry::Rows,
        &tiny,
        &src,
        &LUMINANCE,
        Range { start: 6, end: 5 },
        n,
    );
    assert!(
        matches!(res, Err(Error::InvalidRowRange { .. })),
        "tiny neutral, reversed band: {res:?}"
    );
    assert!(untouched);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tier B: the five held colour files.
// ─────────────────────────────────────────────────────────────────────────────

const CANON: &str = "CANON-EOS-5D-MARK-III/5G4A9394.DNG";
const PIXEL_3A: &str = "GOOGLE-PIXEL-3A/IMG_20190918_164153.dng";
const PIXEL_4A: &str = "GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng";
const PIXEL_7_PRO: &str = "GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng";
const NIKON: &str = "NIKON-D300S/_DSC4615.DNG";

const HELD: [&str; 5] = [CANON, PIXEL_3A, PIXEL_4A, PIXEL_7_PRO, NIKON];

fn corpus_plane(path: &str) -> Option<(Sensor, Vec<u16>)> {
    let manifest = Manifest::load().expect("tests/corpus/manifest.toml must parse");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} must be in the manifest"));
    let Some(file_path) = entry.require(&root) else {
        eprintln!("SKIP {path}: not in $IRRADIANCE_CORPUS_DIR");
        return None;
    };
    Some(load_plane(path, &file_path))
}

fn load_plane(path: &str, file_path: &Path) -> (Sensor, Vec<u16>) {
    let bytes = std::fs::read(file_path).unwrap_or_else(|e| panic!("{path}: read: {e}"));
    let container = Container::parse(&bytes).unwrap_or_else(|e| panic!("{path}: {e}"));
    let sensor = container.sensor().unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut plane = vec![0u16; pixels(&sensor)];
    unpack_into(&sensor, container.byte_order(), &bytes, &mut plane)
        .unwrap_or_else(|e| panic!("{path}: unpack: {e}"));
    (sensor, plane)
}

#[test]
fn ac2_strips_tile_on_held_files() {
    for path in [PIXEL_7_PRO, CANON] {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let lead = u32::from(sensor.height % 256 % 2 == 0);
        let partition = strips(sensor.height, lead, 256);
        let last = partition.last().expect("non-empty");
        assert_eq!((last.end - last.start) % 2, 1, "{path}: an odd remainder");
        assert_strips_tile(&sensor, &src, &LUMINANCE, &partition, path);
    }
}

/// `M̂`, the camera → linear-sRGB matrix recovered from the three one-hot rows
/// (each `M`'s row to `2^-24`), and its inverse.
fn recovered_inverse(sensor: &Sensor) -> [[f64; 3]; 3] {
    let m: Vec<[f64; 3]> = ONE_HOT
        .iter()
        .map(|w| {
            mono_mix_row(sensor, w)
                .expect("row")
                .map(|q| f64::from(q) / 16_777_216.0)
        })
        .collect();
    let [a, b, c] = [m[0], m[1], m[2]];
    let det = a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
        + a[2] * (b[0] * c[1] - b[1] * c[0]);
    let cof = [
        [
            b[1] * c[2] - b[2] * c[1],
            a[2] * c[1] - a[1] * c[2],
            a[1] * b[2] - a[2] * b[1],
        ],
        [
            b[2] * c[0] - b[0] * c[2],
            a[0] * c[2] - a[2] * c[0],
            a[2] * b[0] - a[0] * b[2],
        ],
        [
            b[0] * c[1] - b[1] * c[0],
            a[1] * c[0] - a[0] * c[1],
            a[0] * b[1] - a[1] * b[0],
        ],
    ];
    cof.map(|row| row.map(|v| v / det))
}

#[test]
fn ac4_luminance_matches_the_unclamped_plane() {
    let w = [0.2126, 0.7152, 0.0722];
    for path in HELD {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let q = mono_mix_row(&sensor, &LUMINANCE).expect("row");
        let negative = has_negative(&q);
        let n = pixels(&sensor);
        let mut f = vec![0f32; n * 3];
        srgb_linear_plane_f32_into(&sensor, &src, &mut f).expect("f32 plane");
        let mono = mono(&sensor, &src, &LUMINANCE);
        let inverse = recovered_inverse(&sensor);
        let (mut kept, mut worst) = (0usize, 0f64);
        for (p, &out) in mono.iter().enumerate() {
            let fj = [f[3 * p], f[3 * p + 1], f[3 * p + 2]].map(f64::from);
            if negative {
                // x̂ = M̂⁻¹·(65535·f): keep pixels the fade cannot reach.
                let x =
                    inverse.map(|row| 65535.0 * (row[0] * fj[0] + row[1] * fj[1] + row[2] * fj[2]));
                if x.iter().copied().fold(f64::MIN, f64::max) > 57335.0 {
                    continue;
                }
            }
            kept += 1;
            let reference =
                (65535.0 * (w[0] * fj[0] + w[1] * fj[1] + w[2] * fj[2])).clamp(0.0, 65535.0);
            let fmax = fj.iter().map(|v| v.abs()).fold(0.0, f64::max);
            let bound = 0.5 + 3.0 * 65535.0 / 33_554_432.0 + 65535.0 * fmax / 16_777_216.0 + 1e-6;
            let d = (f64::from(out) - reference).abs();
            worst = worst.max(d);
            assert!(
                d <= bound,
                "{path}: pixel {p}: mono {out}, reference {reference}, bound {bound}"
            );
        }
        let fraction = kept as f64 / n as f64;
        eprintln!(
            "AC4 {path}: row {q:?} (negative entry: {negative}); kept {kept} of {n} ({:.4}%); largest deviation {worst:.6} codes",
            100.0 * fraction
        );
        if negative {
            assert!(
                fraction >= 0.9,
                "{path}: only {:.2}% kept",
                100.0 * fraction
            );
        } else {
            assert_eq!(kept, n);
        }
        assert_eq!(
            negative,
            matches!(path, CANON | NIKON),
            "{path}: which rows have a negative entry"
        );
    }
    // The clamp corrupts a mix: against the clamped u16 plane, some pixels are
    // more than one code off.
    for path in [CANON, PIXEL_4A] {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let mut u = vec![0u16; pixels(&sensor) * 3];
        srgb_linear_plane_into(&sensor, &src, &mut u).expect("u16 plane");
        let mono = mono(&sensor, &src, &LUMINANCE);
        let off = mono
            .iter()
            .enumerate()
            .filter(|&(p, &out)| {
                let mixed = w[0] * f64::from(u[3 * p])
                    + w[1] * f64::from(u[3 * p + 1])
                    + w[2] * f64::from(u[3 * p + 2]);
                (f64::from(out) - (mixed + 0.5).floor()).abs() > 1.0
            })
            .count();
        eprintln!(
            "AC4 {path}: {off} pixels ({:.2}%) more than one code from the clamped plane's mix",
            100.0 * off as f64 / mono.len() as f64
        );
        assert!(off > 0, "{path}: the clamped plane's mix never differs");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5: golden rows and digests of the held files.
// ─────────────────────────────────────────────────────────────────────────────

/// `(file, [luminance, red, ortho] rows)`, probe-measured 2026-11-25
/// (`SPEC-092` AC5). A mismatch is reported, never edited to pass; a new
/// golden value needs a `MONO_MIX_VERSION` bump in the same change.
const GOLDEN_ROWS: [(&str, [[i32; 3]; 3]); 5] = [
    (
        CANON,
        [
            [6_222_052, 14_856_368, -4_008_594],
            [43_457_289, -9_545_613, -44_314],
            [-3_895_542, 7_681_333, 24_802_109],
        ],
    ),
    (
        PIXEL_3A,
        [
            [9_238_644, 11_642_451, 1_534_676],
            [33_977_479, 1_862_915, -432_667],
            [413_112, 8_768_457, 11_975_737],
        ],
    ),
    (
        PIXEL_4A,
        [
            [6_580_281, 12_748_060, 1_667_508],
            [25_202_743, 4_872_113, 209_979],
            [1_344_711, 7_581_953, 15_016_856],
        ],
    ),
    (
        PIXEL_7_PRO,
        [
            [5_656_251, 11_930_046, 2_168_331],
            [21_211_951, 3_142_429, 5_765],
            [21_010, 8_388_347, 14_980_760],
        ],
    ),
    (
        NIKON,
        [
            [9_666_745, 12_584_293, -2_609_097],
            [36_952_549, -3_543_768, -4_355_900],
            [896_662, 6_844_397, 12_247_218],
        ],
    ),
];

/// 64-bit FNV-1a, as `tests/determinism.rs`.
fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// `(file, [luminance, red, ortho] mono-plane digests)`: FNV-1a over each
/// `u16`, little-endian. Recorded at SPEC-092's build on macOS arm64 (a local
/// record; the corpus is not in CI). Never edit one to make a run pass.
const GOLDEN_DIGESTS: [(&str, [u64; 3]); 5] = [
    (
        CANON,
        [
            0xf5b2_d791_3c4b_841c,
            0x8209_545e_a5a3_0323,
            0xc48b_0756_2cd2_228c,
        ],
    ),
    (
        PIXEL_3A,
        [
            0xa95d_e8a9_4cc9_276d,
            0x97ca_2799_20a6_9c85,
            0x033a_2108_715c_b32e,
        ],
    ),
    (
        PIXEL_4A,
        [
            0x46fa_545d_cdd1_fdcc,
            0x6dcb_a961_5f58_852a,
            0x4c7e_dfd2_6880_15c9,
        ],
    ),
    (
        PIXEL_7_PRO,
        [
            0x4443_fa3b_6795_3787,
            0xb52a_9e5a_4549_db21,
            0xfe77_f164_558c_3e5d,
        ],
    ),
    (
        NIKON,
        [
            0x0287_b989_6a6a_f991,
            0xd060_9abb_fb0b_2766,
            0x39b6_7f36_f3a6_4295,
        ],
    ),
];

#[test]
fn ac5_golden_rows_and_digests_of_the_held_files() {
    // The golden values below are mixer version 1's: a change to any of them
    // needs a MONO_MIX_VERSION bump in the same change.
    assert_eq!(MONO_MIX_VERSION, "1");
    let mut mismatched = Vec::new();
    for ((path, rows), (digest_path, digests)) in GOLDEN_ROWS.iter().zip(&GOLDEN_DIGESTS) {
        assert_eq!(path, digest_path);
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        for (i, weights) in [LUMINANCE, RED, ORTHO].iter().enumerate() {
            let q = mono_mix_row(&sensor, weights).expect("row");
            let digest = fnv1a(
                mono(&sensor, &src, weights)
                    .iter()
                    .flat_map(|v| v.to_le_bytes()),
            );
            eprintln!("AC5 {path} set {i}: row {q:?} digest {digest:#018x}");
            if q != rows[i] {
                mismatched.push(format!("{path} set {i}: row {q:?}, golden {:?}", rows[i]));
            }
            if digest != digests[i] {
                mismatched.push(format!("{path} set {i}: digest {digest:#018x}"));
            }
        }
    }
    assert!(mismatched.is_empty(), "diverged: {mismatched:#?}");
}
