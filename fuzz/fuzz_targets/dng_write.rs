//! Fuzz target for the public DNG writer (`SPEC-061` §8, AGENTS.md §12 bar 2),
//! shipped in the same change as the writer.
//!
//! Each input decodes (`tests/support/dng_write_fuzz.rs`, shared with the seed
//! generator) to a plane of at most 64×64 and every `WriteOptions` field plus
//! the pixel budget. Properties:
//!
//! 1. Nothing panics.
//! 2. `written_len` and `write_into` agree: the same `Ok` length (and then
//!    `write_into` into a buffer of exactly that length succeeds), or the same
//!    `Err`, compared by `Debug`, with `write_into` given a buffer of any
//!    length.
//! 3. On `Ok`, the output parses, `sensor()` succeeds, and `unpack_into`
//!    returns the expected samples: the whole plane at full-sensor framing,
//!    otherwise the active-area cut.
//! 4. On `Ok` with a preview (`SPEC-082`), `previews()` yields exactly one
//!    `Ok` preview whose bytes are the caller's JPEG; without one, none.
//!
//! Run with the rustup shim first on PATH (see `fuzz_targets/ifd.rs`):
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run dng_write \
//!     fuzz/corpus/dng_write fuzz/seeds/dng_write -- -max_total_time=60
//! ```

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::dng::write::{write_into, written_len, Framing};
use irradiance::ifd::{ActiveArea, Container};
use irradiance::plane::unpack_into;

#[path = "../../tests/support/dng_write_fuzz.rs"]
mod dng_write_fuzz;

fuzz_target!(|data: &[u8]| {
    let case = dng_write_fuzz::decode(data);
    let owned = case.owned();
    let options = case.options(&owned);
    let (w, h) = (case.width, case.height);
    let budget = usize::from(case.max_pixels);

    let len = written_len(&case.samples, w, h, &options, budget);
    let len = match len {
        Err(e) => {
            // The same refusal from write_into, whatever dst holds.
            let mut dst = [0u8; 16];
            let again = write_into(&case.samples, w, h, &options, budget, &mut dst)
                .expect_err("write_into accepted what written_len refused");
            assert_eq!(format!("{e:?}"), format!("{again:?}"));
            let _ = e.kind();
            let _ = e.to_string();
            return;
        }
        Ok(len) => len,
    };

    let mut out = vec![0xA5u8; len];
    write_into(&case.samples, w, h, &options, budget, &mut out)
        .expect("write_into refused what written_len accepted");

    let container = Container::parse(&out).expect("the written file parses");
    let sensor = container.sensor().expect("the written file has a sensor");
    let mut plane = vec![0u16; (sensor.width as usize) * (sensor.height as usize)];
    unpack_into(&sensor, container.byte_order(), &out, &mut plane)
        .expect("the written plane unpacks");

    let whole = ActiveArea {
        top: 0,
        left: 0,
        bottom: h,
        right: w,
    };
    let area = match options.framing {
        Framing::FullSensor => whole,
        _ => options.active_area.unwrap_or(whole),
    };
    let width = w as usize;
    let expected: Vec<u16> = (area.top as usize..area.bottom as usize)
        .flat_map(|row| {
            let start = row * width;
            case.samples[start + area.left as usize..start + area.right as usize]
                .iter()
                .copied()
        })
        .collect();
    assert_eq!(plane, expected, "the written samples read back unchanged");

    let previews: Vec<_> = container.previews().collect();
    match options.preview {
        Some(p) => assert!(
            matches!(previews.as_slice(), [Ok(q)] if q.bytes == p.jpeg),
            "the preview reads back byte for byte: {previews:?}"
        ),
        None => assert!(previews.is_empty(), "no preview written: {previews:?}"),
    }
});
