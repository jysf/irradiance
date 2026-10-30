---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-066
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-29
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-028

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 43553730 # REAL combined count — what cost-audit reads
  estimated_usd: 102.25   # tokens_total × your rate, or your harness's number
  duration_minutes: 35
  branch: claude/spec-028-multi-strip-b1667e
  pr: https://github.com/jysf/irradiance/pull/41
  completed_at: 2026-10-29         # YYYY-MM-DD
  notes: "all 7 Failing Tests + the corpus check pass; one resolved spec-text tension (per-strip layer-0 is byte-ceil, not literal `== rows*width*bits`, so sub-byte strips can carry their independent tail padding — see Deviations); tests placed in tests/plane_unpack.rs (the unpack_into home, AGENTS.md §12) not src/ifd.rs mod tests; tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated"
  synced_at: 2026-10-29
---

# HANDOFF-066: Multi-strip mosaic plane assembly — unpack a Bayer plane stored as N strips (`plane::unpack_into` is single-strip-only, SPEC-012 `src/plane.rs`; the Nikon D300S is 12 strips)

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-028` to
`claude-opus-4-8` (acting as implementer) for the **build** cycle — generalise
`plane::unpack_into` from the single strip `SPEC-012` scoped it to, to `N` strips
(`N >= 1`) assembled into the plane buffer, each unpacked independently, with the
single-strip path byte-for-byte unchanged.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-006-cfa-sensor-model-and-bit-exact-mosaic-plane.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-028-multi-strip-mosaic-plane-assembly-unpack-a-bayer-plane-stored-as-n-strips-plane-unpack-into-is-single-strip-only-spec-012-src-plane-rs-225-the-nikon-d300s-is-12-strips.md`
  — read in full; the Acceptance Criteria and the Failing Tests are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` traps,
  `just lint-ci` for CI's clippy, the five-lint panic policy, the corpus env var.
- **The code the change sits in:** `src/plane.rs` — `unpack_into` (the whole
  function: the single-strip gate, the layer-0 check, the one strip read, the two
  unpack paths `unpack_byte_aligned` / `unpack_bitstream`). `src/lib.rs` — the
  `Error` enum (`UnsupportedStripLayout`, `PackedSizeMismatch`, `Truncated`), and
  the new `MissingRowsPerStrip`. `tests/plane_unpack.rs` + `tests/support/tiff.rs`
  — the TIFF fixture builder the plane-unpack tests actually use.

### Decisions that apply

- `DEC-008` — the two-path unpack (byte-aligned 8/16-bit in file byte order vs.
  sub-byte 12/14-bit MSB-first bitstream), selected by `bits % 8 == 0`. Its rule
  that **each strip is byte-padded independently** is the crux: strips must be
  unpacked one at a time into their own row slice, never by concatenating strip
  bytes (which for a sub-byte depth would read a strip's first sample across the
  previous strip's tail padding).
- `DEC-016` — `unpack_into` allocates nothing; the caller owns `dst`.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Applying to `src/plane.rs` /
`src/lib.rs`:

- `no-panics-on-untrusted-input` (blocking) — library paths. Every strip
  offset/length is bounds-checked into `Truncated`, every product is `checked_*`
  / `saturating_*`, every write goes through `split_at_mut_checked`. No
  `unwrap`/`expect`/indexing/`panic`/overflowing arithmetic; `just lint-no-allow`
  and the red-proof gate stay green.
- `test-before-implementation` (blocking) — the spec's Failing Tests are the
  contract, added as `tests/plane_unpack.rs` integration tests.
- `one-spec-per-pr` (blocking) — the PR references SPEC-028 only.
- `provenance-recorded-per-algorithm` — multi-strip assembly is **not** a new
  algorithm: it is TIFF 6.0's strip model applied over the existing DEC-008
  unpack, so it extends the existing `src/plane.rs` provenance row (class 1,
  specification), it does not add one. No implementation was consulted.

### Prior related work

- `SPEC-012` / `HANDOFF-???` — the single-strip unpacker this generalises.
- `SPEC-024` (PR #37) — the CFA gate that admits the Bayer plane this now unpacks.
- `SPEC-029` — the WhiteLevel-exceedance policy; a Bayer file may still trip
  `SampleExceedsWhiteLevel` AFTER assembly until 029 lands. The Nikon does.
- `SPEC-027` — registers the Nikon in the corpus manifest + extends the fuzz
  corpus; owns the manifest/oracle pinning this build deliberately did not add.

## Expected Deliverables

- Multi-strip assembly in `src/plane.rs::unpack_into`: a loop over `N` strips
  (`N == StripOffsets.len() == StripByteCounts.len()`, `N >= 1`), each strip
  `rows_i = min(RowsPerStrip, height - i*RowsPerStrip)` rows unpacked
  independently into its own row slice; `RowsPerStrip` required when `N > 1`
  (typed error if absent); `N == 1` byte-for-byte the old path.
- The new `Error::MissingRowsPerStrip`, and the updated `UnsupportedStripLayout`
  (count mismatch / zero strips, not "exactly one strip").
- The spec's Failing Tests + the Nikon corpus check, all passing; the existing
  single-strip plane/develop/warp oracles staying green (the regression).
- All gates green locally — `just test`, `just lint-ci` (`-D warnings`),
  `cargo fmt --check`, `just validate` — plus the `plane` fuzz target run.
- A PR against `main` referencing PROJ-002 / STAGE-006 / SPEC-028 / HANDOFF-066 /
  DEC-008 / DEC-016.

## Out of Scope

- The WhiteLevel-exceedance policy — **SPEC-029** (the Nikon trips it post-assembly).
- The bit-exact plane oracle vs `dnglab` — **SPEC-025**.
- Registering the Nikon in `tests/corpus/manifest.toml` and extending the fuzz
  corpus with a multi-strip/CFA seed — **SPEC-027**.
- Tiled planes (`TileOffsets`/`TileByteCounts`) — still unsupported.
- Demosaic / colour / any pixel interpretation — later stages.

## Return Criteria — how to hand back

On success:
1. Fill the `handback:` front-matter — including a real `tokens_total`.
2. Fill the `## Handback` section (the reflection is part of it).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it.*

### Execution notes

- **Branch / PR:** `claude/spec-028-multi-strip-b1667e` → https://github.com/jysf/irradiance/pull/41
- **Completed at:** 2026-10-29
- **All acceptance criteria met?** Yes — every AC in SPEC-028.
  - **AC1** 2-strip 16-bit plane unpacks to the same samples as the single-strip
    equivalent (`a_two_strip_16bit_plane_unpacks_like_one_strip`, which asserts
    both the samples and `dst == one_strip_dst`).
  - **AC2** 2-strip 14-bit plane with independent per-strip byte padding — strip 2
    not shifted by strip 1's 4 padding bits
    (`a_two_strip_14bit_plane_respects_per_strip_byte_padding`; the packer
    round-trips the measured Q2M head to prove it is the reader's inverse).
  - **AC3** uneven last strip (`height % RowsPerStrip != 0`) fills the right
    remainder (`an_uneven_last_strip_unpacks_the_remainder`, 3 rows / 2-per-strip).
  - **AC4** single-strip unchanged (`a_single_strip_plane_is_unchanged`) + the
    tier-B Q2M 14-bit and M Monochrom 16-bit unpack tests stay green byte-for-byte.
  - **AC5** multi-strip plane without `RowsPerStrip` →
    `Error::MissingRowsPerStrip { strips: 2 }`, no panic
    (`a_multi_strip_plane_without_rows_per_strip_is_a_typed_error`).
  - **AC6** truncated strip → `Error::Truncated`
    (`a_truncated_strip_is_truncated_not_a_panic`); byte-count mismatch →
    `Error::PackedSizeMismatch { expected_bits: 32, strip_bits: 48 }`
    (`a_strip_byte_count_mismatch_is_packed_size_mismatch`). Panic-free on both.
  - **AC7 (corpus)** `NIKON-D300S/_DSC4615.DNG` (12 strips, 16-bit) assembles into
    the full 4352×2868 plane — `UnsupportedStripLayout` is gone
    (`the_nikon_d300s_twelve_strip_plane_no_longer_rejects_the_strip_layout`). It
    then trips `SampleExceedsWhiteLevel` (sample 741623 = 3835 > WhiteLevel 3808),
    which is **expected** and is SPEC-029's to relax.

### Cost self-report

- **Tokens (total):** 43553730
- **Estimated USD:** 102.25
- **Duration (minutes):** ~35
- **Source of the number:** harness report — summed from this session's own Claude
  Code transcript JSONL (`input + output + cache_creation + cache_read` across all
  usage records; this desktop Code interface exposes no `/cost` slash command),
  the same all-inclusive convention SPEC-024 build (48.5M) used. `estimated_usd`
  at per-category Opus list rates — an order-of-magnitude, cache-read-dominated figure.

### Drift and new artifacts

- **New decisions emitted:** none. A new `DEC-*` was weighed for the per-strip
  layer-0 shape and judged unnecessary: it is DEC-008's "each strip padded to a
  byte boundary independently" made arithmetic, not a fresh decision with live
  alternatives. Flagged below for the verify/ship cycle to confirm.
- **Deviations from spec:**
  1. **Per-strip layer-0 is `StripByteCounts[i] == ceil(rows_i*width*bits / 8)`,
     not the spec Outputs' literal `StripByteCounts[i] * 8 == rows_i*width*bits`.**
     The literal equality cannot hold for a sub-byte strip whose
     `rows_i*width*bits` is not a multiple of 8 — exactly the case AC2 ("each
     strip byte-padded independently") requires to be non-vacuous. The byte-ceil
     form reduces to the literal equality whenever there is no padding (every
     8/16-bit strip, and any 12/14-bit strip that happens to land on a byte), so
     it is a strict generalisation, and it keeps `N == 1` byte-for-byte (the
     `layer0_arithmetic_is_enforced` regression still returns the identical
     `PackedSizeMismatch { 140, 112 }`). Resolving the spec-text tension in favour
     of the testable acceptance criterion, not a redesign.
  2. **Tests live in `tests/plane_unpack.rs` + `tests/support/tiff.rs`, not the
     `Build` DSL in `src/ifd.rs mod tests`** the spec names. `unpack_into` is a
     public function; AGENTS.md §12 puts its behaviour tests in `tests/<area>.rs`,
     which is where every existing `unpack_into` test already is, and the TIFF DSL
     those tests use is `tests/support/tiff.rs` (the `Build` DSL in
     `src/ifd.rs mod tests` drives the IFD reader and never `unpack_into`). The
     spec's wording appears to conflate the two builders. Extended the right one
     (a local `MultiStrip` fixture on `tests/support/tiff.rs`'s primitives).
  3. One extra line of surfacing is **not** needed — `src/bin/irr.rs`'s `unpack`
     command already reads the whole plane, so the Nikon's new behaviour is
     observable via `irr unpack` with no §11 reader change.
- **Follow-up work identified:** none ship-blocking. SPEC-029 (relax
  `SampleExceedsWhiteLevel`, which the Nikon now trips) and SPEC-027 (register the
  Nikon in the manifest + a multi-strip fuzz seed) are the already-framed next
  specs; both are consumers/companions of this change, not gaps in it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two small tensions, both resolved in favour of the acceptance criteria and
   the codebase (see Deviations 1 and 2): the literal per-strip layer-0 equality
   contradicts the 14-bit-padding AC, and the named test location
   (`src/ifd.rs mod tests`) is not where `unpack_into` is tested. The core
   algorithm was pinned precisely by the spec and was mechanical to write.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-008's independent-per-strip-padding rule is the load-bearing one and
   the spec's Notes call it out explicitly. The per-strip cursor
   (`split_at_mut_checked`) and the post-loop coverage check are what make it
   panic-free and fully-covering without a separate decision.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Writing the per-strip layer-0 as a byte-ceil from the
   start (rather than the literal equality) was the one judgement call, and
   confirming it against the real 12-strip Nikon file early (via `irr unpack`)
   before writing the synthetic tests made the rest straightforward.
