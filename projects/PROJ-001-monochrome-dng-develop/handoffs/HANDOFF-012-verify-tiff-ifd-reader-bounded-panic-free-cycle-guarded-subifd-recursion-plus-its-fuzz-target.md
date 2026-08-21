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
  id: HANDOFF-012
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-003

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

# HANDOFF-012: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-003` for the **verify** cycle, at
`d867403` (implementation `b79c7ef`). Independent session.

This is the first spec that parses attacker-influenced binary input, so the
panic-free constraint is now load-bearing rather than aspirational.

## Context the Receiving Agent Needs

### Already reconciled — don't just repeat

- **All nine gates green**, run by the orchestrator. 48 tests (31 lib + 9 corpus +
  8 ifd_reader). No `#[allow]` of any policy lint anywhere in `src/`. No fuzz
  artifacts left behind.
- **Criterion 5 verified independently, with a harder fault than the build's.**
  The build's planted fault was an unchecked index — but the lint policy *catches
  that at compile time* (`indexing_slicing`, `src/ifd.rs:704`, level from
  `src/lib.rs:48`), so it never reaches the fuzzer on a clean tree. The
  orchestrator instead planted a **lint-clean** `split_at(end)` that clippy passes,
  and libFuzzer found it: `deadly signal`, crash artifact written. So the fuzz
  target genuinely works on faults the lint policy cannot see — which is precisely
  the gap it exists to cover.

### ⚠ Two facts in MY spec were wrong; the build found both

Neither changed the design, but the record was wrong and a verifier would expect
different numbers:

1. **Byte order: SIX `II`, ONE `MM`** — not "two big-endian" as
   `HANDOFF-011`/the spec said. Confirmed on raw header bytes across all 7 files.
   Only `M2462362.DNG` is `MM`.
2. **`K3III.PEF` has NO SubIFD at all** — zero `SubIFD` mentions, no
   `NewSubfileType` tag, plane in `IFD0`, and it is the only file with a real IFD
   *chain* (`IFD0→IFD1→IFD2`). Confirmed with exiftool. So TIFF's **absent-means-0**
   default for `NewSubfileType` is load-bearing, not decorative — worth checking
   that the reader relies on it deliberately rather than by luck.

**Both durable docs still carry my wrong numbers** (`docs/conformance-matrix.md`
and the spec). The orchestrator corrects them at ship; flag it if you see the
error propagated anywhere else.

### What deserves scrutiny

1. **The guards.** Depth limit and cycle detection are the difference between a
   hostile file and an infinite loop. Are they on *every* recursion path, including
   the IFD *chain* (`next` pointers), not just SubIFD descent? The PEF is the only
   file with a real chain, so chain-walking has exactly one real-world test.
2. **`Error::UnsupportedCompression` on the three JPEG files** — they must be
   rejected but stay **tag-readable**. Is the boundary right?
3. **`DEC-011`** puts `libfuzzer-sys` in a separate `fuzz/` crate so
   `[dependencies]` stays empty. ⚠ The build discloses that **`cargo deny` does
   not reach `fuzz/`** — hand-checked only. Is that acceptable, or does the licence
   gate now have a hole?
4. **Disclosed follow-up: no corpus file exercises a multi-strip plane.** All four
   uncompressed planes are single-strip. Real gap or acceptable for STAGE-001?
5. `packed_bits()` returns *bits* rather than bytes, deliberately, so
   `DEC-008`'s remainder decision stays in STAGE-002. Right call?

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the fuzz red-proof yourself (check #9). ⚠ Two traps:
- `cargo fuzz` needs the rustup shim **first on PATH** —
  `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ifd`
  (see the toolchain brief's "second `+toolchain` trap").
- **An unchecked index will not reach the fuzzer** — the lint policy rejects it at
  compile time. Plant something lint-clean (`split_at`, `unreachable!`, `assert!`)
  or you will be testing clippy, not the fuzzer.

**Label every finding ship-blocking or follow-up.** A panic reachable from
attacker-controlled bytes is ship-blocking. A missing corpus shape is a follow-up.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- Pixel decode / unpack — STAGE-002 and `DEC-008`.
- Re-opening the `-F` gate, `DEC-009`, or the corpus manifest design.

## Return Criteria — how to hand back

1. Paste the gates and both fuzz directions you ran yourself.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: transcript sums
   **double-count ~1.9x** — deduplicate by `message.id` and **say so**, with
   cache-read share.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-003-ifd-reader`; do not merge.

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
