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
  id: HANDOFF-024
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT from tier_map.build, NOT a record.
                                   #   tier_map is 1 for 4 (SPEC-007/FU-6). Read your own
                                   #   message.model and CORRECT this before handing back.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-30
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-010

project:
  id: PROJ-001
  stage: STAGE-005
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

# HANDOFF-024: Distinguish an unparseable tool reading from an absent tag

## Delegation Summary

Build `SPEC-010` — the first spec of `STAGE-005`. The metadata oracle
`SPEC-005` shipped **cannot tell an absent tag from an unreadable one**: both
collapse to `None`, so a garbled tool reading silently *agrees* with a `None` on
our side.

**Everything is under `tests/`. Nothing may touch `src/`.** If you believe a
`src/` change is needed, hand that back as a finding.

**This is unusual: the fix has already been built and measured.** `SPEC-005`'s
verify round 2 (`FU-8`) implemented the tri-state, ran three configurations, and
recorded which one works. That table is in the spec's `## Implementation
Context`. **Reproduce it; do not re-derive it.** Your job is to build it
properly, with the tests and the red-proof — not to rediscover the design.

## Context the Receiving Agent Needs

**Read, in order:** `SPEC-010` in full (its `## Implementation Context` is a
measured probe); `AGENTS.md` **§16** — three rules codified three days ago and
all three bear on this spec; `AGENTS.md` §12 and §15; `guidance/constraints.yaml`;
`decisions/DEC-012` and `decisions/DEC-013` (**`rejected`** — read *why*, it is
this spec's prehistory and `AC7` asks you to decide whether it needs a true
successor).

**Corpus:** `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does not exist on this host. Seven files, none committed.

⚠ **`dnglab` is LGPL-2.1 and is RUN, never linked.** Never add `rawler`,
`rawloader` or any RAW crate, including as a dev-dependency, and do not read
dnglab's source.

## What has already been measured — verify, then build on it

| fact | measured |
|---|---|
| absent == garbled for all four multi-valued tags | orchestrator, 2026-08-22, probe tests added and file restored byte-identical |
| `BlackLevel = [512, 999]` → `Some(512)` | same probe |
| tri-state **with** the `malformed_tags` comparison → 21 green | `SPEC-005/FU-8`, verify round 2 |
| tri-state **without** it → red | same |
| a *partial* fix (one-element → `Some([a,a])`) → red on `K3III.DNG` | same |

The second and third rows are `AC6`'s red-proof and its control — the same code
with one comparison removed. It costs nothing to build; **run it and watch it
fail yourself.**

## The judgement call this spec contains

`AC7`. `diff()`'s doc comment currently argues that removing `DEC-013`'s guard
was right *because* fixing this defect would trip an alarm — and `FU-8` measured
that under the real fix it does **not**, because your `malformed_tags`
comparison *is* the generic guard, on the side that holds the information.

So after your change that doc comment is wrong, and `DEC-013`'s rejected
*conclusion* may deserve a successor decision that is finally true. **Decide it
and write it, or say plainly why not.** Do not leave the doc comment reasoning
about a future you have just made the present.

## Return Criteria

1. **Ten gates + `just lint-ci` + `just oracle-meta`**, run by you and pasted.
   **Sum across all six targets** — a zero-match `cargo test <name>` exits 0.
2. ⚠ **`just lint-ci` is not optional and is not `just lint`.** Local clippy is
   0.1.97; CI floats and is 0.1.98. `PATCH-001` found a blocking constraint's
   gate dark for 17 consecutive runs because nobody ran CI's clippy locally.
3. **Push and READ CI.** `constraints.yaml` now requires the gate to be
   **observed** green on your SHA, not asserted from your laptop.
4. **Both red-proof directions watched by you**, each with its control, each
   mutation **asserted applied and compiled** before any conclusion, tree
   restored byte-identical after.
5. **Confirm each of the eight named tests exists** via per-target `-- --list`.
6. **Fuzz** — `tests/` gains a lane; seeds unchanged is a fine result, say so.
7. Fill the `handback:` with a real `tokens_total` **deduped by `message.id`**,
   and say you deduped. Read your own transcript at
   `~/.claude/projects/<slug>/<session-id>.jsonl` — the session id is in the
   scratchpad path in your system prompt. **You can get this number**; a previous
   build asked the orchestrator to run `/cost` and that was doubly wrong (it is a
   client-side command, and it measures the wrong session). Compute
   `estimated_usd` **per-component at the rates for the model that actually ran**
   — read `message.model`. Capture as late as you can: the floor convention
   measured ~17% low.
8. ⚠ **Branch and commit before reporting done.** `feat/spec-010-…`. Filling the
   handback and committing are part of *doing* the cycle, not of reporting it —
   a previous build reported done with 1,600 lines unversioned on `main`.
9. Do **not** run `just handback-sync`.
10. Findings as `SB-N` / `FU-N` from 1, each with which of §15's four
    dispositions you think it wants.

## Handback

*(Filled by the implementer. Mirror the `handback:` front-matter, and answer
§15's reflection questions — the last build left none, and verify could not run
check 6 at all.)*
