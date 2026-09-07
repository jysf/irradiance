//! `SPEC-018` — the `OpcodeList` byte-stream parser (`AC1`, `AC2`, `AC3`).
//!
//! Byte-level parsing only. `WarpRectilinear`'s geometric APPLICATION
//! (`AC4`-`AC12`) is `tests/warp.rs` — see that file's own header for why the
//! split (`## Failing Tests` states the choice: opcode parsing here, warp
//! application there).
//!
//! All three tests here are tier A: real bytes committed as hex fixtures
//! (`tests/oracle-fixtures/opcodelist3-*.hex`, extracted with `exiftool -b
//! -OpcodeList3` — a design-time probe, not a re-derivation) plus hand-built
//! adversarial streams. No corpus, no tools, run everywhere including CI.

#[path = "support/opcode.rs"]
mod support;

use irradiance::opcode::{parse_opcode_list, parse_warp_rectilinear, Opcode};
use irradiance::Error;

/// SPEC-020/FU-1's per-frame coefficient table (measured independently by
/// this build via `exiftool -b -OpcodeList3` on all three decodable Q2M
/// frames — `AGENTS.md` §16 rule 4, `unrun-docs-carry-errors`: read each
/// frame's own bytes, never carry L1021223's set forward as a camera
/// constant). Only `kr0` is shared; `kr1` varies ~1.9x.
const FRAMES: [(&str, [f64; 4]); 3] = [
    (
        "opcodelist3-L1021223",
        [
            0.999_251_106,
            -0.061_376_512_877_263_58,
            -0.093_915_541_393_360_16,
            0.055_882_009_215_291_75,
        ],
    ),
    (
        "opcodelist3-L1026016",
        [
            0.999_251_106,
            -0.041_845_169_276_595_75,
            -0.102_311_478_808_510_63,
            0.057_876_719_170_212_77,
        ],
    ),
    (
        "opcodelist3-L1026192",
        [
            0.999_251_106,
            -0.032_331_453_292_576_424,
            -0.104_204_110_174_672_49,
            0.056_364_293_537_117_91,
        ],
    ),
];

/// `AC1` — the parser reads each decodable Q2M frame's real `OpcodeList3`
/// bytes, on ALL THREE frames (`measurement-over-generalised`: one frame
/// passing is one measurement, not a boundary), and the parsed `WarpRect`
/// round-trips byte-for-byte back through the same encoder that built the
/// hand-built fixtures elsewhere in this test suite.
#[test]
fn opcode_list_3_round_trips_the_q2m_warp() {
    for (fixture, expected_kr) in FRAMES {
        let bytes = support::load_hex_fixture(fixture);

        let warp = parse_warp_rectilinear(&bytes)
            .unwrap_or_else(|e| panic!("{fixture}: parse: {e}"))
            .unwrap_or_else(|| panic!("{fixture}: expected one WarpRectilinear opcode"));

        assert_eq!(warp.kr, expected_kr, "{fixture}: kr0..kr3");
        assert_eq!(
            warp.kt,
            [0.0, 0.0],
            "{fixture}: kt0, kt1 (no tangential term)"
        );
        assert_eq!((warp.cx, warp.cy), (0.5, 0.5), "{fixture}: optical center");
        assert_eq!(warp.planes, 1, "{fixture}: N (coefficient-set count)");

        // Byte-for-byte round-trip: re-encode the PARSED fields with the same
        // support encoder AC3's hand-built streams use, and compare against
        // the real file's own bytes.
        let reencoded = support::single_warp_rectilinear_list(
            warp.kr,
            warp.kt,
            warp.cx,
            warp.cy,
            warp.planes,
            0,
        );
        assert_eq!(
            reencoded, bytes,
            "{fixture}: parsed WarpRect must re-encode to the identical bytes read from the file"
        );
    }
}

/// `AC2` — panic-free on adversarial input. The **simpler** of the two
/// options `## Failing Tests` offers (state the choice, per that section):
/// this runs the exact seed corpus `fuzz/seeds/warp_opcode/` commits
/// directly through the parser and asserts none of them panics. The REAL
/// 60-second `cargo fuzz run warp_opcode` (`just fuzz-warp`) is run
/// separately during build/CI (`AGENTS.md` §12 bar 2) — this test is the
/// fast, no-toolchain-trap sanity check that runs on every `cargo test`.
#[test]
fn warp_opcode_fuzz_smoke_no_crashes_after_60s() {
    for (name, bytes) in support::fuzz_seed_corpus() {
        // A panic here fails THIS test with the panicking seed's name in the
        // backtrace; a rejected (`Err`) or accepted (`Ok`) parse are both a
        // pass — any answer is acceptable, an abort is not
        // (`no-panics-on-untrusted-input`).
        let _ = parse_opcode_list(&bytes);
        let _ = name;
    }
}

/// `AC3` — skip optional-and-unknown, reject mandatory-and-unknown
/// (DNG 1.7.0.0 Chapter 7's `Flags` bit 0).
#[test]
fn opcode_parser_skips_optional_unknown() {
    let bytes = support::opcode_list(&[support::raw_opcode(
        0xDEAD_BEEF,
        0x0104_0000,
        1, // bit 0 set: optional
        &[1, 2, 3, 4],
    )]);
    let opcodes = parse_opcode_list(&bytes)
        .unwrap_or_else(|e| panic!("optional-unknown must not error: {e}"));
    assert_eq!(
        opcodes,
        vec![Opcode::Unknown {
            id: 0xDEAD_BEEF,
            flags: 1,
            params: vec![1, 2, 3, 4],
        }]
    );
}

#[test]
fn opcode_parser_rejects_mandatory_unknown() {
    let bytes = support::opcode_list(&[support::raw_opcode(
        0xDEAD_BEEF,
        0x0104_0000,
        0, // bit 0 clear: mandatory
        &[1, 2, 3, 4],
    )]);
    let err = parse_opcode_list(&bytes).expect_err("mandatory-unknown must error");
    assert!(
        matches!(err, Error::UnsupportedMandatoryOpcode { id: 0xDEAD_BEEF }),
        "{err:?}"
    );
}
