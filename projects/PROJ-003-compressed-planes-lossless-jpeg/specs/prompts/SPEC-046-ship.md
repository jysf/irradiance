# SPEC-046 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ **APPROVED** (`HANDOFF-094`, at commit
> `54704c2fac232e025c86b552d020d3e2c61b875a`, the build PR #117 head). Based on
> `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **SPEC-046 is NOT the last spec in STAGE-011.** It is the re-plan spec
> (`DEC-037`), sequenced `SPEC-043 (shipped) → SPEC-046 (this) → SPEC-044 → SPEC-045`.
> After it merges, **SPEC-044** (the `dnglab --raw-checksum` bit-exact corpus oracle on the
> two real files) is next, then **SPEC-045** (provenance ledger + determinism + container
> hardening) closes STAGE-011 and PROJ-003. Do **not** run a stage-ship for STAGE-011 at this
> merge — the stage is not complete.
>
> ⚠ **The build PR #117 may STILL be OPEN at ship** (same situation as SPEC-036/#97,
> SPEC-038/#106, SPEC-043/#112). The verify branch (`claude/spec-046-verify-0ab12e`) was cut
> from the build branch, so the verify PR carries **both** the build commit (`54704c2` the
> two-component decode) **and** the verify bookkeeping (HANDOFF-094, the `advance-cycle` stamp,
> the timeline `[x]`, this ship prompt). **Land them in order:** merge the build PR #117 first,
> then the verify PR — or merge the verify PR alone, as it is a superset of #117 against `main`
> and both reference `SPEC-046`, so `one-spec-per-pr` holds either way. Confirm with the
> orchestrator which merge path it wants; do not merge both and create a conflict.
>
> ⚠ **SPEC-046 adds NO public API and NO new `Error` variant.** It grows the internal
> `pub(crate)` SOF-3 decoder (`src/ljpeg.rs`) to decode a two-component (`Nf=2`)
> column-interleave frame (`reconstruct_two_component` + `predict_component` + `CompPredict`,
> per-component predictor state, Td-keyed DC tables, an Nf-keyed length check), extends the
> in-harness `encode_lj92_two_component` round-trip oracle, and sizes the `__fuzz_ljpeg` hook by
> `frame.components`. `Nf>2` still returns `LjpegUnsupportedComponentCount`; the `Nf=1` path is
> byte-for-byte unchanged (the `reconstruct` body is diff-identical). `ljpeg` stays `pub(crate)`;
> the published crate (default features) exposes no SOF-3 symbol (SPEC-045 still owns the public
> surface, DEC-036). A CHANGELOG `[Unreleased]` entry is therefore **optional/defensible either
> way**: if you add one it is `### Added` ("two-component Nf=2 column-interleave decode for the
> internal SOF-3 decoder, no public surface"), NOT `Changed`. The build added **none** — decide
> at ship; do not treat a missing CHANGELOG line as a defect.

Cycle: ship. The SPEC-046 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify confirmed PR #117's CI is all-green on the exact build commit
  (`54704c2` == PR head): `rust / test`, `rust / clippy -D warnings`, `rust / clippy — fuzz
  graph (-D warnings)`, `rust / fmt --check`, `rust / license policy (cargo-deny)`, `rust /
  license policy — fuzz graph (cargo-deny)`, both lint red-proofs, `panic-free policy — no
  #[allow] escape`, MSRV (1.90.0), cost-capture, and the `opcode`/`warp_opcode` fuzz-smoke jobs
  all pass. Verify also ran `just lint-ci` (pinned clippy +stable, `-D warnings`, clean),
  `just fmt-check`, `just validate` (50 artifacts), `just decisions-audit --changed` (no drift),
  and `cargo test --lib ljpeg::` locally (27/27, including the six SPEC-046 tests + all shipped
  SPEC-037 tests). **Confirm GitHub CI is green on the final PR head before merging** (observe the
  job green on the SHA, not just the claim).
- [x] Deployment steps known? — library crate; this spec adds **NO** public surface and **NO**
  runtime behaviour reachable by consumers (the internal decoder's `decode_into` stays
  `pub(crate)`, reached only through SPEC-043's `plane::unpack_into` Compression-7 route and the
  dev-only fuzz hook). The user-visible payoff (both held Compression-7 DNGs decoding to a plane)
  is gated on SPEC-044's mapping confirmation.
- [x] Rollback plan? — a revert of `54704c2` removes `reconstruct_two_component`,
  `predict_component`, `CompPredict`, the `Frame.components` field's two-component handling, the
  two-component encoder + reference decoder + six tests, and the Nf-keyed fuzz-hook sizing,
  returning the decoder to single-component-only (`Nf=2` → `LjpegUnsupportedComponentCount`).
  Nothing in the shipped default-build decode path changes (the Nf=1 `reconstruct` body is
  untouched), so a revert is clean and touches no decoded output of any currently-decodable file.
- [ ] CHANGELOG / release note? — **none present; adding one is optional** (see the ⚠ above — no
  public surface, no new producible behaviour until SPEC-044 confirms the mapping). Decide at ship;
  if you add one, put it under `[Unreleased] → ### Added`.

Before starting, mark ship `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-046-two-component-nf2-sof-3-decode-and-column-de-interleave-timeline.md`
(verify set it `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and 5) at the bottom
of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("the shipped STAGE-009 decoder is
  `Nf=1` only and returns `LjpegUnsupportedComponentCount { nf: 2 }` for both held Compression-7
  DNGs, so neither decodes and SPEC-044's bit-exact oracle has no target"); the Goal is the after
  (the decoder now decodes a two-component `Nf=2` column-interleave SOF-3 frame — two per-component
  predictor streams — and de-interleaves the two column components into one `2·X`-wide monochrome
  `u16` plane, proven by an analytic two-component encode→decode round-trip with a red-proof, at
  P=12 and P=14). User-visible outcome this spec alone: `none` — `ljpeg` is `pub(crate)` and the
  real-file mapping is not yet confirmed (SPEC-044); this is the **decode-capability** half, proven
  only on hand-built and fuzzer-generated bitstreams. (A real, greppable result, not a blank — the
  user-facing payoff lands when SPEC-044 confirms both held files decode bit-exact vs `dnglab`.)
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass with no punch
  list. Verify confirmed the six ACs against #117's green CI on the exact commit, witnessed the six
  SPEC-046 tests pass locally, verified the per-component neighbour stride by hand (two output
  columns back for `Ra`), confirmed the Nf=1 `reconstruct` body is diff-identical between `main` and
  the build head, and confirmed the red-proof's independent reference decoder (own loop, native
  shift) agrees with production on honest input and diverges under all three mutations. No defect
  found.

Then:
- Format the answers as the `## Reflection` block in the spec.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and ship are
  un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ THE TWO METERED CYCLES ARE CONTEXT-OCCUPANCY FLOORS, NOT `/cost` CUMULATIVES.** BOTH the
  build (`HANDOFF-093`) and the verify (`HANDOFF-094`) handbacks carry a real but **lower-bound**
  `tokens_total` read from `ccd_session_mgmt get_usage` (context-window occupancy), because the
  Claude Code desktop (Code tab) exposes no interactive `/cost` (the recurring
  `desktop-cost-tool-unavailable-handbacks-null` gap; neither invented a number, per the contract):
  - **build (HANDOFF-093):** `tokens_total: 276963` — branch `claude/spec-046-build-88e024`,
    model `claude-opus-4-8`. ~40 min.
  - **verify (HANDOFF-094):** `tokens_total: 215327` — branch `claude/spec-046-verify-0ab12e`,
    model `claude-opus-4-8`. ~45 min.
  `just handback-sync SPEC-046` transcribes exactly these numbers into `cost.sessions` (it does not
  estimate). If a true `/cost` cumulative is later recoverable from either transcript, prefer it and
  note the swap; otherwise these floors stand, flagged as floors.
- Recompute `cost.totals` from the session entries (design + build + verify + ship; skip nulls);
  `session_count` includes null-numeric sessions. `tokens_estimate` was 32M.
- Set `complexity_actual` (expected was `M`). The build built whole (~277K-floor tokens), approved
  first pass with no punch list; verify was a clean cold review (~215K-floor tokens). An `M` is the
  likely landing — the two-component interleaved decode + per-component predictor state + the
  extended encoder + reference decoder + six tests is a genuine `M`, comparable to SPEC-037; set
  from the recovered cost + judgment.
- Mark ship `[x]` in the timeline with the merge date + total cost; keep the ship line pointing at
  `prompts/SPEC-046-ship.md`.
- `just advance-cycle SPEC-046 ship` already ran at verify (it stamped `task.cycle: ship` +
  `verify_verdict: approved`). Confirm both are set; then `just archive-spec SPEC-046` (also moves
  the timeline into `done/`).
- Confirm `HANDOFF-094`'s `handback.pr` is set to the verify PR number (the verify session opens the
  PR and fills it).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool, passing agent +
  model (model has no fallback). Suggested impact: "irradiance's internal SOF-3 (lossless JPEG,
  ITU-T T.81 Annex H) decoder — the `pub(crate)` `src/ljpeg.rs` that turns a DNG `Compression 7`
  sensor plane's bitstream into a u16 plane — now decodes the **two-component (`Nf=2`)
  column-interleave** frame both held monochrome Compression-7 DNGs actually use (Leica M Typ246
  P=12 and Pentax K-3 III P=14), not just the single-component case. A new `reconstruct_two_component`
  decodes the interleaved scan MCU by MCU with genuine **per-component predictor state** (each
  component's own `Ra`/`Rb`/`Rc` two-output-columns-back / one-plane-row-back neighbours and the
  §H.2.1 boundary rules applied independently) and de-interleaves the two column phases into one
  `2·X`-wide monochrome plane; the DC Huffman table(s) are keyed by `Td` (both held files share
  `Td=0`, not hard-coded); the length check keys on `Nf`. The `Nf=1` path is byte-for-byte unchanged
  (the `reconstruct` body is diff-identical) and `Nf>2` (Bayer/CFA colour) stays a clean typed
  `LjpegUnsupportedComponentCount` (DEC-037). Proven by an analytic two-component encode→decode
  round-trip that is bit-exact at P=12/P=14 across predictors 1/4/6 with and without restart, with
  an `oracle-must-be-shown-red` red-proof that goes red on a perturbed predictor, a mutated Huffman
  table, and a swapped component order — the red-proof's independent reference decoder (own loop,
  native shift) agreeing with production on honest input defeats the shared-`predict` tautology.
  `ljpeg` stays `pub(crate)`, no new `Error` variant, no new dependency, under
  `#![forbid(unsafe_code)]` + the five-lint deny policy. This restores SPEC-044's bit-exact-vs-`dnglab`
  target (the real-file de-interleave MAPPING is confirmed there, not here —
  `shared-path-oracle-tautology`); verified by a cold review that reran the non-CI gates locally,
  checked the per-component stride and the Nf=1 no-change by hand, and witnessed all six tier-A
  tests pass."
- Update STAGE-011's backlog: mark SPEC-046 shipped. **STAGE-011 is NOT complete** — SPEC-044
  (bit-exact `dnglab` corpus oracle) and SPEC-045 (provenance / determinism / hardening) remain.
  **Do NOT run Prompt 1d (Stage Ship) for STAGE-011.**

**SPEC-046 is the re-plan spec, not the last spec in STAGE-011.** With it shipped, both held
Compression-7 DNGs are decodable by the internal decoder (self-consistency + per-component
prediction proven analytically); the real-file bit-exact confirmation and the exact de-interleave
mapping are **SPEC-044** (`dnglab --raw-checksum`), already in the stage backlog, which is next.
Do not dispatch a STAGE-011 stage-ship at this merge.
