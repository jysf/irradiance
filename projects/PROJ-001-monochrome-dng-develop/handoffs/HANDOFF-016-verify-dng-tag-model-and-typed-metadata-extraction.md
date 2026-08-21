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
  id: HANDOFF-016
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-016: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-004` for the **verify** cycle, at
`37204d0`. Independent session.

⚠ **Read the first scrutiny item before anything else — the spec you are verifying
against was substantially wrong, and the build was right to deviate.**

## Context the Receiving Agent Needs

### ⚠ My spec's Goal was mostly already shipped, and the build caught it

`SPEC-004`'s Context claimed SPEC-003 "stops exactly where geometry begins."
**False.** `main` already carried `black_level`, `white_level`, `active_area`,
`orientation`, `opcode_lists` and `black_level_repeat_dim` as `Option<…>` fields
(`src/ifd.rs:442-460`), extracted via `scalar()`/`array()` with a `malformed`
accumulator — which also already satisfied my AC3 (absent ≠ zero) and part of
`DEC-012`.

I reached that conclusion by grepping `pub struct Sensor` and reading only its
first lines. The struct continued past where I stopped. **I did this in the very
handoff that warned the builder not to assert from an incomplete look.**

So the real remaining work was narrower than the spec says, and the build scoped
it correctly to three things. **Verify the deviation, not the spec's literal
wording** — and judge whether the narrowed scope is genuinely complete.

### Already reconciled by the orchestrator

- **All ten gates green.** `main` untouched; branch is one commit ahead; tree clean.
- **All five literally-named tests exist and pass** (`--list` confirms the names).
  ⚠ My own first check reported "0 passed" for four of them because I took the
  first target's line; they live in a different target. Sum across targets or use
  `--list`.
- **AC1 done:** `ActiveArea`, `DefaultCropOrigin`, `DefaultCropSize` are now
  named-field structs (`src/ifd.rs:421/435/445`), not bare `[u32; N]`.
- **FU-11 closed with a tri-state:** `SensorMatch { Yes | No | Unreadable(tag) }`
  (`src/ifd.rs:579-587`), and `Error::NoSensorIfdCandidatesMalformed` names what
  was unreadable instead of a bare `NoSensorIfd`.

### What deserves scrutiny

1. **Is the narrowed scope complete?** Given the spec's Goal was largely already
   met, did the build correctly identify what remained — or is something in the
   original list genuinely still missing? Check the tag list against
   `docs/measured-q2m-dng.md` yourself.
2. **The FU-11 tri-state is the substance.** `Unreadable(tag)` must produce
   *different, asserted* outcomes for a malformed tag on a **non-sensor** IFD
   versus on the **sensor** IFD. The design warned that silently skipping is wrong
   because it hides a real plane. Does `NoSensorIfdCandidatesMalformed` actually
   say *which* IFD and *which* tag, and is it reachable?
3. **`tokens_total: null`, with a written reason** — this ran as a top-level
   session with no `/cost` and no usage-object access. That is the correct
   behaviour under `DEC-013` (never invent a number), and it differs from earlier
   CLI sessions that could sum transcripts. Confirm the reason is recorded in both
   the handoff and the spec, and consider whether
   `.repo-context.yaml`'s `cost.metering_source` should now say `none` for this
   execution mode rather than leaving the gate asking.
4. **The build's own sharp observation:** `cargo test <name>` matching **zero**
   tests exits **0**. A spec that names tests therefore creates a silent-pass
   hazard if the names drift. Worth a signal, or a check?

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the two malformed-tag fixtures yourself and confirm the outcomes genuinely
differ — that pair is the spec. ⚠ Traps: `cargo +1.90.0` fails (use `just msrv`);
`cargo fuzz` needs the rustup shim first on PATH; an unchecked index will not reach
the fuzzer (the lint policy rejects it at compile time).

**Label every finding ship-blocking or follow-up.** If APPROVED, set
`task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Executing opcodes — STAGE-003.
- Running `handback-sync` (finding 15).

## Return Criteria — how to hand back

1. Paste the ten gates and the two malformed-tag fixtures.
2. Fill `## Completion` and `handback:`. `tokens_total`: deduplicate by
   `message.id` and say so, **or** `null` with a written reason — never a guess.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship` if approved).
4. Commit on `feat/spec-004-tag-model`; do not merge.

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
