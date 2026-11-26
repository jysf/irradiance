//! `SPEC-087` (`DEC-078`): a malformed interpretation tag that a develop path
//! reads refuses that develop, instead of developing with the absent-tag
//! default.
//!
//! The parse stays tolerant (`DEC-012`): a malformed tag is `None` in its
//! field and its number in `Sensor::malformed_tags`, and `sensor()` and
//! `unpack_into` still succeed. Each develop entry now asks
//! `opcode::check_required_processing` about its own path, and the check
//! refuses `Error::MalformedTag { tag }` for the first (lowest) malformed tag
//! in that path's set, before anything is written.
//!
//! Every fixture goes through the real container read
//! (`Container::parse → sensor → unpack_into`) from a file written by the
//! test-only DNG writer, with the sensor in a SubIFD under a preview IFD0, as
//! `tests/unhonoured_tags.rs` does. A malformed tag is written with a field
//! type its reader rejects (`ASCII`, or `SHORT` for the two signatures, which
//! are `ASCII` by definition).
//!
//! AC1 is a generated cross product: every tag of `TAG_SETS` against every
//! entry of `ENTRIES`, the expectation read from the entry's path and the
//! tag's set. One list of (tag, set) drives every row, so a set the library
//! narrows or a path that asks about the wrong set turns a row red without a
//! hand-written case for it.
//!
//! **Tier-A only**: synthetic fixtures, no corpus, no tools.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, mono_mix_plane_into, mono_mix_plane_rows_into,
    srgb_linear_plane_f32_into, srgb_linear_plane_into, srgb_linear_plane_rows_f32_into,
    srgb_linear_plane_rows_into, to_srgb_linear_into,
};
use irradiance::demosaic::{demosaic_into, demosaic_into_threaded};
use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions, scratch_len,
    BlackLevels, DevelopOptions,
};
use irradiance::ifd::{Container, Sensor};
use irradiance::opcode::{check_required_processing, DevelopPath};
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind};

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag, TagIfd,
};
use tiff::Order;

/// SPEC-092's mixer weights (Rec. 709 luminance) for the two mono-mix entries.
const MIX: [irradiance::ifd::Rational; 3] = [
    irradiance::ifd::Rational {
        num: 2126,
        den: 10000,
    },
    irradiance::ifd::Rational {
        num: 7152,
        den: 10000,
    },
    irradiance::ifd::Rational {
        num: 722,
        den: 10000,
    },
];

// ─────────────────────────────────────────────────────────────────────────────
// Tag numbers (DNG 1.7.1.0; TIFF 6.0 for Orientation). Written out here rather
// than imported, so the test states the numbers DEC-078 names.
// ─────────────────────────────────────────────────────────────────────────────

const ORIENTATION: u16 = 274;
const SAMPLE_FORMAT: u16 = 339;
const CFA_REPEAT_PATTERN_DIM: u16 = 33421;
const CFA_PATTERN: u16 = 33422;
const DNG_VERSION: u16 = 50706;
const CFA_PLANE_COLOR: u16 = 50710;
const CFA_LAYOUT: u16 = 50711;
const BLACK_LEVEL_REPEAT_DIM: u16 = 50713;
const BLACK_LEVEL: u16 = 50714;
const WHITE_LEVEL: u16 = 50717;
const DEFAULT_CROP_ORIGIN: u16 = 50719;
const DEFAULT_CROP_SIZE: u16 = 50720;
const COLOR_MATRIX_1: u16 = 50721;
const COLOR_MATRIX_2: u16 = 50722;
const CAMERA_CALIBRATION_1: u16 = 50723;
const CAMERA_CALIBRATION_2: u16 = 50724;
const ANALOG_BALANCE: u16 = 50727;
const AS_SHOT_NEUTRAL: u16 = 50728;
const CALIBRATION_ILLUMINANT_1: u16 = 50778;
const CALIBRATION_ILLUMINANT_2: u16 = 50779;
const ACTIVE_AREA: u16 = 50829;
const MASKED_AREAS: u16 = 50830;
const CAMERA_CALIBRATION_SIGNATURE: u16 = 50931;
const PROFILE_CALIBRATION_SIGNATURE: u16 = 50932;
const FORWARD_MATRIX_1: u16 = 50964;
const FORWARD_MATRIX_2: u16 = 50965;
const OPCODE_LIST_1: u16 = 51008;
const OPCODE_LIST_2: u16 = 51009;

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_UNDEFINED: u16 = 7;

// ─────────────────────────────────────────────────────────────────────────────
// DEC-078 §2: the three sets and which path reads which. The one list.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Set {
    /// Levels and geometry.
    Levels,
    /// `CFAPlaneColor`, `CFALayout`.
    Cfa,
    /// The colour calibration tags.
    Colour,
}

/// Every tag of `DEC-078` §2's three sets, with its set. `BlackLevelRepeatDim`
/// is here with its exception: its rows carry four `BlackLevel` values (see
/// `row_base`), so the tag counts.
const TAG_SETS: [(u16, Set); 20] = [
    (ORIENTATION, Set::Levels),
    (BLACK_LEVEL_REPEAT_DIM, Set::Levels),
    (BLACK_LEVEL, Set::Levels),
    (WHITE_LEVEL, Set::Levels),
    (DEFAULT_CROP_ORIGIN, Set::Levels),
    (DEFAULT_CROP_SIZE, Set::Levels),
    (ACTIVE_AREA, Set::Levels),
    (CFA_PLANE_COLOR, Set::Cfa),
    (CFA_LAYOUT, Set::Cfa),
    (COLOR_MATRIX_1, Set::Colour),
    (COLOR_MATRIX_2, Set::Colour),
    (CAMERA_CALIBRATION_1, Set::Colour),
    (CAMERA_CALIBRATION_2, Set::Colour),
    (ANALOG_BALANCE, Set::Colour),
    (CALIBRATION_ILLUMINANT_1, Set::Colour),
    (CALIBRATION_ILLUMINANT_2, Set::Colour),
    (CAMERA_CALIBRATION_SIGNATURE, Set::Colour),
    (PROFILE_CALIBRATION_SIGNATURE, Set::Colour),
    (FORWARD_MATRIX_1, Set::Colour),
    (FORWARD_MATRIX_2, Set::Colour),
];

/// The sets `path` refuses on (`DEC-078` §2's table).
fn path_sets(path: DevelopPath) -> &'static [Set] {
    match path {
        DevelopPath::Mono => &[Set::Levels],
        DevelopPath::Colour => &[Set::Levels, Set::Cfa, Set::Colour],
        DevelopPath::Demosaic => &[Set::Cfa],
        DevelopPath::ColourTransform => &[Set::Colour],
        other => panic!("no DEC-078 row for {other:?}"),
    }
}

/// The IFD a tag lives in: the colour tags and `Orientation` in IFD0 (where
/// `Container::sensor` reads them), the rest in the sensor IFD.
fn tag_ifd(tag: u16) -> TagIfd {
    match tag {
        ORIENTATION
        | COLOR_MATRIX_1
        | COLOR_MATRIX_2
        | CAMERA_CALIBRATION_1
        | CAMERA_CALIBRATION_2
        | ANALOG_BALANCE
        | CALIBRATION_ILLUMINANT_1
        | CALIBRATION_ILLUMINANT_2
        | CAMERA_CALIBRATION_SIGNATURE
        | PROFILE_CALIBRATION_SIGNATURE
        | FORWARD_MATRIX_1
        | FORWARD_MATRIX_2
        | DNG_VERSION => TagIfd::Ifd0,
        _ => TagIfd::Sensor,
    }
}

/// `tag` written with a field type its reader rejects: `ASCII` "x", or one
/// `SHORT` for the two signatures (`ASCII` and `BYTE` are their own types).
fn malformed(tag: u16) -> RawTag {
    if matches!(
        tag,
        CAMERA_CALIBRATION_SIGNATURE | PROFILE_CALIBRATION_SIGNATURE
    ) {
        return RawTag {
            ifd: tag_ifd(tag),
            tag,
            field_type: TYPE_SHORT,
            count: 1,
            bytes: 7u16.to_le_bytes().to_vec(),
        };
    }
    RawTag {
        ifd: tag_ifd(tag),
        tag,
        field_type: TYPE_ASCII,
        count: 2,
        bytes: b"x\0".to_vec(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures: an 8×8 16-bit plane, mono (LinearRaw) or 2×2 RGGB.
// ─────────────────────────────────────────────────────────────────────────────

const W: u32 = 8;
const H: u32 = 8;

fn samples() -> Vec<u16> {
    (0..W * H).map(|i| 1000 + (i as u16) * 37).collect()
}

fn base_desc(cfa: bool) -> DngDescription {
    let kind = if cfa {
        PlaneKind::Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
        }
    } else {
        PlaneKind::LinearRaw
    };
    let mut meta = DngMeta::default();
    if cfa {
        // The colour calibration the colour develop needs (the same matrix
        // and neutral `tests/unhonoured_tags.rs` uses).
        meta.forward_matrix_1 = Some(
            [
                [60000, 30000, 6422],
                [20000, 70000, 10000],
                [10000, 10000, 62521],
            ]
            .iter()
            .flatten()
            .map(|&num| (num, 100_000))
            .collect(),
        );
        meta.as_shot_neutral = Some(vec![(1, 2), (1, 1), (1, 2)]);
    }
    DngDescription {
        order: Order::Little,
        layout: IfdLayout::PreviewIfd0WithSensorSubIfd,
        width: W,
        height: H,
        bits_per_sample: 16,
        kind,
        organisation: Organisation::Strips { rows_per_strip: H },
        payload: Payload::Samples(samples()),
        meta,
        opcode_lists: [None, None, None],
        extra_tags: Vec::new(),
        fault: None,
    }
}

/// A fixture, read the way a caller reads a camera file.
struct Fixture {
    sensor: Sensor,
    plane: Vec<u16>,
}

/// `load` asserts both reads succeed: that is AC6's "reads unaffected".
fn load(d: &DngDescription) -> Fixture {
    let file = write_dng(d).expect("the fixture writes");
    let container = Container::parse(&file).expect("the fixture parses");
    let sensor = container.sensor().expect("sensor() succeeds");
    let mut plane = vec![0u16; (W * H) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpack_into succeeds");
    Fixture { sensor, plane }
}

fn fixture_for(entry: Entry, modify: &dyn Fn(&mut DngDescription)) -> Fixture {
    let mut d = base_desc(!entry.mono());
    modify(&mut d);
    load(&d)
}

/// What every fixture of `tag`'s row carries besides the tag: four distinct
/// `BlackLevel` values for `BlackLevelRepeatDim`, so its exception does not
/// apply; and, for `ForwardMatrix1`, its absence (the CFA base writes one, and
/// "absent" must mean absent). `CFAPlaneColor` stays as the writer always
/// writes it, `[0, 1, 2]`, which is also its DNG default.
fn row_base(tag: u16, d: &mut DngDescription) {
    match tag {
        BLACK_LEVEL_REPEAT_DIM => d.meta.black_level = Some(vec![10, 20, 30, 40]),
        FORWARD_MATRIX_1 => d.meta.forward_matrix_1 = None,
        _ => {}
    }
}

fn with_malformed(tag: u16) -> impl Fn(&mut DngDescription) {
    move |d| {
        row_base(tag, d);
        d.extra_tags.push(malformed(tag));
    }
}

fn with_absent(tag: u16) -> impl Fn(&mut DngDescription) {
    move |d| row_base(tag, d)
}

/// Whether `entry`'s fixture reads `tag` at all: the CFA tags are read only on
/// a CFA plane.
fn fixture_reads(entry: Entry, tag: u16) -> bool {
    !(entry.mono() && matches!(tag, CFA_PLANE_COLOR | CFA_LAYOUT))
}

// ─────────────────────────────────────────────────────────────────────────────
// The sixteen develop entries (SPEC-091 added four, SPEC-092 two), each its own door (§12).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    DevelopInto,
    DevelopWithInto,
    DevelopWithScratchInto,
    DevelopColorInto,
    DevelopColorF32Into,
    DevelopColorIntoThreaded,
    DevelopColorF32IntoThreaded,
    ToSrgbLinearInto,
    DemosaicInto,
    DemosaicIntoThreaded,
    SrgbLinearPlaneInto,
    SrgbLinearPlaneF32Into,
    SrgbLinearPlaneRowsInto,
    SrgbLinearPlaneRowsF32Into,
    MonoMixPlaneInto,
    MonoMixPlaneRowsInto,
}

const ENTRIES: [Entry; 16] = [
    Entry::DevelopInto,
    Entry::DevelopWithInto,
    Entry::DevelopWithScratchInto,
    Entry::DevelopColorInto,
    Entry::DevelopColorF32Into,
    Entry::DevelopColorIntoThreaded,
    Entry::DevelopColorF32IntoThreaded,
    Entry::ToSrgbLinearInto,
    Entry::DemosaicInto,
    Entry::DemosaicIntoThreaded,
    Entry::SrgbLinearPlaneInto,
    Entry::SrgbLinearPlaneF32Into,
    Entry::SrgbLinearPlaneRowsInto,
    Entry::SrgbLinearPlaneRowsF32Into,
    Entry::MonoMixPlaneInto,
    Entry::MonoMixPlaneRowsInto,
];

impl Entry {
    fn mono(self) -> bool {
        self.path() == DevelopPath::Mono
    }

    /// The path the entry asks about (`DEC-078` §2's table).
    fn path(self) -> DevelopPath {
        match self {
            Entry::DevelopInto | Entry::DevelopWithInto | Entry::DevelopWithScratchInto => {
                DevelopPath::Mono
            }
            Entry::DevelopColorInto
            | Entry::DevelopColorF32Into
            | Entry::DevelopColorIntoThreaded
            | Entry::DevelopColorF32IntoThreaded
            | Entry::SrgbLinearPlaneInto
            | Entry::SrgbLinearPlaneF32Into
            | Entry::SrgbLinearPlaneRowsInto
            | Entry::SrgbLinearPlaneRowsF32Into
            | Entry::MonoMixPlaneInto
            | Entry::MonoMixPlaneRowsInto => DevelopPath::Colour,
            Entry::ToSrgbLinearInto => DevelopPath::ColourTransform,
            Entry::DemosaicInto | Entry::DemosaicIntoThreaded => DevelopPath::Demosaic,
        }
    }

    /// Whether the entry reads `DevelopOptions` (so AC2 runs it under both
    /// `BlackLevels` options).
    fn takes_options(self) -> bool {
        matches!(self, Entry::DevelopWithInto | Entry::DevelopWithScratchInto)
    }
}

/// A sentinel no develop writes, so an untouched buffer is recognisable.
const SENTINEL_U16: u16 = 0xA5A5;
const SENTINEL_F32: f32 = -12345.5;

/// Everything an entry wrote: `dst` (and the caller's scratch, after it).
/// `f32` output is kept as bits so equality is byte equality.
#[derive(Debug, PartialEq, Eq)]
enum Written {
    U16(Vec<u16>),
    F32(Vec<u32>),
}

impl Written {
    fn untouched(&self) -> bool {
        match self {
            Written::U16(v) => v.iter().all(|&x| x == SENTINEL_U16),
            Written::F32(v) => v.iter().all(|&x| x == SENTINEL_F32.to_bits()),
        }
    }
}

fn run(entry: Entry, fx: &Fixture) -> (Result<(), Error>, Written) {
    run_with(entry, fx, &DevelopOptions::default())
}

fn run_with(entry: Entry, fx: &Fixture, options: &DevelopOptions) -> (Result<(), Error>, Written) {
    let sensor = &fx.sensor;
    let plane = &fx.plane;
    // A geometry the develop would refuse still needs a buffer to refuse
    // into; the raw plane's size stands in.
    let out_len = || {
        output_dimensions(sensor)
            .map(|(w, h)| w as usize * h as usize)
            .unwrap_or((W * H) as usize)
    };
    let raw3 = W as usize * H as usize * 3;
    let u16s = |n: usize| vec![SENTINEL_U16; n];
    let f32s = |n: usize| vec![SENTINEL_F32; n];
    let bits = |v: Vec<f32>| Written::F32(v.into_iter().map(f32::to_bits).collect());
    match entry {
        Entry::DevelopInto => {
            let mut dst = u16s(out_len());
            let r = develop_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DevelopWithInto => {
            let mut dst = u16s(out_len());
            let r = develop_with_into(sensor, plane, &mut dst, options);
            (r, Written::U16(dst))
        }
        Entry::DevelopWithScratchInto => {
            let n = scratch_len(sensor, options).unwrap_or(0);
            let mut dst = u16s(out_len());
            let mut scratch = u16s(n);
            let r = develop_with_scratch_into(sensor, plane, &mut dst, &mut scratch, options);
            dst.extend_from_slice(&scratch);
            (r, Written::U16(dst))
        }
        Entry::DevelopColorInto => {
            let mut dst = u16s(out_len() * 3);
            let r = develop_color_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DevelopColorF32Into => {
            let mut dst = f32s(out_len() * 3);
            let r = develop_color_f32_into(sensor, plane, &mut dst);
            (r, bits(dst))
        }
        Entry::DevelopColorIntoThreaded => {
            let mut dst = u16s(out_len() * 3);
            let r = develop_color_into_threaded(sensor, plane, &mut dst, 4);
            (r, Written::U16(dst))
        }
        Entry::DevelopColorF32IntoThreaded => {
            let mut dst = f32s(out_len() * 3);
            let r = develop_color_f32_into_threaded(sensor, plane, &mut dst, 4);
            (r, bits(dst))
        }
        Entry::ToSrgbLinearInto => {
            // Camera-space RGB of the raw plane's size; the values only need
            // to be deterministic.
            let camera: Vec<u16> = (0..raw3).map(|i| (i % 4096) as u16).collect();
            let mut dst = u16s(raw3);
            let r = to_srgb_linear_into(sensor, &camera, &mut dst);
            (r, Written::U16(dst))
        }
        // SPEC-091: the sensor-geometry plane, whole and as the whole-plane band.
        Entry::SrgbLinearPlaneInto => {
            let mut dst = u16s(raw3);
            let r = srgb_linear_plane_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::SrgbLinearPlaneF32Into => {
            let mut dst = f32s(raw3);
            let r = srgb_linear_plane_f32_into(sensor, plane, &mut dst);
            (r, bits(dst))
        }
        Entry::SrgbLinearPlaneRowsInto => {
            let mut dst = u16s(raw3);
            let r = srgb_linear_plane_rows_into(sensor, plane, 0..H, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::SrgbLinearPlaneRowsF32Into => {
            let mut dst = f32s(raw3);
            let r = srgb_linear_plane_rows_f32_into(sensor, plane, 0..H, &mut dst);
            (r, bits(dst))
        }
        // SPEC-092: the mono mixer plane, whole and as the whole-plane band.
        Entry::MonoMixPlaneInto => {
            let mut dst = u16s(raw3 / 3);
            let r = mono_mix_plane_into(sensor, plane, &MIX, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::MonoMixPlaneRowsInto => {
            let mut dst = u16s(raw3 / 3);
            let r = mono_mix_plane_rows_into(sensor, plane, &MIX, 0..H, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DemosaicInto => {
            let mut dst = u16s(raw3);
            let r = demosaic_into(sensor, plane, &mut dst);
            (r, Written::U16(dst))
        }
        Entry::DemosaicIntoThreaded => {
            let mut dst = u16s(raw3);
            let r = demosaic_into_threaded(sensor, plane, &mut dst, 4);
            (r, Written::U16(dst))
        }
    }
}

/// The two `BlackLevels` options, for the entries that take options.
fn black_options() -> [DevelopOptions; 2] {
    [BlackLevels::FirstValue, BlackLevels::PerPosition].map(|black_levels| {
        let mut options = DevelopOptions::default();
        options.black_levels = black_levels;
        options
    })
}

/// The options `entry` is run under: both for an option-taking entry, the
/// default otherwise.
fn options_for(entry: Entry) -> Vec<DevelopOptions> {
    if entry.takes_options() {
        black_options().to_vec()
    } else {
        vec![DevelopOptions::default()]
    }
}

#[track_caller]
fn assert_malformed(r: &Result<(), Error>, tag: u16, context: &str) {
    assert!(
        matches!(r, Err(Error::MalformedTag { tag: got }) if *got == tag),
        "{context}: expected MalformedTag {{ tag: {tag} }}, got {r:?}"
    );
    let err = r.as_ref().expect_err("checked above");
    assert_eq!(err.kind(), ErrorKind::Corrupt, "{context}");
}

/// A result as text, so two errors compare by variant and fields.
fn shown(r: &Result<(), Error>) -> String {
    format!("{r:?}")
}

// Shown red first: written before the implementation, this file did not
// compile (no `Error::MalformedTag`, no `DevelopPath::Demosaic` or
// `DevelopPath::ColourTransform`), and the AC9 mutants in HANDOFF-182's
// handback each turn a named test red.

// ─────────────────────────────────────────────────────────────────────────────
// AC1: each path refuses its own set, on every entry.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac1_each_path_refuses_its_own_malformed_set_on_every_entry() {
    let mut refused = 0;
    for (tag, set) in TAG_SETS {
        for entry in ENTRIES {
            if !path_sets(entry.path()).contains(&set) {
                continue;
            }
            let fx = fixture_for(entry, &with_malformed(tag));
            let context = format!("{tag} ({set:?}) on {entry:?}");
            assert!(
                fx.sensor.malformed_tags.contains(&tag),
                "{context}: malformed_tags {:?} must name {tag}",
                fx.sensor.malformed_tags
            );
            let (r, written) = run(entry, &fx);
            assert_malformed(&r, tag, &context);
            assert!(
                written.untouched(),
                "{context}: dst (and scratch) must be untouched by a refusal"
            );
            refused += 1;
        }
    }
    // 7 levels tags × 13 entries, 2 CFA × 12, 11 colour × 11 (SPEC-091 added
    // four colour entries, SPEC-092 two): the cross product is not vacuous.
    assert_eq!(refused, 7 * 13 + 2 * 12 + 11 * 11);
}

#[test]
fn ac1_a_tag_outside_the_path_set_behaves_as_absent() {
    let mut compared = 0;
    for (tag, set) in TAG_SETS {
        for entry in ENTRIES {
            if path_sets(entry.path()).contains(&set) {
                continue;
            }
            let context = format!("{tag} ({set:?}) on {entry:?}");
            let fx = fixture_for(entry, &with_malformed(tag));
            if fixture_reads(entry, tag) {
                assert!(
                    fx.sensor.malformed_tags.contains(&tag),
                    "{context}: malformed_tags {:?} must name {tag}",
                    fx.sensor.malformed_tags
                );
            }
            let (r, written) = run(entry, &fx);
            let (absent_r, absent_written) = run(entry, &fixture_for(entry, &with_absent(tag)));
            assert_eq!(
                shown(&r),
                shown(&absent_r),
                "{context}: the result must be the absent tag's"
            );
            assert!(
                written == absent_written,
                "{context}: the output must be the absent tag's"
            );
            compared += 1;
        }
    }
    assert_eq!(compared, 7 * 3 + 2 * 4 + 11 * 5);
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: the repeat-dimension exception, both directions.
// ─────────────────────────────────────────────────────────────────────────────

/// The mono and colour develop entries: the two paths that read levels.
fn levels_entries() -> impl Iterator<Item = Entry> {
    ENTRIES
        .into_iter()
        .filter(|e| path_sets(e.path()).contains(&Set::Levels))
}

fn black_levels(values: Vec<u32>) -> impl Fn(&mut DngDescription) {
    move |d| d.meta.black_level = Some(values.clone())
}

fn black_levels_and_malformed_repeat_dim(values: Vec<u32>) -> impl Fn(&mut DngDescription) {
    move |d| {
        d.meta.black_level = Some(values.clone());
        d.extra_tags.push(malformed(BLACK_LEVEL_REPEAT_DIM));
    }
}

#[test]
fn ac2_a_malformed_repeat_dim_with_one_black_value_develops() {
    for entry in levels_entries() {
        let fx = fixture_for(entry, &black_levels_and_malformed_repeat_dim(vec![100]));
        assert!(
            fx.sensor.malformed_tags.contains(&BLACK_LEVEL_REPEAT_DIM),
            "{entry:?}: {:?}",
            fx.sensor.malformed_tags
        );
        let absent = fixture_for(entry, &black_levels(vec![100]));
        for options in options_for(entry) {
            let context = format!("{entry:?} under {:?}", options.black_levels);
            let (r, written) = run_with(entry, &fx, &options);
            let (absent_r, absent_written) = run_with(entry, &absent, &options);
            assert!(absent_r.is_ok(), "{context}: control {absent_r:?}");
            assert!(r.is_ok(), "{context}: must develop, got {r:?}");
            assert!(
                written == absent_written,
                "{context}: output differs from the fixture without the tag"
            );
        }
    }
}

#[test]
fn ac2_a_malformed_repeat_dim_with_four_black_values_refuses() {
    for entry in levels_entries() {
        let fx = fixture_for(
            entry,
            &black_levels_and_malformed_repeat_dim(vec![100, 110, 120, 130]),
        );
        for options in options_for(entry) {
            let context = format!("{entry:?} under {:?}", options.black_levels);
            let (r, written) = run_with(entry, &fx, &options);
            assert_malformed(&r, BLACK_LEVEL_REPEAT_DIM, &context);
            assert!(written.untouched(), "{context}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: DEC-077 first, then the malformed check in ascending order, then the
// opcode lists.
// ─────────────────────────────────────────────────────────────────────────────

/// An `OpcodeList2` whose payload points past the end of the file: recorded
/// in `malformed_tags`, refused as `UnreadableOpcodeList { list: 2 }`.
fn unreadable_opcode_list(tag: u16) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type: TYPE_UNDEFINED,
        count: 0x00FF_FFFF,
        bytes: vec![0xFF; 4],
    }
}

#[test]
fn ac3_order_dec077_then_malformed_then_opcode_lists() {
    // Written in DESCENDING order, so the file's own order cannot be what the
    // check follows.
    let two_malformed = |d: &mut DngDescription| {
        d.extra_tags.push(malformed(ACTIVE_AREA));
        d.extra_tags.push(malformed(WHITE_LEVEL));
    };
    let dec077_first = |d: &mut DngDescription| {
        d.extra_tags.push(malformed(WHITE_LEVEL));
        d.extra_tags.push(RawTag {
            ifd: TagIfd::Sensor,
            tag: SAMPLE_FORMAT,
            field_type: TYPE_SHORT,
            count: 1,
            bytes: 3u16.to_le_bytes().to_vec(),
        });
    };
    let before_lists = |d: &mut DngDescription| {
        d.extra_tags.push(unreadable_opcode_list(OPCODE_LIST_2));
        d.extra_tags.push(malformed(WHITE_LEVEL));
    };
    // The sets interleave on the colour path: the levels set walks first, but
    // CFALayout (50711) sorts below BlackLevel (50714). Lowest wins, so a
    // walk-order check (the first malformed tag of the first set) is red here.
    let interleaved = |d: &mut DngDescription| {
        d.extra_tags.push(malformed(BLACK_LEVEL));
        d.extra_tags.push(malformed(CFA_LAYOUT));
    };
    for entry in ENTRIES {
        let reads_levels = path_sets(entry.path()).contains(&Set::Levels);

        let (r, written) = run(entry, &fixture_for(entry, &interleaved));
        let context = format!("interleaved sets on {entry:?}");
        match entry.path() {
            DevelopPath::Colour | DevelopPath::Demosaic => {
                assert_malformed(&r, CFA_LAYOUT, &context);
                assert!(written.untouched(), "{context}");
            }
            // The mono fixture has no CFA, so only BlackLevel is read.
            DevelopPath::Mono => {
                assert_malformed(&r, BLACK_LEVEL, &context);
                assert!(written.untouched(), "{context}");
            }
            _ => assert!(
                !matches!(r, Err(Error::MalformedTag { .. })),
                "{context}: {r:?}"
            ),
        }

        let (r, written) = run(entry, &fixture_for(entry, &two_malformed));
        if reads_levels {
            assert_malformed(&r, WHITE_LEVEL, &format!("two malformed on {entry:?}"));
            assert!(written.untouched(), "{entry:?}");
        }

        let (r, written) = run(entry, &fixture_for(entry, &dec077_first));
        assert!(
            matches!(r, Err(Error::UnsupportedTag { tag: SAMPLE_FORMAT })),
            "DEC-077 first on {entry:?}: {r:?}"
        );
        assert!(written.untouched(), "{entry:?}");

        let fx = fixture_for(entry, &before_lists);
        assert!(fx.sensor.malformed_tags.contains(&OPCODE_LIST_2));
        let (r, written) = run(entry, &fx);
        if reads_levels {
            assert_malformed(&r, WHITE_LEVEL, &format!("before the lists on {entry:?}"));
        } else {
            assert!(
                matches!(r, Err(Error::UnreadableOpcodeList { list: 2 })),
                "{entry:?}: {r:?}"
            );
        }
        assert!(written.untouched(), "{entry:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC4: a malformed DNGVersion with no DNGBackwardVersion.
// ─────────────────────────────────────────────────────────────────────────────

fn malformed_dng_version(backward: Option<[u8; 4]>) -> impl Fn(&mut DngDescription) {
    move |d| {
        d.meta.dng_backward_version = backward;
        d.extra_tags.push(RawTag {
            ifd: TagIfd::Ifd0,
            tag: DNG_VERSION,
            field_type: TYPE_BYTE,
            count: 3,
            bytes: vec![1, 4, 0],
        });
    }
}

#[test]
fn ac4_a_malformed_dng_version_without_a_backward_version_refuses() {
    for entry in ENTRIES {
        let fx = fixture_for(entry, &malformed_dng_version(None));
        assert!(
            fx.sensor.malformed_tags.contains(&DNG_VERSION),
            "{entry:?}: {:?}",
            fx.sensor.malformed_tags
        );
        assert_eq!(fx.sensor.dng_version, None);
        assert_eq!(fx.sensor.dng_backward_version, None);
        let (r, written) = run(entry, &fx);
        assert!(
            matches!(r, Err(Error::UnsupportedTag { tag: DNG_VERSION })),
            "{entry:?}: {r:?}"
        );
        assert_eq!(
            r.as_ref().map_err(Error::kind),
            Err(ErrorKind::Unsupported),
            "{entry:?}"
        );
        assert!(written.untouched(), "{entry:?}");

        // A well-formed backward version decides instead.
        let fx = fixture_for(entry, &malformed_dng_version(Some([1, 4, 0, 0])));
        assert!(fx.sensor.malformed_tags.contains(&DNG_VERSION));
        let (r, _) = run(entry, &fx);
        assert!(r.is_ok(), "backward 1.4.0.0 on {entry:?}: {r:?}");
    }

    let msg = Error::UnsupportedTag { tag: DNG_VERSION }.to_string();
    assert!(msg.contains("DNGVersion"), "{msg}");
    assert!(msg.contains("50706"), "{msg}");
    assert!(msg.contains("reader version is unknown"), "{msg}");
    assert!(!msg.contains("samples mean"), "{msg}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5: tags already refused another way keep their errors.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac5_already_refused_tags_keep_their_errors() {
    let cfa_readers = [
        Entry::DevelopColorInto,
        Entry::DevelopColorF32Into,
        Entry::DevelopColorIntoThreaded,
        Entry::DevelopColorF32IntoThreaded,
        Entry::DemosaicInto,
        Entry::DemosaicIntoThreaded,
    ];
    let calibration_readers = [
        Entry::DevelopColorInto,
        Entry::DevelopColorF32Into,
        Entry::DevelopColorIntoThreaded,
        Entry::DevelopColorF32IntoThreaded,
        Entry::ToSrgbLinearInto,
    ];
    let sensor_ascii = |tag: u16| RawTag {
        ifd: TagIfd::Sensor,
        tag,
        field_type: TYPE_ASCII,
        count: 2,
        bytes: b"x\0".to_vec(),
    };

    for entry in cfa_readers {
        let fx = fixture_for(entry, &|d| d.extra_tags.push(sensor_ascii(CFA_PATTERN)));
        assert!(fx.sensor.malformed_tags.contains(&CFA_PATTERN));
        let (r, written) = run(entry, &fx);
        assert!(
            matches!(r, Err(Error::NotAColorFilterArray)),
            "CFAPattern on {entry:?}: {r:?}"
        );
        assert!(written.untouched(), "{entry:?}");

        let fx = fixture_for(entry, &|d| {
            d.extra_tags.push(sensor_ascii(CFA_REPEAT_PATTERN_DIM));
        });
        assert!(fx.sensor.malformed_tags.contains(&CFA_REPEAT_PATTERN_DIM));
        let (r, written) = run(entry, &fx);
        assert!(
            matches!(r, Err(Error::UnsupportedCfaPattern { .. })),
            "CFARepeatPatternDim on {entry:?}: {r:?}"
        );
        assert!(written.untouched(), "{entry:?}");
    }

    for entry in calibration_readers {
        let fx = fixture_for(entry, &|d| {
            d.meta.as_shot_neutral = None;
            d.extra_tags.push(RawTag {
                ifd: TagIfd::Ifd0,
                ..sensor_ascii(AS_SHOT_NEUTRAL)
            });
        });
        assert!(fx.sensor.malformed_tags.contains(&AS_SHOT_NEUTRAL));
        let (r, written) = run(entry, &fx);
        assert!(
            matches!(r, Err(Error::MissingColorCalibration)),
            "AsShotNeutral on {entry:?}: {r:?}"
        );
        assert!(written.untouched(), "{entry:?}");
    }

    for entry in ENTRIES {
        let fx = fixture_for(entry, &|d| {
            d.extra_tags.push(unreadable_opcode_list(OPCODE_LIST_1));
        });
        assert!(fx.sensor.malformed_tags.contains(&OPCODE_LIST_1));
        let (r, written) = run(entry, &fx);
        assert!(
            matches!(r, Err(Error::UnreadableOpcodeList { list: 1 })),
            "OpcodeList1 on {entry:?}: {r:?}"
        );
        assert!(written.untouched(), "{entry:?}");

        // MaskedAreas: no develop reads it, so a malformed one develops,
        // byte-identical to the fixture without it.
        let fx = fixture_for(entry, &|d| d.extra_tags.push(sensor_ascii(MASKED_AREAS)));
        assert!(fx.sensor.malformed_tags.contains(&MASKED_AREAS));
        let (r, written) = run(entry, &fx);
        let (base_r, base) = run(entry, &fixture_for(entry, &|_| {}));
        assert!(base_r.is_ok(), "control {entry:?}: {base_r:?}");
        assert!(r.is_ok(), "MaskedAreas on {entry:?}: {r:?}");
        assert!(written == base, "MaskedAreas on {entry:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6: reads unaffected, and the composer's check agrees with the entries.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac6_reads_and_the_composer_check_agree() {
    let paths = [
        DevelopPath::Mono,
        DevelopPath::Colour,
        DevelopPath::Demosaic,
        DevelopPath::ColourTransform,
    ];
    for (tag, set) in TAG_SETS {
        for cfa in [false, true] {
            let mut d = base_desc(cfa);
            with_malformed(tag)(&mut d);
            // `load` asserts `sensor()` and `unpack_into` succeed.
            let fx = load(&d);
            let mut absent_d = base_desc(cfa);
            with_absent(tag)(&mut absent_d);
            assert_eq!(
                fx.plane,
                load(&absent_d).plane,
                "{tag} (cfa {cfa}): the plane read must not change"
            );
            let reads = cfa || !matches!(set, Set::Cfa);
            assert_eq!(
                fx.sensor.malformed_tags.contains(&tag),
                reads,
                "{tag} (cfa {cfa}): {:?}",
                fx.sensor.malformed_tags
            );
            for path in paths {
                let r = check_required_processing(&fx.sensor, path);
                if reads && path_sets(path).contains(&set) {
                    assert_malformed(&r, tag, &format!("{tag} (cfa {cfa}) on {path:?}"));
                } else {
                    assert!(r.is_ok(), "{tag} (cfa {cfa}) on {path:?}: {r:?}");
                }
            }
        }
    }

    // The direct call returns what the entry returns, on the entry's own
    // fixture: the same error when it refuses, and no MalformedTag otherwise.
    for (tag, _) in TAG_SETS {
        for entry in ENTRIES {
            let fx = fixture_for(entry, &with_malformed(tag));
            let check = check_required_processing(&fx.sensor, entry.path());
            let (r, _) = run(entry, &fx);
            match &check {
                Err(_) => assert_eq!(shown(&check), shown(&r), "{tag} on {entry:?}"),
                Ok(()) => assert!(
                    !matches!(r, Err(Error::MalformedTag { .. })),
                    "{tag} on {entry:?}: {r:?}"
                ),
            }
        }
    }
}

#[test]
fn malformed_tag_display_names_the_tag() {
    let msg = Error::MalformedTag { tag: WHITE_LEVEL }.to_string();
    assert!(msg.contains("WhiteLevel"), "{msg}");
    assert!(msg.contains("50717"), "{msg}");
    assert!(msg.contains("malformed"), "{msg}");
    for (tag, _) in TAG_SETS {
        let msg = Error::MalformedTag { tag }.to_string();
        assert!(
            !msg.starts_with("tag "),
            "{tag}: every DEC-078 §2 tag is named, got {msg}"
        );
    }
    let msg = Error::MalformedTag { tag: 1 }.to_string();
    assert!(msg.starts_with("tag (1)"), "{msg}");
}
