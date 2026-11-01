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
/// Never panics: every index is bounds-checked (`.get`/`.get_mut`) and every
/// intermediate is signed `i32` via saturating/checked operations whose inputs
/// are provably far from `i32`'s range at 16-bit sample depth.
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

    // ── Pass 1 — green at every pixel, into each pixel's own green slot. ──────
    // `as_chunks_mut::<3>` yields exactly `pixels` fixed-size `[u16; 3]` chunks
    // (dst.len() == pixels * 3, checked above) and an empty remainder, so the
    // pixel index is the chunk index. Pass 1 reads only `src`, so the mutable
    // chunk borrow does not conflict with anything.
    let (triples, _remainder) = dst.as_chunks_mut::<3>();
    for (pixel, [_red, green, _blue]) in triples.iter_mut().enumerate() {
        let y = u32::try_from(pixel.checked_div(w).unwrap_or(0)).unwrap_or(0);
        let x = u32::try_from(pixel.checked_rem(w).unwrap_or(0)).unwrap_or(0);
        *green = green_at_site(&phase, src, width, height, x, y);
    }

    // ── Pass 2 — red and blue, reading pass 1's finalized green from `dst`. ──
    // Indexed (not `as_chunks_mut`) because this pass reads the green slots of
    // *other* pixels while writing this pixel's red/blue slots; the green reads
    // (immutable, via `.get`) complete before the red/blue writes (mutable, via
    // `.get_mut`), and the two slot sets are disjoint, so there is no aliasing.
    for pixel in 0..expected_src {
        let y = u32::try_from(pixel.checked_div(w).unwrap_or(0)).unwrap_or(0);
        let x = u32::try_from(pixel.checked_rem(w).unwrap_or(0)).unwrap_or(0);
        let (red, blue) = interpolate_red_blue(&phase, src, dst, width, height, x, y);
        let base = pixel.checked_mul(3).unwrap_or(0);
        if let Some(slot) = dst.get_mut(base) {
            *slot = red;
        }
        if let Some(slot) = base.checked_add(2).and_then(|i| dst.get_mut(i)) {
            *slot = blue;
        }
    }

    Ok(())
}

/// Pass 1: the finalized green at one pixel. A green site keeps its own sample;
/// a red/blue site interpolates along the lower-gradient axis with a same-colour
/// second-difference correction (`SPEC-031` Design, pass 1):
///
/// - `CLh = |W−E| + |WW−2C+EE|`, `CLv = |N−S| + |NN−2C+SS|` — axis classifiers.
/// - `gh = 2(W+E) − (WW−2C+EE)`, `gv = 2(N+S) − (NN−2C+SS)` — predictors /4.
/// - lower gradient wins (`CLh<CLv → rnd4(gh)`, `CLv<CLh → rnd4(gv)`,
///   tie → `rnd8(gh+gv)`), then clamp.
fn green_at_site(phase: &BayerPhase, src: &[u16], width: u32, height: u32, x: u32, y: u32) -> u16 {
    let row_odd = (y & 1) == 1;
    let col_odd = (x & 1) == 1;
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
    x: u32,
    y: u32,
) -> (u16, u16) {
    let row_odd = (y & 1) == 1;
    let col_odd = (x & 1) == 1;
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
