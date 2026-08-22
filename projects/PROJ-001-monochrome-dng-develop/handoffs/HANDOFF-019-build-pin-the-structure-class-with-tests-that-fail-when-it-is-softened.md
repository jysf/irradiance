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
  id: HANDOFF-019
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: null    # a DISPATCH HINT is not a measurement (SPEC-007/FU-6);
                    # whoever runs this cycle sets it to what ACTUALLY ran
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-21
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-008

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

# HANDOFF-019: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-008` for the **build** cycle.

`DEC-012`'s Structure class is **stated but not enforced**. Four of its five tags
can be softened to tolerant with **nothing failing**. This spec makes the decision
real.

⚠ `to_agent` is deliberately `null`. `tier_map` was 0-for-2 as a prediction
(`SPEC-007/FU-6`); set it to what actually ran, in the handback.

## Context the Receiving Agent Needs

### The measurement that motivates this spec

Softening each structural tag, running the **full 58-test suite with the corpus
present** — measured by SPEC-007's reviewer, `Compression` reproduced independently
by the orchestrator:

| structural tag → tolerant | full suite |
|---|---|
| `RowsPerStrip` | **RED** |
| `Compression` | all green |
| `StripOffsets` | all green |
| `StripByteCounts` | all green |
| `BitsPerSample` | all green |

**`Compression` is the dangerous one.** Softened it defaults to `1`,
`require_uncompressed()` passes, and **STAGE-002 reads JPEG bytes as raw samples** —
a wrong image from a file that parsed cleanly.

The orchestrator had mutated `RowsPerStrip` alone and reported *"the boundary test
has teeth."* One point on a boundary is not a boundary — that is
`measurement-over-generalised`, now at N=3.

### The pattern already exists; copy it four times

`malformed_structural_tag_is_still_fatal` (`src/ifd.rs:1716`) plants an invalid
field type and asserts `sensor()` errors. It works for `RowsPerStrip` because it is
the only tag it is written for. Accessors measured at design — `BitsPerSample` via
`required_scalar()` (1171), `Compression` via `scalar()?` (1178), `StripOffsets`
and `StripByteCounts` via `values()` (1186/1187) — all reach `uints()` and all
propagate with `?`, so the same shape should reach them. **Verify that; if one does
not error, that is a finding about the code, not a licence to weaken the test.**

⚠ **`SamplesPerPixel` and `Photometric` are equivalent mutants** — re-reads of tags
`is_sensor_ifd` already validated. Unkillable by construction. Do **not**
manufacture a test that appears to cover them; leave a comment saying why.

### FU-4: one global line

`uints()` at **`src/ifd.rs:800`** accepts `TYPE_RATIONAL` in the **global** match,
so every tag reading through it accepts RATIONAL — including `SubIFDs` (330),
which `DEC-012` calls **structural**. `RATIONAL 400/2` now walks a SubIFD where
`main` returned `Err`. Make it **per-tag**, and write the reasoning down.

### FU-1/2/5: the record says something untrue

- **FU-1** — plane in `IFD0`: `Orientation` is costed twice,
  `malformed_tags = [274, 274]`. The Pentax `.PEF` is `sensor_ifd #0` — a **corpus
  shape**, not hypothetical.
- **FU-2** — a *well-formed* `Orientation` on the sensor IFD is recorded as
  malformed anyway.
- **FU-5** — every well-formed RATIONAL fixture uses denominator `1`, so a mutant
  pushing the numerator and ignoring the quotient passes all 58 tests.

## Expected Deliverables

1. A failing-when-softened test for `Compression`, `StripOffsets`,
   `StripByteCounts`, `BitsPerSample`.
2. **Proof each one kills its mutant** — soften the tag, show the suite red,
   restore, show it green. Paste both directions per tag. A test that exists is not
   a test that bites.
3. `uints()`'s RATIONAL acceptance made **per-tag**; `SubIFDs` rejects it again.
4. FU-1, FU-2, FU-5 corrected.
5. Ten gates green.

## Out of Scope

- Redrawing `DEC-012`'s Structure/Interpretation line. Enforce it; do not move it.
- New tolerance anywhere.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.

## Return Criteria — how to hand back

1. Paste the ten gates and **both directions for each of the four mutants**.
2. ⚠ Confirm each named test exists (`--list`) and **sum across targets**.
3. ⚠ **Assert every mutation compiled and applied** before concluding from it —
   that has failed five times on this project, twice in one cycle.
4. Fill `## Completion` and `handback:`; set `to_agent` to the model that
   **actually ran**; capture `tokens_total` deduplicated by `message.id`
   **before the session closes**.
5. `handoff.status: completed`; spec `task.cycle: verify`.
6. Branch `feat/spec-008-pin-structure-class` off `main`; commit; do not merge.

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
