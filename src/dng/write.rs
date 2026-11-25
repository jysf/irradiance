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
//! Metadata a source carries (`SPEC-081`) is written whenever its option is
//! `Some`, under every framing: the eight descriptive ASCII tags (`Make`,
//! `Model`, `Software`, `DateTime`, `Artist`, `Copyright`,
//! `LocalizedCameraModel`, `CameraSerialNumber`; an empty string writes an
//! empty tag, `DEC-081` §2), `XMLPacket` (the XMP packet as `BYTE`s, never
//! parsed), and the four rendering hints `BaselineExposure` (`SRATIONAL`),
//! `BaselineNoise`, `BaselineSharpness` and `LinearResponseLimit`
//! (`RATIONAL`), written as given (`DEC-074`).
//!
//! An [`ExifBlock`] is copied verbatim to file offset [`EXIF_BLOCK_BASE`] (8),
//! right after the header, and never parsed or rebased (`DEC-072`): the caller
//! builds it for that base (ask [`exif_block_base`]). One 0 pad byte follows
//! an odd-length block, then IFD0. `ExifIFD` and `GPSInfo` are written as the
//! base plus each in-block offset given:
//!
//! ```text
//! offset 0   II 42 <IFD0 offset>
//! offset 8   the EXIF block, verbatim (when given), then a 0 pad if odd
//!            IFD0
//!            out-of-line values, in tag order, each at an even offset
//!            the strip, at an even offset
//! ```
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
//! orientation, `UniqueCameraModel`, the descriptive strings in tag order
//! (printable ASCII of at most 1024 bytes, then `DateTime`'s form), the
//! rendering hints in tag order, the EXIF block's IFD offsets (at least one,
//! each even and leaving room for its entry count inside the block), the XMP
//! packet (not empty), the opcode lists, the output size. Every
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
//! layout, `DNGBackwardVersion` 1.3 for opcode lists, tags 50709 and 50730 to
//! 50735), Exif 2.32 §4.6.3 (the `ExifIFD` and `GPSInfo` pointers), XMP
//! Specification Part 3 §1.1.4 (tag 700), `DEC-072`, `DEC-074`, `DEC-081` and
//! `DEC-008` in the writing direction; the layout follows this repository's own test writer
//! (`tests/support/dng_writer.rs`, `SPEC-057`). No writer implementation was
//! read.

use crate::ifd::Rational;
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
    ///
    /// The written file is always 16-bit, so a source with fewer bits needs
    /// its own white level here. When the source has no `WhiteLevel` tag
    /// ([`crate::ifd::Sensor::white_level`] is `None`), DNG's default for it
    /// is `2^bits − 1` (4095 for 12-bit, 16383 for 14-bit): pass that
    /// explicitly, or the written file's samples are read against 65535 and
    /// develop several stops dark.
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
    /// The caller-built EXIF block, placed verbatim at [`EXIF_BLOCK_BASE`]
    /// (`DEC-072`), with `ExifIFD` / `GPSInfo` pointing into it. `None`
    /// writes neither.
    pub exif_block: Option<ExifBlock<'a>>,
    /// The XMP packet, written verbatim as `XMLPacket` (tag 700, `BYTE`) and
    /// never parsed. Must not be empty (`DEC-081` §2).
    pub xmp: Option<&'a [u8]>,
    /// `Make`. Each descriptive string (`DEC-071` §3a) is printable ASCII of
    /// at most 1024 bytes; an empty string writes an empty tag (`DEC-081`
    /// §2) and `None` writes none.
    pub make: Option<&'a str>,
    /// `Model`, as [`WriteOptions::make`].
    pub model: Option<&'a str>,
    /// `Software`, as [`WriteOptions::make`].
    pub software: Option<&'a str>,
    /// `DateTime`, as [`WriteOptions::make`] and also exactly
    /// `YYYY:MM:DD HH:MM:SS` (TIFF 6.0). The form is checked, not the
    /// calendar. The writer never stamps a time itself.
    pub date_time: Option<&'a str>,
    /// `Artist`, as [`WriteOptions::make`].
    pub artist: Option<&'a str>,
    /// `Copyright`, as [`WriteOptions::make`]: one string, the photographer
    /// part only.
    pub copyright: Option<&'a str>,
    /// `LocalizedCameraModel`, as [`WriteOptions::make`] (ASCII in v1).
    pub localized_camera_model: Option<&'a str>,
    /// `CameraSerialNumber`, as [`WriteOptions::make`].
    pub camera_serial_number: Option<&'a str>,
    /// `BaselineExposure` (`SRATIONAL`, `DEC-074`), written as given. Any
    /// sign; a zero denominator is refused.
    pub baseline_exposure: Option<Rational>,
    /// `BaselineNoise` (`RATIONAL`), written as given. A zero denominator or
    /// a negative part is refused (`DEC-081` §3).
    pub baseline_noise: Option<Rational>,
    /// `BaselineSharpness` (`RATIONAL`), as [`WriteOptions::baseline_noise`].
    pub baseline_sharpness: Option<Rational>,
    /// `LinearResponseLimit` (`RATIONAL`), as
    /// [`WriteOptions::baseline_noise`].
    pub linear_response_limit: Option<Rational>,
}

/// A caller-built EXIF block (`DEC-072`, `DEC-081` §1): the source's Exif,
/// GPS and Interop IFDs as one little-endian byte run, built for
/// [`EXIF_BLOCK_BASE`], every offset inside it already absolute.
///
/// The writer copies `bytes` verbatim to file offset [`EXIF_BLOCK_BASE`] and
/// never parses or rebases them; it checks only that each given IFD offset is
/// even and leaves room for the IFD's 2-byte entry count inside the block.
/// `#[non_exhaustive]`: start from [`ExifBlock::default`] and assign fields.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct ExifBlock<'a> {
    /// The block, built for [`EXIF_BLOCK_BASE`], little-endian, every offset
    /// in it already absolute. Copied verbatim; never parsed.
    pub bytes: &'a [u8],
    /// The Exif IFD's offset inside `bytes` (the file offset minus the base).
    /// Written as `ExifIFD` = base + offset.
    pub exif_ifd: Option<u32>,
    /// The GPS IFD's offset inside `bytes`. Written as `GPSInfo` = base +
    /// offset.
    pub gps_ifd: Option<u32>,
}

/// Where the EXIF block starts in every written file (`DEC-072`): right after
/// the 8-byte header. Pinned by a test; changing it is a breaking change that
/// needs a DEC.
pub const EXIF_BLOCK_BASE: u32 = 8;

/// The base the caller must build its EXIF block for. Always
/// [`EXIF_BLOCK_BASE`] today; `DEC-072` asks for a function of the options so
/// that a later layout can say otherwise without a new API.
pub fn exif_block_base(options: &WriteOptions<'_>) -> u32 {
    let _ = options;
    EXIF_BLOCK_BASE
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
            exif_block: None,
            xmp: None,
            make: None,
            model: None,
            software: None,
            date_time: None,
            artist: None,
            copyright: None,
            localized_camera_model: None,
            camera_serial_number: None,
            baseline_exposure: None,
            baseline_noise: None,
            baseline_sharpness: None,
            linear_response_limit: None,
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
/// TIFF 6.0 descriptive tags (§8).
const TAG_MAKE: u16 = 271;
const TAG_MODEL: u16 = 272;
const TAG_SOFTWARE: u16 = 305;
const TAG_DATE_TIME: u16 = 306;
const TAG_ARTIST: u16 = 315;
/// XMP Specification Part 3 §1.1.4: the packet in TIFF.
const TAG_XML_PACKET: u16 = 700;
const TAG_COPYRIGHT: u16 = 33432;
/// Exif 2.32 §4.6.3: the IFD0 pointers to the Exif and GPS IFDs.
const TAG_EXIF_IFD: u16 = 34665;
const TAG_GPS_INFO: u16 = 34853;
/// DNG 1.7.1 descriptive and rendering-hint tags.
const TAG_LOCALIZED_CAMERA_MODEL: u16 = 50709;
const TAG_BASELINE_EXPOSURE: u16 = 50730;
const TAG_BASELINE_NOISE: u16 = 50731;
const TAG_BASELINE_SHARPNESS: u16 = 50732;
const TAG_LINEAR_RESPONSE_LIMIT: u16 = 50734;
const TAG_CAMERA_SERIAL_NUMBER: u16 = 50735;

const TYPE_BYTE: u16 = 1;
const TYPE_ASCII: u16 = 2;
const TYPE_SHORT: u16 = 3;
const TYPE_LONG: u16 = 4;
const TYPE_RATIONAL: u16 = 5;
const TYPE_UNDEFINED: u16 = 7;
const TYPE_SRATIONAL: u16 = 10;

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

/// `DateTime`'s form, TIFF 6.0: `YYYY:MM:DD HH:MM:SS`. `D` is an ASCII digit;
/// every other byte must match exactly.
const DATE_TIME_FORM: &[u8; 19] = b"DDDD:DD:DD DD:DD:DD";

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
    /// `BYTE`s, verbatim (`XMLPacket`).
    Bytes(&'a [u8]),
    Undefined(&'a [u8]),
    /// One `RATIONAL`: numerator, denominator.
    Rational(u32, u32),
    /// One `SRATIONAL`: numerator, denominator.
    SRational(i32, i32),
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
            Value::Bytes4(_) | Value::Bytes(_) => TYPE_BYTE,
            Value::Ascii(_) => TYPE_ASCII,
            Value::Undefined(_) => TYPE_UNDEFINED,
            Value::Rational(..) => TYPE_RATIONAL,
            Value::SRational(..) => TYPE_SRATIONAL,
        }
    }

    /// The TIFF count: values of the field's type.
    fn count(&self) -> Option<u64> {
        let len = |n: usize| u64::try_from(n).ok();
        match self {
            Value::Short(_)
            | Value::Long(_)
            | Value::StripOffset
            | Value::Rational(..)
            | Value::SRational(..) => Some(1),
            Value::Shorts2(_) | Value::Longs2(_) => Some(2),
            Value::Longs4(_) | Value::Bytes4(_) => Some(4),
            Value::Longs(v) => len(v.len()),
            Value::Rects(v) => len(v.len())?.checked_mul(4),
            Value::Ascii(v) => len(v.len())?.checked_add(1),
            Value::Undefined(v) | Value::Bytes(v) => len(v.len()),
        }
    }

    /// The value's size in bytes.
    fn byte_len(&self) -> Option<u64> {
        let unit: u64 = match self.field_type() {
            TYPE_SHORT => 2,
            TYPE_LONG => 4,
            TYPE_RATIONAL | TYPE_SRATIONAL => 8,
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
const SLOTS: usize = 40;

/// Everything [`serialize`] needs, decided and checked by [`plan`].
#[derive(Debug)]
struct Plan<'a> {
    entries: [Option<Entry<'a>>; SLOTS],
    /// The offset of each slot's out-of-line value; 0 for an inline one.
    value_offsets: [u32; SLOTS],
    entry_count: u16,
    /// The header's IFD0 offset: after the header and the EXIF block, even.
    ifd0_offset: u32,
    /// The EXIF block, copied verbatim to [`EXIF_BLOCK_BASE`]; empty without
    /// one.
    exif_block: &'a [u8],
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

    // 6a. The descriptive strings, in tag order (DEC-071 §3a, DEC-081 §4).
    let descriptive = [
        ("Make", options.make),
        ("Model", options.model),
        ("Software", options.software),
        ("DateTime", options.date_time),
        ("Artist", options.artist),
        ("Copyright", options.copyright),
        ("LocalizedCameraModel", options.localized_camera_model),
        ("CameraSerialNumber", options.camera_serial_number),
    ];
    for (field, value) in descriptive {
        let Some(text) = value else { continue };
        let bytes = text.as_bytes();
        if bytes.len() > MAX_ASCII_LEN || !bytes.iter().all(|b| (0x20..=0x7E).contains(b)) {
            return Err(invalid(WriteInputError::InvalidAscii { field }));
        }
        if field == "DateTime" && !date_time_form(bytes) {
            return Err(invalid(WriteInputError::DateTimeInvalid));
        }
    }

    // 6b. The rendering hints, in tag order (DEC-074, DEC-081 §3).
    let baseline_exposure = match options.baseline_exposure {
        Some(r) if r.den == 0 => {
            return Err(invalid(WriteInputError::RationalInvalid {
                field: "BaselineExposure",
            }))
        }
        Some(r) => Some(Value::SRational(r.num, r.den)),
        None => None,
    };
    let baseline_noise = unsigned_hint("BaselineNoise", options.baseline_noise)?;
    let baseline_sharpness = unsigned_hint("BaselineSharpness", options.baseline_sharpness)?;
    let linear_response_limit =
        unsigned_hint("LinearResponseLimit", options.linear_response_limit)?;

    // 6c. The EXIF block (DEC-072, DEC-081 §5). Only the offsets are looked
    // at; no byte of the block is read.
    if let Some(block) = options.exif_block {
        if block.exif_ifd.is_none() && block.gps_ifd.is_none() {
            return Err(invalid(WriteInputError::ExifBlockWithoutIfd));
        }
        let given = [("Exif", block.exif_ifd), ("GPS", block.gps_ifd)];
        for (ifd, offset) in given {
            let Some(offset) = offset else { continue };
            let inside = usize::try_from(offset)
                .ok()
                .and_then(|o| o.checked_add(2))
                .is_some_and(|end| end <= block.bytes.len());
            if offset % 2 != 0 || !inside {
                return Err(invalid(WriteInputError::ExifIfdOffsetInvalid {
                    ifd,
                    offset,
                }));
            }
        }
    }

    // 6d. The XMP packet (DEC-081 §2): never parsed, but never empty.
    if options.xmp.is_some_and(<[u8]>::is_empty) {
        return Err(invalid(WriteInputError::EmptyXmp));
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

    // The EXIF block sits at the base, then one pad byte when its length is
    // odd, then IFD0 (DEC-072). Its pointers are the base plus the caller's
    // in-block offsets.
    let exif_block = options.exif_block.map_or(&[][..], |b| b.bytes);
    let block_len = u64::try_from(exif_block.len()).map_err(|_| too_large())?;
    let ifd0_offset = u64::from(EXIF_BLOCK_BASE)
        .checked_add(align2(block_len)?)
        .ok_or_else(too_large)?;
    let pointer = |offset: Option<u32>| -> Result<Option<Value<'a>>, Error> {
        offset
            .map(|o| {
                EXIF_BLOCK_BASE
                    .checked_add(o)
                    .map(Value::Long)
                    .ok_or_else(too_large)
            })
            .transpose()
    };
    let exif_pointer = pointer(options.exif_block.and_then(|b| b.exif_ifd))?;
    let gps_pointer = pointer(options.exif_block.and_then(|b| b.gps_ifd))?;
    let ascii = |text: Option<&'a str>| text.map(|t| Value::Ascii(t.as_bytes()));

    let e = |tag: u16, value: Value<'a>| Some(Entry { tag, value });
    // Optional tags: the slot is filled only when the option is `Some`.
    let o = |tag: u16, value: Option<Value<'a>>| value.map(|value| Entry { tag, value });
    // Ascending tag order: the IFD's order is this array's order.
    let entries: [Option<Entry<'a>>; SLOTS] = [
        e(TAG_NEW_SUBFILE_TYPE, Value::Long(0)),
        e(TAG_IMAGE_WIDTH, Value::Long(out_width)),
        e(TAG_IMAGE_LENGTH, Value::Long(out_height)),
        e(TAG_BITS_PER_SAMPLE, Value::Short(16)),
        e(TAG_COMPRESSION, Value::Short(1)),
        e(TAG_PHOTOMETRIC, Value::Short(PHOTOMETRIC_LINEAR_RAW)),
        o(TAG_MAKE, ascii(options.make)),
        o(TAG_MODEL, ascii(options.model)),
        e(TAG_STRIP_OFFSETS, Value::StripOffset),
        e(TAG_ORIENTATION, Value::Short(options.orientation)),
        e(TAG_SAMPLES_PER_PIXEL, Value::Short(1)),
        e(TAG_ROWS_PER_STRIP, Value::Long(out_height)),
        e(TAG_STRIP_BYTE_COUNTS, Value::Long(strip_byte_count)),
        e(TAG_PLANAR_CONFIGURATION, Value::Short(1)),
        o(TAG_SOFTWARE, ascii(options.software)),
        o(TAG_DATE_TIME, ascii(options.date_time)),
        o(TAG_ARTIST, ascii(options.artist)),
        o(TAG_XML_PACKET, options.xmp.map(Value::Bytes)),
        o(TAG_COPYRIGHT, ascii(options.copyright)),
        o(TAG_EXIF_IFD, exif_pointer),
        o(TAG_GPS_INFO, gps_pointer),
        e(TAG_DNG_VERSION, Value::Bytes4(DNG_VERSION)),
        e(TAG_DNG_BACKWARD_VERSION, Value::Bytes4(backward)),
        e(TAG_UNIQUE_CAMERA_MODEL, Value::Ascii(model_bytes(options))),
        o(
            TAG_LOCALIZED_CAMERA_MODEL,
            ascii(options.localized_camera_model),
        ),
        options
            .black_level_repeat_dim
            .and_then(|d| e(TAG_BLACK_LEVEL_REPEAT_DIM, Value::Shorts2(d))),
        (!options.black_level.is_empty())
            .then_some(())
            .and_then(|()| e(TAG_BLACK_LEVEL, Value::Longs(options.black_level))),
        e(TAG_WHITE_LEVEL, Value::Long(options.white_level)),
        e(TAG_DEFAULT_CROP_ORIGIN, Value::Longs2(crop_origin)),
        e(TAG_DEFAULT_CROP_SIZE, Value::Longs2(crop_size)),
        o(TAG_BASELINE_EXPOSURE, baseline_exposure),
        o(TAG_BASELINE_NOISE, baseline_noise),
        o(TAG_BASELINE_SHARPNESS, baseline_sharpness),
        o(TAG_LINEAR_RESPONSE_LIMIT, linear_response_limit),
        o(
            TAG_CAMERA_SERIAL_NUMBER,
            ascii(options.camera_serial_number),
        ),
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

    // The layout (DEC-075 §1, DEC-072): header, the EXIF block and its pad,
    // IFD0, out-of-line values in tag order, then the strip, each value and
    // the strip at an even offset.
    let entry_count = entries.iter().flatten().count();
    let ifd_len = u64::try_from(entry_count)
        .ok()
        .and_then(|n| n.checked_mul(12))
        .and_then(|n| n.checked_add(6)) // 2-byte count + 4-byte next-IFD offset
        .ok_or_else(too_large)?;
    let mut pos = ifd0_offset.checked_add(ifd_len).ok_or_else(too_large)?;
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
        ifd0_offset: u32::try_from(ifd0_offset).map_err(|_| too_large())?,
        exif_block,
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

/// Whether `bytes` has `DateTime`'s form (`DATE_TIME_FORM`): the form, not
/// the calendar (`DEC-081` §6).
fn date_time_form(bytes: &[u8]) -> bool {
    bytes.len() == DATE_TIME_FORM.len()
        && bytes.iter().zip(DATE_TIME_FORM).all(|(&b, &f)| {
            if f == b'D' {
                b.is_ascii_digit()
            } else {
                b == f
            }
        })
}

/// One of the three unsigned (`RATIONAL`) hints: a zero denominator or a
/// negative part is refused (`DEC-074`, `DEC-081` §3).
fn unsigned_hint<'a>(
    field: &'static str,
    hint: Option<Rational>,
) -> Result<Option<Value<'a>>, Error> {
    let Some(r) = hint else { return Ok(None) };
    let refuse = || invalid(WriteInputError::RationalInvalid { field });
    if r.den == 0 {
        return Err(refuse());
    }
    let num = u32::try_from(r.num).map_err(|_| refuse())?;
    let den = u32::try_from(r.den).map_err(|_| refuse())?;
    Ok(Some(Value::Rational(num, den)))
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

    put(head, 0, b"II")?;
    put(head, 2, &42u16.to_le_bytes())?;
    put(head, 4, &plan.ifd0_offset.to_le_bytes())?;
    // The EXIF block, verbatim (DEC-072); its pad byte, if any, stays 0.
    let base = usize::try_from(EXIF_BLOCK_BASE).map_err(|_| too_large())?;
    put(head, base, plan.exif_block)?;

    let mut at = usize::try_from(plan.ifd0_offset).map_err(|_| too_large())?;
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
        Value::Rational(num, den) => longs(out, &[num, den]),
        Value::SRational(num, den) => {
            let [n, d] = out.as_chunks_mut::<4>().0 else {
                return;
            };
            *n = num.to_le_bytes();
            *d = den.to_le_bytes();
        }
        Value::Ascii(v) => {
            // The NUL terminator is the byte after the text: `out` is
            // `v.len() + 1` long and the caller zeroed it, but write it anyway
            // so the inline path (an unzeroed word) is right too.
            copy(out, v);
            if let Some(last) = out.last_mut() {
                *last = 0;
            }
        }
        Value::Undefined(v) | Value::Bytes(v) => copy(out, v),
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
