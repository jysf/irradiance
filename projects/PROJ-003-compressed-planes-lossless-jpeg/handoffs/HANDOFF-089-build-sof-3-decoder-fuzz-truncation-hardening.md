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
  id: HANDOFF-089
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-05
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-038

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

# HANDOFF-089: SOF-3 decoder fuzz + truncation hardening

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-038` to `claude-opus-5` (acting
as implementer) for the **`build`** cycle: add a libFuzzer target over the shipped
single-component SOF-3 decoder — reaching it through a `fuzzing` cargo feature
(`DEC-036`, NOT `cfg(fuzzing)`), with own-work SOF-3 seeds and an in-crate
panic-free replay test — proving the decoder returns typed errors, never panics,
on hostile / truncated / oversized-table bitstreams.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-038-sof-3-decoder-fuzz-truncation-hardening.md` — read all of it; `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` are the contract.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-036` — the exposure decision: a `fuzzing` cargo feature (NOT `cfg(fuzzing)`),
  because the `clippy — fuzz graph` gate builds `fuzz/Cargo.toml` with plain
  `cargo clippy` (no `--cfg fuzzing`). Read it before wiring visibility; it names
  the exact mechanism (`#[cfg(feature = "fuzzing")] #[doc(hidden)] pub fn`).
- `DEC-035` — the SOF-3 decoder is clean-room / patent-clean; fuzzing it changes
  nothing about that (no new algorithm).

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — the point of this spec: every hostile /
  truncated / oversized-table bitstream returns a typed `Error`, never panics.
- `no-new-top-level-deps-without-decision` — the `fuzzing` feature is empty (no
  dep); `libfuzzer-sys` is the existing fuzz dev-dependency. Recorded in `DEC-036`.

### Prior related work

- `SPEC-036` + `SPEC-037` (shipped) — the complete decoder in `src/ljpeg.rs`:
  `pub(crate) fn decode_into(&[u8], &mut [u16]) -> Result<Frame, Error>` is what
  the fuzz entry + the replay test drive; `encode_lj92` / `encode_scan` (in `mod
  tests`) are the shape the SOF-3 seeds mirror.
- `fuzz/fuzz_targets/plane.rs` — the analog target (the `MAX_PIXELS` bound);
  `examples/fuzz-seeds.rs` — the committed seed writer (`#[path =
  "../tests/support/*"]` + local builders like `plane_fixture`);
  `.github/workflows/ci.yml` `clippy-fuzz` / `licenses-fuzz` — the gates that
  cover a new target; `tests/opcode.rs` — the seed-replay pattern (here an
  in-crate unit test, since `decode_into` is `pub(crate)`).
- `SPEC-032` (demosaic fuzz) — the prior fuzz-target precedent.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `Cargo.toml` `[features] fuzzing = []`; `src/lib.rs`
  `#[cfg(feature = "fuzzing")] #[doc(hidden)] pub fn __fuzz_ljpeg(&[u8])` (bounds
  dims, calls `decode_into`, drops the `Result`); `fuzz/Cargo.toml` (irradiance
  `features = ["fuzzing"]` + the `[[bin]] ljpeg`); `fuzz/fuzz_targets/ljpeg.rs`;
  `examples/fuzz-seeds.rs` `ljpeg_seeds()` → committed `fuzz/seeds/ljpeg/`; the
  panic-free replay + hostile-input unit tests in `src/ljpeg.rs`.
- All of SPEC-038's `## Failing Tests` passing; `just test`, `just lint-ci`, `just
  fmt-check`, `just validate`, `just lint-fuzz`, `just deny-fuzz` green; `ljpeg`
  still `pub(crate)` (no `ljpeg` symbol in a default-feature `cargo doc`).
- A PR against `main` from branch `claude/spec-038-build-<suffix>`, its
  description referencing this handoff (`HANDOFF-089`), `SPEC-038`, `STAGE-009`,
  `PROJ-003`, and `DEC-036` / `DEC-035` (plus any new `DEC-*`).

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

- The decoder's logic — SPEC-036/037 (shipped); only **exercise** it, don't change it.
- Bit-exactness / correctness (SPEC-037's round-trip oracle; real-file `dnglab`
  checksum is STAGE-010) — this spec targets panic-freedom, not output values.
- The DNG container path / multi-component / lossy DCT — STAGE-010 and later.
- Widening the public API — keep `ljpeg` `pub(crate)`; expose only the
  `fuzzing`-gated `#[doc(hidden)]` entry (`DEC-036`).

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
