---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-086
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-006
repo:
  id: irradiance

created_at: 2026-11-27
supersedes: null                   # amends DEC-084 item 3; DEC-084 stays active
superseded_by: null
status: accepted                   # at the SPEC-098 ship, 2026-11-28 (verify HANDOFF-209 approved it at 2006ceb); jysf accepts by merging the ship PR
deciders: [claude, jysf]

affected_scope:
  - src/opcode.rs
  - src/color.rs
  - src/develop.rs
  - src/lib.rs
  - decisions/DEC-084-the-developed-mix-is-the-mono-develop-geometry-over-the-mixed-plane-mixed-in-row-strips-into-caller-scratch.md
  - projects/PROJ-006-spectral-monochrome-mixer/specs/done/SPEC-098-mandatory-fixbadpixelsconstant-and-warprectilinear-on-the-co.md

tags:
  - opcode
  - bad-pixels
  - warp
  - colour
  - mixer
  - stage-034
---

# DEC-086: the colour entries fix bad pixels from the four nearest same-colour photosites, and the sensor-geometry entries pass the warp through (amending DEC-084 item 3)

## Decision

1. **Who applies `FixBadPixelsConstant`.** The colour develop (`DevelopPath::Colour`:
   `develop_color_into`, `develop_color_f32_into` and their threaded forms), the
   sensor-geometry entries (the new `DevelopPath::SensorGeometry`, item 5) and the
   developed mix (`DevelopPath::MonoMix`) apply the **first** `FixBadPixelsConstant` in
   `OpcodeList1`, whatever its flags, as the mono develop does (`SPEC-017`). Every later
   mandatory instance is refused as before. The stage primitives
   (`DevelopPath::Demosaic`, `DevelopPath::ColourTransform`) are unchanged: they still
   refuse a mandatory one, because no public applier exists for a composer to run on a
   CFA (item 8). The mono develop and its 3×3 median are unchanged.
2. **Where.** On the raw CFA sample, before black subtraction, white scaling, the
   `GainMap`s and demosaic: the DNG stage-1 position of `OpcodeList1`. It runs where the
   colour path reads a raw sample (its level normaliser), so every reader sees the same
   fixed value: the fused traversal, the row bands and their halos, the threads and the
   developed mix's strips. The fix allocates nothing and writes nothing back to `src`.
3. **The kernel.** A photosite is a candidate when its raw value equals `Constant` and it
   lies inside the `ActiveArea` (`SPEC-017`'s boundary). Its colour is read from
   `CFAPattern`, indexed from the `ActiveArea` origin (`DEC-040`).
   - A **green** candidate uses its four diagonal neighbours, `(x ± 1, y ± 1)`.
   - A **red or blue** candidate uses the four photosites two steps away on the axes,
     `(x ± 2, y)` and `(x, y ± 2)`.
   These are its nearest photosites of the same colour in a 2×2 Bayer mosaic. A
   neighbour counts when it is inside the plane (the masked border included, as
   `SPEC-017` rule 1) and its **unfixed** raw value is not `Constant`. With `n` counted
   neighbours summing to `t`, the value is `⌊(t + ⌊n/2⌋) / n⌋`, the mean rounded half
   up. With `n = 0` the photosite stays at `Constant`. Every candidate reads the
   original mosaic and never a fixed value, so the result does not depend on the order
   the photosites are visited. A `Constant` above 65535 matches no `u16` sample, so
   nothing is fixed and nothing is refused (as on the mono path).
4. **`BayerPhase` is read and not used.** `CFAPattern` decides each candidate's colour.
   `BayerPhase` is still parsed (a malformed block is still `MalformedOpcodeParams`), but
   it is neither range-checked nor compared with `CFAPattern`, and a disagreement is not
   refused. The reasons are in Context: the specification and the Adobe SDK give
   `BayerPhase` opposite green parities, all three held files agree with the
   specification and with their `CFAPattern`, and `CFAPattern` is the tag the demosaic
   already reads, so the fill colour can never differ from the colour the demosaic
   assigns.
5. **A new `DevelopPath::SensorGeometry`** (the enum is `#[non_exhaustive]`, so this is
   additive). It is the path of the `SPEC-091` planes (`srgb_linear_plane_into`,
   `srgb_linear_plane_f32_into` and their `_rows_` forms) and the `SPEC-092` mixed planes
   (`mono_mix_plane_into`, `mono_mix_plane_rows_into`). It applies the first
   `FixBadPixelsConstant` in `OpcodeList1` and every `GainMap` in `OpcodeList2`, and it
   **accepts without applying** the first `WarpRectilinear` in `OpcodeList3`, whatever
   its flags and its `N`: onyx's viewport applies it on the GPU (STAGE-034 Design Notes,
   onyx's rule 1). Each entry's documentation says that the plane is not warped and that
   the consumer applies the file's `WarpRectilinear`. A later mandatory instance in
   `OpcodeList3`, or any other mandatory opcode, is refused as on the colour path. Its
   malformed-tag sets are the colour path's three (`DEC-078` §2).
6. **The developed outputs.** `DevelopPath::Colour` applies no warp and still refuses a
   mandatory `WarpRectilinear` (`UnsupportedRequiredOpcode { list: 3, id: 1 }`) until
   `SPEC-099`. `DevelopPath::MonoMix` applies the first `WarpRectilinear` as `DEC-084`
   item 3 says only when its `N` is 1. When the first instance has `N ≠ 1`, the developed
   mix refuses it if it is mandatory (`UnsupportedRequiredOpcode { list: 3, id: 1 }`) and
   skips it if it is optional: one coefficient set over the mixed plane is not the
   per-plane warp DNG specifies, and which set onyx's viewport uses is `SPEC-099`'s
   question. This is the amendment to `DEC-084` item 3, together with item 1 (the mix now
   applies `OpcodeList1`).
7. **Error order.** Every colour path resolves its stage 2 in one order: the levels, the
   `GainMap`s, then the `OpcodeList1` lookup (`parse_fix_bad_pixels_constant`), all
   before the CFA and before any write. On the developed mix that is `DEC-084` item 7's
   rank 5, extended. `scratch_len` does not parse `OpcodeList1` under the mix, because the
   fix needs no scratch. Under the mix, `scratch_len` sizes an optional warp with `N ≠ 1`
   as skipped, as the develop skips it (item 6), so the scratch form agrees with the
   develop (clarified at the SPEC-098 ship, SPEC-098/FU-6). A malformed first `FixBadPixelsConstant`, optional or not, is now
   `MalformedOpcodeParams { id: 4, .. }` on these paths, where an optional one was skipped
   unread before. No held file has one.
8. **Scope kept out.** No public CFA applier (a composer cannot apply this kernel in
   place without a second buffer, because it reads unfixed values only), so the demosaic
   and colour-transform primitives keep refusing a mandatory `FixBadPixelsConstant`. No
   per-plane warp accessor (it comes with `SPEC-099`, which needs the per-plane parse).
9. **Versions and errors.** No `Error` variant is added or changed. `DEVELOP_VERSION`
   (`"8"`) and `MONO_MIX_VERSION` (`"1"`) do not move: no file that develops today
   changes (Validation).

## Context

STAGE-034's last refusal on held files (SPEC-098). The Leica Q2 (`LEICA-Q2/L1000750.DNG`,
CC0), Q3 and SL3 (`LEICA-Q3-PB`, `LEICA-SL3-PB`, Photography Blog, local only) unpack,
and every colour entry refused them with `UnsupportedRequiredOpcode { list: 1, id: 4 }`.
Each file's `OpcodeList1` is one `FixBadPixelsConstant` (flags 0, `Constant` 0;
`BayerPhase` 2 on the Q2, 0 on the Q3 and SL3), and its `OpcodeList3` is one
`WarpRectilinear` (flags 0, `N = 3`). onyx's rules (its SPEC-093 reply, 2026-11-26) say
the fix runs on the CFA before demosaic on every entry from neighbours of the same colour,
and that the warp at sensor geometry is the consumer's.

**What the specification says.** DNG 1.7.1.0 (`FixBadPixelsConstant`, p.111): the opcode
"patches (interpolates over) bad pixels in a Bayer pattern CFA image", marked by the
value `Constant`, and gives no kernel. `BayerPhase` 0 means the image's top-left pixel is
red, 1 green on a green/red row, 2 green on a green/blue row, 3 blue. On all three files
that agrees with `CFAPattern` (Q2 `GBRG` and phase 2; Q3 and SL3 `RGGB` and phase 0;
`ActiveArea` origin `(0, 0)` on all three).

**What the Adobe SDK does** (DNG SDK 1.7.1 source, `dng_bad_pixels.cpp`, read for its
behaviour only, class 4, nothing copied). It reads the unfixed source and writes a copy.
A green pixel takes the mean of its four diagonals, any other pixel the mean of the four
at distance 2 on the axes, skipping neighbours equal to `Constant`, rounded half up, and
left at `Constant` when none is valid. It decides green from `BayerPhase` alone with
`(row + col + phase + (phase >> 1))` even, which is green at the top-left for phases 0
and 3. That is the opposite of the specification's table for every phase.

**Driven, not assumed (2026-11-27):**
- **The SL3** has 27,651 photosites equal to 0, all in its last eight columns
  (9528–9535). Those columns lie inside the `ActiveArea` but outside the `DefaultCrop`
  (columns 4–9523), and they hold either 0 or values near white, not image data. The
  SDK's stage 2 (`dng_validate -2`) equals a model of the SDK kernel with the SDK's
  parity at **all 27,651** sites, within one code of the linearisation, and the parity
  disagrees with `CFAPattern` at every one. With the colour from `CFAPattern` (this
  decision), 10,615 sites are fixed and 17,036 have no valid same-colour neighbour and
  stay at 0. The SDK also leaves those 17,036 at 0. The Q2 and Q3 have no photosite
  equal to their `Constant` (minimum raw values 26 and 481).
- **Synthetic CFA DNGs** (24×20 `RGGB`, black 0, white 65535, so stage 2 equals the raw
  values; 20 bad photosites: isolated ones of each colour, an adjacent pair, a
  same-colour pair, corners and edges, and a red one whose four same-colour neighbours
  are all bad). With `BayerPhase` 1 or 2, where the SDK's parity happens to agree with
  the mosaic, the SDK's stage 2 equals this decision's kernel at 20 of 20 sites, edges
  and the all-bad case included. With phase 0 or 3 (the specification's phases for this
  mosaic) it equals it at 3 of 20 and fills red and blue from the other chroma. The SDK
  model matches the SDK at 20 of 20 in all four cases. With the `ActiveArea` at `(1, 1)`
  the SDK reads `BayerPhase` from the plane origin, not the `ActiveArea` origin.
- **Cost.** `srgb_linear_plane_into` on the 60 MP SL3 and Q3 takes 1,076–1,122 ms with
  `OpcodeList1` present and 1,000–1,100 ms with it removed, which is within run-to-run
  noise. The fix is one comparison per raw read.
- **Output reach.** On the SL3 the fixed plane differs from the unfixed one only in
  columns 9525–9535 (22,761 pixels: the fixed columns and two to three columns of
  demosaic spread), all outside the `DefaultCrop`.

## Alternatives Considered

- **The mono applier's 3×3 median** (`apply_fix_bad_pixels_constant`). On a Bayer
  mosaic its eight neighbours are of two or three colours, so it fills from the wrong
  colour, against onyx's rule. It also depends on the visiting order, since it fixes in
  place. Rejected.
- **The SDK's kernel with the SDK's parity** (exact Adobe stage 2). On every held file it
  would fill red and blue bad pixels from the other chroma, against onyx's rule and the
  specification's `BayerPhase` table. Rejected; recorded as a known difference from the
  SDK (Consequences).
- **Four axial neighbours at distance 2 for every colour.** It needs no colour at all and
  equals the SDK at every true green site, but a green photosite's diagonals are nearer
  (√2 against 2). Rejected for the nearer neighbours, which are also the SDK's own choice
  for green.
- **A median of the same-colour neighbours.** Order-independent too, but with four
  neighbours its even-count tie rule is a choice the SDK does not make, and it gives no
  match with any reference. Rejected.
- **Refusing a `BayerPhase` that disagrees with `CFAPattern`.** A file written to the
  SDK's parity (which Adobe software accepts and handles same-colour) would then be
  refused. Using `CFAPattern` never fills from the wrong colour and never refuses.
- **Fixing in place on a copy of the plane** (the mono path's shape). It costs a `w·h`
  scratch plane (121 MB at 60 MP) and changes the result with the visiting order when two
  bad photosites are neighbours. Rejected for the read-time fix.
- **Warping at sensor geometry, or refusing the warp there.** Both are against onyx's
  rule 1. Rejected.
- **One spec for both opcodes.** The three-plane warp needs a camera-RGB intermediate
  in the colour develop, a per-plane parser and accessor, and onyx's answer on the
  developed mix (DEC-084 warps the mixed plane once). That is `SPEC-099`.

## Consequences

- **Positive:** the held Q2, Q3 and SL3 reach every sensor-geometry entry
  (`srgb_linear_plane_*`, `mono_mix_plane_*`) with `Ok`, the stage's success criterion.
- **Positive:** the fix is a pure function of `src`, so bands, halos, threads and strips
  are bit-identical to the whole plane with no new code, and it allocates nothing.
- **Negative:** irradiance differs from the Adobe SDK on a red or blue bad photosite of
  any file whose `BayerPhase` follows the specification. On the held files every such
  site is outside the `DefaultCrop`, so no developed output can differ. The fixed CFA
  differs from Adobe's stage 2 at 8,948 of the SL3's 27,651 sites, all in its last eight
  columns, and the sensor-geometry planes carry that into columns 9525–9535.
- **Negative:** until `SPEC-099` the three files have no colour develop and no developed
  mix, and the demosaic and colour-transform primitives still refuse them.
- **Neutral:** an optional `FixBadPixelsConstant` is now applied on the colour paths (as
  on the mono path), and a malformed one is refused. No held colour file has one.
- **Accepted departure (maintainer, 2026-11-27, at SPEC-098's build).** The difference
  from the Adobe SDK on red and blue bad photosites (above) is accepted: irradiance
  fills them from the same colour per `CFAPattern` and does not follow the SDK's
  `BayerPhase` parity. The departure is also recorded in `docs/provenance-ledger.md`
  (the kernel's row) and `CHANGELOG.md`. The same ruling made the Q3 and SL3 tier-B
  `[[file]]` entries, `personal-evaluation-only` and never redistributed, as the M11.

## Validation

- **No output change for files that develop today.** No held colour file but the three
  Leica files carries `OpcodeList1` (a scan of all 16 held DNGs). The Pixels carry an
  optional `WarpRectilinear` with `N = 1` in `OpcodeList3`, which is skippable on every
  colour path and still applied by the developed mix (`N = 1`). The mono develop is
  untouched. A prototype of items 1–6 ran the whole suite with the corpus present; only
  the tests that pin the old refusals moved (listed in SPEC-098's Failing Tests).
- SPEC-098's tests pin the kernel on a tier-A CFA fixture with hand-computed values for
  every case above, the order independence, the read-time placement (bands, threads and
  strips equal to the whole plane), the path table, and, on the SL3 when present, the
  10,615 fixed and 17,036 unfixable sites.

## References

- DNG Specification 1.7.1.0, `FixBadPixelsConstant` and `WarpRectilinear`.
- Adobe DNG SDK 1.7.1, `dng_bad_pixels.cpp` (`dng_opcode_FixBadPixelsConstant::ProcessArea`)
  and `dng_bad_pixels.h` (`IsGreen`): read for behaviour, class 4, nothing copied.
- `SPEC-017` (the mono applier), `SPEC-091`, `SPEC-092`, `SPEC-093`, `SPEC-098`,
  `SPEC-099`; `DEC-040`, `DEC-053`, `DEC-069`, `DEC-078`, `DEC-084`.
