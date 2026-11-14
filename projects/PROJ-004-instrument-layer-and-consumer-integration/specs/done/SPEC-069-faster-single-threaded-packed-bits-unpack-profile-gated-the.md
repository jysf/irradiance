---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-069
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-020
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-064, DEC-063, DEC-008, DEC-017, DEC-016, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-068, SPEC-013, SPEC-067]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-068]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-020's <capability>". Optional; null is acceptable.
value_link: "a ~25x faster single-threaded 12/14-bit packed unpack, so STAGE-020's plane decode is fast on one core, not only on eight"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 20000000
  sessions:
    - cycle: build
      handoff: HANDOFF-145
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 10989685
      estimated_usd: 8.21
      duration_minutes: 60
      recorded_at: 2026-11-14
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (65 unique message.ids, all claude-opus-5-5, no subagents): input 133 + cache-creation 166,643 (1-hour) + cache-read 10,776,918 + output 45,991 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). The handback floor was 9,728,429 and $7.35."
    - cycle: verify
      handoff: HANDOFF-146
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 6884347
      estimated_usd: 5.29
      duration_minutes: 55
      recorded_at: 2026-11-14
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (49 unique message.ids, all claude-opus-5-5, no subagents): input 104 + cache-creation 113,425 (1-hour) + cache-read 6,739,266 + output 31,552, same rates. APPROVED at 90c4bc4 (code 7b15289), no code changes. The handback floor was 4,912,796 and $3.81."
  totals:
    tokens_total: 17874032
    estimated_usd: 13.50
    session_count: 2
shipped_at: 2026-11-14
---

# SPEC-069: Faster packed 12/14-bit unpack: whole byte groups, `BitReader` for the tail

## Context

This is the second spec of STAGE-020, and it answers monorail-000 / onyx-000 triage item
1b, a faster single-threaded open.

The requester's Q2M 14-bit uncompressed plane (47 MP, one strip) unpacks in about
**238 ms** on an M4 Pro. A 16-bit plane of similar size unpacks in about 1 ms. With
SPEC-068's units, the same strip takes 31 ms on 8 threads. That still spends 8 cores on a
cost that is almost entirely per-sample bit-twiddling.

The stage made this spec profile-gated: measure first, and change only what the profile
names.

## Goal

Decode 12- and 14-bit packed samples a whole byte group at a time, with today's
`BitReader` reading only the tail.

- **The hard criterion:** output and errors are byte-identical.
- **The target:** the Q2M single-thread `unpack_into` median is **≤ 30 ms** on the build
  machine, from a fixed **238–243 ms at `579aeb2`**.
- **How it is measured:** a committed release example, `just unpack-timing`.

## Design

### 1. Probe results (main at `579aeb2`, 2026-11-14; recorded so build is transcription)

Apple M4 Pro, release build, a throwaway example, median of 7:

| Kernel (Q2M 8424×5632, 14-bit, one strip) | L1021223 | L1026016 | L1026192 |
|---|---|---|---|
| crate `unpack_into` today | 238.0 ms | 234.3 ms | 236.3 ms |
| `u64` accumulator, refilled per byte | 45.0 | 44.8 | 45.3 |
| **byte groups, tail by accumulator** | **9.4** | **9.5** | **9.5** |
| byte groups, lint-clean form | 9.8 | 9.7 | 9.7 |
| `memcpy` of the strip | 1.4 | 1.3 | 1.4 |

- **Every candidate was byte-identical to `unpack_into` on all three files.** On random
  bytes, both candidates also equalled an independent bit-at-a-time reader for 12- and
  14-bit at every sample count from 0 to 299.
- **Profile:** macOS `sample` over 40 `unpack_into` calls put **3,350 of 3,350**
  main-thread samples in `plane::unpack_packed`, with `BitReader::read` inlined. The
  per-sample bit loop *is* the cost; nothing else in `unpack_into` shows.
- **No held uncompressed file is 12-bit.** The other held uncompressed files are 16-bit,
  which `unpack_byte_aligned` reads and this spec does not touch. The 12-bit group is
  therefore proven by tier A only.

### 2. The kernel (DEC-064 §1)

`unpack_bitstream(strip, bits, dst)` keeps its signature and its single caller,
`unpack_packed`. Inside it:

```text
match bits {
    14 => groups of 7 bytes → 4 samples: v = u64 from [0, b0..b6] big-endian;
          samples = (v >> 42, v >> 28, v >> 14, v) & 0x3FFF
    12 => groups of 3 bytes → 2 samples: v = u32 from [0, b0, b1, b2] big-endian;
          samples = (v >> 12, v) & 0x0FFF
    _  => no groups (unreachable: SUPPORTED_BITS sends 8/16 to unpack_byte_aligned)
}
full = min(dst.len() / per_group, strip.len() / group_bytes)
then BitReader over `strip`, starting at byte full · group_bytes, bit 0,
     reads dst[full · per_group ..] exactly as today
```

- **Groups come from `as_chunks::<N>()`, and outputs go to `as_chunks_mut::<M>()`**,
  zipped, so nothing is indexed. Write the `full` boundary with `split_at_mut_checked`
  (or `get_mut`) and checked arithmetic.
- **Shifts are `checked_shr(n).unwrap_or(0)`, and narrowing is `u16::try_from(..)`.**
  This follows the crate's `arithmetic_side_effects` and cast lints. The probe measured
  the lint-clean form at the same speed.
- **The tail `BitReader` is positioned in the same slice.** Add a private constructor,
  or set `byte_pos` directly; both are in this module. Do not hand it a sub-slice:
  `Error::Truncated { at }` must report the same offset as today.
- **Precondition, true at every call site:** `strip` starts on a byte boundary of one
  continuous MSB-first stream. That covers:
  - a whole strip (SPEC-028);
  - a band at `row · row_bytes` inside a strip whose rows are whole bytes (DEC-063 §1);
  - one row of one tile (`check_tile_rows`).

  Groups may straddle row boundaries, which is correct because the stream is continuous.
- **Unchanged:** `BitReader` itself and its unit tests, `pow2`, `unpack_byte_aligned`,
  every check, `decode_unit`, `decode_tile_row` and the units. No public item changes.

### 3. The red-proof retarget (DEC-064 §2)

`tests/plane_oracle.rs`'s `inject_chunk_extraction_fault` corrupts `BitReader::read`.
After this change:
- every held 14-bit file decodes entirely through groups, since 8424 × 5632 is a multiple
  of 4;
- the hand-built fixture does too (4 × 2 = 8 samples, two groups).

The injection would then be a no-op, and both
`an_injected_unpacker_fault_turns_the_oracle_red` (tier B) and
`an_injected_fault_turns_the_fixture_oracle_red_without_a_corpus` (tier A) would fail on
`assert_ne!`.

**The one permitted test edit:** in that function only, change:
- `needle` to the 14-bit group's first shift expression, which must be unique in
  `src/plane.rs`. Suggested shape: `checked_shr(42)`; the builder may pick any unique
  token sequence in that expression;
- the replacement, to the same expression with **42 and 28 swapped**, so samples 0 and 1
  of every group trade places. That changes values but not the number of bits consumed,
  as the module doc requires;
- the function's doc comment, to describe the new fault.

Every assertion, every other function, and the module doc's history stay unedited. The
module doc's account of why `.max(1)` and the cursor shift were rejected is still true
history.

### 4. Timing (DEC-064 §3)

`examples/unpack-timing.rs` and an `unpack-timing` recipe in `app.just` mirror
`orient-timing`:
- release build, `L1021223` from the corpus root;
- a SKIP line and exit 0 when the file is absent;
- one warm-up, then the median and minimum of 7 runs;
- the machine (`sysctl` model and CPU) printed.

It reports:
- `unpack_into`, single-threaded;
- 8 bands of 704 rows on 8 `std::thread::scope` threads, through `plane::units` and
  `unpack_unit_into`, split with `chunks_mut`, and asserted byte-identical to the
  single-thread plane.

No CI gate. The handback records the before figures (the example run on `579aeb2` plus
only the example file) and the after figures, back to back.

## Inputs

- **Files to read:**
  - `src/plane.rs`: `BitReader`, `pow2`, `unpack_bitstream`, `unpack_byte_aligned`,
    `unpack_packed`, `decode_unit`, `decode_tile_row`, `check_tile_rows`;
  - `tests/plane_oracle.rs` (the module doc, `inject_chunk_extraction_fault` and both
    red-proofs);
  - `tests/plane_unpack.rs` (`Fixture`, `MultiStrip`, `pack_msb`);
  - `tests/plane_units.rs` (the tiled builders);
  - `examples/orient-timing.rs` and `app.just`.
- **Decisions:** DEC-064, DEC-063, DEC-008, DEC-017 (the red-proof mechanism), DEC-016,
  DEC-003.

## Outputs

- **Files modified:**
  - `src/plane.rs`: `unpack_bitstream` and a positioned `BitReader` start; module docs if
    they describe the per-sample loop;
  - `tests/plane_oracle.rs`: `inject_chunk_extraction_fault` only (§3);
  - `app.just`: the `unpack-timing` recipe;
  - `CHANGELOG.md`: `[Unreleased]` Changed, with the before and after numbers.
- **Files created:**
  - `tests/plane_groups.rs`: the tier-A AC2 and AC3 tests;
  - `examples/unpack-timing.rs`.
- **New public items:** none.

## Acceptance Criteria

- [ ] **AC1. Byte-identical on every held file (tier B).** Every existing plane oracle
  stays green **unedited**:
  - the dnglab `raw_checksum` pins for the three Q2M files, the M Monochrom, the Canon and
    the Nikon (`plane_oracle`);
  - `plane_unpack`'s Q2M head values;
  - SPEC-068's any-order assembly on held files;
  - the develop pins downstream (`develop_oracle`, `orientation_tiles`).

  The single exception is §3's injector.
- [ ] **AC2. Byte-identical on every tier-A layout, against an independent packer.**
  In `tests/plane_groups.rs`, samples are drawn from a seeded PRNG over the full range,
  including 0 and `2^bits − 1`. They are packed by the test's own MSB-first packer (a
  copy of `pack_msb`; it does not call the crate) and must unpack to exactly the samples
  that went in:
  - 12- and 14-bit;
  - widths 1–17 and heights 1–5, so every tail residue appears at both row and strip
    ends;
  - one strip, and several strips (`RowsPerStrip` 1 and 2, each strip byte-padded);
  - SPEC-068 bands at `max_band_rows` 1 and 2, decoded through `unpack_unit_into`;
  - tiled planes with clipped right and bottom edges, using tile widths that keep tile
    rows whole bytes (14-bit multiples of 4 and 12-bit multiples of 2, at 4 and 8).
- [ ] **AC3. Errors unchanged.**
  - A unit test in `src/plane.rs` feeds `unpack_bitstream` inputs one byte short of every
    residue, 12- and 14-bit, sample counts 0–17. These cases are unreachable through the
    public API after `check_layout`, which is why it is a unit test.
  - It asserts the **same `Result`** (the same `Error::Truncated { at, len }`) **and the
    same `dst` contents** as a `BitReader`-only decode of the same input. Write that as a
    test-local loop over `BitReader::read`; that is today's code, used here as the
    old-behaviour reference, not as an oracle for values.
  - The existing `hostile_strip_bounds_do_not_panic` and `layer0_arithmetic_is_enforced`
    stay green unedited.
- [ ] **AC4. The red-proofs still prove something.** After §3's retarget, both
  `an_injected_fault_turns_the_fixture_oracle_red_without_a_corpus` (tier A) and
  `an_injected_unpacker_fault_turns_the_oracle_red` (tier B) pass. Their `assert_ne!` is
  unedited. The two control tests are green.
- [ ] **AC5. No allocation, no `unsafe`.**
  - `plane_tiles`' allocation scan of `src/plane.rs` is green unedited.
  - `plane_units_alloc` is green unedited: zero allocations for uncompressed decodes.
  - No `unsafe` and no new dependency.
- [ ] **AC6. The timing target (verify checks it; CI does not).** `just unpack-timing` on
  the build machine reports a single-thread Q2M `unpack_into` median of **≤ 30 ms**,
  against the 238–243 ms baseline at `579aeb2`. The 8-thread figure is recorded, with no
  target.
- [ ] **AC7. No regression.** Every gate is green with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv`
  - `cost-audit`
  - `gate-audit`

  `just fuzz-plane 60` is clean. The existing plane target covers this kernel through
  `unpack_into` and the units.

## Failing Tests

Written first in build; they must be red or fail to compile before the kernel changes.

- **`tests/plane_groups.rs`** (new, tier A):
  - `packed_strips_round_trip_on_every_residue` (AC2): one strip and multiple strips, 12-
    and 14-bit, widths 1–17, heights 1–5.
  - `packed_bands_round_trip_on_every_residue` (AC2): units at `max_band_rows` 1 and 2.
  - `packed_tiles_round_trip_with_clipped_edges` (AC2).

  These pass against today's code too, because they pin behaviour, not speed. The test-
  first evidence for this spec is the tier-A **red-proof list below**, not a red AC2.
- **`src/plane.rs` unit test** `groups_keep_bitreader_errors_and_partial_writes` (AC3).
- **`examples/unpack-timing.rs`** (AC6): built and run before and after.

**Red-proofs.** Apply each by hand, show a **tier-A** test red, and revert:
- the 14-bit shifts 42 and 28 swapped (that is §3's injection) → AC2 and
  `hand_built_fixtures_plane_matches_its_known_md5`;
- the 14-bit mask `0x3FFF` changed to `0x1FFF` → AC2;
- the 12-bit first shift changed from 12 to 11 → AC2 (12-bit only; no held file catches
  it);
- 12-bit dispatched to the 14-bit group → AC2;
- the tail `BitReader` started one byte after the last group → AC2, at every width whose
  sample count is not a multiple of the group;
- the tail handed a sub-slice instead of a position → AC3's `at`;
- the group count `full` reduced by one. **Expected to stay green:** the tail then decodes
  one more group's samples correctly, so only the timing example shows it. Say so; that
  is the benign mutant, like SPEC-067's tile constant of 1.

## Non-Goals

- SIMD intrinsics, `unsafe`, and a new dependency.
- Threads inside irradiance (SPEC-068's units are the caller's tool).
- Compression 7 (SPEC-070).
- The 8- and 16-bit path.
- Any output or error change.
- A CI timing gate.
- A bit-offset band start for rows that are not whole bytes (DEC-063 §4).

## Notes for the Implementer

- **Source scans on `src/plane.rs`.** Of the four tests that scan it, three must stay
  unedited and still match exactly once:
  - `plane_oracle`'s byte-order needles in `unpack_byte_aligned`. These are the full
    strings `u16::from_le_bytes(*chunk)` and `u16::from_be_bytes(*chunk)`, so
    `u64::from_be_bytes([0, b0, …])` and `u32::from_be_bytes(…)` do not collide. Do not
    write either full string anywhere else;
  - `plane_validate`'s guard conditions;
  - `plane_tiles`' allocation tokens.

  The fourth, `plane_oracle`'s chunk-extraction needle, is §3's permitted retarget.
  `ljpeg_shape_binding` scans `check_strip_shape` and is untouched.
- **Tail logic.** `BitReader::read`'s needle string
  (`.checked_div(pow2(bits_left.saturating_sub(take)))`) may stay; after the retarget,
  nothing injects into it. Do not edit `BitReader`.
- **Probe kernel.** The probe's 14-bit group, for reference (plain operators; rewrite it
  lint-clean):
  `let v = u64::from_be_bytes([0, c0, c1, c2, c3, c4, c5, c6]); [(v >> 42) & 0x3fff, (v >> 28) & 0x3fff, (v >> 14) & 0x3fff, v & 0x3fff]`.
- **Run the fuzz target early** (SPEC-068's lesson). Then run the long tier-B suite in the
  background.
- **The tier-B any-order test takes about 9.5 min in debug.** It is a known follow-up
  (signal `slow-tier-b-tests-in-debug`). Do not fix it here.

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Check in the design probe which red-proofs a timing example can actually see. The
   spec assumed the timing run would expose a kernel that decodes one group too few; it
   cannot, because the tail reader writes the same samples. Both cycles found it as a
   benign green, and it stays unguarded by design.

2. **Does any template, constraint, or decision need updating?**
   — No. DEC-064's single permitted test edit (the plane_oracle injector needle) was
   enough, and the profile-first rule from STAGE-020 worked as framed. The two process
   notes are XS follow-ups in the stage backlog: the stale injector assert message, and
   AGENTS.md section 6 not listing the timing examples.

3. **Is there a follow-up spec I should write now before I forget?**
   — SPEC-070, the Compression 7 speed-up, is already framed in STAGE-020 and is next.
   Nothing new.

4. **Where was the worst defect caught?**
   — build
   *(A tail constructor on BitReader failed the dead_code lint and was replaced by
   setting the position after `BitReader::new`. No defect reached verify.)*

5. **What can a user do now that they couldn't before?**
   — A Q2M 14-bit plane unpacks on one thread in 9.7 ms instead of 239 ms (about 25x,
   reproduced at verify as 233.4 to 9.9 ms), and 8 plane units on 8 threads take 1.4 ms
   instead of 31.2 ms, with byte-identical output.
