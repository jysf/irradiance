//! Fuzz target for the TIFF/IFD container reader.
//!
//! `SPEC-003`, shipped in the SAME change as the reader — AGENTS.md §12 bar 2.
//! A retrofitted fuzz target tests the shape the code already has; a
//! designed-in one tests the shape the *input* can take. The contract it
//! asserts is the blocking constraint `no-panics-on-untrusted-input` stated
//! executably: **for every byte string, every entry point below returns, and
//! none of them panics.** Any answer is acceptable. An abort is not.
//!
//! ⚠ This target CANNOT be run with the default PATH. `cargo fuzz` shells out
//! to a bare `"cargo" "build"`, which resolves to Homebrew's stable cargo and
//! rejects `-Zsanitizer=address`. The outer command being nightly does not
//! help — the inner one is what breaks. Put the rustup shim first on PATH:
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd \
//!     fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60
//! ```
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`tests/support/tiff.rs`) — valid containers in both byte
//! orders, truncated headers, cycles, absurd offsets and overflowing counts.
//! No corpus file is used as a seed: tier B is never committed (`DEC-003`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::ifd::Container;

fuzz_target!(|data: &[u8]| {
    let Ok(container) = Container::parse(data) else {
        // A rejected input is a SUCCESS for this target: the reader said no
        // with a typed error instead of panicking.
        return;
    };

    // Walk everything the parse exposed. `parse` only reads entry HEADERS;
    // payload resolution is where the hostile `count x sizeof(type)` product
    // actually gets used, so it must be reached from here or the target
    // fuzzes half the reader.
    for ifd in container.ifds() {
        for entry in ifd.entries() {
            let _ = entry.byte_len();
            let _ = container.payload(entry);
            let _ = container.uints(entry);
        }
    }

    let _ = container.sensor_candidates();

    if let Ok(sensor) = container.sensor() {
        let _ = sensor.packed_bits();
        let _ = sensor.require_uncompressed();
    }
});
