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
//! Since `SPEC-039` (`DEC-050`) it drives the public preflight contract
//! `sensor -> validate -> allocate(capped) -> unpack_into` and asserts it both
//! ways: `validate` `Ok(n)` on an uncompressed plane means `unpack_into` into
//! `n` samples is `Ok` and `n * bits <= 8 * len` (the file-bytes bound); a
//! `validate` `Err` means `unpack_into` is also `Err`. Seeds include forged
//! dimensions on both paths and aliased strips.
//!
//! Since `SPEC-068` (`DEC-063`) it also runs the units differential after every
//! `unpack_into` it makes, at `max_band_rows = 1 + (last byte % 64)`: on `Ok`,
//! `units` is `Ok`, its rows tile `0..height` exactly, and every unit decoded
//! in REVERSE order into a second plane is `Ok` and that plane equals the
//! first; on `Err`, `units` returns that error, or it is `Ok` and the units in
//! forward order fail with that error at the first failing unit.
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`examples/fuzz-seeds.rs`) — no corpus file is used as a seed
//! (`DEC-003`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use std::num::NonZeroU32;

use irradiance::ifd::{ByteOrder, Compression, Container, Sensor};
use irradiance::plane::{unpack_into, unpack_unit_into, units, validate, Unit};
use irradiance::Error;

/// The pixel budget this harness passes to `validate` — the caller's number
/// (`DEC-050` §2), chosen here so the harness never chases an OOM. It is the
/// LIBRARY that enforces it: `validate` refuses anything larger with
/// `PlaneTooLarge` before this target allocates.
const MAX_PIXELS: usize = 4_000_000;

/// A second, independent cap on what this harness itself allocates — a
/// belt-and-braces bound that does not rely on the library honouring
/// `MAX_PIXELS` (if it did not, the assertion below catches it).
const HARNESS_CAP: usize = 4_000_000;

fuzz_target!(|data: &[u8]| {
    let Ok(container) = Container::parse(data) else {
        // A rejected container is a SUCCESS for this target: the reader said
        // no with a typed error instead of panicking.
        return;
    };
    let Ok(sensor) = container.sensor() else {
        return;
    };
    let order = container.byte_order();

    // No `require_uncompressed` gate (`SPEC-045`): a DNG `Compression = 7`
    // plane goes through the same preflight and `unpack_into`'s SOF-3 route.
    match validate(&sensor, order, data, MAX_PIXELS) {
        Ok(n) => {
            assert!(n <= MAX_PIXELS, "validate returned {n} over the budget");
            assert!(n <= HARNESS_CAP);
            let pixels = u64::from(sensor.width) * u64::from(sensor.height);
            assert_eq!(n as u64, pixels, "Ok(n) must be width * height");

            let mut dst = vec![0u16; n];
            let unpacked = unpack_into(&sensor, order, data, &mut dst);
            check_units(&sensor, order, data, &unpacked, &dst);
            if sensor.compression == Compression::Uncompressed {
                // The uncompressed contract: Ok(n) means the unpack cannot
                // fail, and the plane is bounded by the bytes supplied.
                assert!(
                    unpacked.is_ok(),
                    "validate Ok({n}) but unpack_into failed: {unpacked:?}"
                );
                let bits = u64::from(sensor.bits_per_sample);
                assert!(
                    (n as u64) * bits <= 8 * data.len() as u64,
                    "Ok({n}) at {bits} bits breaks n*bits <= 8*len ({})",
                    data.len()
                );
            } else if let Err(e) = unpacked {
                // Compression-7: validate is structural only, so only the
                // decode itself may still fail — never a layout/size error.
                assert!(
                    !matches!(
                        e,
                        Error::PlaneBufferWrongLength { .. }
                            | Error::UnsupportedStripLayout { .. }
                            | Error::MissingRowsPerStrip { .. }
                            | Error::UnsupportedCompression { .. }
                    ),
                    "validate Ok({n}) but unpack_into hit a structural error: {e:?}"
                );
            }
        }
        Err(Error::PlaneTooLarge { .. }) => {
            // Over budget: the harness allocates nothing. (validate-only.)
        }
        Err(_) => {
            // validate refused: unpack_into with a correctly sized dst must
            // refuse too (only when that dst is small enough to make).
            let pixels = u64::from(sensor.width) * u64::from(sensor.height);
            if let Ok(len) = usize::try_from(pixels) {
                if len <= HARNESS_CAP {
                    let mut dst = vec![0u16; len];
                    let unpacked = unpack_into(&sensor, order, data, &mut dst);
                    assert!(unpacked.is_err(), "validate Err but unpack_into Ok");
                    check_units(&sensor, order, data, &unpacked, &dst);
                }
            }
            // And with a tiny (wrong-size) dst it is an Err as well.
            let mut tiny = [0u16; 1];
            assert!(
                unpack_into(&sensor, order, data, &mut tiny).is_err(),
                "validate Err but unpack_into into a 1-sample dst Ok"
            );
        }
    }
});

/// The most units the differential walks for one input (`SPEC-068`).
const UNIT_CAP: usize = 1 << 16;

/// The `SPEC-068` units differential against `unpack_into`'s own `result` and
/// `plane` (DEC-063 §3: both run the same kernel over the same units).
fn check_units(
    sensor: &Sensor,
    order: ByteOrder,
    data: &[u8],
    result: &Result<(), Error>,
    plane: &[u16],
) {
    let band = NonZeroU32::new(1 + u32::from(data.last().copied().unwrap_or(0) % 64))
        .expect("1..=64 is non-zero");
    let width = sensor.width as usize;
    let listed = units(sensor, data, band);
    match (result, listed) {
        // The harness's own bound: a caller-chosen band of 1 on a zero-width
        // plane is `height` empty units, a legal answer this harness does not
        // collect.
        (_, Ok(it)) if it.len() > UNIT_CAP => {}
        (Ok(()), Ok(it)) => {
            let len = it.len();
            let us: Vec<Unit> = it.collect();
            assert_eq!(us.len(), len, "units: len() disagrees with the count yielded");
            let mut next = 0u32;
            for u in &us {
                assert_eq!(u.row_start, next, "units leave a gap or overlap at row {next}");
                next = u.row_start + u.row_count;
            }
            // A plane with no samples (zero width) is accepted by the
            // sample-counted coverage check whatever rows its strips declare,
            // and a zero-width tiled plane has no units at all; the rows must
            // reach `height` only when there are samples to place.
            if !plane.is_empty() {
                assert_eq!(next, sensor.height, "units do not end at the plane height");
            }
            let mut second = vec![0u16; plane.len()];
            for u in us.iter().rev() {
                let at = u.row_start as usize * width;
                let n = u.samples(sensor.width).expect("band fits usize");
                unpack_unit_into(sensor, order, data, u, &mut second[at..at + n])
                    .unwrap_or_else(|e| panic!("unpack_into Ok but unit {u:?} failed: {e:?}"));
            }
            assert!(second == plane, "units in reverse order differ from unpack_into");
        }
        (Ok(()), Err(e)) => panic!("unpack_into Ok but units refused: {e:?}"),
        (Err(want), Err(got)) => {
            assert_eq!(format!("{got:?}"), format!("{want:?}"), "units vs unpack_into");
        }
        (Err(want), Ok(it)) => {
            let mut scratch = vec![0u16; plane.len()];
            let mut first = None;
            for u in it {
                let at = u.row_start as usize * width;
                let n = u.samples(sensor.width).expect("band fits usize");
                if let Err(e) = unpack_unit_into(sensor, order, data, &u, &mut scratch[at..at + n]) {
                    first = Some(e);
                    break;
                }
            }
            let got = first.unwrap_or_else(|| panic!("unpack_into {want:?} but every unit Ok"));
            assert_eq!(format!("{got:?}"), format!("{want:?}"), "first failing unit");
        }
    }
}
