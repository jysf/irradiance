---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-070
  type: decision
  confidence: 0.95
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-20
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/develop.rs
  - src/color.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-078-black-normalisation-decide-white-minus-max-black-the-dng-sdk.md
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-080-colour-levels-refusal-fu-1-the-four-colour-entries-refuse-wh.md

tags:
  - develop
  - colour
  - black-level
  - stage-025
---

# DEC-070: black normalisation scales by white minus the maximum black, as the DNG specification defines

## Decision

### 1. The rule

Wherever a develop path subtracts a black that can differ by photosite position, a sample
`v` at a position whose black is `b` normalises as:

`v' = ⌊ clamp(v − b, 0, W − B) · 65535 / (W − B) + ½ ⌋`, in integer arithmetic.

Here `W` is the white the path already uses, and `B` is the **maximum** of the black values
that path applies over the plane. The black values are taken after DEC-068's rounding of a
fractional `BlackLevel`.

The rounding is DEC-018's: add half the denominator, then divide. The clamp below 0 is
DEC-018's clamp. The clamp at `W − B` is the specification's "rescaled values above 1.0
should be clipped". A position whose black is below `B` reaches full scale slightly before
raw white.

### 2. Where it applies

- **The colour path:** all four entry points, `u16` and `f32`, single-thread and threaded.
  `B` is the maximum over the slot or pattern blacks.
- **The mono path under `BlackLevels::PerPosition`** (DEC-061). `B` is the maximum over the
  `BlackLevelRepeatDim` pattern.
- **The mono default, `BlackLevels::FirstValue`:** unchanged. It applies one black
  everywhere, so `B = b` and the formula is today's.

Every file whose black is the same at every position, on any path, develops
byte-identically, because `B = b` there.

### 3. Levels validation

A plane is malformed when `W ≤ B`. This is the existing `Error::InvalidLevels` check, now
made against the maximum black rather than the first or per-position one.

**Amended at SPEC-078's ship (2026-11-20), from its verify ruling.** The refusal applies
where a levels check already exists, which is mono `PerPosition`. The colour path has never
refused levels: `normalize_sample` was total by design, and the colour entries' `# Errors`
never listed `InvalidLevels`. So on colour, `W ≤ B` gives a range of 0 and every sample
develops to 0, never a panic. One consequence: a malformed colour file whose largest black
reaches white, but whose other blacks are below it, used to develop those positions and
now develops entirely to 0. Whether colour should refuse such levels, as mono does, is a
carried follow-up (a behaviour change needing its own spec).

**Amended again at SPEC-080's design (2026-11-20), superseding the paragraph above once
SPEC-080 ships.** The colour path refuses too. On all four colour entries,
`resolve_colour_levels` returns `Error::InvalidLevels { black_level: B, white_level: W }`
when `W ≤ B`, where `B` is the maximum black the path applies (DEC-070 §1) and `W` is the
colour path's own resolved white. That covers the slot path, the general pattern path and
the scalar `BlackLevel` with no pattern. It is checked after `InvalidBlackLevelPattern` and
before any write. The colour path now refuses what mono `PerPosition` refuses, with the same error. (Mono's default, `FirstValue`, checks its first black, not the maximum, so it is not identical to colour there; shipped in SPEC-080, 2026-11-20.)
No file that develops today changes output, so `DEVELOP_VERSION` stays 7. A file that
developed to all zeros is now refused; the release notes name it.

### 4. `DEVELOP_VERSION` goes from 6 to 7

Output changes for any file with position-varying blacks, on the colour path and on mono
`PerPosition`. In the held corpus that is the Canon 5D Mark III (2047/2048) and the Pixel 3a
and 4a (1023/1022).

### 5. Not changed here

- irradiance does not read `BlackLevelDeltaH` / `BlackLevelDeltaV`, which the specification
  adds to the black before taking the maximum. No held file carries them. How such a file
  is treated today is carried as a follow-up to check.
- DEC-068's rounding of a fractional black stays. It is why the Pixel 7 Pro still differs
  from the SDK by up to 2 codes.

## Context

**The specification.** DNG Specification 1.7.1.0, Chapter 5, "Mapping Raw Values to Linear
Reference Values", lists the steps Linearization, Black Subtraction, Rescaling and Clipping:
- Black Subtraction: "The black level for each pixel is then computed and subtracted."
- Rescaling: the scale factor is the inverse of the difference between `WhiteLevel` and
  "the maximum computed black level for the sample plane".

irradiance divided by `W − b` per position (DEC-018's affine map, carried into DEC-061 and
DEC-068). That doesn't follow the specification when blacks differ by position.

**The reference implementation agrees.** Adobe DNG SDK 1.7.1 `dng_linearization_info.cpp`
computes `maxBlack = info.MaxBlackLevel(plane)`, `minRange = fWhiteLevel[plane] − maxBlack`
and `scale = 1 / minRange`. This was first seen in the STAGE-016 probe, DEC-069 §Context.

**The STAGE-025 probe (2026-11-20, on main at e047ea5).** Both rules were computed from
each file's raw plane over the `ActiveArea` and compared with `dng_validate -2` stage 2.
That is stage 2 before `OpcodeList2` for the Pixels (SPEC-060's `-pre.tif`), and the plain
stage 2 for the Canon, which has no opcode lists. "Δ" is SDK minus ours.

| File | Blacks | Today: max \|Δ\| / exact | DEC-070: max \|Δ\| / exact |
|---|---|---|---|
| Canon 5D Mark III | 2047 ×3, 2048 | 6 / 69.36% | 1 / 99.57% (Δ ∈ {0, +1}) |
| Pixel 4a | 1023, 1023, 1022, 1022 | 5 / 93.17% | 1 / 99.79% (Δ ∈ {0, +1}) |
| Pixel 3a | 1023, 1023, 1022, 1022 | 5 / 89.38% | 1 / 99.85% (Δ ∈ {0, +1}) |
| Pixel 7 Pro | 1024 ×4 (fractional in the file) | 2 / 28.35% | 2 / 28.35% (unchanged; DEC-068) |

End to end, after each file's `GainMap`s (`apply_gain_maps` on each rule's stage 2, against
SPEC-060's `-post.tif`):

| File | Today: max \|Δ\| / exact | DEC-070: max \|Δ\| / exact |
|---|---|---|
| Pixel 4a | 7 / 94.39% | 4 / 99.77% |
| Pixel 3a | 6 / 89.42% | 4 / 99.82% |
| Pixel 7 Pro | 4 / 28.34% | 4 / 28.34% |

The remaining one-code differences are all in one direction, the SDK one code above us.
They come from the SDK's `f32` scale; DEC-069's `f32` stepping is the same kind of
artefact.

**Highlights.** Under DEC-070, a position with the lower black reaches full scale one raw
code below `WhiteLevel`. On the Pixel 4a, 153,237 samples reach 65535, against 148,581
today. On the 3a it is 4,010 against 3,746. On the Canon it is 2,338 in both, because its
lower-black positions never come within a code of white. That is the specification's
clipping step, and the SDK does the same.

## Provenance

- **Class 1:** the DNG Specification 1.7.1.0, Chapter 5, quoted above.
- **Confirmed against Adobe DNG SDK 1.7.1:** `dng_linearization_info.cpp`, read and
  recorded with no code copied (class 4, as in DEC-069), and by the measurements above.

## Alternatives Considered

- **Keep `W − b` per position.** Rejected. It contradicts the specification's Rescaling
  step. It also gives positions with different blacks different gains, which is a small
  fixed-pattern gain error of up to 1/12,952 on the Canon.
- **Use the minimum black, or the mean.** Rejected. Neither is the specification's rule,
  and neither is the SDK's.
- **Apply DEC-070 to `FirstValue` as well,** taking the maximum over the whole pattern
  while subtracting the first value. Rejected. `FirstValue` deliberately treats the plane
  as one black (DEC-061), and changing it would alter every mono default develop.
- **Emulate the SDK's `f32` scale bit for bit.** Rejected. As in DEC-069, that copies an
  implementation artefact, and the cost is at most one code on 0.15–0.43% of samples.

## Consequences

- The colour develop and mono `PerPosition` follow the DNG specification on every file
  whose blacks differ by position. They match `dng_validate` within one code on the
  Canon, the Pixel 3a and the Pixel 4a.
- `DEVELOP_VERSION` is `"7"`. Pins change only for the Canon, the Pixel 3a and the Pixel 4a,
  on the colour and `PerPosition` paths. Every file with uniform blacks, and every mono
  default develop, stays byte-identical.
- SPEC-060's SDK bounds tighten. Their new figures are pre-registered in SPEC-078.
- DEC-018's affine map, DEC-061's `PerPosition` and DEC-068 §2 keep their structure. Only
  the denominator changes, to `W − B`.
