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
  id: HANDOFF-144
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-13
  status: accepted                 # pending | accepted | completed | rejected

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

# HANDOFF-144: VERIFY: plane units, full-width row bands decodable in any order (SPEC-068)

## Delegation Summary

You are the **cold verifier** for SPEC-068. You did **not** implement it. Review it
against SPEC-068 and DEC-063, and return exactly one verdict.

- **The build:** **PR #235**, branch `claude/spec-068-build`, cut from `origin/main` at
  `811b513`. Its head is `d555dea`: the code commit `f2fb6ec`, then the handback.
- **Your branch:** your worktree is cut from `claude/spec-068-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`, so the code under review is
  the build's code.

The build delivers:
- In `src/plane.rs`: `Unit`, `Units`, `units`, `unpack_unit_into`. `unpack_into` now
  loops over units through two private kernels, `decode_unit` (strips) and
  `decode_tile_row` (tiles). The four old per-layout functions are deleted.
- In `src/lib.rs`: `Error::PlaneUnitMismatch` (`CallerError`).
- `tests/plane_units.rs`, `tests/plane_units_alloc.rs`, and an `error_kind` row.
- A fuzz differential in `fuzz/fuzz_targets/plane.rs`, plus seeds.
- CHANGELOG and docs.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-068-plane-units-plane-units-sensor-plus-plane-unpack-unit-into-a.md`.
  Read all of it: the §1 unit-count table, Design, AC1–AC10, the red-proof list and the
  Notes.
- **Decision:** `./decisions/DEC-063-a-plane-unit-is-a-full-width-row-band-so-units-split-the-caller-buffer-safely.md`.
  Also DEC-003 (corpus) and the plane decisions it cites (SPEC-028's strip padding).
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-143-build-plane-units-plane-units-sensor-plus-plane-unpack-unit-into-a.md`.
  Treat its claims as claims. The points to judge:
  - **AC6 deviation.** The build says a Compression 7 unit decode allocates exactly what
    `unpack_into` allocates (the Huffman-table `Vec` in `ljpeg::parse_headers`), not
    zero. The test asserts equality for Compression 7. Confirm:
    - that this `Vec` is pre-existing on `811b513`;
    - that it is the only allocation;
    - that the test would catch a new one.
    Then decide whether "no new allocation" is an acceptable reading of AC6.
  - **The zero-width tiled-plane fix** (a fuzz OOM). The claim is that the old
    `unpack_into` walked 0 tiles and the new one would have walked about 1e9 tile rows.
    Check that the fix (no units for such a plane) keeps `unpack_into`'s result and error
    on that input **identical to `811b513`**. Run both on the regression input if you can.
  - **The narrow-`usize` deviation.** `units` returns `PlaneTooLarge` where `unpack_into`
    keeps `PlaneBufferWrongLength`. Judge whether this is acceptable, given that it is
    unreachable on 64-bit.
  - **The structural claim.** `unpack_into` has no loop of its own, and both paths call
    the same kernels. Confirm by reading. Make sure the strip and tile kernels are the
    old bodies moved, not rewritten: diff each old function against its new home.
  - **The fuzz harness relaxation.** Coverage-to-height applies only to planes with
    samples, and collection is capped at 65,536 units. Judge whether this hides anything
    on planes that do have samples.
- **Toolchain:**
  - AGENTS.md §5–6 and §16 (the five oracle rules), and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - `units_assemble_in_any_order_on_held_files` takes about 9.5 min in debug. Run the
    full suite in the background.

## The verify contract

### Acceptance criteria and gates
- Check that every AC1–AC10 is met by a test that would fail if the property broke, and
  that the build reflection is honest.
- Run the gate suite yourself, with the corpus env set:
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
- Run `just decisions-audit --changed`.
- Run `just fuzz-plane 60`.
- Observe CI green on the PR head with `gh pr checks 235`.

### The behavioural surface (this is where defects escape)

**`unpack_into` is unchanged (AC4).** This is the hard criterion.
- `git diff 811b513 -- tests/` must show only new files and the `error_kind` row.
- The source-scanning tests must be unedited and still meaningful:
  - `plane_oracle`'s chunk-extraction and byte-order needles;
  - `plane_validate`'s guards;
  - `plane_tiles`' allocation scan;
  - `ljpeg_shape_binding`.

  Each needle must still match the **live** code path, not a leftover. A needle that now
  sits in dead code makes its test vacuously green. Check that each matches exactly once
  and lies inside `decode_unit` or `decode_tile_row`.
- Confirm that the dnglab, layer-0 and tier-A plane oracles ran (no SKIP) and passed.

**Any-order assembly (AC1, AC2) and the oracle rule.**
- The reference must be independent and sensitive (AGENTS.md §16 rule 5):
  - The expected plane should come from `unpack_into` on `811b513`'s semantics, or from
    an independent decoder. It must not come from the same kernel invoked differently.
    If the test compares units against the new `unpack_into`, both run `decode_unit`.
    Then say whether AC4's unedited oracles carry the independence, or whether this is
    `shared-path-oracle-tautology`.
  - Check that "any order" is actually exercised: reversed or shuffled order, writing
    into a sentinel-filled buffer, with no unit's rows written twice.
- Check the AC2 fixtures cover:
  - a short last band;
  - an odd-width 12-bit plane (whole-strip fallback);
  - a partial last tile row;
  - Compression 7 tiles and a Compression 7 strip.

**Concurrency is the point.** At least one test, or your own probe, must decode units on
separate threads through `chunks_mut` or `split_at_mut` and match. If none exists, write a
throwaway probe to confirm it compiles in safe Rust and matches, and note that it is
missing.

**Mutate it yourself**, outside any test knob, and run **tier A only**:
- the build's 8;
- `units` yielding one extra empty unit (`row_count` 0) at a strip end;
- the tile-row unit's last row unclipped (`TileLength` instead of the remainder);
- `max_band_rows` ignored (always the whole strip). AC3's counts should notice;
- `PlaneUnitMismatch` swapped for `PlaneBufferWrongLength`. AC7 should notice.

A mutant that stays green, or that only tier B catches, is a finding unless you judge it
benign and say why.

**No allocation, no panics.**
- `units` plus iteration allocates nothing.
- An uncompressed unit decode allocates nothing.
- The arithmetic is checked or saturating.
- `NonZeroU32::MAX` on a 1-row plane and on a height-`u32::MAX` zero-width plane both
  terminate quickly.

**Scope.**
- No change to the decode kernels' arithmetic. The SPEC-069 speed-up is **not** here.
- `validate` is unchanged.
- The public additions are exactly DEC-063's five.
- The CHANGELOG states them honestly.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #235.
  - Commit the filled HANDOFF-144 handback (and the timeline's verify line) on a branch
    named `claude/spec-068-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #235 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #235.
  - Reference HANDOFF-144, SPEC-068, DEC-063 and PR #235.
- **Rejected:** comment on #235 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- Any decode-kernel speed-up (SPEC-069 and SPEC-070).
- Restart-interval units.
- Units narrower than the full width.
- A bit-offset band start for rows that are not whole bytes.
- Making the Compression 7 Huffman-table allocation zero (SPEC-070, if anywhere).
- Moving slow tier-B tests to a release lane. This is a build follow-up; note it if you
  agree.

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
