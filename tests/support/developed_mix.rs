//! `SPEC-093` — shared fixtures for the developed mix (`DEC-084`): the tier-A
//! fixtures (i) to (v), the §2 oracle (the mono develop of the mixed plane read
//! as a mono sensor at `MONO_MIX_BLACK` / `MONO_MIX_WHITE`), and the AC8 fault
//! catalogue F0 to F13.
//!
//! Compiled into `tests/developed_mix.rs` and `tests/develop_scratch_alloc.rs`
//! via `#[path]`; each uses a different subset, so `dead_code` is allowed.
//!
//! The oracle never runs through the mix's own code: it is
//! `mono_mix_plane_into` (the whole plane, its own allocations) followed by
//! `develop_with_into` WITHOUT the mix on `mono_of(sensor)`.

#![allow(dead_code)]

/// `assert_eq!` on the `Debug` text: `Error` is not `PartialEq` (it carries
/// `f64` fields), so results are compared as their debug forms, which name
/// every field.
#[macro_export]
macro_rules! assert_same {
    ($a:expr, $b:expr $(,)?) => {
        assert_eq!(format!("{:?}", $a), format!("{:?}", $b))
    };
    ($a:expr, $b:expr, $($arg:tt)+) => {
        assert_eq!(format!("{:?}", $a), format!("{:?}", $b), $($arg)+)
    };
}

use irradiance::color::{mono_mix_plane_into, mono_mix_row, MONO_MIX_BLACK, MONO_MIX_WHITE};
use irradiance::develop::{
    develop_with_into, develop_with_scratch_into, output_dimensions_with, scratch_len,
    DevelopOptions, Framing,
};
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;

use crate::dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind,
};
use crate::opcode_support::{opcode_list, raw_opcode, single_warp_rectilinear_list};
use crate::tiff::Order;

pub const fn r(num: i32, den: i32) -> Rational {
    Rational { num, den }
}

/// Rec. 709 luminance on linear sRGB.
pub const LUMINANCE: [Rational; 3] = [r(2126, 10000), r(7152, 10000), r(722, 10000)];

/// The one-hot weight sets; [`negative_set`] picks the first whose row has a
/// negative entry on a given sensor.
pub const ONE_HOT: [[Rational; 3]; 3] = [
    [r(1, 1), r(0, 1), r(0, 1)],
    [r(0, 1), r(1, 1), r(0, 1)],
    [r(0, 1), r(0, 1), r(1, 1)],
];

/// A sentinel no develop writes, so an untouched buffer is recognisable.
pub const SENTINEL: u16 = 0xA5A5;

/// The mono develop's own strip and ring constants, as the design names them
/// (`DEC-084` item 6): `T = w·(min(h, S + 7) + RING)`.
pub const STRIP_ROWS: u64 = 512;
pub const RING_ROWS: u64 = 8;

/// `T`, the strip working memory, for a `w × h` plane (`SPEC-093` Design §7).
pub fn strip_t(w: u32, h: u32) -> u64 {
    u64::from(w) * (u64::from(h).min(STRIP_ROWS + 7) + RING_ROWS)
}

/// `A + max(A, T)`, the mix's scratch with a non-identity warp.
pub fn warp_scratch(w: u32, h: u32, active: u64) -> u64 {
    active + active.max(strip_t(w, h))
}

/// The first one-hot set whose row has a negative entry on `sensor`.
pub fn negative_set(sensor: &Sensor) -> [Rational; 3] {
    *ONE_HOT
        .iter()
        .find(|w| {
            mono_mix_row(sensor, w)
                .expect("the row builds")
                .iter()
                .any(|&q| q < 0)
        })
        .expect("a one-hot set has a negative row entry")
}

/// xorshift32: integer-only, so the samples are the same everywhere.
pub fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

/// Noise over the whole raw range with a near-full-scale column band (columns
/// `≥ width − 5`), so the developed mix reaches white and the fade band.
pub fn samples(width: u32, height: u32, seed: u32) -> Vec<u16> {
    let mut state = seed;
    (0..width * height)
        .map(|i| {
            let n = xorshift32(&mut state);
            if i % width >= width - 5 {
                (65535 - (n & 0x3ff)) as u16
            } else {
                (n & 0xffff) as u16
            }
        })
        .collect()
}

/// `num / 10000` SRATIONALs, row-major.
pub fn matrix(rows: [[i32; 3]; 3]) -> Vec<(i32, i32)> {
    rows.iter()
        .flat_map(|row| row.iter().map(|&num| (num, 10_000)))
        .collect()
}

/// The levels of fixtures (i) to (v): a 2×2 `BlackLevel` pattern whose four
/// values differ, and a white below 65535, so normalisation matters.
pub const BLACKS: [u32; 4] = [1024, 1100, 1180, 1260];
pub const WHITE: u32 = 60000;

/// A colour RGGB description: `ActiveArea` `[top, left, bottom, right]`,
/// `DefaultCrop` origin `[x, y]` and size `[w, h]`, Orientation 1, the 2×2
/// black pattern, `ColorMatrix1`, `ForwardMatrix1` and `AsShotNeutral`.
pub fn colour_description(
    width: u32,
    height: u32,
    active: [u32; 4],
    crop_origin: [u32; 2],
    crop_size: [u32; 2],
) -> DngDescription {
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::SensorInIfd0,
        width,
        height,
        bits_per_sample: 16,
        kind: PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        },
        organisation: Organisation::Strips {
            rows_per_strip: height,
        },
        payload: Payload::Samples(samples(width, height, 0x5093_0001 ^ width ^ (height << 8))),
        meta: DngMeta {
            black_level: Some(BLACKS.to_vec()),
            black_level_repeat_dim: Some([2, 2]),
            white_level: Some(vec![WHITE]),
            active_area: Some(active),
            default_crop_origin: Some(crop_origin),
            default_crop_size: Some(crop_size),
            orientation: Some(1),
            color_matrix_1: Some(matrix([
                [8000, -2000, -500],
                [-4000, 12000, 2000],
                [-1000, 2500, 7000],
            ])),
            // A forward matrix, so the white balance is not renormalised
            // away (AC8's F11 needs the tiny neutral to reach the row).
            forward_matrix_1: Some(matrix([
                [6000, 3000, 642],
                [2000, 7000, 1000],
                [1000, 1000, 6249],
            ])),
            calibration_illuminant_1: Some(21),
            as_shot_neutral: Some(vec![(1, 2), (1, 1), (2, 3)]),
            ..DngMeta::default()
        },
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

/// A fixture constructor: the parsed sensor and its unpacked plane.
pub type FixtureFn = fn() -> (Sensor, Vec<u16>);

/// Parse and unpack a description.
pub fn read(d: &DngDescription) -> (Sensor, Vec<u16>) {
    let file = write_dng(d).expect("the fixture writes");
    let container = Container::parse(&file).expect("the fixture parses");
    let sensor = container.sensor().expect("the fixture has a sensor");
    let mut plane = vec![0u16; (sensor.width * sensor.height) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpacks");
    (sensor, plane)
}

/// Fixture (i), **odd**: 22 × 18, ActiveArea `[3, 1, 17, 21]`, DefaultCrop
/// origin `(1, 3)` size `17 × 11`.
pub fn odd_description() -> DngDescription {
    colour_description(22, 18, [3, 1, 17, 21], [1, 3], [17, 11])
}

/// Fixture (iii), **tall**: 20 × 1100, ActiveArea `[3, 1, 1099, 19]`
/// (18 × 1096), DefaultCrop origin `(1, 5)` size `16 × 1090`.
pub fn tall_description() -> DngDescription {
    colour_description(20, 1100, [3, 1, 1099, 19], [1, 5], [16, 1090])
}

/// The optional (flags 1) non-identity radial warp of fixtures (ii) and (iv):
/// scale `1 − 0.08·r²`, in `[0.92, 1]`, so its lerps at `α ∈ {½, 1}` pass
/// `DEC-059` §3 under `FullActiveArea`.
pub const RADIAL: [f64; 4] = [1.0, -0.08, 0.0, 0.0];

pub fn warp_list(kr: [f64; 4], kt: [f64; 2], flags: u32) -> Vec<u8> {
    single_warp_rectilinear_list(kr, kt, 0.5, 0.5, 1, flags)
}

pub fn radial_warp(flags: u32) -> Vec<u8> {
    warp_list(RADIAL, [0.0, 0.0], flags)
}

/// Put `list` in `OpcodeList{n}` of a parsed sensor (in memory).
pub fn set_list(sensor: &mut Sensor, n: usize, list: Option<Vec<u8>>) {
    sensor.opcode_lists[n - 1] = list.is_some();
    match n {
        1 => sensor.opcode_list_1 = list,
        2 => sensor.opcode_list_2 = list,
        _ => sensor.opcode_list_3 = list,
    }
}

/// Fixture (i).
pub fn odd() -> (Sensor, Vec<u16>) {
    read(&odd_description())
}

/// Fixture (ii): (i) plus the optional radial warp.
pub fn odd_warped() -> (Sensor, Vec<u16>) {
    let (mut sensor, plane) = odd();
    set_list(&mut sensor, 3, Some(radial_warp(1)));
    (sensor, plane)
}

/// Fixture (iii).
pub fn tall() -> (Sensor, Vec<u16>) {
    read(&tall_description())
}

/// Fixture (iv): (iii) with the warp of (ii).
pub fn tall_warped() -> (Sensor, Vec<u16>) {
    let (mut sensor, plane) = tall();
    set_list(&mut sensor, 3, Some(radial_warp(1)));
    (sensor, plane)
}

/// Four `GainMap`s, one per Bayer phase (pitch 2), each a 2 × 2 grid over the
/// whole `active_w × active_h` stage-2 image, with gains other than one in
/// every phase. Mandatory (flags 0), as the Pixel files carry them.
pub fn phase_gain_maps(active_w: u32, active_h: u32) -> Vec<u8> {
    let mut ops = Vec::new();
    for (k, (top, left)) in [(0u32, 0u32), (0, 1), (1, 0), (1, 1)]
        .into_iter()
        .enumerate()
    {
        let mut p = Vec::new();
        for v in [top, left, active_h, active_w, 0, 1, 2, 2, 2, 2] {
            p.extend_from_slice(&v.to_be_bytes());
        }
        for v in [1.0f64, 1.0, 0.0, 0.0] {
            p.extend_from_slice(&v.to_be_bytes());
        }
        p.extend_from_slice(&1u32.to_be_bytes());
        let base = 1.1 + 0.1 * k as f32;
        for g in [base, 0.85, base + 0.2, 0.95] {
            p.extend_from_slice(&g.to_be_bytes());
        }
        ops.push(raw_opcode(9, 0x0103_0000, 0, &p));
    }
    opcode_list(&ops)
}

/// Fixture (v): (i) with [`phase_gain_maps`] in `OpcodeList2`.
pub fn odd_gained() -> (Sensor, Vec<u16>) {
    let (mut sensor, plane) = odd();
    set_list(&mut sensor, 2, Some(phase_gain_maps(20, 14)));
    (sensor, plane)
}

/// The §2 oracle's sensor: `sensor` read as an already-normalised mono plane,
/// black [`MONO_MIX_BLACK`], white [`MONO_MIX_WHITE`], no black pattern, no
/// `OpcodeList1` and no `OpcodeList2` (its `OpcodeList3`, geometry and
/// orientation kept).
pub fn mono_of(sensor: &Sensor) -> Sensor {
    let mut m = sensor.clone();
    m.black_level = Some(u32::from(MONO_MIX_BLACK));
    m.black_levels = vec![];
    m.black_level_repeat_dim = None;
    m.white_level = Some(u32::from(MONO_MIX_WHITE));
    m.white_levels = vec![];
    m.bits_per_sample = 16;
    set_list(&mut m, 1, None);
    set_list(&mut m, 2, None);
    m
}

pub fn with_mix(options: &DevelopOptions, weights: [Rational; 3]) -> DevelopOptions {
    let mut o = *options;
    o.mono_mix = Some(weights);
    o
}

pub fn without_mix(options: &DevelopOptions) -> DevelopOptions {
    let mut o = *options;
    o.mono_mix = None;
    o
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-100 (DEC-088 item 3): the per-plane warp on the developed mix
// ─────────────────────────────────────────────────────────────────────────────

/// SPEC-099's sets (`tests/plane_warp.rs`): green not the identity, red and
/// blue with CA in opposite directions. Radial only.
pub const CA_GREEN: [f64; 4] = [1.0, -0.05, -0.05, 0.0];
pub const CA_RED: [f64; 4] = [1.08, -0.04, -0.05, 0.01];
pub const CA_BLUE: [f64; 4] = [0.94, -0.06, -0.04, 0.0];

/// A `WarpRectilinear` list with one entry per `(kr, kt)` set (`N` the count).
pub fn sets_list(sets: &[([f64; 4], [f64; 2])], cx: f64, cy: f64, flags: u32) -> Vec<u8> {
    let mut p = (sets.len() as u32).to_be_bytes().to_vec();
    for (kr, kt) in sets {
        for v in kr.iter().chain(kt) {
            p.extend_from_slice(&v.to_be_bytes());
        }
    }
    p.extend_from_slice(&cx.to_be_bytes());
    p.extend_from_slice(&cy.to_be_bytes());
    opcode_list(&[raw_opcode(1, 0x0104_0000, flags, &p)])
}

/// The `N = 3` differing list (red, green, blue) at `(cx, cy)`.
pub fn ca_list(cx: f64, cy: f64, flags: u32) -> Vec<u8> {
    let z = [0.0, 0.0];
    sets_list(&[(CA_RED, z), (CA_GREEN, z), (CA_BLUE, z)], cx, cy, flags)
}

/// `W` as a mono `OpcodeList3`: one `N = 1` set, green's.
pub fn w_list(cx: f64, cy: f64) -> Vec<u8> {
    single_warp_rectilinear_list(CA_GREEN, [0.0, 0.0], cx, cy, 1, 0)
}

/// AC4's three scene variants, as `(name, sensor, plane, cx, cy)` without
/// `OpcodeList3`: no `ActiveArea` with a crop, fixture (i) (an `ActiveArea`
/// and a crop inside it), and (i) with the optical centre at (0.3, 0.6).
pub fn ca_scenes() -> Vec<(&'static str, Sensor, Vec<u16>, f64, f64)> {
    let mut plain = colour_description(22, 18, [0, 0, 18, 22], [1, 3], [17, 11]);
    plain.meta.active_area = None;
    let (plain, plain_src) = read(&plain);
    let (odd, odd_src) = odd();
    vec![
        ("plain", plain, plain_src, 0.5, 0.5),
        ("active", odd.clone(), odd_src.clone(), 0.5, 0.5),
        ("off-centre", odd, odd_src, 0.3, 0.6),
    ]
}

/// The §2 oracle with the mono sensor's `OpcodeList3` replaced by `list3`
/// (`SPEC-100` AC4: `W` as one `N = 1` set over the CA-corrected plane).
pub fn oracle_with_list3(
    sensor: &Sensor,
    mixed: &[u16],
    options: &DevelopOptions,
    list3: Option<Vec<u8>>,
) -> Result<Vec<u16>, Error> {
    let mut mono = mono_of(sensor);
    set_list(&mut mono, 3, list3);
    let o = without_mix(options);
    let (w, h) = output_dimensions_with(&mono, &o)?;
    let mut dst = vec![SENTINEL; w as usize * h as usize];
    develop_with_into(&mono, mixed, &mut dst, &o)?;
    Ok(dst)
}

/// The whole mixed plane.
pub fn mixed_plane(sensor: &Sensor, src: &[u16], weights: &[Rational; 3]) -> Vec<u16> {
    let mut mixed = vec![SENTINEL; src.len()];
    mono_mix_plane_into(sensor, src, weights, &mut mixed).expect("the mixed plane");
    mixed
}

/// The §2 oracle on an already-mixed plane: the mono develop of `mixed` on
/// `mono_of(sensor)`, under `options` without the mix.
pub fn oracle_of_mixed(
    sensor: &Sensor,
    mixed: &[u16],
    options: &DevelopOptions,
) -> Result<Vec<u16>, Error> {
    let mono = mono_of(sensor);
    let o = without_mix(options);
    let (w, h) = output_dimensions_with(&mono, &o)?;
    let mut dst = vec![SENTINEL; w as usize * h as usize];
    develop_with_into(&mono, mixed, &mut dst, &o)?;
    Ok(dst)
}

/// The §2 oracle.
pub fn oracle(
    sensor: &Sensor,
    src: &[u16],
    weights: &[Rational; 3],
    options: &DevelopOptions,
) -> Result<Vec<u16>, Error> {
    oracle_of_mixed(sensor, &mixed_plane(sensor, src, weights), options)
}

/// The developed mix through `develop_with_into`, and the same through
/// `develop_with_scratch_into` with exactly `scratch_len` samples (asserted
/// equal).
pub fn developed(
    sensor: &Sensor,
    src: &[u16],
    weights: [Rational; 3],
    options: &DevelopOptions,
) -> Result<Vec<u16>, Error> {
    let o = with_mix(options, weights);
    let (w, h) = output_dimensions_with(sensor, &o)?;
    let mut dst = vec![SENTINEL; w as usize * h as usize];
    let with = develop_with_into(sensor, src, &mut dst, &o);
    let len = scratch_len(sensor, &o)?;
    let mut scratch = vec![0x5A5Au16; len];
    let mut dst2 = vec![SENTINEL; w as usize * h as usize];
    let scr = develop_with_scratch_into(sensor, src, &mut dst2, &mut scratch, &o);
    crate::assert_same!(with, scr, "the two develop entries disagree on the result");
    assert_eq!(dst, dst2, "the two develop entries disagree on the pixels");
    with.map(|()| dst)
}

pub fn options(framing: Framing, apply_orientation: bool, warp_alpha: f64) -> DevelopOptions {
    let mut o = DevelopOptions::default();
    o.framing = framing;
    o.apply_orientation = apply_orientation;
    o.warp_alpha = warp_alpha;
    o
}

// ─────────────────────────────────────────────────────────────────────────────
// AC8: the ranked faults
// ─────────────────────────────────────────────────────────────────────────────

/// F0 to F14 of AC8, by rank (`SPEC-100` inserted F10, the per-plane warp,
/// after the CFA and before the calibration; the old F10 to F13 are F11 to
/// F14).
pub const FAULTS: usize = 15;

/// One input to the four entries: the sensor, the plane and the options, plus
/// how far `dst` and `scratch` are off their exact lengths.
pub struct Case {
    pub sensor: Sensor,
    pub src: Vec<u16>,
    pub options: DevelopOptions,
    pub dst_delta: isize,
    pub scratch_delta: isize,
}

/// Whether faults `i` and `j` (`i < j`) can coexist on one input, and if not,
/// why: F7 and F8 need opposite framings and both live in `OpcodeList3`; F8
/// and F10 or F14 all live in `OpcodeList3` (a malformed block has no sets,
/// so no tangential term).
pub fn skipped_pair(i: usize, j: usize) -> Option<&'static str> {
    match (i, j) {
        (7, 8) => Some("opposite framings, and both are OpcodeList3"),
        (8, 10) => Some("one OpcodeList3: a malformed block has no per-plane sets"),
        (8, 14) => Some("one OpcodeList3: a malformed block has no tangential term"),
        _ => None,
    }
}

/// A mandatory `FixBadPixelsConstant` (`OpcodeList1`), which the mix applies
/// (`SPEC-098`, `DEC-086` item 1).
pub fn mandatory_fix_bad_pixels() -> Vec<u8> {
    crate::opcode_support::single_fix_bad_pixels_constant_list(0, 0, 0)
}

/// Two mandatory `FixBadPixelsConstant`s (`OpcodeList1`): the mix applies the
/// first and refuses the second, so this is fault F2's list (`SPEC-098`).
pub fn two_mandatory_fix_bad_pixels() -> Vec<u8> {
    let fbpc = || raw_opcode(4, 0x0103_0000, 0, &[0u8; 8]);
    opcode_list(&[fbpc(), fbpc()])
}

/// An optional `FixBadPixelsConstant` whose parameter block is 7 bytes.
pub fn malformed_optional_fix_bad_pixels() -> Vec<u8> {
    opcode_list(&[raw_opcode(4, 0x0103_0000, 1, &[0u8; 7])])
}

/// An optional `WarpRectilinear` whose parameter block is malformed (10 bytes).
pub fn malformed_optional_warp() -> Vec<u8> {
    opcode_list(&[raw_opcode(1, 0x0104_0000, 1, &[0u8; 10])])
}

/// The valid fixture (i) with the faults in `set` (indices into F0..F13).
pub fn case(set: &[usize]) -> Case {
    let (mut sensor, mut src) = odd();
    let has = |f: usize| set.contains(&f);
    let mut o = with_mix(&DevelopOptions::default(), LUMINANCE);
    if has(0) {
        o.warp_alpha = f64::NAN;
    }
    if has(1) {
        o.mono_mix = Some([r(1, 3), r(1, 3), r(1, 4)]);
    }
    if has(2) {
        set_list(&mut sensor, 1, Some(two_mandatory_fix_bad_pixels()));
    }
    if has(3) {
        src.pop();
    }
    if has(4) {
        sensor.orientation = Some(9);
    }
    if has(5) {
        // Every black above the white.
        sensor.white_level = Some(100);
        sensor.white_levels = vec![100];
    }
    // F7, F8, F10 and F14 share OpcodeList3. F7 is a green (W) that fails
    // the bounds rule under FullActiveArea; F10 (`SPEC-100`) is a mandatory
    // N = 3 list whose sets differ, with a tangential term on blue; F14 is a
    // tangential term on W itself, which the applier refuses.
    if has(8) {
        set_list(&mut sensor, 3, Some(malformed_optional_warp()));
    } else if has(7) || has(10) || has(14) {
        if has(7) {
            o.framing = Framing::FullActiveArea;
        }
        let green = match (has(7), has(10)) {
            (true, _) => [1.0, 0.08, 0.0, 0.0],
            (false, true) => CA_GREEN,
            (false, false) => RADIAL,
        };
        let green_kt = if has(14) { [0.01, 0.0] } else { [0.0, 0.0] };
        let list = if has(10) {
            sets_list(
                &[
                    (CA_RED, [0.0, 0.0]),
                    (green, green_kt),
                    (CA_BLUE, [0.002, 0.0]),
                ],
                0.5,
                0.5,
                0,
            )
        } else {
            warp_list(green, green_kt, 1)
        };
        set_list(&mut sensor, 3, Some(list));
    }
    if has(9) {
        sensor.color_filter = None;
    }
    if has(11) {
        // No colour matrix at all: with a ForwardMatrix1 present, removing
        // ColorMatrix1 alone still calibrates.
        sensor.color_matrix_1 = None;
        sensor.forward_matrix_1 = None;
    }
    if has(12) {
        sensor.as_shot_neutral = Some(vec![r(1, 100_000), r(1, 1), r(1, 1)]);
    }
    Case {
        sensor,
        src,
        options: o,
        dst_delta: if has(6) { 1 } else { 0 },
        scratch_delta: if has(13) { -1 } else { 0 },
    }
}

/// The error fault `f` alone gives on the two develop entries.
pub fn expected_error(f: usize) -> fn(&Error) -> bool {
    match f {
        0 => |e| matches!(e, Error::InvalidWarpAlpha { .. }),
        1 => |e| matches!(e, Error::InvalidMixWeights { channel: None }),
        2 => |e| matches!(e, Error::UnsupportedRequiredOpcode { list: 1, id: 4 }),
        3 => |e| matches!(e, Error::SourcePlaneWrongLength { .. }),
        4 => |e| matches!(e, Error::UnsupportedOrientation { orientation: 9 }),
        5 => |e| matches!(e, Error::InvalidLevels { .. }),
        6 => |e| matches!(e, Error::DevelopBufferWrongLength { .. }),
        7 => |e| matches!(e, Error::UnsupportedWarpedFullActiveArea { .. }),
        8 => |e| matches!(e, Error::MalformedOpcodeParams { id: 1, .. }),
        9 => |e| matches!(e, Error::NotAColorFilterArray),
        // F10 and F14 are the same error; F10 (the per-plane check) refuses
        // before any write, F14 (the applier) after the strips.
        10 => |e| matches!(e, Error::UnsupportedWarpTangentialTerms { .. }),
        11 => |e| matches!(e, Error::MissingColorCalibration),
        12 => |e| matches!(e, Error::MixRowOutOfRange { channel: 0 }),
        13 => |e| matches!(e, Error::DevelopScratchWrongLength { .. }),
        _ => |e| matches!(e, Error::UnsupportedWarpTangentialTerms { .. }),
    }
}

/// The exact `dst` and `scratch` lengths for `case`, falling back to the valid
/// fixture's where the case's own cannot be computed.
pub fn buffer_lengths(c: &Case) -> (usize, usize) {
    let base = case(&[]);
    let dims = output_dimensions_with(&c.sensor, &c.options)
        .or_else(|_| output_dimensions_with(&base.sensor, &base.options))
        .expect("the valid fixture sizes");
    let dst = (dims.0 as usize * dims.1 as usize).saturating_add_signed(c.dst_delta);
    let scratch = scratch_len(&c.sensor, &c.options)
        .or_else(|_| scratch_len(&base.sensor, &base.options))
        .expect("the valid fixture sizes its scratch")
        .saturating_add_signed(c.scratch_delta);
    (dst, scratch)
}

/// Run `c` through `develop_with_into` (`scratch: false`) or
/// `develop_with_scratch_into`, from sentinel-filled buffers. Returns the
/// result, whether `dst` is untouched, and whether `scratch` is untouched
/// (always true for `develop_with_into`).
pub fn run_case(c: &Case, with_scratch: bool) -> (Result<(), Error>, bool, bool) {
    let (dst_len, scratch_len) = buffer_lengths(c);
    let mut dst = vec![SENTINEL; dst_len];
    let mut scratch = vec![SENTINEL; scratch_len];
    let result = if with_scratch {
        develop_with_scratch_into(&c.sensor, &c.src, &mut dst, &mut scratch, &c.options)
    } else {
        develop_with_into(&c.sensor, &c.src, &mut dst, &c.options)
    };
    (
        result,
        dst.iter().all(|&v| v == SENTINEL),
        scratch.iter().all(|&v| v == SENTINEL),
    )
}
