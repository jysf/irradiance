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
  id: HANDOFF-041
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

# HANDOFF-041: Re-verify PATCH-003 round 2, at `65f0161`

## Delegation Summary

**Round 2.** You (or your predecessor) returned ⚠ PUNCH LIST at `15c7fe0` — 2
ship-blockers, 9 follow-ups. **Both ship-blockers were right**, both are fixed,
and all nine follow-ups are dispositioned. Re-verify at **`65f0161`** on
`fix/patch-003-close-patch-002s-two-ship-blockers` (PR #10, CI 18/18, **not
merged**). `main` at `b940c0d`.

⚠ **This is the third round on one gate, all authored by the orchestrator.**
`PATCH-002` shipped it, was merged before its verify, and that verify found two
ship-blockers. `PATCH-003` fixed those; its verify found two more. This is the
fix for *those*. **The prior is that I am still wrong somewhere**, and the last
two rounds were both wrong in the same way: an over-claim about what a scan
could reach, defended by a proof that modelled one shape.

## What changed since `15c7fe0` — and what to disbelieve

| finding | fix | attack |
|---|---|---|
| `SB-1` fence | normalise `\r` + trailing whitespace; **fail closed** when front matter never terminates | I have now claimed "unreachable" twice and been wrong once. Find shape three. Ideas: a BOM before the first fence; `---` with a trailing **tab**; `----`; front matter opened but the file ending mid-key; a fence inside a fenced code block in the body |
| `SB-2` CRLF | fixed **in `get_stage_status`**, which four other scripts share | Confirm I did not break `backlog`, `roadmap`, `specs-by-stage`. I ran `cost-audit` only |
| fail-closed logic | `found` and `closed` are now independent, no early exit | ⚠ **My first attempt at this fix was itself wrong** — `closed = 1` on a hit let `found` satisfy the test meant to disqualify it, and N2 still bypassed. Check the second attempt as hard as you checked the first |
| `FU-4` grandfather surface | prints when the list is not the committed default | Is a `warn` enough? It is not a gate; a CI run with an override still exits 0. Judge whether that is the right strength |
| `FU-7` copy scope | `118.32 s → 5.81 s`; copies `scripts projects .repo-context.yaml VERSION justfile app.just AGENTS.md decisions guidance` | **That list is a guess about what the gate reads.** My first attempt omitted `AGENTS.md` and the control failed for a reason unrelated to the gate — it looked exactly like a regression. What else does it read that I have not noticed, and would the red-proof notice if the copy set went stale? |
| `FU-1`, `FU-2`, `FU-3`, `FU-5`, `FU-8`, `FU-9`, `FU-10` | records corrected | Spot-check the attributions rather than the diffs — the whole finding class was **wrong citations** |

## The four new red-proof cases

N1 (trailing-space fence), N2 (no closing fence), N3 (CRLF with a real entry),
plus the existing control. Each is **verified to fail against round 1's code** —
reverting the two scanner lines produces *"N1 …: prose in the body satisfied the
gate."*

**Reproduce that**, and then ask the question the last two rounds turned on:
**does the injection reproduce a shape that can actually exist**, or one I
modelled? That question found `SB-2` in `PATCH-002` and `SB-1` here.

## Your own checks

1. **The copy set.** `FU-7` traded correctness risk for 20× speed. Break it
   deliberately — remove one file from the list — and see whether the red-proof
   fails *loudly* or just stops proving anything.
2. **`get_stage_status`'s four other callers.** Run `just backlog`, `just
   roadmap`, `just specs-by-stage` before and after. I did not.
3. **Is `FU-4`'s warning reachable in CI?** It writes to stderr via `warn`. If
   CI captures only stdout, the surface I added does not exist where it matters.
4. **`DEC-013` §5's amendment note.** I added prose to the **template's**
   namespace (AGENTS.md §10) rather than stamping its front matter. Judge that —
   it is a deliberate call and may be the wrong one.
5. **Anything from round 1 that regressed.** Re-run M1–M3, M4a/M4b, M5.

## Return Criteria

1. **Gates, run by you**, pasted, with the clippy version **and which binary
   answered** (`rustup which` — `--version` cannot say). Say which gate list.
2. **Observe CI green on the SHA you approve.**
3. **All four fence cases plus every prior mutation re-run.** Each: file changed
   **and** ran **and** *output changed*.
4. ⚠ **Mutate in a disposable clone**, and ⚠ **your rtk-wrapped git served stale
   HEAD/branch/status last round** — the orchestrator's shell has `git` unwrapped
   and both agree there, so the staleness is environment-local. Use
   `/usr/bin/git` for anything you will report.
5. Handback: real `tokens_total`, deduped by `message.id`, per-component,
   **rounded up ~20 %**. ⚠ **`notes:` on ONE PHYSICAL LINE.**
6. **Correct `handoff.to_agent`.** No `handback-sync`, no merge.
7. Findings continue this patch's sequence — `FU-1`…`FU-10` are taken, so
   **`FU-11`**.
8. Verdict: ✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- `PATCH-004` (PR #11) — the `FU-5`/`FU-6` toolchain work, verified separately.
- Merging PR #10, `handback-sync`, backfilling `STAGE-001`.
- The gate-count ambiguity and the ID-minter collision — both filed, the latter
  now at **N=3** after firing twice more during these rounds.

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
