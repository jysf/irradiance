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
  id: HANDOFF-109
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-10
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-052

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
  tokens_total: 7025673            # REAL combined count — what cost-audit reads
  estimated_usd: 5.29
  duration_minutes: 21
  branch: claude/spec-052-build-b8dcbf
  pr: 167
  completed_at: 2026-11-10
  notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-109: Parser resource guard — aggregate entry/byte budget (SPEC-052, group A)

## Delegation Summary

The architect hands `SPEC-052` to the implementer for the **build** cycle: add an
aggregate IFD entry-table byte budget to the container walk so a small hostile file
cannot amplify into a large metadata parse (review finding #1). The whole decision
is **DEC-046** — a running total of each IFD's `count × 12` bytes, rejected the
moment it exceeds the file length, checked inside `read_ifd` before the per-IFD
allocation. **Read SPEC-052 in full and DEC-046 first; the guard is ~10 lines, the
care is in the threading, the `checked_add`, and the fixture that isolates the new
guard from the cycle guard (distinct offsets).**

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work.

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-052-parser-resource-guard-aggregate-entry-byte-budget.md` — read all of it, especially `## Design` and `## Failing Tests`.
- **Decision:** `./decisions/DEC-046-bound-aggregate-ifd-entry-table-bytes-by-file-length.md` — the full rationale, the exact rule, and the measured corpus baseline.
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md` (re-planned by DEC-045).
- **Triage:** `./docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md` (finding #1 and its evidence).
- **Code:** `src/ifd.rs` (`Container::parse` l.848, `walk_chain` l.905, `read_ifd` l.955, `MAX_IFDS` l.69, `ENTRY_SIZE` l.81, `Entry` l.339), `src/lib.rs` (`Error` l.217, `TooManyIfds` + its `Display` l.258/l.763), `tests/support/tiff.rs` (`Ifd::new`, `tiff`, `at_offset`), `fuzz/fuzz_targets/ifd.rs`, `examples/fuzz-seeds.rs`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — test framework, lint quirks, the `+toolchain` fuzz trap. Read it (DEC-004 rule 5).

> ⚠ The spec's `src/ifd.rs` line numbers were read on 2026-11-10 against the
> current tree; re-confirm them before editing in case of drift.

### Decisions that apply

- `DEC-046` — the aggregate-table-bytes guard this spec builds (the rule, the error variant, the mechanics).
- `DEC-045` — the re-plan that created this group; `Error` stays `#[non_exhaustive]`, additive only.
- `DEC-009` — control run before any red-proof.
- `DEC-003` — tier-B corpus is never committed; the corpus check is a local run only.

### Constraints that apply

Check `./guidance/constraints.yaml` for full text.

- `no-panics-on-untrusted-input` — five deny lints; the guard runs on hostile bytes; use `checked_add`.
- `oracle-must-be-shown-red` — the amplification test needs a red-proof against a mutated guard.
- `library-not-application` — zero I/O; no allocation added; the library sets no cap on `n` beyond the file-length bound.

### Prior related work

- `HANDOFF-084` (SPEC-039) — **superseded**; its validate-before-allocate work is the plane analogue of this, and its `StripBytesExceedFile` idea is the same "declared bytes may not exceed the file" idiom (DEC-046 "Why this shape").
- `SPEC-003` — the IFD reader + its fuzz target being extended here.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-052's Acceptance Criteria (the DEC-046 guard).
- All failing tests in SPEC-052 now passing, including the red-proof shown red first.
- The `fuzz/fuzz_targets/ifd.rs` extension + a new aliased-table seed; run `just fuzz` 60 s clean.
- CHANGELOG `[Unreleased]` entry and a `docs/provenance-ledger.md` row.
- A PR against `main` from branch `claude/spec-052-build-<suffix>`.
- PR description referencing: this handoff ID (HANDOFF-109), the spec (SPEC-052),
  the stage (STAGE-010), the project (PROJ-002), and the decisions (DEC-045, DEC-046).
- The tier-B corpus run result (both DNGs still parse) recorded in the handback.

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

- Any change to `MAX_IFDS`, `MAX_IFD_DEPTH`, `MAX_TAG_VALUES`, the `visited` cycle
  guard, or how value payloads are read.
- The pixel/plane validate (SPEC-039 / group E) and any `Error::kind`
  classification code (SPEC-040).
- An absolute `MAX_TOTAL_IFD_ENTRIES` cap (DEC-046 rejected it).
- The Compression-7 shape binding (SPEC-053) and the other review findings — they
  are separate specs.

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

- **Branch / PR:** `claude/spec-052-build-b8dcbf` — https://github.com/jysf/irradiance/pull/167
- **Completed at:** 2026-11-10
- **All acceptance criteria met?** yes
  - **Amplification rejected:** `aliased_ifd_tables_exceeding_file_are_rejected`. The
    fixture (`tests/support/tiff.rs::aliased_ifd_chain`) puts 32 IFDs of 32 entries at
    distinct offsets 12 bytes apart: 12,288 declared table bytes in a 770-byte file.
    Without the guard it parsed `Ok` with all 32 IFDs (shown red before the guard was
    written). With the guard it is rejected at the 3rd IFD (`declared 1152`, more than
    770), and the test asserts the rejection lands within one table of the file length,
    so the walk counted no more than ~`file_len / 12` entries. Both byte orders.
  - **Bound holds:** `ok_implies_table_bytes_bounded` over every `tiff::all()` fixture
    plus overlapping-but-within-budget chains; and the fuzz target asserts it on every
    `Ok` parse.
  - **Distinct from the cycle guard:** `distinct_offsets_isolate_the_new_guard` (all
    offsets distinct, under `MAX_IFDS`, tables really overlap, and the error is the new
    variant). The red-proof mutates only the aggregate guard's condition.
  - **No false positive:** every existing `tiff::all()` fixture keeps its verdict
    (`tests/ifd_reader.rs` sweep green, the new `aliased-ifd-tables` fixture added to
    the shared list as a hostile one); `aggregate_table_bytes_within_file_still_parse`
    pins sum-vs-file-length, not overlap (2 overlapping IFDs, 48 bytes in a 50-byte
    file, parse `Ok`). **Tier-B corpus run (local, `IRRADIANCE_CORPUS_DIR` set):**
    Canon 5D-III `5G4A9394.DNG` — 4 IFDs, 1,524 table bytes of 48,382,640, parses Ok;
    Nikon D300S `_DSC4615.DNG` — 3 IFDs, 828 table bytes of 37,562,890, parses Ok.
  - **Panic-free:** running total via `checked_add` (saturating to `u64::MAX` on
    overflow, which is then rejected), `u64::try_from` for the widenings; no new
    allocation; `just lint-ci` green. `just fuzz 60`: 5,672,937 runs in 61 s, clean,
    with the new bound assertion live and the new `aliased-ifd-tables` seed.
  - **Red-proof:** `ifd_guard_red_proof` — honest tree in-process first, then the
    unmutated CONTROL copy-and-rebuild (`err IFD entry tables declare 1152 bytes,
    exceeding the 770-byte file`), then the MUTANT with the guard condition forced
    false (`ok 32`). Asserts the output changed and the mutant accepts the chain.
- **Gates:** `just test` (corpus set), `just lint-ci`, `just fmt-check`,
  `just validate`, `just cost-audit` — all green.

### Cost self-report

- **Tokens (total):** 7,025,673 (input 95 + cache write 140,487 + cache read 6,845,723 + output 39,368)
- **Estimated USD:** ≈ $5.29
- **Duration (minutes):** ~21
- **Source of the number:** this session's own transcript
  (`~/.claude/projects/<worktree>/2d7c0b54-e5e3-46c3-ad16-506eb16b1154.jsonl`,
  anchored on the scratchpad-dir UUID; usage records deduped by message id; input +
  cache write + cache read + output). `/cost` is not callable as a tool in a delegated
  session. USD at Opus list rates (the HANDOFF-107 basis). Excludes the last few
  handback/PR turns.

### Drift and new artifacts

- **New decisions emitted:** none — DEC-046 was followed as written.
- **Deviations from spec:**
  - **Guard placement:** the check sits after the existing table bounds check
    (`self.data.get(entries_at..entries_end)`), not immediately after `table_len` is
    computed. Both satisfy "after `table_len`, before `Vec::with_capacity`", but putting
    it first would have turned every single over-long table (e.g. the existing
    `entry-count-past-eof` fixture) from `Truncated` into the new error, changing
    existing typed-error behaviour. After the bounds check, the new error fires only
    when tables that each fit the file sum past it — exactly the aliasing case.
  - **Red-proof mutation:** the guard's condition is forced false
    (`if false && ...`) rather than the block deleted, so the mutant compiles with no
    unused-variable noise; semantically the guard is removed.
  - **Seed wiring:** the aliased-table seed rides on the shared `tiff::all()` list
    (which `examples/fuzz-seeds.rs` writes to `fuzz/seeds/ifd/`), so the only edit to
    `examples/fuzz-seeds.rs` is a doc line; `fuzz/seeds/ifd/aliased-ifd-tables.tiff`
    is the new committed seed.
  - Added a small `table_bytes_error_display_names_both_numbers` test beyond the
    spec's list.
- **Follow-up work identified:**
  - none required. Note for SPEC-040: `IfdTableBytesExceedFile` is Corrupt-class.
  - No exact-equality boundary test (tables summing to exactly `file_len`): the
    aliased-chain family cannot hit equality (sum minus length is never 0 mod 12 with
    the header and pointer overhead). The strict `>` is documented in code and DEC-046.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only the exact guard placement: "right after `table_len`" would have changed the
   existing `Truncated` verdict for single over-long tables; the spec's other phrase
   ("before `Vec::with_capacity`") allowed the safer spot. Building the aliased fixture
   took the most thought — `tiff()` overwrites overlapping blocks, so it needed a
   hand-laid periodic layout.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` + DEC-009 covered the red-proof shape, and the
   existing `tests/develop.rs` apparatus copied over cleanly.

3. **If you did this task again, what would you do differently?**
   — Nothing material. I would write the fixture's layout invariants (count word vs
   next-pointer slots never colliding; no next pointer reads as tag 330) as asserts
   up front rather than reasoning them out, though the red run confirmed them.
