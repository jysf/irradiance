//! Fuzz target for the `OpcodeList` byte-stream parser, `SPEC-017`'s own
//! side of the shared `src/opcode.rs` module (mirrors `SPEC-018`'s
//! `warp_opcode` target, `fuzz_targets/warp_opcode.rs`).
//!
//! Shipped in the SAME change as this spec's parser branch — AGENTS.md §12
//! bar 2. `OpcodeList1`'s bytes come straight from the IFD
//! (`Sensor::opcode_list_1`), attacker-influenced exactly like every other
//! tag payload. The contract is the same one `fuzz_targets/ifd.rs` states:
//! for every byte string, every entry point below returns, and none of them
//! panics. Any answer is acceptable. An abort is not.
//!
//! `parse_opcode_list` is the same function `warp_opcode` already fuzzes —
//! this target exists as its own named `just fuzz-opcode` recipe with its
//! own seed corpus (the real `OpcodeList1` bytes plus `FixBadPixelsConstant`-
//! shaped adversarial variants) per `SPEC-017`'s handoff, not because the
//! entry point differs.
//!
//! ⚠ Needs the same `+toolchain` handling as `ifd`/`plane`/`develop`/
//! `warp_opcode` — see those files' headers.
//!
//! ```text
//! PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run opcode \
//!     fuzz/corpus/opcode fuzz/seeds/opcode -- -max_total_time=60
//! ```
//!
//! Seeds are hand-built tier-A fixtures written by `cargo run --example
//! fuzz-seeds` (`tests/support/opcode.rs::fix_bad_pixels_fuzz_seed_corpus`,
//! shared with `opcode_parser_fuzz_seeds_do_not_panic` so the committed
//! corpus and that smoke test can never drift apart) — the real 28-byte
//! `OpcodeList1` payload (byte-identical across all three decodable Q2M
//! frames) plus three hand-truncated/adversarial variants.

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::opcode::{parse_fix_bad_pixels_constant, parse_opcode_list};

fuzz_target!(|data: &[u8]| {
    // A rejected input is a SUCCESS for this target: the parser said no
    // with a typed error instead of panicking.
    let _ = parse_opcode_list(data);
    let _ = parse_fix_bad_pixels_constant(data);
});
