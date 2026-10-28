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
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-27
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
