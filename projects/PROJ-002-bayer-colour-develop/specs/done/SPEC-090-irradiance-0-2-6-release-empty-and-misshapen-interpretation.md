---
# A RELEASE SPEC is the spec for cutting a release — tagging, building the
# artifact, publishing it through each channel, and confirming it actually runs
# on a machine that didn't build it. See AGENTS.md "During ship" and
# docs/decisions/DEC-006.
#
# It uses the same task.* schema as a normal spec (so `just validate`,
# `just cost-audit`, and `just status` treat it as first-class), with
# `task.type: release`. The value it adds over a plain spec is the
# ## Release Pre-Flight checklist below: a generic, portable set of
# runtime/operational categories that every release hits, so each project
# stops re-earning the same gotchas one release at a time. This variant
# delegates the cut to an implementer via handoffs/HANDOFF-*.md.

task:
  id: SPEC-090
  type: release                    # epic | story | task | bug | chore | release
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved
  verify_rounds: [approved]

project:
  id: PROJ-002
  stage: STAGE-030
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-078, DEC-076, DEC-067, DEC-006]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-089, SPEC-088]  # [SPEC-NNN]

value_link: "irradiance 0.2.6 on crates.io carries DEC-078 section 7, so the published crate refuses an empty or misshapen interpretation tag as main does"

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 22000000
  sessions:
    - cycle: build
      handoff: HANDOFF-187
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 14419276
      estimated_usd: 10.33
      duration_minutes: 30
      recorded_at: 2026-11-23
      notes: "TRUE cost, re-summed at ship from the whole build transcript (f6e5f73d, 82 unique message.ids, claude-opus-5-5, no subagents): input 172 + cache-creation 175,437 (1-hour) + cache-read 14,184,198 + output 59,469. The handback floor was 12,869,053 and $9.37. Handback text: FLOOR from this session's own transcript (f6e5f73d-413a-4acd-bfe6-b8fd9a9cf071.jsonl, found by the scratchpad UUID; 75 unique message.ids, all claude-opus-5-5, no subagents), deduped by message.id with the largest output_tokens: input 156 + cache-creation 168,265 (1-hour) + cache-read 12,646,102 + output 54,530, opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Measured before the handback commit, so the last turns are not in it; re-sum at ship. Duration is an estimate: the transcript timestamps span 20 minutes with long background runs overlapping."
    - cycle: verify
      handoff: HANDOFF-188
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 12762552
      estimated_usd: 9.82
      duration_minutes: 40
      recorded_at: 2026-11-23
      notes: "TRUE cost, re-summed at ship from the whole verify transcript (0b33b4e6, 66 unique message.ids, claude-opus-5-5, no subagents): input 131 + cache-creation 203,847 (1-hour) + cache-read 12,497,485 + output 61,089. The handback floor was 10,944,131 and $8.71. APPROVED; three changed inputs added to the notes. Handback text: FLOOR from this session's own transcript (0b33b4e6-1c9e-4a72-ae09-f8f167ef5ebd.jsonl, found by the scratchpad UUID; 59 unique message.ids, all claude-opus-5-5, no subagents), deduped by message.id with the largest output_tokens: input 117 + cache-creation 197,507 (1-hour) + cache-read 10,690,847 + output 55,660, opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Measured before the handback commit, so the last turns are not in it. Duration is an estimate."
  totals:
    tokens_total: 27181828
    estimated_usd: 20.15
    session_count: 2
shipped_at: 2026-11-23
---

# SPEC-090: irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

## Context

The release spec of STAGE-030 (PROJ-002). 0.2.5 was published on 2026-11-23 (tag
`v0.2.5` on `c558a4b`). Since then `main` has shipped, to `5a5f079`:
- **SPEC-089** (STAGE-029, PR 319, DEC-078 §7): a present interpretation tag that is
  empty, the wrong length or zero-denominator is malformed, so DEC-078's existing check
  refuses it with `Error::MalformedTag { tag }` before writing. The colour code no longer
  falls back silently from a present matrix it cannot use.
- PRs 317, 318 and 320: project records only.

`git diff v0.2.5 5a5f079 -- src/` touches only `src/ifd.rs` and `src/color.rs`, and
`Cargo.toml` is unchanged. The CHANGELOG's `[Unreleased]` section already has SPEC-089's
entry (under Fixed), written by its build.

This spec ends prep-complete. A cold verify reviews the prep PR. The user then tags and
runs `cargo publish`, as for 0.2.5.

## Release Scope

### Decided for 0.2.6 (design, 2026-11-23)

- **Version / tag:** `0.2.6`, tag `v0.2.6`, a **patch** over the live `0.2.5`. Dependents
  on `irradiance = "0.2"` receive it with `cargo update`.
- **Why it is compatible: the design's mechanical check.** `just semver-check` (crates.io
  baseline, DEC-067, cargo-semver-checks 0.51.0) on `main` at `5a5f079` reports "202
  checks: 202 pass, 58 skip … Summary no semver update required". SPEC-089 adds no public
  item: no `Error` variant (still 76), no `Sensor` field, no `DevelopPath` variant, and
  `Container::scalar` is unchanged except its documentation. New refusals and refusals
  moved to an earlier error shipped as patches in 0.2.4 and 0.2.5. **If the build's
  semver check reports any breaking change, the build stops for the user's decision.**
- **Every input whose result changes since `v0.2.5`.** The design's list follows, derived
  from SPEC-089's CHANGELOG entry and a read of the non-test `src/` diff. The build must
  re-derive it from `git diff v0.2.5 <candidate> -- src/` (every `Err(` path, every new
  check, every reordering) and reconcile the two. A difference is a finding, not an edit
  to make silently. The release-spec template's Release Scope line asks for this.
  - **Output changes:** none for any file that develops. `DEVELOP_VERSION` stays `"8"`.
    SPEC-089's verify found `irr ifd` output identical on all 11 held DNGs.
  - **Newly refused** (as `MalformedTag`, or `UnsupportedTag` for DEC-077's set, on every
    path whose DEC-078 set holds the tag; before anything is written):
    - a **count-zero** `Orientation`, `BlackLevel`, `WhiteLevel`, `CFALayout`,
      `ColorMatrix2`, `ForwardMatrix1/2`, `CameraCalibration2`,
      `CalibrationIlluminant1/2`, `CameraCalibrationSignature`,
      `ProfileCalibrationSignature`, `BlackLevelDeltaH` or `BlackLevelDeltaV`. 0.2.5 read
      each as absent;
    - a **wrong-count** `Orientation`, `CFALayout` or `CalibrationIlluminant1/2` (each must
      be exactly 1). 0.2.5 took the first value;
    - a **wrong-length or zero-denominator** `ColorMatrix2`, `ForwardMatrix1/2` or
      `CameraCalibration2`. 0.2.5 fell back silently: single-illuminant colour for a bad
      `ColorMatrix2`, the ColorMatrix path for a bad `ForwardMatrix1/2`, the identity for a
      bad `CameraCalibration2` it did not apply;
    - an **unused ill-shaped calibration**: SPEC-085's test that such a file develops
      became a test that it refuses (DEC-076 §3 is amended by DEC-078 §7.2).

    A count-zero `Orientation` in one IFD is still recovered from a well-formed one in
    the other, as a malformed one was in 0.2.5.
  - **Already refused, different error:**
    - a misshapen `ColorMatrix1`, `CameraCalibration1/2` or `AnalogBalance`: was
      `MissingColorCalibration` from the colour code, now `MalformedTag { tag }` from
      `check_required_processing`;
    - a count-zero `CFAPlaneColor`: was `UnsupportedCfaPlaneColor`, now
      `MalformedTag { tag: 50710 }`;
    - **precedence:** `check_required_processing` runs before every entry's buffer,
      geometry, options, levels, `GainMap`, CFA and colour checks, so an input 0.2.5
      refused for one of those reasons that also carries a newly malformed tag now reports
      `MalformedTag` (or `UnsupportedTag`). The build names each class it confirms.
  - **Inspection-visible** (`Container::sensor()` still succeeds; no develop needed):
    - `Sensor::malformed_tags` can be longer for the same file;
    - a misshapen colour field (`color_matrix_1/2`, `forward_matrix_1/2`,
      `camera_calibration_1/2`, `analog_balance`) is now `None`, where 0.2.5 kept the
      wrong-length vector;
    - a count-zero `CFAPlaneColor` now reads as the default `[0, 1, 2]` and is listed as
      malformed. **The build confirms this against the code** and corrects the CHANGELOG's
      "left `None`" wording if it does not cover this field;
    - on a plane that is not CFA, ColorPlanes is `SamplesPerPixel`, so a 1-sample
      `LinearRaw` plane now expects 3-entry colour matrices and 1-entry `AnalogBalance`;
      a 9-entry `ColorMatrix1` there is now malformed. The mono develop does not read the
      colour set, so it is unaffected; `to_srgb_linear_into` on such a `Sensor` now
      refuses. **The build confirms this** and says whether any held mono file carries
      colour tags (SPEC-089's verify says none changed).
  - **Composer-visible:** a hand-built or edited `Sensor` that bypasses the reader, with a
    present colour matrix or calibration the colour code cannot use, now gets
    `MissingColorCalibration` from every colour entry and `to_srgb_linear_into`, where
    0.2.5 fell back silently.
- **No held file changes behaviour.** Every held file still develops with the same output,
  and the Pentax K-3 III Monochrome's malformed `BlackLevelRepeatDim` stays exempt.
- **What's included:** the CHANGELOG's `[Unreleased]` section (SPEC-089).
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes.

1. **The changed-input diff** (Release Scope above): reconcile the design's list with
   `git diff v0.2.5 <candidate> -- src/`, confirm or correct the two **confirm** items, and
   record the result in the internal record under Release Notes.
2. **The semver check:** `just semver-check`, recording the tool version and the full
   summary. Any reported breaking change stops the build for the user's decision.
3. **Version:** `Cargo.toml` `version = "0.2.6"`, with every other field unchanged
   (`git diff v0.2.5 -- Cargo.toml` before the bump is empty).
4. **CHANGELOG** (Keep a Changelog): promote `[Unreleased]` to `## [0.2.6] - <date>`, with
   a short lead paragraph like 0.2.5's and a fresh empty `## [Unreleased]` on top.
   **Correct any entry whose wording the changed-input diff contradicts.**
5. **Release notes:** run `just release-notes PROJ-002`, then write `## Release Notes`
   below in 0.2.5's shape (Headline, Fixed, Behaviour changes, Breaking / action required,
   Known limitations, then the internal record), and check them line by line against the
   reconciled changed-input list. Known limitations carry over from 0.2.5 unless this
   release changes one; the ColorChecker ΔE is still pending.
6. **README and crate docs:** check they still describe 0.2.x accurately (`cargo test
   --doc`, `just doc-check`). Wording fixes only.
7. **Tier-B evidence on the candidate SHA,** with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`: every held file present, every pin held.
   - The SDK colour cross-check, with `IRRADIANCE_SDK_COLOUR_DIR` set from a fresh
     `scripts/sdk-colour.sh` run into the scratchpad. The per-file maxima must equal
     0.2.5's 6 / 6 / 1 / 2 / 5.
   - The stage-2 cross-check, with `IRRADIANCE_SDK_STAGE2_DIR` from a fresh
     `scripts/sdk-stage2.sh` run, compared with 0.2.5's table. Every figure should be
     equal; say so or explain.
   - `irr ifd` on every held DNG, diffed against the same command built at `v0.2.5`
     (a worktree of the tag). Expected identical.
   - The three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded against 0.2.5's figures.
8. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`,
   `cargo publish --dry-run`. The package list must be the same files as 0.2.5's.
9. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), using SPEC-088's
   filled pre-flight as the model. Then run every gate: `just test`, `lint-ci`,
   `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`,
   `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
   `semver-check`, `package-targets-check`.

## Release Notes

*Outward-facing, written for whoever uses the crate. `just release-notes PROJ-002` was run
at build: it lists 18 notes, STAGE-029's one (SPEC-089) among them. As for 0.2.5, these
notes are written from the final CHANGELOG instead, with SPEC ids dropped, and checked
line by line against the reconciled changed-input list in the internal record. The
GitHub release text for `v0.2.6` is this section, down to the internal record.*

**Headline:** A malformed interpretation tag now refuses the develop even when it reads
cleanly with the wrong shape. 0.2.5 refused a tag it could not read, but a tag that was
present and empty, had the wrong number of values, or had a zero denominator passed as
absent or valid: a `ColorMatrix2` with 8 entries developed exactly as if it were absent,
as single-illuminant colour. 0.2.6 records such a tag in `Sensor::malformed_tags` and
refuses it with `Error::MalformedTag { tag }` before anything is written. The colour
code also no longer falls back silently from a present matrix it cannot use. No held
file changes: every one develops to the same output, and `DEVELOP_VERSION` stays `"8"`.

A compatible 0.2.x release: run `cargo update -p irradiance`.

**Fixed**
- **The shape rules.** A present interpretation tag is malformed when:
  - its count is zero;
  - it has a fixed DNG count and another count. `Orientation`, `CFALayout` and
    `CalibrationIlluminant1/2` are exactly 1. `ColorMatrix1/2` and `ForwardMatrix1/2`
    are 3 × ColorPlanes, `CameraCalibration1/2` are ColorPlanes × ColorPlanes, and
    `AnalogBalance` is ColorPlanes. ColorPlanes is the `CFAPlaneColor` count on a CFA
    plane (3 when it is absent) and `SamplesPerPixel` otherwise;
  - it is one of those colour rationals and has a zero denominator.

  Such a tag is recorded in `Sensor::malformed_tags` and its field is left `None`,
  except the two `Cfa` fields, which take their defaults: a malformed `CFAPlaneColor`
  reads as `[0, 1, 2]` and a malformed `CFALayout` as 1. An absent tag keeps its
  default as before.
- **The develop refuses it.** `opcode::check_required_processing` already refused a
  malformed tag that the develop path reads (`MalformedTag`, or `UnsupportedTag` for
  the tags that change what the samples mean). It now sees these tags too, on every
  develop entry whose tag set holds them, before anything is written.
- **No silent fallback in the colour code.** A present `ColorMatrix1/2`,
  `ForwardMatrix1/2` or `CameraCalibration1/2` that the colour code cannot use is
  `Error::MissingColorCalibration`, even when it would not have been applied. The
  reader refuses such a tag first, so this matters to a caller that builds or edits a
  `Sensor` itself. The one exception is a `ForwardMatrix2` on the single-illuminant
  path, which the colour code never reads.

**Behaviour changes**
- **One output change, no held file.** When IFD0's `Orientation` has more than one
  value and the sensor IFD carries a well-formed `Orientation` of its own, 0.2.5 used
  IFD0's first value and 0.2.6 uses the sensor IFD's, as it already did for an IFD0
  entry it could not read. Where the two values differ, the file develops with a
  different orientation. No other input that develops changes its output, and
  `DEVELOP_VERSION` stays `"8"`. On every held DNG, `irr ifd` prints the same as
  0.2.5, and no held file carries one of these shapes.
- **Newly refused** (0.2.5 developed these), before anything is written:
  - a count-zero `Orientation`, `BlackLevel`, `WhiteLevel`, `CFALayout`,
    `CalibrationIlluminant1/2`, `CameraCalibrationSignature`,
    `ProfileCalibrationSignature`, `BlackLevelDeltaH` or `BlackLevelDeltaV`. 0.2.5 read
    each as absent. A count-zero or wrong-count `Orientation` in one IFD is still
    recovered from a well-formed one in the other;
  - a wrong-count `Orientation` with no well-formed one in the other IFD, and a
    wrong-count `CFALayout` or `CalibrationIlluminant1/2`. 0.2.5 took the first value;
  - on `color::to_srgb_linear_into`, a 9-entry colour matrix on a 1-sample `LinearRaw`
    plane or on a plane whose `CFAPlaneColor` has 4 entries. 0.2.5 transformed both;
    0.2.6 reads the matrix as malformed for that plane's ColorPlanes;
  - an empty, wrong-length or zero-denominator colour matrix, forward matrix or
    calibration that 0.2.5's colour develop fell back from or did not use:
    - a `ColorMatrix1` or `ColorMatrix2` that sent a file to single-illuminant colour,
      which then developed from `ForwardMatrix1` when there was one;
    - a `ForwardMatrix1` or `ForwardMatrix2` on the dual-illuminant path, which then
      took the ColorMatrix path, and a `ForwardMatrix2` on the single-illuminant path,
      which ignored it;
    - a `CameraCalibration1` or `CameraCalibration2` that was not applied, because the
      calibration signatures differ or (for `CameraCalibration2`) the file took the
      single-illuminant path. 0.2.5 used the identity.
- **Already refused, different error:**
  - a misshapen `AnalogBalance`, and a misshapen matrix or calibration that 0.2.5's
    colour develop did use (for example a `ColorMatrix1` with no `ForwardMatrix1`, or
    an applied `CameraCalibration1`): was `MissingColorCalibration` from the colour
    code, now `MalformedTag { tag }`;
  - a count-zero `CFAPlaneColor`: was `UnsupportedCfaPlaneColor`, now
    `MalformedTag { tag: 50710 }`;
  - `check_required_processing` runs before every develop entry's buffer, geometry,
    options, levels, `GainMap`, CFA and colour checks. So an input 0.2.5 refused for
    one of those reasons that also carries a newly malformed tag now reports
    `MalformedTag` (or `UnsupportedTag`). For example, a 4-entry `CFAPlaneColor` with
    9-entry colour matrices was `UnsupportedCfaPlaneColor` (or `UnsupportedCfaPattern`
    when its `CFAPattern` names the fourth plane) and is now `MalformedTag` for the
    matrix, since 4 colour planes need 12 entries.
- **Inspection, with no develop:** `Container::sensor()` still succeeds on all of these
  files. `Sensor::malformed_tags` can be longer for the same file, and a misshapen
  colour field is `None` where 0.2.5 kept the wrong-length values. On a plane that is
  not CFA, ColorPlanes is `SamplesPerPixel`, so on a 1-sample `LinearRaw` plane a
  9-entry `ColorMatrix1` is malformed. The monochrome develop does not read the colour
  tags, so it still develops such a file; `color::to_srgb_linear_into` on its `Sensor`
  now refuses (above). No held monochrome file carries colour tags.
- **A caller that builds or edits a `Sensor`** with a present colour matrix,
  forward matrix or calibration the colour code cannot use now gets
  `MissingColorCalibration` from every colour develop entry and `to_srgb_linear_into`,
  where 0.2.5 fell back silently. The exception is a `ForwardMatrix2` on the
  single-illuminant path, which the colour code never reads, so it still develops.

**Breaking / action required:** none for the API. `cargo semver-checks` against crates.io
0.2.5 reports no semver update required: there is no new public item, `Error` variant,
`Sensor` field, dependency or feature, and the MSRV stays 1.90. `Container::scalar` is
unchanged; its documentation now says a count-zero tag is `Ok(None)` there.

A caller that keys on a specific error should check its code: some inputs that used to
develop now refuse, and some that were refused now report `MalformedTag`.

**Known limitations** (0.2.5's, plus the unknown-illuminant fallback, which was
already there)
- **No colour-accuracy claim yet.** The SDK match is a transform check on probe values,
  not a measured colour error. The ColorChecker ΔE2000 measurement is still blocked on
  capturing the chart.
- **Tags that change what the samples mean are refused, not applied.** This covers
  `LinearizationTable`, a non-zero `BlackLevelDeltaH` or `BlackLevelDeltaV`, a
  `SampleFormat` other than 1 (floating-point DNGs), a `SubTileBlockSize` other than
  `[1, 1]` and an interleave factor other than 1. No held file carries one.
- **A staggered CFA (SuperCCD-style `CFALayout` 2 to 9) or a non-RGB `CFAPlaneColor`**
  is refused, not demosaiced.
- **An unknown `CalibrationIlluminant` code** still sends a dual-illuminant file to
  single-illuminant colour rather than refusing it. A well-formed but unknown code is
  not malformed, so this release leaves it as 0.2.5 had it.
- **The Pixel 7 Pro still differs from the SDK by up to 2 codes before the `GainMap`,**
  because irradiance rounds its fractional black to the nearest integer. The colour
  transform's 6-code gap on the 7 Pro and 3a is separate.
- **The colour develop runs the `GainMap` but not the other `OpcodeList` stages**
  (`WarpRectilinear`, `FixBadPixelsConstant`) that the monochrome develop runs. A file
  that requires them is refused with a typed error. The develop options are mono-only.
- **A `GainMap` in `OpcodeList1` or `OpcodeList3`** is recognised but not applied; a
  mandatory one is refused with `UnsupportedRequiredOpcode`.
- **A Compression 7 strip is one plane unit.** Restart-interval units are not built, so
  a single-scan lossless-JPEG file (the K-3 III, the Typ 246) cannot be split.

### Internal release record (not part of the GitHub release text)

- **The candidate.** Branch `release/0.2.6-prep`, cut from `origin/main` at `c025e67` (the
  PR 321 merge). The build's commits touch only `CHANGELOG.md`, `Cargo.toml` (the
  version line), `README.md` (wording) and this project's records. `src/` is
  byte-identical to `c025e67` and to `5a5f079`, the design's semver point.
  Where the evidence ran:
  - `just test` with the corpus and both SDK lanes, the SDK `--nocapture` runs and the
    `irr ifd` diff: on `c025e67` (`src/`-identical), then `just test` again on the PR
    head (handback);
  - semver after the bump, timings, gates and the package checks: on `857959c`, the
    version bump.
- **Mechanical semver check:** `cargo-semver-checks 0.51.0`, crates.io 0.2.5 to the
  candidate, by `just semver-check` (DEC-067). Clean: no failure, nothing to accept.
  Verbatim summary, run after the version bump (`857959c`):

  ```
  Checking irradiance v0.2.5 -> v0.2.6 (minor change)
   Checked [   0.009s] 202 checks: 202 pass, 58 skip
   Summary no semver update required
  ```

  Before the bump (`c025e67`) the same run gave "v0.2.5 -> v0.2.5 (no change; assume
  minor) … 202 pass, 58 skip … no semver update required". As for 0.2.4 and 0.2.5, the
  tool labels a 0.x patch bump a "minor change"; no update is required.
- **Changed-input reconciliation** (step 1). The build read `git diff v0.2.5 c025e67 --
  src/` (2 files, +357/−66: `src/ifd.rs` and `src/color.rs`, all from SPEC-089), every
  new `Err(` path, every new shape check and the one reordering. Against the design's
  list:
  - **Confirmed as designed:**
    - no output change and `DEVELOP_VERSION` `"8"` (no pin moved; `irr ifd` identical);
    - every count-zero and wrong-count scalar refusal: `shaped_scalar` (exactly one for
      `Orientation`, `CFALayout` and `CalibrationIlluminant1/2`, non-zero for
      `WhiteLevel`), the count-zero guards on `BlackLevel`, `BlackLevelDeltaH/V` and
      the two signatures, and the count-zero `CFAPlaneColor` guard;
    - the colour shape rule `ifd0_colour_rationals` (exact count, no zero
      denominator) on `ColorMatrix1/2`, `ForwardMatrix1/2`, `CameraCalibration1/2` and
      `AnalogBalance`; `AsShotNeutral` keeps its old read;
    - the colour code's §7.2 changes: `dual_illuminant` returns `Err` for a present
      unreadable `ColorMatrix1/2`, `cam_to_xyz_interpolated` for a present unreadable
      `ForwardMatrix1/2`, and `resolve_calibration` validates every present
      calibration, used or not;
    - the `Orientation` recovery across IFDs, and the precedence rule (no call site
      moved; `check_required_processing` already ran first at every develop entry).
  - **Confirm item 1, confirmed and the CHANGELOG corrected.** A count-zero
    `CFAPlaneColor` is costed by `cost_the_field` and then defaulted by
    `.unwrap_or_else(|| vec![0, 1, 2])`, so `Cfa::plane_color` reads `[0, 1, 2]` and
    50710 is in `malformed_tags`. A malformed `CFALayout` likewise reads as 1
    (`.unwrap_or(1)`; `Cfa::layout` is a `u32`). The CHANGELOG's "left `None`" did not
    cover either `Cfa` field; it now names both defaults.
  - **Confirm item 2, confirmed.** ColorPlanes is `cfa.plane_color.len()` on a CFA
    plane, 3 on a CFA plane whose mosaic did not read, and `SamplesPerPixel`
    otherwise; `SamplesPerPixel` is now read before the colour tags (it was read in
    the struct literal; the scan that chose the IFD already read it cleanly, so the
    `?` cannot newly fire). So a 1-sample `LinearRaw` plane expects 3-entry matrices,
    9-entry calibrations and a 1-entry `AnalogBalance`. The mono path's DEC-078 set
    holds no colour tag, so the mono develop is unaffected. **No held mono file
    carries colour tags:** exiftool finds no `ColorMatrix1`, `CameraCalibration1` or
    `AnalogBalance` on any of the six held monochrome DNGs (all `LinearRaw`), and their
    `malformed_tags` are unchanged (`[]`, the K-3 III `[50713]`).
  - **Corrected (the design's and the CHANGELOG's "changed refusals" bucket too
    broad).** 0.2.5's colour code did not read every misshapen colour tag. Its
    `cam_to_xyz_single_illuminant` takes `ForwardMatrix1` before `ColorMatrix1`, and
    its `resolve_calibration` read `CameraCalibration1/2` only when applied. So:
    - a misshapen `ColorMatrix1` (or `ColorMatrix2`) on a file with a well-formed
      `ForwardMatrix1` sent 0.2.5 to the single-illuminant ForwardMatrix path and
      developed `Ok`. It is a **new** refusal, not "was `MissingColorCalibration`";
    - a misshapen `CameraCalibration1` with differing signatures (and
      `CameraCalibration2` off the dual-illuminant path) was the identity in 0.2.5 and
      developed. Also a new refusal;
    - a misshapen `ForwardMatrix1` on the dual-illuminant path took the ColorMatrix
      path in 0.2.5 (the CHANGELOG named only `ForwardMatrix2`).

    **Probed, not only read:** a scratch crate built against the `v0.2.5` worktree and
    against the candidate parsed the Canon EOS 5D Mark III (dual-illuminant, both
    forward matrices, `com.adobe` signatures), edited the parsed `Sensor`, and called
    `develop_color_into`. With `ColorMatrix1`, `ColorMatrix2` or `ForwardMatrix1`
    truncated to 8 entries, and with an 8-entry `CameraCalibration1` under differing
    signatures, 0.2.5 returned `Ok` every time (output sums differ from the baseline)
    and the candidate returned `MissingColorCalibration` every time. The probe edits a
    parsed `Sensor`, so it shows the colour code's half; the reader half (that such a
    tag in a file is now `MalformedTag`) is SPEC-089's tests. The genuinely changed
    refusals are a misshapen `AnalogBalance` (0.2.5 always checked it) and a misshapen
    matrix or calibration 0.2.5 did use. The CHANGELOG and the notes now say so.
    **Not a stop:** SPEC-089 (DEC-078 §7) specifies the new behaviour for all of these
    tags; only its account of what 0.2.5 did was incomplete (its AC7 probe fixtures
    had no `ForwardMatrix1`).
  - **Added, a precedence example the design did not name:** a CFA file whose
    `CFAPlaneColor` has 4 entries and whose colour matrices have 9 was
    `UnsupportedCfaPlaneColor` (raised in `demosaic.rs`, after
    `check_required_processing`) and is now `MalformedTag` for the matrix, since 4
    planes need 12. Read from the source, not probed.
  - **No behaviour change outside SPEC-089.** So the build does not stop on this item.
- **SDK colour cross-check** (run). A fresh `scripts/sdk-colour.sh` against the local SDK
  1.7.1 objects, into the build's scratchpad, wrote the five `*-camera-to-pcs.txt`
  files. With `IRRADIANCE_SDK_COLOUR_DIR` set, `--test colour_sdk -- --nocapture` (3
  passed) printed per-file maxima 6 / 6 / 1 / 2 / 5, bound 12, equal to 0.2.5's. Every
  one of the 90 deltas and all five white points equal 0.2.5's table digit for digit
  (the colour code's changes touch only unusable inputs).
- **SDK stage-2 cross-check** (run). A fresh `scripts/sdk-stage2.sh` with the local
  `dng_validate` 1.7.1, into the build's scratchpad, wrote 7 TIFFs; each sha256 equals
  the one recorded in HANDOFF-169. With `IRRADIANCE_SDK_STAGE2_DIR` set,
  `--test gain_map_corpus -- --nocapture` (11 passed) printed, Δ the SDK minus ours,
  **every figure equal to 0.2.5's table (SPEC-088) digit for digit:**

  | Comparison | File | max \|Δ\| | Exact |
  |---|---|---|---|
  | No gain | Canon 5D Mark III | 1, Δ ∈ {0, +1} | 99.5663% |
  | No gain | Pixel 4a | 1, Δ ∈ {0, +1} | 99.7911% |
  | No gain | Pixel 3a | 1, Δ ∈ {0, +1} | 99.8536% |
  | No gain | Pixel 7 Pro | 2 | 28.3471% |
  | End to end | Pixel 4a | 4 | 99.7729% |
  | End to end | Pixel 3a | 4 | 99.8210% |
  | End to end | Pixel 7 Pro | 4 | 28.3394% |
  | Gain step | Pixel 4a / 3a / 7 Pro | 1 / 1 / 1 | 99.9794% / 99.9672% / 99.9664% |
- **`irr ifd` against `v0.2.5`.** `irr` built with `--features irr` in a detached scratch
  worktree of `v0.2.5` and on the candidate, run on all 11 held DNGs (the 12th corpus
  file is the K-3 III PEF). The outputs, exit codes included, are byte-identical on
  every file (48 to 50 lines each). `malformed_tags` is `[]` everywhere except the
  K-3 III's `[50713]`.
- **Candidate timings.** Run in a clean detached worktree of `857959c`, stamped
  `v0.2.4-36-g857959c` (see the tag note below), on an Apple M4 Pro, release build,
  medians in ms. 0.2.5's recorded figure (SPEC-088) is in brackets.
  - `just unpack-timing`: Q2M `unpack_into` on 1 thread 9.8 (9.8); 8 units on 8 threads
    1.4 (1.4).
  - `just ljpeg-timing`: K-3 III 103.9 (102.2); Typ 246 90.0 (89.4); Pixel 4a 60.7
    (59.6).
  - `just orient-timing`, warp-free: O1 84 (83), O5 106 (97), O6 124 (114), O7 153
    (150), O8 153 (150).
  - `just orient-timing`, post-warp: O1 591 (574), O5 606 (607), O8 629 (618).

  The machine was shared: another session's test binary held two cores, and the load
  average was 7 to 12 throughout. The first `orient-timing` run (warp-free O6 406,
  min 306) was discarded as contended. Because warp-free O5 and O6 still read 9 to 10%
  above 0.2.5's recorded figures, `orient-timing` was then run back to back at
  `v0.2.5` and at the candidate, twice each, under the same load. On the second pair
  `v0.2.5` gave warp-free O1 84, O5 107, O6 123, O7 153, O8 151 and post-warp O1 592,
  O5 610, O8 630, which is the candidate's row above within 1%. (The first pair was
  noisy on both sides: `v0.2.5` O7 252, the candidate post-warp O5 693.) So the gap to
  0.2.5's recorded figures is this machine's state, not the code, as expected: the
  mono develop's only change is the shape checks at `Container::sensor()`, outside the
  timed develop.
- **Tag note.** `v0.2.5` on origin is a lightweight tag (`git ls-remote` shows no
  `v0.2.5^{}` line; `v0.2.0` to `v0.2.4` are all annotated), so `git describe` skips
  it and the examples stamp `v0.2.4-…`. The runbook's `git tag -a` is what 0.2.6
  should get; pre-flight item 1 adds a `git cat-file -t v0.2.6` check. Not fixed here
  (retagging a published release is the user's call).
- **Package list.** `cargo package --list` on clean detached worktrees of `v0.2.5` and
  `857959c` gives the same 19 paths: `.cargo_vcs_info.json`, `Cargo.lock`,
  `Cargo.toml`, `Cargo.toml.orig`, `LICENSE-APACHE`, `LICENSE-MIT`, `README.md`, and
  the 12 files under `src/`. There is no `scripts/`, `tests/`, SDK or corpus file. The
  dry run prints 51 "ignoring example/test … not included" warnings where 0.2.5 had
  50; diffed, the one extra is SPEC-089's new `tests/misshapen_tags.rs`.
- **Release notes against the reconciled list.** Each class above has a line in the
  notes: no output change; every newly refused input, the `ColorMatrix1`-with-
  `ForwardMatrix1` and unapplied-calibration classes included; each
  already-refused-different-error class and the precedence example; the inspection-
  visible changes (longer `malformed_tags`, `None` fields, the `Cfa` defaults, the
  1-sample `LinearRaw` ColorPlanes); and the composer-visible colour-code refusals.
  The notes add one known limitation, the unknown-`CalibrationIlluminant` fallback
  that SPEC-089 left out of scope, because the headline's "no silent fallback" would
  otherwise overclaim.
- **Cold verify (HANDOFF-188, 2026-11-23, on the PR head `0bec2de`).** Every
  build figure was re-derived, not read from the handback:
  - `just test` with the corpus and both SDK lanes (fresh `sdk-colour.sh` and
    `sdk-stage2.sh` runs into the verify scratchpad): 51 results, 859 passed, 0 failed,
    1 ignored, exit 0, "12/12 present", both lanes "will run". The colour lane's 90
    deltas and five white points, and the stage-2 table, equal 0.2.5's digit for digit;
    the 7 stage-2 TIFF sha256s equal HANDOFF-169's.
  - `just semver-check` (0.51.0): "v0.2.5 -> v0.2.6 (minor change)", 202 pass, 58 skip,
    no semver update required.
  - `irr ifd` at `v0.2.5` and at the head, on all 11 held DNGs: byte-identical, exit
    codes included.
  - `cargo publish --dry-run` on a clean detached worktree of `0bec2de`: 19 files,
    51 "ignoring" warnings, vcs sha1 `0bec2de…`, packaged `rust-version = "1.90"`.
    `cargo package --list` equals `v0.2.5`'s. `Cargo.toml` differs from `v0.2.5` only
    in the version line.
  - **The changed-input list, probed.** One probe source, built against a `v0.2.5`
    worktree and against the head, wrote 73 fixtures with the test-only DNG writer
    (both `Orientation` IFDs, every count-zero tag, every colour shape on the single
    and dual paths, 4-plane and `LinearRaw` ColorPlanes, short-buffer precedence) and
    six edited-`Sensor` cases, and ran `develop_into` or `develop_color_into`,
    `demosaic_into` and `to_srgb_linear_into` on each. Every class the build named was
    confirmed. Three were missing or misstated, and the CHANGELOG and notes above now
    say them:
    - **an output change.** An IFD0 `Orientation` with two values `[3, 1]` beside a
      sensor-IFD `Orientation` of 6 developed `Ok` at orientation 3 in 0.2.5 and
      develops `Ok` at orientation 6 now (different output hash, mono and colour). The
      wrong-count read is an `Err`, so the existing two-IFD recovery takes the other
      IFD's value. This is DEC-078 §7.1's recovery rule ("as a wrong-type one already
      is"), not a defect, but "no output change for any file that develops" was
      false for it. A wrong-count `Orientation` with no well-formed one in the other
      IFD is refused, as the notes said;
    - **`to_srgb_linear_into` refusals.** A 9-entry `ColorMatrix1` on a 1-sample
      `LinearRaw` plane, and on a plane whose `CFAPlaneColor` has 4 entries, was
      transformed `Ok` by 0.2.5's `to_srgb_linear_into` and is now
      `MalformedTag { tag: 50721 }`. The notes said the first "refuses" without saying
      it was `Ok` before, and did not name the second;
    - **the composer claim was too broad.** A parsed `Sensor` edited to carry an
      8-entry `ForwardMatrix2` on the single-illuminant path still develops `Ok` on
      both versions with the same output: `cam_to_xyz_single_illuminant` never reads
      `ForwardMatrix2`. From a file, the reader refuses it (`MalformedTag { tag:
      50965 }`, probed), so only a hand-built `Sensor` sees this.
  - **Also confirmed by probe:** the precedence rule (a count-zero `WhiteLevel` with a
    one-short buffer was `DevelopBufferWrongLength` or `RgbBufferWrongLength`, now
    `MalformedTag`); a count-zero `BlackLevelDeltaH/V` is now `UnsupportedTag` on every
    entry, `demosaic_into` and `to_srgb_linear_into` included; the mono develop is
    unaffected by a count-zero colour tag; the 4-plane precedence example was
    `UnsupportedCfaPlaneColor` with an RGGB-index pattern and `UnsupportedCfaPattern`
    when the pattern names the fourth plane; count-zero `AsShotNeutral`, a NUL-only
    signature and a zero-numerator `AnalogBalance` are unchanged. Every other
    `DEC-078` §7.1(a) tag (the arrays, `SampleFormat`, `LinearizationTable`, the
    interleave factors, the version pair) already refused count zero in 0.2.5, by
    reading the code.
  - **The `[cut]` smoke case, on the real file.** The Canon EOS 5D Mark III's parsed
    `Sensor` with `color_matrix_2` cut to 8 entries: `v0.2.5` `Ok`, the head
    `Err(MissingColorCalibration)` with the destination untouched. Unedited, both give
    the same output hash, and `DEVELOP_VERSION` is `"8"` on both, so only the edited
    case tells the releases apart.

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).** Filled
2026-11-23 by the Phase-1 build (HANDOFF-187) on candidate `857959c`
(`857959c209dee583ef26c4508af1cf6375ce3032`, the version bump). The commits after it
change only project records, and `src/` is byte-identical throughout. `just test` was
repeated on the PR head; the handback has that result.
- `[now]`: verified on the candidate in the build session.
- `[cut]`: verified by the user at Phase 2.
- `N/A`: does not apply.

- [ ] **1. Version / tag integrity & build provenance** — `[now]` done, the rest `[cut]`.
  - `[now]` ✔:
    - `Cargo.toml` says `version = "0.2.6"`. `cargo publish --dry-run` ran in a detached
      scratch worktree of `857959c`, on a clean tree with no `--allow-dirty`. It
      reports `Packaged 19 files, 1.8MiB (397.7KiB compressed)` and `Uploading
      irradiance v0.2.6`, then "aborting upload due to dry run".
    - `git tag -l 'v0.2.6'` is empty, locally and on origin.
    - The packaged `.cargo_vcs_info.json` carries `git.sha1`
      `857959c209dee583ef26c4508af1cf6375ce3032`, the candidate.
    - `git diff v0.2.5 -- Cargo.toml` before the bump was empty, and the bump changes
      only the version line. So `license`, `description`, `keywords`, `categories`,
      `repository`, `include` and `rust-version = "1.90"` are unchanged; the packaged
      manifest shows `rust-version = "1.90"`.
  - `[cut]`:
    - `git tag -a v0.2.6 -m "irradiance 0.2.6"` on the merged prep commit, after
      confirming its `Cargo.toml` shows 0.2.6.
    - `git tag --points-at HEAD` prints exactly `v0.2.6`, and `git cat-file -t v0.2.6`
      prints `tag` (an annotated tag; `v0.2.5` was pushed lightweight). After the
      push, `git ls-remote --tags origin 'v0.2.6*'` shows both `refs/tags/v0.2.6` and
      `refs/tags/v0.2.6^{}`, the line a lightweight tag lacks (added at verify: the
      local check alone does not show what origin received).
    - The published `.crate`'s `.cargo_vcs_info.json` sha1 equals the tagged commit:
      `curl -sL https://static.crates.io/crates/irradiance/irradiance-0.2.6.crate | tar -xzO irradiance-0.2.6/.cargo_vcs_info.json`
      against `git rev-parse v0.2.6^{commit}`.
- [x] **2. Artifact trust on a clean host** — `N/A`: a source crate, no signed binary.
- [ ] **3. Distribution-channel trust** — `[now]` proxy done, the rest `[cut]`.
  - `[now]` ✔:
    - `cargo publish --dry-run` packaged and verified the crate in isolation. Its only
      warnings are the expected "ignoring example/test … not included in the published
      package" lines (51: 0.2.5's 50 plus SPEC-089's `tests/misshapen_tags.rs`).
    - The packaged `Cargo.toml` has no self dev-dependency; its only dev-dependencies
      are `ssimulacra2` and `toml`.
    - `just package-targets-check` passed: no executable in the default build or in the
      package (`irradiance-0.2.6`), `irr` builds under `--features irr`, and its red case
      is caught. `just deny` and `just deny-fuzz` both report `licenses ok`.
    - The package list is the same 19 files as 0.2.5's: `cargo package --list` on a
      clean worktree of `v0.2.5` and of the candidate, diffed, is identical. No
      `scripts/`, `tests/`, SDK or corpus bytes.
  - `[cut]`: on a clean host, `cargo add irradiance@=0.2.6 && cargo build`. crates.io
    shows 0.2.6, and docs.rs is green.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` done on the candidate, `[cut]` on the
  published crate.
  - `[now]` ✔:
    - `just test` (on `c025e67`, `src/`-identical) ran with `IRRADIANCE_CORPUS_DIR`,
      `IRRADIANCE_SDK_COLOUR_DIR` and `IRRADIANCE_SDK_STAGE2_DIR` set. It printed
      "corpus: 12/12 present, exiftool and dnglab on PATH — no tier-B test will skip",
      and "sdk-stage2: present" and "sdk-colour: present", each followed by "will run".
      The 51 test results, doctests included, total 859 passed, 0 failed and 1 ignored,
      exit 0, in 283 s. Every `dnglab` checksum pin and every develop MD5 pin held.
    - `cargo test --doc --all-features`: 4 passed. `just doc-check`: 2 feature
      surfaces, 0 rustdoc warnings.
    - The `[cut]` probe's distinguishing case was run in this session against path
      dependencies: on the Canon EOS 5D Mark III's parsed `Sensor`, `color_matrix_2`
      cut to its first 8 entries gives `Ok` at `v0.2.5` and
      `Err(MissingColorCalibration)` on the candidate (internal record).
  - `[cut]`: in a clean-host probe crate on `irradiance = "=0.2.6"`:
    - `develop_color_into` on the Pixel 7 Pro and `develop_into` on the Pentax K-3 III
      Monochrome return `Ok`, with output hashes equal to the same program built
      against `main`;
    - on the Canon EOS 5D Mark III's parsed `Sensor` (dual-illuminant), replace
      `color_matrix_2` with its first 8 entries. `develop_color_into` returns
      `Error::MissingColorCalibration` and writes nothing. (0.2.5 developed it as
      single-illuminant colour, so this tells the two releases apart. The reader is
      bypassed, so the error is the colour code's §7.2 one, not `MalformedTag`.)
    - `irradiance::DEVELOP_VERSION == "8"`.
- [ ] **6. Rollback / uninstall** — `[cut]`.
  - **Withdraw:** `cargo yank --version 0.2.6`, and dependents on `"0.2"` fall back to
    0.2.5. `cargo yank --undo --version 0.2.6` reverses it.
  - **Git rollback:** revert the Phase-1 PR.

**Gates on the candidate, all green** (2026-11-23, `IRRADIANCE_CORPUS_DIR` set):
- `just test` (above)
- `lint-ci`
- `lint-no-allow`
- `lint-fuzz`
- `fmt-check`
- `validate`
- `doc-check`
- `deny`, plus `deny-fuzz`
- `msrv` (1.90.0)
- `cost-audit`
- `gate-audit`
- `name-length-check`
- `decisions-index --check`
- `semver-check` (202 of 202)
- `package-targets-check`
- `cargo publish --dry-run`

## Notes for the Implementer

- The SDK lanes are local only (DEC-069 §6): the SDK, its outputs and the corpus are never
  committed or packaged. Run both scripts into a scratchpad outside the repository. Set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`; `just test`
  already passes `--all-features`.
- A defect found during prep is reported in the handback, not fixed here: the stage is
  release-only.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).
- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, with a clean tree; confirm `Cargo.toml` shows
     `version = "0.2.6"`.
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.6 -m "irradiance 0.2.6"`, then `git push origin v0.2.6`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5 (the orchestrator can run the
     clean-host crate), reported back for the Cut Record.
  6. Optional: a GitHub release on `v0.2.6` with the `## Release Notes` text, down to
     the internal record.

## Cut Record

v0.2.5 → **v0.2.6 published 2026-11-23** (crates.io, by the user, 16:30 UTC; not yanked;
checksum `b08f960ad58ac617…`). Tag `v0.2.6` is **annotated** (`d3fef8a`; `ls-remote` shows the
`^{}` line) and points at `50d533d`, the PR 322 merge, whose `Cargo.toml` says 0.2.6; the
published `.crate`'s `.cargo_vcs_info.json` `git.sha1` is
`50d533d0823c6d5f906d6675d1eda3e677549459`, the tagged commit. docs.rs had not built 0.2.6 when
this was recorded. Clean-host check, run by the orchestrator in scratch crates on
`irradiance = "=0.2.6"` and `"=0.2.5"` from crates.io and on `main`'s source: all build;
`DEVELOP_VERSION` is `8`; the Pixel 7 Pro (`develop_color_into`, 3866×2908), the K-3 III
Monochrome (`develop_into`, 6192×4128, `malformed_tags [50713]`) and the Canon EOS 5D Mark III
(colour, 5760×3840) give identical output hashes on all three. With the Canon's parsed
`color_matrix_2` cut to 8 entries, 0.2.6 returns `Err(MissingColorCalibration)` with `dst`
untouched, while 0.2.5 returns `Ok` and writes. Empty and misshapen interpretation tags now
refuse on crates.io.

---

## Reflection

*Appended during **ship**. Four questions, short answers.*

1. **What would I do differently next release?**
   — At design, probe the changed-input list against the previous release instead of
   deriving it from the CHANGELOG and a read of the diff. Both later cycles probed
   `v0.2.5` against the candidate with edited `Sensor`s and byte fixtures, and each
   found classes the design had wrong: the build moved two "changed refusals" to new
   refusals, and the verify found an output change (a multi-value IFD0 `Orientation`
   now recovered from the sensor IFD), two `to_srgb_linear_into` refusals and one
   overstated composer claim.

2. **Does any template, constraint, or decision need updating?**
   — Yes, done at this close: the release-spec template's Phase 2 now asks for an
   annotated tag and a remote `^{}` check (0.2.5 was pushed lightweight). The
   release-notes signal stays codified; this release adds a note that its line held
   only because the build and verify probed.

3. **Is there a follow-up spec I should write now before I forget?**
   — None required. The verify's note (the single-illuminant path never reads a
   hand-built `Sensor`'s `ForwardMatrix2`) is not a defect against DEC-078 §7.2. Retagging
   `v0.2.5` as annotated is the user's call.

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — verify
