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
  id: HANDOFF-110
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-053

project:
  id: PROJ-002
  stage: STAGE-010
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

# HANDOFF-110: Compression-7 decoded-frame shape binding (SPEC-053, group B)

## Delegation Summary

The architect hands `SPEC-053` to the implementer for the **build** cycle: bind the
decoded SOF-3 frame's effective shape to the strip's declared geometry (not just its
area) and reject component sampling factors other than `H=V=1`, so a Compression-7
DNG that would otherwise decode to a silently mis-strided image returns a typed
error instead (review finding #2 + the silent-garble signal). The whole decision is
**DEC-047**. **Read SPEC-053 in full and DEC-047 first; `## Design` has the exact
sites (`unpack_jpeg` pre-decode check; `parse_frame_header` H/V check).**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-053-compression-seven-decoded-frame-shape-binding.md` — all of it, especially `## Design` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-047-bind-the-decoded-sof3-frame-shape-to-the-strip.md` — the rule, the two variants, the rejected alternative, and why `DEVELOP_VERSION` does not move.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #2).
- **Code:** `src/plane.rs` (`unpack_jpeg` l.487, the per-strip loop + `decode_into` call l.582), `src/ljpeg.rs` (`parse_headers` l.341, `parse_frame_header` l.436 with the `_hv` read at l.470, `Frame` l.200, the area checks l.993/l.1182), `src/lib.rs` (`Error` l.217), `tests/support/tiff.rs` (`lj92_strip`, `compression7_dng_with_strip`, `compression7_entries`), `fuzz/fuzz_targets/ljpeg.rs`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap (DEC-004 rule 5).

> ⚠ The spec's line numbers were read 2026-11-10 against the current tree;
> re-confirm before editing in case of drift.

### Decisions that apply

- `DEC-047` — the shape-binding + sampling-factor rejection this spec builds.
- `DEC-037` — subsampled `H/V != 1` is out of scope (this rejects it, does not implement it); the `Nf=2` column de-interleave is `2 × samples_per_line` wide.
- `DEC-045` — the re-plan; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus is never committed; the bit-exact corpus check is a local run only.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-panics-on-untrusted-input` — five deny lints; checked arithmetic for the `samples_per_line × components` product; the guards run on hostile SOF-3 bytes.
- `oracle-must-be-shown-red` — two red-proofs (shape, sampling), shown red before green.
- `library-not-application` — zero I/O; caller-owned buffers; no signature changes.

### Prior related work

- `SPEC-043`/`SPEC-046` (`STAGE-011`) — the Compression-7 routing and the two-component (`Nf=2`) de-interleave decode this binds a shape check onto.
- `SPEC-052` (`HANDOFF-109`, group A) — the sibling parser-resource guard; same "plausible-wrong-`Ok` at the boundary" failure class.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-053's Acceptance Criteria (the DEC-047 guards).
- All failing tests in SPEC-053 passing, including the two red-proofs shown red first.
- The `fuzz/fuzz_targets/ljpeg.rs` extension; run `just fuzz` (target `ljpeg`) 60 s clean.
- CHANGELOG `[Unreleased]` entry and a `docs/provenance-ledger.md` row.
- A PR against `main` from branch `claude/spec-053-build-<suffix>`.
- PR description referencing: HANDOFF-110, SPEC-053, STAGE-010, PROJ-002, and
  DEC-045 / DEC-047.
- The tier-B corpus run result (both held Compression-7 files still decode bit-exact)
  recorded in the handback.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- Supporting subsampled (`H/V != 1`) frames or any new SOF process — this rejects them.
- Any change to a correctly-shaped `H=V=1` decode's output, or a `DEVELOP_VERSION` bump.
- Changing `decode_into` / `parse_headers` / `unpack_into` signatures (the shape check
  is a pre-decode call in `unpack_jpeg`; see DEC-047's rejected alternative).
- The restart-interval (`DRI`) MCU-reset hardening — a separate item.
- The parser resource guard (SPEC-052) and the licence/package work (SPEC-055).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

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
