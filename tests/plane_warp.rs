//! `SPEC-099` (`DEC-087`): the per-plane `WarpRectilinear` at sensor geometry.
//!
//! On a first `WarpRectilinear` with `N = 3` whose red or blue set differs from
//! green's, the six sensor-geometry entries resample red and blue to green's
//! geometry on the demosaiced camera triple, before the matrix, and leave
//! green's set to the consumer (`opcode::sensor_geometry_warp`).
//!
//! **The reference** (`tests/support/ca_reference.rs`) is the tests' own
//! implementation of `DEC-087` item 3, written from the decision's text: the 4096-interval table
//! of `f_c/f_g` over `σ = ρ²` with its nodes solved by bisection, the ratio held
//! at the rim beyond green's image, the source `c + k·(x − c)`, and `DEC-024`'s
//! bilinear kernel clamped to the `ActiveArea` and rounded half up. It calls no
//! crate code for any of that. The entries are tied to it through the public
//! stage primitives: with the CA step removed, the fused plane is exactly
//! `to_srgb_linear_into(demosaic_into(src))` (black 0, white 65535), so the
//! expected plane is `to_srgb_linear_into(reference(demosaic_into(src)))`.
//!
//! **Tier A**: synthetic 48×40 RGGB mosaics through the real container read.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

#[path = "support/ca_reference.rs"]
mod ca_reference;
#[path = "support/opcode.rs"]
mod opcode_support;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::color::{
    develop_color_f32_into, develop_color_into, mono_mix_plane_into, mono_mix_plane_rows_into,
    srgb_linear_plane_f32_into, srgb_linear_plane_into, srgb_linear_plane_rows_f32_into,
    srgb_linear_plane_rows_into, to_srgb_linear_into,
};
use irradiance::demosaic::demosaic_into;
use irradiance::develop::{develop_with_into, output_dimensions_with, DevelopOptions};
use irradiance::ifd::{ActiveArea, Container, Rational, Sensor};
use irradiance::opcode::{parse_warp_rectilinear, sensor_geometry_warp, WarpRect};
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind};

use ca_reference::{f, Rule};
use opcode_support::{opcode_list, raw_opcode};
use tiff::Order;

const W: u32 = 48;
const H: u32 = 40;
const MANDATORY: u32 = 0;
const OPTIONAL: u32 = 1;

/// One coefficient set: `kr0..kr3`, `kt0, kt1`.
type Set = ([f64; 4], [f64; 2]);

const NO_KT: [f64; 2] = [0.0, 0.0];

/// Green: not the identity, and `f_g(1) = 0.9`, so the corner pixels
/// (`σ > 0.81`) lie beyond green's image and reach the held rim.
const GREEN: Set = ([1.0, -0.05, -0.05, 0.0], NO_KT);
/// Red: about 3.4 px of CA at the corner (`f_r(1)/f_g(1) ≈ 1.11`).
const RED: Set = ([1.08, -0.04, -0.05, 0.01], NO_KT);
/// Blue: about 2 px the other way (`f_b(1)/f_g(1) ≈ 0.93`).
const BLUE: Set = ([0.94, -0.06, -0.04, 0.0], NO_KT);

/// `WarpRectilinear` parameters with one entry per set (`N = sets.len()`).
fn warp_params(sets: &[Set], cx: f64, cy: f64) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&u32::try_from(sets.len()).unwrap().to_be_bytes());
    for (kr, kt) in sets {
        for v in kr.iter().chain(kt) {
            p.extend_from_slice(&v.to_be_bytes());
        }
    }
    p.extend_from_slice(&cx.to_be_bytes());
    p.extend_from_slice(&cy.to_be_bytes());
    p
}

fn warp_op(sets: &[Set], cx: f64, cy: f64, flags: u32) -> Vec<u8> {
    raw_opcode(1, 0x0103_0000, flags, &warp_params(sets, cx, cy))
}

fn warp_list(sets: &[Set], cx: f64, cy: f64, flags: u32) -> Vec<u8> {
    opcode_list(&[warp_op(sets, cx, cy, flags)])
}

/// A textured mosaic: a smooth ramp plus a fine pattern, so a sub-pixel
/// resample changes values everywhere, and the Bayer offsets of a colour scene.
fn mosaic() -> Vec<u16> {
    (0..H)
        .flat_map(|y| {
            (0..W).map(move |x| {
                let slot = ((y & 1) * 2 + (x & 1)) as usize;
                let offset = [2000u32, 9000, 9000, 15000][slot];
                let texture = (x * x * 7 + y * y * 5 + x * y * 3) % 997;
                (offset + 300 * x + 200 * y + 20 * texture) as u16
            })
        })
        .collect()
}

/// How the parsed `Sensor` is changed before it develops.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Variant {
    Plain,
    /// `ActiveArea` (top 2, left 2, bottom 38, right 46): a masked border.
    Active,
}

/// The 48×40 RGGB DNG over `src` with `list3`, its colour calibration as in
/// `tests/colour_bad_pixels.rs`, and `variant` applied.
fn sensor_with(src: &[u16], list3: Option<&[u8]>, variant: Variant) -> Sensor {
    let data =
        tiff::dng_16bit_with_opcode_lists(Order::Little, true, W, H, src, [None, None, list3]);
    let container = Container::parse(&data).expect("fixture parses");
    let mut sensor = container.sensor().expect("fixture has a sensor IFD");
    let mut plane = vec![0u16; (W * H) as usize];
    unpack_into(&sensor, container.byte_order(), &data, &mut plane).expect("fixture unpacks");
    assert_eq!(plane, src, "the fixture's mosaic reads back as written");
    sensor.forward_matrix_1 = Some(
        [
            [60000, 30000, 6422],
            [20000, 70000, 10000],
            [10000, 10000, 62521],
        ]
        .iter()
        .flatten()
        .map(|&num| Rational { num, den: 100000 })
        .collect(),
    );
    sensor.as_shot_neutral = Some(vec![
        Rational { num: 1, den: 2 },
        Rational { num: 1, den: 1 },
        Rational { num: 1, den: 2 },
    ]);
    if variant == Variant::Active {
        sensor.active_area = Some(ActiveArea {
            top: 2,
            left: 2,
            bottom: 38,
            right: 46,
        });
    }
    sensor
}

/// `sensor` with `OpcodeList3` removed.
fn without_list_3(sensor: &Sensor) -> Sensor {
    let mut s = sensor.clone();
    s.opcode_list_3 = None;
    s.opcode_lists[2] = false;
    s
}

/// Rec. 709 luma.
const LUMA: [Rational; 3] = [
    Rational {
        num: 2126,
        den: 10000,
    },
    Rational {
        num: 7152,
        den: 10000,
    },
    Rational {
        num: 722,
        den: 10000,
    },
];

// ─────────────────────────────────────────────────────────────────────────────
// The reference (tests/support/ca_reference.rs) on these planes
// ─────────────────────────────────────────────────────────────────────────────

/// The reference over this file's `W`-wide planes.
fn reference(cam: &[u16], active: (u32, u32, u32, u32), rule: &Rule) -> Vec<u16> {
    ca_reference::reference(cam, W as usize, active, rule)
}

/// The no-list-3 camera triple of `sensor`, through the public demosaic.
fn camera(sensor: &Sensor, src: &[u16]) -> Vec<u16> {
    let mut cam = vec![0u16; (W * H * 3) as usize];
    demosaic_into(&without_list_3(sensor), src, &mut cam).expect("demosaic");
    cam
}

fn srgb(sensor: &Sensor, cam: &[u16]) -> Vec<u16> {
    let mut out = vec![0u16; cam.len()];
    to_srgb_linear_into(&without_list_3(sensor), cam, &mut out).expect("to_srgb_linear_into");
    out
}

fn active_of(sensor: &Sensor) -> (u32, u32, u32, u32) {
    sensor
        .active_area
        .map_or((0, 0, H, W), |a| (a.top, a.left, a.bottom, a.right))
}

fn plane_u16(sensor: &Sensor, src: &[u16]) -> Result<Vec<u16>, Error> {
    let mut dst = vec![0u16; (W * H * 3) as usize];
    srgb_linear_plane_into(sensor, src, &mut dst).map(|()| dst)
}

/// The six sensor-geometry entries' outputs (rows forms over two strips),
/// each as `u32`s (an `f32` as its bits).
fn six_entries(sensor: &Sensor, src: &[u16]) -> Vec<Vec<u32>> {
    let widen = |v: Vec<u16>| v.into_iter().map(u32::from).collect::<Vec<_>>();
    let bits = |v: Vec<f32>| v.into_iter().map(f32::to_bits).collect::<Vec<_>>();
    let (wu, n) = (W as usize, (W * H) as usize);
    let mut out = Vec::new();
    out.push(widen(
        plane_u16(sensor, src).expect("srgb_linear_plane_into"),
    ));
    let mut f = vec![0f32; n * 3];
    srgb_linear_plane_f32_into(sensor, src, &mut f).expect("srgb_linear_plane_f32_into");
    out.push(bits(f));
    let mut m = vec![0u16; n];
    mono_mix_plane_into(sensor, src, &LUMA, &mut m).expect("mono_mix_plane_into");
    out.push(widen(m));
    let mut rows_u16 = Vec::new();
    let mut rows_f32 = Vec::new();
    let mut rows_mix = Vec::new();
    for rows in [0..17u32, 17..H] {
        let len = (rows.end - rows.start) as usize * wu;
        let mut a = vec![0u16; len * 3];
        srgb_linear_plane_rows_into(sensor, src, rows.clone(), &mut a).expect("rows u16");
        rows_u16.extend(widen(a));
        let mut b = vec![0f32; len * 3];
        srgb_linear_plane_rows_f32_into(sensor, src, rows.clone(), &mut b).expect("rows f32");
        rows_f32.extend(bits(b));
        let mut c = vec![0u16; len];
        mono_mix_plane_rows_into(sensor, src, &LUMA, rows, &mut c).expect("rows mix");
        rows_mix.extend(widen(c));
    }
    out.extend([rows_u16, rows_f32, rows_mix]);
    out
}

const ENTRY_NAMES: [&str; 6] = [
    "srgb_linear_plane_into",
    "srgb_linear_plane_f32_into",
    "mono_mix_plane_into",
    "srgb_linear_plane_rows_into",
    "srgb_linear_plane_rows_f32_into",
    "mono_mix_plane_rows_into",
];

#[track_caller]
fn assert_same_six(got: &[Vec<u32>], want: &[Vec<u32>], case: &str) {
    for (i, name) in ENTRY_NAMES.iter().enumerate() {
        assert!(
            got[i] == want[i],
            "{case}: {name} differs from the no-list-3 output at {} samples",
            got[i].iter().zip(&want[i]).filter(|(a, b)| a != b).count()
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: no per-plane difference is bit-identical
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_no_per_plane_difference_is_bit_identical() {
    let src = mosaic();
    for variant in [Variant::Plain, Variant::Active] {
        let want = six_entries(&sensor_with(&src, None, variant), &src);
        let kt = ([1.0, -0.05, -0.05, 0.0], [0.001, -0.002]);
        let cases: [(&str, Vec<u8>); 4] = [
            (
                "N = 1, a non-identity set",
                warp_list(&[RED], 0.5, 0.5, MANDATORY),
            ),
            (
                "N = 3, three copies",
                warp_list(&[RED, RED, RED], 0.5, 0.5, MANDATORY),
            ),
            (
                "N = 3, three copies, optional",
                warp_list(&[RED, RED, RED], 0.5, 0.5, OPTIONAL),
            ),
            (
                "N = 3, identical sets with kt",
                warp_list(&[kt, kt, kt], 0.4, 0.6, MANDATORY),
            ),
        ];
        for (case, list) in cases {
            let sensor = sensor_with(&src, Some(&list), variant);
            let got = six_entries(&sensor, &src);
            assert_same_six(&got, &want, &format!("{variant:?} {case}"));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: the rule, exactly
// ─────────────────────────────────────────────────────────────────────────────

#[track_caller]
fn assert_rule(sensor: &Sensor, src: &[u16], rule: &Rule, case: &str) {
    let got = plane_u16(sensor, src).unwrap_or_else(|e| panic!("{case}: {e:?}"));
    let cam = camera(sensor, src);
    let want = srgb(sensor, &reference(&cam, active_of(sensor), rule));
    let differ = got.iter().zip(&want).filter(|(a, b)| a != b).count();
    assert!(
        differ == 0,
        "{case}: {differ} samples differ from the reference"
    );
    let no_ca = srgb(sensor, &cam);
    assert!(got != no_ca, "{case}: the CA step changed nothing");
    // The masked border passes through: every pixel outside the ActiveArea is
    // the no-CA output.
    let (top, left, bottom, right) = active_of(sensor);
    for y in 0..H {
        for x in 0..W {
            if (top..bottom).contains(&y) && (left..right).contains(&x) {
                continue;
            }
            let i = ((y * W + x) * 3) as usize;
            assert_eq!(
                got[i..i + 3],
                no_ca[i..i + 3],
                "{case}: masked pixel ({x}, {y}) moved"
            );
        }
    }
}

#[test]
fn ac3_red_and_blue_are_resampled_to_green_geometry() {
    let src = mosaic();
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let rule = Rule {
        green: GREEN.0,
        red: Some(RED.0),
        blue: Some(BLUE.0),
        cx: 0.5,
        cy: 0.5,
    };
    assert_rule(&sensor, &src, &rule, "R, G, B differ");

    // The corner reaches the held rim: σ = 1 > f_g(1)² = 0.81, and the
    // reference's nodes there are exactly 1.
    let rim = f(GREEN.0, 1.0);
    assert!(rim * rim < 1.0, "the corner lies beyond green's image");

    // The optional flag changes nothing: the CA step runs whatever the flags.
    let optional = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, OPTIONAL);
    let sensor = sensor_with(&src, Some(&optional), Variant::Plain);
    assert_rule(&sensor, &src, &rule, "R, G, B differ, optional");

    // Only red differs: blue is not resampled.
    let list = warp_list(&[RED, GREEN, GREEN], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let rule = Rule {
        green: GREEN.0,
        red: Some(RED.0),
        blue: None,
        cx: 0.5,
        cy: 0.5,
    };
    assert_rule(&sensor, &src, &rule, "only red differs");
    let got = plane_u16(&sensor, &src).unwrap();
    let no_ca = srgb(&sensor, &camera(&sensor, &src));
    // Blue's camera channel is untouched, so wherever red did not move, the
    // pixel is the no-CA one; at least one pixel keeps every channel.
    assert!(
        got.chunks(3).zip(no_ca.chunks(3)).any(|(a, b)| a == b),
        "only red differs: some pixel is unchanged"
    );
}

#[test]
fn ac3_the_active_area_and_the_optical_centre() {
    let src = mosaic();
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Active);
    let rule = Rule {
        green: GREEN.0,
        red: Some(RED.0),
        blue: Some(BLUE.0),
        cx: 0.5,
        cy: 0.5,
    };
    assert_rule(&sensor, &src, &rule, "ActiveArea (2, 2, 38, 46)");

    let list = warp_list(&[RED, GREEN, BLUE], 0.3, 0.6, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let rule = Rule {
        green: GREEN.0,
        red: Some(RED.0),
        blue: Some(BLUE.0),
        cx: 0.3,
        cy: 0.6,
    };
    assert_rule(&sensor, &src, &rule, "cx 0.3, cy 0.6");

    let sensor = sensor_with(&src, Some(&list), Variant::Active);
    assert_rule(&sensor, &src, &rule, "ActiveArea and cx 0.3, cy 0.6");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5: bands equal the whole plane
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac5_bands_equal_the_whole_plane() {
    let src = mosaic();
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let mut tilings: Vec<Vec<core::ops::Range<u32>>> = vec![
        vec![0..3, 3..11, 11..H],
        (0..H).map(|y| y..y + 1).collect(),
        vec![0..H],
    ];
    tilings.push(vec![0..1, 1..39, 39..H]);
    for variant in [Variant::Plain, Variant::Active] {
        let sensor = sensor_with(&src, Some(&list), variant);
        let n = (W * H) as usize;
        let whole_u16 = plane_u16(&sensor, &src).unwrap();
        let mut whole_f32 = vec![0f32; n * 3];
        srgb_linear_plane_f32_into(&sensor, &src, &mut whole_f32).unwrap();
        let mut whole_mix = vec![0u16; n];
        mono_mix_plane_into(&sensor, &src, &LUMA, &mut whole_mix).unwrap();
        assert!(
            whole_u16 != plane_u16(&without_list_3(&sensor), &src).unwrap(),
            "{variant:?}: the CA step ran"
        );
        for tiling in &tilings {
            let mut a = Vec::new();
            let mut b = Vec::new();
            let mut c = Vec::new();
            for rows in tiling {
                let len = (rows.end - rows.start) as usize * W as usize;
                let mut x = vec![0u16; len * 3];
                srgb_linear_plane_rows_into(&sensor, &src, rows.clone(), &mut x).unwrap();
                a.extend(x);
                let mut y = vec![0f32; len * 3];
                srgb_linear_plane_rows_f32_into(&sensor, &src, rows.clone(), &mut y).unwrap();
                b.extend(y);
                let mut z = vec![0u16; len];
                mono_mix_plane_rows_into(&sensor, &src, &LUMA, rows.clone(), &mut z).unwrap();
                c.extend(z);
            }
            let first = tiling.first().unwrap().clone();
            assert!(a == whole_u16, "{variant:?} {first:?}..: u16 bands differ");
            assert!(
                b.iter()
                    .map(|v| v.to_bits())
                    .eq(whole_f32.iter().map(|v| v.to_bits())),
                "{variant:?} {first:?}..: f32 bands differ"
            );
            assert!(
                c == whole_mix,
                "{variant:?} {first:?}..: mixed bands differ"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC1: the accessor
// ─────────────────────────────────────────────────────────────────────────────

/// `set` as `parse_warp_rectilinear` returns a `K1` of an `N`-set list.
fn as_warp(set: Set, cx: f64, cy: f64, planes: u32) -> WarpRect {
    let mut w = WarpRect::test_fixture();
    w.kr = set.0;
    w.kt = set.1;
    w.cx = cx;
    w.cy = cy;
    w.planes = planes;
    w
}

/// A monochrome (`LinearRaw`) sensor with `list3`.
fn mono_sensor(list3: &[u8]) -> Sensor {
    let src = vec![1000u16; (W * H) as usize];
    let data = tiff::dng_16bit_with_opcode_lists(
        Order::Little,
        false,
        W,
        H,
        &src,
        [None, None, Some(list3)],
    );
    let container = Container::parse(&data).expect("fixture parses");
    container.sensor().expect("fixture has a sensor IFD")
}

#[test]
fn ac1_sensor_geometry_warp_names_the_consumer_warp_and_every_set() {
    let src = mosaic();

    // No list 3; a list 3 with no WarpRectilinear.
    let sensor = sensor_with(&src, None, Variant::Plain);
    assert_eq!(sensor_geometry_warp(&sensor).unwrap(), None, "no list 3");
    let unknown = opcode_list(&[raw_opcode(77, 0x0103_0000, OPTIONAL, &[1, 2, 3, 4])]);
    let sensor = sensor_with(&src, Some(&unknown), Variant::Plain);
    assert_eq!(
        sensor_geometry_warp(&sensor).unwrap(),
        None,
        "no warp in list 3"
    );

    // CFA, N = 3, three distinct sets; optional flags give the same answer.
    for flags in [MANDATORY, OPTIONAL] {
        let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, flags);
        let sensor = sensor_with(&src, Some(&list), Variant::Plain);
        let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
        let sets = warp.sets();
        assert_eq!(sets.len(), 3, "flags {flags}: three sets");
        for (i, (set, colour)) in [(RED, 0u8), (GREEN, 1), (BLUE, 2)].into_iter().enumerate() {
            assert_eq!(sets[i].plane, i as u32, "plane index");
            assert_eq!(sets[i].color, Some(colour), "flags {flags}: set {i} colour");
            assert_eq!(
                sets[i].warp,
                as_warp(set, 0.5, 0.5, 3),
                "flags {flags}: set {i}"
            );
        }
        assert_eq!(
            *warp.consumer(),
            as_warp(GREEN, 0.5, 0.5, 3),
            "consumer = green"
        );
        assert!(warp.is_per_plane(), "flags {flags}: per plane");
        // parse_warp_rectilinear is unchanged: K1, red's set.
        assert_eq!(
            parse_warp_rectilinear(&list).unwrap(),
            Some(as_warp(RED, 0.5, 0.5, 3)),
            "parse_warp_rectilinear still returns K1"
        );
    }

    // The colour comes from CFAPlaneColor, not the index: the consumer is the
    // set coloured 1.
    for (order, consumer) in [([2u8, 1, 0], 1usize), ([1, 0, 2], 0), ([0, 2, 1], 2)] {
        let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
        let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
        sensor.color_filter.as_mut().unwrap().plane_color = order.to_vec();
        let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
        let colours: Vec<Option<u8>> = warp.sets().iter().map(|s| s.color).collect();
        assert_eq!(colours, order.map(Some).to_vec(), "{order:?}: colours");
        assert_eq!(
            *warp.consumer(),
            warp.sets()[consumer].warp,
            "{order:?}: consumer is the set coloured 1"
        );
    }

    // Three identical sets: not per plane.
    let list = warp_list(&[GREEN, GREEN, GREEN], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
    assert!(!warp.is_per_plane(), "identical sets");
    assert_eq!(*warp.consumer(), as_warp(GREEN, 0.5, 0.5, 3));

    // Only blue differs: per plane.
    let list = warp_list(&[GREEN, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert!(
        sensor_geometry_warp(&sensor)
            .unwrap()
            .unwrap()
            .is_per_plane(),
        "blue differs"
    );

    // CFA, N = 1.
    let list = warp_list(&[RED], 0.4, 0.6, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
    assert_eq!(warp.sets().len(), 1, "N = 1: one set");
    assert_eq!(warp.sets()[0].color, None, "N = 1: no colour");
    assert_eq!(warp.sets()[0].plane, 0);
    assert_eq!(
        *warp.consumer(),
        as_warp(RED, 0.4, 0.6, 1),
        "N = 1: the one set"
    );
    assert!(!warp.is_per_plane(), "N = 1");

    // Monochrome, N = 1 and N = 3: no colour, the consumer is K1.
    for sets in [vec![RED], vec![RED, GREEN, BLUE]] {
        let list = warp_list(&sets, 0.5, 0.5, MANDATORY);
        let sensor = mono_sensor(&list);
        let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
        assert_eq!(warp.sets().len(), sets.len());
        assert!(
            warp.sets().iter().all(|s| s.color.is_none()),
            "mono: no colour"
        );
        assert_eq!(
            Some(*warp.consumer()),
            parse_warp_rectilinear(&list).unwrap(),
            "mono N = {}: the consumer is K1",
            sets.len()
        );
        assert!(!warp.is_per_plane(), "mono: never per plane");
    }

    // N = 2 and N = 4: malformed.
    for sets in [vec![RED, GREEN], vec![RED, GREEN, BLUE, GREEN]] {
        let list = warp_list(&sets, 0.5, 0.5, MANDATORY);
        let sensor = sensor_with(&src, Some(&list), Variant::Plain);
        let r = sensor_geometry_warp(&sensor);
        assert!(
            matches!(r, Err(Error::MalformedOpcodeParams { id: 1, .. })),
            "N = {}: {r:?}",
            sets.len()
        );
    }

    // CFAPlaneColor [0, 1, 1] with N = 3.
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
    sensor.color_filter.as_mut().unwrap().plane_color = vec![0, 1, 1];
    let r = sensor_geometry_warp(&sensor);
    assert!(
        matches!(&r, Err(Error::UnsupportedCfaPlaneColor { plane_color }) if *plane_color == [0, 1, 1]),
        "{r:?}"
    );

    // A second WarpRectilinear is ignored.
    let list = opcode_list(&[
        warp_op(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY),
        warp_op(&[GREEN, BLUE, RED], 0.2, 0.2, MANDATORY),
    ]);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
    assert_eq!(
        *warp.consumer(),
        as_warp(GREEN, 0.5, 0.5, 3),
        "the first warp wins"
    );

    // A non-zero kt is reported, not refused.
    let blue_kt = (BLUE.0, [0.001, -0.002]);
    let list = warp_list(&[RED, GREEN, blue_kt], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let warp = sensor_geometry_warp(&sensor).unwrap().expect("a warp");
    assert_eq!(warp.sets()[2].warp.kt, [0.001, -0.002], "kt reported");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6: refusals and their order
// ─────────────────────────────────────────────────────────────────────────────

const SENTINEL: u16 = 0xBEEF;

/// Run each of the six entries on a sentinel-filled `dst`; return each result
/// and assert nothing was written on refusal.
fn six_results(sensor: &Sensor, src: &[u16], weights: &[Rational; 3]) -> Vec<Result<(), Error>> {
    let n = (W * H) as usize;
    let mut out = Vec::new();
    let mut a = vec![SENTINEL; n * 3];
    let r = srgb_linear_plane_into(sensor, src, &mut a);
    assert!(
        r.is_ok() || a.iter().all(|&v| v == SENTINEL),
        "u16 wrote on refusal"
    );
    out.push(r);
    let mut b = vec![-7.0f32; n * 3];
    let r = srgb_linear_plane_f32_into(sensor, src, &mut b);
    assert!(
        r.is_ok() || b.iter().all(|&v| v == -7.0),
        "f32 wrote on refusal"
    );
    out.push(r);
    let mut c = vec![SENTINEL; n];
    let r = mono_mix_plane_into(sensor, src, weights, &mut c);
    assert!(
        r.is_ok() || c.iter().all(|&v| v == SENTINEL),
        "mix wrote on refusal"
    );
    out.push(r);
    let rows = 5..9u32;
    let len = 4 * W as usize;
    let mut a = vec![SENTINEL; len * 3];
    let r = srgb_linear_plane_rows_into(sensor, src, rows.clone(), &mut a);
    assert!(
        r.is_ok() || a.iter().all(|&v| v == SENTINEL),
        "rows u16 wrote on refusal"
    );
    out.push(r);
    let mut b = vec![-7.0f32; len * 3];
    let r = srgb_linear_plane_rows_f32_into(sensor, src, rows.clone(), &mut b);
    assert!(
        r.is_ok() || b.iter().all(|&v| v == -7.0),
        "rows f32 wrote on refusal"
    );
    out.push(r);
    let mut c = vec![SENTINEL; len];
    let r = mono_mix_plane_rows_into(sensor, src, weights, rows, &mut c);
    assert!(
        r.is_ok() || c.iter().all(|&v| v == SENTINEL),
        "rows mix wrote on refusal"
    );
    out.push(r);
    out
}

#[track_caller]
fn assert_all(results: &[Result<(), Error>], case: &str, pred: impl Fn(&Error) -> bool) {
    for (r, name) in results.iter().zip(ENTRY_NAMES) {
        match r {
            Err(e) if pred(e) => {}
            other => panic!("{case}: {name}: unexpected {other:?}"),
        }
    }
}

#[test]
fn ac6_refusals_and_their_order() {
    let src = mosaic();

    // A non-zero kt on blue, with differing sets: blue's kt.
    let blue_kt = (BLUE.0, [0.001, -0.002]);
    let list = warp_list(&[RED, GREEN, blue_kt], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "blue kt",
        |e| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, kt1 } if *kt0 == 0.001 && *kt1 == -0.002),
    );
    // The first set in plane order with a non-zero kt wins.
    let red_kt = (RED.0, [0.003, 0.0]);
    let list = warp_list(&[red_kt, GREEN, blue_kt], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "red and blue kt",
        |e| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, .. } if *kt0 == 0.003),
    );

    // A folding green: q(1) = 1 − 6 = −5.
    let fold = ([1.0, -2.0, 0.0, 0.0], NO_KT);
    let list = warp_list(&[RED, fold, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "fold",
        |e| matches!(e, Error::UnsupportedWarpFold { slope_min } if *slope_min == -5.0),
    );
    let err =
        srgb_linear_plane_into(&sensor, &src, &mut vec![0; (W * H * 3) as usize]).unwrap_err();
    assert_eq!(
        err.kind(),
        ErrorKind::Unsupported,
        "UnsupportedWarpFold's kind"
    );
    assert!(
        err.to_string().contains("-5"),
        "Display names the slope: {err}"
    );
    // The fold boundary: q's minimum exactly 0 refuses. kr = [0.5, 0, -0.1, 0]
    // gives q = 0.5 − 0.5·s² (5·0.1 is exactly 0.5 in binary), so q(1) = 0.
    let edge = ([0.5, 0.0, -0.1, 0.0], NO_KT);
    let list = warp_list(&[RED, edge, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "q min exactly 0",
        |e| matches!(e, Error::UnsupportedWarpFold { slope_min } if *slope_min == 0.0),
    );
    // A fold with identical sets is not refused: the warp is the consumer's.
    let list = warp_list(&[fold, fold, fold], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    for (r, name) in six_results(&sensor, &src, &LUMA).iter().zip(ENTRY_NAMES) {
        assert!(r.is_ok(), "fold, identical sets: {name}: {r:?}");
    }

    // N = 2, with differing and with identical sets.
    for sets in [vec![RED, GREEN], vec![GREEN, GREEN]] {
        let list = warp_list(&sets, 0.5, 0.5, MANDATORY);
        let sensor = sensor_with(&src, Some(&list), Variant::Plain);
        let results = six_results(&sensor, &src, &LUMA);
        assert_all(&results, "N = 2", |e| {
            matches!(e, Error::MalformedOpcodeParams { id: 1, .. })
        });
    }

    // Order: an unsupported CFA beats the warp refusal.
    let list = warp_list(&[RED, fold, BLUE], 0.5, 0.5, MANDATORY);
    let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
    sensor.color_filter.as_mut().unwrap().pattern = vec![0, 0, 1, 2];
    assert_all(&six_results(&sensor, &src, &LUMA), "CFA first", |e| {
        matches!(e, Error::UnsupportedCfaPattern { .. })
    });
    let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
    sensor.color_filter.as_mut().unwrap().plane_color = vec![0, 1, 1];
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "CFAPlaneColor first",
        |e| matches!(e, Error::UnsupportedCfaPlaneColor { .. }),
    );
    // The warp refusal beats MissingColorCalibration.
    let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
    sensor.forward_matrix_1 = None;
    sensor.color_matrix_1 = None;
    sensor.color_matrix_2 = None;
    sensor.forward_matrix_2 = None;
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "fold before calibration",
        |e| matches!(e, Error::UnsupportedWarpFold { .. }),
    );
    let ok_list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let mut sensor = sensor_with(&src, Some(&ok_list), Variant::Plain);
    sensor.forward_matrix_1 = None;
    sensor.color_matrix_1 = None;
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "calibration after the warp",
        |e| matches!(e, Error::MissingColorCalibration),
    );
    // InvalidMixWeights beats it on the mix entries.
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let bad_weights = [
        Rational { num: 1, den: 2 },
        Rational { num: 1, den: 2 },
        Rational { num: 1, den: 2 },
    ];
    let results = six_results(&sensor, &src, &bad_weights);
    for i in [2usize, 5] {
        assert!(
            matches!(results[i], Err(Error::InvalidMixWeights { .. })),
            "{}: {:?}",
            ENTRY_NAMES[i],
            results[i]
        );
    }
    // The rows forms' InvalidRowRange stays where it is: before the warp.
    let mut dst = vec![SENTINEL; (W * 3) as usize];
    let r = srgb_linear_plane_rows_into(&sensor, &src, 5..5, &mut dst);
    assert!(matches!(r, Err(Error::InvalidRowRange { .. })), "{r:?}");

    // A 1×1 ActiveArea runs no CA step (m = 0), even with a folding green.
    let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
    sensor.active_area = Some(ActiveArea {
        top: 10,
        left: 10,
        bottom: 11,
        right: 11,
    });
    let got = six_entries(&sensor, &src);
    let want = six_entries(&without_list_3(&sensor), &src);
    assert_same_six(&got, &want, "1×1 ActiveArea");

    // SPEC-100 (`DEC-088`): the developed outputs apply these lists (their own
    // oracles are `tests/colour_warp.rs` and `tests/developed_mix.rs`).
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let (cw, ch) = output_dimensions_with(&sensor, &DevelopOptions::default()).unwrap();
    let rgb = (cw * ch * 3) as usize;
    develop_color_into(&sensor, &src, &mut vec![0; rgb]).expect("develop_color_into");
    develop_color_f32_into(&sensor, &src, &mut vec![0.0; rgb]).expect("develop_color_f32_into");
    let mut options = DevelopOptions::default();
    options.mono_mix = Some(LUMA);
    for flags in [MANDATORY, OPTIONAL] {
        let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, flags);
        let sensor = sensor_with(&src, Some(&list), Variant::Plain);
        let (mw, mh) = output_dimensions_with(&sensor, &options).unwrap();
        let mut with = vec![0u16; (mw * mh) as usize];
        develop_with_into(&sensor, &src, &mut with, &options).expect("the developed mix");
        let mut without = vec![0u16; (mw * mh) as usize];
        develop_with_into(&without_list_3(&sensor), &src, &mut without, &options).unwrap();
        assert_ne!(
            with, without,
            "the developed mix applies N = 3, flags {flags}"
        );
    }
}

/// AC5's halo on a tall, narrow `ActiveArea` (8 × 40): there the vertical reach
/// of an inward resample is nearly its radial reach `B`, so a halo shorter than
/// `⌈B⌉ + 1` reads rows that have not arrived (rule-clause map 3.6).
#[test]
fn ac5_the_halo_covers_a_tall_narrow_active_area() {
    let src = mosaic();
    // Blue shrinks to 0.7 of green at the rim: its sources move inward (never
    // clamped) by up to about 5.9 rows, so the halo is what keeps them.
    let strong_blue = ([0.7, -0.035, -0.035, 0.0], NO_KT);
    let list = warp_list(&[RED, GREEN, strong_blue], 0.5, 0.5, MANDATORY);
    let mut sensor = sensor_with(&src, Some(&list), Variant::Plain);
    sensor.active_area = Some(ActiveArea {
        top: 0,
        left: 20,
        bottom: H,
        right: 28,
    });
    let rule = Rule {
        green: GREEN.0,
        red: Some(RED.0),
        blue: Some(strong_blue.0),
        cx: 0.5,
        cy: 0.5,
    };
    assert_rule(&sensor, &src, &rule, "ActiveArea 8 × 40");
    let whole = plane_u16(&sensor, &src).unwrap();
    let mut bands = Vec::new();
    for y in 0..H {
        let mut x = vec![0u16; W as usize * 3];
        srgb_linear_plane_rows_into(&sensor, &src, y..y + 1, &mut x).unwrap();
        bands.extend(x);
    }
    assert!(bands == whole, "single-row bands equal the plane");
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-099 verify: branches the build's fixtures did not separate
// ─────────────────────────────────────────────────────────────────────────────

/// `DEC-087` items 2 and 5: sets are compared on all six coefficients, so a set
/// that differs from green's in its tangential terms only makes the warp per
/// plane, and the refusal names the first set in plane order with a non-zero
/// `kt`, green's included. Tangential terms are refused before the fold.
/// (Verify gaps: no fixture had sets differing in `kt` alone, a non-zero `kt`
/// on green, or a non-zero `kt` together with a folding green.)
#[test]
fn verify_tangential_terms_count_in_the_comparison_and_come_before_the_fold() {
    let src = mosaic();
    let kt_is = |want: f64| move |e: &Error| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, .. } if *kt0 == want);

    // Red and green share kr; only red's kt differs. The accessor calls it per
    // plane and the entries refuse red's kt.
    let red_kt = (GREEN.0, [0.001, 0.0]);
    let list = warp_list(&[red_kt, GREEN, GREEN], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    let warp = sensor_geometry_warp(&sensor).unwrap().unwrap();
    assert!(warp.is_per_plane(), "a kt-only difference is per plane");
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "kt-only difference",
        kt_is(0.001),
    );

    // Only green carries a kt (kr all equal): green's kt, plane 1.
    let green_kt = (GREEN.0, [0.002, 0.0]);
    let list = warp_list(&[GREEN, green_kt, GREEN], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert!(sensor_geometry_warp(&sensor)
        .unwrap()
        .unwrap()
        .is_per_plane());
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "green kt only",
        kt_is(0.002),
    );

    // Differing kr, kt on green and blue: green's comes first in plane order.
    let green_kt = (GREEN.0, [0.004, 0.0]);
    let blue_kt = (BLUE.0, [0.005, 0.0]);
    let list = warp_list(&[RED, green_kt, blue_kt], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "green before blue",
        kt_is(0.004),
    );

    // A folding green and a kt on blue: the tangential refusal comes first.
    let fold = ([1.0, -2.0, 0.0, 0.0], NO_KT);
    let blue_kt = (BLUE.0, [0.006, 0.0]);
    let list = warp_list(&[RED, fold, blue_kt], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(
        &six_results(&sensor, &src, &LUMA),
        "kt before fold",
        kt_is(0.006),
    );
}

/// `DEC-087` item 5's full order: the levels, the `GainMap`s and `OpcodeList1`
/// are checked before the per-plane warp, on all six entries. Each sensor below
/// carries a folding green and one earlier fault; its result must be the one
/// the same sensor gives with no list 3 at all. (Verify gap: AC6 placed the
/// warp after the CFA and before the calibration, but nothing separated it
/// from the three checks before the CFA.)
#[test]
fn verify_the_levels_gain_maps_and_list_1_come_before_the_warp() {
    let src = mosaic();
    let fold = ([1.0, -2.0, 0.0, 0.0], NO_KT);
    let list = warp_list(&[RED, fold, BLUE], 0.5, 0.5, MANDATORY);
    let base = sensor_with(&src, Some(&list), Variant::Plain);
    assert_all(&six_results(&base, &src, &LUMA), "the fold alone", |e| {
        matches!(e, Error::UnsupportedWarpFold { .. })
    });
    let opcode_version = 0x0103_0000;

    let mut levels = base.clone();
    levels.black_levels = Vec::new();
    levels.black_level = Some(65535);
    levels.white_levels = Vec::new();
    levels.white_level = Some(1000);

    let mut gain_map = base.clone();
    gain_map.opcode_list_2 = Some(opcode_list(&[raw_opcode(9, opcode_version, 0, &[0u8; 8])]));
    gain_map.opcode_lists[1] = true;

    let mut list_1 = base.clone();
    list_1.opcode_list_1 = Some(opcode_list(&[raw_opcode(4, opcode_version, 0, &[0u8; 4])]));
    list_1.opcode_lists[0] = true;

    for (case, sensor, want) in [
        ("levels", &levels, "InvalidLevels"),
        ("GainMap", &gain_map, "MalformedOpcodeParams { id: 9"),
        ("OpcodeList1", &list_1, "MalformedOpcodeParams { id: 4"),
    ] {
        let got = six_results(sensor, &src, &LUMA);
        let without = six_results(&without_list_3(sensor), &src, &LUMA);
        for ((g, w), name) in got.iter().zip(&without).zip(ENTRY_NAMES) {
            let (g, w) = (format!("{g:?}"), format!("{w:?}"));
            assert!(g.starts_with(&format!("Err({want}")), "{case}: {name}: {g}");
            assert_eq!(g, w, "{case}: {name}: the earlier refusal wins");
        }
    }
}

/// The build's deviation 2: a non-finite coefficient in the second or third
/// set is `MalformedOpcodeParams { id: 1, .. }`, from the accessor and from all
/// six entries (the first set and the centre were already checked by
/// `parse_warp_rectilinear`). (Verify gap: the refusal had no test.)
#[test]
fn verify_a_non_finite_later_set_is_malformed() {
    let src = mosaic();
    for (case, sets) in [
        (
            "NaN kr in green",
            [RED, ([f64::NAN, 0.0, 0.0, 0.0], NO_KT), BLUE],
        ),
        (
            "infinite kr in blue",
            [RED, GREEN, ([0.94, f64::INFINITY, 0.0, 0.0], NO_KT)],
        ),
        ("NaN kt in blue", [RED, GREEN, (BLUE.0, [0.0, f64::NAN])]),
        (
            "NaN kt with identical sets",
            [GREEN, GREEN, (GREEN.0, [f64::NAN, 0.0])],
        ),
    ] {
        let list = warp_list(&sets, 0.5, 0.5, MANDATORY);
        let sensor = sensor_with(&src, Some(&list), Variant::Plain);
        let r = sensor_geometry_warp(&sensor);
        assert!(
            matches!(r, Err(Error::MalformedOpcodeParams { id: 1, .. })),
            "{case}: accessor {r:?}"
        );
        assert_all(&six_results(&sensor, &src, &LUMA), case, |e| {
            matches!(e, Error::MalformedOpcodeParams { id: 1, .. })
        });
    }
}
