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
  id: HANDOFF-013
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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

# HANDOFF-013: <Task Title — same as the spec's title>

## Delegation Summary

Second build cycle on `SPEC-003` — the punch-list round. Verify returned
⚠ PUNCH LIST with **one ship-blocker**, and it is **documentation and config only:
no `src/` change**. The reader itself was found sound.

## Context the Receiving Agent Needs

### 🚫 SB-1 — the licence record is wrong on a blocking constraint

All three parts **independently reproduced by the orchestrator**:

1. **`libfuzzer-sys` declares `(MIT OR Apache-2.0) AND NCSA`** — verified in its
   own `Cargo.toml`. `AND` is **conjunctive**. `DEC-011:81` records it as
   `MIT OR Apache-2.0`, which is wrong, and `DEC-011:85`'s claim that "no
   exception entry was needed" is therefore false.
2. **NCSA is not in `deny.toml`'s allow list** (MIT, Apache-2.0, Apache-2.0 WITH
   LLVM-exception, BSD-2/3-Clause, Zlib, 0BSD, Unicode-3.0).
3. **The premise everyone accepted is false.**
   `cargo deny --manifest-path fuzz/Cargo.toml check licenses` **runs** — and
   **FAILS** today. The gate was never absent; it was never invoked.

Substance is fine: NCSA is permissive and nothing copyleft is linked. **The record
is wrong**, on `no-copyleft-dependencies`, in the document standing in for a gate
that turns out to exist. That is why it blocks.

Also missing from `DEC-011:42`'s table: **`cfg-if`, `getrandom`, and `r-efi 6.0.0`**
— the last being `MIT OR Apache-2.0 OR LGPL-2.1-or-later`, the only crate in the
graph that mentions LGPL at all. Disjunctive, so permissive is selectable and
nothing is wrong — but an unrecorded LGPL mention in *this* repo is precisely what
the ledger exists to surface.

### The fix

- `DEC-011` — correct the licence table; add the three missing crates; retract the
  "no exception needed" claim.
- `deny.toml` — allow `NCSA` (or a targeted per-crate exception for
  `libfuzzer-sys`; pick one and say why).
- `fuzz/Cargo.toml` — **currently has no `license` field at all.** Add one.
- `guidance/constraints.yaml:45` — the `enforcement:` field is now inaccurate.
- **Wire the fuzz licence check as a real gate**, since it works. That converts a
  hand-check into a mechanism, which is the whole point.

### Three factual corrections (two are the orchestrator's errors)

1. **"three JPEG-compressed" is wrong.** Measured: `M2462362.DNG` and `K3III.DNG`
   are compression **7** (JPEG); `K3III.PEF` is **65535** (vendor-private). The
   spec and `HANDOFF-012` both say three. `CHANGELOG.md:34` already says it
   correctly — fix the spec to match.
2. **`CHANGELOG.md:31` conflates byte order with container:** "5 `II` / 1 `MM` /
   1 PEF". The PEF is **`II` too** — it is 6 `II` / 1 `MM` across 7 files.
3. **`docs/conformance-matrix.md` has no row for the Leica M Monochrom
   (Typ 246)**, against that file's own opening rule that every camera gets a row
   the day it is known. Three bodies now read end-to-end; make the matrix say so.

### Also in scope

- **A third `+toolchain` trap:** bare `cargo +1.90.0` fails with `no such command`,
  and MSRV is **the only gate with no `just` recipe**. Add one, and record the trap
  in `guidance/toolchain-brief.md`.
- **`array()` tolerates malformed tags while `SubIFDs` via `uints()` is fatal to
  the whole container**, with no stated rule. Either state the rule or make them
  consistent — a reader that survives one malformed tag and dies on another needs
  to say which is which on purpose.

## Expected Deliverables

1. SB-1 closed: `DEC-011`, `deny.toml`, `fuzz/Cargo.toml`,
   `constraints.yaml:45`, **and a working licence gate over `fuzz/`**. Paste the
   gate passing.
2. The three factual corrections above.
3. A `just msrv` recipe; the third `+toolchain` trap in the toolchain brief.
4. The malformed-tag rule stated (or the behaviour made consistent).
5. All nine gates green **plus** the new fuzz-licence gate — output pasted.
6. ⚠ **No `src/` change is expected.** If you find yourself editing the reader,
   stop and say why in the handback — verify found it sound.

## Out of Scope

- The reader's logic. Verify approved it; this round is records and config.
- Pixel decode / unpack — STAGE-002, `DEC-008`.
- The multi-strip corpus gap (follow-up, recorded).

## Return Criteria — how to hand back

1. Paste all gates including the new fuzz-licence one.
2. Fill `## Completion` and `handback:`. ⚠ `tokens_total`: deduplicate by
   `message.id` and **say so**, with cache-read share. Note: measured inflation
   factors so far are **1.61× / 1.82× / 1.86× / 1.95× / 2.25×** — it is **not** a
   constant, so no fixed correction may be applied to a raw figure.
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
