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
//! `SPEC-099` (`DEC-087` item 1): every input also runs the per-plane parse
//! through `irradiance::__fuzz_warp_planes` for no CFA and the colour
//! assignments `[0, 1, 2]` and `[2, 1, 0]`, asserting its invariants: `Ok(Some)`
//! means `N` is 1 or 3 and `K1` agrees with `parse_warp_rectilinear`; the
//! consumer is the set coloured 1 for three coloured sets and `K1` otherwise.
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
use irradiance::Error;

fuzz_target!(|data: &[u8]| {
    // A rejected input is a SUCCESS for this target: the parser said no
    // with a typed error instead of panicking.
    let _ = parse_opcode_list(data);
    let k1 = parse_warp_rectilinear(data);

    // SPEC-099 (DEC-087 item 1): the per-plane parse, for no CFA and two
    // colour assignments. Its invariants are asserted, so a violation is a
    // crash.
    for plane_color in [None, Some(&[0u8, 1, 2][..]), Some(&[2u8, 1, 0][..])] {
        match irradiance::__fuzz_warp_planes(data, plane_color) {
            Ok(Some(warp)) => {
                let sets = warp.sets();
                let k1 = match &k1 {
                    Ok(Some(k1)) => *k1,
                    other => panic!("per-plane Ok(Some) but K1 parse {other:?}"),
                };
                assert!(sets.len() == 1 || sets.len() == 3, "N is 1 or 3");
                assert_eq!(sets[0].warp, k1, "the first set is K1");
                if let (3, Some(colours)) = (sets.len(), plane_color) {
                    let green = colours.iter().position(|&c| c == 1).unwrap();
                    assert_eq!(*warp.consumer(), sets[green].warp, "consumer is green's");
                    for (set, &c) in sets.iter().zip(colours) {
                        assert_eq!(set.color, Some(c), "colour from CFAPlaneColor");
                    }
                } else {
                    assert_eq!(*warp.consumer(), k1, "consumer is K1");
                    assert!(
                        !warp.is_per_plane(),
                        "per plane only with three coloured sets"
                    );
                }
            }
            Ok(None) => assert!(
                !matches!(k1, Ok(Some(_))),
                "per-plane None but K1 parse found a warp"
            ),
            Err(Error::MalformedOpcodeParams { id: 1, .. }) => {
                // N other than 1 or 3, or a malformed warp the K1 parse also
                // refuses, or a non-finite later set.
                if let Ok(Some(k1)) = &k1 {
                    assert!(
                        k1.planes != 1,
                        "N = 1 accepted by the K1 parse is never malformed here"
                    );
                }
            }
            Err(e) => assert!(
                k1.is_err(),
                "per-plane error {e:?} on a list the K1 parse accepted"
            ),
        }
    }
});
