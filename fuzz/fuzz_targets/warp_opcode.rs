//! Fuzz target for the `OpcodeList` byte-stream parser (`SPEC-018`).
//!
//! Shipped in the SAME change as `src/opcode.rs` — AGENTS.md §12 bar 2. This
//! is a NEW input surface: `OpcodeList3`'s bytes come straight from the IFD
//! (`Sensor::opcode_list_3`), attacker-influenced exactly like every other
//! tag payload. The contract is the same one `fuzz_targets/ifd.rs` states:
//! for every byte string, every entry point below returns, and none of them
//! panics. Any answer is acceptable. An abort is not.
//!
//! Fuzzes the byte-stream parser DIRECTLY (not the whole container ->
//! sensor -> develop chain `fuzz_targets/develop.rs` already covers) —
//! `parse_opcode_list` and `parse_warp_rectilinear`'s only input is a byte
//! slice, so there is no earlier stage to route through.
//!
//! ⚠ Needs the same `+toolchain` handling as `ifd`/`plane`/`develop` — see
//! those files' headers.
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run warp_opcode \
//!     fuzz/corpus/warp_opcode fuzz/seeds/warp_opcode -- -max_total_time=60
//! ```
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`tests/opcode.rs::fuzz_seed_corpus`, shared with
//! `warp_opcode_fuzz_smoke_no_crashes_after_60s` so the committed corpus and
//! that smoke test can never drift apart) — real `OpcodeList3` bytes from
//! all three decodable Q2M frames, plus three hand-truncated/adversarial
//! variants.

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::opcode::{parse_opcode_list, parse_warp_rectilinear};

fuzz_target!(|data: &[u8]| {
    // A rejected input is a SUCCESS for this target: the parser said no
    // with a typed error instead of panicking.
    let _ = parse_opcode_list(data);
    let _ = parse_warp_rectilinear(data);
});
