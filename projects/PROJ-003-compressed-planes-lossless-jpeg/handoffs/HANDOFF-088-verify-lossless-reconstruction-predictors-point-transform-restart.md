---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-088
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-037

project:
  id: PROJ-003
  stage: STAGE-009
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 235814             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 40
  branch: claude/spec-037-verify-7b258a
  pr: 103                          # the verify PR (carries build commit e5a1062 + verify bookkeeping)
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "Verdict: ⚠ PUNCH LIST (1 low-severity item — a test-coverage gap, not a bug; code is correct, all four gates + all 8 acceptance criteria green). No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-085/086/087). 235814 is this verify session's context-window occupancy at handback, read via ccd_session_mgmt get_usage (235,814 of 1,000,000, 24%). It is a snapshot / lower bound: true cumulative input+output across the verify's turns is higher (per-turn context resends not captured). Not invented, but not a /cost number — treat as a FLOOR in rollups, exactly as HANDOFF-085/086/087 reported; the orchestrator supersedes it from the transcript at ship."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-088: Lossless reconstruction (predictors, point transform, restart) — VERIFY

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-037` to `claude-opus-4-8` (acting
as verifier) for the **`verify`** cycle: a **cold** review of the SPEC-037 build
(PR #102, branch `claude/spec-037-build-233973`) — the lossless-JPEG
reconstruction (predictors 1–7, §H.2.1 boundary rules, point transform, restart
reset) plus the in-harness LJ92 round-trip oracle and its red-proof.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-037-lossless-reconstruction-predictors-point-transform-restart.md`
- **Build handoff:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-087-build-lossless-reconstruction-predictors-point-transform-restart.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-035` — SOF-3 patent-clean; clean-room from T.81 Annex H; provenance row.
- `DEC-016` — caller-owned output buffer, no allocation on the reconstruction path.
- `DEC-018` — u16 sample path; prediction modular `2^16` via explicit `wrapping_*`.

### Constraints that apply

- `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `provenance-recorded-per-algorithm`, `test-before-implementation`.

### Prior related work

- `SPEC-036` / `HANDOFF-085` / **PR #98** — the shipped entropy front-end.
- `HANDOFF-087` / **PR #102** — the build under review.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review it cold. Output exactly
ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- The DNG `Compression 7` container binding + `dnglab --raw-checksum` cross-check — STAGE-010.
- The fuzz / truncation target — SPEC-038.

## Return Criteria — how to hand back

(See the standard return block; filled below.)

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it.*

### Execution notes

- **Reviewed commit:** `e5a1062824086fcf72cf937841584cace622ea7a` (build PR #102,
  branch `claude/spec-037-build-233973`; my verify worktree is cut from it).
- **Verify branch / PR:** `claude/spec-037-verify-7b258a` → **PR #103** (carries build
  commit `e5a1062` + verify bookkeeping; `src/` byte-identical to `e5a1062`).
- **Completed at:** 2026-11-05
- **Verdict:** ⚠ **PUNCH LIST** — one low-severity item (a non-tautological
  test-coverage gap on predictors 5/6/7's floor-toward-negative-infinity, the
  sign trap the spec itself flagged). **The production code is correct; all four
  gates and all 8 acceptance criteria are green.** This is a hardening of the
  spec's own correctness authority (the round-trip oracle), not a bug fix.

### What I checked (reproduced cold, not trusted)

- **Gates green on the reviewed tree (`e5a1062`):**
  - `just test` — full suite green; `ljpeg` 18/18 (the 7 SPEC-036 + 11 SPEC-037/front-end
    tests), including all seven of SPEC-037's `## Failing Tests`.
  - `just lint-ci` — pinned clippy **0.1.98** (the floating CI version), `-D warnings`, clean.
  - `just fmt-check` — clean (exit 0).
  - `just validate` — 46 artifacts valid.
  - `just decisions-audit --changed` — no active decision's scope matches the change.
- **All 8 acceptance criteria met AND actually tested** (7 of 8 bite-confirmed by
  my own mutations; see the punch item for the 8th):
  1. Predictors 1..=7 at an interior sample — `reconstruct_applies_each_predictor_1_through_7`,
     hand-computed (does NOT call `predict`). *(coverage gap on the floor sub-path — punch item.)*
  2. §H.2.1 boundaries (first sample `2^(P−Pt−1)`, first line `Ra`, first col `Rb`,
     post-restart `2^(P−Pt−1)`) — `reconstruct_honours_the_h21_boundary_rules` +
     `reconstruct_resets_prediction_at_each_restart`, hand-computed.
  3. Point transform (`Pt>0` → `out == recon << Pt`, prediction pre-shift; `Pt==0` identity)
     — `reconstruct_applies_the_point_transform` + the round-trip matrix.
  4. Restart (ri>0 across RSTm with reset; ri==0 none) — the restart test + round-trip ri∈{0,7}.
  5. Round-trip bit-identical across predictors 1..=7 × precision {8,12,16} × Pt {0,2} × restart on/off.
  6. `oracle-must-be-shown-red` — `round_trip_red_proof_…` (predictor-4 perturbation in an
     independent reference decoder) + my own mutations below.
  7. `out.len()` mismatch → `Error::LjpegPlaneWrongLength`, never panics; zero-alloc
     — `wrong_out_length_is_a_typed_error` (too-short / too-long / reconstruct-direct) +
     code inspection (only the caller's `&mut [u16]` is written; no Vec/Box on the path).
  8. Modular `2^16` via explicit `wrapping_*` + `& 0xFFFF`; `#![forbid(unsafe_code)]` +
     five-lint deny intact; provenance-ledger row present.
- **Adversarial mutation (the load-bearing independent check — oracle is NOT tautological).**
  Each mutation was made in a staged copy, the ljpeg tests run, then `git checkout` reverted it:
  - **M1 — production `extend` (EXTEND) off-by-one** (not shared with the encoder, which
    uses its own `encode_diff`): `round_trip_is_bit_identical_…` went **RED** (plus 7 others).
    → the round-trip oracle genuinely bites on the decode path.
  - **M2 — the `Pt` shift in `reconstruct` neutralised**: exactly
    `reconstruct_applies_the_point_transform` + `round_trip_is_bit_identical_…` (Pt=2 cases)
    went **RED**; the Pt=0 cases correctly stayed green.
  - **M3 — production predictor 7 (`predict`) broken** (`floor_div2(ra+rb)` → `ra`):
    `round_trip_is_bit_identical_…` **still PASSED** (the encoder shares `predict`, so the
    error cancels — the round-trip is tautological w.r.t. `predict`), but the hand-computed
    `reconstruct_applies_each_predictor_1_through_7` **caught it (RED)**. The suite is
    non-tautological *because of* the hand-computed direct tests, exactly as the builder
    reasoned in HANDOFF-087 reflection Q3.
  - **M4 (the gap) — `floor_div2`'s negative-floor correction dropped** (truncate toward
    zero for negative odd inputs): **all 18 ljpeg tests stayed GREEN.** No non-tautological
    test pins predictors 5/6/7's floor-toward-negative-infinity, because (a) the round-trip
    is tautological w.r.t. `predict`→`floor_div2`, and (b) the hand-computed predictor test
    uses only positive neighbours (Ra=120, Rb=110, Rc=100), so every `>>1` intermediate is
    non-negative where truncation == floor. This is the punch item.
- **Clean-room (`DEC-035`) held.** The `docs/provenance-ledger.md` reconstruction row is
  present and accurate; the code uses the repo's own idioms (`pow2`, `checked_*`, the
  `plane::BitReader` mirror, the shift-free `floor_div2`), reads as from-standard (T.81
  Table H.1 / §H.2.1), and attests no copy/link of `oxideav-mjpeg` / `photocraft-raw` or any DNG SDK.
- **Scope confirmed.** The diff touches only `src/ljpeg.rs`, `src/lib.rs`,
  `docs/provenance-ledger.md`, and SPEC-037 bookkeeping. `src/ifd.rs` and `src/plane.rs`'s
  unpacker are untouched. No DNG container binding, no fuzz target, no public API/CLI change.
  The build commit's only edit to the spec file was the `cycle: build → verify` advance — it
  did **not** alter acceptance criteria or failing tests.

### The punch list (exactly what must change)

1. **[low] Add one non-tautological assertion pinning `floor_div2`'s floor-toward-negative-
   infinity for predictors 5/6** (the `>>1` sign trap the spec's "Notes for the Implementer"
   flagged). No production change — `floor_div2` is correct. Today the only tests that
   exercise predictors 5/6/7 either (a) share `predict` through the encoder, so a
   truncate-toward-zero regression cancels in the round-trip, or (b) hand-compute with
   positive neighbours where floor == truncate. A mutation that drops the negative-floor
   correction ships all-green (verified — M4 above). Fix: extend
   `reconstruct_applies_each_predictor_1_through_7` (or add a small sibling) with an interior
   sample whose intermediate is **negative and odd** — e.g. predictor 5 with `Rb < Rc`
   (`Rb−Rc = −3` → `floor_div2 = −2`, truncate would give `−1`) or predictor 6 with
   `Ra < Rc` — and assert the exact reconstructed value hand-computed with floor division.
   Since the round-trip oracle is this stage's designated correctness authority until
   STAGE-010's `dnglab` real-file cross-check, closing this one hole before ship keeps the
   authority honest on the predictors' defining subtlety. (Disposition is the architect's:
   it is a ~5-line test; if preferred, it can be folded into the SPEC-038 build, which
   already consumes `encode_lj92` — but it is a code change that should be independently
   verified, so it belongs in a build pass, not at ship.)

### Cost self-report

- **Tokens (total):** 235814 — a **FLOOR**, not a `/cost` cumulative.
- **Estimated USD:** null (no rate applied to a floor).
- **Duration (minutes):** ~40.
- **Source of the number:** none available (no interactive `/cost` in the Claude Code
  desktop app's Code tab). 235,814 is this verify session's context-window occupancy at
  handback (`ccd_session_mgmt get_usage`: 235,814 / 1,000,000, 24%). A lower bound —
  per-turn context resends not captured — reported as a floor exactly as HANDOFF-085/086/087
  did; the orchestrator supersedes it from the session transcript at ship.

### Drift and new artifacts

- **New decisions emitted:** None. Reconstruction is straight T.81 Annex H.1 under the
  existing `DEC-035` / `DEC-016` / `DEC-018`; I confirmed nothing novel crept in.
- **Deviations from spec:** The new variant is `LjpegPlaneWrongLength { expected, got }`;
  the crate's other `*WrongLength` variants use `actual` rather than `got`. The builder
  followed the spec's explicit `got` and flagged the inconsistency in HANDOFF-087. Not
  blocking — a trivial rename either way; the architect may settle the convention.
- **Follow-up work identified:** None beyond the punch item and the already-planned
  STAGE-009/010 backlog (SPEC-038 fuzz, STAGE-010 container + `dnglab` cross-check).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec's `## Outputs` is a complete arithmetic contract and the build
   handoff was thorough and honest (it even pre-named the `got`/`actual` nit and the
   restart-precedence reasoning). The only friction was mechanical: the round-trip oracle
   deliberately shares the production `predict`, so confirming the suite is non-tautological
   required mutating *several* targets (EXTEND, Pt, a predictor, and `floor_div2`) rather
   than one — which is also how I found the one surviving mutation.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` is exactly the constraint the punch item serves: the
   oracle IS shown red (predictor 4), but a surviving mutation on a flagged sub-path is the
   same defect class the constraint exists to catch, so the item strengthens the existing
   discipline rather than needing a new rule.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The four-mutation sweep (one per non-shared decode path, one shared
   predictor, one helper sub-path) is the right shape for a shared-`predict` round-trip
   oracle and is what surfaced the gap; I'd reuse it verbatim for the SPEC-038 verify.
