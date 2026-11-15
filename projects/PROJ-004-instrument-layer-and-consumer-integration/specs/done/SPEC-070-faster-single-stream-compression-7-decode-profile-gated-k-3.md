---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-070
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M          # stamped at ship: what it ACTUALLY took, same scale.
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
  decisions: [DEC-065, DEC-056, DEC-037, DEC-050, DEC-018, DEC-016, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-036, SPEC-037, SPEC-044, SPEC-046, SPEC-058, SPEC-069]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-020's <capability>". Optional; null is acceptable.
value_link: "a ~6.5x faster single-threaded Compression 7 decode, so STAGE-020's restart-free single-scan cameras (K-3 III Mono, M Typ 246) open fast without units"

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
  tokens_estimate: 30000000
  sessions:
    - cycle: build
      handoff: HANDOFF-147
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 17015779
      estimated_usd: 12.03
      duration_minutes: 105
      recorded_at: 2026-11-14
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (85 unique message.ids, all claude-opus-5-5, no subagents): input 180 + cache-creation 193,889 (1-hour) + cache-read 16,753,344 + output 68,366 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). The handback floor was 13,803,286 and $10.10."
    - cycle: verify
      handoff: HANDOFF-148
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 14633694
      estimated_usd: 10.22
      duration_minutes: 60
      recorded_at: 2026-11-14
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (81 unique message.ids, all claude-opus-5-5, no subagents): input 168 + cache-creation 171,141 (1-hour) + cache-read 14,410,448 + output 51,937, same rates. APPROVED at ff9137e, no code changes. The handback floor was 13,278,178 and $9.39."
  totals:
    tokens_total: 31649473
    estimated_usd: 22.25
    session_count: 2
shipped_at: 2026-11-14
---

# SPEC-070: Faster Compression 7 decode: a buffered scan reader and an 11-bit Huffman lookahead

## Context

This is the third and last spec of STAGE-020, and it answers monorail-000 / onyx-000
triage item 1b, a faster single-threaded open.

The two held single-strip Compression 7 files decode slowly on an M4 Pro:
- **K-3 III Mono: about 704 ms;**
- **M Typ 246: about 601 ms.**

Both are one restart-free scan, so SPEC-068's units cannot split them. The tiled Pixel
files take 300–330 ms. The stage made this spec profile-gated: measure first, and change
only what the profile names.

## Goal

Make `src/ljpeg.rs`'s entropy decode buffer whole bytes and decode Huffman codes through
an 11-bit lookahead table, keeping today's bit-at-a-time path for everything the table
cannot answer.

- **The hard criteria:**
  - decoded samples are byte-identical on every held file and every tier-A stream;
  - every `Result` is identical, including each error's fields and the samples written
    before an error.
- **The targets** (median, on the build machine):
  - K-3 III **≤ 150 ms**, from a fixed **704–708 ms at `4e48521`**;
  - Typ 246 **≤ 140 ms**, from **601–606 ms**.
- **How it is measured:** a committed release example, `just ljpeg-timing`.

## Design

### 1. Probe results (main at `4e48521`, 2026-11-14; recorded so build is transcription)

Apple M4 Pro, release build, a throwaway example over `plane::unpack_into`, median of 7:

| Kernel | K-3 III | Typ 246 | Pixel 3a | Pixel 4a | Pixel 7 Pro |
|---|---|---|---|---|---|
| crate today | 708.1 ms | 605.7 | 319.3 | 331.3 | 306.1 |
| `u64` buffer + 11-bit `lookup` | 241.6 | 220.3 | 112.4 | 112.5 | 107.8 |
| + rare paths out of the hot function | 217.9 | 203.7 | 102.0 | 100.2 | 96.6 |
| **+ 11-bit `fast` table (code and difference)** | **108.1** | **95.4** | about 60 | 66.3 | about 58 |

- **Every candidate was byte-identical to `unpack_into` on all five files.** The FNV
  digests of the decoded planes matched today's.
- **Profile:** macOS `sample` over 40 K-3 III `unpack_into` calls put **7,676 of 8,395**
  samples (about 91%) in `ScanReader::decode_difference` and `read_bit`. Prediction took
  the rest.
- **Error identity, measured:** the probe linked a copy of today's `src/ljpeg.rs` beside
  the candidate. It compared the whole `Result` (every error field) and the whole
  destination buffer of `decode_into` and `decode_window_into`. Inputs:
  - **28,880** corrupted real streams: 72 Pixel tiles and both strips, mutated by
    truncation, bit flips, injected `0xFF`, injected markers, a trailing lone `0xFF`, and
    random bytes;
  - **3,000** clean and **18,000** corrupted synthetic streams from `encode_lj92`, with
    restart intervals 0–16, precisions 8–16, predictors 1–7 and point transforms 0–2.

  **No difference in any of them.** The probe's full suite was green with the corpus set
  (`cargo test --all-features --no-fail-fast`: 37 binaries, 737 passed, 0 failed, 1
  ignored). A first run had one failure, `q2m_units_allocate_nothing`. That is a
  pre-existing race in `tests/plane_units_alloc.rs`: its counter is one global
  `AtomicUsize`, so a concurrent test's allocation can land in the Q2M window. It passes
  alone on both the probe and main. If it flakes during build, rerun it alone and say so;
  do not fix it here.
- **Table width:** 11 and 12 bits were within 2%; 10, 13 and 14 were slower. 14 was much
  slower on the tiled Pixels, which build 192 tables per file.

### 2. The reader (DEC-065 §1)

`ScanReader` keeps `data`, `byte_pos` and `expected_rst`. It replaces `bit_buffer` and
`bits_left` with:
- `acc: u64`: buffered bits, with the next bit at position `nbits − 1`;
- `nbits: u32`: the count of valid bits, 0–64;
- one "was stuffed" flag per buffered byte, for example a `u64` shifted left once per
  byte, so the most recent byte is bit 0.

**`fill()`** (private, never fails):

```text
while nbits <= 56:
    b = data.get(byte_pos)            else stop      (EOF)
    if b == 0xFF:
        if data.get(byte_pos + 1) == Some(0x00): stuffed, size 2
        else stop                                     (marker, or 0xFF at EOF)
    else: size 1
    acc = acc << 8 | b; nbits += 8; record the stuffed flag; byte_pos += size
```

**`read_bit()`:** if `nbits == 0`, call `fill()`. If it is still 0, return
`self.fetch_byte()`'s error. `fetch_byte` is unchanged, and at this `byte_pos` it must
fail. Keep a defensive `Error::Truncated { at: byte_pos, len: 1 }` for the unreachable
`Ok`, with a comment. Otherwise take the top bit.

**`receive`, `decode_symbol` and `receive_and_extend`** keep their bodies and read bits
through `read_bit`.

**`restart()`:**
1. Let `whole = nbits / 8`.
2. Move `byte_pos` back over the last `whole` buffered bytes, by 1 each, or 2 if
   stuffed.
3. Clear `acc`, `nbits` and the flags.
4. Run today's marker logic unchanged.

### 3. The tables and the fast decode (DEC-065 §2)

`HuffTable` gains two arrays, built at the end of `HuffTable::build` from `mincode`,
`maxcode`, `valptr` and `huffval`:

- **`lookup: [u16; 2048]`:** for each length `l` in 1..=11 and each code `c` of that
  length, entries `c << (11 − l)` to `(c + 1) << (11 − l)`, exclusive, hold `(l, symbol)`.
  0 means "no entry".
- **`fast: [u32; 2048]`:** for each `lookup` entry `(l, s)` at index `i`, with
  `extra = 0` if `s` is 0 or 16, `extra = s` if `s` is 1–15, and no entry if `s > 16`:
  - if `l + extra ≤ 11`: total `= l + extra`;
  - `diff` is 0 for `s = 0`, 32768 for `s = 16`, and otherwise
    `extend((i >> (11 − total)) & (2^s − 1), s)`.

  Any unambiguous encoding of `(total, diff)` is fine. The probe used `total << 16 | diff`
  as 16 bits, which works because −32768 is not a reachable difference and 0x8000 can
  only mean +32768. Separate fields are clearer if they stay small.

**`decode_difference`** (`#[inline]`), in order:

```text
if nbits < 32: fill()
idx = next 11 bits (zero-padded on the right if nbits < 11)
if fast[idx] present and total <= nbits:            consume total; return diff
if lookup[idx] = (l, s) present and l <= nbits:
    s == 0  -> consume l; return 0
    s == 16 -> consume l; return 32768
    1..=15 and l + s <= nbits -> consume l; v = take(s); return extend(v, s)
slow (#[cold], #[inline(never)]): decode_symbol(table) then receive_and_extend
```

- **Consume nothing before choosing the slow path.** That is what makes its result and
  error today's.
- **The fast part must not return `Result`.** Return an `Option<i32>` (or equivalent)
  and map `None` to the cold slow path. The probe measured about 10% from this split.
- **`take(n)`** (n 1–32, `n ≤ nbits`) is a private helper written with `checked_shr` and
  a mask.

### 4. Shape constraints

- **No `unsafe`, no heap allocation, no new dependency, no SIMD.** The tables are arrays,
  and `HuffTable` stays `Clone`.
- **The crate's deny lints hold:**
  - `checked_shl` / `checked_shr`, never `<<` / `>>`;
  - `try_from` or `from`, never `as`;
  - `get` / `get_mut`, never indexing;
  - wrapping or checked arithmetic.

  The probe's hot path already used checked shifts.
- **Unchanged:**
  - `fetch_byte`, `decode_symbol`, `receive`, `receive_and_extend`, `extend` and `pow2`;
  - `parse_headers`, apart from the tables `HuffTable::build` now fills;
  - every reconstruction function, `predict`, `predict_component`, `neighbour`, `Window`
    and `check_window`;
  - every error variant and every public item.
- **The table width is a private `const LUT_BITS: u32 = 11`**, and its sizes derive from
  it.
- **Optional:** an 8-byte refill when the next 8 source bytes hold no `0xFF`. The probe
  measured about 3%. If added, it must still record the stuffed flags (all zero) and stop
  exactly where the byte loop would.

### 5. Timing (DEC-065 §4)

`examples/ljpeg-timing.rs` and a `ljpeg-timing` recipe in `app.just` mirror
`unpack-timing`:
- release build, files from the corpus root;
- one SKIP line per absent file, and exit 0;
- one warm-up, then the median and minimum of 7 runs;
- the machine printed.

It times `plane::unpack_into` on:
- `PENTAX-K3III-MONO/K3III.DNG`;
- `LEICA-M-MONOCHROM-TYP246/M2462362.DNG`;
- `GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng`.

No CI gate. The handback records the before figures (the example run on `4e48521` plus
only the example) and the after figures, back to back.

## Inputs

- **Files to read:**
  - `src/ljpeg.rs`: `HuffTable` and `build`, `ScanReader` (every method), `extend`,
    `pow2`, both reconstruct loops, `decode_into`, `decode_window_into`, `open_scan`, and
    the `mod tests` helpers (`BitWriter`, `encode_scan`, `encode_lj92`, the segment
    builders);
  - `tests/plane_oracle.rs`: `DECODABLE_COMPRESSED`, `inject_two_component_neighbour_fault`
    and `ljpeg_decode_fault_turns_compression7_oracle_red`;
  - `tests/plane_tiles.rs`: the allocation scan of the `Window`…`open_scan` section;
  - `tests/plane_units_alloc.rs`;
  - `fuzz/fuzz_targets/ljpeg.rs`, whose header has the run command;
  - `examples/unpack-timing.rs` and `app.just`.
- **Decisions:** DEC-065, DEC-056 (the window), DEC-037 (two-component), DEC-050 (EOI is
  optional), DEC-018 (modular arithmetic), DEC-016 (no allocation), DEC-003.

## Outputs

- **Files modified:**
  - `src/ljpeg.rs`: `HuffTable` (two tables), `ScanReader` (buffer, `fill`, `take`, the
    fast `decode_difference`, `restart`'s give-back), the module and function docs that
    describe the one-byte reader, and the new unit tests (AC2, AC3);
  - `app.just`: the `ljpeg-timing` recipe;
  - `CHANGELOG.md`: `[Unreleased]` Changed, with the before and after numbers.
- **Files created:** `examples/ljpeg-timing.rs`.
- **New public items:** none.

## Acceptance Criteria

- [ ] **AC1. Byte-identical on every held file (tier B).** Every existing oracle stays
  green **unedited**:
  - the dnglab `raw_checksum` pins for all five `DECODABLE_COMPRESSED` files (K-3 III,
    Typ 246 and the three tiled Pixels) in `plane_oracle`;
  - SPEC-068's any-order assembly and units tests on held files;
  - the develop pins downstream.
- [ ] **AC2. Every `Result` identical to today's reader, on synthetic tables and streams
  (tier A).** A new unit test in `src/ljpeg.rs`'s `mod tests`,
  `buffered_reader_matches_the_bit_at_a_time_reader`, compares the new `ScanReader`
  against a **test-local copy of today's reader**: a `ReferenceReader` holding today's
  `fetch_byte`, `read_bit`, `receive`, `decode_symbol`, `receive_and_extend` and
  `restart`, copied verbatim from `4e48521` and reading the same `HuffTable`'s
  `mincode` / `maxcode` / `valptr` / `huffval`. That is today's code as the old-behaviour
  reference, not a value oracle. The probe's seeded-PRNG generator, for each case:
  - **builds a random valid canonical table.** Random `BITS` over lengths 1–16, including
    tables with codes of length 12–16 and of length exactly 16, symbols drawn from 0–16,
    plus some symbols above 16 (the table is still valid; decoding them is the error
    case);
  - **encodes a random symbol stream** with that table: each code, then its `SSSS` extra
    bits, with `0xFF` stuffing, and `RSTm` markers every `Ri` symbols. `Ri` is 0, 1, 2,
    3 and 7. Use the test `BitWriter`, extended for variable-length codes if needed.
    Include streams where a stuffed `0xFF 0x00` falls in the last bytes before a marker;
  - **corrupts copies of it:** truncation at every byte for short streams, single bit
    flips, an injected `0xFF`, an injected or deleted marker, a trailing lone `0xFF`, and
    deleted bytes;
  - **drives both readers with the same calls** (`decode_difference` per symbol, and
    `restart` every `Ri`), stopping at the first `Err`. It asserts each `Ok` value equal,
    the first `Err` equal (variant and fields, by `PartialEq` or `Debug`), and
    `byte_pos` equal after every `restart`.

  At least 20,000 cases, under 10 s in a debug `just test`. Print the outcome counts
  (Ok, and each error variant) so the verifier can see every class was reached.
- [ ] **AC3. Full-decode identity on synthetic planes (tier A).** A unit test
  `decode_into_is_unchanged_on_lj92_streams_with_restarts` runs `encode_lj92` streams.
  These cover restart intervals 0, 1, 2, 3, 5, 7 and 16, precisions 2, 8, 12, 14 and 16,
  predictors 1–7, point transforms 0–2, and sizes from 1×1 up. For each it asserts:
  - the clean stream decodes to exactly the encoded samples (the existing round-trip
    property, over the wider grid);
  - each corrupted copy gives the same `Result` and the same `out` contents as a decode
    whose `ScanReader` is the `ReferenceReader`. A small test-local generic
    reconstruction is acceptable here, or drive `reconstruct` through a test-only seam.
    **The builder chooses, and says which.**
- [ ] **AC4. Every existing lossless-JPEG test stays green unedited.** That covers:
  - the 40 `ljpeg` unit tests, including `restart_consumes_rstm_and_byte_aligns`'s
    `byte_pos == 3` and `malformed_inputs_are_typed_errors_never_panics`;
  - `ljpeg_shape_binding`;
  - `plane_tiles`;
  - the Compression 7 red-proof `ljpeg_decode_fault_turns_compression7_oracle_red` and
    its control. Its needle, `neighbour(out, slot, 2)`, still occurs exactly twice.
- [ ] **AC5. No allocation, no `unsafe`.**
  - `plane_tiles`' allocation scan of the `Window`…`open_scan` section is green unedited.
  - `plane_units_alloc` is green unedited: allocation counts are unchanged.
  - No `unsafe` and no new dependency.
- [ ] **AC6. The timing targets (verify checks them; CI does not).** `just ljpeg-timing`
  on the build machine reports K-3 III ≤ 150 ms and Typ 246 ≤ 140 ms, against
  704–708 ms and 601–606 ms at `4e48521`. The Pixel 4a is recorded.
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

  The ljpeg fuzz target runs clean for 60 s (the command in `fuzz/fuzz_targets/ljpeg.rs`'s
  header), and so does `just fuzz-plane 60`.

## Failing Tests

Written first in build.

- **`src/ljpeg.rs` unit tests:**
  - `buffered_reader_matches_the_bit_at_a_time_reader` (AC2);
  - `decode_into_is_unchanged_on_lj92_streams_with_restarts` (AC3).

These pin behaviour, so they pass against today's code too: the reference is today's
reader. The test-first evidence is the **red-proof list**. The builder applies each
mutation by hand to the new code, shows the named test red, and reverts:

- `fast` consumes `total − 1` bits → AC2, AC3 and AC1;
- `fill` treats `0xFF xx` (xx ≠ 0) as a data byte → AC2 (marker and truncation errors);
- `restart` gives back every buffered byte as 1 source byte, ignoring the stuffed flag →
  AC2 (`byte_pos` after a restart). The generator must produce a stuffed byte among the
  unread bytes before a marker, or this stays green; say how it was produced;
- the `total <= nbits` guard dropped, so a zero-padded peek is accepted → AC2
  (truncation);
- `SSSS` 16 stored in `fast` as −32768, or as 0 → AC2;
- the extra-bits check in the `lookup` step dropped (`l + s <= nbits`) → AC2;
- `read_bit`'s exhausted branch returning `Truncated { at: byte_pos }` instead of
  `fetch_byte`'s error → AC2 (`LjpegUnexpectedMarker` cases);
- **expected to stay green:**
  - `LUT_BITS` 11 → 10;
  - the fast path disabled, so every symbol takes the slow path.

  Both only change speed, so only the timing example shows them. Say so; these are the
  benign mutants.

## Non-Goals

- SIMD intrinsics, `unsafe`, and a new dependency.
- Threads inside irradiance, and splitting a restart-free scan.
- Speeding up prediction or reconstruction.
- Restart-interval units (DEC-063 §4).
- Any output or error change.
- A CI timing gate.
- A `just fuzz-ljpeg` recipe. Use the header command; adding one is a separate XS.

## Notes for the Implementer

- **Source scans on `src/ljpeg.rs`:**
  - `plane_oracle`'s Compression 7 injector needs `neighbour(out, slot, 2)` exactly
    twice. Do not touch `predict_component`;
  - `plane_tiles` scans from `pub(crate) struct Window` to `fn open_scan` for allocating
    tokens. `ScanReader` and `HuffTable` sit above `Window`, but keep `vec!`, `Vec::`,
    `Box`, `to_vec` and `collect` out of that range anyway;
  - `ljpeg_shape_binding` scans `parse_frame_header`, which is untouched.
- **Field names.** `restart_consumes_rstm_and_byte_aligns` reads `r.byte_pos`. Keep that
  field name and its meaning ("the next source byte the reader would fetch, after a
  restart").
- **The probe's code is in the design PR's description, for reference only.** Its hot
  path used `as` casts in the `fast` decode; rewrite it lint-clean. Its 28,880-case
  corpus differential is not to be committed (DEC-003: no corpus bytes in tests), but you
  may run a local copy to double-check. Say so if you do.
- **Run the ljpeg fuzz target early**, then the long tier-B suite in the background. The
  tier-B any-order test takes about 9.5 min in debug (signal
  `slow-tier-b-tests-in-debug`; do not fix it here).

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Name the inlining in the design. The probe's speed depended on `#[inline(always)]`
   on the hot decode, but the spec said `#[inline]`, which the build and the verify both
   measured about 15% slower. Also, run the full suite with the corpus in parallel more
   than once at design: the `plane_units_alloc` race showed once in the probe, and by
   this head it fails every parallel local run.

2. **Does any template, constraint, or decision need updating?**
   — No decision changes. DEC-065 held as written; the build's two deviations (a biased
   `fast` encoding instead of an `as` cast, and `#[inline(always)]`) are recorded in the
   handback, and the verify judged both. The `plane_units_alloc` race is promoted to the
   top of the STAGE-020 follow-ups, because a local `just test` with the corpus is now
   reliably red until it is fixed.

3. **Is there a follow-up spec I should write now before I forget?**
   — The race fix, as an XS before any more corpus-heavy work. Also a `just fuzz-ljpeg`
   recipe that creates its corpus dir (XS). Prediction is now a larger share of the
   decode, but any change there is profile-gated, and nobody has asked for it.

4. **Where was the worst defect caught?**
   — none
   *(The build met every criterion first time; the verify found no defect in the code.
   Its one finding is the pre-existing test race.)*

5. **What can a user do now that they couldn't before?**
   — A K-3 III Mono Compression 7 plane decodes on one thread in 103 ms instead of
   692 ms (verify: 710 to 105), and a Typ 246 in 91 ms instead of 591 (verify: 605 to
   90), about 6.6x, with the tiled Pixels near 61 ms instead of 324. Samples and errors
   are identical.
