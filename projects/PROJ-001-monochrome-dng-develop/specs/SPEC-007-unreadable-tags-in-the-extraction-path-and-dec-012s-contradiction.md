---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-007
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved         # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-012]                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-004]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "a DNG-legal file must not become unreadable because of one tag"

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
  sessions:
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 19480728
      estimated_usd: 8.62
      duration_minutes: 75
      recorded_at: 2026-08-21
      notes: "Build cycle for SPEC-007 (HANDOFF-017), commit pending on feat/spec-007-extraction-tolerance, not merged. Implemented DEC-012's amended structure/interpretation split in sensor(): added a private Container::cost_the_field<T> helper that catches an Err from a leaf accessor (scalar()/array(), which still report malformed tags honestly) for an interpretation-class tag, records the tag in Sensor::malformed_tags, and continues — the composite-accessor catch DEC-012 prescribes, same shape as SensorMatch's per-IFD tri-state. Applied it to Orientation (closes FU-16 — a malformed tag on IFD0 no longer discards an already-located SubIFD plane), BlackLevel, WhiteLevel, ActiveArea, DefaultCropOrigin, DefaultCropSize and BlackLevelRepeatDim. Structural fields (ImageWidth/Length, BitsPerSample, SamplesPerPixel, Photometric, Compression, RowsPerStrip, StripOffsets, StripByteCounts) were already wired with a bare `?` and are UNCHANGED — each now carries a one-line DEC-012 classification comment instead. Added TYPE_RATIONAL (5) to uints(): reads the two-LONG numerator/denominator pair via checked_div/checked_rem (both report a zero denominator as None, so that shape and a non-integral ratio are the same Error::MalformedRationalValue — a new Error variant, Displayed, added to the module's own exhaustive-Display test). This closes FU-17 for the well-formed case (a DNG-legal RATIONAL DefaultCropSize now reads correctly) and, combined with cost_the_field, for the malformed case (a malformed RATIONAL costs the field, not the file). Fixed FU-20 by short-circuiting is_sensor_ifd: each identifying tag is evaluated (not just read) before the next is even touched, so a readable, disqualifying NewSubfileType != 0 returns SensorMatch::No immediately instead of reading Photometric/SamplesPerPixel and possibly reporting Unreadable for an IFD that was never a real candidate. NO NEW DEC: every change is a direct transcription of DEC-012's amendment and the spec's acceptance-criteria table (structure/interpretation classification, the cost_the_field shape copying SensorMatch, RATIONAL read from TIFF 6.0 §2 Types) — same finding SPEC-004's build recorded for FU-11's SensorMatch, not a new judgment call this build made. FIXTURES IN BOTH DIRECTIONS (all hand-built, src/ifd.rs unit tests — no corpus file exercises any of these paths): malformed_interpretation_tag_costs_only_the_field (BlackLevel, bad field type, file still reads, tag recorded) paired with malformed_structural_tag_is_still_fatal (identical bad-field-type shape on RowsPerStrip, whole sensor() call fails) is the direct boundary pin the handoff asked for. rational_default_crop_size_reads_or_costs_the_field covers both the exact-integral RATIONAL (reads correctly, matches the Q2 Monochrom's real crop size) and the zero-denominator malformed case in one fixture pair; a_non_integral_rational_also_costs_the_field covers the other malformed shape (5/2) separately. malformed_orientation_on_ifd0_keeps_the_plane is FU-16's literal reproduction fixed: sensor_candidates() == [1] (the plane still located on the SubIFD), sensor() succeeds, orientation is None, malformed_tags == [274]. candidates_malformed_names_only_candidates is FU-20's: two IFDs, index 0 disqualified by a readable NewSubfileType == 1 (must never be named) with an ALSO-malformed Photometric, index 1 a genuine candidate with only a malformed Photometric — NoSensorIfdCandidatesMalformed's candidates == [(1, 262)] exactly, not [(0, 262), (1, 262)]. RowsPerStrip's structural classification is confirmed only by the hand-built fixture above; every corpus file remains single-strip (unchanged from SPEC-004's observation) so this classification is still untested by real data — did not soften it, per the handoff's explicit warning. Added a RATIONAL-bearing well-formed fixture (rational-default-crop-size) to tests/support/tiff.rs's shared list, so it is BOTH a tests/ifd_reader.rs hostile-input-list entry (added to the `valid` set) AND a fuzz seed with no second edit — the file's own stated convention. Updated the one existing test that assumed RATIONAL was unimplemented (a_readable_type_this_module_does_not_widen_is_typed) to use SRATIONAL (field type 10) instead, which does remain unimplemented. TEN GATES, all green: cargo fmt --check; cargo clippy --all-targets --all-features -D warnings; cargo test --all-features, 58 passed (37 lib + 0 irr bin + 9 corpus_manifest + 12 ifd_reader + 0 doc, SUMMED across five Running lines, confirmed via `cargo test --lib/--bin irr/--test corpus_manifest/--test ifd_reader/--doc -- --list` per-target counts = 37/0/9/12/0); just msrv (1.90.0 via the rustup shim, no PATH= needed); just deny and just deny-fuzz (both 'licenses ok'); just lint-red-proof (control clean exit 0, injection rejected exit 101, all five lints fired, still fired without -D warnings); just lint-no-allow (clippy --lib -F x5, exit 0). PLUS: scripts/cost-audit.sh (all shipped specs recorded), scripts/decisions-index.sh --check (no INDEX.md, nothing to sync), just validate (7 artifacts, valid front-matter), just decisions-audit --changed (DEC-004/008/009/010/011/012 all advisory-flagged as touching changed paths, all confirmed consistent — DEC-012 is what this build implements; none of the others' rules were violated). Confirmed EACH of the five named Failing Tests exists via `cargo test --lib -- --list` (grep count 1 each, none zero-match) before trusting their green. Fuzz: regenerated fuzz/seeds/ifd (26 seeds, was 25 — the one addition is the new RATIONAL fixture; all 25 prior seeds byte-identical) and ran `PATH=\"$HOME/.cargo/bin:$PATH\" ~/.cargo/bin/cargo +nightly fuzz run ifd fuzz/corpus/ifd fuzz/seeds/ifd -- -max_total_time=60`: 11,553,927 runs in 61s, zero crash artifacts (fuzz/artifacts/ifd/ empty) — this covers the widened uints() because the existing target's loop already calls container.uints(entry) on every entry regardless of type, so TYPE_RATIONAL is reached the same way BYTE/SHORT/LONG/UNDEFINED already were; no target-code change was needed, only seed and library changes. Corpus-dependent tests (tag_model_matches_exiftool, ifd_reaches_sensor_plane, orientation_is_per_frame) passed unchanged against all 7 real files with IRRADIANCE_CORPUS_DIR set — expected, since DEC-012 and this spec both state no corpus file exercises a malformed interpretation tag or a RATIONAL-typed one, so well-formed real files take the identical code path as before (cost_the_field's Ok(value) arm is a pure pass-through). Extended docs/provenance-ledger.md's existing src/ifd.rs row (same class, 1 - specification: TYPE_RATIONAL from TIFF 6.0 Types directly, DEC-012's split is a control-flow refinement, not a new algorithm) rather than adding a new row. tokens_total is a transcript sum, DEDUPED BY message.id, measured via the same methodology SPEC-004's verify used (reading this session's own transcript at ~/.claude/projects/<path-slug>/<session-id>.jsonl, session id taken from the scratchpad path): 174 usage objects, 87 distinct ids, raw 38,007,198 vs deduped 19,480,728 = 1.95x, 98.2% cache-read (all cache-writes were the 1-hour ephemeral tier, confirmed from one usage object's nested cache_creation field, not the 5-minute tier) - within the 1.61x-2.51x band SPEC-004's verify established, not a new extreme. It is a FLOOR, captured before this session closes, per the handoff's explicit warning not to let it go null-with-no-source the way SPEC-004's build did. estimated_usd is a computed estimate (Sonnet published rates: $3/M input, $15/M output, $6/M cache-write-1h, $0.30/M cache-read), not a harness-reported figure — flagged so a reader does not mistake it for measured. Did NOT run handback-sync, per the handoff (finding 15)."
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 7830000
      estimated_usd: 19.35
      duration_minutes: 55
      recorded_at: 2026-08-21
      notes: "Verify cycle for SPEC-007 (HANDOFF-018), reviewing 0de18d4 on feat/spec-007-extraction-tolerance; branch at a94f5b4, main untouched at 99086fb, not merged. VERDICT: APPROVED at 0de18d4 — all six acceptance criteria met, 7 follow-ups (FU-1..FU-7), 0 ship-blockers. TEN GATES RUN BY ME, all green: cargo fmt --check; cargo clippy --all-targets --all-features -D warnings; cargo check --all-targets --all-features; cargo test --all-features 58 passed (37 lib + 0 irr + 9 corpus_manifest + 12 ifd_reader + 0 doc, SUMMED across five targets with IRRADIANCE_CORPUS_DIR set, corpus reporting 7/7 files present); cargo clippy --lib -F x5 exit 0; cargo deny check licenses AND cargo deny --manifest-path fuzz/Cargo.toml check licenses (both licenses ok); ~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features; scripts/lint-red-proof.sh (control clean exit 0 -> injection rejected exit 101 -> all five lints fired, still fired without -D warnings — I WATCHED IT GO RED, check 9); cargo build --release. Plus cost-audit, decisions-index --check, just validate (7 artifacts), just decisions-audit --changed. FUZZ RE-RUN BY ME (check 10): 10,768,231 runs in 61s, cov 620 ft 2048, fuzz/artifacts/ifd empty; confirmed the target actually reaches the widened path — fuzz/fuzz_targets/ifd.rs:47 calls uints() on every entry regardless of field type and :53 calls sensor(), so both TYPE_RATIONAL and cost_the_field are fuzzed; seeds regenerate byte-identical (26 seeds, diff -rq clean). NAMED TESTS: each of the five confirmed to EXIST via per-target -- --list before its green was trusted (anchored grep (^|::)<name>: test$ — a bare ^<name> pattern returns 0 for all five, because lib tests list as ifd::tests::<name>; that near-miss is recorded because it reads as 'the tests do not exist'); all five live in the lib target only, each matched exactly 1, each passes. SCRUTINY 1 (scope extension): CORRECT and nothing structural swept in — cost_the_field has 8 call sites (src/ifd.rs:1103,1112,1120,1125,1136,1141,1152,1158) covering 7 distinct tags (Orientation x2, BlackLevel, WhiteLevel, BlackLevelRepeatDim, ActiveArea, DefaultCropOrigin, DefaultCropSize), a literal 7-for-7 match with DEC-012 amendment's Interpretation row; the 9 remaining bare-? sites are all Structure-class (ImageWidth:1169, ImageLength:1170, BitsPerSample:1171, SamplesPerPixel:1173, Photometric:1175, Compression:1178, RowsPerStrip:1184, StripOffsets:1186, StripByteCounts:1187), unchanged from main, and the three cited lines 1173/1178/1184 are exactly where the handoff said. SCRUTINY 2 (cost_the_field): leaves stay honest (scalar/values/uints/array all still return Result; array() returns Err WITHOUT pushing and pushes only on the Ok(None) count-mismatch branch, so no double-record on that path); cost_the_field is private, takes no &self, called from sensor() and nowhere else. Nothing is dropped unrecorded — mutation M8 (delete malformed.push) compiled and turned FOUR tests red. But the INVERSE is unguarded: FU-1 and FU-2. SCRUTINY 3 (RATIONAL): zero denominator exits via checked_div/checked_rem both returning None before any unchecked division executes, no raw / or %; legitimate integral RATIONAL is genuinely READ not merely tolerated — mutation M12 (success arm can never push) turned rational_default_crop_size_reads_or_costs_the_field red; chunks_exact(8) cannot drop a partial value since payload() returns exactly count*8 bytes for type_size(5)==8. FU-5 is that the DIVISION is unpinned. SCRUTINY 4 (FU-20): MEASURED not reasoned — built irr at main 99086fb in a throwaway worktree and at HEAD, ran irr ifd over all 7 manifest files, diffed both transcripts IN FULL: EMPTY. Byte-identical including sensor_matches, sensor_ifd, every tag value, malformed_tags, layer0 closure and unpackable. Selection: six files matches [1] -> #1, K3III.PEF matches [0] -> #0 (its plane IS IFD0, which is what makes FU-1 a real corpus shape); K3III.DNG still reports malformed [50713]. Mutation M10 (delete the short-circuit) turned candidates_malformed_names_only_candidates AND a_reduced_resolution_linear_raw_ifd_is_not_the_sensor red. SCRUTINY 5 (tier_map): it should RECORD, not PREDICT — 0 for 2, wrong in both directions; the defect is the wiring, a prediction stamped into handoff.to_agent which handback-sync copies into cost.sessions[].agent, a field consumed as a measurement. Recommend tier_map become a dispatch hint only, to_agent start null, handback its sole writer. MUTATION TESTING (every mutation verified PRESENT via git diff and verified to COMPILE before its result was read — the trap that has failed five times here, twice this cycle): M1 RowsPerStrip tolerant -> malformed_structural_tag_is_still_fatal RED and only it (reproduces the orchestrator's result independently); M7 BlackLevel back to bare ? -> malformed_interpretation_tag_costs_only_the_field RED (both directions proven); M8 -> 4 red; M9 accept non-integral RATIONAL -> a_non_integral_rational_also_costs_the_field RED; M10 -> 2 red; M12 -> 1 red. BUT M2/M4/M5/M6 (Compression, StripOffsets, StripByteCounts, BitsPerSample each made tolerant) each compiled and left the FULL 58-test suite GREEN with the corpus present — the structural half of the boundary is pinned for RowsPerStrip ALONE (FU-3); Compression is the sharp one, since a softened Compression defaults to 1 and lets require_uncompressed() pass so STAGE-002 would read JPEG bytes as raw samples. Not a code defect — criterion 5 is met as written — a thin regression net. Recorded as NOT a gap: mutating SamplesPerPixel:1173 or Photometric:1175 also leaves the suite green and that is CORRECT, they are re-reads of tags is_sensor_ifd already read successfully for the selected IFD, so their ? can never fire — equivalent mutants, unkillable by construction. FU-4 (measured at HEAD vs main): uints() type gate is global, so widening it to TYPE_RATIONAL loosened the WALK — SubIFDs (330) as RATIONAL 400/2 was Err(UnexpectedFieldType) on main and is now ACCEPTED (ifds=2, candidates=[1], SubIFD walked), and StripByteCounts (279) as RATIONAL 28/2 now reads [14]; tag 330 is the case DEC-012 singled out as structural. Follow-up not ship-blocker for three measured reasons: it widens a looseness that already existed (uints() accepted TYPE_UNDEFINED/TYPE_BYTE for 330 on main, equally illegal — the gate has never been per-tag), no safety property moves (all bounds/cycle/depth guards still apply, 10.7M fuzz runs clean), and no wrong answer is produced (400/2 is a correct reading of an out-of-spec encoding; anything expressible as RATIONAL was already expressible as LONG). Provenance ledger row extended in place, honest class 1 - specification, RATIONAL cited to TIFF 6.0 §2 Types; no new dependency; no new DEC needed and the build's reasoning for that is right. tokens_total is a transcript sum DEDUPED BY message.id from this session's own JSONL (108 usage objects, 64 distinct ids; raw 13,161,712, deduped 7,736,717 = 1.70x, inside SPEC-004 verify's 1.61x-2.51x band; 97.5% cache-read; deduped cache-creation 143,213 entirely on the 1-hour tier, 5-minute tier 0, read from the nested cache_creation object not assumed), rounded UP to cover the turns spent writing the handback and committing, captured BEFORE the session closed. estimated_usd computed per-component at published Opus rates, NOT harness-reported — flagged because the repo's flat cost.rate_per_mtok 6.60 would give $51.06 for this session, 2.7x high on a 97.5%-cache-read profile; the build hit the same divergence and also went per-component, so it is now on record twice. Did NOT run handback-sync, per the handoff."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-007: Unreadable tags in the extraction path, and DEC-012s contradiction

## Context

`SPEC-004` closed `SPEC-003/FU-11` for the **selection** path — `is_sensor_ifd` is now a
`SensorMatch` tri-state, so a malformed identifying tag on one IFD no longer aborts
the scan of the others. Its verify found the **extraction** path still has the same
gap, twice:

- **`SPEC-004/FU-16`** — `sensor()` reads `Orientation` from `IFD0` with a bare `?`
  (`src/ifd.rs:1011`), so a malformed tag on a **non-sensor** IFD discards an
  already-located plane. Reproduced: `sensor_matches [1]`, then discarded.
- **`SPEC-004/FU-17`** — a **DNG-legal `RATIONAL`** `DefaultCropSize`/`DefaultCropOrigin`/
  `BlackLevel` makes the **whole file unreadable**: `uints()` (`src/ifd.rs:788`)
  returns `UnexpectedFieldType` and `sensor()` propagates it. Reproduced. This is
  fatal to the file, not a missing field — a severity the build's framing
  understated.

Neither is a regression; both are identical on `main`.

⚠ **`DEC-012` must be amended first.** Its principle forbids exactly this, and its
*table* sanctions it. A spec designed against it today would inherit a decision
that blesses the behaviour this spec exists to fix. The contradiction is recorded
on the DEC itself.

## Goal

Make the extraction path obey `DEC-012`'s principle: **a DNG-legal file must not
become unreadable because one interpretation tag is malformed or uses a legal type
we have not implemented.**

`DEC-012` was **amended 2026-08-21** and now answers the question this spec was
framed around, so the spec does not have to. Read the amendment first — it is the
operative text.

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

1. **The Structure / Interpretation split is implemented as `DEC-012` states.**
   Measured at design, the affected call sites in `src/ifd.rs` are:

   | line | tag | class | today | required |
   |---|---|---|---|---|
   | 1012/1014/1016 | `Orientation` | interpretation | bare `?` | costs the field |
   | 1031 | `BlackLevel` | interpretation | bare `?` | costs the field |
   | 1032 | `WhiteLevel` | interpretation | bare `?` | costs the field |
   | 1038 | `ActiveArea` | interpretation | bare `?` | costs the field |
   | 1024 | `SamplesPerPixel` | **structure** | bare `?` | **stays fatal** |
   | 1027 | `Compression` | **structure** | bare `?` | **stays fatal** |
   | 1028 | `RowsPerStrip` | **structure** | bare `?` | **stays fatal** — see note |

   ⚠ `RowsPerStrip` is structural because it maps strips to rows; without it a
   multi-strip plane cannot be assembled honestly. It is *inferable* on a
   single-strip file (`rows_per_strip == height`), and every corpus file is
   single-strip — so **do not let a green corpus talk you out of the fatal
   classification.** If you disagree, say so in the handback; do not just soften it.

2. **A leaf accessor may still return `Err`.** `scalar()`/`array()`/`values()`
   keep reporting a malformed tag honestly. What changes is that **`sensor()` must
   not inherit that failure for an interpretation tag** — it records the tag in
   `Sensor::malformed_tags` and continues.

3. **`RATIONAL` is handled** (`SPEC-004/FU-17`). `TYPE_RATIONAL` is not even
   defined in `src/ifd.rs` today (only BYTE/SHORT/LONG/UNDEFINED/IFD at :141-145).
   Read it as the two-`u32` pair the TIFF spec defines. A zero denominator, or a
   value that is not integral, is a **malformed shape** — it costs the field, it
   does not fail the file.

4. **`SPEC-004/FU-20`:** `NoSensorIfdCandidatesMalformed` must not name IFDs that
   were never candidates (`src/ifd.rs:916`).

5. **Fixtures pin the boundary in BOTH directions** — an interpretation tag
   malformed → the file still reads and the tag is recorded; a structural tag
   malformed → still fatal. A change that only demonstrates the new tolerance has
   not shown the boundary still exists.

6. Ten gates green; fuzz covers the widened `uints()`.

## Failing Tests

```bash
cargo test --all-features malformed_interpretation_tag_costs_only_the_field
cargo test --all-features malformed_structural_tag_is_still_fatal
cargo test --all-features rational_default_crop_size_reads_or_costs_the_field
cargo test --all-features malformed_orientation_on_ifd0_keeps_the_plane   # SPEC-004/FU-16
cargo test --all-features candidates_malformed_names_only_candidates      # SPEC-004/FU-20
```

⚠ **`cargo test <name>` matching ZERO tests exits 0** — a spec that names its
tests can pass vacuously (`named-tests-can-pass-vacuously`). Confirm each name
exists with `cargo test -- --list`, and **sum across targets**; reading one
target's line has produced a wrong answer twice on this project, in both
directions.

## Non-Goals

- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Executing opcodes — STAGE-003.
- Widening `uints()` to types no DNG tag we read can carry. `RATIONAL` is in scope
  because the DNG spec permits it for tags we already read; the signed types and
  `ASCII` are not, unless criterion 1 says otherwise.

## Notes for the Implementer

### `DEC-012`'s amendment is the spec. Read it first.

The line it draws: **"what exists" is the plane — its presence, location and
extent.** A tag that determines whether there is a plane and where it is, is
structural and fatal. Every other tag describes how to *interpret* a plane that
already exists, and malformed costs that field alone.

The defect being fixed is subtle and worth understanding rather than pattern-matching:
the old table said a malformed tag was *"fatal to that call only"* — but `sensor()`
**is** a call, so "only" silently included the plane. It conflated the accessor
that **read** the tag with the accessor the caller **invoked**.

### The shape to copy already exists

`SPEC-004` solved the same problem for the *selection* path: `is_sensor_ifd`
returns a `SensorMatch { Yes | No | Unreadable(tag) }` tri-state, so one bad IFD
does not abort the scan — the structural rule applied **per-IFD instead of
per-file**. Do the analogous thing per-**tag** in `sensor()`.

### Do not treat a green corpus as evidence

Every corpus file is single-strip, so the `RowsPerStrip` classification is
untested by real data. No corpus file carries a malformed tag on the paths this
spec changes — that is why `SPEC-004/FU-16` and `FU-17` were latent for two specs.
The hand-built fixtures are the evidence here; the corpus is a regression check.

### Scope

The extraction path and `uints()`. **No levels arithmetic, no cropping, no
orientation transform** — STAGE-002 and `DEC-008`. Extracting is in scope; applying
is not.

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
