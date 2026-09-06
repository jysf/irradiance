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
  id: HANDOFF-042
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # PREDICTION from tier_map.verify. Correct it to what
                                    # your system prompt reports as message.model.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: PATCH-004

project:
  id: PROJ-001
  stage: STAGE-XXX
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

# HANDOFF-042: Verify PATCH-004 — the lint gates name their compiler, at `d81cecd`

## Delegation Summary

Verify `PATCH-004` at **`d81cecd`** on
`fix/patch-004-lint-gates-state-which-clippy-answered` (PR #11, CI 18/18, **not
merged**). `main` at `b940c0d`. **Small — one script, two `just` recipes, two docs.**

⚠ **Orchestrator-authored, and unverified merges are why this repo currently has
three patches on one gate.** `PATCH-002` was merged before its verify; that verify
found two ship-blockers, and its successor's verify found two more. This one is
being verified *before* merge specifically because of that.

## What it does

`FU-5` reported that `just lint` and `lint-red-proof.sh` **fail**, because the
default toolchain is nightly and nightly has no clippy. **It does not reproduce
here** — Homebrew's `cargo-clippy` (1.97.1) shadows the rustup shim, so both
commands *pass* while linting with a compiler nobody selected.

So the patch does not fix a failure. It makes the gates **state what produced
their result**:

```
lint:    using clippy 0.1.97 (/opt/homebrew/bin/cargo-clippy) — unpinned
lint-ci: using clippy 0.1.98 (88d9e12ae1 2026-08-18) (pinned: ~/.cargo/bin/cargo +stable)
… PROVED BY: clippy 0.1.97 (/opt/homebrew/bin/cargo-clippy).
```

`lint` is left **unpinned on purpose** — it is the fast local check and `lint-ci`
is the pinned one. The claim is that the defect was never that `lint` is
unpinned, only that it did not say so. **Judge that claim.**

## What to attack

1. ⚠ **`command -v cargo-clippy` may not be what `cargo clippy` actually
   executed.** Those are two different resolutions, and the new success line
   asserts they are the same binary. Are they, always? If `cargo` resolves a
   subcommand differently from `command -v`, the line **names the wrong
   compiler — which is this patch's own defect, one level up.** This is the
   check most likely to find something; start here.
2. **The new `die` is the only new enforcement.** It fires when
   `cargo clippy --version` answers but `command -v cargo-clippy` does not. Both
   paths were watched red via `PATH` shims. Reproduce, then find a **third
   state**: a `cargo-clippy` that exists but is not executable; a shell function
   or alias shadowing it; `cargo` itself absent; `command -v` returning a
   relative path.
3. **`rustup which --toolchain stable cargo-clippy` vs `--version`.** Your last
   review drew exactly this distinction. Does the patch's chosen mechanism
   survive it?
4. **The `printf` lines run on every `just lint`.** Confirm a failing `$(...)`
   inside them cannot take down the recipe, and that `@` suppresses echo as
   intended.
5. **`guidance/toolchain-brief.md`** gained a section claiming *PATH order, not
   `RUSTUP_TOOLCHAIN`, selects clippy*. That is a claim about a class, measured
   on one machine. Verify it on yours.

## Out of Scope

- `PATCH-003` / PR #10 — separate re-verify (`HANDOFF-041`).
- Changing the environment (installing clippy into nightly, removing Homebrew's).
  This patch makes the situation visible; fixing the machine is not a repo change.
- Merging, `handback-sync`.

## Return Criteria

1. **Gates, run by you**, pasted, with the clippy version **and which binary**,
   established via `rustup which` rather than `--version`. Say which gate list.
2. **Observe CI green on the SHA you approve.**
3. **Both claimed red-proof paths reproduced**, plus your third state. Each: file
   changed **and** ran **and** *output changed*.
4. ⚠ **Mutate in a disposable clone**, and use `/usr/bin/git` for anything you
   report — rtk-wrapped git served stale HEAD/branch/status in a prior round.
5. Handback: real `tokens_total`, deduped by `message.id`, per-component,
   **rounded up ~20 %**. ⚠ **`notes:` on ONE PHYSICAL LINE.**
6. **Correct `handoff.to_agent`.** No `handback-sync`, no merge.
7. Findings from `FU-1` (this patch's own sequence).
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

---

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
