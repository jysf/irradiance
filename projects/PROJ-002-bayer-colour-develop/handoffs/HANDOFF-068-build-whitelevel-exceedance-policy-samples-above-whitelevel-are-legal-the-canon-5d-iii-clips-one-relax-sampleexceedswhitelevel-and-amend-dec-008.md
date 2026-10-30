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
  id: HANDOFF-068
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-29
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-029

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
  tokens_total: 31829740 # REAL combined count — what cost-audit reads
  estimated_usd: 23.45    # tokens_total × your rate, or your harness's number
  duration_minutes: 20
  branch: claude/spec-029-whitelevel-aab6a4
  pr: PR_PLACEHOLDER
  completed_at: 2026-10-29          # YYYY-MM-DD
  notes: "both Failing Tests + both corpus Bayer DNGs (Canon single-strip, Nikon 12-strip) now unpack; Error::SampleExceedsWhiteLevel REMOVED (not strict-mode); DEC-029 written amending DEC-008's permanent-invariant clause; tests in tests/plane_unpack.rs (the unpack_into home, AGENTS.md §12), not the spec's literal `src/plane.rs` wording — same divergence HANDOFF-066 recorded; tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated"
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-068: WhiteLevel-exceedance policy — samples above `WhiteLevel` are legal (the Canon 5D-III clips one); relax `SampleExceedsWhiteLevel` and amend DEC-008

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-029` to
`claude-opus-4-8` (acting as implementer) for the **build** cycle — amend DEC-008
and relax `plane::unpack_into` so a sample above `WhiteLevel` is no longer an
error (specular highlights are legal), preserving raw values unchanged so the
bit-exact plane survives for SPEC-025's oracle.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-006-cfa-sensor-model-and-bit-exact-mosaic-plane.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-029-whitelevel-exceedance-policy-samples-above-whitelevel-are-legal-the-canon-5d-iii-clips-one-relax-sampleexceedswhitelevel-and-amend-dec-008.md`
  — read in full; the Acceptance Criteria and the Failing Tests are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` traps,
  `just lint-ci` for CI's clippy, the five-lint panic policy, the corpus env var.
- **The code the change sits in:** `src/plane.rs` — `unpack_into`'s
  post-assembly `max > WhiteLevel` loop (the ONLY thing that changes). `src/lib.rs`
  — the `Error` enum (`SampleExceedsWhiteLevel`, removed) and its `Display`. The
  two DEC-008 unpack paths (`unpack_byte_aligned` / `unpack_bitstream`) and the
  per-strip assembly are UNCHANGED. `tests/plane_unpack.rs` — the `unpack_into`
  test home (`Fixture` single-strip + `MultiStrip` builders).

### Decisions that apply

- `DEC-008` — the two-path unpack and its WhiteLevel-assertion clause, which this
  build **amends** via the new DEC. Its byte-alignment rule is untouched.
- `DEC-016` — `unpack_into` allocates nothing; the caller owns `dst`. (Unchanged.)
- `DEC-029` (NEW, this build) — samples above `WhiteLevel` are legal; the
  assertion is retired and the byte-swap guard moves to SPEC-025 + the per-strip
  layer-0 check.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Applying to `src/plane.rs` /
`src/lib.rs`:

- `no-panics-on-untrusted-input` (blocking) — `unpack_into` stays panic-free;
  removing a check cannot add a panic, and the lint gate / red-proof stay green.
- `test-before-implementation` (blocking) — the spec's two Failing Tests are the
  contract, added to `tests/plane_unpack.rs`.
- `one-spec-per-pr` (blocking) — the PR references SPEC-029 only.
- `provenance-recorded-per-algorithm` — no new algorithm (a check is removed, not
  added); no provenance row.

### Prior related work

- `SPEC-028` (PR #41/#43, HANDOFF-066) — multi-strip assembly; the Nikon plane
  it assembles tripped this assertion post-assembly (sample 3835 > 3808). 029
  completes that unblock.
- `SPEC-024` (PR #37) — the CFA gate admitting the Bayer plane.
- `SPEC-025` — the bit-exact mosaic-plane oracle, now unblocked; it inherits the
  byte-swap guard this build retires from the in-library assertion.
- `SPEC-027` — registers both Bayer DNGs in the corpus manifest + fuzz corpus;
  owns the manifest/oracle pinning this build deliberately did not add.

## Expected Deliverables

- `src/plane.rs::unpack_into`: the `max > WhiteLevel` loop removed; raw values
  preserved (no clamp); the two DEC-008 unpack paths + per-strip assembly
  byte-for-byte unchanged.
- `src/lib.rs`: `Error::SampleExceedsWhiteLevel` removed (variant + `Display`);
  doc references reconciled.
- `DEC-029` amending DEC-008's permanent-invariant clause; `just decisions-index`
  regenerated.
- The spec's two Failing Tests + both corpus Bayer checks passing; the mono
  regression (Q2M, M Monochrom) byte-identical.
- All gates green locally — `just test`, `just lint-ci` (`-D warnings`),
  `cargo fmt --check`, `just validate`.
- A PR against `main` referencing PROJ-002 / STAGE-006 / SPEC-029 / HANDOFF-068 /
  DEC-008 / DEC-029.

## Out of Scope

- Clamping/normalising to WhiteLevel — develop (STAGE-007+); would break the
  bit-exact plane.
- The bit-exact oracle vs `dnglab` — **SPEC-025**.
- Registering the Bayer DNGs in `tests/corpus/manifest.toml` + a CFA fuzz seed —
  **SPEC-027**.
- A strict reject-above-white mode — not needed by any consumer (see DEC-029
  Option C); its own spec if ever wanted.

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

- **Branch / PR:** `claude/spec-029-whitelevel-aab6a4` → PR_PLACEHOLDER
- **Completed at:** 2026-10-29
- **All acceptance criteria met?** Yes — every AC in SPEC-029.
  - **AC1** a synthetic plane with a sample above `WhiteLevel` unpacks `Ok` and
    the raw value is present unchanged in `dst`
    (`a_sample_above_white_level_unpacks_and_is_preserved`: WhiteLevel 15000,
    sample 15464 preserved, asserts `dst[2] > WhiteLevel`).
  - **AC2** mono planes within WhiteLevel unpack byte-identically
    (`a_plane_within_white_level_is_unchanged`, tier-A, incl. the `== WhiteLevel`
    boundary) + the tier-B Q2M 14-bit / M Monochrom 16-bit tests stay green.
  - **AC3 (corpus)** `CANON-EOS-5D-MARK-III/5G4A9394.DNG` unpacks with no error,
    its above-white sample (15464 > WhiteLevel 15000) preserved
    (`the_canon_5d_iii_plane_unpacks_with_its_above_white_sample_preserved`).
  - **AC4** `DEC-029` exists (status accepted), records the reversed consequence,
    and names SPEC-025 as the byte-swap guard; DEC-008 carries a forward-pointer
    amendment note.
  - **AC5** DEC-008's two-path byte-alignment rule is untouched — only the
    post-unpack WhiteLevel loop was removed.
- **Combined 028+029 unblock:** both corpus Bayer DNGs now unpack to a full
  `width×height` plane — Canon (single strip, 5920×3950, max 15464) and Nikon
  (12 strips via SPEC-028, 4352×2868, max 4095 > WhiteLevel 3808). Confirmed via
  `irr unpack` and the two corpus tests (which ran, not skipped). SPEC-025 is
  unblocked.

### Cost self-report

- **Tokens (total):** 31829740
- **Estimated USD:** 23.45
- **Duration (minutes):** ~20
- **Source of the number:** harness report — summed from this session's own Claude
  Code transcript JSONL (`input + output + cache_creation + cache_read` across all
  usage records; this desktop Code interface exposes no `/cost` slash command),
  the same all-inclusive convention SPEC-024/028 build used. `estimated_usd` at
  per-category Opus list rates — an order-of-magnitude, cache-read-dominated figure.

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-029` — "Samples above `WhiteLevel` are legal — amending DEC-008's
    'permanent invariant'." Removes the assertion + error variant, keeps raw
    values, moves the byte-swap guard to SPEC-025 + the per-strip layer-0 check.
- **Deviations from spec:**
  1. **Tests live in `tests/plane_unpack.rs`, not the spec's literal
     `src/plane.rs` tests wording.** `unpack_into` is a public function; AGENTS.md
     §12 puts its behaviour tests in `tests/<area>.rs`, where every existing
     `unpack_into` test already is (the `src/plane.rs mod tests` drives only
     `BitReader`). This is the SAME "src/… mod tests" wording divergence
     HANDOFF-066 recorded for SPEC-028; resolved the same way.
  2. **`Error::SampleExceedsWhiteLevel` REMOVED, not kept behind a strict mode**
     (the spec left this to build's judgement; the default had to accept
     above-white). A strict mode would be dead code with no live consumer and no
     red-proof — DEC-029 Option C records why, and reopens it as its own spec if
     a consumer ever appears.
  3. **`src/bin/irr.rs` cosmetic fix (one expression):** the `unpack` command
     printed a hardcoded `{max} <= {w}`, which my change would make a false
     statement for an above-white plane. Now it prints the real relation
     (`15464 > 15000`). A §11 reader tweak, not in the spec's Outputs, but the
     change made the old line factually wrong.
- **Follow-up work identified:** none ship-blocking. SPEC-025 (the oracle, now
  unblocked) and SPEC-027 (manifest + CFA fuzz seed) are the already-framed next
  specs. `decisions-audit` emits one advisory scope-overlap warning (DEC-016 ↔
  DEC-029 on `src/plane.rs`): confirmed non-contradictory — DEC-016 is the
  allocation contract, DEC-029 the value policy; same benign class as the repo's
  existing DEC-018/019/024 and DEC-020/021 overlaps, 0 structural errors.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Almost nothing — the spec was precise and gave the real falsifier numbers.
   The only judgement call was the error variant's fate (remove vs strict mode),
   which the spec explicitly delegated; I removed it and documented the reopen
   path in DEC-029. The test-location wording (`src/plane.rs`) is the same
   §12-vs-spec divergence SPEC-028 already hit.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-008 is the load-bearing one and the spec's Notes call out exactly
   which clause to amend. Probing both corpus files first (confirming 15464 >
   15000 and 3835 > 3808 against the real bytes) made the DEC's evidence concrete
   rather than quoted.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Removing a check is the easy half; the weight was in
   writing DEC-029 honestly about the cost (the free byte-aligned byte-swap guard
   is genuinely reduced, and now rests on SPEC-025). Confirming that trade is
   acceptable only because SPEC-025 makes the oracle a blocking part of the
   plane's contract is the sentence that matters.
