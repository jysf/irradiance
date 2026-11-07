# SPEC-048 — build cycle prompt

You are the **implementer** for SPEC-048 (PROJ-002, STAGE-012), build cycle.
Everything you need is on `main`. Use `/usr/bin/git` and `gh` (gh needs the
sandbox disabled).

Read, in order:

1. `projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-101-build-demosaic-interior-fast-path.md` — your context, the files, constraints, and the handback you must fill before reporting done.
2. `projects/PROJ-002-bayer-colour-develop/specs/SPEC-048-demosaic-interior-fast-path.md` — Acceptance Criteria, Failing Tests, Notes are the contract.
3. `decisions/DEC-041-demosaic-interior-fast-path-and-the-stage-012-perf-discipline.md` — the decision and the perf discipline.
4. `guidance/toolchain-brief.md` and `AGENTS.md`.

## The change (DEC-041)

Give `demosaic_into` (`src/demosaic.rs`) a row-wise **interior fast path**:
- Iterate row by row so `y` is the loop variable and the row base is computed
  once per row — not a `checked_div`/`checked_rem` per pixel.
- For the **interior** (every pixel ≥ 2 rows and ≥ 2 columns from each edge),
  read the ±1/±2 neighbours by direct index into per-row slices of `src` (and,
  in pass 2, `dst`), skipping `reflect()`, `plane_index()` and the per-read
  bounds-checked `get`.
- Keep the current reflecting, bounds-checked path for the **2-pixel border**.
- **Reuse the kernel arithmetic verbatim** (the classifiers, second differences,
  predictors, rounding, clamp) — change only how a neighbour value is fetched,
  never what is computed from it.

Per-row slices (`src[row-2 ..= row+2]` windows) make the interior in-bounds by
construction, so stay panic-free with **no `unsafe`** and no per-read `get`.
Do NOT change the kernel, the two-pass structure, the public signatures, or any
dependency. Keep the `active_area_origin` phase offset (`DEC-040`, SPEC-047) intact.

## Order of work (output-invariant, so prove it)

1. Write the failing/equivalence tests from the spec first: an
   `interior_fast_path_matches_the_reference_demosaic` test asserting the
   fast-path output equals a straightforward reflecting reference **byte-for-byte**
   over all four Bayer phases and a size with a non-trivial interior AND border
   (the `dst_after == dst_before` proof). The existing demosaic golden hashes
   must keep passing unchanged.
2. Implement the fast path; make the equivalence test pass and keep the goldens green.
3. Measure: record a **before/after** `demosaic_into` wall-time on a representative
   frame in `docs/provenance-ledger.md` (the speedup is the point; whatever factor
   it is, report it honestly).
4. `just test` green (NEVER `just test --all-features`), `just lint-ci` and
   `just fmt-check` clean, the five-lint deny policy satisfied, `just validate`
   and `just cost-audit` green. No new dependency; no panic path; determinism held.

## Hand back

Fill HANDOFF-101's `handback:` front matter (REAL `tokens_total` from `/cost`)
and the `## Handback` section, then open a PR against `main` (SPEC-048 only)
referencing HANDOFF-101, SPEC-048, STAGE-012, PROJ-002, DEC-041, DEC-032. The
commit message BODY must be plain prose (the push ruleset rejects code tokens
there — reword if rejected, don't retry verbatim), ending with:
Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>
End the PR description with:
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Report back the PR number, the measured before/after demosaic time, and the true token cost.
