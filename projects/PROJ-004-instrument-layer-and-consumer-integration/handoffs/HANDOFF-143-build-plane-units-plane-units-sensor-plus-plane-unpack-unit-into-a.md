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
  status: completed                # pending | accepted | completed | rejected

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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 23143680           # REAL combined count — what cost-audit reads
  estimated_usd: 16.80             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 85
  branch: claude/spec-068-build
  pr: 235
  completed_at: 2026-11-13         # YYYY-MM-DD
  notes: "FLOOR at handback, from transcript 4580ee72-d8aa-40e9-bea3-9a9798d9982d.jsonl (92 unique message.ids, all claude-opus-5-5, no subagents): input 185 + cache-creation 277,915 (1-hour) + cache-read 22,759,860 + output 105,720 (max per message). USD at opus-5-5 rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Excludes the handback commit and CI turns; re-sum at ship."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
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

- **Branch / PR:** `claude/spec-068-build`, https://github.com/jysf/irradiance/pull/235. The change is f2fb6ec; the handback is the next commit.
- **Completed at:** 2026-11-13
- **All acceptance criteria met?** Yes, with one stated deviation on AC6 (below).
- **Test-first order:** `tests/plane_units.rs` and `tests/plane_units_alloc.rs` were written first, along with the `tests/error_kind.rs` row. They were red by compile failure: unresolved `plane::{units, unpack_unit_into, Unit}`, and no variant `PlaneUnitMismatch`. Then came `src/lib.rs` (the variant, `Display`, `kind()`), then `src/plane.rs`, then fuzz, seeds and docs.
- **Gates (corpus env set):**
  - `just test` gave 690 passed, 0 failed, 1 ignored.
  - Green: `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`, `gate-audit`.
  - `decisions-audit --changed` is advisory only. It flags DEC-063, DEC-061 and older decisions for re-reading; this change is consistent with them.
  - `cost-audit` was run after this handback was filled.
- **AC4:** every existing plane test, the source-scanning tests (`plane_oracle` needles, `plane_validate` guards, `plane_tiles` allocation scan, `ljpeg_shape_binding`) and the develop pins pass **unedited**. Each needle occurs exactly once in `src/plane.rs`.
- **AC3 counts:** all §1 counts are confirmed on the held files, with no corrections:
  - Q2M ×3: 8 / 57 / 1;
  - Nikon: 12 / 34 / 12;
  - Canon: 6 / 40 / 1;
  - M Mono: 5 / 35 / 1;
  - Pixel ×3: 12;
  - K-3 III and Typ 246: 1.
- **Red-proofs** (hand-applied in a scratch copy, tier A only with the corpus unset, each reverted; the control is green):
  1. Band byte offset +1 row: red `units_assemble_in_any_order_on_tier_a_layouts`.
  2. Last band of a strip one row long (iterator): red `unit_counts_and_coverage`, `units_assemble_in_any_order_on_tier_a_layouts`, `units_refuse_what_unpack_into_refuses` and `wrong_length_band_is_refused_before_any_write`.
  3. Band decoded one row short (kernel): red AC2 and `units_refuse_what_unpack_into_refuses`.
  4. A band allowed to cross a strip: red `unit_counts_and_coverage` (the Nikon-shaped every-unit-inside-one-strip check), AC2 and AC5.
  5. Whole-strip fallback skipped: red AC2 (odd-width 12-bit), `unit_counts_and_coverage` and `foreign_unit_is_a_typed_mismatch_before_any_write`.
  6. Tile window `y0` left plane-relative: red AC2 (uncompressed and Compression 7 tile fixtures).
  7. Unit re-derivation skipped: red `foreign_unit_is_a_typed_mismatch_before_any_write`.
  8. `dst` length checked before the mismatch: red `foreign_unit_is_a_typed_mismatch_before_any_write` (the foreign unit with a wrong-length `dst`).
  - **Structural.** `unpack_into` has no loop of its own. `unpack_uncompressed`, `unpack_jpeg`, `unpack_tiles_uncompressed` and `unpack_tiles_jpeg` are deleted. Their bodies now live in `decode_unit` (strips) and `decode_tile_row` (tiles), which both `unpack_into` and `unpack_unit_into` call. Verify confirms this by reading.
- **Fuzz:** `just fuzz-plane 65` ran 3,083,243 runs in 66 s, clean. Seed classification (checked from `Unit`'s `Debug`):
  - `units-strip-bands-12bit-short-last` → StripBand;
  - `units-whole-strips-12bit-odd-width` → Strip;
  - `units-tile-rows-partial-last` and the existing `tiled-*` seeds → TileRow;
  - the existing Compression 7 strip seeds → Strip.
- **Fuzz findings, both fixed before the clean run:**
  1. **OOM: a real regression.** A zero-width tiled plane has an empty tile table, so it is a valid grid, and a height near `u32::MAX` gives about 1e9 tile rows. The new `unpack_into` would have walked them all, where the old one walked 0 tiles. Such a plane now has no units; the tier-A regression test is `a_zero_width_tiled_plane_has_no_units`.
  2. **Harness only.** A zero-width strip plane passes `check_strips`' sample-counted coverage whatever rows its strips declare, so its units end at the strips' rows, not at `height`. The fuzz coverage-to-height assertion now applies to planes with samples. The harness also skips collecting more than 65,536 units, the caller-chosen band 1 on a zero-width plane.
- **Measurement for SPEC-069** (Q2M L1021223, release, M4 Pro, median of 7, today's crate kernel):
  - `unpack_into` single-threaded: 242.3 ms;
  - 8 units on 8 scoped threads: 31.4 ms (7.7×);
  - 16 units of 352 rows on 16 threads: 28.7 ms;
  - byte-identical in each case.

### Cost self-report

- **Tokens (total):** 23,143,680. This is a floor at handback; the handback commit and CI turns are excluded.
- **Estimated USD:** 16.80
- **Duration (minutes):** about 85 by the transcript's timestamps.
- **Source of the number:** transcript `4580ee72-d8aa-40e9-bea3-9a9798d9982d.jsonl`, deduped by message.id with the max output per message, at opus-5-5 rates. There are no subagents.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-063 holds, and the zero-width rule is a corner of its §2 (no tiles, so no tile rows).
- **Deviations from spec:**
  - **AC6.** The Compression 7 header parse (`ljpeg::parse_headers`) has always collected its Huffman tables into a small `Vec`, per strip or tile. So a Compression 7 unit decode allocates exactly what `unpack_into` allocates on the same plane. `tests/plane_units_alloc.rs` asserts that equality ("no new allocation") for Compression 7. It asserts zero for `units` plus iteration on every layout, for every uncompressed decode, and for the error paths. Zero for Compression 7 would need an ljpeg change; that belongs to SPEC-070.
  - **`units` on a target with a narrow `usize`.** Where the pixel count does not fit `usize`, `units` returns `PlaneTooLarge { limit: usize::MAX }`, as `validate` does with no budget. `unpack_into` keeps its `PlaneBufferWrongLength` there. This is unreachable on 64-bit targets.
  - **The foreign-unit case.** A unit is judged structurally, so a Q2M 704-row band that also exists on the Canon (rows 0..704) is a valid Canon unit. The tier-B AC5 case uses the Q2M's last band (rows 4928..+704), which the 3950-row Canon does not have.
  - **Zero-width tiled planes** yield no units (the fuzz finding above).
- **Provenance:** none needed. There is no new algorithm; every sample comes from the existing kernels.
- **Follow-up work identified:**
  - `units_assemble_in_any_order_on_held_files` takes about 9.5 min in debug (11 files × 8 assemblies, including band 1 on the 47 MP planes). It is tier B only, so CI skips it, but it lengthens a local `just test`. That matches `orientation_tiles`; a later spec could move such tests to a release-profile lane.
  - SPEC-069 baseline: 242 ms single-threaded, 31 ms at 8 threads.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — AC6's "Compression 7 … 0 allocations" conflicts with the pre-existing Huffman-table `Vec` in `parse_headers`, and the spec did not mention it. I resolved it as "no new allocation". The fuzz invariant "rows tile `0..height`" is also false for zero-sample planes that the existing coverage check already accepts.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Yes: a unit count can be far larger than the tile or strip table (zero-width tiled planes). DEC-063 §3 assumed iterating units costs what today's traversal costs; the fuzz run showed it did not until the zero-width case was cut.

3. **If you did this task again, what would you do differently?**
   — Run the fuzz target before the full `just test`. Both findings came from it, and each one restarted the 25-minute suite.
