# SPEC-030 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ APPROVED (`HANDOFF-075`, at commit
> `9e908f7`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ Note: PR #63 was **already merged** to `main` (commit `9e908f7`) before
> verify ran — the maintainer merged early — so the merge step is done. This
> ship cycle is the closing ritual (reflection, cost totals, `complexity_actual`,
> archive, CHANGELOG, brag), not a merge. The verify-bookkeeping PR (HANDOFF-075
> + verify cost + advanced spec/timeline + this prompt) is a separate small PR
> from `claude/spec-030-verify-04f726` (PR #64).

Cycle: ship. PR for SPEC-030 is approved.

Pre-ship checklist (all already true):
- [x] CI passing? — green on PR #63's head at merge; verify re-ran every
  substantive gate locally on `9e908f7`: `just test` all green (lib 105;
  demosaic_oracle 10; every STAGE-006 regression oracle green; 1 corpus-gated
  test ignored as usual), `just lint-ci` green (pinned stable clippy, the five
  panic-free lints hold), `cargo fmt --check` green, `just validate` green,
  `just decisions-audit --changed` 0 matches.
- [x] Deployment steps known? — library crate; **new public surface**:
  `demosaic::demosaic_into(sensor, &[u16], &mut [u16])`, three new `Error`
  variants (`NotAColorFilterArray`, `UnsupportedCfaPattern`,
  `DemosaicSourceWrongLength`, `RgbBufferWrongLength`), and a new `irr demosaic`
  subcommand → P6 PPM. The first multi-channel output irradiance produces.
- [x] Rollback plan? — the change is purely additive (new module + new `irr`
  subcommand + new error variants); no existing decoded-pixel path changes. A
  revert removes the demosaic path cleanly and leaves STAGE-006 intact.
- [ ] CHANGELOG / release note? — **yes, warranted**: a new public library
  capability + new public `Error` variants (pre-1.0, crate `0.1.1`, unpublished).
  Add a CHANGELOG line under the unreleased section: "Bilinear Bayer demosaic
  (`demosaic::demosaic_into`) — mosaic → 3-channel interleaved camera-space
  linear RGB, phase-driven, reflect border; `irr demosaic` emits a P6 PPM. New
  `Error` variants for the mono/absent-CFA, unsupported-CFA and wrong-length
  cases (note for the eventual 0.1.0 publish, DEC-028)." Output is uncorrected
  (greenish/flat) — the honest STAGE-007 milestone; the recognisable picture is
  STAGE-007 + STAGE-008.

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-030-demosaic-foundation-the-camera-space-linear-rgb-timeline.md`.

Answer the three Reflection questions and the two extra Reflection prompts (4 and
5) at the bottom of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("a mosaic carries
  one colour per photosite; to make an image every pixel needs all three
  channels"), the Goal is the after ("a 3-channel camera-space linear RGB image
  via a bilinear demosaic driven by `Sensor.color_filter`, verified bit-exact on
  analytic fixtures with a perturbed-kernel red-proof, and surfaced by `irr`").
  Confirm against the verify evidence: both corpus Bayer DNGs demosaic to a valid
  P6 PPM (Nikon 4352×2868 RGGB; Canon 5920×3950 RGGB), payload exactly `w*h*3*2`,
  honestly uncorrected (Canon min 1882 — black level not subtracted).
- **Question 4 (worst defect caught):** `none` — clean build, approved first
  pass; verify independently reproduced the bit-exact tables and confirmed the
  oracle genuinely goes red (3 real-kernel mutations), finding no code defect.
  The one verify follow-up (GRBG/GBRG fixture coverage, below) is spec-compliant
  and non-blocking — not a defect.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and
  ship are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ RECOVER THE TWO METERED NULLS.** BOTH the build and the verify handbacks
  carry `tokens_total: null` because `/cost` is not exposed as a tool in a Claude
  Code desktop session (neither invented a number, per the contract). Read `/cost`
  from each session's transcript and fill `cost.sessions` directly:
  - **build (HANDOFF-074):** anchor on scratchpad UUID
    `93026a80-0784-4ed6-9408-46a32be4a9f4` (floor cross-check: 389,365 context
    tokens at hand-off, mcp `get_usage`).
  - **verify (HANDOFF-075):** anchor on scratchpad UUID
    `eae8c836-5e74-4969-a9b0-f5a132fad651` (floor cross-check: 197,792 context
    tokens mid-verify).
  Anchor on the UUID, not a text grep (memory: `identify-own-transcript-for-cost-handback`
  — a stale pre-`/clear` file can text-match and be a different model). Do NOT run
  `just handback-sync` expecting it to supply these numbers — the handbacks are
  null; sync only transcribes what is there.
- Recompute `cost.totals` from the session entries (build + verify + ship; skip
  nulls once the two are recovered); `session_count` includes null-numeric sessions.
- Set `complexity_actual` (expected was `M`). The build was one cold session
  (~30 min, clean) and verify approved first pass (~35 min) — came in at or under
  `M`; set from the recovered cost + your judgment.
- Mark ship `[x]` in the timeline with the merge date (2026-10-31, PR #63) + total cost.
- Run `just advance-cycle SPEC-030 ship` (stamps `verify_verdict: approved` on the
  verify→ship transition — verify already ran `advance-cycle SPEC-030 ship`? NO:
  verify advanced the cycle to `ship` and left the verdict for this step; confirm
  `task.cycle: ship` and `verify_verdict: approved` are both set, and if the
  verdict is not yet stamped, this is the step that stamps it) and then
  `just archive-spec SPEC-030` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool,
  passing agent + model (model has no fallback). Suggested impact: "irradiance now
  turns a Bayer mosaic into a 3-channel camera-space linear RGB image (bilinear,
  phase-driven, bit-exact-verified) and `irr demosaic` emits it as a P6 PPM — the
  first multi-channel output; demosaic verified bit-exact on analytic fixtures
  with a red-proof confirmed genuinely-red by an independent reviewer."
- Update STAGE-007's backlog: mark SPEC-030 shipped. Next: **SPEC-031** (clean-room
  Malvar-He-Cutler, the quality kernel, against this same harness — L-risk, the
  green-vs-red/blue split is pre-named), then **SPEC-032** (CFA-demosaic fuzz +
  panic-free hardening on degenerate / edge-phase planes).
- **SPEC-030/FU-1 (from verify, non-blocking) — record, do not fix here:** the
  oracle's phase fixtures cover RGGB and BGGR (both *anti-diagonal* green phases);
  **GRBG and GBRG** (greens on the *main* diagonal) are not fixture-verified. The
  spec's AC2 only requires "BGGR (or GRBG)", so the build meets its contract and
  the kernel is provably phase-driven and correct for all four phases by
  inspection — but STAGE-007's Design Notes aspired to "cover the other three
  Bayer phases" and flagged the `measurement-over-generalised` trap. A one-line
  GRBG or GBRG fixture closes the only untested phase path. Fold into **SPEC-032**
  (edge-phase hardening) or add it alongside **SPEC-031**'s MHC fixtures. If not
  acted on now, record it as a `type: lesson` (`measurement-over-generalised`,
  carry its N) in `/guidance/signals.yaml`.
- `just cost-audit` to confirm SPEC-030 has real build + verify cost once the two
  nulls are recovered (it will FAIL on a shipped spec with a null metered cycle —
  that is the gate working; recover the numbers first).

This is the foundation spec in STAGE-007's backlog, not the last (SPEC-031 + 032
remain) — do **not** run the Stage Ship prompt (1d) yet.
