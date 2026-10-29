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
  id: HANDOFF-058
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # actual implementer model (tier_map default is claude-opus-5)
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-27
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-021

project:
  id: PROJ-001
  stage: STAGE-005
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
  tokens_total: 360000             # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: 60
  branch: claude/spec-021-build
  pr: https://github.com/jysf/irradiance/pull/26
  completed_at: 2026-10-27         # YYYY-MM-DD
  notes: "tokens_total is the Claude Code session budget-delta (15,000,000 start minus tokens-left at handback), NOT a transcript input+output sum — not comparable with other specs' transcript-summed figures (SPEC-017/FU-9, token-counts-not-comparable). estimated_usd null: budget-delta has no clean input/output split to price. pr filled in a follow-up commit once opened. All 13 gates green; src/ diff comment-only (0 logic lines); suite ignored 3 -> 1."
  synced_at: 2026-10-27
---

# HANDOFF-058: narrow spec 020 oracle scope excluding warp-bearing pixels

## Delegation Summary

The orchestrator (architect) hands `SPEC-021` to the implementer for the
**build** cycle: formally narrow SPEC-020's develop-layer SSIMULACRA2 oracle to
exclude warp-bearing + Orientation ≠ 1 pixels — a new **DEC-026** amending
DEC-005, a `docs/oracle-contract.md` carve-out, and reframing the two
`#[ignore]`'d SPEC-018 tests in `tests/warp.rs` into a passing characterization
test + an explicit documented skip. **Docs + test-reframe; 0 `src/` logic
lines** (comment-only cross-ref fixes aside). The design is fully specified —
read the spec.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-021-narrow-spec-020-oracle-scope-excluding-warp-bearing-pixels.md`
  — read it whole: the **Inputs** list is the recipe (exact files + line areas),
  **AC1–AC6** are the contract, **Notes** has the key numbers and traps.
- **`tests/warp.rs`** — the two `#[ignore]`'d tests to retire
  (`warp_scores_at_least_eightyfive_via_spec_020_oracle` ~`:628`,
  `warp_oracle_is_red_on_a_zeroed_kr1_coefficient` ~`:675`), the real gate AC4
  (~`:156`) + AC10 (~`:239`), and `develop_and_score`'s dimension `assert_eq!`
  (~`:583`) that panics on L1026016.
- **`decisions/DEC-005-develop-oracle-mechanics.md`** (≥85 clause ~`:47–50`;
  `## Validation` ~`:132–140`), **`decisions/DEC-024-…-broken-oracle.md`**
  (Finding 2 + the deferred-to-ship consequence ~`:185–191`),
  **`docs/oracle-contract.md`** (layer-3 row ~`:22`; "What the layers do NOT
  cover" ~`:187–197`).
- **Templates:** `specs/done/SPEC-015-…` (AC5(a) reframe; AC7 "0 `src/` lines"),
  `decisions/DEC-022-…` (amend-via-new-DEC shape).
- **Toolchain brief:** `./guidance/toolchain-brief.md` — pinned vs local clippy
  (`lint-ci`), the `+toolchain` PATH trap, zsh `$pipestatus`, `just` recipes.

### Decisions that apply

- `DEC-005` — the develop-oracle scope clause this spec narrows (via DEC-026).
- `DEC-024` — Finding 2 (dnglab/rawler implement no DNG opcodes) is the warp
  evidence; **cite it, don't re-inspect dnglab source.** It explicitly defers
  this scope narrowing to ship.
- `DEC-022` — the shape for amending a DEC with a new one (quote verbatim + "why
  amend not revert"). Model DEC-026 on it.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching `decisions/`, `docs/`,
`tests/`, and comment-only `src/`:

- `oracle-must-be-shown-red` — discharged by the **already-shipped, CI-visible
  AC10** (zeroing kr1 moves the peak 339.5 px ≥ 20). **Cite it; do NOT add a
  second red-proof** — the reframed tests are characterizations, not red-proofs.
- `test-before-implementation` — the reframed tests are the deliverable's spine.

### Prior related work

- **SPEC-018** (shipped, PR #16) — shipped the `#[ignore]`'d AC8/AC9 and the real
  analytic gate AC4/AC10; its **FU-6** (Orientation leg) + **FU-10** (scope
  narrowing) route here. DEC-024 came from it.
- **SPEC-017** (shipped, PR #17) — **FU-2** (dnglab ignores Orientation,
  independently re-measured) routes here.
- **SPEC-015** (shipped) — the analytic-oracle + AC5(a) blind-spot-as-passing-fact
  template.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- **`decisions/DEC-026-…-amending-dec-005-oracle-scope.md`** (new, DEC-022
  shape) + a pointer in DEC-005 + `decisions/INDEX.md` regenerated
  (`just decisions-index`); `just decisions-audit` clean (AC1).
- **`docs/oracle-contract.md`** — the warp + Orientation carve-out and the
  annotated layer-3 row (AC2).
- **`tests/warp.rs`** — the two `#[ignore]`s retired and reframed: a passing
  characterization test (`warp_pixels_are_below_the_srgb_oracle_baseline`, the
  `-60.169 < +83.145` inversion, Orientation-1 frames) + an explicit documented
  skip for the Orientation-6 frame (AC3). Evidence/numbers kept in rustdoc.
- **Stale cross-refs fixed, comment-only** in `tests/develop_oracle.rs`,
  `src/warp.rs`, `src/lib.rs` (AC5) — assert `git diff` shows **0 non-comment
  `src/` lines** (AC4).
- All gates green, `just lint-ci` (pinned clippy), CI observed green on the PR
  head; report the suite ignored-count delta (3 → 1) (AC6).
- The handback filled with a REAL `tokens_total`. A PR against `main` whose
  description references HANDOFF-058, SPEC-021, STAGE-005, PROJ-001, DEC-005,
  DEC-024, DEC-022, and the new DEC-026.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- **Any `src/` logic change** — the warp render is correct (AC4/AC10 prove it).
  Only comment-only cross-ref fixes in `src/`.
- **Building a new oracle** — AC4/AC10 already ship it. Point at them.
- **A second, opcode-capable reference decoder** — DEC-024's Validation owns
  "revisit if one appears".
- **Changing the ≥85 threshold** — only its declared scope changes.
- **Re-litigating DEC-024's kernel/pipeline-order decision.**
- The other STAGE-005 items (SPEC-011 fuzz lint; the gate-script audit).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-021-build` → `main`;
  https://github.com/jysf/irradiance/pull/26 (PR #26).
- **Completed at:** 2026-10-27
- **All acceptance criteria met?** Yes.
  - **AC1** — `DEC-026` written in the `DEC-022` shape (quotes `DEC-005`'s ≥85
    clause AND its `## Validation` premise verbatim + a "Why amend rather than
    revert" note); names warp (`OpcodeList3`) and `Orientation ≠ 1` out of
    scope, with the measured evidence (−60.169 vs +83.145; honest −60.193 /
    kr1=0 −55.075; L1026016 correct 5584×8368 vs dnglab 8368×5584); cites
    `DEC-024` Finding 2 (warp) and `SPEC-018/FU-6` + `SPEC-017/FU-2`
    (Orientation). `DEC-005` carries a `## Scope amended by DEC-026` pointer.
    `decisions/INDEX.md` regenerated (26 decisions); `just decisions-audit`
    clean (0 structural errors; the one added scope warning is the expected
    `DEC-005`/`DEC-026` overlap on `docs/oracle-contract.md`, advisory/exit-0,
    same shape as the existing `DEC-018`/`019`/`024` overlaps).
  - **AC2** — `docs/oracle-contract.md` gains a warp + Orientation paragraph
    mirroring the levels one, and the layer-3 row is annotated "warp-free +
    Orientation-1 pixels only"; `warp`, `opcode`, and the "dnglab ignores
    Orientation" fact now appear where they did not; cites `DEC-024` Finding 2
    and `DEC-026`.
  - **AC3** — the two `#[ignore]`s retired and reframed (not deleted):
    `warp_pixels_are_below_the_srgb_oracle_baseline` (characterization, the
    −60.169 < +83.145 inversion, Orientation-1 frames) and
    `rotated_frame_is_out_of_the_srgb_oracle_scope` (explicit documented skip
    for L1026016, keyed on Orientation; replaces the latent `assert_eq!` panic,
    which is now a loud skip in `develop_and_score`). Measured numbers kept in
    rustdoc. `tests/warp.rs` has 0 `#[ignore]`; repo-wide 3 → 1 (the SPEC-017
    `q2m_frames_have_at_least_one_replaced_pixel` in `tests/develop.rs` stays).
    Tier-B: both skip loudly when corpus/`dnglab` absent — never `#[ignore]`d.
  - **AC4** — AC4/AC10 untouched and still pass (warp 17/0/0 with corpus).
    `oracle-must-be-shown-red` discharged by the already-shipped CI-visible
    AC10 — cited, not duplicated. **`git diff main -- src/` is comment-only: 0
    non-comment lines** (only `src/warp.rs`, 6 doc-comment lines).
  - **AC5** — stale cross-refs fixed, comment-only: `tests/develop_oracle.rs`
    (two spots) and `src/warp.rs` no longer point the warp oracle at
    `tests/perceptual_oracle.rs`/`AC8`/`AC9` — they now name the analytic gate
    (`AC4`/`AC10`, `tests/warp.rs`) and `DEC-026`'s scope. `src/lib.rs` carries
    **no** such stale cross-ref (see Deviations). Also fixed (comment-only, not
    enumerated in AC5): `tests/develop.rs` and `tests/warp.rs`'s own header,
    which named the retired AC8 test (see Deviations).
  - **AC6** — all gates green; CI observed on the PR head (recorded in the
    follow-up commit / PR). Suite ignored-count delta: **3 → 1**.

### Cost self-report

- **Tokens (total):** ~360,000
- **Estimated USD:** null
- **Duration (minutes):** ~60
- **Source of the number:** Claude Code session **budget-delta** (15,000,000
  start minus tokens-left at handback). ⚠ This is NOT a transcript input+output
  sum and is **not comparable** with other specs' transcript-summed figures —
  the exact caveat `SPEC-017/FU-9` raised (`token-counts-not-comparable`).
  `estimated_usd` left null because a budget-delta has no clean input/output
  split to price. (The orchestrator transcribes this verbatim via
  `just handback-sync` — I did not run it.)

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-026` — the ≥85 develop oracle covers warp-free, Orientation-1 pixels
    only — amending `DEC-005`'s scope clause.
- **Deviations from spec:**
  - **`src/lib.rs` had no stale warp-oracle cross-ref to fix** (AC5/Inputs
    named it). Its module doc only describes `SPEC-018`'s pipeline order
    (`DEC-024` Finding 1), which is correct and unchanged; it never pointed the
    warp oracle at `perceptual_oracle.rs` or `AC8`/`AC9`. The design-time cite
    was approximate ("read the named item, not the exact line"). Grep-verified:
    no `perceptual_oracle`, `AC8`, `AC9`, `dnglab`, or `>= 85` warp-oracle
    reference in `src/lib.rs`.
  - **Two additional stale cross-refs fixed that AC5 did not enumerate**
    (comment-only, same class): `tests/develop.rs`'s `score_against` named the
    retired `warp_scores_at_least_eightyfive_via_spec_020_oracle` as the "same
    shape ... were it ever run un-ignored"; and `tests/warp.rs`'s own module
    header line listed `AC8`-`AC10` as if those tests still existed. Both now
    point at the reframe / `DEC-026`.
- **Follow-up work identified:** none beyond `DEC-024`'s standing Validation
  ("revisit if a second DNG-opcode-capable reference decoder becomes
  available"), which `DEC-026` carries forward unchanged.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only `src/lib.rs` in AC5/Inputs: it is named as carrying a stale
   warp-oracle cross-ref, but it carries none. Cost one grep to confirm. The
   two genuine stale refs (`tests/develop_oracle.rs`, `src/warp.rs`) were exact;
   a third (`tests/develop.rs`) was unlisted but obvious once the test was
   renamed.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraint. The only addition to the file list was
   `tests/develop.rs:~771`, which referenced the retired `AC8` test name — a
   stale cross-ref of exactly the AC5 class, just not enumerated. Fixed
   comment-only.

3. **If you did this task again, what would you do differently?**
   — Nothing material. Re-measuring the live `AC8`/`AC9` scores first
   (−60.169 / −60.193 / −55.075, reproduced exactly, plus L1026192 −40.281 and
   the L1026016 8368×5584-vs-5584×8368 mismatch) up front made the
   characterization's baseline-constant choice and the Orientation skip obvious
   and removed all guesswork from the reframe. Worth doing early every time a
   spec's load-bearing numbers can be reproduced locally.
