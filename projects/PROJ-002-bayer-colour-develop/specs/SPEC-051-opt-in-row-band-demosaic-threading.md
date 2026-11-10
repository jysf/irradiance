---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-051
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

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-107 build (cold session; records its real model at handback)
  created_at: 2026-11-09

references:
  decisions: [DEC-044, DEC-041, DEC-042, DEC-043] # DEC-044 = the threading model (NEW, this spec's design: output-row-band std::thread::scope, transposed-colour single-thread fallback); DEC-041 = STAGE-012 perf discipline; DEC-042 = fused path + rolling cache; DEC-043 = SPEC-050 sink-side crop/orient scatter (the threading complication).
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation, no-new-top-level-deps-without-decision, one-spec-per-pr, provenance-recorded-per-algorithm]
  related_specs: [SPEC-048, SPEC-049, SPEC-050] # the demosaic fast path, the fuse, and the crop/orient scatter this threads over

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-050]             # the colour-path threading must account for SPEC-050's output scatter (shipped)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-012's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-012's optional P3: an opt-in per-image demosaic speedup for batch and Python-binding consumers, output bit-identical, with no new dependency or unsafe — the last item before STAGE-012 can stage-ship."

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
  tokens_estimate: 16000000        # ~M: threaded entry points + output-band decomposition + bit-identity across N and orientations; the transposed-colour fallback adds branching over a pure demosaic-only thread
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null           # orchestrator main-loop session spans many specs' design/ship cycles; a per-cycle split is not observable (DEC-013: a null here is honest, a guess is not)
      estimated_usd: null
      duration_minutes: 45
      notes: "Design of SPEC-051: DEC-044 (output-row-band std::thread::scope threading — each thread works backwards from its output band to its source rows; colour path threads non-transposing orientations and falls back to single-thread for the 90 degree rotations; no new buffer, no unsafe, no DEVELOP_VERSION bump), HANDOFF-107 build, the build prompt, the byte-equality + determinism failing-test set, cycle advanced to build."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-051: opt-in row-band demosaic threading

## Context

P3 of the pyirradiance/LibRaw benchmark that opened `STAGE-012` (and the external
0.2.0 staff review §5) — the stage's last, **optional** backlog item, after the
correctness arc closed (`SPEC-047` colour, `SPEC-048` speed, `SPEC-049` memory,
`SPEC-050` geometry). `SPEC-048` gave the demosaic an interior fast path (~1.69×)
and `SPEC-049` fused the colour develop; both run single-threaded. The demosaic /
colour-develop interpolation is embarrassingly parallel across output rows — a
fixed-radius kernel with no cross-row accumulation beyond the ±2 halo — so
splitting it into row bands across a few threads is the remaining per-image
speedup for the Python bindings and batch consumers. It must stay **opt-in**
(default 1 thread, single-thread output *and* performance unchanged),
**bit-identical**, **deterministic**, and — like the whole crate — add **no
dependency and no `unsafe`** (`std::thread::scope` only). Shipping it lets
`STAGE-012` stage-ship.

## Goal

Add opt-in row-band parallelism to the demosaic hot loop via `std::thread::scope`,
with a caller-specified thread count that defaults to 1 (the current sequential
path, byte-for-byte). With N > 1 the output is byte-identical to N = 1 for every
Bayer phase and geometry, and deterministic regardless of N and of thread
scheduling; a large image demosaics measurably faster.

## Inputs

- **Files to read:** `src/demosaic.rs` (`demosaic_into` ~232 — writes a contiguous
  `width·height·3` plane, input row-band ↔ output row-band 1:1, so it is the clean
  threading target; `develop_color_fused` ~458 — the fused colour traversal whose
  `SPEC-050` sink scatters to crop/orient output positions), `src/color.rs`
  (`develop_color_into` / `develop_color_f32_into` sinks), `src/develop.rs`
  (`output_dimensions`, `crop_source_coords`, the `SPEC-050` output scatter),
  `guidance/constraints.yaml`.
- **Prior decisions:** `DEC-041` (the STAGE-012 perf discipline — bit-identical
  proof + measured before/after + a DEC per optimisation), `DEC-042` (the fused
  path + rolling cache), `DEC-043` (the sink-side crop/orient scatter — the key
  threading complication). **`DEC-044` is NEW**, written at this spec's **design**.

## Outputs

- **Threaded entry points (`DEC-044`):** three additive siblings, signature shape
  `(sensor, src, dst, threads: usize)` — `demosaic_into_threaded`,
  `develop_color_into_threaded`, `develop_color_f32_into_threaded`. `threads <= 1`
  dispatches straight to the existing single-thread function (unchanged signatures,
  no banding). `threads >= 2` splits the **output** rows into that many contiguous
  bands over `std::thread::scope`, each thread owning a disjoint `dst` slice.
- **Files modified:** `src/demosaic.rs`, `src/color.rs`, `src/lib.rs` (export the
  three functions). **No `DEVELOP_VERSION` bump** — threading is output-invariant,
  so the develop contract does not change.
- **New decision:** `DEC-044` (written at this design) — output-row-band
  `std::thread::scope` threading; each thread works backwards from its output band
  to the source rows it needs; the colour path threads for non-transposing
  orientations and **falls back to single-thread for the 90° rotations**.

## Acceptance Criteria

- [ ] With `threads = N` (N > 1), the threaded function's output is **byte-identical**
  to `threads = 1`, across all four Bayer phases (and, if the colour path is
  threaded, all geometries/orientations including a 90° rotation).
- [ ] `threads = 1` is the exact current sequential path — signatures, output and
  performance unchanged (no banding overhead at N = 1).
- [ ] **Deterministic:** output independent of N and of thread scheduling — no
  cross-thread reduction of the `f64` matrix; each pixel is computed in the same
  order as the single-thread path.
- [ ] A **measured speedup** on a large image (record `threads = 1` vs N ms in
  `docs/provenance-ledger.md`, per `DEC-041`).
- [ ] No new dependency, no `unsafe`, no panic on untrusted input, determinism held.

## Failing Tests

*Formalised at design (`DEC-044`); the build writes them in full, in `tests/`.
Each is a `dst_threaded == dst_single` byte-equality (the `DEC-041` proof), red
until the threaded path exists.*

- `"threads_one_equals_the_single_thread_entry"` — `demosaic_into_threaded(.., 0)`
  and `(.., 1)` are byte-identical to `demosaic_into`, and likewise for the two
  colour entries; the default dispatches to the existing code with no banding.
- `"threaded_demosaic_equals_single_threaded"` — `demosaic_into_threaded(.., N)`
  byte-identical to `demosaic_into` for N in {2, 3, 4}, across all four Bayer
  phases, on a tall image, including a band split that lands a boundary on an
  interior row (so the ±2 halo is exercised across the seam).
- `"threaded_colour_develop_non_transposing_equals_single_threaded"` —
  `develop_color_into_threaded(.., N)` byte-identical to `develop_color_into` for
  **orientations 1–4** (and a non-trivial `ActiveArea`/`DefaultCrop`), N in {2, 4},
  across Bayer phases.
- `"threaded_colour_develop_transposing_falls_back_and_still_matches"` — for
  **orientations 5–8** (the 90° rotations), `develop_color_into_threaded(.., N)`
  with N > 1 is still byte-identical to `develop_color_into` (the single-thread
  fallback keeps correctness even though it does not parallelise).
- `"threaded_f32_equals_single_threaded"` — `develop_color_f32_into_threaded(.., N)`
  byte-identical to `develop_color_f32_into` for N in {2, 4}.
- `"output_is_independent_of_thread_count"` — the threaded output is identical
  across N in {1, 2, 3, 5, 8} (determinism regardless of band count or scheduling).
- `"band_boundary_reflects_at_the_true_edge"` — with a band boundary forced onto an
  interior row, boundary-row pixels equal the single-thread values (the halo reads
  real neighbour rows across the seam; reflection happens only at the true image
  edges, never at a band edge).

*(Not a unit test, per the acceptance criteria: record a `threads=1` vs N
wall-time on a large image in `docs/provenance-ledger.md`, per `DEC-041`.)*

## Non-Goals

- `rayon` or any threadpool / parallelism **dependency**; `unsafe`; SIMD.
- Auto-detecting the thread count (`num_cpus` etc.) — the caller specifies it; a
  sensible default helper can be a later spec.
- Changing the single-thread output, the colour maths (`DEC-038`), or the kernel.
- **Re-introducing an active-area or full-plane RGB intermediate** to make the
  `SPEC-050` colour scatter threadable — that undoes `SPEC-049`/`SPEC-050`'s memory
  win. If the scatter can't be threaded cleanly without `unsafe` or a buffer,
  thread `demosaic_into` and **defer the colour-path scatter threading to its own
  spec** (record the decision in `DEC-044`).
- Threading anything but the demosaic / colour-develop hot loop.

## Notes for the Implementer

- **Partition the OUTPUT, not the input (`DEC-044`).** `std::thread::scope` borrows
  `src` **immutably** (shared) and `dst` **mutably as DISJOINT slices**
  (`split_at_mut` / `chunks_mut`) — the borrow checker enforcing no overlap *is* the
  bit-identity guarantee. Each thread works backwards from its output band to the
  source rows it needs, runs the **existing single-thread traversal** over them
  (local green + rolling cache), and **reflects at the true image edges** (never a
  band edge), so a band boundary never changes a pixel.
- **`demosaic_into_threaded`** is the clean case — output rows ↔ source rows 1:1,
  so output band `[oy0, oy1)` needs source rows `[oy0-2, oy1+2)`. Each thread uses a
  **local** green buffer (the single-thread path keeps green in `dst`; the threaded
  one can't — a neighbour would be writing red/blue into the green it reads).
- **The colour path splits by orientation.** Non-transposing (1–4): an output band
  is fed by a contiguous source band — thread it. Transposing (5–8, the 90°
  rotations): an output band is fed by source *columns* across all rows — **fall
  back to single-thread** (correct, not parallel). Resolve orientation from
  `resolve_geometry` once, before spawning.
- **No new persistent buffer:** the per-band local green sums to ~one
  `width·height` plane (the single-thread plane, partitioned). **Never** add an
  active-area or full-plane RGB intermediate (undoes `SPEC-049`/`SPEC-050`).
- `threads <= 1` must be the literal current code path (no banding, no alloc). Keep
  zero-dependency, no-`unsafe`, no-panic, determinism; no `DEVELOP_VERSION` bump.

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
