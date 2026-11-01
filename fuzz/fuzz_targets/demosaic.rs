//! Fuzz target for the Bayer demosaic (`SPEC-032`).
//!
//! `src/demosaic.rs` (Hamilton-Adams, `SPEC-030`/`SPEC-031`) is already
//! phase-driven and panic-free; this target makes that a *fuzzed* guarantee
//! rather than one the 1×1 / 2×2 / 3×5 unit tests assert on a handful of
//! shapes. It chains the full pipeline — parse → locate the sensor → unpack
//! the mosaic plane → demosaic it — so a byte string has to survive every
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

use irradiance::demosaic::demosaic_into;
use irradiance::ifd::Container;
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
    if sensor.require_uncompressed().is_err() {
        return;
    }

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
});
