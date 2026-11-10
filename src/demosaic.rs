//! Demosaic — Bayer mosaic to 3-channel camera-space linear RGB.
//!
//! STAGE-006 made the Bayer **mosaic plane** bit-exact (`plane::unpack_into`,
//! verified vs `dnglab --raw-checksum`) and parsed the CFA pattern into
//! [`crate::ifd::Sensor::color_filter`]. A mosaic carries one colour per
//! photosite; to make an image every pixel needs all three channels. This
//! module interpolates the two missing channels at each pixel.
//!
//! # The kernel — Hamilton-Adams directional interpolation (`SPEC-031`)
//!
//! `SPEC-030` shipped a bilinear baseline (each missing channel the average of
//! its nearest same-colour neighbours). `SPEC-031` replaces that interpolation
//! with the **Adams-Hamilton** (Hamilton-Adams) directional demosaic, a real
//! quality step: directional green removes most of bilinear's edge zippering and
//! false colour. Per **`DEC-031`** the algorithm is Hamilton-Adams
//! (Hamilton & Adams, **US 5,629,734, Eastman Kodak — expired**), **not**
//! Malvar-He-Cutler, which the project originally named: MHC is covered by the
//! still-live **US 7,502,505** (Microsoft), and a patent-encumbered method in a
//! permissive crate defeats the whole point of this library — clean-room beats
//! copyright, not patents. Hamilton-Adams is patent-clean and ecosystem-standard
//! (dcraw / LibRaw / RawTherapee / darktable ship it). See
//! `docs/provenance-ledger.md`.
//!
//! Two passes, green first (load-bearing for the bit-exact oracle):
//!
//! 1. **Green at every pixel.** At a green site, green is the photosite's own
//!    sample. At a red/blue site, a gradient classifier picks the lower-gradient
//!    axis and interpolates green along it with a same-colour second-difference
//!    correction. The whole green plane is finalized to `u16` before pass 2.
//! 2. **Red and blue**, reading the finalized green. At a green site, each
//!    missing primary comes from its two same-colour neighbours by a
//!    colour-difference (`primary − green`) interpolation. At a red/blue site the
//!    opposite primary comes from the four diagonal neighbours, gradient-directed
//!    to the lower-gradient diagonal.
//!
//! All intermediates are signed `i32` (corrections can go negative); each output
//! channel rounds half up with floor division (`rnd4(n) = ⌊(n+2)/4⌋`,
//! `rnd8(n) = ⌊(n+4)/8⌋` — floor, so negative numerators round correctly) and
//! clamps to `[0, 65535]`. Unlike bilinear, HA's predictor **can** overshoot the
//! input range, so the clamp is load-bearing (`SPEC-030` recorded this so this
//! spec would not inherit a false "no clamp needed" assumption; `DEC-018`).
//!
//! # Phase, not hard-coding
//!
//! The site of a pixel (which colour it samples, and where its same-colour
//! neighbours lie) is read from `Sensor.color_filter.pattern`, not hard-coded to
//! RGGB — so the same code demosaics any of the four 2×2 Bayer phases
//! (RGGB/BGGR/GRBG/GBRG). Both corpus files are RGGB; the oracle's synthetic
//! fixtures cover BGGR (an unverified phase path is a `measurement-over-generalised`
//! trap — STAGE-007 Design Notes). A CFA that is not a 2×2 Bayer (an X-Trans
//! 6×6, a non-Bayer layout) is [`crate::Error::UnsupportedCfaPattern`], a later
//! stage's work.
//!
//! # Border policy — phase-preserving reflect
//!
//! Off-image indices mirror about the edge pixel without repeating it
//! (`x < 0 → -x`, `x ≥ w → 2w-2-x`; same for `y`). Reflection preserves
//! even/odd parity, so it preserves the Bayer phase at edges and corners — the
//! reason a plain clamp-to-edge would be wrong here (it would read a
//! wrong-colour neighbour). HA samples at **offset 2** (`WW`/`EE`/`NN`/`SS`),
//! so a single reflection of `-2` / `len+1` lands back in range only once a
//! dimension is `≥ 3`; on a shorter axis the final clamp in [`reflect`] engages
//! so a degenerate 1- or 2-wide mosaic still cannot index out of range (safe,
//! no panic — the AC's 1×1 / 2×2 / 3×5 cases).
//!
//! # Performance — interior fast path (`DEC-041`)
//!
//! Reflection only matters within two pixels of an edge. For the **interior**
//! (every pixel at least two rows and two columns from each edge) a neighbour
//! read at offset ±1 or ±2 is always in range, so [`demosaic_into`] reads those
//! neighbours by **direct index into per-row slices** of `src` (and, in pass 2,
//! the finalized greens in `dst`), skipping [`reflect`], [`plane_index`] and the
//! per-pixel `(x, y)` recovery that the border path still pays for. The
//! **kernel arithmetic is reused verbatim** — the same classifiers, second
//! differences, predictors, rounding and clamp (`green_from_neighbours` /
//! `diagonal_from_neighbours` / [`colour_difference`]) applied to the same
//! neighbour values in the same order — so the output is **bit-identical** to
//! the all-reflecting implementation (the existing golden hashes are the gate,
//! and `interior_fast_path_matches_the_reference_demosaic` is the proof). The
//! per-row windows make the interior in-bounds **by construction**, so the fast
//! path stays panic-free with no `unsafe` and no per-element bounds check.
//!
//! # Allocation — `DEC-016`
//!
//! Like [`crate::plane::unpack_into`] and [`crate::develop::develop_into`],
//! [`demosaic_into`] takes a caller-owned `&mut [u16]` and allocates nothing.
//! The two passes need a complete green plane between them; rather than allocate
//! one, pass 1 writes each pixel's green into its own green slot of `dst`
//! (`dst[pixel * 3 + 1]`), and pass 2 reads greens back from there while it
//! fills the red and blue slots — which it never reads — so the two slot sets
//! are disjoint and no scratch buffer is needed.
//!
//! # Output — uncorrected camera-space linear RGB
//!
//! No white balance, no colour matrix, no display encode (all STAGE-008), so a
//! demosaiced image reads greenish and flat — the honest STAGE-007 milestone.
//! The first recognisable colour picture is STAGE-007 + STAGE-008 together.
//!
//! # Provenance
//!
//! Adams-Hamilton (US 5,629,734, expired; `DEC-031`), implemented from the
//! method as re-expressed in `SPEC-031`'s Design section — no implementation was
//! read (`dnglab`/`rawler` are LGPL-2.1 and emit only a full colour render; they
//! implement no isolable demosaic step, `DEC-024`). See
//! `docs/provenance-ledger.md`.

use std::sync::{Mutex, PoisonError};

use crate::ifd::{active_area_origin, Cfa, Sensor};
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
        // single colour (the green-site colour-difference depends on it).
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
/// samples), via the Hamilton-Adams directional kernel driven by
/// `sensor.color_filter` (`SPEC-031`, `DEC-031`).
///
/// Both buffers are caller-owned and this function allocates nothing
/// (`DEC-016`): the complete green plane the two passes share lives in `dst`'s
/// own green slots between them. The output is **uncorrected camera-space
/// linear RGB** — no white balance, no colour matrix, no levels (STAGE-008) —
/// so it reads greenish/flat; that is the honest STAGE-007 milestone.
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
/// Never panics: the interior fast path reads each neighbour from a fixed-size
/// per-row window (sliced once, in-bounds by construction — `DEC-041`), and the
/// 2-pixel border stays on the bounds-checked reflecting path
/// (`.get`/`.get_mut`); every intermediate is signed `i32` via
/// saturating/checked operations whose inputs are provably far from `i32`'s
/// range at 16-bit sample depth.
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

    let w = usize::try_from(width).unwrap_or(0);
    let h = usize::try_from(height).unwrap_or(0);

    // The CFA phase repeats from the `ActiveArea` origin, not the plane's
    // top-left (`DEC-040`): index each pixel's colour relative to it so a file
    // with an odd `ActiveArea.top`/`.left` is assigned the right colour. `(0, 0)`
    // when the tag is absent, so an even origin (the whole corpus) is unchanged.
    // `normalize_mosaic` (`src/color.rs`) uses the same origin for the level slot.
    let (active_top, active_left) = active_area_origin(sensor);

    // Nothing to interpolate (also guards the `chunks_mut(w)` zero-width case
    // below, which would otherwise panic). A zero-area plane has already passed
    // the length checks, so returning here leaves `dst` correct and untouched.
    if expected_src == 0 || w == 0 || h == 0 {
        return Ok(());
    }

    // The phase parity relative to the `ActiveArea` origin is a low-bit XOR:
    // `(coord − origin) mod 2 == (coord mod 2) XOR (origin mod 2)`. The slow
    // (`green_at_site`/`interpolate_red_blue`) path recomputes the same parity
    // from the `(u32, u32)` site with `wrapping_sub` — the two agree bit for
    // bit, so a fast-path interior pixel and a slow-path border pixel are
    // assigned the same colour (`DEC-040`, SPEC-047 — kept intact).
    let top_odd = (active_top & 1) == 1;
    let left_odd = (active_left & 1) == 1;

    // The interior is every pixel at least two rows and two columns from each
    // edge — the only region where no neighbour read (offset up to ±2) can
    // leave the plane, so `reflect()` is never needed there. Interior columns
    // are `2..col_interior_end`, interior rows `2..row_interior_end`; both are
    // empty when the axis is shorter than five, which leaves that whole pass on
    // the reflecting border path — still bit-identical to before (`DEC-041`).
    let col_interior_end = w.saturating_sub(2);
    let row_interior_end = h.saturating_sub(2);

    // ── Pass 1 — green at every pixel, into each pixel's own green slot. ──────
    // Row-major over `dst`'s pixel triples (`as_chunks_mut::<3>` then
    // `chunks_mut(w)`) so `y` is the row index, computed once per row rather
    // than via a per-pixel `checked_div`/`checked_rem`. Pass 1 reads only `src`,
    // so the mutable row borrow does not conflict with anything.
    let (triples, _remainder) = dst.as_chunks_mut::<3>();
    for (y, row) in triples.chunks_mut(w).enumerate() {
        let row_odd = ((y & 1) == 1) ^ top_odd;
        let y_interior = y >= 2 && y < row_interior_end;
        for (x, [_red, green, _blue]) in row.iter_mut().enumerate() {
            let col_odd = ((x & 1) == 1) ^ left_odd;
            *green = if y_interior && x >= 2 && x < col_interior_end {
                pass1_green_interior(&phase, src, w, x, y, (row_odd, col_odd)).unwrap_or_else(
                    || {
                        green_at_site(
                            &phase,
                            src,
                            width,
                            height,
                            (u32_from(x), u32_from(y)),
                            (active_top, active_left),
                        )
                    },
                )
            } else {
                green_at_site(
                    &phase,
                    src,
                    width,
                    height,
                    (u32_from(x), u32_from(y)),
                    (active_top, active_left),
                )
            };
        }
    }

    // ── Pass 2 — red and blue, reading pass 1's finalized green from `dst`. ──
    // Row-major with `y` the loop variable (again no per-pixel `(x, y)`
    // recovery). Each pixel reads greens from `dst` *immutably* and then writes
    // its own red/blue slots; the immutable reads complete before the mutable
    // writes, and the two slot sets are disjoint, so there is no aliasing — the
    // same discipline the per-pixel loop used before.
    for y in 0..h {
        let row_odd = ((y & 1) == 1) ^ top_odd;
        let y_interior = y >= 2 && y < row_interior_end;
        let row_base = y.checked_mul(w);
        for x in 0..w {
            let col_odd = ((x & 1) == 1) ^ left_odd;
            let (red, blue) = if y_interior && x >= 2 && x < col_interior_end {
                pass2_red_blue_interior(&phase, src, dst, w, x, y, (row_odd, col_odd))
                    .unwrap_or_else(|| {
                        interpolate_red_blue(
                            &phase,
                            src,
                            dst,
                            width,
                            height,
                            (u32_from(x), u32_from(y)),
                            (active_top, active_left),
                        )
                    })
            } else {
                interpolate_red_blue(
                    &phase,
                    src,
                    dst,
                    width,
                    height,
                    (u32_from(x), u32_from(y)),
                    (active_top, active_left),
                )
            };
            let base = row_base
                .and_then(|rb| rb.checked_add(x))
                .and_then(|pixel| pixel.checked_mul(3));
            if let Some(b) = base {
                if let Some(slot) = dst.get_mut(b) {
                    *slot = red;
                }
                if let Some(slot) = b.checked_add(2).and_then(|i| dst.get_mut(i)) {
                    *slot = blue;
                }
            }
        }
    }

    Ok(())
}

/// [`demosaic_into`] with opt-in **output-row-band** parallelism (`SPEC-051`,
/// `DEC-044`): the same Hamilton-Adams render, byte for byte, split across
/// `threads` scoped threads.
///
/// `threads <= 1` is a straight call to [`demosaic_into`] — no banding, no
/// allocation, the default every existing caller keeps. With `threads >= 2` the
/// output rows are split into `min(threads, height)` near-equal contiguous bands,
/// each thread owning a **disjoint** `dst` slice (`split_at_mut`, so the borrow
/// checker proves no two threads write the same sample). Each band runs the fused
/// traversal (`develop_color_fused`) — which the colour oracle already proves
/// equal to [`demosaic_into`] — over its own rows plus a three-row halo of the
/// shared, read-only `src`, with a **local** green buffer (the single-thread path
/// keeps green in `dst`'s green slots, which a neighbour band would be writing).
/// Reflection happens only at the true image edges, never at a band seam, so the
/// output is byte-identical to [`demosaic_into`] and independent of `threads`
/// and of scheduling. The local green buffers sum to about one `width·height`
/// plane; that is the only extra memory, and only when `threads >= 2`.
///
/// # Errors
///
/// Exactly [`demosaic_into`]'s, in the same order: every input the threaded path
/// does not accept (a non-CFA plane, an unsupported mosaic, a wrong-length
/// buffer) is handed to [`demosaic_into`], which returns its typed error.
///
/// Never panics: inputs are validated before any thread is spawned, a thread
/// that cannot be spawned just leaves its band to the threads that were (the
/// calling thread always works too), and the kernels are the panic-free
/// single-thread ones.
pub fn demosaic_into_threaded(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    threads: usize,
) -> Result<(), Error> {
    if threads <= 1 {
        return demosaic_into(sensor, src, dst);
    }
    // Anything the banded path does not take — an input `demosaic_into` rejects
    // (so its typed error comes back unchanged) or a plane too short to split —
    // goes straight to the single-thread function.
    let Some((w, h)) = demosaic_band_plan(sensor, src.len(), dst.len(), threads) else {
        return demosaic_into(sensor, src, dst);
    };
    let width = sensor.width;
    let normalize = |x: u32, y: u32| -> u16 {
        plane_index(width, x, y)
            .and_then(|i| src.get(i))
            .copied()
            .unwrap_or(0)
    };
    let (triples, _remainder) = dst.as_chunks_mut::<3>();
    run_row_bands(triples, w, h, threads, |y0, y1, band| {
        // Output rows are source rows (1:1), so the band's first pixel is
        // plane pixel `y0·w` and a developed pixel lands at `pixel − y0·w`.
        let first = y0.saturating_mul(w);
        develop_rows_banded(sensor, (y0, y1), &normalize, |pixel, r, g, b| {
            if let Some(out) = pixel.checked_sub(first).and_then(|i| band.get_mut(i)) {
                *out = [r, g, b];
            }
        })
    })
}

/// The `(width, height)` [`demosaic_into_threaded`] bands over, or `None` when
/// the input belongs on the single-thread path: not a supported Bayer CFA, a
/// buffer of the wrong length, or fewer than two rows (or threads) to split.
fn demosaic_band_plan(
    sensor: &Sensor,
    src_len: usize,
    dst_len: usize,
    threads: usize,
) -> Option<(usize, usize)> {
    if !is_supported_bayer(sensor) {
        return None;
    }
    let w = usize::try_from(sensor.width).ok()?;
    let h = usize::try_from(sensor.height).ok()?;
    let pixels = w.checked_mul(h)?;
    let usable = src_len == pixels
        && pixels.checked_mul(3) == Some(dst_len)
        && w > 0
        && core::cmp::min(threads, h) >= 2;
    usable.then_some((w, h))
}

/// Whether `sensor` carries a CFA [`demosaic_into`] / [`develop_color_fused`]
/// accept (a 2×2 RGGB-family Bayer) — the threaded paths' pre-spawn check, so a
/// band can never fail on the CFA.
pub(crate) fn is_supported_bayer(sensor: &Sensor) -> bool {
    sensor
        .color_filter
        .as_ref()
        .and_then(BayerPhase::from_cfa)
        .is_some()
}

/// Rows of halo a band develops beyond each side of its own rows (`DEC-044`):
/// an output row reads green one row away, and that green reads mosaic samples
/// two rows further — so three rows of real neighbours make a band's pixels
/// independent of where the band (and its local reflection) ends.
const BAND_HALO: usize = 3;

/// Run the fused traversal [`develop_color_fused`] over source rows `s0..s1`
/// only (plus [`BAND_HALO`] rows each side, clamped at the true image edges),
/// with a local green buffer, and `emit` each developed pixel of `s0..s1` by its
/// **whole-plane** pixel index — the per-band worker of the threaded entry
/// points (`SPEC-051`, `DEC-044`).
///
/// The halo rows are developed against a local plane whose top/bottom are the
/// halo's edges, so their own values may reflect wrongly — but they are never
/// emitted, and no emitted pixel reads them beyond the first halo row's
/// correctly-sourced samples. Where the halo is clamped, the local edge **is** the
/// true edge, so reflection there is the single-thread reflection. The local
/// top `lo` is rounded down to an even row so every local row has its absolute
/// row's Bayer parity (the `ActiveArea`-origin phase, `DEC-040`, is unchanged).
/// So every emitted pixel equals the whole-plane render bit for bit.
///
/// `normalize(x, y)` is keyed by whole-plane coordinates. Returns
/// [`develop_color_fused`]'s errors (none after the caller's pre-spawn checks).
pub(crate) fn develop_rows_banded<N, Sink>(
    sensor: &Sensor,
    rows: (usize, usize),
    normalize: &N,
    mut emit: Sink,
) -> Result<(), Error>
where
    N: Fn(u32, u32) -> u16,
    Sink: FnMut(usize, u16, u16, u16),
{
    let (s0, s1) = rows;
    let w = usize::try_from(sensor.width).unwrap_or(0);
    let h = usize::try_from(sensor.height).unwrap_or(0);
    let lo = s0.saturating_sub(BAND_HALO) & !1;
    let hi = core::cmp::min(h, s1.saturating_add(BAND_HALO));
    let local_rows = hi.saturating_sub(lo);

    // A view of the plane that starts at row `lo` and is `local_rows` tall. Only
    // the height differs; the CFA, width and ActiveArea origin are the plane's,
    // and `lo` is even, so the phase of every local row is its absolute row's.
    let mut local = sensor.clone();
    local.height = u32::try_from(local_rows).unwrap_or(0);
    let mut green = vec![0u16; w.saturating_mul(local_rows)];
    let lo_row = u32_from(lo);
    let local_normalize = |x: u32, y: u32| normalize(x, y.saturating_add(lo_row));

    let offset = lo.saturating_mul(w);
    let keep = s0.saturating_mul(w)..s1.saturating_mul(w);
    develop_color_fused(&local, &mut green, &local_normalize, || {
        Ok(move |pixel: usize, r: u16, g: u16, b: u16| {
            let whole = pixel.saturating_add(offset);
            if keep.contains(&whole) {
                emit(whole, r, g, b);
            }
        })
    })
}

/// Split `dst` (`rows` rows of `row_len` items) into `min(threads, rows)`
/// contiguous, near-equal row bands and run `work(y0, y1, band)` on each, over
/// `std::thread::scope` (`SPEC-051`, `DEC-044`). Each band is a **disjoint**
/// `&mut` slice carved with `split_at_mut_checked`, so no two threads can touch
/// the same item; the band edges are a pure function of `(rows, threads)`.
///
/// The bands sit in a shared queue that the spawned threads **and the calling
/// thread** drain, so a thread the OS refuses to spawn (`Builder::spawn_scoped`
/// returns an error rather than panicking) only means fewer workers — every band
/// is still run exactly once, and the output is the same. The first error by
/// band order is returned (deterministic); a panic inside `work` (none of the
/// kernels can panic) is re-raised on join rather than swallowed.
pub(crate) fn run_row_bands<T, F>(
    dst: &mut [T],
    row_len: usize,
    rows: usize,
    threads: usize,
    work: F,
) -> Result<(), Error>
where
    T: Send,
    F: Fn(usize, usize, &mut [T]) -> Result<(), Error> + Sync,
{
    let bands = core::cmp::max(core::cmp::min(threads, rows), 1);
    let mut queue = Vec::with_capacity(bands);
    let mut rest = dst;
    for i in 0..bands {
        let y0 = band_edge(rows, bands, i);
        let y1 = band_edge(rows, bands, i.saturating_add(1));
        let len = y1.saturating_sub(y0).saturating_mul(row_len);
        let remaining = rest.len();
        let Some((band, tail)) = rest.split_at_mut_checked(len) else {
            // Unreachable: every caller sized `dst` to `rows · row_len`.
            return Err(Error::RgbBufferWrongLength {
                expected: u64::try_from(rows.saturating_mul(row_len)).unwrap_or(u64::MAX),
                actual: remaining,
            });
        };
        queue.push((i, y0, y1, band));
        rest = tail;
    }

    let queue = Mutex::new(queue);
    let next_band = || queue.lock().unwrap_or_else(PoisonError::into_inner).pop();
    let drain = || {
        let mut errors = Vec::new();
        while let Some((i, y0, y1, band)) = next_band() {
            if let Err(e) = work(y0, y1, band) {
                errors.push((i, e));
            }
        }
        errors
    };

    let mut errors = std::thread::scope(|scope| {
        let drain = &drain;
        let handles: Vec<_> = (1..bands)
            .filter_map(|_| std::thread::Builder::new().spawn_scoped(scope, drain).ok())
            .collect();
        let mut errors = drain();
        for handle in handles {
            match handle.join() {
                Ok(more) => errors.extend(more),
                Err(payload) => std::panic::resume_unwind(payload),
            }
        }
        errors
    });
    errors.sort_by_key(|(i, _)| *i);
    match errors.into_iter().next() {
        Some((_, e)) => Err(e),
        None => Ok(()),
    }
}

/// Row `⌊rows · i / bands⌋` — the start of band `i` (and the end of band
/// `i − 1`). Computed in `u128` so it cannot overflow; `bands ≥ 1`.
fn band_edge(rows: usize, bands: usize, i: usize) -> usize {
    let edge = (rows as u128)
        .saturating_mul(i as u128)
        .checked_div(bands as u128)
        .unwrap_or(0);
    usize::try_from(edge).unwrap_or(rows)
}

/// Narrow a loop index to `u32` for the reflecting (border) path, saturating to
/// `0` on the impossible overflow (a plane dimension is a `u32`, so every pixel
/// coordinate fits). Keeps the border call panic-free without an `unwrap`.
fn u32_from(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(0)
}

/// The **fused** colour-develop demosaic (`SPEC-049`, `DEC-042`): the same
/// Hamilton-Adams kernel as [`demosaic_into`], but driven as a single
/// output-row sweep that normalises each mosaic sample **exactly once** into a
/// small rolling ring of normalised rows, keeps the finalised green in a
/// caller-owned `w·h` `green` plane (not in `dst`), and hands each pixel's
/// camera-RGB triple to `emit` rather than writing a `camera_rgb` intermediate.
/// [`crate::color::develop_color_into`] supplies `normalize` (per-CFA-slot level
/// normalisation) and an `emit` that applies the colour matrix straight into
/// `dst`, so neither the `normalized` mosaic nor the `camera_rgb` buffer is
/// allocated — only the `w·h` green plane and the `RING`-row cache.
///
/// **Why a rolling cache (`DEC-042`, revised).** The first cut normalised on
/// every neighbour read (up to ~9× per photosite), which kept the memory win
/// but regressed develop wall-time ~2.76×. The eliminated `camera_rgb` buffer is
/// the whole memory win; re-normalising bought none of it. So this revision
/// normalises each photosite once — when its row first enters the ring — and
/// both the green pass and the red/blue pass read the cached value. The ring
/// holds the few rows a single output row touches (y−1 … y+3, inside `RING`),
/// so a row's residue slot never collides with a still-live row.
///
/// **Output-invariant.** A cached cell equals `normalize(col, row)`, which is
/// exactly the value [`crate::color`]'s pre-fuse `normalize_mosaic` wrote at the
/// plane index `row·width + col`; the green pass and the red/blue interpolation
/// feed the **same** shared kernel helpers ([`green_from_neighbours`],
/// [`diagonal_from_neighbours`], [`colour_difference`]) on the same neighbours in
/// the same order as [`demosaic_into`]. The interior reads the cache by direct
/// index (the `DEC-041` fast path, over normalised rows), the two-pixel border
/// falls back to [`fused_green_at_site`] / [`fused_interpolate_red_blue`] (which
/// recompute `normalize` — negligibly many pixels), and both agree with
/// [`demosaic_into`] bit for bit. So the fused `u16` render equals the pre-fuse
/// `normalize → demosaic_into → matrix` pipeline byte for byte.
///
/// `normalize` is keyed by the **reflected, in-range** `(col, row)` so it needs
/// no per-read division. `emit(pixel, r, g, b)` receives the camera-RGB triple
/// for pixel index `pixel` (`r` = red, `g` = finalised green, `b` = blue).
///
/// # Errors
///
/// - [`Error::NotAColorFilterArray`] if `sensor.color_filter` is `None`.
/// - [`Error::UnsupportedCfaPattern`] if the CFA is not a 2×2 RGGB-family Bayer.
/// - whatever `setup` returns ([`crate::color`] builds the colour matrix there,
///   so [`Error::MissingColorCalibration`] surfaces **after** the CFA check,
///   matching the pre-fuse error order).
///
/// Never panics: every read goes through `.get` (cached interior) or reflects
/// into range (border), green reads and writes are `.get`/`.get_mut`, and all
/// arithmetic is checked/saturating/wrapping. `green` and the `emit` destination
/// are caller-owned and sized by the caller (`DEC-016`); a short `green` plane
/// would read/write short rather than panic.
pub(crate) fn develop_color_fused<N, Setup, Sink>(
    sensor: &Sensor,
    green: &mut [u16],
    normalize: &N,
    setup: Setup,
) -> Result<(), Error>
where
    N: Fn(u32, u32) -> u16,
    Setup: FnOnce() -> Result<Sink, Error>,
    Sink: FnMut(usize, u16, u16, u16),
{
    let cfa = sensor
        .color_filter
        .as_ref()
        .ok_or(Error::NotAColorFilterArray)?;
    let phase = BayerPhase::from_cfa(cfa).ok_or_else(|| Error::UnsupportedCfaPattern {
        repeat_dim: cfa.repeat_dim,
        pattern: cfa.pattern.clone(),
    })?;

    // Build the per-pixel sink (and, inside it, the colour matrix) only after
    // the CFA is validated, so a missing-calibration error surfaces after an
    // unsupported-CFA error exactly as the pre-fuse path did.
    let mut emit = setup()?;

    let width = sensor.width;
    let height = sensor.height;
    let w = usize::try_from(width).unwrap_or(0);
    let h = usize::try_from(height).unwrap_or(0);

    // The CFA phase and the level slot both start at the `ActiveArea` origin,
    // not the plane's top-left (`DEC-040`); `normalize` captures the same origin
    // for the level slot, and the colour is indexed relative to it here.
    let (active_top, active_left) = active_area_origin(sensor);

    // A zero-area plane has nothing to interpolate (and guards the loops below).
    if w == 0 || h == 0 {
        return Ok(());
    }

    // The rolling ring of normalised rows. `RING` is safely larger than the
    // five-row span (y−1 … y+3) any single output row reads, so a row's residue
    // slot never aliases a still-live row. Each photosite is normalised once,
    // when `fill_norm_row` first writes its row into the ring.
    const RING: usize = 8;
    let mut cache: Vec<u16> = match RING.checked_mul(w) {
        Some(n) => vec![0u16; n],
        // `w` is astronomically large; the caller's length checks precede this,
        // so this is unreachable, but handled rather than risking an overflow.
        None => return Ok(()),
    };
    let mut norm_filled: usize = 0; // count of absolute rows normalised into the ring
    let mut green_done: usize = 0; // count of green rows finalised in the plane

    // The interior (≥ 2 from every edge) reads the cache by direct index; the
    // two-pixel border reflects. Empty when an axis is shorter than five.
    let col_interior_end = w.saturating_sub(2);
    let row_interior_end = h.saturating_sub(2);

    for y in 0..h {
        // Normalise forward so rows up to `y + 3` (capped at the last row) are in
        // the ring — enough for green row `y + 1`'s ±2 reads and this row's reads.
        let need_norm = core::cmp::min(y.saturating_add(4), h);
        while norm_filled < need_norm {
            fill_norm_row(&mut cache, RING, w, normalize, norm_filled);
            norm_filled = norm_filled.saturating_add(1);
        }

        // Advance the green frontier so rows up to `y + 1` are finalised in the
        // plane before red/blue reads them.
        let need_green = core::cmp::min(y.saturating_add(2), h);
        while green_done < need_green {
            let g = green_done;
            let gu = u32_from(g);
            let g_interior = g >= 2 && g < row_interior_end;
            let g_base = g.checked_mul(w);
            for x in 0..w {
                let xu = u32_from(x);
                let parity = (
                    (gu.wrapping_sub(active_top) & 1) == 1,
                    (xu.wrapping_sub(active_left) & 1) == 1,
                );
                let value = if g_interior && x >= 2 && x < col_interior_end {
                    cache_green_interior(&phase, &cache, RING, w, x, g, parity).unwrap_or_else(
                        || {
                            fused_green_at_site(
                                &phase,
                                normalize,
                                width,
                                height,
                                (xu, gu),
                                (active_top, active_left),
                            )
                        },
                    )
                } else {
                    fused_green_at_site(
                        &phase,
                        normalize,
                        width,
                        height,
                        (xu, gu),
                        (active_top, active_left),
                    )
                };
                if let Some(slot) = g_base
                    .and_then(|b| b.checked_add(x))
                    .and_then(|i| green.get_mut(i))
                {
                    *slot = value;
                }
            }
            green_done = green_done.saturating_add(1);
        }

        // Red/blue + colour matrix for row `y`, straight into `dst` via `emit`.
        let yu = u32_from(y);
        let y_interior = y >= 2 && y < row_interior_end;
        let y_base = y.checked_mul(w);
        for x in 0..w {
            let xu = u32_from(x);
            let parity = (
                (yu.wrapping_sub(active_top) & 1) == 1,
                (xu.wrapping_sub(active_left) & 1) == 1,
            );
            let (red, blue) = if y_interior && x >= 2 && x < col_interior_end {
                cache_rgb_interior(&phase, &cache, RING, w, green, (x, y), parity).unwrap_or_else(
                    || {
                        fused_interpolate_red_blue(
                            &phase,
                            normalize,
                            green,
                            width,
                            height,
                            (xu, yu),
                            (active_top, active_left),
                        )
                    },
                )
            } else {
                fused_interpolate_red_blue(
                    &phase,
                    normalize,
                    green,
                    width,
                    height,
                    (xu, yu),
                    (active_top, active_left),
                )
            };
            let pixel = y_base.and_then(|b| b.checked_add(x));
            let gc = pixel.and_then(|p| green.get(p)).copied().unwrap_or(0);
            if let Some(p) = pixel {
                emit(p, red, gc, blue);
            }
        }
    }

    Ok(())
}

/// Normalise mosaic row `r` into the ring's `r % ring` slot, once — the
/// memoisation that replaces per-read recomputation (`DEC-042`, revised). Every
/// column is filled, so any in-range column read later resolves.
fn fill_norm_row<N: Fn(u32, u32) -> u16>(
    cache: &mut [u16],
    ring: usize,
    w: usize,
    normalize: &N,
    r: usize,
) {
    let base = r.checked_rem(ring).unwrap_or(0).saturating_mul(w);
    for x in 0..w {
        let value = normalize(u32_from(x), u32_from(r));
        if let Some(slot) = base.checked_add(x).and_then(|i| cache.get_mut(i)) {
            *slot = value;
        }
    }
}

/// The normalised sample at absolute `(row, col)` from the ring, or `None` if the
/// index does not resolve (unreachable for an in-range cell). The ring slot is
/// `row % ring`; the caller guarantees that slot currently holds `row`.
fn cache_cell(cache: &[u16], ring: usize, w: usize, row: usize, col: usize) -> Option<u16> {
    row.checked_rem(ring)
        .unwrap_or(0)
        .checked_mul(w)?
        .checked_add(col)
        .and_then(|i| cache.get(i))
        .copied()
}

/// Pass-1 green at an **interior** pixel, reading the normalised ring by direct
/// index — the `DEC-041` fast path over normalised rows, the memoised twin of
/// [`fused_green_at_site`]. Same nine plus-pattern neighbours in the same order
/// through the shared [`green_from_neighbours`], so the result is bit-identical.
/// `None` only if an index escapes the ring (impossible for an interior pixel),
/// so the caller can fall back to the reflecting path; never panics.
fn cache_green_interior(
    phase: &BayerPhase,
    cache: &[u16],
    ring: usize,
    w: usize,
    x: usize,
    g: usize,
    parity: (bool, bool),
) -> Option<u16> {
    let (row_odd, col_odd) = parity;
    if phase.colour(row_odd, col_odd) == Colour::Green {
        // A green site keeps its own normalised sample.
        return cache_cell(cache, ring, w, g, x);
    }
    let at = |row: usize, col: usize| -> Option<i32> {
        cache_cell(cache, ring, w, row, col).map(i32::from)
    };
    let xl1 = x.checked_sub(1)?;
    let xl2 = x.checked_sub(2)?;
    let xr1 = x.checked_add(1)?;
    let xr2 = x.checked_add(2)?;
    let g_up1 = g.checked_sub(1)?;
    let g_up2 = g.checked_sub(2)?;
    let g_dn1 = g.checked_add(1)?;
    let g_dn2 = g.checked_add(2)?;

    let c = at(g, x)?;
    let w_neighbour = at(g, xl1)?;
    let e = at(g, xr1)?;
    let n = at(g_up1, x)?;
    let s = at(g_dn1, x)?;
    let ww = at(g, xl2)?;
    let ee = at(g, xr2)?;
    let nn = at(g_up2, x)?;
    let ss = at(g_dn2, x)?;

    Some(green_from_neighbours([
        c,
        w_neighbour,
        e,
        n,
        s,
        ww,
        ee,
        nn,
        ss,
    ]))
}

/// Pass-2 red and blue at an **interior** pixel, reading the normalised ring
/// (`src`) by direct index and the finalised greens from the `w·h` green plane —
/// the memoised twin of [`fused_interpolate_red_blue`], feeding the shared
/// [`colour_difference`] / [`diagonal_from_neighbours`] kernels on the same
/// neighbours in the same order, so the result is bit-identical. The site's own
/// primary at a red/blue site is its normalised sample. `None` only if an index
/// escapes a buffer (impossible for an interior pixel); never panics.
fn cache_rgb_interior(
    phase: &BayerPhase,
    cache: &[u16],
    ring: usize,
    w: usize,
    green: &[u16],
    site: (usize, usize),
    parity: (bool, bool),
) -> Option<(u16, u16)> {
    let (x, y) = site;
    let (row_odd, col_odd) = parity;
    let s_at = |row: usize, col: usize| -> Option<i32> {
        cache_cell(cache, ring, w, row, col).map(i32::from)
    };
    let g_at = |row: usize, col: usize| -> Option<i32> {
        row.checked_mul(w)?
            .checked_add(col)
            .and_then(|i| green.get(i))
            .copied()
            .map(i32::from)
    };
    let up = y.checked_sub(1)?;
    let down = y.checked_add(1)?;
    let xl = x.checked_sub(1)?;
    let xr = x.checked_add(1)?;
    let gc = g_at(y, x)?;

    match phase.colour(row_odd, col_odd) {
        Colour::Green => {
            // Horizontal neighbours carry one primary, vertical the other.
            let horizontal = phase.colour(row_odd, !col_odd);
            let val_h =
                colour_difference(s_at(y, xl)?, s_at(y, xr)?, g_at(y, xl)?, gc, g_at(y, xr)?);
            let val_v = colour_difference(
                s_at(up, x)?,
                s_at(down, x)?,
                g_at(up, x)?,
                gc,
                g_at(down, x)?,
            );
            if horizontal == Colour::Red {
                Some((val_h, val_v))
            } else {
                Some((val_v, val_h))
            }
        }
        Colour::Red => {
            let red = cache_cell(cache, ring, w, y, x)?;
            let blue = diagonal_from_neighbours([
                s_at(up, xl)?,
                s_at(up, xr)?,
                s_at(down, xl)?,
                s_at(down, xr)?,
                g_at(up, xl)?,
                g_at(up, xr)?,
                g_at(down, xl)?,
                g_at(down, xr)?,
                gc,
            ]);
            Some((red, blue))
        }
        Colour::Blue => {
            let blue = cache_cell(cache, ring, w, y, x)?;
            let red = diagonal_from_neighbours([
                s_at(up, xl)?,
                s_at(up, xr)?,
                s_at(down, xl)?,
                s_at(down, xr)?,
                g_at(up, xl)?,
                g_at(up, xr)?,
                g_at(down, xl)?,
                g_at(down, xr)?,
                gc,
            ]);
            Some((red, blue))
        }
    }
}

/// The normalised mosaic sample at the **reflected, in-range** coordinate
/// `(x, y)` as `i32`, the fused counterpart of [`sample_i32`] (which reads a
/// pre-normalised plane). Reflect, then ask `normalize` for the per-slot
/// level-normalised value at that cell.
fn norm_sample_i32<N: Fn(u32, u32) -> u16>(
    normalize: &N,
    width: u32,
    height: u32,
    x: i64,
    y: i64,
) -> i32 {
    let rx = reflect(x, width);
    let ry = reflect(y, height);
    i32::from(normalize(rx, ry))
}

/// The finalised green at `(x, y)` (reflect border) read from the fused `w·h`
/// green plane, the counterpart of [`green_sample`] (which reads green from the
/// interleaved `dst`). `0` if the index somehow does not resolve — unreachable
/// after a reflect into range, but never a panic.
fn green_plane_sample(green: &[u16], width: u32, height: u32, x: i64, y: i64) -> i32 {
    let rx = reflect(x, width);
    let ry = reflect(y, height);
    let g = plane_index(width, rx, ry)
        .and_then(|i| green.get(i))
        .copied()
        .unwrap_or(0);
    i32::from(g)
}

/// Pass-1 green at one pixel for the fused path — the normalise-on-read twin of
/// [`green_at_site`], calling the shared [`green_from_neighbours`] kernel on the
/// same nine plus-pattern neighbours in the same order, so the result is
/// bit-identical. A green site keeps its own normalised sample.
fn fused_green_at_site<N: Fn(u32, u32) -> u16>(
    phase: &BayerPhase,
    normalize: &N,
    width: u32,
    height: u32,
    site: (u32, u32),
    origin: (u32, u32),
) -> u16 {
    let (x, y) = site;
    let (active_top, active_left) = origin;
    let row_odd = (y.wrapping_sub(active_top) & 1) == 1;
    let col_odd = (x.wrapping_sub(active_left) & 1) == 1;

    if phase.colour(row_odd, col_odd) == Colour::Green {
        return normalize(x, y);
    }

    let xi = i64::from(x);
    let yi = i64::from(y);
    let at_h = |dx: i64| norm_sample_i32(normalize, width, height, xi.saturating_add(dx), yi);
    let at_v = |dy: i64| norm_sample_i32(normalize, width, height, xi, yi.saturating_add(dy));

    let c = norm_sample_i32(normalize, width, height, xi, yi);
    let w = at_h(-1);
    let e = at_h(1);
    let n = at_v(-1);
    let s = at_v(1);
    let ww = at_h(-2);
    let ee = at_h(2);
    let nn = at_v(-2);
    let ss = at_v(2);

    green_from_neighbours([c, w, e, n, s, ww, ee, nn, ss])
}

/// Pass-2 `(red, blue)` at one pixel for the fused path — the normalise-on-read
/// twin of [`interpolate_red_blue`], reading the green plane and calling the
/// shared [`colour_difference`] / [`diagonal_from_neighbours`] kernels on the
/// same neighbours in the same order, so the result is bit-identical. The site's
/// own primary at a red/blue site is its normalised sample.
fn fused_interpolate_red_blue<N: Fn(u32, u32) -> u16>(
    phase: &BayerPhase,
    normalize: &N,
    green: &[u16],
    width: u32,
    height: u32,
    site: (u32, u32),
    origin: (u32, u32),
) -> (u16, u16) {
    let (x, y) = site;
    let (active_top, active_left) = origin;
    let row_odd = (y.wrapping_sub(active_top) & 1) == 1;
    let col_odd = (x.wrapping_sub(active_left) & 1) == 1;
    let here = phase.colour(row_odd, col_odd);
    let xi = i64::from(x);
    let yi = i64::from(y);
    let gc = green_plane_sample(green, width, height, xi, yi);

    match here {
        Colour::Green => {
            let horizontal = phase.colour(row_odd, !col_odd);
            let val_h = colour_difference(
                norm_sample_i32(normalize, width, height, xi.saturating_sub(1), yi),
                norm_sample_i32(normalize, width, height, xi.saturating_add(1), yi),
                green_plane_sample(green, width, height, xi.saturating_sub(1), yi),
                gc,
                green_plane_sample(green, width, height, xi.saturating_add(1), yi),
            );
            let val_v = colour_difference(
                norm_sample_i32(normalize, width, height, xi, yi.saturating_sub(1)),
                norm_sample_i32(normalize, width, height, xi, yi.saturating_add(1)),
                green_plane_sample(green, width, height, xi, yi.saturating_sub(1)),
                gc,
                green_plane_sample(green, width, height, xi, yi.saturating_add(1)),
            );
            if horizontal == Colour::Red {
                (val_h, val_v)
            } else {
                (val_v, val_h)
            }
        }
        Colour::Red => {
            let red = normalize(x, y);
            let blue = fused_diagonal_directed(normalize, green, width, height, xi, yi, gc);
            (red, blue)
        }
        Colour::Blue => {
            let blue = normalize(x, y);
            let red = fused_diagonal_directed(normalize, green, width, height, xi, yi, gc);
            (red, blue)
        }
    }
}

/// The opposite primary at a red/blue site for the fused path — the
/// normalise-on-read twin of [`diagonal_directed`], reading the four diagonal
/// normalised samples and their greens from the green plane and calling the
/// shared [`diagonal_from_neighbours`] kernel, so the result is bit-identical.
fn fused_diagonal_directed<N: Fn(u32, u32) -> u16>(
    normalize: &N,
    green: &[u16],
    width: u32,
    height: u32,
    xi: i64,
    yi: i64,
    gc: i32,
) -> u16 {
    let nw = norm_sample_i32(
        normalize,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_sub(1),
    );
    let ne = norm_sample_i32(
        normalize,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_sub(1),
    );
    let sw = norm_sample_i32(
        normalize,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_add(1),
    );
    let se = norm_sample_i32(
        normalize,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_add(1),
    );
    let g_nw = green_plane_sample(
        green,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_sub(1),
    );
    let g_ne = green_plane_sample(
        green,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_sub(1),
    );
    let g_sw = green_plane_sample(
        green,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_add(1),
    );
    let g_se = green_plane_sample(
        green,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_add(1),
    );

    diagonal_from_neighbours([nw, ne, sw, se, g_nw, g_ne, g_sw, g_se, gc])
}

/// Pass 1 green at an **interior** pixel (`≥ 2` from every edge), read directly
/// from per-row slices of `src` — no `reflect()`, no `plane_index()`, no
/// per-pixel `(x, y)` recovery. The neighbour values equal those the reflecting
/// `green_at_site` would read (reflection is the identity in the interior) and
/// the kernel arithmetic is the shared [`green_from_neighbours`], so the result
/// is bit-identical. Each neighbour is read from a fixed-size window, so there
/// is no per-element bounds check. Returns `None` only if an index escapes
/// `src` — impossible for an interior pixel, but handled so the caller can fall
/// back to the reflecting path and the function never panics (`DEC-041`).
fn pass1_green_interior(
    phase: &BayerPhase,
    src: &[u16],
    w: usize,
    x: usize,
    y: usize,
    parity: (bool, bool),
) -> Option<u16> {
    let (row_odd, col_odd) = parity;
    let centre = y.checked_mul(w)?.checked_add(x)?;

    if phase.colour(row_odd, col_odd) == Colour::Green {
        // A green site keeps its own sample — what `green_at_site` returns via
        // `u32_to_u16(sample(..))`, the identity for an in-range value.
        return src.get(centre).copied();
    }

    // Read a horizontal five-window `[a, b, c, d, e]` centred on column index
    // `ci`, as a fixed-size array destructured by value (constant index, no
    // per-element check). The window makes the five reads a single bounds check.
    let window = |ci: usize| -> Option<[u16; 5]> {
        let lo = ci.checked_sub(2)?;
        let hi = ci.checked_add(3)?;
        let cells: &[u16; 5] = src.get(lo..hi)?.try_into().ok()?;
        Some(*cells)
    };

    let up1 = centre.checked_sub(w)?;
    let up2 = up1.checked_sub(w)?;
    let down1 = centre.checked_add(w)?;
    let down2 = down1.checked_add(w)?;

    // Centre row: WW, W, C, E, EE. Vertical rows: only column `x` (index 2).
    let [ww, w_neighbour, c, e, ee] = window(centre)?;
    let n = window(up1)?[2];
    let s = window(down1)?[2];
    let nn = window(up2)?[2];
    let ss = window(down2)?[2];

    Some(green_from_neighbours([
        i32::from(c),
        i32::from(w_neighbour),
        i32::from(e),
        i32::from(n),
        i32::from(s),
        i32::from(ww),
        i32::from(ee),
        i32::from(nn),
        i32::from(ss),
    ]))
}

/// Pass 2 red and blue at an **interior** pixel, reading `src` and the finalized
/// greens in `dst` directly from per-row slices — no `reflect()`/`plane_index()`
/// and no per-element bounds check (each neighbour comes from a fixed-size
/// window). The values equal those `interpolate_red_blue` reads in the interior
/// and the arithmetic is the shared `colour_difference` / `diagonal_from_neighbours`,
/// so the output is bit-identical. `None` only if an index escapes a buffer
/// (impossible for an interior pixel) so the caller can fall back; never panics.
fn pass2_red_blue_interior(
    phase: &BayerPhase,
    src: &[u16],
    dst: &[u16],
    w: usize,
    x: usize,
    y: usize,
    parity: (bool, bool),
) -> Option<(u16, u16)> {
    let (row_odd, col_odd) = parity;
    let centre = y.checked_mul(w)?.checked_add(x)?;
    let up = centre.checked_sub(w)?;
    let down = centre.checked_add(w)?;

    // Three src samples `[x-1, x, x+1]` of the row whose centre pixel is `ci`.
    let src3 = |ci: usize| -> Option<[u16; 3]> {
        let lo = ci.checked_sub(1)?;
        let hi = ci.checked_add(2)?;
        let cells: &[u16; 3] = src.get(lo..hi)?.try_into().ok()?;
        Some(*cells)
    };
    // The three greens `[g(x-1), g(x), g(x+1)]` of that row from the interleaved
    // `dst`: one nine-element window, greens at constant indices 1, 4, 7.
    let green3 = |ci: usize| -> Option<[i32; 3]> {
        let first_pixel = ci.checked_sub(1)?;
        let base = first_pixel.checked_mul(3)?;
        let end = base.checked_add(9)?;
        let cells: &[u16; 9] = dst.get(base..end)?.try_into().ok()?;
        let &[_, g0, _, _, g1, _, _, g2, _] = cells;
        Some([i32::from(g0), i32::from(g1), i32::from(g2)])
    };

    let s_top = src3(up)?;
    let s_mid = src3(centre)?;
    let s_bot = src3(down)?;
    let g_top = green3(up)?;
    let g_mid = green3(centre)?;
    let g_bot = green3(down)?;
    let gc = g_mid[1];

    match phase.colour(row_odd, col_odd) {
        Colour::Green => {
            // Horizontal neighbours carry one primary, vertical the other
            // (greens are diagonal); read which from the pattern as the slow
            // path does, so RGGB and BGGR take the same branch.
            let horizontal = phase.colour(row_odd, !col_odd);
            let val_h = colour_difference(
                i32::from(s_mid[0]),
                i32::from(s_mid[2]),
                g_mid[0],
                gc,
                g_mid[2],
            );
            let val_v = colour_difference(
                i32::from(s_top[1]),
                i32::from(s_bot[1]),
                g_top[1],
                gc,
                g_bot[1],
            );
            if horizontal == Colour::Red {
                Some((val_h, val_v))
            } else {
                Some((val_v, val_h))
            }
        }
        Colour::Red => {
            let red = s_mid[1];
            let blue = diagonal_from_neighbours([
                i32::from(s_top[0]),
                i32::from(s_top[2]),
                i32::from(s_bot[0]),
                i32::from(s_bot[2]),
                g_top[0],
                g_top[2],
                g_bot[0],
                g_bot[2],
                gc,
            ]);
            Some((red, blue))
        }
        Colour::Blue => {
            let blue = s_mid[1];
            let red = diagonal_from_neighbours([
                i32::from(s_top[0]),
                i32::from(s_top[2]),
                i32::from(s_bot[0]),
                i32::from(s_bot[2]),
                g_top[0],
                g_top[2],
                g_bot[0],
                g_bot[2],
                gc,
            ]);
            Some((red, blue))
        }
    }
}

/// Pass 1: the finalized green at one pixel. A green site keeps its own sample;
/// a red/blue site interpolates along the lower-gradient axis with a same-colour
/// second-difference correction (`SPEC-031` Design, pass 1):
///
/// - `CLh = |W−E| + |WW−2C+EE|`, `CLv = |N−S| + |NN−2C+SS|` — axis classifiers.
/// - `gh = 2(W+E) − (WW−2C+EE)`, `gv = 2(N+S) − (NN−2C+SS)` — predictors /4.
/// - lower gradient wins (`CLh<CLv → rnd4(gh)`, `CLv<CLh → rnd4(gv)`,
///   tie → `rnd8(gh+gv)`), then clamp.
fn green_at_site(
    phase: &BayerPhase,
    src: &[u16],
    width: u32,
    height: u32,
    site: (u32, u32),
    origin: (u32, u32),
) -> u16 {
    let (x, y) = site;
    let (active_top, active_left) = origin;
    // Phase relative to the `ActiveArea` origin (`DEC-040`). `wrapping_sub` is
    // panic-free and parity-correct even for `y < active_top` (the low bit of a
    // wrapping subtraction is still the true parity); an even origin reduces to
    // `y & 1`. Neighbour sampling below stays plane-relative — only the colour
    // label each photosite is assigned shifts.
    let row_odd = (y.wrapping_sub(active_top) & 1) == 1;
    let col_odd = (x.wrapping_sub(active_left) & 1) == 1;
    let xi = i64::from(x);
    let yi = i64::from(y);

    if phase.colour(row_odd, col_odd) == Colour::Green {
        return u32_to_u16(sample(src, width, height, xi, yi));
    }

    let at_h = |dx: i64| sample_i32(src, width, height, xi.saturating_add(dx), yi);
    let at_v = |dy: i64| sample_i32(src, width, height, xi, yi.saturating_add(dy));

    let c = sample_i32(src, width, height, xi, yi);
    let w = at_h(-1);
    let e = at_h(1);
    let n = at_v(-1);
    let s = at_v(1);
    let ww = at_h(-2);
    let ee = at_h(2);
    let nn = at_v(-2);
    let ss = at_v(2);

    green_from_neighbours([c, w, e, n, s, ww, ee, nn, ss])
}

/// The pass-1 green at a red/blue site, from its nine plus-pattern neighbours
/// `[C, W, E, N, S, WW, EE, NN, SS]` (`SPEC-031` Design, pass 1):
///
/// - `CLh = |W−E| + |WW−2C+EE|`, `CLv = |N−S| + |NN−2C+SS|` — axis classifiers.
/// - `gh = 2(W+E) − (WW−2C+EE)`, `gv = 2(N+S) − (NN−2C+SS)` — predictors /4.
/// - lower gradient wins (`CLh<CLv → rnd4(gh)`, `CLv<CLh → rnd4(gv)`,
///   tie → `rnd8(gh+gv)`), then clamp.
///
/// This is the one copy of the kernel arithmetic that the reflecting border
/// path ([`green_at_site`]) and the interior fast path ([`pass1_green_interior`])
/// both call, so their outputs are identical by construction (`DEC-041`). The
/// neighbours arrive as an array (not nine arguments) to stay under the
/// argument-count lint.
fn green_from_neighbours(plus: [i32; 9]) -> u16 {
    let [c, w, e, n, s, ww, ee, nn, ss] = plus;
    let lap_h = second_difference(ww, c, ee);
    let lap_v = second_difference(nn, c, ss);
    let cl_h = abs_i32(w.saturating_sub(e)).saturating_add(abs_i32(lap_h));
    let cl_v = abs_i32(n.saturating_sub(s)).saturating_add(abs_i32(lap_v));
    let gh = twice(w.saturating_add(e)).saturating_sub(lap_h);
    let gv = twice(n.saturating_add(s)).saturating_sub(lap_v);

    let green = if cl_h < cl_v {
        rnd4(gh)
    } else if cl_v < cl_h {
        rnd4(gv)
    } else {
        rnd8(gh.saturating_add(gv))
    };
    clamp_i32_u16(green)
}

/// Pass 2: the `(red, blue)` at one pixel, reading pass 1's finalized green from
/// `dst` (`SPEC-031` Design, pass 2).
///
/// - **Green site:** each primary comes from its two same-colour neighbours by a
///   colour-difference interpolation; which primary lies horizontally vs
///   vertically is read from the pattern.
/// - **Red/blue site:** the site's own primary is its sample; the opposite
///   primary comes from the four diagonal neighbours, gradient-directed.
fn interpolate_red_blue(
    phase: &BayerPhase,
    src: &[u16],
    dst: &[u16],
    width: u32,
    height: u32,
    site: (u32, u32),
    origin: (u32, u32),
) -> (u16, u16) {
    let (x, y) = site;
    let (active_top, active_left) = origin;
    // Phase relative to the `ActiveArea` origin (`DEC-040`), as in
    // [`green_at_site`]; `wrapping_sub` is panic-free and parity-correct, and an
    // even origin reduces to `y & 1`.
    let row_odd = (y.wrapping_sub(active_top) & 1) == 1;
    let col_odd = (x.wrapping_sub(active_left) & 1) == 1;
    let here = phase.colour(row_odd, col_odd);
    let xi = i64::from(x);
    let yi = i64::from(y);
    let gc = green_sample(dst, width, height, xi, yi);

    match here {
        Colour::Green => {
            // The horizontal neighbours carry one primary, the vertical the
            // other (greens are diagonal). Read which from the pattern so RGGB
            // and BGGR take the same path.
            let horizontal = phase.colour(row_odd, !col_odd);
            let val_h = colour_difference(
                sample_i32(src, width, height, xi.saturating_sub(1), yi),
                sample_i32(src, width, height, xi.saturating_add(1), yi),
                green_sample(dst, width, height, xi.saturating_sub(1), yi),
                gc,
                green_sample(dst, width, height, xi.saturating_add(1), yi),
            );
            let val_v = colour_difference(
                sample_i32(src, width, height, xi, yi.saturating_sub(1)),
                sample_i32(src, width, height, xi, yi.saturating_add(1)),
                green_sample(dst, width, height, xi, yi.saturating_sub(1)),
                gc,
                green_sample(dst, width, height, xi, yi.saturating_add(1)),
            );
            if horizontal == Colour::Red {
                (val_h, val_v)
            } else {
                (val_v, val_h)
            }
        }
        Colour::Red => {
            let red = u32_to_u16(sample(src, width, height, xi, yi));
            let blue = diagonal_directed(src, dst, width, height, xi, yi, gc);
            (red, blue)
        }
        Colour::Blue => {
            let blue = u32_to_u16(sample(src, width, height, xi, yi));
            let red = diagonal_directed(src, dst, width, height, xi, yi, gc);
            (red, blue)
        }
    }
}

/// The opposite primary at a red/blue site: the four diagonal neighbours
/// (all the opposite colour), gradient-directed to the lower-gradient diagonal,
/// with a green second-difference correction (`SPEC-031` Design, pass 2):
///
/// - `dp = 2(NW+SE) − (gNW−2gC+gSE)`, `dn = 2(NE+SW) − (gNE−2gC+gSW)`.
/// - `CLp = |SE−NW| + |gNW−2gC+gSE|`, `CLn = |SW−NE| + |gNE−2gC+gSW|`.
/// - lower gradient wins (`CLp<CLn → rnd4(dp)`, `CLn<CLp → rnd4(dn)`,
///   tie → `rnd8(dp+dn)`), then clamp.
fn diagonal_directed(
    src: &[u16],
    dst: &[u16],
    width: u32,
    height: u32,
    xi: i64,
    yi: i64,
    gc: i32,
) -> u16 {
    let nw = sample_i32(
        src,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_sub(1),
    );
    let ne = sample_i32(
        src,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_sub(1),
    );
    let sw = sample_i32(
        src,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_add(1),
    );
    let se = sample_i32(
        src,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_add(1),
    );
    let g_nw = green_sample(
        dst,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_sub(1),
    );
    let g_ne = green_sample(
        dst,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_sub(1),
    );
    let g_sw = green_sample(
        dst,
        width,
        height,
        xi.saturating_sub(1),
        yi.saturating_add(1),
    );
    let g_se = green_sample(
        dst,
        width,
        height,
        xi.saturating_add(1),
        yi.saturating_add(1),
    );

    diagonal_from_neighbours([nw, ne, sw, se, g_nw, g_ne, g_sw, g_se, gc])
}

/// The opposite primary at a red/blue site, from the four diagonal source
/// neighbours, the four diagonal greens, and the centre green:
/// `[NW, NE, SW, SE, gNW, gNE, gSW, gSE, gC]` (`SPEC-031` Design, pass 2):
///
/// - `dp = 2(NW+SE) − (gNW−2gC+gSE)`, `dn = 2(NE+SW) − (gNE−2gC+gSW)`.
/// - `CLp = |SE−NW| + |gNW−2gC+gSE|`, `CLn = |SW−NE| + |gNE−2gC+gSW|`.
/// - lower gradient wins (`CLp<CLn → rnd4(dp)`, `CLn<CLp → rnd4(dn)`,
///   tie → `rnd8(dp+dn)`), then clamp.
///
/// The one copy of this kernel, called by both the reflecting
/// [`diagonal_directed`] and the interior [`pass2_red_blue_interior`], so their
/// outputs match by construction (`DEC-041`). The neighbours arrive as an array
/// to stay under the argument-count lint.
fn diagonal_from_neighbours(neighbours: [i32; 9]) -> u16 {
    let [nw, ne, sw, se, g_nw, g_ne, g_sw, g_se, gc] = neighbours;
    let lap_p = second_difference(g_nw, gc, g_se);
    let lap_n = second_difference(g_ne, gc, g_sw);
    let dp = twice(nw.saturating_add(se)).saturating_sub(lap_p);
    let dn = twice(ne.saturating_add(sw)).saturating_sub(lap_n);
    let cl_p = abs_i32(se.saturating_sub(nw)).saturating_add(abs_i32(lap_p));
    let cl_n = abs_i32(sw.saturating_sub(ne)).saturating_add(abs_i32(lap_n));

    let value = if cl_p < cl_n {
        rnd4(dp)
    } else if cl_n < cl_p {
        rnd4(dn)
    } else {
        rnd8(dp.saturating_add(dn))
    };
    clamp_i32_u16(value)
}

/// A primary at a green site from its two same-colour neighbours `(p1, p2)` and
/// the greens at those neighbours (`g1`, `g2`) and the centre (`gc`):
/// `rnd4( 2(p1+p2) − (g1 − 2gC + g2) )`, clamped (`SPEC-031` Design, pass 2).
fn colour_difference(p1: i32, p2: i32, g1: i32, gc: i32, g2: i32) -> u16 {
    let numerator = twice(p1.saturating_add(p2)).saturating_sub(second_difference(g1, gc, g2));
    clamp_i32_u16(rnd4(numerator))
}

/// The symmetric second difference `a − 2·mid + b` (a discrete Laplacian), in
/// `i32` with saturating arithmetic.
fn second_difference(a: i32, mid: i32, b: i32) -> i32 {
    a.saturating_sub(twice(mid)).saturating_add(b)
}

/// `2 * n`, saturating (never actually saturates at 16-bit sample depth).
fn twice(n: i32) -> i32 {
    n.saturating_mul(2)
}

/// `|n|` without the panic `i32::abs` carries on `i32::MIN` (saturating negate).
fn abs_i32(n: i32) -> i32 {
    if n < 0 {
        n.saturating_neg()
    } else {
        n
    }
}

/// Round half up, denominator 4: `⌊(n + 2) / 4⌋`. Floor division (not truncation)
/// so a negative numerator — HA corrections can be negative — rounds correctly.
fn rnd4(n: i32) -> i32 {
    floor_div(n.saturating_add(2), 4)
}

/// Round half up, denominator 8: `⌊(n + 4) / 8⌋`. The tie branch's denominator —
/// it averages both axis predictors, so `gh + gv` is divided by 8, not 4.
fn rnd8(n: i32) -> i32 {
    floor_div(n.saturating_add(4), 8)
}

/// Floor division of `n` by a positive constant `d` (`4` or `8` here), built
/// from `checked_div`/`checked_rem` so it matches `i32::div_euclid` without
/// relying on an operator the panic-free lint set forbids. For `d > 0`,
/// `div_euclid` is floor division.
fn floor_div(n: i32, d: i32) -> i32 {
    let q = n.checked_div(d).unwrap_or(0);
    let r = n.checked_rem(d).unwrap_or(0);
    if r < 0 {
        q.saturating_sub(1)
    } else {
        q
    }
}

/// Clamp an `i32` to `[0, 65535]` and narrow to `u16` (never panics: the clamped
/// value is in range, so the `try_from` always succeeds).
fn clamp_i32_u16(n: i32) -> u16 {
    u16::try_from(n.clamp(0, 65535)).unwrap_or(0)
}

/// The mosaic sample at `(x, y)` (reflect border) as `i32`. `sample` returns a
/// `u32` ≤ 65535, so the `try_from` always succeeds.
fn sample_i32(src: &[u16], width: u32, height: u32, x: i64, y: i64) -> i32 {
    i32::try_from(sample(src, width, height, x, y)).unwrap_or(0)
}

/// The green sample at `(x, y)` (reflect border) read from pass 1's output: the
/// green slot of pixel `(rx, ry)` in the interleaved `dst`, as `i32`.
fn green_sample(dst: &[u16], width: u32, height: u32, x: i64, y: i64) -> i32 {
    let rx = reflect(x, width);
    let ry = reflect(y, height);
    let green = plane_index(width, rx, ry)
        .and_then(|i| i.checked_mul(3))
        .and_then(|i| i.checked_add(1))
        .and_then(|i| dst.get(i))
        .copied()
        .unwrap_or(0);
    i32::from(green)
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
/// `[0, len)` for an offset-1 neighbour once `len ≥ 2` and for an offset-2
/// neighbour once `len ≥ 3`; on a shorter axis the final `clamp` engages, so a
/// degenerate 1- or 2-wide mosaic cannot index out of range (the AC's 1×1 case).
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
    fn reflect_handles_offset_two_in_range_once_wide_enough() {
        // len 5: -2 mirrors to 2, 6 mirrors to 2 (2*5-2-6) — HA's offset-2
        // neighbours land in range on a ≥3-wide axis.
        assert_eq!(reflect(-2, 5), 2);
        assert_eq!(reflect(6, 5), 2);
    }

    #[test]
    fn reflect_on_a_degenerate_axis_cannot_leave_range() {
        // len 1: every neighbour (offset 1 or 2) resolves to the single pixel 0.
        assert_eq!(reflect(-1, 1), 0);
        assert_eq!(reflect(2, 1), 0);
        // len 2: offset-2 would reflect to -2/+... then clamps into [0, 1].
        assert_eq!(reflect(-2, 2), 1);
        assert_eq!(reflect(3, 2), 0);
    }

    #[test]
    fn rnd4_and_rnd8_round_half_up_and_floor_negatives() {
        // Positive: half rounds up.
        assert_eq!(rnd4(2), 1); // (2+2)/4 = 1
        assert_eq!(rnd4(1), 0); // (1+2)/4 = 0 (0.75 floors)
        assert_eq!(rnd4(6), 2); // (6+2)/4 = 2
        assert_eq!(rnd8(4), 1); // (4+4)/8 = 1
        assert_eq!(rnd8(12), 2); // (12+4)/8 = 2
                                 // Negative numerators floor (div_euclid), not truncate toward zero.
        assert_eq!(rnd4(-2), 0); // (-2+2)/4 = 0
        assert_eq!(rnd4(-6), -1); // (-6+2)/4 = -4/4 = -1
        assert_eq!(rnd4(-3), -1); // (-3+2)/4 = -1/4 → floor -1 (truncation would give 0)
        assert_eq!(rnd8(-12), -1); // (-12+4)/8 = -8/8 = -1
    }

    #[test]
    fn floor_div_matches_div_euclid_for_positive_divisors() {
        for n in -40i32..=40 {
            assert_eq!(floor_div(n, 4), n.div_euclid(4));
            assert_eq!(floor_div(n, 8), n.div_euclid(8));
        }
    }

    #[test]
    fn clamp_saturates_to_the_u16_window() {
        assert_eq!(clamp_i32_u16(-1), 0);
        assert_eq!(clamp_i32_u16(0), 0);
        assert_eq!(clamp_i32_u16(65535), 65535);
        assert_eq!(clamp_i32_u16(70000), 65535);
    }

    #[test]
    fn abs_i32_never_panics_at_the_edge() {
        assert_eq!(abs_i32(-5), 5);
        assert_eq!(abs_i32(5), 5);
        assert_eq!(abs_i32(i32::MIN), i32::MAX); // saturating, not a panic
    }
}
