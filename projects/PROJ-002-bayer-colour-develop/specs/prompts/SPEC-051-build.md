# SPEC-051 — build cycle prompt

You are the **implementer** for SPEC-051 (PROJ-002, STAGE-012), build cycle.
Everything is on `main`. Use `/usr/bin/git` and `gh` (gh needs the sandbox disabled).

Read, in order: `projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-107-build-opt-in-row-band-demosaic-threading.md` (your context + the handback you must fill), `projects/PROJ-002-bayer-colour-develop/specs/SPEC-051-opt-in-row-band-demosaic-threading.md` (Acceptance + Failing Tests + Notes are the contract), `decisions/DEC-044-opt-in-output-row-band-threading-for-the-demosaic-hot-loop.md`, then `AGENTS.md` and `guidance/toolchain-brief.md`.

## The change (DEC-044) — opt-in output-row-band threading, `std::thread::scope`

Add three additive `*_threaded` entry points; the existing single-thread functions and their output are untouched.

**1. Partition the OUTPUT, not the input.** `std::thread::scope` borrows `src` immutably (shared) and `dst` mutably as DISJOINT slices (`split_at_mut` / `chunks_mut`) — the borrow checker proving no overlap is also the bit-identity guarantee. Each thread works backwards from its output band to the source rows it needs, runs the EXISTING single-thread traversal over those rows (local green buffer + the SPEC-049 rolling cache), reflecting at the TRUE image edges (never a band edge), and writes only its `dst` band. `threads <= 1` dispatches straight to the existing function (no banding, no alloc). Output is byte-identical to single-thread and independent of N.

**2. `demosaic_into_threaded(sensor, src, dst, threads)`.** Output rows map 1:1 to source rows, so output band `[oy0, oy1)` reads source rows `[oy0-2, oy1+2)` (clamped/reflected). Threads every input. Use a LOCAL per-band green buffer (the single-thread `demosaic_into` keeps green in `dst`, which a neighbour band would be writing red/blue into — so the threaded path can't share it).

**3. `develop_color_into_threaded` / `develop_color_f32_into_threaded`.** Resolve the `Orientation` once via `resolve_geometry`. Non-transposing (orientations 1–4): an output band is fed by a contiguous source band — thread it (develop that source band, scatter through the SPEC-050 sink into the disjoint `dst` band). Transposing (orientations 5–8, the 90° rotations): an output band is fed by source columns across all rows — **fall back to calling the single-thread function** (correct, not parallel). The f32 entry stays exactly `to_linear_f32_into(develop_color_into(..))`.

Do NOT add a dependency, `unsafe`, SIMD, a transposed-colour gather, an RGB intermediate buffer, or a `DEVELOP_VERSION` bump (threading is output-invariant). Do NOT change the single-thread kernels or output.

## Order of work (write the red tests first)

1. Failing tests first (byte-equality, each red before the threaded path exists), in `tests/`:
   - `threads_one_equals_the_single_thread_entry` (threads 0 and 1 equal the existing functions).
   - `threaded_demosaic_equals_single_threaded` (N in {2,3,4}, all Bayer phases, tall image, a boundary on an interior row).
   - `threaded_colour_develop_non_transposing_equals_single_threaded` (orientations 1–4, non-trivial ActiveArea/DefaultCrop, N in {2,4}).
   - `threaded_colour_develop_transposing_falls_back_and_still_matches` (orientations 5–8, N>1 still byte-identical via the fallback).
   - `threaded_f32_equals_single_threaded` (N in {2,4}).
   - `output_is_independent_of_thread_count` (identical across N in {1,2,3,5,8}).
   - `band_boundary_reflects_at_the_true_edge` (a forced interior boundary; boundary-row pixels equal single-thread).
2. Implement; make them pass; the existing single-thread goldens stay green.
3. Measure: record a `threads=1` vs N wall-time on a large image in `docs/provenance-ledger.md` (a real speedup for N>=2), per DEC-041.
4. `just test` green (NEVER `just test --all-features`), `just lint-ci`, `just fmt-check`, `just validate`, `just cost-audit` green; no panic; determinism; no dependency; no `unsafe`.

## Hand back

Fill HANDOFF-107's `handback:` (REAL `tokens_total` from `/cost`) and the `## Handback` section, then open a PR against `main` (SPEC-051 only) referencing HANDOFF-107, SPEC-051, STAGE-012, PROJ-002, DEC-044, DEC-041. Commit message BODY plain prose (reword if the push ruleset rejects code tokens like `#[...]`, `->`, `--flag`), ending with:
Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back: the PR number, confirmation the threaded output is byte-identical to single-thread across N and orientations, the measured `threads=1` vs N speedup, and the true token cost.
