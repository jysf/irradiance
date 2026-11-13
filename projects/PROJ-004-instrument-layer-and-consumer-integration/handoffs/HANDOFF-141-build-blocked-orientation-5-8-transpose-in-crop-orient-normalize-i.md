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
  id: HANDOFF-141
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-13
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-067

project:
  id: PROJ-004
  stage: STAGE-019
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

# HANDOFF-141: BUILD: tiled Orientation 5–8 gather on both mono develop paths (SPEC-067)

## Delegation Summary

You are the **builder** for SPEC-067. Cut `claude/spec-067-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-062
and this handoff.

The task:
- In **both** `crop_orient_normalize_into` (warp-free) and
  `crop_and_orient_from_active_into` (post-warp), visit the output in 64×64 tiles for
  Orientations 5–8. Keep today's row walk for 1–4.
- The per-pixel body is moved, not rewritten. The output must be **byte-identical**: 24
  pre-change MD5 pins are in the spec.
- Add `examples/orient-timing.rs` plus a `just orient-timing` recipe, and record
  before/after timings.

There is no public API change, and `DEVELOP_VERSION` stays `"4"`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-067-blocked-orientation-5-8-transpose-in-crop-orient-normalize-i.md`.
  Read all of it:
  - the §1 timing table and the 24 MD5 pins (main at eaa2ae3);
  - the source-scan test;
  - Design §2–§4;
  - AC1–AC7, the red-proof list and the Notes.
- **Decision:** `./decisions/DEC-062-the-mono-orientation-gather-walks-the-output-in-64x64-tiles-for-orientations-5-8-and-stays-byte-identical.md`.
- **Also:**
  - DEC-058 (the version rule);
  - DEC-060 (the scratch path allocates nothing);
  - DEC-061 (the per-position black inside the per-pixel body);
  - DEC-043 (the colour path is untouched);
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/develop.rs`:
    - `crop_source_coords`;
    - `crop_orient_normalize_for` / `crop_orient_normalize_into`;
    - `crop_and_orient_from_active_into`;
    - `run_passes`.
- **Watch out:** `tests/develop_oracle.rs` (`inject_orientation_identity_fault`) and
  `tests/develop.rs` fault-inject by **scanning `src/develop.rs` source text**.
  - The orientation injector needs **exactly one** `crop_source_coords(` call inside
    `crop_orient_normalize_into`, written with its exact indentation.
  - The only permitted test edit is that injector's `needle` literal.
  - The SPEC-066 build lost time here. Grep `tests/` for `read_to_string` of `src/`
    before you restructure anything.
- **Tests to mirror:**
  - `tests/develop_options.rs` (tier-B gating, MD5 pins);
  - `tests/black_levels.rs` (tier-A fixtures with odd origins);
  - `tests/develop_scratch_alloc.rs`;
  - `tests/support/md5.rs` and `tests/support/corpus.rs`.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Run long tier-B binaries in the background. 24 Q2M develops in debug are slow;
    consider `--release` for that one binary when iterating, but the gate is `just test`.

## Expected Deliverables

1. **Tests first.**
   - Write `tests/orientation_tiles.rs` (AC1, AC2) and the AC3 unit test.
   - AC1 and AC2 **pass on main before the change**, because they pin today. Show that.
   - The AC3 unit test is the one that is red first.
   - Record the order in the handback.
2. **Implement it**, following Design §2:
   - one traversal rule, in both functions;
   - one per-pixel body each, with exactly one `crop_source_coords(` call in
     `crop_orient_normalize_into`;
   - no allocation, no `unsafe`, and no panicking arithmetic.
   - Then update only the injector `needle` and show the fault test still goes red on its
     injected copy (AC4).
3. **Timing (§3, AC6).**
   - `examples/orient-timing.rs` and `just orient-timing`.
   - Record a before table (the example on main at the design merge plus only the example
     file) and an after table, naming the machine.
   - Report against the target (O5–O8 within 1.15× of O1 at each α) and the floor (O6
     warp-free at least 25% faster).
4. **Docs (§4).** The CHANGELOG `[Unreleased]` Changed line with the medians, and the doc
   sentence. No provenance row, and no `DEVELOP_VERSION` bump.
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

   Also run `just decisions-audit --changed`.
6. **Red-proofs.** For each mutation in the spec's list:
   - apply it by hand;
   - show the named **tier-A** test go red;
   - revert it;
   - list it in the handback.

   A mutant that only tier B catches is a gap; close it in tier A.
7. **PR.** Open it against `main` from `claude/spec-067-build`, referencing SPEC-067,
   DEC-062 and HANDOFF-141. Confirm CI is green on its head.

## Out of Scope

- The colour path's scatter.
- Threads.
- SIMD.
- A per-machine tile size.
- A criterion dependency.
- A CI timing gate.
- Any output change.
- Unit-level unpack.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- **Never commit corpus bytes or derived corpus pixels (DEC-003).** MD5 strings and
  timings in source are fine.
- **If a pin goes red, the change altered output.** That applies to any §1 pin, any
  SPEC-063 pin, or any existing test. Fix the change, never the test (except AC4's
  needle).
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
