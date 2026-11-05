# SPEC-036 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ **APPROVED** (`HANDOFF-086`, at commit
> `ef16fa179d77b97cb89dae92b2724a6a48b3c70a`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **The build PR #97 is STILL OPEN** (unlike SPEC-030/032/033/034, whose build PRs were already
> merged by ship). The verify branch (`claude/spec-036-verify-652148`) was cut from the build
> branch, so the verify PR carries **both** the build commit `ef16fa1` (`src/ljpeg.rs`, the seven
> `Ljpeg*` `Error` variants in `src/lib.rs`, the provenance-ledger row) **and** the verify
> bookkeeping (HANDOFF-086, the timeline `[x]`, this ship prompt). **Land them in order:** merge
> the build PR #97 first, then the verify PR (or merge the verify PR alone — it is a superset of
> #97 against `main`, and both reference `SPEC-036`, so `one-spec-per-pr` holds either way).
> Confirm with the orchestrator which merge path it wants; do not merge both and create a conflict.
>
> ⚠ **SPEC-036 adds NO public API** (every `ljpeg` item is `pub(crate)`; the container wiring and
> public surface are STAGE-010's call). It **does** add seven `Ljpeg*` variants to the public
> `#[non_exhaustive] pub enum Error` — **additive and non-breaking**, and **not yet producible**
> (nothing in the crate constructs them until STAGE-010 routes `Compression 7`). A CHANGELOG
> `[Unreleased]` entry is therefore **optional/defensible either way**: if you add one it is
> `### Added` ("typed `Error` variants for the SOF-3 lossless-JPEG front-end, internal until
> STAGE-010 wires it"), NOT `Changed`. The build added **none** — decide at ship; do not treat a
> missing CHANGELOG line as a defect.

Cycle: ship. The SPEC-036 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify re-ran every substantive gate locally on the reviewed tree
  (`ef16fa1` == HEAD): `just test` all binaries green, 0 failed (`ljpeg` 11/11; 132 lib tests
  total); `just lint-ci` green (`-D warnings`, `--all-targets --all-features`, clippy 0.1.98);
  `cargo fmt --check` green; `just validate` green (42 artifacts); `just lint-no-allow` PASS
  (the five-lint panic-free policy is intact and inescapable on the library); `just
  decisions-audit --changed` 0 drift. **Confirm GitHub CI is green on the PR head before merging**
  (constraint `no-panics-on-untrusted-input` enforcement note: observe the job green on the SHA,
  not just locally).
- [x] Deployment steps known? — library crate; this spec adds **NO** public surface (items
  `pub(crate)`) and no runtime behaviour reachable yet (no in-crate caller until STAGE-010). The
  change is a new `src/ljpeg.rs` + seven additive `Error` variants + one provenance-ledger row.
- [x] Rollback plan? — a revert removes the `src/ljpeg.rs` module, the `mod ljpeg;` line and the
  seven `Ljpeg*` variants; nothing else in the crate references them, so a revert is clean and
  touches no shipped decode path. SPEC-037/SPEC-038 are not built.
- [ ] CHANGELOG / release note? — **none present; adding one is optional** (see the ⚠ above — the
  variants are additive, non-breaking, and not yet producible). Decide at ship; if you add one,
  put it under `[Unreleased] → ### Added`, do not duplicate the SPEC-034 `### Changed` entry.

Before starting, mark ship `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-036-sof-3-entropy-front-end-markers-bit-reader-dc-huffman-timeline.md`
(verify set it `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and 5) at the bottom
of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("irradiance decodes only
  uncompressed planes today; a DNG `Compression 7` plane is a lossless-JPEG SOF-3 bitstream it
  currently rejects"); the Goal is the after (the entropy front-end turns that bitstream, down to
  the Huffman-coded scan, into per-sample signed **difference** values plus parsed frame
  parameters). User-visible outcome this spec alone: `none` — the module is `pub(crate)` and
  unreachable until STAGE-010 wires `Compression 7`; it is the first half of STAGE-009's
  compressed-bytes→u16 decoder, exercised only on hand-built bitstreams. (That is a real,
  greppable result, not a blank — the user-facing payoff lands when STAGE-010 routes the decoder.)
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass. Verify ran all
  gates + `lint-no-allow`, confirmed the eight ACs are each actually tested, and did two
  adversarial mutations (neutralised the Huffman over-subscription guard → `oversubscribed…` test
  red; neutralised `EXTEND`'s sign-extension → `receive_extend…` test red, `left: 0 / right: -1`),
  proving the asserts constrain the real decoded values rather than returning green trivially. No
  defect found.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and ship are
  un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ THE TWO METERED CYCLES ARE CONTEXT-OCCUPANCY FLOORS, NOT `/cost` CUMULATIVES.** BOTH the
  build (`HANDOFF-085`) and the verify (`HANDOFF-086`) handbacks carry a real but **lower-bound**
  `tokens_total` read from `ccd_session_mgmt get_usage` (context-window occupancy), because the
  Claude Code desktop (Code tab) exposes no interactive `/cost` (the recurring
  `desktop-cost-tool-unavailable-handbacks-null` gap; neither invented a number, per the contract):
  - **build (HANDOFF-085):** `tokens_total: 299209` — session
    `local_771cd23a-e012-4943-84c1-97c8431a6cb9` (branch `claude/spec-036-build-fbb6dc`, model
    `claude-opus-4-8`). ~25 min.
  - **verify (HANDOFF-086):** `tokens_total: 199216` — branch `claude/spec-036-verify-652148`,
    model `claude-opus-4-8`. ~30 min.
  `just handback-sync SPEC-036` transcribes exactly these numbers into `cost.sessions` (it does
  not estimate). If a true `/cost` cumulative is later recoverable from either transcript, prefer
  it and note the swap; otherwise these floors stand, flagged as floors.
- Recompute `cost.totals` from the session entries (design + build + verify + ship; skip nulls);
  `session_count` includes null-numeric sessions. `tokens_estimate` was 32M.
- Set `complexity_actual` (expected was `M`). The build built **whole** (the pre-named [M] L-risk
  split along the marker/frame | Huffman/scan seam was **not** needed), ~299K-floor tokens, and
  was approved first pass with no punch list. An `M` is the likely landing; set from the recovered
  cost + judgment.
- Mark ship `[x]` in the timeline with the merge date + total cost; keep the ship line pointing at
  `prompts/SPEC-036-ship.md`.
- Confirm `just advance-cycle SPEC-036 ship` ran at verify (verify advances the cycle to `ship` and
  stamps `verify_verdict: approved`). Confirm `task.cycle: ship` and `verify_verdict: approved` are
  both set; if the verdict is somehow not stamped, this is the step that stamps it. Then
  `just archive-spec SPEC-036` (also moves the timeline into `done/`).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool, passing agent +
  model (model has no fallback). Suggested impact: "irradiance can now read the entropy front-end
  of a DNG `Compression 7` sensor plane: the new `pub(crate)` `src/ljpeg.rs` walks the SOF-3
  (lossless JPEG, ITU-T T.81 Annex H) marker/segment structure into the single-component `Frame`
  (precision, lines, samples-per-line, point transform, predictor, restart interval) + the DC
  `HuffTable`, then reads the `0xFF`-destuffing MSB-first entropy scan and `DECODE`+`RECEIVE`+
  `EXTEND`s each sample to a signed difference (`SSSS 0..=16`, `0→0`, `16→32768`) — the first half
  of STAGE-009's compressed-bytes→u16 decoder (SPEC-037 adds predictor reconstruction, SPEC-038
  the fuzz target). Implemented clean-room from the published T.81 standard (DEC-035, patent-clean:
  the lossy-DCT and arithmetic-coding patents expired and never read on SOF-3 Huffman), never from
  Adobe's DNG SDK or an LGPL crate; the `ScanReader` mirrors `plane::BitReader` and adds JPEG byte-
  stuffing. Panic-free on untrusted input: seven additive `Ljpeg*` `Error` variants, every
  marker/length/table/bit read bounded and typed, `#![forbid(unsafe_code)]` + the five-lint deny
  policy, verified by a cold review that ran all gates + `lint-no-allow` and broke the real kernel
  two independent ways to prove the oracle bites. No public API yet (STAGE-010 wires the
  container)."
- Update STAGE-009's backlog: mark SPEC-036 shipped (→ **1 shipped / 2 active** — SPEC-037 and
  SPEC-038 remain).
- Fill the verify PR number into `HANDOFF-086`'s `handback.pr` once the PR is open (the verify
  session left it null pending the PR; the verify session itself opens the PR and should fill it,
  but confirm it is set at ship).

**SPEC-036 is NOT the last spec in STAGE-009.** SPEC-037 (reconstruction) and SPEC-038 (fuzz)
remain (`0 shipped / 3 active` → `1 shipped / 2 active` after this). Do NOT run the Stage Ship
prompt (1d) yet; STAGE-009 (and the PROJ-003 SOF-3 decoder) ships only once SPEC-037 and SPEC-038
land. The natural next step is to dispatch SPEC-037's build.
