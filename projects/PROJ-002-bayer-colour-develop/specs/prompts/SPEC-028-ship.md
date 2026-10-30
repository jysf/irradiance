# SPEC-028 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ APPROVED (`HANDOFF-067`, at commit
> `233ae5c`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ Note: PR #41 was **already merged** to `main` before verify ran, so the
> merge step is done — this ship cycle is the closing ritual (reflection, cost
> totals, archive, CHANGELOG, brag), not a merge. The verify-bookkeeping PR
> (HANDOFF-067 + cost + timeline) is a separate small PR from `verify-spec-028`.

Cycle: ship. PR for SPEC-028 is approved.

Pre-ship checklist (all already true):
- [x] CI passing? — green on PR #41's head at merge; verify re-ran every gate
  locally (test 14 binaries / 0 failed, corpus 7/7; pinned clippy 0.1.98
  `-D warnings`; fmt; validate; `lint-no-allow`; `plane` fuzz 20,465,198 runs /
  91s, 0 crashes).
- [x] Deployment steps known? — library crate; `unpack_into` now assembles N
  strips (N ≥ 1), single-strip byte-for-byte unchanged.
- [x] Rollback plan? — the N-strip loop is a strict generalisation of the
  single-strip path; `N == 1` is byte-identical, so a revert is clean.
- [ ] CHANGELOG / release note? — **yes, warranted** (unlike SPEC-024): this is a
  real library behaviour change — a multi-strip sensor plane (e.g. the Nikon
  D300S's 12 strips) now unpacks where `unpack_into` returned
  `UnsupportedStripLayout` before. No crate release is cut here, but add a
  CHANGELOG line under the unreleased section.

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-028-*-timeline.md`.

**SPEC-028/FU-1 (from verify, non-blocking) — do this at ship:** amend the spec's
`## Outputs` per-strip layer-0 bullet from the literal
`StripByteCounts[i] * 8 == rows_i * width * bits` to the byte-ceil form the code
and `docs/provenance-ledger.md` already use
(`StripByteCounts[i] == ceil(rows_i * width * bits / 8)`, which reduces to the
tight-pack equality when there is no padding). The literal is unsatisfiable for a
padded sub-byte strip — exactly the AC2 case — so the spec text should match the
testable criterion. Code is correct; this is a spec-text correction only.

Answer the three Reflection questions and the two extra Reflection prompts (4 and
5) at the bottom of the spec (`## Reflection` block). Question 5 (before → after):
the Context is the before ("`plane::unpack_into` reads one strip; the Nikon D300S
is 12 strips, so it does not unpack today"), the Goal is the after ("a multi-strip
sensor plane assembles into the contiguous plane buffer; the single-strip path is
byte-for-byte unchanged") — confirm against the verify evidence: the Nikon's
12-strip plane now assembles into the full **4352×2868** buffer (then trips
`SampleExceedsWhiteLevel` 3835 > 3808, which **SPEC-029** relaxes). Question 4
(worst defect caught): `none` — clean build; the two spec-text tensions were
resolved at build and confirmed at verify, no code defect.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and
  ship are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- Recompute `cost.totals` from the session entries (build $102.25 + verify $21.10
  + ship; skip nulls); `session_count` includes null-numeric sessions. (Current
  synced totals: $123.35 / 52,438,502 tokens / 2 sessions, before the ship entry.)
- Set `complexity_actual` (expected was `M`).
- Mark ship `[x]` in the timeline with the merge date (2026-10-29) + total cost.
- Run `just advance-cycle SPEC-028 ship` (stamps `verify_verdict: approved` on the
  verify→ship transition) and then `just archive-spec SPEC-028` (also moves the
  timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool,
  passing agent + model (model has no fallback).
- Update STAGE-006's backlog: mark SPEC-028 shipped. **SPEC-029** (relax
  `SampleExceedsWhiteLevel`, which the Nikon now trips) is the immediate next, then
  **SPEC-025** (the bit-exact mosaic plane oracle) which this unblocks; **SPEC-027**
  registers the Nikon in the manifest + a multi-strip/CFA fuzz seed.
- `just cost-audit` to confirm SPEC-028 has build + verify cost (it does).

This is a precursor spec in STAGE-006's backlog, not the last — do **not** run the
Stage Ship prompt (1d) yet.
