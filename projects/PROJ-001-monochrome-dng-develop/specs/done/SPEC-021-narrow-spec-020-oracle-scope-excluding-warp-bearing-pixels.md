---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-021
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-005
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # build delegated via HANDOFF-058
  created_at: 2026-10-27

references:
  decisions: [DEC-024, DEC-005, DEC-022]
  constraints: [oracle-must-be-shown-red, test-before-implementation]
  related_specs: [SPEC-020, SPEC-018, SPEC-017, SPEC-015]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-005's <capability>". Optional; null is acceptable.
value_link: "STAGE-005: SPEC-020's develop oracle stops asserting a tolerance it cannot meet — its declared scope (warp-free, Orientation-1 pixels) now matches the signal dnglab actually provides; warp-bearing pixels are verified analytically instead."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 20000000        # build + verify; S, docs + test-reframe, no new oracle
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-10-27
      notes: "Design in the orchestrator main loop (un-metered; null-with-note per AGENTS.md §4). A read-only Explore subagent gathered the oracle-scope inputs (152,372 tokens, 29 tool-uses): the two #[ignore]'d SPEC-018 tests in tests/warp.rs, DEC-005's >=85 scope clause, DEC-024's deferred consequence, docs/oracle-contract.md's gaps, and the SPEC-015 AC5(a) reframe precedent. That cost is the subagent's, not transcribed here."
    - cycle: build
      agent: claude-opus-4-8
      interface: other
      tokens_total: 360000
      estimated_usd: null
      duration_minutes: 60
      recorded_at: 2026-10-27
      notes: "tokens_total is the Claude Code session budget-delta (15,000,000 start minus tokens-left at handback), NOT a transcript input+output sum — not comparable with other specs' transcript-summed figures (SPEC-017/FU-9, token-counts-not-comparable). estimated_usd null: budget-delta has no clean input/output split to price. pr filled in a follow-up commit once opened. All 13 gates green; src/ diff comment-only (0 logic lines); suite ignored 3 -> 1."
    - cycle: verify
      agent: claude-opus-4-8
      interface: other
      tokens_total: 3715683
      estimated_usd: null
      duration_minutes: 15
      recorded_at: 2026-10-27
      notes: "APPROVED at b37a70e. Exercised both reframed tests against the REAL corpus (non-vacuous): warp_pixels_are_below_the_srgb_oracle_baseline measured L1021223 -60.169 / L1026192 -40.281 < 83.145 and skips the Orientation-6 frame loudly; rotated_frame_is_out_of_the_srgb_oracle_scope asserts real dims 8368x5584 != 5584x8368. src diff comment-only (0 logic), #[ignore] 3->1, CI 11/11 green. Flagged the build handback tokens_total 360000 as a budget-delta (non-comparable). This figure is a FLOOR deduped by message.id (28 ids); no per-token USD meter."
  totals:
    tokens_total: 4075683
    estimated_usd: 0.00
    session_count: 3
shipped_at: 2026-10-27
---

# SPEC-021: narrow spec 020 oracle scope excluding warp-bearing pixels

## Context

> **Framed at SPEC-018 ship (2026-09-07) as the spec disposition for
> `SPEC-018/FU-10`.** Not designed here — outline captures SCOPE and
> DEPENDENCIES only.

`SPEC-018`'s build and verify established that **`dnglab`/`rawler` do
not implement DNG `OpcodeList` processing at all** (`DEC-024` Finding 2,
confirmed by both source-inspection and independent behavioural NCC-tile
analysis in HANDOFF-047 round 1). Consequence: `SPEC-020`'s develop-layer
oracle (SSIMULACRA2 against `dnglab analyze --srgb`) is **structurally
broken for warp-bearing pixels** — a correct warp scores `-60.169`
against dnglab's uncorrected reference; doing nothing at all scores
`+83.145`. SPEC-018 shipped with AC8/AC9 `#[ignore]`d as the honest
disposition, and `tests/warp.rs`'s AC4/AC10 (analytic geometry check)
as the real correctness gate.

This is the same shape of gap `SPEC-015` closed for levels
(no comparison oracle → analytic oracle). SPEC-018's ship carried
FU-10's disposition: file this spec to formally narrow SPEC-020/DEC-005's
scope (and, per FU-6, record that L1026016 is separately unsatisfiable
against dnglab because dnglab ignores `Orientation`).

`SPEC-017`'s ship (2026-09-08) routes a second, independently-measured
instance here: `SPEC-017/FU-2` reproduced the same behaviour — `dnglab
analyze --srgb` prints `8368 5584` for both an Orientation-6 and an
Orientation-1 frame, i.e. it ignores `Orientation` entirely — surfaced
when SPEC-017 first ran the SPEC-020 oracle comparison un-ignored across
all three Q2M frames. Same root, same fix; no separate spec.

## Goal

Formally narrow SPEC-020's develop-layer oracle scope to exclude warp-
bearing pixels (or the analogous pipeline stages a future opcode adds),
substituting the analytic-check pattern SPEC-015 established. Amend
DEC-005's ≥ 85 tolerance clause to name what it does and does not cover;
retire the current #[ignore] on AC8/AC9 in favour of an explicit
"out of oracle scope" documentation.

## Inputs

(All `file:line` cites are design-time; read the named item, not the exact line.)

- **`tests/warp.rs`** — the two `#[ignore]`'d tests to retire:
  `warp_scores_at_least_eightyfive_via_spec_020_oracle` (SPEC-018 AC8, ~`:628`,
  asserts `score >= 85.0`) and `warp_oracle_is_red_on_a_zeroed_kr1_coefficient`
  (SPEC-018 AC9, ~`:675`, asserts the mutant `< 85.0` **vacuously**). Read their
  full `#[ignore]` reason strings (they carry the measured numbers). Also the
  **real** gate this spec points at: `warp_moves_corner_impulse_to_spike_001s_source_coord`
  (AC4, ~`:156`) and `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more`
  (AC10, ~`:239`, the CI-visible red-proof); `develop_and_score` (~`:550`) whose
  dimension `assert_eq!` (~`:583`) **panics on L1026016 before scoring**;
  `DECODABLE_Q2M` (~`:619`).
- **`tests/perceptual_oracle.rs`** — SPEC-020's actual SSIMULACRA2-vs-`dnglab
  analyze --srgb` oracle (`THRESHOLD: f64 = 85.0`, ~`:89`). **Not**
  `tests/develop_oracle.rs` (that is SPEC-015's analytic oracle, no `#[ignore]`s).
- **`decisions/DEC-005-develop-oracle-mechanics.md`** — the ≥85 clause (~`:47–50`:
  scoped to "geometry and gross tonal correctness only", levels → DEC-004, but
  **silent on warp and Orientation**) and the `## Validation` premise (~`:132–140`,
  "below 85 = the render is wrong, the bar does not move"). `affected_scope`
  already names `docs/oracle-contract.md`.
- **`decisions/DEC-024-warprectilinear-kernel-pipeline-order-and-the-broken-oracle.md`**
  — Finding 2 (dnglab/rawler implement **no** DNG opcodes; measured warp
  `-60.169` vs no-warp baseline `+83.145`), and its Consequences (~`:185–191`)
  that **explicitly defer** the DEC-005/SPEC-020 scope amendment "as a follow-up
  for ship to disposition" — i.e. to this spec. DEC-024 does **not** record the
  Orientation leg.
- **`docs/oracle-contract.md`** — the layer-3 row (~`:22`) and "What the layers
  do NOT cover" (~`:187–197`, which carves out levels/crop/orientation → DEC-004
  only); `warp`/`opcode` appear nowhere.
- **`specs/done/SPEC-015-analytic-levels-and-geometry-oracle.md`** — the template.
  **AC5(a)** reframes a blind-spot into a *passing characterization assertion*
  (a levels fault scores 95.62 and passes — documenting the blindness as a
  fact); AC6 forces the red-proof CI-visible; **AC7: 0 lines changed in `src/`**.
- **`decisions/DEC-022-stage-orchestration-cost-is-a-gate-not-a-warning.md`** —
  the DEC-amendment shape ("… amending DEC-013 §5"): quote the amended clause
  verbatim + a "Why amend rather than revert" note.
- **SPEC-018/FU-6 + FU-10 and SPEC-017/FU-2** (in `specs/done/`) — the follow-ups
  routed here; **FU-6 is the Orientation leg DEC-024 does not record**
  (`dnglab --srgb` emits `8368×5584` for the Orientation-6 frame L1026016, whose
  correct render is `5584×8368`).

## Outputs

A **documentation + test-reframe** spec. No new oracle, no `src/` logic change.

- **`decisions/DEC-026-…-amending-dec-005-oracle-scope.md`** (new) — amends
  DEC-005's scope clause, in the DEC-022 shape (quote DEC-005's ≥85 clause
  verbatim, then a "Why amend rather than revert" note). It records: the ≥85
  develop-layer oracle covers geometry + gross tone of **warp-free,
  Orientation-1** pixels only; **OpcodeList3 `WarpRectilinear` pixels and
  Orientation ≠ 1 frames are OUT of scope** — `dnglab --srgb` applies neither
  (warp: DEC-024 Finding 2; Orientation: the re-measurement in SPEC-018/FU-6 +
  SPEC-017/FU-2, which DEC-024 does not record) — and are verified analytically
  instead (`tests/warp.rs` AC4/AC10 for warp geometry; DEC-004/SPEC-015 for
  levels+geometry). It neutralizes DEC-005's `## Validation` "below 85 = render
  wrong, bar does not move" premise **for warp-bearing and rotated frames only**.
- **`decisions/DEC-005-develop-oracle-mechanics.md`** — a short pointer added so
  a reader lands on DEC-026 (e.g. a `## Scope amended by DEC-026` note); the
  pre-registered table and `## Validation` prose above it stay intact (history
  preserved).
- **`decisions/INDEX.md`** — regenerated (`just decisions-index`).
- **`docs/oracle-contract.md`** — extend "What the layers do NOT cover"
  (~`:187–197`) with a warp + Orientation paragraph mirroring the levels one,
  and annotate the layer-3 row (~`:22`) so the ≥85 verdict is explicitly scoped
  to warp-free / Orientation-1 pixels. Cite DEC-024 Finding 2 and DEC-026.
- **`tests/warp.rs`** — retire the two `#[ignore]`s (**reframe, don't delete the
  evidence**):
  - AC8's `score >= 85` becomes a **passing characterization test** on the
    Orientation-1 frames only, asserting the oracle's blindness is *real*: the
    warp-bearing score diverges **below** the no-warp baseline (`-60.169 <
    +83.145`) — present-and-correct warp matches dnglab's uncorrected reference
    *worse*, not better (SPEC-015 AC5(a) shape).
  - The Orientation ≠ 1 case (L1026016) becomes an **explicit, documented skip**
    keyed on `Orientation != 1`, so the dimension mismatch is a named
    out-of-scope disposition — not the latent `assert_eq!` panic at ~`:583`.
  - Rustdoc on both points at AC4/AC10 as the real gate and at DEC-026 /
    `oracle-contract.md` for scope. Tier-B (needs `dnglab` + corpus): skip
    loudly when absent, like the other tier-B tests — never `#[ignore]`d.
- **Stale cross-references fixed (comment-only):** `tests/develop_oracle.rs`
  (~`:85–86`, which points the warp oracle at the wrong file —
  `tests/perceptual_oracle.rs` — and the wrong ACs), and the related pointers in
  `src/warp.rs` (~`:72`) and `src/lib.rs`. **These are doc-comment edits; 0 `src/`
  logic lines change** (assert it, SPEC-015 AC7).
- **No new provenance-ledger row** — no algorithm added.

## Acceptance Criteria

- [ ] **AC1 — DEC-026 narrows DEC-005's scope.** A new `DEC-026` exists,
      amending DEC-005 (DEC-022 shape: the ≥85 clause quoted verbatim + a "Why
      amend rather than revert" note). It names **warp-bearing (OpcodeList3) and
      Orientation ≠ 1** as out of the ≥85 oracle's scope, with the measured
      evidence (warp `-60.169` vs no-warp `+83.145`; L1026016 `5584×8368` correct
      vs dnglab `8368×5584`), cites DEC-024 Finding 2 (warp) and SPEC-018/FU-6 +
      SPEC-017/FU-2 (Orientation), and neutralizes DEC-005's `## Validation`
      premise for those cases only. DEC-005 carries a pointer to DEC-026.
      `decisions/INDEX.md` regenerated; `just decisions-audit` clean.
      **Checked by:** `just decisions-audit` + a design-time grep the verifier
      reproduces (DEC-026 present, DEC-005 points to it).

- [ ] **AC2 — `docs/oracle-contract.md` states the carve-out.** "What the layers
      do NOT cover" gains a warp + Orientation paragraph (mirroring the levels
      one), and the layer-3 row is annotated as scoped to warp-free /
      Orientation-1 pixels. `warp`, `opcode`, and the "dnglab ignores
      Orientation" fact now appear where they did not. Cites DEC-026.

- [ ] **AC3 — the two `#[ignore]`s are retired and reframed (not deleted).**
      `tests/warp.rs` has **zero `#[ignore]`** (repo-wide drops from 3 to 1 — the
      unrelated SPEC-017 one in `tests/develop.rs` stays). AC8 → a passing
      characterization test on Orientation-1 frames asserting the warp-bearing
      score is **below** the no-warp baseline (the `-60.169 < +83.145`
      inversion). The Orientation ≠ 1 frame (L1026016) → an **explicit documented
      skip** with a loud reason, not the latent `assert_eq!` panic at ~`:583`.
      Both carry rustdoc pointing at AC4/AC10 and DEC-026. Tier-B: they skip
      loudly (not error, not ignore) when `dnglab`/corpus is absent.
      **Tests:** `warp_pixels_are_below_the_srgb_oracle_baseline` (characterization)
      and the Orientation skip (name stated in the handback).

- [ ] **AC4 — the analytic gate is untouched and remains the real proof.** AC4
      (`warp_moves_corner_impulse…`, corners `503.7 / 437.7 / 407.0 px`) and AC10
      (`warp_tier_a_red_proof…`, `339.5 px ≥ 20`) still pass, unchanged. The
      `oracle-must-be-shown-red` constraint is satisfied by the already-shipped,
      CI-visible **AC10** — cite it, do not duplicate a red-proof. **No `src/`
      logic change:** `git diff` shows 0 non-comment lines under `src/` (the only
      `src/` edits are the AC5 cross-ref doc-comments).

- [ ] **AC5 — stale cross-references fixed (comment-only).**
      `tests/develop_oracle.rs` (~`:85–86`), `src/warp.rs` (~`:72`) and
      `src/lib.rs` no longer point the warp oracle at the wrong file
      (`perceptual_oracle.rs` is SPEC-020's; the warp ACs live in `tests/warp.rs`).

- [ ] **AC6 — gates green, CI observed on the shipping SHA.** `just lint-ci`
      (pinned clippy), `test`, `decisions-audit`, the standing red-proofs. The
      un-ignored tests skip cleanly in CI (corpus absent there); report the
      suite count delta (ignored 3 → 1). Name the gate list you ran.

## Failing Tests

Note the inversion (as in SPEC-022): the reframed tests PASS on the correct code
— the warp render is right; dnglab is the one that is blind. What makes them
load-bearing is that they *document the measured blindness* as a green
assertion, while the real warp red-proof (AC10) already ships. "Failing" here is
the reframe: the old `#[ignore]`'d `score >= 85` could never pass; the new
characterization asserts the opposite (and true) fact.

- **`tests/warp.rs`**
  - `warp_pixels_are_below_the_srgb_oracle_baseline` — on Orientation-1 frames,
    asserts the warp-bearing SSIMULACRA2 score is below the no-warp baseline
    (`-60.169 < +83.145`), proving dnglab provides no usable signal for
    OpcodeList3 pixels. Tier-B; skips loudly without `dnglab`/corpus.
  - the Orientation ≠ 1 disposition (L1026016) — an explicit skip with a loud
    reason (dnglab ignores Orientation → dimension mismatch), replacing the
    latent `assert_eq!` panic. Name it in the handback.

## Non-Goals

- **Any `src/` logic change** — the warp render is correct (AC4/AC10 prove it);
  this spec narrows only what the oracle *claims*. The sole `src/` edits are the
  cross-ref doc-comments (AC5).
- **Building a new oracle** — the analytic replacement (`tests/warp.rs`
  AC4/AC10) already shipped with SPEC-018. Point at it; do not duplicate.
- **Finding a second, opcode-capable reference decoder** — DEC-024's Validation
  already says "revisit if one becomes available". Out of scope here.
- **Changing the ≥85 threshold itself** — the number stands for the pixels it
  covers; this spec only states which pixels those are.
- **Re-litigating DEC-024's kernel / pipeline-order decision.**
- The other STAGE-005 items (SPEC-011 fuzz lint; the gate-script audit).

## Notes for the Implementer

- Docs + test-reframe; **0 `src/` logic lines** (SPEC-015 AC7: stop and report
  if the oracle finds a real defect). Comment-only `src/` cross-ref fixes are
  the exception.
- **Reframe, don't delete** the `#[ignore]`s (SPEC-015 AC5(a) — assert the
  blindness as a passing fact; keep the measured numbers in the test/rustdoc).
- The Orientation ≠ 1 case (L1026016) must become an **explicit skip with a loud
  reason**, never the latent `assert_eq!` panic at `tests/warp.rs:~583`.
- Amend DEC-005 via the **new DEC-026** (DEC-022 shape); `just decisions-index`
  to regenerate INDEX; `just decisions-audit` must stay clean.
- Cite **DEC-024 Finding 2** for the warp evidence — don't re-inspect dnglab
  source. The **Orientation leg is not in DEC-024** — record it from
  SPEC-018/FU-6 + SPEC-017/FU-2.
- `oracle-must-be-shown-red` is discharged by the already-shipped **AC10**
  (CI-visible, corpus-free). Cite it; do not add a second red-proof.
- Key numbers: warp `-60.169` vs no-warp `+83.145`; AC9 honest `-60.193` /
  kr1=0 `-55.075`; AC4 `503.7 / 437.7 / 407.0 px`; AC10 `339.5 px` vs 20;
  L1026016 correct `5584×8368`, dnglab `8368×5584`.
- zsh: `$pipestatus`, not `PIPESTATUS`. `just lint-ci` (pinned clippy), read CI.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Gather the actual `file:line` before authoring a spec that references
   another spec's tests from a long-ago frame. The frame pointed at "SPEC-020's
   oracle (`develop_oracle.rs`)" and "AC8/AC9"; a design-time subagent sweep
   corrected it (the SSIMULACRA2 oracle is `tests/perceptual_oracle.rs`; the
   `#[ignore]`'d AC8/AC9 are SPEC-018's in `tests/warp.rs`; the analytic
   replacement already shipped) — which is what kept the build from narrowing
   the wrong file.

2. **Does any template, constraint, or decision need updating?**
   — DEC-005's scope clause is now amended by the new DEC-026 (this spec's own
   output). Otherwise no: the build's budget-delta `tokens_total` (360,000, not
   a transcript sum) is the already-recorded `token-counts-not-comparable` class
   (SPEC-017/FU-9) — flagged in the handbacks and the cost note, no new signal.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. The two extra comment-only cross-ref fixes the build made are in scope.
   DEC-024's "revisit if a second opcode-capable reference decoder appears"
   stays the standing re-open condition, not a follow-up to write now.

4. **Where was the worst defect caught?**
   — none

5. **What can a user do now that they couldn't before?**
   — Before: SPEC-020's develop oracle advertised a ≥85 SSIMULACRA2 tolerance
   over all pixels, but for warp-bearing and rotated frames the `dnglab`
   reference applies no opcodes and ignores Orientation, so the comparison was
   structurally meaningless (a correct warp scores −60.169) — hidden behind two
   dead `#[ignore]`'d tests. After: the oracle's declared scope formally excludes
   those pixels (DEC-026, `oracle-contract.md`), and the two `#[ignore]`s are
   replaced by a passing characterization that asserts the blindness is real
   (−60.169 / −40.281 < +83.145, measured on the corpus) plus an explicit
   Orientation skip — so the oracle's silence on warp can no longer be mistaken
   for a passing verdict. No runtime change (docs + test-reframe).
