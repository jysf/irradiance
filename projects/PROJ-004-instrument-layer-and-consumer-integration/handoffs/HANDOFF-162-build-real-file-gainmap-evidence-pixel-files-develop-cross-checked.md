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
  id: HANDOFF-162
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-19
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-060

project:
  id: PROJ-004
  stage: STAGE-016
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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 13281980           # REAL combined count — what cost-audit reads
  estimated_usd: 9.76              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 19
  branch: claude/spec-060-build
  pr: 269
  completed_at: 2026-11-19         # YYYY-MM-DD
  notes: "LIVE FLOOR, re-sum at ship. Transcript 0c21a153-2805-44db-a24f-743b9eccaa3f.jsonl (found by the scratchpad UUID), 75 unique message.ids, all claude-opus-5-5, no subagents: input 159 + cache-creation 170,325 (all 1-hour) + cache-read 13,050,255 + output 61,241 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour write x2.0). Duration is the transcript span at the re-sum, before the handback commit and CI." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-162: BUILD: Real-file GainMap evidence and the dng_validate cross-check (SPEC-060)

## Delegation Summary

You are the **builder** for SPEC-060. **Start only after SPEC-059's build PR has
merged.** Cut `claude/spec-060-build` from that `origin/main`.

**The task:**
- Show that the three Pixel DNGs develop on every entry, and pin them.
- Cross-check the `GainMap` step against Adobe `dng_validate`'s stage 2, within the
  tolerances SPEC-060 pre-registers.
- Commit a reproducible recipe for the SDK outputs. No SDK bytes are committed.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-060-real-file-gainmap-evidence-pixel-files-develop-cross-checked.md`.
  Read all of it: Design §1–§5, AC1–AC7 and the pre-registered bounds.
- **Decisions:** `DEC-069`, especially §6 and §Context's table. `DEC-068`, for why the
  7 Pro's no-gain differences are negative. `DEC-003` (the corpus). `DEC-009`.
- **The SDK:**
  - It is already built locally:
    `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  - Its build script is `build/build.sh`, with the JPEG XL stub in `build/jxl_stubs.cpp`.
  - Sources are under `src/dng_sdk_1_7_1/`.
  - Write SDK outputs under that `tools/dng_sdk/` tree or your scratchpad, never in the
    repo.
- **Code:**
  - SPEC-059's `apply_gain_maps` and develop wiring, as merged;
  - `tests/metadata_oracle.rs` (the Pixel paths);
  - the corpus skip machinery in `tests/support/corpus.rs`;
  - the existing develop-output pin files.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every red-proof copy its own `CARGO_TARGET_DIR`.

## Expected Deliverables

1. **Commits:**
   1. the recipe and the TIFF reader;
   2. the tier-B develop tests and pins;
   3. the env-gated SDK cross-check;
   4. the AGENTS.md line and the ledger evidence.
2. **Evidence in the handback.** For each Pixel and each of the three comparisons (the
   gain step, end to end, the no-gain root cause): max |Δ|, the fraction exact and the
   histogram. Also the AC6 mutant's numbers. If any measured number falls outside its
   pre-registered bound, **stop and report**. Do not widen it.
3. **Gates:**
   - `just test` (with the corpus, and once more with `IRRADIANCE_SDK_STAGE2_DIR` set);
   - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
   - `cost-audit`, `gate-audit`, `name-length-check`.
4. **PR** against `main` from `claude/spec-060-build`, referencing SPEC-060 and
   HANDOFF-162, with CI green. The SDK checks skip visibly in CI.

## Out of Scope

- Changing `src/` behaviour, including the max-black normalisation.
- Running the SDK in CI, vendoring it, or committing its outputs.
- The Samsung file.
- A colour comparison with SDK stage 3.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- Existing assertions may be strengthened, never weakened.
- **Fill the `handback:` front matter** with a real `tokens_total` re-summed from your
  own transcript (`basis: transcript`, found by your scratchpad UUID, deduped by
  `message.id`, opus-5-5 rates), and fill the `## Handback` section below.

## Handback` section below.** An unfilled handback is an
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

- **Branch / PR:** `claude/spec-060-build`, https://github.com/jysf/irradiance/pull/269
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes. AC2 was amended at build, with the orchestrator's
  approval (see Deviations). Every pre-registered bound held without widening.

**SDK cross-check.** `scripts/sdk-stage2.sh` was run with the local `dng_validate` 1.7.1.
Differences are signed `SDK − ours`; "exact" is the fraction with no difference.

| File | Comparison | max abs diff | exact | histogram |
|---|---|---|---|---|
| 4a | gain step (gate ≤ 1, ≥ 99.9%) | 1 | 99.9794% | {-1: 350, 0: 12,190,254, 1: 2,164} |
| 3a | gain step | 1 | 99.9672% | {-1: 567, 0: 12,188,768, 1: 3,433} |
| 7 Pro | gain step | 1 | 99.9664% | {-1: 565, 0: 11,347,156, 1: 3,247} |
| 4a | end to end (≤ 8) | 7 (min -1) | 94.3870% | {-1: 255, 0: 11,508,394, 1: 193,528, 2: 360,321, 3: 88,500, 4: 31,604, 5: 8,584, 6: 1,568, 7: 14} |
| 3a | end to end | 6 (min -1) | 89.4246% | {-1: 460, 0: 10,903,338, 1: 607,465, 2: 598,539, 3: 68,713, 4: 9,211, 5: 5,013, 6: 29} |
| 7 Pro | end to end | 4 (min -4, max 2) | 28.3394% | {-4: 25, -3: 413,204, -2: 1,910,314, -1: 5,809,105, 0: 3,216,792, 1: 1,458, 2: 70} |
| 4a | no gain (≤ 5, all ≥ 0) | 5, min 0 | 93.1660% | {0: 11,359,509, 1: 614,842, 2: 85,849, 3: 54,562, 4: 66,198, 5: 11,808} |
| 3a | no gain (≤ 5, all ≥ 0) | 5, min 0 | 89.3783% | {0: 10,897,683, 1: 1,243,288, 2: 32,222, 3: 15,783, 4: 3,321, 5: 471} |
| 7 Pro | no gain (≤ 5) | 2 (min -2, max 1) | 28.3471% | {-2: 1,228,430, -1: 6,904,199, 0: 3,217,671, 1: 668} |

**AC6 red-proof.** It ran control first, on a scratch copy with its own `CARGO_TARGET_DIR`.
- **Control:** the unmutated copy passes the gate with the numbers above.
- **Mutant:** both gain-index lines in `src/opcode.rs` drop `+ 0.5`, so the map is sampled
  at pixel corners. The gate goes red on the 4a first.
- **The mutant's numbers on all three files** (assertion disabled to measure them):

  | File | max abs diff | range | exact |
  |---|---|---|---|
  | 4a | 32 | -32 to 16 | 63.83% |
  | 3a | 20 | -20 to 8 | 37.40% |
  | 7 Pro | 13 | -13 to 5 | 70.55% |

**AC2, as amended.** Measured on CFA phase (0, 0) with 128-pixel windows.
- The ratio is the mean of the four corner windows over the centre window.
- The develop is mono `FullActiveArea`, unoriented, `warp_alpha` 0, `PerPosition`.

| File | Map corner gain | Map centre gain | Ratio without GainMap | Ratio with GainMap |
|---|---|---|---|---|
| 4a | 3.2939 | 1.0177 | 3.1962 | 9.3125 |
| 3a | 2.7222 | 1.0401 | 0.2190 | 0.5454 |
| 7 Pro | 1.4457 | 1.0192 | 0.2075 | 0.2843 |

**AC3.** Fifteen MD5 pins are in `tests/gain_map_corpus.rs`, five per file:
- mono default;
- mono `StandardCrop` `PerPosition`;
- mono `FullActiveArea` `PerPosition` at `warp_alpha` 0;
- colour u16;
- colour f32.

No other pin changed, and the full suite passes. On the 7 Pro the mono default and
`PerPosition` hashes are equal, which suggests its four `BlackLevel` values are uniform.

**Gates.**
- `just test` passes twice with `IRRADIANCE_CORPUS_DIR` set and all 12 corpus files
  present: once without `IRRADIANCE_SDK_STAGE2_DIR` (corpus-status prints the
  `sdk-stage2: SKIP` line) and once with it.
- `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`,
  `gate-audit` and `name-length-check` are green.
- `lint-ci` first caught two clippy style findings in the new reader. Both are fixed.

### Cost self-report

- **Tokens (total):** see the `handback:` front matter
- **Estimated USD:** see the front matter
- **Duration (minutes):** see the front matter
- **Source of the number:** this session's transcript (`basis: transcript`), found by its
  scratchpad UUID, deduplicated by `message.id`, with the largest `output_tokens` per
  message, priced at opus-5-5 list rates.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **AC2 amended (orchestrator-approved).** "Toward 1" assumed a scene with plain
    vignetting. The 4a's corners are brighter than its centre (3.196 → 9.313), so lifting
    them moves the ratio away from 1. AC2 now requires:
    1. the output differs from the no-gain develop;
    2. the ratio with the GainMap exceeds the ratio without it, on every file;
    3. the phase-(0,0) map, parsed by hand from `OpcodeList2`, has a corner gain greater
       than its centre gain, which anchors (2) to the map;
    4. the ratio moves toward 1 only where it starts below 1 (the 3a and 7 Pro).

    The spec's AC2 and Design §3 were edited on this branch to match.
  - **AC1's `FullActiveArea` entry runs at `warp_alpha` 0 (orchestrator-approved).**
    - At the default 1.0, DEC-059 §3 refuses the 3a with `UnsupportedWarpedFullActiveArea`.
      The 4a and 7 Pro develop `Ok`.
    - The approval asked for "refused at 1.0", but only the 3a is refused. So the test
      asserts each file's measured outcome exactly (`FULL_WARP_REFUSED`).
    - A build note was added to Design §3.
  - **The SDK SKIP line goes in `examples/corpus-status.rs`.** The spec's Outputs did not
    list that file. It is the surface `just test` shows, because `cargo test` captures a
    passing test's `eprintln!`.
- **Follow-up work identified:** none new. The max-black normalisation follow-up
  (DEC-069 §Context) stands, and the no-gain numbers above quantify it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two items hadn't been measured on the real files at design time:
   - AC2's "toward 1" assumed the scene, and the 4a breaks it.
   - "`FullActiveArea` returns `Ok`" didn't account for DEC-059's warp refusal, which
     applies to the 3a.

   Both were one-command measurements.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-059 (the `FullActiveArea` warp refusal). It decides whether AC1's
   `FullActiveArea` entry can be `Ok` at all.
3. **If you did this task again, what would you do differently?**
   — Print every behavioural ratio on all three files before writing an assertion. That
   would have raised the AC2 question in the first minute, not after the test was written.
