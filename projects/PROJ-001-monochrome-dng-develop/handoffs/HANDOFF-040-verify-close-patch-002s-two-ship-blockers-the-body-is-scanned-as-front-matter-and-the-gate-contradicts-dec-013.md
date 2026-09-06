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
  id: HANDOFF-040
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # PREDICTION from tier_map.verify. Correct it to what
                                    # your own system prompt reports as message.model.
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: PATCH-003

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

# HANDOFF-040: Verify PATCH-003 — the remediation of PATCH-002's ship-blockers, at `15c7fe0`

## Delegation Summary

Verify `PATCH-003` at **`15c7fe0`** on `fix/patch-003-close-patch-002s-two-ship-blockers`
(PR #10, CI 18/18, **not merged**). `main` at `b940c0d`.

⚠ **Same author as the code it fixes, and the code it fixes was written by the
orchestrator too.** `PATCH-002` was merged before its verify ran; that verify then
found 2 ship-blockers. This patch is my remediation of my own defects, so the
reviewer's independence is the only independence in the chain. **Assume I am
still wrong in the same direction.**

Your predecessor's review of `PATCH-002` (`HANDOFF-039`) is the model — its `M4b`
found something my own three mutations missed, and it did so by attacking the
*proof* rather than the code.

## What changed, and what to disbelieve

| id | claimed fix | what to attack |
|---|---|---|
| `SB-2` | awk counts front-matter delimiters and exits at the second, so the body is unreachable | Is it *unreachable*, or just harder to reach? Try: a `---` inside a YAML block scalar in the front matter; CRLF line endings; a file with **no** closing `---`; `--- ` with trailing space; a stage whose front matter is absent entirely |
| `SB-1` | `DEC-022` amends `DEC-013` §5; six files updated | Is the amendment *honest*? I argued "capture first" expired on one capture in three weeks. Judge that, and check I did not quietly amend §5's *"a null is honest; a guess is not"* — I claim I preserved it |
| `FU-1` | the `#` guard is unreachable; comment corrected | Confirm unreachability rather than accepting my proof of it |
| `FU-2` | the red-proof now asserts the stage **name** | Re-run `M4b`. Then ask what *else* the summary line claims that is still unasserted — it also says "the grandfathered stage is still exempt" |
| `FU-4` | quotes stripped from `status` | Try `'shipped'`, `shipped # comment`, trailing whitespace, `Shipped` |
| `FU-6` | `tokens_total: 0` now rejected | Try `00`, `0.0`, `-1`, ` 0 `, `0x0`, and a value larger than awk's integer precision |

**The red-proof gained a fourth case (`SB-2`)**, which I verified fails against
the old awk. Verify that claim, and check the case is not satisfiable some other
way.

## The finding I could not close, restated from measurement — judge whether I got it right

`FU-5` said: *"just lint and lint-red-proof.sh call a bare `cargo clippy`; this
machine's default toolchain is now nightly, which has no clippy — both fail."*

**It does not reproduce on the orchestrator's machine, and the reason matters:**

```
nightly toolchain has cargo-clippy   NO
default toolchain                    nightly-aarch64-apple-darwin
bare `cargo clippy` resolves to      /opt/homebrew/bin/cargo-clippy → clippy 0.1.97
just lint                            rc=0    just lint-red-proof   rc=0
```

Homebrew's clippy **shadows the rustup shim**, so both commands pass — while
linting with a compiler nobody selected. Your environment failed; mine silently
succeeds. **Same root cause, and the silent success is the worse half:** the gate
runs, reports green, and does not state which clippy produced the result.

I deferred it out of `PATCH-003` as "not this patch's." **Judge that call.** If
you think a gate whose result depends on `PATH` belongs in the same patch as a
gate that read prose as data, say so — the argument is available and I may have
split it wrongly to keep this patch small.

## Your own checks

1. **Does `cost-audit` still reject everything `PATCH-002`'s verify proved it
   should?** Re-run `M1`, `M2`, `M3`, `M4a`, `M4b` yourself. `M4a` is expected to
   **survive by design** — confirm the documentation for that is honest rather
   than a rationalisation of dead code.
2. **`DEC-022`'s Validation says: if the grandfather list grows past `STAGE-001`,
   the gate is wrong, not the stages.** Is that falsifier real, or unfalsifiable
   in practice? Who would notice it growing?
3. **The five stage files and the template.** I rewrote a comment in all six.
   Confirm none of them now says something that is false in the other direction,
   and that the replacement is true of `STAGE-001` too (which is grandfathered —
   the comment says the field is gated, and for that file it is not).
4. **Is `PATCH-003` itself missing a `DEC`?** I claimed `DEC-022` covers it. It
   also changed behaviour on `status` parsing and zero-handling without a record.
5. **Scope.** Six findings fixed in one patch. Was `FU-3`'s deferral
   (`cancelled` not audited) a decision or a convenience?

## Return Criteria

1. **Gates, run by you**, pasted, clippy version asserted, and **which list you
   ran**. ⚠ Given `FU-5`, also say **which clippy binary** answered and how you
   established that — this patch's whole subject is surfaces that do not state
   what produced their result.
2. **Observe CI green on the SHA you approve.**
3. **Every mutation re-run**, plus new ones from the table above. Each: file
   changed **and** ran **and** *output changed*.
4. ⚠ **Mutate in a disposable clone.** Two sessions have now lost work to
   `git checkout --` in this repo, one of them the orchestrator this week.
5. Handback: real `tokens_total`, deduped by `message.id`, per-component,
   **rounded up ~20 %**. ⚠ **`notes:` on ONE PHYSICAL LINE.**
6. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not merge.
7. Findings `SB-N`/`FU-N` from `FU-1` (this patch's own sequence) with §15
   dispositions.
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Fixing `FU-5` — it is being handled in parallel as its own patch. Judge my
  deferral; do not implement it.
- Merging PR #10, running `handback-sync`, backfilling `STAGE-001`.
- The gate-count ambiguity — filed, `bar: 3`.

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
