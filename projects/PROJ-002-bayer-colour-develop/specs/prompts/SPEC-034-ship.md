# SPEC-034 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ **APPROVED** (`HANDOFF-083`, at commit
> `2fbc03f1c21b4ae30d470e1a6c11c98daa603da5`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **The build PR #86 is ALREADY MERGED to `main`** (@ `2fbc03f`; like SPEC-030's #63,
> SPEC-032's #71, SPEC-033's #79). So the interpolation code already landed; the verify branch
> (`claude/spec-034-verify-6466d9`) carries **docs only** — HANDOFF-083, the timeline `[x]`,
> this ship prompt, and the `advance-cycle` stamps. Merging the verify PR lands NO code. This
> ship cycle is the closing ritual (reflection, cost totals, `complexity_actual`, archive, brag)
> plus merging the docs-only verify PR.
>
> ⚠ **SPEC-034 adds NO public API and NO new `Error` variant** (the weight is an internal,
> once-per-image refinement of the matrix build; `lib.rs`/`irr.rs` untouched). Its CHANGELOG
> entry is therefore `[Unreleased] → **Changed**` (values shift, public API unchanged), NOT
> `Added`. The build added one — **confirm it is present and under `Changed`; do NOT remove it,
> do NOT add a second.**

Cycle: ship. The SPEC-034 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify re-ran every substantive gate locally on the reviewed tree
  (`2fbc03f`, == HEAD): `just test` all binaries green, 0 failed (the one `ignored` is the
  pre-existing SPEC-017/SB-1 corpus finding; the `color_oracle` 14/14 incl. the interpolation
  oracle + perturbed-`ColorMatrix2` red-proof; every STAGE-006/007 regression oracle + the mono
  `develop_into` green), `just lint-ci` green (`-D warnings`, `--all-targets --all-features`),
  `cargo fmt --check` green, `just validate` green (39 artefacts), `just decisions-audit` 0
  structural errors (the DEC-033/DEC-034 same-scope line is the expected advisory). **Confirm
  GitHub CI is green on the verify PR's head before merging.**
- [x] Deployment steps known? — library crate; this spec adds **NO** public surface. The change
  is localized to `src/color.rs`'s matrix build (`git show --stat 2fbc03f`: `src/color.rs`,
  `tests/color_oracle.rs`, `docs/provenance-ledger.md`, `CHANGELOG.md`, + the spec/timeline/handoff
  docs). Only decoded pixel values shift, and only for dual-calibration files.
- [x] Rollback plan? — a revert restores SPEC-033's illuminant-1-only matrix build; the public
  API, `apply_matrix_into`, the committed `C`, `demosaic_into`, the mono `develop_into` and every
  STAGE-006/007 behaviour are already unchanged by this spec, so a revert touches only the matrix
  selection. No regression surface elsewhere (SPEC-035 not built).
- [x] CHANGELOG / release note? — **PRESENT under `[Unreleased] → Changed`** (values shift, API
  unchanged, single-calibration byte-identical). Confirm it is intact; do not duplicate, do not
  move it to `Added`.

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-034-dual-illuminant-interpolation-chromatic-adaptation-timeline.md`
(verify set it `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and 5) at the
bottom of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("SPEC-033 builds the matrix from
  the **first** illuminant only — a daylight shot rendered through a tungsten calibration");
  the Goal is the after ("the camera→XYZ matrix is **interpolated at the scene colour
  temperature** — the DNG's own chromatic-adaptation mechanism"). User-visible outcome: a
  dual-calibration DNG now renders with the scene-illuminant-correct colour matrix instead of
  illuminant 1's — confirmed end-to-end by `irr color` on the real Canon 5D-III (illuminants
  A/D65, FM(g) path, scene weight g≈0.426) and Nikon D300S (inv(CM(g)) path, g≈0.763) frames,
  both genuine interior weights (interpolation actively engaged, not a fallback), both a valid
  P6 PPM. Colorimetric accuracy itself is SPEC-035's ΔE (blocked on the ColorChecker shot).
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass. Verify
  independently re-derived the bit-exact `NeutralToXY` weight with its own Python reference
  (3-iteration loop, McCamy, mired blend → reproduced `g = 0.5202685520213226` to the bit),
  re-derived McCamy on the D65/D50 white points, audited the committed `ILLUMINANT_CCT`/McCamy
  constants against the CIE/EXIF/McCamy-1992 sources, broke the REAL kernel two independent ways
  (drop an iteration `0..3→0..2`; reverse the mired direction) — each turned the oracle red,
  honest control green — confirmed the single-illuminant fallback byte-identical to SPEC-033, and
  ran `irr color` end-to-end on both real dual-calibration frames. No defect found.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and ship are
  un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ RECOVER THE TWO METERED NULLS.** BOTH the build and the verify handbacks carry
  `tokens_total: null` because `/cost` is not exposed in a Claude Code desktop (Code tab) session
  (the recurring `desktop-cost-tool-unavailable-handbacks-null` gap; neither invented a number,
  per the contract). Read `/cost` from each session's transcript and fill `cost.sessions` directly:
  - **build (HANDOFF-082):** session `local_c7c36760-7b6d-4bd5-bd45-2fe28995468f` (branch
    `claude/spec-034-build-9eab3f`, model `claude-opus-4-8`). Anchor on its scratchpad UUID
    `fca0626e-be12-40d9-b8b7-9dd8cb857b97` (named in HANDOFF-082), not a text grep (memory:
    `identify-own-transcript-for-cost-handback` — a stale pre-`/clear` file can text-match and be a
    different model). Duration self-reported ~45 min.
  - **verify (HANDOFF-083):** anchor on scratchpad UUID `1815e3c4-005e-401b-a35b-dc96bf72a60f`
    (model `claude-opus-4-8`). Duration ~40 min.
  Do NOT run `just handback-sync` expecting it to supply these numbers — the handbacks are null;
  sync only transcribes what is there.
- Recompute `cost.totals` from the session entries (design + build + verify + ship; skip nulls
  once the two are recovered); `session_count` includes null-numeric sessions. `tokens_estimate`
  was 35M (~24M build + ~11M verify).
- Set `complexity_actual` (expected was `M`). The build reused SPEC-033's `color.rs` matrix infra
  + the oracle harness, built **whole** (the pre-named [M] L-risk split was not needed), and was
  approved first pass. An `M` is the likely landing; set from the recovered cost + judgment.
- Mark ship `[x]` in the timeline with the merge date + total cost; keep the ship line pointing at
  `prompts/SPEC-034-ship.md`.
- Confirm `just advance-cycle SPEC-034 ship` already ran at verify (verify advances the cycle to
  `ship` and stamps `verify_verdict: approved`). Confirm `task.cycle: ship` and
  `verify_verdict: approved` are both set; if the verdict is somehow not stamped, this is the step
  that stamps it. Then `just archive-spec SPEC-034` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool, passing agent +
  model (model has no fallback). Suggested impact: "irradiance's colour matrix is now
  **scene-adaptive**: for a dual-calibration DNG, `color::build_camera_to_srgb_linear` interpolates
  the two DNG calibrations (`ColorMatrix1/2`, `ForwardMatrix1/2` at `CalibrationIlluminant1/2`) at
  the scene colour temperature by the DNG reciprocal-temperature `NeutralToXY` method —
  `AsShotNeutral` → 3-iteration `inv(CM(g))` white → McCamy cubic CCT → mired weight `g` (DEC-034),
  all committed-constant `f64`, `+ − × ÷` only, once per image — instead of using illuminant 1
  alone. A daylight shot is no longer rendered through a tungsten calibration. Public API,
  `apply_matrix_into` and the committed `C` are unchanged; a single-calibration sensor is
  byte-identical to SPEC-033 (the regression guard). Verified **bit-exact** against an independent
  `NeutralToXY` re-derivation (the fixture `g = 0.5202685520213226` to the bit), with a perturbed-
  `ColorMatrix2` red-proof; the cold verify re-derived the weight + McCamy in its own reference and
  broke the real kernel two independent ways to prove the oracle bites. Runs end-to-end on the real
  Canon 5D-III (FM(g), g≈0.426) and Nikon D300S (inv(CM(g)), g≈0.763) frames. Colorimetric accuracy
  is SPEC-035's ΔE (blocked on a ColorChecker shot)."
- Update STAGE-008's backlog: mark SPEC-034 shipped (→ **2 shipped / 1 active** — only SPEC-035
  ΔE remains, blocked on the maintainer's ColorChecker shot).
- Fill the verify PR number into HANDOFF-083's `handback.pr` once the PR is open (the verify session
  left it null pending the PR).

**SPEC-034 is NOT the last spec in STAGE-008.** SPEC-035 (ColorChecker ΔE — blocked on the
maintainer's shot) remains. Do NOT run the Stage Ship prompt (1d) yet; STAGE-008 (and therefore
PROJ-002) ships only once SPEC-035 lands.
