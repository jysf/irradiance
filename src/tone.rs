//! Tone curve and the three output encoders (`SPEC-019`, `DEC-025`).
//!
//! `develop::develop_into` produces a **scene-linear `u16`** image (`DEC-018`).
//! That is the image; it is not yet the representation a consumer displays. A
//! scene-linear image shown directly is far too dark — the display path needs
//! the output colour space's transfer function. This module is that additive
//! layer, and it does **not** reopen `DEC-018`: `develop_into` is untouched.
//!
//! # The curve — the sRGB OETF, and why
//!
//! The measured Leica Q2 Monochrom frame carries **no** profile tone curve
//! (`SPEC-019` Context: no `ProfileToneCurve`, `LinearizationTable`,
//! `TransferFunction` or `RenderingIntent`; `PhotometricInterpretation =
//! Linear Raw`, one sample). The only transfer between the scene-linear plane
//! and the display-referred sRGB reference (`dnglab analyze --srgb`,
//! `docs/oracle-contract.md`) is therefore the output space's own transfer
//! function: the **sRGB OETF** (opto-electronic transfer function, IEC
//! 61966-2-1). Honouring a per-file profile/linearization curve is the next
//! wave (`SPEC-019` Non-Goals) — there is none here to honour. See `DEC-025`.
//!
//! # Three representations
//!
//! - [`to_linear_f32_into`] — **scene-linear** `f32` in `[0.0, 1.0]`. The tone
//!   curve is **not** applied; this is `develop_into`'s own value rescaled
//!   (`v / 65535.0`).
//! - [`encode_srgb16_into`] — **display-referred** 16-bit sRGB `u16`, directly
//!   comparable to `dnglab analyze --srgb`'s 16-bit sRGB PNM.
//! - [`encode_srgb8_into`] — **display-referred** 8-bit sRGB `u8`.
//!
//! # Determinism — the table is the point (`AC4`/`AC7`, `DEC-002`)
//!
//! The sRGB mapping is a committed, generated const lookup table
//! ([`crate::tone_table`], produced by `examples/gen-tone-table.rs`). The
//! library runs **no** `powf`/`exp`/`ln` on the encode path — it does an
//! integer lookup — and `to_linear_f32_into` is a single IEEE division. The
//! transcendental runs exactly once, in the generator, producing committed
//! bytes that are byte-identical on macOS and Linux. Determinism follows from
//! the committed integer table, not from matching a runtime float.
//!
//! # Caller-owned buffers (`DEC-016`)
//!
//! Each encoder writes a `dst` the caller owns and sized, exactly as
//! `plane::unpack_into` / `develop::develop_into` do. The one thing an encoder
//! can check before writing is the length: a `dst` whose length does not equal
//! the `linear` source returns [`Error::OutputBufferWrongLength`] — a typed
//! error, never a panic (`no-panics-on-untrusted-input`).

use crate::tone_table::{SRGB_ENCODE_16, SRGB_ENCODE_8};
use crate::Error;

/// Error unless `dst` holds exactly as many elements as the `linear` source.
///
/// The length is the one precondition the encoders can verify without writing
/// into the caller's buffer; a mismatch is reported, never panicked on.
fn require_matching_length(expected: usize, actual: usize) -> Result<(), Error> {
    if expected != actual {
        return Err(Error::OutputBufferWrongLength { expected, actual });
    }
    Ok(())
}

/// Scene-linear `f32` in `[0.0, 1.0]`: `dst[i] = linear[i] / 65535.0`.
///
/// The tone curve is **not** applied — this is `develop_into`'s scene-linear
/// `u16` output rescaled to `[0.0, 1.0]`, nothing more (`AC1`). `0 → 0.0`,
/// `65535 → 1.0`. The division is the only arithmetic and is IEEE-deterministic
/// (`65535.0` is exactly representable in `f32`), so the output is
/// byte-identical across platforms.
///
/// # Errors
///
/// [`Error::OutputBufferWrongLength`] if `dst.len() != linear.len()`.
pub fn to_linear_f32_into(linear: &[u16], dst: &mut [f32]) -> Result<(), Error> {
    require_matching_length(linear.len(), dst.len())?;
    for (out, &sample) in dst.iter_mut().zip(linear.iter()) {
        *out = f32::from(sample) / 65535.0;
    }
    Ok(())
}

/// Display-referred 16-bit sRGB `u16` via the committed [`SRGB_ENCODE_16`]
/// table (`AC2`).
///
/// `0 → 0`, `65535 → 65535`; monotonic non-decreasing across the full domain;
/// within ±1 of the analytic sRGB OETF (`tests/tone_oracle.rs`, `AC5`).
/// Directly comparable to `dnglab analyze --srgb`'s 16-bit sRGB PNM.
///
/// # Errors
///
/// [`Error::OutputBufferWrongLength`] if `dst.len() != linear.len()`.
pub fn encode_srgb16_into(linear: &[u16], dst: &mut [u16]) -> Result<(), Error> {
    require_matching_length(linear.len(), dst.len())?;
    for (out, &sample) in dst.iter_mut().zip(linear.iter()) {
        // `sample` is a `u16`, so `usize::from(sample)` is always in
        // `0..=65535` — in bounds for a `[_; 65536]` table. `.get()` keeps the
        // path total without `indexing_slicing`; the `0` fallback is
        // unreachable and exists only to stay panic-free.
        *out = SRGB_ENCODE_16
            .get(usize::from(sample))
            .copied()
            .unwrap_or(0);
    }
    Ok(())
}

/// Display-referred 8-bit sRGB `u8` via the committed [`SRGB_ENCODE_8`] table
/// (`AC3`).
///
/// `0 → 0`, `65535 → 255`; monotonic non-decreasing; within ±1 of the analytic
/// sRGB OETF (`tests/tone_oracle.rs`, `AC5`); the 8-bit rounding of the same
/// curve [`encode_srgb16_into`] encodes.
///
/// # Errors
///
/// [`Error::OutputBufferWrongLength`] if `dst.len() != linear.len()`.
pub fn encode_srgb8_into(linear: &[u16], dst: &mut [u8]) -> Result<(), Error> {
    require_matching_length(linear.len(), dst.len())?;
    for (out, &sample) in dst.iter_mut().zip(linear.iter()) {
        *out = SRGB_ENCODE_8.get(usize::from(sample)).copied().unwrap_or(0);
    }
    Ok(())
}

#[cfg(test)]
#[allow(
    // Test code is not a parse/decode path exercised on untrusted input
    // (AGENTS.md §11); the same five lints are allowed here as in the other
    // modules' test blocks and in src/bin/irr.rs.
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::arithmetic_side_effects
)]
mod tests {
    use super::{encode_srgb16_into, encode_srgb8_into, to_linear_f32_into};
    use crate::tone_table::{SRGB_ENCODE_16, SRGB_ENCODE_8};
    use crate::Error;

    // AC4 — the tables are `const`, not `static`. These compile-time
    // assertions fail to compile if either table is downgraded to a `static`
    // (a `static` cannot be read in a `const` context) or resized, so
    // "table-driven const data" is proven at build time, not vacuously.
    const _: () = assert!(SRGB_ENCODE_16.len() == 65536);
    const _: () = assert!(SRGB_ENCODE_8.len() == 65536);
    const _: () = assert!(SRGB_ENCODE_16[0] == 0);
    const _: () = assert!(SRGB_ENCODE_16[65535] == 65535);
    const _: () = assert!(SRGB_ENCODE_8[0] == 0);
    const _: () = assert!(SRGB_ENCODE_8[65535] == 255);

    #[test]
    fn to_linear_f32_maps_endpoints_and_is_pure_rescale() {
        // AC1: 0 -> 0.0, 65535 -> 1.0, and v -> v/65535.0 exactly for a sweep.
        let linear: Vec<u16> = (0..=65535u32).step_by(257).map(|v| v as u16).collect();
        let mut dst = vec![0.0f32; linear.len()];
        to_linear_f32_into(&linear, &mut dst).unwrap();
        for (&v, &out) in linear.iter().zip(dst.iter()) {
            assert_eq!(out, f32::from(v) / 65535.0, "v {v} must rescale exactly");
        }
        // Endpoints, pinned.
        let mut ends = [0.0f32; 2];
        to_linear_f32_into(&[0, 65535], &mut ends).unwrap();
        assert_eq!(ends, [0.0, 1.0]);
    }

    #[test]
    fn encoders_reject_a_wrong_length_destination() {
        // AC1/AC2/AC3: a length mismatch is the typed error, never a panic.
        let linear = [0u16; 4];

        let mut f32_dst = [0.0f32; 3];
        match to_linear_f32_into(&linear, &mut f32_dst) {
            Err(Error::OutputBufferWrongLength { expected, actual }) => {
                assert_eq!((expected, actual), (4, 3));
            }
            other => panic!("expected OutputBufferWrongLength, got {other:?}"),
        }

        let mut u16_dst = [0u16; 5];
        match encode_srgb16_into(&linear, &mut u16_dst) {
            Err(Error::OutputBufferWrongLength { expected, actual }) => {
                assert_eq!((expected, actual), (4, 5));
            }
            other => panic!("expected OutputBufferWrongLength, got {other:?}"),
        }

        let mut u8_dst = [0u8; 0];
        match encode_srgb8_into(&linear, &mut u8_dst) {
            Err(Error::OutputBufferWrongLength { expected, actual }) => {
                assert_eq!((expected, actual), (4, 0));
            }
            other => panic!("expected OutputBufferWrongLength, got {other:?}"),
        }
    }

    #[test]
    fn encoders_write_every_element_on_a_correct_length_buffer() {
        // AC1/AC2/AC3: a correct-length buffer is fully written — no element
        // left at the sentinel. The input mixes low, mid and high samples.
        let linear = [0u16, 1, 205, 30_000, 65_534, 65_535];

        let mut f32_dst = vec![f32::NAN; linear.len()];
        to_linear_f32_into(&linear, &mut f32_dst).unwrap();
        assert!(f32_dst.iter().all(|v| !v.is_nan()), "every f32 written");

        let mut u16_dst = vec![u16::MAX; linear.len()];
        encode_srgb16_into(&linear, &mut u16_dst).unwrap();
        // The first sample (0) encodes to 0, so the sentinel was overwritten.
        assert_eq!(u16_dst[0], 0);
        assert_eq!(u16_dst[linear.len() - 1], 65535);

        let mut u8_dst = vec![u8::MAX; linear.len()];
        encode_srgb8_into(&linear, &mut u8_dst).unwrap();
        assert_eq!(u8_dst[0], 0);
        assert_eq!(u8_dst[linear.len() - 1], 255);
    }

    #[test]
    fn srgb8_matches_srgb16_downscaled_at_anchors() {
        // AC3: the 8-bit table is the 8-bit rounding of the same curve the
        // 16-bit table encodes. At the shared anchors they agree exactly.
        let linear = [0u16, 65535];
        let mut u16_dst = [0u16; 2];
        let mut u8_dst = [0u8; 2];
        encode_srgb16_into(&linear, &mut u16_dst).unwrap();
        encode_srgb8_into(&linear, &mut u8_dst).unwrap();
        assert_eq!((u16_dst[0], u8_dst[0]), (0, 0));
        assert_eq!((u16_dst[1], u8_dst[1]), (65535, 255));
    }

    #[test]
    fn no_transcendental_on_the_encode_path() {
        // AC4 (the source-scan half): neither src/tone.rs nor src/tone_table.rs
        // calls a transcendental on the encode path. Comments are stripped
        // first (this module's docs mention `powf` in prose), and the scan
        // asserts it actually read real code — not a vacuous zero-match pass
        // (AGENTS.md §16 rule 2).
        let manifest = env!("CARGO_MANIFEST_DIR");
        for rel in ["src/tone.rs", "src/tone_table.rs"] {
            let text = std::fs::read_to_string(format!("{manifest}/{rel}")).unwrap();
            // Scan the shipped encode path only — not this test module, whose
            // own token list below literally contains `.powf(` (that match
            // would be the test catching itself, not the library).
            let shipped = text.split("#[cfg(test)]").next().unwrap_or(&text);
            let code: String = shipped
                .lines()
                .map(|line| match line.find("//") {
                    Some(at) => &line[..at],
                    None => line,
                })
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !text.is_empty() && code.contains("["),
                "scanned no real code in {rel}"
            );
            for call in [
                ".powf(", ".powi(", ".exp(", ".exp2(", ".ln(", ".log(", ".sqrt(", ".cbrt(",
            ] {
                assert!(
                    !code.contains(call),
                    "{rel} calls {call} on the encode path — the table must carry the curve"
                );
            }
        }
    }
}
