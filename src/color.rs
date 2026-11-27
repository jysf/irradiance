//! Colour transform — camera-space linear RGB to display-referred sRGB.
//!
//! STAGE-007's [`crate::demosaic::demosaic_into`] produces **camera-space**
//! linear RGB: the channels are the sensor's native primaries, scaled by the
//! camera's own white, with no display encoding — so a demosaiced image reads
//! greenish and flat. This module (STAGE-008, `SPEC-033`) turns it into a
//! **display-referred sRGB** image, the first recognisable colour picture.
//!
//! # The transform — one 3×3 matrix built once (`DEC-033`)
//!
//! Three linear stages fold into a single 3×3 matrix `M`, built once per image
//! and applied per pixel:
//!
//! 1. **White balance** from `AsShotNeutral` (`asn`): a neutral subject reads
//!    `asn` in camera space, so `D = diag(1/asn_R, 1/asn_G, 1/asn_B)` maps
//!    neutral to `(1, 1, 1)`.
//! 2. **camera → XYZ(D50)**: `ForwardMatrix1` (`FM1`) already maps white-balanced
//!    camera values to XYZ(D50), so `cam2xyz = FM1 · D`. When no `ForwardMatrix`
//!    is present (the Nikon D300S), `ColorMatrix1` (`CM1`) maps XYZ→camera, so
//!    `cam2xyz = CA(W→D50) · inv(CM1)` with scene white `W = inv(CM1)·asn`,
//!    Bradford-adapted to D50 (DNG spec ch. 6). **No `D`** on this path: `inv(CM)`
//!    expects raw camera values, and `D` already counted white balance (the
//!    original `inv(CM1)·D` tinted grey magenta; fixed in `DEVELOP_VERSION` 2).
//! 3. **XYZ(D50) → linear sRGB**: the committed constant `C = S · B`, where
//!    `BRADFORD_D50_TO_D65` (`B`) is the Bradford chromatic adaptation and
//!    `SRGB_FROM_XYZ_D65` (`S`) is IEC 61966-2-1's linear-sRGB-from-XYZ(D65).
//!
//! `M = C · cam2xyz` (`= C · FM1 · D`, or `C · CA · inv(CM1)`).
//!
//! # Scene-adaptive interpolation (`SPEC-034`, `DEC-034`)
//!
//! A DNG is calibrated at **two** illuminants — `ColorMatrix1/2` (and often
//! `ForwardMatrix1/2`) at `CalibrationIlluminant1/2` — and the colour-correct
//! matrix for a shot is the one **interpolated** between them at the scene's own
//! colour temperature. When both calibrations are present (and both illuminant
//! codes are known), this module finds the scene interpolation weight
//! `g ∈ [0, 1]` from `AsShotNeutral` by the DNG reciprocal-temperature
//! `NeutralToXY` method, run **to convergence** (`DEC-076` §5, which replaced
//! `DEC-034`'s fixed 3 passes): from the D50 chromaticity, each pass derives the
//! CCT of the current `xy` by **McCamy's cubic** (a polynomial, so the whole
//! weight stays `+ − × ÷` only), the mired weight `g` between the two
//! `CalibrationIlluminant` CCTs, and the next `xy` from the camera neutral
//! mapped through `inv(X(g))`, `X(g) = (1−g)·X1 + g·X2` (below). It stops when
//! `|Δx| + |Δy| < 1e-7`, or after 30 passes at the midpoint of the last two. The
//! matrices are then lerped at the final `g` (`FM(g)` when both forward matrices
//! are present, else `inv(X(g))`) and `cam2xyz` built from them exactly as the
//! single-illuminant path does. The weight is computed **once per image**, off
//! the per-pixel path. See `scene_weight`.
//!
//! # Camera calibration (`SPEC-085`, `DEC-076`)
//!
//! The DNG camera model is `XYZtoCamera = AB · CC · CM`: `AnalogBalance` (`AB`,
//! a diagonal; absent is the identity) and `CameraCalibration1/2` (`CC_i`) sit
//! in front of each `ColorMatrix`. `AB` always applies. `CC_1/2` apply only
//! when `CameraCalibrationSignature` equals `ProfileCalibrationSignature`, an
//! absent signature counting as empty; otherwise they are the identity.
//!
//! - **ColorMatrix path:** fold per illuminant, `X_i = AB · CC_i · CM_i`, and
//!   lerp the **products** (Adobe's DNG SDK form); `X(g)` takes `CM`'s place.
//! - **ForwardMatrix path:** `K(g) = AB · lerp(CC_1, CC_2, g)`, the reference
//!   neutral `R = inv(K(g)) · AsShotNeutral`, and
//!   `cam2xyz = FM(g) · diag(1/R) · inv(K(g))`. For a diagonal `K` this equals
//!   `FM(g) · D` in real arithmetic, so there the calibration acts only through
//!   `g`.
//! - **Identity:** when `AB · CC_i` is exactly the identity the fold is skipped
//!   and the ForwardMatrix path keeps `D` from the rationals, so a file without
//!   a calibration (or with an identity one) takes the same arithmetic as
//!   before, to the bit.
//!
//! Only what is used is checked (`DEC-076` §3): `CC_1` whenever the rule
//! applies it, `CC_2` only when the dual-illuminant path also runs. See
//! `resolve_calibration`.
//!
//! When the interpolation preconditions are not met — no `ColorMatrix2`, a
//! missing or unknown `CalibrationIlluminant`, `T1 == T2`, or a degenerate step
//! in the iteration — the build falls back to the **single-illuminant** path
//! (`ForwardMatrix1`, else `inv(ColorMatrix1)`), which is **byte-identical** to
//! `SPEC-033`. So a single-calibration sensor is unchanged (the regression
//! guard), and only dual-calibration files shift — toward the scene illuminant.
//! Public signatures are unchanged; this is an internal accuracy refinement.
//!
//! # Arithmetic — committed-constant `f64`, basic ops only (`DEC-033`)
//!
//! The build and per-pixel apply are `f64` using only `+ − × ÷`. These are
//! IEEE-754 correctly-rounded and byte-identical on macOS and Linux (no FMA, no
//! transcendental) — the same basis [`crate::tone::to_linear_f32_into`] already
//! relies on, and [`crate::warp`]'s radial polynomial. This is **not** the
//! exact-rational discipline of `DEC-030` (the colour-**metadata** oracle, which
//! stays exact): the sRGB primaries and the Bradford CAT are irrational, so —
//! exactly as `DEC-025` runs the OETF's transcendental once into a committed
//! table — the irrational colour constant `C` is built from the **committed**
//! `S`/`B` arrays, and determinism comes from committed data + IEEE basic ops.
//!
//! The sRGB **OETF** is not applied here: this module produces **linear**-sRGB,
//! and [`crate::tone::encode_srgb16_into`] (the committed `DEC-025` table) does
//! the display encode. `color` outputs linear; `tone` encodes.
//!
//! # Scale-free apply (`DEC-018`)
//!
//! Inputs and outputs are interleaved `u16` where full-scale `65535` means
//! `1.0` (`DEC-018`), so the `/65535` on input and `×65535` on output cancel and
//! the apply is scale-free: `out_i = Σ_j M[i][j]·cam_j` directly, with `cam_j`
//! the `u16` value. Each result is clamped to `[0, 65535]` **before** the
//! round-half-up quantize, so a saturated highlight or a negative off-gamut mix
//! is pinned to the window and the rounding can never leave `u16` range (the
//! clamp is load-bearing, `DEC-018`).
//!
//! # Caller-owned buffers (`DEC-016`)
//!
//! Like [`crate::plane::unpack_into`], [`crate::develop::develop_into`] and
//! [`crate::demosaic::demosaic_into`], the public functions here write a
//! caller-owned `dst`. [`develop_color_into`] and [`develop_color_f32_into`]
//! allocate one internal `width·height` green plane (`SPEC-049`'s fuse,
//! `DEC-042`): the mosaic is level-normalised once into a small rolling row
//! cache (not a full scratch plane) and the colour matrix is applied straight
//! into `dst`, so neither a normalized mosaic nor a camera-RGB intermediate is
//! allocated. The public `src`/`dst` stay caller-owned.
//!
//! # Float entry point (`SPEC-049`, `DEC-042`)
//!
//! [`develop_color_f32_into`] shares that fused pipeline but writes scene-linear
//! `f32` directly, equal to `to_linear_f32_into(develop_color_into(..))` to the
//! bit (quantise to `u16`, then `/65535.0`), so a float consumer never holds the
//! `u16` result and its `f32` copy at once.
//!
//! # Sensor-geometry plane (`SPEC-091`, `DEC-079`)
//!
//! [`srgb_linear_plane_into`] and [`srgb_linear_plane_f32_into`] run the same
//! fused pipeline but through an identity sink: every pixel of the
//! `width·height` plane is written, masked border included, with no crop,
//! orientation or warp. `DEC-079` defines the mixer's weights on this plane's
//! space (linear sRGB). The `u16` form is the existing colour pixel (clamped);
//! the `f32` form is the matrix's `f64` sum `/ 65535` cast once, **unclamped**,
//! because the clamp corrupts a mix on up to 5% of pixels (`DEC-079` fact 2).
//! The row-range forms [`srgb_linear_plane_rows_into`] and
//! [`srgb_linear_plane_rows_f32_into`] develop a band with a halo
//! (`develop_rows_banded`, `DEC-044`) and equal the whole plane bit for bit.
//!
//! # Mono mix (`SPEC-092`, `DEC-079`, `DEC-083`)
//!
//! [`mono_mix_plane_into`] and its row-range form [`mono_mix_plane_rows_into`]
//! mix a colour Bayer mosaic to a monochrome `u16` plane at the same sensor
//! geometry, through the same fused traversal and identity sink (one sample
//! per pixel instead of three). The caller's weights are linear-sRGB
//! (`DEC-079`) and fold into the camera matrix as `v = wᵀ·M`, quantised once to
//! the fixed-point row [`mono_mix_row`] returns (`i32`, 24 fraction bits). Each
//! pixel is exact `i64` arithmetic on its camera triple: the plain folded mix,
//! except that above [`MONO_MIX_FADE_START`] the negative-weight terms fade out
//! as the brightest camera channel approaches white, so a highlight never
//! darkens as it brightens through clipping. The plane's levels are the
//! constants [`MONO_MIX_BLACK`] and [`MONO_MIX_WHITE`], and [`MONO_MIX_VERSION`]
//! names the mixer's arithmetic.
//!
//! # Panic policy (`no-panics-on-untrusted-input`)
//!
//! A zero-denominator required rational, an absent or ill-shaped
//! `AsShotNeutral`/`ForwardMatrix`/`ColorMatrix`, a singular `ColorMatrix`, an
//! ill-shaped applied `CameraCalibration` or `AnalogBalance`, a singular
//! calibration or a non-positive reference neutral (`DEC-076` §3), and
//! an off-gamut or saturated mix are typed errors ([`Error::MissingColorCalibration`])
//! or clamped values — never a panic and never a NaN written to `dst`. Wrong
//! buffer lengths are [`Error::ColorSourceWrongLength`] / [`Error::RgbBufferWrongLength`]
//! / [`Error::MonoBufferWrongLength`]; the mixer's weights and row are
//! [`Error::InvalidMixWeights`] / [`Error::MixRowOutOfRange`].
//!
//! # Provenance
//!
//! The DNG colour model (`ForwardMatrix`/`ColorMatrix`/`AsShotNeutral`,
//! camera→XYZ) is the Adobe DNG Specification; the sRGB primaries are
//! IEC 61966-2-1; `B` is the Bradford chromatic-adaptation transform
//! (`D50 → D65`). All three are public standards — provenance **class 1**, no
//! implementation read. The calibration (`SPEC-085`) is the specification's
//! Chapter 6 model (class 1); the signature rule's absent-as-empty, the lerp of
//! the folded products and the convergent `NeutralToXY` loop were settled by
//! reading Adobe DNG SDK 1.7.1's `dng_color_spec.cpp` (class 4, no code copied).
//! See `docs/provenance-ledger.md`.

use crate::demosaic::{
    check_supported_bayer, develop_color_fused, develop_rows_banded, develop_rows_banded_in,
    is_supported_bayer, run_row_bands,
};
use crate::develop::{
    black_pattern_dims, normalize_range, oriented_dimensions, output_coords_from_crop,
    resolve_geometry, Geometry,
};
use crate::ifd::{active_area_origin, Rational, Sensor};
use crate::opcode::GainMaps;
use crate::Error;

/// Bind `$n` to the resolved stage-2 normaliser for `$levels`, `$gains` and
/// the bad-pixel fix `$fix` and evaluate `$body` with it (`SPEC-059`,
/// `SPEC-098`). The match on the shapes runs once, outside every loop, and
/// each arm is its own monomorphisation: with no `GainMap` the normaliser is
/// exactly the [`slot_normalizer`] or [`pattern_normalizer`] closure, so a
/// file without one gains nothing in the hot loop.
macro_rules! with_normalizer {
    ($sensor:expr, $src:expr, $levels:expr, $gains:expr, $fix:expr, |$n:ident| $body:expr) => {
        match ($levels, $gains.is_empty()) {
            (ColourLevels::Slots(levels), true) => {
                let $n = slot_normalizer($sensor, $src, levels, $fix);
                $body
            }
            (ColourLevels::Pattern(pattern, range), true) => {
                let $n = pattern_normalizer($sensor, $src, pattern, range, $fix);
                $body
            }
            (ColourLevels::Slots(levels), false) => {
                let inner = slot_normalizer($sensor, $src, levels, $fix);
                let $n = gained_normalizer($sensor, inner, &$gains);
                $body
            }
            (ColourLevels::Pattern(pattern, range), false) => {
                let inner = pattern_normalizer($sensor, $src, pattern, range, $fix);
                let $n = gained_normalizer($sensor, inner, &$gains);
                $body
            }
        }
    };
}

/// Linear sRGB ← CIE XYZ (D65), the IEC 61966-2-1 primaries (sRGB colour space
/// definition). The inverse of the published XYZ-from-sRGB matrix; its rows map
/// the D65 white point to `(1, 1, 1)`. A committed `f64` constant (`DEC-033`):
/// the sRGB primaries are irrational, so this is transcribed from the standard
/// once and reproduced exactly, as `DEC-025` does for the OETF table.
const SRGB_FROM_XYZ_D65: [[f64; 3]; 3] = [
    [3.2404542, -1.5371385, -0.4985314],
    [-0.9692660, 1.8760108, 0.0415560],
    [0.0556434, -0.2040259, 1.0572252],
];

/// Bradford chromatic adaptation, CIE XYZ (D50) → CIE XYZ (D65). Maps the D50
/// white point `[0.96422, 1.0, 0.82521]` to the D65 white point
/// `[0.95047, 1.0, 1.08883]`. A committed `f64` constant (`DEC-033`): the
/// Bradford cone-response matrix is irrational, transcribed from the standard
/// once. Folded with [`SRGB_FROM_XYZ_D65`] into the one XYZ(D50)→linear-sRGB
/// constant `C = S · B` built in [`build_camera_to_srgb_linear`].
const BRADFORD_D50_TO_D65: [[f64; 3]; 3] = [
    [0.9555766, -0.0230393, 0.0631636],
    [-0.0282895, 1.0099416, 0.0210077],
    [0.0122982, -0.0204830, 1.3299098],
];

/// Correlated colour temperature (kelvin) of each EXIF `LightSource` /
/// DNG `CalibrationIlluminant` code `SPEC-034` recognises (`DEC-034`). A
/// `CalibrationIlluminant` code not in this table is treated as **unknown** and
/// takes the single-illuminant fallback, rather than guessing a temperature.
/// Only the standard illuminants with a crisp, published CCT are listed; the
/// fuzzy daylight/fluorescent codes are deliberately omitted (their nominal CCTs
/// are ranges, not points).
///
/// Sources: EXIF 2.3 `LightSource` tag code assignments (also Adobe DNG
/// Specification §Chapter 5, `CalibrationIlluminant`); the CCTs are the CIE
/// standard-illuminant values — Illuminant A = 2856 K, B = 4874 K, C = 6774 K,
/// and the CIE D-series `Dxx` names the correlated temperature directly
/// (D50 = 5003 K, D55 = 5503 K, D65 = 6504 K, D75 = 7504 K, each the
/// International-Temperature-Scale-corrected value `≈ xx00 × 1.4388/1.4380`).
/// Code 1 (Daylight) takes the nominal 5500 K; codes 3 (Tungsten) and 17
/// (Standard light A) are incandescent ≈ Illuminant A = 2856 K; code 24
/// (ISO studio tungsten) is the ISO 3200 K studio standard.
const ILLUMINANT_CCT: &[(u32, f64)] = &[
    (1, 5500.0),  // Daylight (EXIF nominal)
    (3, 2856.0),  // Tungsten (incandescent) ≈ CIE Illuminant A
    (17, 2856.0), // Standard light A (CIE Illuminant A)
    (18, 4874.0), // Standard light B (CIE Illuminant B)
    (19, 6774.0), // Standard light C (CIE Illuminant C)
    (20, 5503.0), // D55 (CIE D55)
    (21, 6504.0), // D65 (CIE D65)
    (22, 7504.0), // D75 (CIE D75)
    (23, 5003.0), // D50 (CIE D50)
    (24, 3200.0), // ISO studio tungsten
];

/// McCamy's 1992 cubic CCT-from-chromaticity: `n = (x − x0) / (y0 − y)`;
/// `T = c3·n³ + c2·n² + c1·n + c0`. A **polynomial** (`n³ = n·n·n`, no `powf`),
/// so [`mccamy_cct`] stays `+ − × ÷` only and cross-platform (`DEC-033`/`DEC-034`).
/// The epicentre `(0.3320, 0.1858)` and the four coefficients are the published
/// constants of McCamy, *Color Research & Application* 17(2), 1992 (a class-2
/// paper; see `docs/provenance-ledger.md`).
const MCCAMY_EPICENTRE_X: f64 = 0.3320;
const MCCAMY_EPICENTRE_Y: f64 = 0.1858;
const MCCAMY_C3: f64 = 449.0;
const MCCAMY_C2: f64 = 3525.0;
const MCCAMY_C1: f64 = 6823.3;
const MCCAMY_C0: f64 = 5520.33;

/// Transform interleaved **camera-space linear RGB** (`width·height·3` `u16`
/// samples) into interleaved **linear-sRGB** `u16`, by white balance + colour
/// matrix (`SPEC-033`). The 3×3 matrix `M` is built once from `sensor`'s colour
/// metadata (`build_camera_to_srgb_linear`) and applied per pixel; the sRGB
/// OETF is **not** applied (this is linear-sRGB, which [`crate::tone`] encodes).
///
/// Both buffers are caller-owned and this function allocates nothing
/// (`DEC-016`). The apply is scale-free (`65535 ≡ 1.0`, `DEC-018`): each output
/// channel is `Σ_j M[i][j]·cam_j`, clamped to `[0, 65535]` before the
/// round-half-up quantize.
///
/// # Errors
///
/// - Whatever [`crate::opcode::check_required_processing`] returns for
///   [`crate::opcode::DevelopPath::ColourTransform`], checked first: a
///   malformed colour calibration tag ([`Error::MalformedTag`], `DEC-078`), or
///   a mandatory opcode in any list other than an `OpcodeList2` `GainMap`
///   (`SPEC-059`; this
///   primitive takes camera RGB already past stage 2, so a composer applies
///   the maps first with [`crate::develop::apply_gain_maps`])
///   ([`Error::UnsupportedRequiredOpcode`] /
///   [`Error::UnsupportedMandatoryOpcode`]) or a present-but-unreadable list
///   ([`Error::UnreadableOpcodeList`]) refuses (`SPEC-056`, `DEC-053`).
/// - [`Error::ColorSourceWrongLength`] if `camera_rgb.len() != width·height·3`.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != width·height·3`.
/// - [`Error::MissingColorCalibration`] if `sensor` carries no usable
///   `ForwardMatrix1`/`ColorMatrix1`, no `AsShotNeutral`, a zero-denominator
///   required rational, a non-invertible `ColorMatrix`, or an applied
///   `CameraCalibration` / `AnalogBalance` that is ill-shaped, singular, or
///   gives a non-positive reference neutral (`DEC-076` §3).
///
/// Never panics: lengths are checked before any write, every matrix entry is a
/// guarded division, and the quantize clamps before rounding.
pub fn to_srgb_linear_into(
    sensor: &Sensor,
    camera_rgb: &[u16],
    dst: &mut [u16],
) -> Result<(), Error> {
    // Camera-referred pixels on the colour path, which applies only
    // `OpcodeList2`'s `GainMap`s (`SPEC-059`): refuse first, as every colour
    // develop does (`SPEC-056`, `DEC-053`). This primitive reads only the
    // colour tags, so it asks as the transform (`DEC-078` §2).
    crate::opcode::check_required_processing(sensor, crate::opcode::DevelopPath::ColourTransform)?;
    let rgb_len = interleaved_rgb_len(sensor)?;
    let expected = usize::try_from(rgb_len).unwrap_or(usize::MAX);
    if camera_rgb.len() != expected {
        return Err(Error::ColorSourceWrongLength {
            expected: rgb_len,
            actual: camera_rgb.len(),
        });
    }
    if dst.len() != expected {
        return Err(Error::RgbBufferWrongLength {
            expected: rgb_len,
            actual: dst.len(),
        });
    }

    let matrix = build_camera_to_srgb_linear(sensor)?;
    apply_matrix_into(&matrix, camera_rgb, dst);
    Ok(())
}

/// Develop a raw **Bayer mosaic** (`width·height` `u16` samples) all the way to
/// interleaved **linear-sRGB** `u16`, cropped and oriented to the same geometry
/// as [`crate::develop::develop_into`] (`out_w·out_h·3`, where `(out_w, out_h)`
/// is [`crate::develop::output_dimensions`]`(sensor)`): per-channel
/// level-normalize → demosaic → colour matrix, fused into two memory-lean passes
/// (`SPEC-033`; the `SPEC-049` fuse, `DEC-042`; the `SPEC-050` crop, `DEC-043`).
///
/// **Geometry (`SPEC-050`, `DEC-043`).** The output is `ActiveArea` →
/// `DefaultCrop` (relative to the ActiveArea, `DEC-019`) → `Orientation`, the
/// masked border outside the `ActiveArea` and the margin outside the
/// `DefaultCrop` dropped, dimensions swapped for a 90° orientation (5-8) —
/// exactly the mono path's frame, so size `dst` from the public
/// [`crate::develop::output_dimensions`] as `develop_into`'s callers do. The
/// crop happens **at the sink**: the fused kernel still develops every plane
/// pixel with the same neighbours (so a kept pixel's colour is byte-identical to
/// the whole-plane develop at its source — the crop moves the frame, not the
/// colours, and the `SPEC-047` ActiveArea-origin phase still holds), and each
/// pixel is scattered straight into its output slot or dropped. No intermediate
/// RGB buffer exists. (Before `SPEC-050` the output was the whole
/// `width·height·3` plane, border included — a contract change, `DEVELOP_VERSION`
/// 4.) **The one opcode applied on this path is `OpcodeList2`'s `GainMap`**
/// (`SPEC-059`, `DEC-069`): every one, optional or mandatory, at stage 2 inside
/// the normaliser, in `ActiveArea` coordinates. No `WarpRectilinear` or
/// `FixBadPixelsConstant` is applied — so, per `SPEC-054` / `DEC-049`, a file
/// whose opcode lists carry any other **mandatory** opcode is refused with a
/// typed error before anything is allocated; optional opcodes are skipped, as
/// the DNG spec permits.
///
/// The per-channel level-normalize happens on the **mosaic, before demosaic**:
/// black-subtract is per CFA colour and does not commute with interpolation, so
/// each photosite is normalized by the `black_levels`/`white_levels` entry for
/// its 2×2 position (scalar `black_level`/`white_level` fallback, then
/// `0`/`2^bits-1`), clamped per `DEC-018`. Rather than materialise a normalized
/// mosaic and a `camera_rgb` intermediate, the fused path (`DEC-042`) sweeps one
/// output row at a time: it level-normalises each sample **once** into a small
/// rolling cache of recent rows, writes the finalised green into a single
/// `width·height` green plane, then interpolates red/blue, assembles the
/// camera-RGB triple in registers, and applies the colour matrix
/// (`build_camera_to_srgb_linear`) straight into `dst`. The `u16` output is
/// **byte-identical** to the pre-fuse `normalize → demosaic_into → matrix`
/// composition (the colour goldens and `tests/color_oracle.rs`'s byte-equivalence
/// test are the gate).
///
/// The public `src`/`dst` are caller-owned (`DEC-016`); the only `width·height`
/// scratch is the green plane (`~47 MB` at 23 MP, down from the pre-fuse
/// `~187 MB`); the rolling normalised-row cache is a few rows, negligible.
///
/// `OpcodeList1`'s first `FixBadPixelsConstant`, whatever its flags, is applied
/// on the raw mosaic as each sample is read, before the levels (`SPEC-098`,
/// `DEC-086`): a photosite equal to its `Constant` inside the `ActiveArea` takes
/// the mean, rounded half up, of its four nearest photosites of the same
/// `CFAPattern` colour (green from its diagonals, red and blue from two away on
/// the axes) whose own raw value is not `Constant`, or keeps its value when
/// none is. `BayerPhase` is not used. It allocates nothing.
///
/// # Errors
///
/// - [`Error::UnsupportedRequiredOpcode`] (checked first, `SPEC-054`) if any
///   opcode list carries a mandatory recognized opcode other than
///   `OpcodeList1`'s first `FixBadPixelsConstant` and the `OpcodeList2`
///   `GainMap`s — the opcodes this path applies (`SPEC-059`, `SPEC-098`), so a
///   mandatory `WarpRectilinear` is `{ list: 3, id: 1 }` until `SPEC-099` —
///   or [`Error::UnsupportedMandatoryOpcode`] for a mandatory
///   unrecognized one; [`Error::Truncated`] for a broken opcode list.
/// - [`Error::ColorSourceWrongLength`] if `src.len() != width·height`.
/// - [`Error::InvalidActiveArea`] / [`Error::InvalidDefaultCrop`] /
///   [`Error::UnsupportedOrientation`] for a geometry
///   [`crate::develop::develop_into`] would also reject.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != out_w·out_h·3`.
/// - [`Error::InvalidBlackLevelPattern`] if `BlackLevel` has a count other than
///   `BlackLevelRepeatDim`'s rows × columns (`DEC-068` §2), before any write.
/// - [`Error::InvalidLevels`] if the colour white `W` (the first `WhiteLevel`
///   value) is at or below `B`, the maximum black the pattern applies
///   (`DEC-070` §3), reporting `B` and `W`, before any write.
/// - [`Error::MalformedOpcodeParams`] `{ id: 9, .. }` or [`Error::TooManyGainMaps`]
///   for `OpcodeList2`'s `GainMap`s (`SPEC-059`, `DEC-069` §3), before any write.
/// - [`Error::MalformedOpcodeParams`] `{ id: 4, .. }` for a malformed first
///   `FixBadPixelsConstant` in `OpcodeList1`, optional or not (`DEC-086` item
///   7), before any write.
/// - [`Error::NotAColorFilterArray`] / [`Error::UnsupportedCfaPattern`] /
///   [`Error::UnsupportedCfaLayout`] / [`Error::UnsupportedCfaPlaneColor`] for a
///   non-CFA plane or an unsupported mosaic, or [`Error::MissingColorCalibration`]
///   if the colour matrix cannot be built, an unusable applied
///   `CameraCalibration` / `AnalogBalance` included (`DEC-076` §3) — in that
///   order (CFA before matrix).
///
/// Never panics.
pub fn develop_color_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error> {
    check_colour_required_processing(sensor)?;
    let (expected_src, output) = check_color_buffers(sensor, src.len(), dst.len())?;
    // The black pattern and the `GainMap`s resolve before any allocation or
    // write (`DEC-068` §2, `DEC-069` §3), and the match on their shapes runs
    // once, here, not per pixel.
    let levels = resolve_colour_levels(sensor)?;
    let gains = resolve_colour_gain_maps(sensor)?;
    let fix = resolve_colour_bad_pixels(sensor)?;
    with_normalizer!(sensor, src, levels, gains, fix, |normalize| {
        develop_color_with(
            sensor,
            expected_src,
            output,
            &normalize,
            dst,
            || build_camera_to_srgb_linear(sensor),
            store_u16,
        )
    })
}

/// Develop a raw **Bayer mosaic** (`width·height` `u16` samples) to interleaved
/// **scene-linear sRGB** `f32` at [`develop_color_into`]'s cropped, oriented
/// geometry (`out_w·out_h·3`, [`crate::develop::output_dimensions`]; `SPEC-050`),
/// sharing [`develop_color_into`]'s fused pipeline but writing `f32` directly so a float consumer (the Python
/// bindings, ML dataloaders) never holds the `u16` result and its `f32` copy at
/// once (`SPEC-049`, `DEC-042`).
///
/// The value written is **exactly** `to_linear_f32_into(develop_color_into(..))`:
/// each channel is quantised to `u16` first — the identical clamp-and-round
/// `quantize` (`DEC-018`) the integer path uses — and then divided by
/// `65535.0`, matching [`crate::tone::to_linear_f32_into`] to the bit. The raw
/// `f64` matrix result is **not** divided directly; that would skip the
/// clamp-and-round and differ. As in [`crate::tone::to_linear_f32_into`] this is
/// scene-linear, not display-encoded (the sRGB OETF is [`crate::tone`]'s).
///
/// The crop + orient scatter is a pure rearrangement with no arithmetic, so it
/// commutes with the quantise-then-divide: each kept pixel is quantised, then
/// divided, then written to its output slot.
///
/// The only internal scratch is the `width·height` green plane; `src`/`dst` are
/// caller-owned (`DEC-016`).
///
/// # Errors
///
/// - [`Error::UnsupportedRequiredOpcode`] / [`Error::UnsupportedMandatoryOpcode`]
///   / [`Error::Truncated`] first, exactly as [`develop_color_into`] (`SPEC-054`).
/// - [`Error::ColorSourceWrongLength`] if `src.len() != width·height`.
/// - [`Error::InvalidActiveArea`] / [`Error::InvalidDefaultCrop`] /
///   [`Error::UnsupportedOrientation`] for a geometry
///   [`crate::develop::develop_into`] would also reject.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != out_w·out_h·3`.
/// - [`Error::InvalidBlackLevelPattern`] if `BlackLevel` has a count other than
///   `BlackLevelRepeatDim`'s rows × columns (`DEC-068` §2), before any write.
/// - [`Error::InvalidLevels`] if the colour white `W` (the first `WhiteLevel`
///   value) is at or below `B`, the maximum black the pattern applies
///   (`DEC-070` §3), reporting `B` and `W`, before any write.
/// - [`Error::MalformedOpcodeParams`] `{ id: 9, .. }` or [`Error::TooManyGainMaps`]
///   for `OpcodeList2`'s `GainMap`s (`SPEC-059`, `DEC-069` §3), before any write.
/// - [`Error::MalformedOpcodeParams`] `{ id: 4, .. }` for a malformed first
///   `FixBadPixelsConstant` in `OpcodeList1`, optional or not (`DEC-086` item
///   7), before any write.
/// - [`Error::NotAColorFilterArray`] / [`Error::UnsupportedCfaPattern`] /
///   [`Error::UnsupportedCfaLayout`] / [`Error::UnsupportedCfaPlaneColor`] /
///   [`Error::MissingColorCalibration`] (an unusable applied `CameraCalibration`
///   / `AnalogBalance` included, `DEC-076` §3), in that order (CFA before matrix).
///
/// Never panics.
pub fn develop_color_f32_into(sensor: &Sensor, src: &[u16], dst: &mut [f32]) -> Result<(), Error> {
    check_colour_required_processing(sensor)?;
    let (expected_src, output) = check_color_buffers(sensor, src.len(), dst.len())?;
    // As in `develop_color_into`: resolve the black pattern and the
    // `GainMap`s once, before any write (`DEC-068` §2, `DEC-069` §3).
    let levels = resolve_colour_levels(sensor)?;
    let gains = resolve_colour_gain_maps(sensor)?;
    let fix = resolve_colour_bad_pixels(sensor)?;
    with_normalizer!(sensor, src, levels, gains, fix, |normalize| {
        develop_color_with(
            sensor,
            expected_src,
            output,
            &normalize,
            dst,
            || build_camera_to_srgb_linear(sensor),
            store_f32_quantised,
        )
    })
}

/// The colour develop's body for one resolved normalizer, shared by every
/// single-thread colour entry (`SPEC-049`'s fuse, `DEC-042`): one `width·height`
/// green plane, the one fused traversal, and a sink that maps each developed
/// plane pixel to its `dst` sample group of `C` samples (`slots`: the `SPEC-050`
/// crop/orient scatter, or the `SPEC-091` identity) and writes it with `store`
/// (each entry's own arithmetic on the camera triple; `C` is 3 for an RGB
/// entry and 1 for the `SPEC-092` mono mix). `build` makes what `store` reads
/// (the colour matrix, or the mixer's fixed-point row) inside the traversal's
/// `setup`, after its CFA check, so the error order is the fused path's and
/// nothing is written unless it succeeds.
fn develop_color_with<T, P, K, B, S, N, const C: usize>(
    sensor: &Sensor,
    expected_src: usize,
    slots: P,
    normalize: &N,
    dst: &mut [T],
    build: B,
    store: S,
) -> Result<(), Error>
where
    P: PixelSlot,
    B: FnOnce() -> Result<K, Error>,
    S: Fn(&mut [T; C], &K, [u16; 3]),
    N: Fn(u32, u32) -> u16,
{
    // The one remaining scratch buffer: the `width·height` green plane.
    let mut green = vec![0u16; expected_src];
    let (dst_groups, _remainder) = dst.as_chunks_mut::<C>();

    develop_color_fused(sensor, &mut green, normalize, || {
        let coefficients = build()?;
        let mut slots = slots;
        Ok(move |pixel: usize, r: u16, g: u16, b: u16| {
            // Crop + orient at the sink (`DEC-043`): a pixel outside the
            // DefaultCrop rectangle (the masked border included) is dropped
            // before the matrix is even applied. The identity keeps every pixel.
            let Some(slot) = slots.slot_of(pixel) else {
                return;
            };
            if let Some(out) = dst_groups.get_mut(slot) {
                store(out, &coefficients, [r, g, b]);
            }
        })
    })
}

/// The `u16` colour pixel (`develop_color_into`, `DEC-018`): each channel is
/// `Σ_j M[i][j]·cam_j` in `f64`, in this order, clamped and rounded half up by
/// [`quantize`]. Every `u16` colour entry writes exactly this.
fn store_u16(out: &mut [u16; 3], m: &[[f64; 3]; 3], rgb: [u16; 3]) {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let [r, g, b] = rgb;
    let rf = f64::from(r);
    let gf = f64::from(g);
    let bf = f64::from(b);
    *out = [
        quantize(m00 * rf + m01 * gf + m02 * bf),
        quantize(m10 * rf + m11 * gf + m12 * bf),
        quantize(m20 * rf + m21 * gf + m22 * bf),
    ];
}

/// The cropped `f32` colour pixel (`develop_color_f32_into`): the [`store_u16`]
/// value, quantised first, then divided by `65535.0` — the bit-exact definition
/// of `to_linear_f32_into(quantize(..))`.
fn store_f32_quantised(out: &mut [f32; 3], m: &[[f64; 3]; 3], rgb: [u16; 3]) {
    let mut quantised = [0u16; 3];
    store_u16(&mut quantised, m, rgb);
    *out = quantised.map(|v| f32::from(v) / 65535.0);
}

/// The sensor-geometry `f32` pixel (`SPEC-091`, `DEC-079`): the same `f64` sum
/// as [`store_u16`], in the same order, divided by `65535.0` and cast to `f32`
/// once, **unclamped and unquantised**. `1.0` is white; a negative channel is an
/// out-of-gamut colour and one above `1.0` a white-balance over, both of which
/// [`quantize`] would pin. Basic `f64` operations only (no `mul_add`), and the
/// one cast is IEEE round-to-nearest-even, so the value is the same on every
/// conforming platform (`DEC-033`; measured by `tests/determinism.rs`).
fn store_f32_unclamped(out: &mut [f32; 3], m: &[[f64; 3]; 3], rgb: [u16; 3]) {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let [r, g, b] = rgb;
    let rf = f64::from(r);
    let gf = f64::from(g);
    let bf = f64::from(b);
    *out = [
        ((m00 * rf + m01 * gf + m02 * bf) / 65535.0) as f32,
        ((m10 * rf + m11 * gf + m12 * bf) / 65535.0) as f32,
        ((m20 * rf + m21 * gf + m22 * bf) / 65535.0) as f32,
    ];
}

/// Where the fused traversal's plane pixel `pixel = y·width + x` lands in a
/// colour entry's `dst` (as a sample-group index: an RGB triple, or one mono
/// sample), or `None` to drop it.
trait PixelSlot {
    fn slot_of(&mut self, pixel: usize) -> Option<usize>;
}

/// The sensor-geometry sink (`SPEC-091`): plane pixel `p` is sample group `p`
/// (an RGB triple, or a `SPEC-092` mono sample), so every pixel of the
/// `width·height` plane is written, masked border included.
struct WholePlane;

impl PixelSlot for WholePlane {
    fn slot_of(&mut self, pixel: usize) -> Option<usize> {
        Some(pixel)
    }
}

/// The white-balanced **linear-sRGB plane at sensor geometry** (`SPEC-091`,
/// `DEC-079`), as `u16`: interleaved RGB at `width·height·3`, pixel `(x, y)` at
/// index `3·(y·width + x)`, **every** pixel of the plane written, the masked
/// border outside the `ActiveArea` included. No crop, no orientation, no warp.
///
/// **The plane is not warped; the consumer applies `OpcodeList3`'s
/// `WarpRectilinear`.** These entries ask with
/// [`crate::opcode::DevelopPath::SensorGeometry`] (`SPEC-098`, `DEC-086` item
/// 5): the first `WarpRectilinear` in `OpcodeList3`, mandatory or not and
/// whatever its `N`, is accepted and never applied, so a viewer that warps on
/// its GPU reads it from [`crate::ifd::Sensor::opcode_list_3`] (for example
/// with [`crate::opcode::parse_warp_rectilinear`]) and applies it itself.
///
/// The pipeline is [`develop_color_into`]'s, the same fused traversal:
/// `OpcodeList1`'s first `FixBadPixelsConstant` on the raw mosaic (`DEC-086`),
/// levels, `OpcodeList2` `GainMap`s (`DEC-069`), Hamilton-Adams demosaic and the
/// camera-to-linear-sRGB matrix `M` (`DEC-033`). Only the sink differs: the
/// identity instead of the crop. So each pixel is exactly the value
/// [`develop_color_into`] writes for it when it lies in the crop rectangle, and
/// the value the traversal computes (and drops) there when it does not. Like
/// that pixel it is clamped to `[0, 65535]` and rounded half up (`DEC-018`): an
/// out-of-gamut colour pins to `0` and a white-balance over to `65535`. For the
/// unclamped values see [`srgb_linear_plane_f32_into`].
///
/// `src` is the unpacked raw mosaic (`width·height`); both buffers are
/// caller-owned (`DEC-016`), and the one internal scratch is the `width·height`
/// green plane. For a plane too large to hold at once, develop it in row strips
/// with [`srgb_linear_plane_rows_into`].
///
/// # Errors
///
/// [`develop_color_into`]'s, in its order, except that `dst` is sized to the
/// plane and required processing asks with
/// [`crate::opcode::DevelopPath::SensorGeometry`] (so the first
/// `WarpRectilinear` is accepted): required processing and malformed tags,
/// [`Error::ColorSourceWrongLength`], the geometry
/// ([`Error::InvalidActiveArea`] / [`Error::InvalidDefaultCrop`] /
/// [`Error::UnsupportedOrientation`]: the plane exists exactly when the
/// developed image does), [`Error::RgbBufferWrongLength`] if
/// `dst.len() != width·height·3`, then the levels, the `GainMap`s,
/// `OpcodeList1`'s `FixBadPixelsConstant` ([`Error::MalformedOpcodeParams`]
/// `{ id: 4, .. }`), the CFA and the colour calibration. Nothing is written
/// unless every check passes.
///
/// Never panics.
pub fn srgb_linear_plane_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error> {
    srgb_linear_plane(
        sensor,
        src,
        dst,
        PlaneSamples::Rgb,
        matrix_of(sensor),
        store_u16,
    )
}

/// The white-balanced linear-sRGB plane at sensor geometry (`SPEC-091`,
/// `DEC-079`) as **unclamped** `f32`: the layout, pipeline and refusals of
/// [`srgb_linear_plane_into`], but each channel is the matrix's `f64` sum divided
/// by `65535.0` and cast once, `((m_i0·r + m_i1·g + m_i2·b) / 65535.0) as f32`,
/// with no clamp and no quantise. `1.0` is white; a negative value is an
/// out-of-gamut colour and a value above `1.0` a white-balance over (`DEC-079`
/// fact 2), so a mix computed from this plane is not corrupted by the `u16`
/// plane's clamp.
///
/// This is **not** [`develop_color_f32_into`]'s rule, which quantises to `u16`
/// first and so clamps. Where the `u16` plane does not clamp the two agree to
/// within half a code. The arithmetic is `f64` basic operations with committed
/// constants and one IEEE cast (`DEC-033`), and `tests/determinism.rs` pins the
/// plane's bits on x86_64 and aarch64.
///
/// The plane is not warped; the consumer applies `OpcodeList3`'s
/// `WarpRectilinear`, as for [`srgb_linear_plane_into`] (`SPEC-098`).
///
/// # Errors
///
/// Exactly [`srgb_linear_plane_into`]'s, in its order.
///
/// Never panics.
pub fn srgb_linear_plane_f32_into(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [f32],
) -> Result<(), Error> {
    srgb_linear_plane(
        sensor,
        src,
        dst,
        PlaneSamples::Rgb,
        matrix_of(sensor),
        store_f32_unclamped,
    )
}

/// [`srgb_linear_plane_into`] for rows `rows` of the plane only (`SPEC-091`): a
/// band, so a caller can develop a plane too large to hold at once in strips.
/// `rows` is in whole-plane rows (`0..height`, masked rows included); `dst` is
/// `(rows.end − rows.start)·width·3`, plane row `y` at index
/// `3·((y − rows.start)·width + x)`.
///
/// Every value equals the same pixel of [`srgb_linear_plane_into`] **bit for
/// bit**, so strips of any heights that tile `0..height` give exactly the whole
/// plane. The band develops its rows plus a three-row halo each side through
/// the same fused traversal (`SPEC-051`'s `develop_rows_banded`, `DEC-044`), so
/// its internal scratch is proportional to the band, not the plane; `src` is
/// still the whole mosaic.
///
/// The plane is not warped; the consumer applies `OpcodeList3`'s
/// `WarpRectilinear`, as for [`srgb_linear_plane_into`] (`SPEC-098`).
///
/// # Errors
///
/// [`srgb_linear_plane_into`]'s, in its order, with [`Error::InvalidRowRange`]
/// (an empty or reversed range, or `rows.end > height`) after the geometry and
/// before [`Error::RgbBufferWrongLength`], whose `expected` is the band's
/// `(rows.end − rows.start)·width·3`. Nothing is written unless every check
/// passes.
///
/// Never panics.
pub fn srgb_linear_plane_rows_into(
    sensor: &Sensor,
    src: &[u16],
    rows: core::ops::Range<u32>,
    dst: &mut [u16],
) -> Result<(), Error> {
    srgb_linear_plane_rows(
        sensor,
        src,
        rows,
        dst,
        PlaneSamples::Rgb,
        matrix_of(sensor),
        store_u16,
    )
}

/// [`srgb_linear_plane_f32_into`] for rows `rows` of the plane only
/// (`SPEC-091`): the band layout, halo and refusals of
/// [`srgb_linear_plane_rows_into`], every value bit-identical to the same pixel
/// of [`srgb_linear_plane_f32_into`] (unclamped).
///
/// The plane is not warped; the consumer applies `OpcodeList3`'s
/// `WarpRectilinear`, as for [`srgb_linear_plane_into`] (`SPEC-098`).
///
/// # Errors
///
/// Exactly [`srgb_linear_plane_rows_into`]'s, in its order.
///
/// Never panics.
pub fn srgb_linear_plane_rows_f32_into(
    sensor: &Sensor,
    src: &[u16],
    rows: core::ops::Range<u32>,
    dst: &mut [f32],
) -> Result<(), Error> {
    srgb_linear_plane_rows(
        sensor,
        src,
        rows,
        dst,
        PlaneSamples::Rgb,
        matrix_of(sensor),
        store_f32_unclamped,
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// The mono mixer (`SPEC-092`, `DEC-079`, `DEC-083`)
// ─────────────────────────────────────────────────────────────────────────────

/// The mono mixer's fixed-point row has 24 fraction bits (`DEC-083` §3): each
/// entry is `q_i = ⌊v_i·2^24 + 0.5⌋` as `i32`, `v = wᵀ·M` the folded row
/// (`DEC-079`). [`mono_mix_row`] returns it; changing this changes every row,
/// and so bumps [`MONO_MIX_VERSION`].
pub const MONO_MIX_ROW_FRACTION_BITS: u32 = 24;

/// The last camera level at which the mono mixer's fade of negative weights
/// does nothing (`DEC-083` §4, `DEC-079`): `T = 57343 = 65535 − 2^13`, so
/// `τ = T/65535 ≈ 0.875`.
///
/// - **The compared value** is, at each pixel, `m`, the largest of its three
///   camera channels as the demosaic emits them: after the black level is
///   subtracted and the sample scaled by `WhiteLevel − BlackLevel` to
///   `[0, 65535]` (`DEC-070`), after `OpcodeList2` `GainMap`s (`DEC-069`), and
///   before white balance and the matrix. All three channels count,
///   interpolated ones included.
/// - **Against what:** the fade starts above `MONO_MIX_FADE_START = 57343` and
///   is complete at `65535`, white after levels; in between, the
///   negative-weight terms are scaled by `(65535 − m)/8192`
///   ([`MONO_MIX_FADE_WIDTH_BITS`]). At `m ≤ 57343` the pixel is the plain
///   fold; at `m = 65535` only the positive-weight terms remain.
/// - **How it relates to the raw photosite:** a pixel's own CFA channel is its
///   photosite's normalised value exactly (Hamilton-Adams keeps the native
///   sample); the other two are interpolated from its neighbours and clamped
///   to `[0, 65535]`. So wherever the photosite is at or above `WhiteLevel`,
///   its own channel is `65535` and the fade is complete there, unless an
///   `OpcodeList2` `GainMap` gain below 1 at that photosite pulls it back under
///   white. The fade can also be complete where the raw photosite is not
///   clipped: where a neighbour-interpolated channel reaches `65535`, or a gain
///   above 1 lifts a photosite to white. A clip overlay read from the raw CFA
///   plane is therefore a subset of the pixels where the fade is complete, and
///   equals it on files without `GainMap`s apart from interpolated neighbours.
///
/// A fixed part of the mixer, not a parameter; changing it bumps
/// [`MONO_MIX_VERSION`].
pub const MONO_MIX_FADE_START: u16 = 57343;

/// The fade of negative weights spans `2^13 = 8192` camera codes, from above
/// [`MONO_MIX_FADE_START`] to white (`DEC-083` §4). A power of two, so the
/// per-pixel step needs no division: the scale of the negative terms is
/// `u/8192` with `u = min(65535 − m, 8192)`.
pub const MONO_MIX_FADE_WIDTH_BITS: u32 = 13;

/// The mixed mono plane's black level: `0`, for every file. The plane is
/// already level-normalised (`DEC-018` full scale), so a consumer's single
/// normalise step is `v / 65535`. A constant rather than a return value
/// because it cannot vary; it is part of the documented contract, so changing
/// it is an API change and also bumps [`MONO_MIX_VERSION`].
///
/// ```
/// use irradiance::color::{MONO_MIX_BLACK, MONO_MIX_WHITE};
/// assert_eq!((MONO_MIX_BLACK, MONO_MIX_WHITE), (0, 65535));
/// ```
pub const MONO_MIX_BLACK: u16 = 0;

/// The mixed mono plane's white level: `65535`, for every file (`DEC-018`
/// full scale). See [`MONO_MIX_BLACK`]: a constant, part of the documented
/// contract; changing it is an API change and bumps [`MONO_MIX_VERSION`].
pub const MONO_MIX_WHITE: u16 = 65535;

/// The mono **mixer epoch** (`SPEC-092` Design §12, `DEC-083` §6): bumped by
/// any change to the mixer's own arithmetic that can change a row or a mixed
/// pixel for the same `Sensor`, `src` and weights — the weight rule, the fold
/// or its order, the scale or rounding of the row, the fade threshold or
/// width, the kernel's rounding or clamp, or the plane's levels
/// ([`MONO_MIX_BLACK`], [`MONO_MIX_WHITE`]). Like [`crate::DEVELOP_VERSION`] it
/// tracks output identity, independently of the crate's semver.
///
/// It is **not** bumped for a change upstream of the mixer (levels,
/// `GainMap`s, demosaic, the camera matrix): those bump
/// [`crate::DEVELOP_VERSION`]. A mix is fixed by the pair
/// (`DEVELOP_VERSION`, `MONO_MIX_VERSION`), and a caller that stores mixed
/// output or a [`mono_mix_row`] (onyx's recipe) records both.
///
/// The developed mix ([`crate::develop::DevelopOptions::mono_mix`],
/// `SPEC-093`, `DEC-084` item 8) is this mixer's output run through the mono
/// develop's geometry, so it is fixed by the same pair and the options: a
/// change to the mixer's arithmetic bumps this, and a change to where the mix
/// sits in the develop bumps [`crate::DEVELOP_VERSION`].
///
/// `"1"`: SPEC-092 (`DEC-083`), the first mixer.
pub const MONO_MIX_VERSION: &str = "1";

/// `2^24`, the row's scale ([`MONO_MIX_ROW_FRACTION_BITS`]), as `f64`.
const MIX_ROW_SCALE: f64 = 16_777_216.0;

/// The fade width `2^13` ([`MONO_MIX_FADE_WIDTH_BITS`]), as `i64`.
const MIX_FADE_WIDTH: i64 = 8192;

/// The kernel's final shift: the row's 24 fraction bits plus the fade's 13.
const MIX_SHIFT: u32 = 37;

/// Half of `2^37`, the kernel's round-half-up bias.
const MIX_HALF: i64 = 1 << 36;

/// The **fixed-point mixer row** for `weights` (`SPEC-092`, `DEC-083`): the
/// three `i32`s, 24 fraction bits ([`MONO_MIX_ROW_FRACTION_BITS`]), that
/// [`mono_mix_plane_into`] and [`mono_mix_plane_rows_into`] apply to each
/// pixel's camera triple. Entry `i` weighs camera channel `i` (R, G, B).
///
/// `weights` are linear-sRGB `[R, G, B]` (`DEC-079`): each a non-negative
/// [`Rational`] with a positive denominator, the three summing to exactly one
/// (checked exactly). Each is converted to `f64` by one division, folded into
/// the camera → linear-sRGB matrix `M` (the colour develop's own) as
/// `v_i = (w_R·M[0][i] + w_G·M[1][i]) + w_B·M[2][i]` with basic `f64`
/// operations in that order (`DEC-033`), and quantised as
/// `q_i = ⌊v_i·2^24 + 0.5⌋`. So the row is the same on every conforming
/// platform; a caller can store and compare it (with [`MONO_MIX_VERSION`] and
/// [`crate::DEVELOP_VERSION`]).
///
/// No buffer, geometry, opcode or CFA check runs: the row exists whenever the
/// colour calibration does.
///
/// # Errors
///
/// - [`Error::InvalidMixWeights`] for an ill-formed weight or a sum other than
///   one.
/// - [`Error::MissingColorCalibration`] if the colour matrix cannot be built.
/// - [`Error::MixRowOutOfRange`] if an entry does not fit the `i32` row.
///
/// Never panics.
pub fn mono_mix_row(sensor: &Sensor, weights: &[Rational; 3]) -> Result<[i32; 3], Error> {
    let w = mix_weights(weights)?;
    let matrix = build_camera_to_srgb_linear(sensor)?;
    mix_row_of(w, &matrix)
}

/// Mix a colour Bayer mosaic to a **monochrome `u16` plane at sensor
/// geometry** (`SPEC-092`, `DEC-079`, `DEC-083`): `dst` is `width·height`
/// samples, pixel `(x, y)` at `y·width + x`, **every** pixel written, the masked
/// border included. No crop, orientation or warp. The plane's black is
/// [`MONO_MIX_BLACK`] (`0`) and its white [`MONO_MIX_WHITE`] (`65535`) for
/// every file.
///
/// **The plane is not warped; the consumer applies `OpcodeList3`'s
/// `WarpRectilinear`**, as for [`srgb_linear_plane_into`]
/// ([`crate::opcode::DevelopPath::SensorGeometry`], `SPEC-098`).
///
/// The pipeline is [`srgb_linear_plane_into`]'s fused traversal
/// (`OpcodeList1`'s first `FixBadPixelsConstant` on the raw mosaic, levels,
/// `OpcodeList2` `GainMap`s, Hamilton-Adams demosaic); at each pixel, its
/// camera triple `x` is mixed with the row `q` of [`mono_mix_row`] in exact
/// `i64` arithmetic: `P = Σ_{q_i ≥ 0} q_i·x_i`, `N = Σ_{q_i < 0} q_i·x_i`,
/// `u = min(65535 − max(x), 8192)` and
/// `F = clamp(⌊(P·8192 + N·u + 2^36) / 2^37⌋, 0, 65535)`. Below
/// [`MONO_MIX_FADE_START`] that is the plain folded mix rounded half up; above
/// it the negative-weight terms fade out, so highlights stay monotone through
/// clipping (the fade of negative weights). A row with no negative entry is
/// the plain fold at every pixel, bit for bit.
///
/// `src` is the unpacked raw mosaic (`width·height`); both buffers are
/// caller-owned (`DEC-016`), and the one internal scratch is the `width·height`
/// green plane. For row strips use [`mono_mix_plane_rows_into`].
///
/// # Errors
///
/// [`Error::InvalidMixWeights`] first (the argument alone decides it); then
/// [`srgb_linear_plane_into`]'s, in its order, with
/// [`Error::MonoBufferWrongLength`] (`dst.len() != width·height`) in place of
/// [`Error::RgbBufferWrongLength`]; then [`Error::MixRowOutOfRange`] last.
/// Nothing is written unless every check passes.
///
/// Never panics.
pub fn mono_mix_plane_into(
    sensor: &Sensor,
    src: &[u16],
    weights: &[Rational; 3],
    dst: &mut [u16],
) -> Result<(), Error> {
    let w = mix_weights(weights)?;
    srgb_linear_plane(
        sensor,
        src,
        dst,
        PlaneSamples::Mono,
        mix_row_build(sensor, w),
        store_mono,
    )
}

/// [`mono_mix_plane_into`] for rows `rows` of the plane only (`SPEC-092`): a
/// band, for a plane too large to hold at once. `rows` is in whole-plane rows
/// (`0..height`, masked rows included); `dst` is `(rows.end − rows.start)·width`,
/// plane row `y` at index `(y − rows.start)·width + x`.
///
/// Every value equals the same pixel of [`mono_mix_plane_into`] **bit for
/// bit**: the band develops its rows plus a three-row halo through the same
/// fused traversal (`develop_rows_banded`, `DEC-044`) and mixes each pixel with
/// the same kernel and row.
///
/// The plane is not warped; the consumer applies `OpcodeList3`'s
/// `WarpRectilinear`, as for [`srgb_linear_plane_into`] (`SPEC-098`).
///
/// # Errors
///
/// [`mono_mix_plane_into`]'s, in its order, with [`Error::InvalidRowRange`]
/// (an empty or reversed range, or `rows.end > height`) after the geometry and
/// before [`Error::MonoBufferWrongLength`], whose `expected` is the band's
/// `(rows.end − rows.start)·width`. Nothing is written unless every check
/// passes.
///
/// Never panics.
pub fn mono_mix_plane_rows_into(
    sensor: &Sensor,
    src: &[u16],
    weights: &[Rational; 3],
    rows: core::ops::Range<u32>,
    dst: &mut [u16],
) -> Result<(), Error> {
    let w = mix_weights(weights)?;
    srgb_linear_plane_rows(
        sensor,
        src,
        rows,
        dst,
        PlaneSamples::Mono,
        mix_row_build(sensor, w),
        store_mono,
    )
}

/// The mixer's `build` step: the colour matrix, then the row folded from it
/// (so `MixRowOutOfRange` follows `MissingColorCalibration`).
fn mix_row_build(sensor: &Sensor, w: [f64; 3]) -> impl FnOnce() -> Result<[i32; 3], Error> + '_ {
    move || {
        let matrix = build_camera_to_srgb_linear(sensor)?;
        mix_row_of(w, &matrix)
    }
}

/// The mono sink: one sample, the [`mono_mix_pixel`] of the camera triple.
fn store_mono(out: &mut [u16; 1], row: &[i32; 3], rgb: [u16; 3]) {
    *out = [mono_mix_pixel(row, rgb)];
}

/// Check the weights (`DEC-083` §1) and convert each by one `f64` division:
/// every denominator positive and every numerator non-negative, in channel
/// order, then `n0·d1·d2 + n1·d0·d2 + n2·d0·d1 == d0·d1·d2` exactly in `i128`.
fn mix_weights(weights: &[Rational; 3]) -> Result<[f64; 3], Error> {
    for (channel, w) in (0u8..).zip(weights) {
        if w.den <= 0 || w.num < 0 {
            return Err(Error::InvalidMixWeights {
                channel: Some(channel),
            });
        }
    }
    let [w0, w1, w2] = weights;
    let (n0, n1, n2) = (i128::from(w0.num), i128::from(w1.num), i128::from(w2.num));
    let (d0, d1, d2) = (i128::from(w0.den), i128::from(w1.den), i128::from(w2.den));
    // Each product of three `i32`s is below 2^93 and the sum below 2^95, so no
    // step overflows `i128`; the checked forms only satisfy the crate's lints.
    let product = |a: i128, b: i128, c: i128| a.checked_mul(b).and_then(|ab| ab.checked_mul(c));
    let sum = product(n0, d1, d2)
        .zip(product(n1, d0, d2))
        .and_then(|(a, b)| a.checked_add(b))
        .zip(product(n2, d0, d1))
        .and_then(|(ab, c)| ab.checked_add(c));
    if sum.is_none() || sum != product(d0, d1, d2) {
        return Err(Error::InvalidMixWeights { channel: None });
    }
    Ok([weight_to_f64(w0), weight_to_f64(w1), weight_to_f64(w2)])
}

/// One weight as `f64`: `num / den`, one correctly-rounded division
/// (`DEC-083` §1), so `3/10` is the `f64` nearest 0.3.
fn weight_to_f64(w: &Rational) -> f64 {
    f64::from(w.num) / f64::from(w.den)
}

/// The folded row `v = wᵀ·M` (`DEC-079`, `DEC-083` §2): for each camera channel
/// `i`, `(w_0·M[0][i] + w_1·M[1][i]) + w_2·M[2][i]`, basic `f64` operations in
/// that order, no `mul_add` (`DEC-033`). `M`'s rows are linear-sRGB channels
/// and its columns camera channels.
fn fold_mix_row(w: [f64; 3], m: &[[f64; 3]; 3]) -> [f64; 3] {
    let [w0, w1, w2] = w;
    let [m0, m1, m2] = m;
    let [[a0, a1, a2], [b0, b1, b2], [c0, c1, c2]] = [*m0, *m1, *m2];
    [
        (w0 * a0 + w1 * b0) + w2 * c0,
        (w0 * a1 + w1 * b1) + w2 * c1,
        (w0 * a2 + w1 * b2) + w2 * c2,
    ]
}

/// The fixed-point row of `w` on `m`: [`fold_mix_row`], each entry scaled by
/// `2^24` (exact) and quantised by [`quantize_row_entry`].
fn mix_row_of(w: [f64; 3], m: &[[f64; 3]; 3]) -> Result<[i32; 3], Error> {
    let [v0, v1, v2] = fold_mix_row(w, m);
    let q = |channel: u8, v: f64| {
        quantize_row_entry(v * MIX_ROW_SCALE).ok_or(Error::MixRowOutOfRange { channel })
    };
    Ok([q(0, v0)?, q(1, v1)?, q(2, v2)?])
}

/// `⌊t + 0.5⌋` as `i32` (round half up, `DEC-018`), or `None` when `t` is not
/// finite or `t + 0.5` lies outside `[−2^31, 2^31)` (`DEC-083` §3).
fn quantize_row_entry(t: f64) -> Option<i32> {
    if !t.is_finite() {
        return None;
    }
    let shifted = t + 0.5;
    if !(-2_147_483_648.0..2_147_483_648.0).contains(&shifted) {
        return None;
    }
    // In range and floored, so the cast is exact.
    Some(shifted.floor() as i32)
}

/// The mixer's per-pixel step (`DEC-083` §5), exact `i64` on the camera triple
/// `x`: `P = Σ_{q_i ≥ 0} q_i·x_i`, `N = Σ_{q_i < 0} q_i·x_i`, `m = max(x)`,
/// `u = min(65535 − m, 8192)`, `F = clamp((P·8192 + N·u + 2^36) >> 37, 0, 65535)`
/// (an arithmetic shift floors, so this rounds half up). The one kernel every
/// mono entry uses, whole plane and band alike.
///
/// No intermediate can overflow (`DEC-083` "Overflow": `|P|, |N| < 3·2^47`, so
/// every term is below `2^62`); the saturating forms satisfy the crate's lints
/// and never engage.
fn mono_mix_pixel(q: &[i32; 3], x: [u16; 3]) -> u16 {
    let mut positive: i64 = 0;
    let mut negative: i64 = 0;
    for (&qi, &xi) in q.iter().zip(&x) {
        let term = i64::from(qi).saturating_mul(i64::from(xi));
        if qi >= 0 {
            positive = positive.saturating_add(term);
        } else {
            negative = negative.saturating_add(term);
        }
    }
    let m = x.into_iter().max().unwrap_or(0);
    let u = i64::from(MONO_MIX_WHITE)
        .saturating_sub(i64::from(m))
        .min(MIX_FADE_WIDTH);
    let sum = positive
        .saturating_mul(MIX_FADE_WIDTH)
        .saturating_add(negative.saturating_mul(u))
        .saturating_add(MIX_HALF);
    // Clamped to `[0, 65535]` first, so the cast is exact; without the clamp
    // it would wrap (AC14 pins the extreme).
    (sum >> MIX_SHIFT).clamp(0, i64::from(MONO_MIX_WHITE)) as u16
}

// ─────────────────────────────────────────────────────────────────────────────
// The developed mix (`SPEC-093`, `DEC-084`): the mixer's steps for
// `crate::develop`'s `develop_core`, which owns the order and the geometry.
// ─────────────────────────────────────────────────────────────────────────────

/// The developed mix's weights check, `develop_core`'s rank 1 (`DEC-083` §1):
/// [`Error::InvalidMixWeights`], or the weights as `f64`.
pub(crate) fn developed_mix_weights(weights: &[Rational; 3]) -> Result<[f64; 3], Error> {
    mix_weights(weights)
}

/// The developed mix's stage 2, resolved once per develop (rank 5): the colour
/// path's levels (`DEC-068` §2, `DEC-070`, so `DevelopOptions::black_levels` is
/// not read), then `OpcodeList2`'s `GainMap`s, then `OpcodeList1`'s first
/// `FixBadPixelsConstant` (`DEC-086` items 1 and 7), exactly as
/// [`mono_mix_plane_into`] resolves them.
pub(crate) struct MixStageTwo<'a> {
    levels: ColourLevels<'a>,
    gains: GainMaps<'a>,
    fix: Option<CfaBadPixels>,
}

/// Rank 5: [`Error::InvalidLevels`] / [`Error::InvalidBlackLevelPattern`], then
/// the `GainMap` refusals, then `OpcodeList1`'s (a malformed first
/// `FixBadPixelsConstant`, optional or not).
pub(crate) fn resolve_mix_stage_two(sensor: &Sensor) -> Result<MixStageTwo<'_>, Error> {
    let levels = resolve_colour_levels(sensor)?;
    let gains = resolve_colour_gain_maps(sensor)?;
    let fix = resolve_colour_bad_pixels(sensor)?;
    Ok(MixStageTwo { levels, gains, fix })
}

/// Ranks 9 to 11: the CFA ([`Error::NotAColorFilterArray`], then the
/// unsupported-CFA errors), the colour calibration
/// ([`Error::MissingColorCalibration`]) and the row
/// ([`Error::MixRowOutOfRange`]), in the fused path's order.
pub(crate) fn developed_mix_row(sensor: &Sensor, weights: [f64; 3]) -> Result<[i32; 3], Error> {
    check_supported_bayer(sensor)?;
    mix_row_build(sensor, weights)()
}

/// Where the strip pass writes each mixed pixel.
pub(crate) enum MixTarget {
    /// No non-identity warp: the framed rows only, each framed pixel straight to
    /// its output slot (`dst`) through the [`CropScatter`] of the options-applied
    /// geometry.
    Framed,
    /// A non-identity warp: the `ActiveArea` rows, each active pixel into the
    /// `ActiveArea`-sized buffer `A1` ([`ActiveWindow`]), for the mono develop's
    /// warp and gather.
    Active,
}

/// The developed mix's strip pass (`DEC-084` item 5): the mixed value of every
/// pixel of `target`'s rows, written to `out` through `target`'s slot, in strips
/// of `MIX_STRIP_ROWS` rows, each strip through the band traversal on the
/// caller's `work` (`T` samples, [`crate::demosaic::mix_strip_work_len`]).
/// Strips are output-invariant (the band's halo), so each value is the whole
/// plane's [`mono_mix_plane_into`] value at that pixel.
// Eight arguments: the develop's resolved pieces, each borrowed separately so
// `develop_core` keeps owning its plan; bundling them would only rename them.
#[allow(clippy::too_many_arguments)]
pub(crate) fn developed_mix_strips(
    sensor: &Sensor,
    src: &[u16],
    stage: &MixStageTwo<'_>,
    row: &[i32; 3],
    geometry: &Geometry,
    target: MixTarget,
    out: &mut [u16],
    work: &mut [u16],
) -> Result<(), Error> {
    let to_usize = |v: u32| usize::try_from(v).unwrap_or(usize::MAX);
    match target {
        MixTarget::Framed => {
            let top = geometry.active_top.saturating_add(geometry.crop_origin_y);
            let rows = (
                to_usize(top),
                to_usize(top.saturating_add(geometry.crop_height)),
            );
            let slots = CropScatter::from_geometry(sensor.width, geometry);
            mix_strips(sensor, src, stage, row, rows, slots, out, work)
        }
        MixTarget::Active => {
            let top = geometry.active_top;
            let rows = (
                to_usize(top),
                to_usize(top.saturating_add(geometry.active_height)),
            );
            let slots = ActiveWindow::new(sensor.width, geometry);
            mix_strips(sensor, src, stage, row, rows, slots, out, work)
        }
    }
}

/// [`developed_mix_strips`] for one slot: bind the colour normalizer once, then
/// run the strips.
#[allow(clippy::too_many_arguments)]
fn mix_strips<P: PixelSlot>(
    sensor: &Sensor,
    src: &[u16],
    stage: &MixStageTwo<'_>,
    row: &[i32; 3],
    rows: (usize, usize),
    slots: P,
    out: &mut [u16],
    work: &mut [u16],
) -> Result<(), Error> {
    let (levels, gains, fix) = (stage.levels, &stage.gains, stage.fix);
    with_normalizer!(sensor, src, levels, gains, fix, |normalize| {
        mix_strips_with(sensor, &normalize, row, rows, slots, out, work)
    })
}

/// The strips `[s, min(s + MIX_STRIP_ROWS, end))` of `rows`, in order.
fn mix_strips_with<P, N>(
    sensor: &Sensor,
    normalize: &N,
    row: &[i32; 3],
    (start, end): (usize, usize),
    mut slots: P,
    out: &mut [u16],
    work: &mut [u16],
) -> Result<(), Error>
where
    P: PixelSlot,
    N: Fn(u32, u32) -> u16,
{
    let mut s = start;
    while s < end {
        let e = core::cmp::min(s.saturating_add(crate::demosaic::MIX_STRIP_ROWS), end);
        develop_rows_banded_in(sensor, (s, e), normalize, work, |pixel, r, g, b| {
            let Some(slot) = slots.slot_of(pixel) else {
                return;
            };
            if let Some(sample) = out.get_mut(slot) {
                *sample = mono_mix_pixel(row, [r, g, b]);
            }
        })?;
        s = e;
    }
    Ok(())
}

/// The warp branch's slot (`SPEC-093`): plane pixel `(x, y)` inside the
/// `ActiveArea` goes to `A1` index `(y − top)·active_width + (x − left)`; any
/// other pixel (the masked border) is dropped. The row is cached as
/// [`CropScatter`] caches it.
struct ActiveWindow {
    plane_width: usize,
    left: usize,
    top: usize,
    width: usize,
    height: usize,
    /// The cached row: plane indices `row_start..row_end`, and its first `A1`
    /// index (`None` when the row is outside the `ActiveArea`).
    row_start: usize,
    row_end: usize,
    row_base: Option<usize>,
}

impl ActiveWindow {
    fn new(plane_width: u32, geometry: &Geometry) -> Self {
        let to_usize = |v: u32| usize::try_from(v).unwrap_or(usize::MAX);
        Self {
            plane_width: to_usize(plane_width),
            left: to_usize(geometry.active_left),
            top: to_usize(geometry.active_top),
            width: to_usize(geometry.active_width),
            height: to_usize(geometry.active_height),
            row_start: 0,
            row_end: 0,
            row_base: None,
        }
    }

    fn enter_row(&mut self, pixel: usize) {
        let y = pixel.checked_div(self.plane_width).unwrap_or(0);
        self.row_start = y.saturating_mul(self.plane_width);
        self.row_end = self.row_start.saturating_add(self.plane_width);
        self.row_base = y
            .checked_sub(self.top)
            .filter(|active_y| *active_y < self.height)
            .and_then(|active_y| active_y.checked_mul(self.width));
    }
}

impl PixelSlot for ActiveWindow {
    fn slot_of(&mut self, pixel: usize) -> Option<usize> {
        if pixel < self.row_start || pixel >= self.row_end {
            self.enter_row(pixel);
        }
        let row_base = self.row_base?;
        let active_x = pixel
            .wrapping_sub(self.row_start)
            .checked_sub(self.left)
            .filter(|active_x| *active_x < self.width)?;
        row_base.checked_add(active_x)
    }
}

/// The camera → linear-sRGB matrix build, as the `build` step of an RGB plane
/// entry: run inside the traversal's `setup` (whole plane) or after the CFA
/// check (band), so `MissingColorCalibration` keeps the fused path's place.
fn matrix_of(sensor: &Sensor) -> impl FnOnce() -> Result<[[f64; 3]; 3], Error> + '_ {
    move || build_camera_to_srgb_linear(sensor)
}

/// What one pixel of a sensor-geometry plane holds: an interleaved RGB triple
/// (`SPEC-091`) or one mono sample (`SPEC-092`). It sizes `dst` and names the
/// refusal for a `dst` of the wrong length.
#[derive(Clone, Copy)]
enum PlaneSamples {
    Rgb,
    Mono,
}

impl PlaneSamples {
    /// Samples per pixel.
    fn per_pixel(self) -> u64 {
        match self {
            PlaneSamples::Rgb => 3,
            PlaneSamples::Mono => 1,
        }
    }

    /// The typed refusal for a `dst` of `actual` samples where `expected`
    /// were required.
    fn wrong_length(self, expected: u64, actual: usize) -> Error {
        match self {
            PlaneSamples::Rgb => Error::RgbBufferWrongLength { expected, actual },
            PlaneSamples::Mono => Error::MonoBufferWrongLength { expected, actual },
        }
    }
}

/// The whole-plane entries' body: [`develop_color_into`]'s checks in its order,
/// then the one fused traversal through the identity sink. `build` runs inside
/// the traversal's `setup` (after the CFA check), and makes what `store` reads.
fn srgb_linear_plane<T, K, B, S, const C: usize>(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [T],
    samples: PlaneSamples,
    build: B,
    store: S,
) -> Result<(), Error>
where
    B: FnOnce() -> Result<K, Error>,
    S: Fn(&mut [T; C], &K, [u16; 3]),
{
    check_sensor_geometry_required_processing(sensor)?;
    let expected_src = check_plane_buffers(sensor, src.len(), None, dst.len(), samples)?;
    let levels = resolve_colour_levels(sensor)?;
    let gains = resolve_colour_gain_maps(sensor)?;
    let fix = resolve_colour_bad_pixels(sensor)?;
    with_normalizer!(sensor, src, levels, gains, fix, |normalize| {
        develop_color_with(
            sensor,
            expected_src,
            WholePlane,
            &normalize,
            dst,
            build,
            store,
        )
    })
}

/// The band entries' body: the whole-plane checks with the row range, then the
/// CFA and `build` (the matrix, or the mixer's row) in the fused path's order
/// (`develop_rows_banded` builds nothing), all before any write, then the
/// halo'd band develop.
fn srgb_linear_plane_rows<T, K, B, S, const C: usize>(
    sensor: &Sensor,
    src: &[u16],
    rows: core::ops::Range<u32>,
    dst: &mut [T],
    samples: PlaneSamples,
    build: B,
    store: S,
) -> Result<(), Error>
where
    B: FnOnce() -> Result<K, Error>,
    S: Fn(&mut [T; C], &K, [u16; 3]),
{
    check_sensor_geometry_required_processing(sensor)?;
    check_plane_buffers(sensor, src.len(), Some(&rows), dst.len(), samples)?;
    let levels = resolve_colour_levels(sensor)?;
    let gains = resolve_colour_gain_maps(sensor)?;
    let fix = resolve_colour_bad_pixels(sensor)?;
    check_supported_bayer(sensor)?;
    let coefficients = build()?;
    with_normalizer!(sensor, src, levels, gains, fix, |normalize| {
        srgb_linear_plane_rows_with(sensor, &rows, &normalize, &coefficients, dst, &store)
    })
}

/// [`srgb_linear_plane_rows`]' body for one resolved normalizer: develop the band
/// (plus its halo) and write each of its pixels at its row offset from
/// `rows.start`.
fn srgb_linear_plane_rows_with<T, K, S, N, const C: usize>(
    sensor: &Sensor,
    rows: &core::ops::Range<u32>,
    normalize: &N,
    coefficients: &K,
    dst: &mut [T],
    store: &S,
) -> Result<(), Error>
where
    S: Fn(&mut [T; C], &K, [u16; 3]),
    N: Fn(u32, u32) -> u16,
{
    let to_usize = |v: u32| usize::try_from(v).unwrap_or(usize::MAX);
    let (start, end) = (to_usize(rows.start), to_usize(rows.end));
    // The band's first pixel, as a whole-plane index: `dst` starts there.
    let first = start.saturating_mul(to_usize(sensor.width));
    let (groups, _remainder) = dst.as_chunks_mut::<C>();
    develop_rows_banded(sensor, (start, end), normalize, |pixel, r, g, b| {
        if let Some(out) = pixel.checked_sub(first).and_then(|i| groups.get_mut(i)) {
            store(out, coefficients, [r, g, b]);
        }
    })
}

/// Validate a sensor-geometry plane's buffers (`SPEC-091`) in
/// [`check_color_buffers`]' order: `src` is `width·height`
/// ([`Error::ColorSourceWrongLength`]), then the geometry resolves (the same
/// typed errors as the cropped develop), then, for a band, the row range
/// ([`Error::InvalidRowRange`]), then `dst` is `rows·width` times the samples
/// per pixel ([`Error::RgbBufferWrongLength`] for an RGB plane,
/// [`Error::MonoBufferWrongLength`] for the `SPEC-092` mono mix; `rows` the
/// band's or the plane's height). Returns the plane pixel count (the
/// green-plane size).
fn check_plane_buffers(
    sensor: &Sensor,
    src_len: usize,
    rows: Option<&core::ops::Range<u32>>,
    dst_len: usize,
    samples: PlaneSamples,
) -> Result<usize, Error> {
    let pixels = pixel_count(sensor)?;
    let expected_src = usize::try_from(pixels).unwrap_or(usize::MAX);
    if src_len != expected_src {
        return Err(Error::ColorSourceWrongLength {
            expected: pixels,
            actual: src_len,
        });
    }
    resolve_geometry(sensor)?;

    let band_rows = match rows {
        None => sensor.height,
        Some(range) => {
            if range.start >= range.end || range.end > sensor.height {
                return Err(Error::InvalidRowRange {
                    start: range.start,
                    end: range.end,
                    height: sensor.height,
                });
            }
            range.end.saturating_sub(range.start)
        }
    };
    let dst_expected = u64::from(sensor.width)
        .checked_mul(u64::from(band_rows))
        .and_then(|v| v.checked_mul(samples.per_pixel()))
        .ok_or(samples.wrong_length(u64::MAX, dst_len))?;
    if usize::try_from(dst_expected).ok() != Some(dst_len) {
        return Err(samples.wrong_length(dst_expected, dst_len));
    }
    Ok(expected_src)
}

/// [`develop_color_into`] with opt-in **output-row-band** parallelism
/// (`SPEC-051`, `DEC-044`): the same cropped, oriented linear-sRGB render, byte
/// for byte, split across `threads` scoped threads.
///
/// `threads <= 1` is a straight call to [`develop_color_into`] — no banding, no
/// allocation. With `threads >= 2` the geometry is resolved once, then:
///
/// - **Orientations 1–4 (non-transposing)** are threaded. Each output row comes
///   from one source row (identity or vertically flipped), so an output row band
///   is fed by a contiguous source row band. The output rows are split into
///   `min(threads, out_height)` bands, each thread owning a **disjoint** `dst`
///   slice; it develops its source band plus a three-row halo through the same
///   fused traversal (local green buffer, the `SPEC-049` rolling cache) and
///   scatters through the same `SPEC-050` crop/orient map, keeping only the
///   pixels that land in its own slice.
/// - **Orientations 5–8 (the 90° rotations) fall back to [`develop_color_into`]**:
///   an output row band there is fed by source *columns* spanning every source
///   row, so no contiguous source band feeds it. Correct, just not parallel.
///
/// Every pixel is computed by the same kernel, from the same neighbours, with
/// the same `f64` matrix, in the same order — there is no cross-thread
/// reduction — so the output is byte-identical to [`develop_color_into`] and
/// independent of `threads` and of scheduling. No `DEVELOP_VERSION` change:
/// threading is output-invariant. The local green buffers sum to about one
/// `width·height` plane (the single-thread plane, partitioned); no RGB
/// intermediate exists.
///
/// # Errors
///
/// Exactly [`develop_color_into`]'s, in the same order: any input the threaded
/// path does not accept is handed to [`develop_color_into`], which returns its
/// typed error. [`Error::InvalidLevels`] (`W ≤ B`, `DEC-070` §3) is among them,
/// refused on the calling thread before any write.
///
/// Never panics: inputs (buffers, geometry, CFA, colour matrix) are validated
/// once before any thread is spawned; a thread the OS refuses to spawn only
/// means fewer workers.
pub fn develop_color_into_threaded(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    threads: usize,
) -> Result<(), Error> {
    // Its own door, so its own check (`SPEC-054`): the banded path below never
    // passes through `develop_color_into`.
    check_colour_required_processing(sensor)?;
    if threads <= 1 {
        return develop_color_into(sensor, src, dst);
    }
    let Some(plan) = colour_band_plan(sensor, src.len(), dst.len(), threads) else {
        return develop_color_into(sensor, src, dst);
    };
    // The single-thread sink's arithmetic, the same function (`develop_color_into`).
    develop_color_bands(sensor, src, dst, threads, &plan, store_u16)
}

/// [`develop_color_f32_into`] with opt-in output-row-band parallelism
/// (`SPEC-051`, `DEC-044`) — exactly `to_linear_f32_into(develop_color_into_threaded(..))`,
/// bit for bit, and so exactly [`develop_color_f32_into`].
///
/// Same banding as [`develop_color_into_threaded`]: `threads <= 1` is a straight
/// call to [`develop_color_f32_into`]; orientations 1–4 are threaded over
/// disjoint output row bands; the 90° rotations (5–8) fall back to
/// [`develop_color_f32_into`].
///
/// # Errors
///
/// Exactly [`develop_color_f32_into`]'s, in the same order, including
/// [`Error::InvalidLevels`] (`W ≤ B`, `DEC-070` §3), refused on the calling
/// thread before any write.
///
/// Never panics.
pub fn develop_color_f32_into_threaded(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [f32],
    threads: usize,
) -> Result<(), Error> {
    // Its own door, so its own check (`SPEC-054`), as in
    // `develop_color_into_threaded`.
    check_colour_required_processing(sensor)?;
    if threads <= 1 {
        return develop_color_f32_into(sensor, src, dst);
    }
    let Some(plan) = colour_band_plan(sensor, src.len(), dst.len(), threads) else {
        return develop_color_f32_into(sensor, src, dst);
    };
    // The single-thread sink's quantise-then-divide, the same function
    // (`develop_color_f32_into`).
    develop_color_bands(sensor, src, dst, threads, &plan, store_f32_quantised)
}

/// The colour path's required-processing check (`SPEC-054`, `DEC-049`): it
/// applies `OpcodeList1`'s first `FixBadPixelsConstant` (`SPEC-098`,
/// `DEC-086`) and every `OpcodeList2` `GainMap` (`SPEC-059`, `DEC-069`) and
/// nothing else, so any other mandatory opcode in any list is refused. Every
/// developed colour entry calls this first, before any allocation or decode.
fn check_colour_required_processing(sensor: &Sensor) -> Result<(), Error> {
    crate::opcode::check_required_processing(sensor, crate::opcode::DevelopPath::Colour)
}

/// The sensor-geometry entries' required-processing check (`SPEC-098`,
/// `DEC-086` item 5): the colour path's, and the first `WarpRectilinear` in
/// `OpcodeList3` is accepted without being applied (the consumer applies it).
fn check_sensor_geometry_required_processing(sensor: &Sensor) -> Result<(), Error> {
    crate::opcode::check_required_processing(sensor, crate::opcode::DevelopPath::SensorGeometry)
}

/// What a threaded colour develop needs, resolved once before spawning.
struct ColourBandPlan {
    /// The camera → linear-sRGB matrix (the single-thread path's own).
    matrix: [[f64; 3]; 3],
    /// The crop rectangle's rows, in raw-plane coordinates.
    crop_top: usize,
    crop_height: usize,
    /// Orientations 3 and 4: output row `oy` comes from crop row
    /// `crop_height − 1 − oy` (a vertical flip), else from crop row `oy`.
    vertical_flip: bool,
    out_width: usize,
    out_height: usize,
}

/// The [`ColourBandPlan`], or `None` when the input belongs on the single-thread
/// path: anything [`develop_color_into`] would reject (so its typed error comes
/// back unchanged, in its order), a transposing orientation (5–8, `DEC-044`'s
/// fallback), or fewer than two output rows (or threads) to split.
fn colour_band_plan(
    sensor: &Sensor,
    src_len: usize,
    dst_len: usize,
    threads: usize,
) -> Option<ColourBandPlan> {
    check_color_buffers(sensor, src_len, dst_len).ok()?;
    let geometry = resolve_geometry(sensor).ok()?;
    let vertical_flip = match geometry.orientation {
        1 | 2 => false,
        3 | 4 => true,
        _ => return None,
    };
    if !is_supported_bayer(sensor) {
        return None;
    }
    let matrix = build_camera_to_srgb_linear(sensor).ok()?;
    let to_usize = |v: u32| usize::try_from(v).ok();
    let out_height = to_usize(geometry.crop_height)?;
    if core::cmp::min(threads, out_height) < 2 {
        return None;
    }
    Some(ColourBandPlan {
        matrix,
        crop_top: to_usize(geometry.active_top.saturating_add(geometry.crop_origin_y))?,
        crop_height: out_height,
        vertical_flip,
        out_width: to_usize(geometry.crop_width)?,
        out_height,
    })
}

/// Develop a non-transposing colour render over disjoint output row bands
/// (`SPEC-051`, `DEC-044`). Each band works backwards from its output rows to the
/// contiguous source rows that feed them, runs the fused traversal over those
/// rows ([`develop_rows_banded`]), maps each developed pixel through the same
/// `SPEC-050` [`CropScatter`] as the single-thread sink, and `store`s the pixels
/// that land in its own slice. Generic over the output sample so the `u16` and
/// `f32` entries share it; `store` is each entry's single-thread sink arithmetic.
fn develop_color_bands<T, S>(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [T],
    threads: usize,
    plan: &ColourBandPlan,
    store: S,
) -> Result<(), Error>
where
    T: Send,
    S: Fn(&mut [T; 3], &[[f64; 3]; 3], [u16; 3]) + Sync,
{
    // Resolved once, before any band runs or `dst` is touched (`DEC-068` §2,
    // `DEC-069` §3): a refused pattern or `GainMap` returns here, from the
    // calling thread.
    let levels = resolve_colour_levels(sensor)?;
    let gains = resolve_colour_gain_maps(sensor)?;
    let fix = resolve_colour_bad_pixels(sensor)?;
    with_normalizer!(sensor, src, levels, gains, fix, |normalize| {
        develop_color_bands_with(sensor, &normalize, dst, threads, plan, store)
    })
}

/// [`develop_color_bands`]' body for one resolved normalizer.
fn develop_color_bands_with<T, S, N>(
    sensor: &Sensor,
    normalize: &N,
    dst: &mut [T],
    threads: usize,
    plan: &ColourBandPlan,
    store: S,
) -> Result<(), Error>
where
    T: Send,
    S: Fn(&mut [T; 3], &[[f64; 3]; 3], [u16; 3]) + Sync,
    N: Fn(u32, u32) -> u16 + Sync,
{
    let (triples, _remainder) = dst.as_chunks_mut::<3>();
    run_row_bands(
        triples,
        plan.out_width,
        plan.out_height,
        threads,
        |oy0, oy1, band| {
            // Output rows `oy0..oy1` come from crop rows `oy0..oy1`, or — under a
            // vertical flip — from crop rows `crop_height − oy1 .. crop_height − oy0`.
            let (cy0, cy1) = if plan.vertical_flip {
                (
                    plan.crop_height.saturating_sub(oy1),
                    plan.crop_height.saturating_sub(oy0),
                )
            } else {
                (oy0, oy1)
            };
            let rows = (
                plan.crop_top.saturating_add(cy0),
                plan.crop_top.saturating_add(cy1),
            );
            let first = oy0.saturating_mul(plan.out_width);
            let mut scatter = CropScatter::new(sensor)?;
            develop_rows_banded(sensor, rows, normalize, |pixel, r, g, b| {
                let Some(slot) = scatter.slot_of(pixel) else {
                    return;
                };
                if let Some(out) = slot.checked_sub(first).and_then(|i| band.get_mut(i)) {
                    store(out, &plan.matrix, [r, g, b]);
                }
            })
        },
    )
}

/// Validate a colour develop's buffers against `sensor`, in `develop_into`'s
/// order: `src` is `width·height` ([`Error::ColorSourceWrongLength`]), then the
/// geometry resolves (`resolve_geometry`'s typed `InvalidActiveArea` /
/// `InvalidDefaultCrop` / `UnsupportedOrientation`), then `dst` is
/// `output_dimensions·3` ([`Error::RgbBufferWrongLength`]). Returns the plane
/// pixel count (the green-plane size) and the sink's crop/orient map.
fn check_color_buffers(
    sensor: &Sensor,
    src_len: usize,
    dst_len: usize,
) -> Result<(usize, CropScatter), Error> {
    let pixels = pixel_count(sensor)?;
    let expected_src = usize::try_from(pixels).unwrap_or(usize::MAX);
    if src_len != expected_src {
        return Err(Error::ColorSourceWrongLength {
            expected: pixels,
            actual: src_len,
        });
    }

    let output = CropScatter::new(sensor)?;
    let rgb_len = u64::from(output.out_width)
        .checked_mul(u64::from(output.out_height))
        .and_then(|v| v.checked_mul(3))
        .ok_or(Error::RgbBufferWrongLength {
            expected: u64::MAX,
            actual: dst_len,
        })?;
    if usize::try_from(rgb_len).ok() != Some(dst_len) {
        return Err(Error::RgbBufferWrongLength {
            expected: rgb_len,
            actual: dst_len,
        });
    }
    Ok((expected_src, output))
}

/// The colour path's crop + orient scatter (`SPEC-050`, `DEC-043`): maps each
/// developed plane pixel, by its raster index, to its slot in the output — or
/// drops it. It applies the mono path's resolved geometry (`ActiveArea` →
/// `DefaultCrop`, relative to the ActiveArea per `DEC-019` → `Orientation`), so
/// the colour output has [`crate::develop::output_dimensions`]'s shape.
///
/// Every orientation is a flip/transpose of the crop rectangle, so the output
/// index is affine in the crop coordinates: `base + crop_x·step_x +
/// crop_y·step_y`. The three coefficients are read off
/// [`output_coords_from_crop`] once, here, rather than matching the orientation
/// per pixel. The kernel emits each row's pixels together, so the row
/// (`y`, and with it `base + crop_y·step_y`) is re-derived — one `checked_div` —
/// only when the pixel index leaves the current row; any emit order is still
/// correct, a pixel outside the cached row just re-derives it.
struct CropScatter {
    plane_width: usize,
    /// The crop rectangle in raw-plane coordinates: `ActiveArea` origin +
    /// `DefaultCropOrigin`, and `DefaultCropSize`.
    crop_left: usize,
    crop_top: usize,
    crop_width: usize,
    crop_height: usize,
    /// [`crate::develop::output_dimensions`] — the crop, transposed for 5-8.
    out_width: u32,
    out_height: u32,
    /// The affine output-index map (in output pixels, not samples).
    base: i64,
    step_x: i64,
    step_y: i64,
    /// The cached row: plane indices `row_start..row_end`, and its
    /// `base + crop_y·step_y` (`None` when the row is outside the crop).
    row_start: usize,
    row_end: usize,
    row_base: Option<i64>,
}

impl CropScatter {
    fn new(sensor: &Sensor) -> Result<Self, Error> {
        let geometry = resolve_geometry(sensor)?;
        Ok(Self::from_geometry(sensor.width, &geometry))
    }

    /// The scatter for an already-resolved geometry on a plane `plane_width`
    /// wide (`SPEC-093`): the developed mix builds it from the
    /// options-applied geometry, so the framing and `apply_orientation` are the
    /// mono develop's.
    fn from_geometry(plane_width: u32, geometry: &Geometry) -> Self {
        let orientation = geometry.orientation;
        let (crop_width, crop_height) = (geometry.crop_width, geometry.crop_height);
        let (out_width, out_height) = oriented_dimensions(orientation, crop_width, crop_height);

        // The output index of crop pixel (cx, cy), via the inverse orientation map.
        let index_of = |crop_x: u32, crop_y: u32| -> i64 {
            let (out_x, out_y) =
                output_coords_from_crop(orientation, crop_x, crop_y, crop_width, crop_height);
            i64::from(out_y)
                .wrapping_mul(i64::from(out_width))
                .wrapping_add(i64::from(out_x))
        };
        let base = index_of(0, 0);
        // A one-pixel-wide (or -tall) crop never moves along that axis, so its
        // step is irrelevant; 0 keeps it from reading outside the rectangle.
        let step_x = if crop_width > 1 {
            index_of(1, 0).wrapping_sub(base)
        } else {
            0
        };
        let step_y = if crop_height > 1 {
            index_of(0, 1).wrapping_sub(base)
        } else {
            0
        };

        let to_usize = |v: u32| usize::try_from(v).unwrap_or(usize::MAX);
        Self {
            plane_width: to_usize(plane_width),
            // Cannot overflow: `resolve_geometry` proved the crop fits inside
            // the ActiveArea, which fits inside the `u32`-sized plane.
            crop_left: to_usize(geometry.active_left.saturating_add(geometry.crop_origin_x)),
            crop_top: to_usize(geometry.active_top.saturating_add(geometry.crop_origin_y)),
            crop_width: to_usize(crop_width),
            crop_height: to_usize(crop_height),
            out_width,
            out_height,
            base,
            step_x,
            step_y,
            row_start: 0,
            row_end: 0,
            row_base: None,
        }
    }

    /// Re-derive the cached row for `pixel`.
    fn enter_row(&mut self, pixel: usize) {
        let y = pixel.checked_div(self.plane_width).unwrap_or(0);
        self.row_start = y.saturating_mul(self.plane_width);
        self.row_end = self.row_start.saturating_add(self.plane_width);
        self.row_base = y
            .checked_sub(self.crop_top)
            .filter(|crop_y| *crop_y < self.crop_height)
            .and_then(|crop_y| i64::try_from(crop_y).ok())
            .map(|crop_y| self.base.wrapping_add(crop_y.wrapping_mul(self.step_y)));
    }
}

impl PixelSlot for CropScatter {
    /// The output pixel index (into `dst`'s RGB triples) for plane pixel
    /// `pixel = y·width + x`, or `None` if it lies outside the crop rectangle
    /// (the masked border and the non-crop active margin are developed — their
    /// green feeds neighbours — but never written).
    fn slot_of(&mut self, pixel: usize) -> Option<usize> {
        if pixel < self.row_start || pixel >= self.row_end {
            self.enter_row(pixel);
        }
        let row_base = self.row_base?;
        let crop_x = pixel
            .wrapping_sub(self.row_start)
            .checked_sub(self.crop_left)
            .filter(|crop_x| *crop_x < self.crop_width)?;
        let offset = i64::try_from(crop_x).ok()?.wrapping_mul(self.step_x);
        usize::try_from(row_base.wrapping_add(offset)).ok()
    }
}

/// `width·height·3`, the interleaved-RGB sample count, as `u64` — errors (rather
/// than wrapping) if the product overflows, mirroring `demosaic_into`.
fn interleaved_rgb_len(sensor: &Sensor) -> Result<u64, Error> {
    pixel_count(sensor)?
        .checked_mul(3)
        .ok_or(Error::RgbBufferWrongLength {
            expected: u64::MAX,
            actual: 0,
        })
}

/// `width·height`, the mosaic / pixel count, as `u64`.
fn pixel_count(sensor: &Sensor) -> Result<u64, Error> {
    u64::from(sensor.width)
        .checked_mul(u64::from(sensor.height))
        .ok_or(Error::ColorSourceWrongLength {
            expected: u64::MAX,
            actual: 0,
        })
}

/// Build the single camera-linear-RGB → linear-sRGB matrix `M = C · cam2xyz`
/// from `sensor`'s colour metadata (`DEC-033`, `DEC-034`, `DEC-076`):
/// `C = S · B` (committed), and `cam2xyz` from the calibration at the scene
/// weight `g` ([`scene_weight`]), or from illuminant 1 alone when interpolation
/// does not apply. `AnalogBalance` and, under the signature rule,
/// `CameraCalibration1/2` are folded in ([`resolve_calibration`]); an exactly
/// identity calibration takes today's arithmetic, so its output is unchanged
/// to the bit.
///
/// # Errors
///
/// [`Error::MissingColorCalibration`] if `AsShotNeutral` is absent / ill-shaped
/// / non-positive, neither `ForwardMatrix1` nor `ColorMatrix1` is present and
/// well-shaped, a required rational has a zero denominator, an applied
/// calibration or `AnalogBalance` is ill-shaped (`DEC-076` §3), the folded
/// `ColorMatrix` or the calibration `K` is singular, or the ForwardMatrix
/// path's reference neutral has a non-positive component.
fn build_camera_to_srgb_linear(sensor: &Sensor) -> Result<[[f64; 3]; 3], Error> {
    let asn = sensor
        .as_shot_neutral
        .as_ref()
        .ok_or(Error::MissingColorCalibration)?;
    let wb = wb_diagonal(asn)?;

    // `SPEC-034`'s preconditions decide whether the dual-illuminant path runs;
    // that in turn decides whether `CameraCalibration2` is used, and so checked
    // (`DEC-076` §3).
    let dual = dual_illuminant(sensor)?;
    let calibration = resolve_calibration(sensor, dual.is_some())?;

    // The scene weight (`NeutralToXY` to convergence, `DEC-076` §5), found once
    // per image. `None` ⇒ the single-illuminant path.
    let weighted = dual
        .as_ref()
        .and_then(|d| scene_weight(d, &calibration, asn).map(|g| (d, g)));
    let cam2xyz = match weighted {
        Some((d, g)) => cam_to_xyz_interpolated(sensor, &wb, asn, &calibration, d, g)?,
        None => cam_to_xyz_single_illuminant(sensor, &wb, asn, &calibration)?,
    };

    let c = mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
    Ok(mat_mul(c, cam2xyz))
}

/// The 3×3 identity: an absent `AnalogBalance` or calibration, and the test for
/// "the calibration changes nothing" (`DEC-076` §4).
const IDENTITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// The camera calibration the develop applies (`DEC-076` §2–3): `AB` as a
/// diagonal, `CC_1`/`CC_2` after the signature rule (the identity when it says
/// no or the tag is absent), and the folded `K_i = AB · CC_i`.
struct Calibration {
    ab: [[f64; 3]; 3],
    cc1: [[f64; 3]; 3],
    cc2: [[f64; 3]; 3],
    k1: [[f64; 3]; 3],
    k2: [[f64; 3]; 3],
}

/// Resolve [`Calibration`] from `sensor` (`DEC-076` §2–3).
///
/// - `AnalogBalance` is not gated: absent is the identity, present must be 3
///   rationals with positive numerators and denominators.
/// - `CameraCalibration1/2` apply only when `CameraCalibrationSignature` equals
///   `ProfileCalibrationSignature`, an absent signature counting as empty.
/// - Every **present** calibration is checked, used or not (`DEC-078` §7.2,
///   `SPEC-089`): it must be 9 rationals with non-zero denominators. Only
///   what is used is applied: `CameraCalibration1` whenever the rule applies,
///   `CameraCalibration2` only when it applies **and** `dual` (the
///   dual-illuminant path runs). Absent or unused, it is the identity.
///
/// Anything else is [`Error::MissingColorCalibration`], before any write.
fn resolve_calibration(sensor: &Sensor, dual: bool) -> Result<Calibration, Error> {
    let ab = match sensor.analog_balance.as_deref() {
        None => IDENTITY,
        Some([r, g, b]) => [
            [positive_rational(r)?, 0.0, 0.0],
            [0.0, positive_rational(g)?, 0.0],
            [0.0, 0.0, positive_rational(b)?],
        ],
        Some(_) => return Err(Error::MissingColorCalibration),
    };
    let camera: &[u8] = sensor
        .camera_calibration_signature
        .as_deref()
        .unwrap_or_default();
    let profile: &[u8] = sensor
        .profile_calibration_signature
        .as_deref()
        .unwrap_or_default();
    let applies = camera == profile;
    // A present calibration is read (and so validated) whether or not it is
    // used: an unusable one is never silently the identity (`DEC-078` §7.2).
    let used = |tag: Option<&Vec<Rational>>, used: bool| -> Result<[[f64; 3]; 3], Error> {
        match tag
            .map(|entries| matrix_from_rationals(entries))
            .transpose()?
        {
            Some(matrix) if used => Ok(matrix),
            _ => Ok(IDENTITY),
        }
    };
    let cc1 = used(sensor.camera_calibration_1.as_ref(), applies)?;
    let cc2 = used(sensor.camera_calibration_2.as_ref(), applies && dual)?;
    Ok(Calibration {
        ab,
        cc1,
        cc2,
        k1: mat_mul(ab, cc1),
        k2: mat_mul(ab, cc2),
    })
}

/// `num/den` for a rational both of whose parts are positive, else
/// [`Error::MissingColorCalibration`] (an `AnalogBalance` gain, `DEC-076` §3).
fn positive_rational(r: &Rational) -> Result<f64, Error> {
    if r.num <= 0 || r.den <= 0 {
        return Err(Error::MissingColorCalibration);
    }
    Ok(f64::from(r.num) / f64::from(r.den))
}

/// The folded `X = K · CM` (`DEC-076` §4), or `CM` itself when `K` is exactly
/// the identity, so an identity calibration is byte-identical to none.
fn fold(k: [[f64; 3]; 3], cm: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    if k == IDENTITY {
        cm
    } else {
        mat_mul(k, cm)
    }
}

/// The single-illuminant `cam2xyz` — illuminant 1 only (`SPEC-033`, `DEC-076`
/// §4): the ForwardMatrix path with `K = AB · CC_1` when `ForwardMatrix1` is
/// present and well-shaped, else `CA · inv(AB · CC_1 · CM_1)`, else
/// [`Error::MissingColorCalibration`]. With an identity calibration this is
/// byte-identical to `SPEC-033` (the regression guard). `CameraCalibration2` is
/// never read here.
fn cam_to_xyz_single_illuminant(
    sensor: &Sensor,
    wb: &[[f64; 3]; 3],
    asn: &[Rational],
    calibration: &Calibration,
) -> Result<[[f64; 3]; 3], Error> {
    if let Some(fm1) = sensor.forward_matrix_1.as_ref() {
        let fm = matrix_from_rationals(fm1)?;
        let k = (calibration.k1 != IDENTITY).then_some(calibration.k1);
        fm_path_cam_to_xyz(fm, wb, asn, k)
    } else if let Some(cm1) = sensor.color_matrix_1.as_ref() {
        let x = fold(calibration.k1, matrix_from_rationals(cm1)?);
        let inv_x = inv(x).ok_or(Error::MissingColorCalibration)?;
        cm_path_cam_to_xyz(&inv_x, asn)
    } else {
        Err(Error::MissingColorCalibration)
    }
}

/// The interpolated `cam2xyz` at a known scene weight `g` (`SPEC-034`,
/// `DEC-076` §4). The **ForwardMatrix path**, `FM(g) = lerp(FM1, FM2, g)` with
/// `K(g) = AB · lerp(CC_1, CC_2, g)`, runs when **both** forward matrices are
/// present; otherwise (one absent) the **ColorMatrix path** inverts the lerp of
/// the folded products, `X(g) = lerp(AB·CC_1·CM_1, AB·CC_2·CM_2, g)` (the SDK's
/// form). A present forward matrix that cannot be read is an error, never a
/// switch to the ColorMatrix path (`DEC-078` §7.2). A singular `X(g)` is
/// [`Error::MissingColorCalibration`].
fn cam_to_xyz_interpolated(
    sensor: &Sensor,
    wb: &[[f64; 3]; 3],
    asn: &[Rational],
    calibration: &Calibration,
    dual: &DualIlluminant,
    g: f64,
) -> Result<[[f64; 3]; 3], Error> {
    let fm1 = sensor
        .forward_matrix_1
        .as_deref()
        .map(matrix_from_rationals)
        .transpose()?;
    let fm2 = sensor
        .forward_matrix_2
        .as_deref()
        .map(matrix_from_rationals)
        .transpose()?;
    if let (Some(f1), Some(f2)) = (fm1, fm2) {
        let fm = lerp_matrix(f1, f2, g);
        let k = if calibration.k1 == IDENTITY && calibration.k2 == IDENTITY {
            None
        } else {
            Some(mat_mul(
                calibration.ab,
                lerp_matrix(calibration.cc1, calibration.cc2, g),
            ))
        };
        return fm_path_cam_to_xyz(fm, wb, asn, k);
    }

    let x = lerp_matrix(
        fold(calibration.k1, dual.cm1),
        fold(calibration.k2, dual.cm2),
        g,
    );
    let inv_x = inv(x).ok_or(Error::MissingColorCalibration)?;
    cm_path_cam_to_xyz(&inv_x, asn)
}

/// `cam2xyz` for the **ForwardMatrix** path (`DEC-076` §4, DNG spec ch. 6).
/// With no calibration (`k` is `None`) it is `DEC-033`'s `FM · D`, `D` built from
/// the `AsShotNeutral` rationals. With a calibration `K` it takes the reference
/// neutral `R = inv(K) · AsShotNeutral` and returns `FM · diag(1/R) · inv(K)`.
///
/// [`Error::MissingColorCalibration`] if `K` is singular or any component of
/// `R` is not positive.
fn fm_path_cam_to_xyz(
    fm: [[f64; 3]; 3],
    wb: &[[f64; 3]; 3],
    asn: &[Rational],
    k: Option<[[f64; 3]; 3]>,
) -> Result<[[f64; 3]; 3], Error> {
    let Some(k) = k else {
        return Ok(mat_mul(fm, *wb));
    };
    let inv_k = inv(k).ok_or(Error::MissingColorCalibration)?;
    let neutral = neutral_vector(asn).ok_or(Error::MissingColorCalibration)?;
    let [r0, r1, r2] = mul_mat_vec(&inv_k, neutral);
    if !(r0 > 0.0 && r1 > 0.0 && r2 > 0.0) {
        return Err(Error::MissingColorCalibration);
    }
    let d = [
        [1.0 / r0, 0.0, 0.0],
        [0.0, 1.0 / r1, 0.0],
        [0.0, 0.0, 1.0 / r2],
    ];
    Ok(mat_mul(fm, mat_mul(d, inv_k)))
}

/// Bradford cone-response matrix (XYZ → sharpened RGB), the `M_A` of the
/// Bradford chromatic-adaptation transform. A committed `f64` constant
/// (`DEC-033`), transcribed once from the standard (Lindbloom / CIE).
const BRADFORD_CONE: [[f64; 3]; 3] = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

/// CIE XYZ of the D50 white (the DNG profile connection space white), `Y = 1`.
const D50_WHITE: [f64; 3] = [0.96422, 1.0, 0.82521];

/// `cam2xyz` for the **ColorMatrix** path: `CA(W → D50) · inv(CM)`, where the
/// scene white is `W = inv(CM) · AsShotNeutral` (DNG spec ch. 6, "Mapping Camera
/// Color Space to CIE XYZ Space"). `ColorMatrix` maps XYZ **under the
/// calibration illuminant** to camera values, so `inv(CM)` already yields XYZ
/// of the scene as lit; no white-balance diagonal `D` may be applied (it would
/// count white balance twice and tint neutral magenta). Instead the scene white
/// is Bradford-adapted to D50, so a raw neutral `c = AsShotNeutral` lands on D50
/// white (`Y = 1`), which the committed `C = S · B` maps to linear sRGB
/// `(1, 1, 1)`. Basic `f64` ops only (`DEC-033`): the cone matrix, its inverse
/// and a diagonal of ratios.
///
/// [`Error::MissingColorCalibration`] if the neutral is malformed, or the white
/// is degenerate (non-positive cone responses) or the cone matrix fails to invert.
fn cm_path_cam_to_xyz(inv_cm: &[[f64; 3]; 3], asn: &[Rational]) -> Result<[[f64; 3]; 3], Error> {
    let neutral = neutral_vector(asn).ok_or(Error::MissingColorCalibration)?;
    let white = mul_mat_vec(inv_cm, neutral);
    let rho_src = mul_mat_vec(&BRADFORD_CONE, white);
    let rho_dst = mul_mat_vec(&BRADFORD_CONE, D50_WHITE);
    if !(rho_src[0] > 0.0 && rho_src[1] > 0.0 && rho_src[2] > 0.0) {
        return Err(Error::MissingColorCalibration);
    }
    let scale = [
        [rho_dst[0] / rho_src[0], 0.0, 0.0],
        [0.0, rho_dst[1] / rho_src[1], 0.0],
        [0.0, 0.0, rho_dst[2] / rho_src[2]],
    ];
    let cone_inv = inv(BRADFORD_CONE).ok_or(Error::MissingColorCalibration)?;
    let ca = mat_mul(cone_inv, mat_mul(scale, BRADFORD_CONE));
    Ok(mat_mul(ca, *inv_cm))
}

/// What the dual-illuminant path needs (`SPEC-034`): both `ColorMatrix`es and
/// the reciprocal-temperature blend's constants.
struct DualIlluminant {
    cm1: [[f64; 3]; 3],
    cm2: [[f64; 3]; 3],
    /// `1/T1`.
    inv_t1: f64,
    /// `1/T1 − 1/T2`, never zero.
    denom: f64,
}

/// The dual-illuminant preconditions (`DEC-034`): `Ok(Some(_))` when
/// `ColorMatrix1` **and** `ColorMatrix2` are present, **and** both
/// `CalibrationIlluminant` codes are present and known (in
/// [`ILLUMINANT_CCT`]) with distinct reciprocal temperatures.
///
/// `Ok(None)`, the single-illuminant path, only when a tag is **absent**, a
/// code is unknown, or the temperatures coincide. A **present** `ColorMatrix`
/// that cannot be read is [`Error::MissingColorCalibration`], never the
/// single-illuminant fallback (`DEC-078` §7.2, `SPEC-089`); the sensor reader
/// refuses such a tag first, so this is the second line for a hand-built
/// [`Sensor`].
fn dual_illuminant(sensor: &Sensor) -> Result<Option<DualIlluminant>, Error> {
    let cm1 = sensor
        .color_matrix_1
        .as_deref()
        .map(matrix_from_rationals)
        .transpose()?;
    let cm2 = sensor
        .color_matrix_2
        .as_deref()
        .map(matrix_from_rationals)
        .transpose()?;
    let (Some(cm1), Some(cm2)) = (cm1, cm2) else {
        return Ok(None);
    };
    let known = |code: Option<u32>| code.and_then(cct_of_illuminant);
    let (Some(t1), Some(t2)) = (
        known(sensor.calibration_illuminant_1),
        known(sensor.calibration_illuminant_2),
    ) else {
        return Ok(None);
    };
    let inv_t1 = 1.0 / t1;
    let denom = inv_t1 - 1.0 / t2;
    if denom == 0.0 {
        // `T1 == T2` (or coincident reciprocals): the mired blend is undefined.
        return Ok(None);
    }
    Ok(Some(DualIlluminant {
        cm1,
        cm2,
        inv_t1,
        denom,
    }))
}

/// The D50 chromaticity `NeutralToXY` starts from (`DEC-076` §5).
const D50_XY: (f64, f64) = (0.3457, 0.3585);

/// `NeutralToXY` stops when one pass moves the chromaticity by less than this,
/// `|Δx| + |Δy|` (`DEC-076` §5).
const NEUTRAL_TO_XY_TOLERANCE: f64 = 0.0000001;

/// The most passes `NeutralToXY` runs (`DEC-076` §5).
const NEUTRAL_TO_XY_MAX_PASSES: u32 = 30;

/// The scene interpolation weight `g ∈ [0, 1]` (the weight of illuminant 2) by
/// the DNG `NeutralToXY` method, run to convergence (`DEC-076` §5, which
/// replaces `DEC-034`'s fixed 3 passes), or `None` for the single-illuminant
/// fallback.
///
/// From the D50 chromaticity, each pass takes the McCamy CCT `T` of the
/// current `xy`, the mired weight `g = clamp((1/T1 − 1/T) / (1/T1 − 1/T2), 0, 1)`,
/// and `next = xy(inv(X(g)) · AsShotNeutral)` with `X(g)` the lerp of the folded
/// `AB · CC_i · CM_i`. It stops when `|Δx| + |Δy| < 1e-7`, taking `next`; after
/// 30 passes without converging it takes the midpoint of the last two `xy`. The
/// weight is read at the final `xy`.
///
/// `None` whenever a step is degenerate (a singular `X(g)`, an `xy` with sum
/// ≤ 0, a McCamy zero denominator, a malformed neutral) — never a panic or a
/// NaN. The whole computation is `f64` `+ − × ÷` only (McCamy is a polynomial)
/// and runs once per image, off the per-pixel path.
fn scene_weight(dual: &DualIlluminant, calibration: &Calibration, asn: &[Rational]) -> Option<f64> {
    let neutral = neutral_vector(asn)?;
    let x1 = fold(calibration.k1, dual.cm1);
    let x2 = fold(calibration.k2, dual.cm2);
    let weight_at = |(x, y): (f64, f64)| -> Option<f64> {
        let t = mccamy_cct(x, y)?; // zero denominator → fallback
        Some(((dual.inv_t1 - 1.0 / t) / dual.denom).clamp(0.0, 1.0))
    };

    let mut last = D50_XY;
    for pass in 1..=NEUTRAL_TO_XY_MAX_PASSES {
        let g = weight_at(last)?;
        let inv_x = inv(lerp_matrix(x1, x2, g))?; // singular X(g) → fallback
        let mut next = xyz_to_xy(mul_mat_vec(&inv_x, neutral))?; // sum ≤ 0 → fallback
        if (next.0 - last.0).abs() + (next.1 - last.1).abs() < NEUTRAL_TO_XY_TOLERANCE {
            last = next;
            break;
        }
        if pass == NEUTRAL_TO_XY_MAX_PASSES {
            // Not converged: most likely a two-value oscillation, so settle
            // between the last two estimates.
            next = ((last.0 + next.0) * 0.5, (last.1 + next.1) * 0.5);
        }
        last = next;
    }
    weight_at(last)
}

/// [`scene_weight`] straight from a `Sensor`, as the build finds it: `None`
/// when the dual-illuminant preconditions fail or the calibration is unusable.
#[cfg(test)]
fn interpolation_weight(sensor: &Sensor) -> Option<f64> {
    let dual = dual_illuminant(sensor).ok()??;
    let calibration = resolve_calibration(sensor, true).ok()?;
    scene_weight(&dual, &calibration, sensor.as_shot_neutral.as_ref()?)
}

/// The CCT (kelvin) of a `CalibrationIlluminant` / EXIF `LightSource` code from
/// the committed [`ILLUMINANT_CCT`] table, or `None` for an unrecognised code
/// (which takes the single-illuminant fallback rather than a guessed temperature).
fn cct_of_illuminant(code: u32) -> Option<f64> {
    ILLUMINANT_CCT
        .iter()
        .find(|&&(c, _)| c == code)
        .map(|&(_, t)| t)
}

/// McCamy's cubic CCT from a chromaticity `(x, y)` (`DEC-034`):
/// `n = (x − 0.3320) / (0.1858 − y)`, `T = 449·n³ + 3525·n² + 6823.3·n + 5520.33`.
/// `None` when the denominator `0.1858 − y` is exactly zero (never a divide to
/// `inf`/`NaN`). A polynomial — `n³ = n·n·n`, no `powf` — so it stays `+ − × ÷`
/// only and byte-identical across targets.
fn mccamy_cct(x: f64, y: f64) -> Option<f64> {
    let denom = MCCAMY_EPICENTRE_Y - y;
    if denom == 0.0 {
        return None;
    }
    let n = (x - MCCAMY_EPICENTRE_X) / denom;
    Some(MCCAMY_C3 * n * n * n + MCCAMY_C2 * n * n + MCCAMY_C1 * n + MCCAMY_C0)
}

/// The chromaticity `(x, y) = (X/(X+Y+Z), Y/(X+Y+Z))` of an XYZ triple, or `None`
/// when the sum `X+Y+Z` is non-positive (never a 0-divide).
fn xyz_to_xy(xyz: [f64; 3]) -> Option<(f64, f64)> {
    let [x, y, z] = xyz;
    let sum = x + y + z;
    if sum <= 0.0 {
        return None;
    }
    Some((x / sum, y / sum))
}

/// The element-wise linear interpolation `(1−g)·a + g·b` of two 3×3 matrices,
/// fully unrolled (no indexing). At `g = 0` this is exactly `a` and at `g = 1`
/// exactly `b` (`1.0·x == x`, `0.0·x + x == x` in IEEE), so the endpoints
/// reproduce illuminant 1 / illuminant 2 to the bit.
fn lerp_matrix(a: [[f64; 3]; 3], b: [[f64; 3]; 3], g: f64) -> [[f64; 3]; 3] {
    let [[a00, a01, a02], [a10, a11, a12], [a20, a21, a22]] = a;
    let [[b00, b01, b02], [b10, b11, b12], [b20, b21, b22]] = b;
    let w = 1.0 - g;
    [
        [w * a00 + g * b00, w * a01 + g * b01, w * a02 + g * b02],
        [w * a10 + g * b10, w * a11 + g * b11, w * a12 + g * b12],
        [w * a20 + g * b20, w * a21 + g * b21, w * a22 + g * b22],
    ]
}

/// The matrix-vector product `m · v` of a row-major 3×3 and a 3-vector, each dot
/// summed left to right. Only `f64` `+`/`×` (no FMA), byte-identical across
/// targets (`DEC-033`). Used to map the camera neutral through `inv(CM(g))`.
fn mul_mat_vec(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let [v0, v1, v2] = v;
    [
        m00 * v0 + m01 * v1 + m02 * v2,
        m10 * v0 + m11 * v1 + m12 * v2,
        m20 * v0 + m21 * v1 + m22 * v2,
    ]
}

/// The three `AsShotNeutral` rationals as an `f64` vector `(R, G, B)` — the
/// camera neutral the `NeutralToXY` iteration maps. A length other than 3, or a
/// zero-denominator entry, is `None` (→ the single-illuminant fallback). Unlike
/// [`wb_diagonal`] this does **not** require a positive neutral: the weight
/// iteration only reads the chromaticity, and a non-positive component surfaces
/// later as a degenerate `xy` (sum ≤ 0) → fallback, not here.
fn neutral_vector(asn: &[Rational]) -> Option<[f64; 3]> {
    match asn {
        [r, g, b] => Some([
            rational_to_f64(r).ok()?,
            rational_to_f64(g).ok()?,
            rational_to_f64(b).ok()?,
        ]),
        _ => None,
    }
}

/// The white-balance diagonal `D = diag(1/asn_R, 1/asn_G, 1/asn_B)` from the
/// three positive `AsShotNeutral` rationals. Each diagonal entry is `den/num`
/// (`1 / (num/den)`); a zero numerator or denominator, or a non-positive
/// neutral, is [`Error::MissingColorCalibration`] — never a divide that yields
/// `inf`/`NaN`.
fn wb_diagonal(asn: &[Rational]) -> Result<[[f64; 3]; 3], Error> {
    match asn {
        [r, g, b] => Ok([
            [reciprocal_rational(r)?, 0.0, 0.0],
            [0.0, reciprocal_rational(g)?, 0.0],
            [0.0, 0.0, reciprocal_rational(b)?],
        ]),
        _ => Err(Error::MissingColorCalibration),
    }
}

/// `1 / (num/den) = den/num` for a positive neutral. Requires `num > 0` and
/// `den > 0` (`AsShotNeutral` is a positive rational per DNG); anything else is
/// [`Error::MissingColorCalibration`].
fn reciprocal_rational(r: &Rational) -> Result<f64, Error> {
    if r.num <= 0 || r.den <= 0 {
        return Err(Error::MissingColorCalibration);
    }
    Ok(f64::from(r.den) / f64::from(r.num))
}

/// A row-major 3×3 `f64` matrix from exactly 9 rationals. A length other than 9,
/// or a zero-denominator entry, is [`Error::MissingColorCalibration`] (a matrix
/// that cannot be read is unusable calibration, not a panic).
fn matrix_from_rationals(entries: &[Rational]) -> Result<[[f64; 3]; 3], Error> {
    match entries {
        [a, b, c, d, e, f, g, h, i] => Ok([
            [
                rational_to_f64(a)?,
                rational_to_f64(b)?,
                rational_to_f64(c)?,
            ],
            [
                rational_to_f64(d)?,
                rational_to_f64(e)?,
                rational_to_f64(f)?,
            ],
            [
                rational_to_f64(g)?,
                rational_to_f64(h)?,
                rational_to_f64(i)?,
            ],
        ]),
        _ => Err(Error::MissingColorCalibration),
    }
}

/// `num/den` as `f64`. A zero denominator is [`Error::MissingColorCalibration`]
/// rather than a divide to `inf`/`NaN` (`DEC-030`'s rationals keep `den` verbatim,
/// zero included). `f64::from` is lossless for `i32`.
fn rational_to_f64(r: &Rational) -> Result<f64, Error> {
    if r.den == 0 {
        return Err(Error::MissingColorCalibration);
    }
    Ok(f64::from(r.num) / f64::from(r.den))
}

/// The matrix product `A · B` of two row-major 3×3 `f64` matrices, fully
/// unrolled: `(A·B)[i][j] = Σ_k A[i][k]·B[k][j]`, each dot summed left to right.
/// Only `f64` `+`/`×` (no FMA), so it is byte-identical across targets (`DEC-033`).
fn mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let [[a00, a01, a02], [a10, a11, a12], [a20, a21, a22]] = a;
    let [[b00, b01, b02], [b10, b11, b12], [b20, b21, b22]] = b;
    [
        [
            a00 * b00 + a01 * b10 + a02 * b20,
            a00 * b01 + a01 * b11 + a02 * b21,
            a00 * b02 + a01 * b12 + a02 * b22,
        ],
        [
            a10 * b00 + a11 * b10 + a12 * b20,
            a10 * b01 + a11 * b11 + a12 * b21,
            a10 * b02 + a11 * b12 + a12 * b22,
        ],
        [
            a20 * b00 + a21 * b10 + a22 * b20,
            a20 * b01 + a21 * b11 + a22 * b21,
            a20 * b02 + a21 * b12 + a22 * b22,
        ],
    ]
}

/// The exact 3×3 cofactor inverse in `f64`, or `None` if the matrix is singular
/// (`det == 0.0`). Used for the `ColorMatrix1` path (XYZ→camera inverted to
/// camera→XYZ); a singular `ColorMatrix` surfaces as
/// [`Error::MissingColorCalibration`] in [`build_camera_to_srgb_linear`], never
/// a divide by zero.
fn inv(m: [[f64; 3]; 3]) -> Option<[[f64; 3]; 3]> {
    let [[a, b, c], [d, e, f], [g, h, i]] = m;
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if det == 0.0 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        [
            (e * i - f * h) * inv_det,
            (c * h - b * i) * inv_det,
            (b * f - c * e) * inv_det,
        ],
        [
            (f * g - d * i) * inv_det,
            (a * i - c * g) * inv_det,
            (c * d - a * f) * inv_det,
        ],
        [
            (d * h - e * g) * inv_det,
            (b * g - a * h) * inv_det,
            (a * e - b * d) * inv_det,
        ],
    ])
}

/// Apply a built matrix `m` per pixel over interleaved camera RGB into `dst`
/// (`out_i = Σ_j m[i][j]·cam_j`, then [`quantize`]). The buffers are consumed in
/// lock-step 3-sample chunks, so a length mismatch simply stops at the shorter
/// (the callers validate lengths first); every element written is a clamped,
/// rounded `u16`, never a panic or a NaN.
fn apply_matrix_into(m: &[[f64; 3]; 3], camera_rgb: &[u16], dst: &mut [u16]) {
    let [[m00, m01, m02], [m10, m11, m12], [m20, m21, m22]] = *m;
    let (src_triples, _) = camera_rgb.as_chunks::<3>();
    let (dst_triples, _) = dst.as_chunks_mut::<3>();
    for (&[r, g, b], [out_r, out_g, out_b]) in src_triples.iter().zip(dst_triples.iter_mut()) {
        let rf = f64::from(r);
        let gf = f64::from(g);
        let bf = f64::from(b);
        *out_r = quantize(m00 * rf + m01 * gf + m02 * bf);
        *out_g = quantize(m10 * rf + m11 * gf + m12 * bf);
        *out_b = quantize(m20 * rf + m21 * gf + m22 * bf);
    }
}

/// Clamp a transform result to `[0, 65535]` **before** rounding half up, so the
/// rounding is well-defined and can never leave `u16` range (`DEC-018`): a
/// negative off-gamut mix pins to `0`, an overshoot to `65535`. `(v + 0.5).floor()`
/// on a value already in `[0, 65535]` is in `[0, 65535]`, so the cast cannot
/// truncate — the same `as u16` pattern [`crate::warp`]'s resampler uses.
fn quantize(value: f64) -> u16 {
    let clamped = value.clamp(0.0, 65535.0);
    (clamped + 0.5).floor() as u16
}

/// `OpcodeList2`'s `GainMap`s for this sensor's `ActiveArea`-sized stage 2
/// (`SPEC-059`, `DEC-069` §1), or none. The geometry was already validated by
/// [`check_color_buffers`]; a geometry error here is that same typed error.
fn resolve_colour_gain_maps(sensor: &Sensor) -> Result<GainMaps<'_>, Error> {
    let geometry = resolve_geometry(sensor)?;
    crate::develop::resolve_gain_maps(sensor, geometry.active_width, geometry.active_height)
}

/// `inner` (a level normaliser over raw-plane coordinates) followed by
/// `gains` at stage 2 (`DEC-069` §1): `(x, y)` maps to `(r, c) = (y − top,
/// x − left)` from the `ActiveArea` origin. A read outside the `ActiveArea`
/// (the demosaic's reflected border, or a masked photosite) is not part of
/// stage 2 and is left at its normalised value.
fn gained_normalizer<'a, N>(
    sensor: &Sensor,
    inner: N,
    gains: &'a GainMaps<'a>,
) -> impl Fn(u32, u32) -> u16 + Sync + 'a
where
    N: Fn(u32, u32) -> u16 + Sync + 'a,
{
    let (top, left) = active_area_origin(sensor);
    move |x: u32, y: u32| -> u16 {
        let v = inner(x, y);
        match (y.checked_sub(top), x.checked_sub(left)) {
            // `apply` leaves a sample past the bottom or right edge unchanged.
            (Some(r), Some(c)) => gains.apply(r, c, v),
            _ => v,
        }
    }
}

/// `OpcodeList1`'s first `FixBadPixelsConstant`, resolved once per call for
/// the colour paths (`SPEC-098`, `DEC-086` items 1 to 4), or `None` when the
/// list or the opcode is absent, or when the CFA is not a supported 2×2 Bayer
/// mosaic (the entry then refuses at its own CFA check, which runs before any
/// sample is read).
///
/// Ranks after the levels and the `GainMap`s and before the CFA (`DEC-086`
/// item 7): a malformed first instance, optional or not, is
/// [`Error::MalformedOpcodeParams`] `{ id: 4, .. }` here. `BayerPhase` is
/// parsed (so a malformed block is refused) and not used: the colour comes
/// from `CFAPattern` (item 4).
fn resolve_colour_bad_pixels(sensor: &Sensor) -> Result<Option<CfaBadPixels>, Error> {
    let Some(bytes) = sensor.opcode_list_1.as_deref() else {
        return Ok(None);
    };
    let Some(constant) = crate::opcode::parse_fix_bad_pixels_constant(bytes)? else {
        return Ok(None);
    };
    Ok(CfaBadPixels::new(sensor, constant))
}

/// The colour paths' `FixBadPixelsConstant` kernel (`DEC-086` item 3), applied
/// at the raw read of [`slot_normalizer`] and [`pattern_normalizer`], the only
/// two places the colour path reads `src` (item 2). A pure function of `src`:
/// it reads only unfixed values and writes nothing back, so every traversal
/// (the fused one, the bands and their halos, the threads, the developed mix's
/// strips) sees the same fixed value, and no visiting order changes it.
/// `Copy`, a few words: resolved once, never per pixel.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CfaBadPixels {
    /// `Constant`, compared with each raw sample as a `u32`: one above 65535
    /// matches nothing.
    constant: u32,
    /// The plane's size: a neighbour counts anywhere inside it, the masked
    /// border included (`SPEC-017` rule 1).
    width: u32,
    height: u32,
    /// The `ActiveArea`, clamped to the plane: only a photosite inside it is a
    /// candidate, and its colour is indexed from its origin (`DEC-040`).
    top: u32,
    left: u32,
    bottom: u32,
    right: u32,
    /// Whether each 2×2 slot `(row & 1)·2 + (col & 1)` from the `ActiveArea`
    /// origin is green in `CFAPattern`.
    green: [bool; 4],
}

impl CfaBadPixels {
    /// The kernel for `constant` on `sensor`'s CFA, or `None` when the CFA is
    /// not a supported 2×2 Bayer mosaic.
    fn new(sensor: &Sensor, constant: u32) -> Option<CfaBadPixels> {
        if !is_supported_bayer(sensor) {
            return None;
        }
        let cfa = sensor.color_filter.as_ref()?;
        let mut green = [false; 4];
        for (slot, &colour) in green.iter_mut().zip(cfa.pattern.iter()) {
            *slot = colour == 1;
        }
        let (top, left) = active_area_origin(sensor);
        let (bottom, right) = match sensor.active_area {
            Some(a) => (a.bottom.min(sensor.height), a.right.min(sensor.width)),
            None => (sensor.height, sensor.width),
        };
        Some(CfaBadPixels {
            constant,
            width: sensor.width,
            height: sensor.height,
            top,
            left,
            bottom,
            right,
            green,
        })
    }

    /// The fixed value of the raw sample `v` at `(x, y)`. Not a candidate
    /// (`v` returned) unless `v` equals `Constant` and `(x, y)` is inside the
    /// `ActiveArea`. A green candidate reads its four diagonals, a red or blue
    /// one the four photosites two away on the axes; each neighbour inside the
    /// plane whose **unfixed** value is not `Constant` counts. The result is
    /// `⌊(t + ⌊n/2⌋) / n⌋` for `n` neighbours summing to `t` (the mean rounded
    /// half up), or `v` when `n` is 0. Checked arithmetic only: `n ≤ 4` and
    /// `t ≤ 4·65535`.
    #[inline]
    fn sample(&self, src: &[u16], x: u32, y: u32, v: u16) -> u16 {
        if u32::from(v) != self.constant
            || y < self.top
            || y >= self.bottom
            || x < self.left
            || x >= self.right
        {
            return v;
        }
        let row_odd = y.wrapping_sub(self.top) & 1 == 1;
        let col_odd = x.wrapping_sub(self.left) & 1 == 1;
        let slot = match (row_odd, col_odd) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => 3,
        };
        let green = self.green.get(slot).copied().unwrap_or(false);
        let offsets: [(i64, i64); 4] = if green {
            [(-1, -1), (1, -1), (-1, 1), (1, 1)]
        } else {
            [(0, -2), (-2, 0), (2, 0), (0, 2)]
        };
        let width = usize::try_from(self.width).unwrap_or(0);
        let (mut n, mut t) = (0u32, 0u32);
        for (dx, dy) in offsets {
            let nx = i64::from(x).checked_add(dx);
            let ny = i64::from(y).checked_add(dy);
            let (Some(nx), Some(ny)) = (nx, ny) else {
                continue;
            };
            let (Ok(nx), Ok(ny)) = (u32::try_from(nx), u32::try_from(ny)) else {
                continue;
            };
            if nx >= self.width || ny >= self.height {
                continue;
            }
            let index = usize::try_from(ny)
                .ok()
                .and_then(|row| row.checked_mul(width))
                .and_then(|base| base.checked_add(usize::try_from(nx).ok()?));
            let Some(&neighbour) = index.and_then(|i| src.get(i)) else {
                continue;
            };
            if u32::from(neighbour) != self.constant {
                n = n.saturating_add(1);
                t = t.saturating_add(u32::from(neighbour));
            }
        }
        match t.checked_add(n / 2).and_then(|sum| sum.checked_div(n)) {
            Some(mean) => u16::try_from(mean).unwrap_or(v),
            None => v,
        }
    }
}

/// The colour path's resolved levels, in one of two shapes (`SPEC-075`,
/// `DEC-068` §2). [`resolve_colour_levels`] resolves them once per develop;
/// each entry point then matches once, builds that shape's normalizer and runs
/// a body generic over it, so no per-pixel branch picks the shape (the mono
/// path's `Blacks` does the same).
#[derive(Clone, Copy)]
enum ColourLevels<'a> {
    /// `BlackLevelRepeatDim` rows and columns each 1 or 2, or no per-position
    /// black: the four `(row_odd, col_odd)` slots' `(black, range)`, for
    /// [`slot_normalizer`]. Every held file. `range` is `W − B`, the same in
    /// every slot (`DEC-070` §1).
    Slots([(u32, u32); 4]),
    /// A larger pattern and its range `W − B`, for [`pattern_normalizer`]'s
    /// per-photosite lookup.
    Pattern(BlackPattern<'a>, u32),
}

/// A black pattern too large for the four-slot table (`DEC-068` §2): `values`
/// is `rows × cols` long (checked by [`black_pattern_dims`]), and `(top, left)`
/// is the `ActiveArea` origin it repeats from (`DEC-040`).
#[derive(Clone, Copy)]
struct BlackPattern<'a> {
    values: &'a [u32],
    rows: u32,
    cols: u32,
    top: u32,
    left: u32,
}

impl BlackPattern<'_> {
    /// The black at raw `(x, y)`, `values[pattern_index(..)]`. The fallback 0
    /// is unreachable: the index is `< rows × cols == values.len()`.
    fn at(&self, x: u32, y: u32) -> u32 {
        pattern_index(x, y, (self.top, self.left), [self.rows, self.cols])
            .and_then(|i| self.values.get(i))
            .copied()
            .unwrap_or(0)
    }
}

/// The `BlackLevel` index of pattern cell `(row, col)` for a `[rows, cols]`
/// repeat (`DEC-061` §3): `(row mod rows) × cols + (col mod cols)`. `None`
/// only for a zero dimension, which [`black_pattern_dims`] refuses first.
fn pattern_cell(row: u32, col: u32, [rows, cols]: [u32; 2]) -> Option<usize> {
    let r = row.checked_rem(rows)?;
    let c = col.checked_rem(cols)?;
    let index = r.checked_mul(cols)?.checked_add(c)?;
    usize::try_from(index).ok()
}

/// The `BlackLevel` index of the photosite at raw `(x, y)`, for a `[rows, cols]`
/// pattern repeating from the `ActiveArea` origin `(top, left)` (`DEC-061` §3,
/// `DEC-068` §2): [`pattern_cell`] of `(y − top) mod rows`, `(x − left) mod
/// cols`. The offsets are taken mod the repeat in `i64` with a Euclidean
/// remainder, so a photosite above or left of the origin (a reflected border
/// read) still lands on its true cell.
fn pattern_index(x: u32, y: u32, (top, left): (u32, u32), dims: [u32; 2]) -> Option<usize> {
    let [rows, cols] = dims;
    let row = i64::from(y)
        .checked_sub(i64::from(top))?
        .checked_rem_euclid(i64::from(rows))?;
    let col = i64::from(x)
        .checked_sub(i64::from(left))?
        .checked_rem_euclid(i64::from(cols))?;
    pattern_cell(u32::try_from(row).ok()?, u32::try_from(col).ok()?, dims)
}

/// Resolve the colour develop's levels once (`SPEC-049`, `DEC-042`; the black
/// rule is `SPEC-075`, `DEC-068` §2).
///
/// - **White**, for every photosite (`DEC-061` §3): the first `white_levels`
///   value, else `white_level`, else `2^BitsPerSample − 1`.
/// - **Black:** `[R, C]` is `BlackLevelRepeatDim`, or `[1, 1]`. An empty
///   `black_levels` gives every photosite `black_level`, else 0. Otherwise
///   `black_levels.len()` must be `R × C`, or this is
///   [`Error::InvalidBlackLevelPattern`] (the check is
///   [`black_pattern_dims`], shared with the mono path). The black at raw
///   `(x, y)` is `black_levels[((y − top) mod R) × C + ((x − left) mod C)]`
///   from the `ActiveArea` origin.
///
/// When `R` and `C` are each 1 or 2, every photosite's black depends only on
/// the parity of its offset from the origin, so the four `(row_odd, col_odd)`
/// slots are filled from that rule and [`slot_normalizer`] runs, the fused
/// path's existing closure. A larger pattern takes [`pattern_normalizer`].
///
/// **The scale (`DEC-070`):** every photosite scales by `W − B`, with `B` the
/// maximum of the blacks the pattern applies, as the DNG specification's
/// Rescaling step defines; each subtracts its own black first. With one black
/// everywhere `B` is that black and the map is `DEC-018`'s.
///
/// **The refusal (`DEC-070` §3, second amendment; `SPEC-080`):** on every
/// branch (the scalar black, the slots, the general pattern), `W ≤ B` is
/// [`Error::InvalidLevels`] reporting `B` and `W`, as the mono path refuses
/// it. Before `SPEC-080` such a plane developed to all zeros.
///
/// Returns before anything is written, so every colour entry point refuses a
/// bad pattern with `dst` untouched.
fn resolve_colour_levels(sensor: &Sensor) -> Result<ColourLevels<'_>, Error> {
    let white_default = max_value_for_bits(sensor.bits_per_sample).unwrap_or(u32::from(u16::MAX));
    let white = sensor
        .white_levels
        .first()
        .copied()
        .or(sensor.white_level)
        .unwrap_or(white_default);
    let values = sensor.black_levels.as_slice();
    if values.is_empty() {
        let black = sensor.black_level.unwrap_or(0);
        let range = colour_range(white, black)?;
        return Ok(ColourLevels::Slots([(black, range); 4]));
    }
    let dims = black_pattern_dims(sensor)?;
    let [rows, cols] = dims;
    if rows <= 2 && cols <= 2 {
        // Slot order is `(row_odd, col_odd)` = (0,0), (0,1), (1,0), (1,1).
        const PARITIES: [(u32, u32); 4] = [(0, 0), (0, 1), (1, 0), (1, 1)];
        let mut blacks = [0u32; 4];
        for (slot, (row_odd, col_odd)) in blacks.iter_mut().zip(PARITIES) {
            *slot = pattern_cell(row_odd, col_odd, dims)
                .and_then(|i| values.get(i))
                .copied()
                .unwrap_or(0);
        }
        // `B` over the blacks the slots apply (`DEC-070` §1): a `[1, 2]` or
        // `[2, 1]` pattern fills all four slots from its two values.
        let max_black = blacks.iter().copied().max().unwrap_or(0);
        let range = colour_range(white, max_black)?;
        return Ok(ColourLevels::Slots(blacks.map(|black| (black, range))));
    }
    // `values` is non-empty here, so the fallback is never taken.
    let max_black = values.iter().copied().max().unwrap_or(0);
    let range = colour_range(white, max_black)?;
    let (top, left) = active_area_origin(sensor);
    Ok(ColourLevels::Pattern(
        BlackPattern {
            values,
            rows,
            cols,
            top,
            left,
        },
        range,
    ))
}

/// The colour plane's range `W − B` (`DEC-070` §1), or
/// [`Error::InvalidLevels`] reporting `B` and `W` when `W ≤ B` (`DEC-070` §3,
/// second amendment), as the mono path's `resolve_levels_with` refuses it.
/// `max_black` is the maximum black the branch applies, never its first.
fn colour_range(white: u32, max_black: u32) -> Result<u32, Error> {
    if white <= max_black {
        return Err(Error::InvalidLevels {
            black_level: max_black,
            white_level: white,
        });
    }
    Ok(white.saturating_sub(max_black))
}

/// The per-cell level-normalize map the fused develop path
/// ([`develop_color_fused`]) fills its rolling row cache from, for a black
/// pattern that fits the four 2×2 slots ([`resolve_colour_levels`]). Given a mosaic
/// cell `(col, row)` — already reflected into range by the demosaic logic — it
/// returns the level-normalized value for that cell: `[black, white] → [0,
/// 65535]` by the cell's slot in `levels`: its own black subtracted, scaled by
/// the plane's `W − B` and clamped per `DEC-018` and `DEC-070`.
///
/// The fuse calls it **once per row** as that row enters the rolling cache (not
/// a full scratch plane, and not re-run per neighbour read), so each sample is
/// normalised exactly once. The four slot `(black, white)` pairs were resolved
/// once by [`resolve_colour_levels`]; this closure does no per-read division to find
/// its slot.
///
/// The 2×2 slot is taken **relative to the `ActiveArea` origin** (`DEC-040`): in
/// DNG the per-slot level pattern starts at the active area, not the plane's
/// top-left, so a file with an odd `ActiveArea.top`/`.left` would otherwise
/// subtract the wrong black at every photosite. The offset form
/// `(row − active_top) & 1` is computed with `wrapping_sub`, panic-free and
/// parity-correct even above/left of the origin (the low bit of a wrapping
/// subtraction is still the true parity); for an even origin — the whole existing
/// corpus — it reduces to `row & 1`, so the goldens are unchanged. The demosaic
/// uses the same origin for the CFA phase.
fn slot_normalizer<'a>(
    sensor: &'a Sensor,
    src: &'a [u16],
    levels: [(u32, u32); 4],
    fix: Option<CfaBadPixels>,
) -> impl Fn(u32, u32) -> u16 + Sync + 'a {
    let width = usize::try_from(sensor.width).unwrap_or(0);
    let (active_top, active_left) = active_area_origin(sensor);
    let active_top = usize::try_from(active_top).unwrap_or(0);
    let active_left = usize::try_from(active_left).unwrap_or(0);

    move |x: u32, y: u32| -> u16 {
        let col = usize::try_from(x).unwrap_or(0);
        let row = usize::try_from(y).unwrap_or(0);
        let row_odd = (row.wrapping_sub(active_top) & 1) == 1;
        let col_odd = (col.wrapping_sub(active_left) & 1) == 1;
        let pos = match (row_odd, col_odd) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => 3,
        };
        let (black, range) = levels.get(pos).copied().unwrap_or((0, u32::from(u16::MAX)));
        let index = row.checked_mul(width).and_then(|rb| rb.checked_add(col));
        let sample = index.and_then(|i| src.get(i)).copied().unwrap_or(0);
        // `DEC-086` item 2: the fix runs on the raw sample, before the levels.
        let sample = fix.map_or(sample, |f| f.sample(src, x, y, sample));
        normalize_range(sample, black, range)
    }
}

/// [`slot_normalizer`] for a black pattern larger than 2×2 (`DEC-068` §2): the
/// black is looked up per photosite through [`BlackPattern::at`], with no
/// allocation, and `range` is the plane's one `W − B` (`DEC-070`).
fn pattern_normalizer<'a>(
    sensor: &'a Sensor,
    src: &'a [u16],
    pattern: BlackPattern<'a>,
    range: u32,
    fix: Option<CfaBadPixels>,
) -> impl Fn(u32, u32) -> u16 + Sync + 'a {
    let width = usize::try_from(sensor.width).unwrap_or(0);
    move |x: u32, y: u32| -> u16 {
        let col = usize::try_from(x).unwrap_or(0);
        let row = usize::try_from(y).unwrap_or(0);
        let black = pattern.at(x, y);
        let index = row.checked_mul(width).and_then(|rb| rb.checked_add(col));
        let sample = index.and_then(|i| src.get(i)).copied().unwrap_or(0);
        // `DEC-086` item 2: the fix runs on the raw sample, before the levels.
        let sample = fix.map_or(sample, |f| f.sample(src, x, y, sample));
        normalize_range(sample, black, range)
    }
}

/// Map one raw sample through `[black, white] → [0, u16::MAX]`, round to
/// nearest, clamped (`DEC-018`) — the per-channel analogue of
/// [`crate::develop`]'s scalar `normalize`, made total: if `black > white`
/// (malformed) or `black == white` the result is `0` rather than a panic.
/// The develop runs [`normalize_range`] with `W − B` (`DEC-070`); with one
/// black everywhere that is this map exactly, which the unit tests pin.
#[cfg(test)]
fn normalize_sample(sample: u16, black: u32, white: u32) -> u16 {
    let value = u32::from(sample);
    // Clamp without panicking even if `black > white` (`u32::clamp` requires
    // `min <= max`): the normal path has `black <= white` and this is identical.
    let clamped = value.clamp(black.min(white), black.max(white));
    let numerator = u64::from(clamped.saturating_sub(black)).saturating_mul(u64::from(u16::MAX));
    let denominator = u64::from(white.saturating_sub(black));
    let half = denominator.checked_div(2).unwrap_or(0);
    let scaled = numerator
        .checked_add(half)
        .and_then(|v| v.checked_div(denominator))
        .unwrap_or(0);
    u16::try_from(scaled).unwrap_or(u16::MAX)
}

/// `2^bits - 1`, without the panicking `pow`/shift the crate-root lint forbids;
/// `None` if `2^bits` does not fit `u32` (mirrors `develop::max_value_for_bits`).
fn max_value_for_bits(bits: u32) -> Option<u32> {
    2u32.checked_pow(bits)?.checked_sub(1)
}

#[cfg(test)]
#[allow(
    // Unit tests are not a decode path exercised on untrusted input
    // (AGENTS.md §11); the same five lints are allowed here as in the other
    // modules' test blocks and in src/bin/irr.rs.
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::*;

    /// A 3×3 identity, for round-trip / neutral assertions.
    const IDENTITY: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    /// Apply a matrix to a single camera triple, returning the quantized RGB —
    /// a one-pixel wrapper over `apply_matrix_into`.
    fn apply_one(m: &[[f64; 3]; 3], rgb: [u16; 3]) -> [u16; 3] {
        let mut out = [0u16; 3];
        apply_matrix_into(m, &rgb, &mut out);
        out
    }

    #[test]
    fn apply_matrix_is_bit_exact_for_injected_matrices() {
        // Identity passes the camera triple through unchanged.
        assert_eq!(apply_one(&IDENTITY, [100, 200, 300]), [100, 200, 300]);

        // A diagonal of 0.5 halves each channel, with an exact round-half-up
        // tie: 2001·0.5 = 1000.5 → 1001, 2003·0.5 = 1001.5 → 1002, 5·0.5 = 2.5 → 3.
        let half = [[0.5, 0.0, 0.0], [0.0, 0.5, 0.0], [0.0, 0.0, 0.5]];
        assert_eq!(apply_one(&half, [2001, 2003, 5]), [1001, 1002, 3]);

        // A negative entry can drive a channel below zero → clamps to 0.
        // Row0 = R - G = 100 - 150 = -50 → 0; rows 1/2 are the identity.
        let negative = [[1.0, -1.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        assert_eq!(apply_one(&negative, [100, 150, 7]), [0, 150, 7]);

        // An overshoot clamps to 65535: Row0 = 2·R = 2·40000 = 80000 → 65535.
        let overshoot = [[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        assert_eq!(apply_one(&overshoot, [40000, 10, 20]), [65535, 10, 20]);
    }

    #[test]
    fn quantize_clamps_before_rounding() {
        assert_eq!(quantize(-1.0), 0); // negative pins to 0
        assert_eq!(quantize(-0.4), 0);
        assert_eq!(quantize(0.5), 1); // half rounds up
        assert_eq!(quantize(0.4999), 0);
        assert_eq!(quantize(2.5), 3);
        assert_eq!(quantize(2.4999), 2);
        assert_eq!(quantize(65535.0), 65535);
        assert_eq!(quantize(65535.5), 65535); // clamp before +0.5, never 65536
        assert_eq!(quantize(70000.0), 65535); // overshoot pins to 65535
    }

    #[test]
    fn inv_3x3_round_trips_and_flags_singular() {
        // An integer upper-triangular matrix with det = 1: its inverse is the
        // integer matrix [[1,-2,5],[0,1,-4],[0,0,1]], exactly representable, so
        // inv(CM)·CM == I to the bit.
        let cm = [[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [0.0, 0.0, 1.0]];
        let inverse = inv(cm).expect("non-singular");
        assert_eq!(
            inverse,
            [[1.0, -2.0, 5.0], [0.0, 1.0, -4.0], [0.0, 0.0, 1.0]]
        );
        assert_eq!(mat_mul(inverse, cm), IDENTITY);

        // A singular matrix (row 1 = 2·row 0) has det 0 → None.
        let singular = [[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [0.0, 0.0, 1.0]];
        assert!(inv(singular).is_none());
    }

    #[test]
    fn build_matrix_matches_independent_rederivation() {
        // The builder must assemble exactly M = C · cam2xyz, C = S · B, with
        // cam2xyz = FM1·D (forward path) or inv(CM1)·D (matrix path). Re-derive
        // each independently and compare to the bit.

        // Forward path: a synthetic FM1 and asn = (1/2, 1/1, 1/2).
        let fm1_entries = matrix_rationals([
            [60000, 30000, 6422],
            [20000, 70000, 10000],
            [10000, 10000, 62521],
        ]);
        let asn = vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ];
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(fm1_entries.clone()),
            color_matrix_1: None,
            as_shot_neutral: Some(asn.clone()),
        });
        let fm = reference_matrix(&fm1_entries);
        let wb = reference_wb(&asn);
        let c = reference_mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
        let expected = reference_mat_mul(c, reference_mat_mul(fm, wb));
        assert_eq!(build_camera_to_srgb_linear(&sensor).unwrap(), expected);

        // Matrix path: an invertible CM1 and the same asn, no ForwardMatrix.
        let cm1_entries = matrix_rationals([[10000, 2000, 3000], [0, 10000, 4000], [0, 0, 10000]]);
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: Some(cm1_entries.clone()),
            as_shot_neutral: Some(asn.clone()),
        });
        // cam2xyz = CA(W→D50) · inv(CM1) with W = inv(CM1)·asn — no `D` (it would
        // double-count white balance). Re-derive CA by explicit Bradford steps.
        let inv_cm = inv(reference_matrix(&cm1_entries)).expect("invertible");
        let asn_v = [0.5, 1.0, 0.5];
        let w = ref_mul_vec(inv_cm, asn_v);
        let (rs, rd) = (
            ref_mul_vec(BRADFORD_CONE, w),
            ref_mul_vec(BRADFORD_CONE, D50_WHITE),
        );
        let scale = [
            [rd[0] / rs[0], 0.0, 0.0],
            [0.0, rd[1] / rs[1], 0.0],
            [0.0, 0.0, rd[2] / rs[2]],
        ];
        let ca = reference_mat_mul(
            inv(BRADFORD_CONE).expect("cone invertible"),
            reference_mat_mul(scale, BRADFORD_CONE),
        );
        let expected = reference_mat_mul(c, reference_mat_mul(ca, inv_cm));
        let built = build_camera_to_srgb_linear(&sensor).unwrap();
        assert_eq!(built, expected);
        // Defining property: a raw neutral develops to linear-sRGB (1, 1, 1).
        for v in ref_mul_vec(built, asn_v) {
            assert!((v - 1.0).abs() < 1e-6, "grey must map to white, got {v}");
        }
    }

    #[test]
    fn missing_or_unusable_calibration_is_a_typed_error() {
        let asn = vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ];

        // No ForwardMatrix and no ColorMatrix.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral: Some(asn.clone()),
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // No AsShotNeutral.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(matrix_rationals([[1, 1, 1]; 3])),
            color_matrix_1: None,
            as_shot_neutral: None,
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // A zero-denominator neutral.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(matrix_rationals([[1, 1, 1]; 3])),
            color_matrix_1: None,
            as_shot_neutral: Some(vec![
                Rational { num: 1, den: 0 },
                Rational { num: 1, den: 1 },
                Rational { num: 1, den: 1 },
            ]),
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // A singular ColorMatrix.
        let sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: Some(matrix_rationals([
                [10000, 20000, 30000],
                [20000, 40000, 60000],
                [0, 0, 10000],
            ])),
            as_shot_neutral: Some(asn),
        });
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));
    }

    // ── test-only reference helpers (independent of the shipped code path) ──

    /// The colour fields a `Sensor` fixture needs for the matrix builder.
    struct ColorMeta {
        forward_matrix_1: Option<Vec<Rational>>,
        color_matrix_1: Option<Vec<Rational>>,
        as_shot_neutral: Option<Vec<Rational>>,
    }

    /// A `Sensor` carrying only colour metadata (everything else absent/empty).
    fn color_sensor(meta: ColorMeta) -> Sensor {
        use crate::ifd::Compression;
        Sensor {
            ifd_index: 0,
            width: 2,
            height: 2,
            bits_per_sample: 16,
            samples_per_pixel: 1,
            photometric: 32803,
            compression: Compression::Uncompressed,
            rows_per_strip: None,
            strip_offsets: vec![],
            strip_byte_counts: vec![],
            tile_width: None,
            tile_length: None,
            tile_offsets: vec![],
            tile_byte_counts: vec![],
            black_level: None,
            white_level: None,
            black_level_repeat_dim: None,
            active_area: None,
            masked_areas: vec![],
            default_crop_origin: None,
            default_crop_size: None,
            orientation: None,
            opcode_lists: [false, false, false],
            opcode_list_1: None,
            opcode_list_2: None,
            opcode_list_3: None,
            color_filter: None,
            color_matrix_1: meta.color_matrix_1,
            color_matrix_2: None,
            forward_matrix_1: meta.forward_matrix_1,
            forward_matrix_2: None,
            as_shot_neutral: meta.as_shot_neutral,
            calibration_illuminant_1: None,
            calibration_illuminant_2: None,
            camera_calibration_1: None,
            camera_calibration_2: None,
            analog_balance: None,
            camera_calibration_signature: None,
            profile_calibration_signature: None,
            black_levels: vec![],
            black_levels_exact: vec![],
            white_levels: vec![],
            dng_version: None,
            dng_backward_version: None,
            sample_format: None,
            linearization_table: None,
            black_level_delta_h: None,
            black_level_delta_v: None,
            sub_tile_block_size: None,
            row_interleave_factor: None,
            column_interleave_factor: None,
            malformed_tags: vec![],
        }
    }

    /// Nine `num/10000` rationals (row-major) from integer numerators.
    fn matrix_rationals(rows: [[i32; 3]; 3]) -> Vec<Rational> {
        let mut out = Vec::with_capacity(9);
        for row in rows {
            for num in row {
                out.push(Rational { num, den: 10000 });
            }
        }
        out
    }

    /// A reference 3×3 `f64` matrix from 9 rationals — a triple-loop, written
    /// separately from `matrix_from_rationals`.
    fn reference_matrix(entries: &[Rational]) -> [[f64; 3]; 3] {
        let mut m = [[0.0f64; 3]; 3];
        for (idx, r) in entries.iter().enumerate() {
            m[idx / 3][idx % 3] = f64::from(r.num) / f64::from(r.den);
        }
        m
    }

    /// A reference WB diagonal from the three neutral rationals.
    fn reference_wb(asn: &[Rational]) -> [[f64; 3]; 3] {
        let mut d = [[0.0f64; 3]; 3];
        for (idx, r) in asn.iter().enumerate().take(3) {
            d[idx][idx] = f64::from(r.den) / f64::from(r.num);
        }
        d
    }

    /// A reference matrix product via the standard i/j/k triple loop with a
    /// zero accumulator. `0.0 + x == x` in IEEE, so this matches
    /// `mat_mul`'s left-to-right unrolled sum to the bit.
    fn reference_mat_mul(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut out = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0f64;
                for k in 0..3 {
                    sum += a[i][k] * b[k][j];
                }
                out[i][j] = sum;
            }
        }
        out
    }

    #[test]
    fn reference_mat_mul_agrees_with_mat_mul() {
        // The independent reference must match the shipped `mat_mul` to the bit,
        // so the builder test's expected values are a real cross-check.
        let a = SRGB_FROM_XYZ_D65;
        let b = BRADFORD_D50_TO_D65;
        assert_eq!(reference_mat_mul(a, b), mat_mul(a, b));
    }

    /// `SPEC-075` AC5: the general path's per-photosite index, against a table
    /// written out by hand, for a `[3, 2]` pattern (3 rows, 2 columns) from
    /// the odd origin top 1, left 3. A 3-by-2 pattern has no 2×2 form, so only
    /// the general path can serve it.
    #[test]
    fn general_black_pattern_index_matches_hand_table() {
        // Row y has pattern row (y − 1) mod 3 = 2, 0, 1, 2, 0, 1; column x has
        // pattern column (x − 3) mod 2 = 1, 0, 1, 0. Index = row × 2 + col.
        #[rustfmt::skip]
        const TABLE: [[usize; 4]; 6] = [
            [5, 4, 5, 4],
            [1, 0, 1, 0],
            [3, 2, 3, 2],
            [5, 4, 5, 4],
            [1, 0, 1, 0],
            [3, 2, 3, 2],
        ];
        for (y, row) in TABLE.iter().enumerate() {
            for (x, &want) in row.iter().enumerate() {
                let got = pattern_index(x as u32, y as u32, (1, 3), [3, 2]);
                assert_eq!(got, Some(want), "(x {x}, y {y})");
            }
        }

        // The same through `resolve_colour_levels`: a [3, 2] pattern takes the
        // general path, and each photosite gets its value's black. Each sample
        // is its own black + 1000, so with white 65535 the output is
        // 1000 × 65535 / (65535 − black), rounded half up: a wrong black shows.
        let mut sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral: None,
        });
        sensor.width = 4;
        sensor.height = 6;
        sensor.white_level = Some(65_535);
        sensor.black_levels = vec![100, 200, 300, 400, 500, 600];
        sensor.black_level = Some(100);
        sensor.black_level_repeat_dim = Some([3, 2]);
        sensor.active_area = Some(crate::ifd::ActiveArea {
            top: 1,
            left: 3,
            bottom: 6,
            right: 4,
        });
        let blacks = [100u16, 200, 300, 400, 500, 600];
        let mut src = vec![0u16; 24];
        for (y, row) in TABLE.iter().enumerate() {
            for (x, &i) in row.iter().enumerate() {
                src[y * 4 + x] = blacks[i] + 1000;
            }
        }
        let Ok(ColourLevels::Pattern(pattern, range)) = resolve_colour_levels(&sensor) else {
            panic!("a [3, 2] pattern must take the general path");
        };
        let normalize = pattern_normalizer(&sensor, &src, pattern, range, None);
        // `DEC-070`: every photosite subtracts its own black but scales by
        // white minus the fixture's largest black, 65535 − 600.
        let max_black = u64::from(*blacks.iter().max().unwrap());
        for (y, row) in TABLE.iter().enumerate() {
            for (x, &i) in row.iter().enumerate() {
                let range = 65_535 - max_black;
                // The sample less the hand table's black for this photosite:
                // 1000, but only if the general path indexed the same cell.
                let shifted = u64::from(src[y * 4 + x]) - u64::from(blacks[i]);
                let want = (shifted * 65_535 + range / 2) / range;
                let got = normalize(x as u32, y as u32);
                assert_eq!(u64::from(got), want, "(x {x}, y {y})");
            }
        }
    }

    /// The 2×2 fast path fills its slots from the same rule: `[1, 2]` and
    /// `[2, 1]` are each one slot away from what the pre-`SPEC-075` slot
    /// lookup gave (`DEC-068` §2).
    #[test]
    fn small_patterns_take_the_slot_path() {
        let mut sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral: None,
        });
        sensor.black_levels = vec![7, 9];
        for repeat in [[1, 2], [2, 1], [1, 1], [2, 2]] {
            sensor.black_level_repeat_dim = Some(repeat);
            let result = resolve_colour_levels(&sensor);
            match repeat {
                [1, 1] | [2, 2] => assert!(
                    matches!(result, Err(Error::InvalidBlackLevelPattern { .. })),
                    "{repeat:?} with 2 values"
                ),
                _ => assert!(matches!(result, Ok(ColourLevels::Slots(_))), "{repeat:?}"),
            }
        }
    }

    #[test]
    fn normalize_sample_matches_the_scalar_affine_map() {
        // black 100, white 4100 → span 4000; 2100 is the midpoint → 32768.
        assert_eq!(normalize_sample(2100, 100, 4100), 32768);
        assert_eq!(normalize_sample(100, 100, 4100), 0); // at black
        assert_eq!(normalize_sample(4100, 100, 4100), 65535); // at white
        assert_eq!(normalize_sample(50, 100, 4100), 0); // below black clamps
        assert_eq!(normalize_sample(9000, 100, 4100), 65535); // above white clamps
        assert_eq!(normalize_sample(500, 100, 100), 0); // degenerate span, no panic
    }

    /// `SPEC-078` AC1 on the colour normaliser (`DEC-070`): blacks `[2047,
    /// 2047, 2047, 2048]` on `[2, 2]`, white 15000, so `B = 2048` and `W − B =
    /// 12952`. Each `(x, y, sample, black, DEC-070)` row is placed at a
    /// photosite whose slot holds that black; the arithmetic is
    /// `tests/black_normalisation.rs`'s `AC1_ROWS`:
    /// `⌊clamp(v − b, 0, 12952) · 65535 / 12952 + ½⌋`.
    const AC1_COLOUR: [(u32, u32, u16, u32, u16); 6] = [
        (0, 0, 8523, 2047, 32768),  // 6476 of 12952 (today 32765)
        (1, 1, 8524, 2048, 32768),  // 6476 of 12952 on both rules
        (2, 0, 4000, 2047, 9882),   // 1953 of 12952 (today 9881)
        (1, 0, 14999, 2047, 65535), // 12952 of 12952 (today 65530)
        (0, 1, 15000, 2047, 65535), // 12953, clipped at 12952
        (3, 0, 2000, 2047, 0),      // below black
    ];

    /// The AC1 sensor: 4×2, RGGB, the AC1 levels on `[2, 2]` (no
    /// `ActiveArea`, so slot 3 is odd row, odd column), or `blacks` if given.
    fn ac1_colour_sensor(blacks: &[u32]) -> (Sensor, Vec<u16>) {
        let mut sensor = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral: None,
        });
        sensor.width = 4;
        sensor.height = 2;
        sensor.white_level = Some(15_000);
        sensor.black_levels = blacks.to_vec();
        sensor.black_level_repeat_dim = Some([2, 2]);
        let mut src = vec![5000u16; 8];
        for (x, y, sample, _, _) in AC1_COLOUR {
            src[(y * 4 + x) as usize] = sample;
        }
        (sensor, src)
    }

    #[test]
    fn colour_slots_scale_by_white_minus_the_maximum_black() {
        let (sensor, src) = ac1_colour_sensor(&[2047, 2047, 2047, 2048]);
        let Ok(ColourLevels::Slots(levels)) = resolve_colour_levels(&sensor) else {
            panic!("a [2, 2] pattern takes the slot path");
        };
        assert_eq!(
            levels,
            [(2047, 12952), (2047, 12952), (2047, 12952), (2048, 12952)],
            "every slot scales by W − B"
        );
        let normalize = slot_normalizer(&sensor, &src, levels, None);
        for (x, y, sample, black, want) in AC1_COLOUR {
            assert_eq!(
                normalize(x, y),
                want,
                "sample {sample} at black {black} ({x}, {y})"
            );
        }
    }

    #[test]
    fn colour_general_pattern_scales_by_white_minus_the_maximum_black() {
        // The same 2×2 tiled as a 4×4 pattern takes the general path.
        #[rustfmt::skip]
        let tiled = [
            2047, 2047, 2047, 2047,
            2047, 2048, 2047, 2048,
            2047, 2047, 2047, 2047,
            2047, 2048, 2047, 2048,
        ];
        let (mut sensor, src) = ac1_colour_sensor(&tiled);
        sensor.black_level_repeat_dim = Some([4, 4]);
        let Ok(ColourLevels::Pattern(pattern, range)) = resolve_colour_levels(&sensor) else {
            panic!("a [4, 4] pattern takes the general path");
        };
        assert_eq!(range, 12952, "W − B");
        let normalize = pattern_normalizer(&sensor, &src, pattern, range, None);
        for (x, y, sample, black, want) in AC1_COLOUR {
            assert_eq!(
                normalize(x, y),
                want,
                "sample {sample} at black {black} ({x}, {y})"
            );
        }
    }

    #[test]
    fn colour_uniform_black_is_todays_map() {
        // Black 2047 in every slot: W − B = 12953, `normalize_sample` exactly.
        let (sensor, src) = ac1_colour_sensor(&[2047; 4]);
        let Ok(ColourLevels::Slots(levels)) = resolve_colour_levels(&sensor) else {
            panic!("a [2, 2] pattern takes the slot path");
        };
        let normalize = slot_normalizer(&sensor, &src, levels, None);
        for (x, y, sample, _, _) in AC1_COLOUR {
            assert_eq!(
                normalize(x, y),
                normalize_sample(sample, 2047, 15_000),
                "sample {sample} ({x}, {y})"
            );
        }
        // And an absent `BlackLevel` pattern: the scalar black, today's map.
        let (mut scalar, src) = ac1_colour_sensor(&[]);
        scalar.black_level = Some(2047);
        let Ok(ColourLevels::Slots(levels)) = resolve_colour_levels(&scalar) else {
            panic!("no pattern takes the slot path");
        };
        assert_eq!(levels, [(2047, 12953); 4]);
        let normalize = slot_normalizer(&scalar, &src, levels, None);
        assert_eq!(normalize(0, 0), 32765, "today's value at 8523");
    }

    /// `resolve_colour_levels`' result, required to be exactly
    /// `InvalidLevels { black_level: black, white_level: white }`.
    fn assert_invalid_levels(sensor: &Sensor, black: u32, white: u32, ctx: &str) {
        match resolve_colour_levels(sensor) {
            Err(Error::InvalidLevels {
                black_level,
                white_level,
            }) => assert_eq!((black_level, white_level), (black, white), "{ctx}"),
            Err(other) => panic!("{ctx}: expected InvalidLevels, got {other:?}"),
            Ok(_) => panic!("{ctx}: expected InvalidLevels, got Ok"),
        }
    }

    #[test]
    fn colour_with_no_range_is_refused() {
        // SPEC-080 (DEC-070 §3, second amendment) rewrites SPEC-078's
        // `colour_with_no_range_maps_to_zero_without_panicking`: `W ≤ B` is now
        // `InvalidLevels` reporting `B`, as on the mono path, on every branch.
        //
        // The slot branch, with the old test's fixture: W = B = 15000, at the
        // last index.
        let (sensor, _) = ac1_colour_sensor(&[2047, 2047, 2047, 15_000]);
        assert_invalid_levels(&sensor, 15_000, 15_000, "2x2, W = B, B last");
        // B neither first nor last, and the first black below W.
        let (sensor, _) = ac1_colour_sensor(&[2047, 16_000, 2047, 2047]);
        assert_invalid_levels(&sensor, 16_000, 15_000, "2x2, W < B, B second");
        // A [1, 2] pattern fills the slots from two values.
        let (mut sensor, _) = ac1_colour_sensor(&[2047, 15_000]);
        sensor.black_level_repeat_dim = Some([1, 2]);
        assert_invalid_levels(&sensor, 15_000, 15_000, "[1, 2], W = B");
        // The general branch: a 3x3 pattern, B in the middle.
        let (mut sensor, _) =
            ac1_colour_sensor(&[2047, 2047, 2047, 2047, 15_001, 2047, 2047, 2047, 2047]);
        sensor.black_level_repeat_dim = Some([3, 3]);
        assert_invalid_levels(&sensor, 15_001, 15_000, "[3, 3], W < B, B middle");
        // The scalar branch: no pattern, the scalar black.
        let (mut sensor, _) = ac1_colour_sensor(&[]);
        sensor.black_level = Some(15_000);
        assert_invalid_levels(&sensor, 15_000, 15_000, "scalar, W = B");
        // The colour white is white_levels' first value, not white_level.
        let (mut sensor, _) = ac1_colour_sensor(&[2047, 2047, 2047, 2047]);
        sensor.white_levels = vec![2047, 60_000];
        assert_invalid_levels(&sensor, 2047, 2047, "white_levels first");
    }

    #[test]
    fn colour_one_code_of_range_is_accepted_and_clips_at_white() {
        // W = B + 1 on each branch: Ok, range 1, and a sample at or above W
        // maps to 65535 while one at the black maps to 0.
        let (sensor, src) = ac1_colour_sensor(&[2047, 14_999, 2047, 2047]);
        let Ok(ColourLevels::Slots(levels)) = resolve_colour_levels(&sensor) else {
            panic!("a [2, 2] pattern at W = B + 1 takes the slot path");
        };
        assert_eq!(levels.map(|(_, range)| range), [1; 4], "range W − B");
        // With one code of range, a sample above its own black maps to 65535
        // and one at or below it to 0 (the hand rule; no crate call).
        let blacks = [2047, 14_999, 2047, 2047];
        let expect = |sample: u16, black: u32| if u32::from(sample) > black { 65_535 } else { 0 };
        let normalize = slot_normalizer(&sensor, &src, levels, None);
        for y in 0..2u32 {
            for x in 0..4u32 {
                let sample = src[(y * 4 + x) as usize];
                let black = blacks[((y % 2) * 2 + x % 2) as usize];
                assert_eq!(
                    normalize(x, y),
                    expect(sample, black),
                    "slot: sample {sample} at black {black} ({x}, {y})"
                );
            }
        }
        // A sample of W and of u16::MAX through the slot holding B.
        assert_eq!(normalize_range(15_000, 14_999, 1), 65_535, "W at B");
        assert_eq!(normalize_range(u16::MAX, 14_999, 1), 65_535, "above W");

        #[rustfmt::skip]
        let general = [
            2047, 2047, 2047,
            2047, 14_999, 2047,
            2047, 2047, 2047,
        ];
        let (mut sensor, src) = ac1_colour_sensor(&general);
        sensor.black_level_repeat_dim = Some([3, 3]);
        let Ok(ColourLevels::Pattern(pattern, range)) = resolve_colour_levels(&sensor) else {
            panic!("a [3, 3] pattern at W = B + 1 takes the general path");
        };
        assert_eq!(range, 1, "range W − B");
        let normalize = pattern_normalizer(&sensor, &src, pattern, range, None);
        for y in 0..2u32 {
            for x in 0..4u32 {
                let sample = src[(y * 4 + x) as usize];
                let black = general[((y % 3) * 3 + x % 3) as usize];
                assert_eq!(
                    normalize(x, y),
                    expect(sample, black),
                    "general: sample {sample} at black {black} ({x}, {y})"
                );
            }
        }

        let (mut sensor, _) = ac1_colour_sensor(&[]);
        sensor.black_level = Some(14_999);
        let Ok(ColourLevels::Slots(levels)) = resolve_colour_levels(&sensor) else {
            panic!("no pattern takes the slot path");
        };
        assert_eq!(levels, [(14_999, 1); 4]);
    }

    #[test]
    fn normalize_range_stays_total_at_range_zero() {
        // The refusal sits in the resolvers; the normaliser itself still maps
        // range 0 to 0 without dividing by zero (it also serves the mono path).
        for sample in [0, 1, 2047, 15_000, u16::MAX] {
            for black in [0, 2047, 15_000, u32::MAX] {
                assert_eq!(normalize_range(sample, black, 0), 0, "{sample} at {black}");
            }
        }
    }

    // ── SPEC-034: dual-illuminant interpolation (DNG NeutralToXY + McCamy) ──

    /// EXIF/DNG `CalibrationIlluminant` codes used in the fixtures.
    const ILLUMINANT_A: u32 = 17; // Standard light A → 2856 K
    const ILLUMINANT_D65: u32 = 21; // D65 → 6504 K

    /// A `Sensor` with the full dual-calibration colour metadata (reuses
    /// [`color_sensor`] for the inert fields).
    #[allow(clippy::too_many_arguments)]
    fn dual_sensor(
        color_matrix_1: Option<Vec<Rational>>,
        color_matrix_2: Option<Vec<Rational>>,
        forward_matrix_1: Option<Vec<Rational>>,
        forward_matrix_2: Option<Vec<Rational>>,
        as_shot_neutral: Option<Vec<Rational>>,
        calibration_illuminant_1: Option<u32>,
        calibration_illuminant_2: Option<u32>,
    ) -> Sensor {
        let mut s = color_sensor(ColorMeta {
            forward_matrix_1: None,
            color_matrix_1: None,
            as_shot_neutral,
        });
        s.color_matrix_1 = color_matrix_1;
        s.color_matrix_2 = color_matrix_2;
        s.forward_matrix_1 = forward_matrix_1;
        s.forward_matrix_2 = forward_matrix_2;
        s.calibration_illuminant_1 = calibration_illuminant_1;
        s.calibration_illuminant_2 = calibration_illuminant_2;
        s
    }

    /// The fixture neutral `(1/2, 1/1, 1/2)`.
    fn asn_half_one_half() -> Vec<Rational> {
        vec![
            Rational { num: 1, den: 2 },
            Rational { num: 1, den: 1 },
            Rational { num: 1, den: 2 },
        ]
    }

    /// The fixture `ColorMatrix1` (warm calibration) — `num/10000`. Built so the
    /// neutral maps toward a warm white; invertible.
    fn fixture_cm1() -> Vec<Rational> {
        matrix_rationals([
            [39216, -17647, -39216],
            [-24510, 73529, -100490],
            [-9804, -70588, 259804],
        ])
    }

    /// The fixture `ColorMatrix2` (cool calibration) — `num/10000`; invertible.
    fn fixture_cm2() -> Vec<Rational> {
        matrix_rationals([
            [35893, -13393, -9821],
            [-6071, 41071, -23214],
            [-32143, -17857, 53571],
        ])
    }

    /// A 3×3 identity as `num/10000` rationals (`inv(identity) == identity`).
    fn identity_rationals() -> Vec<Rational> {
        matrix_rationals([[10000, 0, 0], [0, 10000, 0], [0, 0, 10000]])
    }

    // Independent reference transcription of the NeutralToXY weight iteration —
    // triple-loop lerp / mat-vec and a plainly-written McCamy, separate from the
    // shipped unrolled helpers (the genuine cross-check is `tests/color_oracle.rs`
    // + the perturbed-ColorMatrix2 red-proof; these mirror the file's existing
    // in-module reference convention, which reuses the private `inv`).

    fn ref_lerp(a: [[f64; 3]; 3], b: [[f64; 3]; 3], g: f64) -> [[f64; 3]; 3] {
        let mut o = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                o[i][j] = (1.0 - g) * a[i][j] + g * b[i][j];
            }
        }
        o
    }

    fn ref_mul_vec(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
        let mut o = [0.0f64; 3];
        for i in 0..3 {
            o[i] = m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2];
        }
        o
    }

    fn ref_mccamy(x: f64, y: f64) -> f64 {
        let n = (x - 0.3320) / (0.1858 - y);
        449.0 * n * n * n + 3525.0 * n * n + 6823.3 * n + 5520.33
    }

    /// The reference mired weight of illuminant 2 at a chromaticity.
    fn ref_weight_at(xy: (f64, f64), inv_t1: f64, denom: f64) -> f64 {
        ((inv_t1 - 1.0 / ref_mccamy(xy.0, xy.1)) / denom).clamp(0.0, 1.0)
    }

    /// `xy(inv(CM(g)) · neutral)` for the reference.
    fn ref_neutral_xy(
        m1: [[f64; 3]; 3],
        m2: [[f64; 3]; 3],
        neutral: [f64; 3],
        g: f64,
    ) -> (f64, f64) {
        let ic = inv(ref_lerp(m1, m2, g)).expect("reference CM(g) invertible");
        let w = ref_mul_vec(ic, neutral);
        let sum = w[0] + w[1] + w[2];
        (w[0] / sum, w[1] / sum)
    }

    /// A reference NeutralToXY weight from the two ColorMatrices, the neutral,
    /// and the two illuminant CCTs: `passes == None` runs the SDK 1.7.1 loop to
    /// convergence (`DEC-076` §5: from D50, at most 30 passes, `|Δx| + |Δy| <
    /// 1e-7`, else the midpoint of the last two); `Some(3)` runs `DEC-034`'s
    /// superseded fixed 3 passes from `g = 0`, kept as the contrast.
    fn ref_weight(
        cm1: &[Rational],
        cm2: &[Rational],
        asn: &[Rational],
        t1: f64,
        t2: f64,
        passes: Option<u32>,
    ) -> f64 {
        let m1 = reference_matrix(cm1);
        let m2 = reference_matrix(cm2);
        let neutral = [
            f64::from(asn[0].num) / f64::from(asn[0].den),
            f64::from(asn[1].num) / f64::from(asn[1].den),
            f64::from(asn[2].num) / f64::from(asn[2].den),
        ];
        let inv_t1 = 1.0 / t1;
        let denom = inv_t1 - 1.0 / t2;
        if let Some(n) = passes {
            let mut g = 0.0f64;
            for _ in 0..n {
                g = ref_weight_at(ref_neutral_xy(m1, m2, neutral, g), inv_t1, denom);
            }
            return g;
        }
        let mut last = (0.3457, 0.3585);
        for pass in 0..30 {
            let g = ref_weight_at(last, inv_t1, denom);
            let mut next = ref_neutral_xy(m1, m2, neutral, g);
            if (next.0 - last.0).abs() + (next.1 - last.1).abs() < 0.0000001 {
                last = next;
                break;
            }
            if pass == 29 {
                next = ((last.0 + next.0) * 0.5, (last.1 + next.1) * 0.5);
            }
            last = next;
        }
        ref_weight_at(last, inv_t1, denom)
    }

    #[test]
    fn interpolation_weight_matches_dng_neutral_to_xy() {
        let cm1 = fixture_cm1();
        let cm2 = fixture_cm2();
        let asn = asn_half_one_half();
        let sensor = dual_sensor(
            Some(cm1.clone()),
            Some(cm2.clone()),
            None,
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );

        let g = interpolation_weight(&sensor).expect("dual calibration interpolates");
        let g_ref = ref_weight(&cm1, &cm2, &asn, 2856.0, 6504.0, None);

        // Bit-exact against the independent NeutralToXY re-derivation.
        assert_eq!(
            g, g_ref,
            "the weight must match the independent NeutralToXY reference to the bit"
        );
        // A genuine interior blend — not a trivially-clamped endpoint — and the
        // hand-computed value.
        assert!(g > 0.0 && g < 1.0, "expected an interior weight, got {g}");
        // Re-pinned at SPEC-085 (DEC-076 §5): the converged weight.
        assert!(
            (g - 0.6708871184066116).abs() < 1e-9,
            "weight drifted from the hand-computed value: {g}"
        );
        // The contrast: DEC-034's fixed 3 passes gave SPEC-034's pinned value,
        // which the converged loop is not.
        let g_three = ref_weight(&cm1, &cm2, &asn, 2856.0, 6504.0, Some(3));
        assert!(
            (g_three - 0.5202685520213226).abs() < 1e-9,
            "the 3-pass contrast drifted: {g_three}"
        );
        assert!((g - g_three).abs() > 1e-4);
    }

    #[test]
    fn weight_clamps_outside_the_calibration_range() {
        // Identity ColorMatrices → the neutral maps to itself (as XYZ), so the
        // McCamy CCT is read straight off the neutral's chromaticity and the
        // weight is a single fixed clamp.
        let identity = identity_rationals();

        // A warm neutral (McCamy ≈ 2714 K, cooler than T1 = 2856 K) → g clamps to 0.
        let warm = vec![
            Rational { num: 6, den: 5 }, // 1.2
            Rational { num: 1, den: 1 }, // 1.0
            Rational { num: 3, den: 5 }, // 0.6
        ];
        let sensor = dual_sensor(
            Some(identity.clone()),
            Some(identity.clone()),
            None,
            None,
            Some(warm),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(
            interpolation_weight(&sensor),
            Some(0.0),
            "a neutral cooler than T1 must clamp to g = 0 (illuminant 1)"
        );

        // A cool neutral (McCamy ≈ 9930 K, hotter than T2 = 6504 K) → g clamps to 1.
        let cool = vec![
            Rational { num: 9, den: 10 }, // 0.9
            Rational { num: 1, den: 1 },  // 1.0
            Rational { num: 7, den: 5 },  // 1.4
        ];
        let sensor = dual_sensor(
            Some(identity.clone()),
            Some(identity),
            None,
            None,
            Some(cool),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(
            interpolation_weight(&sensor),
            Some(1.0),
            "a neutral hotter than T2 must clamp to g = 1 (illuminant 2)"
        );
    }

    #[test]
    fn mccamy_cct_matches_known_chromaticities() {
        // D65 xy → ≈ 6504 K; D50 xy → ≈ 5003 K (McCamy's own exactness).
        let d65 = mccamy_cct(0.31271, 0.32902).expect("D65 is well-defined");
        assert!((d65 - 6504.0).abs() < 5.0, "D65 McCamy CCT off: {d65}");
        let d50 = mccamy_cct(0.34567, 0.35850).expect("D50 is well-defined");
        assert!((d50 - 5002.0).abs() < 5.0, "D50 McCamy CCT off: {d50}");

        // A zero denominator `0.1858 − y` is guarded (no divide to inf/NaN).
        assert_eq!(mccamy_cct(0.4, 0.1858), None);
    }

    #[test]
    fn interpolation_preconditions_that_fall_back() {
        let cm1 = fixture_cm1();
        let cm2 = fixture_cm2();
        let asn = asn_half_one_half();

        // No ColorMatrix2 → cannot interpolate.
        let no_cm2 = dual_sensor(
            Some(cm1.clone()),
            None,
            None,
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&no_cm2), None);

        // An unknown CalibrationIlluminant code → cannot interpolate.
        let unknown_illum = dual_sensor(
            Some(cm1.clone()),
            Some(cm2.clone()),
            None,
            None,
            Some(asn.clone()),
            Some(99),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&unknown_illum), None);

        // T1 == T2 (both Standard A) → the mired blend is undefined.
        let same_temp = dual_sensor(
            Some(cm1),
            Some(cm2),
            None,
            None,
            Some(asn),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_A),
        );
        assert_eq!(interpolation_weight(&same_temp), None);
    }

    #[test]
    fn single_illuminant_build_is_byte_identical_to_spec_033() {
        // A ForwardMatrix1 + ColorMatrix1 sensor with NO ColorMatrix2 falls back
        // to the SPEC-033 illuminant-1 path — byte-identical. Build the SPEC-033
        // result independently (C · FM1 · D) and compare to the bit.
        let fm1 = matrix_rationals([[6000, 3000, 642], [2000, 7000, 1000], [1000, 1000, 6252]]);
        let asn = asn_half_one_half();
        let sensor = dual_sensor(
            Some(fixture_cm1()),
            None, // no CM2 ⇒ fallback
            Some(fm1.clone()),
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&sensor), None, "no CM2 ⇒ fallback");

        let c = reference_mat_mul(SRGB_FROM_XYZ_D65, BRADFORD_D50_TO_D65);
        let expected = reference_mat_mul(
            c,
            reference_mat_mul(reference_matrix(&fm1), reference_wb(&asn)),
        );
        assert_eq!(
            build_camera_to_srgb_linear(&sensor).unwrap(),
            expected,
            "the single-illuminant fallback must equal SPEC-033's C · FM1 · D to the bit"
        );
    }

    #[test]
    fn degenerate_interpolation_inputs_do_not_panic() {
        let asn = asn_half_one_half();

        // (a) A ColorMatrix2 with a zero-denominator entry is unusable. Before
        //     SPEC-089 that silently fell back to single-illuminant colour via
        //     ForwardMatrix1; DEC-078 §7.2 makes it an error, never a panic.
        let mut bad_cm2 = fixture_cm2();
        bad_cm2[0] = Rational { num: 1, den: 0 };
        let fm1 = matrix_rationals([[6000, 3000, 642], [2000, 7000, 1000], [1000, 1000, 6252]]);
        let sensor = dual_sensor(
            Some(fixture_cm1()),
            Some(bad_cm2),
            Some(fm1),
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&sensor), None);
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // (b) A singular ColorMatrix1 (row 1 = 2·row 0), no ForwardMatrix, with a
        //     present ColorMatrix2 and illuminants. Before SPEC-085 the 3-pass
        //     loop started at g = 0, inverted the singular CM1, fell back and
        //     refused. The converged loop (DEC-076 §5) starts at D50 like the
        //     SDK's, settles at g = 1 and so develops on CM2 alone: the singular
        //     CM1 is never inverted. Approved behaviour change at SPEC-085's
        //     build; whether a singular ColorMatrix_i should refuse even when
        //     unused belongs to SPEC-087. Still never a panic and never a NaN.
        let singular_cm1 =
            matrix_rationals([[10000, 20000, 30000], [20000, 40000, 60000], [0, 0, 10000]]);
        let sensor = dual_sensor(
            Some(singular_cm1),
            Some(fixture_cm2()),
            None,
            None,
            Some(asn),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert_eq!(interpolation_weight(&sensor), Some(1.0));
        let built = build_camera_to_srgb_linear(&sensor).expect("develops on CM2 alone");
        assert!(built.iter().flatten().all(|v| v.is_finite()), "{built:?}");

        // (c) The chromaticity guard: a non-positive XYZ sum is None, not a divide.
        assert_eq!(xyz_to_xy([-1.0, 0.0, 0.0]), None);
        assert_eq!(xyz_to_xy([0.0, 0.0, 0.0]), None);
        assert!(xyz_to_xy([0.3, 0.4, 0.3]).is_some());
    }

    // ── SPEC-089 AC4 (DEC-078 §7.2): a present matrix's error propagates ─────
    //
    // Hand-built sensors, bypassing the reader (which refuses these shapes
    // first): the colour code is the second line, and it no longer turns a
    // present matrix it cannot use into a fallback.

    /// `entries` cut to its first eight: the misshapen form of a 3×3 matrix.
    fn eight(entries: Vec<Rational>) -> Vec<Rational> {
        entries.into_iter().take(8).collect()
    }

    #[test]
    fn ac4_dual_illuminant_propagates_a_present_matrix_error() {
        // Both illuminants known and distinct, so only the matrix decides
        // (mutant (d)'s distinguishing fixture).
        let asn = asn_half_one_half();
        let misshapen = dual_sensor(
            Some(fixture_cm1()),
            Some(eight(fixture_cm2())),
            None,
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert!(matches!(
            dual_illuminant(&misshapen),
            Err(Error::MissingColorCalibration)
        ));
        assert!(matches!(
            build_camera_to_srgb_linear(&misshapen),
            Err(Error::MissingColorCalibration)
        ));

        // The absent control still takes the single-illuminant path, Ok.
        let absent = dual_sensor(
            Some(fixture_cm1()),
            None,
            None,
            None,
            Some(asn.clone()),
            Some(ILLUMINANT_A),
            Some(ILLUMINANT_D65),
        );
        assert!(matches!(dual_illuminant(&absent), Ok(None)));
        let single = build_camera_to_srgb_linear(&absent).expect("absent CM2 develops");
        // And it is exactly what an unknown illuminant gives (rank 8 keeps its
        // fallback): today's single-illuminant output.
        let unknown = dual_sensor(
            Some(fixture_cm1()),
            Some(fixture_cm2()),
            None,
            None,
            Some(asn),
            Some(ILLUMINANT_A),
            Some(99),
        );
        assert!(matches!(dual_illuminant(&unknown), Ok(None)));
        assert_eq!(build_camera_to_srgb_linear(&unknown).unwrap(), single);
    }

    /// The AC4 `ForwardMatrix1`.
    fn fm1_fixture() -> Vec<Rational> {
        matrix_rationals([[6000, 3000, 642], [2000, 7000, 1000], [1000, 1000, 6252]])
    }

    /// A `ForwardMatrix2` distinct from [`fm1_fixture`], so the ForwardMatrix
    /// path differs from the ColorMatrix path.
    fn fm2_fixture() -> Vec<Rational> {
        matrix_rationals([[6500, 2500, 642], [2500, 6500, 1000], [800, 1200, 6252]])
    }

    #[test]
    fn ac4_forward_matrix_error_propagates() {
        let asn = asn_half_one_half();
        let with_fm2 = |fm2: Option<Vec<Rational>>| {
            dual_sensor(
                Some(fixture_cm1()),
                Some(fixture_cm2()),
                Some(fm1_fixture()),
                fm2,
                Some(asn.clone()),
                Some(ILLUMINANT_A),
                Some(ILLUMINANT_D65),
            )
        };
        // The two valid paths differ, so taking the wrong one would show
        // (mutant (e)'s distinguishing fixture).
        let fm_path = build_camera_to_srgb_linear(&with_fm2(Some(fm2_fixture())))
            .expect("the ForwardMatrix path develops");
        let cm_path =
            build_camera_to_srgb_linear(&with_fm2(None)).expect("the ColorMatrix path develops");
        assert_ne!(fm_path, cm_path);

        // A present, misshapen ForwardMatrix2 is an error, not the ColorMatrix
        // path's output.
        let misshapen = with_fm2(Some(eight(fm2_fixture())));
        assert!(matches!(
            build_camera_to_srgb_linear(&misshapen),
            Err(Error::MissingColorCalibration)
        ));
    }

    #[test]
    fn ac4_unused_calibration_is_validated() {
        // Single-illuminant (one CalibrationIlluminant, no ColorMatrix2), so
        // CameraCalibration2 is never applied (mutant (f)'s distinguishing
        // fixture: on a dual sensor the old code already refused).
        let asn = asn_half_one_half();
        let mut sensor = dual_sensor(
            Some(fixture_cm1()),
            None,
            None,
            None,
            Some(asn),
            Some(ILLUMINANT_A),
            None,
        );
        let control = build_camera_to_srgb_linear(&sensor).expect("no calibration develops");
        assert!(resolve_calibration(&sensor, false).is_ok());

        sensor.camera_calibration_2 = Some(eight(identity_rationals()));
        assert!(matches!(
            resolve_calibration(&sensor, false),
            Err(Error::MissingColorCalibration)
        ));
        assert!(matches!(
            build_camera_to_srgb_linear(&sensor),
            Err(Error::MissingColorCalibration)
        ));

        // A well-formed but unused CameraCalibration2 changes nothing.
        sensor.camera_calibration_2 = Some(matrix_rationals([
            [10500, 0, 0],
            [0, 10000, 0],
            [0, 0, 9500],
        ]));
        assert_eq!(build_camera_to_srgb_linear(&sensor).unwrap(), control);
    }

    /// `SPEC-091` AC5: each `f32` of the sensor-geometry plane is the matrix's
    /// `f64` sum on the demosaiced camera triple, divided by `65535.0` and cast
    /// once, unclamped — recomputed here from `build_camera_to_srgb_linear` and
    /// `demosaic_into`, and compared by bits. The levels are the identity (16
    /// bits, no black, no white), so `demosaic_into`'s camera values are the
    /// fused traversal's. A strong white balance gives negative and over-1.0
    /// values, so a clamp, a quantise or a `/ 65536` cannot pass.
    #[test]
    fn ac5_the_f32_plane_is_the_unquantised_sum() {
        use crate::ifd::Cfa;
        let mut sensor = color_sensor(ColorMeta {
            forward_matrix_1: Some(matrix_rationals([
                [6000, 3000, 642],
                [2000, 7000, 1000],
                [1000, 1000, 6252],
            ])),
            color_matrix_1: None,
            as_shot_neutral: Some(vec![
                Rational { num: 1, den: 3 },
                Rational { num: 1, den: 1 },
                Rational { num: 2, den: 5 },
            ]),
        });
        let (w, h) = (6u32, 5u32);
        sensor.width = w;
        sensor.height = h;
        sensor.color_filter = Some(Cfa {
            repeat_dim: [2, 2],
            pattern: vec![0, 1, 1, 2],
            plane_color: vec![0, 1, 2],
            layout: 1,
        });
        let n = (w * h) as usize;
        let src: Vec<u16> = (0..n).map(|i| ((i * 7919 + 1234) % 65536) as u16).collect();

        let mut camera = vec![0u16; n * 3];
        crate::demosaic::demosaic_into(&sensor, &src, &mut camera).unwrap();
        let m = build_camera_to_srgb_linear(&sensor).unwrap();
        let expected: Vec<u32> = camera
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|&[r, g, b]| {
                let (r, g, b) = (f64::from(r), f64::from(g), f64::from(b));
                m.map(|row| (((row[0] * r + row[1] * g + row[2] * b) / 65535.0) as f32).to_bits())
            })
            .collect();

        let mut plane = vec![0f32; n * 3];
        srgb_linear_plane_f32_into(&sensor, &src, &mut plane).unwrap();
        let got: Vec<u32> = plane.iter().map(|v| v.to_bits()).collect();
        assert_eq!(got, expected);
        assert!(
            plane.iter().any(|&v| v < 0.0),
            "a negative value is exercised"
        );
        assert!(
            plane.iter().any(|&v| v > 1.0),
            "an over-1.0 value is exercised"
        );
    }

    /// `SPEC-091` mutant (k): the unclamped `f32` sink's sums are separate
    /// multiplies and adds, never a fused `mul_add` (`DEC-033`: Rust never
    /// contracts one, and the crate must not call it). On real camera data an
    /// FMA moves the `f64` sum by about one ulp, which the single cast to `f32`
    /// almost always absorbs, so neither AC5 nor the determinism hash can see
    /// it. This injected matrix makes the sum pure rounding error: `0.1·3`
    /// rounds up in `f64` and `−0.3` cancels it, leaving `2^-54` when the
    /// product is rounded first and `2^-55` when it is fused, which the `f32`
    /// keeps apart.
    #[test]
    fn the_f32_sink_never_fuses_a_multiply_and_an_add() {
        let m = [[0.1, -0.3, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (rf, gf, bf) = (3.0f64, 1.0f64, 0.0f64);
        let separate = ((m[0][0] * rf + m[0][1] * gf + m[0][2] * bf) / 65535.0) as f32;
        let fused = (m[0][0].mul_add(rf, m[0][1] * gf + m[0][2] * bf) / 65535.0) as f32;
        assert_ne!(
            separate.to_bits(),
            fused.to_bits(),
            "the fixture must tell a fused sum from a separate one"
        );
        let mut out = [0f32; 3];
        store_f32_unclamped(&mut out, &m, [3, 1, 0]);
        assert_eq!(
            out[0].to_bits(),
            separate.to_bits(),
            "separate multiply and add"
        );
        assert!(out[0] > 0.0, "the cancellation leaves a positive residue");
    }

    /// `SPEC-091` verify, mutant (k) everywhere: the test above places the
    /// cancellation in channel 0's first term only, so a `mul_add` in channel 1
    /// or 2, or on another term, passes it. Here the inexact product `0.1·3`
    /// sits at every term `j` of every channel `i`, with the cancelling `−0.3`
    /// at each other term `k`, so whichever multiply is fused into an add that
    /// already holds `−0.3`, one case sees `2^-55` instead of `2^-54`.
    #[test]
    fn the_u16_sink_keeps_todays_operation_order() {
        // `SPEC-091` verify: `store_u16` is shared by `develop_color_into` and
        // the plane, so AC1 (plane vs developed image) cannot see a reordered
        // sum: both sides move together. On real data `quantize` almost always
        // absorbs the one-ulp difference, so no hash sees it either. This row
        // sums to just under `2635.5` in today's order `(m0·r + m1·g) + m2·b`
        // and to just over it in every other order, so the round-half-up
        // differs. (Swapping the first two terms is exact, so it is not tested.)
        let row = [1.1f64, 0.3, -0.1];
        let cam = [476u16, 15845, 26416];
        let [r, g, b] = cam.map(f64::from);
        let today = (row[0] * r + row[1] * g) + row[2] * b;
        let reordered = (row[2] * b + row[1] * g) + row[0] * r;
        assert_ne!(
            quantize(today),
            quantize(reordered),
            "the fixture must tell the two orders apart"
        );
        for i in 0..3 {
            let mut m = [[0.0f64; 3]; 3];
            m[i] = row;
            let mut out = [0u16; 3];
            store_u16(&mut out, &m, cam);
            assert_eq!(out[i], quantize(today), "channel {i}");
            assert_eq!(out[i], 2636, "channel {i}");
        }
    }

    #[test]
    fn the_f32_sink_fuses_no_term_of_any_channel() {
        let separate = ((0.1f64 * 3.0 - 0.3) / 65535.0) as f32;
        let fused = (0.1f64.mul_add(3.0, -0.3) / 65535.0) as f32;
        assert_ne!(
            separate.to_bits(),
            fused.to_bits(),
            "the fixture must tell a fused sum from a separate one"
        );
        for i in 0..3 {
            for j in 0..3 {
                for k in (0..3).filter(|&k| k != j) {
                    let mut m = [[0.0f64; 3]; 3];
                    m[i][j] = 0.1;
                    m[i][k] = -0.3;
                    let mut cam = [0u16; 3];
                    cam[j] = 3;
                    cam[k] = 1;
                    let mut out = [f32::NAN; 3];
                    store_f32_unclamped(&mut out, &m, cam);
                    assert_eq!(
                        out[i].to_bits(),
                        separate.to_bits(),
                        "channel {i}: 0.1 at term {j}, -0.3 at term {k}"
                    );
                }
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SPEC-092: the mono mixer's row build and kernel.
    // ─────────────────────────────────────────────────────────────────────────

    /// The kernel's independent oracle (AC1, AC14): `i128`, a division by
    /// `2^37` with `div_euclid` (floor) instead of a shift, so it shares no
    /// code with `mono_mix_pixel`.
    fn mix_oracle(q: &[i32; 3], x: [u16; 3]) -> u16 {
        let (mut p, mut n) = (0i128, 0i128);
        for (&qi, &xi) in q.iter().zip(&x) {
            let t = i128::from(qi) * i128::from(xi);
            if qi >= 0 {
                p += t;
            } else {
                n += t;
            }
        }
        let m = i128::from(*x.iter().max().unwrap());
        let u = (65535 - m).min(8192);
        let num = p * 8192 + n * u + (1i128 << 36);
        num.div_euclid(1i128 << 37).clamp(0, 65535) as u16
    }

    /// The plain fold rounded half up, `clamp(⌊(Σ q_i·x_i + 2^23) / 2^24⌋)`,
    /// in `i128` (AC3, AC6's control).
    fn plain_fold(q: &[i32; 3], x: [u16; 3]) -> u16 {
        let acc: i128 = q
            .iter()
            .zip(&x)
            .map(|(&qi, &xi)| i128::from(qi) * i128::from(xi))
            .sum();
        (acc + (1i128 << 23))
            .div_euclid(1i128 << 24)
            .clamp(0, 65535) as u16
    }

    fn xorshift(state: &mut u32) -> u32 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *state = x;
        x
    }

    #[test]
    fn ac3_no_negative_row_is_the_plain_fold() {
        let mut values: Vec<u16> = vec![0, 1, 57342, 57343, 57344, 61439, 65534, 65535];
        let mut state = 0x5092_0003;
        values.extend((0..64).map(|_| (xorshift(&mut state) & 0xffff) as u16));
        let rows: [[i32; 3]; 4] = [
            [9_238_644, 11_642_451, 1_534_676], // Pixel 3a luminance (AC5)
            [6_580_281, 12_748_060, 1_667_508], // Pixel 4a luminance (AC5)
            [21_010, 8_388_347, 14_980_760],    // Pixel 7 Pro ortho (AC5)
            [16_777_216, 0, 33_554_432],        // a zero entry and a weight above one
        ];
        let mut above = 0usize;
        for q in &rows {
            assert!(q.iter().all(|&v| v >= 0), "{q:?} has no negative entry");
            for &r in &values {
                for &g in &values {
                    for &b in &values {
                        let x = [r, g, b];
                        above += usize::from(r.max(g).max(b) >= 57344);
                        assert_eq!(mono_mix_pixel(q, x), plain_fold(q, x), "row {q:?}, x {x:?}");
                    }
                }
            }
        }
        assert!(above > 0, "some x lie above the threshold");
    }

    /// Whether the kernel ever decreases along a saturating ray
    /// `x_i(k) = min(k·c_i, 65535)`, `k` from 0 until every non-zero channel
    /// is white; returns the number of rays that decrease.
    fn decreasing_rays(q: &[i32; 3], mix: impl Fn(&[i32; 3], [u16; 3]) -> u16) -> usize {
        let mut bad = 0;
        for a in (0..=598u32).step_by(2) {
            for b in (0..=598u32).step_by(2) {
                let c = [a, 299, b];
                let cmin = c.iter().copied().filter(|&v| v > 0).min().unwrap();
                let kmax = 65535 / cmin + 1;
                let mut prev = 0u16;
                for k in 0..=kmax {
                    let x = c.map(|ci| (ci * k).min(65535) as u16);
                    let f = mix(q, x);
                    if f < prev {
                        bad += 1;
                        break;
                    }
                    prev = f;
                }
            }
        }
        bad
    }

    #[test]
    fn ac6_the_fade_is_monotone_on_the_saturating_ray_grid() {
        let rows: [(&str, [i32; 3]); 5] = [
            ("Canon luminance", [6_222_052, 14_856_368, -4_008_594]),
            ("Canon ortho", [-3_895_542, 7_681_333, 24_802_109]),
            ("Canon one-hot green", [-8_076_979, 27_606_901, -12_847_745]),
            ("Nikon luminance", [9_666_745, 12_584_293, -2_609_097]),
            ("Pixel 4a one-hot green", [-528_141, 17_561_180, -943_847]),
        ];
        for (name, q) in &rows {
            let bad = decreasing_rays(q, mono_mix_pixel);
            assert_eq!(bad, 0, "{name}: {bad} rays decrease under the fade");
        }
        // The control: the plain fold on the Canon luminance row is not
        // monotone, so the grid can fail.
        let control = decreasing_rays(&rows[0].1, plain_fold);
        eprintln!("AC6 control: the plain fold decreases on {control} of 90000 rays");
        assert!(control > 0, "the grid must catch a non-monotone mix");
    }

    #[test]
    fn ac8_the_fade_starts_after_57343_and_reads_the_largest_channel() {
        let q = [0, 1 << 30, -(1 << 30)];
        assert_eq!(
            mono_mix_pixel(&q, [0, 57343, 57343]),
            0,
            "u = 8192, no fade"
        );
        // Well below the threshold, u stays capped at 8192: the plain fold
        // 2^30·(1000 − 500)/2^24 = 32000. An uncapped u (65535 − m) would
        // weigh the negative term 7.9 times and clamp to 0.
        assert_eq!(
            mono_mix_pixel(&q, [0, 1000, 500]),
            32000,
            "m far below, u = 8192"
        );
        assert_eq!(
            mono_mix_pixel(&q, [0, 57344, 57344]),
            448,
            "u = 8191, one step"
        );
        assert_eq!(
            mono_mix_pixel(&q, [0, 65535, 65535]),
            65535,
            "u = 0, positive terms only, clamped"
        );
        let q = [1 << 22, 1 << 22, -(1 << 22)];
        for (x, want, plain) in [
            ([65535, 1000, 1000], 16634, 16384),
            ([1000, 65535, 1000], 16634, 16384),
            ([1000, 1000, 65535], 500, 0),
        ] {
            assert_eq!(mono_mix_pixel(&q, x), want, "x {x:?}");
            assert_eq!(plain_fold(&q, x), plain, "x {x:?}: the plain fold");
        }
    }

    /// The unfused value of the cancellation `0.1·3 − 0.3`.
    const CANCEL_UNFUSED: f64 = 5.551_115_123_125_783e-17;
    /// The same with the multiply and the add fused.
    const CANCEL_FUSED: f64 = 2.775_557_561_562_891_4e-17;

    #[test]
    fn ac11_the_row_fuses_no_term_of_any_channel() {
        assert_eq!(0.1f64 * 3.0 - 0.3, CANCEL_UNFUSED);
        assert_eq!(0.1f64.mul_add(3.0, -0.3), CANCEL_FUSED);
        for i in 0..3 {
            for j in 0..3 {
                // The cancellation sits on site `j` of column `i`; the
                // partner term carries the −0.3.
                let mut w = [0.0f64; 3];
                let mut m = [[0.0f64; 3]; 3];
                let partner = if j == 0 { 1 } else { 0 };
                w[j] = 0.1;
                w[partner] = 1.0;
                m[j][i] = 3.0;
                m[partner][i] = -0.3;
                let row = fold_mix_row(w, &m);
                // The mutant fusing site `j`, computed here.
                let t = |k: usize| w[k] * m[k][i];
                let mutant = match j {
                    0 => w[0].mul_add(m[0][i], t(1)) + t(2),
                    1 => w[1].mul_add(m[1][i], t(0)) + t(2),
                    _ => w[2].mul_add(m[2][i], t(0) + t(1)),
                };
                assert_eq!(
                    row[i].to_bits(),
                    CANCEL_UNFUSED.to_bits(),
                    "channel {i}, site {j}"
                );
                assert_ne!(
                    mutant.to_bits(),
                    row[i].to_bits(),
                    "channel {i}, site {j}: the fixture must tell a fused site"
                );
                for other in (0..3).filter(|&o| o != i) {
                    assert_eq!(row[other], 0.0, "channel {i}, site {j}: column {other}");
                }
            }
        }
    }

    #[test]
    fn ac11_the_row_keeps_its_summation_order() {
        let e52 = f64::EPSILON; // 2^-52
        let e53 = e52 / 2.0; // 2^-53
                             // (terms, the wrong pairing it catches)
        let cases: [([f64; 3], &str); 2] = [
            ([1.0, e52, e53], "(t0 + t2) + t1"),
            ([e53, e53, 1.0], "(t1 + t2) + t0"),
        ];
        for i in 0..3 {
            for (t, pairing) in cases {
                let mut m = [[0.0f64; 3]; 3];
                for j in 0..3 {
                    m[j][i] = t[j];
                }
                let row = fold_mix_row([1.0; 3], &m);
                let right = (t[0] + t[1]) + t[2];
                let wrong = if pairing.starts_with("(t0") {
                    (t[0] + t[2]) + t[1]
                } else {
                    (t[1] + t[2]) + t[0]
                };
                assert_eq!(row[i].to_bits(), right.to_bits(), "channel {i}, {pairing}");
                assert_ne!(
                    right.to_bits(),
                    wrong.to_bits(),
                    "channel {i}: {pairing} must differ"
                );
            }
        }
        // The hand-derived values.
        assert_eq!((1.0 + e52) + e53, 1.0 + 2.0 * e52);
        assert_eq!((1.0 + e53) + e52, 1.0 + e52);
        assert_eq!((e53 + e53) + 1.0, 1.0 + e52);
        assert_eq!((e53 + 1.0) + e53, 1.0);
    }

    #[test]
    fn ac11_a_weight_converts_by_one_division() {
        let w = weight_to_f64(&Rational { num: 3, den: 10 });
        assert_eq!(w.to_bits(), 0.3f64.to_bits());
        let reciprocal = 3.0 * (1.0f64 / 10.0);
        assert_eq!(reciprocal, 0.300_000_000_000_000_04);
        assert_ne!(
            reciprocal.to_bits(),
            w.to_bits(),
            "the fixture tells the mutant"
        );
    }

    #[test]
    fn ac11_the_row_rounds_half_up() {
        let mut m = IDENTITY;
        m[0][0] = 2.5 / 16_777_216.0;
        assert_eq!(mix_row_of([1.0, 0.0, 0.0], &m).unwrap(), [3, 0, 0]);
        assert_ne!(2.5f64.round_ties_even(), 3.0, "half to even gives 2");
        m[0][0] = -2.5 / 16_777_216.0;
        assert_eq!(mix_row_of([1.0, 0.0, 0.0], &m).unwrap(), [-2, 0, 0]);
        assert_ne!((-2.5f64).round(), -2.0, "half away from zero gives -3");
    }

    #[test]
    fn ac14_no_overflow_at_the_extremes() {
        let rows = [[i32::MAX; 3], [i32::MIN; 3], [i32::MAX, i32::MIN, i32::MAX]];
        let xs = [[65535u16; 3], [65535, 0, 65535], [57344, 57344, 57344]];
        for q in &rows {
            for &x in &xs {
                assert_eq!(mono_mix_pixel(q, x), mix_oracle(q, x), "row {q:?}, x {x:?}");
            }
        }
        assert_eq!(quantize_row_entry(2_147_483_646.5), Some(i32::MAX));
        assert_eq!(quantize_row_entry(-2_147_483_648.5), Some(i32::MIN));
        for t in [
            2_147_483_647.5,
            -2_147_483_649.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert_eq!(quantize_row_entry(t), None, "t = {t}");
        }
    }

    #[test]
    fn mono_mix_kernel_matches_the_oracle_on_random_rows() {
        let mut state = 0x5092_0001;
        for _ in 0..20_000 {
            let q = [0; 3].map(|_| (xorshift(&mut state) as i32) >> (xorshift(&mut state) % 8));
            let x = [0; 3].map(|_| {
                let v = xorshift(&mut state);
                if v & 3 == 0 {
                    65535 - (v >> 20) as u16 % 9000
                } else {
                    (v >> 8) as u16
                }
            });
            assert_eq!(
                mono_mix_pixel(&q, x),
                mix_oracle(&q, x),
                "row {q:?}, x {x:?}"
            );
        }
    }

    /// Verify (`SPEC-092`, HANDOFF-203): the public constants describe the
    /// kernel, but the kernel reads private ones, so a change to one side alone
    /// moved no test. Pin each public constant to its designed value and to
    /// the private constant the kernel uses, and check the boundary it names.
    #[test]
    fn the_public_mixer_constants_are_the_kernels() {
        assert_eq!(MONO_MIX_ROW_FRACTION_BITS, 24);
        assert_eq!(MONO_MIX_FADE_WIDTH_BITS, 13);
        assert_eq!(MONO_MIX_FADE_START, 57343);
        assert_eq!((MONO_MIX_BLACK, MONO_MIX_WHITE), (0, 65535));
        assert_eq!(MIX_ROW_SCALE, f64::from(1u32 << MONO_MIX_ROW_FRACTION_BITS));
        assert_eq!(MIX_FADE_WIDTH, 1i64 << MONO_MIX_FADE_WIDTH_BITS);
        assert_eq!(
            MIX_SHIFT,
            MONO_MIX_ROW_FRACTION_BITS + MONO_MIX_FADE_WIDTH_BITS
        );
        assert_eq!(MIX_HALF, 1i64 << (MIX_SHIFT - 1));
        assert_eq!(
            i64::from(MONO_MIX_FADE_START),
            i64::from(MONO_MIX_WHITE) - MIX_FADE_WIDTH
        );
        // The last level with no fade is MONO_MIX_FADE_START, the next fades.
        let q = [0, 1 << 30, -(1 << 30)];
        let t = MONO_MIX_FADE_START;
        assert_eq!(mono_mix_pixel(&q, [0, t, t]), 0);
        assert_ne!(mono_mix_pixel(&q, [0, t + 1, t + 1]), 0);
        // A unit weight on an identity entry is 2^MONO_MIX_ROW_FRACTION_BITS.
        assert_eq!(
            mix_row_of([1.0, 0.0, 0.0], &IDENTITY).unwrap(),
            [1 << MONO_MIX_ROW_FRACTION_BITS, 0, 0]
        );
    }

    /// Verify (`SPEC-092`, HANDOFF-203): the kernel rounds an exact tie up.
    /// Only the local corpus digests saw a round-half-down kernel; these ties
    /// put it in the CI suite. `q = 2^23` on `x = 1` is exactly half a code.
    #[test]
    fn the_kernel_rounds_an_exact_tie_up() {
        assert_eq!(mono_mix_pixel(&[1 << 23, 0, 0], [1, 0, 0]), 1);
        assert_eq!(mono_mix_pixel(&[1 << 23, 0, 0], [3, 0, 0]), 2);
        // With the fade engaged: m = 61439, so u = 4096 and the negative term
        // is weighed by one half. q = [0, 2^24, -2^24] on x = [0, 61439, 61438]
        // is 61439 - 61438/2 = 30720 exactly; x_B = 61437 gives the tie
        // 30720.5, which rounds up to 30721.
        assert_eq!(
            mono_mix_pixel(&[0, 1 << 24, -(1 << 24)], [0, 61439, 61438]),
            30720
        );
        assert_eq!(
            mono_mix_pixel(&[0, 1 << 24, -(1 << 24)], [0, 61439, 61437]),
            30721
        );
        for x in [[1u16, 0, 0], [3, 0, 0]] {
            assert_eq!(
                mono_mix_pixel(&[1 << 23, 0, 0], x),
                mix_oracle(&[1 << 23, 0, 0], x)
            );
        }
    }
}
