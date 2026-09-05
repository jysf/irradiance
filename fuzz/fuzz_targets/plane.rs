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

    let mut dst = vec![0u16; len];
    let _ = unpack_into(&sensor, container.byte_order(), data, &mut dst);
});
