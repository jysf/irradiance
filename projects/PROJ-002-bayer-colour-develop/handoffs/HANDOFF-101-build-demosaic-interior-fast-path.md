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
  id: HANDOFF-101
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-048

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

# HANDOFF-101: demosaic interior fast path

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-048` to the implementer for the
**build** cycle: give `demosaic_into` a row-wise interior fast path (direct
neighbour indexing over per-row slices, the 2-pixel border kept on the current
reflecting path), with the kernel arithmetic reused verbatim so the output is
**bit-identical** and the loop gets materially faster (DEC-041).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-012-colour-develop-activearea-origin-correctness-and-demosaic-performance.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-048-demosaic-interior-fast-path.md` — the Acceptance Criteria, Failing Tests and Notes are the contract.
- **Decision:** `./decisions/DEC-041-demosaic-interior-fast-path-and-the-stage-012-perf-discipline.md` — the fast-path structure, why the border stays reflecting, and the bit-identical + measured discipline.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).
- **Code:** `src/demosaic.rs` — `demosaic_into` (the two passes, the per-pixel
  `checked_div`/`checked_rem` at ~line 273/291), `green_at_site` (~322) and
  `interpolate_red_blue`, and the neighbour-read helpers `sample` / `sample_i32`
  / `reflect` / `plane_index` (~556–600). The max neighbour offset is ±2, so the
  interior is every pixel ≥ 2 from each edge.

### Decisions that apply

- `DEC-041` — the interior fast path + the perf discipline. **This is the spec.**
  Interior: direct index over per-row slices, no `reflect`/`get`; border: the
  current reflecting path; kernel arithmetic reused verbatim; output bit-identical.
- `DEC-032` — the demosaic kernel (zero-alloc two-pass green). Do NOT change the
  kernel maths or the two-pass structure; only how neighbours are fetched.
- `DEC-016` — caller-owned `src`/`dst` buffers (keep the contract; no new alloc
  beyond what the slicing needs, which is none).

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These apply here:

- `no-panics-on-untrusted-input` — the fast path must be panic-free; per-row
  slices make the interior in-bounds by construction (no `unsafe`, no per-read `get`).
- `no-new-top-level-deps-without-decision` — zero new dependencies; no SIMD crate.
- `one-spec-per-pr` — this PR is SPEC-048 only.
- (`oracle-must-be-shown-red` does not cleanly apply — this is output-invariant;
  the proof is the `dst_after == dst_before` equivalence test, not a red→green.)

### Prior related work

- `SPEC-047` / `DEC-040` (shipped, PR #139) — the same colour path; the
  `active_area_origin` phase offset `green_at_site` now uses. Leave it intact.
- The pyirradiance rawpy/LibRaw benchmark (report under `docs/reviews/` / the
  pyirradiance repo) — the measured ~520 ms demosaic that motivates this. LibRaw
  is the yardstick, reference only, never linked.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-048's Acceptance Criteria.
- All failing tests in SPEC-048 now passing.
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

- **Any change to the demosaic kernel, its arithmetic, or the output bytes** —
  the golden hashes must stay identical. If the output moves, stop: it is a
  behaviour change, not this spec.
- `unsafe` or SIMD — the fast path stays safe (slices) and zero-dependency.
- The colour-develop memory / f32 work (P2), threading (P3), and the
  colour-geometry crop follow-up — separate STAGE-012 specs.
- `develop_color_into` / `normalize_mosaic` and the mono `develop_into` — unchanged.

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
