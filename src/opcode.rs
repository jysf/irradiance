//! `OpcodeList` byte-stream parser — `SPEC-018`.
//!
//! DNG embeds per-file processing instructions as an `OpcodeList`: a small
//! binary format, always **big-endian regardless of the file's own byte
//! order** (unlike the TIFF payload `src/ifd.rs` reads). This module parses
//! that stream into typed [`Opcode`] values, and extracts
//! [`WarpRectilinear`](Opcode::WarpRectilinear)'s parameters into
//! [`WarpRect`] for [`crate::develop::develop_into`] and
//! [`crate::warp::apply_warp_into`].
//!
//! # Format (DNG 1.7.0.0 Chapter 7, "Opcode List Processing", p.101)
//!
//! ```text
//! OpcodeList := count:u32be, Opcode*
//! Opcode     := id:u32be, version:u32be, flags:u32be, size:u32be, params:[u8; size]
//! ```
//!
//! `flags` bit 0 set means the opcode is optional: "the DNG reader may
//! decide to not apply this opcode if it wishes, or it does not understand
//! the opcode ID." An unset bit 0 on an opcode ID this reader does not
//! recognize is [`Error::UnsupportedMandatoryOpcode`] — applying nothing for
//! a mandatory correction would silently produce a wrong image (`AC3`).
//!
//! # `WarpRectilinear` (OpcodeID 1, p.102-104)
//!
//! ```text
//! Params := N:u32be, (kr0,kr1,kr2,kr3,kt0,kt1: f64be){N}, cx_hat:f64be, cy_hat:f64be
//! ```
//!
//! `cx_hat`/`cy_hat` sit **after every coefficient set**, not per-set — a
//! detail only the spec's own parameter table gets right (`AGENTS.md` §16
//! rule 4, `unrun-docs-carry-errors`: SPEC-018's own `## Context` mis-cited
//! `OpcodeList3`'s IFD tag as `0xC740`, which is actually `OpcodeList1`'s;
//! `TAG_OPCODE_LIST_3` in `src/ifd.rs` already carries the correct `51022` /
//! `0xC74E`, confirmed against DNG 1.7.0.0 p.56-57 and a fresh `exiftool -b
//! -OpcodeList3` read of all three decodable Q2M frames). "N must be 1 or
//! the total number of image planes" (p.102); this crate develops exactly
//! one plane (monochrome), so coefficient set `K1` is always the right one
//! to apply regardless of `N` — "K1 defines the warp function for the first
//! image plane" holds for a single-plane image whatever `N` claims. `planes`
//! is carried through as informational metadata only.
//!
//! # Provenance
//!
//! Written from the published DNG 1.7.0.0 specification only (Adobe,
//! helpx.adobe.com/content/dam/help/en/photoshop/pdf/DNG_Spec_1_7_0_0.pdf),
//! §Chapter 7 and §6.4.1. Not from `dnglab`/`rawler` (LGPL-2.1), which are
//! run as tools only (`provenance-recorded-per-algorithm`). See the row in
//! `docs/provenance-ledger.md`.
//!
//! # Coordination with `SPEC-017`
//!
//! `SPEC-017` (`FixBadPixelsConstant`, `OpcodeID` 4) also lands in this
//! module. `SPEC-018` created it first: [`Opcode::Unknown`] is
//! the escape hatch that let `SPEC-017` add its own variant and match arm
//! to [`parse_opcode_list`] without rewriting it (its own `## Where the
//! opcode module lives` records the same contract from its side).
//!
//! # `FixBadPixelsConstant` (OpcodeID 4, p.95) — `SPEC-017`
//!
//! ```text
//! Params := Constant:u32be, BayerPhase:u32be
//! ```
//!
//! **Chapter correction** (`AGENTS.md` §16 rule 4, `unrun-docs-carry-
//! errors`): `SPEC-017`'s own handoff cited "DNG 1.7.0.0 § Chapter 6" for
//! this opcode. Chapter 6 is "Mapping Camera Color Space to CIE XYZ Space" —
//! unrelated. `FixBadPixelsConstant`, like `WarpRectilinear` above, is
//! documented in **Chapter 7, "Opcode List Processing"** (confirmed by
//! fetching the published DNG 1.6.0.0 PDF directly during this build and
//! reading its table of contents and the `FixBadPixelsConstant` page: opcode
//! framing at p.89, `WarpRectilinear` p.90, `FixBadPixelsConstant` p.95, all
//! under the one "Opcode List Processing" chapter heading). The chapter
//! number was never load-bearing for the parser itself — the byte-level
//! shape was probed against real files (`## Context`,
//! `SPEC-017-fixbadpixelsconstant-opcode.md`) — but a citation this build
//! can verify is corrected rather than carried forward wrong.
//!
//! `Constant` marks a bad pixel: any raw-plane sample equal to `Constant`
//! within `ActiveArea` is replaced. `BayerPhase` (0-3) names which CFA
//! colour the top-left pixel is under DNG's own Bayer-phase convention; on
//! `SamplesPerPixel = 1` (monochrome, every Q2M frame) the field is
//! meaningless and is read (`AC1`) but never branched on (`##
//! Non-Goals`).
//!
//! **The interpolation kernel is this build's own choice, not the DNG
//! spec's.** The spec's own words for this opcode are "patches
//! (interpolates over) bad pixels... The bad pixels are marked... by
//! setting the bad pixels to a value of Constant" — no kernel is specified.
//! `crate::develop::apply_fix_bad_pixels_constant` implements a 3x3
//! median-of-valid-neighbours (pre-registered in `SPEC-017`'s own `## Notes
//! for the Implementer`), provenance class 1 for the opcode identity/marker
//! semantics, and this build's own algorithm (not read from any
//! implementation) for the kernel — see `docs/provenance-ledger.md`.
//!
//! # `GainMap` (OpcodeID 9) — `SPEC-059`, `DEC-069`
//!
//! ```text
//! Params := Top, Left, Bottom, Right, Plane, Planes, RowPitch, ColPitch,
//!           MapPointsV, MapPointsH : u32be,
//!           MapSpacingV, MapSpacingH, MapOriginV, MapOriginH : f64be,
//!           MapPlanes : u32be,
//!           Gain : f32be [MapPointsV][MapPointsH][MapPlanes]
//! ```
//!
//! 76 header bytes, then the gain grid, row-major with the map planes
//! innermost. The parsed view (`GainMapView`) borrows the parameter bytes
//! and copies no gain grid. Every malformed class `DEC-069` §3 names is
//! [`Error::MalformedOpcodeParams`] before any use. The sampling rule lives on
//! `GainMaps::apply`, the one implementation both develop paths and
//! [`crate::develop::apply_gain_maps`] call (`DEC-069` §4).
//!
//! Layout and operation are class 1 (the DNG specification). Five points the
//! specification leaves implicit (pixel-centre sampling, index clamping, the
//! clip at 1, the empty area meaning the whole image, the pitch phase anchored
//! at the area's top-left) were settled by reading Adobe DNG SDK 1.7.1, class
//! 4, with no code copied; see `docs/provenance-ledger.md`.

use crate::ifd::{
    Sensor, TAG_ACTIVE_AREA, TAG_ANALOG_BALANCE, TAG_BLACK_LEVEL, TAG_BLACK_LEVEL_DELTA_H,
    TAG_BLACK_LEVEL_DELTA_V, TAG_BLACK_LEVEL_REPEAT_DIM, TAG_CALIBRATION_ILLUMINANT_1,
    TAG_CALIBRATION_ILLUMINANT_2, TAG_CAMERA_CALIBRATION_1, TAG_CAMERA_CALIBRATION_2,
    TAG_CAMERA_CALIBRATION_SIGNATURE, TAG_CFA_LAYOUT, TAG_CFA_PLANE_COLOR, TAG_COLOR_MATRIX_1,
    TAG_COLOR_MATRIX_2, TAG_COLUMN_INTERLEAVE_FACTOR, TAG_DEFAULT_CROP_ORIGIN,
    TAG_DEFAULT_CROP_SIZE, TAG_DNG_BACKWARD_VERSION, TAG_DNG_VERSION, TAG_FORWARD_MATRIX_1,
    TAG_FORWARD_MATRIX_2, TAG_LINEARIZATION_TABLE, TAG_OPCODE_LIST_1, TAG_OPCODE_LIST_2,
    TAG_OPCODE_LIST_3, TAG_ORIENTATION, TAG_PROFILE_CALIBRATION_SIGNATURE,
    TAG_ROW_INTERLEAVE_FACTOR, TAG_SAMPLE_FORMAT, TAG_SUB_TILE_BLOCK_SIZE, TAG_WHITE_LEVEL,
};
use crate::Error;

/// One opcode from an `OpcodeList`, decoded far enough to be useful.
///
/// **Sealed with `#[non_exhaustive]`** (`SPEC-042`, `DEC-052`, overriding the
/// earlier "`#[non_exhaustive]`-free by choice" note): this enum is grown by
/// whichever opcode spec lands next (`SPEC-017`'s coordination contract), so
/// a new variant must be additive for consumers. A match outside this crate
/// needs a wildcard arm; in-crate matches stay exhaustive on purpose, so a new
/// variant is still a visible compile error here. The seal is on the variant
/// *set* only — each variant can still be built with a literal.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Opcode {
    /// DNG 1.7.0.0 §6.4.1, `OpcodeID` 1 — see [`WarpRect`] for the fields'
    /// meaning. `planes` is `N` from the byte stream, kept for round-trip
    /// fidelity; the applier always uses `kr`/`kt` (coefficient set `K1`)
    /// regardless of its value (module docs, "`WarpRectilinear`").
    WarpRectilinear {
        /// Radial coefficients `kr0..kr3`.
        kr: [f64; 4],
        /// Tangential coefficients `kt0`, `kt1`.
        kt: [f64; 2],
        /// Normalized optical center x, relative to the top-left pixel.
        cx: f64,
        /// Normalized optical center y, relative to the top-left pixel.
        cy: f64,
        /// `N`, the coefficient-set count the file declared.
        planes: u32,
    },
    /// DNG 1.7.0.0 Chapter 7, `OpcodeID` 4, p.95 — `SPEC-017`. Marks every
    /// raw-plane sample equal to `constant` (within `ActiveArea`) as a bad
    /// pixel; `crate::develop::apply_fix_bad_pixels_constant` replaces it
    /// with the median of its valid 3x3 neighbours (module docs,
    /// `FixBadPixelsConstant`).
    FixBadPixelsConstant {
        /// The marker value: any sample equal to this is a bad pixel.
        constant: u32,
        /// DNG's Bayer-phase convention (0-3); meaningless and unread on a
        /// monochrome plane (`## Non-Goals`).
        bayer_phase: u32,
    },
    /// An opcode ID this module does not parse into its own variant —
    /// genuinely unknown, or a known ID declaring a DNG version newer than
    /// this reader supports (its parameters are then not interpreted). Only
    /// ever produced when `flags` bit 0 (optional) is set; an unset bit is
    /// [`Error::UnsupportedMandatoryOpcode`] (or, for a too-new version,
    /// [`Error::UnsupportedOpcodeVersion`]) instead of a value.
    Unknown {
        /// The opcode ID.
        id: u32,
        /// The raw `Flags` word.
        flags: u32,
        /// The raw parameter bytes, unparsed.
        params: Vec<u8>,
    },
    /// DNG 1.7.0.0 Chapter 7, `OpcodeID` 9 — `SPEC-059`, `DEC-069`. A
    /// per-area, per-pitch gain from a bilinearly interpolated grid; the
    /// develop paths apply every one in `OpcodeList2` (module docs,
    /// "`GainMap`"). The fields are the parameter block's, in order.
    GainMap {
        /// Area top, in stage-2 (`ActiveArea`) rows.
        top: u32,
        /// Area left, in stage-2 columns.
        left: u32,
        /// Area bottom (exclusive).
        bottom: u32,
        /// Area right (exclusive).
        right: u32,
        /// The first image plane the map applies to.
        plane: u32,
        /// How many image planes it applies to.
        planes: u32,
        /// Apply to every `row_pitch`-th row from `top`.
        row_pitch: u32,
        /// Apply to every `col_pitch`-th column from `left`.
        col_pitch: u32,
        /// Grid rows.
        points_v: u32,
        /// Grid columns.
        points_h: u32,
        /// Grid row spacing, in units of the image height.
        spacing_v: f64,
        /// Grid column spacing, in units of the image width.
        spacing_h: f64,
        /// Grid origin, in units of the image height.
        origin_v: f64,
        /// Grid origin, in units of the image width.
        origin_h: f64,
        /// Gain planes per grid point.
        map_planes: u32,
        /// `points_v · points_h · map_planes` gains, row-major, map planes
        /// innermost, as the file stores them.
        gains: Vec<f32>,
    },
}

/// Parsed `WarpRectilinear` parameters (DNG 1.7.0.0 §6.4.1), ready for
/// [`crate::warp::apply_warp_into`].
///
/// A separate, smaller type than [`Opcode::WarpRectilinear`] so
/// [`crate::develop`] and [`crate::warp`] depend on this shape only, not on
/// the whole [`Opcode`] enum SPEC-017 also extends.
///
/// **A parser-output view, sealed with `#[non_exhaustive]`** (`SPEC-042`,
/// `DEC-052`): the parser builds it and consumers read it, so a future field
/// is additive, not breaking. Outside this crate a `WarpRect` cannot be built
/// with a struct literal; its public fields stay readable and assignable.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct WarpRect {
    /// Radial coefficients `kr0..kr3`: `f(r) = kr0 + kr1*r^2 + kr2*r^4 + kr3*r^6`.
    pub kr: [f64; 4],
    /// Tangential coefficients `kt0`, `kt1`. Every Q2M frame measures these
    /// as `0.0`; [`crate::warp::apply_warp_into`] implements the pure-radial
    /// path only and reports a non-zero value rather than guessing
    /// (`Error::UnsupportedWarpTangentialTerms`).
    pub kt: [f64; 2],
    /// Normalized optical center x, relative to the top-left pixel
    /// (`0.5` = image center).
    pub cx: f64,
    /// Normalized optical center y, relative to the top-left pixel.
    pub cy: f64,
    /// `N` as read from the byte stream — informational only; see the
    /// module docs for why the applier does not need to branch on it.
    pub planes: u32,
}

impl WarpRect {
    /// The warp moved `alpha` of the way from the identity to `self`
    /// (`SPEC-064`, `DEC-059` §1): its backward radial scale is
    /// `f_α(r) = (1 − α) + α·f(r)`, so every pixel's displacement
    /// `(1 − f(r))·(x − c)` is exactly `alpha` times the full one.
    ///
    /// - `kr0' = (1 − α) + α·kr0`; `kr1..kr3` and `kt0`, `kt1` are each
    ///   multiplied by `α`;
    /// - `cx`, `cy` and `planes` are unchanged.
    ///
    /// `lerp(1.0)` returns `self` bit for bit (`1 − 1 = 0`, `0 + kr0 = kr0`,
    /// `1·k = k`). `lerp(0.0)` returns the identity coefficients
    /// `kr = [1, 0, 0, 0]`, `kt = [0, 0]` (some zeros may be `-0.0`, which
    /// compares equal), which the develop path skips like an identity
    /// opcode.
    ///
    /// Pure arithmetic: `alpha` is **not** validated, and a value outside
    /// `[0, 1]` extrapolates. [`crate::develop::DevelopOptions::warp_alpha`]
    /// is where the domain is enforced.
    #[must_use]
    pub fn lerp(&self, alpha: f64) -> WarpRect {
        // Exactly this form for kr0: any rearrangement rounds differently
        // and would break the alpha = 1 bit identity.
        let [kr0, kr1, kr2, kr3] = self.kr;
        let [kt0, kt1] = self.kt;
        WarpRect {
            kr: [
                (1.0 - alpha) + alpha * kr0,
                alpha * kr1,
                alpha * kr2,
                alpha * kr3,
            ],
            kt: [alpha * kt0, alpha * kt1],
            cx: self.cx,
            cy: self.cy,
            planes: self.planes,
        }
    }

    /// An identity `WarpRect` (`kr = [1, 0, 0, 0]`, no tangential terms,
    /// centred, one plane) for the crate's own integration tests and examples
    /// — **dev-only**, compiled in solely under the `test-fixtures` cargo
    /// feature (`SPEC-042`, `DEC-052`; the same shape as
    /// [`crate::ifd::Sensor::test_fixture`]).
    ///
    /// `#[non_exhaustive]` stops the separate `tests/` crate from writing a
    /// `WarpRect { .. }` literal, so fixtures start here and assign the public
    /// fields they exercise. The published crate (default features) does not
    /// contain it.
    #[cfg(feature = "test-fixtures")]
    #[doc(hidden)]
    pub fn test_fixture() -> WarpRect {
        WarpRect {
            kr: [1.0, 0.0, 0.0, 0.0],
            kt: [0.0, 0.0],
            cx: 0.5,
            cy: 0.5,
            planes: 1,
        }
    }
}

/// Read a big-endian `u32` at `at`, bounds-checked.
///
/// Opcode lists are always big-endian (module docs) — a fixed reader here,
/// unlike `src/ifd.rs`'s [`crate::ifd::ByteOrder`]-parameterized one, which
/// exists because the TIFF payload can legally be either order.
fn read_u32_be(bytes: &[u8], at: usize) -> Result<u32, Error> {
    let end = at.checked_add(4).ok_or(Error::Truncated { at, len: 4 })?;
    let chunk: [u8; 4] = bytes
        .get(at..end)
        .ok_or(Error::Truncated { at, len: 4 })?
        .try_into()
        .map_err(|_| Error::Truncated { at, len: 4 })?;
    Ok(u32::from_be_bytes(chunk))
}

/// Read a big-endian `f64` (DNG `DOUBLE`) at `at`, bounds-checked.
fn read_f64_be(bytes: &[u8], at: usize) -> Result<f64, Error> {
    let end = at.checked_add(8).ok_or(Error::Truncated { at, len: 8 })?;
    let chunk: [u8; 8] = bytes
        .get(at..end)
        .ok_or(Error::Truncated { at, len: 8 })?
        .try_into()
        .map_err(|_| Error::Truncated { at, len: 8 })?;
    Ok(f64::from_be_bytes(chunk))
}

/// `OpcodeID` 1, DNG 1.7.0.0 §6.4.1.
const OPCODE_ID_WARP_RECTILINEAR: u32 = 1;

/// `OpcodeID` 4, DNG 1.7.0.0 Chapter 7 p.95 — `SPEC-017`. NOT 5
/// (`FixBadPixelsList`, a different, unimplemented opcode — `##
/// Non-Goals`); confirmed against the probed Q2M bytes and the published
/// spec, both agreeing on 4.
const OPCODE_ID_FIX_BAD_PIXELS_CONSTANT: u32 = 4;

/// `Flags` bit 0: "the opcode is considered optional" (p.101).
const FLAG_OPTIONAL: u32 = 1;

/// Parse `WarpRectilinear`'s parameter block (already sliced to its own
/// `DataSize` by the caller) into a [`WarpRect`].
///
/// Validates that the declared `N` implies EXACTLY `params.len()` bytes
/// (`4 + 48*N + 16`) before reading a single coefficient — a mismatch is
/// [`Error::MalformedOpcodeParams`], not a best-effort guess. `cx_hat`/
/// `cy_hat` sit after every one of the `N` coefficient sets; only `K1`
/// (the first 48 bytes after `N`) is extracted, per the module docs. A
/// non-finite `K1` coefficient or optical centre is also
/// [`Error::MalformedOpcodeParams`].
fn parse_warp_rectilinear_params(params: &[u8], id: u32) -> Result<WarpRect, Error> {
    let malformed = |declared_size: usize| Error::MalformedOpcodeParams {
        id,
        declared_size: u32::try_from(declared_size).unwrap_or(u32::MAX),
    };

    let n = read_u32_be(params, 0).map_err(|_| malformed(params.len()))?;
    let coeff_bytes = u64::from(n)
        .checked_mul(48)
        .ok_or_else(|| malformed(params.len()))?;
    let required = 4u64
        .checked_add(coeff_bytes)
        .and_then(|v| v.checked_add(16))
        .ok_or_else(|| malformed(params.len()))?;
    if u64::try_from(params.len()).unwrap_or(u64::MAX) != required {
        return Err(malformed(params.len()));
    }

    // K1 starts right after N (offset 4), regardless of N — see module docs.
    let kr0 = read_f64_be(params, 4).map_err(|_| malformed(params.len()))?;
    let kr1 = read_f64_be(params, 12).map_err(|_| malformed(params.len()))?;
    let kr2 = read_f64_be(params, 20).map_err(|_| malformed(params.len()))?;
    let kr3 = read_f64_be(params, 28).map_err(|_| malformed(params.len()))?;
    let kt0 = read_f64_be(params, 36).map_err(|_| malformed(params.len()))?;
    let kt1 = read_f64_be(params, 44).map_err(|_| malformed(params.len()))?;

    // cx, cy sit AFTER all N coefficient sets: offset 4 + 48*N.
    let tail_offset = 4u64
        .checked_add(coeff_bytes)
        .ok_or_else(|| malformed(params.len()))?;
    let tail_offset = usize::try_from(tail_offset).map_err(|_| malformed(params.len()))?;
    let cx = read_f64_be(params, tail_offset).map_err(|_| malformed(params.len()))?;
    let cy_offset = tail_offset
        .checked_add(8)
        .ok_or_else(|| malformed(params.len()))?;
    let cy = read_f64_be(params, cy_offset).map_err(|_| malformed(params.len()))?;

    // A NaN or infinite coefficient or optical centre has no geometric
    // meaning: applied, it turns every source coordinate into NaN and the
    // sampler silently writes black. Refuse it here, once, so no develop path
    // (default crop or full active area) ever sees one.
    let all_finite = [kr0, kr1, kr2, kr3, kt0, kt1, cx, cy]
        .iter()
        .all(|v| v.is_finite());
    if !all_finite {
        return Err(malformed(params.len()));
    }

    Ok(WarpRect {
        kr: [kr0, kr1, kr2, kr3],
        kt: [kt0, kt1],
        cx,
        cy,
        planes: n,
    })
}

/// Parse `FixBadPixelsConstant`'s parameter block (already sliced to its own
/// `DataSize` by the caller): `Constant:u32be, BayerPhase:u32be` — DNG
/// 1.7.0.0 Chapter 7 p.95, an exact 8 bytes, no variable-length tail (unlike
/// `WarpRectilinear`'s `N`-dependent shape above). A declared size other
/// than 8 is [`Error::MalformedOpcodeParams`], not a best-effort guess.
fn parse_fix_bad_pixels_constant_params(params: &[u8], id: u32) -> Result<(u32, u32), Error> {
    let malformed = || Error::MalformedOpcodeParams {
        id,
        declared_size: u32::try_from(params.len()).unwrap_or(u32::MAX),
    };
    if params.len() != 8 {
        return Err(malformed());
    }
    let constant = read_u32_be(params, 0).map_err(|_| malformed())?;
    let bayer_phase = read_u32_be(params, 4).map_err(|_| malformed())?;
    Ok((constant, bayer_phase))
}

/// `OpcodeID` 9, DNG 1.7.0.0 Chapter 7 — `SPEC-059`.
const OPCODE_ID_GAIN_MAP: u32 = 9;

/// The `GainMap` parameter header: ten `u32`, four `f64`, one `u32`.
const GAIN_MAP_HEADER_LEN: usize = 76;

/// The most `GainMap`s one list may hold (`DEC-069` §3). The develop paths
/// keep their maps in a fixed array this long, so the mono scratch develop
/// allocates nothing (`DEC-060`).
pub(crate) const MAX_GAIN_MAPS: usize = 16;

/// A parsed `GainMap`, borrowing its parameter bytes (`SPEC-059`): the
/// header fields are decoded, the gain grid is read in place by
/// [`GainMapView::entry`]. Every field is validated by
/// [`parse_gain_map_params`] (`DEC-069` §3).
#[derive(Debug, Clone, Copy)]
pub(crate) struct GainMapView<'a> {
    pub(crate) top: u32,
    pub(crate) left: u32,
    pub(crate) bottom: u32,
    pub(crate) right: u32,
    pub(crate) plane: u32,
    pub(crate) planes: u32,
    pub(crate) row_pitch: u32,
    pub(crate) col_pitch: u32,
    pub(crate) points_v: u32,
    pub(crate) points_h: u32,
    pub(crate) spacing_v: f64,
    pub(crate) spacing_h: f64,
    pub(crate) origin_v: f64,
    pub(crate) origin_h: f64,
    pub(crate) map_planes: u32,
    /// The whole parameter block; gains start at [`GAIN_MAP_HEADER_LEN`].
    params: &'a [u8],
}

impl GainMapView<'_> {
    /// The gain at grid row `i`, column `j`, map plane `p`: the `f32` at
    /// `76 + 4·((i·points_h + j)·map_planes + p)`. `None` for an index outside
    /// the grid.
    pub(crate) fn entry(&self, i: u32, j: u32, p: u32) -> Option<f32> {
        if i >= self.points_v || j >= self.points_h || p >= self.map_planes {
            return None;
        }
        let index = u64::from(i)
            .checked_mul(u64::from(self.points_h))?
            .checked_add(u64::from(j))?
            .checked_mul(u64::from(self.map_planes))?
            .checked_add(u64::from(p))?
            .checked_mul(4)?
            .checked_add(GAIN_MAP_HEADER_LEN as u64)?;
        let at = usize::try_from(index).ok()?;
        read_f32_be(self.params, at).ok()
    }
}

/// Read a big-endian `f32` (DNG `FLOAT`) at `at`, bounds-checked.
fn read_f32_be(bytes: &[u8], at: usize) -> Result<f32, Error> {
    read_u32_be(bytes, at).map(f32::from_bits)
}

/// Parse `GainMap`'s parameter block (already sliced to its own `DataSize`)
/// into a borrowed [`GainMapView`], checking every `DEC-069` §3 class first:
/// the exact length `76 + 4·MapPointsV·MapPointsH·MapPlanes` (checked
/// arithmetic), non-zero pitches, point counts, `Planes` and `MapPlanes`,
/// finite positive spacings, finite origins and finite gains. Any failure is
/// [`Error::MalformedOpcodeParams`].
pub(crate) fn parse_gain_map_params(params: &[u8], id: u32) -> Result<GainMapView<'_>, Error> {
    let malformed = || Error::MalformedOpcodeParams {
        id,
        declared_size: u32::try_from(params.len()).unwrap_or(u32::MAX),
    };
    let u = |at: usize| read_u32_be(params, at).map_err(|_| malformed());
    let f = |at: usize| read_f64_be(params, at).map_err(|_| malformed());

    let points_v = u(32)?;
    let points_h = u(36)?;
    let map_planes = u(72)?;
    let required = u64::from(points_v)
        .checked_mul(u64::from(points_h))
        .and_then(|n| n.checked_mul(u64::from(map_planes)))
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add(GAIN_MAP_HEADER_LEN as u64))
        .ok_or_else(malformed)?;
    if u64::try_from(params.len()).unwrap_or(u64::MAX) != required {
        return Err(malformed());
    }

    let view = GainMapView {
        top: u(0)?,
        left: u(4)?,
        bottom: u(8)?,
        right: u(12)?,
        plane: u(16)?,
        planes: u(20)?,
        row_pitch: u(24)?,
        col_pitch: u(28)?,
        points_v,
        points_h,
        spacing_v: f(40)?,
        spacing_h: f(48)?,
        origin_v: f(56)?,
        origin_h: f(64)?,
        map_planes,
        params,
    };
    let positive = |s: f64| s.is_finite() && s > 0.0;
    if view.row_pitch == 0
        || view.col_pitch == 0
        || points_v == 0
        || points_h == 0
        || view.planes == 0
        || map_planes == 0
        || !positive(view.spacing_v)
        || !positive(view.spacing_h)
        || !view.origin_v.is_finite()
        || !view.origin_h.is_finite()
    {
        return Err(malformed());
    }
    // Every gain is finite. The length check above makes the grid exactly
    // the bytes after the header, a whole number of `f32`s.
    let grid = params.get(GAIN_MAP_HEADER_LEN..).ok_or_else(malformed)?;
    let (gains, _rest) = grid.as_chunks::<4>();
    if gains
        .iter()
        .any(|bytes| !f32::from_be_bytes(*bytes).is_finite())
    {
        return Err(malformed());
    }
    Ok(view)
}

/// One `GainMap`, prepared for the per-sample apply: the area already
/// resolved against the stage-2 bounds (`DEC-069` §2).
#[derive(Debug, Clone, Copy)]
struct PreparedMap<'a> {
    view: GainMapView<'a>,
    /// The touched rows `[row_start, row_end)` and columns
    /// `[col_start, col_end)`, intersected with the image. Empty when the map
    /// applies to no sample (outside the image, or `Plane > 0`).
    row_start: u32,
    row_end: u32,
    col_start: u32,
    col_end: u32,
    /// The pitch phase anchor: the area's `Top`/`Left` before the
    /// intersection, or `(0, 0)` for an empty area (the whole image).
    anchor_row: u32,
    anchor_col: u32,
}

/// The `GainMap`s of one `OpcodeList2`, in list order, for an `H × W`
/// stage-2 (`ActiveArea`-sized) image — `SPEC-059`, `DEC-069` §4.
///
/// Up to [`MAX_GAIN_MAPS`] views in a fixed array: building one allocates
/// nothing. [`GainMaps::apply`] is the one implementation of `DEC-069` §2;
/// both develop paths and [`crate::develop::apply_gain_maps`] call it.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GainMaps<'a> {
    maps: [Option<PreparedMap<'a>>; MAX_GAIN_MAPS],
    count: usize,
    height: u32,
    width: u32,
}

impl<'a> GainMaps<'a> {
    /// No maps: every sample passes through unchanged.
    pub(crate) fn none(width: u32, height: u32) -> GainMaps<'a> {
        GainMaps {
            maps: [None; MAX_GAIN_MAPS],
            count: 0,
            height,
            width,
        }
    }

    /// Every `GainMap` in `list` (an `OpcodeList` byte stream), for a
    /// `width × height` stage-2 image. Other opcodes are framed and skipped:
    /// whether they may be skipped is [`check_required_processing`]'s
    /// question, not this one's.
    ///
    /// Every `GainMap`'s parameters are validated, optional or not. More than
    /// [`MAX_GAIN_MAPS`] is [`Error::TooManyGainMaps`] with the list's full
    /// count, reported after the whole list is read (so a malformed map
    /// anywhere is reported first).
    pub(crate) fn from_list(list: &'a [u8], width: u32, height: u32) -> Result<Self, Error> {
        let mut maps = GainMaps::none(width, height);
        let count = read_u32_be(list, 0)?;
        let mut pos: usize = 4;
        let mut found: u32 = 0;
        for _ in 0..count {
            let raw = read_raw_opcode(list, &mut pos)?;
            // A GainMap declaring a newer DNG version is not one this reader
            // understands: `check_required_processing` refuses it when
            // mandatory, and an optional one is skipped like any unknown.
            if raw.known_id() != Some(OPCODE_ID_GAIN_MAP) {
                continue;
            }
            let (id, params) = (raw.id, raw.params);
            let view = parse_gain_map_params(params, id)?;
            if let Some(slot) = maps.maps.get_mut(maps.count) {
                *slot = Some(prepare(view, width, height));
                maps.count = maps.count.saturating_add(1);
            }
            found = found.saturating_add(1);
        }
        if usize::try_from(found).unwrap_or(usize::MAX) > MAX_GAIN_MAPS {
            return Err(Error::TooManyGainMaps { count: found });
        }
        Ok(maps)
    }

    /// How many `GainMap`s the list held (every one counts as applied,
    /// `DEC-069` §2, including one that touches no sample).
    pub(crate) fn len(&self) -> usize {
        self.count
    }

    /// `true` when the list held no `GainMap`.
    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Apply every map that touches stage-2 sample `(r, c)` to its normalised
    /// value `v`, in list order, rounding to `u16` after each (`DEC-069` §2).
    /// Computed in `f64`; allocates nothing.
    #[inline]
    pub(crate) fn apply(&self, r: u32, c: u32, v: u16) -> u16 {
        let mut value = v;
        for map in self.maps.iter().flatten() {
            if r < map.row_start || r >= map.row_end || c < map.col_start || c >= map.col_end {
                continue;
            }
            let on_row = r
                .checked_sub(map.anchor_row)
                .and_then(|d| d.checked_rem(map.view.row_pitch))
                == Some(0);
            let on_col = c
                .checked_sub(map.anchor_col)
                .and_then(|d| d.checked_rem(map.view.col_pitch))
                == Some(0);
            if !on_row || !on_col {
                continue;
            }
            let g = gain_at(&map.view, r, c, self.height, self.width);
            // `v' = ⌊clamp(v / 65535 · g, 0, 1) · 65535 + 0.5⌋`: clipped at
            // white, rounded half up. In `[0, 65535]`, so the cast is exact.
            let scaled = (f64::from(value) / 65535.0 * g).clamp(0.0, 1.0);
            value = (scaled * 65535.0 + 0.5).floor() as u16;
        }
        value
    }
}

/// Resolve `view`'s area against a `width × height` image (`DEC-069` §2): an
/// empty area (`Top ≥ Bottom` or `Left ≥ Right`) is the whole image, phase
/// anchored at `(0, 0)`; otherwise the area intersected with the image,
/// phase anchored at its own `Top`/`Left`. `Plane > 0` touches nothing.
fn prepare(view: GainMapView<'_>, width: u32, height: u32) -> PreparedMap<'_> {
    let empty_area = view.top >= view.bottom || view.left >= view.right;
    let (row_start, row_end, col_start, col_end, anchor_row, anchor_col) = if empty_area {
        (0, height, 0, width, 0, 0)
    } else {
        (
            view.top,
            view.bottom.min(height),
            view.left,
            view.right.min(width),
            view.top,
            view.left,
        )
    };
    // Every develop path here has one plane, plane 0.
    let row_end = if view.plane == 0 { row_end } else { 0 };
    PreparedMap {
        view,
        row_start,
        row_end,
        col_start,
        col_end,
        anchor_row,
        anchor_col,
    }
}

/// The bilinear gain of `view` at stage-2 sample `(r, c)` of an `h × w`
/// image (`DEC-069` §2): sampled at the pixel centre, each index clamped at
/// the grid edge, rows interpolated first, then columns, in `f64`. The map
/// plane is `min(Plane, MapPlanes − 1)`.
fn gain_at(view: &GainMapView<'_>, r: u32, c: u32, h: u32, w: u32) -> f64 {
    let fr = ((f64::from(r) + 0.5) / f64::from(h) - view.origin_v) / view.spacing_v;
    let fc = ((f64::from(c) + 0.5) / f64::from(w) - view.origin_h) / view.spacing_h;
    let (i0, i1, tr) = grid_axis(fr, view.points_v);
    let (j0, j1, tc) = grid_axis(fc, view.points_h);
    let p = view.plane.min(view.map_planes.saturating_sub(1));
    // A validated grid has every one of these; 1.0 keeps this total.
    let g = |i, j| f64::from(view.entry(i, j, p).unwrap_or(1.0));
    let left = g(i0, j0) * (1.0 - tr) + g(i1, j0) * tr;
    let right = g(i0, j1) * (1.0 - tr) + g(i1, j1) * tr;
    left * (1.0 - tc) + right * tc
}

/// One grid axis: `f ≤ 0` takes index 0 and `f ≥ points − 1` the last, each
/// with fraction 0; otherwise `⌊f⌋`, `⌊f⌋ + 1` and the fraction `f − ⌊f⌋`.
fn grid_axis(f: f64, points: u32) -> (u32, u32, f64) {
    let last = points.saturating_sub(1);
    if f.is_nan() || f <= 0.0 {
        return (0, 0, 0.0);
    }
    if f >= f64::from(last) {
        return (last, last, 0.0);
    }
    let floor = f.floor();
    // `0 ≤ floor < last ≤ u32::MAX`, so the cast is exact.
    let i = floor as u32;
    (i, i.saturating_add(1).min(last), f - floor)
}

/// The newest DNG specification version whose opcodes this reader
/// understands: 1.7.1.0, as the opcode header's `DNGSpecVersion` bytes
/// `1, 7, 1, 0` read big-endian.
///
/// DNG 1.7.0.0 p.105: a reader must not process an opcode whose declared
/// version is newer than it supports. Such an opcode is treated exactly like
/// an unrecognized ID — skipped when optional, refused when mandatory
/// ([`Error::UnsupportedOpcodeVersion`]) — whatever its `OpcodeID`.
const MAX_SUPPORTED_OPCODE_VERSION: u32 = 0x0107_0100;

/// One opcode's framing, its parameter block sliced but not interpreted.
struct RawOpcode<'a> {
    id: u32,
    version: u32,
    flags: u32,
    params: &'a [u8],
}

impl RawOpcode<'_> {
    /// `id`, unless the opcode declares a DNG version newer than
    /// [`MAX_SUPPORTED_OPCODE_VERSION`]: then `None`, so every dispatcher
    /// treats it as unrecognized and never interprets its parameters.
    fn known_id(&self) -> Option<u32> {
        (self.version <= MAX_SUPPORTED_OPCODE_VERSION).then_some(self.id)
    }

    /// The refusal for this opcode when it is mandatory and not applied
    /// because this reader does not understand it.
    fn unsupported(&self) -> Error {
        if self.version > MAX_SUPPORTED_OPCODE_VERSION {
            Error::UnsupportedOpcodeVersion {
                id: self.id,
                version: self.version,
            }
        } else {
            Error::UnsupportedMandatoryOpcode { id: self.id }
        }
    }

    /// Whether `Flags` bit 0 (optional) is set.
    fn optional(&self) -> bool {
        self.flags & FLAG_OPTIONAL != 0
    }
}

/// Read the opcode framing (`id, version, flags, size, params`) at `*pos` and
/// advance `*pos` past it — the one reader [`parse_opcode_list`] and
/// [`check_required_processing`] share, so both see identical framing and
/// identical framing errors.
///
/// # Errors
///
/// [`Error::Truncated`] if a header field or the declared parameter block runs
/// past the end of `bytes`; [`Error::MalformedOpcodeParams`] if the declared
/// size cannot be represented as a `usize`.
fn read_raw_opcode<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<RawOpcode<'a>, Error> {
    let mut at = *pos;
    let id = read_u32_be(bytes, at)?;
    at = at.checked_add(4).ok_or(Error::Truncated { at, len: 4 })?;
    let version = read_u32_be(bytes, at)?;
    at = at.checked_add(4).ok_or(Error::Truncated { at, len: 4 })?;
    let flags = read_u32_be(bytes, at)?;
    at = at.checked_add(4).ok_or(Error::Truncated { at, len: 4 })?;
    let size = read_u32_be(bytes, at)?;
    at = at.checked_add(4).ok_or(Error::Truncated { at, len: 4 })?;

    let size_usize = usize::try_from(size).map_err(|_| Error::MalformedOpcodeParams {
        id,
        declared_size: size,
    })?;
    let params_end = at.checked_add(size_usize).ok_or(Error::Truncated {
        at,
        len: size_usize,
    })?;
    let params = bytes.get(at..params_end).ok_or(Error::Truncated {
        at,
        len: size_usize,
    })?;
    *pos = params_end;
    Ok(RawOpcode {
        id,
        version,
        flags,
        params,
    })
}

/// Parse a complete `OpcodeList` byte stream (DNG 1.7.0.0 Chapter 7).
///
/// Panic-free on adversarial input (`no-panics-on-untrusted-input`): every
/// offset is `checked_add`ed, every slice read is `.get()`-bounds-checked,
/// and the opcode `count` field is never used to pre-allocate — a huge
/// declared count with insufficient bytes fails on the first entry read
/// rather than looping or allocating unboundedly.
///
/// # Errors
///
/// - [`Error::Truncated`] if the stream ends before a declared field or
///   parameter block can be read.
/// - [`Error::MalformedOpcodeParams`] if a recognized opcode ID's own
///   parameter layout does not match its declared `DataSize`.
/// - [`Error::UnsupportedMandatoryOpcode`] if an opcode ID this module does
///   not recognize has `Flags` bit 0 (optional) unset.
/// - [`Error::UnsupportedOpcodeVersion`] if a mandatory opcode declares a DNG
///   version newer than this reader supports (DNG 1.7.0.0 p.105), whatever
///   its ID; an optional one is kept as [`Opcode::Unknown`].
pub fn parse_opcode_list(bytes: &[u8]) -> Result<Vec<Opcode>, Error> {
    let count = read_u32_be(bytes, 0)?;
    let mut pos: usize = 4;

    let mut opcodes = Vec::new();
    for _ in 0..count {
        let raw = read_raw_opcode(bytes, &mut pos)?;
        let RawOpcode {
            id, flags, params, ..
        } = raw;

        match raw.known_id() {
            Some(OPCODE_ID_WARP_RECTILINEAR) => {
                let warp = parse_warp_rectilinear_params(params, id)?;
                opcodes.push(Opcode::WarpRectilinear {
                    kr: warp.kr,
                    kt: warp.kt,
                    cx: warp.cx,
                    cy: warp.cy,
                    planes: warp.planes,
                });
            }
            Some(OPCODE_ID_FIX_BAD_PIXELS_CONSTANT) => {
                let (constant, bayer_phase) = parse_fix_bad_pixels_constant_params(params, id)?;
                opcodes.push(Opcode::FixBadPixelsConstant {
                    constant,
                    bayer_phase,
                });
            }
            Some(OPCODE_ID_GAIN_MAP) => {
                let view = parse_gain_map_params(params, id)?;
                let grid = params.get(GAIN_MAP_HEADER_LEN..).unwrap_or(&[]);
                let (chunks, _rest) = grid.as_chunks::<4>();
                opcodes.push(Opcode::GainMap {
                    top: view.top,
                    left: view.left,
                    bottom: view.bottom,
                    right: view.right,
                    plane: view.plane,
                    planes: view.planes,
                    row_pitch: view.row_pitch,
                    col_pitch: view.col_pitch,
                    points_v: view.points_v,
                    points_h: view.points_h,
                    spacing_v: view.spacing_v,
                    spacing_h: view.spacing_h,
                    origin_v: view.origin_v,
                    origin_h: view.origin_h,
                    map_planes: view.map_planes,
                    gains: chunks.iter().map(|b| f32::from_be_bytes(*b)).collect(),
                });
            }
            _ => {
                if raw.optional() {
                    opcodes.push(Opcode::Unknown {
                        id,
                        flags,
                        params: params.to_vec(),
                    });
                } else {
                    return Err(raw.unsupported());
                }
            }
        }
    }

    Ok(opcodes)
}

/// A recognised opcode, parsed from its parameter block, as the two
/// first-instance lookups below see it. `FixBadPixelsConstant`'s `BayerPhase`
/// is validated by the parse and then dropped: no lookup returns it.
enum Recognised {
    WarpRectilinear(WarpRect),
    FixBadPixelsConstant {
        constant: u32,
    },
    /// Parsed and validated, then dropped: the develop paths read `GainMap`s
    /// from `OpcodeList2` through [`GainMaps`], never through these lookups.
    GainMap,
}

/// Walk an `OpcodeList` byte stream with no allocation (`SPEC-065`,
/// `DEC-060` §2) and return the first recognised opcode `pick` accepts.
///
/// Exactly [`parse_opcode_list`]'s dispatch, minus its `Vec`: every opcode in
/// the list is read, the parameter block of **every** recognised opcode is
/// parsed (so a malformed later one is still an error), an unrecognised
/// mandatory ID is refused, and an optional unknown is skipped without
/// copying its parameters. The result is therefore identical to
/// `parse_opcode_list(bytes)` followed by `find_map(pick)`, errors included —
/// `tests/opcode.rs::walkers_equal_parse_opcode_list_then_find_map` pins that.
/// It does **not** stop at the first match.
fn first_recognised<T>(
    bytes: &[u8],
    mut pick: impl FnMut(Recognised) -> Option<T>,
) -> Result<Option<T>, Error> {
    let count = read_u32_be(bytes, 0)?;
    let mut pos: usize = 4;

    let mut found = None;
    for _ in 0..count {
        let raw = read_raw_opcode(bytes, &mut pos)?;
        let RawOpcode { id, params, .. } = raw;
        let recognised = match raw.known_id() {
            Some(OPCODE_ID_WARP_RECTILINEAR) => {
                Recognised::WarpRectilinear(parse_warp_rectilinear_params(params, id)?)
            }
            Some(OPCODE_ID_FIX_BAD_PIXELS_CONSTANT) => {
                let (constant, _bayer_phase) = parse_fix_bad_pixels_constant_params(params, id)?;
                Recognised::FixBadPixelsConstant { constant }
            }
            Some(OPCODE_ID_GAIN_MAP) => {
                // Validated like every recognised opcode; no lookup returns it.
                parse_gain_map_params(params, id)?;
                Recognised::GainMap
            }
            _ => {
                if raw.optional() {
                    continue;
                }
                return Err(raw.unsupported());
            }
        };
        if found.is_none() {
            found = pick(recognised);
        }
    }

    Ok(found)
}

/// Parse an `OpcodeList` byte stream and return its `WarpRectilinear`
/// opcode, if any — the entry point [`crate::develop::develop_into`] calls.
///
/// `None` when the list contains no `WarpRectilinear` opcode (a well-formed
/// list is not an error just because this camera's `OpcodeList3` carries a
/// different opcode, or none at all). The first instance wins.
///
/// **Allocates nothing** (`SPEC-065`, `DEC-060` §2): it walks the raw framing
/// rather than building [`parse_opcode_list`]'s `Vec`, with an identical
/// result — every opcode is still read and every recognised one's parameters
/// still validated.
///
/// # Errors
///
/// Exactly what [`parse_opcode_list`] returns for the same bytes.
pub fn parse_warp_rectilinear(bytes: &[u8]) -> Result<Option<WarpRect>, Error> {
    first_recognised(bytes, |op| match op {
        Recognised::WarpRectilinear(warp) => Some(warp),
        Recognised::FixBadPixelsConstant { .. } | Recognised::GainMap => None,
    })
}

/// Parse an `OpcodeList` byte stream and return its `FixBadPixelsConstant`
/// opcode's `constant` marker value, if any — the entry point
/// [`crate::develop::develop_into`] calls (`SPEC-017`).
///
/// `None` when the list contains no `FixBadPixelsConstant` opcode. Mirrors
/// [`parse_warp_rectilinear`]'s shape, including the first instance winning
/// and allocating nothing (`SPEC-065`); `bayer_phase` is not returned because
/// no caller consumes it (module docs, `## Non-Goals` — `AC1`'s round-trip
/// test is what reads it, satisfying the "unread field" rule without
/// forwarding it downstream).
///
/// # Errors
///
/// Exactly what [`parse_opcode_list`] returns for the same bytes — including
/// [`Error::UnsupportedMandatoryOpcode`] if `bytes` carries a DIFFERENT,
/// unrecognized mandatory opcode (`AC6`).
pub fn parse_fix_bad_pixels_constant(bytes: &[u8]) -> Result<Option<u32>, Error> {
    first_recognised(bytes, |op| match op {
        Recognised::FixBadPixelsConstant { constant } => Some(constant),
        Recognised::WarpRectilinear(_) | Recognised::GainMap => None,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Required-processing validation — `SPEC-054`, `DEC-049`
// ─────────────────────────────────────────────────────────────────────────────

/// Which develop path is asking, and so which opcodes it applies (`DEC-049`)
/// and which malformed tags it refuses (`DEC-078` §2).
///
/// Passed to [`check_required_processing`] by anyone composing the stage
/// primitives themselves (`SPEC-056`, `DEC-053`). `#[non_exhaustive]`: a later
/// path (or a later opcode a path learns to apply) is an addition, not a break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DevelopPath {
    /// The monochrome path, [`crate::develop::develop_into`]. It applies the
    /// first `FixBadPixelsConstant` in `OpcodeList1`, **every** `GainMap` in
    /// `OpcodeList2` (`SPEC-059`, `DEC-069`) and the first `WarpRectilinear`
    /// in `OpcodeList3`.
    Mono,
    /// The colour path: [`crate::color::develop_color_into`] and its `f32` /
    /// threaded variants. It applies **every** `GainMap` in `OpcodeList2`
    /// (`SPEC-059`, `DEC-069`) and nothing else, so any other mandatory opcode
    /// in any list is refused. It refuses a malformed tag of the levels and
    /// geometry set, the CFA set or the colour set (`DEC-078` §2). A composer
    /// asking with this path gets the full colour develop's answer.
    Colour,
    /// The demosaic stage primitives, [`crate::demosaic::demosaic_into`] and
    /// [`crate::demosaic::demosaic_into_threaded`] (`SPEC-087`, `DEC-078`).
    /// The same opcodes as [`DevelopPath::Colour`]: the primitive takes a
    /// plane already past stage 2, so a composer applies the `GainMap`s first
    /// with [`crate::develop::apply_gain_maps`]. It refuses only a malformed
    /// tag of the CFA set, the tags a demosaic reads.
    Demosaic,
    /// The colour transform stage primitive,
    /// [`crate::color::to_srgb_linear_into`] (`SPEC-087`, `DEC-078`). The same
    /// opcodes as [`DevelopPath::Colour`]. It refuses only a malformed tag of
    /// the colour set, the tags the transform reads.
    ColourTransform,
    /// The developed mix (`SPEC-093`, `DEC-084` item 3):
    /// [`crate::develop::develop_with_into`] with
    /// [`crate::develop::DevelopOptions::mono_mix`], and its scratch form. It
    /// applies **every** `GainMap` in `OpcodeList2` (inside the mix, as
    /// [`DevelopPath::Colour`]) and the first `WarpRectilinear` in
    /// `OpcodeList3` whatever its flags (as [`DevelopPath::Mono`]), and nothing
    /// from `OpcodeList1`: the mono applier's 3×3 median would mix Bayer
    /// colours, so a mandatory `FixBadPixelsConstant` is refused. It refuses a
    /// malformed tag of the colour path's three sets.
    MonoMix,
}

/// `DEC-078` §2, the levels and geometry set: the tags the mono and colour
/// develops read to resolve levels and framing, in ascending order.
/// `BlackLevelRepeatDim` counts only when `BlackLevel` has more than one value
/// (see [`check_malformed_tags`]).
const LEVELS_AND_GEOMETRY_SET: [u16; 7] = [
    TAG_ORIENTATION,
    TAG_BLACK_LEVEL_REPEAT_DIM,
    TAG_BLACK_LEVEL,
    TAG_WHITE_LEVEL,
    TAG_DEFAULT_CROP_ORIGIN,
    TAG_DEFAULT_CROP_SIZE,
    TAG_ACTIVE_AREA,
];

/// `DEC-078` §2, the CFA set: the tags a demosaic reads beyond `CFAPattern`
/// and `CFARepeatPatternDim` (which are refused another way), ascending.
const CFA_SET: [u16; 2] = [TAG_CFA_PLANE_COLOR, TAG_CFA_LAYOUT];

/// `DEC-078` §2, the colour set: every tag the colour transform reads, even
/// one the chosen sub-path will not use, ascending.
const COLOUR_SET: [u16; 11] = [
    TAG_COLOR_MATRIX_1,
    TAG_COLOR_MATRIX_2,
    TAG_CAMERA_CALIBRATION_1,
    TAG_CAMERA_CALIBRATION_2,
    TAG_ANALOG_BALANCE,
    TAG_CALIBRATION_ILLUMINANT_1,
    TAG_CALIBRATION_ILLUMINANT_2,
    TAG_CAMERA_CALIBRATION_SIGNATURE,
    TAG_PROFILE_CALIBRATION_SIGNATURE,
    TAG_FORWARD_MATRIX_1,
    TAG_FORWARD_MATRIX_2,
];

/// Which instances of an opcode a path applies from one list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Applied {
    /// Nothing from this list.
    Nothing,
    /// The first opcode with this ID, whatever its own flags (the mono
    /// appliers resolve lists 1 and 3 with a first-instance lookup).
    First(u32),
    /// Every opcode with this ID (`GainMap` in list 2, `DEC-069` §2: a file
    /// carries one map per Bayer phase).
    Every(u32),
}

impl DevelopPath {
    /// What this path applies from `OpcodeList{list}`: the first
    /// `FixBadPixelsConstant` in list 1 (mono only), the first
    /// `WarpRectilinear` in list 3 (mono and the developed mix), and every
    /// `GainMap` in list 2 (every path).
    fn applied(self, list: u8) -> Applied {
        match (self, list) {
            (DevelopPath::Mono, 1) => Applied::First(OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
            (DevelopPath::Mono | DevelopPath::MonoMix, 3) => {
                Applied::First(OPCODE_ID_WARP_RECTILINEAR)
            }
            (_, 2) => Applied::Every(OPCODE_ID_GAIN_MAP),
            _ => Applied::Nothing,
        }
    }

    /// The `DEC-078` §2 sets this path refuses a malformed tag of. The sets'
    /// numbers interleave (levels 274 and 50713 to 50829, CFA 50710 and
    /// 50711, colour 50721 to 50965), so [`check_malformed_tags`] takes the
    /// lowest malformed tag across them rather than the first in walk order.
    fn malformed_sets(self) -> &'static [&'static [u16]] {
        match self {
            DevelopPath::Mono => &[&LEVELS_AND_GEOMETRY_SET],
            DevelopPath::Colour | DevelopPath::MonoMix => {
                &[&LEVELS_AND_GEOMETRY_SET, &CFA_SET, &COLOUR_SET]
            }
            DevelopPath::Demosaic => &[&CFA_SET],
            DevelopPath::ColourTransform => &[&COLOUR_SET],
        }
    }
}

/// Check that `sensor` declares no processing `path` will not perform — the
/// question every public develop entry asks first (`SPEC-054`, `DEC-049`;
/// extended by `SPEC-056`, `DEC-053`).
///
/// Every `develop_*` entry and every stage primitive that takes a [`Sensor`]
/// and yields developed or camera-referred pixels
/// ([`crate::demosaic::demosaic_into`],
/// [`crate::demosaic::demosaic_into_threaded`],
/// [`crate::color::to_srgb_linear_into`]) already calls this. A **composer**
/// needs it when building a pipeline from the opcode appliers, which take no
/// `Sensor` and so cannot check for themselves:
/// [`crate::warp::apply_warp_into`] (`OpcodeList3`),
/// [`crate::develop::apply_fix_bad_pixels_constant`] (`OpcodeList1`) and
/// [`crate::develop::apply_gain_maps`] (`OpcodeList2`). Call it with the path
/// whose opcodes your composition applies; if you apply only the `GainMap`s,
/// ask with [`DevelopPath::Colour`].
///
/// First, before any opcode list, the sensor's sample-meaning tags and DNG
/// backward version (`SPEC-086`, `DEC-077` §2): a backward version newer than
/// 1.7.1.0, or a `SampleFormat`, `LinearizationTable`, `BlackLevelDeltaH/V`,
/// `SubTileBlockSize` or interleave factor whose value the develop does not
/// honour, is refused. Every develop entry calls this first, so the refusal
/// lands before anything is written.
///
/// Then, before any opcode list, a malformed tag that `path` reads (`SPEC-087`,
/// `DEC-078`): the levels and geometry tags on [`DevelopPath::Mono`]; the CFA
/// tags `CFAPlaneColor` and `CFALayout` on [`DevelopPath::Demosaic`]; the
/// colour calibration tags on [`DevelopPath::ColourTransform`]; all three sets
/// on [`DevelopPath::Colour`]. A malformed `BlackLevelRepeatDim` counts only
/// when `BlackLevel` has more than one value.
///
/// Lists are checked in order 1, 2, 3. For each list:
///
/// 1. If its tag is present but unreadable (recorded in
///    [`Sensor::malformed_tags`], `DEC-012`), the check refuses: the library
///    cannot show that a list it cannot read holds no mandatory opcode.
/// 2. Otherwise every **mandatory** opcode in it (`Flags` bit 0 unset) must be
///    one `path` applies: the first `FixBadPixelsConstant` in list 1 and the
///    first `WarpRectilinear` in list 3 on the mono path, and every `GainMap`
///    in list 2 on both paths (`SPEC-059`, `DEC-069`). Optional opcodes are
///    always skippable.
///
/// Reads only the sensor's fields; allocates nothing and never panics on a
/// hostile `Sensor`.
///
/// # Errors
///
/// - [`Error::UnsupportedDngVersion`] `{ backward_version }` when the effective
///   DNG backward version is newer than 1.7.1.0.
/// - [`Error::UnsupportedTag`] `{ tag }` for the first (lowest-numbered)
///   sample-meaning tag with a value the develop does not honour, or present
///   but malformed; or a malformed `DNGBackwardVersion`, or a malformed
///   `DNGVersion` with no `DNGBackwardVersion`.
/// - [`Error::MalformedTag`] `{ tag }` for the lowest malformed tag `path`
///   reads (`DEC-078` §2).
/// - [`Error::UnreadableOpcodeList`] `{ list }` when an `OpcodeList` tag is
///   present but its payload could not be read.
/// - [`Error::UnsupportedRequiredOpcode`] `{ list, id }` for a mandatory,
///   recognized opcode `path` does not apply.
/// - [`Error::UnsupportedMandatoryOpcode`] `{ id }` for a mandatory opcode ID
///   this reader does not recognize.
/// - [`Error::Truncated`] / [`Error::MalformedOpcodeParams`] for broken list
///   framing (a present list is read in full even when every opcode in it is
///   optional).
pub fn check_required_processing(sensor: &Sensor, path: DevelopPath) -> Result<(), Error> {
    check_unhonoured_tags(sensor)?;
    check_malformed_tags(sensor, path)?;
    let unreadable = [TAG_OPCODE_LIST_1, TAG_OPCODE_LIST_2, TAG_OPCODE_LIST_3]
        .map(|tag| sensor.malformed_tags.contains(&tag));
    check_lists(
        [
            sensor.opcode_list_1.as_deref(),
            sensor.opcode_list_2.as_deref(),
            sensor.opcode_list_3.as_deref(),
        ],
        unreadable,
        path,
    )
}

/// `DEC-078` §2 and §3: refuse the lowest tag of `path`'s sets that is in
/// [`Sensor::malformed_tags`], as [`Error::MalformedTag`]. A malformed tag is
/// present but unreadable, so the develop cannot fall back to the absent-tag
/// default: absent is the DNG default, malformed is "the file says something
/// here and we cannot read it".
///
/// The one exception is `BlackLevelRepeatDim`: with at most one `BlackLevel`
/// value the repeat dimension cannot change any photosite's black, so a
/// malformed one is not refused (the Pentax K-3 III Monochrome writes count 1).
/// It holds under every `DevelopOptions`, since an option never skips a tag's
/// validation (`DEC-058` §3).
///
/// Reads `malformed_tags` and `black_levels` only; allocates nothing.
fn check_malformed_tags(sensor: &Sensor, path: DevelopPath) -> Result<(), Error> {
    let repeat_dim_matters = sensor.black_levels.len() > 1;
    let first = path
        .malformed_sets()
        .iter()
        .flat_map(|set| set.iter().copied())
        .filter(|&tag| tag != TAG_BLACK_LEVEL_REPEAT_DIM || repeat_dim_matters)
        .filter(|tag| sensor.malformed_tags.contains(tag))
        .min();
    match first {
        Some(tag) => Err(Error::MalformedTag { tag }),
        None => Ok(()),
    }
}

/// The newest DNG backward version this reader reads: 1.7.1.0, the version
/// the library is written against and the same as
/// [`MAX_SUPPORTED_OPCODE_VERSION`] (`DEC-077` §2).
const MAX_SUPPORTED_BACKWARD_VERSION: [u8; 4] = [1, 7, 1, 0];

/// `DEC-077` §2: refuse a sensor whose DNG backward version is newer than this
/// reader, or that carries a tag changing what the samples mean with a value
/// the develop does not honour.
///
/// The version guard runs first. The effective backward version is
/// `DNGBackwardVersion` when present; otherwise `DNGVersion` with its last two
/// bytes zeroed (DNG 1.7.1.0's default); with both absent there is no check. A
/// malformed `DNGBackwardVersion` is refused as [`Error::UnsupportedTag`]
/// rather than replaced by the default, and so is a malformed `DNGVersion`
/// when `DNGBackwardVersion` is absent (`DEC-078` §2). Arrays compare byte by byte from the
/// first, which is the version order.
///
/// Then the tag set, in ascending tag order, refusing the first tag that is
/// present and malformed or holds a value outside its honoured (exact no-op)
/// set. `LinearizationTable` has no honoured value.
fn check_unhonoured_tags(sensor: &Sensor) -> Result<(), Error> {
    let malformed = |tag: u16| sensor.malformed_tags.contains(&tag);

    if malformed(TAG_DNG_BACKWARD_VERSION) {
        return Err(Error::UnsupportedTag {
            tag: TAG_DNG_BACKWARD_VERSION,
        });
    }
    // `DEC-078` §2: with no backward version, the default comes from
    // `DNGVersion`; a malformed one leaves the reader version unknown.
    if sensor.dng_backward_version.is_none() && malformed(TAG_DNG_VERSION) {
        return Err(Error::UnsupportedTag {
            tag: TAG_DNG_VERSION,
        });
    }
    let effective = sensor
        .dng_backward_version
        .or_else(|| sensor.dng_version.map(|[a, b, _, _]| [a, b, 0, 0]));
    if let Some(backward_version) = effective {
        if backward_version > MAX_SUPPORTED_BACKWARD_VERSION {
            return Err(Error::UnsupportedDngVersion { backward_version });
        }
    }

    // A zero delta: `-0.0 == 0.0`, so a negative-zero numerator is honoured.
    let all_zero = |deltas: &Option<Vec<f64>>| {
        deltas
            .as_deref()
            .is_none_or(|d| d.iter().all(|&x| x == 0.0))
    };
    let honoured: [(u16, bool); 7] = [
        (
            TAG_SAMPLE_FORMAT,
            sensor
                .sample_format
                .as_deref()
                .is_none_or(|formats| formats.iter().all(|&f| f == 1)),
        ),
        (
            TAG_LINEARIZATION_TABLE,
            sensor.linearization_table.is_none(),
        ),
        (
            TAG_BLACK_LEVEL_DELTA_H,
            all_zero(&sensor.black_level_delta_h),
        ),
        (
            TAG_BLACK_LEVEL_DELTA_V,
            all_zero(&sensor.black_level_delta_v),
        ),
        (
            TAG_SUB_TILE_BLOCK_SIZE,
            sensor.sub_tile_block_size.is_none_or(|size| size == [1, 1]),
        ),
        (
            TAG_ROW_INTERLEAVE_FACTOR,
            sensor.row_interleave_factor.is_none_or(|f| f == 1),
        ),
        (
            TAG_COLUMN_INTERLEAVE_FACTOR,
            sensor.column_interleave_factor.is_none_or(|f| f == 1),
        ),
    ];
    for (tag, value_honoured) in honoured {
        if malformed(tag) || !value_honoured {
            return Err(Error::UnsupportedTag { tag });
        }
    }
    Ok(())
}

/// The body of [`check_required_processing`] over raw list bytes
/// (`OpcodeList1`/`2`/`3`, in that order; `None` = absent or unreadable) and
/// each list's "present but unreadable" flag.
///
/// This walks the **raw framing** with flags, not [`parse_opcode_list`]'s typed
/// output: the typed parse keeps `Flags` only on [`Opcode::Unknown`], and cannot
/// see list position or a second instance, so it cannot answer the question.
///
/// Rule, per opcode in list `n`: `Flags` bit 0 set (optional) → skippable.
/// Unset (mandatory) → it must be an instance `path` applies
/// ([`DevelopPath::applied`]: the first opcode with that ID in the list,
/// whatever that first instance's own flags, or every `GainMap` in list 2);
/// otherwise:
///
/// - a recognized opcode ID (`WarpRectilinear`, `FixBadPixelsConstant`,
///   `GainMap`) →
///   [`Error::UnsupportedRequiredOpcode`] `{ list: n, id }`;
/// - an unrecognized ID → [`Error::UnsupportedMandatoryOpcode`] `{ id }`, the
///   same error [`parse_opcode_list`] has always given for it;
/// - any ID declaring a DNG version newer than this reader supports →
///   [`Error::UnsupportedOpcodeVersion`] `{ id, version }`. Such an opcode is
///   never the applied instance: the appliers skip it like an unknown ID.
///
/// A list flagged unreadable refuses with [`Error::UnreadableOpcodeList`]
/// before its own scan, so with several defects the lowest list's is reported.
///
/// Parameter blocks are not interpreted here — the mono appliers still parse
/// the opcodes they apply and report [`Error::MalformedOpcodeParams`]
/// themselves. Panic-free on hostile bytes: the shared framing reader is fully
/// checked, and the declared `count` never pre-allocates (each iteration
/// consumes at least 16 bytes or fails).
pub(crate) fn check_lists(
    lists: [Option<&[u8]>; 3],
    unreadable: [bool; 3],
    path: DevelopPath,
) -> Result<(), Error> {
    for (list, (bytes, unreadable)) in (1u8..=3).zip(lists.into_iter().zip(unreadable)) {
        if unreadable {
            return Err(Error::UnreadableOpcodeList { list });
        }
        if let Some(bytes) = bytes {
            check_list(bytes, list, path.applied(list))?;
        }
    }
    Ok(())
}

/// One list of [`check_lists`]: `applied` is what this path applies from
/// this list.
fn check_list(bytes: &[u8], list: u8, applied: Applied) -> Result<(), Error> {
    let count = read_u32_be(bytes, 0)?;
    let mut pos: usize = 4;
    let mut applied_seen = false;

    for _ in 0..count {
        let raw = read_raw_opcode(bytes, &mut pos)?;

        // An opcode declaring a newer DNG version is not one the appliers
        // read (they skip it like an unknown ID), so it is never the applied
        // instance and never shadows a later same-ID instance that is.
        let Some(id) = raw.known_id() else {
            if raw.optional() {
                continue;
            }
            return Err(raw.unsupported());
        };

        let is_applied = match applied {
            Applied::Nothing => false,
            // The applier consumes the first instance of its ID, optional or
            // not; every later instance is not applied.
            Applied::First(applied_id) => {
                let first = applied_id == id && !applied_seen;
                if applied_id == id {
                    applied_seen = true;
                }
                first
            }
            Applied::Every(applied_id) => applied_id == id,
        };

        if is_applied || raw.optional() {
            continue;
        }
        return Err(match id {
            OPCODE_ID_WARP_RECTILINEAR | OPCODE_ID_FIX_BAD_PIXELS_CONSTANT | OPCODE_ID_GAIN_MAP => {
                Error::UnsupportedRequiredOpcode { list, id }
            }
            other => Error::UnsupportedMandatoryOpcode { id: other },
        });
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    /// One raw opcode entry: `id, version, flags, size, params`, big-endian.
    /// A tiny local builder — the fuller encoder integration tests share
    /// lives in `tests/support/opcode.rs`; this module's own unit tests stay
    /// self-contained (no cross-directory `#[path]` from inside `src/`).
    fn raw_opcode(id: u32, version: u32, flags: u32, params: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&id.to_be_bytes());
        out.extend_from_slice(&version.to_be_bytes());
        out.extend_from_slice(&flags.to_be_bytes());
        out.extend_from_slice(&u32::try_from(params.len()).unwrap().to_be_bytes());
        out.extend_from_slice(params);
        out
    }

    fn opcode_list(opcodes: &[Vec<u8>]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&u32::try_from(opcodes.len()).unwrap().to_be_bytes());
        for op in opcodes {
            out.extend_from_slice(op);
        }
        out
    }

    fn warp_rectilinear_params(kr: [f64; 4], kt: [f64; 2], cx: f64, cy: f64) -> Vec<u8> {
        let mut p = Vec::new();
        p.extend_from_slice(&1u32.to_be_bytes()); // N = 1
        for v in kr {
            p.extend_from_slice(&v.to_be_bytes());
        }
        for v in kt {
            p.extend_from_slice(&v.to_be_bytes());
        }
        p.extend_from_slice(&cx.to_be_bytes());
        p.extend_from_slice(&cy.to_be_bytes());
        p
    }

    #[test]
    fn empty_list_parses_to_no_opcodes() {
        let bytes = opcode_list(&[]);
        assert_eq!(
            parse_opcode_list(&bytes).expect("empty list parses"),
            vec![]
        );
    }

    #[test]
    fn optional_unknown_opcode_is_skipped_into_unknown_variant() {
        let bytes = opcode_list(&[raw_opcode(999, 0x0104_0000, 1, &[1, 2, 3])]);
        let opcodes = parse_opcode_list(&bytes).expect("optional-unknown must not error");
        assert_eq!(
            opcodes,
            vec![Opcode::Unknown {
                id: 999,
                flags: 1,
                params: vec![1, 2, 3],
            }]
        );
    }

    #[test]
    fn mandatory_unknown_opcode_is_rejected() {
        let bytes = opcode_list(&[raw_opcode(999, 0x0104_0000, 0, &[1, 2, 3])]);
        let err = parse_opcode_list(&bytes).expect_err("mandatory-unknown must error");
        assert!(
            matches!(err, Error::UnsupportedMandatoryOpcode { id: 999 }),
            "{err:?}"
        );
    }

    #[test]
    fn truncated_count_is_rejected_not_panicked() {
        let err = parse_opcode_list(&[0, 0]).expect_err("2 bytes cannot hold a u32 count");
        assert!(matches!(err, Error::Truncated { .. }), "{err:?}");
    }

    #[test]
    fn truncated_parameter_block_is_rejected_not_panicked() {
        // count=1, id=1 (WarpRectilinear), version=0, flags=0, size=68, but
        // zero actual parameter bytes follow.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&68u32.to_be_bytes());
        let err = parse_opcode_list(&bytes).expect_err("declared 68 bytes, supplied 0");
        assert!(matches!(err, Error::Truncated { .. }), "{err:?}");
    }

    #[test]
    fn warp_rectilinear_with_wrong_declared_size_is_malformed() {
        // N=1 needs exactly 68 bytes; the opcode's own DataSize (computed
        // from the now-67-byte params by `raw_opcode`) is internally
        // consistent with the outer container, so this is a WarpRectilinear-
        // specific N/size mismatch, not a container-level truncation.
        let mut params = warp_rectilinear_params([0.999, -0.06, -0.09, 0.05], [0.0, 0.0], 0.5, 0.5);
        params.pop();
        let bytes = opcode_list(&[raw_opcode(1, 0x0104_0000, 0, &params)]);
        let err = parse_opcode_list(&bytes).expect_err("N=1 implies 68 bytes, only 67 supplied");
        assert!(
            matches!(
                err,
                Error::MalformedOpcodeParams {
                    id: 1,
                    declared_size: 67
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn warp_rectilinear_round_trips_hand_built_coefficients() {
        let params = warp_rectilinear_params(
            [0.999251106, -0.0613765129, -0.0939155414, 0.0558820092],
            [0.0, 0.0],
            0.5,
            0.5,
        );
        let bytes = opcode_list(&[raw_opcode(
            OPCODE_ID_WARP_RECTILINEAR,
            0x0104_0000,
            0,
            &params,
        )]);
        let warp = parse_warp_rectilinear(&bytes)
            .expect("parses")
            .expect("one WarpRectilinear opcode");
        assert_eq!(
            warp.kr,
            [0.999251106, -0.0613765129, -0.0939155414, 0.0558820092]
        );
        assert_eq!(warp.kt, [0.0, 0.0]);
        assert_eq!((warp.cx, warp.cy), (0.5, 0.5));
        assert_eq!(warp.planes, 1);
    }

    #[test]
    fn non_finite_warp_coefficient_or_centre_is_malformed() {
        let good_kr = [1.0, 0.0, 0.0, 0.0];
        let cases: [([f64; 4], [f64; 2], f64, f64); 5] = [
            ([f64::NAN, 0.0, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5),
            ([1.0, 0.0, f64::INFINITY, 0.0], [0.0, 0.0], 0.5, 0.5),
            (good_kr, [f64::NAN, 0.0], 0.5, 0.5),
            (good_kr, [0.0, 0.0], f64::NAN, 0.5),
            (good_kr, [0.0, 0.0], 0.5, f64::NEG_INFINITY),
        ];
        for (kr, kt, cx, cy) in cases {
            let params = warp_rectilinear_params(kr, kt, cx, cy);
            let bytes = opcode_list(&[raw_opcode(
                OPCODE_ID_WARP_RECTILINEAR,
                0x0103_0000,
                0,
                &params,
            )]);
            for err in [
                parse_warp_rectilinear(&bytes).expect_err("non-finite warp"),
                parse_opcode_list(&bytes).expect_err("non-finite warp"),
            ] {
                assert!(
                    matches!(
                        err,
                        Error::MalformedOpcodeParams {
                            id: OPCODE_ID_WARP_RECTILINEAR,
                            ..
                        }
                    ),
                    "kr={kr:?} kt={kt:?} cx={cx} cy={cy}: {err:?}"
                );
            }
        }
    }

    #[test]
    fn no_warp_rectilinear_opcode_is_none_not_an_error() {
        let bytes = opcode_list(&[raw_opcode(999, 0x0104_0000, 1, &[])]);
        assert_eq!(parse_warp_rectilinear(&bytes).expect("parses"), None);
    }

    #[test]
    fn fix_bad_pixels_constant_round_trips_hand_built_params() {
        let mut params = Vec::new();
        params.extend_from_slice(&0u32.to_be_bytes()); // Constant
        params.extend_from_slice(&2u32.to_be_bytes()); // BayerPhase
        let bytes = opcode_list(&[raw_opcode(
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
            0x0103_0000,
            0,
            &params,
        )]);
        let opcodes = parse_opcode_list(&bytes).expect("parses");
        assert_eq!(
            opcodes,
            vec![Opcode::FixBadPixelsConstant {
                constant: 0,
                bayer_phase: 2,
            }]
        );
        assert_eq!(
            parse_fix_bad_pixels_constant(&bytes).expect("parses"),
            Some(0)
        );
    }

    #[test]
    fn fix_bad_pixels_constant_with_wrong_declared_size_is_malformed() {
        // Constant/BayerPhase is exactly 8 bytes; 7 is malformed, not a
        // best-effort partial read.
        let bytes = opcode_list(&[raw_opcode(
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
            0x0103_0000,
            0,
            &[0, 0, 0, 0, 0, 0, 0],
        )]);
        let err = parse_opcode_list(&bytes).expect_err("7 bytes cannot hold Constant+BayerPhase");
        assert!(
            matches!(
                err,
                Error::MalformedOpcodeParams {
                    id: OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
                    declared_size: 7
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn no_fix_bad_pixels_constant_opcode_is_none_not_an_error() {
        let bytes = opcode_list(&[raw_opcode(999, 0x0104_0000, 1, &[])]);
        assert_eq!(parse_fix_bad_pixels_constant(&bytes).expect("parses"), None);
    }

    // ── GainMap (SPEC-059, DEC-069) ─────────────────────────────────────────

    /// A `GainMap` block with `points_v × points_h × map_planes` gains, gain
    /// `k` (in file order) being `k as f32 + 0.5`.
    fn gain_map_block(points_v: u32, points_h: u32, map_planes: u32) -> Vec<u8> {
        let mut p = Vec::new();
        for v in [0u32, 0, 4, 4, 0, 1, 1, 1, points_v, points_h] {
            p.extend_from_slice(&v.to_be_bytes());
        }
        for v in [0.5f64, 0.25, 0.0, 0.0] {
            p.extend_from_slice(&v.to_be_bytes());
        }
        p.extend_from_slice(&map_planes.to_be_bytes());
        for k in 0..points_v * points_h * map_planes {
            p.extend_from_slice(&(k as f32 + 0.5).to_be_bytes());
        }
        p
    }

    #[test]
    fn gain_map_entry_reads_every_written_gain_in_place() {
        let params = gain_map_block(3, 5, 2);
        let view = parse_gain_map_params(&params, OPCODE_ID_GAIN_MAP).expect("valid");
        assert_eq!((view.points_v, view.points_h, view.map_planes), (3, 5, 2));
        assert_eq!((view.spacing_v, view.spacing_h), (0.5, 0.25));
        let mut k = 0u32;
        for i in 0..3 {
            for j in 0..5 {
                for p in 0..2 {
                    assert_eq!(view.entry(i, j, p), Some(k as f32 + 0.5), "({i}, {j}, {p})");
                    k += 1;
                }
            }
        }
        assert_eq!(view.entry(3, 0, 0), None);
        assert_eq!(view.entry(0, 5, 0), None);
        assert_eq!(view.entry(0, 0, 2), None);
    }

    #[test]
    fn grid_axis_clamps_at_both_edges_and_splits_between() {
        assert_eq!(grid_axis(-0.25, 4), (0, 0, 0.0));
        assert_eq!(grid_axis(0.0, 4), (0, 0, 0.0));
        assert_eq!(grid_axis(1.25, 4), (1, 2, 0.25));
        assert_eq!(grid_axis(3.0, 4), (3, 3, 0.0));
        assert_eq!(grid_axis(9.5, 4), (3, 3, 0.0));
        assert_eq!(grid_axis(0.5, 1), (0, 0, 0.0));
        assert_eq!(grid_axis(f64::INFINITY, 2), (1, 1, 0.0));
    }

    #[test]
    fn gain_maps_from_list_counts_past_the_cap_and_skips_other_opcodes() {
        let one = raw_opcode(OPCODE_ID_GAIN_MAP, 0, 0, &gain_map_block(1, 1, 1));
        let mut ops = vec![warp(MANDATORY), raw_opcode(999, 0, 1, &[9])];
        ops.extend(std::iter::repeat_n(one, 16));
        let sixteen = opcode_list(&ops);
        let maps = GainMaps::from_list(&sixteen, 4, 4).expect("sixteen");
        assert_eq!(maps.len(), 16);
        ops.push(ops[2].clone());
        let err = GainMaps::from_list(&opcode_list(&ops), 4, 4).unwrap_err();
        assert!(
            matches!(err, Error::TooManyGainMaps { count: 17 }),
            "{err:?}"
        );
    }

    // ── check_required_processing (SPEC-054, DEC-049) ───────────────────────

    const MANDATORY: u32 = 0;
    const OPTIONAL: u32 = FLAG_OPTIONAL;

    fn fbpc(flags: u32) -> Vec<u8> {
        raw_opcode(
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
            0x0103_0000,
            flags,
            &[0; 8],
        )
    }

    fn warp(flags: u32) -> Vec<u8> {
        let params = warp_rectilinear_params([1.0, 0.0, 0.0, 0.0], [0.0, 0.0], 0.5, 0.5);
        raw_opcode(OPCODE_ID_WARP_RECTILINEAR, 0x0104_0000, flags, &params)
    }

    /// Run the check with `bytes` as the only present list, in slot `list`.
    fn check_one(bytes: &[u8], list: u8, path: DevelopPath) -> Result<(), Error> {
        let mut lists: [Option<&[u8]>; 3] = [None; 3];
        lists[usize::from(list - 1)] = Some(bytes);
        check_lists(lists, [false; 3], path)
    }

    fn required(list: u8, id: u32) -> String {
        format!("{:?}", Error::UnsupportedRequiredOpcode { list, id })
    }

    /// A `FixBadPixelsConstant` declaring DNG 2.0.0.0, newer than any
    /// version this reader supports.
    fn fbpc_v2(flags: u32) -> Vec<u8> {
        raw_opcode(
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
            0x0200_0000,
            flags,
            &[0; 8],
        )
    }

    fn too_new(id: u32, version: u32) -> String {
        format!("{:?}", Error::UnsupportedOpcodeVersion { id, version })
    }

    #[test]
    fn a_mandatory_opcode_newer_than_supported_is_refused_everywhere() {
        let bytes = opcode_list(&[fbpc_v2(MANDATORY)]);
        let want = too_new(OPCODE_ID_FIX_BAD_PIXELS_CONSTANT, 0x0200_0000);
        // The required-processing check, on every path and in every list
        // (it is recognized by ID, but not at this version).
        for path in [DevelopPath::Mono, DevelopPath::Colour] {
            for list in 1..=3 {
                let err = check_one(&bytes, list, path).expect_err("too-new mandatory");
                assert_eq!(format!("{err:?}"), want, "{path:?} list {list}");
            }
        }
        // ...and every parser.
        assert_eq!(
            format!("{:?}", parse_opcode_list(&bytes).unwrap_err()),
            want
        );
        assert_eq!(
            format!("{:?}", parse_fix_bad_pixels_constant(&bytes).unwrap_err()),
            want
        );
        assert_eq!(
            format!("{:?}", parse_warp_rectilinear(&bytes).unwrap_err()),
            want
        );
    }

    #[test]
    fn an_optional_opcode_newer_than_supported_is_skipped_not_applied() {
        // Garbage params: a too-new opcode's parameters are never interpreted.
        let v2_optional = raw_opcode(OPCODE_ID_FIX_BAD_PIXELS_CONSTANT, 0x0200_0000, 1, &[1, 2]);
        let bytes = opcode_list(std::slice::from_ref(&v2_optional));
        assert_eq!(
            parse_opcode_list(&bytes).expect("skippable"),
            vec![Opcode::Unknown {
                id: OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
                flags: 1,
                params: vec![1, 2],
            }]
        );
        assert_eq!(
            parse_fix_bad_pixels_constant(&bytes).expect("skippable"),
            None
        );
        for path in [DevelopPath::Mono, DevelopPath::Colour] {
            assert!(check_one(&bytes, 1, path).is_ok(), "{path:?}");
        }

        // It does not take the applied slot: a later supported mandatory
        // FixBadPixelsConstant is the first instance the applier sees, and
        // the check agrees it is applied.
        let then_v13 = opcode_list(&[v2_optional, fbpc(MANDATORY)]);
        assert_eq!(
            parse_fix_bad_pixels_constant(&then_v13).expect("parses"),
            Some(0)
        );
        assert!(check_one(&then_v13, 1, DevelopPath::Mono).is_ok());
    }

    #[test]
    fn the_newest_supported_version_is_still_applied() {
        let v171 = opcode_list(&[raw_opcode(
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
            MAX_SUPPORTED_OPCODE_VERSION,
            MANDATORY,
            &[0; 8],
        )]);
        assert_eq!(
            parse_fix_bad_pixels_constant(&v171).expect("parses"),
            Some(0)
        );
        assert!(check_one(&v171, 1, DevelopPath::Mono).is_ok());
        let v172 = opcode_list(&[raw_opcode(
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT,
            MAX_SUPPORTED_OPCODE_VERSION + 0x100,
            MANDATORY,
            &[0; 8],
        )]);
        assert!(check_one(&v172, 1, DevelopPath::Mono).is_err());
    }

    #[test]
    fn no_lists_and_empty_lists_pass_on_both_paths() {
        for path in [DevelopPath::Mono, DevelopPath::Colour] {
            assert!(check_lists([None; 3], [false; 3], path).is_ok());
            let empty = opcode_list(&[]);
            for list in 1..=3 {
                assert!(check_one(&empty, list, path).is_ok());
            }
        }
    }

    #[test]
    fn mono_applied_set_is_first_fbpc_in_list_1_and_first_warp_in_list_3() {
        let l1 = opcode_list(&[fbpc(MANDATORY)]);
        let l3 = opcode_list(&[warp(MANDATORY)]);
        assert!(check_lists([Some(&l1), None, Some(&l3)], [false; 3], DevelopPath::Mono).is_ok());
        // ...and every other placement of the same mandatory opcodes is not applied.
        let cases = [
            (1, warp(MANDATORY), OPCODE_ID_WARP_RECTILINEAR),
            (2, fbpc(MANDATORY), OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
            (2, warp(MANDATORY), OPCODE_ID_WARP_RECTILINEAR),
            (3, fbpc(MANDATORY), OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
        ];
        for (list, op, id) in cases {
            let err = check_one(&opcode_list(&[op]), list, DevelopPath::Mono).unwrap_err();
            assert_eq!(format!("{err:?}"), required(list, id));
        }
    }

    #[test]
    fn colour_applies_nothing() {
        for list in 1..=3 {
            for (op, id) in [
                (fbpc(MANDATORY), OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
                (warp(MANDATORY), OPCODE_ID_WARP_RECTILINEAR),
            ] {
                let err = check_one(&opcode_list(&[op]), list, DevelopPath::Colour).unwrap_err();
                assert_eq!(format!("{err:?}"), required(list, id));
            }
        }
    }

    /// `SPEC-093` AC6 (v), `DEC-084` item 3: the developed mix applies every
    /// `GainMap` in list 2 and the first `WarpRectilinear` in list 3, whatever
    /// its flags, and nothing from list 1; its malformed sets are the colour
    /// path's.
    #[test]
    fn ac6_mono_mix_path_applies_gain_maps_and_the_first_warp() {
        let path = DevelopPath::MonoMix;
        let gain = |flags| raw_opcode(OPCODE_ID_GAIN_MAP, 0x0103_0000, flags, &[]);
        // List 1: every mandatory opcode is refused; optional ones skipped.
        for (op, id) in [
            (fbpc(MANDATORY), OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
            (warp(MANDATORY), OPCODE_ID_WARP_RECTILINEAR),
            (gain(MANDATORY), OPCODE_ID_GAIN_MAP),
        ] {
            let err = check_one(&opcode_list(&[op]), 1, path).unwrap_err();
            assert_eq!(format!("{err:?}"), required(1, id));
        }
        assert!(check_one(&opcode_list(&[fbpc(OPTIONAL)]), 1, path).is_ok());
        // List 2: every GainMap, nothing else.
        let four = opcode_list(&[
            gain(MANDATORY),
            gain(MANDATORY),
            gain(MANDATORY),
            gain(MANDATORY),
        ]);
        assert!(check_one(&four, 2, path).is_ok());
        for (op, id) in [
            (fbpc(MANDATORY), OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
            (warp(MANDATORY), OPCODE_ID_WARP_RECTILINEAR),
        ] {
            let err = check_one(&opcode_list(&[op]), 2, path).unwrap_err();
            assert_eq!(format!("{err:?}"), required(2, id));
        }
        // List 3: the first WarpRectilinear only.
        assert!(check_one(&opcode_list(&[warp(MANDATORY)]), 3, path).is_ok());
        let second = opcode_list(&[warp(MANDATORY), warp(MANDATORY)]);
        assert_eq!(
            format!("{:?}", check_one(&second, 3, path).unwrap_err()),
            required(3, OPCODE_ID_WARP_RECTILINEAR)
        );
        let behind = opcode_list(&[warp(OPTIONAL), warp(MANDATORY)]);
        assert!(check_one(&behind, 3, path).is_err());
        for (op, id) in [
            (fbpc(MANDATORY), OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
            (gain(MANDATORY), OPCODE_ID_GAIN_MAP),
        ] {
            let err = check_one(&opcode_list(&[op]), 3, path).unwrap_err();
            assert_eq!(format!("{err:?}"), required(3, id));
        }
        // The applied table, list by list, and the malformed sets.
        assert_eq!(path.applied(1), Applied::Nothing);
        assert_eq!(path.applied(2), Applied::Every(OPCODE_ID_GAIN_MAP));
        assert_eq!(path.applied(3), Applied::First(OPCODE_ID_WARP_RECTILINEAR));
        assert_eq!(path.malformed_sets(), DevelopPath::Colour.malformed_sets());
    }

    #[test]
    fn only_the_first_instance_is_applied() {
        // A second mandatory instance is the `find_map` gap.
        let two = opcode_list(&[fbpc(MANDATORY), fbpc(MANDATORY)]);
        let err = check_one(&two, 1, DevelopPath::Mono).unwrap_err();
        assert_eq!(
            format!("{err:?}"),
            required(1, OPCODE_ID_FIX_BAD_PIXELS_CONSTANT)
        );
        // The first instance is the applied one whatever its own flags, so a
        // mandatory one behind an optional one is NOT applied.
        let behind = opcode_list(&[fbpc(OPTIONAL), fbpc(MANDATORY)]);
        assert!(check_one(&behind, 1, DevelopPath::Mono).is_err());
        // Optional duplicates are always fine.
        let dup = opcode_list(&[warp(MANDATORY), warp(OPTIONAL), warp(OPTIONAL)]);
        assert!(check_one(&dup, 3, DevelopPath::Mono).is_ok());
    }

    #[test]
    fn optional_opcodes_are_skipped_everywhere() {
        let all_optional = opcode_list(&[
            fbpc(OPTIONAL),
            warp(OPTIONAL),
            raw_opcode(999, 0x0104_0000, OPTIONAL, &[1, 2]),
            // Any flags word with bit 0 set is optional (bit 1 = preview-skip).
            raw_opcode(998, 0x0104_0000, 0b11, &[]),
        ]);
        for path in [DevelopPath::Mono, DevelopPath::Colour] {
            for list in 1..=3 {
                assert!(check_one(&all_optional, list, path).is_ok());
            }
        }
    }

    #[test]
    fn mandatory_unknown_keeps_its_existing_error_in_every_list() {
        let bytes = opcode_list(&[raw_opcode(999, 0x0104_0000, MANDATORY, &[])]);
        for path in [DevelopPath::Mono, DevelopPath::Colour] {
            for list in 1..=3 {
                let err = check_one(&bytes, list, path).unwrap_err();
                assert!(
                    matches!(err, Error::UnsupportedMandatoryOpcode { id: 999 }),
                    "{err:?}"
                );
            }
        }
    }

    #[test]
    fn broken_framing_is_reported_even_when_every_opcode_is_optional() {
        let mut bytes = opcode_list(&[fbpc(OPTIONAL)]);
        bytes.pop();
        let err = check_one(&bytes, 2, DevelopPath::Colour).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }), "{err:?}");
        // A huge declared count over a short stream fails, never loops.
        let mut huge = u32::MAX.to_be_bytes().to_vec();
        huge.extend_from_slice(&fbpc(OPTIONAL));
        let err = check_one(&huge, 1, DevelopPath::Mono).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }), "{err:?}");
        let err = check_one(&[0, 0], 3, DevelopPath::Mono).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }), "{err:?}");
    }

    #[test]
    fn lists_are_checked_in_order_1_2_3() {
        let l1 = opcode_list(&[warp(MANDATORY)]);
        let l2 = opcode_list(&[fbpc(MANDATORY)]);
        let err = check_lists(
            [Some(&l1), Some(&l2), None],
            [false; 3],
            DevelopPath::Colour,
        )
        .unwrap_err();
        assert_eq!(format!("{err:?}"), required(1, OPCODE_ID_WARP_RECTILINEAR));
    }

    #[test]
    fn shared_framing_reader_keeps_parse_opcode_list_errors_unchanged() {
        // The refactor onto `read_raw_opcode` must report the same offsets.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&1u32.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&68u32.to_be_bytes());
        let err = parse_opcode_list(&bytes).unwrap_err();
        assert!(
            matches!(err, Error::Truncated { at: 20, len: 68 }),
            "{err:?}"
        );
        let err = parse_opcode_list(&bytes[..10]).unwrap_err();
        assert!(matches!(err, Error::Truncated { at: 8, len: 4 }), "{err:?}");
    }
}
