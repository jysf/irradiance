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
  to_agent: claude-sonnet-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-21
  status: completed                  # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  tokens_total: 19480728           # REAL combined count — what cost-audit reads
  estimated_usd: 8.62              # tokens_total × your rate, or your harness's number
  duration_minutes: 75
  branch: feat/spec-007-extraction-tolerance
  pr: null
  completed_at: 2026-08-21         # YYYY-MM-DD
  notes: "BACK-FILLED 2026-08-21 by the orchestrator, closing SPEC-007/FU-7: this block was left entirely null while `handoff.status: completed` and the prose `## Handback` below was complete. Nothing was lost — the build wrote `cost.sessions` directly, so SPEC-007 has always carried the real figure — but the machine-readable half said nothing, which is `cost-field-has-two-owners` firing from the other side. Values transcribed verbatim from SPEC-007's own build session; not re-derived. tokens_total is a transcript sum deduplicated by message.id (174 usage objects, 87 distinct ids; raw 38,007,198, deduped 19,480,728 = 1.95x, 98.2% cache-read); estimated_usd computed per-component at published Sonnet rates, NOT harness-reported."
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

- **Branch / PR:** `feat/spec-007-extraction-tolerance` (local commit; not pushed, no PR opened per return criteria — "commit; do not merge")
- **Completed at:** 2026-08-21
- **All acceptance criteria met?** yes.
  1. Structure/interpretation split transcribed exactly per the spec's table: Orientation, BlackLevel, WhiteLevel, ActiveArea now cost the field (via a new `Container::cost_the_field<T>` composite-catch, same shape as `SensorMatch`); SamplesPerPixel, Compression, RowsPerStrip left untouched (already bare-`?`, already fatal) with a one-line DEC-012 classification comment added to each. Also applied `cost_the_field` to DefaultCropOrigin/DefaultCropSize/BlackLevelRepeatDim, which the amendment's table classifies as interpretation even though the design-time table's 7-line list didn't enumerate their call sites separately — HANDOFF-017's own Context names DefaultCropSize/Origin explicitly as part of FU-17, so this is required by the amendment, not a scope expansion.
  2. Leaf accessors (`scalar`/`array`/`uints`) still return `Err` honestly — unchanged. Only `sensor()`'s call sites changed.
  3. `TYPE_RATIONAL` (5) added and read as TIFF 6.0's two-`LONG` pair; zero denominator or non-integral ratio is `Error::MalformedRationalValue`, via `checked_div`/`checked_rem` (no raw `/`/`%`, so `clippy::arithmetic_side_effects` stays clean).
  4. FU-20 fixed: `is_sensor_ifd` now evaluates each identifying tag before reading the next, so a readable, disqualifying tag short-circuits to `SensorMatch::No` before a later tag's malformed read can wrongly mark the IFD `Unreadable`.
  5. Fixtures in both directions — see the ten-gates section below; five hand-built pairs, all in `src/ifd.rs`'s own unit tests, plus one shared well-formed RATIONAL fixture added to `tests/support/tiff.rs` (reused by `tests/ifd_reader.rs`'s hostile-input list and the fuzz seed corpus, per that file's own stated convention).
  6. Ten gates green; fuzz run against the widened `uints()` (11,553,927 runs, 61s, zero crashes) — see below.
- **For `verify`:** N/A — this is the build handback.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 19,480,728 — a transcript sum, **deduplicated by `message.id`**, and I say so: this session's own transcript at `~/.claude/projects/<path-slug>/f65fb967-f26f-4c03-a16b-bdae0c0b917a.jsonl` (session id read off the scratchpad path the harness gave me — the exact method SPEC-004's verify used) has 174 `usage` objects across 87 distinct message ids; raw sum 38,007,198, deduped 19,480,728 (1.95x — inside the 1.61x-2.51x band already on record, not a new extreme). 98.2% cache-read; the one cache-creation sample I inspected carries `ephemeral_1h_input_tokens` only (0 in the 5-minute tier), so all cache writes this session were the 1-hour kind. **This is a floor** — captured with a few turns of work still left (finishing this handback, committing), per the handoff's explicit instruction not to let it go uncaptured.
- **Estimated USD:** 8.62 — computed, not harness-reported: input 174 × $3/M + output 89,257 × $15/M + cache-write(1h) 255,875 × $6/M + cache-read 19,135,422 × $0.30/M, using published Sonnet rates. Flagged as an estimate so it isn't mistaken for a metered figure.
- **Duration (minutes):** ~75 (session start to this handback, by wall clock; not separately instrumented).
- **Source of the number:** none available via `/cost` from inside a turn (same limitation SPEC-004's build hit) — **transcript `usage` objects**, read directly, same methodology as SPEC-004's verify.

### Drift and new artifacts

- **New decisions emitted:**
  - None. Every change is a direct transcription of `DEC-012`'s amendment and the spec's own acceptance-criteria table — the `cost_the_field` composite-catch is the same shape `SensorMatch` already established for the selection path, and `TYPE_RATIONAL` is read straight from TIFF 6.0 §2. This is the same finding SPEC-004's build recorded for its own comparable additions (`SensorMatch`, the new `Error` variant) — a prescribed shape, not an implementer judgment call.
- **Deviations from spec:**
  - Widened `cost_the_field` tolerance to `DefaultCropOrigin`/`DefaultCropSize`/`BlackLevelRepeatDim` in addition to the four sites (Orientation, BlackLevel, WhiteLevel, ActiveArea) the design-time table lists by line number. The amendment's own classification table puts all of these under Interpretation, and this handoff's Context explicitly names `DefaultCropSize`/`DefaultCropOrigin` as part of FU-17's blast radius — so I read the 7-line table as "the sites verified at design," not "the exhaustive set to change," and applied the stated rule uniformly rather than leave three interpretation-classified array-tags with only partial (count-mismatch-only) tolerance. Flagging this explicitly in case verify reads it differently.
  - `to_agent` on this handoff is `claude-sonnet-5`, not the `claude-opus-5` `.repo-context.yaml`'s `tier_map.build` names. I am the model actually running this session (system prompt: "Claude Sonnet 5"); recording the tier_map's prediction instead of the truth is exactly the silent-cost-surprise DEC-004 rule 3 (and the tier_map's own 2026-08-18 correction comment) exists to prevent. Recorded truthfully in both the front-matter `to_agent` and the spec's `cost.sessions[].agent`; not a build-scope deviation, but worth surfacing since it means SPEC-004's `build: claude-opus-5` tier assumption did not hold for this cycle either.
- **Follow-up work identified:**
  - None new. `RowsPerStrip`'s structural classification is still unverified by real data (every corpus file remains single-strip) — this was already flagged as a known, accepted gap in `DEC-012`'s amendment and the spec itself, not a new finding.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing blocking. The one soft ambiguity: the acceptance-criteria table names 7 call sites "measured at design," but the amendment's classification table and the handoff's own Context (FU-17 naming `DefaultCropSize`/`DefaultCropOrigin`/`BlackLevel`) imply a broader set. I resolved it by applying the stated rule to every DEC-012-Interpretation-classified tag rather than only the 7 lines, and documented the reading as a deviation above rather than silently narrowing scope.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-012` (amended), `guidance/constraints.yaml`'s five blocking constraints, and `guidance/toolchain-brief.md`'s three `+toolchain` traps were exactly what this build needed and all were already listed.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. I would capture the transcript-based token count once at the very start (to get a true zero-point) and once at the end, rather than only near the end, to make the floor-vs-final gap explicit rather than inferred.
