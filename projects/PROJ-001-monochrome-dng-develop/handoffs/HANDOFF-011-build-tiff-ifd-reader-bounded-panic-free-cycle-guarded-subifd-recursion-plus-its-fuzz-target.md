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
  id: HANDOFF-011
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-003

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

# HANDOFF-011: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-003` for the **build** cycle.

The first spec that actually reads a RAW container. It is also the first to touch
**attacker-influenced binary input**, so `no-panics-on-untrusted-input` stops being
a policy and starts being the work.

## Context the Receiving Agent Needs

### ⚠ Read the toolchain brief's "SECOND `+toolchain` trap" before you fuzz

`cargo fuzz` shells out to a bare `"cargo" "build"` which resolves to Homebrew's
**stable** cargo and rejects `-Zsanitizer`. Even
`~/.cargo/bin/cargo +nightly fuzz run` fails, because the *inner* call is what
breaks. Use:

```bash
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run <target>
```

**Proven at design**, so criteria 4 and 5 are known-achievable: `cargo fuzz init`
works, a target ran **32.9 M execs in 16 s**, and a planted unchecked index was
**caught** — exit 77 plus a crash artifact.

### SPIKE-001's code is DISCARDED — its measurements are not

Do **not** consult that decoder as an implementation; `test-before-implementation`
is why, and retro-fitting tests to working code yields tests that cannot fail.
Reusable facts:

- Sensor-IFD selection: `NewSubfileType == 0 && Photometric == 34892 &&
  SamplesPerPixel == 1` — **never largest dimensions**. `SubIFD2` is a
  full-resolution JPEG preview only **56 px** narrower than the plane.
- Guards required: depth limit, cycle detection on visited offsets, bounds-checked
  payload ranges.
- ⚠ Its version used bounds-check-**then-index** (`buf.get(..)?` then `s[0]`),
  which the lint policy **rejects**. Use `try_into` on the slice. Its "229 lines"
  is an underestimate for exactly that reason — not a target.

### The corpus shapes the tests

Seven files in `tests/corpus/manifest.toml`, read through the **SPEC-002 reader** —
do not hardcode paths, and let absent files skip visibly. Two are **big-endian
(`MM`)** against five `II`. Three are **JPEG-compressed** and must be **rejected
cleanly**, not decoded. The Pentax carries a `BlackLevelRepeatDim` tag dnglab
itself warns is malformed — a free regression fixture the reader must not panic on.

### Scope fence

Container only. `StripOffsets`/`StripByteCounts` are read **as tags**; reading the
strip is STAGE-002, where `DEC-008`'s two-path (`bits % 8`) unpack rule lands.

## Expected Deliverables

1. The IFD reader: byte-order handling, IFD chain walk, SubIFD (tag 330)
   recursion, typed errors on every bounds failure.
2. Depth and cycle guards, with tests that a self-referential SubIFD terminates.
3. **A fuzz target in this change**, seeded from tier-A including truncated and
   malformed inputs.
4. **Evidence the fuzz target works:** plant an unchecked index, show libFuzzer
   catching it (exit 77 + crash artifact), then remove it. Paste both.
5. Tag extraction matching `exiftool` on all 7 corpus files.
6. All nine gates green, output pasted.

## Out of Scope

- Any pixel decode or unpack — STAGE-002.
- Consulting SPIKE-001's decoder as an implementation.
- Hardcoding corpus paths; SPEC-002's reader exists for this.
- Widening the lint exceptions. If the panic-free policy makes something awkward,
  that awkwardness is the constraint working — say so in the handback rather than
  reaching for `#[allow]` (which `SPEC-006`'s gate will reject anyway).

## Return Criteria — how to hand back

1. Paste all nine gates, plus both fuzz directions from deliverable 4.
2. Fill `## Completion` and `handback:`. ⚠ For `tokens_total`: **transcript sums
   double-count ~1.9x** — deduplicate by `message.id` and **say that you did**,
   with cache-read share. See `token-counts-not-comparable`.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-003-ifd-reader` off `main`; commit; do not merge.

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
