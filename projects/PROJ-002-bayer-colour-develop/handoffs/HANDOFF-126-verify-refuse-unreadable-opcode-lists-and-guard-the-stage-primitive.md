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
  id: HANDOFF-126
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-11
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-056

project:
  id: PROJ-002
  stage: STAGE-014
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

# HANDOFF-126: VERIFY: refuse unreadable opcode lists and guard the stage primitives (SPEC-056, 0.2.1 housekeeping)

## Delegation Summary

You are the **cold verifier** for SPEC-056. You did **not** implement it. Review it
against SPEC-056 and DEC-053 and return exactly one verdict.

- **The build:** open **PR #193**, branch `claude/spec-056-build-b1`.
  - Code commit: `1973abf`.
  - Two later commits are handback docs only: `bd08547` and `93f2862`.
- **Your branch:** your worktree is cut from `claude/spec-056-verify-base`. That is
  the build head plus this handoff, so the code under review is the build's code.

The build is meant to do four things:
1. **Refuse an unreadable opcode list.** When an `OpcodeList1/2/3` tag is listed in
   `Sensor::malformed_tags`, every develop entry returns the new
   `Error::UnreadableOpcodeList { list }`, with kind `Corrupt`. Parsing stays
   tolerant.
2. **Guard the stage primitives.** `demosaic_into`, `demosaic_into_threaded` and
   `to_srgb_linear_into` run the colour-path required-processing check first.
   `opcode::check_required_processing(&Sensor, DevelopPath)` and `DevelopPath` become
   public, with `DevelopPath` marked `#[non_exhaustive]`.
3. **Fix the 19 rustdoc private intra-doc links,** and add a `just doc-check` gate,
   its red-proof, and CI jobs.
4. **Isolate the nested-cargo tests.** All 7 nested-cargo red-proof tests build in a
   target dir that does not depend on `CARGO_TARGET_DIR`.

**No `DEVELOP_VERSION` change and no new dependency.**

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-056-refuse-unreadable-opcode-lists-and-guard-the-stage-primitive.md` (all of it).
- **Decision:** `./decisions/DEC-053-refuse-unreadable-opcode-lists-at-develop-and-guard-the-stage-primitives.md`. Its rejected alternatives are settled; do not re-litigate them:
  - no sentinel-id reuse;
  - no parse-time failure;
  - no `&Sensor` on the appliers;
  - no escape hatch.
- **Prior contract:** DEC-049 and SPEC-054, which define the scan being extended.
- **Build handoff and handback:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-125-build-refuse-unreadable-opcode-lists-and-guard-the-stage-primitive.md`. Treat its claims as claims, and check them yourself.
- **Toolchain:** AGENTS.md §5 "Measured toolchain" and §6 commands. `just test` already passes `--all-features`; add no flag.

## The verify contract

Review it cold.

### Acceptance criteria and gates
- Is every SPEC-056 acceptance criterion met?
- Is the build reflection honest?
- Run the full gate suite yourself:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `deny`
  - `deny-fuzz`
  - `doc-check`
  - `doc-check-red-proof`
  - `msrv`
- Check decision drift with `just decisions-audit --changed`.

### The §12 behavioural surface (this is where defects escape)

**The refusal fires from each entry.** Confirm `UnreadableOpcodeList` is returned by
**each** of these, not one representative, and for each of lists 1, 2 and 3:
- `develop_into`
- `develop_color_into`
- `develop_color_f32_into`
- both threaded colour entries
- the three guarded primitives

Use **adversarial mutation**: remove the new check, or the guard call in a primitive,
and confirm tests go red per entry.

**The fixtures are genuinely unreadable.** Confirm the test asserts the tag is in
`malformed_tags`. A fixture that silently degraded to an absent tag would prove
nothing.

**The red-proofs were real.** Check that the unreadable-list red-proof and the
doc-check red-proof each show a control first and a red for the stated reason.
Break `doc-check` once yourself by injecting a private link.

**The `CARGO_TARGET_DIR` claim holds.** Run the nested-cargo tests with
`CARGO_TARGET_DIR` set to a temp dir, and confirm all 7 pass. Confirm the outer
target dir receives no mutated build.

**The corpus is unchanged.**
- Develop all 8 held files with
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
- Each must be `Ok` and byte-identical to `main` on its path (mono or colour).
- `DEVELOP_VERSION` must be unchanged.

**The public surface is right.** `DevelopPath` must be `#[non_exhaustive]`, and the
public check must return the same result as the matching `develop_*`. Also confirm
the docs of `apply_warp_into` and `apply_fix_bad_pixels_constant` state that the
caller owns validation.

**The CHANGELOG is right.** `[Unreleased]` should carry the Fixed / Added / Changed
entries, and the two closed Known-limitations bullets should be removed. There
should be no version bump.

**CI is observed green on the PR head** with `gh run view`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- A punch-list fix goes on a fresh branch `claude/spec-056-verify-<suffix>`, opened
  as a NEW PR against `main` that supersets #193. It must reference HANDOFF-126,
  SPEC-056, DEC-053 and PR #193.

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
