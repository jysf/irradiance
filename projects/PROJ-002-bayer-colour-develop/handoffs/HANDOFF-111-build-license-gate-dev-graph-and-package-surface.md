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
  id: HANDOFF-111
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-055

project:
  id: PROJ-002
  stage: STAGE-010
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

# HANDOFF-111: Licence gate over the dev-graph + package surface (SPEC-055, group D)

## Delegation Summary

The architect hands `SPEC-055` to the implementer for the **build** cycle: make the
licence gate actually evaluate the dev-dependency graph (root + `fuzz/`), proven by a
mutation red-proof, and gate the internal `irr` binary out of the default
build/package/install (review findings #4 and #6). The whole decision is **DEC-048**.
**Read SPEC-055 in full and DEC-048 first; DEC-048 records two cargo-deny knobs that
already FAILED in probing — start from `cargo deny list`, not from those.**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-055-license-gate-dev-graph-and-package-surface.md` — all of it, especially `## Design` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-048-license-gate-over-the-dev-graph-and-gate-the-irr-binary.md` — the mechanism, the two dead-end probes, and the `irr` disposition rationale.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (findings #4, #6).
- **Files:** `deny.toml`, `Cargo.toml` (the `[[bin]] irr` + `[features]`), `.github/workflows/ci.yml` (l.142 `cargo-deny-action@v2`, and the red-proof steps l.33/58/76 as the pattern), `app.just` (`deny` l.89, `deny-fuzz` l.106), `scripts/lint-red-proof.sh` + `gate-audit-red-proof.sh` (the red-proof template), `docs/license-policy.md`, `tests/demosaic_oracle.rs` (l.824 `CARGO_BIN_EXE_irr`).
- **Toolchain brief:** `./guidance/toolchain-brief.md` (DEC-004 rule 5).

> ⚠ Line numbers read 2026-11-10 against the current tree; re-confirm before editing.
> The exact cargo-deny dev-graph knob is yours to find (see DEC-048) — the red-proof
> is what proves you found it.

### Decisions that apply

- `DEC-048` — the dev-graph licence evaluation + the `irr` feature-gate this spec builds.
- `DEC-011` — `fuzz/` is a second cargo graph outside the library's; both must be denied (`just deny` + `just deny-fuzz`).
- `DEC-036` — the existing `fuzzing` feature precedent for a dev-only empty cargo feature.
- `DEC-045` — the re-plan. `DEC-009` — control run before any red-proof. `DEC-023`/`DEC-010` — why `ssimulacra2`/`toml` are dev-deps.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-copyleft-dependencies` — the gate must reject a GPL dep; the red-proof proves it.
- `oracle-must-be-shown-red` — the licence red-proof is the load-bearing deliverable.
- `library-not-application` — `irr` is a dev/oracle tool, not a shipped surface.

### Prior related work

- `SPEC-011` / `DEC-027` — the `clippy — fuzz graph` job: the same "one policy, two
  cargo graphs" move this applies to the licence gate.
- `SPEC-052` / `SPEC-053` (groups A / B) — sibling pre-0.2.0 hardening specs.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- The cargo-deny dev-graph fix + `scripts/deny-red-proof.sh` (control green, GPL-dev-dep
  mutation red) for root and `fuzz/`, wired into CI with a `just deny-red-proof` recipe.
- The `irr` feature-gate (`required-features = ["irr"]`) + a package-target-list check,
  run in CI; `just test` (`--all-features`) still builds `irr` and the oracle test passes.
- `docs/license-policy.md` reconciled; CHANGELOG `[Unreleased]` entry.
- A PR against `main` from branch `claude/spec-055-build-<suffix>`.
- PR description referencing: HANDOFF-111, SPEC-055, STAGE-010, PROJ-002, DEC-045 / DEC-048.
- Confirmation in the handback that a default `cargo build` produces no `irr` binary and
  `just deny`/`just deny-fuzz` both still pass on the real tree.

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

- Any change to the licence policy allow-list or `exceptions` (the NCSA/libfuzzer-sys
  exception stays); this closes the gap between the policy and the gate, nothing more.
- Any new runtime or dev dependency committed to the real tree (the red-proof's GPL
  crate is a throwaway in a temp dir only).
- The `fuzzing` feature, the parser guard (SPEC-052), the Compression-7 shape binding
  (SPEC-053).
- Deciding a DIFFERENT `irr` disposition than DEC-048's `required-features` gate — if
  the maintainer wants `examples/` or `exclude`, that is a new decision at PR review,
  not a build-time change.

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
