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
  id: HANDOFF-175
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-21
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-061

project:
  id: PROJ-005
  stage: STAGE-017
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 21952535           # REAL combined count — what cost-audit reads
  estimated_usd: 16.56             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 30
  branch: claude/spec-061-build
  pr: 329
  completed_at: 2026-11-24         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript d2a634a0-951d-47e7-93f0-1c1293abce88.jsonl (87 unique message.ids, all claude-opus-5-5, no subagents): input 177 + cache-creation 295,078 (1-hour) + cache-read 21,543,887 + output 113,393. Re-sum at ship. dng_validate: 12 of 12 outputs valid, no warnings." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-24
---

# HANDOFF-175: Build SPEC-061, the public `write` feature's mono u16 DNG writer core

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-061` to `claude-opus-5` (implementer) for the
**build** cycle: implement `irradiance::dng::write` behind the `write` feature so the
spec's failing tests pass, on a branch cut from `main`.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-061-public-write-feature-sealed-api-over-the-stage-013-core-call.md` — read Design §1 to §9 and the probe table in
  `## Implementation Context`. The API names in §2 are binding.
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
- **Project brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` (the `+toolchain` traps for fuzz,
  lint-ci and msrv).
- **Starting code:** `tests/support/dng_writer.rs` (SPEC-057's own-work writer: layout,
  inline rule, determinism) and `tests/dng_writer.rs` (oracle helpers).
- **Corpus:** set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
- **Adobe `dng_validate`:** set
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
  for AC9. Local only; never in CI; never commit its outputs.

### Decisions that apply

- `DEC-071` (accepted) — v1 scope: mono u16 unchanged, LE uncompressed strips, structural
  tags the writer's alone, `UniqueCameraModel` required, size query + caller buffer.
- `DEC-073` (accepted) — framing and orientation as tags; samples cut only at `ActiveArea`.
- `DEC-075` (proposed, this design) — layout (raw plane in IFD0, one strip), versions,
  `Orientation` always written, the `OpcodeList1` allow-list walk, masked areas only under
  FullSensor, two `Error` variants, the budget as an argument, no allocation.
- `DEC-072` (accepted) — only reserve the layout slot (IFD0 offset in the header is not
  hard-coded to the header's end); the EXIF block itself is SPEC-081.
- `DEC-074` (proposed) — rendering hints; **not built here** (SPEC-081).
- `DEC-055` — `src/dng/write.rs` behind a feature; container types only.
- `DEC-050` / `DEC-016` / `DEC-051` / `DEC-052` / `DEC-058` — the caller's pixel budget,
  caller-owned buffers, the wildcard-free `kind()` match, `#[non_exhaustive]` + `Default`.
- `DEC-069` §6 — `dng_validate` is a local, black-box reference only.

### Constraints that apply

- `no-panics-on-untrusted-input` — the crate-root lint policy applies to `src/dng`;
  `just lint-no-allow` must pass.
- `oracle-must-be-shown-red` — oracles 3 and 5, negative control first.
- `provenance-recorded-per-algorithm` — the ledger row in Design §9.
- `no-copyleft-dependencies` — no new dependency at all.
- `library-not-application` — no I/O in `src/dng`.

### Prior related work

- `HANDOFF-127` / SPEC-057 — the test-only writer this generalises.
- PR 284 and PR 291 — the STAGE-017 frame and the DEC-072 amendment.

## Expected Deliverables

- Every file in SPEC-061's `## Outputs`, and every test in `## Failing Tests` passing.
- `just test` (with the corpus set and `IRRADIANCE_DNG_VALIDATE` set), `just lint-ci`,
  `just lint-no-allow`, `just fmt-check`, `just doc-check`, `just deny`, `just deny-fuzz`,
  `just lint-fuzz`, `just msrv`, `just validate`, `just gate-audit`, `just cost-audit`,
  and 60 s of `just fuzz-dng-write`, all green; `cargo semver-checks` additions only.
- A PR against `main` from `claude/spec-061-build`, naming HANDOFF-175, SPEC-061,
  STAGE-017, PROJ-005, DEC-071/072/073/074/075 and any new DEC. Commit messages plain
  prose (no code tokens).
- In the handback: the `dng_validate` result per output (warnings, render equality),
  and the token count.

## Out of Scope

- EXIF block, XMP, descriptive tags other than `UniqueCameraModel`, rendering hints
  (SPEC-081); previews (SPEC-082).
- Compression, float, CFA, tiles, big-endian, several strips.
- Any change to the reader (`src/ifd.rs`, `src/plane.rs`, `src/develop.rs`). If a round
  trip exposes a reader bug, stop and report it.
- Migrating `tests/support/dng_writer.rs` onto the public writer.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-061-build` → https://github.com/jysf/irradiance/pull/329
- **Completed at:** 2026-11-24
- **All acceptance criteria met?** Yes for AC1 to AC12, with one oracle adapted (deviation 1 below).
- **Staleness check (designed 2026-11-21; main has since gained 0.2.5, 0.2.6 and SPEC-089):** it holds. The writer emits only well-formed counts (Orientation 1, BlackLevel matching the repeat dimensions, versions as 4 BYTEs). The backward version is at most 1.3, under DEC-077's 1.7.1 guard. Every round trip reads back with an empty `malformed_tags`. Oracle 4 is exact on all four held files, and the dng_validate numbers reproduce the probe table exactly (132,753,649 differing samples for the forbidden shape). No reader change was needed.
- **Gates:** `just test` with the corpus and `IRRADIANCE_DNG_VALIDATE` set: 900 passed, 0 failed, 55 binaries. `lint-ci`, `lint-no-allow`, `fmt-check`, `doc-check`, `deny`, `deny-fuzz`, `lint-fuzz`, `msrv`, `validate`, `gate-audit` and `cost-audit` are all green. `cargo semver-checks --all-features`: no semver update required (202 pass). `cargo check --no-default-features` and `--features write` both build. `just fuzz-dng-write 60`: 3,612,299 runs, no crash.
- **dng_validate 1.7.1 (AC9):**

  | Output | Result |
  |---|---|
  | Q2M `L1021223` FullActiveArea | valid, no warnings |
  | Q2M StandardCrop | valid, no warnings; `-16 -tif` render equals the source's (8368×5584, 140,180,736 samples, 0 differ) |
  | Q2M FullSensor | valid, no warnings; render equals the FullActiveArea output's (8392×5632, 141,791,232 samples, 0 differ) |
  | Q2M crop cut with the warp verbatim (DEC-073's forbidden shape) | render differs in 132,753,649 of 140,180,736 samples |
  | K-3 III, M Monochrom, M Monochrom Typ 246 × 3 framings | valid, no warnings (9 of 9) |

- **dnglab / exiftool (AC8):** on all four files, the FullSensor `--raw-checksum` equals dnglab's checksum of the source file, and the cut outputs equal the MD5 of the active-area plane. exiftool reads DNGVersion, UniqueCameraModel, BitsPerSample, ImageWidth, ActiveArea, DefaultCropSize and Orientation back as written at every framing.

### Cost self-report

- **Tokens (total):** 21,952,535 (a floor, taken before the handback commit)
- **Estimated USD:** 16.56 (claude-opus-5-5: $5/M input, $25/M output, cache read at 0.1×, 1-hour cache writes at 2×)
- **Duration (minutes):** about 30
- **Source of the number:** this session's transcript `d2a634a0-951d-47e7-93f0-1c1293abce88.jsonl`, found by the scratchpad UUID and measured per docs/cost-tracking.md "Measuring a cycle": 87 unique message.ids, all claude-opus-5-5, no subagents, deduplicated by message.id with each message's largest output_tokens (`basis: transcript`).

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-080` (proposed): the writer's error payload types are compiled in every build and named under `dng::write`. The spec requires both `Error` variants in every build, but names their payload types under a module AC1 compiles only with the feature.
- **Deviations from spec:**
  1. **Oracle 8, the FullSensor render.** §7 item 8 asks that the Q2M FullSensor render equal the *source's* render. §3's table writes FullSensor's DefaultCrop as (0,0) plus the active-area size, so its render is 8392×5632 against the source's 8368×5584 and cannot be equal. (The design probe wrote the source crop at FullSensor.) The test instead requires the FullSensor render to equal the FullActiveArea output's render, sample for sample: the margin changes nothing. If FullSensor should carry the caller's crop, that is a §3/DEC-073 change for the architect.
  2. `tests/error_kind.rs`'s variant parser now reads a tuple variant, because `InvalidWriteInput(WriteInputError)` is the first tuple variant in `Error`. Without this change it would silently miss the variant.
  3. The self dev-dependency now also enables `write`, as it already does `test-fixtures`, so a bare `cargo test` compiles the writer tests.
  4. Added two support files that the Outputs list does not name: `tests/support/dng_write_oracle.rs` (oracle 1, shared by both tiers) and `tests/support/dng_write_fuzz.rs` (the fuzz input format, shared by the target and the seed generator). Also added `fuzz_seeds_reach_their_named_outcome`.
  5. `OutputTooLarge` is reached in tier A with a 4 GiB zeroed `OpcodeList2`. The OS maps it lazily and planning never reads it, so the test costs no real memory.
- **Follow-up work identified:**
  - **FixBadPixelsConstant at the active-area edge.** `apply_fix_bad_pixels_constant` takes neighbours from the plane extent (SPEC-017 rule 1), so a bad pixel on the active-area edge borrows margin neighbours in the source but not after a cut. A cut file can then develop differently at that pixel. DEC-075 §5 calls id 4 position-independent, which is true of its parameters but not quite of its effect. The held files are unaffected (oracle 4 is exact on all four). It needs an architect call: accept it and document it in DEC-075, or refuse/flag such files.
  - `Sensor.white_level` absent on a 12/14-bit source: the caller must pass `2^bits − 1` explicitly, because the written file is 16-bit (the tests' `options_from_sensor` does this). This is worth a sentence in the rustdoc or onyx's integration notes.

### Round 2 (punch list P1)

- **Commit:** `15ed4fc` on `claude/spec-061-build`, fast-forwarded first onto the verify commits (`41d062b`, HANDOFF-190).
- **Fix:** `scripts/lint-fuzz-red-proof.sh` now also copies `tests/support/dng_write_fuzz.rs` into its temp dir, because `fuzz_targets/dng_write.rs` includes it by path, so the control build compiles. `just lint-fuzz-red-proof`, `just fmt-check` and `just gate-audit` are green locally. PR 329's CI is 22 of 22 passing, including `rust / fuzz lint red-proof (must fail red)`.
- **Tokens for this round:** 3,583,551 (about $1.95), a floor from the same transcript (`d2a634a0-…`), counting only messages after the round-2 request (10 unique message.ids, claude-opus-5-5, `basis: transcript`). The front-matter numbers above are round 1 only and are left unchanged.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two places where the spec contradicts itself: oracle 8's "FullSensor render equals the source's" against §3's FullSensor crop, and "Error variants exist in every build" against payload types named under a feature-gated module. Both were resolvable, but neither was flagged at design.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-017/SPEC-017's neighbour rule for FixBadPixelsConstant (plane extent, not active area) bears directly on DEC-075 §5's allow-list claim. Also, `tests/error_kind.rs`'s text parser accepts only struct and unit variants, a trap for any spec that adds a tuple variant.

3. **If you did this task again, what would you do differently?**
   — Run the dng_validate render comparison on the very first tier-B pass. It surfaced the oracle-8 contradiction in seconds, and checking it earlier would have let me flag it before writing the rest of the oracle.
