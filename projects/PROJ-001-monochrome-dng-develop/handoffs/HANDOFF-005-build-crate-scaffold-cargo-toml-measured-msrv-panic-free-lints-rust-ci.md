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
  id: HANDOFF-005
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5    # corrected; see .repo-context.yaml tier_map           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-18
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-001

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

# HANDOFF-005: <Task Title — same as the spec's title>

## Delegation Summary

Third build cycle on `SPEC-001` — the round that closes the lint-policy gate for
good, per **`DEC-009`** (which supersedes `DEC-007`, which superseded `DEC-006`).

Verify round 2 returned ⚠ PUNCH LIST with two P1s. **Both were reproduced
independently by the orchestrator**, and both produce *seven green gates with a
panic shipped in the library*.

## Context the Receiving Agent Needs

Read **`DEC-009` first** — it settles the design; do not redesign it.

### The failure, measured

A plain `//` comment is legal in a prologue. `find_injection_line`
(`scripts/lint-red-proof.sh:63-90`) handles blank, `//!` and `#![` — a `//` line
falls through and **becomes the injection point**, above the inner attributes.
`pub fn` there is a syntax error; clippy exits non-zero; and rustc **renders the
attribute's source span in the diagnostic**, so all three expected lint names
appear in the log with no lint firing.

Reproduced: `//` comment before `#![forbid]`, `#![deny(` → `#![allow(` at column 0,
plus `pub fn boom(v: &[u8], n: u8) -> u8 { v[0] + n }` and one using `.expect()`:

```
BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0
✓ lint policy red-proof: ... rejected the injected violations (... all fired).
```

Every word of that success message is false.

**And `DEC-007` claimed this exact case was mitigated.** It is now marked
falsified in place. Do not trust a superseded DEC's reasoning.

### The fix — DEC-009, in order of importance

1. **NEGATIVE CONTROL.** Run the *same* clippy invocation on the **unmutated**
   copy first and require **exit 0**. If it is non-zero, fail with a message
   saying the toolchain/copy/crate is broken and nothing downstream is meaningful.
   *This is the one that closes the class* — every assertion so far inspected only
   the mutated run, so none could tell "failed for my reason" from "failed for any
   reason".
2. **All five lints.** `unwrap_used`, `expect_used`, `indexing_slicing`, `panic`,
   `arithmetic_side_effects` — each needs its own injected violation **and** an
   `EXPECTED_LINTS` entry. Today `panic` and `expect_used` are in neither, so
   deleting exactly those two from the policy passes everything (PL-2).
3. **Prologue parser** skips plain `//` comments, and refuses any injection point
   that is not strictly after the last inner attribute.
4. **`INJECT_AT=1` crashes** — `head: illegal line count -- 0`. Hit independently
   by both reviewer and orchestrator.

### The rest of the punch list

- Three artifacts assert a discrimination property the script does not yet have
  (the reviewer names them). Correct the **artifacts**, not just the script — a
  doc that overstates a guarantee is the same defect one level up.
- `deny` → `warn` currently passes. It must not.
- **Run `just handback-sync SPEC-001`** — build-2's 15,379,660 is un-synced, so
  `cost.totals` understates by ~74%.
- `guidance/constraints.yaml`'s `enforcement:` for `no-panics-on-untrusted-input`
  still reads *"fuzz targets…; clippy; review"*. **Now** is the round to add the
  red-proof — the reviewer was right that writing it earlier would have documented
  a guarantee that did not exist. It does after this change.

## Expected Deliverables

1. `scripts/lint-red-proof.sh` per `DEC-009`: control run, five lints, hardened
   prologue parser, no `INJECT_AT=1` crash.
2. **All four `DEC-009` Validation cases demonstrated in the handback**, with
   pasted output:
   - policy present → passes, all five lints named
   - policy deleted → fails
   - policy `deny`→`allow` → fails
   - `//` comment in the prologue → fails (or injects correctly); never passes
3. `guidance/constraints.yaml` `enforcement:` names the red-proof.
4. The three over-claiming artifacts corrected.
5. `just handback-sync SPEC-001` run; `cost.totals` correct.
6. All seven gates green, run for real, output pasted.

## Out of Scope

- Redesigning `DEC-009`. If you believe a fifth bypass exists, **say so in the
  handback** rather than inventing a sixth mechanism — DEC-009 names the
  compile-fail-harness alternative and why it needs its own decision.
- Settled: MSRV 1.90 · fuzz deferral to SPEC-003 · `[lints]` in `Cargo.toml` ·
  `core::` (verified) · `AGENTS.md` §7 (verified correct).
- The `signals.yaml` merge divergence — leave it.
- Any decoding work.

## Return Criteria — how to hand back

1. Paste output for all seven gates **and all four Validation cases**. The
   orchestrator will re-run the `//`-comment bypass itself; it has the exact
   reproduction.
2. Fill `## Completion` and `handback:`. For `tokens_total`: if `/cost` is
   unavailable, sum your transcript usage objects and **say that is what you
   did**, with your cache-read share (see the `token-counts-not-comparable`
   signal).
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

**This gate has now been wrong three times, and each author believed it closed.**
If something feels unproven, say so in the handback rather than shipping a fourth
confident mechanism.

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
