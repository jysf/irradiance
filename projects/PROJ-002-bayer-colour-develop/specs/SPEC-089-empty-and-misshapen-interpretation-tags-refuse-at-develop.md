---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-089
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-002
  stage: STAGE-029
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-078, DEC-077, DEC-076, DEC-012, DEC-051, DEC-015]  # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red]  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-087, SPEC-086, SPEC-085]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-029's <capability>". Optional; null is acceptable.
value_link: "an interpretation tag that is present but empty or the wrong shape refuses the develop that reads it, instead of passing as absent"

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
  tokens_estimate: 25000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-089: Empty and misshapen interpretation tags refuse at develop

## Context

`STAGE-029` (PROJ-002), its only spec. An external follow-up review of 0.2.5 at
c78ec4c reported two P2 findings. Both are holes in how SPEC-087 implemented DEC-078:
DEC-078 refuses a tag in `Sensor::malformed_tags`, but the readers put a tag there only
when it cannot be *read*. A tag that reads cleanly with the wrong *shape* passes as
absent, or as valid.

**Reproduced at design (2026-11-23, at c78ec4c).** A throwaway probe wrote fixtures with
the test-only DNG writer and read them through `Container::parse`, `sensor()` and
`unpack_into`, as `tests/malformed_tags.rs` does:

| Fixture | `malformed_tags` | Result |
|---|---|---|
| count-zero `Orientation` (both IFDs), mono, `develop_into` | `[]` | `Ok(())` |
| count-zero `BlackLevel`, mono, `develop_into` | `[]` | `Ok(())` |
| count-zero `WhiteLevel`, mono, `develop_into` | `[]` | `Ok(())` |
| count-zero `CFALayout`, CFA, `demosaic_into` | `[]` | `Ok(())` |
| `ColorMatrix2` valid (dual illuminant, `develop_color_into`) | `[]` | `Ok`, output differs from absent |
| `ColorMatrix2` with 8 entries | `[]` | `Ok`, output **identical** to absent |
| `ColorMatrix2` with 9 zero-denominator entries | `[]` | `Ok`, output **identical** to absent |

**Audit of the neighbouring readers (same probe):**

| Fixture | Today | Verdict |
|---|---|---|
| `ActiveArea` count 3 or 0, `DefaultCropOrigin` count 1, `DefaultCropSize` count 0 | recorded, `MalformedTag` | already right (`array()` checks length) |
| `BlackLevelRepeatDim` count 0, one black value | recorded, develops | already right (DEC-078's exception) |
| `CFAPlaneColor` count 0 | not recorded, `UnsupportedCfaPlaneColor { [] }` | refuses, but not as malformed |
| `CalibrationIlluminant2` count 0 | not recorded, `Ok` | **hole** (silent single-illuminant) |
| `ColorMatrix2` count 0 | not recorded, `Ok` | **hole** |
| `ForwardMatrix1` valid, `ForwardMatrix2` 8 entries | not recorded, `Ok` | **hole** (silently takes the ColorMatrix path) |
| `CameraCalibrationSignature` count 0 | not recorded, `Ok` | **hole** (reads as an empty signature) |
| `ColorMatrix1` count 0, `CameraCalibration2` 8 entries, `CameraCalibration1` zero denominators, `AnalogBalance` 2 entries or count 0 | not recorded, `MissingColorCalibration` | refuses, but late and as the wrong variant |

In code:
- `Container::scalar` (`src/ifd.rs:1620`) returns `.first()` of the values, so a
  count-zero tag is `Ok(None)`, the same as absent, and `cost_the_field` (2160) records
  only `Err`. Every interpretation scalar is read through it: `Orientation` (2405 to 2423),
  `WhiteLevel` (2439), `CFALayout` (2360), both `CalibrationIlluminant`s via
  `ifd0_scalar` (2321), and DEC-077's set (2585 to 2615).
- `black_level_values` (1743) returns empty values for count zero, so `BlackLevel` reads
  as absent.
- `ifd0_rationals` (2179) checks neither the length nor the denominators.
- `ifd0_signature` / `signature_bytes` (2188) accept a count-zero `ASCII`.
- In `src/color.rs`, `dual_illuminant` (1235) drops `matrix_from_rationals`'s error with
  `.ok()?` and falls back to single-illuminant colour; `cam_to_xyz_interpolated` (1112)
  takes the ForwardMatrix path only `if let (Ok(_), Ok(_))` and otherwise silently takes
  the ColorMatrix path; `resolve_calibration` (1026) only validates a `CameraCalibration`
  it is about to use.

DEC-078's own Context names this exact case ("A malformed `ColorMatrix2` silently drops
to single-illuminant colour"), so this is the rule's intent not being met, not a new
policy. It is decided as an amendment, DEC-078 §7, written at this design.

**Corpus re-measured at design (2026-11-23, `exiftool -v2` on all 11 held DNGs):** every
`ColorMatrix`, `ForwardMatrix` and `CameraCalibration` has 9 entries, every
`AnalogBalance` and `AsShotNeutral` 3, `CFAPlaneColor` 3, `BlackLevel` 1 or 4 (the
`BlackLevelRepeatDim` area), and every scalar tag count 1. Nothing is count zero. The
K-3 III Monochrome's `BlackLevelRepeatDim` is count 1, as before. So no held file should
refuse and no held output should move; AC8 confirms it with the library's own reader.

## Goal

An interpretation tag that is present but empty, the wrong length, or (for the colour
rationals) has a zero denominator is recorded in `malformed_tags`, so DEC-078's existing
check refuses every develop path that reads it with `MalformedTag` before writing (and
DEC-077's check refuses its own set as `UnsupportedTag`). The colour code no longer turns
a present matrix's error into a fallback; absent tags keep their defaults, and no held
output changes.

## Inputs

- **Files to read:**
  - `decisions/DEC-078-...md`: §1 to §6, and the new §7 this spec implements.
  - `decisions/DEC-077-...md` §3 (a malformed DEC-077 tag refuses as `UnsupportedTag`).
  - `decisions/DEC-012-...md` and `DEC-015` (tolerant read; an absent tag is not
    recorded).
  - `src/ifd.rs`: `scalar` (1620), `values` (1695), `array` (1702 onward, the model:
    it already records a wrong length), `black_level_values` (1743), `cost_the_field`
    (2160), `ifd0_rationals` (2179), `ifd0_signature` (2188), `ifd0_scalar` (2321),
    `cfa` (around 2340), `sensor()` (2380 to 2704).
  - `src/color.rs`: `check_colour_required_processing` (650),
    `build_camera_to_srgb_linear` (972), `resolve_calibration` (1026),
    `cam_to_xyz_interpolated` (1112), `dual_illuminant` (1235),
    `matrix_from_rationals` (1426), `rational_to_f64` (1452).
  - `src/opcode.rs`: the sets (1085 to 1117), `check_malformed_tags` (1250),
    `check_unhonoured_tags` (1285).
  - `tests/malformed_tags.rs`: the fixture, entry and sentinel machinery this spec
    extends; `tests/unhonoured_tags.rs` for DEC-077's set.

## Outputs

- **Files modified:**
  - `src/ifd.rs`: a private count-aware scalar read for interpretation tags (count zero
    is an `Err`, so `cost_the_field` records it); the fixed-count checks; the zero-count
    `BlackLevel` and signature cases; length and denominator checks in the colour
    rational read. The public `Container::scalar` is **unchanged**, and its doc comment
    gains one sentence saying a count-zero tag is `Ok(None)` there.
  - `src/color.rs`: `dual_illuminant`, `cam_to_xyz_interpolated` and
    `resolve_calibration` propagate a present matrix's error instead of falling back
    (`dual_illuminant` becomes `Result<Option<_>, Error>`).
  - `tests/malformed_tags.rs`: the AC1 to AC7 tests (or a new
    `tests/misshapen_tags.rs` that reuses the same support modules; the build chooses,
    and says which).
  - `tests/ifd_reader.rs` / unit tests in `src/ifd.rs`: the reader-level rows (AC1).
  - `CHANGELOG.md` `[Unreleased]`: the new refusals, and the changed refusal (a
    misshapen `ColorMatrix1`, `CameraCalibration` or `AnalogBalance`, and a count-zero
    `CFAPlaneColor`, now refuse as `MalformedTag` where they refused as
    `MissingColorCalibration` or `UnsupportedCfaPlaneColor`).
- **Not modified:** `Error` (no new variant; `tests/error_kind.rs` stays at 76),
  `DEVELOP_VERSION` (stays `"8"`), the DEC-078 sets in `src/opcode.rs`, any Tier-B pin.
- **New flags / options:** none.

## Design

1. **Reader rule (DEC-078 §7.1).** In `sensor()`, for every interpretation tag it
   reads, a present tag is an `Err` (so `cost_the_field` records it and the field is
   `None`) when:
   - **(a)** its count is zero (every interpretation tag, including DEC-077's set,
     `DNGVersion`, `DNGBackwardVersion`, `BlackLevel`, `WhiteLevel`, `CFAPlaneColor`
     and both signatures);
   - **(b)** it has a fixed DNG count and the count differs: `Orientation`,
     `CFALayout`, `CalibrationIlluminant1/2` are 1; `ColorMatrix1/2` and
     `ForwardMatrix1/2` are 3 × ColorPlanes; `CameraCalibration1/2` are
     ColorPlanes × ColorPlanes; `AnalogBalance` is ColorPlanes. ColorPlanes is the
     `CFAPlaneColor` count on a CFA plane (3 when that tag is absent) and
     `SamplesPerPixel` otherwise;
   - **(c)** it is one of the colour rational tags above and any value has a zero
     denominator.
   Tags with a variable count (`BlackLevel`, `WhiteLevel`) keep their current count
   rules beyond (a). The array tags keep `array()`'s existing check.
2. **Orientation keeps its recovery.** A count-zero `Orientation` in one IFD with a
   well-formed one in the other is recovered (not recorded), exactly as a wrong-type one
   is today (2400 to 2423).
3. **No silent fallback in colour (DEC-078 §7.2).** `dual_illuminant` returns
   `Result<Option<DualIlluminant>, Error>`: `Ok(None)` only when a tag is **absent** (or
   an illuminant code is unknown, audit rank 8, unchanged), and a present matrix that
   fails `matrix_from_rationals` is `Err`. `cam_to_xyz_interpolated` takes the
   ForwardMatrix path when both forward matrices are present, and propagates their
   error; it takes the ColorMatrix path only when one is absent. `resolve_calibration`
   validates every present `CameraCalibration`, used or not. With the reader rule these
   errors are unreachable from a parsed file (the check refuses first); they are the
   second line for a `Sensor` built by hand, and they keep the old variant
   (`MissingColorCalibration`) there.
4. **Refusal and order unchanged.** DEC-078 §3's order and DEC-077's check run as
   today. The only change is which tags reach `malformed_tags`.
5. **No output change.** A held file has none of these shapes, so `DEVELOP_VERSION`
   stays `"8"` and no Tier-B pin moves.

## Acceptance Criteria

Pre-registered: every check is exact (an error variant and its fields, a
`malformed_tags` list, or byte equality). There is no numeric tolerance in this spec.

- [ ] **AC1, the reader records each shape.** For each row below, `sensor()` returns
  `Ok`, the field is `None` (or its absent value), the tag is in `malformed_tags`
  exactly once, and `unpack_into` succeeds:
  - count zero, for every tag in DEC-078 §2's three sets, `DNGVersion`,
    `DNGBackwardVersion`, and every DEC-077 tag;
  - a wrong count, for every fixed-count tag in Design §1(b), on both sides (one fewer
    and one more: for a matrix, 8 and 10 entries);
  - a zero denominator in one entry (the last), for every colour rational tag;
  - the valid control (the tag well-formed: not recorded, field `Some`) and the absent
    control (not recorded, field `None`), for every tag above.
- [ ] **AC2, each path refuses, with buffers untouched.** Every AC1 malformed row
  refuses `MalformedTag { tag }` (DEC-077 tags: `UnsupportedTag { tag }`) on every entry
  whose path's DEC-078 set contains the tag (DEC-077: every entry), with `dst` and scratch
  exactly as filled with the sentinel; on every other entry it behaves exactly as the
  absent control. Generated as a cross product from one table, as SPEC-087's AC1 is.
- [ ] **AC3, the review's two findings, end to end.** On a dual-illuminant CFA fixture:
  a valid `ColorMatrix2` gives output that differs from `ColorMatrix2` absent (the
  control that proves the fixture uses it); 8 entries and 9 zero-denominator entries each
  refuse `MalformedTag { tag: 50722 }` on all four colour develop entries and
  `to_srgb_linear_into`, dst untouched. The same three-way check for `ForwardMatrix2`
  with `ForwardMatrix1` present (valid differs from absent; misshapen refuses 50965),
  and for count-zero `CalibrationIlluminant2` (refuses 50779).
- [ ] **AC4, the colour code no longer swallows.** Unit tests in `src/color.rs` on
  hand-built `Sensor`s (bypassing the reader): a present 8-entry `color_matrix_2` with
  both illuminants known makes `dual_illuminant` return `Err`, not `Ok(None)`; a present
  8-entry `forward_matrix_2` with a valid `forward_matrix_1` makes the colour develop
  return `Err`, not the ColorMatrix-path output; an unused 8-entry
  `camera_calibration_2` on a single-illuminant sensor makes `resolve_calibration`
  return `Err`. Absent-tag controls for each still give `Ok` with today's output.
- [ ] **AC5, Orientation recovery.** Count-zero `Orientation` in IFD0 with a valid one
  in the sensor IFD is not recorded and develops with the sensor IFD's value; count zero
  in both is recorded once and refuses on the mono and colour entries.
- [ ] **AC6, exceptions kept.** The K-3 III-shaped fixture (one `BlackLevel` value,
  `BlackLevelRepeatDim` count 1) still develops byte-identical to the repeat-dim-absent
  control under both `BlackLevels` options. A count-zero `BlackLevelRepeatDim` with one
  black value develops too. `AsShotNeutral`, `CFAPattern` and the opcode lists keep their
  own errors (DEC-078 §2 "not in any set"), and so does an unknown
  `CalibrationIlluminant` code (it still falls back: audit rank 8, out of scope).
- [ ] **AC7, changed refusals are the listed ones.** Count-zero `ColorMatrix1`,
  `CameraCalibration1` with 8 entries, `AnalogBalance` with 2 entries and count-zero
  `CFAPlaneColor` now refuse `MalformedTag` with their tag (were
  `MissingColorCalibration` / `UnsupportedCfaPlaneColor`). The CHANGELOG lists exactly
  these as changed refusals.
- [ ] **AC8, nothing held changes.** `just test` with the corpus: every Tier-B pin passes
  unchanged; `irr ifd` on every held DNG shows the same `malformed_tags` as at c78ec4c
  (empty except the K-3 III's `[50713]`); `DEVELOP_VERSION` is `"8"`;
  `tests/error_kind.rs` still counts 76.
- [ ] **AC9, red-proof.** Each mutant is applied, shown red on the named test, and
  reverted, with a green control run first:
  - (a) count-zero scalar read back to `Ok(None)`: AC1 (count zero) and AC2.
  - (b) matrix length check removed: AC1 (8 and 10 entries) and AC3.
  - (c) denominator check removed: AC1 (zero denominator) and AC3.
  - (d) `dual_illuminant` back to `.ok()?`: AC4.
  - (e) the ForwardMatrix branch back to `if let (Ok, Ok)`: AC4.
  - (f) `resolve_calibration` validating only a used calibration: AC4.
  - (g) Orientation recovery dropped (a count-zero IFD0 entry recorded even when the
    sensor IFD's is valid): AC5.
  - (h) the length check taken as "at least" instead of "exactly" (so 10 entries
    pass): AC1's 10-entry rows.
  - (i) ColorPlanes fixed at 3 instead of read from `CFAPlaneColor`: AC1's ColorPlanes
    row (see the distinguishing fixtures).
  - (j) the zero-denominator check applied to the first entry only: AC1 (the zero is in
    the last entry).

## Failing Tests

- **`tests/malformed_tags.rs`** (or `tests/misshapen_tags.rs`), Tier A, fixtures through
  `Container::parse → sensor → unpack_into`:
  - `ac1_the_reader_records_each_empty_or_misshapen_tag`
  - `ac1_valid_and_absent_controls_are_not_recorded`
  - `ac2_each_path_refuses_a_misshapen_tag_it_reads`
  - `ac3_a_misshapen_second_colour_matrix_refuses_instead_of_falling_back`
  - `ac3_a_misshapen_second_forward_matrix_refuses_instead_of_switching_path`
  - `ac5_an_empty_orientation_is_recovered_from_the_other_ifd`
  - `ac6_the_k3iii_exception_and_the_already_refused_tags_hold`
  - `ac7_the_changed_refusals_are_malformed_tag`
- **`src/color.rs` unit tests:** `ac4_dual_illuminant_propagates_a_present_matrix_error`,
  `ac4_forward_matrix_error_propagates`, `ac4_unused_calibration_is_validated`.

**Rule-clause map (DEC-078 §7).**

| Clause | Test | Path where it changes the output |
|---|---|---|
| §7.1(a) count zero | `ac1_the_reader…`, `ac2_…` | `malformed_tags`, then each entry's result |
| §7.1(b) fixed count, both sides | `ac1_the_reader…` (8 and 10) | `malformed_tags` |
| §7.1(b) ColorPlanes from `CFAPlaneColor` | `ac1_the_reader…` (ColorPlanes row) | `malformed_tags` |
| §7.1(c) zero denominator | `ac1_the_reader…`, `ac3_…` | `malformed_tags`, colour result |
| §7.1 absent is not malformed | `ac1_valid_and_absent…` | `malformed_tags` |
| §7.1 Orientation recovery | `ac5_…` | mono develop output |
| §7.2 no silent fallback | `ac4_…` ×3, `ac3_…` | `dual_illuminant`, the FM branch, `resolve_calibration` |
| §7.3 changed refusals | `ac7_…` | the error variant |
| §7.4 no new variant, no output change | AC8 (corpus, `error_kind.rs`) | Tier B |

**Distinguishing fixtures.**
- Mutant (h), "at least 9": a 10-entry matrix. A fixture with only 8 entries cannot
  tell "exactly" from "at least".
- Mutant (i), ColorPlanes fixed at 3: a CFA fixture whose `CFAPlaneColor` has 4 entries
  (a 2×2 pattern using colours 0..3, as a CMYG file would) carrying a 9-entry
  `ColorMatrix1`. With ColorPlanes read from the tag, 9 is wrong (12 required) and the
  tag is recorded; with ColorPlanes fixed at 3 it is not. (The develop refuses that
  fixture anyway, as `UnsupportedCfaPlaneColor` or `MalformedTag`; AC1 checks
  `malformed_tags`, where the two answers differ.)
- Mutant (j), first entry only: the zero denominator in the **last** entry.
- Mutant (d): the matrix must be present and misshapen **with both illuminants known
  and distinct**. Otherwise `dual_illuminant` returns `None` for the illuminant reason
  and the mutant agrees with the fix.
- Mutant (e): `ForwardMatrix1` valid and `ForwardMatrix2` misshapen, on a fixture
  where the ColorMatrix path gives different output from the ForwardMatrix path (assert
  the two valid paths differ first).
- Mutant (f): an unused `CameraCalibration2`, so a single-illuminant fixture (one
  `CalibrationIlluminant`, no `ColorMatrix2`). On a dual fixture the calibration is used
  and the old code already refuses.
- Mutant (g): a count-zero IFD0 `Orientation` **and** a valid sensor-IFD one whose value
  is not 1, so a wrong recovery also shows in the output.

## Non-Goals

- A new `Error` variant, a `DEVELOP_VERSION` change, or any Tier-B re-pin.
- Changing the public `Container::scalar`, `values` or `array` behaviour.
- Refusing a well-formed singular matrix (DEC-078 §4).
- An unknown `CalibrationIlluminant` code (audit rank 8), several sensor IFDs (rank 9),
  degenerate warp and gains (rank 11), rank 12.
- Extra values beyond the DNG count for the variable-count tags (`BlackLevel`,
  `WhiteLevel`): their existing rules stand.
- The 0.2.6 release (its own release spec after this ships).

## Notes for the Implementer

- Write the count-zero fixtures with the test-only writer's `extra_tags` and
  `count: 0, bytes: vec![]` (the design probe did, and they parse). The probe source is
  reproduced in HANDOFF-185 so the build can start from it.
- Keep the count-aware scalar private and next to `cost_the_field`, with a doc comment
  citing DEC-078 §7, so the public `scalar` keeps its contract.
- `WhiteLevel` has two fields: `white_level` (costed) and `white_levels` (the colour
  path's array, `values().unwrap_or_default()`, not costed). Make sure a count-zero
  `WhiteLevel` lands in `malformed_tags` once, through `white_level`.
- `CFAPlaneColor`'s count gives ColorPlanes for the matrix checks, so read it before the
  colour tags (it is read in `cfa()` today; the colour tags are read later in
  `sensor()`). A malformed `CFAPlaneColor` leaves ColorPlanes at 3.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?**
   — <one word>

5. **What can a user do now that they couldn't before?**
   — <answer | none>
