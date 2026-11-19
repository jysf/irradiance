---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-069
  type: decision
  confidence: 0.9
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
  - src/opcode.rs
  - src/develop.rs
  - src/color.rs
  - src/lib.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-059-gainmap-parse-apply-on-mono-and-colour-paths-with-the-analyt.md
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-060-real-file-gainmap-evidence-pixel-files-develop-cross-checked.md

tags:
  - develop
  - colour
  - opcode
  - gainmap
  - stage-016
---

# DEC-069: OpcodeList2 GainMap applies at stage 2 on both develop paths, by the SDK-confirmed sampling rule

## Decision

### 1. Where it applies

`GainMap` (OpcodeID 9) is applied from `OpcodeList2` on **both** develop paths, at the
DNG stage-2 position:
- after level normalisation (black subtract and white scale to `[0, 65535]`, including
  DEC-061's per-position black and DEC-068's colour pattern);
- before `WarpRectilinear` (mono), before demosaic (colour), and before `DefaultCrop` and
  `Orientation`.

`OpcodeList1`'s `FixBadPixelsConstant` still runs first, on raw samples.

Stage-2 coordinates are relative to the `ActiveArea` (DEC-040). The stage-2 image is
`ActiveArea`-sized, `H` rows by `W` columns. For raw photosite `(x, y)`, `r = y − top` and
`c = x − left`.

### 2. The rule

For each `GainMap` in `OpcodeList2`, in list order:

- **Area.** `Top, Left, Bottom, Right` are in stage-2 coordinates.
  - If `Top ≥ Bottom` or `Left ≥ Right`, the area is the whole image (the SDK's "empty
    area" rule).
  - Otherwise the area is intersected with `[0, H) × [0, W)`. An area wholly outside the
    image applies to no sample. That is not an error.
- **Pitch phase.** A sample is touched only if `(r − Top) mod RowPitch = 0` and
  `(c − Left) mod ColPitch = 0`, with `Top` and `Left` taken before the intersection.
  On the Pixels this gives the four Bayer phases with pitch 2.
- **Plane.** It applies only when `Plane = 0` (every develop path here has one plane). It
  uses map plane `min(Plane, MapPlanes − 1)`. If `Plane > 0`, it applies to no sample.
- **Gain**, at **pixel centres** over the stage-2 bounds, in `f64` from the file's `f32`
  entries:
  - `fr = ((r + 0.5) / H − MapOriginV) / MapSpacingV`, and the same for `fc` with
    `(c + 0.5) / W`, `MapOriginH` and `MapSpacingH`.
  - Each index clamps: `f ≤ 0` takes index 0, and `f ≥ points − 1` takes the last index,
    each with fraction 0. Otherwise `i = ⌊f⌋` and `i + 1`, with fraction `f − i`.
  - `g` is bilinear: interpolate rows first, then columns.
- **Apply.** `v' = ⌊clamp(v / 65535 · g, 0, 1) · 65535 + 0.5⌋` on the normalised `u16`
  sample `v`.
  - The result is clipped at white, as the SDK's `Min(v·g, 1)` does.
  - Each `GainMap` rounds back to `u16` before the next one sees the sample.
- **Optional maps apply too.** A `GainMap` with `Flags` bit 0 set is applied like a
  mandatory one, as the SDK does. Optional only means a reader *may* skip it.

The required-processing check (DEC-049, DEC-053) treats **every** `GainMap` instance in
`OpcodeList2` as applied, on both paths. This differs from the first-instance rule for
lists 1 and 3, because a file carries one map per Bayer phase. A mandatory `GainMap` in
`OpcodeList1` or `OpcodeList3` stays `UnsupportedRequiredOpcode`, and `GainMap` becomes a
recognised ID there. Every other `OpcodeList2` opcode behaves as before.

### 3. Malformed parameters

These are typed errors, never a panic or a guess. Each is
`Error::MalformedOpcodeParams { id: 9, declared_size }`, returned before anything is
written:
- the parameter block is not exactly `76 + 4·MapPointsV·MapPointsH·MapPlanes` bytes, in
  checked arithmetic;
- `RowPitch` or `ColPitch` is 0;
- `MapPointsV`, `MapPointsH`, `Planes` or `MapPlanes` is 0;
- either spacing is not finite or is `≤ 0`;
- either origin is not finite;
- any gain entry is not finite.

The map is read **in place** from the opcode bytes, with no copy of the gain grid, so a
large grid costs only its own bytes and needs no separate cap. A list may hold at most
**16** `GainMap`s; more is `Error::TooManyGainMaps { count }`, also before any write. The
develop paths keep their maps in a fixed-capacity array, which keeps the mono scratch
develop at zero allocations (DEC-060).

### 4. One implementation, two callers

A single crate-internal type holds a list's parsed maps and computes the gain for a
stage-2 position and a sample. It is used:
- inside each path's per-sample normaliser (the colour fused kernel's `normalize`
  closure, and the mono gather and active-area loops);
- by a new public applier, `apply_gain_maps(stage2, width, height, opcode_list_2)`. That
  applier takes an `ActiveArea`-sized normalised `u16` image and returns how many
  `GainMap`s it applied.

The public applier exists for the real-file reference check (§6), for the analytic
oracle, and for callers that compose stages themselves (DEC-053). A file with no
`GainMap` takes today's normaliser unchanged, as a separate monomorphisation, so its
output and speed are unchanged.

### 5. `DEVELOP_VERSION` goes from 5 to 6

A file whose `OpcodeList2` holds only optional `GainMap`s developed before with them
skipped, and now develops with them applied. DEC-058 §5's rule is about inputs, so the
version goes to `"6"`. No held file changes, because the only held files with a
`GainMap` were refused before.

### 6. The reference is Adobe's `dng_validate`, run locally

The real-file reference is Adobe DNG SDK 1.7.1's `dng_validate`, built locally outside
the repository:
- The zip, `dng_sdk_1_7_1.zip`, comes from download.adobe.com, sha256
  `5366d9ab…e6e0c2`.
- It is built without the XMP toolkit, with JPEG XL stubbed out.
- `dng_validate -2` writes stage 2 **after** `OpcodeList2`.

The SDK is never vendored, never run in CI, and never a build dependency. Its outputs are
never committed. Tests that read them skip visibly unless `IRRADIANCE_SDK_STAGE2_DIR` is
set.

## Context

The STAGE-016 design probe ran on 2026-11-19, on main at e69417f.

**The files:**
- **Pixel 3a, 4a and 7 Pro** each carry four **mandatory** 30×40 `GainMap`s in
  `OpcodeList2`, one per Bayer phase. All have pitch 2, origin 0, spacing·(points−1) = 1
  (the map spans the image), and gains from 1.005 to 3.43. Their only other opcode is an
  **optional** `WarpRectilinear` in `OpcodeList3`.
- Today both paths refuse these files with `UnsupportedMandatoryOpcode { id: 9 }`. With
  `OpcodeList2` disabled in a scratch copy, all three develop `Ok` in colour (about
  0.23 s) and in mono. So `GainMap` is their only blocker.
- **The Samsung SM-G960F** (in `data.lfs`, not the held corpus) carries four optional 1×1
  maps of exactly 1.0, which change nothing.

**Checking the rule against the SDK.** A throwaway prototype of §2 ran on the SDK's own
pre-gain stage 2 (a scratch copy with `OpcodeList2` disabled). Its output was compared
with the SDK's post-gain stage 2:

| File | Samples | Exact | Off by 1 code | Worse |
|---|---|---|---|---|
| Pixel 4a | 12,192,768 | 12,190,254 (99.98%) | 2,514 | 0 |
| Pixel 3a | 12,192,768 | 12,188,768 (99.97%) | 4,000 | 0 |
| Pixel 7 Pro | 11,350,968 | 11,347,156 (99.97%) | 3,812 | 0 |

The off-by-one samples come from the SDK stepping the gain across columns in `f32`
increments (`fValueBase + fValueStep·index`), where §2 computes it directly in `f64`.

**A separate finding, not changed here.** The SDK scales every photosite by
`1 / (white − max black)` and subtracts that photosite's own black. Our normalisers scale
by `white − black` per position. On the Pixel 4a that puts the SDK 0 to 5 codes above us
before any gain. End to end, after gains of up to 3.4, the difference is up to 7 codes.

This is pre-existing, affects every file whose blacks differ by position (the Canon's
2047/2048 included), and would change held colour output. So it is recorded as a
follow-up, and SPEC-060's tolerance accounts for it rather than hiding it.

## Provenance

The parameter layout and the operation (a per-area, per-pitch gain from an interpolated
grid) come from the DNG specification's `GainMap` opcode, class 1.

Five points the specification leaves implicit were settled by reading Adobe DNG SDK 1.7.1
(`dng_gain_map.cpp`, `dng_misc_opcodes.cpp`):
- sampling at pixel centres over the stage-2 bounds;
- index clamping at the map edges;
- the clip at 1.0;
- an empty area meaning the whole image;
- the pitch phase anchored at the area's top-left.

That is provenance-ledger class 4, a permissively licensed implementation, read and
recorded, with no code copied. The SDK's source files point to the Adobe licence
agreement that accompanies the SDK. The builder records that agreement's terms in the
ledger row.

All five points were then confirmed empirically against the SDK's output, in the table
above.

## Alternatives Considered

- **Bit-exact emulation of the SDK's `f32` column stepping.** Rejected. It copies an
  implementation artefact, not the specification, and costs at most one code on 0.03% of
  samples.
- **Gain on the float value before the first rounding** (one rounding instead of two).
  Rejected. The SDK holds stage 2 as `u16` and rounds after each opcode, and the
  agreement above depends on matching that.
- **Refuse an area outside the image** (STAGE-016's original success criterion).
  Rejected. The SDK intersects it, and a refusal would turn a harmless map into a
  failure. Hostile values are still typed errors under §3.
- **Skip optional `GainMap`s.** Rejected. The SDK applies them, and a reader that can
  apply a correction should.
- **Fix the max-black normalisation in this stage.** Deferred. It changes held colour
  output and is unrelated to `GainMap`.
- **darktable, or an independent Python implementation, as the reference.** Rejected in
  favour of the SDK, which is the DNG reference implementation and settles the
  ambiguities above. The user chose this on 2026-11-19.

## Consequences

- The Pixel 3a, 4a and 7 Pro develop in colour and in mono. This is the "phone DNGs
  develop" outcome PROJ-004 has carried since STAGE-013.
- `apply_gain_maps` and `Error::TooManyGainMaps` are additive public API (0.2.x).
- `DevelopPath`'s documentation changes: both paths apply every `OpcodeList2` `GainMap`.
- `DEVELOP_VERSION` is `"6"`.
- A follow-up is carried for the max-black normalisation.
