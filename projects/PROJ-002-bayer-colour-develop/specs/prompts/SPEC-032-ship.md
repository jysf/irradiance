# SPEC-032 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ APPROVED (`HANDOFF-079`, at commit
> `a7eae9e`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **The build PR #71 is ALREADY MERGED to `main`** (the maintainer merged it
> early — like SPEC-030's #63, unlike SPEC-031's #67). So the code already landed;
> the verify branch (`claude/spec-032-verify-cfbf9c`) carries **docs only** — the
> timeline `[x]`, this ship prompt, and the `advance-cycle` stamps. Merging the
> verify PR lands NO code. This ship cycle is therefore the closing ritual only
> (reflection, cost totals, `complexity_actual`, archive, brag) plus merging the
> docs-only verify PR.
>
> ⚠ **SPEC-032 is the LAST STAGE-007 spec.** Shipping it makes STAGE-007 fully
> shipped (3 shipped / 0 active). After the closing ritual, run the **STAGE-007
> stage-ship (Prompt 1d)**, then STAGE-008 (colour + ColorChecker ΔE).

Cycle: ship. The SPEC-032 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify re-ran every substantive gate locally on `a7eae9e`:
  `just test` all green (whole suite; **demosaic_oracle 23/23**; every STAGE-006/007
  regression oracle green; tier-B corpus SKIPs loud, not silent; the one `ignored`
  is the pre-existing SPEC-017/SB-1 Q2M finding, unrelated), `just lint-ci` green
  (pinned stable clippy, `-D warnings`), `just lint-fuzz` green (DEC-027),
  `cargo fmt --check` green, `just validate` green, `just decisions-audit --changed`
  0 matches. Confirm GitHub CI is green on the verify PR's head before merging.
- [x] Deployment steps known? — library crate; **no public surface change at all**
  this spec. `git show --stat a7eae9e` touches only `fuzz/`, `examples/`, `tests/`
  and docs — **no `src/`**. `demosaic_into`, the `Error` variants, the `irr
  demosaic` P6 output are untouched. This is test + fuzz hardening of the already-
  shipped SPEC-031 kernel; nothing is deployed.
- [x] Rollback plan? — the change is confined to a new fuzz target + seeds + the
  demosaic oracle's phase fixtures. A revert removes the added tests/fuzz; the
  demosaic kernel (SPEC-031) and every other path are untouched. No src regression
  surface.
- [x] CHANGELOG / release note? — **NONE.** Test + fuzz only, no public API change
  → no CHANGELOG, per the repo convention (cf. SPEC-025 / SPEC-027, which also
  shipped without one). Confirm the build did NOT add a CHANGELOG entry (it did
  not — `git show a7eae9e -- CHANGELOG.md` is empty) and do NOT add one.

Before starting, mark ship `[~]` in
`projects/PROJ-002-bayer-colour-develop/specs/SPEC-032-cfa-demosaic-fuzz-panic-free-hardening-on-timeline.md`
(verify already set it to `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and
5) at the bottom of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("SPEC-030 + SPEC-031
  built `demosaic_into` + its Tier-A oracle; only the 1×1/2×2/3×5 unit tests
  exercised it, and the oracle's phase fixtures covered only RGGB + BGGR — the
  anti-diagonal greens"), the Goal is the after ("a `demosaic` libFuzzer target
  making the panic-free contract a *fuzzed* guarantee, GRBG + GBRG bit-exact
  fixtures closing SPEC-030/FU-1, and DC-preservation across all four Bayer
  phases"). The user-visible outcome: `irr demosaic` is now proven panic-free under
  fuzzing (8.6M–11M runs, 0 crashes) and bit-exact on all four Bayer phases — a
  robustness/coverage result, not a new feature (write the outcome, note there is
  no new public API).
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass.
  Verify independently hand-recomputed one interior pixel per NEW phase from the HA
  definition (GRBG (3,2)=(135,140,138), GBRG (1,2)=(94,92,96) — both matching the
  pinned asserts + the EXPECTED tables), re-ran the nightly fuzz (8.6M runs / 0
  crashes), and confirmed the panic-free test is non-vacuous (an Ok control guards
  against an all-errors pass) — finding no defect.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and
  ship are un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ RECOVER THE TWO METERED NULLS.** BOTH the build and the verify handbacks
  carry `tokens_total: null` because `/cost` is not exposed as a tool in a Claude
  Code desktop (Code tab) session (neither invented a number, per the contract).
  Read `/cost` from each session's transcript and fill `cost.sessions` directly:
  - **build (HANDOFF-078):** anchor on scratchpad UUID
    `0d68a8ef-a193-413b-8680-dfdf10ad0777` (model `claude-opus-4-8`).
  - **verify (HANDOFF-079):** anchor on scratchpad UUID
    `0b206331-7897-4011-ae34-b36b97f66c84` (model `claude-opus-4-8`).
  Anchor on the UUID, not a text grep (memory: `identify-own-transcript-for-cost-handback`
  — a stale pre-`/clear` file can text-match and be a different model). Do NOT run
  `just handback-sync` expecting it to supply these numbers — the handbacks are
  null; sync only transcribes what is there.
- Recompute `cost.totals` from the session entries (build + verify + ship; skip
  nulls once the two are recovered); `session_count` includes null-numeric sessions.
- Set `complexity_actual` (expected was `S`). This was a test + fuzz spec with no
  `src` change that reused SPEC-030/031's oracle harness, the `develop.rs` fuzz
  template and SPEC-027's CFA builders wholesale; it approved first pass. Set from
  the recovered cost + your judgment (an `S` is the likely landing).
- Mark ship `[x]` in the timeline with the merge date + total cost.
- Confirm `just advance-cycle SPEC-032 ship` already ran at verify (it did — verify
  advances the cycle to `ship` and stamps `verify_verdict: approved`). Confirm
  `task.cycle: ship` and `verify_verdict: approved` are both set; if the verdict is
  somehow not stamped, this is the step that stamps it. Then `just archive-spec
  SPEC-032` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool,
  passing agent + model (model has no fallback). Suggested impact: "irradiance's
  demosaic now has a libFuzzer target proving `demosaic_into` panic-free on
  arbitrary bytes (8.6M–11M runs, 0 crashes) and bit-exact oracle fixtures on ALL
  FOUR Bayer phases — GRBG/GBRG (main-diagonal greens) added this spec, closing
  SPEC-030/FU-1, each hand-computed AND reproduced by an independent clean-room
  reference, with the new phases' interior pixels re-derived from the HA arithmetic
  by an independent reviewer. DC-preservation now holds on all four phases. Test +
  fuzz only, no kernel change (DEC-031 HA intact)."
- Update STAGE-007's backlog: mark SPEC-032 shipped (count → **3 shipped / 0
  active** — STAGE-007 fully shipped).

**SPEC-032 is the LAST STAGE-007 spec.** Once it is shipped and archived, STAGE-007
is fully shipped — so **DO run the Stage Ship prompt (1d) for STAGE-007 next**
(status → shipped, gated orchestration_cost, stage reflection, walk the signals),
then frame STAGE-008 (colour + ColorChecker ΔE — still needs a ColorChecker shot
from the user: same camera, even light, no clipped patches).

- `just cost-audit` to confirm SPEC-032 has real build + verify cost once the two
  nulls are recovered (it will FAIL on a shipped spec with a null metered cycle —
  that is the gate working; recover the numbers first).
