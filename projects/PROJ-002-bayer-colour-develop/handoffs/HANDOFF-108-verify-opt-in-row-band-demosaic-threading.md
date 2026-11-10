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
  id: HANDOFF-108
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-09
  status: completed                  # pending | accepted | completed | rejected

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
  tokens_total: 3917704               # REAL combined count — what cost-audit reads
  estimated_usd: 3.12              # tokens_total × your rate, or your harness's number
  duration_minutes: 35
  branch: claude/spec-051-verify
  pr: 158
  completed_at: 2026-11-09               # YYYY-MM-DD
  notes: "verify APPROVED at 7ee2a86; tokens summed from this session's own transcript usage records (claude-opus-5-5), /cost not callable as a tool in a delegated session; excludes the final commit/PR turns"                      # one line if unusual (rework, no meter, etc.)
  synced_at: 2026-11-09            # cost recorded at ship from transcript df90e54b (true 5,273,738 tok, floor 3,917,704 superseded); marked synced so handback-sync does not re-transcribe the floor
---

# HANDOFF-108: opt-in row-band demosaic threading (verify)

## Delegation Summary

`claude-opus-5` (acting as `architect`) hands `SPEC-051` to `claude-opus-5`
(acting as `verifier`) for the **`verify`** cycle: a cold, adversarial review of
the build in PR #157 (`HANDOFF-107`).

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-051-opt-in-row-band-demosaic-threading.md`
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-107-build-opt-in-row-band-demosaic-threading.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-044` — the threading model under review: output-row-band
  `std::thread::scope`, disjoint `dst` slices, colour 90° rotations fall back.
- `DEC-041` — STAGE-012 perf discipline: bit-identical proof + measured
  before/after in the ledger.
- `DEC-042` — the fused colour traversal + rolling cache each band reuses.
- `DEC-043` — the `SPEC-050` sink-side crop/orient scatter the colour bands map
  through.

### Constraints that apply

- `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `no-new-top-level-deps-without-decision`,
  `one-spec-per-pr`, `provenance-recorded-per-algorithm`.

### Prior related work

- `HANDOFF-107` / PR #157 — the build under review.
- `HANDOFF-106` / PR #153 — the SPEC-050 verify this follows.

## Expected Deliverables

### The verify contract

Review cold; run the gates on the PR branch; attack byte-identity, disjoint
writes, determinism, halo/seam correctness, the orientation predicate,
panic-freedom and the forbidden means (dependency, `unsafe`, SIMD, RGB
intermediate, `DEVELOP_VERSION` bump). Output exactly ONE verdict.

## Out of Scope

- Merging PR #157 or advancing `SPEC-051`'s cycle (the orchestrator's job).
- Fixing anything: findings go in the punch list or the follow-ups below.

## Return Criteria — how to hand back

Fill the `handback:` front-matter (real `tokens_total`) and the `## Handback`
section; set both statuses to `completed`; open the verify PR.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-051-verify` → PR #158 against `main` (this
  file is the only change). Reviewed PR #157 at head **`7ee2a86`** (branch
  `claude/spec-051-row-band-threading` — the brief named it
  `claude/adoring-kilby-565b10`, which is not the PR's branch).
- **Completed at:** 2026-11-09
- **All acceptance criteria met?** yes.
- **Verdict:** ✅ **APPROVED** (at `7ee2a86`).

#### Gates (run by me on the PR head, not taken from the handback)

| Gate | Result |
|---|---|
| `just test` | green — 442 passed, 0 failed (incl. the 8 in `tests/threaded.rs`) |
| `just lint-ci` | green |
| `just fmt-check` | green |
| `just validate` | green (55 artifacts / 229 front-matter files) |
| `just cost-audit` | green |
| `just decisions-audit --changed main` | advisory only — DEC-041/042/043/044 govern the touched files; no contradiction found (see drift below) |
| PR #157 CI | all 13 checks green (fmt, clippy, fuzz-graph clippy, test, cargo-deny ×2, fuzz smoke ×2, MSRV 1.90, both red-proofs, panic-free policy) |

#### Per-criterion findings

1. **Byte-identity for N > 1, all phases and geometries — MET.**
   `tests/threaded.rs` compares every threaded output against the real
   single-thread functions (`demosaic_into`, `develop_color_into`,
   `develop_color_f32_into`), never threaded-vs-threaded, into buffers
   pre-filled with a sentinel (`0xA5A5` / `-1.0`) so an unwritten band cannot
   pass; f32 is compared by bit pattern. Coverage: 4 Bayer phases; demosaic
   N ∈ {2,3,4} × 7 sizes (tall, odd, 3-wide, 6×2, 2×2, 1×1) × even and odd
   ActiveArea origin; colour u16 orientations absent/1–4 on an odd-top
   (3), odd-left (1) ActiveArea + DefaultCrop fixture (crop rows start at 4, so
   N=2/4 seams land on source rows 19 and 11/19/27 — odd interior rows) plus
   the whole plane; orientations 5–8 at N ∈ {2,3,4}; f32 for 1–4 and 6; N > rows
   (N = 50 on 15 rows) and one-row bands (N = 15).
   **Independent real-file check** (temporary, uncommitted example, release
   build): Canon 5D3 `5G4A9394.DNG` (5920×3950, ActiveArea (80,122)) and Nikon
   D300S `_DSC4615.DNG` (4352×2868). Demosaic at N ∈ {2,3,4,7,8,16,64} × 3 runs
   each and colour u16 + f32 at N ∈ {2,3,5,8,13} × 2 runs for the file's
   orientation and every orientation 1–8: **0 mismatches** anywhere.
2. **`threads <= 1` is the literal single-thread path — MET.** All three entries
   open with `if threads <= 1 { return <single-thread fn>(..) }` — no scope, no
   banding, no allocation. The diff only adds code: `demosaic_into`,
   `develop_color_into`, `develop_color_f32_into`, the fused kernel and
   `DEVELOP_VERSION` are untouched (`src/develop.rs` not in the diff).
3. **No overlapping writes — MET.** `run_row_bands` carves `dst` (as RGB
   triples) into `min(threads, rows)` bands with `split_at_mut_checked`, each
   pushed once into a queue, so the borrow checker proves disjointness. Within a
   band, writes go through `band.get_mut(slot − first)`, so a halo pixel (or any
   pixel mapping outside the band) is dropped rather than written elsewhere; the
   `keep` row filter in `develop_rows_banded` is a second guard.
4. **Determinism — MET.** Band edges are `⌊rows·i/bands⌋` (pure function of
   `(rows, N)`); each pixel is computed by the same fused kernel and the same
   `f64` matrix with no cross-thread reduction. Bands are popped from a
   `Mutex` queue in scheduling-dependent order, but each owns a disjoint slice,
   so the order cannot affect bytes — confirmed by the repeated real-file runs
   above and `output_is_independent_of_thread_count` (N ∈ {1,2,3,5,8}, twice
   each, baseline = the single-thread render). Errors are reported by band index
   (deterministic).
5. **Halo / seam correctness — MET.** Each band runs the fused traversal on a
   height-shortened `Sensor` clone starting at `lo = (s0 − 3) & !1` and ending at
   `min(h, s1 + 3)`, with an offset normaliser keyed by whole-plane coords and a
   local green buffer. Three rows is the true dependency radius (R/B read green
   ±1 row, which reads mosaic ±2 more), so emitted rows never see a local
   reflection; where the halo is clamped the local edge *is* the true edge. The
   kernel's phase is `(y − active_top) & 1`, so the even `lo` preserves the
   ActiveArea-origin phase. **Mutations (each run against `tests/threaded.rs`,
   then reverted): all 8 red.**
   | Mutation | Result |
   |---|---|
   | halo 3 → 2 | 5/8 red |
   | drop the even rounding of `lo` | 5/8 red |
   | top halo → 1 row | 5/8 red |
   | bottom halo → 1 row | 5/8 red |
   | thread orientations 5–8 as non-flip | 2/8 red (transposing + f32) |
   | treat 3/4 as no vertical flip | 4/8 red |
   | demosaic threaded stub (`Ok(())` for N > 1) | 4/8 red |
   | colour threaded stub | 5/8 red |
   This also independently confirms the handback's "red before implementation".
6. **Orientation predicate — MET.** `colour_band_plan` maps `1|2 → no flip`,
   `3|4 → vertical flip`, anything else → single-thread fallback. That matches
   the TIFF table (2 = mirror horizontal, 3 = 180°, 4 = mirror vertical; 5–8
   transpose). Orientation absent resolves to 1 through `resolve_geometry`
   (tested). Real-file timings confirm the split: 1–4 run ≈7× faster at N = 8,
   5–8 run at single-thread speed.
7. **Panic-free / error parity — MET.** Every input the banded path does not take
   (buffer lengths, geometry, CFA, matrix, < 2 rows) returns `None` from the plan
   and goes to the single-thread function, so the typed error is identical by
   construction (`threaded_errors_match_the_single_thread_errors`). Threads are
   spawned with `Builder::spawn_scoped(..).ok()` and the calling thread drains
   the same queue, so a refused spawn means fewer workers, never a panic or a
   skipped band. A worker panic is re-raised via `resume_unwind` on join, but
   the kernels are the existing panic-free ones; the lib-level panic-free lint
   policy CI check is green.
8. **No forbidden means — MET.** `Cargo.toml` / `Cargo.lock` unchanged (only
   `std::sync::Mutex` + `std::thread`); `#![forbid(unsafe_code)]` holds and no
   `unsafe` in `src/`; no SIMD; the only new allocations are the per-band local
   green (≈ one `w·h` plane in total + 6 halo rows per band) and the band queue —
   no RGB intermediate; no `DEVELOP_VERSION` bump.
9. **Measured speedup (DEC-041 ledger row) — MET.** The ledger row records
   N = 1 vs 2/4/8 on synthetic 24 MP / 23 MP planes (demosaic ≈1.83× / 3.30× /
   3.75×, colour u16 ≈1.95× / 3.69× / 5.85×; orientation 6 unchanged). My
   real-file run on the same class of host measured more (Canon demosaic
   328 → 47 ms, colour 401 → 54 ms at N = 8) — host-load dependent, not a gate.
10. **Build reflection — honest.** It names a real stumbling point (shortened
    sensor view + even top) and two lessons (`spawn` panics on OS refusal; a
    threaded-vs-threaded baseline passes against a stub). Not a "nothing was
    unclear".

### Cost self-report

- **Tokens (total):** see `handback.tokens_total` (input + cache write + cache
  read + output, deduped by message id).
- **Estimated USD:** see `handback.estimated_usd` (Opus list rates).
- **Duration (minutes):** ~35
- **Source of the number:** this session's own transcript
  (`df90e54b-f9b1-4b03-8410-58d24b2e6619.jsonl`, anchored on the scratchpad
  UUID), summing every assistant usage record — `/cost` is not callable as a
  tool in a delegated session. Excludes the final commit/PR turns.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec (all acceptable, none blocking):**
  - `demosaic_into_threaded` bands run the fused traversal with an identity
    normaliser rather than `demosaic_into` itself (avoids a per-band RGB
    buffer). Equivalence to `demosaic_into` is proven directly by the tests and
    the real-file runs.
  - Bands are drained from a shared `Mutex` queue by `Builder::spawn_scoped`
    workers plus the calling thread, rather than one `scope.spawn` per band.
- **Follow-up work identified (non-blocking):**
  - **FU-1 (DEC-044 addendum).** DEC-044 says the demosaic band needs source
    rows `[oy0−2, oy1+2)`; the correct radius is **3** (the build uses 3, and
    halo 2 goes red). Also record the even-rounded halo top, the
    `Builder::spawn_scoped` + work-queue choice (`scope.spawn` panics on OS
    refusal), and the fused-traversal demosaic bands.
  - **FU-2 (resource foot-gun).** The band count is `min(threads, rows)`, so a
    caller passing a huge `threads` on a 4000-row image spawns ~4000 OS threads
    with one-row bands, each doing 7 rows of work (≈7× total work). Correct and
    panic-free, but a later spec could floor the band height (e.g. ≥ 16 rows)
    or cap N.
  - **FU-3 (test sweep).** The committed colour fixture has an odd ActiveArea
    origin and the whole-plane case has origin 0; a non-zero *even* origin is
    covered only by the real files in this verify. Optional to add.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The brief named the PR branch `claude/adoring-kilby-565b10`; the PR's
   real head is `claude/spec-051-row-band-threading`. `gh pr view` settled it.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-044's ±2 halo is wrong (it is ±3); a verifier trusting the DEC over
   the code would flag a correct build. Worth the FU-1 addendum.
3. **If you did this task again, what would you do differently?**
   — Run the real-file differential first: it covers every orientation and many
   N in one pass and makes the fallback visible through timing alone.
