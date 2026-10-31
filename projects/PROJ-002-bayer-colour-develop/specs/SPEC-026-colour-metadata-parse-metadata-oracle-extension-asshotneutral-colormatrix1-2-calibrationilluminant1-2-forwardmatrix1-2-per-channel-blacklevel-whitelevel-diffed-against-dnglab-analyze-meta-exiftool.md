---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-026
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  decisions: [DEC-012, DEC-015]    # cost-the-field; malformed_tags recorded only when the value is truly lost
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input]
  related_specs: [SPEC-005, SPEC-010, SPEC-024, SPEC-025, SPEC-027]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-024, SPEC-027]   # SPEC-024: the gate accepts CFA; SPEC-027: the Bayer files are in the manifest the oracle iterates

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Advances STAGE-006's 'colour metadata parses truthfully' — produces the trusted matrices/white-balance STAGE-007/008 consume."

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
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 29850028
      estimated_usd: 59.66
      duration_minutes: 45
      recorded_at: 2026-10-30
      notes: "Cold Claude Code build session. src/ifd.rs: new Rational{num:i32,den:i32} + Container::rationals() reader (SRATIONAL type 10 two SLONGs; fractional RATIONAL type 5 narrowed i32::try_from, overflow -> new Error::RationalValueOutOfRange, zero denominator kept verbatim; MAX_TAG_VALUES + as_chunks::<8> bounds like uints/payload); added ByteOrder::i32; 7 new TAG_* consts (ColorMatrix1/2 50721/50722, AsShotNeutral 50728, CalibrationIlluminant1/2 50778/50779, ForwardMatrix1/2 50964/50965); 9 additive Sensor fields (color_matrix_1/2, forward_matrix_1/2, as_shot_neutral, calibration_illuminant_1/2, black_levels, white_levels) parsed interpretation-class (DEC-012) from ifd0() for the matrices/illuminants/AsShotNeutral and from the sensor IFD for the full per-channel levels; scalar black_level/white_level + uints()/mono ToolReading/diff UNCHANGED (SPEC-005/010 red-proofs byte-for-byte). tests/colour_metadata_oracle.rs (new): ColourReading + pure colour_from_fields split, decimal_to_rational + exact i128 cross-multiply rationals_equal (no float tolerance), diff_colour, dnglab_black_levels (parse_rational_u32 per blacklevels.levels element) / dnglab_white_levels (plain-int whitelevels), tier-A fixture control + one-element-perturbed red + comparator red/green, 5 src/ifd.rs unit tests. src/bin/irr.rs prints all 9 fields (AC8, confirmed on the Canon). fuzz/fuzz_targets/ifd.rs now calls rationals() per entry. metadata_oracle.rs CFA_DEFERRED_TO_SPEC_026 -> CFA_COVERED_BY_COLOUR_ORACLE (rename + repointed comment + AC7 test). Made tools.rs unique_array_str + parse_rational_u32 pub (additive). ALL measured against the real corpus (exiftool -v3 -n + dnglab 0.7.2) 2026-10-30. Gates observed green locally: just test exit 0 (full corpus suite), colour oracle 23/23 (2/2 Bayer files present), lib 100/100, lint-ci clippy 0.1.98 -D warnings exit 0, lint-fuzz exit 0, fmt --check clean, validate exit 0, ifd fuzz 11,222,600 runs/61s zero crashes. Deviations (PR, DEC-012-style): (a) dnglab whitelevels are plain integers not N/D so dnglab_white_levels parses u32 (only blacklevels.levels use parse_rational_u32); (b) the Failing-Tests line 'two Bayer files absent from the renamed list' is self-contradictory (the mono oracle must KEEP skipping them) so AC7 implemented per its own text: renamed list names them covered-elsewhere, colour oracle covers them. tokens_total summed from this session's own Claude Code transcript JSONL (scratchpad-UUID ff60457c anchored, deduped by message.id across 105 usage records, all claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (98.5%); estimated_usd at per-category Opus list rates, no cache discount, order-of-magnitude. Hand-back snapshot."
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 10300008
      estimated_usd: 23.63
      duration_minutes: 27
      recorded_at: 2026-10-30
      notes: "Cold Claude Code verify session. Verdict ✅ APPROVED at cc67ad6 (build PR #56, merged). Verified independently, corpus present (IRRADIANCE_CORPUS_DIR): just test exit 0, 0 failed across 14 binaries; lib 100/100 (incl. 5 new ifd colour unit tests), colour oracle 23/23 with 2/2 Bayer files present (proven via --nocapture, not skipped), metadata_oracle 31/31; the 1 ignored is the pre-existing SPEC-017/SB-1 corpus finding. AC1–AC8 confirmed against exiftool -T -n -s3 + dnglab analyze --meta --json + irr ifd on both corpus files (Canon matrices n/10000, AsShotNeutral 606276/1000000..., levels [2047,2047,2047,2048]/[15000]; Nikon ForwardMatrix None, AsShotNeutral 64000/100000..., levels [0]/[3808], illuminants 17/21). oracle-must-be-shown-red re-proven with two OWN adversarial mutations: neutering rationals_equal fails exactly the two red-proofs; perturbing src rationals() (num+1) turns the live tier-B oracle red (Canon 5 fields disagree) — both reverted, tree clean. Exact i128 cross-multiply spot-checked across three non-reduced denominators (n/10000, n/1000000, n/100000); decimal_to_rational None forces a mismatch. Mono SPEC-005/010 oracle byte-for-byte behaviourally unchanged (pure rename + additive fixture fields; red-proofs + metadata_matches_exiftool + dnglab_scalars green). ifd fuzz 11,876,467 runs/61s/0 crashes (rationals() on every entry). fmt --check 0, validate 0; CI observed green on merged SHA cc67ad6 (gh pr checks 56). Adjudicated: Deviation 1 (dnglab whitelevels plain ints, parse_rational_u32 only on blacklevels) HONEST + load-bearing (parse_rational_u32 needs a '/', would fail on whitelevels); Deviation 2 (AC7 Failing-Tests bullet self-contradictory) HONEST, AC7 genuinely met (Canon 4-value BlackLevel cannot pass the mono single-valued ToolReading, so files stay IN CFA_COVERED_BY_COLOUR_ORACLE + the colour oracle covers them); candidate exact-rational-oracle decision recommended for minting as DEC-030 at ship (cross-cutting, DEC-026 is the precedent). tokens_total from this session's own transcript JSONL (scratchpad-UUID 03bbdfe2 anchored, deduped by message.id, all claude-opus-4-8, input+output+cache_creation+cache_read), 97.3% cache-read; estimated_usd at per-category Opus list rates, no cache discount, order-of-magnitude. Hand-back snapshot. Ship (and DEC-030 minting + totals) is the maintainer's."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-026: Colour metadata parse + metadata-oracle extension: `AsShotNeutral`, `ColorMatrix1/2`, `CalibrationIlluminant1/2`, `ForwardMatrix1/2`, per-channel `BlackLevel`/`WhiteLevel` — diffed against `dnglab analyze --meta` + `exiftool`

## Context

The demosaic (STAGE-007) and colour transform (STAGE-008) consume colour metadata
that nothing parses today — `AsShotNeutral`, `ColorMatrix1/2`,
`CalibrationIlluminant1/2`, `ForwardMatrix1/2`, and per-channel `BlackLevel` /
`WhiteLevel`. This spec parses them into `Sensor` and verifies them against the
metadata oracle (`dnglab analyze --meta` + `exiftool`), extended from PROJ-001's
SPEC-005. Parsed-but-unapplied is deliberate: the oracle is the fields' reader
(AGENTS.md §11, as PROJ-001 did for the opcode byte-streams), so STAGE-007/008
build on already-trusted values rather than re-deriving trust mid-pipeline.

## Goal

Parse `AsShotNeutral`, `ColorMatrix1/2`, `CalibrationIlluminant1/2`,
`ForwardMatrix1/2` and per-channel `BlackLevel` / `WhiteLevel` into `Sensor`, and
diff them against `dnglab analyze --meta` and `exiftool` for both corpus files —
absent optional tags stay `None`, a malformed tag costs that field alone
(DEC-012). No value is applied to pixels here.

## Design (measured 2026-10-30, Canon 5D-III + Nikon D300S)

Everything below rests on a measurement pass over the two corpus Bayer DNGs
(`exiftool -v3 -n`, `dnglab analyze --meta --json`), not on the DNG spec alone.
Five facts drive the whole design; each changes what the implementer builds:

1. **The colour matrices live in `IFD0`, not the sensor sub-IFD.**
   `ColorMatrix1/2`, `ForwardMatrix1/2`, `CalibrationIlluminant1/2` and
   `AsShotNeutral` all print under exiftool group `[IFD0]`. Only `BlackLevel` /
   `WhiteLevel` are in the sensor plane (`[SubIFD]`). So the new tags parse from
   `self.ifd0()` — the same source `Orientation` already falls back to
   (`src/ifd.rs:1320`), *not* the `ifd` the rest of `sensor()` reads.

2. **The matrices are `SRATIONAL` (signed) and `uints()` refuses them.**
   `uints()` (`src/ifd.rs:921`) reads `RATIONAL` only by integer division
   (`checked_rem == 0`), and rejects `SRATIONAL` (type 10) outright. The matrices
   carry negative, fractional values (`ColorMatrix1 = 0.7234 -0.1413 …`). A **new
   signed/fractional rational reader** is the core of the src work — `uints()` is
   untouched.

3. **The stored rationals are exact terminating decimals, and exiftool prints
   them in full.** `AsShotNeutral = 0.606276 1 0.461885` is stored
   `606276/1000000 1000000/1000000 461885/1000000`; the matrices are stored
   `n/10000`. exiftool's `-n` decimal is therefore the *exact* value, not a
   rounded view — so the oracle compares **exactly, by cross-multiplication**,
   with **no float tolerance**. A value exiftool had to round would (correctly)
   go red.

4. **`dnglab`'s `--meta --json` carries no matrices.** Its `metadata.rawParams`
   exposes `blacklevels.levels` (exact rationals, e.g. `"524032/256"`),
   `whitelevels`, and a derived float `wbCoeffs` — but nothing for the colour
   matrices or illuminants. So, exactly as `SPEC-005` scoped `dnglab` to the six
   scalars, **exiftool is the sole oracle for the matrices / illuminants /
   AsShotNeutral**; `dnglab` cross-checks only the per-channel `BlackLevel` and
   `WhiteLevel`.

5. **`BlackLevel` is per-channel (`2047 2047 2047 2048` on the Canon), and
   `ForwardMatrix` is absent on the Nikon.** The per-channel black is precisely
   why the scalar mono oracle (`Sensor::black_level: Option<u32>`, which reads
   only the first value) defers the Bayer files today. The Nikon carrying no
   `ForwardMatrix` gives a free present/absent contrast — one file exercises
   `Some`, one `None`, like `ActiveArea` in `SPEC-005`.

## Inputs

- **`src/ifd.rs`** — `Sensor` (`:556`), the `sensor()` parse (`:1288`+), `uints()`
  (`:921`), `payload()` (`:890`), `scalar()`/`values()`/`array()`,
  `cost_the_field()` (`:1227`), `type_size()` (`:170`, already sizes `SRATIONAL`
  as 8 bytes), the `TAG_*` constants (`:93`+), and the `Cfa` precedent (`:528`)
  for "store the DNG values verbatim, no lossy typed conversion."
- **`tests/support/tools.rs`** — the oracle machinery to parallel, NOT to mutate:
  `ToolReading` (`:258`), `reading_from_fields` (`:293`), `tri_state` (`:210`),
  `diff`/`compare_optional` (`:406`/`:511`), `parse_field_values` (`:152` — note:
  it `parse::<u32>()`s and will panic on a signed decimal, so the colour path
  needs its own token parser), `parse_rational_u32` (`:716`, reuse for dnglab's
  `num/den` strings), `unique_array_str` (`:658`), `exiftool`/`run_exiftool`.
- **`tests/metadata_oracle.rs`** — the scalar mono oracle and the
  `CFA_DEFERRED_TO_SPEC_026` hook (`:74`) this spec retires; `EXIFTOOL_SENSOR_GROUP`
  (`:44`, add the two CFA files, group measured), and the tier-A red-proof pattern
  (`oracle_is_clean_on_an_unmodified_reading` / `oracle_names_the_one_field_that_was_perturbed`, `:475`+).
- **`src/bin/irr.rs`** — the `ifd` subcommand prints every sensor field (`:148`+);
  §11 ("ship the reader with the field") requires the new fields appear here too.
- **External tools:** `exiftool` (group `IFD0` for the matrices, the sensor group
  for the levels) and `dnglab analyze --meta --json` (`blacklevels.levels`,
  `whitelevels`). Both tier-B, tool-gated + corpus-gated, skipping loudly by name
  when absent — the `SPEC-005` idiom.

## Outputs

**`src/ifd.rs` — parse + reader (the field, with its reader, §11):**

- A new value type `Rational { num: i32, den: i32 }` (`#[derive(Debug, Clone,
  Copy, PartialEq, Eq)]`), signed so it holds both `SRATIONAL` (matrices) and the
  positive `RATIONAL` `AsShotNeutral`. Stored verbatim — numerator and
  denominator as written, no reduction, no float — the `Cfa` "verbatim DNG
  values" precedent.
- A new reader `fn rationals(&self, entry: &Entry) -> Result<Vec<Rational>, Error>`
  parallel to `uints()`: accepts field type `SRATIONAL` (10, read two `i32`s) and
  `RATIONAL` (5, read two `u32`s and `i32::try_from` each — a value past
  `i32::MAX` is a typed error, never a wrap or panic). Same bounds discipline as
  `uints()`/`payload()` (`count` capped by `MAX_TAG_VALUES`, `u64` lengths,
  `as_chunks::<8>`). A zero denominator is **kept verbatim**, not rejected — these
  are not divided here (unlike `uints`), so `0/0`-type garbage is a value this
  reader returns and the oracle's cross-multiply then judges; a malformed *shape*
  (wrong byte length / wrong type) is the `Err`.
- New `Sensor` fields (all additive — the mono scalar path is untouched):
  - `color_matrix_1: Option<Vec<Rational>>`, `color_matrix_2: Option<Vec<Rational>>`
  - `forward_matrix_1: Option<Vec<Rational>>`, `forward_matrix_2: Option<Vec<Rational>>`
    (`None` on the Nikon)
  - `as_shot_neutral: Option<Vec<Rational>>`
  - `calibration_illuminant_1: Option<u32>`, `calibration_illuminant_2: Option<u32>`
    (`SHORT` scalar, the DNG LightSource code — read via the existing `scalar()`
    against `ifd0()`)
  - `black_levels: Vec<u32>` and `white_levels: Vec<u32>` — the FULL per-channel
    arrays (empty when absent), read via the existing integral `values()` path.
    `black_level`/`white_level: Option<u32>` stay as the first-value scalars the
    mono oracle and `develop` already use; do not change or remove them.
  - `malformed_tags` semantics unchanged: a present-but-wrong-shaped matrix tag
    costs that field alone (`→ None`, tag recorded), never the file (`DEC-012` /
    `DEC-015`), via `cost_the_field()`.
- New `TAG_*` constants (confirm each against `exiftool -v3`; a wrong number is
  caught instantly by the oracle): `TAG_COLOR_MATRIX_1 = 50721`,
  `TAG_COLOR_MATRIX_2 = 50722`, `TAG_AS_SHOT_NEUTRAL = 50728`,
  `TAG_CALIBRATION_ILLUMINANT_1 = 50778`, `TAG_CALIBRATION_ILLUMINANT_2 = 50779`,
  `TAG_FORWARD_MATRIX_1 = 50964`, `TAG_FORWARD_MATRIX_2 = 50965`.
- The colour tags parse from `ifd0()` (fact 1); the per-channel levels from the
  sensor `ifd`.

**`src/bin/irr.rs`** — add `println!` lines for every new field (§11).

**`tests/colour_metadata_oracle.rs` (new) — the colour oracle + its red-proof:**

- A `ColourReading` struct (exiftool's IFD0 colour tags + the sensor group's
  per-channel levels), a `colour_reading(path, sensor_group)` that shells out, and
  a pure `colour_from_fields(...)` shared with the tier-A fixture — the
  `exiftool_reading`/`reading_from_fields` split, so the red-proof runs the real
  parse.
- A decimal-token parser `fn decimal_to_rational(tok: &str) -> Option<(i64, i64)>`
  (sign + optional fraction → `num / 10^frac_digits`) and an exact comparator
  `fn rationals_equal(ours: Rational, theirs: (i64, i64)) -> bool` by
  cross-multiplication in `i128` (`ours.num * theirs.1 == theirs.0 * ours.den`).
  No float, no tolerance.
- `diff_colour(sensor, reading) -> Vec<Mismatch>` reusing `tools::Mismatch`,
  element-by-element with the same tri-state spirit (absent vs present-wrong vs
  value) as `compare_optional`.
- dnglab helpers `dnglab_black_levels(json) -> Vec<u32>` / `dnglab_white_levels`
  reusing `parse_rational_u32` per element of `blacklevels.levels` / `whitelevels`.

**`tests/metadata_oracle.rs`** — retire `CFA_DEFERRED_TO_SPEC_026`: rename to
`CFA_COVERED_BY_COLOUR_ORACLE` (same two files) and repoint its doc comment at
`tests/colour_metadata_oracle.rs`. The three scalar mono tests keep skipping the
two Bayer files — their per-channel black genuinely does not fit the scalar
comparison — but now for a "covered elsewhere" reason, not "not yet built."

## Acceptance Criteria

- [ ] **AC1 — matrices parse exactly, both files.** For each corpus Bayer file,
  `sensor.color_matrix_1`/`_2`, `forward_matrix_1`/`_2` and `as_shot_neutral` each
  equal exiftool's `IFD0` reading element-for-element, compared by exact
  cross-multiplication (no tolerance). The Canon carries `ForwardMatrix1/2`; the
  Nikon's `forward_matrix_1`/`_2` read `None` and exiftool confirms the tag is
  absent.
- [ ] **AC2 — illuminants + AsShotNeutral.** `calibration_illuminant_1 == Some(17)`
  and `_2 == Some(21)` on both files (exiftool), and `as_shot_neutral` matches
  (Canon `0.606276 1 0.461885`, Nikon `0.64 1 0.76417`).
- [ ] **AC3 — per-channel levels, cross-checked by both tools.** `sensor.black_levels`
  equals exiftool's full `BlackLevel` array (Canon `[2047,2047,2047,2048]`, Nikon
  `[0]`) AND dnglab's `blacklevels.levels` (parsed from `num/den`); `white_levels`
  equals exiftool's `WhiteLevel` and dnglab's `whitelevels` (Canon `[15000]`,
  Nikon `[3808]`).
- [ ] **AC4 — absent is `None`/empty, not zero or malformed.** A colour tag not in
  a file reads `None` (or empty vec) with its tag NOT in `malformed_tags`; the
  Nikon `ForwardMatrix` is the live case.
- [ ] **AC5 — a malformed matrix tag costs the field, not the file (`DEC-012`).** A
  synthetic sensor whose `ColorMatrix1` entry is present but wrong-shaped (bad
  count / wrong type) parses to a `Sensor` with `color_matrix_1 == None`,
  `TAG_COLOR_MATRIX_1` in `malformed_tags`, and every other field intact — the
  file does not error.
- [ ] **AC6 — the oracle is shown red, with a control
  (`oracle-must-be-shown-red`).** Tier A, no tool/corpus: a committed fixture of
  exiftool's real colour output, replayed through the SAME parse, diffs clean
  (control); perturbing exactly one matrix element produces exactly one named
  mismatch (red). The cross-multiply comparator itself has a direct red
  (`0.7234` vs `0.7235` → unequal) and green (`7234/10000` vs `0.7234` → equal,
  across a non-reduced denominator).
- [ ] **AC7 — a missing tool skips loudly, naming it** (the `SPEC-005` AC6 idiom),
  and the two Bayer files are no longer in `CFA_DEFERRED_TO_SPEC_026` — the scalar
  mono oracle names them "covered by the colour oracle," and the colour oracle
  covers them.
- [ ] **AC8 — `irr ifd` surfaces every new field (§11).** The reader ships with
  the field: `irr ifd <bayer.dng>` prints the matrices, illuminants, AsShotNeutral
  and per-channel levels.

## Failing Tests

Written now, before handoff; build's job is to make them pass.

- **`tests/colour_metadata_oracle.rs`** (new)
  - `colour_matrices_match_exiftool_on_both_bayer_files` — AC1 (tier B).
  - `forward_matrix_is_present_on_canon_and_absent_on_nikon` — AC1/AC4 contrast.
  - `illuminants_and_as_shot_neutral_match_exiftool` — AC2 (tier B).
  - `per_channel_levels_agree_with_exiftool_and_dnglab` — AC3 (tier B, both tools).
  - `colour_oracle_is_clean_on_an_unmodified_reading` — AC6 control (tier A).
  - `colour_oracle_names_the_one_matrix_element_that_was_perturbed` — AC6 red (tier A).
  - `cross_multiply_is_exact_across_a_non_reduced_denominator` — AC6 comparator
    green; `a_rounded_matrix_value_goes_red` — AC6 comparator red (tier A).
  - `a_missing_tool_skips_loudly_naming_it` — AC7 (reuse `tools::run_tool`).
- **`src/ifd.rs`** (unit, tier A — synthetic fixtures via `tests/support/tiff.rs`)
  - `srational_matrix_reads_signed_and_fractional_verbatim` — AC1 reader.
  - `rational_as_shot_neutral_reads_without_integer_division` — AC2 reader (the
    value `uints()` would reject as non-integral).
  - `a_malformed_color_matrix_costs_the_field_not_the_file` — AC5 (`DEC-012`).
  - `an_absent_forward_matrix_is_none_not_malformed` — AC4 (`DEC-015`).
  - `per_channel_black_level_keeps_every_value` — AC3 (`black_levels` not truncated
    to `black_level`'s first).
- **`tests/metadata_oracle.rs`**
  - the three scalar tests no longer reference `CFA_DEFERRED_TO_SPEC_026`; a test
    asserts the two Bayer files are absent from the (renamed) list — AC7.

## Non-Goals

- Applying white balance, the colour matrices, or the per-channel levels to pixels
  — STAGE-008 (and demosaic is STAGE-007). Parsed and exposed only.
- `CameraCalibration1/2`, `AnalogBalance`, `ReductionMatrix`, `BaselineExposure`
  and the rest of the DNG colour block — only the eight tags the Goal names are in
  scope. (They read trivially once `rationals()` exists; a later spec if needed.)
- Reducing or normalising the stored rationals — they are kept verbatim like `Cfa`.
- A float/epsilon tolerance in the comparator — the exact-decimal measurement
  (fact 3) makes cross-multiplication correct; a tolerance would hide real drift.
- Cross-checking `AsShotNeutral` against dnglab's derived float `wbCoeffs` — it is
  a reciprocal, lossy, and dnglab is already scoped off the matrices.
- A typed EXIF/colour-space helper beyond the parse (crustyimg walks IFD0 — O3).
- Promoting the exact-rational-oracle method to a `DEC` — flagged in Notes as a
  candidate; mint the next free `DEC` only if verify agrees it is cross-cutting.

## Notes for the Implementer

- **Read the matrices from `ifd0()`, the levels from the sensor `ifd`** (fact 1).
  The easy mistake is reading all of them from the sensor IFD, where the matrices
  are absent — you'd get `None` everywhere and AC1 would fail loudly, but burn a
  cycle. `ifd0()` is `Option`; treat `None` as "all colour tags absent."
- **Do not touch `uints()` or the mono `ToolReading`/`diff`.** The matrices need a
  separate reader and a separate oracle struct precisely so the seven mono files
  and `SPEC-005`/`SPEC-010`'s red-proofs stay byte-for-byte unchanged — the same
  separation `plane_oracle.rs` keeps from `metadata_oracle.rs`.
- **`parse_field_values` (`tools.rs:152`) will panic on a signed decimal** — it is
  `parse::<u32>()`. The colour path must not route through it; write a
  decimal-token parser in the new file.
- **The exiftool sensor group for the CFA files is measured, not assumed** — add
  both to `EXIFTOOL_SENSOR_GROUP` after `exiftool -n -G1 <file>` (the Canon's
  `BlackLevel` measured as `[SubIFD]`). The matrices are always `IFD0:`.
- **Candidate decision (do not mint a number yet):** "the colour-metadata oracle
  compares fractional DNG rationals *exactly*, by cross-multiplication against
  exiftool's full-precision decimal, with dnglab scoped off the matrices its JSON
  does not carry." If verify judges this cross-cutting, write it as the next free
  `DEC`; otherwise it lives here and in the Reflection.
- Panic-freedom (`no-panics-on-untrusted-input`): `rationals()` parses attacker
  bytes — mirror `uints()`/`payload()`'s bounds checks exactly; no `unwrap`,
  no indexing, no silent wrap on the `RATIONAL→i32` narrowing.

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
