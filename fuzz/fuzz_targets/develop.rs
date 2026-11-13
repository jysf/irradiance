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
//!
//! **Options byte (`SPEC-063`, `DEC-058`).** After the plain `develop_into`
//! call, the input's LAST byte selects a [`DevelopOptions`] value: bit 0 sets
//! `Framing::FullActiveArea`, bit 1 clears `apply_orientation`. The same
//! input then goes through `output_dimensions_with` / `develop_with_into`
//! under the same `MAX_PIXELS` cap, so all four option combinations are
//! reached. Appending a byte to a TIFF does not change how it parses, so each
//! seed exists once per trailing option byte (`examples/fuzz-seeds.rs`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::develop::{
    develop_into, develop_with_into, output_dimensions, output_dimensions_with, DevelopOptions,
    Framing,
};
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

    // SPEC-063: the same input under the options the last byte selects.
    let option_byte = data.last().copied().unwrap_or(0);
    let mut options = DevelopOptions::default();
    if option_byte & 1 != 0 {
        options.framing = Framing::FullActiveArea;
    }
    if option_byte & 2 != 0 {
        options.apply_orientation = false;
    }
    let Ok((opt_width, opt_height)) = output_dimensions_with(&sensor, &options) else {
        return;
    };
    let Some(opt_pixels) = u64::from(opt_width).checked_mul(u64::from(opt_height)) else {
        return;
    };
    if opt_pixels > MAX_PIXELS {
        return;
    }
    let Ok(opt_len) = usize::try_from(opt_pixels) else {
        return;
    };
    let mut developed_with = vec![0u16; opt_len];
    let _ = develop_with_into(&sensor, &plane, &mut developed_with, &options);
});
