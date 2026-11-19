---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-068
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

created_at: 2026-11-19
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/ifd.rs
  - src/color.rs
  - src/develop.rs
  - src/lib.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-075-black-level-fixes-round-fractional-blacklevel-colour-blackle.md

tags:
  - develop
  - colour
  - metadata
  - black-level
  - stage-023
---

# DEC-068: a fractional BlackLevel rounds to nearest, the colour path indexes BlackLevelRepeatDim, and MaskedAreas passes through

## Decision

### 1. A non-integral RATIONAL `BlackLevel` rounds to nearest

DNG allows `BlackLevel` as RATIONAL, and its values need not be whole numbers. Today a
RATIONAL that divides exactly (the Canon's `524032/256`) reads correctly through
`Container::uints`. A non-integral one (the Pixel 7 Pro's `1024.109375`, `1024.3125`,
`1024.308594`, `1024.132812`) fails that read. The tag is then costed as malformed
(DEC-012), and every path takes black as 0.

From this decision on:
- **`Sensor::black_level` and `Sensor::black_levels`** hold each RATIONAL value rounded
  to the nearest integer, with an exact half rounding up: `(2·num + den) / (2·den)` in
  `u64`. On the Pixel 7 Pro all four values become 1024. A RATIONAL that divides exactly
  gives the same value as today.
- **A zero denominator** is still malformed. The tag is costed as today, and the
  integer fields keep their absent values.
- **The new field `Sensor::black_levels_exact: Vec<Rational>`** holds every value as
  written, in file order, when `BlackLevel` is RATIONAL. It is empty when the tag is
  SHORT or LONG, because `black_levels` is then already exact, and empty when the tag is
  absent or malformed. `Sensor` is `#[non_exhaustive]` (DEC-052), so the field is
  additive.
- **Develop uses the rounded integers.** Neither normalizer changes.

The user chose rounding over a fractional black carried through develop (2026-11-19).
The two differ by at most one or two output units out of 65,535, near black, on a
fractional-black file, and are byte-identical on every integral one. Rounding leaves the
tuned mono and colour kernels untouched. The exact values stay on `Sensor` for a caller
that needs them, for example to average many frames, where a constant 0.1 to 0.3 code
bias does not average out. A fractional-black develop can still be added later without
breaking anything.

### 2. The colour path indexes `BlackLevel` by `BlackLevelRepeatDim`

The colour normalizer applies DEC-061 §3's rule, as the mono `PerPosition` option does:
- `[R, C]` is `black_level_repeat_dim`, or the DNG default `[1, 1]` when it is absent or
  malformed. `B` is `black_levels`.
- If `B` is empty, every photosite takes `black_level.unwrap_or(0)`, as today.
- Otherwise `B.len()` must equal `R × C` (checked arithmetic, `R, C ≥ 1`). Anything else
  is `Error::InvalidBlackLevelPattern { repeat_rows, repeat_cols, count }`, the error
  DEC-061 introduced. The colour develop entry points return it before writing to `dst`.
- The black at raw `(x, y)` is `B[((y − top) mod R) × C + ((x − left) mod C)]`, with
  `(left, top)` the `ActiveArea` origin (DEC-040). The colour path already phases from
  that origin.
- **White is per sample** (DEC-061 §3): the first `white_levels` value, else
  `white_level`, else `2^BitsPerSample − 1`. Today white goes through the same 2×2 slot
  lookup. That gives the same value whenever `WhiteLevel` has one value, which is true
  of every held file.

When `R` and `C` are each 1 or 2, the pattern still fits the existing four-slot table,
which is filled from the rule above, so the hot loop is unchanged. A larger pattern
(`[4, 4]` and so on) takes a per-photosite lookup. That path is new, but no held file
reaches it.

The mono path's `FirstValue` default is unchanged: DEC-061 keeps it as the default by
design.

### 3. `MaskedAreas` passes through, verbatim

`Sensor::masked_areas: Vec<ActiveArea>` reads DNG `MaskedAreas` (tag 50830, SHORT or
LONG, count 4·N, each rectangle top, left, bottom, right in raw-plane coordinates) from
the sensor IFD, in file order. It reuses the existing `ActiveArea` struct, because the
field order and meaning are the same. This is monorail-000 / onyx request 5
(`feedback/2026-11-11-monorail-000-requests.md` item 8).

- **Interpretation class (DEC-012).** A malformed tag leaves the field empty and pushes
  50830 to `malformed_tags`; the sensor read still succeeds. Malformed means:
  - a count that is not a multiple of 4;
  - more than 16 rectangles;
  - any rectangle with `top ≥ bottom` or `left ≥ right`;
  - any rectangle that reaches past the plane's width or height;
  - a field type that is not SHORT or LONG.
- **No policing beyond that.** irradiance does not check that a rectangle lies outside
  `ActiveArea`, does not name strips, and does not use the field in develop. onyx does
  the optical-black reductions itself (onyx DEC-010).
- The cap of 16 is far above any real layout (DNG cameras use one to four strips). It
  bounds what a hostile file can make the read allocate.

### 4. `DEVELOP_VERSION` goes from 4 to 5

DEC-058 §5 says to bump the version when the output changes for any existing options
value on the same input bytes. Sections 1 and 2 change output for a file with a
fractional black, and for a colour file with a `[1, 2]`, `[2, 1]` or larger pattern. No
held file is one of those, so every pinned hash and oracle stays unchanged. The rule is
about inputs, though, not about the corpus, so the version goes to `"5"`. A caller that
caches developed pixels re-develops once.

## Context

Both black-level gaps were found in the SPEC-066 design (DEC-061 §5) and carried as an
`[S]` follow-up from STAGE-019. The STAGE-023 design probe (main at 4c5dd2c, 2026-11-19)
read every held DNG with exiftool:

| File | BlackLevel | RepeatDim | Photometric | Develops today |
|---|---|---|---|---|
| Canon 5D3 | 2047 2047 2047 2048 (RATIONAL, integral) | 2 2 | CFA | yes, colour |
| Nikon D300S | 0 | 1 1 | CFA | yes, colour |
| Pixel 3a, 4a | 1023 1023 1022 1022 | 2 2 | CFA | no (opcode 9) |
| Pixel 7 Pro | 1024.109375 1024.3125 1024.308594 1024.132812 | 2 2 | CFA | no (opcode 9) |
| Leica M, Typ 246, three Q2M, K-3 III | one value | 1 1 (or malformed) | LinearRaw | yes, mono |

Every `WhiteLevel` has one value and `SamplesPerPixel` is 1. So:
- no developable held file reaches either bug, and the fix changes no held output;
- the Pixel 7 Pro's `sensor().black_level` is `None` today, and once STAGE-016 makes the
  Pixel files develop, it would develop with black 0, about 1,024 codes too dark a
  floor. This decision is a prerequisite for STAGE-016's real-file spec.

No held DNG carries `MaskedAreas`, so its fixtures, like the pattern fixtures, come from
the SPEC-057 test-only writer's `extra_tags`.

## Alternatives Considered

- **A fractional black carried through both normalizers in fixed point.** It would match
  DNG and Adobe's SDK exactly, and is byte-identical on integral files. Rejected for now
  by the user: it edits the mono and colour kernels that SPEC-069 and SPEC-070 tuned, for
  at most one or two output units out of 65,535. It can still be added later without
  breaking anything, because the exact values are kept.
- **Truncate instead of round.** Rejected: it biases every fractional black low, by up
  to almost one code.
- **Keep the colour path lenient on a count mismatch** (fall back to the last value, as
  `level_at` does today). Rejected: it applies an answer DNG does not define, and DEC-061
  already chose a typed `Corrupt` error for the same case on the mono path. No held file
  has a mismatch.
- **Bring `BlackLevels::FirstValue` to the colour path too.** Not needed. The colour path
  has always applied per-position black, and DEC-058's options are mono only.
- **A fixed-capacity array for `masked_areas`.** Rejected: `Sensor` already holds
  `Vec`s (`black_levels`, `white_levels`, opcode lists), so a `Vec` with a 16-rectangle
  cap is consistent and bounded.
- **A typed error for a malformed `MaskedAreas`** (the feedback note's suggestion).
  Rejected: under DEC-012 an interpretation tag costs only itself and never the
  container. A develop that never reads the field should not be refused because of it.
- **Leave `DEVELOP_VERSION` at 4, because no held output changes.** Rejected: DEC-058
  §5's rule is about input bytes, not the corpus.

## Consequences

- `Sensor` gains `black_levels_exact` and `masked_areas`. Both are additive, so this is
  0.2.x.
- `Sensor` stops reporting 50714 in `malformed_tags` for a non-integral RATIONAL
  `BlackLevel`.
- The colour develop entry points gain one refusal, `InvalidBlackLevelPattern`, for a
  black pattern whose count disagrees with its repeat dimensions.
- `DEVELOP_VERSION` is `"5"`.
- STAGE-016's real-file GainMap spec can develop the Pixel 7 Pro with a correct black.
