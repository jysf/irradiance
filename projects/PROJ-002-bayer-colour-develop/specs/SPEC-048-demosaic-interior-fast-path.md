---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-048
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  stage: STAGE-012
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-101 (build); tier_map.build
  created_at: 2026-11-06

references:
  decisions: [DEC-041, DEC-032, DEC-016] # DEC-041 = the interior fast path + the STAGE-012 perf discipline (this spec's decision); DEC-032 = the demosaic (zero-alloc two-pass green); DEC-016 = caller-owned buffers.
  constraints: []                  # design fills exact ids: no-panic, determinism, no-new-deps
  related_specs: [SPEC-047]        # same colour path (ships first)

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-012's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-012's performance goal: a materially faster demosaic with bit-identical output, so a develop is cheap enough to run per image."

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

# SPEC-048: demosaic interior fast path

## Context

Demosaic dominates develop time. On the 23.4 MP Canon the pyirradiance
rawpy/LibRaw benchmark measured `develop` at ~587 ms, of which unpack is ~1 ms
and `demosaic_into` is **~520 ms** (LibRaw's whole develop is 477 ms). Per
pixel, `demosaic_into` does a `checked_div` + `checked_rem` to recover `(x, y)`,
and each of its ~9 neighbour reads goes through `sample()` → `reflect()` (i64
saturating maths) → `plane_index()` → a bounds-checked `get`. Edge reflection
only matters for the **2-pixel border**; the entire interior pays for machinery
it never needs. This is the staff review's §5 and the pyirradiance report's P1.
Second spec of `STAGE-012`, after the P0 correctness fix (`SPEC-047`) shipped.

## Goal

Give `demosaic_into` a row-wise **interior fast path**: handle the 2-pixel
border with the current reflecting code, and the interior with direct per-row
offsets and no reflection or per-read bounds checks. The arithmetic is
unchanged, so the output is **bit-identical** — the existing golden hashes are
the acceptance test — and the loop gets materially faster.

## Inputs

- **Files to read:** `src/demosaic.rs` (`demosaic_into` and the `green_at_site`
  / colour-site helpers, `sample` / `sample_i32` / `reflect` / `plane_index`,
  and the per-pixel `checked_div`/`checked_rem`), `src/color.rs`
  (`develop_color_into` — the caller), `tests/demosaic*` / the golden fixtures.
- **Reference only:** the pyirradiance benchmark (`bench/README.md` and the
  report) for the measured split; LibRaw is the throughput yardstick, not linked.

## Outputs

- **Files modified:** `src/demosaic.rs` — an interior fast path over per-row
  slices (`src[row-2..=row+2]`), the border kept on the current reflecting path.
- **No change to:** the public signatures, the demosaic kernel/arithmetic, the
  output bytes, or dependencies.
- **New decision:** `DEC-041` — the interior fast-path structure, and the perf
  discipline every STAGE-012 optimisation follows: a `dst_after == dst_before`
  proof plus a before/after ms (and peak-RSS where relevant) line in
  `docs/provenance-ledger.md`. Written at this spec's **design** cycle.
- **Provenance ledger:** a before/after timing row.

## Acceptance Criteria

- [ ] `demosaic_into` output is **bit-identical** to the pre-change implementation
  for every Bayer phase (RGGB, BGGR, GRBG, GBRG) and across sizes that exercise
  the border and the interior — the existing golden hashes pass unchanged.
- [ ] A measured **before/after** wall-time for `demosaic_into` on a
  representative frame is recorded in `docs/provenance-ledger.md` (the speedup is
  the point; the exact factor is whatever it is).
- [ ] The fast path is **panic-free** — the per-row slices make the interior
  reads in-bounds by construction, without per-read `get` checks — under
  `#![forbid(unsafe_code)]` and the five-lint deny policy.
- [ ] Determinism held; no new dependency; no public API change.

## Failing Tests

Written during the **design** cycle. Because the change is output-invariant, the
"test" is an equivalence/invariance gate rather than a red→green feature test.

- **`tests/demosaic*` (and the golden hashes)**
  - `"interior_fast_path_matches_the_reference_demosaic"` — asserts the fast-path
    output equals a straightforward reflecting reference, byte-for-byte, over all
    four Bayer phases and a size with a non-trivial interior AND border (the
    bit-identity proof, `dst_after == dst_before`).
  - The existing demosaic golden-hash tests continue to pass unchanged.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- Any change to the demosaic kernel or its arithmetic, or to the output bytes.
- Colour-develop memory / the f32 entry point (P2) and threading (P3) — separate
  STAGE-012 specs.
- The colour-geometry crop follow-up — separate STAGE-012 spec.
- SIMD or unsafe — the fast path stays safe (slices), zero-dependency.

## Notes for the Implementer

- **The shape (DEC-041, at design):** iterate row by row; for interior rows and
  columns (≥ 2 from every edge) read neighbours by direct index into per-row
  slices (`src[row-2..=row+2]`), skipping `reflect()` and the per-read
  bounds-checked `get`; keep the current reflecting path for the 2-pixel border.
  The slices make the interior in-bounds by construction, so it stays panic-free
  without per-element checks.
- **Output must not move.** Same integer operations in the same order — the
  existing golden hashes are the gate, and the equivalence test is the proof.
  This is the STAGE-012 perf discipline (staff review §5): bit-identical + a
  measured before/after + its own `DEC-*`.
- Keep the zero-dependency, no-panic, determinism rules. LibRaw/rawpy are the
  benchmark yardstick in pyirradiance — reference only, never linked.

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
