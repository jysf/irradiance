//! `SPEC-091` — the white-balanced linear-sRGB plane at sensor geometry
//! (`DEC-079`): `srgb_linear_plane_into` (`u16`), `srgb_linear_plane_f32_into`
//! (unclamped `f32`) and their row-range (band) forms.
//!
//! - **Tier A** (always runs): a synthetic RGGB plane written by the SPEC-057
//!   test writer with an **odd** `ActiveArea` top and left, a masked border on
//!   every side and a `DefaultCrop` smaller than the `ActiveArea` (AC2, AC6,
//!   AC11).
//! - **Tier B** (`$IRRADIANCE_CORPUS_DIR`, skips per file by name): the five held
//!   colour files (AC1, AC3, AC4, AC7, AC11, AC13).
//!
//! The cross-architecture evidence for the plane's bits is
//! `tests/determinism.rs`'s colour fixture (AC12); the digests here are a local
//! record because the corpus is not in CI.

#[path = "support/corpus.rs"]
mod corpus;
#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use core::ops::Range;
use std::path::Path;

use corpus::{CorpusRoot, Manifest};
use dng_writer::{write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind};
use irradiance::color::{
    develop_color_into, srgb_linear_plane_f32_into, srgb_linear_plane_into,
    srgb_linear_plane_rows_f32_into, srgb_linear_plane_rows_into, to_srgb_linear_into,
};
use irradiance::demosaic::demosaic_into;
use irradiance::develop::output_dimensions;
use irradiance::ifd::{Container, Sensor};
use irradiance::plane::unpack_into;
use irradiance::Error;
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// The tier-A fixture
// ─────────────────────────────────────────────────────────────────────────────

const W: u32 = 20;
const H: u32 = 17;
/// `ActiveArea` `[top, left, bottom, right]`: odd top and left, so the CFA
/// phase from the ActiveArea origin differs from the plane origin's on both
/// axes (`DEC-040`, mutant f), and a masked border on all four sides.
const ACTIVE: [u32; 4] = [3, 1, 15, 19];
/// `DefaultCropOrigin` `[x, y]` (relative to the ActiveArea) and
/// `DefaultCropSize` `[w, h]`: a crop smaller than the 18 × 12 ActiveArea.
const CROP_ORIGIN: [u32; 2] = [2, 1];
const CROP_SIZE: [u32; 2] = [14, 9];

const SENTINEL_U16: u16 = 0xA5A5;
const SENTINEL_F32: f32 = -12345.5;

/// xorshift32: integer-only, so the samples are the same everywhere.
fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

/// Full-range noise with a saturated column band, so the plane has both
/// out-of-gamut (negative) and over-white values.
fn samples() -> Vec<u16> {
    let mut state = 0x5091_0001;
    (0..W * H)
        .map(|i| {
            let n = xorshift32(&mut state);
            if i % W >= 14 {
                // Near full scale: the white balance pushes red and blue over.
                (65535 - (n & 0x3ff)) as u16
            } else {
                (n & 0xffff) as u16
            }
        })
        .collect()
}

/// No `BlackLevel` and no `WhiteLevel` at 16 bits, so the level normaliser is
/// the identity and the plane can be checked against the independent
/// `demosaic_into → to_srgb_linear_into` composition (AC2 iv).
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
            default_crop_origin: Some(CROP_ORIGIN),
            default_crop_size: Some(CROP_SIZE),
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

fn plane_len(sensor: &Sensor) -> usize {
    sensor.width as usize * sensor.height as usize * 3
}

fn plane_u16(sensor: &Sensor, src: &[u16]) -> Vec<u16> {
    let mut dst = vec![SENTINEL_U16; plane_len(sensor)];
    srgb_linear_plane_into(sensor, src, &mut dst).expect("the u16 plane develops");
    dst
}

fn plane_f32(sensor: &Sensor, src: &[u16]) -> Vec<f32> {
    let mut dst = vec![SENTINEL_F32; plane_len(sensor)];
    srgb_linear_plane_f32_into(sensor, src, &mut dst).expect("the f32 plane develops");
    dst
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: the whole plane is written.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_every_pixel_of_the_plane_is_written() {
    let (sensor, src) = fixture();
    let u = plane_u16(&sensor, &src);
    let f = plane_f32(&sensor, &src);

    // (i) No sentinel survives, masked border included.
    let stale_u = u.iter().filter(|&&v| v == SENTINEL_U16).count();
    let stale_f = f
        .iter()
        .filter(|v| v.to_bits() == SENTINEL_F32.to_bits())
        .count();
    assert_eq!(stale_u, 0, "u16 plane: {stale_u} sentinel samples left");
    assert_eq!(stale_f, 0, "f32 plane: {stale_f} sentinel samples left");

    // (ii) Inside the ActiveArea (the crop's margin included) the plane is
    // develop_color_into with the DefaultCrop widened to the whole ActiveArea,
    // which changes neither the CFA phase nor the levels (DEC-040).
    let mut widened = sensor.clone();
    widened.default_crop_origin = None;
    widened.default_crop_size = None;
    let (ow, oh) = output_dimensions(&widened).expect("dimensions");
    let [top, left, bottom, right] = ACTIVE;
    assert_eq!((ow, oh), (right - left, bottom - top), "the widened crop");
    let mut developed = vec![SENTINEL_U16; (ow * oh * 3) as usize];
    develop_color_into(&widened, &src, &mut developed).expect("develops");
    let mut margin = 0;
    for y in top..bottom {
        for x in left..right {
            let p = ((y * W + x) * 3) as usize;
            let d = (((y - top) * ow + (x - left)) * 3) as usize;
            assert_eq!(
                u[p..p + 3],
                developed[d..d + 3],
                "plane pixel ({x}, {y}) vs the developed image"
            );
            let in_crop = (x - left) >= CROP_ORIGIN[0]
                && (x - left) < CROP_ORIGIN[0] + CROP_SIZE[0]
                && (y - top) >= CROP_ORIGIN[1]
                && (y - top) < CROP_ORIGIN[1] + CROP_SIZE[1];
            margin += usize::from(!in_crop);
        }
    }
    assert!(
        margin > 0,
        "the fixture has an ActiveArea margin outside the crop"
    );

    // (iii) Two calls give identical planes.
    assert_eq!(u, plane_u16(&sensor, &src), "u16 plane is repeatable");
    assert_eq!(
        bits(&f),
        bits(&plane_f32(&sensor, &src)),
        "f32 plane is repeatable"
    );

    // (iv) The whole plane, border included, equals the independent pre-fuse
    // composition demosaic_into -> to_srgb_linear_into (the levels here are
    // the identity). demosaic_into indexes the CFA phase on its own, so a fused
    // traversal phased from the plane origin (mutant f) differs here.
    let mut camera = vec![0u16; plane_len(&sensor)];
    demosaic_into(&sensor, &src, &mut camera).expect("demosaics");
    let mut composed = vec![0u16; plane_len(&sensor)];
    to_srgb_linear_into(&sensor, &camera, &mut composed).expect("transforms");
    assert!(u == composed, "the u16 plane equals demosaic -> matrix");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6: refusals, with dst untouched.
// ─────────────────────────────────────────────────────────────────────────────

/// Which of the four entries a refusal runs through.
#[derive(Debug, Clone, Copy)]
enum Entry {
    U16,
    F32,
    RowsU16,
    RowsF32,
}

const ENTRIES: [Entry; 4] = [Entry::U16, Entry::F32, Entry::RowsU16, Entry::RowsF32];

/// Run `entry` over `rows` (the band forms) into a sentinel `dst` of `len`;
/// return the result and whether `dst` is still exactly its sentinel fill.
fn refuse(
    entry: Entry,
    sensor: &Sensor,
    src: &[u16],
    rows: Range<u32>,
    len: usize,
) -> (Result<(), Error>, bool) {
    match entry {
        Entry::U16 | Entry::RowsU16 => {
            let mut dst = vec![SENTINEL_U16; len];
            let r = if matches!(entry, Entry::U16) {
                srgb_linear_plane_into(sensor, src, &mut dst)
            } else {
                srgb_linear_plane_rows_into(sensor, src, rows, &mut dst)
            };
            (r, dst.iter().all(|&v| v == SENTINEL_U16))
        }
        Entry::F32 | Entry::RowsF32 => {
            let mut dst = vec![SENTINEL_F32; len];
            let r = if matches!(entry, Entry::F32) {
                srgb_linear_plane_f32_into(sensor, src, &mut dst)
            } else {
                srgb_linear_plane_rows_f32_into(sensor, src, rows, &mut dst)
            };
            (r, dst.iter().all(|v| v.to_bits() == SENTINEL_F32.to_bits()))
        }
    }
}

#[test]
fn ac6_buffer_and_input_refusals_leave_dst_untouched() {
    let (sensor, src) = fixture();
    let n = plane_len(&sensor);
    let whole = 0..H;
    let whole_expected = u64::from(W * H * 3);

    for entry in ENTRIES {
        // A wrong src length.
        let (r, untouched) = refuse(entry, &sensor, &src[1..], whole.clone(), n);
        assert!(
            matches!(r, Err(Error::ColorSourceWrongLength { expected, actual })
                if expected == u64::from(W * H) && actual == src.len() - 1),
            "{entry:?} short src: {r:?}"
        );
        assert!(untouched, "{entry:?} short src: dst written");

        // dst one short and one long.
        for len in [n - 1, n + 1] {
            let (r, untouched) = refuse(entry, &sensor, &src, whole.clone(), len);
            assert!(
                matches!(r, Err(Error::RgbBufferWrongLength { expected, actual })
                    if expected == whole_expected && actual == len),
                "{entry:?} dst {len}: {r:?}"
            );
            assert!(untouched, "{entry:?} dst {len}: dst written");
        }

        // A non-CFA plane.
        let mut mono = sensor.clone();
        mono.color_filter = None;
        let (r, untouched) = refuse(entry, &mono, &src, whole.clone(), n);
        assert!(
            matches!(r, Err(Error::NotAColorFilterArray)),
            "{entry:?} non-CFA: {r:?}"
        );
        assert!(untouched, "{entry:?} non-CFA: dst written");

        // No AsShotNeutral: the calibration check runs last (mutant g).
        let mut no_neutral = sensor.clone();
        no_neutral.as_shot_neutral = None;
        let (r, untouched) = refuse(entry, &no_neutral, &src, whole.clone(), n);
        assert!(
            matches!(r, Err(Error::MissingColorCalibration)),
            "{entry:?} no AsShotNeutral: {r:?}"
        );
        assert!(untouched, "{entry:?} no AsShotNeutral: dst written");
    }

    // The band forms' own refusal: an empty, a reversed and an over-long range.
    for entry in [Entry::RowsU16, Entry::RowsF32] {
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
            let (r, untouched) = refuse(entry, &sensor, &src, rows, n);
            assert!(
                matches!(r, Err(Error::InvalidRowRange { start: s, end: e, height })
                    if s == start && e == end && height == H),
                "{entry:?} rows {start}..{end}: {r:?}"
            );
            assert!(untouched, "{entry:?} rows {start}..{end}: dst written");
        }

        // A band dst one short reports the band's expected length.
        let band = 4..11;
        let band_len = ((band.end - band.start) * W * 3) as usize;
        let (r, untouched) = refuse(entry, &sensor, &src, band, band_len - 1);
        assert!(
            matches!(r, Err(Error::RgbBufferWrongLength { expected, actual })
                if expected == band_len as u64 && actual == band_len - 1),
            "{entry:?} short band dst: {r:?}"
        );
        assert!(untouched, "{entry:?} short band dst: dst written");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC11: strips tile to the whole plane, bit for bit.
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

/// Develop `partition` strip by strip, into one whole-plane buffer each, and
/// require both sample types to equal the whole-plane forms exactly.
fn assert_strips_tile(sensor: &Sensor, src: &[u16], partition: &[Range<u32>], context: &str) {
    let whole_u = plane_u16(sensor, src);
    let whole_f = bits(&plane_f32(sensor, src));
    let row = sensor.width as usize * 3;
    let mut tiled_u = Vec::with_capacity(whole_u.len());
    let mut tiled_f = Vec::with_capacity(whole_f.len());
    let mut covered = 0;
    for rows in partition {
        assert_eq!(
            rows.start, covered,
            "{context}: the partition is contiguous"
        );
        covered = rows.end;
        let len = (rows.end - rows.start) as usize * row;
        let mut band_u = vec![SENTINEL_U16; len];
        srgb_linear_plane_rows_into(sensor, src, rows.clone(), &mut band_u)
            .unwrap_or_else(|e| panic!("{context}: rows {rows:?}: {e}"));
        let mut band_f = vec![SENTINEL_F32; len];
        srgb_linear_plane_rows_f32_into(sensor, src, rows.clone(), &mut band_f)
            .unwrap_or_else(|e| panic!("{context}: rows {rows:?}: {e}"));
        tiled_u.extend_from_slice(&band_u);
        tiled_f.extend(band_f.iter().map(|v| v.to_bits()));
    }
    assert_eq!(
        covered, sensor.height,
        "{context}: the partition covers 0..height"
    );
    if let Some(i) = (0..whole_u.len()).find(|&i| tiled_u[i] != whole_u[i]) {
        let p = i / 3;
        panic!(
            "{context}: u16 strips differ from the whole plane first at ({}, {}) channel {}",
            p % sensor.width as usize,
            p / sensor.width as usize,
            i % 3
        );
    }
    if let Some(i) = (0..whole_f.len()).find(|&i| tiled_f[i] != whole_f[i]) {
        let p = i / 3;
        panic!(
            "{context}: f32 strips differ from the whole plane first at ({}, {}) channel {}",
            p % sensor.width as usize,
            p / sensor.width as usize,
            i % 3
        );
    }
}

#[test]
fn ac11_strips_tile_to_the_whole_plane() {
    let (sensor, src) = fixture();
    let partitions: [(&str, Vec<Range<u32>>); 4] = [
        ("one strip", strips(H, 0, H)),
        ("strips of 1 row", strips(H, 0, 1)),
        ("strips of 7 rows", strips(H, 0, 7)),
        // Row 1 is inside the masked top border (ActiveArea top is 3), and odd.
        ("two strips split at row 1", vec![0..1, 1..H]),
    ];
    for (name, partition) in &partitions {
        assert_strips_tile(&sensor, &src, partition, name);
    }
    // The 7-row strips start at odd rows (7) as well as even ones.
    assert!(partitions[2].1.iter().any(|r| r.start % 2 == 1));
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

/// The held file's sensor and unpacked raw plane, or `None` (skip, by name).
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
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, container.byte_order(), &bytes, &mut plane)
        .unwrap_or_else(|e| panic!("{path}: unpack: {e}"));
    (sensor, plane)
}

/// The crop rectangle `(left, top, width, height)` in plane coordinates, from
/// the public `Sensor` fields: ActiveArea origin + DefaultCropOrigin, and the
/// output size (the DefaultCropSize under orientation 1).
fn crop_rect(sensor: &Sensor) -> (u32, u32, u32, u32) {
    let (active_top, active_left) = sensor.active_area.map_or((0, 0), |a| (a.top, a.left));
    let (origin_x, origin_y) = sensor.default_crop_origin.map_or((0, 0), |o| (o.x, o.y));
    let (w, h) = output_dimensions(sensor).expect("dimensions");
    (active_left + origin_x, active_top + origin_y, w, h)
}

#[test]
fn ac1_the_plane_cropped_equals_the_developed_image() {
    for path in HELD {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let mut upright = sensor.clone();
        upright.orientation = Some(1);
        let plane = plane_u16(&upright, &src);
        let (left, top, w, h) = crop_rect(&upright);
        let mut developed = vec![0u16; (w * h * 3) as usize];
        develop_color_into(&upright, &src, &mut developed).expect("develops");
        let width = upright.width as usize;
        let mut cut = Vec::with_capacity(developed.len());
        for y in top as usize..(top + h) as usize {
            let start = (y * width + left as usize) * 3;
            cut.extend_from_slice(&plane[start..start + w as usize * 3]);
        }
        assert!(
            cut == developed,
            "{path}: the plane cut to the crop is not develop_color_into's output"
        );
        eprintln!(
            "AC1 {path}: plane {}x{}, crop {w}x{h} at ({left}, {top}): {} bytes equal",
            upright.width,
            upright.height,
            developed.len() * 2
        );
    }
}

/// `(negative f32 channels where the u16 is 0, over-1.0 f32 channels where the
/// u16 is 65535)` over the whole plane.
fn clamp_counts(u: &[u16], f: &[f32]) -> (usize, usize) {
    let negative = u.iter().zip(f).filter(|(&u, &f)| u == 0 && f < 0.0).count();
    let over = u
        .iter()
        .zip(f)
        .filter(|(&u, &f)| u == 65535 && f > 1.0)
        .count();
    (negative, over)
}

#[test]
fn ac3_the_f32_plane_keeps_negatives_and_overs() {
    if let Some((sensor, src)) = corpus_plane(CANON) {
        let (negative, over) = clamp_counts(&plane_u16(&sensor, &src), &plane_f32(&sensor, &src));
        eprintln!("AC3 {CANON}: {negative} negative channel values, {over} over 1.0");
        assert!(negative > 0, "{CANON}: no negative f32 where the u16 is 0");
    }
    if let Some((sensor, src)) = corpus_plane(PIXEL_4A) {
        let (negative, over) = clamp_counts(&plane_u16(&sensor, &src), &plane_f32(&sensor, &src));
        eprintln!("AC3 {PIXEL_4A}: {negative} negative channel values, {over} over 1.0");
        assert!(
            over > 0,
            "{PIXEL_4A}: no f32 above 1.0 where the u16 is 65535"
        );
    }
}

#[test]
fn ac4_the_two_planes_agree_where_nothing_clamps() {
    for path in HELD {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let u = plane_u16(&sensor, &src);
        let f = plane_f32(&sensor, &src);
        // Half a code from the u16 rounding, plus the f32 cast's own rounding:
        // a u16 below 65535 means f < 1, where half an f32 ulp is 2^-25, i.e.
        // 65535 / 2^25 ≈ 0.00195 of a code. (The spec's 1e-3 slack is below
        // that; a Canon pixel at 0.5211032 sits 0.5029 codes from its u16.)
        let slack = 0.5 + 65535.0 / 33_554_432.0 + 1e-9;
        for (i, (&u, &f)) in u.iter().zip(&f).enumerate() {
            let ok = match u {
                0 => f <= 0.5 / 65535.0 + 1e-7,
                65535 => f >= (65535.0 - 0.5) / 65535.0 - 1e-7,
                _ => (f64::from(f) * 65535.0 - f64::from(u)).abs() <= slack,
            };
            assert!(ok, "{path}: sample {i}: u16 {u}, f32 {f}");
        }
    }
}

#[test]
fn ac7_gain_maps_apply_to_the_plane() {
    let Some((sensor, src)) = corpus_plane(PIXEL_7_PRO) else {
        return;
    };
    assert!(
        sensor.opcode_list_2.is_some(),
        "{PIXEL_7_PRO} carries OpcodeList2"
    );
    let mut without = sensor.clone();
    without.opcode_list_2 = None;
    assert!(
        plane_u16(&sensor, &src) != plane_u16(&without, &src),
        "{PIXEL_7_PRO}: the u16 plane ignores the GainMaps"
    );
    assert!(
        bits(&plane_f32(&sensor, &src)) != bits(&plane_f32(&without, &src)),
        "{PIXEL_7_PRO}: the f32 plane ignores the GainMaps"
    );
}

#[test]
fn ac11_strips_tile_on_held_files() {
    for path in [PIXEL_7_PRO, CANON] {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        // 256-row strips with an odd remainder: lead with a one-row strip when
        // the height leaves an even one, which also puts every 256-row strip
        // on an odd start.
        let lead = u32::from(sensor.height % 256 % 2 == 0);
        let partition = strips(sensor.height, lead, 256);
        let last = partition.last().expect("non-empty");
        assert_eq!((last.end - last.start) % 2, 1, "{path}: an odd remainder");
        eprintln!(
            "AC11 {path}: height {}, lead {lead}, {} strips, remainder {}",
            sensor.height,
            partition.len(),
            last.end - last.start
        );
        assert_strips_tile(&sensor, &src, &partition, path);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC13: golden digests of the held files (a local record; AC12 is the
// cross-architecture evidence).
// ─────────────────────────────────────────────────────────────────────────────

/// 64-bit FNV-1a, as `tests/determinism.rs`.
fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// `(file, u16 plane digest, f32 plane digest)`: FNV-1a over each `u16` and
/// each `f32`'s IEEE bits, little-endian. Recorded at SPEC-091's build on
/// macOS arm64; never edit one to make a run pass.
const GOLDEN: [(&str, u64, u64); 5] = [
    (CANON, 0x6013_2438_0cf1_9724, 0x2bdc_24d3_fb8f_0887),
    (PIXEL_3A, 0x74e2_30ed_d6b7_3330, 0xd4e5_1ae5_398f_bfdc),
    (PIXEL_4A, 0xce10_ba92_e648_05f3, 0xf2a8_dd47_127d_ae01),
    (PIXEL_7_PRO, 0x151f_ef25_e6ff_5fe2, 0x051d_4b2d_2719_32eb),
    (NIKON, 0xba5e_3669_46ef_4d98, 0xcf80_f1dd_4853_e6eb),
];

#[test]
fn ac13_golden_digests_of_the_held_files() {
    let mut mismatched = Vec::new();
    for (path, want_u, want_f) in GOLDEN {
        let Some((sensor, src)) = corpus_plane(path) else {
            continue;
        };
        let u = fnv1a(
            plane_u16(&sensor, &src)
                .iter()
                .flat_map(|v| v.to_le_bytes()),
        );
        let f = fnv1a(
            plane_f32(&sensor, &src)
                .iter()
                .flat_map(|v| v.to_bits().to_le_bytes()),
        );
        eprintln!("AC13 {path}: u16 {u:#018x} f32 {f:#018x}");
        if (u, f) != (want_u, want_f) {
            mismatched.push(path);
        }
    }
    assert!(mismatched.is_empty(), "digests diverged: {mismatched:?}");
}
