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
  id: HANDOFF-202
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-25
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-092

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

# HANDOFF-202: The mixer at sensor geometry: weights-only rational API, the fixed-point row, the fade of negative weights near clip, the plane's black and white levels

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-092` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle: three public functions in
`src/color.rs` that mix a colour Bayer DNG to a mono `u16` plane at sensor geometry (whole
plane and row band) and expose the fixed-point row, six public constants (one the mixer version), three `Error`
variants, the fade of negative weights near clip in exact integer arithmetic, a
cross-architecture determinism hash, and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (read "onyx's reply to the design (2026-11-25)")
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md` (Design §1 to §11, AC1 to AC15, Failing Tests, mutants (a) to (u))
- **Decisions:** `./decisions/DEC-083-the-mono-mixer-row-is-i32-with-24-fraction-bits-and-its-fade-starts-at-57343.md` (the numbers) and `./decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md` (the approach and the fade)
- **The shipped sibling:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/done/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` and its handoffs HANDOFF-189 / HANDOFF-198 (`## Handback`): the sink, band form, determinism fixture and the three oracle gaps verify found
- **Toolchain brief:** `./guidance/toolchain-brief.md` (DEC-004 rule 5)

### Decisions that apply

- `DEC-083` — weights rule, row format (`i32`, 24 fraction bits, round half up), `τ` (`T = 57343`), the per-pixel step and the overflow argument; this spec implements it exactly
- `DEC-079` — weights mean linear sRGB, applied folded as `v = wᵀ·M`; the fade and its monotonicity argument
- `DEC-033` — `f64` basic operations only, no `mul_add`; the row build must keep its order (AC11)
- `DEC-018` — `u16` full scale, round half up; the mixed plane's levels are 0 and 65535
- `DEC-016` — caller-owned buffers; nothing written unless every check passes
- `DEC-040` — CFA phase and level slot from the ActiveArea origin
- `DEC-042` / `DEC-044` — the fused traversal and `develop_rows_banded`; reuse, do not copy
- `DEC-051` — classify the three new variants in `Error::kind()`, no wildcard arm
- `DEC-053`, `DEC-078` — required processing and malformed tags refuse first (`DevelopPath::Colour`)
- `DEC-069`, `DEC-070` — `GainMap`s at stage 2 and the `W − B` scale: what the fade's `m` reads (Design §5)
- `DEC-002` — determinism; AC12 measures the row and the plane on both CI legs

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-panics-on-untrusted-input` — deny-level lints; the new entries join the `demosaic` fuzz target (AC9)
- `oracle-must-be-shown-red` — AC15: apply each pre-registered mutant by hand, list which test failed
- `test-before-implementation` — write the Failing Tests first and see them fail
- `one-spec-per-pr` — this PR is SPEC-092 only
- `provenance-recorded-per-algorithm` — add the ledger row drafted in Design §10

### Prior related work

- `HANDOFF-189` / `HANDOFF-198` — SPEC-091 build and verify; their findings shaped AC1 (an oracle that does not share the traversal), AC11 (a fixture per FMA and order site) and AC4 (a tolerance derived from the casts)
- PR 343, PR 351 — SPEC-091's build and ship
- The design-time probe that produced AC5's rows and DEC-083's table: source and output in the PROJ-006 session notes folder (`spec-092-design-probe.rs.txt`, `spec-092-design-probe-output.txt`), outside the repo

## Expected Deliverables

- Code changes implementing SPEC-092's Acceptance Criteria; all Failing Tests passing.
- A PR against `main` from branch `feat/spec-092-mono-mixer`, cut from current
  `origin/main` (fetch first; check `gh pr list` for anything touching `src/color.rs` or
  `src/lib.rs`).
- PR description referencing this handoff, SPEC-092, STAGE-031, PROJ-006, DEC-083,
  DEC-079 and any other `DEC-*` touched; the provenance row; the oracle red-proofs.

## Out of Scope

- Filter presets, spatial highlight reconstruction, the developed output (SPEC-093).
- A clip-mask output, a fade parameter, negative weights, a reusable band plan, threaded
  variants, an `irr` hook.
- Any change to an existing function's output, determinism hash or Tier-B pin, or to
  `DEVELOP_VERSION`.
- Editing a golden row in AC5 or a bound in AC4 to make a test pass: report it instead.

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
