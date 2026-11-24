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
  id: HANDOFF-189
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-24
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-091

project:
  id: PROJ-006
  stage: STAGE-031
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-189: The white-balanced linear RGB step at sensor geometry, with the working-space DEC and the arithmetic decision for the new path

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-091` to a cold `claude-opus-5`
session (acting as implementer) for the **`build`** cycle: two new public functions in
`src/color.rs` that write the white-balanced linear-sRGB plane at sensor geometry, `u16`
(today's values) and `f32` (unclamped), with no change to any existing output.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` (Design, AC1 to AC10, Failing Tests, the mutant list)
- **Decision:** `./decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-079` — the working space is linear sRGB; this spec's `f32` plane is unclamped because the probe showed the `u16` plane clamps (its table is the evidence for AC3)
- `DEC-033` — `f64` basic operations only, committed constants: the `u16` path must keep today's expression and order
- `DEC-018` — `u16` full scale, clamp before round half up (`quantize`)
- `DEC-016` — caller-owned buffers; nothing written unless every check passes
- `DEC-040` — CFA phase and level slot indexed from the ActiveArea origin (mutant f)
- `DEC-042` — the fused traversal and its single green plane: reuse it, do not copy it
- `DEC-053`, `DEC-078` — required processing and malformed tags refuse first, `DevelopPath::Colour`
- `DEC-069` — `GainMap`s at stage 2 (AC7)
- `DEC-002` — determinism; no new nondeterministic arithmetic

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — deny-level lints; the new entries join the fuzz target (AC9)
- `oracle-must-be-shown-red` — AC10: apply each pre-registered mutant by hand, list which test failed
- `test-before-implementation` — write the Failing Tests first and see them fail
- `one-spec-per-pr` — this PR is SPEC-091 only
- `provenance-recorded-per-algorithm` — no new algorithm here (same pipeline); say so in the PR

### Prior related work

- `SPEC-049` / `SPEC-050` — the fused colour develop and its crop sink, which this spec generalises
- PR 324, PR 325 — PROJ-006 brief and STAGE-031 frame (onyx's answers are in the brief)
- **Parallel work:** PROJ-005's SPEC-061 build (the DNG writer) runs in another session.
  It appends two `Error` variants and edits `CHANGELOG.md` [Unreleased],
  `fuzz/Cargo.toml`, `app.just`, `docs/provenance-ledger.md`. This spec adds **no**
  `Error` variant. On a conflict in a shared file, keep both sides' entries.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-091's Acceptance Criteria.
- All failing tests in SPEC-091 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `feat/spec-091-srgb-linear-plane`, cut from current
  `origin/main` (fetch first; check `gh pr list` for anything touching `src/color.rs`).
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

- The mixer, the folded row and the fixed-point arithmetic (SPEC-092).
- Any developed or cropped output, `DevelopOptions` changes (SPEC-093).
- Threaded variants of the new entries.
- Any change to an existing function's output, a new `Error` variant, or a
  `DEVELOP_VERSION` bump. If one seems necessary, stop and hand back blocked.

**Repo gotchas.** Run `just test`, not `just test --all-features` (the recipe already
passes it). Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
for the Tier-B tests. Commit-message bodies must be plain prose: a push whose message
contains code tokens (`#[...]`, `->`, `--flag`) is rejected by a ruleset; PR bodies and
files are fine.

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
