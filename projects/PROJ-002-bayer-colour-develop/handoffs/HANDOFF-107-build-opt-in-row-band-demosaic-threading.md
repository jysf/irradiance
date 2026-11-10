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
  id: HANDOFF-107
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-09
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-051

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 8783063            # REAL combined count — what cost-audit reads
  estimated_usd: 6.88
  duration_minutes: 25
  branch: claude/spec-051-row-band-threading
  pr: null
  completed_at: 2026-11-09
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id; 8,538,715 of it cache reads) at handback time — /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the few final PR-opening turns."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-107: opt-in row-band demosaic threading

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-051` to the implementer for the
**build** cycle: add opt-in parallelism to the demosaic / colour-develop hot loop
(`DEC-044`) — three additive `*_threaded` entry points that split the **output**
into row bands over `std::thread::scope`, each thread owning a disjoint `dst`
slice. `threads <= 1` is the existing single-thread path byte-for-byte; `threads >=
2` is **byte-identical** to it and deterministic. `demosaic_into_threaded` threads
every input; the colour path threads for non-transposing orientations and **falls
back to single-thread for the 90° rotations**. No dependency, no `unsafe`, no
`DEVELOP_VERSION` bump (threading is output-invariant).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-051-opt-in-row-band-demosaic-threading.md` — Acceptance Criteria, Failing Tests and Notes are the contract.
- **Decision:** `./decisions/DEC-044-opt-in-output-row-band-threading-for-the-demosaic-hot-loop.md` — **this is the design.** Output-row-band scope threading; work backwards from each output band to its source rows; the non-transposing/transposing colour split; the memory + determinism + safety rules.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, runtime globals. Read it (DEC-004 rule 5).
- **Code:**
  - `src/demosaic.rs` — `demosaic_into` (~232, the single-thread mono demosaic,
    green kept in `dst`'s green slots) and `develop_color_fused` (~458, the fused
    colour traversal: pass 1 green into a `w·h` plane, pass 2 red/blue + the sink's
    matrix, rolling normalise cache). The threaded paths run **this same traversal
    per band** over a band's source rows with a **local** green buffer — do not
    change the single-thread kernels.
  - `src/color.rs` — `develop_color_into` (~265) / `develop_color_f32_into` (~332)
    and their sink (the `SPEC-050` crop/orient scatter). The threaded colour entries
    wrap these.
  - `src/develop.rs` — `resolve_geometry` (~135, resolve the `Orientation` once to
    pick threaded-vs-fallback), `output_dimensions` (~230), `crop_source_coords`
    (~249) / `output_coords_from_crop` (the `SPEC-050` maps).
  - `src/lib.rs` — export the three `*_threaded` functions.

### Decisions that apply

- `DEC-044` — **this is the spec.** Output bands; `std::thread::scope`; disjoint
  `dst` slices; local per-band green; non-transposing colour threaded, transposing
  colour single-thread fallback; `threads <= 1` = current path; no `unsafe`, no dep,
  no `DEVELOP_VERSION` bump.
- `DEC-041` — STAGE-012 perf discipline: a `dst_threaded == dst_single` proof +
  a measured `threads=1` vs N wall-time in `docs/provenance-ledger.md`.
- `DEC-042` (fused path + rolling cache) and `DEC-043` (the `SPEC-050` sink scatter)
  — reused unchanged; the per-band traversal is the same code.
- `DEC-016` (caller-owned buffers) / `DEC-018` (quantise) — unchanged.

### Constraints that apply

See `./guidance/constraints.yaml`. These apply:

- `oracle-must-be-shown-red` / `test-before-implementation` — write the byte-equality
  tests first; each must be red before the threaded path exists.
- `no-panics-on-untrusted-input` — validate geometry once (typed `resolve_geometry`
  errors) before spawning; `std::thread::scope` re-raises a thread panic on join,
  but the kernels are already panic-free, so add no new panic path.
- `no-new-top-level-deps-without-decision` — **zero** new dependencies; no `rayon`,
  no threadpool, no SIMD, no `unsafe`.
- `provenance-recorded-per-algorithm` — the ledger row (`DEC-041`).
- `one-spec-per-pr` — SPEC-051 only.

### Prior related work

- `SPEC-048`/`DEC-041` (demosaic fast path), `SPEC-049`/`DEC-042` (fuse + rolling
  cache + `develop_color_f32_into`), `SPEC-050`/`DEC-043` (crop/orient sink scatter)
  — all shipped, all reused byte-for-byte by the per-band traversal. `depends_on`
  SPEC-050.
- The pyirradiance rawpy/LibRaw benchmark (`docs/reviews/`) raised P3.

## Expected Deliverables

- The three `*_threaded` entry points and the byte-equality + determinism tests
  passing; the existing single-thread goldens unmoved.
- A measured `threads=1` vs N wall-time on a large image in
  `docs/provenance-ledger.md` (a real speedup for `N >= 2`).
- A PR against `main` from a `claude/spec-051-*` branch, referencing HANDOFF-107,
  SPEC-051, STAGE-012, PROJ-002, DEC-044 (and DEC-041).

### If `cycle: verify` — the verify contract

n/a (build cycle).

## Out of Scope

- **Any change to the single-thread output or the single-thread kernels** — the
  threaded output must equal them byte-for-byte; if a single-thread result moves,
  stop.
- `rayon` or any threadpool/parallelism **dependency**; `unsafe`; SIMD.
- Auto-detecting the thread count (the caller passes `threads`).
- A transposed-orientation colour **gather** to parallelise the 90° rotations — the
  fallback is single-thread; threading those is a possible later spec.
- Any `DEVELOP_VERSION` bump (threading is output-invariant) or colour-maths change.
- Re-introducing an active-area or full-plane RGB intermediate.

## Return Criteria — how to hand back

**Completing this handoff means filling the `handback:` front-matter (including a
real `tokens_total`) AND the `## Handback` section.**

On success:
1. Fill `handback:` — real `tokens_total`.
2. Fill `## Handback` (the reflection is part of it).
3. `handoff.status` → `completed`, `handback.status` → `completed`.
4. Open a PR (build).

If blocked: fill `## Handback` with what blocked you, set `handoff.status` →
`rejected` / `handback.status` → `blocked`, still report tokens, set the spec's
`task.blocked: true`, add a question to `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator transcribes it; it does not
reconstruct it.*

### Execution notes

- **Branch / PR:** `claude/spec-051-row-band-threading` → PR #__PR__ (against `main`).
- **Completed at:** 2026-11-09
- **All acceptance criteria met?** yes.
  - *Byte-identical for N > 1, all phases and geometries:* `tests/threaded.rs`
    compares every threaded output with the existing single-thread function —
    demosaic N in {2,3,4} × 4 phases × 7 sizes (tall 9×37 / 13×61, 20×24, a
    3-wide, 6×2, 2×2, 1×1) × even and odd ActiveArea origin; colour u16
    N in {2,4} × orientations absent/1–4 × 4 phases on an odd-top/odd-left
    ActiveArea + DefaultCrop fixture, plus the whole plane; orientations 5–8
    (the fallback) N in {2,3,4}; f32 N in {2,4} × orientations 1–4 and 6, by
    bit pattern. All 8 tests were red against no-op stubs first (one,
    `output_is_independent_of_thread_count`, initially passed vacuously against
    the stub — its baseline is now the single-thread render, and it went red).
  - *N = 1 unchanged:* `threads <= 1` is a literal call to the existing function
    (`threads_one_equals_the_single_thread_entry` pins 0 and 1); the
    single-thread kernels and functions are untouched (the diff only adds code).
  - *Deterministic:* `output_is_independent_of_thread_count` (N in
    {1,2,3,5,8}, each run twice) and the band edges are a pure function of
    `(rows, threads)`; no cross-thread reduction.
  - *Seams:* `band_boundary_reflects_at_the_true_edge` puts a seam on odd
    interior row 7, makes every row its own band (N = rows), and over-asks
    (N = 50 > rows), on the demosaic and through a vertical flip on colour.
    Mutation check: shrinking the halo to 1 row, or dropping the even-row
    rounding of the halo top, turns 5 of the 8 tests red.
  - *Measured speedup (ledger row, DEC-041; median of 7, M4 Pro):* demosaic
    6000×4000 334.5 ms → 182.5 / 101.4 / 89.3 ms at N = 2/4/8 (≈1.83× / 3.30× /
    3.75×); colour u16 5712×4096 orientation 1 412.5 ms → 211.6 / 111.9 /
    70.5 ms (≈1.95× / 3.69× / 5.85×); f32 similar; orientation 6 unchanged
    (fallback). Every N's buffer equal to N = 1 in the harness.
  - *No dependency, no `unsafe`, no panic:* `Cargo.toml`/`Cargo.lock`
    untouched; `#![forbid(unsafe_code)]` + the five-lint deny policy hold
    (`just lint-ci` green). Inputs are validated before spawning (any input the
    banded path does not take goes to the single-thread function, so the typed
    errors are identical by construction — `threaded_errors_match_the_single_thread_errors`);
    threads are spawned with `Builder::spawn_scoped` (an error, not a panic, if
    the OS refuses) and the calling thread drains the same band queue, so a
    refused spawn only means fewer workers.
  - Gates: `just test`, `just lint-ci`, `just fmt-check`, `just validate`,
    `just cost-audit` green.

### Cost self-report

- **Tokens (total):** 8,783,063 (input 95 + cache write 186,628 + cache read 8,538,715 + output 57,625)
- **Estimated USD:** ≈ $6.88
- **Duration (minutes):** ~25
- **Source of the number:** this session's own transcript
  (`~/.claude/projects/<worktree>/164f1a9f-….jsonl`, anchored on the
  scratchpad UUID), summing every assistant usage record (input + cache write +
  cache read + output, deduped by message id) — `/cost` is not callable as a
  tool in a delegated session. USD at Opus list rates.

### Drift and new artifacts

- **New decisions emitted:** none (DEC-044 implemented as written).
- **Deviations from spec:**
  - *How a band "runs the existing traversal":* rather than duplicate the
    kernel loops band-aware, each band runs the unchanged fused traversal
    `develop_color_fused` over a **local view** of the plane — a clone of the
    `Sensor` with `height` set to the band's rows plus a 3-row halo, a
    normaliser offset by the halo's top row, and a local green buffer — and its
    sink keeps only the band's own pixels. The halo top is rounded down to an
    even row so each local row keeps its absolute Bayer parity. This reuses the
    single-thread code path byte for byte with zero changes to it.
  - *`demosaic_into_threaded` bands run the fused traversal (identity
    normaliser), not `demosaic_into` itself.* `demosaic_into` keeps green in
    `dst` and would need a local RGB buffer per band to be run on a sub-plane;
    the fused traversal is already proven equal to `demosaic_into`
    (`fused_develop_color_is_byte_identical_to_the_reference`) and needs only the
    local green DEC-044 budgets. Cost: per-band throughput is a little lower
    than `demosaic_into` (2 threads ≈1.83×, not 2×).
  - *Band-count clamping:* bands = `min(threads, rows)`; fewer than two output
    rows (or a non-banded input) goes to the single-thread function.
  - *A work queue instead of one thread per band:* bands sit in a
    `Mutex<Vec<…>>` drained by `threads − 1` spawned workers plus the calling
    thread — the no-panic answer to a refused spawn (`thread::scope`'s plain
    `spawn` panics there). Output does not depend on which thread takes which band.
  - One extra test beyond the spec's seven: `threaded_errors_match_the_single_thread_errors`.
- **Follow-up work identified:** a transposed-orientation colour gather to
  thread portrait (5–8) images, if that case proves worth it (DEC-044 names it).
  A thread-count default helper (e.g. `available_parallelism`) for callers, also
  named in the spec's non-goals. Neither is needed for STAGE-012 to ship.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — Only
   "run the existing single-thread traversal per band": the kernels index the
   whole plane absolutely, so it took a moment to see that a height-shortened
   `Sensor` view plus an offset normaliser runs them unchanged — and that the
   view's top must sit on an even row to keep the CFA phase.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That `std::thread::scope`'s `spawn` panics if the OS refuses a thread;
   under `no-panics-on-untrusted-input` that pushes you to `Builder::spawn_scoped`.
   Worth a line in DEC-044.
3. **If you did this task again, what would you do differently?** — Make every
   byte-equality test's baseline the single-thread function from the start: a
   threaded-vs-threaded comparison passes against a stub that writes nothing.
