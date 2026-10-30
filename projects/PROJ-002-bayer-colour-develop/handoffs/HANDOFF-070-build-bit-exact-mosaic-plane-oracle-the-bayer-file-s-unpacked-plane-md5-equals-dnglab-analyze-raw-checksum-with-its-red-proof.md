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
  id: HANDOFF-070
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-30
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-025

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
  tokens_total: 15781793 # REAL combined count — what cost-audit reads
  estimated_usd: 46.15    # tokens_total × your rate, or your harness's number
  duration_minutes: 18
  branch: claude/spec-025-plane-oracle-1bdbe1
  pr: null                          # filled in the PR-ref bookkeeping commit
  completed_at: 2026-10-30          # YYYY-MM-DD
  notes: "test-only — no src/ change, no manifest change, no new DEC. Both corpus Bayer planes (Canon single-strip, Nikon 12-strip) MD5-match dnglab analyze --raw-checksum; the byte-aligned red-proof (byte-order swap in unpack_byte_aligned) turns the Nikon plane red with a green control — the guard DEC-029 moved onto this oracle. Added to tests/plane_oracle.rs reusing SPEC-013's apparatus; stage_probe_crate generalised from a bool to an Option<fn(&Path)> injector so the mono (sub-byte) and Bayer (byte-aligned) faults share it, mono behaviour unchanged. tokens summed from this session's own Claude Code transcript JSONL (in+out+cache); estimated_usd at Opus list rates, cache-read-dominated."
  synced_at: 2026-10-30
---

# HANDOFF-070: Bit-exact mosaic plane oracle — the Bayer file's unpacked plane MD5 equals `dnglab analyze --raw-checksum`, with its red-proof

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-025` to
`claude-opus-4-8` (acting as implementer) for the **build** cycle — assert that
both corpus Bayer DNGs' unpacked mosaic planes are bit-exact (MD5 ==
`dnglab analyze --raw-checksum`) and add the mandatory red-proof on the
byte-aligned unpack path. STAGE-006's capstone; test-only, no `src/` change.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-006-cfa-sensor-model-and-bit-exact-mosaic-plane.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-025-bit-exact-mosaic-plane-oracle-the-bayer-file-s-unpacked-plane-md5-equals-dnglab-analyze-raw-checksum-with-its-red-proof.md`
  — Acceptance Criteria + Failing Tests are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain` traps,
  `just lint-ci` for CI's clippy, the five-lint panic policy, the corpus env var.
- **The oracle apparatus reused verbatim:** `tests/plane_oracle.rs` — SPEC-013's
  `decode_and_hash`, `plane_md5`, `TempDir`, `stage_probe_crate`,
  `build_and_run_probe`, `PROBE_MAIN`. The direct-path skip pattern copied from
  `tests/plane_unpack.rs:895` (`the_nikon_d300s_twelve_strip_plane_...`).
- **The red-proof's injection target:** `src/plane.rs:211–212` —
  `unpack_byte_aligned`'s 16-bit branch, the `from_le_bytes`/`from_be_bytes` arms.
  Read-only — the fault is injected into a staged *copy*, never the real tree.

### Decisions that apply

- `DEC-029` — samples above `WhiteLevel` are legal; the in-library assertion that
  used to catch a byte-swapped (out-of-range) sample was retired, and DEC-029
  **explicitly moves that byte-swap guard onto this oracle**. The byte-aligned
  red-proof here is that guard — this closes the loop DEC-029 opened.
- `DEC-017` — the red-proof's mechanism: a fault must be proven to change the
  OUTPUT (control digest ≠ mutant digest), via a real separate compilation of a
  mutated crate copy. Reused unchanged.
- `DEC-008` — the two-path unpack and native-LE byte order; the byte-aligned path
  is the one the Bayer files (both 16-bit) take and the one the red-proof swaps.
- `DEC-003` — CI has no corpus; tier-B + red-proof skip loudly, tier-A is all CI
  sees.

### Constraints that apply

Full text in `./guidance/constraints.yaml`.

- `oracle-must-be-shown-red` (blocking) — the new Bayer oracle ships with a
  deliberate-fault test proving it FAILS on a broken read, plus a green control.
- `test-before-implementation` (blocking) — the spec's three Failing Tests are
  the contract.
- `one-spec-per-pr` (blocking) — the PR references SPEC-025 only.
- `provenance-recorded-per-algorithm` — no new algorithm (test-only); no row.

### Prior related work

- `SPEC-024` (PR #37/#38) — the CFA gate admitting the Bayer plane.
- `SPEC-028` (PR #41/#43, HANDOFF-066) — multi-strip assembly (the Nikon is 12
  strips). The red-proof uses the Nikon as its reference file so it also
  exercises that assembly inside the staged copy.
- `SPEC-029` (PR #44/#46, HANDOFF-068) — relaxed `SampleExceedsWhiteLevel`;
  handed the byte-swap guard to this oracle.
- `SPEC-027` — registers both Bayer DNGs in `tests/corpus/manifest.toml` + a CFA
  fuzz seed. This build deliberately keeps them direct-path (NOT in the manifest)
  so `compressed_files_are_skipped_by_name`'s count stays correct; SPEC-027 owns
  the migration and may later fold these tests into the manifest-driven oracle.

## Expected Deliverables

- `tests/plane_oracle.rs`: the two pinned checksum constants, the two Bayer plane
  oracles, the byte-aligned red-proof + its control. No `src/` change, no manifest
  change.
- The spec's three Failing Tests passing; the mono oracle (4 `DECODABLE` files)
  and its sub-byte red-proof byte-identical in behaviour.
- All gates green locally — `just test`, `just lint-ci` (`-D warnings`),
  `cargo fmt --check`, `just validate`.
- A PR against `main` referencing PROJ-002 / STAGE-006 / SPEC-025 / HANDOFF-070 /
  DEC-029 / DEC-017 / DEC-008.

## Out of Scope

- Any demosaic or mosaic → RGB interpolation — STAGE-007.
- Parsing or checking colour metadata — SPEC-026.
- Registering the Bayer DNGs in `tests/corpus/manifest.toml` + a CFA fuzz seed —
  SPEC-027.
- Changing the mono plane oracle (SPEC-013) beyond the shared-apparatus refactor.
- Tolerance-based comparison — this layer stays bit-exact.

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

- **Branch / PR:** `claude/spec-025-plane-oracle-1bdbe1` → *(PR link added in the PR-ref bookkeeping commit)*
- **Completed at:** 2026-10-30
- **All acceptance criteria met?** Yes — every AC in SPEC-025.
  - **AC1** both Bayer planes' MD5 == `dnglab analyze --raw-checksum`
    (`bayer_plane_md5_matches_dnglab_raw_checksum`): Canon single-strip 5920×3950
    = `a18235d6abced4df9948b6fbe91aba7c`, Nikon 12-strip 4352×2868 =
    `b8e8025d370cc2e97f675e63a0e6bbdb`. The first bit-exact proof the Bayer mosaic
    decodes.
  - **AC2** a byte-order swap in `unpack_byte_aligned` turns the Nikon plane red
    (`a_byte_order_swap_turns_the_bayer_oracle_red`: honest
    `b8e8025d…` ≠ mutant `fdcb897fbae2bd9d180e05f9217f5011`); the unmutated
    control reproduces the pinned checksum (`the_bayer_red_proof_control_is_green`).
  - **AC3** the mono oracle (4 `DECODABLE` files) and its sub-byte red-proof stay
    green — `plane_oracle` is 15/15 with corpus; the mono red-proof
    (`an_injected_unpacker_fault_turns_the_oracle_red`) still reddens.
  - **AC4** no corpus → the Canon + Nikon oracle and the red-proof skip loudly
    (SKIP lines printed); the tier-A tests still run (15 passed, 1.4s, no release
    build) — verified with `$IRRADIANCE_CORPUS_DIR` unset.
  - **AC5** no `src/` change and no manifest change — `git diff --name-only`
    shows only `tests/plane_oracle.rs` (plus this cycle's spec/handoff bookkeeping).

### Cost self-report

- **Tokens (total):** 15781793
- **Estimated USD:** 46.15
- **Duration (minutes):** ~18
- **Source of the number:** harness report — summed from this session's own
  Claude Code transcript JSONL (`input + output + cache_creation + cache_read`
  across all usage records; this desktop Code interface exposes no `/cost` slash
  command), the all-inclusive convention SPEC-024/028/029 build used.
  `estimated_usd` at Opus list rates — an order-of-magnitude, cache-read-dominated
  figure.

### Drift and new artifacts

- **New decisions emitted:** none — test-only, no algorithm or policy change.
- **Deviations from spec:**
  1. **`stage_probe_crate`'s signature changed from `mutate: bool` to
     `inject: Option<fn(&Path)>`.** The spec said "reuse `stage_probe_crate` /
     `build_and_run_probe`"; the mono apparatus hard-coded the sub-byte injector.
     Generalising it to take the injector is the cleanest reuse — the mono and
     Bayer faults now share one staging function — and the four existing mono
     call sites (`true`→`Some(inject_chunk_extraction_fault)`, `false`→`None`)
     keep byte-identical behaviour (the mono red-proof + control still pass). No
     `src/` or oracle-method change; purely the test harness.
  2. **Tests added to `tests/plane_oracle.rs`, not a sibling
     `tests/plane_oracle_bayer.rs`.** The spec allowed either; the oracle helpers
     (`decode_and_hash`, `stage_probe_crate`, `PROBE_MAIN`, …) are module-private,
     so a sibling file would have had to duplicate or `pub`-expose them. Same file
     keeps the reuse free and the migration SPEC-027 wants easy.
- **Follow-up work identified:** none ship-blocking. SPEC-026 (colour metadata)
  and SPEC-027 (manifest + CFA fuzz seed) are the already-framed next specs; with
  this green the Bayer mosaic decode is bit-exact-verified end to end.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The spec named the exact injection lines
   (`src/plane.rs:210–212`), the reference file (Nikon, for multi-strip), and the
   direct-path pattern to copy. The one judgement call — how to let the mono and
   Bayer faults share `stage_probe_crate` — the spec implicitly answered by saying
   "only the inject fn and the reference file are new", i.e. the staging function
   should be parameterised, not duplicated.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-029 is the load-bearing one (it is why this red-proof must exist at
   all), and the spec's Notes call it out explicitly. Pinning the two checksums
   against real bytes with `dnglab 0.7.2` first made AC1 concrete rather than
   assumed.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The byte-order swap goes through a placeholder token so
   the two `replacen` calls can't collide — a small robustness touch worth keeping
   over a brittle two-line whitespace match. Confirming both files are 16-bit (so
   they take the byte-aligned path the sub-byte mono red-proof never exercises) is
   the sentence that justifies this as a genuinely new guard, not a duplicate.
