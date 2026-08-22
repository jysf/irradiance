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
  id: HANDOFF-017
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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

# HANDOFF-017: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-007` for the **build** cycle.

Make the extraction path obey `DEC-012`. **A DNG-legal file must not become
unreadable because one interpretation tag is malformed.**

`DEC-012` was **amended 2026-08-21** and now answers the question this spec was
framed around — read the amendment before anything else; it is the operative text
where it and the old table disagree.

## Context the Receiving Agent Needs

### The defect, precisely

`DEC-012`'s old table said a malformed tag is *"fatal to that call only"*. But
`sensor()` **is** a call, so "only" silently included the plane. It conflated the
accessor that **read** the tag with the accessor the caller **invoked**.

Two live consequences, both reproduced, neither a regression:

- **`SPEC-004/FU-16`** — `sensor()` reads `Orientation` from `IFD0` with a bare
  `?` (`src/ifd.rs:1012`), so a malformed tag on a **non-sensor** IFD discards an
  already-located plane.
- **`SPEC-004/FU-17`** — a **DNG-legal `RATIONAL`** `DefaultCropSize`/`Origin`/
  `BlackLevel` makes the **whole file unreadable**: `uints()` (`src/ifd.rs:788`)
  returns `UnexpectedFieldType` and `sensor()` propagates it. Fatal to the file,
  not a missing field.

### The line, from the amendment

> **"What exists" is the plane — its presence, its location and its extent.**
> A tag that determines *whether there is a plane and where it is* is structural:
> malformed is fatal. Every other tag describes how to *interpret* a plane that
> already exists, and malformed costs **that field alone**.

The spec's Acceptance Criteria carry the **per-line classification measured at
design** — seven call sites, four to change, three to leave fatal. Transcribe it.

⚠ **`RowsPerStrip` stays fatal**, and every corpus file is single-strip, so real
data cannot test that classification. **Do not let a green corpus talk you out of
it.** If you disagree, argue it in the handback rather than quietly softening it.

### The shape to copy already exists

`SPEC-004` solved this for the *selection* path: `SensorMatch { Yes | No |
Unreadable(tag) }` — the structural rule applied **per-IFD instead of per-file**.
Do the analogous thing per-**tag** in `sensor()`: record it in
`Sensor::malformed_tags` and continue.

### `RATIONAL` is not even defined yet

`src/ifd.rs:141-145` declares BYTE/SHORT/LONG/UNDEFINED/IFD only. Add
`TYPE_RATIONAL` and read the two-`u32` pair TIFF defines. A zero denominator or a
non-integral value is a **malformed shape** — costs the field, does not fail the
file.

### A green corpus proves nothing here

No corpus file carries a malformed tag on the paths this spec changes — which is
exactly why FU-16 and FU-17 stayed latent across two specs. **Hand-built fixtures
are the evidence; the corpus is a regression check.**

## Expected Deliverables

1. The Structure / Interpretation split implemented per the spec's table.
2. `sensor()` records interpretation-tag failures instead of propagating them;
   leaf accessors keep returning `Err` honestly.
3. `TYPE_RATIONAL` read; zero-denominator and non-integral values cost the field.
4. `SPEC-004/FU-20` — `NoSensorIfdCandidatesMalformed` names only real candidates.
5. **Fixtures in BOTH directions** — interpretation malformed → file still reads
   and the tag is recorded; structural malformed → still fatal. A change that only
   shows the new tolerance has not shown the boundary still exists.
6. Ten gates green; fuzz covers the widened `uints()`.

## Out of Scope

- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Executing opcodes — STAGE-003.
- Re-litigating `DEC-012`'s amended line. If you think it is wrong, say so in the
  handback; do not implement a different one.

## Return Criteria — how to hand back

1. Paste the ten gates and the both-directions fixtures.
2. ⚠ Confirm each named test **exists** (`cargo test -- --list`) and **sum across
   targets** — a zero-match `cargo test <name>` exits **0**, and reading one
   target's line has given a wrong answer twice on this project, in both
   directions.
3. Fill `## Completion` and `handback:`. `tokens_total`: deduplicate by
   `message.id` and say so, **or** `null` with a written reason — never a guess.
   ⚠ `SPEC-004`'s build left it null and had to be grandfathered out of the cost
   gate, because the figure was unrecoverable once the session closed. Capture it
   **before** you finish.
4. `handoff.status: completed`; spec `task.cycle: verify`.
5. Branch `feat/spec-007-extraction-tolerance` off `main`; commit; do not merge.
   ⚠ Do not run `handback-sync` (finding 15).

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
