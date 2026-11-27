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

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

#[path = "support/corpus.rs"]
mod corpus;

use corpus::{CorpusRoot, Manifest};
use irradiance::color::{develop_color_into, mono_mix_plane_into, srgb_linear_plane_into};
use irradiance::develop::{
    develop_with_into, output_dimensions, output_dimensions_with, DevelopOptions,
};
use irradiance::ifd::{Container, Rational, Sensor};
use irradiance::plane::unpack_into;

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
