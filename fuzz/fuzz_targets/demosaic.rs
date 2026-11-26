//! Fuzz target for the Bayer demosaic (`SPEC-032`).
//!
//! `src/demosaic.rs` (Hamilton-Adams, `SPEC-030`/`SPEC-031`) is already
//! phase-driven and panic-free; this target makes that a *fuzzed* guarantee
//! rather than one the 1×1 / 2×2 / 3×5 unit tests assert on a handful of
//! shapes. It chains the full pipeline — parse → locate the sensor → unpack
//! the mosaic plane → demosaic it, then develop it to the sensor-geometry
//! linear-sRGB plane, whole and as a fuzzer-chosen row band (`SPEC-091`) —
//! so a byte string has to survive every
//! earlier stage before it can reach the kernel at all; a byte string that is
//! rejected anywhere along the way is a SUCCESS for this target, exactly as in
//! `fuzz_targets/develop.rs`. The contract is the one every target states:
//! **for every byte string, every entry point below returns, and none of them
//! panics** (`no-panics-on-untrusted-input`). The tier-A CI stand-in — asserted
//! without nightly — is `tests/demosaic_oracle.rs::demosaic_is_panic_free_on_hostile_cfa`.
//!
//! ⚠ Needs the same `+toolchain` handling as `ifd`/`plane`/`develop` — see
//! those files' headers (`cargo fuzz` shells out to a bare `"cargo" "build"`
//! that resolves to Homebrew's stable cargo and rejects `-Zsanitizer`, so the
//! rustup shim must be FIRST on PATH, not merely invoked).
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run demosaic \
//!     fuzz/corpus/demosaic fuzz/seeds/demosaic -- -max_total_time=60
//! ```
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`examples/fuzz-seeds.rs`) — no corpus file is used as a seed
//! (`DEC-003`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::color::{
    mono_mix_plane_into, mono_mix_plane_rows_into, mono_mix_row, srgb_linear_plane_f32_into,
    srgb_linear_plane_into, srgb_linear_plane_rows_f32_into, srgb_linear_plane_rows_into,
};
use irradiance::demosaic::demosaic_into;
use irradiance::ifd::{Container, Rational};
use irradiance::plane::unpack_into;

/// Same cap `fuzz_targets/plane.rs` and `develop.rs` use, for the same reason:
/// bounding a hostile `width x height` before THIS HARNESS allocates its own
/// buffers — `demosaic_into` itself allocates nothing (`DEC-016`). The RGB
/// destination is three samples per pixel, so `pixel_count * 3` is bounded the
/// same way before the harness allocates it.
const MAX_PIXELS: u64 = 4_000_000;

fuzz_target!(|data: &[u8]| {
    let Ok(container) = Container::parse(data) else {
        // A rejected container is a SUCCESS for this target: the reader said
        // no with a typed error instead of panicking.
        return;
    };
    let Ok(sensor) = container.sensor() else {
        return;
    };
    // No `require_uncompressed` gate (`SPEC-045`): a DNG `Compression = 7`
    // plane goes through `unpack_into`'s SOF-3 route like any other, so the
    // PUBLIC Compression-7 path is fuzzed end to end. `unpack_into` itself
    // returns `UnsupportedCompression` for every other code.

    let Some(pixel_count) = u64::from(sensor.width).checked_mul(u64::from(sensor.height)) else {
        return;
    };
    if pixel_count > MAX_PIXELS {
        return;
    }
    let Ok(len) = usize::try_from(pixel_count) else {
        return;
    };

    let mut plane = vec![0u16; len];
    if unpack_into(&sensor, container.byte_order(), data, &mut plane).is_err() {
        return;
    }

    // The demosaic surface under test. `dst` is `pixel_count * 3`; bound that
    // product against MAX_PIXELS the same way the plane itself is bounded. A
    // rejected CFA (mono plane / non-Bayer pattern / wrong-length buffer) is a
    // SUCCESS here, same as a rejected container above.
    let Some(rgb_count) = pixel_count.checked_mul(3) else {
        return;
    };
    if rgb_count > MAX_PIXELS {
        return;
    }
    let Ok(rgb_len) = usize::try_from(rgb_count) else {
        return;
    };

    let mut rgb = vec![0u16; rgb_len];
    let _ = demosaic_into(&sensor, &plane, &mut rgb);

    // SPEC-091: the sensor-geometry linear-sRGB plane, whole (into the same
    // `width·height·3` buffers) and as a band over a fuzzer-chosen row range,
    // valid or not. The range comes from the input's last two bytes, so an
    // empty, reversed or over-long range (`InvalidRowRange`) is reachable as
    // well as every valid one; `dst` is sized for the range as asked, saturating,
    // so a reversed range still meets a buffer and the range check refuses it.
    let _ = srgb_linear_plane_into(&sensor, &plane, &mut rgb);
    let mut rgb_f32 = vec![0f32; rgb_len];
    let _ = srgb_linear_plane_f32_into(&sensor, &plane, &mut rgb_f32);

    let pick = |back: usize| {
        data.len()
            .checked_sub(back)
            .and_then(|i| data.get(i))
            .map_or(0, |&b| u32::from(b))
    };
    // Up to two rows past the height, so `end > height` is reachable.
    let span = sensor.height.saturating_add(2);
    let start = pick(2).checked_rem(span).unwrap_or(0);
    let end = pick(1).checked_rem(span).unwrap_or(0);
    let band_rows = u64::from(end.saturating_sub(start));
    let Some(band_len) = band_rows
        .checked_mul(u64::from(sensor.width))
        .and_then(|v| v.checked_mul(3))
        .filter(|&v| v <= MAX_PIXELS)
        .and_then(|v| usize::try_from(v).ok())
    else {
        return;
    };
    // A third byte sometimes makes `dst` one short or one long, so the band
    // forms' `RgbBufferWrongLength` is reachable too (SPEC-091 verify); half the
    // inputs keep the exact length, so `Ok` stays common.
    let band_len = match pick(3) % 4 {
        2 => band_len.saturating_sub(1),
        3 => band_len.saturating_add(1),
        _ => band_len,
    };
    let mut band = vec![0u16; band_len];
    let _ = srgb_linear_plane_rows_into(&sensor, &plane, start..end, &mut band);
    let mut band_f32 = vec![0f32; band_len];
    let _ = srgb_linear_plane_rows_f32_into(&sensor, &plane, start..end, &mut band_f32);

    // SPEC-092: the mono mixer. Half the inputs mix with valid luminance
    // weights; the rest take three signed-byte rationals from the input, so
    // a zero or negative denominator, a negative numerator, a sum other than
    // one (`InvalidMixWeights`) and the odd valid set are all reachable.
    let weights = if pick(4) % 2 == 0 {
        [
            Rational { num: 2126, den: 10000 },
            Rational { num: 7152, den: 10000 },
            Rational { num: 722, den: 10000 },
        ]
    } else {
        let signed = |back: usize| i32::from(i8::from_le_bytes([pick(back) as u8]));
        [
            Rational { num: signed(5), den: signed(6) },
            Rational { num: signed(7), den: signed(8) },
            Rational { num: signed(9), den: signed(10) },
        ]
    };
    let _ = mono_mix_row(&sensor, &weights);
    // `dst` one short or one long on half the inputs (`MonoBufferWrongLength`).
    let resize = |len: usize, byte: u32| match byte % 4 {
        2 => len.saturating_sub(1),
        3 => len.saturating_add(1),
        _ => len,
    };
    let mono_len = resize(rgb_len / 3, pick(11));
    let mut mono = vec![0u16; mono_len];
    let _ = mono_mix_plane_into(&sensor, &plane, &weights, &mut mono);
    let Some(mono_band_len) = band_rows
        .checked_mul(u64::from(sensor.width))
        .and_then(|v| usize::try_from(v).ok())
    else {
        return;
    };
    let mut mono_band = vec![0u16; resize(mono_band_len, pick(3))];
    let _ = mono_mix_plane_rows_into(&sensor, &plane, &weights, start..end, &mut mono_band);
});
