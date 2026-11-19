---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-076
  type: chore  # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S  # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-023
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-060, DEC-061, DEC-062]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-065, SPEC-066, SPEC-067]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-075]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-023's <capability>". Optional; null is acceptable.
value_link: "closes the five test-strength gaps the STAGE-019 verifies named, each proved by a mutant shown red"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 9000000
  sessions:
    - cycle: build
      handoff: HANDOFF-158
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 9294458
      estimated_usd: 6.67
      duration_minutes: 25
      recorded_at: 2026-11-19
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (63 unique message.ids, all claude-opus-5-5, no subagents): input 133 + cache-creation 123,246 (1-hour) + cache-read 9,136,363 + output 34,716 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Covers the build, its background gate and mutant runs, the CI wait and the handback commit. The handback floor was 6,316,433 and $4.78."
    - cycle: verify
      handoff: HANDOFF-160
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 6482112
      estimated_usd: 5.12
      duration_minutes: 30
      recorded_at: 2026-11-19
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (47 unique message.ids, all claude-opus-5-5, no subagents): input 93 + cache-creation 108,154 (1-hour) + cache-read 6,339,103 + output 34,762, opus-5-5 rates. PUNCH LIST, tests only: two of seven verifier mutants survived (an allocation on the applier's own parse error, and a helper call hidden in a trailing comment), pinned in d79dc50 and carried into PR 264 by fast-forward. The handback floor was 5,498,418."
  totals:
    tokens_total: 15776570
    estimated_usd: 11.79
    session_count: 2
shipped_at: 2026-11-19
---

# SPEC-076: STAGE-019 verify test hardenings

## Context

The three STAGE-019 verifies each approved their spec and named small test-strength gaps,
recorded as `[XS]` follow-ups:
- SPEC-065: HANDOFF-138 §Follow-up work;
- SPEC-066: HANDOFF-140 §Follow-up work;
- SPEC-067: HANDOFF-142 §Follow-up work.

None of them is a known bug. Each is a place where a plausible regression would pass the
test suite, or would be caught only by an unrelated test. This spec closes all five in one
tests-only build. It does not depend on SPEC-075's behaviour, but both edit
`tests/black_levels.rs`, and SPEC-075 may touch `src/develop.rs`. So it is built
after SPEC-075 merges, from that `main`. Its H3 and H4 edits sit beside SPEC-075's
`DEVELOP_VERSION` pin change.

## Goal

Make each of the five tests catch the regression it is named for, and prove that with a
mutant shown red. No `src/` behaviour changes.

## Design

The five items, each with the mutant it must catch. Each mutant is applied to the source
by hand, shown red, reverted, and recorded in the handback (file, change, failing test).

1. **H1, the median fixture with unsorted neighbours** (SPEC-065, HANDOFF-138 mutant 11).
   - Today "neighbours not sorted before the median" is caught only by
     `develop_warp_branch_actually_warps`. The corner/edge median fixture
     (`fix_bad_pixels_median_with_fewer_than_eight_neighbours`, in `tests/develop.rs`)
     uses a row-major ramp, so its neighbours arrive sorted.
   - Add a fixture to the `apply_fix_bad_pixels_constant` tests whose neighbour values
     are **not monotone in scan order**: an interior pixel and an edge pixel, each with an
     expected median written out by hand in a comment.
   - **Mutant:** skip the sort, or take the middle element unsorted. It must go red in
     the new test.
2. **H2, a develop-level malformed `OpcodeList1` row in the allocation test**
   (SPEC-065).
   - `tests/develop_scratch_alloc.rs`'s error-path list has no develop call with a
     malformed `OpcodeList1`. Add one row: a truncated `FixBadPixelsConstant` list on
     `sensor.opcode_list_1`, called through `develop_with_scratch_into`. Assert the error
     is the opcode error and that the call allocates nothing, matching the list's
     existing rows.
   - **Mutant:** make that error path allocate (for example `format!` into a discarded
     `String`). It must go red.
3. **H3, AC5's "the warp ran" guard** (SPEC-066, `tests/black_levels.rs` about line 528).
   - The comment claims the warp ran because `FirstValue` differs from `PerPosition`, but
     the two differ without a warp too.
   - Replace it with an assertion that the warped `FirstValue` output differs from the
     same develop with the warp off (`warp_alpha: 0.0`, or the sensor without the warp
     opcode, whichever the test already builds). Run it under both `StandardCrop` and
     `FullActiveArea`.
   - **Mutant:** skip the warp. It must go red in this test.
4. **H4, `develop_err` checks the scratch sentinel** (SPEC-066, `tests/black_levels.rs`
   line 682).
   - Fill `scratch` with `SENTINEL`, not 0, and assert it is untouched after the refusal,
     as AC6's `refused()` does.
   - **Mutant:** write to scratch before the pattern check. It must go red.
5. **H5, the traversal-helper source scan** (SPEC-067, `src/develop.rs`
   `both_gathers_walk_through_the_traversal_helper`).
   - Assert that each gather's call to the traversal helper passes `geometry.orientation`
     as its first argument, and skip `//` comment lines when scanning.
   - **Mutants:** pass a constant orientation (`for_each_out_pixel(1, …)`, the surviving
     mutant in HANDOFF-142), and comment out the real call while leaving its text in a
     comment. Both must go red.

## Inputs

- HANDOFF-138, HANDOFF-140 and HANDOFF-142, each at §Follow-up work and its mutant
  table.
- `tests/develop.rs`, `tests/develop_scratch_alloc.rs`, `tests/black_levels.rs`, and
  `src/develop.rs` (the test module, `for_each_out_pixel`, both gathers).

## Outputs

- **Modified:** `tests/develop.rs`, `tests/develop_scratch_alloc.rs`,
  `tests/black_levels.rs`, and `src/develop.rs` (the `#[cfg(test)]` module only).
- **Not modified:** any non-test code. `git diff` outside `#[cfg(test)]` and `tests/` is
  empty.

## Acceptance Criteria

- [ ] **AC1:** H1's new fixture exists, with neighbours that are not monotone in scan
  order and medians written out by hand. The unsorted-median mutant goes red in it.
- [ ] **AC2:** H2's develop-level malformed `OpcodeList1` row exists and asserts zero
  allocations. The allocating mutant goes red.
- [ ] **AC3:** H3's assertion compares the warped output with an unwarped develop under
  both framings, and the misleading comment is gone. The skip-the-warp mutant goes red.
- [ ] **AC4:** H4 sentinel-fills scratch and asserts it is untouched. The
  write-before-check mutant goes red.
- [ ] **AC5:** H5 checks the first argument of each helper call and ignores comments.
  Both mutants go red.
- [ ] **AC6:** no non-test code changes. `just test` (with the corpus) and every gate
  pass. CI is green. `DEVELOP_VERSION` is untouched.

## Failing Tests

The new and strengthened tests are AC1 to AC5. Each is "failing" in the sense that its
mutant must fail it. On unmutated code they pass, because these are hardenings, not bug
fixes.

## Non-Goals

- Reordering O7/O8 tile visits (HANDOFF-142's optional performance item).
- Speeding up the 24-pin `tiled_output_matches_the_pre_change_pins` test.
- The DEC-051 prose-table sweep.
- Any SPEC-075 work.

## Notes for the Implementer

- **Record every mutant in the handback:** where it was applied, what it changed, which
  test went red, and that it was reverted. That record is the spec's main deliverable.
- **Plain-prose commit bodies**, with no code tokens.

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Name the path each mutant must reach, not just the mutant. The spec's H2 row was a
   truncated list, which the required-processing check refuses before the applier's own
   parse ever runs, so an allocation on that parse survived the build. H5 said "skip
   comment lines", which a trailing comment on the same line beats. The verify found both,
   and both were pinned before merge. The spec also put the ramp fixture in the wrong
   file. The bundle cost about 15.8M tokens against a 9M estimate: two cold sessions each
   ran a dozen mutant copies with their own target dirs, which is the price of this kind
   of spec.

2. **Does any template, constraint, or decision need updating?**
   — No DEC. Lesson for tests-only specs: give each mutant the code path it must reach and
   a check that the red comes from the target assertion (read the panic message). Both
   cold sessions independently hit a mutant that went red for the wrong reason or never
   reached its path.

3. **Is there a follow-up spec I should write now before I forget?**
   — None required. Optional `[XS]`: a narrow-spread warp fixture large enough for H3's
   guard to run on both AC5 cases; the verify judged the k = 0 guard sufficient, because
   whether the warp pass runs does not depend on pixel values.

4. **Where was the worst defect caught?**
   — verify
   *(Test gaps only: two of the verifier's seven mutants survived the build's suite.
   Pinned in d79dc50 and carried into PR 264 before merge. No source change.)*

5. **What can a user do now that they couldn't before?**
   — none. Tests only: five STAGE-019 regressions (unsorted median, an allocating
   opcode-error path, a skipped warp, a scratch write before a refusal, a gather that stops
   using the traversal helper) now fail the suite, where before they passed it or were
   caught only by an unrelated test.
