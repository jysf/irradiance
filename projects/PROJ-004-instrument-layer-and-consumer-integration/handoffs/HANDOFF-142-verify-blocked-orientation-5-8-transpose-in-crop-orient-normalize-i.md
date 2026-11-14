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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 5406692            # REAL combined count — what cost-audit reads
  estimated_usd: 4.44              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 46
  branch: claude/spec-067-verify-270044
  pr: 230
  completed_at: 2026-11-13         # YYYY-MM-DD
  notes: "APPROVED at 26bcf48, no code changes; verdict commented on PR 230. Tokens are a floor: measured before the commit/push/comment turns." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
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

- **Branch / PR:** branch `claude/spec-067-verify-270044` (handback only, no PR, per the brief); verdict commented on https://github.com/jysf/irradiance/pull/230
- **Completed at:** 2026-11-13
- **All acceptance criteria met?** yes. AC6's warp-free 1.15× target is missed as a ratio but accepted (see Timing).
- **For `verify`:** ✅ APPROVED at `26bcf48` (PR #230 head), with no code changes.

**Gates (run here, corpus env set, 12/12 present).**
- `just test`: 675 passed, 0 failed, 1 ignored.
- `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`, `cost-audit` and `gate-audit` are all green.
- `decisions-audit --changed` is advisory only.
- `gh pr checks 230` is all green on `26bcf48`.

**Byte identity (the hard criterion).**
- **AC1.** The 24 pins in `tests/orientation_tiles.rs` match the spec's §1 list field for field. I compared them by script; the only difference is the label `Std`/`Full` versus `Framing::…`.
  - `tiled_output_matches_the_pre_change_pins` **ran**: it took over 60 s and printed no SKIP. That includes the O1 α 0 `Std` and `Full` pins.
  - Each develop writes into a sentinel-filled buffer and is MD5'd whole. So every pixel is computed on both entry points, and no work is skipped.
  - `git diff 56976a1 -- tests/` shows only the new file and the O8 alloc row. The SPEC-063 pins are unedited.
- **AC2 (independent reference).**
  - The test writes its own forward-map orientation table (crop → output, the opposite direction from the crate's gather) and its own `normalize`, with round-half-up, clamp and per-position black.
  - It does not call `crop_source_coords`, `develop` internals or a crate re-export. Only the public develop entry points are under test.
  - The crop sizes are 1×1, 63×65, 64×64, 65×63, 130×67 and 200×129, with odd `ActiveArea` (3,1) and `DefaultCrop` (1,3) origins.
  - The warped case compares against the `apply_orientation: false` develop, re-oriented by the test's own formulas. The test also asserts that the warp is not a no-op.
  - `PerPosition` is covered by a 2×2 pattern on two sizes.
- **AC4.** `git diff 56976a1 -- tests/develop_oracle.rs` is empty.
  - The needle (12/16-space indentation) matches the **live** `crop_source_coords(` call inside the closure, exactly once. The function's doc comment does not contain it.
  - `the_oracle_is_red_on_an_orientation_fault` passes, which means the injected copy went red.
  - Under mutants b4, b5 and v1 that test failed, which is the expected collateral of a broken gather.

**Mutants (by hand, tier A only, on a scratch copy of the tree; corpus env unset; all reverted, original file byte-compared after).** Tests run: the `develop::tests` unit tests, `orientation_tiles`, `develop_oracle` and `develop_scratch_alloc`.
- **The build's 8:** all red.
  - b1 inclusive clip, b2 end not clipped, b3 `step_by(tile + 1)`, b4 tile axes swapped and b5 transposed write: AC2 and AC3.
  - b6 5–8 untiled and b7 1–4 tiled: AC3's order test.
  - b8 `crop_and_orient_from_active_into` on its own row loop: `both_gathers_walk_through_the_traversal_helper`.
- **Tile start `tile_x + 1`:** red (AC2, AC3 and existing unit tests).
- **Split as `(4..=8)` or `(5..8)`:** both red (AC3).
- **`ORIENT_TILE = 1`:** red. AC3's order test hard-codes 64, so the timing example is not the only guard.
- **Visiting a pixel twice:** red (AC3's "exactly once").
- **Survivor (mine):** `crop_and_orient_from_active_into` calls `for_each_out_pixel(1, …)` instead of `geometry.orientation`.
  - The post-warp gather stays a row walk with byte-identical output. It makes exactly one helper call and has no own row loop, so every tier-A test is green.
  - Only `just orient-timing` would show it, on the requester's path.
  - **Judged benign:** it is a deliberately wrong argument, not a plausible refactor slip. Recorded as an XS follow-up below.
  - The source scan has the same class of gap: it counts a `for_each_out_pixel(` substring, so a commented-out call next to a loop with renamed variables would also pass.

**No allocation, no panics, scope.**
- `scratch_develop_allocates_nothing` now covers O1, O6 and O8 × fbp × warp × framing × α × apply_orientation, and it is green.
- The helper uses `saturating_add` and `.min`, and `step_by` receives `tile.max(1)`, so it never gets 0. There is no `unsafe` and no allocation.
- `crop_source_coords`, the per-pixel bodies and the colour path have no diff beyond the loop header becoming a closure.
- There is no `pub` change, and `DEVELOP_VERSION` is `"4"`.

**Timing (AC6).** `just orient-timing`, Apple M4 Pro (aarch64), release, median of 7, in ms. "Before" is `56976a1` plus only `examples/orient-timing.rs`; "after" is `26bcf48`'s code. Both were run back-to-back on an idle machine.

| α | | O1 | O2 | O3 | O4 | O5 | O6 | O7 | O8 | O6 unoriented |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | before | 198 | 201 | 215 | 198 | 391 | 392 | 413 | 416 | 198 |
| 0 | after | 86 | 101 | 104 | 103 | 112 | 124 | 163 | 164 | 87 |
| 1 | before | 621 | 617 | 619 | 620 | 784 | 785 | 793 | 788 | 614 |
| 1 | after | 615 | 617 | 607 | 606 | 644 | 630 | 646 | 647 | 612 |

- **The build's tables reproduce** to within about 10% (my machine ran about 5–10% slower in both columns).
- **Floor: met.** Warp-free O6 went from 392 to 124 ms (−68%).
- **Target, post-warp (the requester's path; every Q2M frame warps): met**, at 0.99–1.05×. O6 went from 785 to 630 ms. The requester's 774/617 gap is gone.
- **Target, warp-free: missed as a ratio**, at 1.30–1.90×. **Accepted.**
  - The denominator moved: O1 fell from 198 to 86 ms.
  - Every O5–O8 is now 2.5–3.5× faster than before, and faster than the old O1.
  - The spec's ratio assumed O1 would not move; this is not a regression.
- **The O1 speed-up is real codegen, not skipped work.**
  - In `crop_orient_normalize_into` the only change is that `for out_y { for out_x { body } }` became `for_each_out_pixel(…, |out_x, out_y| body)`. For 1–4 the helper runs one tile spanning the whole output, so it is the same nested row-major loop. The body is identical text.
  - The output is pinned whole (O1 α 0 `Std`/`Full` MD5s over sentinel-filled buffers).
  - The unoriented O6 control (the same O1 walk) moves identically, from 198 to 87.
  - The post-warp O1, whose gather got the same closure treatment, did not move. So the gain is specific to how LLVM optimises the warp-free body once it is inlined through a generic closure (black-level closure plus normalize), most likely hoisting or unswitching it could not do before.
  - I did not disassemble to name the exact transform.
- **O7/O8 warp-free remain the slowest** (about 163 ms). They read the source in descending row order.

### Cost self-report

- **Tokens (total):** 5,406,692 (floor)
- **Estimated USD:** 4.44
- **Duration (minutes):** about 45
- **Source of the number:** transcript `8d1256ae-02c8-466c-8164-acd2deb24e31.jsonl` (found by the scratchpad UUID), deduplicated by `message.id` with the max `output_tokens` per message, priced per component at opus-5-5 rates. There are no subagents. This is a floor: the commit, push and PR-comment turns after the measurement are excluded.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - AC6's warp-free target is missed as a ratio only, accepted above.
  - The spec named `justfile` for the recipe; the build put it in `app.just`, which is where the repo's cargo recipes live. That is fine.
- **Follow-up work identified:**
  - **[XS] Harden the AC3 source scan.** Assert that each gather's helper call passes `geometry.orientation` as its first argument, and ignore comment lines. This closes the surviving constant-orientation mutant and the commented-call gap.
  - **[XS, optional] Reorder O7/O8 tile visits** so that source rows ascend. The build proposed this; I agree it is the remaining warp-free headroom, about 1.9× O1 against about 1.3× for O5/O6.
  - Speeding up the 24-pin debug test (about 10 min) remains the build's follow-up.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. `just decisions-audit --changed` must be invoked with the flag as an argument, not quoted as one recipe name. My gate loop quoted it and got a false "no recipe" error.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — A timing target relative to O1 should state the baseline it is anchored to (the pre-change O1, or absolute ms). The build said the same.

3. **If you did this task again, what would you do differently?**
   — The same approach: run the mutants on an archived copy of the tree with its own target dir. That allowed the 10-minute corpus `just test` to run in parallel without the fault-injecting oracle tests copying a mutated `src/develop.rs`.
