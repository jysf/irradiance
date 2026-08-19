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
  id: HANDOFF-002
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

# HANDOFF-002: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` to `claude-opus-5` (reviewer) for the
**verify** cycle.

Review the crate scaffold on `feat/spec-001-crate-scaffold`. **You are a different
session from the builder — that independence is the entire point of this cycle**
and the dogfood's best-evidenced quality lever. Do not read the build session's
report as evidence; re-run things.

## Context the Receiving Agent Needs

Branch `feat/spec-001-crate-scaffold`, 3 commits on top of `e8633b6`.

**The orchestrator has already reconciled the build against git and disk**
(DEC-004 rule 1) and re-ran every gate. All six pass. So your job is **not** to
re-confirm the gates go green — it is to ask whether green means what it claims.

### What the build did that is worth scrutiny

1. **It changed my spec's red-proof design, and I agreed.** SPEC-001's literal
   snippet would have been a **false green**: a `tests/*.rs` file is its own crate
   root and does *not* inherit `src/lib.rs`'s `#![deny(...)]`. The builder caught
   this and made the snippet carry its own `#![deny]`, swapped in by
   `scripts/lint-red-proof.sh`. I verified the claim independently. **Verify the
   fix is sound, not just present** — e.g. does the script restore state on
   failure, and would it still fail if the lints were silently removed from
   `src/lib.rs`?
2. **It edited `AGENTS.md` §5/§6/§7**, which was not in its deliverables list. My
   read: legitimate, because those sections said "no `Cargo.toml` exists yet",
   which the change made false, and §1 requires AGENTS.md to be true. **Confirm
   the edits are accurate and did not overreach.**
3. **It fixed a `.gitignore` bug** — `Cargo.lock  # comment` — gitignore does not
   strip inline comments, so the pattern matched nothing. Real bug, out of scope,
   correctly fixed. Confirm it now actually ignores `Cargo.lock`.
4. **It emitted `DEC-006`** for the red-proof mechanics at confidence 0.85.
5. **It correctly declined to wire a fuzz job**, deferring to SPEC-003 per §12
   bar 2. Confirm that is the right call rather than an omission.

### Cost is already handled — do not re-open it

`tokens_total: 197940` was filled by the orchestrator from the Agent result
metadata, and `handback-sync` transcribed it. The builder left it `null` and said
why; that was correct under `metering_source: subagent_tokens` (DEC-013).

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**.

Work the checklist in `AGENTS.md` §15 "During verify" — the 8 standard checks plus
this repo's 4 extra, of which these apply here:

- **#9 Did the oracle go red?** There is no decode oracle in this spec, but the
  **lint policy is a gate that must be shown red**. Run
  `./scripts/lint-red-proof.sh` yourself and watch it fail the compile. A
  red-proof you did not personally observe failing is a self-report.
- **#12 Is any new dependency permissive, and not a RAW decoder?** The answer
  should be *zero dependencies*. Confirm `Cargo.toml` and `Cargo.lock` agree.

Additionally, the questions I most want answered:

- Does `rust-version = "1.90"` actually hold? Run
  `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` yourself.
  ⚠ `cargo +1.90.0` without the shim path FAILS — see the toolchain brief.
- Are the five panic-free lints **deny-level on the library** and **allowed only**
  in `#[cfg(test)]` and `src/bin/irr.rs`? A blanket allow anywhere else is a
  rejection.
- Does anything in CI or the README imply the decoder is verified? Per DEC-003 CI
  **cannot** run tier-B tests, so a green badge must not overclaim.
- Is `irr` genuinely absent from the library's public API?

## Out of Scope

- Re-doing the build. If something is wrong, punch-list it; don't fix it silently.
- Any decoding work — SPEC-002 onward.
- Re-litigating the MSRV number. 1.90 is measured-and-conservative by design; the
  true floor is knowingly unmeasured. Lowering it is a separate change.
- The cost entry (already correct).

## Return Criteria — how to hand back

1. Append a **verify** cost session to the spec's `cost.sessions` with a real
   `tokens_total` from your own interface (`/cost` in Claude Code). If your
   platform genuinely cannot report one, write `null` **and say why** — do not
   invent a number (DEC-013).
2. Fill this file's `## Completion` and `handback:` block; set
   `handoff.status: completed`.
3. State the verdict with the SHA you reviewed.
4. If APPROVED, set the spec's `task.cycle: ship` and say so; the orchestrator
   handles the ship cycle (reflection, `complexity_actual`, archive, CHANGELOG).
5. If PUNCH LIST, list each item with file:line and why it matters. Send it back
   to build rather than fixing it yourself — the independence cuts both ways.

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
