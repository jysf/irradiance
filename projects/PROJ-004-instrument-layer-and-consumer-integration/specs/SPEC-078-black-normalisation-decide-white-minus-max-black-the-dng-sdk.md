---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-078
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  stage: STAGE-025
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-070, DEC-018, DEC-061, DEC-068, DEC-069]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-060, SPEC-066, SPEC-075, SPEC-079]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-025's <capability>". Optional; null is acceptable.
value_link: "Settles STAGE-025's black-normalisation question: the colour and PerPosition develops follow the DNG specification and match dng_validate within one code."

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
  tokens_estimate: 25000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-078: Black normalisation scales by white minus the maximum black (DEC-070)

## Context

STAGE-025 (PROJ-004). Every develop path today normalises a sample as
`(v − b) / (W − b)`, with `b` the black at that sample's position (DEC-018, DEC-061,
DEC-068). The DNG specification instead scales by `W` minus the **maximum** black for the
plane: DNG 1.7.1.0, Chapter 5, "Rescaling". Adobe's SDK does the same.

On files whose blacks differ by position, the two rules disagree by up to 6 codes. The
STAGE-025 design probe measured the specification's rule against `dng_validate` stage 2
on every held file whose blacks differ by position. It matches within one code. DEC-070
records the rule and the numbers.

## Goal

The colour path, and the mono path under `BlackLevels::PerPosition`, scale by `W − B`,
where `B` is the maximum black they apply. Every file with uniform blacks, and every mono
default (`FirstValue`) develop, stays byte-identical. `DEVELOP_VERSION` is 7. SPEC-060's
SDK cross-check is tightened to the new figures, and the Canon is added to it.

## Design

1. **The rule (DEC-070 §1).**
   - Formula: `v' = ⌊clamp(v − b, 0, W − B) · 65535 / (W − B) + ½⌋`, in `u64`. The
     rounding is DEC-018's: add `(W − B) / 2`, then divide.
   - `B` is computed once per develop, over the blacks that the path applies: the colour
     slot or pattern blacks, or the `PerPosition` pattern. It is never computed per sample.
   - With a uniform black (`B = b`), the formula is exactly today's `normalize` /
     `normalize_sample`. Keep it that way. The `FirstValue` and uniform cases must not
     change code path in a way that changes output or speed.
2. **Where.**
   - **Mono:** `src/develop.rs`. `normalize` and the `PerPosition` resolution (`Blacks`
     and `resolve_levels` / the per-position resolver) carry `B` alongside the
     per-position blacks.
   - **Colour:** `src/color.rs`. `slot_normalizer`, `pattern_normalizer` and
     `normalize_sample` take `B`, at every resolution site (`u16`, `f32` and banded
     threaded). These are the sites SPEC-059's `with_normalizer!` wraps.
   - `apply_gain_maps` is unchanged: it works on the normalised image.
3. **Validation (DEC-070 §3).**
   - `W ≤ B` is `Error::InvalidLevels`, with the same variant and fields as today.
   - The check uses `B`. A pattern whose largest black reaches white is now refused even
     if its first black is below white. Add a tier-A test for that.
4. **`DEVELOP_VERSION` goes to `"7"`,** with all four pins. Add a CHANGELOG
   `[Unreleased] → Changed` entry naming the affected files and paths.
5. **Docs.**
   - `BlackLevels::PerPosition`'s and the colour entry points' docs state the scale.
   - DEC-061's and DEC-068's text stay as they are; DEC-070 records the change.
   - Add a provenance-ledger row: class 1 (the specification text), plus class 4 for the
     SDK confirmation, as in DEC-069.
6. **The SDK cross-check (SPEC-060's `tests/gain_map_corpus.rs` and
   `scripts/sdk-stage2.sh`).**
   - The recipe also writes `CANON-EOS-5D-MARK-III-stage2.tif`: `dng_validate -2` on the
     Canon, which has no opcode lists.
   - The tests use the comparisons and **pre-registered bounds** in AC4. Δ is SDK minus
     ours. The tests remain env-gated on `IRRADIANCE_SDK_STAGE2_DIR`.
   - Keep the change to the script minimal. SPEC-079 rewrites its tag search afterwards.

## Acceptance Criteria

- [ ] **AC1. Analytic values (tier A).** Use a writer fixture with blacks `[2047, 2047,
  2047, 2048]`, a `[2, 2]` repeat and `W = 15000`, so `B = 2048` and `W − B = 12952`. Each
  value is asserted on mono `PerPosition` and on colour normalisation, with its arithmetic
  shown in the test:

  | Sample | Black | Today | DEC-070 |
  |---|---|---|---|
  | 8523 | 2047 | 32765 | **32768** |
  | 8524 | 2048 | 32768 | 32768 |
  | 4000 | 2047 | 9881 | **9882** |
  | 14999 | 2047 | 65530 | **65535** (clip at `W − B`) |
  | 15000 | 2047 | 65535 | 65535 |
  | 2000 | 2047 | 0 | 0 |

  A uniform-black fixture gives exactly today's values.
- [ ] **AC2. No collateral change.**
  - Every develop pin on a uniform-black file holds byte-for-byte: the Leica M Monochrom,
    Typ 246, Q2 Mono ×3, the Pentax K-3 III, the Nikon D300S and the Pixel 7 Pro.
  - Every mono default (`FirstValue`) pin holds on **every** held file, the Canon and the
    Pixels included.
  - Only the four `DEVELOP_VERSION` pins move.
- [ ] **AC3. Both paths.** Mono `PerPosition` and all four colour entries apply DEC-070,
  in `u16` and `f32`. Threaded output equals single-thread. The mono scratch develop still
  allocates nothing (`tests/develop_scratch_alloc.rs`).
- [ ] **AC4. The SDK cross-check, pre-registered.** With `IRRADIANCE_SDK_STAGE2_DIR` set:
  - **No gain:** mono `FullActiveArea`, unoriented, warp 0, `PerPosition`, with no
    `OpcodeList2`, against `-pre.tif` (Pixels) or `-stage2.tif` (Canon).
    - Canon, Pixel 3a and Pixel 4a: **max |Δ| ≤ 1, every Δ ∈ {0, +1}, at least 99.5%
      exact.** The probe measured 99.57%, 99.85% and 99.79%.
    - Pixel 7 Pro: max |Δ| ≤ 2, unchanged (DEC-068's fractional black).
  - **End to end:** the same develop with the file's `GainMap`s, against `-post.tif`.
    - Pixel 3a and 4a: **max |Δ| ≤ 4, at least 99.7% exact.** The probe measured 4 / 99.82%
      and 4 / 99.77%.
    - Pixel 7 Pro: max |Δ| ≤ 4. The probe measured 4.
  - **The gain-step gate** (max |Δ| ≤ 1, at least 99.9% exact) is unchanged.
  - **Unset:** a visible SKIP, as today.
- [ ] **AC5. Re-pins are listed and explained.**
  - SPEC-060's Pixel 3a and 4a colour and `PerPosition` pins change; list each old and new
    hash. The Pixel 7 Pro's pins do not change.
  - `tests/black_levels.rs::canon_per_position_differs_only_at_position_3` is
    **pre-approved** for rewrite. Under DEC-070, positions 0–2 differ from `FirstValue`
    too, because their scale changes. Rename it to state the new fact, and assert the
    hand-computable relation: at positions 0–2 the outputs differ only through the scale.
  - No other existing assertion is weakened. Any other pin that moves is a stop-and-report.
- [ ] **AC6. Red-proof, control first.** Each mutant fails at least one test, by its
  target assertion:
  - (a) the old per-position denominator `W − b`: fails AC1 and AC4;
  - (b) `B` as the minimum black: fails AC1;
  - (c) no clip at `W − B` (`v − b` unclamped above): fails AC1's 14999 row, or is shown
    equivalent with a proof as in SPEC-059;
  - (d) DEC-070 applied to `FirstValue` (`B` from the whole pattern): fails AC2;
  - (e) the validation check against the first black instead of `B`: fails AC3's new
    refusal test.

  Record the table in the handback.
- [ ] **AC7. Gates:**
  - `just test` with the corpus, and once more with the SDK variable;
  - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`;
  - `cost-audit`, `gate-audit`, `name-length-check`, `semver-check`;
  - a short `fuzz-opcode` run;
  - CI green.

## Failing Tests

Each clause of DEC-070 has a named test on the code path where it changes the output:
- **§1, the formula and the `W − B` clip:** AC1's rows, on mono `PerPosition` **and** on
  colour normalisation;
- **§1, uniform black unchanged:** AC1's uniform fixture and AC2's corpus pins;
- **§2, `FirstValue` unchanged:** AC2's mono default pins on the Canon and the Pixels,
  which are the files where `B ≠ b`;
- **§3, validation:** AC3's new refusal test;
- **§4, `DEVELOP_VERSION`:** the four version pins;
- **The real-file agreement:** AC4, env-gated.

## Non-Goals

- `BlackLevelDeltaH` / `BlackLevelDeltaV`. DEC-070 §5 carries them as a follow-up.
- DEC-068's fractional-black rounding. The Pixel 7 Pro's two-code gap stays.
- Emulating the SDK's `f32` scale.
- SPEC-079's script hardening.

## Notes for the Implementer

- **The probe's own numbers** (DEC-070 §Context) are the expected results. If a measured
  number lands outside a pre-registered bound, stop and report it. Don't widen the bound.
- **Regenerate the SDK inputs** with the committed `scripts/sdk-stage2.sh` (plus the Canon
  line) into your scratchpad. The `dng_validate` is at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
- **Keep the per-sample cost** the same integer affine map. `B` is resolved once, never
  per sample.
- **Commits:** plain-prose bodies with no code tokens. `Cargo.lock` is gitignored. Never
  commit corpus or SDK bytes.

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
