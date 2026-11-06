//! Fuzz target for the single-component SOF-3 lossless-JPEG decoder
//! (`SPEC-038`; the decoder is `SPEC-036`'s entropy front-end + `SPEC-037`'s
//! reconstruction, `src/ljpeg.rs`).
//!
//! A DNG `Compression 7` sensor plane is a lossless-JPEG (SOF-3, ITU-T T.81
//! Annex H) bitstream — attacker-influenced binary, the highest-risk surface
//! for `no-panics-on-untrusted-input`. The contract is the same one
//! `fuzz_targets/plane.rs` and `ifd.rs` state: **for every byte string the
//! entry below returns, and it never panics.** Every malformed / truncated /
//! oversized-table stream must come back as a typed `irradiance::Error`.
//!
//! The decoder is `pub(crate)`; this target reaches it through the
//! `#[doc(hidden)]` `irradiance::__fuzz_ljpeg`, compiled in only by the
//! `fuzzing` cargo feature this crate enables on its `irradiance` dependency
//! (`DEC-036` — a feature, NOT `cfg(fuzzing)`, because the `clippy — fuzz
//! graph` gate builds this manifest with plain `cargo clippy`, no `--cfg
//! fuzzing`). `__fuzz_ljpeg` bounds the plane dimensions before allocating, so
//! a forged `65535 x 65535` frame declines the OOM rather than chasing it.
//!
//! ⚠ Needs the same `+toolchain` handling as `plane`/`ifd` — see that file's
//! header and `guidance/toolchain-brief.md`.
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ljpeg \
//!     fuzz/corpus/ljpeg fuzz/seeds/ljpeg -- -max_total_time=60
//! ```
//!
//! Seeds are own-work SOF-3 bitstreams written by `cargo run --example
//! fuzz-seeds` (`examples/fuzz-seeds.rs`) — valid scans across predictors,
//! precision, point transform and restart, plus the hostile shapes (truncated
//! header, truncated scan, oversized DHT, `Nf != 1`, bad marker). No corpus
//! file is used as a seed (`DEC-003`).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    irradiance::__fuzz_ljpeg(data);
});
