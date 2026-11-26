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
  id: HANDOFF-203
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-26
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

# HANDOFF-203: VERIFY: the mixer at sensor geometry: weights-only rational API, the fixed-point row, the fade of negative weights near clip, the plane's black and white levels (SPEC-092)

## Delegation Summary

You are the **verifier** for SPEC-092. Review it cold: you did not build it.

- **Under review:** PR 354 (`feat/spec-092-mono-mixer`). The code is commit `b822fa2`; CI
  run 45855649150 on it was green, including both `determinism` legs. The later build
  commit `0efa11a` touches the handoff, spec and timeline only. The build ran under
  HANDOFF-202 in a separate session.
- **Your branch:** `claude/spec-092-verify`, cut from that head, plus this handoff and the
  verify prompt. The build already advanced the spec to `verify`.

SPEC-092 adds, in `src/color.rs`, `mono_mix_row`, `mono_mix_plane_into` and
`mono_mix_plane_rows_into`, six public constants (`MONO_MIX_ROW_FRACTION_BITS`,
`MONO_MIX_FADE_START`, `MONO_MIX_FADE_WIDTH_BITS`, `MONO_MIX_BLACK`, `MONO_MIX_WHITE`,
`MONO_MIX_VERSION`) and three `Error` variants. It generalises SPEC-091's identity sink so
the mono plane reuses the one fused traversal; no existing output may change.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-092-the-mixer-at-sensor-geometry-weights-only-rational-api-raw-c.md`.
  Judge the ACs against the spec **as designed**: `git show origin/main:<spec path>` (after
  PR 353). That is Design §1 to §12, AC1 to AC15, mutants (a) to (u), and the Failing Tests.
- **Decisions:** DEC-083 (accepted: row format, rounding, threshold, kernel, overflow),
  DEC-079 (the fold and the fade), DEC-033 (f64 basic operations only), DEC-018, DEC-016,
  DEC-040, DEC-044, DEC-051, DEC-053, DEC-078, DEC-069, DEC-070, DEC-002.
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `test-before-implementation`, `provenance-recorded-per-algorithm`, `one-spec-per-pr`.
- **Build handback:** HANDOFF-202's `## Handback` (mutant table, deviations, digests).
- **Prior verify:** HANDOFF-198 (SPEC-091) and its FU-10 lesson: a shared code path makes
  "equals the old path" oracles move with the mutant.
- **Toolchain:** run `just test`, never `just test --all-features`; set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`; use
  `/usr/bin/git` for branch and HEAD; touch a file after restoring it between mutants.

## What to check hard

1. **Every AC (1 to 15) against the spec as designed.** AC5's 15 golden rows must equal the
   spec's table as written. `MONO_MIX_VERSION == "1"` asserted beside AC5 and AC12, and
   documented like `DEVELOP_VERSION` (a mix is fixed by both versions, Design §12). The
   levels documented as part of the contract.
2. **The shared-sink generalisation.** It touches SPEC-091's shipped path. Confirm no
   existing output or determinism hash moved, and that the "equals the old path" oracles do
   not move with a mutant.
3. **The build's four deviations.** Rule on each: tests code-first (red by compile failure
   plus mutants); the AC8 point for mutant (i); the second order fixture
   `[2^-53, 2^-53, 1]`; the AC6 grid at about 135 s in the debug CI job.
4. **Mutants.** Re-run at least (e), (h), (i), (j), (k), one (l), one (m) and (r). Add:
   the fade decided on `v_i` sign versus `q_i` sign; the shift replaced by a truncating
   division; `u` computed from a white-balanced channel; band `dst` length off by one row;
   `MixRowOutOfRange` checked before the file checks. Check the argued equivalents (t), (u).
5. **The integer kernel** against DEC-083's overflow argument and AC14's extremes.
6. **Determinism.** Read the CI logs: both legs run the mixer hashes, and the fade engages
   on the fixture (8 of 384 pixels).
7. **Allocations and panics.** List the band path's allocations; no panics; the fuzz target
   reaches all three functions.
8. **Docs and API.** The threshold rustdoc (Design §5), CHANGELOG under Unreleased,
   `just semver-check` additions only.
9. **Gates.** `just test` with the corpus, `just validate`, `just cost-audit`, fmt, clippy,
   doc-check.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- Evidence for each item above, including the mutant table and the deviation rulings.
- Fixes on the verify branch are tests or docs only; `src/` changes go in a punch list.
- If approved: `just advance-cycle SPEC-092 ship --verdict approved`, `just handback-sync
  SPEC-092`, push, and fast-forward `feat/spec-092-mono-mixer` only if it is a
  fast-forward. Do not merge PR 354; do not start the ship.

## Out of Scope

- The developed mix (SPEC-093); presets; negative weights; a clip mask; a fade parameter.
- A reusable band plan (FU-9 closed as measured, Design §7); threaded variants.
- Any change to `DEVELOP_VERSION` or an existing hash. If one seems necessary, that is a
  rejection.

## Return Criteria — how to hand back

Fill the `handback:` front matter (with a real `tokens_total`) and the `## Handback`
section, including the reflection; set `handoff.status` and `handback.status` to
`completed`; return the verdict.

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
