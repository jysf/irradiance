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
  id: HANDOFF-151
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-151: VERIFY: STAGE-020 follow-ups, bundled (SPEC-071)

## Delegation Summary

You are the **cold verifier** for SPEC-071. You did **not** implement it. Review it
against the spec, and return exactly one verdict.

- **The build:** **PR #247**, branch `claude/spec-071-build`, cut from `origin/main` at
  `eb3310a`. Its head is `9bb96de`: the build commit `1dcf5c6`, then the handback.
- **Your branch:** your worktree is cut from `claude/spec-071-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`.

The build changes five files:
- **`tests/plane_units_alloc.rs`:** `C7_ALLOCATIONS_PER_CHUNK = 10`, asserted as an
  absolute count beside the old comparison.
- **`tests/plane_oracle.rs`:** one assert message.
- **`app.just`:** `fuzz-ljpeg`.
- **`fuzz/fuzz_targets/ljpeg.rs`:** a header line.
- **AGENTS.md §6:** the missing recipes.

No `src/` file changes.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-071-stage-020-follow-ups-bundled-1-pin-the-absolute-compression.md`.
  Read all of it: §1–§4, AC1–AC5 and the Notes.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-149-build-stage-020-follow-ups-bundled-1-pin-the-absolute-compression.md`.
  Treat its claims as claims. The points to judge:
  - **The explanation of the 10.** The build says it is two `parse_headers` calls per
    chunk (DEC-047's shape binding in `plane`, then `ljpeg::open_scan`) × 5 allocations
    each. Read the code and confirm that the comment in the test is true. If it is
    wrong, the pin may still be right but its comment misleads.
  - **The chunk count** must come from the sensor's strip or tile offsets, not a literal.
    Confirm both fixtures (4 strips, 12 tiles) at every band height.
  - **AC1's red-proof.** Reproduce it yourself on an archived copy: one extra allocation
    per chunk in each Compression 7 arm makes the absolute assertion red, while the
    old-equals-new assertion stays green.
  - **The deviation:** the build also added `doc-check`, `doc-check-red-proof` and
    `validate-red-proof` to §6, and renamed the `licences` header. AC4 asked for every
    recipe, so judge whether the additions are correct and accurate. Re-run AC4's list
    comparison yourself.
  - **The message names `unpack_groups14`**, not the spec's suggested
    `unpack_bitstream`. Confirm that the needle really lives there.
  - **AC3:** delete `fuzz/corpus/ljpeg`, then run `just fuzz-ljpeg 60`.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - `just test` still takes about 37 min in debug (SPEC-072 fixes that), so run it in
    the background.

## The verify contract

### Acceptance criteria and gates
- **Check that each of AC1–AC5 is met** by evidence that would fail if the property
  broke.
- **Run the gate suite yourself, with the corpus env set:**
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
- **Observe CI** green on the PR head with `gh pr checks 247`.

### The behavioural surface
- **No `src/` change.** `git diff eb3310a -- src/` must be empty.
- **Test edits are only the ones permitted.** `git diff eb3310a -- tests/` may show only:
  - the new constant and assertion, plus their comment, in `plane_units_alloc.rs`;
  - the one message literal in `plane_oracle.rs`.
- **Every existing test is unedited** and green, including both `plane_oracle`
  red-proofs and their controls.
- **AGENTS.md:** every command added to the §6 block must actually run as written.
  Spot-check two of them.

**Mutate it yourself** (tier A, archived copy):
- the constant changed to 9 and to 11;
- the chunk count taken from a literal for one fixture;
- the new assertion removed, with the extra allocation present (this should go green,
  which shows the old check alone misses it).

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #247.
  - Commit the filled HANDOFF-151 handback (and the timeline's verify line) on a branch
    named `claude/spec-071-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #247 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #247.
  - Reference HANDOFF-151, SPEC-071 and PR #247.
- **Rejected:** comment on #247 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- The test lane (SPEC-072).
- Reducing the 10 allocations.
- Pinning held files.
- Any `src/` change.

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
