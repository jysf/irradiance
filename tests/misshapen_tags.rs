//! `SPEC-089` (`DEC-078` §7): an interpretation tag that is present but empty,
//! the wrong length, or (for the colour rationals) carries a zero denominator
//! is **malformed**, exactly as an unreadable one is. It lands in
//! `Sensor::malformed_tags`, and `DEC-078`'s existing check refuses every
//! develop path that reads it (`DEC-077`'s check refuses its own set), before
//! anything is written.
//!
//! `tests/malformed_tags.rs` (SPEC-087) proves the refusal for a tag written
//! with a type its reader rejects. This file proves the **shape** half: count
//! zero, a wrong fixed count on both sides, and a zero denominator in the last
//! entry. Every fixture goes through the real container read
//! (`Container::parse → sensor → unpack_into`) from a file written by the
//! test-only DNG writer, whose `extra_tags` replace a same-numbered entry.
//!
//! AC2 is a generated cross product: every malformed row against every entry,
//! the expectation read from the entry's path and the row's set.
//!
//! **Tier-A only**: synthetic fixtures, no corpus, no tools.

#[path = "support/dng_writer.rs"]
mod dng_writer;
#[path = "support/tiff.rs"]
mod tiff;

use irradiance::color::{
    develop_color_f32_into, develop_color_f32_into_threaded, develop_color_into,
    develop_color_into_threaded, to_srgb_linear_into,
};
use irradiance::demosaic::{demosaic_into, demosaic_into_threaded};
use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions, scratch_len,
    BlackLevels, DevelopOptions,
};
use irradiance::ifd::{Container, Sensor};
use irradiance::opcode::DevelopPath;
use irradiance::plane::unpack_into;
use irradiance::{Error, ErrorKind};

use dng_writer::{
    write_dng, DngDescription, DngMeta, IfdLayout, Organisation, Payload, PlaneKind, RawTag, TagIfd,
};
use tiff::Order;

// ─────────────────────────────────────────────────────────────────────────────
// Tag numbers (DNG 1.7.1.0; TIFF 6.0 for Orientation and SampleFormat).
// ─────────────────────────────────────────────────────────────────────────────

const ORIENTATION: u16 = 274;
const SAMPLE_FORMAT: u16 = 339;
const CFA_PATTERN: u16 = 33422;
const DNG_VERSION: u16 = 50706;
const DNG_BACKWARD_VERSION: u16 = 50707;
const CFA_PLANE_COLOR: u16 = 50710;
const CFA_LAYOUT: u16 = 50711;
const LINEARIZATION_TABLE: u16 = 50712;
const BLACK_LEVEL_REPEAT_DIM: u16 = 50713;
const BLACK_LEVEL: u16 = 50714;
const BLACK_LEVEL_DELTA_H: u16 = 50715;
const BLACK_LEVEL_DELTA_V: u16 = 50716;
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
const CAMERA_CALIBRATION_SIGNATURE: u16 = 50931;
const PROFILE_CALIBRATION_SIGNATURE: u16 = 50932;
const FORWARD_MATRIX_1: u16 = 50964;
const FORWARD_MATRIX_2: u16 = 50965;
const SUB_TILE_BLOCK_SIZE: u16 = 50974;
const ROW_INTERLEAVE_FACTOR: u16 = 50975;
const COLUMN_INTERLEAVE_FACTOR: u16 = 51157;

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;
const TYPE_SRATIONAL: u16 = 10;

/// `CalibrationIlluminant` codes: Standard A (2856 K) and D65 (6504 K).
const ILLUMINANT_A: u16 = 17;
const ILLUMINANT_D65: u16 = 21;

// ─────────────────────────────────────────────────────────────────────────────
// The sets: DEC-078 §2's three, plus DEC-077's and the version pair (which
// refuse as `UnsupportedTag` on every path).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Set {
    /// Levels and geometry.
    Levels,
    /// `CFAPlaneColor`, `CFALayout`.
    Cfa,
    /// The colour calibration tags.
    Colour,
    /// `DEC-077`'s set and the DNG version pair: `UnsupportedTag` everywhere.
    Unhonoured,
}

/// Every interpretation tag `DEC-078` §7.1(a) names, with its set.
const TAGS: [(u16, Set); 29] = [
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
    (DNG_VERSION, Set::Unhonoured),
    (DNG_BACKWARD_VERSION, Set::Unhonoured),
    (SAMPLE_FORMAT, Set::Unhonoured),
    (LINEARIZATION_TABLE, Set::Unhonoured),
    (BLACK_LEVEL_DELTA_H, Set::Unhonoured),
    (BLACK_LEVEL_DELTA_V, Set::Unhonoured),
    (SUB_TILE_BLOCK_SIZE, Set::Unhonoured),
    (ROW_INTERLEAVE_FACTOR, Set::Unhonoured),
    (COLUMN_INTERLEAVE_FACTOR, Set::Unhonoured),
];

/// The `DEC-078` sets `path` refuses on (§2's table). `Unhonoured` is on
/// every path and handled apart.
fn path_sets(path: DevelopPath) -> &'static [Set] {
    match path {
        DevelopPath::Mono => &[Set::Levels],
        DevelopPath::Colour => &[Set::Levels, Set::Cfa, Set::Colour],
        DevelopPath::Demosaic => &[Set::Cfa],
        DevelopPath::ColourTransform => &[Set::Colour],
        other => panic!("no DEC-078 row for {other:?}"),
    }
}

/// The IFD a tag lives in: the colour tags, the versions and `Orientation` in
/// IFD0 (where `Container::sensor` reads them first), the rest in the sensor
/// IFD.
fn tag_ifd(tag: u16) -> TagIfd {
    match tag {
        ORIENTATION
        | COLOR_MATRIX_1
        | COLOR_MATRIX_2
        | CAMERA_CALIBRATION_1
        | CAMERA_CALIBRATION_2
        | ANALOG_BALANCE
        | AS_SHOT_NEUTRAL
        | CALIBRATION_ILLUMINANT_1
        | CALIBRATION_ILLUMINANT_2
        | CAMERA_CALIBRATION_SIGNATURE
        | PROFILE_CALIBRATION_SIGNATURE
        | FORWARD_MATRIX_1
        | FORWARD_MATRIX_2
        | DNG_VERSION
        | DNG_BACKWARD_VERSION => TagIfd::Ifd0,
        _ => TagIfd::Sensor,
    }
}

/// The field type DNG gives `tag`, so a misshapen fixture differs from a valid
/// one only in its count or values.
fn type_of(tag: u16) -> u16 {
    match tag {
        BLACK_LEVEL | WHITE_LEVEL | DEFAULT_CROP_ORIGIN | DEFAULT_CROP_SIZE | ACTIVE_AREA => {
            TYPE_LONG
        }
        COLOR_MATRIX_1 | COLOR_MATRIX_2 | CAMERA_CALIBRATION_1 | CAMERA_CALIBRATION_2
        | FORWARD_MATRIX_1 | FORWARD_MATRIX_2 | BLACK_LEVEL_DELTA_H | BLACK_LEVEL_DELTA_V => {
            TYPE_SRATIONAL
        }
        ANALOG_BALANCE | AS_SHOT_NEUTRAL => TYPE_RATIONAL,
        CFA_PLANE_COLOR | DNG_VERSION | DNG_BACKWARD_VERSION | CFA_PATTERN => TYPE_BYTE,
        CAMERA_CALIBRATION_SIGNATURE | PROFILE_CALIBRATION_SIGNATURE => TYPE_ASCII,
        _ => TYPE_SHORT,
    }
}

// ── raw tag builders ────────────────────────────────────────────────────────

fn raw(tag: u16, field_type: u16, count: usize, bytes: Vec<u8>) -> RawTag {
    RawTag {
        ifd: tag_ifd(tag),
        tag,
        field_type,
        count: u32::try_from(count).unwrap(),
        bytes,
    }
}

fn shorts(tag: u16, values: &[u16]) -> RawTag {
    let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    raw(tag, TYPE_SHORT, values.len(), bytes)
}

fn longs(tag: u16, values: &[u32]) -> RawTag {
    let bytes = values.iter().flat_map(|v| v.to_le_bytes()).collect();
    raw(tag, TYPE_LONG, values.len(), bytes)
}

fn bytes_of(tag: u16, values: &[u8]) -> RawTag {
    raw(tag, TYPE_BYTE, values.len(), values.to_vec())
}

/// SRATIONAL or RATIONAL by `type_of(tag)`, `(num, den)` verbatim.
fn rationals(tag: u16, values: &[(i32, i32)]) -> RawTag {
    let bytes = values
        .iter()
        .flat_map(|&(n, d)| [n.to_le_bytes(), d.to_le_bytes()])
        .flatten()
        .collect();
    raw(tag, type_of(tag), values.len(), bytes)
}

/// `tag` with count zero and no payload, in its own DNG type.
fn empty(tag: u16) -> RawTag {
    raw(tag, type_of(tag), 0, Vec::new())
}

/// A 3×3 matrix as nine `num/10000` rationals, row-major.
fn matrix(rows: [[i32; 3]; 3]) -> Vec<(i32, i32)> {
    rows.iter().flatten().map(|&n| (n, 10_000)).collect()
}

/// A distinct, invertible matrix per colour tag, so a valid tag reads as a
/// value that differs from every other's.
fn matrix_for(tag: u16) -> Vec<(i32, i32)> {
    match tag {
        COLOR_MATRIX_1 => matrix([
            [9000, -2000, -500],
            [-4000, 12000, 1500],
            [-500, 1500, 6000],
        ]),
        COLOR_MATRIX_2 => matrix([
            [7000, -1500, -300],
            [-3500, 11000, 1800],
            [-300, 1200, 6500],
        ]),
        FORWARD_MATRIX_1 => matrix([[6000, 3000, 642], [2000, 7000, 1000], [1000, 1000, 6252]]),
        FORWARD_MATRIX_2 => matrix([[6500, 2500, 642], [2500, 6500, 1000], [800, 1200, 6252]]),
        CAMERA_CALIBRATION_1 => matrix([[10500, 0, 0], [0, 10000, 0], [0, 0, 9500]]),
        _ => matrix([[9800, 0, 0], [0, 10000, 0], [0, 0, 10200]]),
    }
}

/// The well-formed value of `tag`, different from its absent default.
fn valid(tag: u16) -> RawTag {
    match tag {
        ORIENTATION => shorts(tag, &[6]),
        BLACK_LEVEL_REPEAT_DIM => shorts(tag, &[2, 2]),
        BLACK_LEVEL => longs(tag, &[100]),
        WHITE_LEVEL => longs(tag, &[60_000]),
        DEFAULT_CROP_ORIGIN => longs(tag, &[1, 1]),
        DEFAULT_CROP_SIZE => longs(tag, &[6, 6]),
        ACTIVE_AREA => longs(tag, &[0, 0, 8, 8]),
        CFA_PLANE_COLOR => bytes_of(tag, &[2, 1, 0]),
        CFA_LAYOUT => shorts(tag, &[2]),
        COLOR_MATRIX_1 | COLOR_MATRIX_2 | CAMERA_CALIBRATION_1 | CAMERA_CALIBRATION_2
        | FORWARD_MATRIX_1 | FORWARD_MATRIX_2 => rationals(tag, &matrix_for(tag)),
        ANALOG_BALANCE => rationals(tag, &[(1, 1), (11, 10), (1, 1)]),
        CALIBRATION_ILLUMINANT_1 => shorts(tag, &[ILLUMINANT_A]),
        CALIBRATION_ILLUMINANT_2 => shorts(tag, &[ILLUMINANT_D65]),
        CAMERA_CALIBRATION_SIGNATURE | PROFILE_CALIBRATION_SIGNATURE => {
            raw(tag, TYPE_ASCII, 4, b"sig\0".to_vec())
        }
        DNG_VERSION => bytes_of(tag, &[1, 6, 0, 0]),
        DNG_BACKWARD_VERSION => bytes_of(tag, &[1, 3, 0, 0]),
        SAMPLE_FORMAT => shorts(tag, &[1]),
        LINEARIZATION_TABLE => shorts(tag, &[0, 1000, 2000]),
        BLACK_LEVEL_DELTA_H | BLACK_LEVEL_DELTA_V => rationals(tag, &[(0, 1); 8]),
        SUB_TILE_BLOCK_SIZE => shorts(tag, &[1, 1]),
        ROW_INTERLEAVE_FACTOR | COLUMN_INTERLEAVE_FACTOR => shorts(tag, &[1]),
        other => panic!("no valid value for {other}"),
    }
}

/// The malformed shapes of `tag`, named: count zero for every tag; for a
/// fixed-count tag, one fewer and one more; for a colour rational, a zero
/// denominator in the **last** entry (mutant (j) checks only the first).
fn shapes(tag: u16) -> Vec<(&'static str, RawTag)> {
    let mut out = vec![("count zero", empty(tag))];
    match tag {
        ORIENTATION | CFA_LAYOUT | CALIBRATION_ILLUMINANT_1 | CALIBRATION_ILLUMINANT_2 => {
            // One fewer is count zero, above.
            out.push(("count 2", shorts(tag, &[1, 1])));
        }
        COLOR_MATRIX_1 | COLOR_MATRIX_2 | CAMERA_CALIBRATION_1 | CAMERA_CALIBRATION_2
        | FORWARD_MATRIX_1 | FORWARD_MATRIX_2 => {
            let m = matrix_for(tag);
            out.push(("8 entries", rationals(tag, &m[..8])));
            let mut ten = m.clone();
            ten.push((10_000, 10_000));
            out.push(("10 entries", rationals(tag, &ten)));
            let mut zero = m;
            if let Some(last) = zero.last_mut() {
                last.1 = 0;
            }
            out.push(("zero denominator in the last entry", rationals(tag, &zero)));
        }
        ANALOG_BALANCE => {
            out.push(("2 entries", rationals(tag, &[(1, 1), (1, 1)])));
            out.push(("4 entries", rationals(tag, &[(1, 1); 4])));
            out.push((
                "zero denominator in the last entry",
                rationals(tag, &[(1, 1), (1, 1), (1, 0)]),
            ));
        }
        _ => {}
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixtures: an 8×8 16-bit plane, mono (LinearRaw) or 2×2 RGGB.
// ─────────────────────────────────────────────────────────────────────────────

const W: u32 = 8;
const H: u32 = 8;

fn samples() -> Vec<u16> {
    (0..W * H).map(|i| 1000 + (i as u16) * 37).collect()
}

fn cfa_kind(plane_color: Vec<u8>) -> PlaneKind {
    let pattern = if plane_color.len() == 4 {
        vec![0, 1, 2, 3]
    } else {
        vec![0, 1, 1, 2]
    };
    PlaneKind::Cfa {
        repeat_dim: [2, 2],
        pattern,
        plane_color,
    }
}

/// The base fixture. The CFA one carries the colour calibration the colour
/// develop needs (`ForwardMatrix1` and `AsShotNeutral`, as SPEC-087's does).
fn base_desc(cfa: bool) -> DngDescription {
    let kind = if cfa {
        cfa_kind(vec![0, 1, 2])
    } else {
        PlaneKind::LinearRaw
    };
    let mut meta = DngMeta::default();
    if cfa {
        meta.forward_matrix_1 = Some(matrix_for(FORWARD_MATRIX_1));
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

/// `load` asserts both reads succeed: AC1's "`sensor()` returns `Ok` and
/// `unpack_into` succeeds".
fn load(d: &DngDescription) -> Fixture {
    let file = write_dng(d).expect("the fixture writes");
    let container = Container::parse(&file).expect("the fixture parses");
    let sensor = container.sensor().expect("sensor() succeeds");
    let mut plane = vec![0u16; (W * H) as usize];
    unpack_into(&sensor, container.byte_order(), &file, &mut plane).expect("unpack_into succeeds");
    Fixture { sensor, plane }
}

fn fixture(cfa: bool, modify: &dyn Fn(&mut DngDescription)) -> Fixture {
    let mut d = base_desc(cfa);
    modify(&mut d);
    load(&d)
}

fn fixture_for(entry: Entry, modify: &dyn Fn(&mut DngDescription)) -> Fixture {
    fixture(!entry.mono(), modify)
}

/// What every fixture of `tag`'s row carries besides the tag, so "absent"
/// means absent and the exceptions stay out of the way:
/// - `BlackLevelRepeatDim`: four `BlackLevel` values, so DEC-078 §2's
///   one-value exception does not apply;
/// - `ForwardMatrix1`: the CFA base's own one removed;
/// - the versions: the writer's defaults removed, and for `DNGVersion` the
///   backward version too, so the version guard needs it (`DEC-078` §2).
fn row_base(tag: u16, d: &mut DngDescription) {
    match tag {
        BLACK_LEVEL_REPEAT_DIM => d.meta.black_level = Some(vec![10, 20, 30, 40]),
        FORWARD_MATRIX_1 => d.meta.forward_matrix_1 = None,
        DNG_VERSION => {
            d.meta.dng_version = None;
            d.meta.dng_backward_version = None;
        }
        DNG_BACKWARD_VERSION => d.meta.dng_backward_version = None,
        _ => {}
    }
}

fn with_tag(tag: u16, written: RawTag) -> impl Fn(&mut DngDescription) {
    move |d| {
        row_base(tag, d);
        d.extra_tags.push(written.clone());
    }
}

fn with_absent(tag: u16) -> impl Fn(&mut DngDescription) {
    move |d| row_base(tag, d)
}

/// Whether a fixture of this plane kind reads `tag` at all: the CFA tags are
/// read only on a CFA plane.
fn reads(cfa: bool, tag: u16) -> bool {
    cfa || !matches!(tag, CFA_PLANE_COLOR | CFA_LAYOUT)
}

/// The plane kinds a tag's AC1 rows run on. A colour tag's count depends on
/// ColorPlanes, so it is checked on the 3-plane CFA fixture here (the mono
/// and 4-plane cases are their own rows); the CFA set needs a CFA plane.
fn kinds_for(set: Set) -> &'static [bool] {
    match set {
        Set::Cfa | Set::Colour => &[true],
        Set::Levels | Set::Unhonoured => &[false, true],
    }
}

/// The value(s) `sensor()` reads for `tag`, as text, so a malformed row can be
/// compared with the absent control and a valid one shown to differ from it.
fn field(s: &Sensor, tag: u16) -> String {
    match tag {
        ORIENTATION => format!("{:?}", s.orientation),
        BLACK_LEVEL_REPEAT_DIM => format!("{:?}", s.black_level_repeat_dim),
        BLACK_LEVEL => format!(
            "{:?} {:?} {:?}",
            s.black_level, s.black_levels, s.black_levels_exact
        ),
        WHITE_LEVEL => format!("{:?} {:?}", s.white_level, s.white_levels),
        DEFAULT_CROP_ORIGIN => format!("{:?}", s.default_crop_origin),
        DEFAULT_CROP_SIZE => format!("{:?}", s.default_crop_size),
        ACTIVE_AREA => format!("{:?}", s.active_area),
        CFA_PLANE_COLOR => format!("{:?}", s.color_filter.as_ref().map(|c| &c.plane_color)),
        CFA_LAYOUT => format!("{:?}", s.color_filter.as_ref().map(|c| c.layout)),
        COLOR_MATRIX_1 => format!("{:?}", s.color_matrix_1),
        COLOR_MATRIX_2 => format!("{:?}", s.color_matrix_2),
        CAMERA_CALIBRATION_1 => format!("{:?}", s.camera_calibration_1),
        CAMERA_CALIBRATION_2 => format!("{:?}", s.camera_calibration_2),
        ANALOG_BALANCE => format!("{:?}", s.analog_balance),
        CALIBRATION_ILLUMINANT_1 => format!("{:?}", s.calibration_illuminant_1),
        CALIBRATION_ILLUMINANT_2 => format!("{:?}", s.calibration_illuminant_2),
        CAMERA_CALIBRATION_SIGNATURE => format!("{:?}", s.camera_calibration_signature),
        PROFILE_CALIBRATION_SIGNATURE => format!("{:?}", s.profile_calibration_signature),
        FORWARD_MATRIX_1 => format!("{:?}", s.forward_matrix_1),
        FORWARD_MATRIX_2 => format!("{:?}", s.forward_matrix_2),
        DNG_VERSION => format!("{:?}", s.dng_version),
        DNG_BACKWARD_VERSION => format!("{:?}", s.dng_backward_version),
        SAMPLE_FORMAT => format!("{:?}", s.sample_format),
        LINEARIZATION_TABLE => format!("{:?}", s.linearization_table),
        BLACK_LEVEL_DELTA_H => format!("{:?}", s.black_level_delta_h),
        BLACK_LEVEL_DELTA_V => format!("{:?}", s.black_level_delta_v),
        SUB_TILE_BLOCK_SIZE => format!("{:?}", s.sub_tile_block_size),
        ROW_INTERLEAVE_FACTOR => format!("{:?}", s.row_interleave_factor),
        COLUMN_INTERLEAVE_FACTOR => format!("{:?}", s.column_interleave_factor),
        other => panic!("no field for {other}"),
    }
}

fn times_recorded(s: &Sensor, tag: u16) -> usize {
    s.malformed_tags.iter().filter(|&&t| t == tag).count()
}

// ─────────────────────────────────────────────────────────────────────────────
// The ten develop entries.
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
}

const ENTRIES: [Entry; 10] = [
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
];

/// The four colour develop entries and the colour transform: AC3's set.
const COLOUR_READERS: [Entry; 5] = [
    Entry::DevelopColorInto,
    Entry::DevelopColorF32Into,
    Entry::DevelopColorIntoThreaded,
    Entry::DevelopColorF32IntoThreaded,
    Entry::ToSrgbLinearInto,
];

impl Entry {
    fn mono(self) -> bool {
        self.path() == DevelopPath::Mono
    }

    fn path(self) -> DevelopPath {
        match self {
            Entry::DevelopInto | Entry::DevelopWithInto | Entry::DevelopWithScratchInto => {
                DevelopPath::Mono
            }
            Entry::DevelopColorInto
            | Entry::DevelopColorF32Into
            | Entry::DevelopColorIntoThreaded
            | Entry::DevelopColorF32IntoThreaded => DevelopPath::Colour,
            Entry::ToSrgbLinearInto => DevelopPath::ColourTransform,
            Entry::DemosaicInto | Entry::DemosaicIntoThreaded => DevelopPath::Demosaic,
        }
    }

    fn takes_options(self) -> bool {
        matches!(self, Entry::DevelopWithInto | Entry::DevelopWithScratchInto)
    }
}

const SENTINEL_U16: u16 = 0xA5A5;
const SENTINEL_F32: f32 = -12345.5;

/// Everything an entry wrote: `dst` (and the caller's scratch, after it).
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
            let camera: Vec<u16> = (0..raw3).map(|i| (i % 4096) as u16).collect();
            let mut dst = u16s(raw3);
            let r = to_srgb_linear_into(sensor, &camera, &mut dst);
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

fn options_for(entry: Entry) -> Vec<DevelopOptions> {
    if entry.takes_options() {
        [BlackLevels::FirstValue, BlackLevels::PerPosition]
            .map(|black_levels| {
                let mut options = DevelopOptions::default();
                options.black_levels = black_levels;
                options
            })
            .to_vec()
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

fn shown(r: &Result<(), Error>) -> String {
    format!("{r:?}")
}

// Shown red first: HANDOFF-185's handback records which of these failed on
// main's behaviour (48a8310) before src/ifd.rs and src/color.rs changed.

// ─────────────────────────────────────────────────────────────────────────────
// AC1: the reader records each shape.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac1_the_reader_records_each_empty_or_misshapen_tag() {
    let mut rows = 0;
    for (tag, set) in TAGS {
        for &cfa in kinds_for(set) {
            let absent = fixture(cfa, &with_absent(tag));
            assert_eq!(times_recorded(&absent.sensor, tag), 0, "{tag} absent");
            for (shape, written) in shapes(tag) {
                let context = format!("{tag} {shape} (cfa {cfa})");
                let fx = fixture(cfa, &with_tag(tag, written));
                assert_eq!(
                    times_recorded(&fx.sensor, tag),
                    1,
                    "{context}: recorded exactly once, malformed_tags {:?}",
                    fx.sensor.malformed_tags
                );
                assert_eq!(
                    field(&fx.sensor, tag),
                    field(&absent.sensor, tag),
                    "{context}: the field must be its absent value"
                );
                assert_eq!(fx.plane, absent.plane, "{context}: the plane read");
                rows += 1;
            }
        }
    }
    // Count zero on every tag (levels and unhonoured twice, on both planes),
    // count 2 on the four single-value tags (Orientation on both planes), three shapes on each of the six
    // matrices and on AnalogBalance: the table is not vacuous.
    assert_eq!(rows, (7 + 9) * 2 + 13 + 5 + 3 * 7);
}

/// DEC-078 §7.1(b)'s ColorPlanes: the `CFAPlaneColor` count on a CFA plane
/// (3 when absent or malformed), `SamplesPerPixel` otherwise. The 4-plane row
/// is mutant (i)'s distinguishing fixture: fixed at 3, a 9-entry matrix passes.
#[test]
fn ac1_colour_planes_come_from_the_plane() {
    let cm = |n: usize| -> RawTag {
        let mut values = matrix_for(COLOR_MATRIX_1);
        values.resize(n, (1000, 10_000));
        rationals(COLOR_MATRIX_1, &values)
    };
    let four_planes = |d: &mut DngDescription| d.kind = cfa_kind(vec![0, 1, 2, 3]);
    /// (name, fixture change, CFA plane, ColorMatrix1 recorded).
    type Case = (&'static str, Box<dyn Fn(&mut DngDescription)>, bool, bool);
    let cases: Vec<Case> = vec![
        (
            "4 planes, 9 entries",
            Box::new(move |d: &mut DngDescription| {
                four_planes(d);
                d.extra_tags.push(cm(9));
            }),
            true,
            true,
        ),
        (
            "4 planes, 12 entries",
            Box::new(move |d: &mut DngDescription| {
                four_planes(d);
                d.extra_tags.push(cm(12));
            }),
            true,
            false,
        ),
        (
            "3 planes, 12 entries",
            Box::new(move |d: &mut DngDescription| d.extra_tags.push(cm(12))),
            true,
            true,
        ),
        (
            "empty CFAPlaneColor, 9 entries",
            Box::new(move |d: &mut DngDescription| {
                d.extra_tags.push(empty(CFA_PLANE_COLOR));
                d.extra_tags.push(cm(9));
            }),
            true,
            false,
        ),
        (
            "mono, 9 entries",
            Box::new(move |d: &mut DngDescription| d.extra_tags.push(cm(9))),
            false,
            true,
        ),
        (
            "mono, 3 entries",
            Box::new(move |d: &mut DngDescription| d.extra_tags.push(cm(3))),
            false,
            false,
        ),
    ];
    for (name, modify, cfa, recorded) in cases {
        let fx = fixture(cfa, &*modify);
        assert_eq!(
            times_recorded(&fx.sensor, COLOR_MATRIX_1),
            usize::from(recorded),
            "{name}: malformed_tags {:?}",
            fx.sensor.malformed_tags
        );
        assert_eq!(fx.sensor.color_matrix_1.is_some(), !recorded, "{name}");
    }
}

#[test]
fn ac1_valid_and_absent_controls_are_not_recorded() {
    for (tag, set) in TAGS {
        for &cfa in kinds_for(set) {
            let context = format!("{tag} (cfa {cfa})");
            let absent = fixture(cfa, &with_absent(tag));
            let fx = fixture(cfa, &with_tag(tag, valid(tag)));
            assert_eq!(times_recorded(&absent.sensor, tag), 0, "{context} absent");
            assert_eq!(
                times_recorded(&fx.sensor, tag),
                0,
                "{context} valid: malformed_tags {:?}",
                fx.sensor.malformed_tags
            );
            assert_ne!(
                field(&fx.sensor, tag),
                field(&absent.sensor, tag),
                "{context}: the valid tag must be read"
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC2: each path refuses a misshapen tag it reads, buffers untouched.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac2_each_path_refuses_a_misshapen_tag_it_reads() {
    let (mut refused, mut as_absent) = (0, 0);
    for (tag, set) in TAGS {
        for (shape, written) in shapes(tag) {
            for entry in ENTRIES {
                let context = format!("{tag} {shape} on {entry:?}");
                let fx = fixture_for(entry, &with_tag(tag, written.clone()));
                let reads = reads(!entry.mono(), tag);
                for options in options_for(entry) {
                    let (r, out) = run_with(entry, &fx, &options);
                    if set == Set::Unhonoured {
                        assert!(
                            matches!(r, Err(Error::UnsupportedTag { tag: got }) if got == tag),
                            "{context}: expected UnsupportedTag {{ tag: {tag} }}, got {r:?}"
                        );
                        assert!(out.untouched(), "{context}: dst untouched");
                        refused += 1;
                    } else if reads && path_sets(entry.path()).contains(&set) {
                        assert_malformed(&r, tag, &context);
                        assert!(out.untouched(), "{context}: dst and scratch untouched");
                        refused += 1;
                    } else {
                        let absent = fixture_for(entry, &with_absent(tag));
                        let (absent_r, absent_out) = run_with(entry, &absent, &options);
                        assert_eq!(shown(&r), shown(&absent_r), "{context}: as absent");
                        assert!(out == absent_out, "{context}: output as absent");
                        as_absent += 1;
                    }
                }
            }
        }
    }
    assert!(refused > 300 && as_absent > 100, "{refused} / {as_absent}");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC3: the review's two findings, end to end, on a dual-illuminant fixture.
// ─────────────────────────────────────────────────────────────────────────────

/// A CFA fixture that takes the dual-illuminant path: `ColorMatrix1` and `2`,
/// Standard A and D65, no `ForwardMatrix`.
fn dual(d: &mut DngDescription) {
    d.meta.forward_matrix_1 = None;
    d.meta.color_matrix_1 = Some(matrix_for(COLOR_MATRIX_1));
    d.meta.color_matrix_2 = Some(matrix_for(COLOR_MATRIX_2));
    d.meta.calibration_illuminant_1 = Some(ILLUMINANT_A);
    d.meta.calibration_illuminant_2 = Some(ILLUMINANT_D65);
    d.meta.as_shot_neutral = Some(vec![(1, 2), (1, 1), (2, 3)]);
}

/// The three-way check: the valid tag changes the output (the fixture uses
/// it), and each misshapen form refuses with its tag on every colour reader,
/// dst untouched.
fn three_way(
    tag: u16,
    base: &dyn Fn(&mut DngDescription),
    absent: &dyn Fn(&mut DngDescription),
    misshapen: &[(&str, RawTag)],
) {
    let valid_fx = fixture(true, &|d: &mut DngDescription| {
        base(d);
        d.extra_tags.push(valid(tag));
    });
    let absent_fx = fixture(true, &|d: &mut DngDescription| {
        base(d);
        absent(d);
    });
    let (valid_r, valid_out) = run(Entry::DevelopColorInto, &valid_fx);
    let (absent_r, absent_out) = run(Entry::DevelopColorInto, &absent_fx);
    assert!(
        valid_r.is_ok() && absent_r.is_ok(),
        "{valid_r:?} {absent_r:?}"
    );
    assert!(
        valid_out != absent_out,
        "{tag}: the valid tag must change the output, or the fixture does not use it"
    );
    for (shape, written) in misshapen {
        let fx = fixture(true, &|d: &mut DngDescription| {
            base(d);
            d.extra_tags.push(written.clone());
        });
        for entry in COLOUR_READERS {
            let context = format!("{tag} {shape} on {entry:?}");
            let (r, out) = run(entry, &fx);
            assert_malformed(&r, tag, &context);
            assert!(out.untouched(), "{context}: dst untouched");
        }
    }
}

#[test]
fn ac3_a_misshapen_second_colour_matrix_refuses_instead_of_falling_back() {
    let shaped = shapes(COLOR_MATRIX_2);
    let misshapen: Vec<_> = shaped
        .into_iter()
        .filter(|(s, _)| *s == "8 entries" || s.starts_with("zero denominator"))
        .collect();
    assert_eq!(misshapen.len(), 2);
    three_way(
        COLOR_MATRIX_2,
        &dual,
        &|d| d.meta.color_matrix_2 = None,
        &misshapen,
    );

    // A count-zero CalibrationIlluminant2 refuses too, where it silently took
    // the single-illuminant path.
    three_way(
        CALIBRATION_ILLUMINANT_2,
        &|d| {
            dual(d);
            d.meta.calibration_illuminant_2 = None;
        },
        &|_| {},
        &[("count zero", empty(CALIBRATION_ILLUMINANT_2))],
    );
}

#[test]
fn ac3_a_misshapen_second_forward_matrix_refuses_instead_of_switching_path() {
    let with_fm1 = |d: &mut DngDescription| {
        dual(d);
        d.meta.forward_matrix_1 = Some(matrix_for(FORWARD_MATRIX_1));
    };
    let misshapen: Vec<_> = shapes(FORWARD_MATRIX_2)
        .into_iter()
        .filter(|(s, _)| *s != "count zero")
        .collect();
    three_way(FORWARD_MATRIX_2, &with_fm1, &|_| {}, &misshapen);
    three_way(
        FORWARD_MATRIX_2,
        &with_fm1,
        &|_| {},
        &[("count zero", empty(FORWARD_MATRIX_2))],
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC5: Orientation's recovery from the other IFD.
// ─────────────────────────────────────────────────────────────────────────────

fn sensor_orientation(value: u16) -> RawTag {
    RawTag {
        ifd: TagIfd::Sensor,
        ..shorts(ORIENTATION, &[value])
    }
}

#[test]
fn ac5_an_empty_orientation_is_recovered_from_the_other_ifd() {
    let empty_ifd0_valid_sensor = |d: &mut DngDescription| {
        d.extra_tags.push(empty(ORIENTATION));
        d.extra_tags.push(sensor_orientation(6));
    };
    let sensor_only = |d: &mut DngDescription| d.extra_tags.push(sensor_orientation(6));
    let empty_both = |d: &mut DngDescription| {
        d.extra_tags.push(empty(ORIENTATION));
        d.extra_tags.push(RawTag {
            ifd: TagIfd::Sensor,
            ..empty(ORIENTATION)
        });
    };
    for entry in ENTRIES {
        let fx = fixture_for(entry, &empty_ifd0_valid_sensor);
        assert_eq!(times_recorded(&fx.sensor, ORIENTATION), 0, "{entry:?}");
        assert_eq!(fx.sensor.orientation, Some(6), "{entry:?}");
        let (r, out) = run(entry, &fx);
        let (control_r, control) = run(entry, &fixture_for(entry, &sensor_only));
        assert_eq!(shown(&r), shown(&control_r), "{entry:?}");
        assert!(
            out == control,
            "{entry:?}: develops with the sensor IFD's value"
        );
        if entry.mono() {
            assert!(r.is_ok(), "{entry:?}: {r:?}");
            let (_, unrotated) = run(entry, &fixture_for(entry, &|_| {}));
            assert!(out != unrotated, "{entry:?}: orientation 6 must show");
        }

        let fx = fixture_for(entry, &empty_both);
        assert_eq!(times_recorded(&fx.sensor, ORIENTATION), 1, "{entry:?}");
        assert_eq!(fx.sensor.orientation, None);
        let (r, out) = run(entry, &fx);
        if path_sets(entry.path()).contains(&Set::Levels) {
            assert_malformed(&r, ORIENTATION, &format!("empty in both on {entry:?}"));
            assert!(out.untouched(), "{entry:?}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC6: the exceptions hold.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac6_the_k3iii_exception_and_the_already_refused_tags_hold() {
    // The K-3 III Monochrome's shape: one BlackLevel, BlackLevelRepeatDim of
    // count 1. And a count-zero BlackLevelRepeatDim with one black value.
    let one_black = |d: &mut DngDescription| d.meta.black_level = Some(vec![100]);
    let k3iii = |d: &mut DngDescription| {
        one_black(d);
        d.extra_tags.push(shorts(BLACK_LEVEL_REPEAT_DIM, &[2]));
    };
    let empty_repeat_dim = |d: &mut DngDescription| {
        one_black(d);
        d.extra_tags.push(empty(BLACK_LEVEL_REPEAT_DIM));
    };
    for entry in ENTRIES
        .into_iter()
        .filter(|e| path_sets(e.path()).contains(&Set::Levels))
    {
        let control = fixture_for(entry, &one_black);
        for modify in [&k3iii as &dyn Fn(&mut DngDescription), &empty_repeat_dim] {
            let fx = fixture_for(entry, modify);
            assert_eq!(times_recorded(&fx.sensor, BLACK_LEVEL_REPEAT_DIM), 1);
            for options in options_for(entry) {
                let context = format!("{entry:?} under {:?}", options.black_levels);
                let (r, out) = run_with(entry, &fx, &options);
                let (control_r, control_out) = run_with(entry, &control, &options);
                assert!(control_r.is_ok(), "{context}: control {control_r:?}");
                assert!(r.is_ok(), "{context}: must develop, got {r:?}");
                assert!(
                    out == control_out,
                    "{context}: byte-identical to the control"
                );
            }
        }
    }

    // AsShotNeutral keeps MissingColorCalibration; CFAPattern keeps its own
    // CFA error. Neither is in a DEC-078 set.
    for entry in COLOUR_READERS {
        let fx = fixture_for(entry, &|d| {
            d.meta.as_shot_neutral = None;
            d.extra_tags.push(empty(AS_SHOT_NEUTRAL));
        });
        let (r, out) = run(entry, &fx);
        assert!(
            matches!(r, Err(Error::MissingColorCalibration)),
            "AsShotNeutral on {entry:?}: {r:?}"
        );
        assert!(out.untouched(), "{entry:?}");
    }
    for entry in ENTRIES
        .into_iter()
        .filter(|e| matches!(e.path(), DevelopPath::Colour | DevelopPath::Demosaic))
    {
        let fx = fixture_for(entry, &|d| {
            d.extra_tags.push(RawTag {
                ifd: TagIfd::Sensor,
                ..empty(CFA_PATTERN)
            });
        });
        let (r, out) = run(entry, &fx);
        assert!(
            matches!(
                r,
                Err(Error::UnsupportedCfaPattern { .. } | Error::NotAColorFilterArray)
            ),
            "CFAPattern on {entry:?}: {r:?}"
        );
        assert!(out.untouched(), "{entry:?}");
    }

    // An unknown CalibrationIlluminant code still falls back to the single
    // illuminant (audit rank 8, out of scope): byte-identical to the second
    // illuminant absent.
    let unknown = fixture(true, &|d| {
        dual(d);
        d.meta.calibration_illuminant_2 = Some(99);
    });
    let single = fixture(true, &|d| {
        dual(d);
        d.meta.calibration_illuminant_2 = None;
    });
    for entry in COLOUR_READERS {
        let (r, out) = run(entry, &unknown);
        let (single_r, single_out) = run(entry, &single);
        assert!(
            r.is_ok() && single_r.is_ok(),
            "{entry:?}: {r:?} {single_r:?}"
        );
        assert!(out == single_out, "{entry:?}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC7: the changed refusals are MalformedTag.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn ac7_the_changed_refusals_are_malformed_tag() {
    type Case = (u16, Box<dyn Fn(&mut DngDescription)>);
    let cases: Vec<Case> = vec![
        // Count-zero ColorMatrix1 with no ForwardMatrix1: it was the matrix
        // the develop used, refused late as MissingColorCalibration.
        (
            COLOR_MATRIX_1,
            Box::new(|d: &mut DngDescription| {
                d.meta.forward_matrix_1 = None;
                d.extra_tags.push(empty(COLOR_MATRIX_1));
            }),
        ),
        (
            CAMERA_CALIBRATION_1,
            Box::new(|d: &mut DngDescription| {
                d.extra_tags.push(rationals(
                    CAMERA_CALIBRATION_1,
                    &matrix_for(CAMERA_CALIBRATION_1)[..8],
                ));
            }),
        ),
        (
            ANALOG_BALANCE,
            Box::new(|d: &mut DngDescription| {
                d.extra_tags
                    .push(rationals(ANALOG_BALANCE, &[(1, 1), (1, 1)]))
            }),
        ),
        // Count-zero CFAPlaneColor: was UnsupportedCfaPlaneColor { [] }.
        (
            CFA_PLANE_COLOR,
            Box::new(|d: &mut DngDescription| d.extra_tags.push(empty(CFA_PLANE_COLOR))),
        ),
    ];
    for (tag, modify) in cases {
        for entry in ENTRIES {
            let refuses = !entry.mono()
                && (tag == CFA_PLANE_COLOR && entry.path() != DevelopPath::ColourTransform
                    || tag != CFA_PLANE_COLOR && entry.path() != DevelopPath::Demosaic);
            if !refuses {
                continue;
            }
            let fx = fixture_for(entry, &*modify);
            let (r, out) = run(entry, &fx);
            assert_malformed(&r, tag, &format!("{tag} on {entry:?}"));
            assert!(out.untouched(), "{tag} on {entry:?}");
        }
    }
}
