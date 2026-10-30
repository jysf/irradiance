# SPEC-024 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ APPROVED (`HANDOFF-065`, at commit
> `2bef1d6`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ Note: PR #37 was **already merged** to `main` before verify ran, so the
> merge + deploy step is done — this ship cycle is the closing ritual
> (reflection, cost totals, archive), not a merge.

Cycle: ship. PR for SPEC-024 is approved.

Pre-ship checklist (all already true):
- [x] CI passing? — 13 named jobs green on the head (test, clippy `-D warnings`,
  clippy fuzz-graph, fmt, MSRV 1.90.0, panic-free `--lib`, both red-proofs,
  license ×2, fuzz smoke ×2, cost-capture audit).
- [x] Deployment steps known? — library crate; the develop path is unchanged
  (SPEC-024 only teaches the sensor model to *recognise/parse* a Bayer DNG).
- [x] Rollback plan? — additive, mono path byte-for-byte unchanged; revert is clean.
- [ ] CHANGELOG / release note? — no crate release here; SPEC-024 ships no
  user-visible develop change (it is the foundation SPEC-025/026/027 build on).

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-024-*-timeline.md`.

Answer the three Reflection questions and the two extra Reflection prompts (4 and
5) at the bottom of the spec (`## Reflection` block). Question 5 (before → after):
the Context is the before ("`sensor()` rejects a Bayer DNG outright"), the Goal is
the after ("a Bayer-CFA DNG is accepted at the sensor gate with a structured `Cfa`
mosaic model, mono stays `None`") — confirm against the verify evidence, both
corpus Bayer DNGs gave `Some(Cfa { repeat_dim: [2,2], … })`.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and
  ship are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- Compute `cost.totals` from the session entries (build $105.56 + verify $17.25 +
  ship; skip nulls); `session_count` includes null-numeric sessions.
- Set `complexity_actual` (expected was `M`).
- Mark ship `[x]` in the timeline with the merge date (2026-10-29) + total cost.
- Run `just advance-cycle SPEC-024 ship` (already done by verify — no-op) and then
  `just archive-spec SPEC-024` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual.
- Update STAGE-006's backlog: mark SPEC-024 shipped. SPEC-025/026/027 are the
  already-framed next specs that consume `color_filter`; SPEC-025 is now unblocked.
- `just cost-audit` to confirm SPEC-024 has build + verify cost (it does).

This is the FIRST spec in STAGE-006's backlog, not the last — do **not** run the
Stage Ship prompt (1d) yet.
