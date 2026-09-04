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
  id: HANDOFF-028
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT. tier_map's BUILD hint is 0 for 5.
                                   #   Read your own message.model and CORRECT this.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-04
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-012

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

# HANDOFF-028: Strip location and sample unpack — two paths per DEC-008

## Delegation Summary

Build `SPEC-012`. **This is the first spec in the project that produces pixels.**
Eight have read metadata; this one turns a strip into a linear `u16` plane.

`SPEC-009` shipped yesterday and is why you can trust the tags you are about to
read: every Structure-class membership is now load-bearing, so
`require_uncompressed()` cannot be walked past by a `RATIONAL 2/2` `Compression`.

## ⚠ The failure this spec is shaped around

`SPIKE-001` decoded 14-bit bit-exact **on its first attempt**. Its unpacker took
`bits` as a parameter and every frame it ever saw was 14, so `DEC-008`'s two
cases were indistinguishable. `SPIKE-002` ran it on a 16-bit body and got a
**byte-swapped plane** — wrong in a way that:

- still decodes without error,
- still has exactly the right length, and
- **still passes the layer-0 arithmetic check.**

Only the value range caught it. That is why `AC4` (`max > WhiteLevel` as a loud
error) is not a nicety, and why `AC3` asserts the *measured impossible values*
rather than "the outputs differ".

## What has been measured for you — reproduce, do not re-derive

The spec's `## Implementation Context` carries the first eight samples of both
files, obtained **two independent ways that agree exactly**: hand-unpacked from
the raw strip bytes, and read out of `dnglab --raw-pixel`'s own plane.

**Use them as your first checkpoint.** `SPEC-013` builds the MD5 oracle; until
then a whole-plane mismatch tells you nothing about *where*. Sample 0 tells you
which path you are on:

```
Q2M 14-bit  correct: 746      wrong (as 16-bit LE): 43019
M Mono 16-bit correct: 4761   wrong (as big-endian): 39186
```

Both wrong values exceed `WhiteLevel 16383` and are impossible.

## The decision you must make and record

`8424 × 5632 × 2 = 94,887,936` bytes for the plane, on top of an 86 MB input.
`library-not-application` says the consumer picks the allocator; `DEC-002`
(**proposed**, 0.72) is unresolved on `no_std`/`alloc`.

So: `unpack_into(&mut [u16])` or `unpack() -> Vec<u16>`? The spec sets out both
and gives the orchestrator's read (**`unpack_into` as the primitive**), **offered
as input, not as the answer**. **Write the `DEC` either way**, including if you
disagree — this is an API commitment, and `library-not-application` is a blocking
constraint.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, **summed across all six
   targets**. Then **push and read CI** — `constraints.yaml` requires the gate
   *observed* green on your SHA.
2. ⚠ **Fuzz is not optional here** (§12 bar 2) — the unpacker is a new input
   surface over attacker-controlled `bits`, `width`, `height` and strip bounds.
   **A target that only ever drives 14-bit recreates `SPIKE-001`'s exact blind
   spot.** Say how you know both paths were reached.
3. **The provenance row is required** — new algorithm, class 1 (specification),
   TIFF 6.0 + `DEC-008`. ⚠ **`SPIKE-001`'s decoder is discarded and must not be
   consulted**, and no copyleft RAW implementation may be read
   (`provenance-recorded-per-algorithm`). If the algorithm seems available only
   that way, **stop and ask** — that is a decision, not a build step.
4. **`AC8` wants a measurement, not an estimate.** Peak RSS for a 47 MP decode,
   by whatever means you can defend, with the method stated.
5. Every mutation: **assert it changed the file and compiled** before concluding.
   ⚠ **Stage your work before mutate-and-revert experiments** — `SPEC-010`'s
   build lost its entire change to `git checkout --` and shipped a
   reconstruction.
6. **Branch and commit before reporting done** (`feat/spec-012-…`), and fill the
   `handback:` — a real `tokens_total` **deduped by `message.id`**, said so.
   ⚠ **You can get this yourself**: read your own transcript at
   `~/.claude/projects/<slug>/<session-id>.jsonl` and sum `usage`, keeping one
   object per distinct `message.id`. The session id is in the scratchpad path in
   your system prompt. Do **not** ask the orchestrator to run `/cost` — it is a
   client-side command *and* it measures the wrong session. Price
   `estimated_usd` per-component at the rates for the model `message.model`
   reports, not `tier_map`.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1, each with a §15 disposition. ⚠ A `spec:`
   disposition must **name an AC in that spec that would fail** if the finding
   were left undone.
9. Answer §15's reflection questions in the handback.

## Handback

*(Filled by the implementer.)*
