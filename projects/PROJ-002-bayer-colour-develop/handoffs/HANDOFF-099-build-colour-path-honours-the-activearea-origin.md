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
  id: HANDOFF-099
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-047

project:
  id: PROJ-002
  stage: STAGE-012
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
  tokens_total: 28690878           # REAL combined count — what cost-audit reads
  estimated_usd: 69.65             # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: claude/gifted-northcutt-d29013
  pr: 137
  completed_at: 2026-11-06         # YYYY-MM-DD
  notes: "offset form, no crop (DEC-040); four Tier-A oracle tests, red-proof shown red pre-fix; DEVELOP_VERSION 2 to 3"
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-099: colour path honours the ActiveArea origin

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-047` to the implementer for the
**build** cycle: make the colour develop path index the CFA phase and the
BlackLevel/WhiteLevel slot from the ActiveArea origin (offset form, no crop —
`DEC-040`), so odd-origin DNGs develop to correct colour; bump `DEVELOP_VERSION`.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-047-colour-path-honours-the-activearea-origin.md` — the Acceptance Criteria, Failing Tests and Notes are the contract.
- **Decision:** `./decisions/DEC-040-index-the-cfa-phase-and-level-slot-from-the-activearea-origin.md` — the offset form, and why not a crop.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).
- **Code:** `src/color.rs` (`normalize_mosaic`, ~line 723), `src/demosaic.rs`
  (`demosaic_into` and the green/colour site helpers, the `(y & 1, x & 1)`
  computations ~lines 301/357), `src/develop.rs` (`resolve_geometry` — the
  `active_area` → `(0,0)`-when-absent precedent), `src/ifd.rs` (`ActiveArea`,
  `Sensor`), `tests/color_oracle.rs` (the Tier-A harness to extend).

### Decisions that apply

- `DEC-040` — index the phase/slot from the ActiveArea origin via the offset form
  `(y.wrapping_sub(active_top) & 1, x.wrapping_sub(active_left) & 1)`; no crop, no
  buffer-length change; `DEVELOP_VERSION` 2 → 3. **This is the spec.**
- `DEC-038` — the colour develop path and `DEVELOP_VERSION` (which you bump); do
  not touch the ColorMatrix/Bradford maths it governs.
- `DEC-016` — caller-owned `src`/`dst` buffers (keep the contract).
- `DEC-018` — clamp/modular integer discipline (don't introduce float or panics).

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These apply here:

- `no-panics-on-untrusted-input` — the offset must be panic-free for any origin,
  including at/beyond the plane edge (use `wrapping_sub`).
- `oracle-must-be-shown-red` — the grey-stays-grey test must be shown to FAIL on
  the plane-relative computation before the fix (the red-proof control).
- `test-before-implementation` — the failing tests are specified in the spec;
  write them first, see them red, then make them pass.
- `no-new-top-level-deps-without-decision` — zero new dependencies.
- `one-spec-per-pr` — this PR is SPEC-047 only.

### Prior related work

- `docs/reviews/2026-11-06-proj-003-close-review-triage.md` and the pyirradiance
  report (`~/PSeven/experiments/RustLibraries/pyirradiance/notes/upstream-review-2026-11-06.md`)
  — the rawpy/LibRaw benchmark that raised this (reference only; LibRaw not linked).
- `DEC-038` / the DEC-038 colour red-proof in `tests/color_oracle.rs` — the
  pattern to mirror for the grey-stays-grey red-proof.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-047's Acceptance Criteria.
- All failing tests in SPEC-047 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `feat/spec-XXX-<slug>`.
- PR description referencing: this handoff ID, the spec ID, the stage
  ID, the project ID, all referenced `DEC-*`, and any new `DEC-*`
  created during implementation.

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

- **Cropping** the colour output to the active area, removing the masked border,
  or any `DefaultCrop`/`Orientation` work — deferred follow-up (DEC-040); do NOT
  change `develop_color_into`'s buffer lengths or output dimensions.
- The colour maths (ColorMatrix/Bradford, DEC-038) and the ColorChecker ΔE work.
- Demosaic performance, the fused-buffer/f32 memory work, and threading — the
  P1/P2/P3 specs in STAGE-012's backlog.
- Subsampled (`Hi`/`Vi != 1`) or tiled layouts.

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

- **Branch / PR:** `claude/gifted-northcutt-d29013` → https://github.com/jysf/irradiance/pull/137
- **Completed at:** 2026-11-06
- **All acceptance criteria met?** yes. Grey stays grey (per-channel spread ≤ 8
  counts, well inside the `< 1e-3` tolerance) across all 4 ActiveArea origin
  parities × 4 Bayer phases; the plane-relative red-proof is shown red before
  the fix (pre-fix the origin-declaring render is byte-identical to the
  plane-relative one, so the `assert_ne!` fails) and green after; an even origin
  is byte-identical to the plane-relative render; no panic for an out-of-range
  origin; `DEVELOP_VERSION == "3"`; `develop_color_into`'s signature and buffer
  lengths unchanged; no new dependency; determinism held. `just test`,
  `just lint-ci`, `just fmt-check`, `just validate` and `just cost-audit` all
  green.
- **For `verify`:** n/a (build cycle).

### Cost self-report

- **Tokens (total):** 28,690,878
- **Estimated USD:** 69.65 (Opus standard rates: input 15, output 75, cache-read
  1.50, cache-write 18.75, all per Mtok)
- **Duration (minutes):** ~40
- **Source of the number:** harness report — summed `input + output +
  cache_read + cache_creation` across every API call in this build session's
  transcript (anchored on the scratchpad-dir session UUID, not a text grep).
  This is the near-final total at handback; the few remaining close-out turns
  add a little more. Dominated by cache reads (27.8M), consistent with the
  STAGE-009 build-cycle costs (036 = 28.5M, 038 = 25.0M).

### Drift and new artifacts

- **New decisions emitted:**
  - none — DEC-040 was written at design and is the whole decision.
- **Deviations from spec:**
  - none. One mechanical adjustment the spec did not spell out: adding the
    origin to `green_at_site`/`interpolate_red_blue` pushed them to 9 arguments,
    which trips clippy's `too_many_arguments` (a `-D warnings` CI failure), so
    the pixel coordinate and the origin are each grouped into a `(u32, u32)`
    tuple to stay within the limit on the library path (no test-only allow).
- **Follow-up work identified:**
  - none new. The deferred crop / `DefaultCrop` + `Orientation` parity for the
    colour path is already its own STAGE-012 backlog spec (DEC-040 Non-Goals);
    the P1/P2/P3 demosaic performance + memory + threading specs are already in
    the backlog.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The spec and DEC-040 were precise (the offset form, the
   four tests, the no-crop/no-length-change constraint). The one thing worth
   recording for the template: the design files (DEC-040, HANDOFF-099,
   SPEC-047-build.md) were not yet on the worktree branch at dispatch and had to
   be merged from `main` — the dispatch anticipated this and said so, which saved
   the loop.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The clippy `too_many_arguments` interaction is a toolchain-brief-level
   detail, not a missing decision; grouping into tuples kept the fix on the
   library path without a style allow, so no new rule is needed.

3. **If you did this task again, what would you do differently?**
   — Group the coordinate/origin into tuples from the first edit rather than
   passing four scalars and then reflowing when `too_many_arguments` fired — one
   fewer fmt/lint round-trip. Otherwise the test-first flow (write the four
   tests, see the red-proof go red, then make them pass) was the right order and
   caught the behaviour cleanly.
