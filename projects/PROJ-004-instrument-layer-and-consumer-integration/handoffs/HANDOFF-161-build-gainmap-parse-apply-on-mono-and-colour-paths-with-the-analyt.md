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
  id: HANDOFF-161
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-19
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-059

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

# HANDOFF-161: BUILD: GainMap parse and apply on both develop paths (SPEC-059)

## Delegation Summary

You are the **builder** for SPEC-059. Cut `claude/spec-059-build` from `origin/main`
after the STAGE-016 design PR merges.

**The task:** implement DEC-069.
- `GainMap` (OpcodeID 9) from `OpcodeList2` is applied on the mono and colour develop
  paths at stage 2, by the exact rule in DEC-069 §2.
- The required-processing check accepts every `OpcodeList2` `GainMap` on both paths.
- A new public `apply_gain_maps` stage primitive is added.
- `DEVELOP_VERSION` goes to `"6"`.
- The rule is proven bit-exact against a hand-written oracle on writer-built fixtures,
  with a red-proof for each part of it.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-059-gainmap-parse-apply-on-mono-and-colour-paths-with-the-analyt.md`.
  Read all of it: Design §1–§8, AC1–AC9 and the Notes.
- **Decisions:**
  - `DEC-069`, all of it. This is the contract: the rule, the malformed classes, the
    16-map cap, one implementation with two callers, and the version.
  - `DEC-049` and `DEC-053`: required processing and stage primitives.
  - `DEC-058` §5: version.
  - `DEC-060`: the mono scratch develop allocates nothing.
  - `DEC-061` and `DEC-068`: black levels.
  - `DEC-040`: the `ActiveArea` origin.
  - `DEC-012`.
  - `DEC-009`: red-proof, control first.
- **Code:**
  - `src/opcode.rs`: the framing reader, `check_list`, `DevelopPath::applied_id`, the
    `Opcode` enum, `parse_opcode_list` and `__fuzz_required_processing`'s caller in
    `src/lib.rs`.
  - `src/develop.rs`: `crop_orient_normalize_into`, `normalize_active_area_into`,
    `resolve_levels_with`, `develop_core`, `develop_with_scratch_into` and
    `apply_fix_bad_pixels_constant`.
  - `src/color.rs`: the four entry points, `develop_color_bands`, `slot_normalizer` and
    `pattern_normalizer`.
- **Fixtures:** `tests/support/dng_writer.rs` `extra_tags` writes `OpcodeList2` (tag
  51009, UNDEFINED) with no writer change.
- **Facts from the design probe**, so you don't re-derive them:
  - The Pixel 3a, 4a and 7 Pro carry four mandatory 30×40 maps each. Their params are in
    DEC-069 §Context and STAGE-016's Design Notes.
  - `GainMap` is their only blocker on both paths.
  - The rule was checked against Adobe `dng_validate`. You do **not** need the SDK for
    this spec; SPEC-060 uses it.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - `just test` already passes `--all-features`.
  - Give every red-proof copy its own `CARGO_TARGET_DIR`, and read each panic message.
    Red must come from the target assertion.

## Expected Deliverables

1. **Commits:**
   1. the parser and the `GainMaps` type, with AC1;
   2. the oracle and `apply_gain_maps`, with AC2 and AC3;
   3. mono and colour wiring, with AC4 and AC5;
   4. required processing, with AC6;
   5. the version, CHANGELOG, provenance row and fuzz.

   Combine commits where that keeps each one green.
2. **Evidence for each AC** in the handback:
   - every AC8 mutant, with the failing test and its message;
   - the AC7 corpus `just test` summary;
   - the fuzz run counts;
   - any public-shape choices (`Opcode::GainMap`, the wrong-length error).
3. **Gates (AC9),** all green with the corpus env set:
   - `just test`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`;
   - `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`;
   - `name-length-check`, `semver-check`, `decisions-index --check`.
4. **PR** against `main` from `claude/spec-059-build`, referencing SPEC-059, HANDOFF-161
   and DEC-069, with CI green on its head. The macOS determinism leg is non-blocking.

## Out of Scope

- The Pixel tier-B tests, pins and the SDK cross-check (SPEC-060).
- The max-black normalisation difference.
- Other `OpcodeList2` opcodes, and `GainMap` in lists 1 and 3.
- Colour `WarpRectilinear`.
- A release.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003), or any Adobe SDK file or output.
- **If an existing test or oracle goes red, the change is wrong.** The only permitted
  edits to existing tests are:
  - the `DEVELOP_VERSION` pins (`"5"` to `"6"`; grep for them);
  - new rows in `tests/required_processing.rs` and `tests/develop_scratch_alloc.rs`.

  If a fixture is now refused by design, stop and ask.
- **Fill the `handback:` front matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section below before reporting done.

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
