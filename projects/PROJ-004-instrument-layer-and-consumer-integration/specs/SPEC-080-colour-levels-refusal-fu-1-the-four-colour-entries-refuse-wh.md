---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-080
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  stage: STAGE-026
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-070, DEC-068, DEC-051]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-078, SPEC-079]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-026's <capability>". Optional; null is acceptable.
value_link: "Makes the colour develop refuse malformed levels as mono does, the last behaviour change before STAGE-026's release."

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
  tokens_estimate: 15000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-080: Colour levels refusal (FU-1): the four colour entries refuse WhiteLevel at or below the maximum applied black with Error::InvalidLevels before writing, as mono PerPosition does; DEC-070 §3 amended again; no pin moves. Plus the two SPEC-079 tooling follow-ups: sdk-stage2.sh finds the repo through git, and just validate parses feedback/ front matter

## Context

STAGE-026 (PROJ-004). This spec closes FU-1 from SPEC-078's verify and SPEC-079's two tooling
follow-ups. The release spec that follows it puts both STAGE-025 and this spec on crates.io.

Read at design, on `origin/main` at the STAGE-026 merge:

1. **The colour path refuses no levels.** `resolve_colour_levels` (`src/color.rs`) computes
   `range = white.saturating_sub(max_black)` on all three branches (the scalar `BlackLevel`
   with no pattern, the 2×2 slots, the general pattern) and never returns `InvalidLevels`.
   So a plane with `W ≤ B` develops to all zeros on `develop_color_into`,
   `develop_color_f32_into`, `develop_color_into_threaded` and `develop_color_f32_into_threaded`. Mono already refuses the same input:
   `resolve_levels` (`FirstValue`, black ≥ white) and `resolve_levels_with` (`PerPosition`,
   `B ≥ W`), with `Error::InvalidLevels`. The colour entries' `# Errors` docs don't list it.
   - The colour white is its own: `white_levels.first()`, else `white_level`, else
     `2^BitsPerSample − 1`. Keep it; don't switch to mono's `resolve_white`.
   - All three colour entries call `resolve_colour_levels` after `check_color_buffers` and
     before `resolve_colour_gain_maps`, before any write. The threaded entries call it from
     `develop_color_bands` on the calling thread.
   - `src/color.rs`'s unit test `colour_with_no_range_maps_to_zero_without_panicking`
     asserts today's behaviour: `resolve_colour_levels` is `Ok` on `[2047, 2047, 2047,
     15000]` with `W = 15000`, and every sample maps to 0. This spec changes that.
2. **`scripts/sdk-stage2.sh` finds "the repo" from its own location**
   (`$(dirname "$0")/..`). Run from another worktree, the main checkout is not "the repo",
   so an output directory inside the main checkout passes the in-repo check.
3. **`just validate` parses front matter only under `projects/` and the decisions dir**
   (`scripts/validate.sh`, the parse gate). `feedback/` is never parsed. Of its 11 files,
   9 start with front matter; `2026-04-20-bragfile-project.md` has none, and `_template.md`
   has its front matter after prose, as an example.

## Goal

The four colour develop entries refuse `W ≤ B` with `Error::InvalidLevels`, as mono does,
with no change to any file that develops today; and the two tooling gaps are closed.

## Design

1. **The colour refusal (DEC-070 §3, second amendment).** In `resolve_colour_levels`, on
   every branch, once `B` (the maximum black the path applies) and `W` (the colour white)
   are known: if `W ≤ B`, return `Error::InvalidLevels { black_level: B, white_level: W }`.
   - The scalar branch: `B` is the scalar black (`black_level`, 0 if absent).
   - The order is unchanged: `check_colour_required_processing`, `check_color_buffers`,
     then `resolve_colour_levels` (pattern shape first, then levels), then the
     `GainMap`s. Nothing is written or allocated before the refusal.
   - The threaded entries refuse from the calling thread, as for the pattern refusal.
   - `normalize_range` stays total (it is also used by the mono path); don't remove its
     range-0 handling.
   - Document `InvalidLevels` in the four colour entries' `# Errors`, in the position above,
     and update the `Error::InvalidLevels` doc (`SPEC-014` wording mentions only
     `BlackLevel >= WhiteLevel`) to say mono and colour check `B`.
   - `DEVELOP_VERSION` stays 7: no file that develops today changes output.
2. **`sdk-stage2.sh` judges "inside the repository" through git.**
   - If `git` is available and the script sits in a git checkout, refuse an output
     directory inside **any** worktree of that repository: the paths from
     `git worktree list --porcelain`, resolved with `pwd -P`.
   - Otherwise fall back to today's script-relative root, and say so on stderr.
   - Keep SPEC-079's order: resolve without creating, check, then `mkdir -p`.
3. **`just validate` parses `feedback/` front matter.**
   - Add every `feedback/*.md` that **starts** with a `---` line to the parse targets,
     except `_template.md`. A file with no front matter is not a parse target, as for any
     other scope. List it in the success line's count, so it is visible what was parsed.
   - Also check `status` is one of `open`, `addressed`, `deferred` (the template's
     vocabulary), failing loudly otherwise.
   - `feedback/archive/` is included, by the same rule.
   - `scripts/validate.sh` is template-derived. Keep the change small and say in a comment
     that `feedback/` is this repo's addition.

## Inputs

- `src/color.rs` (`resolve_colour_levels`, the three entries, `develop_color_bands`, the
  unit tests near `colour_with_no_range_maps_to_zero_without_panicking`), `src/develop.rs`
  (`resolve_levels`, `resolve_levels_with`, for the mono shape), `src/lib.rs`
  (`Error::InvalidLevels`).
- `tests/black_normalisation.rs`, `tests/colour_black_pattern.rs`, `tests/color_oracle.rs`,
  `tests/error_kind.rs` (fixture shapes and the error-category table).
- `scripts/sdk-stage2.sh`, `scripts/validate.sh`, `feedback/`.
- DEC-070 (§1, §3 and both amendments), DEC-068 §2 (the colour pattern), DEC-051 (error
  categories: `InvalidLevels` is `ErrorKind::Corrupt`).

## Outputs

- **Modified:** `src/color.rs`, `src/lib.rs` (docs only), `tests/black_normalisation.rs` (or a
  new `tests/colour_levels.rs`), `scripts/sdk-stage2.sh`, `scripts/validate.sh`,
  `CHANGELOG.md` (`[Unreleased]`, a "Changed" entry naming the newly refused input),
  `docs/provenance-ledger.md` only if a row describes the colour path as total.
- **No change** to output, pins, `DEVELOP_VERSION` or the public API's shape.

## Acceptance Criteria

- [ ] **AC1. Refusal on every colour entry and branch.** For each of `develop_color_into`,
  `develop_color_f32_into` and both threaded entries, and for each branch (scalar
  `BlackLevel` with no pattern; a 2×2 slot pattern; a `[1, 2]` pattern; a general 4×4
  pattern), a sensor with `W ≤ B` returns exactly `Error::InvalidLevels { black_level: B,
  white_level: W }`, and `dst` is untouched (fill it with a sentinel and check). Cover
  `W = B` and `W < B`, and a case where `B` is not the first or last value (SPEC-079's
  lesson), so the error reports the maximum.
- [ ] **AC2. The boundary.** `W = B + 1` develops `Ok` on every entry and branch, and every
  sample at or above `W` maps to 65535.
- [ ] **AC3. Order.** A sensor that is also wrong in an earlier check (a wrong `dst` length;
  a wrong `BlackLevel` count) reports that earlier error, not `InvalidLevels`; and a sensor
  with `W ≤ B` and a malformed `GainMap` reports `InvalidLevels` (levels before gains).
- [ ] **AC4. Nothing that develops changes.** Every pin holds, tier A and tier B with the
  corpus; `DEVELOP_VERSION` is still `"7"`; no held file is refused.
- [ ] **AC5. Mutants.** With an unmutated control green first, each goes red on tier A:
  the check removed; `<` in place of `≤`; the check made against the first black rather
  than `B`; the check on the slot branch only (missing on the general and scalar branches);
  the threaded entry skipping it. Record what goes red.
- [ ] **AC6. `sdk-stage2.sh`.** Record each outcome: an output directory inside this
  worktree is refused; one inside **another** worktree of the repository is refused (the
  case that passed before); one outside every worktree is created; with `git` absent from
  `PATH`, the fallback runs and says so. Re-run the cold four-file recipe: outputs are
  byte-identical to SPEC-079's sha256s.
- [ ] **AC7. `just validate` on `feedback/`.** It parses the 9 front-matter files; it fails
  loudly on a scratch copy with broken YAML and on one with `status: closed`; it skips
  `_template.md` and the file with no front matter. Show each outcome without committing
  the broken copies.
- [ ] **AC8. Gates.** `just test` with the corpus, with and without the SDK variable;
  `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`,
  `gate-audit`, `name-length-check`, `decisions-index --check`; `semver-check` (record its
  verdict: a new refusal on an existing variant is expected to need no version change);
  CI green.

## Failing Tests

- `src/color.rs` unit tests: `resolve_colour_levels` refuses `W ≤ B` on each branch with
  `B` reported. **Pre-approved rewrite:** `colour_with_no_range_maps_to_zero_without_panicking`
  becomes a refusal test for `resolve_colour_levels`, plus a direct `normalize_range(…, 0)`
  check that range 0 still maps to 0 without panicking (the normaliser stays total). No other
  existing assertion may move; stop and report if one has to.
- The public-entry tests for AC1–AC3 (all four entries, each branch, sentinel `dst`).

**Rule-clause map.**

| DEC-070 §3 clause (second amendment) | Test | Path it must reach |
|---|---|---|
| refuse `W ≤ B` | AC1 | each of the scalar, slot and general branches of `resolve_colour_levels`, through each public entry |
| `B` is the maximum applied black | AC1's not-first, not-last case | the slot and general branches |
| after the pattern check, before any write | AC3 and the sentinel `dst` | the three entries, and the threaded path's calling thread |
| `W` is the colour white | AC1 with `white_levels` set and `white_level` different | the colour resolver |

## Non-Goals

- Any output change, `DEVELOP_VERSION` change or new error variant.
- The release (the next spec).
- `BlackLevelDeltaH/V` (DEC-070 §5).
- Changing mono's white resolution to match colour's, or the reverse.
- The public DNG writer (PROJ-005).

## Notes for the Implementer

- **Find every test that relies on the old behaviour first.** Run the full tier-A and
  corpus suites against a throwaway refusal before writing the new tests, and list every
  assertion that goes red (signal `a-changed-formula-lists-every-assertion-it-moves`).
  Only the pre-approved unit test may move.
- **Check each mutant can fail on its fixture** before relying on it (signal
  `a-designed-mutant-must-be-able-to-fail-on-its-fixture`): the "first black" mutant needs a
  fixture whose first black is below `W` while `B` is not.
- **Plain-prose commit bodies**, no code tokens. Never `git add Cargo.lock`. Stage explicit
  paths. Mutants each get their own `CARGO_TARGET_DIR`.

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
