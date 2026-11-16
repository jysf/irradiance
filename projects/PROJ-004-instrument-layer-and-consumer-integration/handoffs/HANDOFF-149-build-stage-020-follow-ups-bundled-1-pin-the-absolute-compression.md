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
  id: HANDOFF-149
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-16
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-071

project:
  id: PROJ-004
  stage: STAGE-021
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

# HANDOFF-149: BUILD: STAGE-020 follow-ups, bundled (SPEC-071)

## Delegation Summary

You are the **builder** for SPEC-071. Cut `claude/spec-071-build` from `origin/main`
**after** the design PR merges. **SPEC-072 is cut after your PR merges**, because both
edit AGENTS.md §6.

**The task**, four small items, with no `src/` change:
1. **An absolute Compression 7 allocation pin** in `tests/plane_units_alloc.rs`. It is
   10 per strip or tile, measured at design.
2. **One assert message** reworded in `tests/plane_oracle.rs`'s
   `inject_chunk_extraction_fault`.
3. **A `just fuzz-ljpeg` recipe** that creates `fuzz/corpus/ljpeg` first.
4. **AGENTS.md §6** lists the recipes it is missing: `orient-timing`, `unpack-timing`,
   `ljpeg-timing` and `fuzz-ljpeg`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-071-stage-020-follow-ups-bundled-1-pin-the-absolute-compression.md`.
  Read all of it: the §1 measured table, §2–§4, AC1–AC5 and the Notes.
- **Decisions:** DEC-063 §2, DEC-064 §2 and DEC-016.
- **Code:**
  - `tests/plane_units_alloc.rs`. Its tally is per thread since PR 243; keep it so.
  - `src/ljpeg.rs` (`parse_headers`, `parse_dht`), to explain the 10.
  - `src/plane.rs` (`decode_unit`), for AC1's red-proof.
  - `tests/plane_oracle.rs`.
  - `app.just`.
  - `fuzz/fuzz_targets/ljpeg.rs`.
  - AGENTS.md §6.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - The full `just test` still takes about 37 min in debug (SPEC-072 fixes that), so run
    it in the background.
  - Do AC1's red-proof on an **archived copy** with its own target dir.

## Expected Deliverables

1. **The four items**, as the spec's §1–§4 describe.
2. **AC1's red-proof:** the added allocation turns the new absolute assertion red, while
   the old-equals-new assertion alone stays green. Show both.
3. **AC3:** delete `fuzz/corpus/ljpeg`, run `just fuzz-ljpeg 60`, and show that it is
   created and the run is clean.
4. **AC4:** list the `app.just` recipe names against the §6 block, with none missing.
5. **Gates, all green with the corpus env set:**
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

   Also confirm `git diff origin/main -- src/` is empty.
6. **PR.** Open it against `main` from `claude/spec-071-build`, referencing SPEC-071 and
   HANDOFF-149. Confirm CI is green on its head.

## Out of Scope

- Any `src/` change.
- Reducing the allocations.
- Pinning held files.
- The test lane (SPEC-072).
- CHANGELOG (this is test and tooling only).

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
- **If an existing test goes red, the change is wrong.** Fix the change, never the test.
  The only permitted test edits are §1's new assertion and §2's message.
- **Fill the `handback:` front-matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

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
