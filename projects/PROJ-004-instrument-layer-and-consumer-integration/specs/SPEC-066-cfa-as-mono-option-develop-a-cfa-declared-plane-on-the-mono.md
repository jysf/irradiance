---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-066
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-019
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-061, DEC-058, DEC-060, DEC-040, DEC-018, DEC-012, DEC-051, DEC-052, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-026, SPEC-063, SPEC-064, SPEC-065]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-063]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-019's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-019's CFA-as-mono: a converted camera's CFA-declared plane develops on the mono path with its declared per-position black, opt-in and byte-identical by default."

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
  tokens_estimate: 28000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-066: Per-position black levels on the mono path, so a converted camera's CFA-declared plane develops as monochrome

## Context

SPEC-066 is the fourth spec in STAGE-019 (PROJ-004 instrument layer C). It covers
monorail-000 / onyx-000 request 6 (`feedback/2026-11-11-monorail-000-requests.md`). Their
words: a camera with the colour filter array removed still declares `CFAPattern`, so it
gets routed to demosaic. They asked for an option that treats it as monochrome while
keeping per-CFA-position black and white levels, with three acceptance tests:
- a uniform synthetic CFA plane gives the mono output;
- per-position levels are honoured;
- swapping positions is a red-proof.

It depends on SPEC-063 (shipped), which built `DevelopOptions`.

**DEC-061** (minted with this design) records what the probe found:
- **The routing already exists.** The mono develop never refuses a CFA plane, and
  SPEC-063 pins the Canon and Nikon mono outputs.
- **The gap is levels.** The mono path applies the first `BlackLevel` value to every
  photosite.

DEC-061 fixes the option's shape, the DNG `BlackLevelRepeatDim` semantics, the per-sample
white level, the one new error, and where the check sits.

## Goal

Add `DevelopOptions::black_levels: BlackLevels`. The default `FirstValue` is today's
behaviour, byte for byte. `PerPosition` applies `BlackLevel` per photosite as indexed by
`BlackLevelRepeatDim` and phased from the `ActiveArea` origin, on every mono develop entry
point, with no new allocation.

## Design

### 1. Probe results (main at a36cdc1, 2026-11-13; recorded so build is transcription)

Every held DNG was read twice: by exiftool, and by `Container::sensor()` in a throwaway
example (deleted, not committed). The full table is in DEC-061 §Context. What the build
needs:

- **`LinearRaw` files** (M Monochrom, Typ 246, three Q2M frames, K-3 III Mono) each carry
  one `BlackLevel` value. Their `BlackLevelRepeatDim` is `[1, 1]`, or on the K-3 III it is
  malformed, so `None` and listed in `malformed_tags`. The two modes must give
  **byte-identical** output on all six files.
- **The Canon 5D3** (CFA RGGB):
  - `black_levels = [2047, 2047, 2047, 2048]`, `black_level_repeat_dim = [2, 2]`,
    `white_levels = [15000]`;
  - `ActiveArea` (top 80, left 122, bottom 3950, right 5918), so 5796×3870 = 22,430,520
    photosites;
  - no opcode lists, Orientation 1, `DefaultCropOrigin` (18, 16).

  Under `Framing::FullActiveArea` with `apply_orientation: false`, the output **is** the
  active area in stored order. Measured by recomputing the normalize per photosite:
  - `PerPosition` differs from `FirstValue` on **5,602,332** samples;
  - the largest difference is **6** code values;
  - every difference is at position 3 (odd row, odd column from the `ActiveArea` origin),
    where `PerPosition`'s value is lower or equal.
- **The Nikon D300S** (CFA): `[0]`, `[1, 1]`. Both modes are identical.
- **Pixel 3a and 4a:** `[1023, 1023, 1022, 1022]`, `[2, 2]`. These are real-world
  per-position values for tier-A fixtures. Pixel files do not develop (opcode 9).
- **Pixel 7 Pro:** `BlackLevel` is RATIONAL and non-integral, so irradiance gives
  `black_level = None`, `black_levels = []`, and 50714 in `malformed_tags`. `PerPosition`
  must equal `FirstValue` on such a sensor (DEC-061 §3, "B empty").
- **Optical-black check on the Canon** (recorded so nobody uses the Canon as an accuracy
  oracle). The left masked columns average 2047.94 / 2048.06 / 2048.11 / 2047.20 at
  positions 0–3, phased from the `ActiveArea` origin. The declared pattern is applied as
  DNG defines it, and is not judged.

### 2. Public API (`src/develop.rs`), exactly DEC-061 §2

```rust
/// Which `BlackLevel` value each photosite's black comes from
/// ([`DevelopOptions::black_levels`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum BlackLevels {
    /// The first `BlackLevel` value for every photosite: `develop_into`'s
    /// behaviour. The default.
    #[default]
    FirstValue,
    /// `BlackLevel` indexed by `BlackLevelRepeatDim`, the pattern's origin at the
    /// `ActiveArea` top-left (DNG 1.7). For a converted (CFA-removed) camera
    /// developed as monochrome, and for any plane with a black pattern.
    PerPosition,
}
```

- **The new field `pub black_levels: BlackLevels`** goes on `DevelopOptions`. `Default`
  sets it to `FirstValue`.
- **`DevelopOptions`' doc example** gains nothing. **The field's doc** gives:
  - the DNG rule (DEC-061 §3);
  - the per-sample white;
  - the `ActiveArea` phase;
  - the new error;
  - that the 1×1 case is identical to `FirstValue`;
  - that a CFA plane already develops on this path, so this is the "CFA as mono" option.
- **Re-export `BlackLevels`** wherever `Framing` is re-exported, if anywhere. Check
  `src/lib.rs`.
- **`Error::InvalidBlackLevelPattern { repeat_rows: u32, repeat_cols: u32, count: usize }`**,
  `ErrorKind::Corrupt`. Its `Display` names all three numbers.

### 3. Resolving the levels

Replace the scalar `(black, white)` that `develop_core` threads through with a small
`Copy` value. For example:

```rust
enum Blacks<'a> {
    Uniform(u32),
    Pattern { values: &'a [u32], rows: u32, cols: u32, top: u32, left: u32 },
}
```

The exact form is the build's choice. It is built by a `resolve_levels_with(sensor,
options)` that keeps `resolve_levels`' current behaviour for `FirstValue`:

1. Resolve `white` exactly as today.
2. **`FirstValue`**, or **`PerPosition` with `black_levels` empty**: use
   `Uniform(black_level.unwrap_or(0))`, then today's `black >= white` check, then today's
   error.
3. **`PerPosition`**:
   - `[R, C] = black_level_repeat_dim.unwrap_or([1, 1])`.
   - If `R == 0`, `C == 0`, `R × C` overflows, or `R × C != black_levels.len()`, the
     result is `InvalidBlackLevelPattern { repeat_rows: R, repeat_cols: C, count: len }`.
   - Otherwise, the first `i` with `black_levels[i] >= white` gives
     `InvalidLevels { black_level: black_levels[i], white_level: white }`.
   - If every value is equal, the result is `Uniform(v)`. This is the fast path that keeps
     the 1×1 files on today's loop.
   - Otherwise `Pattern { … }`, with `(top, left)` the `ActiveArea` origin, or `(0, 0)`
     when the tag is absent.

**Call position.** It runs in `develop_core` exactly where `resolve_levels(sensor)?` is
today (after `apply_options(resolve_geometry(..))`, before the `dst` length check). No
other step moves.

### 4. The two normalize sites

`crop_orient_normalize_into` and `normalize_active_area_into` take the new value in place
of `(black, white)`. For each sample they already compute `raw_x` and `raw_y`, its raw
plane coordinates. The black is:
- `Uniform(b)`: `b`;
- `Pattern`: `values[((raw_y − top) mod R) × C + ((raw_x − left) mod C)]`.

`raw_y ≥ top` and `raw_x ≥ left` always hold, because the passes read only inside the
`ActiveArea`. Still, use `checked_sub`/`checked_rem` or `wrapping_sub` with a fallback, so
there is no panicking path (clippy's arithmetic lints). The index is always in bounds for
the same reason. Keep `.get().copied().unwrap_or(..)` totality anyway.

`normalize(sample, black, white)` is unchanged. Each photosite simply passes its own
`black`.

**Performance.** The `Uniform` arm must not compute a modulo per pixel. Monomorphise or
branch once outside the loops; the shape is the build's choice. A `Pattern` arm may
compute the row's base index once per row and step the column phase with a counter
instead of `%`. No timing gate. Record a before/after Q2M `develop_into` wall time in the
handback, release build, single run. The Q2M takes `Uniform`, so the time should not move.

### 5. Fuzz (`fuzz/fuzz_targets/develop.rs`, `examples/fuzz-seeds.rs`)

- **Options byte.** Bit 5 (`0b10_0000`) selects `BlackLevels::PerPosition`. Bits 0–4
  keep their meaning. Update the module doc.
- **Differential.** When bit 5 is set and the develop is `Ok`, also run with
  `FirstValue`. If the sensor's `black_levels` has at most one distinct value, the two
  outputs must be byte-equal. This relation is independent of the pattern arm's indexing
  (shared-path-oracle-tautology), because it only says the pattern arm collapses.
- **Seeds.** Add **three** synthetic seeds via `examples/fuzz-seeds.rs`, each with the
  option byte's bit 5 set:
  - a 2×2 non-uniform `BlackLevel` with `BlackLevelRepeatDim [2, 2]` and an **odd**
    `ActiveArea` origin;
  - a `[1, 2]` pattern;
  - a count-mismatch pattern (4 values with `[1, 1]`), which reaches
    `InvalidBlackLevelPattern`.

  Classify every develop seed by mode in the handback. No corpus bytes (DEC-003).

### 6. Docs

- **CHANGELOG `[Unreleased]`:** Added: `DevelopOptions::black_levels`, `BlackLevels` and
  `Error::InvalidBlackLevelPattern`. Name the converted-camera use, and note that output
  is unchanged by default.
- **The `develop` module doc** gets one paragraph: CFA planes develop on this path, and
  the levels choice is `black_levels`.
- **`DEVELOP_VERSION` stays `"4"`.** No provenance-ledger row: DNG `BlackLevel`
  semantics, no new algorithm. Say so in the handback.

## Inputs

- **Files to read:**
  - `src/develop.rs`:
    - `DevelopOptions` ~261;
    - `resolve_levels` ~504;
    - `normalize` ~539;
    - `crop_orient_normalize_into` ~748;
    - `normalize_active_area_into` ~815;
    - `develop_core` ~1183, levels at ~1220;
    - `run_passes` ~1391, the four call sites ~1410–1437.
  - `src/color.rs` ~1205–1283: `make_normalizer`/`level_at`. These show the colour path's
    `ActiveArea`-origin phase. Do **not** reuse `level_at` (DEC-061 Alternatives).
  - `src/ifd.rs`: `Sensor::black_level`, `black_levels`, `black_level_repeat_dim`,
    `white_level`, `active_area`, `malformed_tags`, `color_filter`, and
    `Sensor::test_fixture` / `Cfa::test_fixture` under the `test-fixtures` feature.
  - `src/lib.rs`: `Error`, `kind()`.
- **Tests to mirror:**
  - `tests/develop_options.rs`: tier-B gating, the eight MD5 pins, sentinel probes;
  - `tests/develop_scratch.rs` and `tests/develop_scratch_alloc.rs`: the scratch-path
    identity and the counting allocator;
  - `tests/error_kind.rs`.
- **Decisions:**
  - DEC-061 (this spec);
  - DEC-058 (options, version rule);
  - DEC-060 (allocation promise);
  - DEC-040 (`ActiveArea`-origin phase);
  - DEC-018 (clamp);
  - DEC-012 (malformed tag costs the field);
  - DEC-051, DEC-052, DEC-003.

## Outputs

- **Files modified:**
  - `src/develop.rs`: the enum, the field, level resolution, and the two normalize sites;
  - `src/lib.rs`: the variant and `kind()`;
  - `fuzz/fuzz_targets/develop.rs` and `examples/fuzz-seeds.rs`, plus three new seed
    files;
  - `CHANGELOG.md`;
  - `tests/error_kind.rs`.
- **Files created:** `tests/black_levels.rs`.
- **New public items:**
  - `develop::BlackLevels { FirstValue, PerPosition }`;
  - `DevelopOptions::black_levels`;
  - `Error::InvalidBlackLevelPattern`.
- **New option value:** `black_levels`, accepting `FirstValue` (the default) or
  `PerPosition`.

## Acceptance Criteria

- [ ] **AC1. The default is unchanged.**
  - `DevelopOptions::default().black_levels == BlackLevels::FirstValue`.
  - SPEC-063's eight MD5 pins stay green and unedited, through `develop_into`,
    `develop_with_into` and `develop_with_scratch_into`.
  - `DEVELOP_VERSION == "4"`.
- [ ] **AC2. The uniform-plane oracle (request 6, tier A, analytic).**
  - **Fixture:** a CFA-declared sensor (`Cfa::test_fixture`, `photometric` 32803).
    - non-square, with an `ActiveArea` whose top and left are both **odd**, and a
      `DefaultCrop` whose origin is also odd;
    - `black_levels = [1023, 1023, 1022, 1022]` (the Pixel 3a/4a values),
      `black_level_repeat_dim = [2, 2]`, `white_level = 16368`;
    - every raw sample = `black[pos] + k`, with pos phased from the `ActiveArea` origin.
  - **Under `PerPosition`**, every output sample equals `normalize(k)` against
    `[0, 16368 − black]`. The expected value is computed in the test from the formula,
    not by calling the crate:
    - for `k` = 0, 200 and 15345;
    - under both framings, both `apply_orientation` values, and Orientation 1 and 6.
  - **It also equals the mono output** of a `LinearRaw` sensor with the same geometry,
    one `BlackLevel` value `b`, and raw `b + k`, when the per-position range is equal.
    Pick `k` so it is. If the ranges differ, assert the analytic value only and say why.
  - **Under `FirstValue`**, the same plane is **not** uniform. That proves the fixture
    can tell the two modes apart.
- [ ] **AC3. Per-position is honoured, and the phase is the `ActiveArea` origin.**
  - **Fixture:** four distinct blacks `[100, 200, 300, 400]`, `[2, 2]`, white 4095, an
    odd `ActiveArea` origin and an odd `DefaultCrop` origin, and a hand-built plane.
  - The `StandardCrop` output equals a hand-computed expected image for at least the
    4×4 top-left output block, written as literals in the test.
  - **Red-proofs**, each of which must turn this test red:
    - swapping two pattern entries (request 6);
    - phasing from the plane origin;
    - phasing from the `DefaultCrop` origin;
    - transposing the index (`col × R + row`).
- [ ] **AC4. Non-square patterns.**
  - **`[1, 2]`:** blacks alternate by column only. **`[2, 1]`:** they alternate by row
    only.
  - **`[3, 3]`:** nine values, with a plane whose active width and height are not
    multiples of 3.

  Each is checked against a hand-computed block. Together they kill the "assume 2×2"
  mutant.
- [ ] **AC5. The warp path applies the black before the warp.**
  - **Fixture:** a non-identity, in-bounds `WarpRectilinear`, the AC3 fixture's pattern,
    and a plane equal to `black[pos] + k` everywhere.
  - **Under `PerPosition`**, the warped output is uniform `normalize(k)` (a constant
    field is invariant under the warp). Use both framings, at `warp_alpha` 1 and 0.5.
  - **Red-proof:** taking the black by output position after the warp, rather than by
    raw position before it, makes the output non-uniform.
- [ ] **AC6. Count, shape and range refusals.** Each `PerPosition` case below is
  `InvalidBlackLevelPattern` with the right fields, `kind() == Corrupt`, and `dst`
  unchanged (sentinel), from `develop_with_into` and `develop_with_scratch_into`:
  - 4 values with `[1, 1]`;
  - 3 values with `[2, 2]`;
  - `[0, 2]`;
  - `[u32::MAX, u32::MAX]` (overflow).

  Range and absent-tag cases:
  - A value ≥ white gives `InvalidLevels` carrying the first offending value.
  - The **same sensors under `FirstValue` develop `Ok`**. Pin that, because it is the
    "validated only when used" rule.
  - `black_levels` empty with `black_level = None` (the Pixel 7 Pro shape) develops
    under `PerPosition` and equals `FirstValue`.
  - `black_level_repeat_dim = None` with one value develops, and equals `FirstValue`.
  - `None` with four values is refused, because the default `[1, 1]` meets four values.
- [ ] **AC7. Error order.** On a sensor that is also wrong in each of these ways, the
  first one is still reported ahead of the pattern error:
  - out-of-domain `warp_alpha`;
  - a mandatory unknown opcode;
  - a wrong `src` length;
  - an invalid `ActiveArea`.

  On a sensor with a bad pattern **and** a wrong `dst` length, `InvalidBlackLevelPattern`
  wins. Every existing precedence test stays green unedited.
- [ ] **AC8. Tier B.**
  - **Canon 5D3**, `FullActiveArea`, `apply_orientation: false`:
    - the output is 5796×3870;
    - `PerPosition` equals `FirstValue` exactly at positions 0, 1 and 2;
    - at position 3 it is ≤, differs on exactly **5,602,332** samples, and by at most
      **6**.
  - **The six `LinearRaw` files and the Nikon:** `PerPosition` output MD5-equals
    `FirstValue` output, which is their SPEC-063 pin.
  - A missing file is reported by name.
- [ ] **AC9. Allocation.** In `tests/develop_scratch_alloc.rs`:
  - `develop_with_scratch_into` under `PerPosition` makes 0 allocations, on the AC3
    fixture with and without a warp and `FixBadPixelsConstant`, and on the AC6 error
    paths.
  - `develop_with_into` makes at most 1 allocation.
  - `scratch_len` is the same under both modes.
- [ ] **AC10. Error surface.**
  - `InvalidBlackLevelPattern` is in `Error::kind()` with no wildcard and has a row in
    `tests/error_kind.rs`. Its `Display` contains all three numbers, pinned by a test.
  - The `error_kind` counts move by exactly one.
- [ ] **AC11. API hygiene.**
  - The enum and field are documented as §2 says. An external test sets `black_levels`
    only by field assignment on `Default`.
  - `cargo doc` has no broken links.
  - CHANGELOG `[Unreleased]` is updated as §6 says.
- [ ] **AC12. Fuzz.** The §5 bit, the differential and the three seeds are in place. A
  60 s `develop` run is clean. Report the run count and the seed classification.
- [ ] **AC13. No regression.**
  - Every existing test passes with no assertion changed, except `error_kind`'s rows and
    counts.
  - `just test` passes with the corpus env set. So do `lint-ci`, `lint-no-allow`,
    `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny` and `msrv`.
  - No new dependency.

## Failing Tests

- **`tests/black_levels.rs`:**
  - `default_is_first_value_and_pins_hold`: AC1. The scratch-path pins are already in
    `develop_scratch.rs`; add only the options-default assertion and the `develop_into`
    pin loop if they are not covered elsewhere.
  - `uniform_cfa_plane_develops_uniform_under_per_position`: AC2.
  - `per_position_matches_hand_computed_block`: AC3.
  - `non_square_and_3x3_patterns`: AC4.
  - `black_is_applied_before_the_warp`: AC5.
  - `bad_patterns_are_refused_only_when_used`: AC6.
  - `pattern_error_order`: AC7.
  - `canon_per_position_differs_only_at_position_3`: AC8, tier B.
  - `one_by_one_files_are_identical_under_both_modes`: AC8, tier B.
  - `pattern_error_display_names_all_three`: AC10.
- **`tests/develop_scratch_alloc.rs`:** `per_position_allocates_nothing` (AC9).
- **`tests/error_kind.rs`:** the new row (AC10).

**Red-proof expectations.** The verify mutates each of these, and each must turn a named
test red in **tier A**:
- the pattern ignored (`FirstValue` always) → AC2, AC3;
- two entries swapped → AC3;
- phase from the plane origin → AC3 (odd `ActiveArea`);
- phase from the `DefaultCrop` origin → AC3 (odd crop);
- index transposed → AC3, AC4;
- a fixed 2×2 slot (`level_at`-style) → AC4 (`[1, 2]`, `[3, 3]`);
- the black taken by output position, or after the warp → AC5;
- no count check (index clamped instead) → AC6;
- the pattern validated under `FirstValue` too → AC6's "same sensor under `FirstValue`
  develops";
- the pattern check moved after the `dst` length check → AC7;
- per-position white (`white_levels[pos]`) → AC2's equality to the mono output, because
  `white_levels` has one value. Also add a fixture with `white_levels = [4095, 1000]` and
  assert the result equals the one with `white_levels = [4095]` (the first value only).

## Non-Goals

- **Routing.** No refusal of CFA planes and no change to which entry point a caller uses.
- **Per-position white** (DNG defines `WhiteLevel` per sample).
- **`BlackLevelDeltaH` and `BlackLevelDeltaV`.**
- **RATIONAL `BlackLevel`.** It is dropped to 0 on every path today. That is filed as a
  follow-up, not fixed here.
- **The colour path's `BlackLevelRepeatDim` handling.** Filed as a follow-up.
- SPEC-067 (blocked transpose), `irr` CLI flags, and a release.

## Notes for the Implementer

- **Odd origins on purpose.** Every held file has an even `ActiveArea` origin, so a phase
  bug is invisible on real data. AC3's odd `ActiveArea` and odd `DefaultCrop` fixture is
  the only thing that separates the three plausible origins. Make sure each of the three
  gives a different expected block.
- **Do not compute AC2–AC5's expected values by calling `develop`.** Write the normalize
  formula in the test (round-half-up integer form, DEC-018 clamp). Byte identity between
  the two modes is no oracle for the pattern arm.
- **AC8's Canon numbers** came from a recomputation over the raw active area. If the
  build's count differs, investigate before changing the number.
- **Keep `Uniform` on the fast loop.** The Q2M frames must not pay for this option.

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
