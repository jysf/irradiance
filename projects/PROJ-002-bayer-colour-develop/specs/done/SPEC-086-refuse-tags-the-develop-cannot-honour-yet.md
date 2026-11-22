---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-086
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [punch-list, approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-002
  stage: STAGE-027
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-077, DEC-051, DEC-053, DEC-012, DEC-049]  # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red]  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-085, SPEC-087, SPEC-054]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-085]  # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-027's <capability>". Optional; null is acceptable.
value_link: "a file whose tags change what its samples mean refuses with a typed error instead of developing to wrong pixels with Ok"

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
  sessions:
    - cycle: build
      handoff: HANDOFF-180
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 16925255
      estimated_usd: 12.33
      duration_minutes: 22
      recorded_at: 2026-11-22
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (82 unique message.ids, all claude-opus-5-5, no subagents): input 168 + cache-creation 224,002 (1-hour) + cache-read 16,630,097 + output 70,988, opus-5-5 rates. Includes the verify punch-list fix (dbc2017, f71f703). The handback floor was 15,838,413. Duration is the active turns (12 build + about 10 fix), not the 51-minute transcript span with its idle wait. PR 309."
    - cycle: verify
      handoff: HANDOFF-181
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 10839645
      estimated_usd: 8.25
      duration_minutes: 18
      recorded_at: 2026-11-22
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (63 unique message.ids, all claude-opus-5-5, no subagents): input 127 + cache-creation 164,644 (1-hour) + cache-read 10,623,245 + output 51,629, opus-5-5 rates. Two rounds: round 1 PUNCH LIST at 258eee9 (P1 interleave factors read through scalar(), so count 0 or 2 developed Ok; P2 SampleFormat every-vs-any untested, fixed by the verifier), round 2 APPROVED at f71f703. 16 own mutants, 15 red in round 1, v5 red after P2. Handback carried into PR 309 by fast-forward to 0e6cc72. The handback floor was 9,951,163 and $7.64."
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop orchestration (design in PR 308, with a corpus exiftool and SOF-3 precision re-measure); counted at stage grain in STAGE-027's orchestration_cost, not split per spec."
    - cycle: ship
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop ship; counted at stage grain in STAGE-027's orchestration_cost."
  totals:
    tokens_total: 27764900
    estimated_usd: 20.58
    session_count: 4
shipped_at: 2026-11-22
---

# SPEC-086: Refuse tags the develop cannot honour yet

## Context

`STAGE-027` (PROJ-002), second of three. The audit in PR 303
(`docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md` §3, §4 ranks 4, 4b, 5, 6, 7
and 10, "Spec A") lists tags that change what the samples mean but that irradiance never
reads. A file carrying one develops with `Ok` and wrong pixels: the class PR 301 fixed for
`CFALayout`, `CFAPlaneColor` and the opcode version.

- `LinearizationTable` (50712): the samples are table indices, so tone is wrong across
  the whole range.
- `DNGBackwardVersion` (50707): the specification's own "this reader is too old" guard.
- `SampleFormat` (339) other than 1: half-floats or signed samples read as unsigned.
- `BlackLevelDeltaH/V` (50715/50716): per-column and per-row black offsets dropped.
- `RowInterleaveFactor`, `ColumnInterleaveFactor`, `SubTileBlockSize`: the plane is stored
  out of order and comes out scrambled.
- A Compression 7 SOF-3 precision `P` that disagrees with `BitsPerSample`: with no
  `WhiteLevel`, white defaults from `BitsPerSample`, so a P = 16 frame tagged 12-bit clips
  at 4095 (moved here from the withdrawn Spec C).

**Re-measured at design (2026-11-22, `exiftool -n` over every held DNG, plus a scan of each
Compression 7 file's SOF-3 headers):**
- No held file carries `LinearizationTable`, `BlackLevelDeltaH/V`, either interleave
  factor or `SubTileBlockSize`.
- `SampleFormat` is on the Nikon D300S only, as 1.
- `DNGBackwardVersion` is 1.1 to 1.4 where present. The Leica M Monochrom has `DNGVersion`
  1.0.0.0 and no backward version.
- `P` equals `BitsPerSample` on every Compression 7 file: the three Pixels 16/16 (192
  tiles each), the M Monochrom Typ 246 12/12, the Pentax K-3 III Monochrome 14/14.

So this spec refuses nothing held, and no held output changes.

`depends_on: [SPEC-085]` is ordering only: each spec in this stage appends `Error`
variants and moves the `tests/error_kind.rs` counts.

## Goal

Each listed tag, when present with a value the develop cannot honour, makes every develop
entry refuse with a typed `Error` before writing. A newer `DNGBackwardVersion` does the
same. Metadata and plane reads still succeed, except a Compression 7 plane whose frame
precision disagrees with `BitsPerSample`, which refuses at decode (DEC-077 §3). No held
file stops developing and no held output changes.

## Inputs

- **Files to read:**
  - `decisions/DEC-077-...md`: the whole rule. This spec implements it.
  - `src/ifd.rs`: `Sensor` (lines 735 to 916, `#[non_exhaustive]`, last field
    `malformed_tags`), `sensor()` (from line 2214; `malformed` starts at 2221), the
    readers `scalar` (1548), `ifd0_scalar` (2155), `ifd0_rationals` (2107),
    `ifd0_signature` (2116), `cost_the_field` (2088), and `parse_cfa` (2175) as the model
    for an optional tag. Tag constants live at lines 93 to 208.
  - `src/opcode.rs`: `check_required_processing` (1133), the gate every develop entry
    calls first, and `MAX_SUPPORTED_OPCODE_VERSION` (759).
  - `src/plane.rs`: `decode_unit` (1709; `parse_headers` 1776, `check_strip_shape` 1777),
    the tile path (`parse_headers` 1910), `check_strip_shape` (about 1945).
  - `src/ljpeg.rs`: `Frame.precision` (211), `parse_frame_header` (574).
  - `src/lib.rs`: the end of `Error` (`UnsupportedCfaPlaneColor`, 1141), its `Display`
    arms (the `UnsupportedCfaLayout` arm at 1552 is the style), `Error::kind()` (1701, no
    wildcard).
  - `tests/error_kind.rs`: `dec_051_table()` (37), the 72 and the bucket counts (470 to
    500), the history comment (23 to 35).
  - `tests/required_processing.rs`: the per-entry test pattern this spec's AC1 copies.
  - `tests/ljpeg_shape_binding.rs`: the decode-time pattern AC5 copies.
  - `tests/support/dng_writer.rs`: `write_dng`, `DngMeta.dng_version` /
    `dng_backward_version`, `DngDescription.extra_tags` (`RawTag`, `TagIfd::Sensor`).
  - `tests/support/tiff.rs`: `lj92_strip(samples, cols, rows, precision)` (417) and
    `compression7_entries` (605), for a frame whose `P` differs from `BitsPerSample`.

## Outputs

- **Files created:**
  - `tests/unhonoured_tags.rs`: AC1 to AC4 and AC6.
- **Files modified:**
  - `src/ifd.rs`: nine tag constants (`TAG_SAMPLE_FORMAT`, `TAG_DNG_VERSION`,
    `TAG_DNG_BACKWARD_VERSION`, `TAG_LINEARIZATION_TABLE`, `TAG_BLACK_LEVEL_DELTA_H`,
    `TAG_BLACK_LEVEL_DELTA_V`, `TAG_SUB_TILE_BLOCK_SIZE`, `TAG_ROW_INTERLEAVE_FACTOR`,
    `TAG_COLUMN_INTERLEAVE_FACTOR`); the nine `Sensor` fields of DEC-077 §1, appended
    before `malformed_tags`; their reads in `sensor()`.
  - `src/opcode.rs` (or a new private function it calls): the DEC-077 §2 check, first
    in `check_required_processing`, version guard then the tag set in ascending tag
    order.
  - `src/plane.rs`: the DEC-077 §3 precision check after the shape check, on the strip
    and the tile path.
  - `src/lib.rs`: three `Error` variants appended in DEC-077 §4's order, with `Display`
    arms and `kind()` arms. `DEVELOP_VERSION` is not touched.
  - `tests/error_kind.rs`: three table rows; 72 to 75; `Unsupported` 24 to 26,
    `Corrupt` 31 to 32; the history comment.
  - `tests/ljpeg_shape_binding.rs` (or a new `tests/ljpeg_precision_binding.rs`): AC5.
  - `decisions/DEC-051-...md`: the three new rows, as the test's comment requires.
  - `CHANGELOG.md` `[Unreleased]`: the new refusals and the new `Sensor` fields.
  - `docs/conformance-matrix.md`: the refused tags, if the matrix lists tag support.
- **New flags / options:** none.

## Design

1. **Read** (DEC-077 §1). IFD0 for the two versions, the sensor IFD for the rest. Each
   through `cost_the_field`, so a malformed value is `None` plus a `malformed_tags`
   entry. `BYTE` × 4 for the versions; a count other than 4 is malformed.
2. **Check** (DEC-077 §2), as one private function called first from
   `check_required_processing`, before the opcode lists:
   1. effective backward version (present; else `DNGVersion` with bytes 3 and 4 zeroed;
      else no check) compared byte-wise with `[1, 7, 1, 0]`; greater refuses
      `UnsupportedDngVersion`. A malformed `DNGBackwardVersion` refuses with
      `UnsupportedTag { tag: 50707 }`, not by the default.
   2. then 339, 50712, 50715, 50716, 50974, 50975, 51157 in that order: a value outside
      the honoured column, or the tag in `malformed_tags`, refuses
      `UnsupportedTag { tag }`.
3. **Decode** (DEC-077 §3). After `check_strip_shape` on each Compression 7 unit,
   `frame.precision as u32 != sensor.bits_per_sample` refuses `LjpegPrecisionMismatch`.
4. **Errors** (DEC-077 §4), appended in order, `kind()` arms explicit (no wildcard).
5. **No output change.** `DEVELOP_VERSION` stays `"8"`; no Tier-B pin moves.

## Acceptance Criteria

Pre-registered: every check below is exact (an error variant and its fields, or byte
equality). There is no numeric tolerance in this spec.

- [ ] **AC1, every develop entry refuses.** For each row of the refusal table below, each
  develop entry returns the expected error and leaves `dst` exactly as filled with a
  sentinel. Entries: `develop_into`, `develop_with_scratch_into`, `develop_color_into`,
  `develop_color_f32_into`, `develop_color_into_threaded`,
  `develop_color_f32_into_threaded`, `to_srgb_linear_into`, `demosaic_into`,
  `demosaic_into_threaded` (mono entries on a mono fixture, the rest on a CFA fixture).

  | Row | Fixture change | Expected |
  |---|---|---|
  | a | `SampleFormat` = 3 | `UnsupportedTag { tag: 339 }` |
  | b | `SampleFormat` = 2 | `UnsupportedTag { tag: 339 }` |
  | c | `LinearizationTable` = identity 0..=65535 | `UnsupportedTag { tag: 50712 }` |
  | d | `BlackLevelDeltaH` = [0, …, 0, 1/2] | `UnsupportedTag { tag: 50715 }` |
  | e | `BlackLevelDeltaV` = [−1/1, 0, …] | `UnsupportedTag { tag: 50716 }` |
  | f | `SubTileBlockSize` = [2, 2] | `UnsupportedTag { tag: 50974 }` |
  | g | `RowInterleaveFactor` = 2 | `UnsupportedTag { tag: 50975 }` |
  | h | `ColumnInterleaveFactor` = 2 | `UnsupportedTag { tag: 51157 }` |
  | i | `DNGBackwardVersion` = 1.7.1.1 | `UnsupportedDngVersion { backward_version: [1, 7, 1, 1] }` |
  | j | two tags at once: 51157 = 2 and 339 = 3 | `UnsupportedTag { tag: 339 }` (ascending order) |
  | k | backward 2.0.0.0 and 50712 present | `UnsupportedDngVersion` (version first) |

- [ ] **AC2, honoured values are no-ops.** Each of: `SampleFormat` = 1 (on the mono and
  the CFA fixture, both one sample per pixel), all-zero `BlackLevelDeltaH` and `V`, `SubTileBlockSize`
  = [1, 1], both interleave factors = 1, `DNGBackwardVersion` = 1.7.1.0, develops on every
  AC1 entry **byte-identical** to the same fixture without the tag.
- [ ] **AC3, the version default.** Absent backward version with `DNGVersion` 1.8.0.0
  refuses `UnsupportedDngVersion { backward_version: [1, 8, 0, 0] }`; with `DNGVersion`
  1.7.9.9 it develops (default 1.7.0.0); with both absent it develops. Present backward
  1.4.0.0 with `DNGVersion` 9.0.0.0 develops (the present value wins).
- [ ] **AC4, metadata and plane reads succeed.** For every AC1 row, `ifd::sensor()` is
  `Ok`, the new field holds the written value, and `plane::unpack_into` is `Ok` with the
  same samples as the fixture without the tag.
- [ ] **AC5, Compression 7 precision.** A strip with `P` = 16 and `BitsPerSample` = 12
  refuses `LjpegPrecisionMismatch { precision: 16, bits_per_sample: 12 }` on
  `unpack_into`, `unpack_unit_into` and `PlaneDecoder::unpack_unit_into`; `P` = 12 with
  `BitsPerSample` = 16 refuses likewise; a tiled Compression 7 fixture with a mismatch
  refuses on the tile path; `P` = `BitsPerSample` decodes as today.
- [ ] **AC6, malformed is refused.** Each of the seven tags written with a wrong type
  (`ASCII`), and `BlackLevelDeltaH` with a zero denominator, gives `Ok` from `sensor()`
  with the tag in `malformed_tags` and the field `None`, and refuses
  `UnsupportedTag { tag }` on `develop_into` and `develop_color_into`. A malformed
  `DNGBackwardVersion` (count 3) refuses `UnsupportedTag { tag: 50707 }`.
- [ ] **AC7, nothing held changes.** `just test` with the corpus: every existing Tier-B
  pin passes unchanged and no held file refuses. `DEVELOP_VERSION` is still `"8"`.
- [ ] **AC8, error classification.** `tests/error_kind.rs` passes with 75 variants,
  `Unsupported` 26, `Corrupt` 32, `CallerError` 17, and DEC-051 lists the three rows.
- [ ] **AC9, red-proof.** Each mutant below is applied, run, shown red on the named test,
  and reverted, with a green control run first:
  - (a) the check not called from `check_required_processing`: AC1.
  - (b) `SampleFormat` 3 treated as honoured: AC1 row a.
  - (c) the version compared with `>=` instead of `>`: AC2 (1.7.1.0 refused).
  - (d) the default ignoring `DNGVersion`: AC3.
  - (e) the `malformed_tags` refusal dropped: AC6.
  - (f) the zero-delta exemption dropped: AC2.
  - (g) the precision check removed: AC5.
  - (h) the check placed after the first write to `dst` on one entry (for example the
    threaded colour path's plan): AC1's sentinel assertion.

## Failing Tests

- **`tests/unhonoured_tags.rs`** (Tier A, synthetic fixtures through
  `Container::parse → sensor → unpack_into`, as `tests/required_processing.rs` does)
  - `ac1_every_entry_refuses_each_unhonoured_tag`: the AC1 table × the nine entries, with
    the sentinel check.
  - `ac1_first_offending_tag_is_reported_in_ascending_order`: rows j and k.
  - `ac2_honoured_values_develop_byte_identical`
  - `ac3_backward_version_defaults_from_dng_version`
  - `ac4_metadata_and_plane_reads_still_succeed`
  - `ac6_a_malformed_unhonoured_tag_is_refused`
- **`tests/ljpeg_precision_binding.rs`** (or appended to `tests/ljpeg_shape_binding.rs`)
  - `ac5_precision_must_equal_bits_per_sample_on_every_decode_path`
  - `ac5_matching_precision_still_decodes`
- **`tests/error_kind.rs`**: the updated counts (AC8).

**Rule-clause map (DEC-077).**

| Clause | Test | Path where it changes the output |
|---|---|---|
| §1 each field read | `ac4_…` | `sensor()` field value |
| §1 malformed → `None` + `malformed_tags` | `ac6_…` | `sensor()` |
| §2 version guard, present value | `ac1_…` row i; `ac2_…` (1.7.1.0) | `check_required_processing`, every entry |
| §2 version default from `DNGVersion` | `ac3_…` | same |
| §2 version before tags | `ac1_…order` row k | same |
| §2 each tag's refused value | `ac1_…` rows a to h | same |
| §2 each tag's honoured value | `ac2_…` | output bytes |
| §2 ascending order | `ac1_…order` row j | error field |
| §2 malformed refuses | `ac6_…` | develop entries |
| §2 `dst` untouched | `ac1_…` sentinel | threaded colour plan and single paths |
| §2 metadata and plane reads unaffected | `ac4_…` | `sensor()`, `unpack_into` |
| §3 precision, strip / unit / decoder / tile | `ac5_…` | each decode path |
| §4 variants and kinds | `error_kind.rs` | `kind()` |
| §5 no output change | AC7 (corpus pins) | Tier B |

## Non-Goals

- Applying `LinearizationTable` or `BlackLevelDelta*`. A later spec, by demand.
- SPEC-087's general rule for malformed interpretation tags (black, white, area, crop,
  orientation, colour, CFA). Only this spec's seven tags and the backward version refuse
  when malformed here.
- Refusing these tags at plane read (DEC-077 Alternatives).
- Audit ranks 8, 9 and 11 (unknown illuminant, several sensor IFDs, degenerate warp and
  gains).
- Any change to `DEVELOP_VERSION`, any Tier-B re-pin.

## Notes for the Implementer

- Fixtures: `tests/support/dng_writer.rs`'s `extra_tags` with `TagIfd::Sensor` for the
  sensor tags, `DngMeta` for the two versions. `SRATIONAL` bytes are two `i32`s in the
  file's byte order.
- The threaded colour entries decide their plan before falling back to the single-thread
  path. The check is in `check_required_processing`, which they already call first;
  AC1's sentinel row proves nothing is written before it.
- `cmd_develop` in `irr` unpacks before it develops, so a `SampleFormat` 3 file unpacks
  and then refuses at develop. That is the intended order; nothing to change in the CLI.
- Keep the new `Sensor` fields' doc comments citing the tag number and DEC-077.
- Commit messages are plain prose (the repo ruleset rejects code tokens in the body).

---


## Reflection

1. **What would I do differently next time?**
   — Name the reader for every scalar tag in the design, not only its type and count.
   DEC-077 §1 said "SHORT or LONG" for the interleave factors, and the build reached for
   the existing `scalar()`, which keeps the first of any number of values and treats count
   0 as absent. So the decision's "one value" never reached the code. The cold verify
   caught it (P1). A design line such as "read with a count-exactly-1 reader" would have
   prevented the round.

2. **Does any template, constraint, or decision need updating?**
   — No decision change: DEC-077 already said one value. Lesson, N=2 with SPEC-085's
   FU-1/FU-2: a Tier-A fixture that exercises only the easy shape (one `SampleFormat`
   value, one interleave value) leaves an "every vs any" or "one vs many" mutant alive.
   Not yet filed as a signal; watch for a third.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. A malformed `DNGVersion` with no `DNGBackwardVersion` is skipped, so the
   version guard does not run; the verify ruled that acceptable under DEC-077 and routed
   it to SPEC-087 (written into its Context at this ship). Applying `LinearizationTable`
   and `BlackLevelDelta*` stays a later spec, by demand.

4. **Where was the worst defect caught?** — verify

5. **What can a user do now that they couldn't before?** — A DNG whose tags change what
   its samples mean (float or signed samples, a linearization table, per-row or
   per-column black offsets, interleaved or sub-tiled storage, a reader version newer than
   1.7.1, or a compressed frame whose precision disagrees with its bit depth) used to
   develop to wrong pixels with `Ok`; it now refuses with a typed error before writing,
   while all 12 held files still develop byte-identically.
