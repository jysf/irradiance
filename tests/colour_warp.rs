//! `SPEC-100` (`DEC-088` item 2): the colour develop applies a **mandatory**
//! first `WarpRectilinear` in DNG's stage order: SPEC-099's CA step on the
//! demosaiced camera triple, then green's set `W` on each camera plane over the
//! `ActiveArea`, then the colour matrix, then the `DefaultCrop` and the
//! orientation. An optional first warp is still skipped.
//!
//! **The composition** every case is checked against is built from public
//! primitives and the tests' own CA reference (`tests/support/ca_reference.rs`),
//! never from the colour develop's code: `demosaic_into` and
//! `to_srgb_linear_into` on the `Sensor` with `OpcodeList3` removed (black 0,
//! white 65535, so the fused plane is exactly their composition),
//! `ca_reference::reference` for the CA step, `apply_warp_into(W, ..)` per
//! camera plane over the `ActiveArea`, and this file's own crop and TIFF
//! orientation table.
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
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, srgb_linear_plane_into, to_srgb_linear_into,
};
use irradiance::demosaic::demosaic_into;
use irradiance::develop::{develop_with_into, output_dimensions, DevelopOptions};
use irradiance::ifd::{
    ActiveArea, Container, DefaultCropOrigin, DefaultCropSize, Rational, Sensor,
};
use irradiance::opcode::{parse_warp_rectilinear, WarpRect};
use irradiance::plane::unpack_into;
use irradiance::warp::apply_warp_into;
use irradiance::Error;

use ca_reference::Rule;
use opcode_support::{opcode_list, raw_opcode};
use tiff::Order;

const W: u32 = 48;
const H: u32 = 40;
const MANDATORY: u32 = 0;
const OPTIONAL: u32 = 1;
const SENTINEL: u16 = 0xA5A5;

/// One coefficient set: `kr0..kr3`, `kt0, kt1`.
type Set = ([f64; 4], [f64; 2]);

const NO_KT: [f64; 2] = [0.0, 0.0];

/// SPEC-099's sets (`tests/plane_warp.rs`): green not the identity and
/// `f_g(1) = 0.9`, red about 3.4 px of CA at the corner, blue about 2 px the
/// other way.
const GREEN: Set = ([1.0, -0.05, -0.05, 0.0], NO_KT);
const RED: Set = ([1.08, -0.04, -0.05, 0.01], NO_KT);
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

fn warp_list(sets: &[Set], cx: f64, cy: f64, flags: u32) -> Vec<u8> {
    opcode_list(&[raw_opcode(
        1,
        0x0103_0000,
        flags,
        &warp_params(sets, cx, cy),
    )])
}

/// `set` as the one warp of an `N = 1` list, through the public parser.
fn as_warp(set: Set, cx: f64, cy: f64) -> WarpRect {
    parse_warp_rectilinear(&warp_list(&[set], cx, cy, MANDATORY))
        .unwrap()
        .unwrap()
}

/// A textured mosaic (as `tests/plane_warp.rs`): a ramp plus a fine pattern,
/// so a sub-pixel resample changes values everywhere.
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

/// The scene's three geometry variants (AC3).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Variant {
    /// No `ActiveArea`; a `DefaultCrop` (3, 2) 42×35.
    Plain,
    /// `ActiveArea` (top 2, left 2, bottom 38, right 46) and a `DefaultCrop`
    /// (1, 2) 40×31 inside it.
    Active,
    /// As `Plain`, with the optical centre at (0.3, 0.6).
    OffCentre,
}

impl Variant {
    const ALL: [Variant; 3] = [Variant::Plain, Variant::Active, Variant::OffCentre];

    fn centre(self) -> (f64, f64) {
        match self {
            Variant::OffCentre => (0.3, 0.6),
            _ => (0.5, 0.5),
        }
    }
}

/// The 48×40 RGGB DNG over `src` with `list3`, a colour calibration (as
/// `tests/plane_warp.rs`), `variant`'s geometry and `orientation`.
fn sensor_with(src: &[u16], list3: Option<&[u8]>, variant: Variant, orientation: u32) -> Sensor {
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
    let (origin, size) = match variant {
        Variant::Active => {
            sensor.active_area = Some(ActiveArea {
                top: 2,
                left: 2,
                bottom: 38,
                right: 46,
            });
            ((1, 2), (40, 31))
        }
        _ => ((3, 2), (42, 35)),
    };
    sensor.default_crop_origin = Some(DefaultCropOrigin {
        x: origin.0,
        y: origin.1,
    });
    sensor.default_crop_size = Some(DefaultCropSize {
        width: size.0,
        height: size.1,
    });
    sensor.orientation = Some(orientation);
    sensor
}

/// `sensor` with `OpcodeList3` removed.
fn without_list_3(sensor: &Sensor) -> Sensor {
    let mut s = sensor.clone();
    s.opcode_list_3 = None;
    s.opcode_lists[2] = false;
    s
}

fn active_of(sensor: &Sensor) -> (u32, u32, u32, u32) {
    sensor
        .active_area
        .map_or((0, 0, H, W), |a| (a.top, a.left, a.bottom, a.right))
}

/// Output pixel `(ox, oy)`'s position in the crop, from the TIFF
/// `Orientation` table (row 0 / column 0 of the stored image): this file's own.
fn crop_of(orientation: u32, ox: u32, oy: u32, cw: u32, ch: u32) -> (u32, u32) {
    match orientation {
        1 => (ox, oy),
        2 => (cw - 1 - ox, oy),
        3 => (cw - 1 - ox, ch - 1 - oy),
        4 => (ox, ch - 1 - oy),
        5 => (oy, ox),
        6 => (oy, ch - 1 - ox),
        7 => (cw - 1 - oy, ch - 1 - ox),
        8 => (cw - 1 - oy, ox),
        _ => unreachable!(),
    }
}

/// Crop and orient an interleaved plane-sized RGB image as the colour develop
/// frames it (`DefaultCrop` inside the `ActiveArea`, then the orientation).
fn crop_and_orient(sensor: &Sensor, plane: &[u16]) -> Vec<u16> {
    let (top, left, _, _) = active_of(sensor);
    let origin = sensor.default_crop_origin.unwrap();
    let size = sensor.default_crop_size.unwrap();
    let (cw, ch) = (size.width, size.height);
    let o = sensor.orientation.unwrap_or(1);
    let (ow, oh) = if o >= 5 { (ch, cw) } else { (cw, ch) };
    let mut out = Vec::with_capacity((ow * oh * 3) as usize);
    for oy in 0..oh {
        for ox in 0..ow {
            let (cx, cy) = crop_of(o, ox, oy, cw, ch);
            let (x, y) = (left + origin.x + cx, top + origin.y + cy);
            let i = ((y * W + x) * 3) as usize;
            out.extend_from_slice(&plane[i..i + 3]);
        }
    }
    out
}

/// `apply_warp_into(w, ..)` on each camera plane over the `ActiveArea`; the
/// masked border as it is.
fn warp_planes(sensor: &Sensor, cam: &[u16], w: &WarpRect) -> Vec<u16> {
    let (top, left, bottom, right) = active_of(sensor);
    let (aw, ah) = (right - left, bottom - top);
    let mut out = cam.to_vec();
    for c in 0..3 {
        let plane: Vec<u16> = (top..bottom)
            .flat_map(|y| (left..right).map(move |x| (x, y)))
            .map(|(x, y)| cam[((y * W + x) * 3) as usize + c])
            .collect();
        let mut warped = vec![0u16; plane.len()];
        apply_warp_into(w, aw, ah, &plane, &mut warped).unwrap();
        for (i, v) in warped.into_iter().enumerate() {
            let (x, y) = (left + i as u32 % aw, top + i as u32 / aw);
            out[((y * W + x) * 3) as usize + c] = v;
        }
    }
    out
}

/// The no-list-3 camera triple, through the public demosaic.
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

/// AC3's composition: crop and orientation of
/// `to_srgb_linear_into(warp_planes(reference(demosaic_into(src))))`, the CA
/// step left out when `rule` is `None`.
fn composition(sensor: &Sensor, src: &[u16], rule: Option<&Rule>, w: &WarpRect) -> Vec<u16> {
    let cam = camera(sensor, src);
    let ca = match rule {
        Some(rule) => ca_reference::reference(&cam, W as usize, active_of(sensor), rule),
        None => cam,
    };
    crop_and_orient(sensor, &srgb(sensor, &warp_planes(sensor, &ca, w)))
}

/// The rejected post-matrix order: `apply_warp_into` per channel of
/// `srgb_linear_plane_into`'s plane, then the crop.
fn post_matrix(sensor: &Sensor, src: &[u16], w: &WarpRect) -> Vec<u16> {
    let mut lin = vec![0u16; (W * H * 3) as usize];
    srgb_linear_plane_into(sensor, src, &mut lin).expect("the sensor-geometry plane");
    crop_and_orient(sensor, &warp_planes(sensor, &lin, w))
}

fn out_len(sensor: &Sensor) -> usize {
    let (w, h) = output_dimensions(sensor).expect("dims");
    (w * h * 3) as usize
}

/// The four colour entries' outputs (`f32` as bits), the threaded forms at
/// `threads`.
fn four(sensor: &Sensor, src: &[u16], threads: usize) -> Result<[Vec<u32>; 4], Error> {
    let n = out_len(sensor);
    let widen = |v: Vec<u16>| v.into_iter().map(u32::from).collect::<Vec<_>>();
    let bits = |v: Vec<f32>| v.into_iter().map(f32::to_bits).collect::<Vec<_>>();
    let mut a = vec![SENTINEL; n];
    develop_color_into(sensor, src, &mut a)?;
    let mut b = vec![0f32; n];
    develop_color_f32_into(sensor, src, &mut b)?;
    let mut c = vec![SENTINEL; n];
    develop_color_into_threaded(sensor, src, &mut c, threads)?;
    let mut d = vec![0f32; n];
    develop_color_f32_into_threaded(sensor, src, &mut d, threads)?;
    Ok([widen(a), bits(b), widen(c), bits(d)])
}

fn develop_u16(sensor: &Sensor, src: &[u16]) -> Result<Vec<u16>, Error> {
    let mut dst = vec![SENTINEL; out_len(sensor)];
    develop_color_into(sensor, src, &mut dst).map(|()| dst)
}

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

fn developed_mix(sensor: &Sensor, src: &[u16]) -> Result<Vec<u16>, Error> {
    let mut o = DevelopOptions::default();
    o.mono_mix = Some(LUMA);
    let (w, h) = irradiance::develop::output_dimensions_with(sensor, &o)?;
    let mut dst = vec![SENTINEL; (w * h) as usize];
    develop_with_into(sensor, src, &mut dst, &o).map(|()| dst)
}

fn full_rule(variant: Variant) -> Rule {
    let (cx, cy) = variant.centre();
    Rule {
        green: GREEN.0,
        red: Some(RED.0),
        blue: Some(BLUE.0),
        cx,
        cy,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: the unchanged and identity cases are bit-identical
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_unchanged_cases_are_bit_identical() {
    let src = mosaic();
    for variant in [Variant::Plain, Variant::Active] {
        let plain = sensor_with(&src, None, variant, 1);
        let want = four(&plain, &src, 1).expect("the no-list-3 develop");
        let optional: [(&str, Vec<u8>); 3] = [
            (
                "optional N = 1 radial",
                warp_list(&[RED], 0.5, 0.5, OPTIONAL),
            ),
            (
                "optional N = 3 differing",
                warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, OPTIONAL),
            ),
            (
                "optional malformed N = 2",
                warp_list(&[RED, GREEN], 0.5, 0.5, OPTIONAL),
            ),
        ];
        for (case, list) in optional {
            let sensor = sensor_with(&src, Some(&list), variant, 1);
            for threads in [1, 4] {
                let got = four(&sensor, &src, threads)
                    .unwrap_or_else(|e| panic!("{variant:?} {case} at {threads}: {e:?}"));
                assert!(
                    got == want,
                    "{variant:?} {case} at {threads} threads: differs from no list 3"
                );
            }
        }

        // A mandatory N = 3 warp with identical sets is the mandatory N = 1
        // warp with that set, on the colour develop and on the developed mix.
        let n1 = sensor_with(
            &src,
            Some(&warp_list(&[RED], 0.5, 0.5, MANDATORY)),
            variant,
            1,
        );
        let n3 = sensor_with(
            &src,
            Some(&warp_list(&[RED, RED, RED], 0.5, 0.5, MANDATORY)),
            variant,
            1,
        );
        let a = four(&n1, &src, 4).expect("mandatory N = 1");
        assert!(
            a != want,
            "{variant:?}: the mandatory N = 1 warp is applied"
        );
        assert!(
            four(&n3, &src, 4).expect("mandatory identical N = 3") == a,
            "{variant:?}: identical N = 3 equals N = 1 (colour)"
        );
        assert_eq!(
            developed_mix(&n3, &src).expect("mix, identical N = 3"),
            developed_mix(&n1, &src).expect("mix, N = 1"),
            "{variant:?}: identical N = 3 equals N = 1 (mix)"
        );

        // A mandatory identity warp (N = 3, three identity sets) copies.
        let identity = ([1.0, 0.0, 0.0, 0.0], NO_KT);
        let id3 = sensor_with(
            &src,
            Some(&warp_list(
                &[identity, identity, identity],
                0.5,
                0.5,
                MANDATORY,
            )),
            variant,
            1,
        );
        assert!(
            four(&id3, &src, 4).expect("mandatory identity") == want,
            "{variant:?}: an identity warp is the unwarped develop"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: the colour develop is the DNG-order composition, exactly
// ─────────────────────────────────────────────────────────────────────────────

#[track_caller]
fn assert_composition(sensor: &Sensor, src: &[u16], rule: Option<&Rule>, w: &WarpRect, case: &str) {
    let got = develop_u16(sensor, src).unwrap_or_else(|e| panic!("{case}: {e:?}"));
    let want = composition(sensor, src, rule, w);
    let differ = got.iter().zip(&want).filter(|(a, b)| a != b).count();
    assert!(
        got.len() == want.len() && differ == 0,
        "{case}: {differ} of {} samples differ from the composition",
        got.len()
    );
    let mut f = vec![0f32; got.len()];
    develop_color_f32_into(sensor, src, &mut f).unwrap_or_else(|e| panic!("{case}: f32 {e:?}"));
    assert!(
        f.iter()
            .zip(&got)
            .all(|(a, &b)| a.to_bits() == (f32::from(b) / 65535.0).to_bits()),
        "{case}: the f32 entry is the u16 one divided by 65535"
    );
}

#[test]
fn ac3_the_colour_develop_is_the_dng_order_composition() {
    let src = mosaic();
    let w = as_warp(GREEN, 0.5, 0.5);
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain, 1);
    let rule = full_rule(Variant::Plain);
    assert_composition(&sensor, &src, Some(&rule), &w, "plain");
    let got = develop_u16(&sensor, &src).unwrap();
    // Neither shortcut passes.
    assert_ne!(
        got,
        composition(&sensor, &src, None, &w),
        "the CA step left out"
    );
    assert_ne!(got, post_matrix(&sensor, &src, &w), "the post-matrix order");
    // W is green's set, not red's (K1).
    assert_ne!(
        got,
        composition(&sensor, &src, Some(&rule), &as_warp(RED, 0.5, 0.5)),
        "red's set as the warp"
    );

    // Only red differs: blue is not resampled.
    let list = warp_list(&[RED, GREEN, GREEN], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain, 1);
    let red_only = Rule {
        blue: None,
        ..full_rule(Variant::Plain)
    };
    assert_composition(&sensor, &src, Some(&red_only), &w, "red only");

    // A mandatory N = 1 warp: no CA step, W on all three planes.
    let list = warp_list(&[GREEN], 0.5, 0.5, MANDATORY);
    let sensor = sensor_with(&src, Some(&list), Variant::Plain, 1);
    assert_composition(&sensor, &src, None, &w, "N = 1");
}

#[test]
fn ac3_every_orientation_and_the_active_area() {
    let src = mosaic();
    for variant in Variant::ALL {
        let (cx, cy) = variant.centre();
        let w = as_warp(GREEN, cx, cy);
        let list = warp_list(&[RED, GREEN, BLUE], cx, cy, MANDATORY);
        let rule = full_rule(variant);
        for orientation in 1..=8 {
            let sensor = sensor_with(&src, Some(&list), variant, orientation);
            let case = format!("{variant:?} orientation {orientation}");
            assert_composition(&sensor, &src, Some(&rule), &w, &case);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6: refusals and their order
// ─────────────────────────────────────────────────────────────────────────────

/// The four colour entries' results from sentinel buffers (threaded at 3), and
/// whether every buffer is untouched.
fn four_results(sensor: &Sensor, src: &[u16]) -> Vec<(Result<(), Error>, bool)> {
    let n = out_len(sensor);
    let mut out = Vec::new();
    let mut a = vec![SENTINEL; n];
    let r = develop_color_into(sensor, src, &mut a);
    out.push((r, a.iter().all(|&v| v == SENTINEL)));
    let mut b = vec![f32::NAN; n];
    let r = develop_color_f32_into(sensor, src, &mut b);
    out.push((r, b.iter().all(|v| v.is_nan())));
    let mut c = vec![SENTINEL; n];
    let r = develop_color_into_threaded(sensor, src, &mut c, 3);
    out.push((r, c.iter().all(|&v| v == SENTINEL)));
    let mut d = vec![f32::NAN; n];
    let r = develop_color_f32_into_threaded(sensor, src, &mut d, 3);
    out.push((r, d.iter().all(|v| v.is_nan())));
    out
}

#[track_caller]
fn assert_refused(sensor: &Sensor, src: &[u16], case: &str, pred: impl Fn(&Error) -> bool) {
    for (i, (r, untouched)) in four_results(sensor, src).into_iter().enumerate() {
        match r {
            Err(e) if pred(&e) => assert!(untouched, "{case}: entry {i} wrote before {e:?}"),
            other => panic!("{case}: entry {i}: unexpected {other:?}"),
        }
    }
}

#[test]
fn ac6_colour_refusals_and_their_order() {
    let src = mosaic();
    let blue_kt = (BLUE.0, [0.001, -0.002]);
    let fold = ([1.0, -2.0, 0.0, 0.0], NO_KT);
    let kt = ([1.0, -0.05, -0.05, 0.0], [0.001, -0.002]);
    type Case = (&'static str, Vec<Set>, fn(&Error) -> bool);
    let cases: [Case; 6] = [
        ("N = 2", vec![RED, GREEN], |e| {
            matches!(e, Error::MalformedOpcodeParams { id: 1, .. })
        }),
        ("N = 2, identical", vec![GREEN, GREEN], |e| {
            matches!(e, Error::MalformedOpcodeParams { id: 1, .. })
        }),
        (
            "blue kt",
            vec![RED, GREEN, blue_kt],
            |e| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, kt1 } if *kt0 == 0.001 && *kt1 == -0.002),
        ),
        (
            "folding green",
            vec![RED, fold, BLUE],
            |e| matches!(e, Error::UnsupportedWarpFold { slope_min } if *slope_min == -5.0),
        ),
        (
            "identical sets with kt",
            vec![kt, kt, kt],
            |e| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, kt1 } if *kt0 == 0.001 && *kt1 == -0.002),
        ),
        (
            "N = 1 with kt",
            vec![kt],
            |e| matches!(e, Error::UnsupportedWarpTangentialTerms { kt0, kt1 } if *kt0 == 0.001 && *kt1 == -0.002),
        ),
    ];
    let plain = sensor_with(&src, None, Variant::Plain, 1);
    let want = four(&plain, &src, 3).unwrap();
    for (case, sets, pred) in cases {
        let list = warp_list(&sets, 0.5, 0.5, MANDATORY);
        let sensor = sensor_with(&src, Some(&list), Variant::Plain, 1);
        assert_refused(&sensor, &src, case, pred);

        // An unsupported CFA beats it.
        let mut s = sensor.clone();
        s.color_filter.as_mut().unwrap().pattern = vec![0, 0, 1, 2];
        assert_refused(&s, &src, &format!("{case}, CFA first"), |e| {
            matches!(e, Error::UnsupportedCfaPattern { .. })
        });
        // It beats MissingColorCalibration.
        let mut s = sensor.clone();
        s.forward_matrix_1 = None;
        s.color_matrix_1 = None;
        s.color_matrix_2 = None;
        s.forward_matrix_2 = None;
        assert_refused(&s, &src, &format!("{case}, before calibration"), pred);

        // Flagged optional, the same list develops as with no list 3.
        let list = warp_list(&sets, 0.5, 0.5, OPTIONAL);
        let s = sensor_with(&src, Some(&list), Variant::Plain, 1);
        assert!(
            four(&s, &src, 3).unwrap_or_else(|e| panic!("{case}, optional: {e:?}")) == want,
            "{case}, optional: develops as with no list 3"
        );
    }
    // A valid differing list without a calibration: the calibration refusal.
    let list = warp_list(&[RED, GREEN, BLUE], 0.5, 0.5, MANDATORY);
    let mut s = sensor_with(&src, Some(&list), Variant::Plain, 1);
    s.forward_matrix_1 = None;
    s.color_matrix_1 = None;
    assert_refused(&s, &src, "calibration after the warp", |e| {
        matches!(e, Error::MissingColorCalibration)
    });
    // A second mandatory warp is still refused.
    let op = raw_opcode(1, 0x0103_0000, MANDATORY, &warp_params(&[GREEN], 0.5, 0.5));
    let s = sensor_with(
        &src,
        Some(&opcode_list(&[op.clone(), op])),
        Variant::Plain,
        1,
    );
    assert_refused(&s, &src, "a second mandatory warp", |e| {
        matches!(e, Error::UnsupportedRequiredOpcode { list: 3, id: 1 })
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8: threads
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac8_threads_give_the_same_bytes() {
    let src = mosaic();
    for variant in Variant::ALL {
        let (cx, cy) = variant.centre();
        for sets in [vec![RED, GREEN, BLUE], vec![RED, GREEN, GREEN], vec![GREEN]] {
            let list = warp_list(&sets, cx, cy, MANDATORY);
            for orientation in 1..=8 {
                let sensor = sensor_with(&src, Some(&list), variant, orientation);
                let one = four(&sensor, &src, 1).unwrap();
                assert!(one[0] == one[2] && one[1] == one[3], "threaded at 1");
                for threads in [2, 3, 8] {
                    let got = four(&sensor, &src, threads).unwrap();
                    assert!(
                        got[2] == one[0] && got[3] == one[1],
                        "{variant:?} {} sets, orientation {orientation}, {threads} threads",
                        sets.len()
                    );
                }
            }
        }
    }
}
