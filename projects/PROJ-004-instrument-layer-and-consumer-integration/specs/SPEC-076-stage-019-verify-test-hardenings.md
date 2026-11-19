---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-076
  type: chore  # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S  # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
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
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
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

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
