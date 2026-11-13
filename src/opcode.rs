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

use crate::ifd::{Sensor, TAG_OPCODE_LIST_1, TAG_OPCODE_LIST_2, TAG_OPCODE_LIST_3};
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
    /// genuinely unknown. Only ever produced when `flags` bit 0 (optional)
    /// is set; an unset bit on an unrecognized ID is
    /// [`Error::UnsupportedMandatoryOpcode`] instead of a value.
    Unknown {
        /// The opcode ID.
        id: u32,
        /// The raw `Flags` word.
        flags: u32,
        /// The raw parameter bytes, unparsed.
        params: Vec<u8>,
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
/// (the first 48 bytes after `N`) is extracted, per the module docs.
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

/// One opcode's framing, its parameter block sliced but not interpreted.
struct RawOpcode<'a> {
    id: u32,
    flags: u32,
    params: &'a [u8],
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
    let _version = read_u32_be(bytes, at)?;
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
    Ok(RawOpcode { id, flags, params })
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
pub fn parse_opcode_list(bytes: &[u8]) -> Result<Vec<Opcode>, Error> {
    let count = read_u32_be(bytes, 0)?;
    let mut pos: usize = 4;

    let mut opcodes = Vec::new();
    for _ in 0..count {
        let RawOpcode { id, flags, params } = read_raw_opcode(bytes, &mut pos)?;

        match id {
            OPCODE_ID_WARP_RECTILINEAR => {
                let warp = parse_warp_rectilinear_params(params, id)?;
                opcodes.push(Opcode::WarpRectilinear {
                    kr: warp.kr,
                    kt: warp.kt,
                    cx: warp.cx,
                    cy: warp.cy,
                    planes: warp.planes,
                });
            }
            OPCODE_ID_FIX_BAD_PIXELS_CONSTANT => {
                let (constant, bayer_phase) = parse_fix_bad_pixels_constant_params(params, id)?;
                opcodes.push(Opcode::FixBadPixelsConstant {
                    constant,
                    bayer_phase,
                });
            }
            other => {
                if flags & FLAG_OPTIONAL != 0 {
                    opcodes.push(Opcode::Unknown {
                        id: other,
                        flags,
                        params: params.to_vec(),
                    });
                } else {
                    return Err(Error::UnsupportedMandatoryOpcode { id: other });
                }
            }
        }
    }

    Ok(opcodes)
}

/// Parse an `OpcodeList` byte stream and return its `WarpRectilinear`
/// opcode, if any — the entry point [`crate::develop::develop_into`] calls.
///
/// `None` when the list contains no `WarpRectilinear` opcode (a well-formed
/// list is not an error just because this camera's `OpcodeList3` carries a
/// different opcode, or none at all).
///
/// # Errors
///
/// Whatever [`parse_opcode_list`] returns.
pub fn parse_warp_rectilinear(bytes: &[u8]) -> Result<Option<WarpRect>, Error> {
    let opcodes = parse_opcode_list(bytes)?;
    Ok(opcodes.into_iter().find_map(|op| match op {
        Opcode::WarpRectilinear {
            kr,
            kt,
            cx,
            cy,
            planes,
        } => Some(WarpRect {
            kr,
            kt,
            cx,
            cy,
            planes,
        }),
        Opcode::FixBadPixelsConstant { .. } | Opcode::Unknown { .. } => None,
    }))
}

/// Parse an `OpcodeList` byte stream and return its `FixBadPixelsConstant`
/// opcode's `constant` marker value, if any — the entry point
/// [`crate::develop::develop_into`] calls (`SPEC-017`).
///
/// `None` when the list contains no `FixBadPixelsConstant` opcode. Mirrors
/// [`parse_warp_rectilinear`]'s shape; `bayer_phase` is not returned because
/// no caller consumes it (module docs, `## Non-Goals` — `AC1`'s round-trip
/// test is what reads it, satisfying the "unread field" rule without
/// forwarding it downstream).
///
/// # Errors
///
/// Whatever [`parse_opcode_list`] returns — including
/// [`Error::UnsupportedMandatoryOpcode`] if `bytes` carries a DIFFERENT,
/// unrecognized mandatory opcode (`AC6`): the dispatch happens inside
/// [`parse_opcode_list`] itself, not in a second layer here.
pub fn parse_fix_bad_pixels_constant(bytes: &[u8]) -> Result<Option<u32>, Error> {
    let opcodes = parse_opcode_list(bytes)?;
    Ok(opcodes.into_iter().find_map(|op| match op {
        Opcode::FixBadPixelsConstant { constant, .. } => Some(constant),
        Opcode::WarpRectilinear { .. } | Opcode::Unknown { .. } => None,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Required-processing validation — `SPEC-054`, `DEC-049`
// ─────────────────────────────────────────────────────────────────────────────

/// Which develop path is asking, and so which opcodes it applies (`DEC-049`).
///
/// Passed to [`check_required_processing`] by anyone composing the stage
/// primitives themselves (`SPEC-056`, `DEC-053`). `#[non_exhaustive]`: a later
/// path (or a later opcode a path learns to apply) is an addition, not a break.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DevelopPath {
    /// The monochrome path, [`crate::develop::develop_into`]. It applies the
    /// first `FixBadPixelsConstant` in `OpcodeList1` and the first
    /// `WarpRectilinear` in `OpcodeList3`, and nothing from `OpcodeList2`.
    Mono,
    /// The colour path: [`crate::color::develop_color_into`] and its `f32` /
    /// threaded variants, and the colour stage primitives
    /// ([`crate::demosaic::demosaic_into`],
    /// [`crate::demosaic::demosaic_into_threaded`],
    /// [`crate::color::to_srgb_linear_into`]). It applies no opcode at all, so
    /// any mandatory opcode in any list is refused.
    Colour,
}

impl DevelopPath {
    /// The opcode ID this path applies from `OpcodeList{list}` — its FIRST
    /// instance only, because the mono appliers resolve the list with
    /// `find_map` — or `None` when the path applies nothing from that list.
    /// Nothing applies `OpcodeList2`.
    fn applied_id(self, list: u8) -> Option<u32> {
        match (self, list) {
            (DevelopPath::Mono, 1) => Some(OPCODE_ID_FIX_BAD_PIXELS_CONSTANT),
            (DevelopPath::Mono, 3) => Some(OPCODE_ID_WARP_RECTILINEAR),
            _ => None,
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
/// [`crate::warp::apply_warp_into`] (`OpcodeList3`) and
/// [`crate::develop::apply_fix_bad_pixels_constant`] (`OpcodeList1`). Call it
/// with the path whose opcodes your composition applies; if you apply neither
/// opcode, ask with [`DevelopPath::Colour`].
///
/// Lists are checked in order 1, 2, 3. For each list:
///
/// 1. If its tag is present but unreadable (recorded in
///    [`Sensor::malformed_tags`], `DEC-012`), the check refuses: the library
///    cannot show that a list it cannot read holds no mandatory opcode.
/// 2. Otherwise every **mandatory** opcode in it (`Flags` bit 0 unset) must be
///    exactly the instance `path` applies (the first opcode with that path's
///    ID in that list). Optional opcodes are always skippable.
///
/// Reads only the sensor's fields; allocates nothing and never panics on a
/// hostile `Sensor`.
///
/// # Errors
///
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

/// The body of [`check_required_processing`] over raw list bytes
/// (`OpcodeList1`/`2`/`3`, in that order; `None` = absent or unreadable) and
/// each list's "present but unreadable" flag.
///
/// This walks the **raw framing** with flags, not [`parse_opcode_list`]'s typed
/// output: the typed parse keeps `Flags` only on [`Opcode::Unknown`], and cannot
/// see list position or a second instance, so it cannot answer the question.
///
/// Rule, per opcode in list `n`: `Flags` bit 0 set (optional) → skippable.
/// Unset (mandatory) → it must be exactly the instance `path` applies (the
/// first opcode with [`DevelopPath::applied_id`]'s ID in that list, whatever
/// that first instance's own flags); otherwise:
///
/// - a recognized opcode ID (`WarpRectilinear`, `FixBadPixelsConstant`) →
///   [`Error::UnsupportedRequiredOpcode`] `{ list: n, id }`;
/// - an unrecognized ID → [`Error::UnsupportedMandatoryOpcode`] `{ id }`, the
///   same error [`parse_opcode_list`] has always given for it.
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
            check_list(bytes, list, path.applied_id(list))?;
        }
    }
    Ok(())
}

/// One list of [`check_lists`]: `applied_id` is the ID whose
/// first instance this path applies from this list, if any.
fn check_list(bytes: &[u8], list: u8, applied_id: Option<u32>) -> Result<(), Error> {
    let count = read_u32_be(bytes, 0)?;
    let mut pos: usize = 4;
    let mut applied_seen = false;

    for _ in 0..count {
        let RawOpcode { id, flags, .. } = read_raw_opcode(bytes, &mut pos)?;

        // The applier consumes the first instance of its ID, optional or not;
        // every later instance is not applied.
        let is_applied = applied_id == Some(id) && !applied_seen;
        if applied_id == Some(id) {
            applied_seen = true;
        }

        if is_applied || flags & FLAG_OPTIONAL != 0 {
            continue;
        }
        return Err(match id {
            OPCODE_ID_WARP_RECTILINEAR | OPCODE_ID_FIX_BAD_PIXELS_CONSTANT => {
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
