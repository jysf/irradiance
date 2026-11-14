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
  id: HANDOFF-142
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier
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

# HANDOFF-142: VERIFY: tiled Orientation 5–8 gather on both mono develop paths (SPEC-067)

## Delegation Summary

You are the **cold verifier** for SPEC-067. You did **not** implement it. Review it
against SPEC-067 and DEC-062, and return exactly one verdict.

- **The build:** **PR #230**, branch `claude/spec-067-build`, cut from `origin/main` at
  `56976a1`. Its head is `26bcf48`: the code commit `714400f`, then the handback.
- **Your branch:** your worktree is cut from `claude/spec-067-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`, so the code under review is
  the build's code.

The build delivers:
- A shared private traversal helper in `src/develop.rs`. Both `crop_orient_normalize_into`
  and `crop_and_orient_from_active_into` walk through it: 64×64 output tiles for
  Orientations 5–8, today's row walk for 1–4.
- `tests/orientation_tiles.rs`: the 24 tier-B pins and an independent naive-gather
  reference.
- Two AC3 unit tests: the visit order, and a source scan that both gathers use the helper.
- An Orientation 8 row in `tests/develop_scratch_alloc.rs`.
- `examples/orient-timing.rs`, and an `orient-timing` recipe in `app.just`.
- CHANGELOG.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-067-blocked-orientation-5-8-transpose-in-crop-orient-normalize-i.md`.
  Read all of it: the §1 timing table and the 24 pins, Design §2–§4, AC1–AC7, the
  red-proof list and the Notes.
- **Decision:** `./decisions/DEC-062-the-mono-orientation-gather-walks-the-output-in-64x64-tiles-for-orientations-5-8-and-stays-byte-identical.md`.
  Also DEC-058 (the version rule), DEC-060 (no allocation), DEC-061 (the per-position
  black inside the body), DEC-043 (the colour path is untouched) and DEC-003.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-141-build-blocked-orientation-5-8-transpose-in-crop-orient-normalize-i.md`.
  Treat its claims as claims. The points to judge:
  - **AC6's warp-free target is missed.** The build reports O5–O8 at 1.21–1.87× of O1 at
    α = 0, because warp-free **O1 itself** fell from 190 to 78–84 ms. O2–O4 fell to about
    95–100. The build attributes this to the closure form optimising the row walk better.
    The post-warp target (the requester's path) is met, at 1.02–1.10×. The floor is met:
    O6 warp-free went from 324 to 110 ms.
    - **Reproduce both tables yourself.** Build the example on `56976a1` plus only the
      example file, then on the build head, and run `just orient-timing` on each.
    - **Judge whether the O1 speed-up is real** and how it arises, by reading the loop
      shapes. A 2.4× change on a path the spec said would be untouched is surprising. Make
      sure it is not a sign that work is being skipped. The 24 pins are the check: confirm
      the O1 α 0 `Std` and `Full` pins ran and passed.
    - **Decide whether the missed ratio is acceptable.** My design stated the target as a
      ratio to O1, assuming O1 would not move; the build's reflection says so. A missed
      target that comes only from O1 improving is not a regression. Say so, or say why
      it is.
  - **AC4: no needle edit was needed.** The claim is that rustfmt kept the per-pixel body
    at the old indentation, so `tests/develop_oracle.rs` has no diff. Confirm
    `git diff 56976a1 -- tests/develop_oracle.rs` is empty. Confirm
    `the_oracle_is_red_on_an_orientation_fault` actually injects into the **live** call
    inside the closure, so the injected copy goes red, not vacuously green.
  - **AC3 gained a second, source-scan test** (`both_gathers_walk_through_the_traversal_helper`).
    Judge whether it is robust: does it check a call, or just a substring that a comment
    could satisfy?
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - The 24-pin tier-B test takes about 10 min in debug. Run it in the background.

## The verify contract

### Acceptance criteria and gates
- Check that every AC1–AC7 is met by a test that would fail if the property broke, and
  that the build reflection is honest.
- Run the gate suite yourself, with the corpus env set:
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
- Run `just decisions-audit --changed`.
- Observe CI green on the PR head with `gh pr checks 230`.

### The behavioural surface (this is where defects escape)

**Byte identity is the hard criterion.**
- Confirm the 24 pins in `tests/orientation_tiles.rs` are character-for-character the
  spec's §1 list, and that the test **ran** (no SKIP line).
- Confirm the eight SPEC-063 pins and every other existing test are unedited:
  `git diff 56976a1 -- tests/` shows only new files and the alloc row.

**The independent reference.** Read AC2's helper in `tests/orientation_tiles.rs`.
- The eight orientation formulas and the normalize must be written in the test. It must
  not call `crop_source_coords`, `develop` or a crate re-export
  (`shared-path-oracle-tautology`, N=3).
- Check that the sizes straddle every tile edge (1×1, 63×65, 64×64, 65×63, 130×67,
  200×129) with odd origins.
- Check that the warped case compares against unoriented output re-oriented by the
  test's own formulas.

**Mutate it yourself**, outside any test knob, and run **tier A only** (CI has no corpus):
- the build's 8;
- an off-by-one in the tile **start** (`tile_x + 1`);
- the 5–8 / 1–4 split as `(4..=8)` or `(5..8)`;
- the tile constant set to 1 (still correct, just slow; which test, if any, notices? A
  silent pass here is acceptable only if the timing example is the guard. Say which);
- the helper visiting a pixel twice (correct output, wasted work; does AC3's
  "exactly once" catch it?).

A mutant that stays green, or that only tier B catches, is a finding unless you judge it
benign and say why.

**No allocation, no panics.** Check:
- the traversal allocates nothing (`scratch_develop_allocates_nothing` with O6 and O8);
- the arithmetic uses saturating or checked forms;
- `step_by` cannot receive 0.

**Scope.**
- `crop_source_coords`, the per-pixel body and the colour path are unchanged in
  substance.
- No public API change; `DEVELOP_VERSION` is still `"4"`.
- The CHANGELOG line states the before and after numbers honestly.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #230; it would conflict once #230 is
    squash-merged.
  - Commit the filled HANDOFF-142 handback (and the timeline's verify line) on a branch
    named `claude/spec-067-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #230 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #230.
  - Reference HANDOFF-142, SPEC-067, DEC-062 and PR #230.
- **Rejected:** comment on #230 with the reasons, and push the handback branch.

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

- The colour path's scatter.
- Threads and SIMD.
- A per-machine tile size.
- A criterion dependency or a CI timing gate.
- Any output change.
- Reordering O7/O8 so their source reads ascend. The build proposed it as an optional
  follow-up; note it if you agree, don't do it.
- Speeding up the 24-pin test. Also a build follow-up.

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
