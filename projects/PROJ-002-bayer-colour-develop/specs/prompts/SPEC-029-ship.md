# SPEC-029 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ APPROVED (`HANDOFF-069`, at commit
> `1112c18`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ Note: PR #44 was **already merged** to `main` before verify ran, so the
> merge step is done — this ship cycle is the closing ritual (reflection, cost
> totals, archive, CHANGELOG, brag), not a merge. The verify-bookkeeping PR
> (HANDOFF-069 + cost + timeline) is a separate small PR from `verify-spec-029`.

Cycle: ship. PR for SPEC-029 is approved.

Pre-ship checklist (all already true):
- [x] CI passing? — green on PR #44's head at merge; verify re-ran every gate
  locally (test 14 binaries / 0 failed, corpus 7/7; pinned CI clippy `-D warnings`;
  fmt; validate; decisions-audit 0 structural errors; `plane` fuzz 25,936,891
  runs / 91s, 0 crashes).
- [x] Deployment steps known? — library crate; `unpack_into` no longer errors on
  a sample above `WhiteLevel` (specular highlights are legal), raw values
  preserved; `Error::SampleExceedsWhiteLevel` removed.
- [x] Rollback plan? — the change only *removes* a post-assembly assertion + its
  error variant; a revert re-adds the loop. No data path changed, so a revert is
  clean (but would re-break both corpus Bayer planes — DEC-029 is the reason not to).
- [ ] CHANGELOG / release note? — **yes, warranted**: a real library behaviour +
  public-API change. A Bayer plane with a specular highlight above `WhiteLevel`
  (e.g. the Canon 5D-III, sample 15464 > 15000) now unpacks where `unpack_into`
  returned `SampleExceedsWhiteLevel` before, and that error variant is **removed**
  from the public `Error` enum (pre-1.0, crate `0.1.1`, unpublished). Add a
  CHANGELOG line under the unreleased section (note the enum change for the
  eventual 0.1.0 publish, DEC-028).

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-029-*-timeline.md`.

**SPEC-029/FU-1 (from verify, non-blocking) — do this at ship:** drop the stale
"`DEC-029` is informally reserved for the crustyimg LUT boundary — take the next
free number" hedge. The reservation was released in the build brief and DEC-029
IS this decision, so the hedge is now wrong wherever it still appears:
- `SPEC-029-...-amend-dec-008.md` — `## Goal` ("reconcile its number — DEC-029 is
  informally reserved…") and `## Outputs` (the same note under "New DEC").
- `STAGE-006-...md` (~line 184) and `SPEC-024-...md` (in `done/`, ~line 233) — the
  "DEC-029 is informally reserved for the crustyimg LUT boundary" lines.
- `HANDOFF-064` / `HANDOFF-065` (SPEC-024) — the same reservation sentences.
- Optional tidy: `docs/provenance-ledger.md`'s SPEC-028 `src/plane.rs` row
  forward-references "SPEC-029's to relax" — now done; a one-line past-tense fix.
Code is correct everywhere; this is text-only.

Answer the three Reflection questions and the two extra Reflection prompts (4 and
5) at the bottom of the spec (`## Reflection` block). Question 5 (before → after):
the Context is the before ("`plane::unpack_into` asserts every sample `<=
WhiteLevel` and errors otherwise — the Canon 5D-III's 15464 > 15000 is rejected"),
the Goal is the after ("a sample above `WhiteLevel` is legal; the plane unpacks
with raw values preserved, not clamped") — confirm against the verify evidence:
both corpus Bayer planes unpack fully (Canon 5920×3950 max 15464; Nikon 4352×2868
max 4095 > 3808), the mono regression is byte-for-byte (md5 == `dnglab
--raw-checksum` on all 4 decodable frames), SPEC-025 unblocked. Question 4 (worst
defect caught): `none` — clean build; the one spec-text residue (the DEC-029
reservation hedge) is a documentation tidy, not a code defect.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and
  ship are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- Recompute `cost.totals` from the session entries (build $23.45 + verify $20.83
  + ship; skip nulls); `session_count` includes null-numeric sessions. (Verify
  cost is synced into the spec by `just handback-sync SPEC-029`, which verify ran:
  9,134,583 tokens / $20.83.)
- Set `complexity_actual` (expected was `S`).
- Mark ship `[x]` in the timeline with the merge date (2026-10-29, PR #44) + total cost.
- Run `just advance-cycle SPEC-029 ship` (stamps `verify_verdict: approved` on the
  verify→ship transition) and then `just archive-spec SPEC-029` (also moves the
  timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool,
  passing agent + model (model has no fallback).
- Update STAGE-006's backlog: mark SPEC-029 shipped. With SPEC-028 + SPEC-029 both
  done, **SPEC-025** (the bit-exact mosaic-plane oracle) is fully unblocked and is
  the immediate next; **SPEC-027** (register both Bayer DNGs in the manifest + a
  CFA fuzz seed) and **SPEC-026** (colour-metadata oracle) round out STAGE-006.
- `just cost-audit` to confirm SPEC-029 has build + verify cost (it does).

This is a precursor spec in STAGE-006's backlog, not the last — do **not** run the
Stage Ship prompt (1d) yet.
