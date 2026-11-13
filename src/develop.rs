//! Level normalization and geometry — `SPEC-014`.
//!
//! `src/plane.rs` produces a **correct, uncropped, un-normalised** `u16`
//! plane and `tests/plane_oracle.rs` (`SPEC-013`) asserts it bit-for-bit
//! against `dnglab --raw-checksum`. This module turns that plane into the
//! image a consumer would actually display: `OpcodeList1`'s
//! `FixBadPixelsConstant` defect correction applied to the raw plane FIRST
//! (`SPEC-017`), then black subtracted, white normalized, `OpcodeList3`'s
//! `WarpRectilinear` geometric correction applied over the `ActiveArea`
//! window (`SPEC-018`), then `DefaultCrop` → `Orientation` extracts the
//! final displayed rectangle. See [`develop_into`]'s own doc for why the
//! warp runs BEFORE the crop, not after (a correction from `SPEC-018`'s own
//! design-time assumption), and for why `FixBadPixelsConstant` runs before
//! normalization at all (DNG 1.7.0.0 Chapter 7's own pipeline: opcodes act
//! on the raw plane's own value range, not the normalized `[0, 65535]`
//! output range).
//!
//! # No oracle covers this — `DEC-004`
//!
//! `dnglab --raw-checksum` attaches **before** all of this (it hashes the
//! uncropped, un-normalised plane) and `DEC-004` found that no comparison
//! oracle ever will: the plane checksum is structurally blind to a levels
//! error (no black subtraction), and the develop oracle (SSIMULACRA2 against
//! `dnglab --srgb`) misses a black-level error up to **+256 — 50% of the true
//! black level**, because a levels error is nearly an affine tone change and
//! perceptual metrics forgive those. `tests/develop.rs`'s analytic assertions
//! against tag values read from the file are therefore the only check this
//! arithmetic has until `SPEC-015` lands the independent analytic oracle.
//!
//! # Provenance
//!
//! DNG 1.7 §Chapter 4 (`BlackLevel`, `WhiteLevel`, `ActiveArea`,
//! `DefaultCropOrigin`/`Size`) for the levels and crop arithmetic — a direct
//! transcription of the spec's own affine mapping and three-stage crop,
//! provenance class one (published specification). `Orientation`'s
//! eight-value semantics (TIFF 6.0 tag 274, extended by the Exif
//! specification to the eight row/column combinations used here) are
//! cross-checked against `exiftool`'s own labels already recorded in this
//! repo (`docs/measured-q2m-dng.md`: value 6 = "Rotate 90 CW") rather than
//! transcribed from a page number this repo cannot re-verify. `DEC-019`
//! records why `DefaultCropOrigin` is applied relative to `ActiveArea` rather
//! than the raw plane, with the `dnglab`/`exiftool` evidence that settles it.
//! Nothing here was read from `dnglab`/`rawler` (LGPL-2.1) — both are run as
//! tools only (`provenance-recorded-per-algorithm`). See the row in
//! `docs/provenance-ledger.md`.
//!
//! # Output representation — `DEC-018`
//!
//! The developed image is `u16`, full-scale at [`u16::MAX`] — not `f32` in
//! `[0, 1]`. `DEC-018` also records this module's chosen handling of samples
//! outside `[BlackLevel, WhiteLevel]` (`AC2`): **clamp**, not saturate-as-error.
//!
//! # Allocation
//!
//! [`develop_into`] takes both the source plane and the destination image as
//! caller-owned buffers — the same shape `DEC-016` chose for
//! [`crate::plane::unpack_into`]. It is **not in-place**: the destination is
//! smaller than the source (crop) and may have swapped dimensions
//! (orientation 5-8), so a second buffer is unavoidable regardless of the
//! warp stage below.
//!
//! **Working memory — `SPEC-065`, `DEC-060`** (replacing the `SPEC-017` /
//! `SPEC-018` allocation notes). Two opcodes need scratch beyond `src` and
//! `dst`. `OpcodeList1`'s `FixBadPixelsConstant` mutates the raw plane, but
//! `src` is a shared `&[u16]`, so the develop fixes a full-plane copy (P =
//! `width * height` samples: a bad pixel's 3x3 neighbourhood may reach the
//! padding outside `ActiveArea`). `OpcodeList3`'s non-identity
//! `WarpRectilinear` resamples the whole normalized `ActiveArea` (A samples)
//! into a second buffer of the same size. Once normalize has read the fixed
//! plane, the copy is dead and the warp output reuses it, so the peak is
//! **P + A** samples, not P + 2A.
//!
//! [`scratch_len`] reports that need for a sensor and options (0 when neither
//! opcode applies). [`develop_with_scratch_into`] takes it as one caller-owned
//! slice and **performs no heap allocation at all**, error paths included.
//! [`develop_into`] and [`develop_with_into`] run the same code with **one**
//! internal allocation of exactly that length, and none when it is 0. On a
//! Q2M frame that is one allocation of 189,415,424 bytes, where it was six
//! allocations totalling 283,943,504 bytes before `SPEC-065`; output is
//! unchanged.
//!
//! # Options — `SPEC-063`, `DEC-058`
//!
//! [`develop_with_into`] / [`output_dimensions_with`] take a
//! [`DevelopOptions`]: [`Framing::FullActiveArea`] keeps the whole
//! `ActiveArea` instead of extracting `DefaultCrop`, and `apply_orientation:
//! false` leaves the pixels in stored order. [`develop_into`] /
//! [`output_dimensions`] are exactly the `_with` forms called with
//! [`DevelopOptions::default()`], so their output is unchanged. Options
//! rewrite the geometry *after* it is resolved and validated: a malformed
//! tag an option makes unused is still refused (`DEC-058` §3).
//! [`DevelopOptions::warp_alpha`] (`SPEC-064`, `DEC-059`) applies
//! `OpcodeList3`'s `WarpRectilinear` partially: the warp is replaced by
//! [`crate::opcode::WarpRect::lerp`]`(warp_alpha)`, so `1.0` (the default) is
//! the full correction and `0.0` none at all. `FullActiveArea` with a warp
//! is accepted when the lerped warp provably samples only inside the image
//! (its radial scale stays in `[0, 1]` and its optical centre is inside the
//! image), and otherwise refused, typed, before any write
//! ([`Error::UnsupportedWarpedFullActiveArea`], `DEC-059` §3, amending
//! `DEC-058` §4). Options are mono only; the colour path takes none. No
//! option changes any existing (input, options) output, so `DEVELOP_VERSION`
//! is unchanged (`DEC-058` §5, `DEC-059` Consequences).
//!
//! **CFA planes develop here too — `SPEC-066`, `DEC-061`.** Nothing on this
//! path reads `PhotometricInterpretation` or the CFA tags, so a CFA-declared
//! plane (a converted, CFA-removed camera) develops as monochrome; choosing
//! demosaic or mono is the caller's decision. What such a plane needs is its
//! per-position black: [`DevelopOptions::black_levels`] chooses between the
//! first `BlackLevel` value for every photosite ([`BlackLevels::FirstValue`],
//! the default) and `BlackLevel` indexed by `BlackLevelRepeatDim` from the
//! `ActiveArea` origin ([`BlackLevels::PerPosition`]).

use crate::ifd::{ActiveArea, Sensor};
use crate::Error;

/// Resolved geometry, defaults applied, every rectangle validated to fit.
///
/// `pub(crate)` so the colour path ([`crate::color::develop_color_into`],
/// `SPEC-050`, `DEC-043`) crops and orients to exactly this geometry rather than
/// re-deriving it.
pub(crate) struct Geometry {
    /// `ActiveArea`'s origin, in raw-plane coordinates (`(0, 0)` when the tag
    /// is absent — `SPEC-014` `AC3`'s `L1000622.DNG` case).
    pub(crate) active_left: u32,
    pub(crate) active_top: u32,
    /// `ActiveArea`'s own size — `SPEC-018`: this is the coordinate space
    /// `WarpRectilinear` operates in (see [`develop_into`]'s module doc,
    /// "Pipeline order"), distinct from `crop_width`/`crop_height` below.
    pub(crate) active_width: u32,
    pub(crate) active_height: u32,
    /// `DefaultCropOrigin`, relative to `ActiveArea`'s origin (`AC4`;
    /// `DEC-019`). `(0, 0)` when the tag is absent.
    pub(crate) crop_origin_x: u32,
    pub(crate) crop_origin_y: u32,
    /// `DefaultCropSize`. Defaults to the full active area when the tag is
    /// absent (the DNG specification's own stated default) — untested by any
    /// corpus file, since every decodable file carries an explicit size; only
    /// `hostile_geometry_does_not_panic`'s hand-built fixture reaches it.
    pub(crate) crop_width: u32,
    pub(crate) crop_height: u32,
    /// `Orientation`, defaulted to 1 (normal) when absent, validated to
    /// `1..=8`.
    pub(crate) orientation: u32,
}

/// Resolve `ActiveArea` → `DefaultCrop` → `Orientation`, applying DNG's
/// stated defaults for each absent tag and rejecting every hostile shape
/// `AC6` names: an inverted or out-of-plane `ActiveArea`, a crop that does
/// not fit inside it (including zero-sized), and an `Orientation` outside
/// `1..=8`.
pub(crate) fn resolve_geometry(sensor: &Sensor) -> Result<Geometry, Error> {
    let (active_top, active_left, active_bottom, active_right) = match sensor.active_area {
        Some(a) => (a.top, a.left, a.bottom, a.right),
        // Absent means "the whole raw plane is active" — DNG's own default,
        // and `L1000622.DNG`'s actual shape (`AC3`).
        None => (0, 0, sensor.height, sensor.width),
    };
    let invalid_active_area = || Error::InvalidActiveArea {
        top: active_top,
        left: active_left,
        bottom: active_bottom,
        right: active_right,
        plane_width: sensor.width,
        plane_height: sensor.height,
    };
    if active_right <= active_left
        || active_bottom <= active_top
        || active_right > sensor.width
        || active_bottom > sensor.height
    {
        return Err(invalid_active_area());
    }
    let active_width = active_right
        .checked_sub(active_left)
        .ok_or_else(invalid_active_area)?;
    let active_height = active_bottom
        .checked_sub(active_top)
        .ok_or_else(invalid_active_area)?;

    let (crop_origin_x, crop_origin_y) = match sensor.default_crop_origin {
        Some(o) => (o.x, o.y),
        None => (0, 0),
    };
    let (crop_width, crop_height) = match sensor.default_crop_size {
        Some(s) => (s.width, s.height),
        None => (active_width, active_height),
    };
    let invalid_crop = || Error::InvalidDefaultCrop {
        origin_x: crop_origin_x,
        origin_y: crop_origin_y,
        crop_width,
        crop_height,
        active_width,
        active_height,
    };
    if crop_width == 0 || crop_height == 0 {
        return Err(invalid_crop());
    }
    let crop_right = crop_origin_x
        .checked_add(crop_width)
        .ok_or_else(invalid_crop)?;
    let crop_bottom = crop_origin_y
        .checked_add(crop_height)
        .ok_or_else(invalid_crop)?;
    if crop_right > active_width || crop_bottom > active_height {
        return Err(invalid_crop());
    }

    let orientation = sensor.orientation.unwrap_or(1);
    if !(1..=8).contains(&orientation) {
        return Err(Error::UnsupportedOrientation { orientation });
    }

    Ok(Geometry {
        active_left,
        active_top,
        active_width,
        active_height,
        crop_origin_x,
        crop_origin_y,
        crop_width,
        crop_height,
        orientation,
    })
}

/// The developed image's dimensions: `(crop_width, crop_height)`, swapped for
/// an `Orientation` that rotates 90° (5-8) — `AC5`.
pub(crate) fn oriented_dimensions(orientation: u32, width: u32, height: u32) -> (u32, u32) {
    match orientation {
        1..=4 => (width, height),
        // 5-8 (validated to 1..=8 by `resolve_geometry`'s caller).
        _ => (height, width),
    }
}

/// Options for the monochrome develop — [`develop_with_into`] and
/// [`output_dimensions_with`] (`SPEC-063`, `DEC-058`).
///
/// [`DevelopOptions::default()`] is exactly [`develop_into`]: every field's
/// default reproduces the output from before the field existed, and always
/// will (`DEC-058` §2).
///
/// `#[non_exhaustive]`, so outside this crate options are built from
/// [`Default`] and then assigned field by field; a later field is an addition,
/// not a break:
///
/// ```
/// use irradiance::develop::{DevelopOptions, Framing};
///
/// let mut options = DevelopOptions::default();
/// options.framing = Framing::FullActiveArea;
/// options.apply_orientation = false;
/// ```
///
/// **Mono only.** The colour path ([`crate::color::develop_color_into`] and
/// its variants) takes no options.
///
/// An option overrides what a tag is *used for*, never whether it is
/// validated: a `DefaultCrop` that does not fit is still
/// [`Error::InvalidDefaultCrop`] under [`Framing::FullActiveArea`], and an
/// `Orientation` outside `1..=8` is still [`Error::UnsupportedOrientation`]
/// with `apply_orientation: false` (`DEC-058` §3).
///
/// A caller caching developed pixels keys on `(DEVELOP_VERSION, options)`;
/// [`crate::DEVELOP_VERSION`] identifies the output for every options value,
/// not only the default (`DEC-058` §5).
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct DevelopOptions {
    /// Which rectangle of the active area becomes the output. Default
    /// [`Framing::StandardCrop`]. [`Framing::FullActiveArea`] together with a
    /// `WarpRectilinear` is refused with
    /// [`Error::UnsupportedWarpedFullActiveArea`] unless the warp, lerped to
    /// [`DevelopOptions::warp_alpha`], provably samples only inside the image
    /// (`DEC-059` §3).
    pub framing: Framing,
    /// Whether `Orientation` is applied to the output. Default `true`. With
    /// `false` the output is in stored (sensor) order, at the unswapped
    /// framed size, as if `Orientation` were 1 — for a caller that rotates
    /// on its own (for example on a GPU).
    pub apply_orientation: bool,
    /// How much of `OpcodeList3`'s `WarpRectilinear` lens correction to apply
    /// (`SPEC-064`, `DEC-059`). Default `1.0`, the full correction. The warp
    /// applied is [`crate::opcode::WarpRect::lerp`]`(warp_alpha)`, whose
    /// backward radial scale is `(1 − α) + α·f(r)`: every pixel moves `α`
    /// times as far as under the full warp. `0.0` disables the warp
    /// entirely, its tangential terms included (a non-zero tangential term
    /// is then not refused, because nothing is applied).
    ///
    /// **Domain:** a finite value in `[0, 1]`; `-0.0` is accepted. Anything
    /// else is [`Error::InvalidWarpAlpha`], checked first by
    /// [`develop_with_into`] and [`output_dimensions_with`], under every
    /// framing, whether or not the file carries a warp.
    ///
    /// **Under [`Framing::FullActiveArea`]** the lerped warp, unless it is the
    /// identity, must satisfy `DEC-059` §3's bounds rule: every coefficient
    /// finite, the optical centre inside the image, and the radial scale in
    /// `[0, 1]` for every `r² ∈ [0, 1]` (exact cubic extrema, no sampling).
    /// Then no sample is clamped to the edge. Otherwise the develop is
    /// refused with [`Error::UnsupportedWarpedFullActiveArea`] before any
    /// write. [`Framing::StandardCrop`] has no such check, at any α
    /// (`DEC-059` §4).
    pub warp_alpha: f64,
    /// Which `BlackLevel` value each photosite's black comes from
    /// (`SPEC-066`, `DEC-061`). Default [`BlackLevels::FirstValue`], the first
    /// value for every photosite, exactly [`develop_into`].
    ///
    /// **This is the "CFA as mono" option.** A CFA-declared plane (a camera
    /// with its colour filter array removed still declares `CFAPattern`)
    /// already develops on this path; what it lacks is its per-position
    /// black. [`BlackLevels::PerPosition`] applies DNG's rule: with
    /// `BlackLevelRepeatDim` `[R, C]` (`[1, 1]` when absent or malformed),
    /// the photosite at raw `(x, y)` takes `BlackLevel[((y − top) mod R) × C +
    /// ((x − left) mod C)]`, phased from the `ActiveArea` origin `(left,
    /// top)` (not the `DefaultCrop` origin, not the plane origin). The black
    /// is taken on raw samples, before `WarpRectilinear`.
    ///
    /// White stays per sample: DNG gives a one-sample plane one `WhiteLevel`,
    /// resolved as today, so there is no per-position white.
    ///
    /// The pattern is validated only when it is used. Under `PerPosition`, a
    /// `BlackLevel` count other than `R × C` (or `R` or `C` zero, or `R × C`
    /// overflowing) is [`Error::InvalidBlackLevelPattern`], and any value not
    /// below the white level is [`Error::InvalidLevels`], both reported where
    /// [`Error::InvalidLevels`] always was (after the geometry, before the
    /// `dst` length). A sensor with an absent or unreadable `BlackLevel`, or
    /// a 1×1 pattern, develops identically under both values.
    pub black_levels: BlackLevels,
}

impl Default for DevelopOptions {
    /// [`Framing::StandardCrop`], `apply_orientation: true`,
    /// `warp_alpha: 1.0` and [`BlackLevels::FirstValue`]: exactly
    /// [`develop_into`].
    fn default() -> DevelopOptions {
        DevelopOptions {
            framing: Framing::StandardCrop,
            apply_orientation: true,
            warp_alpha: 1.0,
            black_levels: BlackLevels::FirstValue,
        }
    }
}

/// Which `BlackLevel` value each photosite's black comes from
/// ([`DevelopOptions::black_levels`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum BlackLevels {
    /// The first `BlackLevel` value for every photosite: [`develop_into`]'s
    /// behaviour. The default.
    #[default]
    FirstValue,
    /// `BlackLevel` indexed by `BlackLevelRepeatDim`, the pattern's origin at
    /// the `ActiveArea` top-left (DNG 1.7). For a converted (CFA-removed)
    /// camera developed as monochrome, and for any plane with a black
    /// pattern.
    PerPosition,
}

/// Which rectangle of the active area [`develop_with_into`] outputs
/// ([`DevelopOptions::framing`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Framing {
    /// `DefaultCropOrigin`/`DefaultCropSize` within `ActiveArea` — the
    /// image a consumer would display, and [`develop_into`]'s output. The
    /// default.
    #[default]
    StandardCrop,
    /// The whole `ActiveArea` (the whole raw plane when the tag is absent):
    /// every recorded photosite, including the border `DefaultCrop` trims.
    /// `DefaultCrop` is still validated, only not applied. Refused with
    /// [`Error::UnsupportedWarpedFullActiveArea`] when `OpcodeList3` carries
    /// a `WarpRectilinear` that, lerped to [`DevelopOptions::warp_alpha`], is
    /// not the identity and could sample outside the image, where its
    /// clamped edge samples would be visible (`DEC-059` §3, `DEC-024`).
    FullActiveArea,
}

/// Rewrite the resolved, already-validated geometry for `options`
/// (`DEC-058` §3). Validation is [`resolve_geometry`]'s job and has already
/// run; this only changes what the passes downstream read, so none of them
/// grows an option-specific branch.
fn apply_options(mut geometry: Geometry, options: &DevelopOptions) -> Geometry {
    match options.framing {
        Framing::StandardCrop => {}
        Framing::FullActiveArea => {
            geometry.crop_origin_x = 0;
            geometry.crop_origin_y = 0;
            geometry.crop_width = geometry.active_width;
            geometry.crop_height = geometry.active_height;
        }
    }
    if !options.apply_orientation {
        geometry.orientation = 1;
    }
    geometry
}

/// The developed image's dimensions the caller must size `dst` to, without
/// running the transform itself — mirrors how a caller of
/// [`crate::plane::unpack_into`] computes `sensor.width * sensor.height`
/// itself before allocating.
///
/// Exactly [`output_dimensions_with`] with [`DevelopOptions::default()`].
///
/// # Errors
///
/// Whatever [`develop_into`] would reject about `sensor`'s geometry — see its
/// docs. Never fails on `src`/`dst`, because neither is examined here.
pub fn output_dimensions(sensor: &Sensor) -> Result<(u32, u32), Error> {
    output_dimensions_with(sensor, &DevelopOptions::default())
}

/// The dimensions [`develop_with_into`] writes for `options`: the framed
/// size (the `DefaultCrop` size, or the `ActiveArea` size under
/// [`Framing::FullActiveArea`]), swapped for an `Orientation` of 5-8 only
/// when `options.apply_orientation` is set.
///
/// Does not read the opcode lists, so it returns the active size even for a
/// sensor [`develop_with_into`] will refuse with
/// [`Error::UnsupportedWarpedFullActiveArea`].
///
/// # Errors
///
/// [`Error::InvalidWarpAlpha`] first, for a
/// [`DevelopOptions::warp_alpha`] outside `[0, 1]`. Then whatever
/// [`develop_with_into`] would reject about `sensor`'s geometry, under every
/// options value (`DEC-058` §3): [`Error::InvalidActiveArea`],
/// [`Error::InvalidDefaultCrop`], [`Error::UnsupportedOrientation`].
pub fn output_dimensions_with(
    sensor: &Sensor,
    options: &DevelopOptions,
) -> Result<(u32, u32), Error> {
    validate_warp_alpha(options)?;
    let geometry = apply_options(resolve_geometry(sensor)?, options);
    Ok(oriented_dimensions(
        geometry.orientation,
        geometry.crop_width,
        geometry.crop_height,
    ))
}

/// `DEC-059` §2, step 0 of [`develop_with_into`]: `warp_alpha` must be a
/// finite value in `[0, 1]`. `RangeInclusive::contains` is false for NaN and
/// for either infinity, and true for `-0.0`.
fn validate_warp_alpha(options: &DevelopOptions) -> Result<(), Error> {
    if (0.0..=1.0).contains(&options.warp_alpha) {
        Ok(())
    } else {
        Err(Error::InvalidWarpAlpha {
            warp_alpha: options.warp_alpha,
        })
    }
}

/// `OpcodeList3`'s first `WarpRectilinear` lerped to `alpha`
/// ([`crate::opcode::WarpRect::lerp`]), or `None` when the list is absent,
/// carries none, or the lerped warp is the identity (which this path skips —
/// so `alpha = 0` skips every warp).
fn non_identity_warp(
    sensor: &Sensor,
    alpha: f64,
) -> Result<Option<crate::opcode::WarpRect>, Error> {
    let warp = match &sensor.opcode_list_3 {
        Some(bytes) => crate::opcode::parse_warp_rectilinear(bytes)?,
        None => None,
    };
    Ok(warp
        .map(|w| w.lerp(alpha))
        .filter(|w| !crate::warp::is_identity(w)))
}

/// Map a destination (post-orientation) pixel to its position within the
/// **crop rectangle** — `(0..crop_width, 0..crop_height)`, before the
/// active-area and crop-origin offsets are added back in.
///
/// The eight cases are TIFF/Exif `Orientation`'s own semantics: which edge of
/// the correctly-oriented image the stored data's row 0 and column 0
/// correspond to. Derived by hand from that table (module doc, "Provenance")
/// and verified against a worked 2x3 example in this module's tests for all
/// eight values; `SPEC-014` `AC5` exercises 1 and 6 against real corpus
/// frames, the only two orientations any held file carries.
fn crop_source_coords(
    orientation: u32,
    out_x: u32,
    out_y: u32,
    crop_width: u32,
    crop_height: u32,
) -> (u32, u32) {
    let flip_x = |v: u32| crop_width.saturating_sub(1).saturating_sub(v);
    let flip_y = |v: u32| crop_height.saturating_sub(1).saturating_sub(v);
    match orientation {
        1 => (out_x, out_y),
        2 => (flip_x(out_x), out_y),
        3 => (flip_x(out_x), flip_y(out_y)),
        4 => (out_x, flip_y(out_y)),
        5 => (out_y, out_x),
        6 => (out_y, flip_y(out_x)),
        7 => (flip_x(out_y), flip_y(out_x)),
        8 => (flip_x(out_y), out_x),
        // Unreachable: `orientation` is validated to `1..=8` before this is
        // ever called. A total match rather than a partial one, so this
        // function has no panicking path at all (`no-panics-on-untrusted-input`).
        _ => (0, 0),
    }
}

/// The inverse of [`crop_source_coords`]: map a pixel's position **within the
/// crop rectangle** — `(0..crop_width, 0..crop_height)` — to its destination
/// (post-orientation) position. Orientations 1-4 keep a `crop_width ×
/// crop_height` output; 5-8 transpose it to `crop_height × crop_width`.
///
/// The mono path gathers (output → source, [`crop_source_coords`]); the colour
/// path (`SPEC-050`, `DEC-043`) scatters each developed pixel straight into its
/// output slot as the fused kernel emits it, so it needs the source → output
/// direction. Derived by solving each of [`crop_source_coords`]'s eight cases
/// for `(out_x, out_y)`: the mirrors (2, 3, 4) and the transpose (5) and its
/// 180° companion (7) are their own inverses; the two 90° rotations (6, 8) swap
/// roles with each other. Pinned by
/// `output_coords_from_crop_is_the_inverse_of_crop_source_coords`, a round trip
/// over every crop pixel of several shapes for all eight orientations.
pub(crate) fn output_coords_from_crop(
    orientation: u32,
    crop_x: u32,
    crop_y: u32,
    crop_width: u32,
    crop_height: u32,
) -> (u32, u32) {
    let flip_x = |v: u32| crop_width.saturating_sub(1).saturating_sub(v);
    let flip_y = |v: u32| crop_height.saturating_sub(1).saturating_sub(v);
    match orientation {
        1 => (crop_x, crop_y),
        2 => (flip_x(crop_x), crop_y),
        3 => (flip_x(crop_x), flip_y(crop_y)),
        4 => (crop_x, flip_y(crop_y)),
        5 => (crop_y, crop_x),
        6 => (flip_y(crop_y), crop_x),
        7 => (flip_y(crop_y), flip_x(crop_x)),
        8 => (crop_y, flip_x(crop_x)),
        // Unreachable: `orientation` is validated to `1..=8` before this is
        // ever called; a total match keeps this function panic-free.
        _ => (0, 0),
    }
}

/// `BlackLevel`/`WhiteLevel`, defaults applied, validated to a non-empty
/// range.
///
/// `WhiteLevel`'s DNG default when absent is `2^BitsPerSample - 1` — the
/// widest range the sample width can represent. Untested by any corpus file
/// (every decodable file carries an explicit `WhiteLevel`); only a hand-built
/// fixture reaches it.
fn resolve_levels(sensor: &Sensor) -> Result<(u32, u32), Error> {
    let black = sensor.black_level.unwrap_or(0);
    let white = resolve_white(sensor);
    if black >= white {
        return Err(Error::InvalidLevels {
            black_level: black,
            white_level: white,
        });
    }
    Ok((black, white))
}

/// Each photosite's black, resolved for one develop (`SPEC-066`,
/// `DEC-061` §3). `Copy` and borrowed from `sensor.black_levels`, so
/// resolving it allocates nothing.
#[derive(Clone, Copy)]
enum Blacks<'a> {
    /// One black for every photosite: [`BlackLevels::FirstValue`], and any
    /// `PerPosition` pattern whose values are all equal (or absent).
    Uniform(u32),
    /// `values[row × cols + col]`, with `row = (raw_y − top) mod rows` and
    /// `col = (raw_x − left) mod cols`. `values.len() == rows × cols`, both
    /// at least 1, checked by [`resolve_levels_with`].
    Pattern {
        values: &'a [u32],
        rows: u32,
        cols: u32,
        left: u32,
        top: u32,
    },
}

impl Blacks<'_> {
    /// The black of the photosite at raw `(raw_x, raw_y)`. Total: the
    /// passes read only inside `ActiveArea`, so `raw ≥ origin` and the index
    /// is in range; the fallbacks are never reached.
    fn at(&self, raw_x: u32, raw_y: u32) -> u32 {
        match *self {
            Blacks::Uniform(black) => black,
            Blacks::Pattern {
                values,
                rows,
                cols,
                left,
                top,
            } => {
                let row = raw_y.checked_sub(top).and_then(|v| v.checked_rem(rows));
                let col = raw_x.checked_sub(left).and_then(|v| v.checked_rem(cols));
                row.zip(col)
                    .and_then(|(row, col)| row.checked_mul(cols)?.checked_add(col))
                    .and_then(|i| usize::try_from(i).ok())
                    .and_then(|i| values.get(i).copied())
                    .unwrap_or(0)
            }
        }
    }
}

/// [`resolve_levels`] for `options.black_levels` (`DEC-061` §3).
/// [`BlackLevels::FirstValue`] is [`resolve_levels`] exactly.
fn resolve_levels_with<'a>(
    sensor: &'a Sensor,
    options: &DevelopOptions,
) -> Result<(Blacks<'a>, u32), Error> {
    let values = sensor.black_levels.as_slice();
    if options.black_levels == BlackLevels::FirstValue || values.is_empty() {
        let (black, white) = resolve_levels(sensor)?;
        return Ok((Blacks::Uniform(black), white));
    }
    let white = resolve_white(sensor);
    let [rows, cols] = sensor.black_level_repeat_dim.unwrap_or([1, 1]);
    let fits = rows >= 1
        && cols >= 1
        && rows.checked_mul(cols).and_then(|n| usize::try_from(n).ok()) == Some(values.len());
    if !fits {
        return Err(Error::InvalidBlackLevelPattern {
            repeat_rows: rows,
            repeat_cols: cols,
            count: values.len(),
        });
    }
    if let Some(&black_level) = values.iter().find(|&&v| v >= white) {
        return Err(Error::InvalidLevels {
            black_level,
            white_level: white,
        });
    }
    // All equal (a 1×1 pattern included): today's scalar loop, no per-pixel
    // index at all.
    if let Some(&first) = values.first() {
        if values.iter().all(|&v| v == first) {
            return Ok((Blacks::Uniform(first), white));
        }
    }
    let (left, top) = sensor
        .active_area
        .map_or((0, 0), |area| (area.left, area.top));
    Ok((
        Blacks::Pattern {
            values,
            rows,
            cols,
            left,
            top,
        },
        white,
    ))
}

/// `WhiteLevel`, or its DNG default `2^BitsPerSample - 1` when absent
/// ([`resolve_levels`]). Per sample, never per position (`DEC-061` §3).
fn resolve_white(sensor: &Sensor) -> u32 {
    sensor
        .white_level
        .unwrap_or_else(|| max_value_for_bits(sensor.bits_per_sample).unwrap_or(u32::MAX))
}

/// `2^bits - 1`, without the panicking `pow`/shift operators
/// `clippy::arithmetic_side_effects` denies at the crate root. `None` if
/// `bits` is large enough that `2^bits` does not fit `u32` — a malformed
/// `BitsPerSample` this function declines to guess about rather than wrap.
fn max_value_for_bits(bits: u32) -> Option<u32> {
    2u32.checked_pow(bits)?.checked_sub(1)
}

/// Map one raw sample through `[BlackLevel, WhiteLevel] -> [0, u16::MAX]`.
///
/// **`AC2`'s chosen behaviour (`DEC-018`): clamp.** A sample outside the
/// range is clamped to `[black, white]` before scaling, so it maps to exactly
/// 0 or exactly `u16::MAX` rather than wrapping or producing a value outside
/// the normalized range. Both measured real files have samples below
/// `BlackLevel` (`AC2` is live on the first file, not hypothetical) and both
/// reach `WhiteLevel` exactly.
///
/// Integer arithmetic throughout (`DEC-002`'s determinism concern applies to
/// this affine map just as much as a tone curve — no `powf`, no per-target
/// rounding differences). `white > black` is guaranteed by
/// [`resolve_levels`]'s own check before this is ever called.
fn normalize(sample: u16, black: u32, white: u32) -> u16 {
    let clamped = u32::from(sample).clamp(black, white);
    let numerator = u64::from(clamped.saturating_sub(black)).saturating_mul(u64::from(u16::MAX));
    let denominator = u64::from(white.saturating_sub(black));
    // Round to nearest rather than truncate, so the interior isn't biased
    // low. `denominator` is never 0 here (`resolve_levels`), but the
    // fallback keeps this function total rather than trusting the caller.
    let half = denominator.checked_div(2).unwrap_or(0);
    let scaled = numerator
        .checked_add(half)
        .and_then(|v| v.checked_div(denominator))
        .unwrap_or(0);
    u16::try_from(scaled).unwrap_or(u16::MAX)
}

/// The eight 3x3-neighbour offsets, excluding the centre — `SPEC-017`'s
/// `## Notes for the Implementer`, rule 1.
const NEIGHBOR_OFFSETS: [(i8, i8); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Apply `delta` (`-1`, `0`, or `1`) to `v`, returning `None` if the result
/// would be negative or `>= bound` — the plane-extent exclusion rule
/// (`SPEC-017` Notes rule 1), expressed with `checked_add`/`checked_sub`
/// rather than a signed cast so this stays inside
/// `clippy::arithmetic_side_effects`'s allowed operations.
fn offset_coord(v: u32, delta: i8, bound: u32) -> Option<u32> {
    let candidate = match delta {
        -1 => v.checked_sub(1)?,
        0 => v,
        1 => v.checked_add(1)?,
        _ => return None,
    };
    (candidate < bound).then_some(candidate)
}

/// Row-major index into a `width`-wide plane, bounds-checked against
/// `usize`/`u64` overflow the same way every other index computation in this
/// module is (`crop_orient_normalize_into`, etc.).
fn plane_index(width: u32, x: u32, y: u32) -> Option<usize> {
    u64::from(y)
        .checked_mul(u64::from(width))
        .and_then(|v| v.checked_add(u64::from(x)))
        .and_then(|v| usize::try_from(v).ok())
}

/// The lower-of-two-middles median of a non-empty, ALREADY-SORTED slice
/// (`SPEC-017` Notes rule 3's deterministic tie-break: odd count -> the
/// middle; even count -> the lower of the two middles). `None` only if
/// `sorted` is empty — callers only reach this with `>= 1` element.
fn lower_median(sorted: &[u16]) -> Option<u16> {
    let len = sorted.len();
    let half = len.checked_div(2)?;
    let is_odd = len.checked_rem(2) == Some(1);
    let index = if is_odd { half } else { half.checked_sub(1)? };
    sorted.get(index).copied()
}

/// `OpcodeList1`'s `FixBadPixelsConstant` (`SPEC-017`, DNG 1.7.0.0 Chapter 7
/// p.95, `src/opcode.rs`'s own module doc for the chapter correction):
/// replace every raw-plane sample equal to `constant`, within
/// `active_area`, with the median of its 3x3 valid neighbours. Mutates
/// `plane` in place — DNG's own semantics for this opcode, not a
/// `develop_into`-specific choice (module docs, "Allocation"). Allocates
/// nothing (`SPEC-065`).
///
/// Candidate pixels are restricted to `active_area` (`## Notes`: the
/// padding columns outside it are not a real image region, matching
/// `dnglab`'s own boundary). Each candidate's NEIGHBOURHOOD is drawn from
/// the full `width x height` plane extent, not clipped to `active_area` — a
/// bad pixel at the active-area edge may still borrow a genuine sensor
/// reading from the padding (Notes rule 1 names the PLANE extent, not
/// `active_area`, as the neighbour boundary).
///
/// # Rules (pre-registered, `SPEC-017`'s `## Notes for the Implementer`)
///
/// 1. Neighbours outside the plane extent (`0..width`, `0..height`) are
///    excluded.
/// 2. Neighbours whose value equals `constant` are excluded (they are
///    themselves bad).
/// 3. With `>= 1` valid neighbour, replace with `lower_median` of the
///    valid ones.
/// 4. With `0` valid neighbours, the pixel is left AT `constant` and is
///    **not** counted in the returned total — `AC5`'s `count > 0` refers to
///    rules 1-3 only. A pixel left this way is still `constant` after this
///    function returns, so a caller can recover that sub-count by
///    rescanning `active_area` for remaining `constant` values (this
///    build's `## Notes`-required separate reporting, kept out of the
///    `AC4`-pinned `Result<usize, Error>` signature rather than widened
///    into a tuple).
///
/// # Required processing — the caller validates
///
/// This is the **applier** of `OpcodeList1`'s `FixBadPixelsConstant`, not a
/// develop entry: it takes no [`crate::ifd::Sensor`], so it cannot see the
/// file's other opcodes or whether a list was unreadable (`SPEC-056`,
/// `DEC-053`). A caller composing its own pipeline from this function owns
/// that question and should ask it first with
/// [`crate::opcode::check_required_processing`] (`DevelopPath::Mono` if it
/// also applies `OpcodeList3`'s `WarpRectilinear`, as [`develop_into`] does).
///
/// # Errors
///
/// [`Error::FixBadPixelsPlaneWrongLength`] if `plane` does not hold exactly
/// `width * height` samples.
pub fn apply_fix_bad_pixels_constant(
    plane: &mut [u16],
    width: u32,
    height: u32,
    active_area: ActiveArea,
    constant: u32,
) -> Result<usize, Error> {
    let expected = u64::from(width).checked_mul(u64::from(height)).ok_or(
        Error::FixBadPixelsPlaneWrongLength {
            expected: u64::MAX,
            actual: plane.len(),
        },
    )?;
    let expected_len =
        usize::try_from(expected).map_err(|_| Error::FixBadPixelsPlaneWrongLength {
            expected,
            actual: plane.len(),
        })?;
    if plane.len() != expected_len {
        return Err(Error::FixBadPixelsPlaneWrongLength {
            expected,
            actual: plane.len(),
        });
    }

    let top = active_area.top.min(height);
    let left = active_area.left.min(width);
    let bottom = active_area.bottom.min(height);
    let right = active_area.right.min(width);
    if bottom <= top || right <= left {
        return Ok(0);
    }

    let mut replaced = 0usize;
    // At most eight neighbours, so a fixed array and a fill count: this pass
    // allocates nothing (`SPEC-065`, `DEC-060` §2).
    let mut neighbours = [0u16; NEIGHBOR_OFFSETS.len()];
    for y in top..bottom {
        for x in left..right {
            let Some(index) = plane_index(width, x, y) else {
                continue;
            };
            let Some(&value) = plane.get(index) else {
                continue;
            };
            if u32::from(value) != constant {
                continue;
            }

            let mut filled = 0usize;
            for (dx, dy) in NEIGHBOR_OFFSETS {
                let Some(nx) = offset_coord(x, dx, width) else {
                    continue;
                };
                let Some(ny) = offset_coord(y, dy, height) else {
                    continue;
                };
                let Some(nidx) = plane_index(width, nx, ny) else {
                    continue;
                };
                let Some(&nv) = plane.get(nidx) else {
                    continue;
                };
                if u32::from(nv) != constant {
                    if let Some(slot) = neighbours.get_mut(filled) {
                        *slot = nv;
                        filled = filled.saturating_add(1);
                    }
                }
            }

            if filled == 0 {
                continue; // rule 4: left at constant, not counted here
            }
            let Some(valid) = neighbours.get_mut(..filled) else {
                continue;
            };
            valid.sort_unstable();
            let Some(median) = lower_median(valid) else {
                continue;
            };
            if let Some(slot) = plane.get_mut(index) {
                *slot = median;
            }
            replaced = replaced.saturating_add(1);
        }
    }

    Ok(replaced)
}

/// The crop+orient+normalize pass — the fast, no-warp path: reads directly
/// from the raw plane and writes the final oriented+cropped+normalized
/// image straight into `dst` in one pass. Used when `OpcodeList3` is absent
/// or identity (`develop_into`'s only path before `SPEC-018`, and still the
/// zero-extra-allocation path after it). Assumes `src`/`dst` are already
/// length-checked by the caller.
fn crop_orient_normalize_for(
    sensor: &Sensor,
    geometry: &Geometry,
    blacks: Blacks<'_>,
    white: u32,
    src: &[u16],
    dst: &mut [u16],
) {
    // Branch once, outside the loops: `Uniform` monomorphises to today's loop
    // with a constant black (`SPEC-066` §4).
    match blacks {
        Blacks::Uniform(black) => {
            crop_orient_normalize_into(sensor, geometry, |_, _| black, white, src, dst);
        }
        pattern @ Blacks::Pattern { .. } => {
            let black_at = |x, y| pattern.at(x, y);
            crop_orient_normalize_into(sensor, geometry, black_at, white, src, dst);
        }
    }
}

/// The loop [`crop_orient_normalize_for`] runs; `black_at(raw_x, raw_y)` is
/// each photosite's black, read on the raw plane's own coordinates.
fn crop_orient_normalize_into(
    sensor: &Sensor,
    geometry: &Geometry,
    black_at: impl Fn(u32, u32) -> u32,
    white: u32,
    src: &[u16],
    dst: &mut [u16],
) {
    let (out_width, out_height) = oriented_dimensions(
        geometry.orientation,
        geometry.crop_width,
        geometry.crop_height,
    );
    for out_y in 0..out_height {
        for out_x in 0..out_width {
            let (crop_x, crop_y) = crop_source_coords(
                geometry.orientation,
                out_x,
                out_y,
                geometry.crop_width,
                geometry.crop_height,
            );
            // Structurally in-bounds: `resolve_geometry` already proved
            // `active_left + crop_origin_x + crop_width <= sensor.width`
            // (and the `y` equivalent), and `crop_x < crop_width`,
            // `crop_y < crop_height` by this loop's own ranges. The
            // `unwrap_or`/`get().copied().unwrap_or(0)` fallbacks below are
            // never actually reached — kept so this function has no
            // panicking path at all, the same shape as `plane::pow2`.
            let raw_x = geometry
                .active_left
                .checked_add(geometry.crop_origin_x)
                .and_then(|v| v.checked_add(crop_x))
                .unwrap_or(u32::MAX);
            let raw_y = geometry
                .active_top
                .checked_add(geometry.crop_origin_y)
                .and_then(|v| v.checked_add(crop_y))
                .unwrap_or(u32::MAX);
            let src_index = u64::from(raw_y)
                .checked_mul(u64::from(sensor.width))
                .and_then(|v| v.checked_add(u64::from(raw_x)))
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX);
            let sample = src.get(src_index).copied().unwrap_or(0);

            let out_index = u64::from(out_y)
                .checked_mul(u64::from(out_width))
                .and_then(|v| v.checked_add(u64::from(out_x)))
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX);
            if let Some(slot) = dst.get_mut(out_index) {
                *slot = normalize(sample, black_at(raw_x, raw_y), white);
            }
        }
    }
}

/// The warp path's first stage: normalize the `ActiveArea` window (and ONLY
/// that window — no `DefaultCrop`, no `Orientation` yet) into `dst_active`,
/// `active_width * active_height` samples. This is the coordinate space
/// `WarpRectilinear` operates in (`develop_into`'s module doc, "Pipeline
/// order") — DNG 1.7 Chapter 4: `DefaultCropOrigin`/`Size` describe the
/// "final image area", extracted from an already-fully-processed
/// (opcode-lists included) `ActiveArea`-sized image, not the other way
/// round. Assumes `src`/`dst_active` are already length-checked by the
/// caller.
fn normalize_active_area_for(
    sensor: &Sensor,
    geometry: &Geometry,
    blacks: Blacks<'_>,
    white: u32,
    src: &[u16],
    dst_active: &mut [u16],
) {
    match blacks {
        Blacks::Uniform(black) => {
            normalize_active_area_into(sensor, geometry, |_, _| black, white, src, dst_active);
        }
        pattern @ Blacks::Pattern { .. } => {
            let black_at = |x, y| pattern.at(x, y);
            normalize_active_area_into(sensor, geometry, black_at, white, src, dst_active);
        }
    }
}

/// The loop [`normalize_active_area_for`] runs. The black is taken here, on raw
/// coordinates, BEFORE the warp resamples: black subtraction is per
/// photosite and does not commute with interpolation (`DEC-061` §4).
fn normalize_active_area_into(
    sensor: &Sensor,
    geometry: &Geometry,
    black_at: impl Fn(u32, u32) -> u32,
    white: u32,
    src: &[u16],
    dst_active: &mut [u16],
) {
    for ay in 0..geometry.active_height {
        for ax in 0..geometry.active_width {
            // In-bounds by `resolve_geometry`'s own check
            // (`active_right <= sensor.width`, `active_bottom <=
            // sensor.height`); the fallbacks keep this function total.
            let raw_x = geometry.active_left.saturating_add(ax);
            let raw_y = geometry.active_top.saturating_add(ay);
            let src_index = u64::from(raw_y)
                .checked_mul(u64::from(sensor.width))
                .and_then(|v| v.checked_add(u64::from(raw_x)))
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX);
            let sample = src.get(src_index).copied().unwrap_or(0);

            let active_index = u64::from(ay)
                .checked_mul(u64::from(geometry.active_width))
                .and_then(|v| v.checked_add(u64::from(ax)))
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX);
            if let Some(slot) = dst_active.get_mut(active_index) {
                *slot = normalize(sample, black_at(raw_x, raw_y), white);
            }
        }
    }
}

/// The warp path's last stage: `DefaultCrop` extraction + `Orientation`,
/// reading from an `ActiveArea`-sized buffer (already normalized, and
/// already warped if a real `WarpRectilinear` was present) into `dst`,
/// `out_width * out_height` samples. Assumes `active_src`/`dst` are already
/// length-checked by the caller.
fn crop_and_orient_from_active_into(
    geometry: &Geometry,
    out_width: u32,
    out_height: u32,
    active_src: &[u16],
    dst: &mut [u16],
) {
    for out_y in 0..out_height {
        for out_x in 0..out_width {
            let (crop_x, crop_y) = crop_source_coords(
                geometry.orientation,
                out_x,
                out_y,
                geometry.crop_width,
                geometry.crop_height,
            );
            // In-bounds by `resolve_geometry`'s own check (`crop_origin +
            // crop_size <= active_width/height`); the fallbacks keep this
            // function total.
            let active_x = geometry.crop_origin_x.saturating_add(crop_x);
            let active_y = geometry.crop_origin_y.saturating_add(crop_y);
            let active_index = u64::from(active_y)
                .checked_mul(u64::from(geometry.active_width))
                .and_then(|v| v.checked_add(u64::from(active_x)))
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX);
            let sample = active_src.get(active_index).copied().unwrap_or(0);

            let out_index = u64::from(out_y)
                .checked_mul(u64::from(out_width))
                .and_then(|v| v.checked_add(u64::from(out_x)))
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(usize::MAX);
            if let Some(slot) = dst.get_mut(out_index) {
                *slot = sample;
            }
        }
    }
}

/// Develop the sensor plane [`crate::plane::unpack_into`] produced into the
/// image a consumer would display: `OpcodeList1`'s `FixBadPixelsConstant`
/// defect correction applied to the raw plane FIRST (`SPEC-017`), levels
/// normalized, `OpcodeList3`'s `WarpRectilinear` geometric correction
/// applied (`SPEC-018`), then the three-stage crop applied and oriented.
///
/// ⚠ **`FixBadPixelsConstant` runs before normalization, not after.** DNG
/// 1.7.0.0 Chapter 7: `OpcodeList1` opcodes act on the raw plane in its own
/// value range (`Constant` is a raw sample value, e.g. `0` on every Q2M
/// frame — nowhere near the normalized `[0, 65535]` range `WhiteLevel`
/// would otherwise scale it into). `docs/measured-q2m-dng.md` line 44 states
/// the same order. See [`apply_fix_bad_pixels_constant`]'s own doc for the
/// replacement rules, and this module's own doc ("Allocation") for the
/// working memory a real `OpcodeList1` and `OpcodeList3` need.
///
/// ⚠ **Pipeline order, corrected from `SPEC-018`'s own design-time
/// assumption.** `SPEC-018`'s own design pass stated "OpcodeList3
/// runs after cropping and orientation" — this build's design-time probe
/// against DNG 1.7.0.0's own tag descriptions found that wrong:
/// `DefaultCropOrigin`/`Size` (§Chapter 4) describe "the origin/size of the
/// FINAL image area... relative to the top-left corner of the `ActiveArea`
/// rectangle", and `OpcodeList3`'s own tag description says it runs "just
/// after \[the raw image\] has been demosaiced" — i.e. on the full
/// `ActiveArea`-sized image, with `DefaultCrop` extracting the smaller
/// "final" display rectangle LAST. The warp therefore runs on the
/// `ActiveArea` window, BEFORE `DefaultCrop` extraction and `Orientation`,
/// not after. (No comparison oracle can confirm this choice either way —
/// see `## No warp-correctness oracle exists` below — so this is verified
/// against the specification text, not a tool.) The tone-curve stage
/// `SPEC-019` will add after this one is, for now, a no-op: this function's
/// output IS the final image.
///
/// `src` must hold exactly `sensor.width * sensor.height` samples (the
/// uncropped, un-normalised plane) and `dst` must hold exactly
/// [`output_dimensions`]`(sensor)`'s product — the caller computes both, the
/// same caller-owned-buffer shape `DEC-016` chose for `unpack_into`.
///
/// # No warp-correctness oracle exists — read before trusting a score
///
/// `SPEC-020`'s oracle (SSIMULACRA2 against `dnglab analyze --srgb`) CANNOT
/// validate this function's warp stage: `dnglab`/`rawler` do not implement
/// DNG `OpcodeList` processing at all (verified by inspecting the
/// `dnglab/dnglab` source: `OpcodeList1`/`2`/`3` appear only as tag ID
/// constants and `IFD::copy_tag` pass-through calls in
/// `rawler/src/decoders/dng.rs`; `WarpRectilinear` and
/// `FixBadPixelsConstant` appear nowhere in the repository). `--srgb`'s
/// reference render is therefore UNCORRECTED, so a MORE geometrically
/// correct warp scores WORSE against it, not better — this is not a kernel
/// or implementation defect, see the kernel-choice decision, `DEC-024`
/// Finding 2, and the handback's `AC8`/`AC9` finding. Correctness here rests
/// on `tests/warp.rs`'s analytic checks (`AC4`, `AC10`) instead — an exact,
/// independently-derived transcription of DNG 1.7.0.0 §6.4.1, verified
/// against `SPIKE-001`'s measured ~504 px figure on real per-frame
/// coefficients.
///
/// # Allocation — `SPEC-065`, `DEC-060`
///
/// At most **one** internal allocation, of exactly
/// [`scratch_len`]`(sensor, &DevelopOptions::default())` samples, and none
/// when that is 0 (no `FixBadPixelsConstant` and no non-identity
/// `WarpRectilinear`). It holds the fixed copy of the raw plane and the
/// warp's active-area buffers, P + A samples at most (module docs,
/// "Allocation"). On a Q2M frame: one allocation of 189,415,424 bytes, where
/// it was six totalling 283,943,504 before `SPEC-065`. A caller that wants to
/// own that memory uses [`develop_with_scratch_into`], which allocates
/// nothing.
///
/// # Errors
///
/// - [`Error::UnsupportedRequiredOpcode`] (checked first, `SPEC-054`) if any
///   opcode list carries a mandatory opcode this path does not apply — anything
///   in `OpcodeList2`, or anything beyond the first `FixBadPixelsConstant` in
///   `OpcodeList1` and the first `WarpRectilinear` in `OpcodeList3`; or
///   [`Error::UnsupportedMandatoryOpcode`] for an unrecognized mandatory ID in
///   any list.
/// - [`Error::SourcePlaneWrongLength`] / [`Error::DevelopBufferWrongLength`]
///   if `src`/`dst` do not hold exactly the length required.
/// - [`Error::InvalidActiveArea`] if `ActiveArea` is empty, inverted, or
///   extends past the raw plane.
/// - [`Error::InvalidDefaultCrop`] if `DefaultCropSize` is zero in either
///   dimension, or `DefaultCropOrigin + DefaultCropSize` does not fit inside
///   `ActiveArea`.
/// - [`Error::UnsupportedOrientation`] if `Orientation` is present and
///   outside `1..=8`.
/// - [`Error::InvalidLevels`] if `BlackLevel >= WhiteLevel`.
/// - Whatever [`crate::opcode::parse_fix_bad_pixels_constant`] or
///   [`apply_fix_bad_pixels_constant`] return, if `OpcodeList1` is present
///   — including [`Error::UnsupportedMandatoryOpcode`] if the list carries a
///   DIFFERENT, unrecognized mandatory opcode (`AC6`).
/// - Whatever [`crate::opcode::parse_warp_rectilinear`] or
///   [`crate::warp::apply_warp_into`] return, if `OpcodeList3` is present.
///
/// # Options
///
/// Exactly [`develop_with_into`] with [`DevelopOptions::default()`]
/// (`DEC-058` §2): it delegates, so the two cannot drift. Use
/// [`develop_with_into`] for the uncropped active area or unrotated output.
pub fn develop_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error> {
    develop_with_into(sensor, src, dst, &DevelopOptions::default())
}

/// [`develop_into`] with [`DevelopOptions`] (`SPEC-063`, `DEC-058`): the same
/// pipeline — `FixBadPixelsConstant`, levels, `WarpRectilinear` over the
/// `ActiveArea`, then framing and orientation — with the framing and the
/// orientation step chosen by `options`. See [`develop_into`] for the
/// pipeline order, the allocation notes and every error it shares. It makes
/// one allocation of exactly [`scratch_len`]`(sensor, options)` samples (none
/// when that is 0); [`develop_with_scratch_into`] is the same develop into
/// caller-owned scratch, with none.
///
/// `dst` must hold exactly [`output_dimensions_with`]`(sensor, options)`'s
/// product. **Mono only**: the colour path takes no options.
///
/// # Errors
///
/// Every error [`develop_into`] documents, under every options value — an
/// option never skips a tag's validation (`DEC-058` §3) — plus:
///
/// - [`Error::InvalidWarpAlpha`] for a [`DevelopOptions::warp_alpha`] that is
///   not a finite value in `[0, 1]` (`DEC-059` §2). It is checked **first**,
///   before the required-processing check and before any buffer or tag.
/// - [`Error::UnsupportedWarpedFullActiveArea`] for
///   [`Framing::FullActiveArea`] when `OpcodeList3`'s `WarpRectilinear`,
///   lerped to `warp_alpha`, is not the identity and fails `DEC-059` §3's
///   bounds rule (a coefficient not finite, the optical centre outside the
///   image, or the radial scale leaving `[0, 1]`). It is checked after the
///   buffer lengths and before `OpcodeList1` is parsed, so it comes before
///   the `FixBadPixelsConstant` plane copy and before any write to `dst`. A
///   consequence: under `FullActiveArea` a malformed `OpcodeList3` is
///   reported before a malformed `OpcodeList1`; under
///   [`Framing::StandardCrop`] the order is [`develop_into`]'s. The bounds
///   rule reads the radial scale only: a warp with non-zero tangential terms
///   whose radial scale passes reaches
///   [`Error::UnsupportedWarpTangentialTerms`] from the applier, at any
///   `warp_alpha > 0`.
pub fn develop_with_into(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    options: &DevelopOptions,
) -> Result<(), Error> {
    develop_core(sensor, src, dst, Scratch::Allocate, options)
}

/// How many `u16` samples of scratch [`develop_with_scratch_into`] needs for
/// this sensor and these options (`SPEC-065`, `DEC-060`).
///
/// `0` when the develop needs no scratch (no `FixBadPixelsConstant` in
/// `OpcodeList1` and no non-identity `WarpRectilinear` in `OpcodeList3` at
/// [`DevelopOptions::warp_alpha`]); the caller may then pass an empty slice.
/// Otherwise room for a fixed copy of the raw plane and/or the
/// active-area-sized buffers the warp reads and writes.
///
/// It depends only on `(sensor, options)`, but **ask for it rather than
/// hard-coding it**: a later irradiance version may need a different amount
/// for the same file without the output changing (and without a
/// [`crate::DEVELOP_VERSION`] bump).
///
/// It answers "how much scratch", not "will this develop succeed": it reads
/// the geometry and the two opcode lists, but does not run the
/// required-processing check, the levels check or the
/// [`Framing::FullActiveArea`] bounds check. [`develop_with_scratch_into`]
/// still runs all of them.
///
/// # Errors
///
/// - [`Error::InvalidWarpAlpha`] for an out-of-domain
///   [`DevelopOptions::warp_alpha`].
/// - [`Error::InvalidActiveArea`], [`Error::InvalidDefaultCrop`] or
///   [`Error::UnsupportedOrientation`] from the geometry, as
///   [`output_dimensions_with`] reports them.
/// - Whatever [`crate::opcode::parse_fix_bad_pixels_constant`] or
///   [`crate::opcode::parse_warp_rectilinear`] return for a present list.
/// - [`Error::DevelopScratchWrongLength`] `{ expected: u64::MAX, actual: 0 }`
///   when the length overflows (a hostile header whose plane is near
///   `u32::MAX` squared samples; no real plane is).
pub fn scratch_len(sensor: &Sensor, options: &DevelopOptions) -> Result<usize, Error> {
    validate_warp_alpha(options)?;
    let geometry = apply_options(resolve_geometry(sensor)?, options);
    let fix_bad_pixels = match &sensor.opcode_list_1 {
        Some(bytes) => crate::opcode::parse_fix_bad_pixels_constant(bytes)?.is_some(),
        None => false,
    };
    let warp = non_identity_warp(sensor, options.warp_alpha)?.is_some();
    needed_scratch(sensor, &geometry, fix_bad_pixels, warp, 0)
}

/// [`develop_with_into`] into caller-owned scratch, **with no heap
/// allocation at all** (`SPEC-065`, `DEC-060`).
///
/// `scratch` must hold **exactly** [`scratch_len`]`(sensor, options)`
/// samples — the same exact-length rule as `src` and `dst`. A caller reusing
/// one large buffer across frames passes `&mut buf[..len]`. When
/// [`scratch_len`] is `0`, pass an empty slice.
///
/// - **No allocation**, on success and on every error path. The opcode
///   lookups it runs allocate nothing either.
/// - **Scratch contents are unspecified** on entry and on return. Every
///   sample is written before it is read, so the output never depends on
///   what the caller left there; do not read meaning into it afterwards.
/// - **Output is identical** to [`develop_with_into`]`(sensor, src, dst,
///   options)` for every input and options value: the two run the same code,
///   which `develop_with_into` feeds with one allocation of exactly
///   [`scratch_len`] samples.
/// - **Mono only**, like [`develop_with_into`].
///
/// # Errors
///
/// Every error [`develop_with_into`] returns, in the same order, plus
/// [`Error::DevelopScratchWrongLength`] when `scratch.len()` is not exactly
/// [`scratch_len`]. That check runs after both opcode lists are parsed (so a
/// malformed list is reported first) and before anything is written to `dst`
/// or `scratch`.
pub fn develop_with_scratch_into(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    scratch: &mut [u16],
    options: &DevelopOptions,
) -> Result<(), Error> {
    develop_core(sensor, src, dst, Scratch::Caller(scratch), options)
}

/// Where [`develop_core`] gets its scratch: the caller's slice
/// ([`develop_with_scratch_into`]) or one allocation of exactly the needed
/// length ([`develop_with_into`]).
enum Scratch<'a> {
    Caller(&'a mut [u16]),
    Allocate,
}

/// `DEC-060` §3's scratch length for a plane of `plane` samples and an active
/// area of `active`, or `None` on overflow. The layout it pays for is in
/// [`run_passes`].
fn layout_len(plane: u64, active: u64, fix_bad_pixels: bool, warp: bool) -> Option<u64> {
    match (fix_bad_pixels, warp) {
        (false, false) => Some(0),
        (true, false) => Some(plane),
        (false, true) => active.checked_mul(2),
        // The warp output reuses the dead plane copy (`A <= P`).
        (true, true) => plane.checked_add(active),
    }
}

/// [`layout_len`] for this sensor and resolved geometry, as a `usize`, or
/// [`Error::DevelopScratchWrongLength`] carrying `actual` when it cannot be
/// represented.
fn needed_scratch(
    sensor: &Sensor,
    geometry: &Geometry,
    fix_bad_pixels: bool,
    warp: bool,
    actual: usize,
) -> Result<usize, Error> {
    let plane = u64::from(sensor.width).checked_mul(u64::from(sensor.height));
    let active = u64::from(geometry.active_width).checked_mul(u64::from(geometry.active_height));
    let expected = plane
        .zip(active)
        .and_then(|(plane, active)| layout_len(plane, active, fix_bad_pixels, warp))
        .ok_or(Error::DevelopScratchWrongLength {
            expected: u64::MAX,
            actual,
        })?;
    usize::try_from(expected).map_err(|_| Error::DevelopScratchWrongLength { expected, actual })
}

/// Everything the passes read, resolved and validated by [`develop_core`].
struct Plan<'a> {
    geometry: Geometry,
    /// Each photosite's black (`SPEC-066`).
    blacks: Blacks<'a>,
    white: u32,
    out_width: u32,
    out_height: u32,
    /// `OpcodeList1`'s first `FixBadPixelsConstant` constant.
    fix_bad_pixels: Option<u32>,
    /// `OpcodeList3`'s first `WarpRectilinear`, lerped, non-identity.
    warp: Option<crate::opcode::WarpRect>,
}

/// The one develop both public entries run (`DEC-060` §4). Validation order,
/// unchanged from before `SPEC-065` except for the new step 9:
///
/// 0. `warp_alpha`; 1. required processing; 2. `src` length;
/// 3. geometry, then options; 4. levels; 5. `dst` length;
/// 6. under `FullActiveArea`, `OpcodeList3` and the bounds rule;
/// 7. `OpcodeList1`; 8. under `StandardCrop`, `OpcodeList3`;
/// 9. the scratch length (refused, or allocated); 10. the passes.
///
/// Nothing is written to `dst` or the scratch before step 10.
fn develop_core(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    scratch: Scratch<'_>,
    options: &DevelopOptions,
) -> Result<(), Error> {
    // Step 0 (`DEC-059` §2): `warp_alpha` is a pure argument check, so it is
    // reported before anything about the file, under every framing.
    validate_warp_alpha(options)?;

    // Required-processing validation next, before any allocation or decode
    // (`SPEC-054`, `DEC-049`): a mandatory opcode this path will not apply is a
    // typed error, never a silently-uncorrected image.
    crate::opcode::check_required_processing(sensor, crate::opcode::DevelopPath::Mono)?;

    let expected_src = u64::from(sensor.width)
        .checked_mul(u64::from(sensor.height))
        .ok_or(Error::SourcePlaneWrongLength {
            expected: u64::MAX,
            actual: src.len(),
        })?;
    let expected_src_len =
        usize::try_from(expected_src).map_err(|_| Error::SourcePlaneWrongLength {
            expected: expected_src,
            actual: src.len(),
        })?;
    if src.len() != expected_src_len {
        return Err(Error::SourcePlaneWrongLength {
            expected: expected_src,
            actual: src.len(),
        });
    }

    // Validate first (unchanged), then let the options rewrite what the
    // passes below read (`DEC-058` §3).
    let geometry = apply_options(resolve_geometry(sensor)?, options);
    // Step 4: levels, under `options.black_levels` (`DEC-061` §4: the pattern
    // check sits exactly where the levels check always was).
    let (blacks, white) = resolve_levels_with(sensor, options)?;
    let (out_width, out_height) = oriented_dimensions(
        geometry.orientation,
        geometry.crop_width,
        geometry.crop_height,
    );

    let expected_dst = u64::from(out_width)
        .checked_mul(u64::from(out_height))
        .ok_or(Error::DevelopBufferWrongLength {
            expected: u64::MAX,
            actual: dst.len(),
        })?;
    let expected_dst_len =
        usize::try_from(expected_dst).map_err(|_| Error::DevelopBufferWrongLength {
            expected: expected_dst,
            actual: dst.len(),
        })?;
    if dst.len() != expected_dst_len {
        return Err(Error::DevelopBufferWrongLength {
            expected: expected_dst,
            actual: dst.len(),
        });
    }

    // `DEC-059` §3 (amending `DEC-058` §4): under `FullActiveArea` the
    // lerped warp is parsed here and must provably sample only inside the
    // image, or it is refused — before `OpcodeList1` is parsed and before any
    // write. Only under `FullActiveArea`, so the parse order (and error
    // precedence) under `StandardCrop` is exactly the pre-`SPEC-064` one.
    // `Some(warp)` carries the result down so the list is not parsed twice.
    let early_warp = if options.framing == Framing::FullActiveArea {
        let warp = non_identity_warp(sensor, options.warp_alpha)?;
        if let Some(w) = &warp {
            let (scale_min, scale_max) = crate::warp::scale_bounds(w);
            if !crate::warp::full_active_area_admits(w, (scale_min, scale_max)) {
                return Err(Error::UnsupportedWarpedFullActiveArea {
                    active_width: geometry.active_width,
                    active_height: geometry.active_height,
                    warp_alpha: options.warp_alpha,
                    scale_min,
                    scale_max,
                });
            }
        }
        Some(warp)
    } else {
        None
    };

    // `OpcodeList1`'s `FixBadPixelsConstant` (`SPEC-017`). It is APPLIED
    // first, on the raw plane, before levels normalize (module docs,
    // "Pipeline order"); here it is only parsed. The lookup propagates
    // `Error::UnsupportedMandatoryOpcode` for any DIFFERENT unrecognized
    // mandatory opcode in the same list (`AC6`).
    let fix_bad_pixels = match &sensor.opcode_list_1 {
        Some(bytes) => crate::opcode::parse_fix_bad_pixels_constant(bytes)?,
        None => None,
    };

    let warp = match early_warp {
        Some(warp) => warp,
        None => non_identity_warp(sensor, options.warp_alpha)?,
    };

    let plan = Plan {
        geometry,
        blacks,
        white,
        out_width,
        out_height,
        fix_bad_pixels,
        warp,
    };

    // Step 9 (`DEC-060` §2): after both lists are parsed, before any write.
    match scratch {
        Scratch::Caller(scratch) => {
            let needed = needed_scratch(
                sensor,
                &plan.geometry,
                plan.fix_bad_pixels.is_some(),
                plan.warp.is_some(),
                scratch.len(),
            )?;
            if scratch.len() != needed {
                return Err(Error::DevelopScratchWrongLength {
                    expected: u64::try_from(needed).unwrap_or(u64::MAX),
                    actual: scratch.len(),
                });
            }
            run_passes(sensor, &plan, src, dst, scratch)
        }
        Scratch::Allocate => {
            let needed = needed_scratch(
                sensor,
                &plan.geometry,
                plan.fix_bad_pixels.is_some(),
                plan.warp.is_some(),
                0,
            )?;
            // The one allocation `develop_with_into` makes (none when
            // `needed` is 0: an empty `vec!` does not allocate).
            let mut owned = vec![0u16; needed];
            run_passes(sensor, &plan, src, dst, &mut owned)
        }
    }
}

/// `scratch.split_at_mut(at)`, panic-free. The caller has already checked
/// `scratch`'s length against [`layout_len`], so the `Err` is unreachable;
/// it reports the length it needed rather than panicking.
fn split_scratch(scratch: &mut [u16], at: usize) -> Result<(&mut [u16], &mut [u16]), Error> {
    let actual = scratch.len();
    scratch
        .split_at_mut_checked(at)
        .ok_or(Error::DevelopScratchWrongLength {
            expected: u64::try_from(at).unwrap_or(u64::MAX),
            actual,
        })
}

/// Copy the raw plane into `plane` and apply `OpcodeList1`'s
/// `FixBadPixelsConstant` to the copy (`SPEC-017`): `src` is the caller's
/// shared `&[u16]` and is never modified.
fn copy_and_fix_bad_pixels(
    sensor: &Sensor,
    geometry: &Geometry,
    constant: u32,
    src: &[u16],
    plane: &mut [u16],
) -> Result<(), Error> {
    // Equal lengths (P) by construction; `zip` keeps the copy panic-free.
    for (to, from) in plane.iter_mut().zip(src) {
        *to = *from;
    }
    let fix_active_area = ActiveArea {
        top: geometry.active_top,
        left: geometry.active_left,
        bottom: geometry
            .active_top
            .checked_add(geometry.active_height)
            .unwrap_or(sensor.height),
        right: geometry
            .active_left
            .checked_add(geometry.active_width)
            .unwrap_or(sensor.width),
    };
    apply_fix_bad_pixels_constant(
        plane,
        sensor.width,
        sensor.height,
        fix_active_area,
        constant,
    )?;
    Ok(())
}

/// Step 10: the passes, by case, over `scratch` laid out as `DEC-060` §3
/// records (P = the raw plane, A = the active area, `A <= P`):
///
/// | fix | warp | scratch | plane copy | normalized | warped |
/// |---|---|---|---|---|---|
/// | no | no | 0 | | | |
/// | yes | no | P | `[0, P)` | | |
/// | no | yes | 2A | | `[0, A)` | `[A, 2A)` |
/// | yes | yes | P + A | `[0, P)` | `[P, P + A)` | `[0, A)` |
///
/// In the last case the warp writes over the plane copy, which is dead once
/// normalize has read it into `[P, P + A)`. Every scratch sample is written
/// before it is read.
fn run_passes(
    sensor: &Sensor,
    plan: &Plan<'_>,
    src: &[u16],
    dst: &mut [u16],
    scratch: &mut [u16],
) -> Result<(), Error> {
    let geometry = &plan.geometry;
    let (blacks, white) = (plan.blacks, plan.white);
    let active_len = u64::from(geometry.active_width)
        .checked_mul(u64::from(geometry.active_height))
        .and_then(|v| usize::try_from(v).ok())
        .ok_or(Error::DevelopBufferWrongLength {
            expected: u64::MAX,
            actual: dst.len(),
        })?;

    match (plan.fix_bad_pixels, &plan.warp) {
        (None, None) => {
            crop_orient_normalize_for(sensor, geometry, blacks, white, src, dst);
        }
        (Some(constant), None) => {
            copy_and_fix_bad_pixels(sensor, geometry, constant, src, scratch)?;
            crop_orient_normalize_for(sensor, geometry, blacks, white, scratch, dst);
        }
        (None, Some(warp)) => {
            let (normalized, warped) = split_scratch(scratch, active_len)?;
            normalize_active_area_for(sensor, geometry, blacks, white, src, normalized);
            crate::warp::apply_warp_into(
                warp,
                geometry.active_width,
                geometry.active_height,
                normalized,
                warped,
            )?;
            crop_and_orient_from_active_into(
                geometry,
                plan.out_width,
                plan.out_height,
                warped,
                dst,
            );
        }
        (Some(constant), Some(warp)) => {
            let (plane, normalized) = split_scratch(scratch, src.len())?;
            copy_and_fix_bad_pixels(sensor, geometry, constant, src, plane)?;
            normalize_active_area_for(sensor, geometry, blacks, white, plane, normalized);
            // The plane copy is dead now; its first A samples take the warp.
            let (warped, _) = split_scratch(plane, active_len)?;
            crate::warp::apply_warp_into(
                warp,
                geometry.active_width,
                geometry.active_height,
                normalized,
                warped,
            )?;
            crop_and_orient_from_active_into(
                geometry,
                plan.out_width,
                plan.out_height,
                warped,
                dst,
            );
        }
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
    use crate::ifd::{ActiveArea, Compression, DefaultCropOrigin, DefaultCropSize};

    /// A minimal, otherwise-valid `Sensor` a test can override one field of.
    fn base_sensor() -> Sensor {
        Sensor {
            ifd_index: 0,
            width: 4,
            height: 3,
            bits_per_sample: 14,
            samples_per_pixel: 1,
            photometric: 34892,
            compression: Compression::Uncompressed,
            rows_per_strip: None,
            strip_offsets: vec![],
            strip_byte_counts: vec![],
            tile_width: None,
            tile_length: None,
            tile_offsets: vec![],
            tile_byte_counts: vec![],
            black_level: Some(0),
            white_level: Some(16383),
            black_level_repeat_dim: None,
            active_area: None,
            default_crop_origin: None,
            default_crop_size: None,
            orientation: None,
            opcode_lists: [false, false, false],
            opcode_list_1: None,
            opcode_list_2: None,
            opcode_list_3: None,
            color_filter: None,
            // SPEC-026 colour metadata — additive, not read by this fixture.
            color_matrix_1: None,
            color_matrix_2: None,
            forward_matrix_1: None,
            forward_matrix_2: None,
            as_shot_neutral: None,
            calibration_illuminant_1: None,
            calibration_illuminant_2: None,
            black_levels: vec![],
            white_levels: vec![],
            malformed_tags: vec![],
        }
    }

    #[test]
    fn normalize_maps_the_endpoints_and_an_interior_point() {
        assert_eq!(normalize(512, 512, 16383), 0);
        assert_eq!(normalize(16383, 512, 16383), u16::MAX);
        // Interior: exactly the midpoint. Pinned to its exact value, not a
        // band — but the midpoint is precisely where round-to-nearest and
        // truncation AGREE (both give 32765), so this alone cannot pin the
        // rounding rule. See `normalize_rounds_to_nearest_rather_than_truncating`.
        let mid = 512u32.checked_add(16383).unwrap() / 2;
        let got = normalize(u16::try_from(mid).unwrap(), 512, 16383);
        assert_eq!(got, 32765, "got {got}");
    }

    /// `FU-4` — the doc comment on [`normalize`] claims round-to-nearest, but
    /// nothing asserted it: the interior point pinned above is the midpoint,
    /// the one sample where round-to-nearest and truncation give the same
    /// answer. Measured over Q2M's own levels (`black 512`, `white 16383`):
    /// the two rules disagree on 7,935 of 15,872 in-range samples — 50.0% —
    /// and `516` is the first one. This test fails if someone "simplifies"
    /// away the `+ half` in [`normalize`], because truncation gives `16`
    /// where round-to-nearest gives `17`.
    #[test]
    fn normalize_rounds_to_nearest_rather_than_truncating() {
        let truncated = (u32::from(516u16) - 512) * u32::from(u16::MAX) / (16383 - 512);
        assert_eq!(truncated, 16, "truncation would give 16 at this sample");
        assert_eq!(
            normalize(516, 512, 16383),
            17,
            "normalize must round to nearest (17), not truncate (16)"
        );
    }

    #[test]
    fn normalize_clamps_below_black_and_above_white() {
        assert_eq!(normalize(0, 512, 16383), 0, "below BlackLevel clamps to 0");
        assert_eq!(
            normalize(u16::MAX, 512, 16383),
            u16::MAX,
            "above WhiteLevel clamps to u16::MAX"
        );
    }

    #[test]
    fn resolve_levels_rejects_black_at_or_above_white() {
        let mut sensor = base_sensor();
        sensor.black_level = Some(16383);
        sensor.white_level = Some(16383);
        let err = resolve_levels(&sensor).expect_err("black == white has no range to map onto");
        assert!(matches!(err, Error::InvalidLevels { .. }), "{err:?}");
    }

    #[test]
    fn absent_white_level_defaults_to_the_bit_depth_maximum() {
        let mut sensor = base_sensor();
        sensor.bits_per_sample = 12;
        sensor.white_level = None;
        let (_, white) = resolve_levels(&sensor).expect("12-bit default is 4095");
        assert_eq!(white, 4095);
    }

    #[test]
    fn absent_active_area_is_the_whole_raw_plane() {
        let mut sensor = base_sensor(); // 4x3, no ActiveArea
        let geometry = resolve_geometry(&sensor).expect("no ActiveArea is legal");
        assert_eq!((geometry.active_left, geometry.active_top), (0, 0));

        // A crop the size of the WHOLE raw plane must fit exactly — it would
        // not, if the active area had defaulted to anything smaller.
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 4,
            height: 3,
        });
        assert_eq!(output_dimensions(&sensor).expect("fits"), (4, 3));
    }

    #[test]
    fn crop_origin_is_relative_to_active_area_not_the_raw_plane() {
        // AC4. A hand-built Sensor with a NON-ZERO ActiveArea origin — the
        // one shape no decodable corpus file carries (SPEC-014's "blind
        // spot": K3III.DNG is the only real file with a non-zero origin, and
        // it is JPEG-compressed, undecodable). "Relative to ActiveArea" and
        // "relative to the raw plane" give IDENTICAL output whenever the
        // origin is (0, 0) — this fixture is the only thing that can tell
        // them apart (DEC-019).
        let mut sensor = base_sensor();
        sensor.width = 10;
        sensor.height = 10;
        sensor.active_area = Some(ActiveArea {
            top: 3,
            left: 2,
            bottom: 9,
            right: 8,
        }); // 6x6 active area, origin (2, 3)
        sensor.default_crop_origin = Some(DefaultCropOrigin { x: 1, y: 1 });
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 2,
            height: 2,
        });

        // Raw plane, row-major, 10x10: sample value == row*10 + col, so the
        // sample AT a coordinate names its own source position.
        let src: Vec<u16> = (0..100u32).map(|v| u16::try_from(v).unwrap()).collect();
        let mut dst = [0u16; 4]; // 2x2 crop
        develop_into(&sensor, &src, &mut dst).expect("fits");

        // If the crop origin were relative to the RAW PLANE (the wrong
        // reading), the top-left output sample would come from (1, 1) = 11,
        // normalized. Relative to ActiveArea (correct — DEC-019), it comes
        // from active_left + crop_origin_x = 2 + 1 = 3, active_top +
        // crop_origin_y = 3 + 1 = 4, i.e. raw sample (row 4, col 3) = 43.
        let expected_top_left_raw_sample = 43u16;
        let expected = normalize(expected_top_left_raw_sample, 0, 16383);
        assert_eq!(
            dst[0], expected,
            "DefaultCropOrigin must be applied relative to ActiveArea, not the raw plane"
        );

        let wrong_reading_sample = 11u16; // what "relative to the raw plane" would read
        assert_ne!(
            dst[0],
            normalize(wrong_reading_sample, 0, 16383),
            "this fixture must be able to observe the wrong reading, or it proves nothing"
        );
    }

    #[test]
    fn the_three_stage_crop_produces_the_documented_dimensions() {
        // Q2M's own measured shape (SPEC-014 Implementation Context):
        // 8424x5632 -> ActiveArea 8392x5632 -> DefaultCrop 8368x5584.
        let mut sensor = base_sensor();
        sensor.width = 8424;
        sensor.height = 5632;
        sensor.active_area = Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 5632,
            right: 8392,
        });
        sensor.default_crop_origin = Some(DefaultCropOrigin { x: 12, y: 24 });
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 8368,
            height: 5584,
        });
        let dims = output_dimensions(&sensor).expect("fits");
        assert_eq!(dims, (8368, 5584));
    }

    #[test]
    fn orientation_six_swaps_the_output_dimensions() {
        let mut sensor = base_sensor();
        sensor.width = 8;
        sensor.height = 6;
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 8,
            height: 6,
        });
        sensor.orientation = Some(6);
        assert_eq!(output_dimensions(&sensor).expect("fits"), (6, 8));

        sensor.orientation = Some(1);
        assert_eq!(output_dimensions(&sensor).expect("fits"), (8, 6));
    }

    #[test]
    fn crop_source_coords_matches_the_worked_example_for_all_eight_orientations() {
        // A 2-wide x 3-tall crop (crop_width=2, crop_height=3). Three
        // destination points per orientation — the top-left corner (A), and
        // the two OTHER destination corners (B, C) — pin each of the eight
        // reflection/rotation transforms exactly; two corners alone cannot
        // distinguish a rotation from its mirror. Hand-derived from the
        // module docs' "Row 0 / Column 0" formulas, independently of the
        // implementation.
        //
        // For orientations 1-4 the destination is 2x3 (no swap): A=(0,0),
        // B=(1,0), C=(0,2). For 5-8 the destination is 3x2 (swapped):
        // A=(0,0), B=(2,0), C=(0,1).
        type Case = (u32, (u32, u32), (u32, u32), (u32, u32));
        let cases: [Case; 8] = [
            (1, (0, 0), (1, 0), (0, 2)),
            (2, (1, 0), (0, 0), (1, 2)),
            (3, (1, 2), (0, 2), (1, 0)),
            (4, (0, 2), (1, 2), (0, 0)),
            (5, (0, 0), (0, 2), (1, 0)),
            (6, (0, 2), (0, 0), (1, 2)),
            (7, (1, 2), (1, 0), (0, 2)),
            (8, (1, 0), (1, 2), (0, 0)),
        ];
        for (orientation, expect_a, expect_b, expect_c) in cases {
            let (dest_b, dest_c) = if (1..=4).contains(&orientation) {
                ((1, 0), (0, 2))
            } else {
                ((2, 0), (0, 1))
            };
            assert_eq!(
                crop_source_coords(orientation, 0, 0, 2, 3),
                expect_a,
                "orientation {orientation}, dest (0,0)"
            );
            assert_eq!(
                crop_source_coords(orientation, dest_b.0, dest_b.1, 2, 3),
                expect_b,
                "orientation {orientation}, dest {dest_b:?}"
            );
            assert_eq!(
                crop_source_coords(orientation, dest_c.0, dest_c.1, 2, 3),
                expect_c,
                "orientation {orientation}, dest {dest_c:?}"
            );
        }
    }

    #[test]
    fn output_coords_from_crop_is_the_inverse_of_crop_source_coords() {
        // SPEC-050 (DEC-043): the colour sink scatters crop→output, so its map
        // must be the exact inverse of the mono gather's output→crop map. Over
        // every crop pixel of several crop shapes (square, wide, tall, 1-wide,
        // 1-tall) and all eight orientations: the round trip is the identity,
        // the output coordinate lands inside the (transposed for 5-8) output
        // rectangle, and no two crop pixels share an output slot.
        for (crop_w, crop_h) in [(4, 4), (5, 3), (2, 7), (1, 6), (6, 1), (1, 1)] {
            for orientation in 1..=8 {
                let (out_w, out_h) = oriented_dimensions(orientation, crop_w, crop_h);
                let mut seen = vec![false; (out_w * out_h) as usize];
                for crop_y in 0..crop_h {
                    for crop_x in 0..crop_w {
                        let (out_x, out_y) =
                            output_coords_from_crop(orientation, crop_x, crop_y, crop_w, crop_h);
                        assert!(
                            out_x < out_w && out_y < out_h,
                            "orientation {orientation}, crop {crop_w}x{crop_h}: \
                             ({crop_x},{crop_y}) -> ({out_x},{out_y}) outside {out_w}x{out_h}"
                        );
                        assert_eq!(
                            crop_source_coords(orientation, out_x, out_y, crop_w, crop_h),
                            (crop_x, crop_y),
                            "orientation {orientation}, crop {crop_w}x{crop_h}: round trip"
                        );
                        let slot = (out_y * out_w + out_x) as usize;
                        assert!(!seen[slot], "orientation {orientation}: slot reused");
                        seen[slot] = true;
                    }
                }
                assert!(
                    seen.iter().all(|&s| s),
                    "orientation {orientation}: slot missed"
                );
            }
        }
    }

    #[test]
    fn hostile_active_area_is_rejected_not_panicked() {
        // Inverted rectangle.
        let mut sensor = base_sensor();
        sensor.active_area = Some(ActiveArea {
            top: 5,
            left: 0,
            bottom: 2,
            right: 4,
        });
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::InvalidActiveArea { .. })
        ));

        // Extends past the raw plane.
        let mut sensor = base_sensor();
        sensor.active_area = Some(ActiveArea {
            top: 0,
            left: 0,
            bottom: 100,
            right: 100,
        });
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::InvalidActiveArea { .. })
        ));
    }

    #[test]
    fn hostile_crop_is_rejected_not_panicked() {
        // DefaultCropSize larger than ActiveArea.
        let mut sensor = base_sensor();
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 100,
            height: 100,
        });
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::InvalidDefaultCrop { .. })
        ));

        // Crop origin outside the plane (fits no crop at all once added).
        let mut sensor = base_sensor();
        sensor.default_crop_origin = Some(DefaultCropOrigin { x: 3, y: 2 });
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 4,
            height: 3,
        });
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::InvalidDefaultCrop { .. })
        ));

        // Zero dimensions.
        let mut sensor = base_sensor();
        sensor.default_crop_size = Some(DefaultCropSize {
            width: 0,
            height: 1,
        });
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::InvalidDefaultCrop { .. })
        ));
    }

    #[test]
    fn hostile_orientation_is_rejected_not_panicked() {
        let mut sensor = base_sensor();
        sensor.orientation = Some(0);
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::UnsupportedOrientation { orientation: 0 })
        ));

        sensor.orientation = Some(9);
        assert!(matches!(
            resolve_geometry(&sensor),
            Err(Error::UnsupportedOrientation { orientation: 9 })
        ));
    }

    #[test]
    fn wrong_length_buffers_are_rejected_not_panicked() {
        let sensor = base_sensor(); // 4x3 = 12 samples
        let src = vec![0u16; 11]; // one short
        let mut dst = [0u16; 12];
        assert!(matches!(
            develop_into(&sensor, &src, &mut dst),
            Err(Error::SourcePlaneWrongLength { .. })
        ));

        let src = vec![0u16; 12];
        let mut dst = [0u16; 11]; // one short
        assert!(matches!(
            develop_into(&sensor, &src, &mut dst),
            Err(Error::DevelopBufferWrongLength { .. })
        ));
    }
}
