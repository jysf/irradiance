---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-075
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M  # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-023
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-068, DEC-061, DEC-058, DEC-012, DEC-040]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-066, SPEC-057, SPEC-026]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-023's <capability>". Optional; null is acceptable.
value_link: "a correct black for every BlackLevel shape DNG allows, so no file develops against black 0 or a mis-indexed pattern, plus onyx's MaskedAreas pass-through"

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
  tokens_estimate: 22000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-075: Black-level fixes: round fractional BlackLevel, colour BlackLevelRepeatDim, MaskedAreas

## Context

STAGE-019 carried an `[S]` follow-up out of the SPEC-066 design (DEC-061 §5), and it was
never framed:
- a non-integral RATIONAL `BlackLevel` is costed as malformed and resolves to black 0 on
  every path;
- the colour path indexes `black_levels` by a fixed 2×2 slot, falls back to the last
  value, and ignores `BlackLevelRepeatDim`.

monorail-000 / onyx request 5 (`feedback/2026-11-11-monorail-000-requests.md` item 8)
asks for `MaskedAreas` to be passed through. It reads the same sensor IFD and edits the
same `Sensor` struct, so it is bundled here.

DEC-068 records every choice. In short:
- round the fraction to nearest and keep the exact values (the user's choice,
  2026-11-19);
- the colour path follows DEC-061 §3 exactly;
- `MaskedAreas` is passed through verbatim as an interpretation-class tag;
- `DEVELOP_VERSION` goes to `"5"`.

## Goal

A fractional `BlackLevel` resolves to its nearest integer instead of 0, with the exact
values kept on `Sensor`. The colour develop indexes black by `BlackLevelRepeatDim` and
refuses a pattern whose count disagrees. `Sensor` reports `MaskedAreas`. Output on every
held file stays byte-identical.

## Design

### 1. Probe results (main at 4c5dd2c, 2026-11-19; recorded so build is transcription)

exiftool over all 11 held DNGs (the table is in DEC-068 §Context):
- **Only the Pixel 7 Pro has a non-integral black:** `262172/256 262224/256
  262223/256 262178/256` (1024.109375, 1024.3125, 1024.308594, 1024.132812), `[2, 2]`, white 16368. All four round to **1024**. Today it reads as
  `black_level: None`, `black_levels: []`, with 50714 in `malformed_tags`. It does not
  develop (mandatory opcode 9, STAGE-016).
- **The Canon's `BlackLevel` is RATIONAL and integral** (`524032/256` = 2047, and so on).
  It reads correctly today. After this spec it also fills `black_levels_exact` with the
  four rationals as written, a real tier-B fixture for that field.
- **Every other file** is SHORT/LONG with a `[1, 1]` or `[2, 2]` pattern, or a
  malformed one on the K-3 III. Every `WhiteLevel` has one value and every
  `SamplesPerPixel` is 1.
- **No held file has `MaskedAreas`**, so every fixture comes from the writer.
- **The test writer needs no change.** `DngDescription::extra_tags` (`RawTag`: ifd, tag,
  field_type, count, bytes) writes a RATIONAL `BlackLevel`, any `BlackLevelRepeatDim` and
  a `MaskedAreas` of any type or count, replacing a generated tag with the same
  `(ifd, tag)`.

### 2. Reading `BlackLevel` (`src/ifd.rs`, sensor interpretation)

- **The sensor read** keeps `black_level` (scalar) and `black_levels` (all values).
  - When the entry is RATIONAL, read it with `Container::rationals`. Map each value to
    `u32` by `(2·num + den) / (2·den)` in `u64`, using checked arithmetic. A zero
    denominator, or a result that does not fit `u32`, makes the whole tag malformed,
    costed once as today.
  - SHORT and LONG keep today's `uints` read.
  - The scalar is the first rounded value. Implement this in one helper used by both
    fields, so they cannot disagree.
- **New field, under `black_levels` with its own doc:**
  ```rust
  /// DNG `BlackLevel` exactly as written, when the tag is `RATIONAL` (DEC-068 §1).
  /// `black_level` / `black_levels` hold these rounded to nearest. Empty when the
  /// tag is SHORT or LONG (then `black_levels` is already exact), absent, or
  /// malformed.
  pub black_levels_exact: Vec<Rational>,
  ```
  Every `Sensor { .. }` literal in `src/` (test helpers in `color.rs`, `plane.rs` and
  `ifd.rs`) gains the field. `tests/` cannot build a `Sensor` literal
  (`#[non_exhaustive]`), so it needs no change.
- **Update the doc on `black_levels`**: it is rounded when RATIONAL, and
  `black_levels_exact` holds the fractions.
- **The existing `an_interpretation_tag_still_accepts_a_rational` test stays green
  unedited.** It reads through `uints`, which this spec leaves alone.

### 3. The colour normalizer (`src/color.rs`, `make_normalizer`)

- **Resolve once, before any write**, following DEC-068 §2 and DEC-061 §3: an empty `B`
  gives the scalar or 0; a count that does not equal `R × C` gives
  `Error::InvalidBlackLevelPattern`. Reuse the mono path's checks where it fits, for
  example by moving the pattern-fit check from `develop.rs` into a `pub(crate)` helper
  both paths call. The mono path's existing behaviour must stay identical.
  - **Do not add** the mono path's black ≥ white `InvalidLevels` check to colour. The
    colour `normalize_sample` is total for that case today, and changing it is outside
    this spec.
- **`make_normalizer` returns `Result`.** All four colour entry points
  (`develop_color_into`, `develop_color_f32_into` and the two `_threaded` variants, via
  `develop_color_bands`) propagate the error before touching `dst`.
- **The fast path, when `R` and `C` are each 1 or 2:** fill the existing four-slot
  `(black, white)` table, where slot `(row_odd, col_odd)` takes
  `B[(row_odd mod R) × C + (col_odd mod C)]`. The per-pixel closure is unchanged.
- **The general path, otherwise:** look up the black per photosite by the DEC-061
  formula from the `ActiveArea` origin, with no allocation in the closure. A small
  `enum` over the two cases is fine, with the per-pixel `match` hoisted the way
  `develop.rs`'s `Blacks` does it.
- **White, for every slot:** the first `white_levels` value, else `white_level`, else
  `2^BitsPerSample − 1` (today's default).
- **`level_at` is deleted** once nothing calls it.

### 4. `MaskedAreas` (`src/ifd.rs`)

- Add `pub const TAG_MASKED_AREAS: u16 = 50830;` beside `TAG_ACTIVE_AREA`.
- **New `Sensor` field** after `active_area`:
  ```rust
  /// DNG `MaskedAreas` (tag 50830): masked (optical-black) rectangles in raw-plane
  /// coordinates, verbatim and in file order (DEC-068 §3). Empty when absent or
  /// malformed. irradiance does not interpret or use them.
  pub masked_areas: Vec<ActiveArea>,
  ```
- **Read from the sensor IFD.** The field is empty, and 50830 goes to `malformed_tags`
  once, when any of these hold:
  - the type is not SHORT or LONG;
  - the count is not a multiple of 4, or is 0;
  - there are more than 16 rectangles (`MAX_MASKED_AREAS = 16`);
  - a rectangle has `top ≥ bottom` or `left ≥ right`;
  - a rectangle has `bottom > height` or `right > width`.

  The sensor read itself never fails on this tag.
- **The cap is checked before allocating.** Allocate after the count check, with
  `Vec::with_capacity(n)` where `n ≤ 16`.
- **Update `ActiveArea`'s doc line** to say it is also the type of each
  `masked_areas` rectangle.

### 5. Version and changelog

- **`DEVELOP_VERSION` goes from `"4"` to `"5"`** (DEC-068 §4). Its doc comment gets one
  line naming SPEC-075. Three tests pin `"4"`: `tests/black_levels.rs:247`,
  `tests/develop_options.rs:308` and `tests/orientation_tiles.rs:290`. They become
  `"5"`, and each assert message names DEC-068.
- **CHANGELOG `[Unreleased]`** gets entries for:
  - the fractional `BlackLevel` (Fixed);
  - the colour `BlackLevelRepeatDim` (Fixed);
  - the new `InvalidBlackLevelPattern` refusal on the colour path (Changed);
  - `black_levels_exact` and `masked_areas` (Added);
  - `DEVELOP_VERSION` 5 (Changed).

## Inputs

- `decisions/DEC-068-…` (all of it), `DEC-061` §3, `DEC-012`, `DEC-040`, `DEC-058` §5.
- `src/ifd.rs` (the sensor interpretation around `TAG_BLACK_LEVEL`, `array::<4>` for
  `ActiveArea`, `rationals`), `src/color.rs` (`make_normalizer`, `level_at`,
  `normalize_sample`), `src/develop.rs` (`resolve_levels_with`, `Blacks`).
- `tests/support/dng_writer.rs` (`extra_tags`), `tests/black_levels.rs` (SPEC-066's
  oracle style), `tests/colour_metadata_oracle.rs`, `tests/color_oracle.rs`.
- `feedback/2026-11-11-monorail-000-requests.md` item 8.

## Outputs

- **Modified:** `src/ifd.rs`, `src/color.rs`, `src/develop.rs` (only to share the
  pattern check, if done that way), `src/lib.rs` (`DEVELOP_VERSION`), `CHANGELOG.md`, the
  three version-pinning tests, and the feedback item-8 table row (built, with the spec
  ID).
- **Created:** `tests/sensor_levels.rs` (the `BlackLevel` read and `MaskedAreas`,
  writer-built), and `tests/colour_black_pattern.rs` (the colour pattern).
- **New public items:**
  - `Sensor::black_levels_exact: Vec<Rational>`;
  - `Sensor::masked_areas: Vec<ActiveArea>`;
  - `ifd::TAG_MASKED_AREAS`.

  No new error variant and no new flag.

## Acceptance Criteria

- [ ] **AC1, fractional black reads rounded.** A writer-built mono fixture with
  `BlackLevel` RATIONAL `[2049/2]` (1024.5) gives `black_level == Some(1025)`, and one
  with `[20491/20]` (1024.55) gives `1025`.
  - A 4-value CFA fixture with the Pixel 7 Pro's four rationals as written
    (`262172/256`, `262224/256`, `262223/256`, `262178/256`, little-endian at
    offset 3200 in the held file) gives `black_levels == [1024; 4]`. `black_levels_exact`
    equals the four `Rational`s, and `malformed_tags` does not contain 50714.
  - A zero denominator gives `black_level == None` and 50714 in `malformed_tags` once.
- [ ] **AC2, integral and integer types unchanged.** A SHORT `BlackLevel` fixture gives
  an empty `black_levels_exact` and the same `black_levels` as before. A RATIONAL
  integral fixture (`524032/256`) gives 2047, with `black_levels_exact` filled.
- [ ] **AC3, a fractional black develops with its rounded value.** A writer-built mono
  LinearRaw fixture with a fractional black `b` develops byte-identically to the same
  fixture with SHORT `round(b)`. The test then shows red: under today's code the
  fractional fixture develops as black 0, and the test asserts that output differs from
  the black-0 output it computes itself. Through `develop_into`, and through the colour
  develop with a CFA fixture.
- [ ] **AC4, colour pattern equivalence.** On a writer-built CFA fixture with an **odd**
  `ActiveArea` origin and non-uniform values:
  - `[1, 2]` with `[a, b]` develops byte-identically to `[2, 2]` with `[a, b, a, b]`;
  - `[2, 1]` with `[a, b]` develops byte-identically to `[2, 2]` with `[a, a, b, b]`;
  - `[4, 4]` with a 2×2 pattern tiled 4 times develops byte-identically to `[2, 2]`
    (the general path).

  Each assertion must be shown red against today's `level_at`, recorded in the
  handback: the `[1, 2]` and `[2, 1]` cases differ at one slot each.
- [ ] **AC5, colour general path is correct, not only equivalent.** A `src/color.rs`
  unit test checks the per-photosite lookup against an index table written out by hand
  for a `[3, 2]` pattern from an odd origin, with a mutant that swaps `R` and `C` shown
  red. A 3-by-2 pattern cannot be expressed as 2×2, so it exercises the general path.
- [ ] **AC6, count mismatch is refused.** `[2, 2]` with 3 values, and `[1, 2]` with 4,
  return `InvalidBlackLevelPattern { repeat_rows, repeat_cols, count }` from all four
  colour entry points with `dst` untouched (sentinel-filled). An empty `BlackLevel`
  still develops with black 0.
- [ ] **AC7, MaskedAreas.** Writer-built:
  - one rectangle and four rectangles read back verbatim and in order;
  - LONG and SHORT both read;
  - absent gives an empty field;
  - each malformed case in §4 (count 5, count 0, 17 rectangles, `top == bottom`,
    `right > width`, RATIONAL type) gives an empty field and 50830 in `malformed_tags`
    once, with the sensor read succeeding and every other field unchanged.

  One fixture's read is checked by hand against `exiftool -MaskedAreas`, with the
  output pasted in the handback.
- [ ] **AC8, the held corpus is unchanged.** `just test` with `IRRADIANCE_CORPUS_DIR`
  set is green, with every pinned develop hash and every oracle unedited apart from the
  three `DEVELOP_VERSION` pins. The Canon colour output is byte-identical to its pin.
  The Pixel 7 Pro sensor reads `black_level == Some(1024)` (one new tier-B assertion),
  and the Canon reads 4 `black_levels_exact`.
- [ ] **AC9, gates.** `just fmt-check`, `just lint`, `just test`, `just semver-check`
  (additive: two fields on a `#[non_exhaustive]` struct, one const), `just validate`,
  `just decisions-index --check`, `just cost-audit` and `just name-length-check` all
  pass, and CI is green.

## Failing Tests

- **`tests/sensor_levels.rs`** (new, writer-built, tier A):
  - `fractional_black_level_rounds_to_nearest` (AC1);
  - `pixel_7_pro_black_levels_read_exact_and_rounded` (AC1);
  - `zero_denominator_black_level_is_malformed` (AC1);
  - `integer_black_level_has_no_exact_values` (AC2);
  - `integral_rational_black_level_keeps_exact_values` (AC2);
  - `fractional_black_develops_as_its_rounded_value` (AC3, mono);
  - `masked_areas_read_verbatim_in_order` (AC7);
  - `masked_areas_short_and_long` (AC7);
  - `malformed_masked_areas_cost_only_the_tag` (AC7, table-driven over the six cases).
- **`tests/colour_black_pattern.rs`** (new):
  - `one_by_two_pattern_equals_its_two_by_two_expansion` (AC4);
  - `two_by_one_pattern_equals_its_two_by_two_expansion` (AC4);
  - `four_by_four_pattern_takes_the_general_path` (AC4);
  - `colour_pattern_count_mismatch_is_refused_on_every_entry_point` (AC6);
  - `fractional_black_develops_as_its_rounded_value_in_colour` (AC3).
- **`src/color.rs`** unit test: `general_black_pattern_index_matches_hand_table` (AC5).
- **Tier B** in the existing corpus tests: the Pixel 7 Pro `black_level == Some(1024)`
  and the Canon `black_levels_exact.len() == 4` (AC8), in `tests/ifd_reader.rs` or
  `tests/colour_metadata_oracle.rs`, whichever already opens those files.

## Non-Goals

- A fractional black carried through develop (DEC-068 §1, rejected for now).
- `BlackLevelDeltaH` / `BlackLevelDeltaV`.
- Any change to the mono `FirstValue` / `PerPosition` behaviour, or a colour-path
  `BlackLevels` option.
- A colour-path black ≥ white refusal.
- Using `MaskedAreas` in develop, naming strips, or checking them against `ActiveArea`.
- GainMap (STAGE-016), and making the Pixel files develop.
- A release.

## Notes for the Implementer

- **Byte identity on the held corpus is the main regression guard.** Run the full
  corpus `just test` (it uses the `corpus` profile, so a few minutes, not 37).
- **Don't let the oracle call the code it checks** (`shared-path-oracle-tautology`).
  AC3 and AC4 compare two inputs whose DNG meaning is the same: one uses the new
  feature, the other a shape the held corpus already pins. AC5 checks against an index
  table written out by hand. None of the three derives its expected value from the crate.
- **Commit message bodies are plain prose**, with no code tokens (the repository's push
  rule rejects them).
- **The colour path is fused and threaded** (`demosaic::develop_color_fused`). The
  normalizer is built once per call and shared across bands, so the `Result` belongs
  where it is built, before any band runs.

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
