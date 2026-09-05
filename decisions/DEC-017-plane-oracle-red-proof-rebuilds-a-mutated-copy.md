---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-017
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-04
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - tests/plane_oracle.rs
  - tests/support/md5.rs

tags:
  - testing
  - oracle
  - red-proof
  - decode
---

# DEC-017: the plane oracle's red-proof rebuilds a mutated copy of the crate

## Decision

`tests/plane_oracle.rs`'s red-proof (`an_injected_unpacker_fault_turns_the_oracle_red`,
`the_honest_tree_is_the_negative_control`) proves a fault in `unpack_into`
turns the oracle red by **copying the crate to a temp dir, textually injecting
one fault into the copy's `src/plane.rs`, `cargo build --release`-ing a small
synthesized probe binary in that copy, and running it against a real corpus
file** — never by mutating anything in-process. The working tree's
`src/plane.rs` is never touched. The injected fault is: in `BitReader::read`,
swap `.checked_div(pow2(bits_left.saturating_sub(take)))` (keep the TOP `take`
bits of the remaining byte window) for `.checked_rem(pow2(take))` (keep the
BOTTOM `take` bits instead) — same total bits consumed, wrong value whenever a
read does not consume a byte down to zero.

## Context

`SPEC-013`'s design probe needed a red-proof for the plane oracle and found
that `tests/metadata_oracle.rs`'s existing red-proof pattern — perturb an
in-memory struct field, no rebuild needed — does not apply here. That pattern
works because the metadata oracle compares two already-parsed `Sensor`
values; the plane oracle's whole subject is `unpack_into`'s **compiled
behaviour** on a byte stream, and Rust has no way to swap a function's
behaviour at runtime. Proving a fault in it for real requires a real, separate
compilation — the same root cause `scripts/lint-red-proof.sh` (`DEC-006`,
`DEC-009`) exists for, one level down: that script proves clippy's lint
policy bites by mutating a temp-dir copy of `src/lib.rs` and re-running
clippy; this decision does the analogous thing for a *decode* fault instead
of a *lint* fault.

Two things had to be measured, not assumed, before this mechanism could be
trusted:

1. **A copy-and-rebuild-per-test-run is fast enough not to time out.**
   Measured on this machine: `cargo build --release` for this zero-`[dependencies]`
   crate takes ~1.8 s from a clean `target/`; decoding a real 86 MB, 47-megapixel
   Q2M frame via the release binary takes ~0.3 s. The full red-proof test
   (stage a temp copy, inject, build, run, compare) completed in ~18 s;
   the negative control in ~3 s — both dominated by the build, not the decode.
   **Release, not debug, is load-bearing**: the design probe's design-time
   attempts at a genuine faulty digest were killed by session timeouts on the
   same 95 MB plane, almost certainly because `cargo test`'s default debug
   profile makes the per-sample bit-reader loop over 47 million pixels an
   order of magnitude slower.
2. **The first candidate fault was measured and rejected before being used.**
   Starting `BitReader`'s cursor at `bit_in_byte: 1` instead of `0` (a
   plausible "off-by-one" analogous to the design probe's own `.max(1)`) was
   built and run through this exact apparatus. It did not produce a wrong
   digest — it produced `Error::Truncated` at the very last read, because the
   strip is packed with **zero slack** (`width * height * bits ==
   StripByteCounts * 8`, exactly), and any CONSTANT additive shift to the
   total bit budget consumes one bit more than the buffer holds by the final
   sample, for any file this invariant holds on. That is a real fault too
   (an error is not a false green), but it does not exercise the "assert the
   digest differs" clause this spec exists to prove, and it does not
   demonstrate the class of bug the spec is actually worried about: one that
   decodes to completion and returns plausible-looking, silently wrong pixels.
   The chunk-extraction swap this decision selects preserves the total bit
   budget exactly (same `take` per read, same cursor advancement), so it
   cannot hit that boundary, and it was verified (not assumed) to produce a
   different digest: `cb653b5bec24d166eef2fd258ee61ac4` (honest) vs.
   `59b032fe4320a27989ce61f3e3da7ff2` (mutant), both on `L1021223.DNG`.

## Alternatives Considered

- **Option A: perturb an in-memory value, like `tests/metadata_oracle.rs`'s
  red-proof.**
  - What it is: build a `Sensor`/plane value, flip one field or sample, diff.
  - Why rejected: this oracle's subject is `unpack_into`'s *compiled behaviour*
    on raw bytes, not a value that already exists in memory to be perturbed.
    Flipping a sample in the OUTPUT plane after decoding tests the comparison
    logic, not the decoder — it cannot distinguish "the decoder is correct"
    from "the decoder is broken but this test doesn't reach the bug".

- **Option B: a `cfg`-gated fault compiled into the real binary, toggled by an
  environment variable or feature flag.**
  - What it is: `if std::env::var("INJECT_FAULT").is_ok() { /* wrong path */ }`
    inside `unpack_into` itself, or an off-by-default Cargo feature.
  - Why rejected: this puts fault-injection code in `src/plane.rs` permanently
    — a real change to the shipped library for the sole benefit of one test,
    and a live landmine if the flag or feature is ever mis-set outside tests.
    The spec's Non-Goals rule out any `src/` change; this option would violate
    that even if the flag defaulted off.

- **Option C (chosen): copy the crate, mutate the copy's source textually,
  rebuild, run.**
  - What it is: `stage_probe_crate`/`inject_chunk_extraction_fault`/
    `build_and_run_probe` in `tests/plane_oracle.rs`.
  - Why selected: the real `unpack_into`, actually miscompiled the intended
    way, run against a real file — no permanent change to the shipped crate,
    and the same mechanism `scripts/lint-red-proof.sh` already established as
    trustworthy in this repo. The cost (a `cargo build` per red-proof test
    run) was measured acceptable (~18 s and ~3 s respectively) rather than
    assumed so, given the design probe's prior timeout.

## Consequences

- **Positive.** The red-proof exercises the actual compiled decoder, not a
  proxy for it, with no permanent `src/` change and no fault-injection
  surface left in the shipped library.
- **Positive.** The rejected first attempt (`bit_in_byte: 1`) is recorded
  in the module doc comment and here, so a future spec touching this
  red-proof does not re-discover "a constant bit-shift on a zero-slack strip
  truncates instead of corrupting" by losing a loop to it, the way `.max(1)`'s
  no-op nearly went unnoticed at design.
- **Negative.** Each red-proof test run pays a `cargo build --release` (~1-2 s
  measured) it would not need if the fault were toggled by a flag. Judged
  acceptable given the alternative (Option B) permanently changes the library.
- **Neutral.** The probe binary embeds `tests/support/md5.rs` verbatim via
  `include_str!` rather than being copied as a file, so there is exactly one
  MD5 implementation to keep correct, not two.

## Validation

Right if the red-proof continues to catch a real decode fault (asserted every
run: `mutant_digest != honest_digest`) without ever reporting red on the
honest, unmutated tree (the negative control, asserted every run:
`apparatus_digest == pinned_raw_checksum`). Revisit if a future spec needs a
SECOND injected fault for a different code path (`unpack_byte_aligned`, say)
and the same "measure it actually changes the digest, do not assume" discipline
should extend rather than duplicate this decision's mechanism.

## References

- Related specs: SPEC-012, SPEC-013
- Related decisions: DEC-006, DEC-007, DEC-008, DEC-009, DEC-010, DEC-016
- External docs: RFC 1321; `scripts/lint-red-proof.sh`; `docs/oracle-contract.md`
