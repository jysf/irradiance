//! Fuzz target for level normalization and geometry (`SPEC-014`).
//!
//! Shipped in the SAME change as `src/develop.rs` — AGENTS.md §12 bar 2.
//! Geometry is a NEW input surface over attacker-controlled `ActiveArea`,
//! `DefaultCropOrigin`/`Size` and `Orientation` — none of which `plane.rs`'s
//! own fuzz target exercises past the tag-read stage. This target chains the
//! full pipeline: parse -> locate the sensor -> unpack the plane -> develop
//! it, so a byte string has to survive every earlier stage before it can
//! reach the geometry code at all. The contract is the same one
//! `fuzz_targets/plane.rs` states: for every byte string, every entry point
//! below returns, and none of them panics.
//!
//! ⚠ Needs the same `+toolchain` handling as `ifd`/`plane` — see those
//! files' headers.
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run develop \
//!     fuzz/corpus/develop fuzz/seeds/develop -- -max_total_time=60
//! ```
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`examples/fuzz-seeds.rs`) — no corpus file is used as a seed
//! (`DEC-003`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::develop::{develop_into, output_dimensions};
use irradiance::ifd::Container;
use irradiance::plane::unpack_into;

/// Same cap `fuzz_targets/plane.rs` uses, for the same reason: bounding a
/// hostile `width x height` before THIS HARNESS allocates its own buffers —
/// not a library guarantee, `unpack_into`/`develop_into` allocate nothing
/// themselves (`DEC-016`, `DEC-018`).
const MAX_PIXELS: u64 = 4_000_000;

fuzz_target!(|data: &[u8]| {
    let Ok(container) = Container::parse(data) else {
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

    // The geometry surface under test: a rejected shape is a SUCCESS for
    // this target, same as a rejected container above.
    let Ok((out_width, out_height)) = output_dimensions(&sensor) else {
        return;
    };
    let Some(out_pixels) = u64::from(out_width).checked_mul(u64::from(out_height)) else {
        return;
    };
    if out_pixels > MAX_PIXELS {
        return;
    }
    let Ok(out_len) = usize::try_from(out_pixels) else {
        return;
    };

    let mut developed = vec![0u16; out_len];
    let _ = develop_into(&sensor, &plane, &mut developed);
});
