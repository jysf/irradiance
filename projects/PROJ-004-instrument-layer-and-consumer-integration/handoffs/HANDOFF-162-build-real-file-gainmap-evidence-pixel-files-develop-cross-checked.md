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
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
