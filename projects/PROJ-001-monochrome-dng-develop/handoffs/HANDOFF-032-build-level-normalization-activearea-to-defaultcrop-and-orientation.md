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
  id: HANDOFF-032
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT. The BUILD hint is 0 FOR 7 while the
                                   #   verify hint is 2 for 2. Read your own message.model
                                   #   and CORRECT this before handing back.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-014

project:
  id: PROJ-001
  stage: STAGE-002
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

# HANDOFF-032: Level normalization, ActiveArea to DefaultCrop, and orientation

## Delegation Summary

Build `SPEC-014`. `SPEC-012` produces a correct uncropped plane and `SPEC-013`
asserts it bit-for-bit; this spec turns it into an image — black subtracted,
white normalized, three-stage crop, orientation applied.

## ⚠ Two things that make this spec different from the last three

**1. This spec has NO ORACLE, and cannot have one.** `SPEC-013`'s
`--raw-checksum` attaches to the **uncropped, un-normalised** plane by contract,
so nothing you write here is covered by it. And `DEC-004` already settled that a
comparison oracle never will cover it: `SPIKE-001` measured the plane checksum is
**structurally blind** to a levels error, and the develop oracle misses one up to
**+256 (50 %)**. `SPEC-015` is the analytic oracle; until it lands, **your tests
are the only check that exists**. Write them accordingly.

**2. The corpus cannot see the thing most likely to be wrong.** On every
decodable file `ActiveArea`'s origin is `(0,0)` or absent. The only file with a
non-zero origin — `K3III.DNG`, `top 34, left 26` — is JPEG and undecodable.

So an implementation that **ignores the `ActiveArea` origin entirely** passes
every corpus test in this repo.

That is `SPIKE-001`'s shape — *"the parameter was always 14"* — and `SPIKE-002`
is the precedent for what it costs: a different camera body revealed a
byte-swapped plane that decoded, sized, and layer-0-checked correctly.
**`AC4`'s hand-built fixture with a non-zero ActiveArea origin is the only thing
in this spec that can observe the distinction. It is not optional.**

Independent evidence for which reading is right, so you do not have to guess:
`dnglab` reports `cropArea.p` **sensor-absolute** — on `K3III.DNG`,
`(26,34) + (28,24) = (54,58)`, exactly what it prints — while `exiftool` reports
the file's own `28 24`. Two tools, two conventions, and the arithmetic between
them settles what DNG means.

## What is already measured — in the spec, reproduce rather than re-derive

The full geometry table for all four decodable files, both crops shown to fit,
and the levels edges: **both** real files contain samples **below** `BlackLevel`
(min 2 and 108) and **both** reach `WhiteLevel` **exactly**. So `AC2`'s
out-of-range handling is not a hypothetical — it fires on the first file.

## The decision you must record

What is the normalized output — `u16` rescaled in place, or `f32` in `[0,1]`?
The spec argues both and gives the orchestrator's read (`u16`, consistent with
`DEC-016`'s no-allocation shape, since `f32` is **190 MB** on top of `SPEC-012`'s
measured 182 MB peak and `DEC-002` is still `proposed`). **Offered as input, not
as the answer — write the `DEC` either way.** Constraint: `SPEC-015` will assert
`BlackLevel → 0` and `WhiteLevel → 1`, so your representation must make that
expressible.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Push and read CI** — the gate must be *observed* green on your SHA.
2. ⚠ **`SPEC-013`'s oracle must keep passing untouched.** It attaches before your
   transform; if it moves, you have changed something you should not have.
3. **Fuzz** — geometry is a new input surface over attacker-controlled crop
   origin, crop size, ActiveArea and orientation. §12 bar 2.
4. Every mutation: file changed **and** compiled **and** *output changed*. That
   third clause has caught three false red-proofs in two specs; the most recent
   was a fault that compiled and returned `Error::Truncated` instead of a wrong
   digest.
5. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build lost its
   entire change to `git checkout --` and shipped a reconstruction.
6. **Branch and commit before reporting done** (`feat/spec-014-…`). Fill the
   `handback:` with a real `tokens_total` **deduped by `message.id`** — read your
   own transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`, the session
   id is in the scratchpad path in your system prompt. Price **per-component** at
   the rates for the model `message.model` reports, never a flat rate.
   ⚠ **Do not hand-write `cost.sessions`** — fill the handback block only, so
   `handback-sync` runs once cleanly. Hand-writing it has caused four
   duplicate-entry cleanups.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Answer §15's reflection questions in the handback.

## Handback

*(Filled by the implementer.)*
