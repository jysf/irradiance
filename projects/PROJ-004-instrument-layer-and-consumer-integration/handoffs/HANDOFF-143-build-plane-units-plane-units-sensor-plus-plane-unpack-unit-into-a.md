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
  id: HANDOFF-143
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer
  created_at: 2026-11-13
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-068

project:
  id: PROJ-004
  stage: STAGE-020
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-143: BUILD: plane units, full-width row bands decodable in any order (SPEC-068)

## Delegation Summary

You are the **builder** for SPEC-068. Cut `claude/spec-068-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-063
and this handoff.

The task:
- Add `plane::Unit` (`#[non_exhaustive]`), `plane::Units` (an allocation-free
  `ExactSizeIterator`), `plane::units(sensor, file, max_band_rows: NonZeroU32)`,
  `plane::unpack_unit_into(sensor, byte_order, file, &unit, dst)`, and
  `Error::PlaneUnitMismatch` (CallerError).
- Every unit is a **full-width band of whole rows**:
  - bands of at most `max_band_rows` rows inside one uncompressed strip whose rows are
    whole bytes;
  - otherwise the whole strip;
  - one tile row for tiled planes;
  - a whole Compression 7 strip.
- `unpack_into` loops over the same units through the same private kernel. Its output
  and error order must not change.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-068-plane-units-plane-units-sensor-plus-plane-unpack-unit-into-a.md`.
  Read all of it:
  - the §1 unit-count table and the parallel probe (Q2M 44.7 ms → 6.1 ms at 8 threads);
  - Design §2–§7: precedence, kernels, the `unpack_into` loop, fuzz;
  - AC1–AC10, the red-proof list, and the Notes, **especially the four source-scanning
    tests on `src/plane.rs`, which must stay unedited**.
- **Decision:** `./decisions/DEC-063-a-plane-unit-is-a-full-width-row-band-so-units-split-the-caller-buffer-safely.md`.
- **Also:**
  - DEC-056 (the tile window);
  - DEC-008 (the two unpack paths);
  - DEC-016 (caller-owned buffers);
  - DEC-047 (frame-shape binding);
  - DEC-050 (`validate` shares the checks);
  - DEC-051, DEC-052;
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/plane.rs` (all of it; the spec's Inputs list the functions);
  - `src/ljpeg.rs`: `Window`, `decode_window_into`, `decode_into`, `parse_headers`;
  - `src/lib.rs`.
- **Tests to mirror:**
  - `tests/plane_unpack.rs`, `tests/plane_tiles.rs`, `tests/plane_validate.rs`,
    `tests/plane_oracle.rs`;
  - `tests/previews_alloc.rs` (the counting allocator);
  - `tests/support/tiff.rs` and `tests/support/dng_writer.rs` (tier-A fixtures);
  - `tests/error_kind.rs`.
- **Toolchain:**
  - AGENTS.md §5–6 and `guidance/toolchain-brief.md`. Also read **AGENTS.md §16 rule 5**:
    an oracle must not run the code under test, and must not be blind to the property.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Run long binaries in the background.

## Expected Deliverables

1. **Tests first.** Write `tests/plane_units.rs`, `tests/plane_units_alloc.rs` and the
   `error_kind` row, as the Failing Tests list says. Show them red; a compile failure
   counts for the new API. Record the order in the handback.
   - AC1 and AC2 compare against `unpack_into` and the **existing** pinned plane MD5s
     and dnglab checksums. Those pins predate this change, so they are independent of
     it.
   - AC3's counts are arithmetic from DEC-063.
2. **Implement it, following Design §2–§5:**
   - the types and the iterator, which walks strips and tiles lazily with no `Vec`;
   - the unit re-derivation in `unpack_unit_into`, ordered as §3 says;
   - one private kernel that both `unpack_unit_into` and the new `unpack_into` loop call.
   - **Reuse** `BitReader`, `unpack_byte_aligned`, `decode_into` and
     `decode_window_into`; never copy them, because the source scans need exactly one
     occurrence.
   - No `unsafe`, no allocation, and no panicking arithmetic.
3. **Fuzz (§6).** The `plane` differential, plus three seeds via `examples/fuzz-seeds.rs`.
   Run at least 60 s and report the run count and seed classification.
4. **Docs (§7).**
   - CHANGELOG `[Unreleased]` Added.
   - The plane module-doc section, with a `std::thread::scope` doc test.
   - No provenance row.
5. **Gates, all green with the corpus env set:**
   - `just test`
   - `lint-ci`
   - `lint-no-allow`
   - `lint-fuzz`
   - `fmt-check`
   - `validate`
   - `doc-check`
   - `deny`
   - `msrv`
   - `cost-audit`
   - `gate-audit`

   Also run `just decisions-audit --changed`.
6. **Red-proofs.** For each mutation in the spec's list, apply it by hand, show the named
   **tier-A** test red, and revert. List them in the handback. For the structural one
   ("`unpack_into` left on the old loops"), say where the old loops went.
7. **Optional measurement.** Record the Q2M 8-thread AC1 wall time with today's crate
   kernel, so SPEC-069's baseline exists. No gate.
8. **PR.** Open it against `main` from `claude/spec-068-build`, referencing SPEC-068,
   DEC-063 and HANDOFF-143. Confirm CI is green on its head.

## Out of Scope

- Restart-interval units.
- Units narrower than the full width.
- Threads inside irradiance.
- Kernel speed-ups (SPEC-069, SPEC-070).
- `validate` changes.
- The develop path.
- A release.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). MD5 strings and counts in
  source are fine.
- **If an existing plane test or a downstream develop pin goes red, the change altered
  output.** Fix the change, never the test. That includes the four source-scanning tests.
- **Fill the `handback:` front-matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

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
