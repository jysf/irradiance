# SPEC-033 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ **APPROVED** (`HANDOFF-081`, at commit
> `bb4aab11`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **The build PR #79 is ALREADY MERGED to `main`** (@ `bb4aab1`; like SPEC-030's
> #63 and SPEC-032's #71). So the colour code already landed; the verify branch
> (`claude/spec-033-verify-28f03a`) carries **docs only** — HANDOFF-081, the timeline
> `[x]`, this ship prompt, and the `advance-cycle` stamps. Merging the verify PR lands
> NO code. This ship cycle is the closing ritual (reflection, cost totals,
> `complexity_actual`, archive, brag) plus merging the docs-only verify PR.
>
> ⚠ **SPEC-033 adds PUBLIC API** (`color::to_srgb_linear_into`,
> `color::develop_color_into`, `Error::ColorSourceWrongLength`,
> `Error::MissingColorCalibration`, `irr color`) → it **NEEDS** a CHANGELOG entry
> (unlike the test-only SPEC-025/027/032). The build added one under
> `[Unreleased] → Added` — **confirm it is present; do NOT remove it, do NOT add a
> second.**

Cycle: ship. The SPEC-033 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify re-ran every substantive gate locally on the reviewed tree
  (`bb4aab1`, byte-identical to HEAD for the colour code): `just test` all green
  (whole suite; the `color_oracle` 9/9; every STAGE-006/007 regression oracle green;
  the mono `develop_into` green), `just lint-ci` green (pinned stable clippy,
  `-D warnings`), `cargo fmt --check` green, `just validate` green (39 artefacts),
  `just msrv` (1.90.0) green, `just decisions-audit --changed main` → no changed files
  in scope (build already on `main`). **Confirm GitHub CI is green on the verify PR's
  head before merging.**
- [x] Deployment steps known? — library crate; this spec **adds public surface**
  (the two `color` fns, two `Error` variants, `irr color`). Additive only:
  `demosaic_into`, the mono `develop_into`, every STAGE-006/007 behaviour unchanged
  (`git show --stat bb4aab1` adds `src/color.rs`, `tests/color_oracle.rs`, and touches
  `src/lib.rs`/`src/bin/irr.rs`/`docs/`/`CHANGELOG.md` additively).
- [x] Rollback plan? — a revert removes `src/color.rs`, the oracle, the two `Error`
  variants, `irr color` and the CHANGELOG entry; nothing else depends on them yet
  (SPEC-034/035 are not built). No regression surface in the existing paths.
- [x] CHANGELOG / release note? — **REQUIRED and PRESENT** (public API). Confirm the
  build's `[Unreleased] → Added` colour-transform entry is intact; do not duplicate.

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-033-colour-transform-core-first-colour-develop-wb-timeline.md`
(verify set it `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and 5)
at the bottom of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("STAGE-007 delivers
  demosaiced **camera-space** RGB — greenish, flat, no display encoding"); the Goal is
  the after ("a **display-referred sRGB** image from a Bayer DNG — WB + camera→XYZ +
  XYZ→linear-sRGB, emitted by `irr color`"). User-visible outcome: `irr color <bayer
  DNG>` now produces the first recognisable colour picture — confirmed end-to-end on
  the real Canon 5D-III (FM1 path) and Nikon D300S (inv(CM1) path) frames, valid P6
  PPM with genuine colour. Name the new public API.
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass.
  Verify independently re-derived the bit-exact oracle (own Python reference,
  standards-transcribed `S`/`B` constants, exact f64 op order → reproduced the real
  kernel to the bit on the FM path, the inv(CM) path, neutral→grey, off-gamut and the
  red-proof camera), broke the REAL kernel two independent ways (perturbed committed
  `S[0][0]` → 6 oracle tests red; removed the WB `1/asn` → 7 red; honest control green)
  and ran `irr color` end-to-end on both real frames — finding no defect.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and ship
  are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ RECOVER THE TWO METERED NULLS.** BOTH the build and the verify handbacks carry
  `tokens_total: null` because `/cost` is not exposed in a Claude Code desktop (Code
  tab) session (the recurring `desktop-cost-tool-unavailable-handbacks-null` gap;
  neither invented a number, per the contract). Read `/cost` from each session's
  transcript and fill `cost.sessions` directly:
  - **build (HANDOFF-080):** session `local_80d10c9d-5c8d-415c-a78f-27c7f1a2d186`
    (branch `claude/spec-033-build-f223ec`, model `claude-opus-4-8`). Find its
    scratchpad UUID from that session's transcript and anchor on the UUID, not a text
    grep (memory: `identify-own-transcript-for-cost-handback` — a stale pre-`/clear`
    file can text-match and be a different model). Duration self-reported ~95 min.
  - **verify (HANDOFF-081):** anchor on scratchpad UUID
    `790c21da-873f-437d-9be9-d67eb55a46dd` (model `claude-opus-4-8`). Duration ~50 min.
  Do NOT run `just handback-sync` expecting it to supply these numbers — the handbacks
  are null; sync only transcribes what is there.
- Recompute `cost.totals` from the session entries (design + build + verify + ship;
  skip nulls once the two are recovered); `session_count` includes null-numeric
  sessions.
- Set `complexity_actual` (expected was `M`). The build reused STAGE-007's
  demosaic/tone/irr harness and oracle style, built **whole** (no pre-named [M] split),
  and approved first pass. Set from the recovered cost + judgment (an `M` is the likely
  landing; the frame's L-risk did not materialise).
- Mark ship `[x]` in the timeline with the merge date + total cost; keep the ship line
  pointing at `prompts/SPEC-033-ship.md`.
- Confirm `just advance-cycle SPEC-033 ship` already ran at verify (it did — verify
  advances the cycle to `ship` and stamps `verify_verdict: approved`). Confirm
  `task.cycle: ship` and `verify_verdict: approved` are both set; if the verdict is
  somehow not stamped, this is the step that stamps it. Then
  `just archive-spec SPEC-033` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool,
  passing agent + model (model has no fallback). Suggested impact: "irradiance now
  renders a Bayer DNG to a **display-referred sRGB** picture: new `color::to_srgb_linear_into`
  (WB from `AsShotNeutral` + camera→XYZ via `ForwardMatrix1` else inv(`ColorMatrix1`) +
  committed Bradford-D50→D65·linear-sRGB constant `C`, one `f64` 3×3 built once per
  image, DEC-033) and `color::develop_color_into` (per-channel mosaic normalize →
  Hamilton-Adams demosaic → the matrix), emitted by `irr color` as a P6 PPM. Verified
  **bit-exact** on analytic fixtures against an independent reference — the FM path, the
  inv(CM) path, neutral→grey (`R==G==B`), off-gamut clamp, the compose, and a REAL-`FM1`-
  coefficient red-proof — and the cold verify re-derived the oracle from the standards
  and broke the kernel two independent ways to prove it bites. Runs end-to-end on the
  real Canon 5D-III (FM1) and Nikon D300S (inv(CM1)) frames. First colour render of
  PROJ-002's final stage; colorimetric accuracy is SPEC-035's ΔE (blocked on a
  ColorChecker shot)."
- Update STAGE-008's backlog: mark SPEC-033 shipped (→ **1 shipped / 2 active** —
  SPEC-034 dual-illuminant, SPEC-035 ΔE still open; SPEC-035 blocked on the
  ColorChecker shot).
- Record the non-blocking verify observation (HANDOFF-081 "Follow-up work"): the
  explicit `quantize` clamp is numerically redundant with Rust's saturating `f64 as u16`
  cast, so the `quantize_clamps_before_rounding` unit test passes even with the clamp
  removed. Correctness-neutral (output identical, DEC-018 pinning still holds). If not
  acted on now, drop a `signals.yaml` `lesson`
  (`saturating-as-cast-masks-explicit-clamp-in-quantize-tests`) so a future stage-close
  decides whether to strengthen the test.
- `just cost-audit` to confirm SPEC-033 has real build + verify cost once the two nulls
  are recovered (it will FAIL on a shipped spec with a null metered cycle — that is the
  gate working; recover the numbers first).

**SPEC-033 is the FOUNDATION of STAGE-008, not the last spec.** SPEC-034
(dual-illuminant interpolation + the scene-adaptive matrix selection) and SPEC-035
(ColorChecker ΔE — blocked on the maintainer's shot) remain. Do NOT run the Stage Ship
prompt (1d) yet.
