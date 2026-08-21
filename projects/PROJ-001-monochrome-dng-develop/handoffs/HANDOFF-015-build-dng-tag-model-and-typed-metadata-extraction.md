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
  id: HANDOFF-015
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-21
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-004

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

# HANDOFF-015: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-004` for the **build** cycle.

Extract the remaining DNG tags the develop pipeline reads — and close the two
obligations SPEC-003 deferred here rather than make a `src/` edit in a
records-only round.

## Context the Receiving Agent Needs

### The two obligations are one question: what does a malformed tag cost?

**Read `DEC-012` first.** It states the rule — *a malformedness that changes what
exists is fatal; one that changes only what a known-optional field says costs that
field alone* — and it was written during SPEC-003's fix round specifically so this
spec would not re-derive it.

**FU-11 is where the code contradicts that rule.** Measured at design:
`is_sensor_ifd` (`src/ifd.rs:836`) calls `self.scalar(...)?` three times, and
`sensor_candidates`, `sensor_ifd` and `sensor` each run it over **every** IFD. So a
malformed tag on a *thumbnail* fails the whole container.

It is **latent, not live** — no corpus file carries a malformed tag on that path
(the Pentax's malformed `BlackLevelRepeatDim`, tag 50713, is not one of the three).
**Do not conclude from a green corpus that the path is sound.**

⚠ **The obvious fix is wrong.** Silently treating a malformed scalar as "not a
sensor IFD" hides a real plane behind a bad tag: if the *sensor* IFD's
`Photometric` is malformed, you convert a readable file into a bare `NoSensorIfd`
with no explanation. A malformed candidate must be **skipped and recorded**, and if
no candidate is then found the error must say **why**. Same discipline as the
corpus reader's loud skip — an invisible skip is the defect.

### Corpus facts, re-measured 2026-08-20 — use these, not older numbers

- **6 `II`, 1 `MM`** across 7 files
- **4 uncompressed, 2 JPEG (code 7), 1 vendor-private (65535)**
- `K3III.PEF`: **no SubIFD, no `NewSubfileType`**, plane in `IFD0`, and the only
  file with a real IFD *chain*
- M Monochrom: **no `ActiveArea`**, **no opcode lists**

⚠ Three earlier claims in this project's specs were wrong on exactly these points,
and each was a `find`/`exiftool` away. **Re-measure anything you are about to
assert.**

### Types matter here

`ActiveArea` as a bare `Vec<u32>` makes the caller remember it is
`[top, left, bottom, right]`. Give it a shape. And **absent must not collapse into
zero** — `ActiveArea` is missing on the M Monochrom, `NewSubfileType` is missing on
the PEF, and TIFF's absent-means-0 default for the latter is *what finds that
plane at all*.

## Expected Deliverables

1. Typed extraction of `BlackLevel`, `WhiteLevel`, `ActiveArea`,
   `DefaultCropOrigin`, `DefaultCropSize`, `Orientation`, and the **presence** of
   `OpcodeList1`/`OpcodeList3`.
2. `DEC-012` implemented; **FU-11 closed** per the subtlety above.
3. Hand-constructed TIFFs (via `tests/support/tiff.rs`, shipped by SPEC-003)
   proving a malformed tag on a **non-sensor** IFD and on the **sensor** IFD have
   **different, asserted** outcomes.
4. Values matching `exiftool` on all 7 files, pinned as an expected table.
5. Fuzz coverage of the new extraction paths.
6. All ten gates green, output pasted.

## Out of Scope

- **Levels arithmetic, cropping, orientation transforms** — STAGE-002 and
  `DEC-008`. Extracting `BlackLevel` is in scope; subtracting it is not.
- Executing opcodes — STAGE-003. Presence only.
- Any new dependency.

## Return Criteria — how to hand back

1. Paste all ten gates and the two malformed-tag tests from deliverable 3.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: deduplicate by
   `message.id` and **say so**, with cache-read share. Seven measured factors span
   **1.61×–2.25×** — not a constant.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-004-tag-model` off `main`; commit; do not merge.
   ⚠ Do **not** run `handback-sync` — see finding 15.

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
