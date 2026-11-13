---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-061
  type: decision
  confidence: 0.85
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

created_at: 2026-11-13
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/develop.rs
  - src/lib.rs
  - fuzz/fuzz_targets/develop.rs
  - examples/fuzz-seeds.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-066-cfa-as-mono-option-develop-a-cfa-declared-plane-on-the-mono.md

tags:
  - develop
  - options
  - levels
  - cfa
  - public-api
  - stage-019
---

# DEC-061: the mono develop honours BlackLevelRepeatDim under an opt-in per-position black-level option, and white stays per-sample

## Decision

### 1. No routing change: a CFA plane already develops on the mono path

`develop_into`, `develop_with_into` and `develop_with_scratch_into` never read
`PhotometricInterpretation` or `Sensor::color_filter`, and none of them refuses a CFA
plane. The SPEC-063 probe showed this, and SPEC-063's AC2 pins it on the Canon 5D3 and
Nikon D300S. **That stays as it is.** Choosing demosaic or mono is the caller's decision,
as it has always been. "Treat a CFA sensor as monochrome" therefore needs no switch. What
the request actually lacks is the levels. The mono path uses `sensor.black_level`, the
first `BlackLevel` value, for every photosite.

### 2. Shape: one options field, default unchanged

```rust
#[non_exhaustive]
pub enum BlackLevels {
    #[default]
    FirstValue,   // today: the first BlackLevel value for every photosite
    PerPosition,  // BlackLevel indexed by BlackLevelRepeatDim, phased from ActiveArea
}

pub struct DevelopOptions { /* … */ pub black_levels: BlackLevels }
```

- **The default is `FirstValue`.** It is exactly today's output for every (input,
  options) pair. Under DEC-058 §5, `DEVELOP_VERSION` stays `"4"`.
- **The name says what changes, not "CFA".** The option is not CFA-specific. A
  `LinearRaw` plane may carry a black pattern too, and DNG applies `BlackLevel` the same
  way to both. On every held `LinearRaw` file the two modes give byte-identical output,
  because each declares a 1×1 pattern (§Context).
- **An enum, not a `bool`.** `BlackLevelDeltaH` and `BlackLevelDeltaV` (§5) would be a
  third mode later. That would be an additive variant, not a second flag.

### 3. `PerPosition` semantics (DNG 1.7 `BlackLevel`, `BlackLevelRepeatDim`)

Let `[R, C]` be `sensor.black_level_repeat_dim`. If it is absent, or recorded as
malformed (DEC-012), the DNG default `[1, 1]` applies. Let `B = sensor.black_levels`.

- **`B` empty** (`BlackLevel` absent or unreadable): every photosite's black is
  `sensor.black_level.unwrap_or(0)`. That is `FirstValue`'s value, so the option never
  refuses a file whose `BlackLevel` is absent or unreadable.
- **Otherwise `B.len()` must equal exactly `R × C`.** `SamplesPerPixel` is 1 on every
  plane the mono path accepts. The product uses checked arithmetic, and `R` and `C` must
  each be ≥ 1. Anything else is the new
  `Error::InvalidBlackLevelPattern { repeat_rows, repeat_cols, count }` with
  `ErrorKind::Corrupt`.
- **The black of the photosite at raw `(x, y)`** is `B[((y − top) mod R) × C + ((x −
  left) mod C)]`. Here `(left, top)` is the `ActiveArea` origin, or `(0, 0)` when the tag
  is absent. DNG fixes the pattern's origin at the `ActiveArea` top-left corner. That is
  the same origin the colour path uses for its 2×2 slots (DEC-040), and it is **not** the
  `DefaultCrop` origin or the plane origin.
- **White is per sample, not per position.** DNG's `WhiteLevel` has `SamplesPerPixel`
  values, one here, so a one-sample plane has no per-position white level. Both modes use
  `sensor.white_level`, resolved as today. The request asked for per-position white too.
  DNG does not define it, so it is not built.
- **Every position's range must be non-empty.** If any `B[i] ≥ white`, the result is
  `Error::InvalidLevels { black_level: B[i], white_level }`, using the first such `i` in
  index order. That is today's variant, today's kind, and today's step.
- **Validation happens only when the values are used.** Under `FirstValue` nothing new
  is checked, so no existing result becomes a refusal. This is consistent with DEC-058
  §3: an option never stops a tag from being validated, and here a tag becomes validated
  because the option starts using it.

### 4. Where it sits in the develop

- **The check happens at the same point as `resolve_levels` today.** That is after
  geometry and before the `dst` length check, so error order among existing variants is
  unchanged. `InvalidBlackLevelPattern` is reported where `InvalidLevels` would be.
- **Both normalize sites take each photosite's own black.** They are
  `crop_orient_normalize_into` (no warp) and `normalize_active_area_into` (warp). In both,
  the black is taken **before** the warp, on raw samples. Black subtraction is per
  photosite, so it cannot commute with the warp's interpolation.
- **`FixBadPixelsConstant` still runs on raw values, before levels.** The bad-pixel test
  compares the raw sample with the opcode's constant, so levels play no part in it.
- **No allocation.** The per-position blacks are read from `sensor.black_levels` in
  place. DEC-060's promise that `develop_with_scratch_into` allocates nothing on any path
  covers the new mode, and so does the single allocation in `develop_with_into`.
  `scratch_len` does not depend on this option.
- **A 1×1 pattern takes today's scalar loop.** A `PerPosition` develop whose `B` is
  empty or has one value is byte-identical to `FirstValue` by construction. It runs no
  per-pixel modulo.
- `output_dimensions_with` is unaffected, because levels do not change dimensions. It
  neither reads nor refuses this option.

### 5. Out of scope, recorded

- **`BlackLevelDeltaH` and `BlackLevelDeltaV`** (per-column and per-row black offsets) are
  read by no path today, and no held file carries them. They could become a future
  additive mode.
- **A RATIONAL non-integral `BlackLevel`** (the Pixel 7 Pro writes `1024.109375 …`) is
  recorded as malformed and resolves to 0 on every path today. That is a pre-existing
  gap, not this option's, and it is filed as a follow-up. No Pixel file develops today
  anyway, because of an unrecognised mandatory opcode 9.
- **The colour path** indexes `black_levels` by a fixed 2×2 slot and ignores
  `BlackLevelRepeatDim`. That is correct for every held `[2, 2]` and `[1, 1]` pattern,
  and wrong for a `[1, 2]` or `[2, 1]` one. It is not changed here (DEC-058: the options
  are mono only), and it is filed as a follow-up.

## Context

monorail-000 / onyx-000 request 6 asked for an option that treats a converted
(CFA-removed) camera's CFA-declared plane as monochrome and keeps per-CFA-position black
and white levels. It set three acceptance tests:
- a uniform synthetic CFA plane gives the mono output;
- per-position levels are honoured;
- a red-proof with the positions swapped.

The SPEC-066 design probe (main at a36cdc1) read every held DNG with exiftool and with
irradiance's own `Container::sensor()`:

| File | Photometric | `BlackLevel` | `BlackLevelRepeatDim` | `WhiteLevel` | `ActiveArea` origin |
|---|---|---|---|---|---|
| Canon 5D3 | CFA RGGB | 2047 2047 2047 **2048** | 2 2 | 15000 | (122, 80) |
| Nikon D300S | CFA RGGB | 0 | 1 1 | 3808 | (0, 0) |
| Pixel 3a | CFA BGGR | 1023 1023 1022 1022 | 2 2 | 16368 | (0, 0) |
| Pixel 4a | CFA RGGB | 1023 1023 1022 1022 | 2 2 | 16368 | (0, 0) |
| Pixel 7 Pro | CFA BGGR | 1024.109375 … (RATIONAL; irradiance: malformed, empty) | 2 2 | 16368 | (0, 0) |
| M Monochrom | LinearRaw | 220 | 1 1 | 16383 | none |
| M Typ 246 | LinearRaw | 0 | 1 1 | 3750 | none |
| Q2M (×3) | LinearRaw | 512 | 1 1 | 16383 | (0, 0) |
| K-3 III Mono | LinearRaw | 64 | malformed (count 1; irradiance: `None`, in `malformed_tags`) | 16378 | (26, 34) |

- **Every held `LinearRaw` file has a 1×1 pattern.** Every mono develop the requester
  makes today is therefore unchanged under either mode.
- **The Canon is the one held file whose develop changes.** Over its 5796×3870 active
  area (22,430,520 photosites, no opcodes, Orientation 1), `PerPosition` changes
  **5,602,332** samples by at most **6** code values. They are all at position 3, where
  the black is 2048, not 2047. That is a quarter of the photosites, less those clamped at
  either end.
- **The declared pattern is honoured, not second-guessed.** The Canon's left masked
  (optical-black) columns, phased from the `ActiveArea` origin, average 2047.94, 2048.06,
  2048.11 and **2047.20** across the four positions. Position 3 measures *lowest*, yet it
  is declared the highest. The difference is about one code value either way. irradiance
  applies the tag as DNG defines it, so the Canon is a fixture for "the pattern is
  applied", not evidence that per-position black is more accurate.
- **No held file carries `BlackLevelDeltaH` or `BlackLevelDeltaV`, and every
  `WhiteLevel` has one value.**

## Alternatives Considered

- **Make `PerPosition` the default and bump `DEVELOP_VERSION` to 5.** Rejected. It
  changes the Canon's pinned output and invalidates every caller's cache, for a change no
  held mono file needs. The requester opts in. It stays a one-line change later if the
  default should follow DNG strictly.
- **A `cfa_as_mono: bool` that also refuses CFA planes when it is off.** Rejected. It
  turns today's Canon and Nikon results into refusals, which DEC-058 §4 and §5 rule out,
  and the routing it names has never existed.
- **Per-position white levels.** Rejected, because DNG defines `WhiteLevel` per sample,
  not per position (§3).
- **Fall back to `FirstValue` when the pattern count is wrong.** Rejected. The caller
  asked for per-position levels, so applying one value silently would answer a different
  question. A typed `Corrupt` error lets the caller choose `FirstValue` knowingly.
- **Reuse the colour path's `level_at`.** Rejected. Its fixed 2×2 slot and its fallback
  to the last value ignore `BlackLevelRepeatDim` (§5). The mono path follows the tag
  exactly, and aligning the colour path is a separate follow-up.
- **Phase the pattern from the `DefaultCrop` origin or the plane origin.** Rejected. DNG
  fixes the origin at the `ActiveArea`.

## Consequences

- New public items:
  - `develop::BlackLevels` (`#[non_exhaustive]`, `Default` = `FirstValue`);
  - the `DevelopOptions::black_levels` field;
  - `Error::InvalidBlackLevelPattern` (`Corrupt`).

  `DevelopOptions` is `#[non_exhaustive]` and unreleased, so the field is not a break.
- `DEVELOP_VERSION` stays `"4"`.
- A converted camera's CFA-declared plane develops on the mono path with its declared
  per-position black and its per-sample white. Every allocation guarantee from DEC-060
  still holds.
