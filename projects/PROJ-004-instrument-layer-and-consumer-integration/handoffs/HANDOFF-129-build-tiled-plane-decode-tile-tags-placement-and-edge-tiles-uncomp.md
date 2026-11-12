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
  id: HANDOFF-129
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 31088670               # REAL combined count — what cost-audit reads
  estimated_usd: 21.47              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 120
  branch: claude/spec-058-build
  pr: __PRURL__
  completed_at: 2026-11-11               # YYYY-MM-DD
  notes: "FLOOR measured from this session transcript just before the handback commit (1 jsonl, 110 unique message.ids, all claude-opus-5-5, no subagents): input 221 + cache-creation 334,681 (1-hour) + cache-read 30,641,631 + output 112,137 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Excludes the commit, push and PR turns."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-129: Tiled plane decode, uncompressed and lossless-JPEG tiles with edge tiles, allocation-free (SPEC-058)

## Delegation Summary

`claude-opus-5` (architect, orchestrator session) hands `SPEC-058` to a cold
`claude-opus-5` session (implementer) for the **build** cycle. The task is to
teach `plane::validate` and `plane::unpack_into` to read tiled sensor planes,
both uncompressed and Compression-7, including partial edge tiles. Per
**DEC-056** this is allocation-free, through a clipped, strided reconstruct
window. The three Pixel phone DNGs must unpack bit-exact against dnglab, and
every strip file must stay unchanged.

## Context the Receiving Agent Needs

### Primary

- **Spec (the contract):** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-058-tiled-plane-decode-tile-tags-placement-and-edge-tiles-uncomp.md`.
  Read the design-time probe table and findings first. The probe already
  established three things, so you will not have to rediscover them:
  - Today's decoder decodes single Pixel tiles bit-exact against dnglab: interior,
    right-edge and corner tiles. P=16 and Nf=2 work, and the codec needs no change.
  - Every one of the 192 tiles per file is a full 256×256 frame, edge tiles
    included.
  - Sensor selection already finds the SubIFD.
- **Decision:** `./decisions/DEC-056-tiled-planes-decode-in-place-through-a-clipped-strided-window.md`.
  Do not reopen its rejected alternatives: no scratch buffer, no caller scratch,
  no new entry point.
- **Stage:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/stages/STAGE-015-tiled-dng-plane-reader-uncompressed-and-lossless-jpeg-tiles.md`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` (lints, fuzz, msrv, the
  corpus env var).
- **The fixture writer (SPEC-057):** `tests/support/dng_writer.rs`.
  - Use `Organisation::Tiles { tile_width, tile_length, edge_fill }`.
  - Uncompressed tiles come from `Payload::Samples`.
  - Compression-7 tiles come from `Payload::Encoded { compression: 7, chunks }`,
    with one `tiff::lj92_strip_components(..)` chunk per tile, each encoding the
    full tile including padding.
  - The writer needs `tiff.rs` mounted as `mod tiff`.
- **The probe's per-tile checksums** (Pixel 4a, as one-strip DNGs) are a
  debugging aid:
  - tile 0: `a340b03173c939060b228de1c1b49b29`
  - tile 15: `a3cf5c3297d02d88076426ac03311423`
  - tile 191: `c9c984f5bfa594dd5f70c676c3a09180`
  - Pixel 7 Pro tile 191: `1dbd6e768c13c3f7179dbbea470de6a5`

### Decisions that apply

- `DEC-056`: in-place window decode, discard right-of-plane differences, stop after
  the last visible row, bind each tile's frame shape to the tile.
- `DEC-016`: `unpack_into` allocates nothing. This must remain true.
- `DEC-008`: per-row packing rules. Rows that aren't a whole number of bytes are
  refused, per the SPEC-057 probe.
- `DEC-047`: frame-shape binding, now applied per tile.
- `DEC-050`: validate-before-allocate parity. `validate` and `unpack_into` share
  one structural pass.
- `DEC-051`: `Error::kind()` has no wildcard. Every new variant must be classified.
- `DEC-012`: tile tags are structural; a malformed one fails the read.
- `DEC-049`: the Pixel files must still be refused at develop (mandatory GainMap).
- `DEC-003`: corpus bytes are never committed.
- `DEC-009`: run the negative control before each red-proof.

### Constraints that apply

- `no-panics-on-untrusted-input`: every hostile tile table is a typed error. The
  crate-root lints apply.
- `oracle-must-be-shown-red`: the window-clip red-proof (AC7).
- `provenance-recorded-per-algorithm`: the ledger row cites TIFF 6.0 §15, DNG 1.7,
  T.81 Annex H and DEC-056.
- `no-copyleft-dependencies`: never read rawler, LibTIFF or the DNG SDK. dnglab is
  a black box only.
- `test-before-implementation`: write the failing tests first.
- `one-spec-per-pr`.

### Prior related work

- **SPEC-057** (#199, #200): the fixture writer.
- **SPEC-046 / DEC-037:** Nf=2 decode.
- **SPEC-039 / DEC-050:** validate-before-allocate.
- **SPEC-053 / DEC-047:** shape binding.
- **SPEC-054 / DEC-049:** required processing.

## Expected Deliverables

- SPEC-058 AC1–AC11, with every named failing test present and passing.
- **Corpus, machine-local:** copy the three Pixel files from
  `~/PSeven/experiments/crustimg_redo_plus/data.lfs/Google/<model>/` into
  `$IRRADIANCE_CORPUS_DIR` (`~/PSeven/experiments/crustimg_redo_plus/images`), under
  `GOOGLE-PIXEL-4A/`, `GOOGLE-PIXEL-3A/` and `GOOGLE-PIXEL-7-PRO/`. Copy them; do
  not symlink. Add the manifest entries with the pinned values from the spec's
  table. Never `git add` a corpus file.
- **Run the tier-B tests with the corpus present.** Report that they ran rather than
  skipped, and give the three Pixel checksums plus confirmation that every
  pre-existing corpus checksum is unchanged.
- **Fuzz:** run each plane-reaching target for at least 10 minutes with the new
  tiled seeds. Report the run counts and that there were no crashes.
- **Gates all green:**
  - `just test` (it already passes `--all-features`; do not add the flag)
  - `just lint-ci`
  - `just lint-no-allow`
  - `just fmt-check`
  - `just validate`
  - `just doc-check`
  - `just deny`
  - `just deny-fuzz`
  - `just msrv`
  - `just cost-audit`
  - `just gate-audit`
- A PR against `main` from a fresh branch such as `claude/spec-058-build`. The PR
  body references HANDOFF-129, SPEC-058, STAGE-015, PROJ-004 and DEC-056/016/047.
- **Commit-message bodies are plain prose.** No code tokens such as arrows,
  `#[...]` or `--flags`; the push ruleset rejects them. PR bodies are fine.
  `Cargo.lock` is gitignored; never add it.
- Never merge, tag, publish, enable auto-merge or force-push.

## Out of Scope

- Applying the GainMap or any opcode (SPEC-059/060). The Pixel files must still be
  refused at develop.
- Fractional or malformed `BlackLevel` on the Pixel 7 Pro. Record it as a finding if
  it bites.
- New compression schemes, PlanarConfiguration 2, or parallel tile decode.
- A scratch buffer or a new public entry point. If the window argument seems to
  fail in the code, stop and report it.
- A version bump or release.

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

- **Branch / PR:** `claude/spec-058-build`, PR __PR__
- **Completed at:** 2026-11-11
- **All acceptance criteria met?** yes. AC1 through AC11 are all met. Two placements differ from the spec's Failing Tests list; see Deviations.
- **Window argument held in the code.** Both facts DEC-056 rests on are true in `src/ljpeg.rs`:
  - `decode_difference` takes only the Huffman table, never a prediction.
  - Predictors 1–7 read only left, above and above-left.

  The window needed no scratch buffer and no new public entry point.

**What changed**

- **`src/ifd.rs`:** the four `TAG_TILE_*` constants and four `Sensor` fields. The tags are structural: they are in `is_structural_tag`, and the structural-tag table test now covers 15 tags.
- **`src/ljpeg.rs`:**
  - Added `Window` and `check_window`, plus `reconstruct_window`, `reconstruct_two_component_window` and `decode_window_into`.
  - The old `reconstruct`, `reconstruct_two_component` and `decode_into` keep their signatures and length checks. They now call the window versions with `Window::dense`.
  - The clip decision is one method, `Window::is_visible`. Discarded samples still count toward restart intervals.
  - The point transform is applied to visible samples only. Decoding stops after the last visible row.
- **`src/plane.rs`:**
  - `select_layout` → `tile_grid` → `TileWalk` / `check_tiles`, wrapped by `check_layout`, which both `validate` and `unpack_into` call.
  - Then `unpack_tiles_uncompressed` and `unpack_tiles_jpeg`. The latter does a per-tile `check_strip_shape(frame, TileWidth, TileLength)` before decoding.
  - The strip path is unchanged.
- **`src/lib.rs`:** `InvalidTileLayout` (Corrupt), `StripsAndTiles` (Corrupt) and `UnsupportedTileRowBits` (Unsupported). `UnsupportedStripLayout`'s text no longer says tiles are unsupported. DEC-051's table note and `tests/error_kind.rs` are updated: 59 variants, 12/19/28.
- **`irr ifd`** prints the tile fields.

**Evidence**

- **Failing tests first.** The new `tests/plane_tiles.rs` ran red on the pre-implementation tree, 10 of 10 failing. Only the type additions were in place: the Sensor fields and the error variants.
- **Tier B ran with the corpus present.** `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, nothing skipped. Pixel plane MD5s, all equal to the pinned dnglab checksums:
  - 4a: `d1830234ac3f32b07ff73792f0faeabb`
  - 3a: `eb736a494bdfd2fc5ec799fb798ed2c3`
  - 7 Pro: `eb72cb300a27c4bcab745702542269de`

  Every pre-existing corpus checksum is unchanged. `plane_md5_matches_the_pinned_raw_checksum` passed for all 6 uncompressed files and the 2 held Compression-7 strip files, plus the 3 Pixel files.
- **Corpus copies.** The three Pixel files were copied, not symlinked, from `data.lfs/Google/<model>/` into `GOOGLE-PIXEL-4A/`, `GOOGLE-PIXEL-3A/` and `GOOGLE-PIXEL-7-PRO/`. Their sha256 values match the spec. Nothing from the corpus is committed.
- **AC7 red-proof** (`tests/plane_oracle.rs::window_clip_red_proof`):
  - **Negative controls first.** The in-process honest digest, then the unmutated staged-copy apparatus.
  - **Then the mutation:** `Window::is_visible` made `<=`, which keeps one extra column. It turned red on the tier-A tiled fixture and on all three Pixel files, with mutant digests `019c7c51…`, `242dc49d…` and `8df8b249…`.
  - The existing ljpeg two-component red-proof also now covers the Pixel files, and goes red on them.
- **AC8.** Develop refuses all three Pixel files, through both `develop_color_into` and `develop_into`, with `UnsupportedMandatoryOpcode { id: 9 }` (GainMap, `ErrorKind::Unsupported`).
- **AC9 fuzz** (`cargo +nightly fuzz`, 630 s each, run concurrently, seeds include 5 writer-built tiled fixtures per target):
  - `plane`: 36,288,821 runs, 0 crashes
  - `develop`: 31,584,752 runs, 0 crashes
  - `demosaic`: 29,868,019 runs, 0 crashes
  - `ljpeg` (its reconstruct loops changed shape): 34,629,975 runs, 0 crashes

  All four exited 0, with no artifacts.
- **AC11 gates.** All green: `just test` (with the corpus), lint-ci, lint-no-allow, fmt-check, validate, doc-check, deny, deny-fuzz, msrv, cost-audit and gate-audit.

### Cost self-report

- **Tokens (total):** 31,088,670 (a floor; see notes)
- **Estimated USD:** $21.47
- **Duration (minutes):** ~120
- **Source of the number:** this session's transcript (`basis: transcript`).
  - Deduplicated by `message.id`, keeping the max `output_tokens` per message. All messages are `claude-opus-5-5`, with no subagents.
  - Priced per component at opus-5-5 rates: $5 per Mtok in, $25 per Mtok out, cache read ×0.10, and 1-hour cache write ×2.0.
  - Measured just before the handback commit, so the final commit and PR messages are not included. It is a floor.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-051's "later additions" note records the three new variants' kinds.
- **Deviations from spec:**
  - **`window_clip_red_proof` lives in `tests/plane_oracle.rs`, not `tests/plane_tiles.rs`.** That file already holds the copy-and-rebuild apparatus. It also runs on a tier-A writer fixture, so CI exercises it too.
  - **The AC5 tile-count overflow case is asserted on `validate` only.** It uses 641 × 6,700,417 one-pixel tiles, which is 2^32 + 1 and wraps to 1 in `u32`. A correctly sized `dst` would be about 8 GiB, and `unpack_into`'s `dst`-length check comes first by the documented precedence.
  - **The transposed-frame case is `Ok` from `validate`.** `validate` does not decode, which is its documented contract for strips too. It is `LjpegPlaneShapeMismatch` from `unpack_into`.
  - **AC8's refusal is `UnsupportedMandatoryOpcode { id: 9 }`, not `UnsupportedRequiredOpcode`.** GainMap is not a recognized opcode, so it is refused as unknown-and-mandatory, which is still the DEC-049 check.
  - **The Pixel files were added to `DECODABLE_COMPRESSED` in `tests/plane_oracle.rs`.** They therefore also ride the determinism pin and the ljpeg red-proof. They are `CFA_CHECKED_ELSEWHERE` in `tests/ifd_reader.rs`, and `MANIFEST_ENTRIES` went from 9 to 12.
  - **README's tested-DNG-modes table was updated.** It said tiles were refused.
  - **Two legacy tests changed with the behaviour.**
    - `tests/plane_unpack.rs::tiled_compression7_plane_is_unsupported_strip_layout` pinned the old refusal. It is now `a_hand_built_single_tile_compression7_plane_decodes`, which asserts the samples, using the hand-placed `tiff.rs` builder so it stays independent of the writer.
    - `tests/metadata_oracle.rs` skips the Pixel files by name in `CFA_METADATA_NOT_YET_ORACLED`, with a stated reason; see Follow-up.
    - The tile alias-bound variables are named differently from the strip ones, so `tests/plane_validate.rs`'s red-proof needle `if declared > file_len {` still matches exactly the strip guard.
- **Follow-up work identified:**
  - **Finding (pre-existing, not introduced here).** The Compression-7 path is not strictly allocation-free: `ljpeg::parse_headers` returns a `Vec<HuffTable>`, and each table owns a `Vec<u8>`. That is small, header-sized, and per strip or tile, but DEC-016's "allocates nothing" is literally true only for uncompressed planes. Worth a sentence in DEC-016, or a fixed-size table array, in a later spec. The AC6 grep test pins the window section and all of `src/plane.rs`, and documents this.
  - **The Pixel 7 Pro's malformed `BlackLevel` did not bite** the plane decode, as expected.
  - **No metadata oracle covers the Pixel files yet.** They carry a 4-value per-channel `BlackLevel`, which the scalar oracle in `tests/metadata_oracle.rs` cannot compare, and the colour oracle does not list them. They are skipped by name, with a stated reason, in `CFA_METADATA_NOT_YET_ORACLED`. Adding them to `tests/colour_metadata_oracle.rs`, cross-checked against exiftool and dnglab, belongs before or with STAGE-016's GainMap work.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The probe table and the DEC-056 argument were exactly what was needed. Two places made me guess:
   - AC5's "both validate and unpack_into" for the overflow and transpose cases, which `validate`'s own contract makes impossible as written.
   - AC8's expected variant name.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two things in other tests:
   - The manifest-partition tests in `tests/ifd_reader.rs` and `tests/plane_oracle.rs`, which every new manifest entry trips.
   - The `plane_oracle` red-proof's literal needle, `neighbour(out, slot, 2)`, which a reconstruct refactor must preserve.

   Naming them in the handoff would save a loop.

3. **If you did this task again, what would you do differently?**
   — Add the manifest entries and the partition-list updates in the same step as the types, so the first full-suite run is already clean.
