# SPEC-043 — SHIP prompt

> Cycle: **ship**. Verify returned ✅ **APPROVED** (`HANDOFF-092`, at commit
> `594dcc8f0e7c8de6e644d0e32b1dae51bbfd229f`). Based on `FIRST_SESSION_PROMPTS.md` Prompt 5.
>
> ⚠ **This is the FIRST spec in STAGE-011, not the last — do NOT run Stage Ship.** STAGE-011
> (DNG `Compression 7` binding + bit-exact corpus verification) still has SPEC-044 (the bit-exact
> `dnglab --raw-checksum` corpus oracle) and SPEC-045 (provenance + determinism + container fuzz)
> ahead of it, and both now need **re-planning** before build — see the ⚠ below. Leave
> `STAGE-011.status: active`.
>
> ⚠ **The build PR #112 is ALREADY MERGED to `main`** (unlike SPEC-036/#97 and SPEC-038/#106, whose
> build PRs were open at ship). The merged build commit `594dcc8` **is** `main`'s HEAD. The verify
> branch (`claude/spec-043-verify-7e8390`) was cut from `main` after that merge, so the verify PR
> carries **only the verify bookkeeping** (HANDOFF-092, the `advance-cycle` stamp, the timeline
> `[x]`, this ship prompt) atop the already-merged change — a clean superset against `main`, no code
> diff. **Merge the verify PR alone**; there is no second build PR to land first and no
> `one-spec-per-pr` tension (both reference `SPEC-043`).
>
> ⚠ **A separate orchestrator PR (#113) carries the `Nf=2` re-plan.** The build's real-file probe
> found both held Compression-7 files are **two-component (`Nf = 2`) SOF-3** frames, logged in
> `guidance/questions.yaml` as `stage-011-held-files-are-two-component-sof3` (orchestrator-owned).
> That is a STAGE-011 re-planning matter for SPEC-044/045, **NOT a SPEC-043 defect** — SPEC-043's
> deliverable (recognise Compression 7 and route the strip to the decoder) is complete and verified.
> Do not conflate the two; do not touch `guidance/questions.yaml` from the SPEC-043 ship.
>
> ⚠ **SPEC-043 adds NO new public API signature, NO new `Error` variant, NO new dependency.** It
> changes the **behaviour** of the existing `pub fn plane::unpack_into` (via a `match
> sensor.compression`): a `Compression 7` plane now routes to `ljpeg::decode_into` instead of
> returning `UnsupportedCompression`. `decode_into` stays `pub(crate)` (DEC-036 untouched; the only
> new surface is a `#[cfg(test)]`-only `encode_lj92` re-export). A CHANGELOG `[Unreleased]` entry is
> **defensible either way**: if you add one, `### Changed` ("DNG `Compression 7` / SOF-3
> lossless-JPEG sensor planes now route through the internal decoder instead of being rejected at
> the compression gate; single-component single/multi-strip decodes bit-exact, multi-component
> (`Nf≥2`) is a clean typed deferral pending STAGE-011 follow-ups") is the honest framing. The build
> added none — decide at ship; a missing CHANGELOG line is not a defect.

Cycle: ship. The SPEC-043 verify PR is approved.

Pre-ship checklist:
- [x] CI passing? — verify confirmed PR #112's CI is all-green on the exact build commit
  (`594dcc8` == `main` HEAD == verify base): `rust / test`, `rust / clippy -D warnings`,
  `rust / clippy — fuzz graph (-D warnings)`, `rust / fmt --check`, `rust / license policy
  (cargo-deny)`, `rust / license policy — fuzz graph (cargo-deny)`, both lint red-proofs,
  `rust / panic-free policy — no #[allow] escape (--lib)`, MSRV (1.90.0), cost-capture audit, and
  the `opcode` / `warp_opcode` fuzz-smoke jobs all pass. Verify also ran `just lint-ci`
  (clippy 0.1.98 +stable, `-D warnings`, clean), `just fmt-check`, `just validate` (49 artifacts),
  `just decisions-audit --changed 594dcc8~1` (no drift), the six SPEC-043 tier-A tests individually
  (all ok), and the tier-B real-file routing test with `IRRADIANCE_CORPUS_DIR` set
  (`compression7_planes_route_to_the_decoder` — ok; the real Leica + Pentax C7 files reach the
  decoder). **Confirm GitHub CI is green on the final verify-PR head before merging** (observe the
  job green on the SHA — the verify PR has no code diff, so its CI should mirror #112's).
- [x] Deployment steps known? — library crate. The change is a behaviour change to the existing
  `pub fn plane::unpack_into` (compression dispatch), reachable through the `irr` dev binary's
  `ifd` / `develop` / `demosaic` / `color` / `unpack` subcommands transparently. No new public
  symbol, no new runtime dependency.
- [x] Rollback plan? — a revert of `594dcc8` restores the unconditional `require_uncompressed()?`
  gate at the top of `unpack_into`, re-folds `unpack_uncompressed` back inline, removes `unpack_jpeg`
  and the six tier-A tests, reverts the `irr` diagnostics and the `tests/plane_unpack.rs` split, and
  drops the `#[cfg(test)]` `encode_lj92` re-export. The uncompressed decode path is byte-for-byte
  identical before and after, so a revert touches no uncompressed decoded output.
- [ ] CHANGELOG / release note? — **none present; adding one is optional** (see the ⚠ above). If you
  add one, put it under `[Unreleased] → ### Changed`.

Before starting, mark ship `[~]` in
`projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-043-compression-7-recognition-strip-path-routing-timeline.md`
(verify set it `[x]` for verify and left ship `[ ]`).

Answer the three Reflection questions and the two extra Reflection prompts (4 and 5) at the bottom
of the spec (`## Reflection` block).
- **Question 5 (before → after):** the Context is the before ("STAGE-009 shipped
  `ljpeg::decode_into` but kept it `pub(crate)` with no caller — a DNG whose sensor IFD carries
  `Compression == 7` still returns `Error::UnsupportedCompression` at the plane gate"); the Goal is
  the after (`unpack_into` recognises `Compression 7` and routes each strip through
  `ljpeg::decode_into`, so a Compression-7 DNG decodes to a `u16` plane — single-component
  single/multi-strip bit-exact — while uncompressed stays byte-for-byte unchanged and the binding
  stays panic-free). User-visible outcome this spec alone: a **single-component** Compression-7 DNG
  now decodes to a plane via `unpack_into` instead of `UnsupportedCompression`; the two **held**
  files (both `Nf=2` multi-component) now reach the decoder and defer cleanly with a typed
  `LjpegUnsupportedComponentCount` rather than the compression-gate rejection. End-to-end bit-exact
  decode of a real held file waits on multi-component support (SPEC-044/045 re-plan) — a real,
  greppable partial result, not a blank.
- **Question 4 (worst defect caught):** `none` — clean build, approved first pass, no punch list.
  The `Nf=2` discovery was a real-file **finding** (the decoder correctly defers with a typed
  error), not a defect in SPEC-043's routing deliverable. Verify ran the four gates, confirmed the
  six ACs against #112's green CI on the exact commit plus local targeted runs, checked the
  uncompressed path is a verbatim extraction (characterization test + full suite as the real guard),
  and behaviorally confirmed the real held files reach `decode_into`.

Then:
- Format the answers as the `## Reflection` block in the spec.
- **Append the `## Follow-ups` table** — one row per `FU-N`/finding raised across SPEC-043's cycles.
  SPEC-043 raised **no `SB-N`/`FU-N`** at verify (✅ APPROVED, no punch list). The `Nf=2`
  two-component finding is the one carried item; disposition it as `spec: SPEC-044` (the
  re-planning of SPEC-044/045 owns it) with a pointer to
  `guidance/questions.yaml` → `stage-011-held-files-are-two-component-sof3`. It does not cross this
  ship undecided.
- Append a ship cost session entry to `cost.sessions` (`cycle: ship`); design and ship are
  un-metered main-loop cycles, so a `null`-with-note is acceptable there.
- **⚠ THE TWO METERED CYCLES ARE CONTEXT-OCCUPANCY FLOORS, NOT `/cost` CUMULATIVES.** BOTH the
  build (`HANDOFF-091`) and the verify (`HANDOFF-092`) handbacks carry a real but **lower-bound**
  `tokens_total` read from `ccd_session_mgmt get_usage` (context-window occupancy), because the
  Claude Code desktop (Code tab) exposes no interactive `/cost` (the recurring
  `desktop-cost-tool-unavailable-handbacks-null` gap; neither invented a number, per the contract):
  - **build (HANDOFF-091):** `tokens_total: 306881` — branch `claude/spec-043-build-5bc7e8`,
    model `claude-opus-4-8`. ~55 min.
  - **verify (HANDOFF-092):** `tokens_total: 205139` — branch `claude/spec-043-verify-7e8390`,
    model `claude-opus-4-8`. ~45 min.
  `just handback-sync SPEC-043` transcribes exactly these numbers into `cost.sessions` (it does not
  estimate). If a true `/cost` cumulative is later recoverable from either transcript, prefer it and
  note the swap; otherwise these floors stand, flagged as floors.
- Recompute `cost.totals` from the session entries (design + build + verify + ship; skip nulls);
  `session_count` includes null-numeric sessions. `tokens_estimate` was 28M.
- Set `complexity_actual` (expected was `M`). The build built whole (~307K-floor tokens), approved
  first pass with no punch list; verify was a clean cold review (~205K-floor tokens). An `M` is the
  likely landing (it matched the estimate and reused SPEC-028's assembly without a new mechanism);
  set from the recovered cost + judgment.
- Mark ship `[x]` in the timeline with the merge date + total cost; keep the ship line pointing at
  `prompts/SPEC-043-ship.md`.
- `just advance-cycle SPEC-043 ship` already ran at verify (it stamped `task.cycle: ship` +
  `verify_verdict: approved`). Confirm both are set; then `just archive-spec SPEC-043` (also moves
  the timeline into `done/`).
- **Confirm `docs/provenance-ledger.md` and `docs/conformance-matrix.md` are current.** SPEC-043
  adds **no new algorithm** (it feeds the already-ledgered SOF-3 decoder real bytes), so the
  provenance ledger needs no new row — confirm that is the case. The conformance matrix: Compression
  7 is now *recognised and routed*, but no real held file decodes end-to-end yet (`Nf=2`), so do not
  mark the held files as decoded — note the routing-reached-but-deferred state if the matrix tracks
  it, consistent with the questions-ledger item.
- Fill the verify PR number into `HANDOFF-092`'s `handback.pr` once the PR is open (the verify
  session opens the PR and should fill it via a bookkeeping commit — like the build's "record PR 112"
  commit — so confirm it is set at ship).
- Log the brag entry (before→after impact) per the ship ritual — the brag tool, passing agent +
  model (model has no fallback). Suggested impact: "irradiance now RECOGNISES DNG `Compression 7`
  (SOF-3 lossless JPEG) on the sensor IFD and ROUTES each strip through the shipped single-component
  decoder (`ljpeg::decode_into`) — the binding half of STAGE-011. `plane::unpack_into` dispatches on
  the `Compression` tag: uncompressed planes take the existing sub-byte / byte-aligned path
  byte-for-byte (verbatim-extracted, pinned by a characterization test), a Compression-7 plane
  decodes each independent SOF-3 strip into its own `rows × width` row band (reusing SPEC-028's
  multi-strip slicing, skipping the meaningless layer-0 packed-byte check and the sub-byte unpack),
  and any other code is the clean typed `UnsupportedCompression` it always was. Six tier-A tests
  (hand-built SOF-3 via the STAGE-009 encoder) prove single-strip bit-exact decode for both
  container byte orders, the unchanged uncompressed path, two-strip multi-band decode, a
  forged-dimension `LjpegPlaneWrongLength`, a truncated-strip typed error, and the non-1/7
  rejection — all panic-free under `#![forbid(unsafe_code)]` + the five-lint deny policy, no new
  public API / `Error` variant / dependency. The two held real files (Leica M-Monochrom Typ246,
  Pentax K-3 III Mono) now REACH the decoder instead of the compression gate — the real-file
  behavioral surface — though both carry a two-component (`Nf=2`) SOF-3 frame the single-component
  decoder defers on cleanly, which reshapes SPEC-044/045 (bit-exact corpus oracle) into a
  multi-component problem (logged in `guidance/questions.yaml`). Verified by a cold review that ran
  the four gates, the six tier-A tests individually, and the tier-B real-file routing test on the
  corpus, and confirmed the uncompressed path is a byte-for-byte verbatim extraction."
- Update STAGE-011's backlog: mark SPEC-043 shipped. **STAGE-011 stays `active`** — SPEC-044 and
  SPEC-045 remain (and need re-planning for the `Nf=2` reality). Do NOT run Stage Ship.

**SPEC-043 is the FIRST spec of STAGE-011.** With it shipped, the DNG container recognises and
routes Compression 7 to the decoder; the stage's remaining specs (SPEC-044 bit-exact corpus oracle,
SPEC-045 provenance + determinism + container fuzz) are next, after the orchestrator re-plans them
around the `Nf=2` multi-component finding. Do not dispatch SPEC-044/045 build until that re-plan
lands.
