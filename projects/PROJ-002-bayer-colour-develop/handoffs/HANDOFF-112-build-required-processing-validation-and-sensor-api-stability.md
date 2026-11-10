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
  id: HANDOFF-112
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-054

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

# HANDOFF-112: Required-processing validation + Sensor API stability (SPEC-054, group C)

## Delegation Summary

The architect hands `SPEC-054` to the implementer for the **build** cycle: keep the
`OpcodeList2` bytes, and make every public develop entry return a typed error when a
DNG carries a **mandatory** opcode the pipeline will not apply — instead of today's
silent `Ok` with a plausible-looking wrong image — on the mono path **and** the colour
path; then seal `Sensor` (`#[non_exhaustive]`) because the fix adds a field. The whole
decision is **DEC-049**. **Read SPEC-054 in full and DEC-049 first; `## Design` has the
exact sites and the applied-set rule, and DEC-049 records the corpus measurement that
makes this pure hardening.**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-054-required-processing-validation-and-sensor-api-stability.md` — all of it, especially `## Design`, `## Acceptance Criteria`, `## Failing Tests`.
- **Decision:** `./decisions/DEC-049-validate-required-opcode-processing-and-seal-the-sensor-view.md` — the opcode contract, the applied-set per path, the `Sensor` seal + fixture migration, the corpus-safety probe, and the rejected alternatives (do NOT re-litigate them: no typed-parse validation, no colour-path opcode application, no deferring the `Sensor` seal).
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #3, open maintainer decision #3) and `./docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` §4 (the `#[non_exhaustive]` urgency).
- **Code:** `src/ifd.rs` (`Sensor` `:642`, opcode-list fields `:685-704`, assembly `:1708-1714`, `TAG_OPCODE_LIST_2` `:132`, `DEC-012` interpret helper `:1246`, `malformed_tags` `:767`); `src/opcode.rs` (`parse_opcode_list` `:299`, `FLAG_OPTIONAL` `:208`, `find_map` entries `:381`/`:417`); `src/develop.rs` (`develop_into` opcode dispatch `:831-872`, the `f32`/threaded variants); `src/color.rs` (`develop_color_into` `:288`, `develop_color_f32_into`, threaded `:383`, the `:255` note); `src/lib.rs` (`Error` `#[non_exhaustive]` `:143`, `UnsupportedMandatoryOpcode` `:512`/`Display` `:913`, `DEVELOP_VERSION`); `tests/develop_oracle.rs` + `tests/colour_metadata_oracle.rs` + `tests/color_oracle.rs` (the 7 `Sensor` fixtures); `fuzz/fuzz_targets/`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap (DEC-004 rule 5).

> ⚠ The spec's line numbers were read 2026-11-10 against the current tree;
> re-confirm before editing in case of drift.

### Decisions that apply

- `DEC-049` — the opcode-validation contract + the `Sensor` seal this spec builds.
- `DEC-012` — interpret-phase tolerance: a malformed `OpcodeList2` tag is `None` +
  `malformed_tags`, parity with lists 1/3.
- `DEC-036` — the `fuzzing` dev-feature precedent for the in-crate fixture constructor.
- `DEC-045` — the re-plan; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus never committed; the bit-exact corpus check is a local run.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-panics-on-untrusted-input` — five deny lints; the flags-aware scan runs on hostile
  opcode-list bytes with checked arithmetic over the `id/version/flags/size` framing.
- `oracle-must-be-shown-red` — the opcode-validation red-proof (mono list-2 + colour)
  is the load-bearing deliverable, shown red before green.
- `library-not-application` — the `Sensor` fixture constructor is dev-feature-gated and
  must not enter the default/published surface.
- `provenance-recorded-per-algorithm` — a ledger row for the scan (DNG 1.7.0.0 Ch.7
  `Flags` semantics; this build's own scan).

### Prior related work

- `SPEC-017` / `SPEC-018` — the `OpcodeList1` / `OpcodeList3` readers + appliers and
  `parse_opcode_list`'s existing `UnsupportedMandatoryOpcode` for unknown-mandatory ids.
- `SPEC-053` (group B) — the sibling "parser-accepts-what-pipeline-cannot-honor" fix
  (`Hi/Vi` + shape); same silent-wrong-`Ok` class.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `Sensor.opcode_list_2` kept (DEC-012 parity); `Sensor` sealed `#[non_exhaustive]`.
- The `pub(crate)` flags-aware required-processing scan + `Error::UnsupportedRequiredOpcode { list, id }` and its `Display`, called at **every** public develop entry (mono/colour, `f32`, threaded).
- The 7 `tests/` `Sensor` fixtures migrated onto a dev-feature-gated in-crate constructor/builder; `just test` (`--all-features`) green.
- Red-proofs shown red first (mono list-2 mandatory, colour mandatory, duplicate-mandatory); per-entry behavioural tests (§12); the fuzz extension run clean for the configured window.
- Corpus bit-exact (local tier-B run) recorded in the handback; `DEVELOP_VERSION` unchanged.
- CHANGELOG `[Unreleased]` entry + a `docs/provenance-ledger.md` row.
- A PR against `main` from branch `claude/spec-054-build-<suffix>`.
- PR description referencing: HANDOFF-112, SPEC-054, STAGE-010, PROJ-002, DEC-045 / DEC-049.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12). Here: the error must fire from EACH
  public develop entry, not one representative.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- Honouring any new opcode, or applying opcodes on the colour path — this rejects
  unsupported required processing, it does not add new honouring.
- Any change to a corpus develop's output, or a `DEVELOP_VERSION` bump.
- Sealing `Cfa` / `WarpRect` / `Opcode` — that sweep is SPEC-042 (group F).
- Reshaping the `Opcode` enum to carry `flags` (DEC-049 rejected it; use the raw scan).
- The parser guard (SPEC-052), the Compression-7 shape binding (SPEC-053), and the
  licence/package work (SPEC-055).

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
