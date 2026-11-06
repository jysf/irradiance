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
  id: HANDOFF-091
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-043

project:
  id: PROJ-003
  stage: STAGE-011
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
  status: pending                  # completed | blocked | rejected
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-091: Compression-7 recognition + strip-path routing (build)

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-043` to `claude-opus-5` (acting
as implementer) for the **`build`** cycle: route a DNG `Compression 7` (SOF-3
lossless JPEG) sensor plane through the shipped `ljpeg::decode_into` by
dispatching on `sensor.compression` at the top of `plane::unpack_into`, so a
Compression-7 file decodes to the `u16` plane instead of returning
`UnsupportedCompression` — the uncompressed path **byte-for-byte unchanged**, the
binding **panic-free**. First spec of `STAGE-011`; makes the STAGE-009 decoder
reachable on real files.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-043-compression-7-recognition-strip-path-routing.md` — read all of it; `## Inputs` / `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` / `## Notes for the Implementer` are the contract, and `## Inputs` carries the §12 probe of both held files (both single-strip).
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the test framework, lint
  quirks, runtime globals and gotchas; and the corpus env
  (`IRRADIANCE_CORPUS_DIR`) — though this spec is **tier-A only** (no corpus; the
  real-file oracle is SPEC-044).

### Decisions that apply

- `DEC-016` — caller-owned, no-alloc buffer. `unpack_into` keeps that shape; the
  Compression-7 route writes into the caller's `dst`, allocating nothing.
- `DEC-012` — strict-on-structure / tolerant-on-shape; the `Compression` dispatch
  is a structural gate decision (reject the unknown code cleanly).
- `DEC-036` — `ljpeg` visibility: `decode_into` stays `pub(crate)`. **Do NOT
  widen it** and do NOT retire the `fuzzing` feature / `__fuzz_ljpeg` hook — that
  public-surface decision is SPEC-045's.
- `DEC-003` — own-work fixtures: the SOF-3 strip fixtures are hand-built via the
  STAGE-009 encoder (`encode_lj92` / `encode_scan` in `src/ljpeg.rs mod tests`),
  never a committed corpus file.

### Constraints that apply

- `no-panics-on-untrusted-input` — a Compression-7 strip is attacker-influenced;
  every malformed / truncated / dimension-mismatched strip returns a typed
  `Error`, never a panic (inherited from `decode_into`, preserved by the binding).
- `no-new-top-level-deps-without-decision` — none needed; the decoder already
  ships.

### Prior related work

- `SPEC-037` / `SPEC-038` (shipped) — the decoder in `src/ljpeg.rs`:
  `pub(crate) fn decode_into(data: &[u8], out: &mut [u16]) -> Result<Frame, Error>`
  (≈ line 992) and the `Frame` struct (≈ 182: `precision`, `lines`,
  `samples_per_line`, `point_transform`, `predictor`, `restart_interval`).
  `encode_lj92` / `encode_scan` in `mod tests` are the fixture shape to reuse.
- `SPEC-028` (shipped) — the multi-strip, independent-per-strip assembly in
  `plane::unpack_into` this routing reuses (row-band slicing; each strip decoded
  from a fresh cursor).
- `SPEC-024` / `SPEC-029` (shipped) — the `Compression` enum and
  `require_uncompressed` gate this spec turns into a dispatch.

### The build contract

- Implement SPEC-043's `## Outputs` and make every entry in its `## Failing
  Tests` pass (tier-A, hand-built SOF-3 strips).
- Keep the `Compression::Uncompressed` arm byte-for-byte identical; add the
  `Compression::Jpeg` arm (per-strip `decode_into` into the row band, skipping the
  layer-0 byte-count check and the sub-byte unpack); keep `other =>` returning
  `UnsupportedCompression`.
- `decode_into` stays `pub(crate)`; `unpack_into`'s signature is unchanged.
- `just test`, `just lint-ci`, `just fmt-check`, `just validate` all green;
  `#![forbid(unsafe_code)]` + the five-lint deny policy hold.
- Open a PR against `main` from branch `claude/spec-043-build-<suffix>`, its
  description referencing `HANDOFF-091`, `SPEC-043`, `STAGE-011`, `PROJ-003`, and
  the decisions above (plus any new `DEC-*` — none expected).

## Out of Scope

- The bit-exact `dnglab --raw-checksum` corpus oracle on the two real files —
  **SPEC-044**.
- The provenance-ledger entry, the macOS/Linux determinism assertion, and the
  container→decoder hardening fuzz — **SPEC-045**.
- Any change to the SOF-3 decoder algorithm — **STAGE-009** (shipped); only feed
  it bytes.
- Multi-component / Bayer-in-LJ92 interleave, OpcodeList2 GainMap, tiled
  Compression-7 — deferred (no held file).
- Widening `ljpeg`'s public API or retiring the `fuzzing` hook — **SPEC-045 /
  DEC-036**.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`** (or
   null-with-note if your interface genuinely exposes none, as the Claude Code
   desktop Code tab does — then report the `ccd_session_mgmt get_usage` occupancy
   as a FLOOR and say so; the orchestrator supersedes it from the transcript at
   ship).
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open the PR.

If you cannot complete the task:
1. Fill `## Handback` with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage.**
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** <fill>
- **Completed at:** <YYYY-MM-DD>
- **All acceptance criteria met?** <yes/no — walk each of SPEC-043's six>

### Cost self-report

- **Tokens (total):** <real `/cost` cumulative, or a FLOOR with a note>
- **Estimated USD:** <or null>
- **Duration (minutes):** <n>
- **Source of the number:** <interface>

### Drift and new artifacts

- **New decisions emitted:** <none expected>
- **Deviations from spec:** <...>
- **Follow-up work identified:** <SPEC-044 is next (the corpus oracle); note anything surprising about the real-file strip layout if you probed it>

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>
