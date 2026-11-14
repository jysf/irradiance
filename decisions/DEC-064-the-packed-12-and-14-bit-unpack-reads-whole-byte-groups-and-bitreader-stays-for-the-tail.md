---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-064
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-14
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/plane.rs
  - tests/plane_oracle.rs
  - examples/unpack-timing.rs
  - app.just
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-069-faster-single-threaded-packed-bits-unpack-profile-gated-the.md

tags:
  - plane
  - unpack
  - performance
  - stage-020
---

# DEC-064: the packed 12- and 14-bit unpack reads whole byte groups, and `BitReader` stays for the tail

## Decision

### 1. Byte groups first, `BitReader` for what is left

`unpack_bitstream` is the MSB-first path for 12- and 14-bit samples (DEC-008). It
decodes in two parts.

1. **Whole groups.** A group is the smallest run of whole bytes that holds whole
   samples:
   - **14-bit:** 7 bytes hold 4 samples;
   - **12-bit:** 3 bytes hold 2 samples.

   Each group is read as one big-endian integer (`u64` from 7 bytes, `u32` from 3), and
   each sample is a fixed shift and mask of it:
   - 14-bit: shifts 42, 28, 14, 0, mask `0x3FFF`;
   - 12-bit: shifts 12, 0, mask `0x0FFF`.

   The group count is `min(dst.len() / samples_per_group, bytes.len() / group_bytes)`.
2. **The tail.** The remaining samples (at most 3 for 14-bit, at most 1 for 12-bit) are
   read by today's `BitReader`, **positioned at the first byte after the last group in
   the same slice**. Any short-input `Error::Truncated` therefore reports the same `at`
   it reports today, and a failing read writes the same samples before it fails.

Why this is valid everywhere `unpack_packed` is called:
- **Every call starts on a byte boundary of one continuous bitstream.** That holds for a
  whole strip (SPEC-028), a band inside a strip whose rows are whole bytes (DEC-063 §1),
  and a tile row (`TileWidth · bits` is whole bytes, `check_tile_rows`).
- **Groups do not care where rows end.** The stream runs across row boundaries, so a
  group may span two rows. It needs no whole-byte-row precondition.
- **Byte order plays no part** (DEC-008: the packed stream has none).

The output is byte-identical by construction: each sample is the same bits of the same
stream.

### 2. Shape constraints

- **No `unsafe`, no allocation, no new dependency, no SIMD intrinsics.** Groups come from
  `as_chunks::<7>()` / `as_chunks::<3>()` and outputs go to `as_chunks_mut::<4>()` /
  `::<2>()`, so nothing is indexed.
- **Shifts and casts follow the crate lints.** That means `checked_shr(..).unwrap_or(0)`
  and `u16::try_from(..)`, not `>>` and `as`. The probe measured this lint-clean form at
  the same speed as the plain form (9.7 against 9.6 ms).
- **8- and 16-bit are untouched.** `unpack_byte_aligned` stays, and so do the byte-order
  needles that `tests/plane_oracle.rs` scans.
- **`BitReader` stays, unchanged.** It reads the tail, its unit tests stay, and it is the
  definition the groups must agree with.
- **One permitted test edit, and only one:** in `tests/plane_oracle.rs`,
  `inject_chunk_extraction_fault`'s `needle`, its replacement text, and its doc comment.
  - **Why.** Today the injector corrupts `BitReader::read`. After this change, every
    held 14-bit file and the hand-built fixture (4 × 2 samples, two whole groups) decode
    entirely through groups. The old injection would change nothing, and the two
    red-proofs that assert `mutant != honest` would fail.
  - **The new fault** must change **values, never the number of bits consumed**, for the
    reason the file's module doc gives. Swap the first two 14-bit shifts (42 and 28):
    samples 0 and 1 of every group trade places.
  - **Every assertion in that file stays as it is.** That includes
    `assert_ne!(mutant_digest, honest_digest)`.

### 3. How the speed is measured

This follows SPEC-067's practice:
- a committed release example, `examples/unpack-timing.rs`, run by `just unpack-timing`
  (`app.just`);
- it reads `LEICA-Q2-MONO/L1021223.DNG`, does one warm-up, and reports the median and
  minimum of 7 runs;
- it prints a SKIP line and exits 0 when the file is absent;
- there is no CI timing gate.

It times:
- `unpack_into` on one thread;
- the 8-band, 8-thread unit assembly (`plane::units` with `max_band_rows` 704 and
  `std::thread::scope`), so SPEC-068's parallel figure is re-measured on the new kernel.

**The target is absolute, on a fixed baseline** (the SPEC-067 lesson, not a ratio to a
case the change might also move). On the build machine, the single-thread Q2M
`unpack_into` median must be **≤ 30 ms**, against **238–243 ms at `579aeb2`**. It is a
target, not a gate. Byte identity is the hard criterion.

## Context

monorail-000 / onyx-000 triage item 1b asked for a faster single-threaded open. STAGE-020
framed this spec as profile-gated: measure first, and change only what the profile names.

The SPEC-069 design probe ran on main at `579aeb2` (Apple M4 Pro, release build, a
throwaway example, median of 7):

| Kernel (Q2M 8424×5632, 14-bit, one strip) | L1021223 | L1026016 | L1026192 |
|---|---|---|---|
| crate `unpack_into` today (`BitReader`, per sample) | 238.0 ms | 234.3 ms | 236.3 ms |
| `u64` accumulator, refilled per byte | 45.0 | 44.8 | 45.3 |
| **byte groups (7 bytes → 4 samples), tail by accumulator** | **9.4** | **9.5** | **9.5** |
| byte groups, lint-clean form (`checked_shr`, `try_from`) | 9.8 | 9.7 | 9.7 |
| `memcpy` of the strip (the memory floor) | 1.4 | 1.3 | 1.4 |

- **Every candidate was byte-identical to `unpack_into` on all three files.** Both
  candidates also equalled an independent bit-at-a-time reader on random bytes, for 12-
  and 14-bit and every sample count from 0 to 299 (every tail residue).
- **The profile names the bit loop.** macOS `sample` over a 40-iteration `unpack_into`
  loop put **3,350 of 3,350** main-thread samples in `plane::unpack_packed` (with
  `BitReader::read` inlined). Nothing else in `unpack_into` registers.
- **Byte groups are 25× faster than today and 4.7× faster than the accumulator.** They
  are about 7× the `memcpy` floor, so a hand-vectorised kernel has little left to win.
- **No held uncompressed file is 12-bit.** The 12-bit group is proven by tier-A fixtures
  against an independent reader only. The Typ 246 is 12-bit but Compression 7
  (SPEC-070).

## Alternatives Considered

- **The `u64` accumulator for every width.** Rejected. It is simpler, but 4.7× slower
  than groups on the requester's file, for no gain in coverage.
- **Replace `BitReader` entirely.** Rejected. It is proven against dnglab (SPEC-013) and
  covered by unit tests, and it handles the tail with no new logic. Keeping it also keeps
  the tail's errors identical.
- **SIMD intrinsics or `unsafe` loads.** Rejected. The scalar group kernel is about 7×
  the memory floor, and the crate forbids `unsafe`.
- **A group path only for planes whose rows are whole bytes.** Rejected as unnecessary:
  the stream is continuous across rows, so groups apply to any width.
- **Leave the red-proof injector alone and inject into the group kernel from a new
  test.** Rejected. The existing two red-proofs would fail (`mutant == honest`), and they
  are the file's purpose. Retargeting the one needle keeps their assertions and their
  meaning: a value fault in the live 14-bit path changes the dnglab-pinned digest.

## Consequences

- The Q2M's single-threaded unpack should fall from about 238 ms to about 10 ms. The
  measured result is recorded at ship.
- The 8-thread unit assembly is expected to approach the memory floor; it is measured,
  not targeted.
- **There is no public API change**, and output and errors are unchanged.
- `tests/plane_oracle.rs`'s `inject_chunk_extraction_fault` now names the group kernel.
  A future change to the 14-bit group must keep exactly one occurrence of its needle, or
  update the injector.
