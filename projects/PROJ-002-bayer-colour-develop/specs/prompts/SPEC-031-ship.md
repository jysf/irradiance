# SPEC-031 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ APPROVED (`HANDOFF-077`, at commit
> `5450c70`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **The build PR #67 is UNMERGED.** Unlike SPEC-030 (whose #63 was merged
> early), nothing has landed on `main` yet. The verify branch
> (`claude/spec-031-verify-311cc7`) is cut from #67's branch, so the **verify PR
> carries build + verify together** — merging it lands the whole spec and #67
> then closes as included. So this ship cycle DOES include a real merge, plus the
> usual closing ritual (reflection, cost totals, `complexity_actual`, archive,
> brag). Confirm with the maintainer which PR to merge (the verify PR, not #67).

Cycle: ship. The SPEC-031 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify re-ran every substantive gate locally on `5450c70`:
  `just test` all green (**16/16 test binaries**; demosaic_oracle **22/22**;
  every STAGE-006 regression oracle green; tier-B corpus SKIPs loud, not silent),
  `just lint-ci` green (pinned stable clippy 0.1.98, the five panic-free lints
  hold, `-D warnings`), `cargo fmt --check` green, `just validate` green,
  `just decisions-audit --changed` 0 matches. Confirm GitHub CI is green on the
  verify PR's head before merging.
- [x] Deployment steps known? — library crate; **no new public surface** this
  spec. `demosaic_into`'s signature, the `Error` variants, the caller-owned
  zero-alloc buffer contract (DEC-016/DEC-032) and the `irr demosaic` P6 output
  are all unchanged from SPEC-030; **only the decoded pixel values change**
  (bilinear → Hamilton-Adams). `DEC-032` is new (internal: the two passes share
  the green plane in `dst`'s own green slots).
- [x] Rollback plan? — the change is confined to `demosaic_into`'s interpolation
  (`src/demosaic.rs`) + its oracle (`tests/demosaic_oracle.rs`) + docs. A revert
  restores SPEC-030's bilinear kernel cleanly; no other decoded-pixel path is
  touched (STAGE-006 intact, confirmed by every STAGE-006 oracle staying green).
- [x] CHANGELOG / release note? — **already written by the build** under
  `[Unreleased] → Changed` ("Demosaic kernel upgraded to Hamilton-Adams"). It is
  correct and complete (names SPEC-031, DEC-031/016/032, the clean-room-beats-
  copyright-not-patents rationale, bit-exact verification, uncorrected output).
  Confirm it, don't duplicate it.

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-031-malvar-he-cutler-gradient-corrected-5-5-kernels-timeline.md`
(verify already set it to `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and
5) at the bottom of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("SPEC-030 shipped a
  bilinear baseline — each missing channel the average of its nearest same-colour
  neighbours"), the Goal is the after ("the Adams-Hamilton directional demosaic —
  directional green on the lower-gradient axis, then colour-difference red/blue —
  verified bit-exact on analytic fixtures with a perturbed-kernel red-proof").
  The user-visible outcome: `irr demosaic` now emits a higher-quality render
  (directional green removes most of bilinear's edge zippering / false colour),
  still uncorrected camera-space linear RGB — the recognisable colour picture is
  STAGE-007 + STAGE-008 together.
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass.
  Verify independently re-derived the bit-exact math (hand-recomputed pixel (2,2)
  incl. a nested classifier tie) and confirmed the oracle genuinely goes red by
  breaking the REAL kernel three ways (pass-1 green predictor, pass-2 diagonal,
  pass-2 green-site colour-difference) — each turned the oracle red — finding no
  code defect. The one open follow-up (GRBG/GBRG fixture) is spec-compliant and
  non-blocking, not a defect.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and
  ship are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ RECOVER THE TWO METERED NULLS.** BOTH the build and the verify handbacks
  carry `tokens_total: null` because `/cost` is not exposed as a tool in a Claude
  Code desktop (Code tab) session (neither invented a number, per the contract).
  Read `/cost` from each session's transcript and fill `cost.sessions` directly:
  - **build (HANDOFF-076):** anchor on scratchpad UUID
    `9b462a6e-5ff6-421d-be8a-1b6ea3098b04`.
  - **verify (HANDOFF-077):** anchor on scratchpad UUID
    `21f3a4a2-4f9e-4e86-9dba-37e22e208e34`.
  Anchor on the UUID, not a text grep (memory: `identify-own-transcript-for-cost-handback`
  — a stale pre-`/clear` file can text-match and be a different model). Do NOT run
  `just handback-sync` expecting it to supply these numbers — the handbacks are
  null; sync only transcribes what is there.
- Recompute `cost.totals` from the session entries (build + verify + ship; skip
  nulls once the two are recovered); `session_count` includes null-numeric sessions.
- Set `complexity_actual` (expected was `M`). The FULL Hamilton-Adams (green +
  R/B) landed as one cold [M] build — the pre-named green-only fallback was not
  needed — and verify approved first pass; it came in at `M`. Set from the
  recovered cost + your judgment.
- Mark ship `[x]` in the timeline with the merge date + total cost.
- Confirm `just advance-cycle SPEC-031 ship` already ran at verify (it did — verify
  advances the cycle to `ship` and stamps `verify_verdict: approved`). Confirm
  `task.cycle: ship` and `verify_verdict: approved` are both set; if the verdict is
  somehow not stamped, this is the step that stamps it. Then `just archive-spec
  SPEC-031` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool,
  passing agent + model (model has no fallback). Suggested impact: "irradiance's
  demosaic is now Hamilton-Adams directional interpolation (patent-clean, DEC-031)
  instead of SPEC-030's bilinear — directional green on the lower-gradient axis +
  colour-difference red/blue, zero-alloc two-pass (DEC-032), verified bit-exact on
  analytic fixtures with a red-proof confirmed genuinely-red by an independent
  reviewer who broke the real kernel three ways. Output stays uncorrected
  camera-space linear RGB (colour is STAGE-008)."
- Update STAGE-007's backlog: mark SPEC-031 shipped (count → 2 shipped / 1 active).
  Next: **SPEC-032** (CFA-demosaic fuzz + panic-free hardening on degenerate /
  edge-phase planes), the last STAGE-007 spec.
- **SPEC-030/FU-1 (still open, re-confirmed at this verify) — record, do not fix
  here:** the oracle's phase fixtures cover RGGB and BGGR (both *anti-diagonal*
  green phases); **GRBG and GBRG** (greens on the *main* diagonal) are handled by
  the kernel by construction (`BayerPhase::from_cfa`'s `greens_main_diagonal`
  branch + phase-driven R/B assignment) but are not fixture-verified. SPEC-031's
  AC6 only requires a BGGR fixture, so the build meets its contract. Fold the
  GRBG/GBRG fixture into **SPEC-032** (edge-phase hardening). If not acted on in
  SPEC-032, record it as a `type: lesson` (`measurement-over-generalised`, carry
  its N) in `/guidance/signals.yaml`.
- `just cost-audit` to confirm SPEC-031 has real build + verify cost once the two
  nulls are recovered (it will FAIL on a shipped spec with a null metered cycle —
  that is the gate working; recover the numbers first).

This is the quality kernel in STAGE-007's backlog, **not** the last (SPEC-032
remains) — do **not** run the Stage Ship prompt (1d) yet.
