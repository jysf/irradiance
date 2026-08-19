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
  id: HANDOFF-004
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-004: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` to `claude-opus-5` (reviewer) for a
**second verify** cycle, at `7446edd`.

Round 1 returned ⚠ PUNCH LIST with two P1s. Both are addressed. **No independent
reviewer has seen the fix** — the orchestrator reconciled it, which is a different
job and deliberately not a substitute.

## Context the Receiving Agent Needs

Read `HANDOFF-002`'s handback (the round-1 punch list), `HANDOFF-003` (the fix
brief), and **`DEC-007`** (which supersedes `DEC-006` and settles the design).

### What the orchestrator already did — don't just repeat it

All seven gates re-run: green. The policy-removal attack re-run independently:
`BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 1`. Previously
all seven were 0.

**That is reconciliation, not verification.** You should still run the attack —
§15 check #9 says a red-proof you did not personally observe failing is a
self-report — but spend the cycle on *judgement*, not on re-confirming green.

⚠ **A trap that cost the orchestrator a false negative.** `src/lib.rs` now
contains **two** occurrences of `#![deny(` — the real attribute, and a module-doc
paragraph naming the proof. A naive `index('#![deny(')` hits the doc one and
deletes 14 characters of prose, leaving the policy intact and the attack
invalid — it looks like the fix failed when it didn't. Target the occurrence at
**column 0**. (This is the third doc-comment collision this spec has produced;
consider whether that is worth a lesson signal.)

### What actually deserves scrutiny

1. **The three assertions.** clippy ran, exited non-zero, and named all three
   lints. The builder added two attacks of its own: a stub `cargo` (caught by
   assertion 1) and **deleting two of the five lints — which still exits 101**, so
   exit-code-only would have passed. Are three assertions sufficient, or is there
   a fourth failure mode? Consider: lints present but at `warn`; the injection
   landing somewhere the lints don't reach; clippy running against a stale copy.
2. **The injection heuristic** parses the attribute prologue by tracking bracket
   depth. `DEC-007` records this as the design's main weakness. Try to break it.
3. **A disclosed deviation beyond scope:** `AGENTS.md` §7 said
   `specs/  # (none yet — STAGE-001 is unframed by design)`, false with SPEC-001–005
   on disk. The builder corrected it and flagged it. Was that the right call?
4. **`core::fmt` / `core::error::Error`** — claimed measured on 1.90.0. Verify.
   `DEC-002` is still `proposed`, so this must not have quietly committed us to
   `no_std`.
5. **A follow-up the builder declined:** `guidance/constraints.yaml`'s
   `enforcement:` for `no-panics-on-untrusted-input` still reads
   *"fuzz targets…; clippy; review"* and should now name the red-proof. Out of
   HANDOFF-003's scope, so filed rather than done. **Confirm declining was right**
   — and if it belongs in this spec after all, say so.

### Settled — do not reopen

MSRV 1.90; the fuzz-job deferral to SPEC-003; `[lints]` in `Cargo.toml` (rejected
in DEC-007); the cost entries; `tier_map.build` (corrected on `main` to
`claude-opus-5` after this build ran on Opus while the map said Sonnet).

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**.

Work `AGENTS.md` §15 "During verify". The repo-specific checks that bite here:

- **#9** — run `./scripts/lint-red-proof.sh` and the policy-removal attack
  yourself. Watch the proof fail. Mind the doc-comment trap above.
- **#12** — zero dependencies. Confirm `Cargo.toml` and `Cargo.lock` agree.

If **APPROVED**, set `task.cycle: ship` and say so; the orchestrator runs ship
(reflection, `complexity_actual`, `archive-spec`, CHANGELOG).

⚠ **A merge hazard to be aware of, not to fix:** `guidance/signals.yaml` has
diverged — three signals on `main`, one on this branch, both appends to the same
region. Expect a conflict at merge; resolution is "keep all four." Do not
reconcile it here.

## Out of Scope

- Fixing anything. Punch-list it with file:line and send it back.
- Re-litigating settled items (see above).
- Any decoding work — SPEC-002 onward.
- The `signals.yaml` divergence.

## Return Criteria — how to hand back

1. Append a **verify** cost session with a real `tokens_total`. If `/cost` is
   unavailable to you, sum your transcript's usage objects and **say that is what
   you did** — the previous two cycles reported 197,940 and 15,379,660 by
   different methods, and the `token-counts-not-comparable` process-debt signal
   exists because of it. Note your cache-read share if you can.
2. Fill `## Completion` and the `handback:` block; `handoff.status: completed`.
3. State the verdict with the SHA reviewed.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

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
