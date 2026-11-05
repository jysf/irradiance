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
  id: HANDOFF-087
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-037

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

# HANDOFF-087: Lossless reconstruction (predictors, point transform, restart)

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-037` to `claude-opus-5` (acting
as implementer) for the **`build`** cycle: implement the lossless-JPEG
reconstruction on top of SPEC-036's shipped entropy front-end — predictors 1–7,
the T.81 §H.2.1 boundary rules, the point transform, and restart resets, turning
the decoded differences into a caller-owned `u16` plane — plus the in-harness LJ92
encoder that drives the bit-exact round-trip oracle and its red-proof.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-037-lossless-reconstruction-predictors-point-transform-restart.md` — read all of it; `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-035` — SOF-3 is patent-clean; implement the reconstruction clean-room from
  T.81 Annex H and add the provenance-ledger row. Cross-ref crates read-for-structure only.
- `DEC-016` — caller-owned output buffer; no allocation on the reconstruction path.
- `DEC-018` — u16 sample path; prediction is modular `2^16`, done with explicit
  `wrapping_*` (so `arithmetic_side_effects` is satisfied by intent).

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — every neighbour / restart / length path returns
  a typed `Error`, never a panic (SPEC-038 fuzzes it).
- `oracle-must-be-shown-red` — the encode→decode round-trip must include a
  perturbed-decoder red-proof that provably goes RED, not just green round-trips.
- `provenance-recorded-per-algorithm` — add a `docs/provenance-ledger.md` row for
  the lossless reconstruction (T.81 Annex H.1).
- `test-before-implementation` — the spec's `## Failing Tests` come first.

### Prior related work

- `SPEC-036` / `HANDOFF-085` / **PR #98** — the shipped entropy front-end this
  builds on: `src/ljpeg.rs` `Frame`, `HuffTable`, `parse_headers -> (Frame,
  HuffTable, usize)`, `ScanReader::decode_difference(&HuffTable) -> i32`,
  `ScanReader::restart()`. Use these EXACT names; do not re-decode.
- `src/plane.rs` — the `u16` plane is row-major, one sample/pixel (match it).
  `src/lib.rs` — the `Ljpeg*` and `*WrongLength` error conventions; add
  `LjpegPlaneWrongLength`.
- SPEC-030 / SPEC-031 (demosaic) — the analytic-oracle + perturbed-kernel red-proof
  pattern to mirror for the round-trip oracle.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `src/ljpeg.rs`: `reconstruct(frame, table, reader, out)` + `decode_into(data,
  out) -> Result<Frame>` implementing SPEC-037's `## Outputs` (predictors 1–7,
  §H.2.1 boundaries, point transform, restart reset), plus the `encode_lj92` test
  harness, the bit-exact round-trip oracle, and the perturbed-decoder red-proof.
- `src/lib.rs`: the new `LjpegPlaneWrongLength` variant. `docs/provenance-ledger.md`:
  the reconstruction row (T.81 Annex H.1).
- All of SPEC-037's `## Failing Tests` passing; `just test`, `just lint-ci -D
  warnings`, `just fmt-check`, `just validate` green; `#![forbid(unsafe_code)]` +
  the five-lint deny policy intact.
- A PR against `main` from branch `claude/spec-037-build-<suffix>`, its
  description referencing this handoff (`HANDOFF-087`), `SPEC-037`, `STAGE-009`,
  `PROJ-003`, and `DEC-035` / `DEC-016` / `DEC-018` (plus any new `DEC-*`).

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

- The marker/frame parse, bit reader, and DC Huffman decode — that is SPEC-036
  (shipped); **call** it, don't re-implement it.
- The fuzz / truncation target — SPEC-038 (the `encode_lj92` harness seeds it, but
  the libFuzzer target itself is SPEC-038).
- The DNG `Compression 7` container binding + real-file `dnglab --raw-checksum`
  verification — STAGE-010; do not touch `src/ifd.rs` or `plane.rs`'s unpacker.
- Multi-component / Bayer-in-LJ92, lossy DCT, arithmetic coding, Nikon
  NEF-lossless, Pentax PEF.

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
