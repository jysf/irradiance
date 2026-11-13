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
//!
//! **`warp_alpha` (`SPEC-064`, `DEC-059`).** The same byte's bits 2-3 select
//! `warp_alpha` from `{1.0, 0.0, 0.5, NaN}` (in that order), and bit 4
//! replaces the choice with `1.5`, out of range. So the lerped warp, the
//! `FullActiveArea` bounds rule and the `InvalidWarpAlpha` refusal are all
//! reached. Bits 5-7 are unused.
//!
//! ```text
//! bit 0     FullActiveArea
//! bit 1     apply_orientation = false
//! bits 2-3  warp_alpha: 00 = 1.0, 01 = 0.0, 10 = 0.5, 11 = NaN
//! bit 4     warp_alpha = 1.5 (overrides bits 2-3)
//! ```
//!
//! **Scratch differential (`SPEC-065`, `DEC-060`).** After `develop_with_into`,
//! the same input and options go through `scratch_len` and
//! `develop_with_scratch_into` with a scratch prefilled with `0xA5A5`. The
//! `Result` (compared by `Debug`) and `dst` must equal `develop_with_into`'s,
//! so the shared core's byte identity and error order are fuzzed, not only
//! its freedom from panics. A scratch one sample long must be refused with
//! `DevelopScratchWrongLength` whenever `develop_with_into` succeeded. The
//! existing seeds reach all four `DEC-060` §3 cases.

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::develop::{
    develop_into, develop_with_into, develop_with_scratch_into, output_dimensions,
    output_dimensions_with, scratch_len, DevelopOptions, Framing,
};
use irradiance::ifd::Container;
use irradiance::plane::unpack_into;
use irradiance::Error;

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
    options.warp_alpha = match (option_byte >> 2) & 0b11 {
        0 => 1.0,
        1 => 0.0,
        2 => 0.5,
        _ => f64::NAN,
    };
    if option_byte & 0b1_0000 != 0 {
        options.warp_alpha = 1.5;
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
    let reference = develop_with_into(&sensor, &plane, &mut developed_with, &options);

    // SPEC-065: the scratch path must agree with the allocating one exactly.
    let Ok(needed) = scratch_len(&sensor, &options) else {
        return;
    };
    if u64::try_from(needed).map_or(true, |n| n > MAX_PIXELS * 3) {
        return;
    }
    let mut scratch = vec![0xA5A5u16; needed];
    let mut developed_scratch = vec![0u16; opt_len];
    let result = develop_with_scratch_into(
        &sensor,
        &plane,
        &mut developed_scratch,
        &mut scratch,
        &options,
    );
    assert_eq!(
        format!("{result:?}"),
        format!("{reference:?}"),
        "develop_with_scratch_into's result differs from develop_with_into's"
    );
    assert!(
        developed_scratch == developed_with,
        "develop_with_scratch_into's output differs from develop_with_into's"
    );

    let mut long = vec![0xA5A5u16; needed + 1];
    let mut developed_long = vec![0u16; opt_len];
    let result =
        develop_with_scratch_into(&sensor, &plane, &mut developed_long, &mut long, &options);
    if reference.is_ok() {
        assert!(
            matches!(result, Err(Error::DevelopScratchWrongLength { .. })),
            "a scratch one sample long was not refused: {result:?}"
        );
    }
});
