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
  id: SPEC-088
  type: release                    # epic | story | task | bug | chore | release
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

project:
  id: PROJ-002
  stage: STAGE-028
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-067, DEC-006, DEC-076, DEC-077, DEC-078]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-083, SPEC-085, SPEC-086, SPEC-087]  # [SPEC-NNN]

value_link: "irradiance 0.2.5 on crates.io carries STAGE-027 and PR 301, so the published crate matches what main develops and refuses"

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 15000000
  sessions:
    - cycle: build
      handoff: HANDOFF-184
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 17708903
      estimated_usd: 12.32
      duration_minutes: 60
      recorded_at: 2026-11-23
      notes: "TRUE cost, re-summed at ship from the whole prep worktree transcript dir (89 unique message.ids, all claude-opus-5-5, no subagents): input 182 + cache-creation 204,761 (1-hour) + cache-read 17,441,755 + output 62,205, opus-5-5 rates. The handback floor was 16,659,440 and $11.62. PR 316 (Phase 1); semver clean; SDK colour 6/6/1/2/5 and stage-2 equal to 0.2.4's; changed-input reconciliation found no behaviour change outside every spec and PR."
    - cycle: verify
      agent: claude-opus-5-5
      interface: claude-code
      basis: transcript
      tokens_total: 6112132
      estimated_usd: 3.53
      duration_minutes: 15
      recorded_at: 2026-11-23
      notes: "NOT a cold verify. The orchestrator's own review stood in for one: it read PR 316's diff, release notes, reconciliation and SDK figures against the spec's stop conditions before the user merged, then after the publish checked the tag, crates.io, docs.rs and ran a clean-host smoke crate on =0.2.5 whose Pixel 7 Pro colour and K-3 III mono hashes equal main's. Measured from the orchestrator transcript (session a0013105) from the first look at release/0.2.5-prep (2026-11-23T02:41Z) to this record: 15 unique message.ids, input 34 + cache-creation 20,450 (1-hour) + cache-read 6,080,162 + output 11,486, opus-5-5 rates. A floor. 0.2.4's release spec had a cold verify (HANDOFF-174) that found a missed changed input; this one did not."
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop orchestration (STAGE-028 frame and this design in PR 315, with a semver-check on main); counted at stage grain in STAGE-028's orchestration_cost."
    - cycle: ship
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-23
      notes: "Un-metered main-loop ship: the orchestrator's prep review, the post-publish checks (tag, crates.io, docs.rs, the clean-host smoke crate) and this record; counted at stage grain in STAGE-028's orchestration_cost. There was no separate cold verify cycle: the orchestrator reviewed the prep PR against the spec's stop conditions before the user merged, tagged and published."
  totals:
    tokens_total: 23821035
    estimated_usd: 15.85
    session_count: 4
shipped_at: 2026-11-23
---

# SPEC-088: irradiance 0.2.5 release: calibration, converged weight, unhonoured and malformed tag refusals

## Context

The release spec of STAGE-028 (PROJ-002). 0.2.4 was published on 2026-11-21 (tag `v0.2.4`
on `ef37e6b`). Since then `main` has shipped, 28 commits to `5a8eb61`:
- **PR 301** (2026-11-21, no spec): the Codex review fixes. Non-finite warp coefficients,
  staggered `CFALayout`, `CFAPlaneColor`, the opcode version guard, and the new public
  `PlaneDecoder`. **It added no CHANGELOG entry.**
- **PR 302:** docs only.
- **STAGE-027:** SPEC-085 (DEC-076, the camera calibration and the converged illuminant
  weight, `DEVELOP_VERSION` 8), SPEC-086 (DEC-077, refusing tags that change what the
  samples mean, a newer DNG backward version, and a Compression 7 precision mismatch) and
  SPEC-087 (DEC-078, refusing a malformed tag the develop path reads).

This spec ends prep-complete. The user tags and runs `cargo publish`, as for 0.2.4
(SPEC-083).

## Release Scope

### Decided for 0.2.5 (design, 2026-11-22)

- **Version / tag:** `0.2.5`, tag `v0.2.5`, a **patch** over the live `0.2.4`. Dependents
  on `irradiance = "0.2"` receive it with `cargo update`.
- **Why it is compatible: the design's mechanical check.** `just semver-check` (crates.io
  baseline, DEC-067) on `main` at `5a8eb61` reports "202 checks: 202 pass, 58 skip …
  Summary no semver update required". The public additions are all additive:
  - the new type `PlaneDecoder`;
  - new fields on the `#[non_exhaustive]` `Sensor`;
  - new variants of the `#[non_exhaustive]` `Error` (eight since 0.2.4, 68 to 76) and
    `DevelopPath` (`Demosaic`, `ColourTransform`).

  This follows 0.2.4's precedent: a patch for an output change and new refusals.
  **If the build's semver check reports any breaking change, the build stops for the
  user's decision** (0.3.0 or a fix).
- **Every input whose result changes since `v0.2.4`.** The design's list follows. The
  build must re-derive it from `git diff v0.2.4 <candidate> -- src/` (every `Err(` path,
  every new check, every reordering) and reconcile the two. A difference is a finding,
  not an edit to make silently. Signal
  `release-notes-diff-every-changed-refusal-and-what-it-preempts` asks for this.
  - **Output changes:**
    - Colour develops of every held dual-illuminant file change: the Pixel 7 Pro, 3a
      and 4a, the Canon EOS 5D Mark III and the Nikon D300S (SPEC-085: calibration
      applied and `NeutralToXY` converged). Every held file is now within 7 codes of
      Adobe DNG SDK 1.7.1's camera-to-XYZ transform in SPEC-085's probe.
    - Mono output does not change. `DEVELOP_VERSION` goes from `"7"` to `"8"`.
    - A file with an optional opcode newer than DNG 1.7.1.0 now skips it (PR 301). The
      build should confirm whether 0.2.4 applied such an opcode, for example an optional
      `GainMap`, and say so either way.
  - **Newly developing:** a dual-illuminant file whose singular `ColorMatrix` (or applied
    calibration) the converged weight never uses. 0.2.4 refused it with
    `MissingColorCalibration`. Whether it develops depends on the scene (SPEC-085,
    DEC-076 §3, DEC-078 §4).
  - **Newly refused:**
    - PR 301: non-finite `WarpRectilinear` coefficients (`MalformedOpcodeParams` at parse;
      `NonFiniteWarpCoefficients` from `apply_warp_into`), `CFALayout` other than 1
      (`UnsupportedCfaLayout`), `CFAPlaneColor` other than `[0, 1, 2]`
      (`UnsupportedCfaPlaneColor`), and a mandatory opcode newer than 1.7.1.0
      (`UnsupportedOpcodeVersion`).
    - SPEC-085: an applied calibration or `AnalogBalance` that is unusable, or a
      non-positive reference neutral on the ForwardMatrix path
      (`MissingColorCalibration`).
    - SPEC-086: a DNG backward version above 1.7.1.0 (`UnsupportedDngVersion`), the seven
      tags with a value that is not an exact no-op (`UnsupportedTag`), and, **at plane
      read**, a Compression 7 frame whose precision is not `BitsPerSample`
      (`LjpegPrecisionMismatch`).
    - SPEC-087: a malformed tag the path reads (`MalformedTag`), and a malformed
      `DNGVersion` with no backward version (`UnsupportedTag { tag: 50706 }`).
  - **Already refused, different error:** every new develop check runs first in
    `check_required_processing`, so an input that 0.2.4 refused with a later error, and
    that also trips a new check, now reports the new one. Examples to confirm or correct:
    - a malformed `WhiteLevel` together with an unreadable `OpcodeList`, which was
      `UnreadableOpcodeList`;
    - a malformed `CFAPattern` together with a malformed `CFALayout`, which was
      `NotAColorFilterArray`;
    - any input that also fails `InvalidLevels`, `InvalidActiveArea` or
      `MissingColorCalibration`.

    The build names each class it finds.
  - **Composer-visible:** `check_required_processing(sensor, DevelopPath::Colour)` now also
    refuses a malformed levels, geometry, CFA or colour tag. `demosaic_into`,
    `demosaic_into_threaded` and `to_srgb_linear_into` now check their own, narrower sets.
- **No held file changes behaviour** except the colour output above. Every held file still
  develops, and the Pentax K-3 III Monochrome's malformed `BlackLevelRepeatDim` stays
  exempt.
- **What's included:** the CHANGELOG's `[Unreleased]` section, plus the PR 301 entries the
  build adds.
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes.

1. **The PR 301 CHANGELOG entries.** Add them to `[Unreleased]`, under Added (`PlaneDecoder`,
   the four `Error` variants) and Changed (the four refusals, the `CFAPattern` reading,
   the opcode version rule). Take the facts from PR 301's description
   (`gh pr view 301 --repo jysf/irradiance`) and its diff, not from memory.
2. **The changed-input diff** (Release Scope above): reconcile the design's list with
   `git diff v0.2.4 <candidate> -- src/`, and record the result in the internal release
   record.
3. **The semver check:** `just semver-check`, recording the tool version and the full
   summary. Any reported breaking change stops the build for the user's decision.
4. **Version:** `Cargo.toml` `version = "0.2.5"`, with every other field unchanged from
   0.2.4 (`git diff v0.2.4 -- Cargo.toml` before the bump is empty).
5. **CHANGELOG** (Keep a Changelog): promote `[Unreleased]` to `## [0.2.5] - <date>`, with
   a short lead paragraph like 0.2.4's and a fresh empty `## [Unreleased]` on top.
   **Correct any entry whose wording the changed-input diff contradicts.**
6. **Release notes:** run `just release-notes PROJ-002`, edit into `## Release Notes` below
   in 0.2.4's shape (Headline, Changed, Behaviour changes, Breaking / action required,
   Known limitations, then the internal record), and check them line by line against the
   reconciled changed-input list. Update Known limitations: `BlackLevelDelta*` and
   `LinearizationTable` are now refused rather than ignored, the Pixel 7 Pro's colour gap
   is closed to within 6 codes, and the ColorChecker ΔE is still pending.
7. **README and crate docs:** check they still describe 0.2.x accurately (`cargo test
   --doc`, `just doc-check`). Wording fixes only.
8. **Tier-B evidence on the candidate SHA,** with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`: all 12 held files present, every pin held.
   - The SDK colour cross-check, with `IRRADIANCE_SDK_COLOUR_DIR` set from a fresh
     `scripts/sdk-colour.sh` run into the scratchpad. Record the 18 deltas per file; they
     must equal SPEC-085's 6 / 6 / 1 / 2 / 5 maxima.
   - The stage-2 cross-check, with `IRRADIANCE_SDK_STAGE2_DIR` from a fresh
     `scripts/sdk-stage2.sh` run. Record the table as 0.2.4 did and compare it with
     0.2.4's. The colour change does not touch stage 2, so every figure should be equal;
     say so or explain.
   - The three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded against 0.2.4's figures.
9. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`,
   `cargo publish --dry-run`. The package list must be the same files as 0.2.4: no
   `scripts/`, `tests/`, SDK or corpus bytes.
10. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), using SPEC-083's
    filled pre-flight as the model. Then run every gate: `just test`, `lint-ci`,
    `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`,
    `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
    `semver-check`, `package-targets-check`.

## Release Notes

*Outward-facing, written for whoever uses the crate. `just release-notes PROJ-002` was run
at build: it lists 17 notes from STAGE-006 to STAGE-014 and none from STAGE-027 or PR 301,
so these notes are written from the final CHANGELOG instead, as for 0.2.4, with SPEC ids
dropped, and checked line by line against the reconciled changed-input list in the
internal record. The GitHub release text for `v0.2.5` is this section, down to the
internal record.*

**Headline:** Colour now follows Adobe's DNG SDK more closely, and inputs that used to
develop wrongly are refused. The colour develop applies `AnalogBalance` and the camera
calibration, and runs the illuminant weight to convergence. On each of the five held
colour files, its camera-to-sRGB transform is now within 6 codes of the SDK's on 18
probe values; the Pixel 7 Pro was 1,160 codes off before. Files the develop cannot honour
are refused with a typed error, where they used to return `Ok` with wrong pixels:
- tags that change what the samples mean;
- a DNG backward version above 1.7.1.0, or a malformed tag the develop reads;
- a staggered or non-RGB CFA;
- a non-finite lens warp;
- a mandatory opcode newer than DNG 1.7.1.0.

A compatible 0.2.x release: run `cargo update -p irradiance`.

**Added**
- **`plane::PlaneDecoder`** checks a plane's layout once and then decodes each unit in
  time proportional to that unit. Decoding every one-row band of a 900-strip plane falls
  from about 3 ms to 16 µs. It is `Copy`, `Send` and `Sync`, and its results, errors
  included, equal `plane::unpack_unit_into`'s.
- **`Sensor` reads more tags:**
  - the calibration tags: `camera_calibration_1/2`, `analog_balance` and the two
    calibration signatures;
  - the DNG version pair, `dng_version` and `dng_backward_version`;
  - the tags that change what the samples mean: `sample_format`,
    `linearization_table`, `black_level_delta_h/v`, `sub_tile_block_size` and the two
    interleave factors.

  Each has a public tag constant. A malformed one is recorded in
  `Sensor::malformed_tags`, as for every other tag, so that list can now be longer for
  the same file.
- **Eight `Error` variants**, appended so the existing discriminants stay put:
  - `NonFiniteWarpCoefficients` (`CallerError`);
  - `UnsupportedOpcodeVersion`, `UnsupportedCfaLayout`, `UnsupportedCfaPlaneColor`,
    `UnsupportedDngVersion` and `UnsupportedTag` (`Unsupported`);
  - `LjpegPrecisionMismatch` and `MalformedTag` (`Corrupt`).
- **`DevelopPath::Demosaic` and `DevelopPath::ColourTransform`,** so
  `opcode::check_required_processing` checks only the tags each stage primitive reads.

**Changed**
- **The colour develop applies the camera calibration and converges the illuminant
  weight.** All four colour entries and `to_srgb_linear_into` apply `AnalogBalance`
  always. They apply `CameraCalibration1/2` when the camera and profile calibration
  signatures are equal, an absent one counting as empty, as Adobe's SDK does. The
  `NeutralToXY` weight now starts at D50 and iterates to convergence (at most 30
  passes), instead of 3 passes from illuminant 1. Against Adobe DNG SDK 1.7.1's own
  camera-to-XYZ matrix, the largest gap on 18 probe values per file is now:

  | File | 0.2.4 | 0.2.5 |
  |---|---|---|
  | Pixel 7 Pro | 1,160 codes | 6 codes |
  | Pixel 3a | 39 codes | 6 codes |
  | Pixel 4a | 23 codes | 1 code |
  | Canon EOS 5D Mark III | 3 codes | 2 codes |
  | Nikon D300S | 5 codes | 5 codes |

  The Nikon's output still moves, from the converged weight.

**Behaviour changes**
- **`DEVELOP_VERSION` goes from `"7"` to `"8"`,** so a pixel cache keyed on it
  re-develops.
- **Colour output changes on every held colour file:** the Pixel 7 Pro, 3a and 4a, the
  Canon EOS 5D Mark III and the Nikon D300S, all dual-illuminant. Mono output does not
  change on any held file.
- **A dual-illuminant file can now take a different colour path.**
  - 0.2.4 began the weight loop at illuminant 1, so a singular `ColorMatrix1` (or its
    applied calibration) sent the file to the single-illuminant fallback. That fallback
    developed it from `ForwardMatrix1` when there was one, and refused it with
    `MissingColorCalibration` when there was not.
  - 0.2.5 starts at D50 and inverts only the matrix at the weight it settles on. Such a
    file now develops from the interpolated transform when that weight avoids the
    singular matrix, and is refused with `MissingColorCalibration` when the weight needs
    it. So whether it develops, and with which matrix, depends on the scene.
- **Newly refused, before anything is written:**
  - **On every develop entry**, through `opcode::check_required_processing`:
    - `UnsupportedDngVersion`: the effective DNG backward version is above 1.7.1.0
      (`DNGBackwardVersion`, else `DNGVersion` with its last two bytes zeroed).
    - `UnsupportedTag`: the file carries one of these values: a `SampleFormat` other
      than 1, any `LinearizationTable`, a non-zero `BlackLevelDeltaH` or
      `BlackLevelDeltaV`, a `SubTileBlockSize` other than `[1, 1]`, or an interleave
      factor other than 1. A malformed one of these tags, or a malformed
      `DNGBackwardVersion`, is refused the same way. So is a malformed `DNGVersion`
      when there is no backward version.
    - `MalformedTag`: a tag the path reads is malformed. The mono set is
      `Orientation`, `BlackLevel`, `WhiteLevel`, `DefaultCropOrigin/Size` and
      `ActiveArea`, plus `BlackLevelRepeatDim` when there is more than one black. The
      demosaic set is `CFAPlaneColor` and `CFALayout`. The colour-transform set is the
      colour matrices, calibrations, `AnalogBalance`, the illuminants, the signatures
      and the forward matrices. The colour develop checks all three sets. Such a file
      used to develop as if the tag were absent. The Pentax K-3 III Monochrome, whose
      `BlackLevelRepeatDim` is malformed but has one black, still develops.
    - `UnsupportedOpcodeVersion`: a mandatory opcode declares a DNG version above
      1.7.1.0. 0.2.4 ignored the version, and applied such an opcode or refused it
      with `UnsupportedMandatoryOpcode`, `UnsupportedRequiredOpcode` or
      `MalformedOpcodeParams`.
  - **On the demosaic and every colour develop entry:**
    - `UnsupportedCfaLayout`: a `CFALayout` other than 1. Staggered layouts used to be
      demosaiced as a rectangular grid.
    - `UnsupportedCfaPlaneColor`: a `CFAPlaneColor` other than `[0, 1, 2]`.
      `CFAPattern` values are now read directly as colour codes. 0.2.4 remapped them
      through `CFAPlaneColor`, which swapped colours whenever it was not the identity.
  - **On every colour develop entry and `to_srgb_linear_into`:**
    `MissingColorCalibration` when the applied
    `AnalogBalance` or calibration is unusable, or when a calibrated ForwardMatrix path
    has a non-positive reference neutral.
  - **On every `WarpRectilinear` parser and the mono develop:**
    `MalformedOpcodeParams` for a NaN or infinite coefficient or optical centre. Under
    the default `Framing::StandardCrop` such a warp used to resample to black and
    return `Ok`. `warp::apply_warp_into` refuses one with `NonFiniteWarpCoefficients`.
  - **At plane decode:** `LjpegPrecisionMismatch` for a Compression 7 frame whose
    precision is not `BitsPerSample`. This is raised by `unpack_into`,
    `unpack_unit_into`, `PlaneDecoder::unpack_unit_into` and the tile path, after the
    shape check. Such a frame used to decode and then clip in the develop.
- **Optional opcodes newer than DNG 1.7.1.0 are skipped.** 0.2.4 applied an optional
  `WarpRectilinear`, `FixBadPixelsConstant` or `GainMap` whatever its declared version,
  or refused it with `MalformedOpcodeParams` when its parameters did not parse. 0.2.5
  skips it without reading its parameters. A later supported instance of the same ID is
  then the one applied. No held file carries one.
- **An input that was already refused can report a different error.**
  - The new tag checks run first in `opcode::check_required_processing`, which every
    develop entry calls before its buffer, geometry, options, levels, `GainMap`, CFA
    and colour-matrix checks, and before its own opcode-list checks. An input that 0.2.4
    refused for one of those reasons, and that also trips a new check, now reports
    `UnsupportedDngVersion`, `UnsupportedTag` or `MalformedTag`. For example, a
    malformed `WhiteLevel` with an unreadable `OpcodeList` was `UnreadableOpcodeList`.
    A malformed `CFAPattern` still gives `NotAColorFilterArray`.
  - The CFA checks come before the colour matrix, so a staggered or non-RGB CFA with an
    unusable `ColorMatrix` was `MissingColorCalibration` and now reports the CFA error.
  - Under `Framing::FullActiveArea` a non-finite warp was
    `UnsupportedWarpedFullActiveArea` and is now `MalformedOpcodeParams`.
  - The mono develop's `warp_alpha` argument check still comes first.

**Breaking / action required:** none for the API. `cargo semver-checks` against crates.io
0.2.4 reports no semver update required. Every public addition is additive: a new type,
new constants, and new fields and variants on `#[non_exhaustive]` types. There is no new
dependency or feature, and the MSRV stays 1.90.

Two kinds of caller should check their code:
- one that maps errors, since the new variants reach its wildcard arm (`Error` is
  `#[non_exhaustive]`), or that keys on a specific error for an input that now trips an
  earlier check;
- one that composes the stage primitives itself gets the narrower checks the
  primitives now run.

**Known limitations**
- **No colour-accuracy claim yet.** The SDK match above is a transform check on probe
  values, not a measured colour error. The ColorChecker ΔE2000 measurement is still
  blocked on capturing the chart.
- **Tags that change what the samples mean are refused, not applied.** This covers
  `LinearizationTable`, a non-zero `BlackLevelDeltaH` or `BlackLevelDeltaV`, a
  `SampleFormat` other than 1 (floating-point DNGs), a `SubTileBlockSize` other than
  `[1, 1]` and an interleave factor other than 1. No held file carries one.
- **A staggered CFA (SuperCCD-style `CFALayout` 2 to 9) or a non-RGB `CFAPlaneColor`**
  is refused, not demosaiced.
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

- **The candidate.** Branch `release/0.2.5-prep`, cut from `origin/main` at `2724fff` (the
  PR 315 merge). The build's commits touch only `CHANGELOG.md`, `Cargo.toml` (the
  version line), `README.md` (wording) and this project's records. `src/` is
  byte-identical to `2724fff` and to `5a8eb61`, the design's semver point.
  Where the evidence ran:
  - `just test` with the corpus and both SDK lanes: on `2724fff`, then again on the
    PR head (handback);
  - timings, gates and the package checks: on `92ad58f`, the version bump.
- **Mechanical semver check:** `cargo-semver-checks 0.51.0`, crates.io 0.2.4 to the
  candidate, by `just semver-check` (DEC-067). Clean: no failure, nothing to accept.
  Verbatim summary, run after the version bump (`92ad58f`):

  ```
  Checking irradiance v0.2.4 -> v0.2.5 (minor change)
   Checked [   0.009s] 202 checks: 202 pass, 58 skip
   Summary no semver update required
  ```

  Before the bump (`bde275b`) the same run gave "v0.2.4 -> v0.2.4 (no change; assume
  minor) … 202 pass, 58 skip … no semver update required". As for 0.2.4, the tool labels
  a 0.x patch bump a "minor change"; no update is required.
- **Changed-input reconciliation** (step 2). The build read `git diff v0.2.4 2724fff --
  src/` (9 files, +1953/−222, from PR 301, SPEC-084, PR 302, SPEC-085, SPEC-086 and
  SPEC-087), reading every new or changed `Err(` path and every reordering. SPEC-084
  touched only a `#[cfg(test)]` module in `src/plane.rs`, and PR 302 touched doc comments
  only. Against the design's list:
  - **Confirmed as designed:**
    - every output change, the `DEVELOP_VERSION` bump and the mono no-change;
    - the singular-`ColorMatrix` newly-developing class;
    - every newly refused class: PR 301's four, SPEC-085's `MissingColorCalibration`
      conditions (`resolve_calibration`, `positive_rational`, `fm_path_cam_to_xyz`),
      SPEC-086's three and SPEC-087's two;
    - `LjpegPrecisionMismatch` at plane read (`check_precision`, after the shape check,
      on the strip and tile paths);
    - the composer-visible narrower path sets;
    - the precedence rule: `check_required_processing` already ran first at every
      develop entry's call site in 0.2.4 (unchanged call sites), and now runs
      `check_unhonoured_tags`, then `check_malformed_tags`, before `check_lists`.
  - **Confirmed, the design's open question answered:** 0.2.4 did apply an optional
    opcode newer than 1.7.1.0. Its `read_raw_opcode` read the version into `_version`
    and dispatched on the ID alone. So an optional too-new `WarpRectilinear` or
    `FixBadPixelsConstant` (mono) or `GainMap` (both paths) was parsed and applied.
  - **Corrected (design example wrong):** "a malformed `CFAPattern` together with a
    malformed `CFALayout`" does not change error. `Container::parse_cfa`
    (`src/ifd.rs`, from line 2340) returns at the `CFAPattern` read when the pattern is
    malformed, so `CFALayout` and `CFAPlaneColor` are never read and never recorded in
    `malformed_tags`. That input still gets `NotAColorFilterArray`. This is a reading of
    the source, not a probe. The CHANGELOG and notes use a malformed `CFALayout` with no
    usable `ColorMatrix` instead (`MissingColorCalibration` to `MalformedTag`). That
    example is also from the source: the malformed layout defaulted to 1 in 0.2.4, and
    the matrix error comes after the CFA.
  - **Added, each described by its spec or PR but missing from the design's list:**
    - Already refused, different error, from PR 301: a mandatory too-new opcode was
      `UnsupportedMandatoryOpcode` (unknown ID), `UnsupportedRequiredOpcode` (an ID the
      path does not apply) or `MalformedOpcodeParams`, and is now
      `UnsupportedOpcodeVersion`. A staggered or non-RGB CFA with an unusable
      `ColorMatrix` was `MissingColorCalibration` and is now the CFA error. Under
      `Framing::FullActiveArea` a non-finite warp was `UnsupportedWarpedFullActiveArea`
      (the `full_active_area_admits` finiteness clause) and is now
      `MalformedOpcodeParams` from the parser. PR 301's description names the last one
      ("refused at parse time under both framings").
    - Newly developing, from PR 301: an optional too-new opcode whose parameters do not
      parse was `MalformedOpcodeParams` and is now skipped.
    - Output, from PR 301: a too-new optional opcode followed by a supported one of the
      same ID. 0.2.4 applied the first; 0.2.5 applies the later one.
    - Already refused, different error, from SPEC-086/087: the buffer-length errors
      (`SourcePlaneWrongLength`, `DevelopBufferWrongLength`,
      `DemosaicSourceWrongLength`, `RgbBufferWrongLength`), the options errors and the
      `GainMap` errors join the list the design named. All of them run after
      `check_required_processing`.
    - Colour path, from SPEC-085: a dual-illuminant file that 0.2.4 sent to the
      single-illuminant fallback because the first pass inverted a singular
      `ColorMatrix1`. If it has a `ForwardMatrix1`, 0.2.4 developed it from that; 0.2.5
      can take the interpolated path instead, with different output. This is the
      design's newly-developing mechanism (DEC-076 §5) seen from the other side; no new
      refusal follows from it.
    - Composer-visible, from SPEC-085/086: `Sensor::malformed_tags` gains entries for
      the newly read tags (DEC-012's contract). The `irr` binary (feature `irr`) prints
      the five calibration fields.
  - **No behaviour change outside every spec and PR.** So the build does not stop on
    this item. The CHANGELOG's SPEC-085 sentence "left to SPEC-087" was corrected to
    DEC-078 §4's outcome.
- **SDK colour cross-check** (run). A fresh `scripts/sdk-colour.sh` against the local SDK
  1.7.1 objects, into the build's scratchpad, wrote the five `*-camera-to-pcs.txt`
  files. With `IRRADIANCE_SDK_COLOUR_DIR` set, `--test colour_sdk -- --nocapture` (3
  passed) printed the 18 deltas per file (ours minus the SDK's, in codes). The maxima
  equal SPEC-085's AC4 exactly, 6 / 6 / 1 / 2 / 5, bound 12:

  | File | max \|Δ\| | white xy | 18 deltas |
  |---|---|---|---|
  | Pixel 7 Pro | 6 | 0.319745 0.333331 | −5 1 2 −6 1 2 −4 2 3 −6 2 2 −6 1 3 −5 2 2 |
  | Pixel 3a | 6 | 0.359539 0.373618 | −5 1 2 −6 1 1 −4 1 3 −5 2 3 −6 1 3 −4 1 2 |
  | Pixel 4a | 1 | 0.339264 0.363595 | 0 0 0 1 0 1 0 0 −1 0 0 0 0 0 0 0 0 0 |
  | Canon 5D Mark III | 2 | 0.397340 0.400342 | 0 0 0 1 −1 −1 0 1 1 0 0 −2 0 0 1 0 0 −2 |
  | Nikon D300S | 5 | 0.345475 0.352773 | −4 1 3 −5 2 1 −5 1 3 −5 1 2 −5 1 2 −4 1 2 |
- **SDK stage-2 cross-check** (run). A fresh `scripts/sdk-stage2.sh` with the local
  `dng_validate` 1.7.1, into the build's scratchpad, wrote 7 TIFFs. Each sha256 equals
  the one recorded in HANDOFF-169 / HANDOFF-171. With `IRRADIANCE_SDK_STAGE2_DIR` set,
  `--test gain_map_corpus -- --nocapture` (11 passed) printed the numbers below. Δ is the
  SDK minus ours. **Every figure equals 0.2.4's table (SPEC-083) digit for digit.**
  That is expected: the colour transform is applied after stage 2.

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
- **Candidate timings.** Run on `92ad58f`, stamped `v0.2.4-31-g92ad58f`, on an Apple M4
  Pro, release build, medians in ms. 0.2.4's figure is in brackets.
  - `just unpack-timing`: Q2M `unpack_into` on 1 thread 9.8 (9.9); 8 units on 8 threads
    1.4 (1.4).
  - `just ljpeg-timing`: K-3 III 102.2 (100.5); Typ 246 89.4 (87.3); Pixel 4a 59.6
    (58.8). This is the second run. The first run's Typ 246 median was 166.1 with a min
    of 91.4, an outlier; its other two rows were 105.2 and 61.4.
  - `just orient-timing`, warp-free: O1 83 (85), O5 97 (98), O6 114 (114), O7 150
    (143), O8 150 (143).
  - `just orient-timing`, post-warp: O1 574 (570), O5 607 (610), O8 618 (609).

  All are within 3% of 0.2.4 except warp-free O7 and O8, which are 5% slower on both
  runs (151 and 156, then 150 and 150). The mono develop's only change is the tag checks
  in `check_required_processing`, once per develop, so this is not attributed to a code
  change. It was not investigated further.
- **Package list.** `cargo package --list` on clean detached worktrees of `v0.2.4` and
  `92ad58f` gives the same 19 paths: `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`,
  `Cargo.toml.orig`, `LICENSE-APACHE`, `LICENSE-MIT`, `README.md`, and the 12 files under
  `src/` (`bin/irr.rs`, `color.rs`, `demosaic.rs`, `develop.rs`, `ifd.rs`, `lib.rs`,
  `ljpeg.rs`, `opcode.rs`, `plane.rs`, `tone.rs`, `tone_table.rs`, `warp.rs`). There is
  no `scripts/`, `tests/`, SDK or corpus file.
- **Release notes against the reconciled list.** Each class above has a line in the
  notes:
  - the output changes and the dual-illuminant path change;
  - every newly refused input, and the optional too-new opcode skip;
  - each already-refused-different-error class;
  - `DEVELOP_VERSION` 7 to 8;
  - the composer-visible path sets and `malformed_tags`.

  The `irr` print lines are a feature-gated binary and are left out.

## Release cut is two-phase

A release session **cannot finish in one pass** — the irreversible tag/publish is
gated by a human/coordinator. Structure it so the reversible work lands first:
- **Phase 1 — reversible prep (CI-gated PR):** CHANGELOG entry, version bump
  (`just next-version`), stage-backlog tick. Fully revertable — land it as a
  normal PR.
- **Phase 2 — irreversible cut (human/coordinator-gated):** create the tag and
  publish to each channel, only after Phase 1 merges. The spec ships
  **"prep-complete, cut-deferred."** Before tagging, confirm the checked-out tree
  is the merged Phase-1 commit: the version in the manifest is the new one. A tag
  cut on the commit before the merge points at the old version (irradiance 0.2.4's
  first attempt).

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).** Filled
2026-11-22 by the Phase-1 build (HANDOFF-184) on candidate `92ad58f`
(`92ad58ff7ffba9db2ab25d2854db467be3ea060f`, the version bump). The commits after it
change only `README.md` wording and project records, and `src/` is byte-identical
throughout. The dry run and `just test` were repeated on the PR head; the handback has
those results.
- `[now]`: verified on the candidate in the build session.
- `[cut]`: verified by the user at Phase 2.
- `N/A`: does not apply.

- [ ] **1. Version / tag integrity & build provenance** — `[now]` done, the rest `[cut]`.
  - `[now]` ✔:
    - `Cargo.toml` says `version = "0.2.5"`. `cargo publish --dry-run` ran in a detached
      scratch worktree of `92ad58f`, on a clean tree with no `--allow-dirty`. It reports
      `Packaged 19 files, 1.8MiB (394.8KiB compressed)` and `Uploading irradiance v0.2.5`,
      then "aborting upload due to dry run".
    - `git tag -l 'v0.2.5'` is empty. The timing examples stamp the candidate as
      `v0.2.4-31-g92ad58f`.
    - The packaged `.cargo_vcs_info.json` carries `git.sha1`
      `92ad58ff7ffba9db2ab25d2854db467be3ea060f`, the candidate.
    - `git diff v0.2.4 -- Cargo.toml` before the bump was empty, and the bump changes
      only the version line. So `license`, `description`, `keywords`, `categories`,
      `repository`, `include` and `rust-version = "1.90"` are unchanged; the packaged
      manifest shows `rust-version = "1.90"`.
  - `[cut]`:
    - `git tag -a v0.2.5 -m "irradiance 0.2.5"` on the merged prep commit, after
      confirming its `Cargo.toml` shows 0.2.5.
    - `git tag --points-at HEAD` prints exactly `v0.2.5`.
    - The published `.crate`'s `.cargo_vcs_info.json` sha1 equals the tagged commit:
      `curl -sL https://static.crates.io/crates/irradiance/irradiance-0.2.5.crate | tar -xzO irradiance-0.2.5/.cargo_vcs_info.json`
      against `git rev-parse v0.2.5^{commit}`.
- [x] **2. Artifact trust on a clean host** — `N/A`: a source crate, no signed binary.
- [ ] **3. Distribution-channel trust** — `[now]` proxy done, the rest `[cut]`.
  - `[now]` ✔:
    - `cargo publish --dry-run` packaged and verified the crate in isolation. Its only
      warnings are the expected "ignoring example/test … not included in the published
      package" lines (50), as in 0.2.4.
    - The packaged `Cargo.toml` has no self dev-dependency; its only dev-dependencies
      are `ssimulacra2` and `toml`.
    - `just package-targets-check` passed: no executable in the default build or in the
      package (`irradiance-0.2.5`), `irr` builds under `--features irr`, and its red case
      is caught. `just deny` and `just deny-fuzz` both report `licenses ok`.
    - The package list is the same 19 files as 0.2.4's: `cargo package --list` on a
      clean worktree of `v0.2.4` and of the candidate, diffed, is identical. No
      `scripts/`, `tests/`, SDK or corpus bytes.
  - `[cut]`: on a clean host, `cargo add irradiance@=0.2.5 && cargo build`. crates.io
    shows 0.2.5, and docs.rs is green.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` done on the candidate, `[cut]` on the
  published crate.
  - `[now]` ✔:
    - `just test` (on `2724fff`, `src/`-identical) ran with `IRRADIANCE_CORPUS_DIR`,
      `IRRADIANCE_SDK_COLOUR_DIR` and `IRRADIANCE_SDK_STAGE2_DIR` set. It printed
      "corpus: 12/12 present, exiftool and dnglab on PATH — no tier-B test will skip",
      and "sdk-stage2: present" and "sdk-colour: present", each followed by "will run".
      The 50 test results, doctests included, total 845 passed, 0 failed and 1 ignored,
      exit 0, in 233 s. Every `dnglab` checksum pin and every develop MD5 pin held.
    - `just doc-check`: 2 feature surfaces, 0 rustdoc warnings.
  - `[cut]`: in a clean-host probe crate on `irradiance = "=0.2.5"`:
    - develop the Pixel 7 Pro and the Canon EOS 5D Mark III with `develop_color_into`;
      each returns `Ok` with the expected dimensions;
    - on the Canon's parsed `Sensor`, set `sample_format` to `Some(vec![3])`.
      `develop_color_into` then returns `Error::UnsupportedTag { tag: 339 }` and writes
      nothing. `Sensor` is `#[non_exhaustive]`, so change a parsed one rather than
      build one with a struct literal;
    - `irradiance::DEVELOP_VERSION == "8"`.
- [ ] **6. Rollback / uninstall** — `[cut]`.
  - **Withdraw:** `cargo yank --version 0.2.5`, and dependents on `"0.2"` fall back to
    0.2.4. `cargo yank --undo --version 0.2.5` reverses it.
  - **Git rollback:** revert the Phase-1 PR.

**Gates on the candidate, all green** (2026-11-22, `IRRADIANCE_CORPUS_DIR` set):
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

- Before tagging, the user confirms the checked-out tree is the merged Phase-1 commit
  (`Cargo.toml` shows 0.2.5). 0.2.4's first tag landed on the commit before the merge.
- The SDK lanes are local only (DEC-069 §6): the SDK, its outputs and the corpus are never
  committed or packaged. Run both scripts into a scratchpad outside the repository.
- A defect found during prep is reported in the handback, not fixed here: the stage is
  release-only.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).
- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, with a clean tree; confirm `Cargo.toml` shows
     `version = "0.2.5"`.
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.5 -m "irradiance 0.2.5"`, then `git push origin v0.2.5`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5, reported back for the Cut Record.
  6. Optional: a GitHub release on `v0.2.5` with the `## Release Notes` text, down to
     the internal record.

## Cut Record

v0.2.4 → **v0.2.5 published 2026-11-23** (crates.io, by the user, 02:47 UTC; not yanked;
checksum `1cbe5f6439987371…`). Tag `v0.2.5` (lightweight) points at `c558a4b`, the PR 316
merge, whose `Cargo.toml` says 0.2.5; the published `.crate`'s `.cargo_vcs_info.json`
`git.sha1` is `c558a4b8f42b7bd99361f89f331b554042976dec`, the tagged commit. docs.rs reports
`doc_status: true`. No GitHub release was created at the cut. Clean-host check, run by the
orchestrator in a scratch crate on `irradiance = "=0.2.5"` from crates.io (Cargo.lock
`registry+https://github.com/rust-lang/crates.io-index`): it built clean;
`DEVELOP_VERSION` is `8`; `develop_color_into` on the Pixel 7 Pro (3866×2908) and
`develop_into` on the Pentax K-3 III Monochrome (6192×4128, `malformed_tags [50713]`, still
exempt) return `Ok`, with output hashes identical to the same program built against
`main`'s source. Calibrated colour and the unhonoured and malformed tag refusals are now on
crates.io.

---

## Reflection

1. **What would I do differently next time?**
   — Check that every merged change carries its CHANGELOG entry when it merges, not at
   release. PR 301 had no spec and no entry, and only the release design caught it. Also
   tag annotated, as 0.2.4 was; `v0.2.5` is a lightweight tag (cosmetic, the commit is
   right).

2. **Does any template, constraint, or decision need updating?**
   — No change made. The changed-input reconciliation (signal
   `release-notes-diff-every-changed-refusal-and-what-it-preempts`, N=2 before this) earned
   its keep again: the build corrected one wrong design example and added six classes the
   design missed, all described by their own specs or PR. That is a third same-outcome
   instance, for the stage close to disposition.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. A GitHub release for `v0.2.5` (this spec's Release Notes) is the user's call.

4. **Where was the worst defect caught?** — design

5. **What can a user do now that they couldn't before?** — A crates.io user on
   `irradiance = "0.2"` now gets colour within 6 codes of Adobe's DNG SDK on every held
   colour file (the Pixel 7 Pro was 1,160 codes off in 0.2.4), and typed refusals instead
   of silently wrong pixels for unhonoured, malformed or out-of-version tags, staggered or
   non-RGB CFAs, non-finite warps and too-new mandatory opcodes.
