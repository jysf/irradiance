//! `SPEC-097` AC7 (`DEC-085`): the colour Leica M10 and M11 reach every
//! PROJ-006 entry once a lossless-JPEG precision below `BitsPerSample`
//! decodes.
//!
//! Both files store a SOF-3 precision of 14 in a `BitsPerSample` of 16, which
//! `DEC-077` §3 refused at unpack and `DEC-085` accepts. For each present file
//! this asserts `Ok` from `unpack_into`, `develop_color_into`,
//! `srgb_linear_plane_into`, `mono_mix_plane_into` (Rec. 709 luma, 2126/7152/722
//! per 10000) and `develop_with_into` with `mono_mix` set to the same weights.
//!
//! The FNV-1a 64 digests are **regression pins, not an oracle**: they are the
//! existing pipeline's output on these files, measured by SPEC-097's design
//! probe (2026-11-27) and copied from the spec's Implementation Context. The
//! only code change is the precision check, so a mismatch means something else
//! moved: report it, never edit the pin to match. The plane's oracle is
//! dnglab's checksum, in `tests/plane_oracle.rs`.
//!
//! **Tier B**: the files are read in place under `$IRRADIANCE_CORPUS_DIR`
//! (`DEC-003`) and each is skipped by name when absent (`CorpusFile::require`).
//! ⚠ The M11 (`LEICA-M11-PB`) has no stated licence: it is never committed,
//! copied or redistributed, and no CI test may need it.
//!
//! `SPEC-098` AC7 and AC8 (`DEC-086`) add the colour Leica Q2, Q3 and SL3, each
//! with a mandatory `OpcodeList1` `FixBadPixelsConstant` and a mandatory
//! `OpcodeList3` `WarpRectilinear` with `N = 3`: the sensor-geometry entries
//! are `Ok` (the fix applied, the warp passed through to the consumer), and the
//! developed outputs still refuse the warp until `SPEC-099`. Their digests are
//! likewise regression pins from SPEC-098's design probe, not an oracle; the
//! kernel's oracle is `tests/colour_bad_pixels.rs` (the Adobe SDK's stage 2 on
//! synthetic mosaics), and the SL3's photosite counts below.
//!
//! `SPEC-099` AC7 (`DEC-087`) moves the three files' sensor-geometry planes:
//! red and blue are resampled to green's geometry and only green's set is left
//! to the consumer (`opcode::sensor_geometry_warp`). The `u16` plane's digest
//! was pre-registered by SPEC-099's design probe, whose no-CA pipeline first
//! reproduced SPEC-098's pins; the `f32` and mono digests were re-pinned at the
//! build after the `u16` pin matched. On the Q2 and Q3 the plane is also
//! checked at every sample against the tests' own reference of the rule
//! (`tests/support/ca_reference.rs`) through the public stage primitives; the
//! registration oracle is `tests/colour_sdk_stage3.rs` (`dng_validate -3`).
//!
//! `SPEC-100` AC7 (`DEC-088`): the developed outputs apply the warp. The colour
//! develop is the CA step, then green's set on each camera plane, then the
//! matrix, the crop and the orientation; the developed mix is the mono develop
//! of the CA-corrected mixed plane with green's set. Their digests were
//! pre-registered by SPEC-100's design probe (public primitives and the tests'
//! own CA reference); on the Q2 and Q3 the colour develop is also checked at
//! every sample against that composition.
//! The developed outputs still refuse the warp until `SPEC-100`.
//! ⚠ The Q3 (`LEICA-Q3-PB`) and SL3 (`LEICA-SL3-PB`) have no stated licence:
//! never committed, copied or redistributed, and no CI test may need them.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

#[path = "support/ca_reference.rs"]
mod ca_reference;
#[path = "support/corpus.rs"]
mod corpus;

use corpus::{CorpusRoot, Manifest};
use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, mono_mix_plane_into, mono_mix_plane_rows_into, mono_mix_row,
    srgb_linear_plane_f32_into, srgb_linear_plane_into, srgb_linear_plane_rows_f32_into,
    srgb_linear_plane_rows_into, to_srgb_linear_into,
};
use irradiance::demosaic::demosaic_into;
use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions,
    output_dimensions_with, scratch_len, DevelopOptions, Framing,
};
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::opcode::sensor_geometry_warp;
use irradiance::plane::unpack_into;
use irradiance::warp::apply_warp_into;
use irradiance::Error;

const M10: &str = "LEICA-M10/f5381888.dng";
const M11: &str = "LEICA-M11-PB/leica_m11_01.dng";

/// One file's pins, from SPEC-097's Implementation Context table.
struct Pins {
    path: &'static str,
    /// `develop_color_into`'s output size.
    developed: (u32, u32),
    develop_color: u64,
    srgb_linear: u64,
    mono_mix_plane: u64,
    developed_mix: u64,
}

const PINS: [Pins; 2] = [
    Pins {
        path: M10,
        developed: (5976, 3984),
        develop_color: 0x7031_d21e_a67b_dd45,
        srgb_linear: 0xad88_388f_947b_99db,
        mono_mix_plane: 0x3caa_9be8_99d6_ee98,
        developed_mix: 0xb01b_59c4_4e2b_c0d3,
    },
    Pins {
        path: M11,
        developed: (9528, 6328),
        develop_color: 0xfb38_2cb0_38df_474b,
        srgb_linear: 0xe521_4f13_a0a4_c09c,
        mono_mix_plane: 0x380c_7521_2c77_8806,
        developed_mix: 0xfcf9_8717_0824_5c05,
    },
];

/// Rec. 709 luma, 2126/7152/722 per 10000, as SPEC-093's probe used.
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

/// FNV-1a 64 over `u16` samples as little-endian bytes (basis
/// `0xcbf29ce484222325`, prime `0x100000001b3`).
fn fnv1a(samples: &[u16]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in samples {
        for b in v.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// The file's bytes, or `None` (skip, by name) when it is absent.
fn load(path: &str) -> Option<Vec<u8>> {
    let manifest = Manifest::load().expect("manifest parses");
    let root = CorpusRoot::resolve();
    let entry = manifest
        .get(path)
        .unwrap_or_else(|| panic!("{path} is not a [[file]] manifest entry"));
    let file = entry.require(&root)?;
    Some(std::fs::read(&file).unwrap_or_else(|e| panic!("{path}: read failed: {e}")))
}

/// Parse and unpack the whole mosaic; `unpack_into` must be `Ok`.
fn unpack(path: &str, bytes: &[u8]) -> (Sensor, Vec<u16>) {
    let container = Container::parse(bytes).unwrap_or_else(|e| panic!("{path}: parse: {e}"));
    let sensor = container
        .sensor()
        .unwrap_or_else(|e| panic!("{path}: sensor: {e}"));
    let mut src = vec![0u16; sensor.width as usize * sensor.height as usize];
    unpack_into(&sensor, container.byte_order(), bytes, &mut src)
        .unwrap_or_else(|e| panic!("{path}: unpack_into: {e}"));
    (sensor, src)
}

#[track_caller]
fn assert_pin(path: &str, entry: &str, got: u64, want: u64) {
    assert_eq!(
        got, want,
        "{path}: {entry} digest 0x{got:016x} != pinned 0x{want:016x} \
         (a pin from the spec that set it: report it, do not edit it)"
    );
}

#[test]
fn ac7_leica_files_reach_every_colour_entry() {
    let mut ran = 0;
    for pins in &PINS {
        let path = pins.path;
        let Some(bytes) = load(path) else {
            continue;
        };
        let (sensor, src) = unpack(path, &bytes);
        let plane = sensor.width as usize * sensor.height as usize;

        let (w, h) = output_dimensions(&sensor).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!((w, h), pins.developed, "{path}: developed size");
        let mut rgb = vec![0u16; w as usize * h as usize * 3];
        develop_color_into(&sensor, &src, &mut rgb)
            .unwrap_or_else(|e| panic!("{path}: develop_color_into: {e}"));
        assert_pin(path, "develop_color_into", fnv1a(&rgb), pins.develop_color);
        drop(rgb);

        let mut lin = vec![0u16; plane * 3];
        srgb_linear_plane_into(&sensor, &src, &mut lin)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_into: {e}"));
        assert_pin(
            path,
            "srgb_linear_plane_into",
            fnv1a(&lin),
            pins.srgb_linear,
        );
        drop(lin);

        let mut mono = vec![0u16; plane];
        mono_mix_plane_into(&sensor, &src, &LUMA, &mut mono)
            .unwrap_or_else(|e| panic!("{path}: mono_mix_plane_into: {e}"));
        assert_pin(
            path,
            "mono_mix_plane_into",
            fnv1a(&mono),
            pins.mono_mix_plane,
        );
        drop(mono);

        let mut options = DevelopOptions::default();
        options.mono_mix = Some(LUMA);
        let (mw, mh) =
            output_dimensions_with(&sensor, &options).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!((mw, mh), pins.developed, "{path}: developed-mix size");
        let mut mix = vec![0u16; mw as usize * mh as usize];
        develop_with_into(&sensor, &src, &mut mix, &options)
            .unwrap_or_else(|e| panic!("{path}: develop_with_into mono_mix: {e}"));
        assert_pin(
            path,
            "develop_with_into mono_mix",
            fnv1a(&mix),
            pins.developed_mix,
        );
        ran += 1;
    }
    eprintln!(
        "ac7_leica_files_reach_every_colour_entry: {ran}/{} file(s) present",
        PINS.len()
    );
}

/// DEC-029's clip case on a real file: the M10's samples run past its
/// `WhiteLevel` (15000) up to 16383, the 14-bit ceiling, and none reaches
/// `2^14`.
#[test]
fn ac7_m10_samples_above_white_level_are_pinned() {
    let Some(bytes) = load(M10) else {
        return;
    };
    let (sensor, src) = unpack(M10, &bytes);
    assert_eq!(sensor.bits_per_sample, 16, "{M10}: BitsPerSample");
    assert_eq!(sensor.white_level, Some(15000), "{M10}: WhiteLevel");
    let max = src.iter().copied().max().expect("non-empty plane");
    assert_eq!(max, 16383, "{M10}: decoded max");
    let above = src.iter().filter(|&&v| v > 15000).count();
    assert_eq!(above, 1_243_041, "{M10}: samples above WhiteLevel");
    assert_eq!(
        src.iter().filter(|&&v| v >= 1 << 14).count(),
        0,
        "{M10}: samples at or above 2^14"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-098: the colour Leica Q2, Q3 and SL3 (`DEC-086`)
// ─────────────────────────────────────────────────────────────────────────────

const Q2: &str = "LEICA-Q2/L1000750.DNG";
const Q3: &str = "LEICA-Q3-PB/leica_q3_01.dng";
const SL3: &str = "LEICA-SL3-PB/leica_sl3_01.dng";

/// One opcode file's facts and pins: SPEC-098's Implementation Context, and
/// SPEC-099's (`DEC-087`: red and blue resampled to green's geometry).
struct OpcodePins {
    path: &'static str,
    /// `OpcodeList1`'s `FixBadPixelsConstant` `BayerPhase`.
    bayer_phase: u32,
    /// The three `WarpRectilinear` sets' `kr` (R, G, B), from SPEC-099's
    /// Implementation Context; `kt = 0`, `cx = cy = 0.5` on all three files.
    sets: [[f64; 4]; 3],
    /// `srgb_linear_plane_into`, pre-registered by SPEC-099's design probe
    /// (`DEC-087`): an oracle-backed pin, never edited to match.
    srgb_linear: u64,
    /// `srgb_linear_plane_f32_into` and `mono_mix_plane_into`, re-pinned at
    /// SPEC-099's build after the `u16` pin matched (the unclamped sum and the
    /// mixer pixel are private, so no independent prediction exists).
    srgb_linear_f32: u64,
    mono_mix_plane: u64,
    /// SPEC-098's pins for the three planes (the warp passed through): each
    /// must now differ.
    spec098: [u64; 3],
    /// The plain mono `develop_into` (unchanged by SPEC-098 and SPEC-099).
    mono_develop: u64,
    /// Whether the file has no bad photosite (so the exactness route needs no
    /// fix: the Q2 and Q3).
    no_bad_photosite: bool,
}

const OPCODE_PINS: [OpcodePins; 3] = [
    OpcodePins {
        path: Q2,
        bayer_phase: 2,
        sets: [
            [
                0.9996216308165431,
                -0.04787844936561948,
                -0.09821650922200259,
                0.05547169955651253,
            ],
            [
                0.999251106,
                -0.047206985980232964,
                -0.0985599100713025,
                0.055324309073773385,
            ],
            [
                0.9991769974193434,
                -0.04597953970427109,
                -0.09942464206440757,
                0.055257758618096245,
            ],
        ],
        srgb_linear: 0xd352_bb75_bf29_d270,
        srgb_linear_f32: 0x46f4_4d43_f688_2eb4,
        mono_mix_plane: 0xe6bb_2752_f00f_7913,
        spec098: [
            0x98af_7675_eef4_3b27,
            0xc6a4_5aaf_c786_35d6,
            0x9452_ae0e_e0d3_622f,
        ],
        mono_develop: 0x1c1c_a269_399d_11ad,
        no_bad_photosite: true,
    },
    OpcodePins {
        path: Q3,
        bayer_phase: 0,
        sets: [
            [
                0.9996347155524453,
                -0.05455320828191769,
                -0.08112921079548925,
                0.04461464522856981,
            ],
            [
                0.999265877659388,
                -0.054056477178819784,
                -0.08104570564304178,
                0.0441974160013766,
            ],
            [
                0.999240053568403,
                -0.05345818671371268,
                -0.08040421688304702,
                0.04313300087098262,
            ],
        ],
        srgb_linear: 0xc6f6_7381_a8b9_4593,
        srgb_linear_f32: 0xaf64_a2b4_e74a_ef07,
        mono_mix_plane: 0x9732_9ab8_1317_c5ec,
        spec098: [
            0x5525_7363_2a3f_a8df,
            0x1776_ef7c_92e4_d4c2,
            0xcbf4_3ca3_6ab2_0784,
        ],
        mono_develop: 0xae9a_8252_ccfe_f17b,
        no_bad_photosite: true,
    },
    OpcodePins {
        path: SL3,
        bayer_phase: 0,
        sets: [
            [
                0.9947909946452878,
                0.02081664155561191,
                -0.024833699323286502,
                0.001712725185655068,
            ],
            [
                0.9944783937976746,
                0.021191865020737728,
                -0.024949742830202244,
                0.0016391091050309678,
            ],
            [
                0.9945095289729575,
                0.021647657199074587,
                -0.02497465736219387,
                0.0014086172248721613,
            ],
        ],
        srgb_linear: 0xbb99_08b9_19ce_2b08,
        srgb_linear_f32: 0x1112_c770_5d76_31e7,
        mono_mix_plane: 0x00d4_7ed9_b11d_1be2,
        spec098: [
            0x0aa0_de90_63b2_509a,
            0x0eb0_1bee_c9da_33f4,
            0xe45e_c468_23e9_c4eb,
        ],
        mono_develop: 0x73fd_7636_37dd_dbc5,
        no_bad_photosite: false,
    },
];

/// FNV-1a 64 over `f32` samples, each as its bits in little-endian bytes.
fn fnv1a_f32(samples: &[f32]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in samples {
        for b in v.to_bits().to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// `(id, flags, the first parameter word)` of a one-opcode list's opcode,
/// read from its raw framing (count, then id, version, flags, size, params).
fn only_opcode(path: &str, list: u8, bytes: Option<&[u8]>) -> (u32, u32, u32, u32) {
    let bytes = bytes.unwrap_or_else(|| panic!("{path}: OpcodeList{list} is absent"));
    let word = |at: usize| u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap());
    assert_eq!(word(0), 1, "{path}: OpcodeList{list} holds one opcode");
    (word(4), word(12), word(20), word(24))
}

#[track_caller]
fn assert_refused(path: &str, entry: &str, r: Result<(), Error>, list: u8, id: u32) {
    assert!(
        matches!(r, Err(Error::UnsupportedRequiredOpcode { list: l, id: i }) if l == list && i == id),
        "{path}: {entry}: expected UnsupportedRequiredOpcode {{ list: {list}, id: {id} }}, got {r:?}"
    );
}

/// The `CFAPattern` the DNG specification's `BayerPhase` names (DNG 1.7.1.0,
/// `FixBadPixelsConstant`): 0 red at the top left, 1 green on a green/red row,
/// 2 green on a green/blue row, 3 blue.
fn pattern_of_bayer_phase(phase: u32) -> [u8; 4] {
    match phase {
        0 => [0, 1, 1, 2],
        1 => [1, 0, 2, 1],
        2 => [1, 2, 0, 1],
        3 => [2, 1, 1, 0],
        other => panic!("BayerPhase {other} is out of range"),
    }
}

/// Every sensor-geometry entry's `rows` band against its whole plane.
#[track_caller]
fn assert_bands(
    path: &str,
    sensor: &Sensor,
    src: &[u16],
    lin: &[u16],
    lin_f32: &[f32],
    mono: &[u16],
) {
    let width = sensor.width as usize;
    let h = sensor.height;
    for rows in [100..200u32, 0..5, h - 5..h] {
        let (a, b) = (rows.start as usize * width, rows.end as usize * width);
        let mut band = vec![0u16; (b - a) * 3];
        srgb_linear_plane_rows_into(sensor, src, rows.clone(), &mut band)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_rows_into: {e}"));
        assert!(
            band == lin[a * 3..b * 3],
            "{path}: u16 band {rows:?} equals the plane"
        );
        let mut band = vec![0f32; (b - a) * 3];
        srgb_linear_plane_rows_f32_into(sensor, src, rows.clone(), &mut band)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_rows_f32_into: {e}"));
        assert!(
            band.iter()
                .map(|v| v.to_bits())
                .eq(lin_f32[a * 3..b * 3].iter().map(|v| v.to_bits())),
            "{path}: f32 band {rows:?} equals the plane"
        );
        let mut band = vec![0u16; b - a];
        mono_mix_plane_rows_into(sensor, src, &LUMA, rows.clone(), &mut band)
            .unwrap_or_else(|e| panic!("{path}: mono_mix_plane_rows_into: {e}"));
        assert!(
            band == mono[a..b],
            "{path}: mixed band {rows:?} equals the plane"
        );
    }
}

/// `SPEC-099` AC7 (`DEC-087`; replaces the sensor-geometry half of SPEC-098's
/// AC7): the three opcode files give chromatic-aberration-corrected planes
/// (red and blue resampled to green's geometry), the accessor names green's set
/// as the consumer's warp, and the developed outputs still refuse the warp
/// (SPEC-100). On the Q2 and Q3 the plane is checked at every sample against
/// the tests' own reference of `DEC-087` item 3 through the public stage
/// primitives.
#[test]
fn ac7_the_opcode_files_are_ca_corrected() {
    let mut ran = 0;
    for pins in &OPCODE_PINS {
        let path = pins.path;
        let Some(bytes) = load(path) else {
            continue;
        };
        let (sensor, src) = unpack(path, &bytes);
        drop(bytes);
        let (width, height) = (sensor.width as usize, sensor.height as usize);
        let plane = width * height;

        // The file's opcode facts.
        let (id, flags, constant, phase) = only_opcode(path, 1, sensor.opcode_list_1.as_deref());
        assert_eq!(
            (id, flags, constant),
            (4, 0, 0),
            "{path}: mandatory FBPC, Constant 0"
        );
        assert_eq!(phase, pins.bayer_phase, "{path}: BayerPhase");
        let active = sensor.active_area.expect("an ActiveArea");
        assert_eq!(
            (active.top, active.left),
            (0, 0),
            "{path}: ActiveArea origin"
        );
        let cfa = sensor.color_filter.as_ref().expect("a CFA");
        assert_eq!(
            cfa.pattern,
            pattern_of_bayer_phase(phase),
            "{path}: BayerPhase agrees with CFAPattern (the specification's meaning)"
        );
        let (id, flags, n, _) = only_opcode(path, 3, sensor.opcode_list_3.as_deref());
        assert_eq!(
            (id, flags, n),
            (1, 0, 3),
            "{path}: mandatory WarpRectilinear, N = 3"
        );

        // The accessor: the table's three sets, coloured R, G, B, green's the
        // consumer's.
        let warp = sensor_geometry_warp(&sensor)
            .unwrap_or_else(|e| panic!("{path}: sensor_geometry_warp: {e}"))
            .expect("a warp");
        assert_eq!(warp.sets().len(), 3, "{path}: three sets");
        for (i, set) in warp.sets().iter().enumerate() {
            assert_eq!(set.color, Some(i as u8), "{path}: set {i} colour");
            assert_eq!(set.warp.kr, pins.sets[i], "{path}: set {i} kr");
            assert_eq!(set.warp.kt, [0.0, 0.0], "{path}: set {i} kt");
            assert_eq!((set.warp.cx, set.warp.cy), (0.5, 0.5), "{path}: centre");
        }
        assert_eq!(warp.consumer().kr, pins.sets[1], "{path}: consumer = green");
        assert!(warp.is_per_plane(), "{path}: per plane");

        // The sensor-geometry entries: Ok, CA-corrected, pinned.
        let mut lin = vec![0u16; plane * 3];
        srgb_linear_plane_into(&sensor, &src, &mut lin)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_into: {e}"));
        let digest = fnv1a(&lin);
        assert_pin(path, "srgb_linear_plane_into", digest, pins.srgb_linear);
        assert_ne!(
            digest, pins.spec098[0],
            "{path}: u16 plane moved from SPEC-098"
        );

        let mut lin_f32 = vec![0f32; plane * 3];
        srgb_linear_plane_f32_into(&sensor, &src, &mut lin_f32)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_f32_into: {e}"));
        let digest = fnv1a_f32(&lin_f32);
        assert_ne!(
            digest, pins.spec098[1],
            "{path}: f32 plane moved from SPEC-098"
        );
        assert_pin(
            path,
            "srgb_linear_plane_f32_into",
            digest,
            pins.srgb_linear_f32,
        );

        let mut mono = vec![0u16; plane];
        mono_mix_plane_into(&sensor, &src, &LUMA, &mut mono)
            .unwrap_or_else(|e| panic!("{path}: mono_mix_plane_into: {e}"));
        let digest = fnv1a(&mono);
        assert_ne!(
            digest, pins.spec098[2],
            "{path}: mixed plane moved from SPEC-098"
        );
        assert_pin(path, "mono_mix_plane_into", digest, pins.mono_mix_plane);
        mono_mix_row(&sensor, &LUMA).unwrap_or_else(|e| panic!("{path}: mono_mix_row: {e}"));

        assert_bands(path, &sensor, &src, &lin, &lin_f32, &mono);
        drop(lin_f32);
        drop(mono);

        // The exactness route (Q2, Q3: no bad photosite): the plane equals
        // to_srgb_linear_into(reference(demosaic_into(n))) at every sample,
        // the primitives on the Sensor with lists 1 and 3 removed.
        if pins.no_bad_photosite {
            let mut stripped = sensor.clone();
            stripped.opcode_list_1 = None;
            stripped.opcode_list_3 = None;
            stripped.opcode_lists[0] = false;
            stripped.opcode_lists[2] = false;
            assert_eq!(sensor.black_level, Some(512), "{path}: black 512");
            assert_eq!(sensor.white_level, Some(16383), "{path}: white 16383");
            assert!(
                sensor.black_levels.iter().all(|&b| b == 512),
                "{path}: every black 512"
            );
            assert!(src.iter().all(|&v| v != 0), "{path}: no bad photosite");
            let n: Vec<u16> = src
                .iter()
                .map(|&raw| {
                    let v = (f64::from(raw) - 512.0).max(0.0) * 65535.0 / (16383.0 - 512.0);
                    (v + 0.5).floor().min(65535.0) as u16
                })
                .collect();
            let mut cam = vec![0u16; plane * 3];
            demosaic_into(&stripped, &n, &mut cam)
                .unwrap_or_else(|e| panic!("{path}: demosaic_into: {e}"));
            drop(n);
            let mut resampled = cam.clone();
            let rule = ca_reference::Rule {
                green: pins.sets[1],
                red: Some(pins.sets[0]),
                blue: Some(pins.sets[2]),
                cx: 0.5,
                cy: 0.5,
            };
            let area = (active.top, active.left, active.bottom, active.right);
            ca_reference::resample_in_place(&cam, &mut resampled, width, area, &rule);
            drop(cam);
            let mut want = vec![0u16; plane * 3];
            to_srgb_linear_into(&stripped, &resampled, &mut want)
                .unwrap_or_else(|e| panic!("{path}: to_srgb_linear_into: {e}"));
            drop(resampled);
            let differ = lin.iter().zip(&want).filter(|(a, b)| a != b).count();
            assert_eq!(differ, 0, "{path}: samples that differ from the reference");
        }
        drop(lin);

        // The plain mono develop is unchanged (AC9).
        let (w, h) = output_dimensions(&sensor).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut developed = vec![0u16; w as usize * h as usize];
        develop_into(&sensor, &src, &mut developed)
            .unwrap_or_else(|e| panic!("{path}: develop_into: {e}"));
        assert_pin(path, "develop_into", fnv1a(&developed), pins.mono_develop);
        drop(developed);

        // The stage primitives still refuse the fix (the developed outputs
        // apply the warp since SPEC-100: `ac7_the_opcode_files_develop`). Each
        // refusal is the entry's first check, so no buffer is needed.
        assert_refused(
            path,
            "demosaic_into",
            demosaic_into(&sensor, &src, &mut []),
            1,
            4,
        );
        assert_refused(
            path,
            "to_srgb_linear_into",
            to_srgb_linear_into(&sensor, &[], &mut []),
            1,
            4,
        );
        ran += 1;
    }
    eprintln!(
        "ac7_the_opcode_files_are_ca_corrected: {ran}/{} file(s) present",
        OPCODE_PINS.len()
    );
}

/// `SPEC-100`'s pre-registered digests for one opcode file (Implementation
/// Context): the colour develop (`u16`, `f32`) and the developed mix (Rec. 709
/// luma) under default options, `warp_alpha` 0.5 and 0, `FullActiveArea` and
/// `apply_orientation: false`.
struct DevelopedPins {
    path: &'static str,
    colour: u64,
    colour_f32: u64,
    mix: [u64; 5],
}

const DEVELOPED_PINS: [DevelopedPins; 3] = [
    DevelopedPins {
        path: Q2,
        colour: 0x15fd_e0a3_aad3_b45e,
        colour_f32: 0xbd4b_ee08_8e77_3f2c,
        mix: [
            0x0ade_8942_5f87_6db3,
            0x617d_4140_fb3c_402b,
            0x6ee6_dcab_f9bc_96b6,
            0xa3cf_e647_9e1a_d080,
            0x0ade_8942_5f87_6db3,
        ],
    },
    DevelopedPins {
        path: Q3,
        colour: 0x3a82_2d18_12f1_9464,
        colour_f32: 0x6af7_d2cd_38f9_5e42,
        mix: [
            0xe928_25ab_103f_946b,
            0x6ca0_d150_26c5_bcd4,
            0xcaa9_94ae_f757_9659,
            0xc689_0e7b_9cd4_b4e4,
            0xe2b1_12f9_36cc_588f,
        ],
    },
    DevelopedPins {
        path: SL3,
        colour: 0x1d8d_6442_e403_cd46,
        colour_f32: 0xd5f7_9bb4_f80c_b5d0,
        mix: [
            0xbf5f_e5e4_fe7a_7c6c,
            0x0bca_0992_e2b9_f84b,
            0xc900_805e_87b9_b166,
            0xd384_f0f8_66b2_b422,
            0x4cca_e484_4a48_c7ac,
        ],
    },
];

/// Output pixel `(ox, oy)`'s position in the crop, from the TIFF
/// `Orientation` table: the test's own.
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
        other => panic!("orientation {other}"),
    }
}

/// AC3's composition on a held file (Q2, Q3: no bad photosite), compared
/// with `got` at every sample: `to_srgb_linear_into(warp_planes(reference(
/// demosaic_into(n))))` on the `Sensor` with lists 1 and 3 removed, then the
/// crop and orientation; `n` the levels by hand (black 512, white 16383).
fn assert_composition(path: &str, sensor: &Sensor, src: &[u16], pins: &OpcodePins, got: &[u16]) {
    let (width, height) = (sensor.width as usize, sensor.height as usize);
    let plane = width * height;
    let mut stripped = sensor.clone();
    stripped.opcode_list_1 = None;
    stripped.opcode_list_3 = None;
    stripped.opcode_lists[0] = false;
    stripped.opcode_lists[2] = false;
    assert_eq!(sensor.black_level, Some(512), "{path}: black 512");
    assert_eq!(sensor.white_level, Some(16383), "{path}: white 16383");
    assert!(src.iter().all(|&v| v != 0), "{path}: no bad photosite");
    let n: Vec<u16> = src
        .iter()
        .map(|&raw| {
            let v = (f64::from(raw) - 512.0).max(0.0) * 65535.0 / (16383.0 - 512.0);
            (v + 0.5).floor().min(65535.0) as u16
        })
        .collect();
    let mut cam = vec![0u16; plane * 3];
    demosaic_into(&stripped, &n, &mut cam).unwrap_or_else(|e| panic!("{path}: demosaic: {e}"));
    drop(n);
    let mut ca = cam.clone();
    let active = sensor.active_area.expect("an ActiveArea");
    let area = (active.top, active.left, active.bottom, active.right);
    let rule = ca_reference::Rule {
        green: pins.sets[1],
        red: Some(pins.sets[0]),
        blue: Some(pins.sets[2]),
        cx: 0.5,
        cy: 0.5,
    };
    ca_reference::resample_in_place(&cam, &mut ca, width, area, &rule);
    drop(cam);
    // W per camera plane over the ActiveArea (green's set, public accessor).
    let warp = *sensor_geometry_warp(sensor).unwrap().unwrap().consumer();
    let (top, left) = (active.top as usize, active.left as usize);
    let (aw, ah) = (
        (active.right - active.left) as usize,
        (active.bottom - active.top) as usize,
    );
    let mut channel = vec![0u16; aw * ah];
    let mut warped = vec![0u16; aw * ah];
    for c in 0..3 {
        for y in 0..ah {
            for x in 0..aw {
                channel[y * aw + x] = ca[((y + top) * width + x + left) * 3 + c];
            }
        }
        apply_warp_into(&warp, aw as u32, ah as u32, &channel, &mut warped)
            .unwrap_or_else(|e| panic!("{path}: apply_warp_into: {e}"));
        for y in 0..ah {
            for x in 0..aw {
                ca[((y + top) * width + x + left) * 3 + c] = warped[y * aw + x];
            }
        }
    }
    drop((channel, warped));
    let mut lin = vec![0u16; plane * 3];
    to_srgb_linear_into(&stripped, &ca, &mut lin)
        .unwrap_or_else(|e| panic!("{path}: to_srgb_linear_into: {e}"));
    drop(ca);
    let origin = sensor.default_crop_origin.expect("a DefaultCropOrigin");
    let size = sensor.default_crop_size.expect("a DefaultCropSize");
    let (cw, ch) = (size.width, size.height);
    let o = sensor.orientation.unwrap_or(1);
    let (ow, oh) = if o >= 5 { (ch, cw) } else { (cw, ch) };
    assert_eq!(got.len(), (ow * oh * 3) as usize, "{path}: output size");
    let mut differ = 0usize;
    for oy in 0..oh {
        for ox in 0..ow {
            let (cx, cy) = crop_of(o, ox, oy, cw, ch);
            let (x, y) = (
                left + (origin.x + cx) as usize,
                top + (origin.y + cy) as usize,
            );
            let want = &lin[(y * width + x) * 3..][..3];
            let at = ((oy * ow + ox) * 3) as usize;
            differ += got[at..at + 3]
                .iter()
                .zip(want)
                .filter(|(a, b)| a != b)
                .count();
        }
    }
    assert_eq!(
        differ, 0,
        "{path}: samples that differ from the composition"
    );
}

/// `SPEC-100` AC7 (`DEC-088`): the Q2, Q3 and SL3 develop to colour and to a
/// developed mix with no refusal, at the pre-registered digests; the threaded
/// colour forms at 2, 3 and 8 threads give the same bytes; both mix entries
/// agree and `scratch_len` is `2A`; on the Q2 and Q3 the colour develop is
/// the composition at every sample. A digest that misses is reported, never
/// edited.
#[test]
fn ac7_the_opcode_files_develop() {
    let mut ran = 0;
    let mut mismatches = Vec::new();
    for (pins, developed) in OPCODE_PINS.iter().zip(&DEVELOPED_PINS) {
        let path = pins.path;
        assert_eq!(path, developed.path);
        let Some(bytes) = load(path) else {
            continue;
        };
        let (sensor, src) = unpack(path, &bytes);
        drop(bytes);
        let mut check = |entry: &str, got: u64, want: u64| {
            eprintln!("ac7 {path}: {entry} 0x{got:016x}");
            if got != want {
                mismatches.push(format!(
                    "{path}: {entry} 0x{got:016x}, registered 0x{want:016x}"
                ));
            }
        };

        // The colour develop.
        let (w, h) = output_dimensions(&sensor).unwrap_or_else(|e| panic!("{path}: {e}"));
        let n = w as usize * h as usize * 3;
        let mut colour = vec![0u16; n];
        develop_color_into(&sensor, &src, &mut colour)
            .unwrap_or_else(|e| panic!("{path}: develop_color_into: {e}"));
        check("develop_color_into", fnv1a(&colour), developed.colour);
        for threads in [2, 3, 8] {
            let mut t = vec![0xA5A5u16; n];
            develop_color_into_threaded(&sensor, &src, &mut t, threads)
                .unwrap_or_else(|e| panic!("{path}: threaded: {e}"));
            assert!(t == colour, "{path}: {threads} threads");
        }
        if pins.no_bad_photosite {
            assert_composition(path, &sensor, &src, pins, &colour);
        }
        let mut colour_f32 = vec![0f32; n];
        develop_color_f32_into(&sensor, &src, &mut colour_f32)
            .unwrap_or_else(|e| panic!("{path}: develop_color_f32_into: {e}"));
        assert!(
            colour_f32
                .iter()
                .zip(&colour)
                .all(|(a, &b)| a.to_bits() == (f32::from(b) / 65535.0).to_bits()),
            "{path}: the f32 develop is the u16 one over 65535"
        );
        drop(colour);
        check(
            "develop_color_f32_into",
            fnv1a_f32(&colour_f32),
            developed.colour_f32,
        );
        for threads in [2, 3, 8] {
            let mut t = vec![-1.0f32; n];
            develop_color_f32_into_threaded(&sensor, &src, &mut t, threads)
                .unwrap_or_else(|e| panic!("{path}: f32 threaded: {e}"));
            assert!(
                t.iter()
                    .map(|v| v.to_bits())
                    .eq(colour_f32.iter().map(|v| v.to_bits())),
                "{path}: f32 {threads} threads"
            );
        }
        drop(colour_f32);

        // The developed mix, on both entries.
        let active = sensor.active_area.expect("an ActiveArea");
        let a = (active.right - active.left) as usize * (active.bottom - active.top) as usize;
        let sets = [
            (Framing::StandardCrop, true, 1.0),
            (Framing::StandardCrop, true, 0.5),
            (Framing::StandardCrop, true, 0.0),
            (Framing::FullActiveArea, true, 1.0),
            (Framing::StandardCrop, false, 1.0),
        ];
        for (k, (framing, apply, alpha)) in sets.into_iter().enumerate() {
            let mut o = DevelopOptions::default();
            o.framing = framing;
            o.apply_orientation = apply;
            o.warp_alpha = alpha;
            o.mono_mix = Some(LUMA);
            let (mw, mh) = output_dimensions_with(&sensor, &o).unwrap();
            let mut dst = vec![0u16; mw as usize * mh as usize];
            develop_with_into(&sensor, &src, &mut dst, &o)
                .unwrap_or_else(|e| panic!("{path}: developed mix {k}: {e}"));
            check(&format!("developed mix {k}"), fnv1a(&dst), developed.mix[k]);
            let len = scratch_len(&sensor, &o).unwrap();
            if alpha > 0.0 {
                assert_eq!(len, 2 * a, "{path}: scratch_len is 2A");
            }
            let mut scratch = vec![0x5A5Au16; len];
            let mut dst2 = vec![0u16; dst.len()];
            develop_with_scratch_into(&sensor, &src, &mut dst2, &mut scratch, &o)
                .unwrap_or_else(|e| panic!("{path}: scratch mix {k}: {e}"));
            assert!(dst2 == dst, "{path}: the two mix entries agree ({k})");
        }
        ran += 1;
    }
    eprintln!(
        "ac7_the_opcode_files_develop: {ran}/{} file(s) present",
        DEVELOPED_PINS.len()
    );
    assert!(mismatches.is_empty(), "AC7 digests: {mismatches:#?}");
}

/// `SPEC-098` AC8: the SL3's bad photosites. 27,651 equal its `Constant` (0),
/// all in columns 9528 to 9535; 10,615 have a valid same-colour neighbour
/// (counted here from `CFAPattern`) and 17,036 do not. The fix changes the
/// plane in exactly 22,761 pixels, all in columns 9525 to 9535, and the
/// developed crop ends at column 9523.
#[test]
fn ac8_the_sl3_bad_columns_are_fixed() {
    let Some(bytes) = load(SL3) else {
        return;
    };
    let (sensor, src) = unpack(SL3, &bytes);
    drop(bytes);
    let (width, height) = (sensor.width as usize, sensor.height as usize);

    // The candidates: every photosite equal to 0 (the ActiveArea is the
    // whole plane on this file).
    let active = sensor.active_area.expect("an ActiveArea");
    assert_eq!(
        (active.top, active.left, active.bottom, active.right),
        (0, 0, sensor.height, sensor.width),
        "{SL3}: the ActiveArea is the whole plane"
    );
    let bad: Vec<(usize, usize)> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|&(x, y)| src[y * width + x] == 0)
        .collect();
    assert_eq!(bad.len(), 27_651, "{SL3}: photosites equal to 0");
    assert!(
        bad.iter().all(|&(x, _)| (9528..=9535).contains(&x)),
        "{SL3}: every bad photosite is in columns 9528 to 9535"
    );

    // How many have at least one valid same-colour neighbour: green reads
    // its diagonals, red and blue the photosites two away on the axes.
    let cfa = sensor.color_filter.as_ref().expect("a CFA");
    let fixable = bad
        .iter()
        .filter(|&&(x, y)| {
            let green = cfa.pattern[(y & 1) * 2 + (x & 1)] == 1;
            let offsets: [(isize, isize); 4] = if green {
                [(-1, -1), (1, -1), (-1, 1), (1, 1)]
            } else {
                [(0, -2), (-2, 0), (2, 0), (0, 2)]
            };
            offsets.iter().any(|&(dx, dy)| {
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                nx >= 0
                    && ny >= 0
                    && (nx as usize) < width
                    && (ny as usize) < height
                    && src[ny as usize * width + nx as usize] != 0
            })
        })
        .count();
    assert_eq!(
        (fixable, bad.len() - fixable),
        (10_615, 17_036),
        "{SL3}: fixed and unfixable photosites"
    );

    // The plane with list 1 against the plane with it removed. Both drop list
    // 3: since SPEC-099 (DEC-087) the planes resample red and blue to green's
    // geometry, which spreads each fixed photosite into its neighbours' red
    // and blue (28,012 changed pixels with list 3 kept); without the warp the
    // comparison isolates the fix, as SPEC-098 measured it.
    let mut sensor = sensor;
    sensor.opcode_list_3 = None;
    sensor.opcode_lists[2] = false;
    let mut fixed = vec![0u16; width * height * 3];
    srgb_linear_plane_into(&sensor, &src, &mut fixed).expect("the fixed plane");
    let mut unfixed_sensor = sensor.clone();
    unfixed_sensor.opcode_list_1 = None;
    unfixed_sensor.opcode_lists[0] = false;
    let mut unfixed = vec![0u16; width * height * 3];
    srgb_linear_plane_into(&unfixed_sensor, &src, &mut unfixed).expect("the unfixed plane");
    let changed: Vec<usize> = fixed
        .as_chunks::<3>()
        .0
        .iter()
        .zip(unfixed.as_chunks::<3>().0)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(pixel, _)| pixel % width)
        .collect();
    assert_eq!(changed.len(), 22_761, "{SL3}: plane pixels the fix changes");
    assert!(
        changed.iter().all(|x| (9525..=9535).contains(x)),
        "{SL3}: every changed pixel is in columns 9525 to 9535"
    );

    // The developed crop never reaches them.
    let origin = sensor.default_crop_origin.expect("a DefaultCropOrigin");
    let size = sensor.default_crop_size.expect("a DefaultCropSize");
    assert_eq!(
        active.left + origin.x + size.width - 1,
        9523,
        "{SL3}: the developed crop's last column"
    );
}
