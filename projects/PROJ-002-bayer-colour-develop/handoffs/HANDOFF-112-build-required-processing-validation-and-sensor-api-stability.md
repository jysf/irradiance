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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  tokens_total: 18666929           # REAL combined count — what cost-audit reads
  estimated_usd: 12.71
  duration_minutes: 45
  branch: claude/spec-054-build-938c45
  pr: 170
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns."
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
above AND the `## Handback

### Execution notes

- **Branch / PR:** `claude/spec-054-build-938c45` → https://github.com/jysf/irradiance/pull/170
- **Completed at:** 2026-11-10
- **All acceptance criteria met?** yes. The DEC/AC readings below are recorded as deviations.
  - **List-2 bytes kept:** `Sensor.opcode_list_2` is read like lists 1/3. Covered by
    `ifd::tests::opcode_list_2_bytes_are_kept_when_readable`,
    `malformed_opcode_list_2_costs_the_field_alone` (`None` + `malformed_tags`), and the
    container-level `opcode_list_2_bytes_are_kept_on_read`.
  - **Mono / colour required-processing:** `tests/required_processing.rs` (23 tests) has
    5 controls (the Q2-Mono shape: mandatory list-1 FBPC + list-3 warp; optional-only in every
    list; an optional duplicate; no lists) and the rejections: mandatory list-2 op; duplicate
    mandatory FBPC; mandatory FBPC behind an optional one; duplicate mandatory warp;
    recognized-op-in-wrong-list ×3; mandatory unknown in list 2; truncated list 2; and, on
    the colour path, mandatory ops in lists 1/2/3, after optional ones, unknown, and truncated.
    In-crate unit tests in `src/opcode.rs` pin the applied-set rule directly.
  - **Each public entry validates (§12):** the mono path has exactly one public entry
    (`develop_into`; there are no mono `f32`/threaded variants), and every colour test
    loops all four colour entries (`develop_color_into`, `develop_color_f32_into`,
    `develop_color_into_threaded`, `develop_color_f32_into_threaded`). The threaded entries
    run the check themselves, because their banded path never reaches the single-thread entry.
  - **Red-proof (shown red first, DEC-009):** (1) on the pre-fix tree, the same test file in
    plain `is_err()` form had all 12 required cases fail with `Ok(())` and all 5 controls pass.
    (2) The durable `required_processing_red_proof` rebuilds the crate twice. The unmutated
    control gives `err UnsupportedRequiredOpcode {..}` for mono list-2, mono duplicate, and a
    colour list-1 op on all 4 colour entries, with the controls `ok`. The copy with the check
    short-circuited to `Ok(())` returns a silent `ok` on all 6.
  - **Corpus unchanged (local tier-B, DEC-003):** all 9 corpus files are present. Each of the
    8 developable files went through every applicable public entry (`develop_into` for the 6
    mono files; all 4 colour entries for CANON-5D3 and NIKON-D300S). The output hashes are
    **byte-identical** between pre-fix `main` (75f681a) and this branch, all `Ok`. The
    K3III.PEF is not a develop target. Opcode presence per file matched DEC-049's probe: the
    Q2-Mono frames are `[true, false, true]` and every other file is `[false, false, false]`.
    The full suite with `$IRRADIANCE_CORPUS_DIR` set: 483 passed, 0 failed, 1 ignored (the
    pre-existing SPEC-017/SB-1 ignore). The 3 SKIP lines are the pre-existing by-design ones.
  - **`Sensor` sealed:** `#[non_exhaustive]`. A default `cargo build` is unaffected and has no
    constructor. `just test` (`--all-features`) is green.
  - **No determinism change:** `DEVELOP_VERSION` is still `"4"`.
  - **Gates:** `just test`, `lint-ci`, `lint-fuzz`, `lint-no-allow`, `fmt-check`, `validate`,
    `cost-audit`, `deny`, `deny-fuzz` are all green. Fuzz: `just fuzz-opcode 60` → 4,944,443
    runs clean (the target now drives the scan with two cross-path invariants), and
    `just fuzz-develop 60` → 6,341,644 runs clean (`develop_into` now runs the scan first).

### Cost self-report

- **Tokens (total):** 18,666,929 (input 167 + cache write 238,674 + cache read 18,346,380 + output 81,708)
- **Estimated USD:** ≈ $12.71
- **Duration (minutes):** ~45
- **Source of the number:** this session's own transcript
  (`~/.claude/projects/<this worktree>/180264f4-….jsonl`, anchored on the scratchpad UUID),
  usage records deduped by message id; `/cost` is not callable as a tool in a delegated
  session. USD at Opus list rates (the HANDOFF-109 basis). Excludes the last few
  commit/report turns.

### Drift and new artifacts

- **New decisions emitted:**
  - none — DEC-049 covered every choice; the deviations below are readings of it.
- **Deviations from spec:**
  - **Fixture count was 12, not 7.** `#[non_exhaustive]` also forbids external `..base`
    functional-update, so every `Sensor { .. }` in `tests/` moved: 12 sites across
    `develop_oracle`, `develop`, `warp`, `color_oracle`, `colour_metadata_oracle`,
    `metadata_oracle`, `threaded`, and `demosaic_oracle`. A 13th fixture lives inside
    `develop_oracle.rs`'s red-proof **probe-crate source string**, whose staged
    `cargo build` now passes `--features test-fixtures`. I did the migration with a
    brace-aware script. It drops fields equal to the blank defaults only when no `..base`
    exists (over a spread, such a field is a real override).
  - **Constructor shape:** a blank `Sensor::test_fixture()` (feature `test-fixtures`,
    `#[doc(hidden)]`) plus public-field assignment, not a chained builder. Fields are still
    assignable on a sealed struct, so this gives the same "set only what you exercise" result
    with no 30-method builder.
  - **Unknown mandatory id keeps `UnsupportedMandatoryOpcode`.** The spec's `## Design` says
    to keep surfacing it, while AC3 says "any mandatory opcode … returns
    `UnsupportedRequiredOpcode`". I followed `## Design`: recognized ids (1, 4) not applied →
    `UnsupportedRequiredOpcode { list, id }`, and unrecognized mandatory ids →
    `UnsupportedMandatoryOpcode { id }` in every list on both paths. That keeps the existing
    mono-path error unchanged. Both are typed refusals, and the colour fixture for AC3 uses a
    recognized id.
  - **Shared framing reader:** `parse_opcode_list`'s inline framing moved into
    `read_raw_opcode`, which the scan also uses, so the two cannot drift. Errors and offsets
    are pinned unchanged (`shared_framing_reader_keeps_parse_opcode_list_errors_unchanged`).
  - **Error order:** the scan runs before the buffer-length checks in every entry ("before any
    allocation or decode"). An input that is wrong in both ways now reports the opcode error.
  - **Applied-instance rule:** "the first instance of the applied id is applied, whatever its
    own flags" (that is what `find_map` does). So `[optional FBPC, mandatory FBPC]` is
    refused, because the mandatory one is not applied.
- **Follow-up work identified:**
  - **Group E (SPEC-039) preflight:** `demosaic_into` / `demosaic_into_threaded` take a
    `Sensor` but are stage primitives, not develop entries, so they do not run the scan.
    E's unified preflight should decide whether they should.
  - **DEC-012 tolerance vs required processing:** a *malformed* `OpcodeList2` (or 1/3) tag is
    `None` + `malformed_tags`. The develop then proceeds without seeing its opcodes, so a
    structurally broken list tag can still hide a mandatory opcode. This is the parity DEC-049
    asked for, but E or F may want "presence flag set but bytes unreadable" to refuse.
  - `irr ifd` prints opcode-list presence only, as before for lists 1/3. Parity holds, so
    nothing was added.
  - Conflict note: SPEC-055 (group D) also edits `Cargo.toml` `[features]` (the `irr`
    feature). The second of #170 and the D PR to merge will need a trivial rebase.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Two things. The fixture count (7) missed the `..base` spreads, which `#[non_exhaustive]`
   also forbids, and the probe-crate source string inside `develop_oracle.rs`; together that
   was 13 sites. And AC3's "any mandatory opcode → `UnsupportedRequiredOpcode`" conflicts with
   `## Design`'s "still surface `UnsupportedMandatoryOpcode` for an unknown id". I followed
   `## Design` and recorded it above. The handoff also listed "`f32`/threaded" variants of the
   mono `develop_into`, which do not exist; the mono path has only one public entry.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Not a constraint. One mechanism was missing: the repo's red-proofs rebuild a crate copy
   as a separate `bin`, and that bin is outside the crate, so any probe that builds a `Sensor`
   literal broke on the seal. Naming "probe crates count as external" would have surfaced the
   13th site up front.

3. **If you did this task again, what would you do differently?**
   — I would write the scripted migration with the spread-safety rule from the start. My first
   pass dropped default-valued fields even over `..base`, which would have changed fixture
   meaning; the compiler's "unneeded `mut`" warning exposed it. I would also add a whole-output
   hash comparison of the corpus as a standing local recipe. No existing test pins develop
   output byte-for-byte, so I built a one-off probe that hashes main against this branch.
