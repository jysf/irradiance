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
  id: HANDOFF-030
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ DISPATCH HINT. The BUILD hint is 0 for 6.
                                   #   Read your own message.model and CORRECT this.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-04
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-013

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

# HANDOFF-030: Bit-exact plane oracle against dnglab raw-checksum, with its red-proof

## Delegation Summary

Build `SPEC-013`. **The plane is already bit-exact — this spec makes the repo
assert it, and proves the assertion can fail.**

`SPEC-012`'s unpacker matches `dnglab analyze --raw-checksum` on all four
decodable files, verified twice independently, both times with a **throwaway
probe built outside the repo**. The digests are already pinned in
`tests/corpus/manifest.toml`. Nothing needs discovering.

**So the oracle will be green on day one, and that is the danger.** A green
oracle that cannot fail manufactures confidence — `oracle-must-be-shown-red` is
this project's founding discipline and `AC4` is the whole spec.

## ⚠⚠ Read this before you write the red-proof

The design probe injected an off-by-one into the bit cursor:

- `diff` confirmed the file changed ✅
- `cargo build` confirmed it compiled ✅
- **the plane digest was byte-identical to the honest one** ❌

`remaining.min(bits_left).max(1)` differs only when the min is zero, which never
happens. **A semantic no-op that satisfied every check this repo's rules
require.**

*"Concluding from a mutation that never applied"* is a failure measured **five
times** here, and the rule written to stop it — *assert it changed the file and
compiled* — **is not enough**. The design session followed it exactly and still
produced a false red-proof.

**Your red-proof must assert the OUTPUT changed** — control digest ≠ mutant
digest — **before** concluding anything about what the test caught. That is the
one sentence this spec exists for.

⚠ **The design probe did NOT obtain a genuine faulty digest** — two re-runs were
killed by timeouts on a 95 MB plane. No faulty number is quoted in the spec
because none was measured. **Producing it is your job.**

## What is settled, so you do not re-derive it

- **The four honest digests** are in the manifest and confirmed 4/4, twice.
- **MD5 must be implemented, not depended on.** `tests/support/corpus.rs` already
  hand-writes SHA-256 from FIPS 180-4 — dev-only, class 1, proven against the
  published NIST vectors, `DEC-010` recording why it is not a dependency. RFC
  1321 is the same shape and ships its own vector suite. **No new dependency**;
  if you conclude otherwise, **stop and ask**.
- **Do not shell out for MD5.** `md5`/`md5sum` exist on both hosts, but the
  tier-A half is the only half CI runs (`DEC-003`), and a CI half that depends on
  an external binary is one `PATH` change from silent — `SPEC-005/FU-3` and
  `SPEC-012` both measured exactly that.

## The thing that will matter to the next spec

`AC3`. MD5 says *different*, never *where*. `SPEC-014` will debug a
47-megapixel plane against this oracle, so a failure must name the **first
differing sample index and both values**. `docs/oracle-contract.md` documents
the reference route: `--raw-pixel | tail -c +20 | dd conv=swab`.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Push and read CI** — `constraints.yaml` requires the gate
   *observed* green on your SHA.
2. **Watch the red-proof fail yourself**, and paste the **two digests** that
   prove the fault was real.
3. **Provenance row required** — MD5, class 1, RFC 1321. Written from the
   published standard, not from any implementation.
4. Every mutation: file changed **and** compiled **and** output changed. Stage
   your work before mutate-and-revert — `SPEC-010`'s build lost its entire
   change to `git checkout --`.
5. ⚠ **Peak RSS is ~182 MB per decode** (`DEC-016`, amended). Four files in one
   test run is a real consideration; say what you did about it.
6. **Branch and commit before reporting done** (`feat/spec-013-…`), and fill the
   `handback:` with a real `tokens_total` **deduped by `message.id`** — read your
   own transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`; the session
   id is in the scratchpad path in your system prompt. Price per-component at the
   rates for the model `message.model` reports, **not** a flat rate — a reviewer
   who used a flat ceiling last week was 6.2× high and said so themselves.
   ⚠ **Do not hand-write the cost session into the spec** — fill the handback
   block and leave `cost.sessions` alone, so `handback-sync` can run once
   cleanly. Hand-writing it is what has forced four separate duplicate-entry
   cleanups.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Answer §15's reflection questions in the handback.

## Handback

*(Filled by the implementer.)*
