# SPEC-047 — build cycle prompt

You are the **implementer** for SPEC-047 (PROJ-002, STAGE-012), build cycle.
Read, in order:

1. `projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-099-build-colour-path-honours-the-activearea-origin.md` — your context, the files, the constraints, and the handback you must fill before reporting done.
2. `projects/PROJ-002-bayer-colour-develop/specs/SPEC-047-colour-path-honours-the-activearea-origin.md` — the Acceptance Criteria, Failing Tests, and Notes are the contract.
3. `decisions/DEC-040-index-the-cfa-phase-and-level-slot-from-the-activearea-origin.md` — the decision (offset, not crop) and why.
4. `guidance/toolchain-brief.md` and `AGENTS.md` — build/test/lint commands and conventions.

## The change (DEC-040)

Index the CFA phase and the BlackLevel/WhiteLevel 2×2 slot from the **ActiveArea
origin**, via the offset form, in the colour develop path:

```
row_odd = (y.wrapping_sub(active_top) & 1) == 1
col_odd = (x.wrapping_sub(active_left) & 1) == 1
```

- Add a shared `active_area_origin(sensor) -> (u32, u32)` helper returning the
  `ActiveArea` origin, `(0, 0)` when `sensor.active_area` is `None` (match
  `resolve_geometry`). Put it where `src/color.rs` and `src/demosaic.rs` both use
  it (and it is reachable from `src/develop.rs`) — one source of truth.
- Apply it in `normalize_mosaic` (`src/color.rs`) for the level slot, and in
  `demosaic_into` and its green/colour site helpers (`src/demosaic.rs`) for the
  phase.
- Bump `DEVELOP_VERSION` `"2"` → `"3"` in `src/lib.rs`.
- **Do NOT crop, do NOT change `develop_color_into`'s buffer lengths or output
  dimensions.** The offset is the whole change (the crop is a deferred follow-up).

## Order of work (test-before-implementation)

1. Write the failing tests from the spec's **Failing Tests** section in
   `tests/color_oracle.rs`, built on `fixture_sensor` / `rggb()` / the D50
   forward-matrix helpers / `a_neutral_subject_is_grey`. See the red-proof go red.
2. Make them pass with the offset change. The even-origin byte-identity test must
   stay green throughout (no regression for the corpus).
3. `just test` green, `just lint-ci` / `just fmt-check` clean, and the five-lint
   deny policy satisfied. No new dependency; no panic path; determinism held.

## Acceptance

Everything in SPEC-047's **Acceptance Criteria**: grey stays grey (channel spread
within tolerance) across the 4 origin parities × 4 Bayer phases; the red-proof is
shown red on the plane-relative path; even-origin output is byte-identical; no
panic for any origin; `DEVELOP_VERSION == "3"`.

## Hand back

Fill HANDOFF-099's `handback:` front matter (with a REAL `tokens_total` from
`/cost`) and the `## Handback` section, open a PR against `main` (SPEC-047 only),
and reference HANDOFF-099, SPEC-047, STAGE-012, PROJ-002, DEC-040, DEC-038.
Remember: the commit-message BODY must be plain prose (the push ruleset rejects
code tokens there); the PR body may contain them.
