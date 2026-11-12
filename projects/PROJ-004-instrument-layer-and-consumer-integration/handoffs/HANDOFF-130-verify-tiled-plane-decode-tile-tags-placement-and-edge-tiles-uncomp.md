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
  id: HANDOFF-130
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-11
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-058

project:
  id: PROJ-004
  stage: STAGE-015
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: punch-list                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 12863934               # REAL combined count — what cost-audit reads
  estimated_usd: 9.12              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 150
  branch: claude/spec-058-verify-dewdney
  pr: https://github.com/jysf/irradiance/pull/204
  completed_at: 2026-11-11               # YYYY-MM-DD
  notes: "TRUE cost re-summed at ship by the orchestrator from the ended verify transcript (1 jsonl, 71 unique message.ids, all claude-opus-5-5, no subagents): input 149 + cache-creation 179,569 (1-hour) + cache-read 12,644,236 + output 39,980 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 10,428,396 / 7.56 before the commit, push and PR turns."
  synced_at: 2026-11-11
---


# HANDOFF-130: VERIFY: tiled plane decode, tile tags, placement and edge tiles (SPEC-058)

## Delegation Summary

You are the **cold verifier** for SPEC-058. You did **not** implement it. Review it
against SPEC-058 and DEC-056 and return exactly one verdict.

- **The build:** **PR #202**, branch `claude/spec-058-build-5f1a2f`, cut from
  `origin/main` at `e5480ef`. The head when this handoff was minted is `3692e9b`;
  its last commit is the handback PR link only.
- **Your branch:** your worktree is cut from `claude/spec-058-verify-base`. That is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under
  review is the build's code.

The build delivers:
- **Tiled planes in `plane::unpack_into` and `validate`:**
  - tile tags 322–325 on `Sensor`;
  - layout dispatch and a shared tile-table check;
  - uncompressed tiles copied row by row.
- **Compression-7 tiles in `src/ljpeg.rs`:** they decode through a clipped,
  strided destination window (DEC-056). This is the change with the most risk: the
  bit-exact reconstruct loops changed shape, and the strip path must come out
  byte-identical.
- **New `Error` variants** with `ErrorKind` classification.
- **Tests:** `tests/plane_tiles.rs`, plus additions to `plane_oracle`.
- **Fuzz seeds, corpus manifest entries** for the three Pixel files, and docs.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-058-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md`.
  Read all of it, especially the design-time probe table, Design §1–10 and
  AC1–AC11.
- **Decisions:**
  - `./decisions/DEC-056-tiled-planes-decode-in-place-through-a-clipped-strided-window.md`
  - DEC-016: `unpack_into` allocates nothing.
  - DEC-047: frame shape is bound to the plane, here to the tile.
  - DEC-050: validate before allocate, and a shared validate pass.
  - DEC-051: wildcard-free `Error::kind`.
  - DEC-049: required and mandatory opcodes.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-129-build-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation is sound:
  - AC5 overflow asserted on `validate` only;
  - the transposed tile is `Ok` from `validate` and `LjpegPlaneShapeMismatch` from
    `unpack_into`;
  - AC8 refuses with `UnsupportedMandatoryOpcode { id: 9 }`;
  - the window red-proof lives in `tests/plane_oracle.rs`;
  - the Pixel files are skipped by name in `metadata_oracle`;
  - two legacy tests were rewritten.

  The build also says the Compression-7 path allocates `HuffTable` `Vec`s in
  `parse_headers`, an issue that predates this spec. Confirm that this spec added no
  allocation and the old one is unchanged.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test`
  already passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. The three
  Pixel files are in `GOOGLE-PIXEL-4A`, `GOOGLE-PIXEL-3A` and `GOOGLE-PIXEL-7-PRO`.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC11 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run the gate suite yourself, with the corpus env set:
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

### The behavioural surface (this is where defects escape)

**The Pixel oracle runs, and is independent.**
- Run `cargo test --all-features --test plane_tiles --test plane_oracle -- --nocapture`
  with the corpus set. Confirm no SKIP line appears for the Pixel files.
- Independently recompute one Pixel checksum outside the test code. For example,
  dump `unpack_into`'s plane as LE `u16` with a few lines of Rust or Python run with
  `-I`, MD5 it, and compare it with `dnglab analyze --raw-checksum` on the same file.
  The pinned values are in the spec's probe table.
- Confirm the expected values are dnglab's and are not derived from irradiance's own
  output.

**The strip path is byte-identical.** This is the core risk of DEC-056.
- Every held strip file's `raw_checksum` oracle must pass unchanged. Check in
  `tests/corpus/manifest.toml` that no strip expectation was edited:
  `git diff origin/main -- tests/corpus/manifest.toml` should be additions only.
- Check the LJ92 round-trip suite and the held Compression-7 files (Nf=2, the
  Pentax with APPn/COM).
- Read the window code in `src/ljpeg.rs`. Confirm a fully visible window takes
  exactly the old arithmetic: stride equals width, origin (0, 0), no discard.

**The window clip is right at every edge.** Mutate it yourself, outside any test
knob, and confirm a test goes red for each mutation:
- one extra or one fewer visible column;
- one extra or one fewer visible row;
- not discarding a right-of-plane difference (the bitstream desyncs);
- applying `Pt` to discarded samples, if that is observable;
- an off-by-one in the tile origin or stride;
- for `Nf = 2`, an odd visible width.

Use writer fixtures where the Pixel files cannot reach a case. Pixel edges may be
whole-tile or not; check which. The writer-built Compression-7 fixtures must have
partial right **and** bottom tiles, in II and MM, for Nf=1 and Nf=2.

**Hostile tile tables are refused before any write.** For each Design §3 case,
confirm the typed error comes from both `validate` and `unpack_into`, where the
deviation list does not explain otherwise, and that `dst` is untouched on refusal:
- zero `TileWidth` or `TileLength`;
- count mismatch;
- tile count overflow in `u64`;
- both strip and tile tags present;
- a tile out of file bounds;
- an uncompressed byte-count mismatch;
- counts summing past `file.len()`;
- a transposed frame;
- a clipped-size edge frame;
- a non-whole-byte tile row.

Confirm no arithmetic can panic or wrap: tile origin times stride, tile index
products, `u32` to `usize` conversions.

**No allocation added.**
- `git diff origin/main -- src/` must add no `Vec`, `Box`, `vec!`, `collect`,
  `to_vec` or `String` on the decode path.
- Check what AC6's grep test actually pins, and that it would catch a new
  allocation.

**Error surface.**
- Every new variant is in `Error::kind()` with no wildcard.
- Each has `Display` and a test.
- `Sensor` is still `#[non_exhaustive]`, and the new fields are documented.
- `CHANGELOG [Unreleased]` names the additions.

**Develop refuses the Pixel files (AC8).** It must be a refusal, not a silent
decode that ignores the GainMap.

**Fuzz.**
- Seeds in `fuzz/seeds/{plane,develop,demosaic}` include tiled fixtures.
- Run `just fuzz-smoke`, or at least the plane and ljpeg targets for 60s each, if
  the toolchain allows. Report what you ran.

**CI is observed green on the PR head** with `gh pr checks 202` or `gh run view`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are
  fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes
  (DEC-003).
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.
- Open a NEW PR against `main` from `claude/spec-058-verify-<suffix>` carrying the
  handback, and any punch-list fixes. It must superset #202 and reference
  HANDOFF-130, SPEC-058, DEC-056 and PR #202.

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

- **Branch / PR:** `claude/spec-058-verify-dewdney`. PR https://github.com/jysf/irradiance/pull/204, which supersets PR #202 (build head `3692e9b`).
- **Completed at:** 2026-11-11
- **All acceptance criteria met?** yes. AC1 through AC11 are met by the build. Three test gaps against this handoff's verify contract are fixed here, in tests only. `src/` is unchanged from #202.
- **Verdict: ⚠ PUNCH LIST.** There are 3 items, all test gaps, all fixed in this PR. The decoder is sound: no source change was needed.

**Punch list (all fixed in `tests/plane_tiles.rs`):**

1. **Nothing pinned that `dst` stays untouched when a tile table is refused.** The hostile-table test checked only the error variants.
   - **Fix:** `assert_both_refuse` now pre-fills `dst` with a sentinel and asserts that every sample is unchanged after the refusal.
   - **Red-proof:** I mutated `unpack_into` to skip the shared tile pass (M26 below). The per-tile checks in the write loop then refused with the same variant, but only after earlier tiles were written. The old test stayed green on that; the new assertion goes red.
2. **Nothing pinned the refusal of a clipped-size edge frame** (DEC-056 §3).
   - **Fix:** a new test, `clipped_size_edge_frame_is_refused`, with two cases:
     - a single tile larger than its plane, coded at only the visible 5×3. It must refuse before any write, so `dst` stays untouched.
     - a 21×13 plane in 8×4 tiles whose edge tiles are coded at their visible size. It must refuse at the first right-edge tile.
   - **Red-proof:** dropping the per-tile shape binding (M25) turns it red. Without the binding, the window would accept the frame and decode it.
3. **AC10's "a test" for each new variant covered only `ErrorKind`, not the `Display` text.**
   - **Fix:** a new test, `tile_error_display_names_the_numbers`, checks that each variant's message names its values.

**Independent Pixel checksum.** A standalone cargo program in my scratchpad, outside the repo's tests, called `Container::parse` → `sensor` → `unpack_into` and dumped the plane as little-endian `u16`. I took its MD5 with `md5` and compared it with a live `dnglab 0.7.2 analyze --raw-checksum` run on the same file. Both sides matched:
- Pixel 4a: `d1830234ac3f32b07ff73792f0faeabb`
- Pixel 3a: `eb736a494bdfd2fc5ec799fb798ed2c3`
- Pixel 7 Pro: `eb72cb300a27c4bcab745702542269de`

These match the spec's probe table, which pins dnglab values, not irradiance output.

The Pixel edges are partial, not whole-tile:
- 4032 = 15×256 + 192 and 3024 = 11×256 + 208.
- 3882 and 2924 leave 42 and 172.

**The strip path is byte-identical.**
- `git diff origin/main -- tests/corpus/manifest.toml` has no removed lines: additions only.
- With the corpus set, `just test` was green: 581 passed, 0 failed, and no Pixel SKIP line. That covers:
  - every held strip file's `raw_checksum`, including the two Nf=2 Compression-7 files, one of them the Pentax with APPn/COM segments;
  - the LJ92 round-trip suite.
- **Reading the code:** `Window::dense(w, h)` gives stride = width, origin (0, 0) and everything visible. So:
  - `row_base(r) + c` equals the old running `idx`;
  - the neighbour offsets `up` and `up_left` equal the old `width` and `width + 1`;
  - `is_visible` is always true;
  - the point transform covers the whole buffer;
  - the old `expected == 0` early return is the new zero-width or zero-height return;
  - the length check and its error are unchanged in `check_dense_length`.
- The ljpeg unit test `full_window_is_byte_identical_to_dense_decode` pins all of this over predictors 1–7, Pt 0 and 2, restart intervals 0 and 3, and Nf=1 and Nf=2.

**Window-clip mutations.** Each was applied to production `src/` in an isolated copy of the tree, then the ljpeg unit tests plus `plane_tiles`, `plane_oracle`, `plane_unpack` and `plane_validate` were run with the corpus set. The baseline was green. **All 26 mutations went red:**

- **One extra / one fewer visible column** in the window (M01, M02).
- **One extra / one fewer row,** for Nf=1 and Nf=2 separately (M03–M06).
- **No discard of right-of-plane differences:** Nf=1 (M07), Nf=2 component 1 (M08) and Nf=2 component 0 (M09).
- **Point transform:**
  - applied to discarded columns (M10) or to the row below (M11). The ljpeg unit test catches both.
  - The writer fixtures and the Pixel files all use Pt 0, so the unit test is the only guard here, and it is enough: it uses a sentinel-filled, offset window with Pt 2.
- **Tile origin x or y off by one** (M12, M13).
- **Compression-7 stride off by one** (M14).
- **`row_base` drops `x0`** (M15).
- **The Nf=1 "up" offset uses the frame width instead of the stride** (M16), and **the Nf=2 stride does the same** (M17).
  - The Pixel files use predictor 1 and cannot see an up-neighbour mutation.
  - The all-predictor ljpeg window test in a wider plane can, and catches both.
- **Nf=2 odd visible width keeps component 1** (M18).
- **Uncompressed tiles: one extra column, one fewer column, one fewer row** (M19–M21).
- **The visible extent computed one short:** width (M22), height (M23).
- **Wrong up-left offset** (M24).
- **No per-tile shape binding** (M25). It is caught by the transposed-tile test and by the new clipped-size test.
- **`unpack_into` skips the shared tile-table pass** (M26). It is caught by the new dst-untouched assertion and by the existing alias tests.

**Hostile tile tables.** Every Design §3 case is typed from both `validate` and `unpack_into`, and now `dst` is untouched. The build's three declared deviations are sound:
- **The u64 overflow case runs on `validate` only.** `unpack_into`'s documented `dst`-length precedence makes the 8 GiB buffer the only way to reach it.
- **A transposed or clipped-size frame is `Ok` from `validate`.** `validate` never decodes; that is its contract for strips too.
- **AC8 refuses with `UnsupportedMandatoryOpcode { id: 9 }`.** GainMap is unknown and mandatory, so that is DEC-049's refusal.

**Arithmetic:**
- The tile origin and extent are computed in `u64` with saturation, then `try_from`.
- `row_base` and `check_window` use checked arithmetic.
- The tile index is bounded by the table length, which equals `across × down` in `u64`.

I found no path that can panic or wrap.

**Allocation.**
- `git diff origin/main -- src/` adds no `Vec`, `vec!`, `collect`, `to_vec`, `Box`, `String` or `format!` on the decode path. The only additions are the `Sensor` fields at IFD parse time, plus test code.
- The pre-existing `parse_headers` `Vec<HuffTable>` is unchanged. It is called twice per tile (the shape bind, then the decode), exactly the pattern the strip path already uses per strip.
- AC6's grep test covers all of `plane.rs`'s production code and the window section of `ljpeg.rs`. It would catch any of those spellings.

**Error surface.**
- The three new variants are in `Error::kind()`, which has no wildcard.
- `Display` is now tested (item 3).
- `Sensor` is still `#[non_exhaustive]`, and its new fields are documented.
- CHANGELOG `[Unreleased]` names everything that was added.

**Fuzz, run locally for 60 s each.** All exited 0 with no crash artifacts:
- `just fuzz-plane`: 4,472,447 runs.
- `cargo +nightly fuzz run ljpeg` with the ljpeg seeds: 6,027,044 runs.
- `ifd` (`just fuzz`): 7,094,631 runs.

The seeds in `fuzz/seeds/{plane,develop,demosaic}` include 5 tiled fixtures each.

**Gates.** All green on this branch:
- `just test` with the corpus;
- `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `cost-audit`, `gate-audit`, `doc-check` and `deny`;
- `plane_tiles` re-run after my edits: 13 passed.

`decisions-audit --changed` is advisory only. CI on #202's head: all checks pass (`gh pr checks 202`).

### Cost self-report

- **Tokens (total):** 10,428,396. This is a floor; see notes.
- **Estimated USD:** $7.56
- **Duration (minutes):** ~150
- **Source of the number:** this session's transcript (`basis: transcript`).
  - Deduplicated by `message.id`, keeping the max output per message.
  - All messages are claude-opus-5-5, with no subagents.
  - It excludes the commit, push and PR turns.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in the build beyond those it declared, and all of those are judged sound above. This PR changes tests only.
- **Follow-up work identified:**
  - **The Pixel metadata oracle is still missing.** It is the build's follow-up: a per-channel `BlackLevel` colour-oracle entry, which belongs with STAGE-016.
  - **DEC-016's wording versus the `parse_headers` `Vec`s.** This is the build's finding: the Compression-7 path allocates header-sized Huffman tables per strip or tile. Either a sentence in DEC-016 or a fixed-size table array would settle it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. One thing cost time: the plane_oracle copy-and-rebuild red-proofs make a mutation run slow in debug builds. Skipping them and using opt-level 1 made 26 mutations practical.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — "dst untouched on refusal" sits in the verify contract but not in the spec's AC5. If it were in the AC, the build would have pinned it.

3. **If you did this task again, what would you do differently?**
   — I would start the mutation runner in an isolated copy at the very start, in parallel with `just test`.
