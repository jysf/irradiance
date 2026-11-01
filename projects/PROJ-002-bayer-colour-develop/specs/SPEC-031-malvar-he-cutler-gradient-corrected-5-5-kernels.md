---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-031
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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

project:
  id: PROJ-002
  stage: STAGE-007
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-016, DEC-017, DEC-018, DEC-031]    # buffer-no-alloc / red-proof / u16 repr / Hamilton-Adams-not-MHC (patent)
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input]
  related_specs: [SPEC-030, SPEC-032]                # bilinear baseline + harness it reuses; CFA fuzz

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: "STAGE-007's quality demosaic — Adams-Hamilton directional interpolation (patent-clean, DEC-031) replacing SPEC-030's bilinear baseline on the same bit-exact oracle harness."

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
  tokens_estimate: 50000000        # prediction (design): HA is more involved than SPEC-030's bilinear (34M actual); directional green + colour-difference R/B
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-031: Hamilton-Adams directional demosaic (patent-clean quality kernel)

> **Algorithm changed from Malvar-He-Cutler → Hamilton-Adams — see DEC-031.** MHC
> is patent-encumbered (US 7,502,505, active to ~2027-01-25) and a patented method
> defeats irradiance's permissive-crate thesis (clean-room beats copyright, not
> patents). Hamilton-Adams (US 5,629,734, **expired**) is the patent-clean
> directional demosaic. The frame-stage filename slug still reads
> "malvar-he-cutler" — historical; the algorithm is Hamilton-Adams.

## Context

STAGE-007's quality demosaic — the kernel above SPEC-030's shipped **bilinear**
baseline, on the same `demosaic::demosaic_into` harness and the same bit-exact
analytic oracle. SPEC-030 left `demosaic_pixel` phase-driven with offset-1 averages;
this spec replaces that interpolation with **Adams-Hamilton** directional
demosaicing — a real quality step (directional green removes most of bilinear's
edge zippering / false colour). Per **DEC-031** the algorithm is Hamilton-Adams, not
the MHC the brief/STAGE-007 first named (patent). `depends_on` SPEC-030 (shipped):
reuses its `Cfa` / `BayerPhase`, the caller-owned RGB buffer (DEC-016),
`reflect` / `sample`, `u32_to_u16`, and the oracle harness.

## Goal

Replace SPEC-030's bilinear interpolation inside `demosaic::demosaic_into` with the
**Adams-Hamilton** directional demosaic — directional green (gradient-chosen axis,
second-difference corrected) then red/blue by gradient-directed colour differences —
phase-driven by `Sensor.color_filter`, verified **bit-exact** on analytic fixtures
(plus DC-preservation and a directionality fixture) with a red-proof. Output stays
camera-space linear `u16` RGB; the public signature is unchanged.

## Design

Exact algorithm, from the canonical Adams-Hamilton (US 5,629,734), re-expressed as
integer arithmetic. Neighbours of centre `C` at `(x,y)`: offset-1 `W,E,N,S`;
offset-2 `WW,EE,NN,SS`; diagonals `NW,NE,SW,SE`. All intermediates are **signed
`i32`** (corrections can go negative); each output channel divides with round-half-up
then clamps to `[0, 65535]`. `rnd4(n) = (n + 2).div_euclid(4)`, `rnd8(n) = (n + 4).div_euclid(8)`
(Euclidean/floor division so negative numerators round correctly).

**Two passes, green first (load-bearing for bit-exactness).** Pass 1 fills green at
every pixel and finalizes it to `u16`; pass 2 fills red+blue reading that green
plane. R/B interpolation reads the *interpolated* green at neighbouring non-green
sites, so green must be complete and rounded before R/B.

**Pass 1 — green at a red/blue site** (green sites keep their own sample):
- Classifiers: `CLh = |W − E| + |WW − 2·C + EE|`, `CLv = |N − S| + |NN − 2·C + SS|`.
- Predictor numerators (denom 4): `gh = 2·(W + E) − (WW − 2·C + EE)`,
  `gv = 2·(N + S) − (NN − 2·C + SS)`.
- Pick the **lower-gradient** axis: `CLh < CLv → rnd4(gh)`; `CLv < CLh → rnd4(gv)`;
  **tie → `rnd8(gh + gv)`**; clamp.

**Pass 2 — red/blue** (`g(·)` = pass-1 green):
- **At a green site:** one neighbour-pair is red, the other blue (read which from the
  pattern). For the colour with same-colour neighbours `(P1,P2)` on an axis:
  `rnd4( 2·(P1 + P2) − (g1 − 2·gC + g2) )`, clamp — `g1,gC,g2` the greens at those two
  neighbours and the centre. (Horizontal pair ⇒ W/E greens; vertical ⇒ N/S.)
- **At the opposite-colour site** (red site → blue, and vice-versa), diagonal,
  gradient-directed:
  - `dp = 2·(NW + SE) − (g_NW − 2·gC + g_SE)`, `dn = 2·(NE + SW) − (g_NE − 2·gC + g_SW)`.
  - `CLp = |SE − NW| + |g_NW − 2·gC + g_SE|`, `CLn = |SW − NE| + |g_NE − 2·gC + g_SW|`.
  - `CLp < CLn → rnd4(dp)`; `CLn < CLp → rnd4(dn)`; tie → `rnd8(dp + dn)`; clamp.
  - The site's own primary is its sample.

**Border (offset-2).** SPEC-030's `reflect` was written for offset-1; HA samples at
offset-2, so a single reflection of `−2`/`len+1` needs `len ≥ 3` to land in range
(else it clamps — safe, no panic). Reuse `reflect` / `sample` unchanged; verify
edge/corner pixels on a fixture ≥ 5 wide/tall (offset-2 reflection in range) and keep
the degenerate no-panic tests (1×1, 2×2, 3×5) exercising the clamp path.

**DC preservation.** Every predictor sums to 1 (an average minus a zero-sum
second-difference), so a constant mosaic → that constant on all three channels — an
invariant the oracle asserts independent of the exact weights.

## Inputs
- **Read / modify:** `src/demosaic.rs` — replace the interpolation in `demosaic_pixel`
  (or add an `adams_hamilton` path `demosaic_into` switches to); reuse `BayerPhase`,
  `reflect`, `sample`, `u32_to_u16`, the buffer contract.
- **Reuse:** `tests/demosaic_oracle.rs` + `tests/support/{tiff,perturb,pnm}.rs`.
- **Decisions:** `DEC-031` (HA not MHC + patent), `DEC-016/017/018`;
  `docs/provenance-ledger.md`.

## Outputs
- **`src/demosaic.rs`:** the Adams-Hamilton two-pass interpolation behind the
  **unchanged** `pub fn demosaic_into(sensor, &[u16], &mut [u16])` (interleaved
  `w*h*3`, caller-owned, typed errors, panic-free — SPEC-030 contract). `rnd4`/`rnd8`
  + clamp; green plane finalized to `u16` before R/B.
- **`tests/demosaic_oracle.rs`:** HA analytic fixtures (hand-computed, bit-exact) —
  interior + edges/corners (≥5×5), the classifier tie, a **directional** fixture, and
  DC-preservation; a perturbed-HA red-proof (mutant ≠ honest); RGGB + BGGR. Tier-A.
- **`docs/provenance-ledger.md`:** an Adams-Hamilton entry (US 5,629,734, expired;
  DEC-031) beside the SPEC-030 bilinear note.
- **`irr demosaic`** unchanged (now emits HA output).

## Acceptance Criteria
- [ ] `demosaic_into` produces Adams-Hamilton output: directional green (gradient-
  chosen axis + second-difference correction) then colour-difference red/blue,
  phase-driven by `Sensor.color_filter`
- [ ] Every pixel of ≥2 hand-built fixtures (interior, edges, corners) equals the
  hand-computed HA value **exactly**, including the classifier tie
- [ ] A **directional** fixture proves green follows the low-gradient axis — output
  differs from bilinear on an edge the way HA predicts
- [ ] DC-preservation: a constant mosaic → that constant on all three channels
- [ ] The oracle goes **red** on a perturbed HA step (flipped classifier, dropped
  correction term, or changed weight): mutant ≠ honest, honest control green
- [ ] Phase-driven: a BGGR fixture is correct with the same code (no hard-coded RGGB)
- [ ] No panic / typed errors on wrong lengths, absent/unsupported CFA, and degenerate
  dims (1×1, 2×2, 3×5); mono path + STAGE-006 oracles stay green
- [ ] Panic-free lints hold; the oracle is Tier-A (no corpus) so CI runs it

## Failing Tests
In **`tests/demosaic_oracle.rs`** (extending SPEC-030's):
- `"adams_hamilton_rggb_matches_hand_computed"` — full interleaved RGB == a
  hand-written expected table (interior + edges + corners).
- `"green_follows_the_low_gradient_axis"` — a strong-horizontal-gradient fixture:
  green at R/B sites interpolates vertically; asserts the directional value, distinct
  from bilinear's 4-neighbour average.
- `"classifier_tie_averages_both_predictors"` — a symmetric fixture (CLh == CLv).
- `"constant_mosaic_is_preserved_on_all_channels"` — DC-preservation.
- `"phase_is_driven_by_the_pattern_bggr"` — BGGR fixture, same code path.
- `"a_perturbed_hamilton_adams_step_turns_the_oracle_red"` + `"the_honest_control_is_green"`
  — red-proof; assert mutant ≠ honest (DEC-017, the `plane_oracle.rs` lesson).
- `"degenerate_dims_do_not_panic"` (1×1, 2×2, 3×5) + the SPEC-030 typed-error cases carry over.

## Non-Goals
- **Malvar-He-Cutler** / any `'505`-encumbered method (DEC-031) — not now, not as a
  later swap unless `'505` expiry is revisited in its own spec.
- White balance / colour matrix / ΔE (STAGE-008); levels / black-subtract / crop /
  orientation.
- Newer methods (AHD / AMaZE / LMMSE / RCD) — out of STAGE-007's scope (DEC-031 alt D).
- Non-2×2 / X-Trans CFA. Real-image dnglab comparison (deferred to STAGE-008 ΔE).

## Notes for the Implementer
- **L-risk / pre-named split** (from the frame): if full HA (green + R/B) exceeds
  [M], ship **green first** (Adams-Hamilton directional green, R/B left bilinear from
  SPEC-030 — a coherent, shippable quality increment), then R/B colour-difference as
  a follow-on spec. Green is the dominant quality lever.
- **Signed arithmetic + rounding are load-bearing for the bit-exact oracle.** `i32`
  intermediates; `rnd4(n)=(n+2).div_euclid(4)`, `rnd8(n)=(n+4).div_euclid(8)`; clamp
  `[0,65535]`. The green plane is finalized to `u16` before R/B reads it — pin the
  two-pass order.
- Reuse `reflect`/`sample` (offset-1-origin) for offset-2; add ≥5-wide fixtures that
  exercise offset-2 reflection in range, keep the degenerate no-panic tests.
- Red-proof: mutate a REAL HA step (flip `CLh < CLv`, or drop `(WW−2·C+EE)`) and
  assert the OUTPUT changes — not a no-op (`plane_oracle.rs` lesson, DEC-017).
- Provenance: Adams-Hamilton, US 5,629,734 (expired); record beside bilinear.

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
