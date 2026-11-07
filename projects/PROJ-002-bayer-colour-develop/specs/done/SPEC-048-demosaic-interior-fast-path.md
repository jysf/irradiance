---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-048
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M              # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved         # approved | punch-list | rejected — the OUTCOME of the verify
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
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 35
      recorded_at: 2026-11-07
      notes: "Main-session design (un-metered; no /cost meter in the Claude Code Code tab). Formalized DEC-041 (row-wise interior fast path over per-row slices, border kept reflecting, kernel arithmetic verbatim so output is bit-identical) plus the STAGE-012 perf discipline; wrote the equivalence/bit-identity failing test, HANDOFF-101 and the build prompt."
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 12177062
      estimated_usd: 29.99
      duration_minutes: 40
      recorded_at: 2026-11-07
      notes: "Built whole, APPROVED by verify first pass (HANDOFF-101, PR #142, b760613). 24 MP median demosaic 533.0 → 316.2 ms (~1.69x), bit-identical (full-buffer checksum unchanged, equivalence test + golden hashes green). Recovered 12,177,062 tok / $29.99 (claude-opus-4-8, from transcript 7c0299e2, 57 deduped assistant messages, 97.4% cache reads); the handback was a floor, superseded by the deduped transcript sum."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 11812130
      estimated_usd: 25.07
      duration_minutes: 45
      recorded_at: 2026-11-07
      notes: "Verdict APPROVED at b760613 (HANDOFF-102, PR #143). Proved the equivalence test non-tautological by injecting an interior off-by-one and watching it go red; all gates green (just test 425/0 with the Tier-B corpus). Recovered 11,812,130 tok / $25.07 (claude-opus-4-8, from transcript ae01a0bd, 61 deduped assistant messages, 97.9% cache reads); the HANDOFF-102 handback 217,733 was a context-window floor, superseded."
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-11-07
      notes: "Orchestrator ship bookkeeping (un-metered main session). Recovered the build and verify true costs from their transcripts, wrote the reflection and totals, archived the spec, flipped the STAGE-012 backlog, and logged the brag + changelog."
  totals:
    tokens_total: 23989192
    estimated_usd: 55.06
    session_count: 4
shipped_at: 2026-11-07
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
   — Little about the spec itself: framing the perf change as output-invariant
   (an equivalence proof against a reflecting reference + the golden hashes) made
   it low-risk and it was APPROVED first pass. The one friction was process, not
   code — each docs PR in this chain hit a squash-merge conflict because the
   branch carried the prior PR's pre-squash commit; basing each new cycle's
   branch on a freshly-updated `main` (which this ship did) avoids it.

2. **Does any template, constraint, or decision need updating?**
   — No new rule. `DEC-041` captured the fast-path structure and the STAGE-012
   perf discipline (bit-identical + measured + a DEC per optimisation), which now
   governs P2 and P3.

3. **Is there a follow-up spec I should write now before I forget?**
   — Already in the `STAGE-012` backlog: P2 (fuse `develop_color_into`'s scratch
   buffers + add `develop_color_f32_into`), the colour-geometry crop follow-up,
   and P3 (opt-in threading).

4. **Where was the worst defect caught?**
   — none
   *(APPROVED on the first verify pass; the verify's adversarial interior
   off-by-one confirmed the equivalence test is load-bearing rather than finding
   a real defect.)*

5. **What can a user do now that they couldn't before?**
   — Develop about 1.69× faster: 24 MP median `demosaic_into` fell from 533.0 ms
   to 316.2 ms, with byte-identical output (the full-buffer checksum is unchanged).
