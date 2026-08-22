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
  id: HANDOFF-018
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-21
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-007

project:
  id: PROJ-001
  stage: STAGE-001
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

# HANDOFF-018: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-007` for the **verify** cycle, at
`0de18d4`. Independent session.

The spec is a direct transcription of `DEC-012`'s 2026-08-21 amendment. Read that
amendment before the spec — it is the operative text.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator

- **Ten gates green**, 58 tests (37 lib + 9 corpus + 12 ifd_reader), `main`
  untouched at `99086fb`, branch one commit ahead, tree clean.
- **All five named tests exist and pass** — confirmed with `--list` and summed
  across targets.
- **The boundary is preserved in code**: structural tags
  (`SamplesPerPixel:1173`, `Compression:1178`, `RowsPerStrip:1184`) are still bare
  `?`. Interpretation tags go through the new `cost_the_field` helper (11 sites).
- **The boundary test has teeth — mutation-tested by me.** Making `RowsPerStrip`
  tolerant (`.ok().flatten()`) turns `malformed_structural_tag_is_still_fatal`
  **red**; restoring turns it green. A change demonstrating only the *new*
  tolerance would not have proven the *old* fatality survived, so this was the
  property worth checking.

### What deserves scrutiny

1. **A disclosed scope extension.** The design table enumerated **7** call sites;
   the build applied the tolerance to **three more** array-tags —
   `DefaultCropOrigin`, `DefaultCropSize`, `BlackLevelRepeatDim` — arguing the
   amendment's own classification names them. **I think that is correct** (the
   amendment's Interpretation row lists all three explicitly, and my table was the
   narrower artefact). Confirm, and check nothing *structural* was swept in.
2. **`cost_the_field` is a new abstraction on a panic-free path.** Does it
   preserve the leaf/composite distinction the amendment requires — leaves still
   returning `Err` honestly, only the composite swallowing? And does it record
   **every** costed tag, or can one be dropped silently?
3. **`TYPE_RATIONAL`.** Zero denominator and non-integral ratios must cost the
   field, not fail the file. Check `checked_div`/`checked_rem` handle the
   denominator-zero case *before* the division, and that a legitimate integral
   RATIONAL is actually **read**, not merely tolerated.
4. **`SPEC-004/FU-20`** — `is_sensor_ifd` now short-circuits per identifying tag.
   Verify a readable *disqualifying* tag returns `No` before a later malformed tag
   can name a non-candidate, and that this did not change which IFD is selected on
   the 7 real files.
5. **The build ran on Sonnet 5, not the `tier_map.build`-predicted Opus 5**, and
   said so rather than letting it pass silently. That is the **second** tier_map
   mismatch, now in the opposite direction from the first. Worth a view: should
   the map stop predicting and record actuals only?

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the both-directions fixtures yourself (check #9), and **mutation-test at least
one structural tag** — a tolerance change that does not break the fatality test
would mean the boundary is unguarded.

⚠ Traps, all of which have produced wrong answers on this project: `cargo test
<name>` matching zero tests **exits 0** — confirm names with `--list`; **sum
across targets** — reading one target's line has misled in both directions; and
**assert your mutation compiled and applied** before concluding from it (that has
now failed five times here, including twice in this cycle).

**Label every finding `SB-N` / `FU-N`** per AGENTS.md §15, numbered for this spec.
If APPROVED, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Re-litigating `DEC-012`'s amended line.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.

## Return Criteria — how to hand back

1. Paste the ten gates, the both-directions fixtures, and your structural
   mutation test.
2. Fill `## Completion` and `handback:`. `tokens_total` deduplicated by
   `message.id`, and **captured before the session closes** — `SPEC-004`'s build
   left it null and had to be grandfathered out of the cost gate.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship`).
4. Commit on `feat/spec-007-extraction-tolerance`; do not merge. Do not run
   `handback-sync`.

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
