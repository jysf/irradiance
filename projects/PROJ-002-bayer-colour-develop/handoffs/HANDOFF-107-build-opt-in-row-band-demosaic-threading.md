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
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
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

- **Branch / PR:** <branch> → PR #<n> (against `main`).
- **Completed at:** <YYYY-MM-DD>
- **All acceptance criteria met?** <yes/no + evidence per criterion — the
  byte-equality across N and orientations, the measured speedup, N<=1 unchanged>

### Cost self-report

- **Tokens (total):** <REAL number from `/cost`>
- **Estimated USD:** <tokens × rate>
- **Duration (minutes):** <n>
- **Source of the number:** <`/cost`, usage object, harness>

### Drift and new artifacts

- **New decisions emitted:** <none / DEC-NNN>
- **Deviations from spec:** <none / what and why — e.g. the threaded-vs-fallback
  predicate, the band-count clamping>
- **Follow-up work identified:** <none / e.g. a transposed-colour gather spec>

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>
