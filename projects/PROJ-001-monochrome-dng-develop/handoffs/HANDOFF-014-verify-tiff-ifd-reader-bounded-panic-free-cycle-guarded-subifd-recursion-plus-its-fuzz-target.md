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
  id: HANDOFF-014
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-014: <Task Title — same as the spec's title>

## Delegation Summary

Second verify cycle on `SPEC-003`, at `93dcae0` (fix `ff46fd9`). Independent
session.

Round 1 returned one ship-blocker — **documentation and config only, no `src/`
change** — and found the reader itself sound. That finding stands; do not re-derive
it.

## Context the Receiving Agent Needs

### Already reconciled by the orchestrator — don't just repeat

- **`src/` is byte-identical to `b79c7ef`.** Empty diff. The fix touched records
  and config only.
- **Ten gates green**, including the new `just deny-fuzz`.
- **SB-1's gate red-proofed by me, both directions:** removing the
  `libfuzzer-sys` exception → **exit 4**, `error[rejected]` naming
  `(MIT OR Apache-2.0) AND NCSA`; restored → 0; the library `just deny` gate
  unaffected. It has teeth.
- **The `handback-sync` hazard is handled** — `synced_at` stamped on
  HANDOFF-011/012/013 so the tool is a no-op. Without it, three hand-written
  sessions plus three transcriptions would have doubled `cost.totals`. Recorded as
  template finding 15. **Do not run `handback-sync` on this spec.**

### What deserves scrutiny

1. **The scope widening — was it right?** The handoff asked for one matrix row;
   the build wrote three, arguing that fixing only the named body satisfies the
   handoff and not the rule the handoff invoked. It also split the "validates
   against ONE camera" section and reclassified the Pentax fixture from tier A to
   tier B. **Check the tier reclassification especially** — a 37 MB uncommitted
   file gating nothing is a real correction, but it changes what CI is claimed to
   cover.
2. **`DEC-012` — "strict on structure, tolerant on shape."** A real decision made
   during a *fix* round, which is unusual. It narrows FU-5's framing (`array()`
   tolerates a wrong count and nothing else) and **defers its own implementation
   to SPEC-004's first edit** rather than editing `src/` here. Is that deferral
   sound, or does it leave an unstated rule live in shipped code?
3. **The provenance finding.** `libfuzzer-sys`'s README says its vendored
   directory is NCSA; all 49 vendored files carry `Apache-2.0 WITH LLVM-exception`
   and none mentions NCSA. So the crate's SPDX expression *and* its README are both
   stale against its own code. The gate enforces the stricter reading — right call,
   but does `docs/provenance-ledger.md` need a row, given that is exactly the
   declared-vs-carried distinction it exists for?
4. **The exception vs. widened `allow` choice.** Reasoned in `deny.toml`. Agree?
5. **`just msrv`** now exists (the third `+toolchain` trap). Does it use the shim
   correctly, and is the trap recorded in the toolchain brief?

### Settled — do not reopen

The reader's logic (approved round 1) · the `-F` gate · `DEC-009` · the corpus
manifest design · the multi-strip gap (follow-up, recorded) · the cost figures.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the fuzz red-proof yourself (check #9) — ⚠ two traps: `cargo fuzz` needs the
rustup shim **first on PATH**, and **an unchecked index will not reach the
fuzzer** (the lint policy rejects it at compile time), so plant something
lint-clean.

Also red-proof `just deny-fuzz` yourself. ⚠ The exception is an
`exceptions = [...]` **array**, not `[[licenses.exceptions]]` blocks — the
orchestrator's first mutation targeted the wrong form, was a silent no-op, and
nearly produced a false "no teeth" finding. **Assert your mutation changed the
file before you run the check.**

**Label every finding ship-blocking or follow-up.** If the gate is sound and the
remainder are follow-ups, **approve** — this spec has had two build rounds and the
reader was found sound in round 1.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- The reader's logic; pixel decode/unpack (STAGE-002, `DEC-008`).
- Running `handback-sync` on this spec — it would duplicate.

## Return Criteria — how to hand back

1. Paste the gates and both red-proofs you ran yourself.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: deduplicate by
   `message.id` and **say so**, with cache-read share. Six measured factors now
   span **1.61×–2.25×** — not a constant, so no fixed correction is valid on a raw
   figure.
3. `handoff.status: completed`; spec `task.cycle: verify` (or `ship` if approved).
4. Commit on `feat/spec-003-ifd-reader`; do not merge.

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
