# SPEC-038 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ **APPROVED** (`HANDOFF-090`, at commit
> `d6b73ad9aa92f2bfdd13ba043b80c7ff04888cdc`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **This is the LAST spec in STAGE-009.** SPEC-036 (entropy front-end) and SPEC-037
> (reconstruction) are shipped; SPEC-038 (fuzz + truncation hardening) is `3 of 3`. When it
> merges, STAGE-009 (the single-component SOF-3 lossless-JPEG core decoder) is **complete**, so
> **run Prompt 1d (Stage Ship) for STAGE-009 next** — write the stage-level reflection (including
> how many framed outlines survived), set the stage `status: shipped`, and roll up the stage cost.
> Do not leave STAGE-009 `active` after this merge.
>
> ⚠ **The build PR #106 may STILL be OPEN at ship** (same situation as SPEC-036/#97). The verify
> branch (`claude/spec-038-verify-862e91`) was cut from the build branch, so the verify PR carries
> **both** the build commits (`1c3e5ed` the fuzz target + `d6b73ad` the build-cycle bookkeeping)
> **and** the verify bookkeeping (HANDOFF-090, the timeline `[x]`, this ship prompt). **Land them
> in order:** merge the build PR #106 first, then the verify PR — or merge the verify PR alone, as
> it is a superset of #106 against `main` and both reference `SPEC-038`, so `one-spec-per-pr` holds
> either way. Confirm with the orchestrator which merge path it wants; do not merge both and create
> a conflict.
>
> ⚠ **SPEC-038 adds NO public API and NO new `Error` variant.** It adds a dev-only `fuzzing` cargo
> feature (`[features] fuzzing = []`, empty) + a `#[cfg(feature = "fuzzing")] #[doc(hidden)] pub fn
> __fuzz_ljpeg` hook, a `ljpeg` libFuzzer target, 14 own-work seeds, and two in-crate panic-free
> replay unit tests. `ljpeg` stays `pub(crate)`; the published crate (default features) exposes no
> SOF-3 symbol (STAGE-010 still owns the real public surface, DEC-036). A CHANGELOG `[Unreleased]`
> entry is therefore **optional/defensible either way**: if you add one it is `### Added` ("fuzz
> target + panic-free hardening for the internal SOF-3 decoder, no public surface"), NOT `Changed`.
> The build added **none** — decide at ship; do not treat a missing CHANGELOG line as a defect.

Cycle: ship. The SPEC-038 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify confirmed PR #106's CI is all-green on the exact build commit
  (`d6b73ad` == verify HEAD): `rust / test`, `rust / clippy -D warnings`, `rust / clippy — fuzz
  graph (-D warnings)`, `rust / fmt --check`, `rust / license policy (cargo-deny)`, `rust / license
  policy — fuzz graph (cargo-deny)`, both red-proofs, `panic-free policy — no #[allow] escape`,
  MSRV, cost-capture, and the `opcode`/`warp_opcode` fuzz-smoke jobs all pass. Verify also ran
  `just validate` locally (46 artifacts) and a nightly `cargo +nightly fuzz run ljpeg` spot-run
  (**3,098,220 runs in 31 s, no crash**), on top of the build's reported 2.55M. **Confirm GitHub CI
  is green on the final PR head before merging** (observe the job green on the SHA, not just the
  claim).
- [x] Deployment steps known? — library crate; this spec adds **NO** public surface and **NO**
  runtime behaviour reachable by consumers. The change is a dev-only `fuzzing` feature + a
  doc-hidden fuzz hook + a `ljpeg` fuzz target + 14 committed seeds + two in-crate unit tests.
- [x] Rollback plan? — a revert removes the `fuzzing` feature, the `__fuzz_ljpeg` entry, the
  `fuzz/fuzz_targets/ljpeg.rs` target + its `[[bin]]`, the `fuzz/seeds/ljpeg/` seeds, the
  `examples/fuzz-seeds.rs` `ljpeg_seeds()` block, and the two `src/ljpeg.rs` `mod tests` tests.
  Nothing in the shipped decode path references any of them, so a revert is clean and touches no
  decoded output.
- [ ] CHANGELOG / release note? — **none present; adding one is optional** (see the ⚠ above — no
  public surface, no new producible behaviour). Decide at ship; if you add one, put it under
  `[Unreleased] → ### Added`.

Before starting, mark ship `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-038-sof-3-decoder-fuzz-truncation-hardening-timeline.md`
(verify set it `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and 5) at the bottom
of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("the complete single-component SOF-3
  decoder — SPEC-036 + SPEC-037 — exists in `src/ljpeg.rs`, but its panic-freedom on hostile input
  is asserted only by its own unit tests"); the Goal is the after (a libFuzzer `ljpeg` target + 14
  own-work hostile/valid seeds + two in-crate panic-free replay tests prove the decoder returns a
  typed `Error`, never panics, on truncated / oversized-table / forged-dimension bitstreams).
  User-visible outcome this spec alone: `none` — `ljpeg` is `pub(crate)` and unreachable until
  STAGE-010 wires `Compression 7`; this is the **safety** half of STAGE-009's deliverable,
  exercised only on hand-built and fuzzer-generated bitstreams. (A real, greppable result, not a
  blank — the user-facing payoff lands when STAGE-010 routes the decoder through the container.)
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass. Verify ran
  `just validate` + confirmed the six ACs against #106's green CI on the exact commit, checked the
  replay tests are non-tautological (each hostile input asserts a distinct typed `Error` variant
  that exists in `lib.rs`; the corpus test asserts non-empty), confirmed DEC-036 keeps `ljpeg`
  `pub(crate)` while the fuzz-graph gate stays green, and ran a nightly fuzz spot-run (3.1M runs,
  no crash). No defect found.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and ship are
  un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ THE TWO METERED CYCLES ARE CONTEXT-OCCUPANCY FLOORS, NOT `/cost` CUMULATIVES.** BOTH the
  build (`HANDOFF-089`) and the verify (`HANDOFF-090`) handbacks carry a real but **lower-bound**
  `tokens_total` read from `ccd_session_mgmt get_usage` (context-window occupancy), because the
  Claude Code desktop (Code tab) exposes no interactive `/cost` (the recurring
  `desktop-cost-tool-unavailable-handbacks-null` gap; neither invented a number, per the contract):
  - **build (HANDOFF-089):** `tokens_total: 267704` — session
    `local_264196a3-b026-4f22-b7e8-e6c1ce429398` (branch `claude/spec-038-build-a93ad6`, model
    `claude-opus-4-8`). ~40 min.
  - **verify (HANDOFF-090):** `tokens_total: 185694` — branch `claude/spec-038-verify-862e91`,
    model `claude-opus-4-8`. ~50 min.
  `just handback-sync SPEC-038` transcribes exactly these numbers into `cost.sessions` (it does not
  estimate). If a true `/cost` cumulative is later recoverable from either transcript, prefer it and
  note the swap; otherwise these floors stand, flagged as floors.
- Recompute `cost.totals` from the session entries (design + build + verify + ship; skip nulls);
  `session_count` includes null-numeric sessions. `tokens_estimate` was 20M.
- Set `complexity_actual` (expected was `S`). The build built whole (~268K-floor tokens), approved
  first pass with no punch list; verify was a clean cold review (~186K-floor tokens). An `S` is the
  likely landing; set from the recovered cost + judgment.
- Mark ship `[x]` in the timeline with the merge date + total cost; keep the ship line pointing at
  `prompts/SPEC-038-ship.md`.
- `just advance-cycle SPEC-038 ship` already ran at verify (it stamped `task.cycle: ship` +
  `verify_verdict: approved`). Confirm both are set; then `just archive-spec SPEC-038` (also moves
  the timeline into `done/`).
- Fill the verify PR number into `HANDOFF-090`'s `handback.pr` once the PR is open (the verify
  session opens the PR and should fill it, but confirm it is set at ship).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool, passing agent +
  model (model has no fallback). Suggested impact: "irradiance's internal single-component SOF-3
  (lossless JPEG, ITU-T T.81 Annex H) decoder — the `pub(crate)` `src/ljpeg.rs` that turns a DNG
  `Compression 7` sensor plane's bitstream into a u16 plane (SPEC-036 entropy front-end + SPEC-037
  reconstruction) — is now proven panic-free on hostile input: a new `ljpeg` libFuzzer target
  reaches the otherwise-internal decoder through a dev-only `fuzzing` cargo feature (DEC-036 — a
  feature, not `cfg(fuzzing)`, so the plain-`cargo-clippy` fuzz-graph CI gate still compiles it)
  and drove it 3.1M+ runs with no crash; 14 own-work SOF-3 seeds (9 valid across predictors 1..7 /
  8-12-16-bit / point-transform / restart + 5 hostile: truncated header, truncated scan,
  oversized DHT, Nf≠1, bad marker) and two in-crate panic-free replay unit tests assert every
  truncated / oversized-table / forged-65535×65535 bitstream returns a typed `Ljpeg*` `Error`
  rather than panicking, under `#![forbid(unsafe_code)]` + the five-lint deny policy. `ljpeg` stays
  `pub(crate)` — no public API change — so STAGE-010 still owns the real surface. This completes
  STAGE-009, the single-component SOF-3 core decoder; verified by a cold review that reran the
  non-CI gate locally, confirmed the replay tests bite (distinct typed variant per hostile shape),
  and ran its own nightly fuzz."
- Update STAGE-009's backlog: mark SPEC-038 shipped (→ **3 shipped / 0 active** — the stage is
  complete).

**SPEC-038 IS the last spec in STAGE-009.** With it shipped, STAGE-009 (and PROJ-003's
single-component SOF-3 lossless-JPEG core decoder) is complete. **Run Prompt 1d (Stage Ship) for
STAGE-009 next:** write the stage-level reflection, set `STAGE-009.status: shipped`, roll up the
stage cost, and tee up STAGE-010 (DNG container binding + `dnglab` bit-exact), which is already
framed. Do not dispatch any further STAGE-009 build/verify — there are none.
