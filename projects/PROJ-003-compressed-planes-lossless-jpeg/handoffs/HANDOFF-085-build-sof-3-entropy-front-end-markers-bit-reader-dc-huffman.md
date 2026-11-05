---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-036` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md
#
# NOTE: numbered HANDOFF-085, NOT the auto-numbered 084 — HANDOFF-084 is held by
# the unmerged PROJ-002 STAGE-010 branch (PR #95). Created by hand to avoid the
# collision (see check-remote-state lesson); `just new-handoff` would have reused
# 084.

handoff:
  id: HANDOFF-085
  cycle: build                     # build | verify — which cycle is delegated
  from_agent: claude-opus-5        # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5          # from tier_map.build — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-036

project:
  id: PROJ-003
  stage: STAGE-009
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

# HANDOFF-085: SOF-3 entropy front-end (markers, bit reader, DC Huffman)

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-036` to `claude-opus-5`
(acting as implementer) for the **`build`** cycle: implement the single-component
SOF-3 (lossless JPEG, ITU-T T.81 Annex H) **entropy front-end** — marker/frame
parse, the `0xFF`-destuffing JPEG bit reader, DC Huffman table build, and
per-sample `DECODE`+`RECEIVE`+`EXTEND` to a signed difference — in a new
`src/ljpeg.rs`, panic-free and typed-error on every malformed input. Stop at the
difference stream; predictor reconstruction is SPEC-037.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-036-sof-3-entropy-front-end-markers-bit-reader-dc-huffman.md`
  — read **all of it**; the `## Outputs`, `## Acceptance Criteria` and
  `## Failing Tests` sections are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks and gotchas. Read it (DEC-004 rule 5).
- **The standard:** ITU-T T.81 (1992) — Annex H (lossless), Annex C (Huffman
  table generation), Annex F §F.2.2 + Figure F.16 (`RECEIVE`/`EXTEND`/`DECODE`),
  Annex B (markers, `0xFF` stuffing). Implement **from the standard** — see
  `DEC-035`.

### Decisions that apply

- `DEC-035` — SOF-3 lossless Huffman JPEG is patent-clean; implement it from
  T.81, clean-room, and add a `docs/provenance-ledger.md` row. The cross-ref
  crates (`oxideav-mjpeg`, `photocraft-raw`) are read-for-structure only, never
  copied or linked.
- `DEC-018` — the u16 sample path; prediction/accumulation is modular `2^16`
  arithmetic done with explicit `wrapping_*` (so `arithmetic_side_effects` is
  satisfied by intent). Front-end accumulation is `checked_*`, same as
  `plane::BitReader`.
- `DEC-016` — caller-owned buffers / no alloc on the decode path (the front-end
  allocates nothing per-sample; relevant mainly to SPEC-037, noted for continuity).

### Constraints that apply

Full text in `./guidance/constraints.yaml`.

- `no-panics-on-untrusted-input` — RAW is attacker-influenced; every marker,
  length, table and bit read is bounded and returns a typed `Error`, never a
  panic. This is the load-bearing invariant (SPEC-038 fuzzes it).
- `provenance-recorded-per-algorithm` — add the SOF-3 front-end row to
  `docs/provenance-ledger.md` (T.81 Annexes B/C/F/H, `DEC-035`).
- `test-before-implementation` — the spec's `## Failing Tests` are written
  first; make them pass.

### Prior related work

- `src/plane.rs` — the private `BitReader` (~lines 102–169) is the
  checked-arithmetic MSB-first pattern to **mirror** (not reuse): copy its shape
  and add `0xFF` destuffing + marker detection. Its `mod tests` is the model for
  where the new unit tests live.
- `src/lib.rs` — `pub enum Error` (~line 144), `#[non_exhaustive]`: add the new
  `Ljpeg*` variants here in the documented struct-variant style; reuse
  `Error::Truncated` for EOF.
- `src/ifd.rs` — `Compression::Jpeg == 7` (~line 485) already exists but routes
  to a clean rejection; **do not touch it** (STAGE-010 wires the container).
- Spec design PR for `SPEC-036` (this handoff's companion) — carries `DEC-035`.

## Expected Deliverables

- A new `src/ljpeg.rs` (declared `mod ljpeg;`, items `pub(crate)`) implementing
  `SPEC-036`'s `## Outputs`: `parse_headers`, `Frame`, `HuffTable`, `ScanReader`
  with `decode_difference` + `restart`, and the new `Error` variants.
- All `## Failing Tests` passing; plus any edge-case tests needed for the ACs.
- The `docs/provenance-ledger.md` row (T.81 / `DEC-035`).
- Compiles under `#![forbid(unsafe_code)]` and the five-lint deny policy;
  `just test`, `just lint-ci -D warnings`, `just fmt-check`, `just validate`
  green.
- A PR against `main` from branch `claude/spec-036-build-<suffix>`, its
  description referencing: this handoff (`HANDOFF-085`), `SPEC-036`, `STAGE-009`,
  `PROJ-003`, and the decisions used (`DEC-035`, `DEC-018`, `DEC-016`).

## Out of Scope

If any of these seem necessary, add a spec to the stage backlog — do **not**
expand this handoff:

- Predictor reconstruction, initial prediction `2^(P−Pt−1)`, applying the point
  transform, the per-restart predictor reset, and the `u16` output plane —
  **SPEC-037** (this spec stops at the difference stream and the parsed `Frame`;
  `ScanReader::restart` is here, but *calling* it per interval and resetting
  prediction is SPEC-037).
- The analytic encode→decode round-trip oracle and the fuzz target — SPEC-037 /
  SPEC-038.
- Any DNG container binding (`Compression == 7` routing, strip/tile assembly,
  byte order) — STAGE-010; do not touch `src/ifd.rs` or `src/plane.rs`'s
  unpacker.
- Multi-component / Bayer-in-LJ92 (`Nf != 1`), lossy DCT (SOF-0/2), arithmetic
  coding (SOF-9/10/11).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**
   (Claude Code: run `/cost`; if your interface reports no count, set null and
   say why in `notes` — do not invent one).
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open the PR.

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

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec.**

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any — e.g. if the Huffman-table representation or an
    error-variant consolidation warranted one)
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
