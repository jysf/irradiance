---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. This is the BUILD half of SPEC-039 (group E).
# The verify half will be a later handoff. `handoff.cycle` distinguishes them.
#
# The `handback:` block is the RETURN path for cost; do not leave it unfilled.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-117
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.build — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-10
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-039

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Report the REAL token
# number from your own interface. If your platform exposes none, set
# tokens_total: null AND say why in notes. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null
  notes: null
  synced_at: null
---

# HANDOFF-117: Build the unified validate-before-allocate preflight (SPEC-039, group E)

## Delegation Summary

The architect hands `SPEC-039` to the implementer for the **build** cycle: add
`pub fn plane::validate(sensor, byte_order, file, max_pixels) -> Result<usize, Error>`
— one allocation-free, panic-free pre-allocation gate covering **both** the uncompressed
and the Compression-7 path, with a caller-selected pixel budget checked first — and
refactor `unpack_into` onto the same shared checks so the two cannot drift. Also ratify
EOI-optional with a test, add the `ifd`/`plane`/`ljpeg` fuzz smoke jobs, extend the
`plane` fuzz target to the validate contract, and reconcile the stale docs. The whole
decision is **DEC-050**. **Read SPEC-039 and DEC-050 in full first** — `## Design` has
the budget-first ordering and the precedence-preserving refactor, which are the
non-obvious parts.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-039-plane-validate-validate-before-allocate-extract.md` — all of it, especially `## Design`, `## Acceptance Criteria`, `## Failing Tests`.
- **Decision:** `./decisions/DEC-050-unified-validate-before-allocate-preflight-and-caller-selected-pixel-limit.md` — the contract and the rejected alternatives (do NOT re-litigate: no 3-arg structural-only `validate`, no library-chosen cap, no decode inside `validate`, no required EOI, and the two group-C follow-ups stay deferred).
- **Code:** `src/plane.rs` (`unpack_into` `:278`, `unpack_uncompressed`, `unpack_jpeg` `:526`, `check_strip_shape` `:475`, the compression dispatch `:290`, `SUPPORTED_BITS`); `src/ifd.rs` (`Sensor` fields, `packed_bits()`, `MAX_TAG_VALUES`); `src/lib.rs` (`Error` `#[non_exhaustive]` `:143` + `Display`); `src/ljpeg.rs` (`parse_headers` returns at `SOS`, the scan completes on sample count, `MARKER_EOI` — for the EOI test/doc); `fuzz/fuzz_targets/plane.rs`; `.github/workflows/ci.yml` (the `fuzz-opcode` job `:253` to mirror); `examples/fuzz-seeds.rs`.
- **Docs to reconcile:** `README.md` (status `:42-45`, PROJ-003 scope `:157`), `docs/conformance-matrix.md` (Leica M Typ 246 + Pentax K-3 III rows), `CHANGELOG.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap (DEC-004 rule 5).

> ⚠ Line numbers were read 2026-11-10 against `main` after A–D landed; re-confirm
> before editing in case of drift.

### Decisions that apply

- `DEC-050` — the contract this builds.
- `DEC-016` — zero-allocation discipline (`validate` and the shared helpers allocate nothing).
- `DEC-047` (SPEC-053) — the decoded-frame shape binding stays a runtime decode check; `validate` does not duplicate or pre-empt it.
- `DEC-045` — the re-plan; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus never committed; the bit-exact + no-new-error corpus check is a local run, recorded in the handback.

### Constraints that apply (see `./guidance/constraints.yaml`)

- `no-panics-on-untrusted-input` — five deny lints; the budget product is `u64` with checked arithmetic; no indexing/slicing on hostile strip tables.
- `oracle-must-be-shown-red` — the three red-proofs (coverage, aliasing, budget removed) are load-bearing, shown red before green.
- `library-not-application` — `max_pixels` is the caller's number; the library adds no cap of its own.

### Prior related work

- `SPEC-052` (A) — the IFD-parse resource guard, upstream of `Sensor`; `validate` composes with it, does not duplicate it.
- `SPEC-053` (B) — `check_strip_shape`, the decoded-frame shape binding `validate` relies on at decode time.
- `SPEC-054` (C) — the required-processing check; its two verify follow-ups are **deferred**, not part of this build.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- `pub fn plane::validate(sensor, byte_order, file, max_pixels) -> Result<usize, Error>` — allocation-free, panic-free, both paths, budget checked first (DEC-050 §2–§5).
- `Error::StripBytesExceedFile { declared: u64, file_len: usize }` and `Error::PlaneTooLarge { pixels: u64, limit: usize }` + their `Display`.
- `unpack_into` refactored onto the shared helpers with error precedence preserved (`PlaneBufferWrongLength` still wins).
- EOI-optional test + a one-line doc on the decode entry (no code change to the decoder).
- `tests/plane_validate.rs` with every `## Failing Tests` case; the three red-proofs shown red first.
- `fuzz/fuzz_targets/plane.rs` extended; `ifd`/`plane`/`ljpeg` 60 s CI smoke jobs added (mirror `fuzz-opcode`).
- README + CHANGELOG + conformance-matrix reconciled; a tested-DNG-modes compatibility table; a `docs/provenance-ledger.md` row.
- Tier-B corpus bit-exact and trips neither new error (local run, recorded in the handback).
- A PR against `main` from `claude/spec-039-build-<suffix>` referencing HANDOFF-117, SPEC-039, STAGE-010, PROJ-002, DEC-045 / DEC-050.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass (`just test`, `just lint`, the fuzz smoke)? Build reflection answered honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- §12 behavioural surface (AGENTS.md §12): confirm the **budget check fires first on BOTH paths** (not just uncompressed), `validate` and `unpack_into` genuinely share the checks (mutate a layout, see both change), the uncompressed bound holds under aliased strips, a Compression-7 plane validates structurally **and** the shape binding still fires at decode, and EOI-optional holds — exercised, not just shape-checked.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- `Error::kind()` (SPEC-040) and the 0.2.0 release (SPEC-042).
- Group C's two deferred follow-ups (malformed `OpcodeList` tag read as absent; stage functions skipping the required-processing check) — DEC-050 §Scope. A cheap safe fix → a new spec, not this handoff.
- Validate helpers for `demosaic_into`/`to_linear_f32_into`; any library cap on `n`.
- The parser guard (SPEC-052), the shape binding (SPEC-053), the opcode validation (SPEC-054).

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (with a real `tokens_total`) and the
`## Handback` section, set `handoff.status` → `completed` and `handback.status` →
`completed`, and open the PR. If blocked: fill `## Handback` with what blocked you, set
`handoff.status` → `rejected` / `handback.status` → `blocked`, still report tokens, set
the spec's `task.blocked: true`, and add a question to `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator transcribes, it does not reconstruct.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **Tier-B corpus:** bit-exact? trips neither new error? (the local run result)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:** `DEC-NNN` — <title> (if any)
- **Deviations from spec:** [list]
- **Follow-up work identified:** [any new specs for the stage backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?** — <answer>
3. **If you did this task again, what would you do differently?** — <answer>
