//! The two types the DNG writer's [`crate::Error`] variants carry
//! (`SPEC-061`, `DEC-075` §7, `DEC-080`).
//!
//! They live here, compiled in every build, and not in `src/dng/write.rs`
//! (behind the `write` feature), because [`crate::Error`]'s shape must not
//! depend on features: `Error::InvalidWriteInput` and
//! `Error::UnsupportedOpcodeForFraming` exist in every build. With the `write`
//! feature on they are named [`crate::dng::write::Framing`] and
//! [`crate::dng::write::WriteInputError`] (re-exported there); without it no
//! function can return those two variants, so the types are present but have
//! no public path. `DEC-080` records the choice.

use core::fmt;

/// How much of the stored plane the writer puts in the file, and how it frames
/// it (`DEC-073`, `SPEC-061` §3).
///
/// Distinct from [`crate::develop::Framing`]: that one picks which rectangle a
/// develop outputs, this one picks which samples a written DNG stores. The
/// writer never rotates or crops inside the active area; it cuts samples only
/// at the `ActiveArea` edge and expresses every other framing as tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Framing {
    /// The whole stored plane, margin included. `ActiveArea` is written when
    /// the caller gave one, `MaskedAreas` when the list is non-empty, every
    /// opcode list verbatim, and the default crop is the whole active area.
    FullSensor,
    /// Only the active area's samples, with the default crop set to the whole
    /// of them. `ActiveArea` and `MaskedAreas` are not written.
    FullActiveArea,
    /// Only the active area's samples, with the caller's default crop (or the
    /// whole active area when none is given) as tags. The default.
    #[default]
    StandardCrop,
}

impl fmt::Display for Framing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Framing::FullSensor => "full sensor",
            Framing::FullActiveArea => "full active area",
            Framing::StandardCrop => "standard crop",
        })
    }
}

/// Why the writer refused the caller's input, carried by
/// [`crate::Error::InvalidWriteInput`] (`SPEC-061` §5, `DEC-075` §7).
///
/// Every reason is about the arguments of one call, so the error is classified
/// [`crate::ErrorKind::CallerError`]. `#[non_exhaustive]` (`DEC-052`): a later
/// writer spec adds reasons without a breaking change.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteInputError {
    /// The plane's width or height is 0.
    EmptyPlane,
    /// `samples.len()` is not `width × height`.
    SampleCountMismatch {
        /// `width × height`.
        expected: u64,
        /// `samples.len()`.
        actual: usize,
    },
    /// The active area is empty, inverted, or reaches past the plane.
    ActiveAreaInvalid,
    /// The default crop is empty, or its origin plus size reaches past the
    /// active area.
    DefaultCropInvalid,
    /// A masked area is empty, reaches past the plane, or intersects the
    /// active area.
    MaskedAreaInvalid {
        /// The rectangle's position in the caller's list.
        index: usize,
    },
    /// Masked areas were given for a framing that drops the margin they
    /// describe (`DEC-075` §6). Only [`Framing::FullSensor`] writes them.
    MaskedAreasNeedFullSensor,
    /// `Orientation` is not one of TIFF's eight values (1 to 8).
    OrientationInvalid {
        /// The caller's value.
        value: u16,
    },
    /// The number of `BlackLevel` values does not match the repeat
    /// dimensions (rows × cols, or 1 without a repeat dimension; none at all is
    /// allowed only without one).
    BlackLevelCount {
        /// rows × cols, or 1.
        expected: u32,
        /// The number of values given.
        actual: usize,
    },
    /// `BlackLevelRepeatDim` has a zero dimension.
    BlackLevelRepeatDimInvalid,
    /// `UniqueCameraModel` is missing, empty, or only spaces (`DEC-071` §3a).
    MissingUniqueCameraModel,
    /// An ASCII field holds a byte outside printable ASCII (0x20 to 0x7E), or
    /// is longer than 1024 bytes.
    InvalidAscii {
        /// The DNG tag name of the field.
        field: &'static str,
    },
    /// An opcode list's headers cannot be walked: the declared count and the
    /// per-opcode parameter sizes do not add up to exactly the list's length.
    /// Only `OpcodeList1` is walked, and only under a framing that cuts.
    MalformedOpcodeList {
        /// 1, 2 or 3.
        list: u8,
    },
    /// An offset or the file's total length would not fit TIFF's 32 bits.
    OutputTooLarge,
    /// `dst` is not exactly the length the size query returned.
    OutputBufferWrongLength {
        /// The required length, in bytes.
        expected: usize,
        /// `dst.len()`.
        actual: usize,
    },
}

impl fmt::Display for WriteInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteInputError::EmptyPlane => f.write_str("the plane's width or height is 0"),
            WriteInputError::SampleCountMismatch { expected, actual } => write!(
                f,
                "the plane holds {actual} sample(s), width x height is {expected}"
            ),
            WriteInputError::ActiveAreaInvalid => {
                f.write_str("the active area is empty, inverted, or outside the plane")
            }
            WriteInputError::DefaultCropInvalid => {
                f.write_str("the default crop is empty or reaches outside the active area")
            }
            WriteInputError::MaskedAreaInvalid { index } => write!(
                f,
                "masked area {index} is empty, outside the plane, or overlaps the active area"
            ),
            WriteInputError::MaskedAreasNeedFullSensor => f.write_str(
                "masked areas can only be written at full-sensor framing, which keeps the margin",
            ),
            WriteInputError::OrientationInvalid { value } => {
                write!(f, "orientation {value} is not between 1 and 8")
            }
            WriteInputError::BlackLevelCount { expected, actual } => write!(
                f,
                "{actual} BlackLevel value(s) given, the repeat dimensions need {expected}"
            ),
            WriteInputError::BlackLevelRepeatDimInvalid => {
                f.write_str("BlackLevelRepeatDim has a zero dimension")
            }
            WriteInputError::MissingUniqueCameraModel => {
                f.write_str("UniqueCameraModel is required and must not be empty or blank")
            }
            WriteInputError::InvalidAscii { field } => {
                write!(f, "{field} must be printable ASCII of at most 1024 bytes")
            }
            WriteInputError::MalformedOpcodeList { list } => {
                write!(
                    f,
                    "OpcodeList{list}'s opcode headers do not add up to its length"
                )
            }
            WriteInputError::OutputTooLarge => {
                f.write_str("the file would be larger than TIFF's 4 GiB offset limit")
            }
            WriteInputError::OutputBufferWrongLength { expected, actual } => write!(
                f,
                "destination buffer holds {actual} byte(s), expected {expected}"
            ),
        }
    }
}
