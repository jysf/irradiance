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
  id: HANDOFF-128
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-11
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-057

project:
  id: PROJ-004
  stage: STAGE-013
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
  verdict: punch-list              # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 7793217            # REAL combined count — what cost-audit reads
  estimated_usd: 5.31           # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 40
  branch: claude/spec-057-verify-e8b552
  pr: https://github.com/jysf/irradiance/pull/199
  completed_at: 2026-11-11
  notes: "PUNCH LIST with its one item fixed in this PR - Design 2 says IFD entries are sorted by tag, but removing the writer's sort left all 25 tests green (irradiance and dnglab both accept unsorted IFDs); verify added ifd_entries_are_written_in_ascending_tag_order, green honestly, red under that mutation. Everything else checked out - gates green, src unchanged, dnglab and exiftool ran, 4 hand checksums matched, 11 of 12 writer mutations red (the 12th equivalent). Tokens re-summed from this session transcript (dedupe message.id, max output per message, opus-5-5 at 5/25 USD per Mtok, cache read 0.10x, cache write 1.25x), measured just before the handback commit."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-128: VERIFY: test-only DNG writer, strip and tile layouts, arbitrary opcode bytes (SPEC-057)

## Delegation Summary

You are the **cold verifier** for SPEC-057. You did **not** implement it. Review it
against SPEC-057 and DEC-055 and return exactly one verdict.

- **The build:** **PR #198**, branch `build/spec-057-test-only-dng-writer`, cut
  from `origin/main` at `02a95d4`. The head when this handoff was minted is
  `9e897a1`; its last commit is the handback docs only.
- **Your branch:** your worktree is cut from `claude/spec-057-verify-base`. That is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under
  review is the build's code.

The build delivers:
- `tests/support/dng_writer.rs`: a declarative, test-only DNG writer. Its parts are
  `DngDescription`, `write_dng`, `WriteError`, `Payload::{Samples, Encoded}`,
  `Organisation::{Strips, Tiles}`, `IfdLayout`, `extra_tags` overrides, and a
  `WriterFault` red-proof knob.
- `tests/dng_writer.rs`: 12 tests.
- A `docs/provenance-ledger.md` row and a `docs/oracle-contract.md` paragraph.

**`src/` must be unchanged.**

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-057-test-only-dng-writer-uncompressed-mono-and-bayer-fixtures-st.md`
  (all of it, especially the design-time probe table, Design §2–5, and AC1–AC12).
- **Decision:** `./decisions/DEC-055-...` (writer placement). DEC-008 covers
  packing: sub-byte is an MSB-first bitstream; byte-aligned is container order.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-127-build-test-only-dng-writer-uncompressed-mono-and-bayer-fixtures-st.md`.
  Treat its claims as claims and check them yourself. It declares five deviations:
  - three extra `WriteError` variants;
  - a 12-bit edge fill of `0xBEEF & 0xFFF`;
  - `Order` reused from `tiff.rs`;
  - LONG tile tags;
  - inline `extra_tags` keyed on byte length.

  Judge whether each one is sound.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test`
  already passes `--all-features`; add no flag.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC12 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run the gate suite yourself:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `doc-check`
  - `deny`
- Run `just decisions-audit --changed`.
- Confirm `git diff origin/main -- src/` is empty.

### The §12 behavioural surface (this is where defects escape)

**The tool-gated tests actually run.** dnglab 0.7.2 and exiftool are on PATH.
- Run `cargo test --test dng_writer -- --nocapture`.
- Confirm no SKIP line appears and the dnglab assertions execute.
- **Recompute one strip checksum and one tile checksum independently.** Write the
  fixture to disk, run `dnglab analyze --raw-checksum` by hand, and compare it with
  an MD5 of the input samples as LE `u16` that you compute outside the test code,
  for example with a few lines of Python run with `-I`.

**The oracle is not tautological.** Check that the expected MD5 is computed from
the *input samples*, not from bytes the writer produced or irradiance read back. A
test that hashes the writer's own output proves nothing.

**The red-proof is real.** For each `WriterFault`:
- confirm the honest control passes first (DEC-009);
- confirm the fault turns the dnglab checksum red;
- for strips, confirm it also turns the irradiance round-trip red.

Then mutate the writer yourself outside the knob and confirm the tests go red. For
example: flip the sub-byte packing to LSB-first, write tile offsets wrong, or drop
the edge-tile padding.

**Tiles are genuinely tiled.** Confirm the tiled fixture has partial right **and**
bottom tiles, that padding uses the `edge_fill` sentinel, and that both II and MM
are exercised.

**Rows that aren't a whole number of bytes are refused**, with the typed error, for
both strips and tiles. `src/plane.rs` must not have been touched to work around them.

**Placement matches the reader.** With `PreviewIfd0WithSensorSubIfd`:
- `sensor_candidates()` has exactly one entry, the SubIFD;
- colour and identity tags are read from IFD0;
- plane, CFA and level tags are read from the SubIFD.

Check this against `Container::sensor` (`src/ifd.rs`).

**Clean-room provenance.** The ledger row cites TIFF 6.0, DNG 1.7 and DEC-008
only. There must be no sign of ported writer code; spot-check the structure and
comments.

**Promotion readiness (advisory, not a gate).** Note anything that would make
SPEC-061's move into `src/` costly: panicking paths, unchecked arithmetic, or the
`fault` field.

**CI is observed green on the PR head** with `gh pr checks 198` or `gh run view`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are
  fine.
- `Cargo.lock` is gitignored; never `git add` it.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.
- Open a NEW PR against `main` from `claude/spec-057-verify-<suffix>` carrying the
  handback, and any punch-list fixes. It must superset #198 and reference
  HANDOFF-128, SPEC-057, DEC-055 and PR #198.

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

- **Branch / PR:** `claude/spec-057-verify-e8b552` → https://github.com/jysf/irradiance/pull/199. This supersets PR #198: it is the build head `9e897a1`, plus HANDOFF-128, plus one verify-added test and this handback.
- **Completed at:** 2026-11-11
- **All acceptance criteria met?** Yes. AC1–AC12 are each met by a test that goes red when the property breaks. One Design §2 contract property (tag-sorted entries) had no test at all; it is fixed in this PR, see the punch list.
- **Verdict:** ⚠ **PUNCH LIST** — one item, **already fixed in this PR**, with nothing outstanding for the builder. The build itself is sound. If the orchestrator prefers to read it this way: approvable at this PR's head once CI is green.

#### What was checked, and how

**Sync.** HEAD was `19fc547`, which is `origin/claude/spec-057-verify-base`, checked with `/usr/bin/git`. `git diff origin/main -- src/` is **empty**.

**Gates, run locally, all exit 0:**
- `just test`: 564 passed, 0 failed, no `SKIP` line, `IRRADIANCE_CORPUS_DIR` set;
- `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `cost-audit`, `gate-audit`, `doc-check`, `deny`;
- `decisions-audit --changed` (advisory).

After the verify test was added, `fmt-check`, `lint-ci`, `lint-no-allow` and `cargo test --test dng_writer` were re-run: green, 26 tests.

**Tool-gated tests really run.** dnglab 0.7.2 and exiftool are at `/opt/homebrew/bin`. `cargo test --test dng_writer -- --nocapture` printed the dnglab strip checksum (`bc94225…`) and the four tile checksums (`81a5b727…` for 16-bit and `a713d221…` for 12-bit, II and MM). There was no SKIP.

**Independent checksum recompute, outside the test code.** I wrote a throwaway test file, never committed, that dumped four fixtures to the scratchpad. All four use geometry the build never tested and the SubIFD layout:

| Fixture | `dnglab analyze --raw-checksum` | Python `-I` MD5 of the samples as LE u16, regenerated from the ramp formula |
|---|---|---|
| strip 14-bit MM 24×10, 4 strips | `1938b57f…52d0` | `1938b57f…52d0` ✅ |
| tile 12-bit MM 50×37, 16×16 tiles (4×3, partial right **and** bottom) | `3dc9f88b…6f26` | match ✅ |
| tile 16-bit II 50×37, **32×16** non-square tiles | `dd4c8e5a…70fa` | match ✅ |
| tile 8-bit MM 50×37, 16×16 tiles | `bdf7ad43…38f5` | match ✅ |

A separate Python parser read `TileOffsets` and `TileByteCounts` from those three tiled files and unpacked every tile: MSB-first for 12-bit, container order for 16-bit. In each file, every in-plane sample equalled the input, every one of the **1,222 padding samples** equalled that file's `edge_fill` (on both the right and the bottom edges), and every tile offset was word-aligned. There were 0 mismatches.

**The oracle is not tautological.** In `md5_le(input_samples(d))`, the expected value comes from the description's `Payload::Samples`, never from writer output or a read-back. The red-proof compares each faulted file against that same input MD5.

**The red-proof is real.** In `red_proof_writer_faults_turn_oracles_red`, the honest control runs first over all 8 cases, both oracles. It then applies each fault and asserts:
- the bytes changed;
- the dnglab checksum differs from the input MD5 (or dnglab refuses the file);
- for the 4 strip cases, the irradiance round-trip differs.

**Mutations, applied by hand outside the `WriterFault` knob and reverted each time:**

| Mutation | Result |
|---|---|
| M1: 12/14-bit always LSB-first | RED (6 tests) |
| M2: chunk placement +2 bytes | green, but **equivalent**: the offset tag and the write position move together, so the file is still valid |
| M2b: `StripOffsets` tag +2 while the bytes stay put | RED (8 tests) |
| M2c: `TileOffsets` tag +2 while the bytes stay put | RED (3 tests, dnglab included) |
| M3: edge fill written as 0 | RED (`tile_tags_read_back_from_container`) |
| M4: bottom padding rows dropped | RED (3 tests) |
| M5: inline values right-justified (the BE SHORT trap) | RED (11 tests) |
| M6: wrong 8-bit byte | RED (2 tests) |
| M7: tiles in column-major order | RED (3 tests) |
| M8: 16-bit always LE | RED (6 tests) |
| M9: colour matrices moved into the sensor IFD | RED (`metadata_round_trips_in_subifd_layout`) |
| **M10: the per-IFD sort by tag removed** | **green before this verify** → the punch-list item; **RED now** |

**Rows that aren't a whole number of bytes are refused.** `write_errors_are_typed` asserts `RowNotWholeBytes { row_samples: 6, bits: 14 }` for strips. For tiles it uses `tile_width` 5 at 12 bits. `src/plane.rs` is untouched. The plane-width check correctly uses `tile_width` for tiles, so a 50-wide 12-bit plane in 16-wide tiles is accepted, and dnglab matches it.

**Placement matches the reader** (`src/ifd.rs`):
- `Container::sensor` reads `ColorMatrix1/2`, `ForwardMatrix1/2`, `AsShotNeutral` and `CalibrationIlluminant1/2` through `ifd0_rationals` and `ifd0_scalar`. The writer puts them in IFD0.
- `Orientation` is read from IFD0 first, falling back to the sensor IFD. The writer puts it in IFD0.
- Plane, CFA, level, crop, ActiveArea and opcode tags are read from the selected sensor IFD. The writer puts them there.
- Under `PreviewIfd0WithSensorSubIfd`, IFD0 has `NewSubfileType = 1` and Photometric BlackIsZero, so `is_sensor_ifd` says No. The SubIFD has `NewSubfileType = 0` and CFA or LinearRaw, so it says Yes.
- `metadata_round_trips_in_subifd_layout` asserts exactly one candidate, at an index other than 0. M9 proves that this test watches placement.

**Clean-room provenance.**
- The ledger row cites TIFF 6.0 §2/§8/§15, DNG 1.7 and DEC-008 only.
- dnglab and exiftool appear only as black-box oracles.
- The writer's structure is a two-pass "build fields, lay out, rebuild, check stable, serialise" design, with comments citing the spec sections. Nothing resembles rawler's or the `dng` crate's writer idioms.
- There are no copied tables or magic beyond the TIFF and DNG tag numbers.

**CI on PR #198** at head `9e897a1`: **44/44 checks pass**, and the PR is MERGEABLE.

#### The five declared build deviations: all sound

1. **The three extra `WriteError` variants** (`InvalidGeometry`, `EdgeFillOutOfRange`, `Overflow`) are needed to avoid a divide-by-zero, an illegal sentinel, and a u32 overflow. All three are tested. Sound.
2. **The 12-bit edge fill of `0xBEEF & 0xFFF`** is forced, because `0xBEEF` is not a 12-bit value. It is still a recognisable sentinel. Sound.
3. **Reusing `Order` from `tiff.rs`** is sound. For promotion it couples the module to `tiff.rs`, so SPEC-061 must move `Order` (already noted by the build).
4. **LONG tile tags** are allowed by TIFF 6.0 §15, and the reader and dnglab both accept them. Sound.
5. **Inline `extra_tags` keyed on byte length** is documented on `RawTag`, and it is what makes the "huge count, inline word" hostility in AC8 expressible. Sound.

#### Punch list

1. **Fixed in this PR.** Design §2's contract that "entries are sorted by tag" had no test. irradiance's parser and dnglab both accept unsorted IFDs, so M10 (deleting `ifd.sort_by_key`) left all 25 tests green. Today's output is in fact unsorted *before* the sort, because IFD0's DNG tags (50706+) are pushed ahead of the baseline tags. So this is a live property, not a theoretical one.
   - **The fix:** `ifd_entries_are_written_in_ascending_tag_order` in `tests/dng_writer.rs`. It covers both layouts, both byte orders, strips and tiles, and an appended `extra_tags` entry.
   - It reads `Container::ifds()[i].entries()`, which the parser keeps in file order.
   - It is green on the honest writer and red under M10.

#### Advisory (not gating)

- **Padding coverage.** Only the 16-bit right-edge fill value is asserted in tests. Bottom-edge and 12-bit fill are covered only by dnglab not reading them, and dnglab ignores padding. My parser check above confirmed both are right today. SPEC-058's tiled reader tests will cover them naturally.
- **Promotion readiness (SPEC-061).**
  - `write_dng` has no `unwrap`/`expect`/indexing panics; size arithmetic is checked.
  - `max_sample` shifts by `16 − bits` and relies on validation having run first. Keep it private.
  - `serialize`'s `out.resize(o, 0)` relies on `tail` being in ascending offset order. That holds by construction, but a promoted version could `debug_assert` it or return an error.
  - Drop `fault` on promotion, as the build noted.
- **`dng_validate`** was not run: it is not installed and its licence is unchecked. Per the spec this is not a gate.

### Cost self-report

- **Tokens (total):** 7793217
- **Estimated USD:** 5.31
- **Duration (minutes):** ~40
- **Source of the number:** this session's transcript (`basis: transcript`): deduplicated by `message.id`, the max `output_tokens` per message, no subagents. Priced at opus-5-5 rates per component: $5/Mtok in, $25/Mtok out, cache read at 0.10×, cache write at 1.25×. Measured just before the handback commit, so the final commit and PR messages are not included.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by the verify. It added one test, the punch-list fix.
- **Follow-up work identified:** none new. SPEC-058 and SPEC-061 inherit the advisory notes above.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The handoff's suggested mutation "write tile offsets wrong" has an equivalent form: moving the chunk and its offset together stays valid. Only a mismatch between the tag and the bytes is a real fault. Worth wording that way next time.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Design §2's layout rules (sorted entries, word alignment, inline packing) are contract, but no AC named them. So the build tested what the ACs named, and sorting slipped through. An AC line "each IFD is strictly tag-ascending" would have caught it at build.

3. **If you did this task again, what would you do differently?**
   — Run the mutation sweep before reading the tests closely. The surviving mutant pointed straight at the gap. Also, don't start the long gate suite while writing scratch test files in `tests/`; I moved mine out quickly so the gates compiled only committed code.
