//! Level normalization and geometry — `SPEC-014`.
//!
//! `src/plane.rs` produces a **correct, uncropped, un-normalised** `u16`
//! plane and `tests/plane_oracle.rs` (`SPEC-013`) asserts it bit-for-bit
//! against `dnglab --raw-checksum`. This module turns that plane into the
//! image a consumer would actually display: black subtracted, white
//! normalized, the three-stage `ActiveArea` → `DefaultCrop` → `Orientation`
//! geometry applied, in one pass over the output buffer.
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
//! caller-owned buffers, allocating nothing itself — the same shape
//! `DEC-016` chose for [`crate::plane::unpack_into`]. It is **not in-place**:
//! the destination is smaller than the source (crop) and may have swapped
//! dimensions (orientation 5-8), so a second buffer is unavoidable. Measured
//! via `irr develop` on `L1021223.DNG` (`AC7`): peak RSS **275,890,176
//! bytes** — `SPEC-012`'s already-measured 182,435,840 (file + raw plane,
//! `DEC-016`) plus the 93,453,824-byte developed image (8368×5584×2), to
//! within rounding. `develop_into`'s own working memory is `O(1)`, not
//! `O(pixels)` — the added cost is entirely the caller's second buffer, not
//! anything this function allocates. See `DEC-018`'s Consequences.

use crate::ifd::Sensor;
use crate::Error;

/// Resolved geometry, defaults applied, every rectangle validated to fit.
struct Geometry {
    /// `ActiveArea`'s origin, in raw-plane coordinates (`(0, 0)` when the tag
    /// is absent — `SPEC-014` `AC3`'s `L1000622.DNG` case).
    active_left: u32,
    active_top: u32,
    /// `DefaultCropOrigin`, relative to `ActiveArea`'s origin (`AC4`;
    /// `DEC-019`). `(0, 0)` when the tag is absent.
    crop_origin_x: u32,
    crop_origin_y: u32,
    /// `DefaultCropSize`. Defaults to the full active area when the tag is
    /// absent (the DNG specification's own stated default) — untested by any
    /// corpus file, since every decodable file carries an explicit size; only
    /// `hostile_geometry_does_not_panic`'s hand-built fixture reaches it.
    crop_width: u32,
    crop_height: u32,
    /// `Orientation`, defaulted to 1 (normal) when absent, validated to
    /// `1..=8`.
    orientation: u32,
}

/// Resolve `ActiveArea` → `DefaultCrop` → `Orientation`, applying DNG's
/// stated defaults for each absent tag and rejecting every hostile shape
/// `AC6` names: an inverted or out-of-plane `ActiveArea`, a crop that does
/// not fit inside it (including zero-sized), and an `Orientation` outside
/// `1..=8`.
fn resolve_geometry(sensor: &Sensor) -> Result<Geometry, Error> {
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
        crop_origin_x,
        crop_origin_y,
        crop_width,
        crop_height,
        orientation,
    })
}

/// The developed image's dimensions: `(crop_width, crop_height)`, swapped for
/// an `Orientation` that rotates 90° (5-8) — `AC5`.
fn oriented_dimensions(orientation: u32, width: u32, height: u32) -> (u32, u32) {
    match orientation {
        1..=4 => (width, height),
        // 5-8 (validated to 1..=8 by `resolve_geometry`'s caller).
        _ => (height, width),
    }
}

/// The developed image's dimensions the caller must size `dst` to, without
/// running the transform itself — mirrors how a caller of
/// [`crate::plane::unpack_into`] computes `sensor.width * sensor.height`
/// itself before allocating.
///
/// # Errors
///
/// Whatever [`develop_into`] would reject about `sensor`'s geometry — see its
/// docs. Never fails on `src`/`dst`, because neither is examined here.
pub fn output_dimensions(sensor: &Sensor) -> Result<(u32, u32), Error> {
    let geometry = resolve_geometry(sensor)?;
    Ok(oriented_dimensions(
        geometry.orientation,
        geometry.crop_width,
        geometry.crop_height,
    ))
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

/// `BlackLevel`/`WhiteLevel`, defaults applied, validated to a non-empty
/// range.
///
/// `WhiteLevel`'s DNG default when absent is `2^BitsPerSample - 1` — the
/// widest range the sample width can represent. Untested by any corpus file
/// (every decodable file carries an explicit `WhiteLevel`); only a hand-built
/// fixture reaches it.
fn resolve_levels(sensor: &Sensor) -> Result<(u32, u32), Error> {
    let black = sensor.black_level.unwrap_or(0);
    let white = sensor
        .white_level
        .unwrap_or_else(|| max_value_for_bits(sensor.bits_per_sample).unwrap_or(u32::MAX));
    if black >= white {
        return Err(Error::InvalidLevels {
            black_level: black,
            white_level: white,
        });
    }
    Ok((black, white))
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

/// Develop the sensor plane [`crate::plane::unpack_into`] produced into the
/// image a consumer would display: levels normalized, the three-stage crop
/// applied, oriented.
///
/// `src` must hold exactly `sensor.width * sensor.height` samples (the
/// uncropped, un-normalised plane) and `dst` must hold exactly
/// [`output_dimensions`]`(sensor)`'s product — the caller computes both, the
/// same caller-owned-buffer shape `DEC-016` chose for `unpack_into`
/// (`DEC-018`'s Consequences: this is **not in-place**, `dst` is a second
/// buffer).
///
/// # Errors
///
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
pub fn develop_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error> {
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

    let geometry = resolve_geometry(sensor)?;
    let (black, white) = resolve_levels(sensor)?;
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
                *slot = normalize(sample, black, white);
            }
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
            black_level: Some(0),
            white_level: Some(16383),
            black_level_repeat_dim: None,
            active_area: None,
            default_crop_origin: None,
            default_crop_size: None,
            orientation: None,
            opcode_lists: [false, false, false],
            malformed_tags: vec![],
        }
    }

    #[test]
    fn normalize_maps_the_endpoints_and_an_interior_point() {
        assert_eq!(normalize(512, 512, 16383), 0);
        assert_eq!(normalize(16383, 512, 16383), u16::MAX);
        // Interior: exactly the midpoint rounds to the midpoint.
        let mid = 512u32.checked_add(16383).unwrap() / 2;
        let got = normalize(u16::try_from(mid).unwrap(), 512, 16383);
        assert!((32000..=33500).contains(&got), "got {got}");
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
