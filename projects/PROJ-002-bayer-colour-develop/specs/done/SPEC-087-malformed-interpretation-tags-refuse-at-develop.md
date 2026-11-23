---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-087
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
  decisions: [DEC-078, DEC-012, DEC-053, DEC-051, DEC-077, DEC-076, DEC-058]  # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red]  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-086, SPEC-085, SPEC-056]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-086]  # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-027's <capability>". Optional; null is acceptable.
value_link: "a malformed tag the develop reads refuses with a typed error instead of silently developing with the absent-tag default"

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
      handoff: HANDOFF-182
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 11571180
      estimated_usd: 8.94
      duration_minutes: 13
      recorded_at: 2026-11-22
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (60 unique message.ids, all claude-opus-5-5, no subagents): input 124 + cache-creation 179,130 (1-hour) + cache-read 11,332,528 + output 59,398, opus-5-5 rates. The handback floor was 10,156,524 and $8.05. PR 312, build commit f1c6290; nine AC9 mutants red; corpus run 845 passed, 0 failed."
    - cycle: verify
      handoff: HANDOFF-183
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 9819437
      estimated_usd: 7.37
      duration_minutes: 16
      recorded_at: 2026-11-22
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (59 unique message.ids, all claude-opus-5-5, no subagents): input 120 + cache-creation 149,906 (1-hour) + cache-read 9,627,168 + output 42,243, opus-5-5 rates. Two rounds: round 1 PUNCH LIST at c8211c5 (P1, tests only: nothing separated lowest-tag order from walk order; the verifier added the interleaved BlackLevel plus CFALayout case to AC3), round 2 APPROVED at 3c0077e with no src change. Handback carried into PR 312 by fast-forward to 48c9a38. The handback floor was 8,989,130 and $6.83."
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop orchestration (design in PR 311, with an irr ifd re-measure of malformed_tags on every held DNG); counted at stage grain in STAGE-027's orchestration_cost, not split per spec."
    - cycle: ship
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop ship; counted at stage grain in STAGE-027's orchestration_cost."
  totals:
    tokens_total: 21390617
    estimated_usd: 16.31
    session_count: 4
shipped_at: 2026-11-22
---

# SPEC-087: Malformed interpretation tags refuse at develop

## Context

`STAGE-027` (PROJ-002), third of three. The audit in PR 303 (§1, §4 rank 2,
"Spec B") found that a malformed interpretation-class tag (`DEC-012`) is pushed
onto `Sensor::malformed_tags` and then treated as absent. The only consumer of
`malformed_tags` is the opcode-list check (`DEC-053`). For `BlackLevel`,
`WhiteLevel`, `ActiveArea`, `DefaultCropOrigin/Size`, `Orientation`, the
second-illuminant colour tags, `CFALayout` and `CFAPlaneColor`, a malformed
value develops exactly like an absent one. That re-opens PR 301's two CFA
refusals through a bad field type.

DEC-012's rationale for tolerance ("not consumed yet") has aged: these tags are
consumed now. No held file trips this today (the audit found `malformed_tags`
empty on the held files), so it is ranked for breadth, not for a live hit.

`depends_on: [SPEC-086]` is about ordering (the shared `Error` tail), not content.

**Carried in from SPEC-085 (shipped 2026-11-22).** Decide at design whether this spec
also covers a dual-illuminant file with a singular `ColorMatrix1/2`, or a singular
applied `CameraCalibration1/2`, that the converged weight never uses. Since DEC-076 §5
such a file develops (the loop starts at D50 and inverts only the product at the
weight it lands on), and is refused only when the weight needs the singular product.
So whether it develops depends on the scene. Refuse it outright, or keep the SDK's
behaviour; see SPEC-085's build ruling and the CHANGELOG entry.

**Carried in from SPEC-086 (shipped 2026-11-22).** A malformed `DNGVersion` (50706) with
no `DNGBackwardVersion` is recorded in `malformed_tags` and then skipped, so DEC-077's
version guard does not run. The SPEC-086 verify ruled that correct under DEC-077 as
written, and asked that this spec refuse it as `UnsupportedTag { tag: 50706 }`, matching
a malformed 50707, because the file's reader version is then unknown. Give 50706 its own
`Display` wording, as 50707 has.

**Re-measured at design (2026-11-22, `irr ifd` on every held DNG):** every
`malformed_tags` is empty except the Pentax K-3 III Monochrome's `[50713]`
(`BlackLevelRepeatDim`, count 1), whose `BlackLevel` has one value. It is developed mono
in held tests (`develop_oracle`, `black_levels` md5 under `PerPosition`,
`develop_options`, `develop_scratch`, `warp_alpha`), so it must keep developing.

**Both carried-in questions are decided in DEC-078:** the singular unused matrix keeps the
SDK's behaviour (§4, no change); the malformed `DNGVersion` refuses as
`UnsupportedTag { tag: 50706 }` (§2).

## Goal

A malformed interpretation tag that a develop path reads refuses that develop with a
typed `Error` before writing, instead of developing with the absent-tag default
(DEC-078). The tolerant parse is kept, so metadata, previews and plane reads still work.
A malformed `BlackLevelRepeatDim` stays harmless when `BlackLevel` has one value, so the
Pentax K-3 III Monochrome still develops. No held output changes.

## Inputs

- **Files to read:**
  - `decisions/DEC-078-...md`: the whole rule. This spec implements it.
  - `src/opcode.rs`: `check_required_processing` (1149), `check_unhonoured_tags` (1183;
    the malformed-50707 case at 1186 is the model for 50706), `DevelopPath` (1049).
  - `src/ifd.rs`: `sensor()` (2380 to 2704) and where each §2 tag is costed. Orientation
    is hand-rolled (2405 to 2423), `ifd0_version` (2228).
  - `src/demosaic.rs`: `demosaic_into` (278) and `demosaic_into_threaded` (478), which
    pass `DevelopPath::Colour` today.
  - `src/color.rs`: `to_srgb_linear_into` (287), which passes `Colour` today;
    `check_colour_required_processing` (647).
  - `src/develop.rs`: `develop_core` (1661), `black_pattern_dims` (686).
  - `src/lib.rs`: `UnsupportedTag`'s `Display` arms (1709 and 1719),
    `unhonoured_tag_name` (1744), `UnreadableOpcodeList`'s kind arm (1848).
  - `tests/unhonoured_tags.rs`: the per-entry, sentinel and malformed-fixture patterns
    this spec copies.
  - `tests/error_kind.rs`: 75 / 17 / 26 / 32 today.
  - `tests/ifd_reader.rs` (the K3III `malformed: &[50713]` row) and
    `tests/black_levels.rs` (701 to 708, the one-value repeat-dim case).

## Outputs

- **Files created:**
  - `tests/malformed_tags.rs`: AC1 to AC6.
- **Files modified:**
  - `src/opcode.rs`: `DevelopPath::Demosaic` and `DevelopPath::ColourTransform` (both
    opcode-equivalent to `Colour`); the DEC-078 §2 check after `check_unhonoured_tags`
    and before the opcode lists; the malformed-50706 case in the version part.
  - `src/demosaic.rs`, `src/color.rs`: the two demosaic entries pass `Demosaic`;
    `to_srgb_linear_into` passes `ColourTransform`. The four colour develop entries keep
    `Colour`.
  - `src/lib.rs`: `Error::MalformedTag { tag: u16 }` appended at the end, `Corrupt`; its
    `Display` arm; the name lookup widened to every DEC-078 §2 tag; an `UnsupportedTag`
    `Display` arm for 50706. `DEVELOP_VERSION` is not touched.
  - `tests/error_kind.rs`: one row; 75 to 76; `Corrupt` 32 to 33; the history comment.
  - `decisions/DEC-051-...md`: the new row.
  - `CHANGELOG.md` `[Unreleased]`: the new refusal, the new `DevelopPath` variants, and
    the change that the demosaic and transform entries now pass their own path.
- **New flags / options:** none.

## Design

1. **Path split.** Add the two `DevelopPath` variants. In the opcode-list logic, treat
   them as `Colour`. A composer calling `check_required_processing(sensor, Colour)` keeps
   today's opcode behaviour and now also gets the full colour malformed set.
2. **Malformed check.** A private function taking `(sensor, path)` that walks the
   path's set from DEC-078 §2 in ascending tag order and returns
   `MalformedTag { tag }` for the first tag in `malformed_tags`.
   - `BlackLevelRepeatDim` (50713) counts only when the sensor's `black_levels` has more
     than one value.
   - Sets are `const` slices, one per set, combined per path. No allocation.
3. **Version.** In `check_unhonoured_tags`, when `DNGBackwardVersion` is absent and 50706
   is in `malformed_tags`, refuse `UnsupportedTag { tag: 50706 }`, beside the existing
   50707 case. All paths.
4. **Order** (DEC-078 §3): DEC-077 checks, then the malformed check, then the opcode lists.
5. **Errors:** one variant appended, explicit `kind()` arm.
6. **No output change.** `DEVELOP_VERSION` stays `"8"`; no Tier-B pin moves.

## Acceptance Criteria

Pre-registered: every check below is exact (an error variant and its fields, or byte
equality). There is no numeric tolerance in this spec.

- [ ] **AC1, each path refuses its own set, on every entry.** For every tag in DEC-078 §2,
  a fixture with that tag malformed (written with a wrong type, `ASCII`) gives `Ok` from
  `sensor()` with the tag in `malformed_tags`. Then:
  - every entry whose path's set contains the tag returns `MalformedTag { tag }` and
    leaves `dst` (and scratch) exactly as filled with a sentinel;
  - every entry whose set does not contain it behaves exactly as on the same fixture with
    the tag absent (the same `Ok` bytes, or the same error).

  Entries: the three mono, the four colour develop, `to_srgb_linear_into`, and both
  demosaic entries. Mono entries on a mono fixture, the rest on a CFA fixture with colour
  tags. The test table is the cross product, generated from one list of (tag, sets), so
  a tag added to a set without a row fails.
- [ ] **AC2, the repeat-dimension exception.** A malformed `BlackLevelRepeatDim` with one
  `BlackLevel` value develops on every mono and colour entry, byte-identical to the same
  fixture with the tag absent, under both `BlackLevels::FirstValue` and `PerPosition`.
  With four `BlackLevel` values it refuses `MalformedTag { tag: 50713 }` on every mono and
  colour entry, under both options.
- [ ] **AC3, order.** A fixture with malformed `WhiteLevel` and malformed `ActiveArea`
  refuses 50717 (ascending). A fixture with a DEC-077 refused tag (`SampleFormat` 3) and a
  malformed `WhiteLevel` refuses `UnsupportedTag { tag: 339 }` (DEC-077 first). A fixture
  with a malformed `WhiteLevel` and an unreadable `OpcodeList2` refuses
  `MalformedTag { tag: 50717 }` (the malformed check before the opcode lists).
- [ ] **AC4, the version.** A malformed `DNGVersion` (count 3) with no
  `DNGBackwardVersion` refuses `UnsupportedTag { tag: 50706 }` on every entry. With a
  well-formed `DNGBackwardVersion` 1.4.0.0 present, it develops (the backward version
  decides). Its `Display` says the reader version is unknown.
- [ ] **AC5, already-refused tags keep their errors.** Malformed `CFAPattern`,
  `CFARepeatPatternDim`, `AsShotNeutral` and `OpcodeList1` give the same error as on
  `main`: `NotAColorFilterArray`, `UnsupportedCfaPattern`, `MissingColorCalibration` and
  `UnreadableOpcodeList { list: 1 }`. Malformed `MaskedAreas` develops.
- [ ] **AC6, reads unaffected and the composer check.** For every AC1 row, metadata
  (`sensor()`) and `plane::unpack_into` succeed. `check_required_processing(sensor, path)`
  called directly returns the same result as the entry for each of the four paths.
- [ ] **AC7, nothing held changes.** `just test` with the corpus: every existing Tier-B
  pin passes unchanged, the Pentax K-3 III Monochrome develops on every pin that develops
  it today, and `DEVELOP_VERSION` is still `"8"`.
- [ ] **AC8, error classification.** `tests/error_kind.rs` passes with 76 variants,
  `Corrupt` 33, `Unsupported` 26, `CallerError` 17. DEC-051 lists the row.
- [ ] **AC9, red-proof.** Each mutant below is applied, run, shown red on the named test,
  and reverted, with a green control run first:
  - (a) the malformed check not called: AC1.
  - (b) `demosaic_into` left on `Colour`: AC1 (a malformed `WhiteLevel` refuses it).
  - (c) `to_srgb_linear_into` left on `Colour`: AC1.
  - (d) the repeat-dimension exception dropped: AC2 (one value) and AC7 (the K-3 III).
  - (e) the exception taken as "any count" so four values develop: AC2.
  - (f) one tag removed from the colour set (for example `CameraCalibration2`): AC1.
  - (g) the malformed check placed after the opcode lists: AC3.
  - (h) the malformed-50706 case dropped: AC4.
  - (i) the check placed after the first write on one entry: AC1's sentinel.

## Failing Tests

- **`tests/malformed_tags.rs`** (Tier A, fixtures through
  `Container::parse → sensor → unpack_into`, as `tests/unhonoured_tags.rs` does)
  - `ac1_each_path_refuses_its_own_malformed_set_on_every_entry`
  - `ac1_a_tag_outside_the_path_set_behaves_as_absent`
  - `ac2_a_malformed_repeat_dim_with_one_black_value_develops`
  - `ac2_a_malformed_repeat_dim_with_four_black_values_refuses`
  - `ac3_order_dec077_then_malformed_then_opcode_lists`
  - `ac4_a_malformed_dng_version_without_a_backward_version_refuses`
  - `ac5_already_refused_tags_keep_their_errors`
  - `ac6_reads_and_the_composer_check_agree`
- **`tests/error_kind.rs`**: the updated counts (AC8).

**Rule-clause map (DEC-078).**

| Clause | Test | Path where it changes the output |
|---|---|---|
| §1 refuse, not default | `ac1_each…` | every entry's result and `dst` |
| §1 reads stay tolerant | `ac6_…` | `sensor()`, `unpack_into` |
| §2 each path's set, each tag | `ac1_each…` (cross product) | each entry |
| §2 outside the set is absent | `ac1_a_tag_outside…` | demosaic and transform entries |
| §2 repeat-dim exception, both directions | `ac2_…` ×2 | mono and colour levels |
| §2 50706 | `ac4_…` | every entry |
| §2 already-refused keep their errors | `ac5_…` | each error |
| §3 order | `ac3_…` | the error field |
| §3 variant and kind | `error_kind.rs` | `kind()` |
| §4 singular unused matrix unchanged | existing SPEC-085 tests | colour |
| §5 no output change | AC7 (corpus pins) | Tier B |

## Non-Goals

- Changing DEC-012's read rule. Parsing stays tolerant.
- Refusing a well-formed but singular unused `ColorMatrix` or calibration (DEC-078 §4).
- Audit ranks 8, 9 and 11 (unknown illuminant, several sensor IFDs, degenerate warp and
  gains), and rank 12.
- Any change to `DEVELOP_VERSION`, any Tier-B re-pin.
- Re-wording `UnreadableOpcodeList` or DEC-077's `UnsupportedTag` arms other than adding
  50706.

## Notes for the Implementer

- A malformed fixture: `tests/support/dng_writer.rs`'s `extra_tags` replaces an entry, so
  write the tag with `ASCII` and a short payload. Check that `sensor()` records it (some
  readers accept several numeric types, so use a type every reader rejects).
- `Orientation` is costed only when neither IFD0 nor the sensor IFD gives a value; a
  fixture must malform both, or write it in one IFD only.
- `WhiteLevel` has two fields: `white_level` (costed) and `white_levels` (the colour path's
  array, `values().unwrap_or_default()`, not costed). The tag in `malformed_tags` is what
  refuses; confirm the colour path cannot read a value through `white_levels` before the
  check.
- Keep the per-set `const` slices next to `DevelopPath` with a doc comment citing
  DEC-078 §2, so the sets and the decision stay in one place.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).

---


## Reflection

1. **What would I do differently next time?**
   — When a decision says "ascending order" over a union of sets, design the test case
   that tells union order from walk order. I suggested Orientation against a CFA tag,
   which cannot separate them (274 is lowest either way); the verify found the right
   pair (`BlackLevel` 50714 against `CFALayout` 50711). The build chose the right
   semantics (`min`) unprompted, but nothing pinned it.

2. **Does any template, constraint, or decision need updating?**
   — No decision change. The easy-shape fixture lesson is now N=3 across STAGE-027
   (SPEC-085 FU-1/FU-2, SPEC-086 P1/P2, SPEC-087 P1): each time a fixture exercised only
   the case where two readings agree. Raise it at the STAGE-027 close as a `lesson`
   signal; a spec-template line ("for each ordering or quantifier in the rule, name the
   fixture where the alternatives disagree") is the likely codification.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. STAGE-027's three specs are shipped. The audit's ranks 8, 9, 11 and 12 and the
   by-demand applies (`LinearizationTable`, `BlackLevelDelta*`) stay unscheduled, for the
   stage close to record.

4. **Where was the worst defect caught?** — verify

5. **What can a user do now that they couldn't before?** — A DNG with a malformed tag
   the develop reads (levels, active area, crop, orientation, CFA layout, colour
   matrices, calibration) used to develop with the absent-tag default and `Ok`, which
   could bypass PR 301's CFA refusals and SPEC-085's calibration; it now refuses with
   `MalformedTag` before writing, while all held files, the Pentax K-3 III Monochrome
   included, still develop byte-identically.
