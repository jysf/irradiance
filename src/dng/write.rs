//! The mono `u16` DNG writer (`SPEC-061`): one monochrome sensor plane,
//! unchanged, as an uncompressed little-endian DNG in a caller-sized buffer.
//!
//! # Use
//!
//! Ask for the size, allocate, write:
//!
//! ```
//! use irradiance::dng::write::{write_into, written_len, Framing, WriteOptions};
//!
//! let (width, height) = (4, 2);
//! let samples = [0u16, 1, 2, 3, 4, 5, 6, 7];
//! let mut options = WriteOptions::default();
//! options.framing = Framing::FullSensor;
//! options.unique_camera_model = Some("Example Monochrome");
//! let max_pixels = 1 << 28;
//!
//! let len = written_len(&samples, width, height, &options, max_pixels)?;
//! let mut file = vec![0u8; len];
//! write_into(&samples, width, height, &options, max_pixels, &mut file)?;
//! assert_eq!(&file[..4], b"II*\0");
//! # Ok::<(), irradiance::Error>(())
//! ```
//!
//! [`write_to_vec`] does both steps with one allocation.
//!
//! # What is written
//!
//! The raw plane is IFD0 (`NewSubFileType` 0), stored as one strip of
//! little-endian `u16` samples (`DEC-075` §1, §2). `DNGVersion` is 1.4.0.0;
//! `DNGBackwardVersion` is 1.3.0.0 when any opcode list is written, otherwise
//! 1.1.0.0 (§3). `Orientation`, `WhiteLevel`, `DefaultCropOrigin`/`Size` and
//! `UniqueCameraModel` are always written; `BlackLevel`,
//! `BlackLevelRepeatDim`, `ActiveArea`, `MaskedAreas` and the three opcode
//! lists when the options carry them and the framing keeps them.
//!
//! Framing (`DEC-073`) is expressed as tags. Samples are cut only at the
//! `ActiveArea` edge and never rotated:
//!
//! | [`Framing`] | Samples | `ActiveArea` | `DefaultCrop` | `MaskedAreas` |
//! |---|---|---|---|---|
//! | `FullSensor` | the whole plane | when given | the whole active area | when given |
//! | `FullActiveArea` | the active area | omitted | the whole active area | refused |
//! | `StandardCrop` | the active area | omitted | the caller's, or the whole active area | refused |
//!
//! Opcode lists are copied verbatim, never applied or edited. Under the two
//! framings that cut, `OpcodeList1` (which works on stored-plane coordinates)
//! is accepted only when every opcode in it is position-independent: today
//! only `FixBadPixelsConstant` (id 4, `DEC-075` §5).
//!
//! # Checks and their order
//!
//! [`written_len`] and [`write_into`] run the same checks in the same order
//! and return the same error for the same input; [`write_into`] then also
//! checks `dst`'s length. The order is: the pixel budget
//! ([`Error::PlaneTooLarge`], before any size arithmetic), the sample count,
//! the geometry (active area, default crop, masked areas), the levels, the
//! orientation, `UniqueCameraModel`, the opcode lists, the output size. Every
//! refusal is [`Error::InvalidWriteInput`] with a [`WriteInputError`] reason
//! (`ErrorKind::CallerError`), except an `OpcodeList1` opcode a cut would
//! shift, which is [`Error::UnsupportedOpcodeForFraming`]
//! (`ErrorKind::Unsupported`).
//!
//! # Allocation and determinism
//!
//! [`written_len`] and [`write_into`] allocate nothing (`DEC-075` §9).
//! [`write_into`] writes every byte of `dst`, so whatever the buffer held
//! never reaches the file, and the same input always gives the same bytes.
//!
//! **Provenance** (`docs/provenance-ledger.md`): TIFF 6.0 §2 and §8, the Adobe
//! DNG Specification 1.7.1 (tag set and placement, the opcode-list header
//! layout, `DNGBackwardVersion` 1.3 for opcode lists) and `DEC-008` in the
//! writing direction; the layout follows this repository's own test writer
//! (`tests/support/dng_writer.rs`, `SPEC-057`). No writer implementation was
//! read.

use crate::ifd::{
    ActiveArea, DefaultCropOrigin, DefaultCropSize, TAG_ACTIVE_AREA, TAG_BITS_PER_SAMPLE,
    TAG_BLACK_LEVEL, TAG_BLACK_LEVEL_REPEAT_DIM, TAG_COMPRESSION, TAG_DEFAULT_CROP_ORIGIN,
    TAG_DEFAULT_CROP_SIZE, TAG_DNG_BACKWARD_VERSION, TAG_DNG_VERSION, TAG_IMAGE_LENGTH,
    TAG_IMAGE_WIDTH, TAG_MASKED_AREAS, TAG_NEW_SUBFILE_TYPE, TAG_OPCODE_LIST_1, TAG_OPCODE_LIST_2,
    TAG_OPCODE_LIST_3, TAG_ORIENTATION, TAG_PHOTOMETRIC, TAG_ROWS_PER_STRIP, TAG_SAMPLES_PER_PIXEL,
    TAG_STRIP_BYTE_COUNTS, TAG_STRIP_OFFSETS, TAG_WHITE_LEVEL,
};
use crate::Error;

pub use crate::write_types::{Framing, WriteInputError};

// ─────────────────────────────────────────────────────────────────────────────
// The options
// ─────────────────────────────────────────────────────────────────────────────

/// What to write beside the samples (`SPEC-061` §2).
///
/// `#[non_exhaustive]` (`DEC-052`): later writer specs add fields (the EXIF
/// block, XMP, descriptive tags, rendering hints, a preview). Start from
/// [`WriteOptions::default`] and assign the fields you need.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct WriteOptions<'a> {
    /// Which samples to store and how to frame them. Default
    /// [`Framing::StandardCrop`].
    pub framing: Framing,
    /// The active area, in stored-plane coordinates. `None` means the whole
    /// plane is active.
    pub active_area: Option<ActiveArea>,
    /// The default crop, relative to the active area (as DNG stores it), so
    /// it carries over a cut unchanged. `None` means the whole active area.
    /// Written only under [`Framing::StandardCrop`]; always validated.
    pub default_crop: Option<(DefaultCropOrigin, DefaultCropSize)>,
    /// TIFF `Orientation`, 1 to 8. Default 1. Always written (`DEC-075` §4).
    pub orientation: u16,
    /// `BlackLevel`, one value per repeat position, row-major. Empty means the
    /// tag is omitted.
    pub black_level: &'a [u32],
    /// `BlackLevelRepeatDim` `[rows, cols]`. `None` means the tag is omitted
    /// (DNG's default is 1 × 1).
    pub black_level_repeat_dim: Option<[u16; 2]>,
    /// `WhiteLevel`. Default 65535, the DNG default for 16-bit samples.
    /// Always written.
    pub white_level: u32,
    /// `MaskedAreas`, in stored-plane coordinates. Only
    /// [`Framing::FullSensor`] keeps the margin they describe, so a non-empty
    /// list under another framing is refused (`DEC-075` §6).
    pub masked_areas: &'a [ActiveArea],
    /// `OpcodeList1`'s bytes, written verbatim.
    pub opcode_list_1: Option<&'a [u8]>,
    /// `OpcodeList2`'s bytes, written verbatim.
    pub opcode_list_2: Option<&'a [u8]>,
    /// `OpcodeList3`'s bytes, written verbatim.
    pub opcode_list_3: Option<&'a [u8]>,
    /// `UniqueCameraModel`. Required (`DEC-071` §3a): printable ASCII, 1 to
    /// 1024 bytes, not only spaces.
    pub unique_camera_model: Option<&'a str>,
}

impl Default for WriteOptions<'_> {
    fn default() -> Self {
        WriteOptions {
            framing: Framing::default(),
            active_area: None,
            default_crop: None,
            orientation: 1,
            black_level: &[],
            black_level_repeat_dim: None,
            white_level: 65535,
            masked_areas: &[],
            opcode_list_1: None,
            opcode_list_2: None,
            opcode_list_3: None,
            unique_camera_model: None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// The entry points
// ─────────────────────────────────────────────────────────────────────────────

/// The length in bytes of the DNG [`write_into`] would write for this input.
///
/// `samples` is the whole stored plane, row-major, `width × height`: exactly
/// what [`crate::plane::unpack_into`] returns. The writer cuts it; the caller
/// never does. `max_pixels` is the caller's budget for `width × height`, as
/// [`crate::plane::validate`] takes it (`DEC-050`).
///
/// Allocates nothing.
///
/// # Errors
///
/// Every error [`write_into`] returns except the `dst` length, in the same
/// order (see the [module documentation](self)).
pub fn written_len(
    samples: &[u16],
    width: u32,
    height: u32,
    options: &WriteOptions<'_>,
    max_pixels: usize,
) -> Result<usize, Error> {
    plan(samples, width, height, options, max_pixels).map(|p| p.total)
}

/// Write the DNG into `dst`, which must be exactly [`written_len`] bytes.
///
/// Writes every byte of `dst` and allocates nothing.
///
/// # Errors
///
/// - [`Error::PlaneTooLarge`] when `width × height` exceeds `max_pixels`.
/// - [`Error::InvalidWriteInput`] for any caller-input problem, its
///   [`WriteInputError`] naming the reason, including a `dst` of the wrong
///   length ([`WriteInputError::OutputBufferWrongLength`], checked last).
/// - [`Error::UnsupportedOpcodeForFraming`] when the framing cuts the margin
///   and `OpcodeList1` holds an opcode whose coordinates the cut would shift.
pub fn write_into(
    samples: &[u16],
    width: u32,
    height: u32,
    options: &WriteOptions<'_>,
    max_pixels: usize,
    dst: &mut [u8],
) -> Result<(), Error> {
    let plan = plan(samples, width, height, options, max_pixels)?;
    if dst.len() != plan.total {
        return Err(invalid(WriteInputError::OutputBufferWrongLength {
            expected: plan.total,
            actual: dst.len(),
        }));
    }
    serialize(&plan, samples, dst)
}

/// [`written_len`] then [`write_into`] into one `Vec` of exactly that length.
///
/// # Errors
///
/// As [`write_into`], except that the buffer is always the right length.
pub fn write_to_vec(
    samples: &[u16],
    width: u32,
    height: u32,
    options: &WriteOptions<'_>,
    max_pixels: usize,
) -> Result<Vec<u8>, Error> {
    let plan = plan(samples, width, height, options, max_pixels)?;
    let mut out = vec![0u8; plan.total];
    serialize(&plan, samples, &mut out)?;
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// TIFF 6.0 `PlanarConfiguration`.
const TAG_PLANAR_CONFIGURATION: u16 = 284;
/// DNG `UniqueCameraModel`.
const TAG_UNIQUE_CAMERA_MODEL: u16 = 50708;

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_UNDEFINED: u16 = 7;

/// DNG `LinearRaw`.
const PHOTOMETRIC_LINEAR_RAW: u16 = 34892;

const DNG_VERSION: [u8; 4] = [1, 4, 0, 0];
/// Opcode lists are a DNG 1.3 feature (`DEC-075` §3).
const BACKWARD_WITH_OPCODES: [u8; 4] = [1, 3, 0, 0];
const BACKWARD_WITHOUT_OPCODES: [u8; 4] = [1, 1, 0, 0];

/// The longest ASCII field the writer accepts, in bytes before the NUL.
const MAX_ASCII_LEN: usize = 1024;

/// `OpcodeList1` opcodes that name no coordinates, so a cut cannot move them
/// (`DEC-075` §5): `FixBadPixelsConstant`. Widening it needs a DEC.
const LIST_1_POSITION_INDEPENDENT: [u32; 1] = [4];

/// The header: `II`, 42, and the IFD0 offset.
const HEADER_LEN: u64 = 8;
/// IFD0 sits right after the header in this spec. SPEC-081 moves it after the
/// EXIF block (`DEC-072`), which is why the header's IFD0 offset is a planned
/// value and not hard-coded to the header's end.
const IFD0_OFFSET: u64 = HEADER_LEN;

// ─────────────────────────────────────────────────────────────────────────────
// Field values
// ─────────────────────────────────────────────────────────────────────────────

/// One IFD entry's value, borrowed from the caller where it is a slice. Its
/// bytes are produced at serialisation, so planning allocates nothing.
#[derive(Debug, Clone, Copy)]
enum Value<'a> {
    Short(u16),
    Shorts2([u16; 2]),
    Long(u32),
    Longs2([u32; 2]),
    Longs4([u32; 4]),
    Longs(&'a [u32]),
    /// Each rectangle as four LONGs: top, left, bottom, right.
    Rects(&'a [ActiveArea]),
    Bytes4([u8; 4]),
    /// Written with a terminating NUL, which the count includes.
    Ascii(&'a [u8]),
    Undefined(&'a [u8]),
    /// The strip's offset, a LONG known only after layout.
    StripOffset,
}

impl Value<'_> {
    fn field_type(&self) -> u16 {
        match self {
            Value::Short(_) | Value::Shorts2(_) => TYPE_SHORT,
            Value::Long(_)
            | Value::Longs2(_)
            | Value::Longs4(_)
            | Value::Longs(_)
            | Value::Rects(_)
            | Value::StripOffset => TYPE_LONG,
            Value::Bytes4(_) => TYPE_BYTE,
            Value::Ascii(_) => TYPE_ASCII,
            Value::Undefined(_) => TYPE_UNDEFINED,
        }
    }

    /// The TIFF count: values of the field's type.
    fn count(&self) -> Option<u64> {
        let len = |n: usize| u64::try_from(n).ok();
        match self {
            Value::Short(_) | Value::Long(_) | Value::StripOffset => Some(1),
            Value::Shorts2(_) | Value::Longs2(_) => Some(2),
            Value::Longs4(_) | Value::Bytes4(_) => Some(4),
            Value::Longs(v) => len(v.len()),
            Value::Rects(v) => len(v.len())?.checked_mul(4),
            Value::Ascii(v) => len(v.len())?.checked_add(1),
            Value::Undefined(v) => len(v.len()),
        }
    }

    /// The value's size in bytes.
    fn byte_len(&self) -> Option<u64> {
        let unit: u64 = match self.field_type() {
            TYPE_SHORT => 2,
            TYPE_LONG => 4,
            _ => 1,
        };
        self.count()?.checked_mul(unit)
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry<'a> {
    tag: u16,
    value: Value<'a>,
}

/// One slot per tag the writer can emit, in ascending tag order, so the IFD
/// is a fixed-size array (no allocation) and sorted by construction.
const SLOTS: usize = 25;

/// Everything [`serialize`] needs, decided and checked by [`plan`].
#[derive(Debug)]
struct Plan<'a> {
    entries: [Option<Entry<'a>>; SLOTS],
    /// The offset of each slot's out-of-line value; 0 for an inline one.
    value_offsets: [u32; SLOTS],
    entry_count: u16,
    strip_offset: u32,
    /// The stored-plane rectangle written, as rows `top..bottom` and columns
    /// `left..right`.
    cut: ActiveArea,
    /// The stored plane's width, for walking `samples` by row.
    plane_width: u32,
    /// The file's total length.
    total: usize,
}

fn invalid(reason: WriteInputError) -> Error {
    Error::InvalidWriteInput(reason)
}

fn too_large() -> Error {
    invalid(WriteInputError::OutputTooLarge)
}

// ─────────────────────────────────────────────────────────────────────────────
// Planning: every check, then the layout
// ─────────────────────────────────────────────────────────────────────────────

fn plan<'a>(
    samples: &[u16],
    width: u32,
    height: u32,
    options: &WriteOptions<'a>,
    max_pixels: usize,
) -> Result<Plan<'a>, Error> {
    // 1. The pixel budget, first (DEC-050). Two u32s always fit a u64.
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    let budget = u64::try_from(max_pixels).unwrap_or(u64::MAX);
    if pixels > budget {
        return Err(Error::PlaneTooLarge {
            pixels,
            limit: max_pixels,
        });
    }

    // 2. The sample count.
    if width == 0 || height == 0 {
        return Err(invalid(WriteInputError::EmptyPlane));
    }
    if u64::try_from(samples.len()).ok() != Some(pixels) {
        return Err(invalid(WriteInputError::SampleCountMismatch {
            expected: pixels,
            actual: samples.len(),
        }));
    }

    // 3. The geometry.
    let framing = options.framing;
    let active = match options.active_area {
        Some(a) => {
            if a.top >= a.bottom || a.left >= a.right || a.bottom > height || a.right > width {
                return Err(invalid(WriteInputError::ActiveAreaInvalid));
            }
            a
        }
        None => ActiveArea {
            top: 0,
            left: 0,
            bottom: height,
            right: width,
        },
    };
    // Non-empty and inside the plane, so both differences are positive.
    let active_width = active
        .right
        .checked_sub(active.left)
        .ok_or_else(too_large)?;
    let active_height = active
        .bottom
        .checked_sub(active.top)
        .ok_or_else(too_large)?;

    if let Some((origin, size)) = options.default_crop {
        let fits = |o: u32, s: u32, limit: u32| o.checked_add(s).is_some_and(|end| end <= limit);
        if size.width == 0
            || size.height == 0
            || !fits(origin.x, size.width, active_width)
            || !fits(origin.y, size.height, active_height)
        {
            return Err(invalid(WriteInputError::DefaultCropInvalid));
        }
    }

    if !options.masked_areas.is_empty() && framing != Framing::FullSensor {
        return Err(invalid(WriteInputError::MaskedAreasNeedFullSensor));
    }
    for (index, m) in options.masked_areas.iter().enumerate() {
        let empty = m.top >= m.bottom || m.left >= m.right;
        let outside = m.bottom > height || m.right > width;
        let overlaps = m.top < active.bottom
            && active.top < m.bottom
            && m.left < active.right
            && active.left < m.right;
        if empty || outside || overlaps {
            return Err(invalid(WriteInputError::MaskedAreaInvalid { index }));
        }
    }

    // 4. The levels.
    let expected_blacks = match options.black_level_repeat_dim {
        Some([rows, cols]) => {
            if rows == 0 || cols == 0 {
                return Err(invalid(WriteInputError::BlackLevelRepeatDimInvalid));
            }
            // Two u16s: the product always fits a u32.
            u32::from(rows)
                .checked_mul(u32::from(cols))
                .ok_or_else(too_large)?
        }
        None => 1,
    };
    let blacks = options.black_level.len();
    let blacks_ok = u32::try_from(blacks).ok() == Some(expected_blacks)
        || (blacks == 0 && options.black_level_repeat_dim.is_none());
    if !blacks_ok {
        return Err(invalid(WriteInputError::BlackLevelCount {
            expected: expected_blacks,
            actual: blacks,
        }));
    }

    // 5. The orientation.
    if !(1..=8).contains(&options.orientation) {
        return Err(invalid(WriteInputError::OrientationInvalid {
            value: options.orientation,
        }));
    }

    // 6. The strings.
    let model = options.unique_camera_model.unwrap_or("").as_bytes();
    if model.iter().all(|&b| b == b' ') {
        // Also true for an empty string.
        return Err(invalid(WriteInputError::MissingUniqueCameraModel));
    }
    if model.len() > MAX_ASCII_LEN || !model.iter().all(|b| (0x20..=0x7E).contains(b)) {
        return Err(invalid(WriteInputError::InvalidAscii {
            field: "UniqueCameraModel",
        }));
    }

    // 7. The opcode lists (DEC-075 §5). Only list 1 works on stored-plane
    // coordinates, so only list 1 is walked, and only when the cut can move
    // them.
    if framing != Framing::FullSensor {
        if let Some(list) = options.opcode_list_1 {
            if let Some(opcode_id) = walk_list_1(list)? {
                return Err(Error::UnsupportedOpcodeForFraming {
                    list: 1,
                    opcode_id,
                    framing,
                });
            }
        }
    }

    // What is written, per the framing table.
    let (cut, written_active, written_masked) = match framing {
        Framing::FullSensor => (
            ActiveArea {
                top: 0,
                left: 0,
                bottom: height,
                right: width,
            },
            options.active_area,
            options.masked_areas,
        ),
        Framing::FullActiveArea | Framing::StandardCrop => (active, None, &[][..]),
    };
    let (crop_origin, crop_size) = match (framing, options.default_crop) {
        (Framing::StandardCrop, Some((origin, size))) => {
            ([origin.x, origin.y], [size.width, size.height])
        }
        _ => ([0, 0], [active_width, active_height]),
    };
    let out_width = cut.right.checked_sub(cut.left).ok_or_else(too_large)?;
    let out_height = cut.bottom.checked_sub(cut.top).ok_or_else(too_large)?;

    // 8. The output size: the strip and every offset fit TIFF's u32.
    let strip_bytes = u64::from(out_width)
        .checked_mul(u64::from(out_height))
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(too_large)?;
    let strip_byte_count = u32::try_from(strip_bytes).map_err(|_| too_large())?;

    let any_opcodes = options.opcode_list_1.is_some()
        || options.opcode_list_2.is_some()
        || options.opcode_list_3.is_some();
    let backward = if any_opcodes {
        BACKWARD_WITH_OPCODES
    } else {
        BACKWARD_WITHOUT_OPCODES
    };

    let e = |tag: u16, value: Value<'a>| Some(Entry { tag, value });
    let entries: [Option<Entry<'a>>; SLOTS] = [
        e(TAG_NEW_SUBFILE_TYPE, Value::Long(0)),
        e(TAG_IMAGE_WIDTH, Value::Long(out_width)),
        e(TAG_IMAGE_LENGTH, Value::Long(out_height)),
        e(TAG_BITS_PER_SAMPLE, Value::Short(16)),
        e(TAG_COMPRESSION, Value::Short(1)),
        e(TAG_PHOTOMETRIC, Value::Short(PHOTOMETRIC_LINEAR_RAW)),
        e(TAG_STRIP_OFFSETS, Value::StripOffset),
        e(TAG_ORIENTATION, Value::Short(options.orientation)),
        e(TAG_SAMPLES_PER_PIXEL, Value::Short(1)),
        e(TAG_ROWS_PER_STRIP, Value::Long(out_height)),
        e(TAG_STRIP_BYTE_COUNTS, Value::Long(strip_byte_count)),
        e(TAG_PLANAR_CONFIGURATION, Value::Short(1)),
        e(TAG_DNG_VERSION, Value::Bytes4(DNG_VERSION)),
        e(TAG_DNG_BACKWARD_VERSION, Value::Bytes4(backward)),
        e(TAG_UNIQUE_CAMERA_MODEL, Value::Ascii(model_bytes(options))),
        options
            .black_level_repeat_dim
            .and_then(|d| e(TAG_BLACK_LEVEL_REPEAT_DIM, Value::Shorts2(d))),
        (!options.black_level.is_empty())
            .then_some(())
            .and_then(|()| e(TAG_BLACK_LEVEL, Value::Longs(options.black_level))),
        e(TAG_WHITE_LEVEL, Value::Long(options.white_level)),
        e(TAG_DEFAULT_CROP_ORIGIN, Value::Longs2(crop_origin)),
        e(TAG_DEFAULT_CROP_SIZE, Value::Longs2(crop_size)),
        written_active.and_then(|a| {
            e(
                TAG_ACTIVE_AREA,
                Value::Longs4([a.top, a.left, a.bottom, a.right]),
            )
        }),
        (!written_masked.is_empty())
            .then_some(())
            .and_then(|()| e(TAG_MASKED_AREAS, Value::Rects(written_masked))),
        options
            .opcode_list_1
            .and_then(|b| e(TAG_OPCODE_LIST_1, Value::Undefined(b))),
        options
            .opcode_list_2
            .and_then(|b| e(TAG_OPCODE_LIST_2, Value::Undefined(b))),
        options
            .opcode_list_3
            .and_then(|b| e(TAG_OPCODE_LIST_3, Value::Undefined(b))),
    ];

    // The layout (DEC-075 §1): header, IFD0, out-of-line values in tag order,
    // then the strip, each value and the strip at an even offset.
    let entry_count = entries.iter().flatten().count();
    let ifd_len = u64::try_from(entry_count)
        .ok()
        .and_then(|n| n.checked_mul(12))
        .and_then(|n| n.checked_add(6)) // 2-byte count + 4-byte next-IFD offset
        .ok_or_else(too_large)?;
    let mut pos = IFD0_OFFSET.checked_add(ifd_len).ok_or_else(too_large)?;
    let mut value_offsets = [0u32; SLOTS];
    for (slot, offset) in entries.iter().zip(value_offsets.iter_mut()) {
        let Some(entry) = slot else { continue };
        // The count must fit TIFF's u32 too.
        let count = entry.value.count().ok_or_else(too_large)?;
        u32::try_from(count).map_err(|_| too_large())?;
        let len = entry.value.byte_len().ok_or_else(too_large)?;
        if len > 4 {
            pos = align2(pos)?;
            *offset = u32::try_from(pos).map_err(|_| too_large())?;
            pos = pos.checked_add(len).ok_or_else(too_large)?;
        }
    }
    pos = align2(pos)?;
    let strip_offset = u32::try_from(pos).map_err(|_| too_large())?;
    pos = pos.checked_add(strip_bytes).ok_or_else(too_large)?;
    if pos > u64::from(u32::MAX) {
        return Err(too_large());
    }
    let total = usize::try_from(pos).map_err(|_| too_large())?;

    Ok(Plan {
        entries,
        value_offsets,
        entry_count: u16::try_from(entry_count).map_err(|_| too_large())?,
        strip_offset,
        cut,
        plane_width: width,
        total,
    })
}

/// `UniqueCameraModel`'s bytes, already validated by [`plan`].
fn model_bytes<'a>(options: &WriteOptions<'a>) -> &'a [u8] {
    options.unique_camera_model.unwrap_or("").as_bytes()
}

/// `n` rounded up to an even offset (TIFF 6.0 §2: values begin on a word
/// boundary).
fn align2(n: u64) -> Result<u64, Error> {
    n.checked_add(n % 2).ok_or_else(too_large)
}

/// Walk `OpcodeList1`'s opcode headers (DNG 1.7.1 ch. 6: big-endian whatever
/// the file's byte order; a 4-byte count, then per opcode its id, version,
/// flags and parameter byte count, 4 bytes each, and that many parameter
/// bytes). Reads nothing else.
///
/// `Ok(Some(id))` names the first opcode outside the position-independent
/// allow-list; `Ok(None)` means every opcode is on it. The whole list is
/// walked before an opcode is reported, so a list that is both malformed and
/// holds an unlisted opcode is always reported as malformed.
fn walk_list_1(list: &[u8]) -> Result<Option<u32>, Error> {
    let malformed = || invalid(WriteInputError::MalformedOpcodeList { list: 1 });
    let be_u32 = |at: usize| -> Result<u32, Error> {
        let end = at.checked_add(4).ok_or_else(malformed)?;
        let bytes = list.get(at..end).ok_or_else(malformed)?;
        let mut word = [0u8; 4];
        copy(&mut word, bytes);
        Ok(u32::from_be_bytes(word))
    };
    let count = be_u32(0)?;
    let mut pos: usize = 4;
    let mut first_unlisted = None;
    // Each opcode takes at least 16 bytes, so a hostile count ends the loop at
    // the list's end, not after `count` iterations.
    for _ in 0..count {
        let id = be_u32(pos)?;
        let params = pos.checked_add(12).ok_or_else(malformed)?;
        let size = usize::try_from(be_u32(params)?).map_err(|_| malformed())?;
        pos = pos
            .checked_add(16)
            .and_then(|p| p.checked_add(size))
            .ok_or_else(malformed)?;
        if pos > list.len() {
            return Err(malformed());
        }
        if first_unlisted.is_none() && !LIST_1_POSITION_INDEPENDENT.contains(&id) {
            first_unlisted = Some(id);
        }
    }
    if pos != list.len() {
        return Err(malformed());
    }
    Ok(first_unlisted)
}

// ─────────────────────────────────────────────────────────────────────────────
// Serialisation
// ─────────────────────────────────────────────────────────────────────────────

/// Write the planned file into `dst`, which is exactly `plan.total` bytes.
/// Every byte is written: the header, IFD0 and values region is zeroed first
/// (its pads stay 0), and the strip runs to the end.
fn serialize(plan: &Plan<'_>, samples: &[u16], dst: &mut [u8]) -> Result<(), Error> {
    let strip_at = usize::try_from(plan.strip_offset).map_err(|_| too_large())?;
    let (head, strip) = dst.split_at_mut_checked(strip_at).ok_or_else(too_large)?;
    head.fill(0);

    let ifd0 = u32::try_from(IFD0_OFFSET).map_err(|_| too_large())?;
    put(head, 0, b"II")?;
    put(head, 2, &42u16.to_le_bytes())?;
    put(head, 4, &ifd0.to_le_bytes())?;

    let mut at = usize::try_from(IFD0_OFFSET).map_err(|_| too_large())?;
    at = put(head, at, &plan.entry_count.to_le_bytes())?;
    for (slot, &offset) in plan.entries.iter().zip(&plan.value_offsets) {
        let Some(entry) = slot else { continue };
        let value = entry.value;
        let count = value
            .count()
            .and_then(|c| u32::try_from(c).ok())
            .ok_or_else(too_large)?;
        at = put(head, at, &entry.tag.to_le_bytes())?;
        at = put(head, at, &value.field_type().to_le_bytes())?;
        at = put(head, at, &count.to_le_bytes())?;
        let len = value
            .byte_len()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(too_large)?;
        if len > 4 {
            at = put(head, at, &offset.to_le_bytes())?;
            let start = usize::try_from(offset).map_err(|_| too_large())?;
            let end = start.checked_add(len).ok_or_else(too_large)?;
            let out = head.get_mut(start..end).ok_or_else(too_large)?;
            emit(value, out, plan.strip_offset);
        } else {
            // TIFF 6.0 §2: an inline value is left-justified in the 4-byte
            // field, whatever its type. In little-endian a lone SHORT thus
            // sits in the low half; the rest of the word stays 0.
            let mut word = [0u8; 4];
            let out = word.get_mut(..len).ok_or_else(too_large)?;
            emit(value, out, plan.strip_offset);
            at = put(head, at, &word)?;
        }
    }
    put(head, at, &0u32.to_le_bytes())?; // no next IFD

    write_strip(plan, samples, strip)
}

/// Copy `bytes` into `dst` at `at`; returns the offset after them.
fn put(dst: &mut [u8], at: usize, bytes: &[u8]) -> Result<usize, Error> {
    let end = at.checked_add(bytes.len()).ok_or_else(too_large)?;
    let out = dst.get_mut(at..end).ok_or_else(too_large)?;
    copy(out, bytes);
    Ok(end)
}

/// `dst[i] = src[i]` over the shorter of the two. Never panics on a length
/// mismatch (unlike `copy_from_slice`); every caller passes equal lengths.
fn copy(dst: &mut [u8], src: &[u8]) {
    for (d, s) in dst.iter_mut().zip(src) {
        *d = *s;
    }
}

/// Write `value`'s little-endian bytes into `out`, which is exactly its
/// `byte_len`.
fn emit(value: Value<'_>, out: &mut [u8], strip_offset: u32) {
    match value {
        Value::Short(v) => copy(out, &v.to_le_bytes()),
        Value::Shorts2(v) => {
            for (o, x) in out.as_chunks_mut::<2>().0.iter_mut().zip(v) {
                *o = x.to_le_bytes();
            }
        }
        Value::Long(v) => copy(out, &v.to_le_bytes()),
        Value::StripOffset => copy(out, &strip_offset.to_le_bytes()),
        Value::Longs2(v) => longs(out, &v),
        Value::Longs4(v) => longs(out, &v),
        Value::Longs(v) => longs(out, v),
        Value::Rects(v) => {
            for (o, r) in out.as_chunks_mut::<16>().0.iter_mut().zip(v) {
                longs(o, &[r.top, r.left, r.bottom, r.right]);
            }
        }
        Value::Bytes4(v) => copy(out, &v),
        Value::Ascii(v) => {
            // The NUL terminator is the byte after the text: `out` is
            // `v.len() + 1` long and the caller zeroed it, but write it anyway
            // so the inline path (an unzeroed word) is right too.
            copy(out, v);
            if let Some(last) = out.last_mut() {
                *last = 0;
            }
        }
        Value::Undefined(v) => copy(out, v),
    }
}

fn longs(out: &mut [u8], v: &[u32]) {
    for (o, x) in out.as_chunks_mut::<4>().0.iter_mut().zip(v) {
        *o = x.to_le_bytes();
    }
}

/// The strip: the cut's rows, each from its `left` to its `right`, as
/// little-endian `u16`.
fn write_strip(plan: &Plan<'_>, samples: &[u16], strip: &mut [u8]) -> Result<(), Error> {
    let plane_width = usize::try_from(plan.plane_width).map_err(|_| too_large())?;
    let top = usize::try_from(plan.cut.top).map_err(|_| too_large())?;
    let left = usize::try_from(plan.cut.left).map_err(|_| too_large())?;
    let right = usize::try_from(plan.cut.right).map_err(|_| too_large())?;
    let rows = plan
        .cut
        .bottom
        .checked_sub(plan.cut.top)
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(too_large)?;
    let row_bytes = right
        .checked_sub(left)
        .and_then(|n| n.checked_mul(2))
        .ok_or_else(too_large)?;
    if plane_width == 0
        || row_bytes == 0
        || strip.len() != row_bytes.checked_mul(rows).ok_or_else(too_large)?
    {
        return Err(too_large());
    }
    let source_rows = samples.chunks_exact(plane_width).skip(top).take(rows);
    for (out_row, source_row) in strip.chunks_exact_mut(row_bytes).zip(source_rows) {
        let source = source_row.get(left..right).ok_or_else(too_large)?;
        for (o, s) in out_row.as_chunks_mut::<2>().0.iter_mut().zip(source) {
            *o = s.to_le_bytes();
        }
    }
    Ok(())
}
