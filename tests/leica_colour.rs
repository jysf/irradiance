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
//! ⚠ The Q3 (`LEICA-Q3-PB`) and SL3 (`LEICA-SL3-PB`) have no stated licence:
//! never committed, copied or redistributed, and no CI test may need them.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

#[path = "support/corpus.rs"]
mod corpus;

use corpus::{CorpusRoot, Manifest};
use irradiance::color::{
    develop_color_f32_into, develop_color_into, mono_mix_plane_into, mono_mix_plane_rows_into,
    mono_mix_row, srgb_linear_plane_f32_into, srgb_linear_plane_into,
    srgb_linear_plane_rows_f32_into, srgb_linear_plane_rows_into, to_srgb_linear_into,
};
use irradiance::demosaic::demosaic_into;
use irradiance::develop::{
    develop_into, develop_with_into, output_dimensions, output_dimensions_with, DevelopOptions,
};
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::plane::unpack_into;
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
         (a regression pin from SPEC-097's design probe: report it, do not edit it)"
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

/// One opcode file's facts and pins, from SPEC-098's Implementation Context.
struct OpcodePins {
    path: &'static str,
    /// `OpcodeList1`'s `FixBadPixelsConstant` `BayerPhase`.
    bayer_phase: u32,
    srgb_linear: u64,
    srgb_linear_f32: u64,
    mono_mix_plane: u64,
    /// The plain mono `develop_into` (unchanged by SPEC-098: `AC9`).
    mono_develop: u64,
}

const OPCODE_PINS: [OpcodePins; 3] = [
    OpcodePins {
        path: Q2,
        bayer_phase: 2,
        srgb_linear: 0x98af_7675_eef4_3b27,
        srgb_linear_f32: 0xc6a4_5aaf_c786_35d6,
        mono_mix_plane: 0x9452_ae0e_e0d3_622f,
        mono_develop: 0x1c1c_a269_399d_11ad,
    },
    OpcodePins {
        path: Q3,
        bayer_phase: 0,
        srgb_linear: 0x5525_7363_2a3f_a8df,
        srgb_linear_f32: 0x1776_ef7c_92e4_d4c2,
        mono_mix_plane: 0xcbf4_3ca3_6ab2_0784,
        mono_develop: 0xae9a_8252_ccfe_f17b,
    },
    OpcodePins {
        path: SL3,
        bayer_phase: 0,
        srgb_linear: 0x0aa0_de90_63b2_509a,
        srgb_linear_f32: 0x0eb0_1bee_c9da_33f4,
        mono_mix_plane: 0xe45e_c468_23e9_c4eb,
        mono_develop: 0x73fd_7636_37dd_dbc5,
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

/// `SPEC-098` AC7: the three opcode files reach every sensor-geometry entry
/// (the bad-pixel fix applied, the warp passed through), with their pinned
/// digests, and the developed outputs refuse the multi-plane warp.
#[test]
fn ac7_the_opcode_files_reach_the_sensor_geometry_entries() {
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

        // The sensor-geometry entries: Ok, pinned.
        let mut lin = vec![0u16; plane * 3];
        srgb_linear_plane_into(&sensor, &src, &mut lin)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_into: {e}"));
        assert_pin(
            path,
            "srgb_linear_plane_into",
            fnv1a(&lin),
            pins.srgb_linear,
        );
        let rows = 100..200u32;
        let band_range = rows.start as usize * width * 3..rows.end as usize * width * 3;
        let mut band = vec![0u16; band_range.len()];
        srgb_linear_plane_rows_into(&sensor, &src, rows.clone(), &mut band)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_rows_into: {e}"));
        assert!(
            band == lin[band_range.clone()],
            "{path}: the u16 band equals the plane"
        );
        drop(lin);

        let mut lin = vec![0f32; plane * 3];
        srgb_linear_plane_f32_into(&sensor, &src, &mut lin)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_f32_into: {e}"));
        assert_pin(
            path,
            "srgb_linear_plane_f32_into",
            fnv1a_f32(&lin),
            pins.srgb_linear_f32,
        );
        let mut band = vec![0f32; band_range.len()];
        srgb_linear_plane_rows_f32_into(&sensor, &src, rows.clone(), &mut band)
            .unwrap_or_else(|e| panic!("{path}: srgb_linear_plane_rows_f32_into: {e}"));
        assert!(
            band.iter()
                .map(|v| v.to_bits())
                .eq(lin[band_range].iter().map(|v| v.to_bits())),
            "{path}: the f32 band equals the plane"
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
        let mono_range = rows.start as usize * width..rows.end as usize * width;
        let mut band = vec![0u16; mono_range.len()];
        mono_mix_plane_rows_into(&sensor, &src, &LUMA, rows, &mut band)
            .unwrap_or_else(|e| panic!("{path}: mono_mix_plane_rows_into: {e}"));
        assert!(
            band == mono[mono_range],
            "{path}: the mixed band equals the plane"
        );
        drop(mono);
        mono_mix_row(&sensor, &LUMA).unwrap_or_else(|e| panic!("{path}: mono_mix_row: {e}"));

        // The plain mono develop is unchanged (AC9).
        let (w, h) = output_dimensions(&sensor).unwrap_or_else(|e| panic!("{path}: {e}"));
        let mut developed = vec![0u16; w as usize * h as usize];
        develop_into(&sensor, &src, &mut developed)
            .unwrap_or_else(|e| panic!("{path}: develop_into: {e}"));
        assert_pin(path, "develop_into", fnv1a(&developed), pins.mono_develop);
        drop(developed);

        // The developed outputs refuse the warp (SPEC-099); the primitives
        // still refuse the fix. Each refusal is the entry's first check, so no
        // buffer is needed.
        assert_refused(
            path,
            "develop_color_into",
            develop_color_into(&sensor, &src, &mut []),
            3,
            1,
        );
        assert_refused(
            path,
            "develop_color_f32_into",
            develop_color_f32_into(&sensor, &src, &mut []),
            3,
            1,
        );
        let mut options = DevelopOptions::default();
        options.mono_mix = Some(LUMA);
        assert_refused(
            path,
            "develop_with_into mono_mix",
            develop_with_into(&sensor, &src, &mut [], &options),
            3,
            1,
        );
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
        "ac7_the_opcode_files_reach_the_sensor_geometry_entries: {ran}/{} file(s) present",
        OPCODE_PINS.len()
    );
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

    // The plane with list 1 against the plane with it removed.
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
