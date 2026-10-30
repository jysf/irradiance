---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-027
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-003]             # tier-B corpus policy: the manifest is committed, the files never are
  constraints: [no-panics-on-untrusted-input]
  related_specs: [SPEC-002, SPEC-013, SPEC-025]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-024]             # a CFA file is a meaningful fixture once the gate accepts it

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Registers the Bayer fixtures the oracle specs run against and keeps the parser panic-free on CFA input."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-027: Corpus + fuzz: register the uncompressed Bayer DNG in the corpus manifest and extend the fuzz corpus; panic-free on CFA input

> **DESIGN — `cycle: design`.** Approach + failing tests below; a build session
> makes the Failing Tests pass. STAGE-006's housekeeping spec: move the Bayer
> fixtures into the corpus manifest the oracle already reads, and give the CFA
> parse path a fuzz seed.

## Context

The two uncompressed Bayer DNGs are already physically placed in
`$IRRADIANCE_CORPUS_DIR` (`CANON-EOS-5D-MARK-III/5G4A9394.DNG`,
`NIKON-D300S/_DSC4615.DNG`). This spec registers them in the corpus manifest
(SPEC-002's reader) so the oracle specs locate them via the manifest rather than a
hard-coded path, and extends the fuzz corpus (SPEC-003/011) with a CFA seed so the
parser is exercised on hostile Bayer input. The manifest registration is the part
SPEC-025/026 lean on, so it should land first within the stage.

## Goal

Register the two Bayer DNGs in the corpus manifest (honouring skip-when-absent),
and extend the fuzz corpus with a CFA seed so the parser stays **panic-free** on
truncated/hostile Bayer input.

## Inputs

- **Files to read:**
  - `tests/corpus/manifest.toml` — the committed tier-B index (SPEC-002). Each
    `[[file]]` carries `path`/`tier`/`bytes`/`sha256`/`licence`/`source` +
    `[file.oracle] raw_checksum`; copy an existing mono entry's shape. The header
    (`:14`–`:46`) states the required fields and the "`licence` has no second
    chance" rule.
  - `tests/corpus_manifest.rs` — `MANIFEST_ENTRIES` (currently `7`, `:18`) and the
    reader's per-entry field assertions; `corpus_files_match_their_pinned_sha256`
    verifies each present file's on-disk sha256.
  - `tests/plane_oracle.rs` — `DECODABLE` (4 mono paths, `:83`); the manifest-driven
    oracle `plane_md5_matches_the_pinned_raw_checksum` (`:591`, reads **only**
    `entry.oracle.raw_checksum`); `compressed_files_are_skipped_by_name` (`:115`,
    the `DECODABLE + SKIPPED_COMPRESSED == manifest.files.len()` partition); and the
    SPEC-025 Bayer block (`:944`–`:1089`): the `CANON_BAYER`/`NIKON_BAYER` +
    `CANON_RAW_CHECKSUM`/`NIKON_RAW_CHECKSUM` constants, the `require_bayer` helper,
    the bespoke Bayer oracle test, and the byte-aligned red-proof.
  - `tests/plane_unpack.rs:882`–`~945` — the SPEC-028 (Nikon 12-strip) and SPEC-029
    (Canon above-white) corpus tests: direct-path, skip message literally
    "set $IRRADIANCE_CORPUS_DIR … (SPEC-027 registers it)".
  - `tests/support/tiff.rs` — the hand-built tier-A TIFF DSL (`short`/`long`/
    `at_offset`/`Ifd`/`tiff`; `sensor_entries`/`valid_sensor` build the `LINEAR_RAW`
    plane; `PHOTOMETRIC = 262`, `:160`). There is **no CFA builder yet** — add one.
  - `examples/fuzz-seeds.rs` — writes `fuzz/seeds/{ifd,plane,develop,…}`; the `ifd`
    seeds ARE the `tests/support/tiff.rs` fixtures, the same set `tests/ifd_reader.rs`
    asserts against (a fixture cannot be added to one lane and forgotten by the other).
  - `fuzz/fuzz_targets/ifd.rs` + `plane.rs` — the panic-free contract
    (`no-panics-on-untrusted-input`): for every byte string every entry point
    returns and none panics. `ifd` already walks `sensor_candidates()`/`sensor()`;
    `plane` drives `unpack_into`.
- **Corpus:** `CANON-EOS-5D-MARK-III/5G4A9394.DNG` (single-strip, 16-bit, 5920×3950,
  `raw_checksum a18235d6abced4df9948b6fbe91aba7c`) and `NIKON-D300S/_DSC4615.DNG`
  (12-strip, 16-bit, 4352×2868, `raw_checksum b8e8025d370cc2e97f675e63a0e6bbdb`)
  under `$IRRADIANCE_CORPUS_DIR`.

## Outputs

- **`tests/corpus/manifest.toml` — two new `[[file]]` entries**, one per Bayer DNG,
  with all required fields: `path`, `tier = "b"`, `make`/`model`, `bytes` and
  `sha256` **measured from the file on disk** (not guessed), `licence` + `source`
  (see Notes — provenance has no second chance), and `[file.oracle] raw_checksum` =
  the pinned dnglab value above. `pgm_bytes`/`strip_bytes` are optional and
  **omitted** (a 12-strip plane has no single layer-0 strip figure; the oracle keys
  on `raw_checksum` only).
- **`tests/corpus_manifest.rs`** — `MANIFEST_ENTRIES` `7 → 9`.
- **`tests/plane_oracle.rs` — fold the Bayer oracle onto the manifest:**
  - Add `CANON_BAYER` + `NIKON_BAYER` to `DECODABLE` (→ 6); the manifest-driven
    `plane_md5_matches_the_pinned_raw_checksum` then covers both, reading each
    expected checksum from the new manifest entry.
  - Update `compressed_files_are_skipped_by_name`: partition is now
    `DECODABLE (6) + SKIPPED_COMPRESSED (3) == 9`.
  - **Delete** the bespoke direct-path Bayer oracle test and the now-dead
    `CANON_RAW_CHECKSUM`/`NIKON_RAW_CHECKSUM` constants + `require_bayer` helper.
  - **Keep** the byte-aligned red-proof, but source its reference file + expected
    checksum from the manifest (`manifest.get(NIKON_BAYER)` → `entry.require(&root)`
    + `entry.oracle.raw_checksum`), not a hard-coded constant.
- **`tests/plane_unpack.rs`** — the SPEC-028/029 corpus tests locate the file via
  the manifest (`manifest.get(path)` + `entry.require(&root)`); drop the
  "SPEC-027 registers it" forward-reference from the skip message now that it does.
- **`tests/support/tiff.rs` — a synthetic CFA fixture family:** `cfa_sensor_entries`
  / `valid_cfa_sensor(order)` (`PhotometricInterpretation = 32803`, `SamplesPerPixel
  = 1`, `CFARepeatPatternDim = [2,2]`, `CFAPattern` RGGB, `CFAPlaneColor`,
  `CFALayout`, a small uncompressed strip) + ≥1 truncated/hostile CFA variant (e.g.
  `CFAPattern` count past EOF, or an absurd `CFARepeatPatternDim`).
- **`examples/fuzz-seeds.rs`** — emit the CFA fixture(s) into `fuzz/seeds/ifd`
  (and `fuzz/seeds/plane` if the valid CFA fixture carries an unpackable strip);
  **commit the regenerated `fuzz/seeds/ifd/`**.
- **`tests/ifd_reader.rs`** (the `ifd` seed test lane) — assert the new CFA fixture
  parses, matching how the existing fixtures are asserted there.

## Acceptance Criteria

- [ ] Both Bayer DNGs are registered in `tests/corpus/manifest.toml` with every
  required field and `[file.oracle] raw_checksum` = their pinned dnglab value;
  `corpus_manifest_parses` passes with `MANIFEST_ENTRIES == 9`, and with the corpus
  present each file's on-disk `sha256` matches (`corpus_files_match_their_pinned_sha256`).
- [ ] The Bayer plane oracle is manifest-driven: Canon + Nikon are in `DECODABLE`,
  `plane_md5_matches_the_pinned_raw_checksum` hashes both and matches the manifest
  checksum, and **no hard-coded Bayer path/checksum constant remains** in
  `tests/plane_oracle.rs` (save a single path key for the red-proof reference, itself
  resolved through the manifest).
- [ ] `compressed_files_are_skipped_by_name` still accounts for every entry
  (`DECODABLE 6 + SKIPPED_COMPRESSED 3 == 9`) and, corpus present, classifies all 9
  by real Compression.
- [ ] The byte-aligned Bayer red-proof still reddens (mutant ≠ honest, green
  control), sourcing its reference from the manifest; the mono oracle + sub-byte
  red-proof stay green.
- [ ] A synthetic CFA fixture is in the `ifd` fuzz seed set and the `ifd` test lane;
  `cargo run --example fuzz-seeds` regenerates `fuzz/seeds/ifd` including it.
- [ ] Parser is panic-free on CFA input: a well-formed synthetic CFA container
  parses to a `sensor()` carrying a `Cfa`, and truncated/hostile CFA variants return
  a typed `Err` (never panic). Tier-A — no corpus, no tools.
- [ ] All gates green; a bounded `ifd` + `plane` fuzz run over the new CFA seed finds
  0 crashes. Skip-when-absent stays loud.

## Failing Tests

- **`tests/corpus_manifest.rs`**
  - `corpus_manifest_parses` — with `MANIFEST_ENTRIES = 9`, asserts 9 entries, each
    with required fields (fails until the 2 Bayer entries are added).
  - `corpus_files_match_their_pinned_sha256` — covers the 2 new entries when present.
- **`tests/plane_oracle.rs`**
  - `plane_md5_matches_the_pinned_raw_checksum` — now iterates 6 `DECODABLE` paths
    incl. `CANON_BAYER`/`NIKON_BAYER`; each plane MD5 == its manifest `raw_checksum`.
  - `compressed_files_are_skipped_by_name` — partition `6 + 3 == 9`.
  - the byte-aligned red-proof — same assertion, manifest-sourced reference.
- **`tests/ifd_reader.rs`** (or the `ifd` seed lane)
  - `"a well-formed CFA container parses to a CFA sensor"` — `valid_cfa_sensor`
    parses; `sensor().color_filter` is `Some(Cfa …)`.
  - `"a truncated CFA container returns without panicking"` — each hostile CFA
    variant → typed `Err`, no panic.
- **`examples/fuzz-seeds.rs` / `tests/support/tiff.rs`**
  - the `ifd` seed set includes the CFA fixture(s); regenerating writes them to
    `fuzz/seeds/ifd`.

## Non-Goals

- Any new camera beyond the two Bayer bodies, or a ColorChecker — STAGE-008.
- Native vendor containers or lossless-JPEG SOF-3.
- Changing WHAT the oracle asserts — SPEC-025 owns the Bayer plane claim, SPEC-026
  the metadata claim. This spec only MOVES the Bayer fixtures into the manifest and
  adds a CFA fuzz seed; the pinned checksum values do not change.

## Notes for the Implementer

- **Provenance has no second chance (DEC-003).** Both DNGs trace to the
  **raw.pixls.us** sample library (CC0-1.0): the Canon is an Adobe-written DNG, the
  Nikon a `dnglab`-converted copy of a pixls NEF. Fill `licence = "CC0-1.0"` and a
  `source`/`provenance` naming raw.pixls.us + the body — **confirm the exact sample
  with the maintainer before writing**, since the manifest header forbids a later fix.
- **The three counts interlock.** `MANIFEST_ENTRIES` (corpus_manifest.rs),
  `DECODABLE.len()`, and the `compressed_files_are_skipped_by_name` partition must all
  agree at 9 / 6 / 3. Change them together or the suite fails by design — the point is
  that a new entry can't land with nothing consuming it.
- **Measure, don't guess.** `bytes` and `sha256` come from the file on disk; the
  reader re-verifies sha256 against the bytes. `raw_checksum` is already pinned above.
- **The fuzz seed is synthetic, never the corpus file (DEC-003).** Build the CFA
  fixture from TIFF 6.0 in `tests/support/tiff.rs` (reuse `sensor_entries`'s shape,
  swap `PHOTOMETRIC` to CFA 32803, add the four CFA tags). A truncated 46 MB Canon
  frame would still be a Canon frame — tier B is never committed.
- **Reuse, don't re-derive.** `decode_and_hash`, `assert_plane_matches`,
  `CorpusFile::require` already do the work; this spec mostly MOVES references from
  constants to `manifest.get()`. The red-proof apparatus is SPEC-025's and stays —
  only its reference lookup changes.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
