//! Demosaic — Bayer mosaic to 3-channel camera-space linear RGB (`SPEC-030`).
//!
//! STAGE-006 made the Bayer **mosaic plane** bit-exact (`plane::unpack_into`,
//! verified vs `dnglab --raw-checksum`) and parsed the CFA pattern into
//! [`crate::ifd::Sensor::color_filter`]. A mosaic carries one colour per
//! photosite; to make an image every pixel needs all three channels. This
//! module interpolates the two missing channels at each pixel.
//!
//! # The kernel — bilinear (`SPEC-030`), the baseline
//!
//! The simplest kernel that is still a real demosaic: each known channel is the
//! photosite's own sample, and each missing channel is the integer average of
//! its nearest same-colour neighbours, **rounded half up**, computed in `u32`
//! (two neighbours `(a+b+1)/2`, four `(a+b+c+d+2)/4`). The quality kernel
//! (clean-room Malvar-He-Cutler) swaps in at `SPEC-031` against this same
//! harness; its gradient term CAN overshoot and will clamp, but **bilinear
//! cannot** — an average of samples in `[0, 65535]` stays in range — so this
//! spec needs no clamp (recorded so `SPEC-031` does not inherit a false
//! assumption; spec `## Design` point 4).
//!
//! # Phase, not hard-coding
//!
//! The site of a pixel (which colour it samples, and where its same-colour
//! neighbours lie) is read from `Sensor.color_filter.pattern`, not hard-coded
//! to RGGB — so the same code demosaics any of the four 2×2 Bayer phases
//! (RGGB/BGGR/GRBG/GBRG). Both corpus files are RGGB; the oracle's synthetic
//! fixtures cover the other phases (an unverified phase path is a
//! `measurement-over-generalised` trap — STAGE-007 Design Notes). A CFA that is
//! not a 2×2 Bayer (an X-Trans 6×6, a non-Bayer layout) is
//! [`crate::Error::UnsupportedCfaPattern`], a later stage's work.
//!
//! # Border policy — phase-preserving reflect
//!
//! Off-image indices mirror about the edge pixel without repeating it
//! (`x < 0 → -x`, `x ≥ w → 2w-2-x`; same for `y`). Reflection preserves
//! even/odd parity, so it preserves the Bayer phase at edges and corners — the
//! reason a plain clamp-to-edge would be wrong here (it would read a
//! wrong-colour neighbour). A single reflection lands back in range for every
//! bilinear neighbour (offset 1) once a dimension is ≥ 2; a degenerate 1-wide
//! or 1-tall dimension is clamped so it cannot index out of range (the AC's
//! 1×1 case).
//!
//! # Allocation — `DEC-016`
//!
//! Like [`crate::plane::unpack_into`] and [`crate::develop::develop_into`],
//! [`demosaic_into`] takes a caller-owned `&mut [u16]` and allocates nothing:
//! `src` is the `width * height` mosaic, `dst` the `width * height * 3`
//! interleaved R, G, B output. Wrong lengths are typed errors, the one
//! precondition checked before writing.
//!
//! # Output — uncorrected camera-space linear RGB
//!
//! No white balance, no colour matrix, no display encode (all STAGE-008), so a
//! demosaiced image reads greenish and flat — the honest STAGE-007 milestone.
//! The first recognisable colour picture is STAGE-007 + STAGE-008 together.
//!
//! # Provenance
//!
//! Bilinear CFA interpolation is textbook (a fixed linear filter); no external
//! source was read. See `docs/provenance-ledger.md`. The Malvar-He-Cutler
//! provenance row lands with `SPEC-031`.

use crate::ifd::{Cfa, Sensor};
use crate::Error;

/// A photosite's colour, as the `CFAPattern` colour indices name them
/// (`0 = Red`, `1 = Green`, `2 = Blue`; `src/ifd.rs`'s [`Cfa`] doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Colour {
    Red,
    Green,
    Blue,
}

/// A validated 2×2 Bayer mosaic phase: the colour at each of the four
/// positions in the repeating block, indexed by `(row_odd, col_odd)`.
///
/// Constructed only by [`BayerPhase::from_cfa`], which accepts exactly the
/// RGGB family (one red, two diagonal greens, one blue) — so a value of this
/// type is a guarantee the mosaic is a demosaicable Bayer.
struct BayerPhase {
    top_left: Colour,
    top_right: Colour,
    bottom_left: Colour,
    bottom_right: Colour,
}

impl BayerPhase {
    /// Validate a [`Cfa`] as a 2×2 Bayer mosaic and read its phase, or `None`
    /// if it is not the RGGB family (`repeat_dim != [2, 2]`, the wrong colour
    /// multiset, or greens that are not diagonal).
    ///
    /// The colour of each position is resolved through `CFAPlaneColor`
    /// (`plane_color[pattern[i]]`), falling back to the pattern index itself
    /// when `plane_color` is the DNG default `[0, 1, 2]` identity or absent —
    /// so both corpus files (default plane colour) and a file that spells the
    /// mapping out read the same.
    fn from_cfa(cfa: &Cfa) -> Option<BayerPhase> {
        if cfa.repeat_dim != [2, 2] || cfa.pattern.len() != 4 {
            return None;
        }

        let colour_at_slot = |slot: usize| -> Option<Colour> {
            let plane_idx = *cfa.pattern.get(slot)?;
            // plane_color maps a plane index to a colour index; for the DNG
            // default [0, 1, 2] this is the identity, so the pattern value IS
            // the colour. Fall back to the pattern value when the mapping is
            // absent or does not cover this index.
            let colour_idx = cfa
                .plane_color
                .get(usize::from(plane_idx))
                .copied()
                .unwrap_or(plane_idx);
            match colour_idx {
                0 => Some(Colour::Red),
                1 => Some(Colour::Green),
                2 => Some(Colour::Blue),
                _ => None,
            }
        };

        let top_left = colour_at_slot(0)?;
        let top_right = colour_at_slot(1)?;
        let bottom_left = colour_at_slot(2)?;
        let bottom_right = colour_at_slot(3)?;

        let quad = [top_left, top_right, bottom_left, bottom_right];
        let count = |want: Colour| quad.iter().filter(|c| **c == want).count();
        if count(Colour::Red) != 1 || count(Colour::Green) != 2 || count(Colour::Blue) != 1 {
            return None;
        }

        // The two greens must be diagonal — the defining property of a Bayer
        // mosaic, and what lets a green pixel's horizontal neighbours be a
        // single colour (the bilinear green-site formula depends on it).
        let greens_main_diagonal = top_left == Colour::Green && bottom_right == Colour::Green;
        let greens_anti_diagonal = top_right == Colour::Green && bottom_left == Colour::Green;
        if !(greens_main_diagonal || greens_anti_diagonal) {
            return None;
        }

        Some(BayerPhase {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        })
    }

    /// The colour sampled at a pixel, from its row/column parity alone (the
    /// mosaic repeats every 2 pixels, so parity is the whole phase).
    fn colour(&self, row_odd: bool, col_odd: bool) -> Colour {
        match (row_odd, col_odd) {
            (false, false) => self.top_left,
            (false, true) => self.top_right,
            (true, false) => self.bottom_left,
            (true, true) => self.bottom_right,
        }
    }
}

/// Demosaic a Bayer `src` mosaic (`sensor.width * sensor.height` samples) into
/// `dst`, an interleaved R, G, B image (`sensor.width * sensor.height * 3`
/// samples), via a bilinear kernel driven by `sensor.color_filter`.
///
/// Both buffers are caller-owned and this function allocates nothing
/// (`DEC-016`). The output is **uncorrected camera-space linear RGB** — no
/// white balance, no colour matrix, no levels (STAGE-008) — so it reads
/// greenish/flat; that is the honest STAGE-007 milestone.
///
/// # Errors
///
/// - [`Error::NotAColorFilterArray`] if `sensor.color_filter` is `None` (a
///   monochrome plane has nothing to demosaic).
/// - [`Error::UnsupportedCfaPattern`] if the CFA is not a 2×2 Bayer of the
///   RGGB family.
/// - [`Error::DemosaicSourceWrongLength`] if `src.len() != width * height`.
/// - [`Error::RgbBufferWrongLength`] if `dst.len() != width * height * 3`.
///
/// Never panics: every index is bounds-checked and every average is computed in
/// `u32` with no overflow (`4 * 65535 + 2 < u32::MAX`; the result is `≤ 65535`).
pub fn demosaic_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error> {
    let cfa = sensor
        .color_filter
        .as_ref()
        .ok_or(Error::NotAColorFilterArray)?;
    let phase = BayerPhase::from_cfa(cfa).ok_or_else(|| Error::UnsupportedCfaPattern {
        repeat_dim: cfa.repeat_dim,
        pattern: cfa.pattern.clone(),
    })?;

    let width = sensor.width;
    let height = sensor.height;

    let pixels = u64::from(width).checked_mul(u64::from(height)).ok_or(
        Error::DemosaicSourceWrongLength {
            expected: u64::MAX,
            actual: src.len(),
        },
    )?;
    let expected_src = usize::try_from(pixels).map_err(|_| Error::DemosaicSourceWrongLength {
        expected: pixels,
        actual: src.len(),
    })?;
    if src.len() != expected_src {
        return Err(Error::DemosaicSourceWrongLength {
            expected: pixels,
            actual: src.len(),
        });
    }

    let rgb_len = pixels.checked_mul(3).ok_or(Error::RgbBufferWrongLength {
        expected: u64::MAX,
        actual: dst.len(),
    })?;
    let expected_dst = usize::try_from(rgb_len).map_err(|_| Error::RgbBufferWrongLength {
        expected: rgb_len,
        actual: dst.len(),
    })?;
    if dst.len() != expected_dst {
        return Err(Error::RgbBufferWrongLength {
            expected: rgb_len,
            actual: dst.len(),
        });
    }

    // One interleaved R, G, B triple per pixel. `as_chunks_mut::<3>` yields
    // exactly `pixels` fixed-size `[u16; 3]` chunks (dst.len() == pixels * 3,
    // checked above) and an empty remainder, so the pixel index is the chunk
    // index — no output-index arithmetic, no panicking slice write.
    let w = usize::try_from(width).unwrap_or(0);
    let (triples, _remainder) = dst.as_chunks_mut::<3>();
    for (pixel, [red, green, blue]) in triples.iter_mut().enumerate() {
        let y = u32::try_from(pixel.checked_div(w).unwrap_or(0)).unwrap_or(0);
        let x = u32::try_from(pixel.checked_rem(w).unwrap_or(0)).unwrap_or(0);
        let (r, g, b) = demosaic_pixel(&phase, src, width, height, x, y);
        *red = r;
        *green = g;
        *blue = b;
    }

    Ok(())
}

/// The bilinear RGB value at one pixel, phase-driven (`SPEC-030` Notes). The
/// known channel is the photosite's own sample; the two missing channels are
/// averages of same-colour neighbours (reflect border, round half up):
///
/// - **R / B site:** the other primary is the 4 diagonal neighbours, green is
///   the 4 orthogonal neighbours.
/// - **Green site:** its horizontal neighbours are one primary and its vertical
///   neighbours the other; which is which is read from the pattern, so the four
///   Bayer phases all work with this one code path.
fn demosaic_pixel(
    phase: &BayerPhase,
    src: &[u16],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
) -> (u16, u16, u16) {
    let row_odd = (y & 1) == 1;
    let col_odd = (x & 1) == 1;
    let here = phase.colour(row_odd, col_odd);

    let xi = i64::from(x);
    let yi = i64::from(y);
    let at = |dx: i64, dy: i64| {
        sample(
            src,
            width,
            height,
            xi.saturating_add(dx),
            yi.saturating_add(dy),
        )
    };

    let centre = at(0, 0);
    let left = at(-1, 0);
    let right = at(1, 0);
    let up = at(0, -1);
    let down = at(0, 1);
    let diag_orthogonal = [up, down, left, right];
    let diag_corners = [at(-1, -1), at(1, -1), at(-1, 1), at(1, 1)];

    match here {
        Colour::Red => (
            u32_to_u16(centre),
            avg4(diag_orthogonal),
            avg4(diag_corners),
        ),
        Colour::Blue => (
            avg4(diag_corners),
            avg4(diag_orthogonal),
            u32_to_u16(centre),
        ),
        Colour::Green => {
            // A green pixel's horizontal neighbours share one colour and its
            // vertical neighbours the other. Read which from the pattern (the
            // horizontal neighbour sits at the flipped column parity), so RGGB
            // and BGGR take the same path.
            let horizontal = phase.colour(row_odd, !col_odd);
            let green = u32_to_u16(centre);
            if horizontal == Colour::Red {
                (avg2(left, right), green, avg2(up, down))
            } else {
                (avg2(up, down), green, avg2(left, right))
            }
        }
    }
}

/// The sample at `(x, y)` with the off-image index reflected about the edge,
/// returned as `u32` (`0` if the computed index somehow does not resolve —
/// unreachable after a reflect into range, but never a panic).
fn sample(src: &[u16], width: u32, height: u32, x: i64, y: i64) -> u32 {
    let rx = reflect(x, width);
    let ry = reflect(y, height);
    match plane_index(width, rx, ry) {
        Some(index) => src.get(index).copied().map(u32::from).unwrap_or(0),
        None => 0,
    }
}

/// Reflect an index about the edge of a `len`-long axis, phase-preserving:
/// `i < 0 → -i`, `i ≥ len → 2·len - 2 - i`. A single reflection lands in
/// `[0, len)` for every bilinear neighbour (offset 1) once `len ≥ 2`; the
/// final `clamp` only engages for a degenerate `len == 1` axis (or `len == 0`,
/// which has no pixels), so a 1×1 mosaic cannot index out of range.
fn reflect(i: i64, len: u32) -> u32 {
    if len == 0 {
        return 0;
    }
    let len_i = i64::from(len);
    let reflected = if i < 0 {
        i.saturating_neg()
    } else if i >= len_i {
        len_i.saturating_mul(2).saturating_sub(2).saturating_sub(i)
    } else {
        i
    };
    let clamped = reflected.clamp(0, len_i.saturating_sub(1));
    u32::try_from(clamped).unwrap_or(0)
}

/// Row-major index into a `width`-wide plane, bounds-checked against
/// `usize`/`u64` overflow — the same shape as `develop::plane_index`.
fn plane_index(width: u32, x: u32, y: u32) -> Option<usize> {
    u64::from(y)
        .checked_mul(u64::from(width))
        .and_then(|v| v.checked_add(u64::from(x)))
        .and_then(|v| usize::try_from(v).ok())
}

/// Round-half-up mean of two samples, in `u32`: `(a + b + 1) / 2`. Inputs are
/// `≤ 65535`, so the sum cannot overflow `u32` and the result is `≤ 65535`.
fn avg2(a: u32, b: u32) -> u16 {
    let sum = a.saturating_add(b).saturating_add(1);
    u32_to_u16(sum.checked_div(2).unwrap_or(0))
}

/// Round-half-up mean of four samples, in `u32`: `(a + b + c + d + 2) / 4`.
/// `4 * 65535 + 2 = 262142 < u32::MAX`, so no overflow; the result is `≤ 65535`.
fn avg4(values: [u32; 4]) -> u16 {
    let [a, b, c, d] = values;
    let sum = a
        .saturating_add(b)
        .saturating_add(c)
        .saturating_add(d)
        .saturating_add(2);
    u32_to_u16(sum.checked_div(4).unwrap_or(0))
}

/// Narrow a value already proven `≤ 65535` to `u16`, saturating at `u16::MAX`
/// rather than ever panicking (the callers guarantee the bound).
fn u32_to_u16(v: u32) -> u16 {
    u16::try_from(v).unwrap_or(u16::MAX)
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

    #[test]
    fn reflect_is_the_identity_in_range() {
        for i in 0..5i64 {
            assert_eq!(reflect(i, 5), i as u32);
        }
    }

    #[test]
    fn reflect_mirrors_one_past_each_edge_without_repeating() {
        // len 5: index -1 mirrors to 1, index 5 mirrors to 3 (2*5-2-5).
        assert_eq!(reflect(-1, 5), 1);
        assert_eq!(reflect(5, 5), 3);
    }

    #[test]
    fn reflect_on_a_degenerate_axis_cannot_leave_range() {
        // len 1: every neighbour resolves to the single pixel 0, no panic.
        assert_eq!(reflect(-1, 1), 0);
        assert_eq!(reflect(1, 1), 0);
        assert_eq!(reflect(0, 1), 0);
    }

    #[test]
    fn avg_rounds_half_up() {
        assert_eq!(avg2(0, 1), 1); // (0+1+1)/2 = 1
        assert_eq!(avg2(2, 3), 3); // (2+3+1)/2 = 3
        assert_eq!(avg4([1, 1, 2, 2]), 2); // (1+1+2+2+2)/4 = 2
        assert_eq!(avg4([0, 0, 1, 1]), 1); // (0+0+1+1+2)/4 = 1
    }

    #[test]
    fn avg_cannot_overflow_at_full_scale() {
        assert_eq!(avg2(65535, 65535), 65535);
        assert_eq!(avg4([65535, 65535, 65535, 65535]), 65535);
    }
}
