//! `SPEC-018` — the `OpcodeList` byte-stream parser (`AC1`, `AC2`, `AC3`).
//! `SPEC-017` adds its own `AC1`-`AC3` below (`FixBadPixelsConstant`,
//! `OpcodeID` 4) — same module, same split rationale, so its parser-only
//! tests land in this file too rather than a second one; the applier's
//! algorithm (`AC4`-`AC9`) lives in `tests/develop.rs`, where
//! `apply_fix_bad_pixels_constant` itself does (state-which choice, `##
//! Outputs`).
//!
//! Byte-level parsing only. `WarpRectilinear`'s geometric APPLICATION
//! (`AC4`-`AC12`) is `tests/warp.rs` — see that file's own header for why the
//! split (`## Failing Tests` states the choice: opcode parsing here, warp
//! application there).
//!
//! Every test here is tier A: real bytes committed as hex fixtures
//! (`tests/oracle-fixtures/opcodelist3-*.hex`, `opcodelist1-q2m.hex`,
//! extracted with `exiftool -b -OpcodeList1`/`-OpcodeList3` — a design-time
//! probe, not a re-derivation, and independently re-verified against all
//! three decodable Q2M frames during this build) plus hand-built adversarial
//! streams. No corpus, no tools, run everywhere including CI.

#[path = "support/opcode.rs"]
mod support;

use irradiance::opcode::{
    parse_fix_bad_pixels_constant, parse_opcode_list, parse_warp_rectilinear, Opcode,
};
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

// ─────────────────────────────────────────────────────────────────────────────
// SPEC-017 — FixBadPixelsConstant (OpcodeID 4), byte-level parsing only.
// The applier's algorithm (AC4-AC9) lives in tests/develop.rs.
// ─────────────────────────────────────────────────────────────────────────────

/// `AC1` — the parser round-trips the probed Q2M `OpcodeList1` bytes.
/// Reproduced independently against all three decodable frames during this
/// build (`exiftool -b -OpcodeList1`, byte-identical on every one) before
/// this test was written — `AGENTS.md` §16 rule 4,
/// `unrun-docs-carry-errors`.
#[test]
fn parse_opcode_list_reads_the_q2m_fixbadpixels_bytes() {
    let bytes = support::load_hex_fixture("opcodelist1-q2m");
    let opcodes = parse_opcode_list(&bytes).unwrap_or_else(|e| panic!("parse: {e}"));
    assert_eq!(
        opcodes,
        vec![Opcode::FixBadPixelsConstant {
            constant: 0,
            bayer_phase: 2,
        }],
        "the probed 28-byte Q2M OpcodeList1 payload must decode to exactly this one opcode"
    );
    assert_eq!(
        parse_fix_bad_pixels_constant(&bytes).unwrap_or_else(|e| panic!("parse: {e}")),
        Some(0),
        "the develop_into entry point must read the same Constant"
    );
}

/// `AC2` — panic-free on adversarial input. Same shape as
/// `warp_opcode_fuzz_smoke_no_crashes_after_60s` above: runs the `opcode`
/// fuzz target's exact seed corpus directly through the parser (the real
/// 60s `cargo fuzz run opcode` / `just fuzz-opcode` is run separately during
/// build/CI, `AGENTS.md` §12 bar 2).
#[test]
fn opcode_parser_fuzz_seeds_do_not_panic() {
    for (name, bytes) in support::fix_bad_pixels_fuzz_seed_corpus() {
        // A panic here fails THIS test with the panicking seed's name in the
        // backtrace; Err or Ok are both a pass (`no-panics-on-untrusted-input`).
        let _ = parse_opcode_list(&bytes);
        let _ = name;
    }
}

/// `AC3` — the parser records `Opcode::Unknown` for unknown IDs and
/// preserves their bytes, round-tripping opcode ID 99 with a 4-byte payload.
///
/// ⚠ **SPEC-017/FU-1** — this spec's own pre-registered example used
/// `flags: 0` (mandatory) and expected `Ok(Opcode::Unknown{..})`. SPEC-018,
/// which built this module FIRST, already dispatches mandatory-vs-optional
/// AT PARSE TIME: an unrecognized ID with `Flags` bit 0 clear is
/// `Error::UnsupportedMandatoryOpcode` (`mandatory_unknown_opcode_is_rejected`,
/// `src/opcode.rs`; `opcode_parser_rejects_mandatory_unknown`, this file) —
/// `Opcode::Unknown` for an unrecognized ID is reachable ONLY with `Flags`
/// bit 0 SET (optional). This test uses `flags: 1` for that reason; `AC6`'s
/// mandatory-unknown-in-`OpcodeList1` case is covered separately by
/// `develop_errors_on_mandatory_unknown_opcode` (`tests/develop.rs`), which
/// exercises `parse_opcode_list`'s existing error through `develop_into`,
/// not a second dispatch layer inside this module.
#[test]
fn opcode_parser_returns_unknown_for_ninetynine() {
    let bytes = support::opcode_list(&[support::raw_opcode(99, 0x0103_0000, 1, &[1, 2, 3, 4])]);
    let opcodes = parse_opcode_list(&bytes)
        .unwrap_or_else(|e| panic!("optional-unknown must not error: {e}"));
    assert_eq!(
        opcodes,
        vec![Opcode::Unknown {
            id: 99,
            flags: 1,
            params: vec![1, 2, 3, 4],
        }]
    );
}
