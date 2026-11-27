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
  id: HANDOFF-207
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-27
  status: completed                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-097

project:
  id: PROJ-006
  stage: STAGE-034
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 15174129               # REAL combined count — what cost-audit reads
  estimated_usd: 10.18              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 50
  branch: claude/spec-097-verify
  pr: https://github.com/jysf/irradiance/pull/363
  completed_at: 2026-11-27               # YYYY-MM-DD
  notes: "Floor, measured live from transcript f32aa39b-afea-4a0a-a3c6-79ec81471dc7.jsonl, deduplicated by message.id; re-sum at ship. USD at Opus 5.5 rates (5/25 per M, cache reads 0.5, 1-hour cache writes 2x)."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-27
---

# HANDOFF-207: VERIFY: accept a lossless-JPEG precision below BitsPerSample, the Leica M10 file (SPEC-097)

## Delegation Summary

The PROJ-006 orchestrator (architect) hands SPEC-097 to a verifier for the **verify**
cycle. Review it cold: the verifier did not build it.

- **Under review:** PR 362 (`feat/spec-097-ljpeg-precision-below-bps`). The code is
  commit `4a02aa0`; the later `903fd2d` is the build's handback (docs only). CI run
  46022034252 on `903fd2d` was green, 25 checks. The build ran under HANDOFF-206.
- **Verify branch:** `claude/spec-097-verify`, cut from `903fd2d`. The build created no
  verify handoff or prompt, so the verifier created this one with `just new-handoff`.

SPEC-097 relaxes `check_precision` in `src/plane.rs` from `P = BitsPerSample` to
`P ≤ BitsPerSample ≤ 16` (DEC-085, amending DEC-077 §3), so the colour Leica M10 (CC0) and
M11 (local only, no licence) unpack and reach every colour and mixer entry.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-097-accept-a-lossless-jpeg-precision-below-bitspersample-the-lei.md`,
  judged **as designed** on `origin/main` (PR 361): Design §1 to §7, AC1 to AC8, mutants
  (a) to (i).
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md`.
- **Build handback:** HANDOFF-206 `## Handback` (mutant table, deviations).

### Decisions that apply

DEC-085 (the rule), DEC-077 §3 and its amendment note, DEC-029 (samples above
`WhiteLevel`), DEC-047 (the shape binding that runs first), DEC-061 (the white default),
DEC-003 (tier B, no corpus in CI).

### Constraints that apply

`no-panics-on-untrusted-input`, `oracle-must-be-shown-red`, `test-before-implementation`,
`provenance-recorded-per-algorithm`, `one-spec-per-pr`, `cost-captured-per-cycle`.

### Prior related work

HANDOFF-198, HANDOFF-203 and HANDOFF-205 (the PROJ-006 verifies): each found test-oracle
gaps, not code bugs, so look hard at the oracles.

## Expected Deliverables

- **Exactly one verdict:** APPROVED (at a SHA), PUNCH LIST, or REJECTED.
- Evidence for every AC, re-running the gates (`just test` with and without the corpus,
  `lint-ci`, `lint-fuzz`, `fmt-check`, `doc-check`, `validate`, `cost-audit`,
  `semver-check`).
- AC7's digests and the seven preview pins compared with the spec's registered values by
  script.
- The boundaries: P = BitsPerSample, P = BitsPerSample + 1, BitsPerSample 16 and 17, P = 2;
  refusal before any sample write on every Compression 7 path; white from BitsPerSample.
- Mutants: a sample of (a) to (i), plus the verifier's own.
- The licence rule: no M11 bytes or derived seed in the diff; every tier-B test skips
  cleanly with no corpus.
- Rulings on the build's deviations.
- Fixes on the verify branch are tests or docs only; `src/` changes go in a punch list.

## Out of Scope

- The STAGE-034 opcode spec (Q2, Q3, SL3 mandatory opcodes on the colour path).
- Highlight recovery above `WhiteLevel` (onyx's open question).
- Any change to `DEVELOP_VERSION`, `MONO_MIX_VERSION`, an `Error` variant or an existing
  hash. If one seems necessary, that is a rejection.
- Merging PR 362 or starting the ship.

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

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-097-verify`, cut from PR 362's head `903fd2d`. Verify PR: https://github.com/jysf/irradiance/pull/363. Not merged.
- **Completed at:** 2026-11-27
- **All acceptance criteria met?** Yes, each re-run here (below).
- **For `verify`:** APPROVED at `4a02aa0` (the code; `903fd2d` adds only the build handback). The verify commit adds tests and docs only; `src/` is untouched.

### Results, per AC

- **AC1:** green on (2,16), (8,16), (12,16), (14,16), (15,16), (8,12), (12,14), both orders, strips and tiles, all three paths, exact equality. P = 2 is reached. **Added here:** `ac1_two_component_precision_below_bits_per_sample_decodes_unchanged`, the same grid at Nf = 2 (the Leica layout) for (14,16), (12,16), (8,12).
- **AC2:** green on (16,12), (14,12), (13,12), (16,14), (16,15): exact variant and fields, `ErrorKind::Corrupt`, sentinel `dst` untouched on every cell. P = BitsPerSample + 1 is (13,12) and (16,15).
- **AC3:** green on (14,17), (14,20), (16,17), (16,32), (8,32), the same way. BitsPerSample 16 is accepted (AC1, `ac5_matching_…` P16) and 17 refused.
- **AC4:** green. **Added here:** `ac4_white_default_is_bits_per_sample_on_mono_and_colour`: mono P8 in BitsPerSample 12 analytically (white 4095, so 255 and a fixed 65535 both fail), and the colour path (`develop_color_into`) at P12 in 16 against the tag (no `WhiteLevel` equals an explicit 65535 and differs from 4095). AC4 as built reached only mono at BitsPerSample 16.
- **AC5:** `plane_md5_matches_the_pinned_raw_checksum` green on both. Red-proof observed here with `--nocapture`: M10 `a1c6050a…` to mutant `90a70468a8070a869b565740adb55879`, M11 `9ae8c096…` to `8d262e716b4824d3c72f4a75c41f32b1` (both equal the build's).
- **AC6:** `just test` with the corpus (14 of 14 present): 993 passed, 0 failed, 1 ignored, equal to the build. `DEVELOP_VERSION` "8" and `MONO_MIX_VERSION` "1" untouched (no diff in `src/` beyond `plane.rs` and `lib.rs` docs). `just semver-check`: 202 pass, no semver update required.
- **AC7:** compared by script with the spec on `origin/main`: 8 FNV-1a digests, 2 developed sizes, 0 mismatches; both `raw_checksum`, `bytes` and `sha256` equal Design §5. M10 facts asserted and green.
- **Preview pins:** 7 of 7 compared by script with Design §5 (index, size, offset, length, kind, compression, photometric), 0 mismatches; `corpus_previews_match_exiftool_pins` green.
- **AC8:** CI run 46022034252, `fuzz smoke — plane`: 21 seeds read (19 + the 2 new), 639,624 runs in 61 s, no crash. `just fuzz-seeds` regenerates every seed byte-identical.
- **CI determinism:** both legs on run 46022034252 green; ubuntu ran 12 of 12 (plane, colour, mono-mix and developed-mix hashes).
- **Gates:** `fmt-check`, `lint-ci`, `lint-fuzz`, `doc-check`, `validate`, `cost-audit` green. `just decisions-audit --changed` names DEC-085 and DEC-004 for the test file; the new mono check is analytic per DEC-004.

### Licence rule (M11)

- The diff adds no DNG: its only binaries are the two seeds (709 and 853 bytes), written by `examples/fuzz-seeds.rs` from synthetic 16 x 16 samples; no "Leica"/"M11" string in either. No `.dng` in any commit since `origin/main`.
- With `IRRADIANCE_CORPUS_DIR` pointed at an empty folder, `just test` gives 994 passed, 0 failed (the extra test is this verify's). With it unset, `leica_colour`, `plane_oracle`, `previews` and `corpus_manifest` all pass by skipping.

### Mutants

All applied by a harness to the honest tree, restored after each (`git status` clean in `src/`). All red.

- Build's, re-run: (a) equality, `ac1_…`, `ac4_…`; (b) `<`, `ac5_matching_…`; (c) no 16 bound, `ac3_…`; (d) no P bound, `ac2_…`; (e) `< 16`, `ac1_…`, `ac4_…`, `ac5_…`; (f) tile call removed, `ac2_…`, `ac3_…`; (g) strip call removed, `ac2_…`, `ac3_…`; (h) mono white `2^(BPS-4)-1`, both `ac4_…`; (i) shift, `ac1_…`, `ac4_…` and the Tier-B plane oracle, red-proof and AC7.
- Verifier's own:
  - limit 17: `ac3_…`; limit 15: `ac1_…`, `ac4_…`, `ac5_…`;
  - refusal moved after the sample write, strip arm and tile arm separately: `ac2_…`, `ac3_…` (the sentinel check);
  - colour white from `2^(BPS-2)-1` and `2^(BPS-4)-1` (white from P on the Leica and the fixture): the new `ac4_…` plus the colour determinism hashes and colour unit tests;
  - white fixed at 65535: the new `ac4_…` and `absent_white_level_defaults_to_the_bit_depth_maximum`;
  - a shared-path decoder fault, the initial prediction off by one on Nf = 2 P = 14 frames: before this verify it was caught **only** by Tier-B pins (dnglab checksum, AC7, the M10 facts), none in CI. It is now also caught by the new Nf = 2 AC1 test, so CI sees it.

### Rulings on the build's deviations

- `MAX_LJPEG_PRECISION` private in `src/plane.rs`: accepted. The spec asked for one named constant; the `ljpeg` parser has only a literal range, and touching the decoder was out of bounds.
- `tests/corpus_manifest.rs` count bumps (12 to 14 entries, 21 to 28 previews): accepted; both are the same facts as the spec's named constants.
- `CFA_CHECKED_ELSEWHERE` in `tests/ifd_reader.rs`: accepted; its partition test requires every manifest entry be named somewhere, and both files are checked by the plane oracle and AC7. The build's handback omitted it from its deviation list (its findings note has it).
- Seeds generated by `examples/fuzz-seeds.rs` rather than the test: accepted, the repo's seed source.

### Cost self-report

- **Tokens (total):** 15,174,129 (input 179, cache creation 165953, cache read 14966563, output 41434), a floor
- **Estimated USD:** about 10.18, Opus 5.5 list rates with 1-hour cache writes
- **Duration (minutes):** about 50
- **Source of the number:** this session's transcript, deduplicated by `message.id` (docs/cost-tracking.md), measured live

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the code. Two oracle gaps, closed by tests on this branch:
  - SPEC-097/FU-1 (closed here): AC1 had no Nf = 2 cell, so an Nf = 2 decode fault at P below BitsPerSample reached CI unseen.
  - SPEC-097/FU-2 (closed here): AC4 reached only the mono path at BitsPerSample 16, where "from BitsPerSample" and "always 65535" agree, and not the colour path.
- **Follow-up work identified:**
  - SPEC-097/FU-3 [XS]: `tests/leica_colour.rs` `continue`s past an absent file and reports only a count (same shape as SPEC-093's AC4 follow-up).
  - SPEC-097/FU-4 [XS]: `fuzz/corpus/plane` must be created by hand in a fresh worktree before `just fuzz-plane` (the build's note); CI already does `mkdir -p`.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the spec. The build made no verify handoff, so this one was created at verify.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-004, which `decisions-audit` names for the test file; it shaped how the new white check is written.

3. **If you did this task again, what would you do differently?**
   — Check the fixture's component count against the real files first: the Leica layout (Nf = 2) was the one thing the Tier-A grid did not cover.
