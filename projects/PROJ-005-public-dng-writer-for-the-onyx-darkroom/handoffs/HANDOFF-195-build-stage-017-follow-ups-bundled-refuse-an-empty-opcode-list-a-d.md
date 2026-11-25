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
  id: HANDOFF-195
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-094

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

# HANDOFF-195: Build SPEC-094, STAGE-017's three follow-ups, bundled

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-094` to `claude-opus-5` (implementer) for the
**build** cycle, on a branch cut from `main`. Make the spec's failing tests pass and close
three small items:
1. the writer refuses an empty opcode list;
2. CI gains a 60 s `dng_write` fuzz smoke job;
3. three private doc links stop breaking a no-feature `--document-private-items` build.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-094-stage-017-follow-ups-bundled-refuse-an-empty-opcode-list-a-d.md`.
  The reason's name and payload, the check's place in the order, the job's name, and the
  rule-clause map are binding.
- **Stage:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`
- **Prior specs:** SPEC-061, SPEC-081 and SPEC-082 in `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/done/`. These are the
  writer you extend. Their verify handoffs (HANDOFF-190, 192, 194) list the findings this
  spec closes (F3, F4) and the mutant style the verify will use.
- **Precedent for a bundled follow-up spec:** SPEC-074 (PROJ-004).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.
- **Environment:** set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  and
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`
  so `just test` runs the corpus tests. Never commit `dng_validate` or its outputs.

### Decisions that apply

- `DEC-081` §2 and its Follow-up: an empty XMP is refused, and this spec does the same for
  opcode lists. At build, record "resolved by SPEC-094" in that Follow-up bullet.
- `DEC-075` §5, §7 and §9: the opcode-list rules, the two error variants, and no
  allocation.
- `DEC-080`: the payload types in `src/write_types.rs`, compiled in every build.
- `DEC-051` (no wildcard match) and `DEC-052` (`#[non_exhaustive]`).

### Constraints that apply

- `no-panics-on-untrusted-input`: `just lint-no-allow` must pass, with no `#[allow]` for
  the doc links either.
- `oracle-must-be-shown-red`: show each new test failing on `main`'s `src/` before the fix.
  Say how in the handback.
- `library-not-application`: no I/O in `src/dng`.
- `no-copyleft-dependencies`: no new dependency.

## Expected Deliverables

- Every file in SPEC-094's `## Outputs`, and every test in `## Failing Tests` passing.
- **Three commits, one per item,** so the verify can mutate each item on its own.
- **Gates, all green:**
  - `just test`, with the corpus and `IRRADIANCE_DNG_VALIDATE` set;
  - `just lint-ci`, `just lint-no-allow`, `just fmt-check`, `just doc-check`;
  - `just deny`, `just deny-fuzz`, `just lint-fuzz`, `just lint-fuzz-red-proof`, `just msrv`;
  - `just validate`, `just gate-audit`, `just cost-audit`, `just semver-check`;
  - 60 s of `just fuzz-dng-write`.

  **Before handing back, confirm every job in `gh pr checks` for your PR passes. There
  should be 23 now, the new `dng_write` smoke job included.**
- **The PR:** against `main` from `claude/spec-094-build`. It names HANDOFF-195, SPEC-094,
  STAGE-017 and PROJ-005, plus DEC-075 and DEC-081. Commit messages are plain prose.
- **In the handback:**
  - The AC6 doc command's output, before and after, without features and with
    `--all-features`.
  - Which form you chose for the `src/lib.rs` link, and why.
  - Every fuzz seed or test that changed outcome because of the refusal.
  - The token count.

## Out of Scope

- The 13 pre-existing `ljpeg` private doc links, and any CI gate on
  `--document-private-items`.
- Walking `OpcodeList2` or `OpcodeList3`; refusing a count-0 opcode header; changing the
  `OpcodeList1` allow-list.
- Any reader change.
- The STAGE-017 close and the release spec. The orchestrator does both.

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
