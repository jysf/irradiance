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
//!
//! `SPEC-054` extends it over the required-processing check (`DEC-049`): the
//! same bytes go through `__fuzz_required_processing` (the `fuzzing`-feature
//! hook onto the `pub(crate)` check) in every list slot, with two cross-path
//! invariants asserted in `check_required` below.
//!
//! `SPEC-065` adds a differential (`DEC-060` §2): the allocation-free
//! walkers `parse_warp_rectilinear` and `parse_fix_bad_pixels_constant` must
//! return exactly what `parse_opcode_list` followed by `find_map` returns,
//! errors included (compared by `Debug`).

#![no_main]

use libfuzzer_sys::fuzz_target;

use irradiance::opcode::{
    parse_fix_bad_pixels_constant, parse_opcode_list, parse_warp_rectilinear, Opcode,
};
use irradiance::{__fuzz_required_processing, Error};

/// `SPEC-054` / `DEC-049`: the required-processing check every public develop
/// entry runs, over the same hostile bytes, in each list slot and in all three
/// at once. Two invariants beyond "never panics":
///
/// 1. The colour path applies nothing and the mono path applies a superset,
///    so a list set the colour check accepts, the mono check must accept.
/// 2. The check and `parse_opcode_list` share one framing reader, so a list
///    the colour check accepts (every opcode optional, framing intact) must
///    not be one the parser rejects as truncated or as a mandatory unknown.
fn check_required(
    lists: [Option<&[u8]>; 3],
    parsed: &Result<Vec<irradiance::opcode::Opcode>, Error>,
) {
    let [mono_ok, colour_ok] = __fuzz_required_processing(lists);
    assert!(
        !colour_ok || mono_ok,
        "colour accepted a list set the mono path rejected: {lists:?}"
    );
    if colour_ok && lists.iter().any(Option::is_some) {
        assert!(
            !matches!(
                parsed,
                Err(Error::Truncated { .. } | Error::UnsupportedMandatoryOpcode { .. })
            ),
            "the check accepted framing the parser rejects: {parsed:?}"
        );
    }
}

fuzz_target!(|data: &[u8]| {
    // A rejected input is a SUCCESS for this target: the parser said no
    // with a typed error instead of panicking.
    let parsed = parse_opcode_list(data);

    // SPEC-065: the walkers against the typed parse plus `find_map`.
    let reference_fbp = parsed.as_ref().map(|opcodes| {
        opcodes.iter().find_map(|op| match op {
            Opcode::FixBadPixelsConstant { constant, .. } => Some(*constant),
            _ => None,
        })
    });
    assert_eq!(
        format!("{:?}", parse_fix_bad_pixels_constant(data)),
        format!("{reference_fbp:?}"),
        "parse_fix_bad_pixels_constant differs from parse_opcode_list + find_map"
    );
    let reference_warp = parsed.as_ref().map(|opcodes| {
        opcodes.iter().find_map(|op| match op {
            Opcode::WarpRectilinear {
                kr,
                kt,
                cx,
                cy,
                planes,
            } => Some((*kr, *kt, *cx, *cy, *planes)),
            _ => None,
        })
    });
    let walked_warp =
        parse_warp_rectilinear(data).map(|found| found.map(|w| (w.kr, w.kt, w.cx, w.cy, w.planes)));
    assert_eq!(
        format!("{walked_warp:?}"),
        format!("{reference_warp:?}"),
        "parse_warp_rectilinear differs from parse_opcode_list + find_map"
    );

    for slot in 0..3 {
        let mut lists: [Option<&[u8]>; 3] = [None; 3];
        lists[slot] = Some(data);
        check_required(lists, &parsed);
    }
    check_required([Some(data); 3], &parsed);
});
