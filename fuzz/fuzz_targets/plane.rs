//! Fuzz target for the sensor-plane unpacker (`SPEC-012`, `DEC-008`).
//!
//! Shipped in the SAME change as the unpacker — AGENTS.md §12 bar 2. This
//! target reaches BOTH of `DEC-008`'s paths (`bits % 8 == 0` and not): a
//! target that only ever drives one recreates `SPIKE-001`'s exact blind spot
//! (`DEC-008`'s own Consequences section says so). The contract is the same
//! one `fuzz_targets/ifd.rs` states: for every byte string, every entry point
//! below returns, and none of them panics.
//!
//! ⚠ Needs the same `+toolchain` handling as `ifd` — see that file's header.
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run plane \
//!     fuzz/corpus/plane fuzz/seeds/plane -- -max_total_time=60
//! ```
//!
//! Since `SPEC-045` it also drives the DNG `Compression = 7` route
//! (`unpack_into` -> `ljpeg::decode_into`) from a synthetic SOF-3 seed
//! (`compression7-sof3.tiff`), so the public lossless-JPEG path is fuzzed
//! through the container, not only via the direct `ljpeg` target.
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`examples/fuzz-seeds.rs`) — no corpus file is used as a seed
//! (`DEC-003`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::ifd::Container;
use irradiance::plane::unpack_into;

/// Cap the allocation THIS TARGET is willing to make. `unpack_into` itself
/// allocates nothing (`DEC-016`) — bounding a hostile `width x height` before
/// this harness allocates ITS OWN destination buffer is this target's
/// choice, not a library guarantee being weakened. A real caller with a real
/// buffer may attempt any size; this target is only declining to chase an
/// OOM on `width = height = u32::MAX`, which `unpack_into` would in any case
/// reject via `PlaneBufferWrongLength` the moment such a buffer failed to
/// materialise.
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

    let mut dst = vec![0u16; len];
    let _ = unpack_into(&sensor, container.byte_order(), data, &mut dst);
});
