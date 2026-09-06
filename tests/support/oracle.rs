//! Reusable analytic-oracle helpers — `SPEC-015`. Shared by
//! `tests/develop_oracle.rs`'s tier-A/tier-B checks and its two red-proofs.
//!
//! ⚠ **Deliberately reimplements NOTHING about `Orientation`.** Every helper
//! here operates on the crop window taken in RASTER ORDER, before any
//! orientation is applied — `SPEC-015`'s central design decision (see the
//! spec's "The design decision this spec rests on"). Only `ActiveArea` /
//! `DefaultCrop` rectangle arithmetic is resolved here, using DNG 1.7
//! Chapter 4's own stated defaults (`ActiveArea` absent → the whole plane;
//! `DefaultCropOrigin` absent → `(0, 0)`; `DefaultCropSize` absent → the rest
//! of the active area) — never `src/develop.rs`'s own resolution. Comparisons
//! against `develop_into`'s actual output use RANK correspondence
//! (`bound_check`) or a value histogram (`histogram`/`multiset_equal`), never
//! a positional/per-pixel map — see `DEC-020` for why that is what lets AC1
//! and AC3 avoid the eight-case orientation table: `Orientation` only ever
//! *permutes* positions, and `exact_affine`/`rounded_affine` are monotonic
//! (`AC4`), so pairing the i-th smallest raw sample with the i-th smallest
//! actual output sample (counting repeats) reconstructs the same pairing a
//! positional check would need the permutation table to state explicitly.
//!
//! These helpers assume VALID geometry (a real decodable file, or a
//! deliberately hand-built honest fixture) — hostile-input rejection is
//! `src/develop.rs`'s own job, already covered by
//! `hostile_geometry_does_not_panic` (`tests/develop.rs`) and not duplicated
//! here.
//!
//! ## Why a frequency-table merge, not a 47-megapixel sort
//!
//! `bound_check` and `multiset_equal` compare `develop_into`'s actual output
//! against an expectation via value → count FREQUENCY TABLEs over the
//! bounded 65536-value `u16` domain, never by sorting the full pixel array.
//! Measured to matter: the first version of this file sorted with a
//! closure-based `f64` comparator and a single tier-B test alone took
//! 91.78s; switching to `sort_unstable` on the raw `u16`s (still sorting
//! ~47M elements) only reduced that to 79.32s — the comparison SORT itself,
//! not the comparator, was `AC8`'s pre-registered 60s bound's dominant cost.
//!
//! ⚠ **A bare per-DISTINCT-value pairing is NOT a valid replacement for a
//! full sorted zip, and a first attempt here measured wrong.** It paired the
//! i-th smallest DISTINCT raw value against the i-th smallest DISTINCT
//! actual value, weighting only the truncation tally by count — but many raw
//! values legitimately collapse to the SAME output (everything below
//! `BlackLevel` maps to exactly `0`), so "is `0` a value that occurs" cannot
//! see "did the WRONG NUMBER of pixels collapse to `0`", which is exactly
//! the shape of a `BlackLevel` fault. It let `the_oracle_is_red_on_a_levels_fault`'s
//! own honest tree report a spurious infinite deviation. `bound_check`'s
//! actual mechanism is a two-pointer merge over RUNS (value, remaining
//! count) in each frequency table — the counting-sort equivalent of
//! `sort_unstable`-ing both arrays and zipping them rank-for-rank, weight
//! and all, in O(n + 65536) rather than O(n log n) and without ever
//! materializing a sorted ~47-megapixel array.

use std::collections::HashMap;

use irradiance::ifd::Sensor;

/// `[BlackLevel, WhiteLevel]`, DNG 1.7's own stated defaults applied: absent
/// `BlackLevel` is 0; absent `WhiteLevel` is `2^BitsPerSample - 1`.
pub fn resolve_levels(sensor: &Sensor) -> (u32, u32) {
    let black = sensor.black_level.unwrap_or(0);
    let white = sensor.white_level.unwrap_or_else(|| {
        2u32.checked_pow(sensor.bits_per_sample)
            .and_then(|max| max.checked_sub(1))
            .unwrap_or(u32::MAX)
    });
    (black, white)
}

/// The crop window's bounds in RAW-PLANE coordinates. Stops here: no
/// orientation is ever resolved by this module, by design.
#[derive(Debug, Clone, Copy)]
pub struct CropWindow {
    pub left: u32,
    pub top: u32,
    pub width: u32,
    pub height: u32,
}

/// `ActiveArea` → `DefaultCrop`, DNG 1.7 Chapter 4's own defaults — not
/// `src/develop.rs`'s `resolve_geometry`.
pub fn resolve_crop_window(sensor: &Sensor) -> CropWindow {
    let (active_left, active_top, active_width, active_height) = match sensor.active_area {
        Some(a) => (a.left, a.top, a.right - a.left, a.bottom - a.top),
        // Absent means "the whole raw plane is active" — DNG's own default.
        None => (0, 0, sensor.width, sensor.height),
    };
    let (origin_x, origin_y) = match sensor.default_crop_origin {
        Some(o) => (o.x, o.y),
        None => (0, 0),
    };
    let (width, height) = match sensor.default_crop_size {
        Some(s) => (s.width, s.height),
        // Absent means "the rest of the active area" — DNG's own default,
        // independent of the crop origin.
        None => (active_width, active_height),
    };
    CropWindow {
        left: active_left + origin_x,
        top: active_top + origin_y,
        width,
        height,
    }
}

/// The crop window's samples, taken from `plane` (the UNCROPPED,
/// UN-ORIENTED raw plane — `sensor.width * sensor.height` samples) in RASTER
/// ORDER: row by row, left to right, with NO orientation applied. This is
/// the fixed reference AC1/AC2/AC3 compare `develop_into`'s actual output
/// against.
pub fn crop_window_samples(sensor: &Sensor, plane: &[u16]) -> Vec<u16> {
    let window = resolve_crop_window(sensor);
    let plane_width = sensor.width as usize;
    let mut samples = Vec::with_capacity(window.width as usize * window.height as usize);
    for row in 0..window.height {
        let raw_y = (window.top + row) as usize;
        let start = raw_y * plane_width + window.left as usize;
        let end = start + window.width as usize;
        samples.extend_from_slice(&plane[start..end]);
    }
    samples
}

/// The exact real-valued affine map DNG 1.7 Chapter 4 defines for
/// `BlackLevel`/`WhiteLevel` normalization: `(clamp(raw, B, W) - B) * 65535 /
/// (W - B)`, computed in `f64`. Never rounded, and never `DEC-018`'s own
/// rounding rule — `SPEC-015 AC1`'s pre-registered bound is stated against
/// exactly this, so it is satisfied by ANY correct rounding rule and
/// violated by an incorrect map (the spec's "L2 is the move that keeps this
/// honest").
pub fn exact_affine(raw: u16, black: u32, white: u32) -> f64 {
    let raw = f64::from(raw);
    let black = f64::from(black);
    let white = f64::from(white);
    let clamped = raw.clamp(black, white);
    (clamped - black) * 65535.0 / (white - black)
}

/// `exact_affine`, rounded to the nearest integer — a GENERIC mathematical
/// rounding of the real value, not `DEC-018`'s own saturating-integer
/// formula. Ties are never observed on real data (`AC1`'s own measured
/// headroom: max deviation 0.499968, zero pixels at or above 0.5), so the
/// tie-breaking DIRECTION is immaterial in practice; `f64::round`'s
/// half-away-from-zero convention is as good as any other. Used by AC3
/// (the histogram property, which — unlike AC1 — is allowed to reference the
/// LEVELS mapping; only the ORIENTATION table is off-limits, `SPEC-015`'s
/// central decision) and by AC4 (distinct-level counts).
pub fn rounded_affine(raw: u16, black: u32, white: u32) -> u16 {
    let value = exact_affine(raw, black, white).round();
    // `exact_affine`'s range is [0.0, 65535.0] by construction (the clamp
    // already bounds `raw` to `[black, white]`), so this cast never
    // truncates a meaningfully out-of-range value; `.clamp` is defensive
    // rounding-error insurance only.
    value.clamp(0.0, 65535.0) as u16
}

/// A value → count histogram — the position-independent comparison AC3's
/// permutation property and AC4's injectivity check both rest on.
pub fn histogram(values: &[u16]) -> HashMap<u16, u32> {
    let mut counts = HashMap::new();
    for &value in values {
        *counts.entry(value).or_insert(0u32) += 1;
    }
    counts
}

/// A value → count frequency table over the FULL `u16` domain (65536
/// entries, index == value) — the fast, allocation-cheap alternative to
/// [`histogram`]'s `HashMap` at ~47-megapixel scale. See the module doc,
/// "Why frequency tables, not a 47-megapixel sort".
fn frequency_table(values: &[u16]) -> Vec<u32> {
    let mut counts = vec![0u32; 1 << 16];
    for &value in values {
        counts[usize::from(value)] += 1;
    }
    counts
}

/// AC1 (the per-pixel bound) and AC2 (the truncation trap), computed via a
/// RANK-preserving merge of two frequency tables rather than a positional
/// map, a full sort, or (a first, BROKEN attempt) a bare distinct-value
/// list. `Orientation` is a pure permutation of positions and `exact_affine`
/// is monotonic (strictly, in `[BlackLevel, WhiteLevel]` — `AC4`), so the
/// i-th smallest RAW sample (by rank, counting repeats) must correspond to
/// the i-th smallest ACTUAL output sample — exactly what sorting both
/// ~47-megapixel arrays and zipping them would compute, without the O(n log
/// n) sort. ⚠ A bare per-DISTINCT-value pairing (ignoring how many pixels
/// share each value) is NOT equivalent and was measured wrong: it let a
/// `BlackLevel + 64` fault hide, because many raw values below the true
/// `BlackLevel` all legitimately collapse to output `0` — a bare distinct-value
/// list cannot see that the WRONG NUMBER of pixels collapsed there, only that
/// `0` is a value that occurs. Weighting by run length (this function) is
/// what makes a MASS shift into an otherwise-valid bucket visible.
pub struct BoundCheck {
    /// `max |actual - exact_affine(expected)|` over every pixel — AC1's own
    /// quantity, pre-registered against `< 0.5`.
    pub max_deviation: f64,
    /// How many pixels do NOT equal the TRUNCATED map (`exact_affine(...).floor()`)
    /// — AC2's own quantity, pre-registered against `> 40%` of `total`.
    pub truncation_disagreements: usize,
    pub total: usize,
}

pub fn bound_check(
    actual_output: &[u16],
    crop_samples: &[u16],
    black: u32,
    white: u32,
) -> BoundCheck {
    assert_eq!(
        actual_output.len(),
        crop_samples.len(),
        "output and crop window must be the same size — develop_into applies a \
         permutation, never a resize"
    );
    let total = actual_output.len();

    let crop_freq = frequency_table(crop_samples);
    let actual_freq = frequency_table(actual_output);

    let mut max_deviation = 0.0f64;
    let mut truncation_disagreements = 0usize;

    // A two-pointer merge over RUNS (value, remaining-count), the
    // counting-sort equivalent of `sort_unstable`-ing both ~47-megapixel
    // arrays and zipping them rank-for-rank — never materializing or
    // comparison-sorting either one. `total` matching (asserted above)
    // guarantees both sides exhaust together.
    let mut raw = 0u32;
    let mut raw_remaining = crop_freq[0];
    let mut actual = 0u32;
    let mut actual_remaining = actual_freq[0];
    loop {
        while raw_remaining == 0 && raw < u32::from(u16::MAX) {
            raw += 1;
            raw_remaining = crop_freq[raw as usize];
        }
        while actual_remaining == 0 && actual < u32::from(u16::MAX) {
            actual += 1;
            actual_remaining = actual_freq[actual as usize];
        }
        if raw_remaining == 0 || actual_remaining == 0 {
            break; // both sides exhausted, in lockstep (equal totals)
        }

        let raw_value = u16::try_from(raw).expect("bounded by u16::MAX above");
        let actual_value = u16::try_from(actual).expect("bounded by u16::MAX above");
        let run = raw_remaining.min(actual_remaining);

        let expected = exact_affine(raw_value, black, white);
        let deviation = (f64::from(actual_value) - expected).abs();
        if deviation > max_deviation {
            max_deviation = deviation;
        }
        if f64::from(actual_value) != expected.floor() {
            truncation_disagreements += run as usize;
        }

        raw_remaining -= run;
        actual_remaining -= run;
    }

    BoundCheck {
        max_deviation,
        truncation_disagreements,
        total,
    }
}

/// Whether `a` and `b` hold the same multiset of values — a frequency-table
/// alternative to comparing two [`histogram`]s at ~47-megapixel scale (see
/// the module doc, "Why frequency tables, not a 47-megapixel sort").
pub fn multiset_equal(a: &[u16], b: &[u16]) -> bool {
    a.len() == b.len() && frequency_table(a) == frequency_table(b)
}

/// The count of distinct values in `values` — a fixed 65536-entry presence
/// bitmap rather than a `HashSet<u16>`, for the same reason `multiset_equal`
/// avoids a `HashMap` at ~47-megapixel scale: no hashing at all, just direct
/// indexing.
pub fn distinct_count(values: &[u16]) -> usize {
    let mut seen = vec![false; 1 << 16];
    let mut count = 0usize;
    for &value in values {
        let slot = seen
            .get_mut(usize::from(value))
            .expect("a u16 always indexes a 65536-entry table");
        if !*slot {
            *slot = true;
            count += 1;
        }
    }
    count
}
