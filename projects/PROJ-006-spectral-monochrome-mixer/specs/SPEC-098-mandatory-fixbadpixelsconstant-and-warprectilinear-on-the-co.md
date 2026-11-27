---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-098
  type: story                      # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-006
  stage: STAGE-034
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-086, DEC-084, DEC-053, DEC-040, DEC-078]  # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation, provenance-recorded-per-algorithm]
  related_specs: [SPEC-017, SPEC-064, SPEC-091, SPEC-092, SPEC-093, SPEC-097, SPEC-099]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: the Leica Q2, Q3 and SL3 colour DNGs (mandatory FixBadPixelsConstant and WarpRectilinear) give the linear plane and the mixed plane instead of refusing"

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
  tokens_estimate: 22000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 180
      recorded_at: 2026-11-27
      notes: "Design in a side session started by the PROJ-006 orchestrator: reading, dng_validate stage 1/2/3 on the three files, the SDK bad-pixel source read, synthetic CFA DNGs through dng_validate, an uncommitted prototype driven on every colour and mixer entry and through the whole corpus suite, the manifest count audit, the split into SPEC-099, DEC-086, this design, HANDOFF-208. Un-metered main-loop cycle; measured at the STAGE-034 close as orchestration cost."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-098: Mandatory FixBadPixelsConstant on the colour entries and WarpRectilinear passed through at sensor geometry, the Q2, Q3 and SL3 files

## Context

Second spec of STAGE-034 (PROJ-006). After SPEC-097 (shipped 2026-11-27) the M10 and M11
reach every colour and mixer entry, so the only refusals left on held Leica colour files
are three bodies:

- **Q2** (`LEICA-Q2/L1000750.DNG`, raw.pixls.us, CC0), **Q3**
  (`LEICA-Q3-PB/leica_q3_01.dng`) and **SL3** (`LEICA-SL3-PB/leica_sl3_01.dng`)
  (Photography Blog, no stated licence, local corpus only, never committed).
- All three unpack, bit-identical to dnglab's checksum and to `dng_validate`'s stage 1.
  Every colour entry then refuses with `UnsupportedRequiredOpcode { list: 1, id: 4 }`:
  each file carries a mandatory `OpcodeList1` `FixBadPixelsConstant` and a mandatory
  `OpcodeList3` `WarpRectilinear` with `N = 3`.
- The colour entries apply only `OpcodeList2` `GainMap`s (`src/opcode.rs:1145`,
  `DevelopPath::applied`). The mono develop applies both opcodes (`SPEC-017`, `SPEC-064`),
  with a 3×3 median that mixes Bayer colours (`src/develop.rs:948`).

onyx's rules (its SPEC-093 reply, 2026-11-26; STAGE-034 Design Notes):
1. **Warp at sensor geometry is the consumer's.** The SPEC-091 planes and
   `mono_mix_plane_*` must not refuse a mandatory `WarpRectilinear` and must say the
   consumer applies it. Only the developed outputs apply it, or refuse when they cannot.
2. **`FixBadPixelsConstant` runs on the CFA before demosaic, on every colour entry,**
   from neighbours of the same CFA colour.

**Split at design (DEC-086).** The three files' warps have one coefficient set per colour
plane (`N = 3`, up to 1.3 px apart at the corner). Applying them in the developed outputs
needs a camera-RGB intermediate, a per-plane parser and accessor, and onyx's answer on the
developed mix, which DEC-084 warps once over the mixed plane. That half is **SPEC-099**.
This spec is the bad-pixel fix on every colour entry plus the sensor-geometry
pass-through, which together meet STAGE-034's success criterion (a linear plane and a
mixed plane with no refusal).

## Goal

The held Q2, Q3 and SL3 colour files produce the linear plane and the mixed plane at
sensor geometry with no refusal, with `FixBadPixelsConstant` applied on the CFA before
demosaic from the four nearest same-colour photosites on every colour entry, and the
sensor-geometry entries pass a mandatory `WarpRectilinear` through to the consumer. No
output changes for any file that develops today.

## Inputs

- **Files to read:**
  - `decisions/DEC-086-...md` (this spec's decision); `DEC-084` item 3 and its amendment
    note; `DEC-040` (CFA phase from the `ActiveArea` origin); `DEC-053`, `DEC-078` §2.
  - `src/opcode.rs`: `DevelopPath` (1056), `enum Applied` (1129), `applied` (1145),
    `malformed_sets` (1160), `check_required_processing` (1236), `check_list` (1420),
    `parse_fix_bad_pixels_constant` (1037) and its params parse (430); the unit tests
    `colour_applies_nothing` (1932) and `ac6_mono_mix_path_applies_gain_maps_and_the_first_warp`
    (1949).
  - `src/color.rs`: `with_normalizer!` (196), `slot_normalizer` (2716; the raw read at
    2739), `pattern_normalizer` (2747; raw read at 2759), `resolve_colour_gain_maps`
    (2505), `check_colour_required_processing` (1562), `srgb_linear_plane` (1333),
    `srgb_linear_plane_rows` (1366), `resolve_mix_stage_two` (1121), and every public
    colour entry (424, 489, 650, 681, 718, 745, 887, 924, 960, 1503, 1538).
  - `src/develop.rs`: `develop_core` (1813; the mix skips list 1 at 1931), `scratch_len`
    (1635), `non_identity_warp` (555), `apply_fix_bad_pixels_constant` (948, read only).
  - `src/lib.rs`: `__fuzz_required_processing` (306), `DEVELOP_VERSION` (204).
  - `fuzz/fuzz_targets/opcode.rs` (73: the mono/colour subset invariant).
- **External (design only; the build needs none of it):** DNG 1.7.1.0
  `FixBadPixelsConstant` (p.111) and `WarpRectilinear`; Adobe DNG SDK 1.7.1
  `dng_bad_pixels.cpp`/`.h` (read for behaviour, class 4) and `dng_validate -1/-2/-3`;
  dnglab 0.7.2; exiftool 13.55.
- **Corpus:** `$IRRADIANCE_CORPUS_DIR` = `~/PSeven/experiments/crustimg_redo_plus/images`.
  The three files are recorded at this design as `[[available]]` entries (metadata only).

## Outputs

- **Files modified:**
  - `src/opcode.rs`: `DevelopPath::SensorGeometry` (new variant, documented); `applied`
    gives `First(FixBadPixelsConstant)` in list 1 to `Colour`, `SensorGeometry` and
    `MonoMix`; `First(WarpRectilinear)` in list 3 to `SensorGeometry`; a new `Applied`
    case for `MonoMix` list 3 (the first `WarpRectilinear` only when its `N` is 1, read
    from its first four parameter bytes); `malformed_sets` for `SensorGeometry` = the
    colour three; the `check_required_processing` and `check_list` docs; the two unit
    tests above (moved, see Failing Tests).
  - `src/color.rs`: the fix (Design §2–§3) at both normalisers' raw read; the
    `OpcodeList1` lookup after the `GainMap`s in every colour resolver (the colour
    develops, the planes and bands, `resolve_mix_stage_two`); the plane entries ask with
    `DevelopPath::SensorGeometry`; each entry's `# Errors` and a "the plane is not warped;
    the consumer applies `OpcodeList3`'s `WarpRectilinear`" paragraph on the six
    sensor-geometry entries.
  - `src/develop.rs`: under the mix, skip an optional first `WarpRectilinear` whose `N`
    is not 1 (Design §5); the `DevelopOptions::mono_mix` and `develop_core` docs.
  - `src/lib.rs`: `__fuzz_required_processing` returns the four paths
    `[Mono, Colour, MonoMix, SensorGeometry]`; `fuzz/fuzz_targets/opcode.rs` asserts the
    chain Colour ⊆ MonoMix ⊆ SensorGeometry ⊆ Mono (whatever a path accepts, every path
    to its right accepts).
  - `tests/colour_bad_pixels.rs` (new, tier A), `tests/leica_colour.rs` (extended, tier
    B), and the moved tests (Failing Tests).
  - `tests/corpus/manifest.toml`: the three `[[available]]` entries become `[[file]]`
    entries (Design §7), and every count site below.
  - `docs/provenance-ledger.md` (a row: the colour bad-pixel kernel, class 1 for the
    opcode, own choice for the kernel, the SDK read class 4 with nothing copied),
    `docs/conformance-matrix.md` (Q2, Q3, SL3 rows), `CHANGELOG.md` `[Unreleased]`
    (Added: `DevelopPath::SensorGeometry`; Changed: the colour entries apply
    `FixBadPixelsConstant`, the planes pass `WarpRectilinear` through, the developed mix
    refuses a mandatory multi-plane warp).
  - `decisions/DEC-084-...md`: a dated amendment note on item 3 (done at this design).
- **Not modified:** `DEVELOP_VERSION` (`"8"`), `MONO_MIX_VERSION` (`"1"`), the `Error`
  enum, `apply_fix_bad_pixels_constant` and the mono path, `src/warp.rs`, the demosaic and
  colour-transform primitives' behaviour, every existing output, determinism hash and
  Tier-B pin.
- **New public API:** `DevelopPath::SensorGeometry` only (additive: `#[non_exhaustive]`).

## Design

1. **The paths (DEC-086 items 1, 5, 6).** What each path applies:

   | Path | Entries | List 1 | List 2 | List 3 |
   |---|---|---|---|---|
   | `Mono` | `develop_into`, `develop_with_*` | first FBPC (3×3 median) | every GainMap | first warp, applied |
   | `Colour` | `develop_color_*`, threaded | **first FBPC (CFA kernel)** | every GainMap | nothing (refuse mandatory) |
   | `SensorGeometry` (new) | `srgb_linear_plane_*`, `mono_mix_plane_*` | **first FBPC (CFA kernel)** | every GainMap | **first warp, passed through** |
   | `MonoMix` | `develop_with_*` + `mono_mix` | **first FBPC (CFA kernel)** | every GainMap | first warp if **N = 1**; mandatory N ≠ 1 refused, optional skipped |
   | `Demosaic`, `ColourTransform` | the primitives | unchanged (refuse mandatory) | every GainMap | unchanged |

   "First" means the first instance with that ID, whatever its flags, as the mono
   appliers already read lists. `mono_mix_row` reads no samples and checks nothing
   today (`src/color.rs:887`); unchanged.
2. **Where the fix runs.** In `slot_normalizer` and `pattern_normalizer`, between the raw
   read and `normalize_range`: `sample = fix.sample(src, x, y, sample)`. These are the
   only two places the colour path reads `src` (`src/color.rs:2739`, `2759`), so the
   fused traversal, the bands and their halos, the threads and the developed mix's strips
   all see the same fixed value with no other change. The fix's parameters are resolved
   once per call (`Option<CfaBadPixels>`, `Copy`, `Sync`), never per pixel.
3. **The kernel (DEC-086 item 3).** For a raw `v` at `(x, y)`:
   - Not a candidate (return `v`) unless the list has a first FBPC, `v == Constant`
     (compared as `u32`), and `(x, y)` is inside the `ActiveArea`.
   - Colour from `CFAPattern` at slot `((y − top) & 1, (x − left) & 1)` with
     `wrapping_sub` (`DEC-040`'s form). Green → offsets `(±1, ±1)`. Red or blue →
     `(±2, 0)`, `(0, ±2)`.
   - Count a neighbour when inside `0..width × 0..height` and its value in `src` (never a
     fixed value) is not `Constant`. Return `(t + n / 2) / n` in `u32`, or `v` when
     `n == 0`.
   - Panic-free: checked index arithmetic only; `n ≤ 4` and `t ≤ 4 · 65535`.
   - If the CFA is absent or not a 2×2 pattern, no fix is built and the entry refuses at
     its CFA check as today (that check runs before any sample is read).
4. **`BayerPhase` is not used (DEC-086 item 4).** Parsed (a malformed block is still
   `MalformedOpcodeParams { id: 4, .. }`), not range-checked, not compared, not refused.
5. **The warp.** `SensorGeometry` accepts the first `WarpRectilinear` and never reads it.
   `Colour` still refuses a mandatory one. `MonoMix` reads `N` (the first `u32` of the
   parameter block) in `check_list`; a block too short to hold it is left to the
   applier's parse to refuse. In `develop_core` under the mix, `non_identity_warp`'s
   result is dropped when its `planes != 1`, so an optional `N ≠ 1` warp is skipped and
   the mandatory case never gets there (refused first).
6. **Error order (DEC-086 item 7).** On every colour resolver: levels, `GainMap`s, then
   `parse_fix_bad_pixels_constant`, then the CFA, before any write. Under the mix this is
   inside `resolve_mix_stage_two` (DEC-084 rank 5). `scratch_len` with the mix does not
   parse list 1. The scratch need of every entry is unchanged (the fix allocates nothing).
7. **Corpus manifest.** At design the three files are `[[available]]` entries. The build
   promotes them to `[[file]]` (tier `"b"`), because the plane oracle and the Tier-B ACs
   read them through the manifest. Values (all measured 2026-11-27):

   | | Q2 | Q3 | SL3 |
   |---|---|---|---|
   | path | `LEICA-Q2/L1000750.DNG` | `LEICA-Q3-PB/leica_q3_01.dng` | `LEICA-SL3-PB/leica_sl3_01.dng` |
   | make / model | `LEICA CAMERA AG` / `LEICA Q2` | `LEICA CAMERA AG` / `LEICA Q3` | `LEICA CAMERA AG` / `LEICA SL3` |
   | bits | 14 | 14 | 14 |
   | bytes | 87194624 | 90079744 | 77727232 |
   | sha256 | `a71f98405e9e9c1e4ac0f2d2e6c52d65daae3234fc0a9f6eed2a84ee4ce794ad` | `2bdafb50ef37cd7e7f94678675db3166fe4e8ab3552dd8f2ddec7b8f47668623` | `31f7f69a235a00dae8a18f29fcc74c83b490b986e0a1d9735a301078abd23330` |
   | licence | `CC0-1.0` | `personal-evaluation-only` | `personal-evaluation-only` |
   | raw_checksum (dnglab 0.7.2) | `271a816a8ae17496988ebdda4e6780d9` | `935a67a9ada37d56fb0ed6949e1827c0` | `bfa290077f179102ddc28411104688d9` |

   The Q2's source is raw.pixls.us (Leica Q2); the Q3's and SL3's are the Photography
   Blog review galleries (`img.photographyblog.com/reviews/leica_q3/sample_images/leica_q3_01.dng`,
   `.../leica_sl3/sample_images/leica_sl3_01.dng`), each with the M11's note: **no
   stated licence, never committed or redistributed, and no CI test may need it.** Make
   is spelt as exiftool prints it (`LEICA CAMERA AG`); if the reader's normalisation
   expects another spelling, follow the M10 entry and report it.

   Preview pins (exiftool 13.55, the header's procedure; all `kind "primary"`,
   compression 7, photometric 6):
   - **Q2:** `ifd_index` 0, 720×480, offset 29184, length 79360; `ifd_index` 2,
     1620×1080, offset 4015104, length 152576; `ifd_index` 3, 8368×5584, offset 108544,
     length 3906560.
   - **Q3:** 0, 720×480, 32256, 134689; 2, 1620×1080, 14689280, 693669; 3, 9520×6336,
     167424, 14521358.
   - **SL3:** 0, 720×480, 33280, 92440; 2, 1620×1080, 11169280, 379153; 3, 9520×6336,
     125952, 11042894.

   If `tests/previews.rs` disagrees with a pin, report it and do not edit the pin.
8. **Fuzz.** No new parser and no new input surface: the fix reads `src` the normalisers
   already read, and `check_list` reads four more bytes of a block it already frames.
   The `opcode` target gains the four-path chain (Outputs); seed it with one list-3
   `WarpRectilinear` with `N = 3` (`fuzz/seeds/opcode/warp-rectilinear-n3.bin`) and run
   it for the toolchain brief's smoke length.
9. **Provenance.** The opcode's meaning is class 1 (DNG 1.7.1.0). The kernel (the four
   nearest same-colour photosites, mean rounded half up, unfixed reads) is this design's
   choice and also the SDK's intended kernel; the SDK was read for behaviour (class 4,
   nothing copied) and run as an oracle. One ledger row records both.

## Implementation Context

**Measured 2026-11-27 (release build, this machine).** An uncommitted prototype of Design
§1–§3 (no §5 `N` skip in the applier, errors swallowed) was driven on the real files and on
synthetic DNGs. Its sources and outputs are in the PROJ-006 notes folder
(`spec-098-design-*`), not the repo.

| | Q2 `L1000750.DNG` | Q3 `leica_q3_01.dng` | SL3 `leica_sl3_01.dng` |
|---|---|---|---|
| Plane | 8424×5632, uncompressed, 1 strip | 9536×6344, Comp. 7 (P14 Nf2, BPS 14), 1 strip | as Q3 |
| CFA / `ActiveArea` / crop | GBRG; 0 0 5632 8392; origin x 12, y 24, 8368×5584 | RGGB; full; origin (4, 4), 9520×6336 | as Q3 |
| Black / white / Orientation | 512 (2×2) / 16383 / 1 | 512 (2×2) / 16383 / 8 | as Q3 |
| OpcodeList1 | FBPC, flags 0, version 1.3, `Constant` 0, `BayerPhase` 2 | `Constant` 0, `BayerPhase` 0 | `Constant` 0, `BayerPhase` 0 |
| `BayerPhase` vs `CFAPattern` (spec meaning) | agree (G on a G/B row) | agree (R) | agree (R) |
| OpcodeList3 | `WarpRectilinear`, flags 0, `N` 3, cx = cy = 0.5, kt = 0 | as Q2 | as Q2 |
| Warp at the corner (K1) / plane spread | −460 px / 1.12 px | −524 px / 1.18 px | −43 px / 1.33 px |
| unpack = dnglab = SDK stage 1 | yes (0 differ) | yes | yes |
| Photosites equal to `Constant` | 0 (min raw 26) | 0 (min raw 481) | **27,651**, all in columns 9528–9535 |
| Fixed / left at `Constant` | – | – | 10,615 / 17,036 |
| Plane change from the fix | none | none | 22,761 px, columns 9525–9535 only (crop ends at 9523) |
| SDK stage 2 vs clamp-normalise, other sites | max \|Δ\| 1 | max \|Δ\| 1 | max \|Δ\| 1 |
| `srgb_linear_plane_into` (FNV-1a) | Ok `0x98af7675eef43b27` | Ok `0x552573632a3fa8df` | Ok `0x0aa0de9063b2509a` |
| `srgb_linear_plane_f32_into` | Ok `0xc6a45aafc78635d6` | Ok `0x1776ef7c92e4d4c2` | Ok `0x0eb01beec9da33f4` |
| `mono_mix_plane_into` (2126/7152/722) | Ok `0x9452ae0ee0d3622f` | Ok `0xcbf43ca36ab20784` | Ok `0xe45ec46823e9c4eb` |
| `_rows_` forms (rows 100..200) | Ok, equal to the plane | Ok, equal | Ok, equal |
| `mono_mix_row` | Ok (as today) | Ok | Ok |
| `develop_into` (plain mono, unchanged) | Ok `0x1c1ca269399d11ad` | Ok `0xae9a8252ccfef17b` | Ok `0x73fd763637dddbc5` |
| `develop_color_*`, threaded | `UnsupportedRequiredOpcode { list: 3, id: 1 }` | same | same |
| `develop_with_into` + `mono_mix` | `{ list: 3, id: 1 }` (Design §5) | same | same |
| `demosaic_into`, `to_srgb_linear_into` | `{ list: 1, id: 4 }` (unchanged) | same | same |

- Digests are FNV-1a 64 over the output's samples, little-endian bytes (`f32` as its
  bits), as `tests/leica_colour.rs` does. **They are regression pins, not an oracle**:
  the oracle is the tier-A kernel test and the SL3 site counts. The Q2 and Q3 have no bad
  photosite, so their plane digests are the existing pipeline's output with the fix inert.
- **The SL3's bad columns** hold 0 or values near 16383 (columns 9528–9535 are inside the
  `ActiveArea` but outside the `DefaultCrop`); they are not image data.
- **SDK stage 2 on the SL3** equals a model of the SDK's kernel with its own parity at
  27,651 of 27,651 sites (within one code of the linearisation), and that parity is the
  opposite of `CFAPattern` at every site (DEC-086 Context). This design's kernel equals
  the SDK there at 18,703 sites; the other 8,948, red and blue sites the SDK fills from
  the other chroma, are all in columns 9528–9535.
- **Synthetic DNGs** (`dng_validate -2`; black 0, white 65535, so stage 2 is the raw
  value). Scene 24×20 RGGB, `v(x, y) = 1000 + 37x + 53y + [0, 9000, 9000, 20000][slot]`
  with slot `(y & 1)·2 + (x & 1)`, `Constant` 0 at the 20 sites below. Where the SDK's
  parity matches the mosaic (`BayerPhase` 1 or 2 with `ActiveArea` at the origin;
  `BayerPhase` 0 with `ActiveArea` (0, 1)) the SDK's stage 2 equals this kernel at every
  site. These are the AC1 literals (independently computed by Adobe's code):
  - **`ActiveArea` (0, 0):** (0,0) R 1090; (1,0) G 10090; (23,0) G 10867; (3,1) B 21199;
    (1,3) B 21221; (6,6) R 1540; (7,6) G 10572; (6,7) G 10598; (7,7) B 21630; (18,8) R
    2055; (12,10) R 1974; (13,10) G 11011; (16,10) R 2097; **(18,10) R stays 0**; (20,10)
    R 2295; (18,12) R 2337; (12,14) R 2161; (14,14) R 2285; (0,19) G 10991; (23,19) B
    22768.
  - **`ActiveArea` (top 0, left 1, bottom 20, right 24)**, same scene plus a bad (0,5):
    column 0 is masked, so (0,0), (0,5) and (0,19) **stay 0**; (1,0) 10127; (23,0) 10867;
    (3,1) 1164; (1,3) 1196; (6,6) 21510; (7,6) 10577; (6,7) 10593; (7,7) 1660; (18,8)
    22090; (12,10) 21974; (13,10) 11011; (16,10) 22122; (18,10) 22196; (20,10) 22270;
    (18,12) 22302; (12,14) 22186; (14,14) 22260; (23,19) 2768. A plane-origin colour
    (the mutant) matches the SDK at only 2 of these 18.
  - The prototype, read through `Container::parse`, gave planes (u16 and f32), bands,
    mixed planes, `develop_color_into` and the developed mix (no list 3 in these files)
    **identical** to the same develop of the hand-fixed mosaic with list 1 removed, for
    `BayerPhase` 0, 1, 2 and 3 and both `ActiveArea`s.
- **Cost:** `srgb_linear_plane_into` on the SL3 and Q3, 3 runs each: 1,076–1,122 ms with
  list 1, 1,000–1,100 ms with it removed (noise).
- **No output change for files that develop today:** a scan of all 16 held DNGs finds
  `OpcodeList1` only on the three Leica colour files and the Q2 Monochrom (mono path,
  untouched). The Pixels' `OpcodeList3` is an optional `N = 1` warp. The prototype ran
  the whole corpus suite (`cargo test --all-features --profile corpus --no-fail-fast`,
  corpus present): 963 passed, 13 failed, and all 13 are the moved assertions below.
- **`dng_validate`** gives no warning on the three files beyond two "not NULL terminated"
  Exif strings (Q3, SL3). Stage 3 is the camera-RGB `ActiveArea` after the per-plane
  warp (3 planes): SPEC-099's oracle, already generated.

## Questions for design — answered

- **Which entries apply which opcode, and in what order?** Design §1's table. The fix runs
  on the raw CFA before black, white, `GainMap`s, white balance and demosaic (the DNG
  stage-1 position). The opcode check passes a mandatory list-3 warp on the
  sensor-geometry entries only (`DevelopPath::SensorGeometry`); the developed colour
  still refuses it, and the developed mix refuses a mandatory `N ≠ 1` one. Source: the
  DNG stage order, onyx's rules, and the prototype's suite run.
- **The same-colour fill and `BayerPhase`.** The four nearest same-colour photosites
  (green: diagonals; red and blue: ±2 on the axes), mean rounded half up, unfixed reads,
  colour from `CFAPattern`. `BayerPhase` is read and not used, because the SDK's parity is
  the opposite of the specification's (shown on 27,651 real sites and 8 synthetic files)
  and all three held files agree with the specification. **Bound against the SDK:**
  exact (0 codes) wherever the SDK's parity matches the mosaic (20 of 20 and 18 of 18
  synthetic sites); on the held files the developed crop is identical with or without
  the fix, and the only difference from SDK stage 2 is the SL3's 8,948 red and blue sites
  in columns 9528–9535. Source: DEC-086, the probes.
- **How many photosites equal the constant?** Q2 0, Q3 0, SL3 27,651 (10,615 fixed,
  17,036 with no valid neighbour). Because no file has one in the developed crop, the
  tests prove the fix runs on a tier-A synthetic CFA fixture with the SDK-derived
  literals, plus the SL3's counts and column range when present.
- **The developed colour warp and `scratch_len`.** Moved to **SPEC-099** (the split). The
  fix itself adds no scratch to any entry, so `scratch_len` is unchanged.
- **Pass-through: documentation or an accessor?** Documentation in this spec. The
  existing `Sensor::opcode_list_3` and `parse_warp_rectilinear` give the bytes and `K1`;
  a per-plane accessor comes with SPEC-099, which needs the per-plane parse (open
  question for onyx below).
- **Versions.** No file that develops today changes (scan and suite above), so
  `DEVELOP_VERSION` stays `"8"` and `MONO_MIX_VERSION` `"1"`. A new `DevelopPath` value is
  needed (`SensorGeometry`), because the planes now accept what the colour develop
  refuses.
- **Further refusals.** Planes and mixed planes: none. Developed colour and developed mix:
  `{ list: 3, id: 1 }` until SPEC-099. Primitives: `{ list: 1, id: 4 }` (unchanged; a
  public CFA applier is a follow-up).
- **Spec size.** Split: the warp half is M on its own (camera-RGB intermediate of about
  363 MB at 60 MP, three-plane parse, f32 and threaded forms, a new oracle, and an onyx
  decision), so it is SPEC-099 (`depends_on: [SPEC-098]`). This spec is S to M; M is set.

## Acceptance Criteria

- **AC1. The kernel (tier A).** On the 24×20 scene above, built with
  `tests/support/tiff.rs`'s `dng_16bit_with_opcode_lists` (CFA, RGGB) and read through
  `Container::parse`, with list 1 a mandatory FBPC, `Constant` 0, `BayerPhase` 0: each of
  `srgb_linear_plane_into`, `srgb_linear_plane_f32_into`, `mono_mix_plane_into`,
  `develop_color_into`, `develop_color_f32_into` and `develop_with_into` with `mono_mix`
  equals, exactly, the same entry on the hand-fixed mosaic (the literals above) with list
  1 removed. The same holds for the `ActiveArea` (0, 1) variant (set on the parsed
  `Sensor`) with its literals, masked sites staying at 0. And each output differs from the
  unfixed one.
- **AC2. `BayerPhase` is not used.** `BayerPhase` 1, 2, 3 and 7 give output identical to
  `BayerPhase` 0 on both scenes.
- **AC3. The constant and the flags.** `Constant` 4000 (the scene's bad sites set to 4000;
  no other sample is 4000) gives AC1's literals, with (18,10) staying **4000**. `Constant`
  65536 fixes nothing (equal to the unfixed develop). An optional FBPC (flags 1) is
  applied. A second mandatory FBPC is `UnsupportedRequiredOpcode { list: 1, id: 4 }`. A
  malformed FBPC (7 bytes), optional or not, is `MalformedOpcodeParams { id: 4, .. }`.
- **AC4. Raw, not normalised.** With `BlackLevel` 512 and some non-bad samples below black
  (e.g. 100), only `Constant` photosites are fixed (equal to the hand-fixed develop).
- **AC5. Placement (tier A).** Bands of `srgb_linear_plane_rows_into`,
  `srgb_linear_plane_rows_f32_into` and `mono_mix_plane_rows_into` that tile the plane
  (e.g. 0..3, 3..11, 11..20) equal the whole plane; `develop_color_into_threaded` and its
  `f32` form at 1, 2 and 3 threads equal the single-thread develop; the developed mix's
  scratch form equals its allocating form.
- **AC6. The path table (tier A).** For each path, each list shape in
  `tests/required_processing.rs`, plus a mandatory `WarpRectilinear` with `N = 1` and with
  `N = 3`: `check_required_processing` agrees with each entry, and the outcomes match
  Design §1. Mandatory `N = 3` on the developed mix is `{ list: 3, id: 1 }`; optional
  `N = 3` is skipped (equal to the no-list-3 mix); `N = 1` is applied as today.
- **AC7. Leica files (tier B, skips loudly by name).** For each present file:
  `unpack_into`, `srgb_linear_plane_into`, `srgb_linear_plane_f32_into`,
  `mono_mix_plane_into`, the two `_rows_` forms over rows 100..200 (equal to the plane)
  and `mono_mix_row` are `Ok`, and the three digests equal the table. `develop_color_into`,
  `develop_color_f32_into` and `develop_with_into` with `mono_mix` are
  `UnsupportedRequiredOpcode { list: 3, id: 1 }`; `demosaic_into` and
  `to_srgb_linear_into` are `{ list: 1, id: 4 }`. Each file's opcode facts in the table
  (flags, `Constant`, `BayerPhase` agreeing with `CFAPattern`, `N` = 3) are asserted.
- **AC8. The SL3 fix (tier B).** With the SL3 present: 27,651 photosites equal 0, all in
  columns 9528–9535; the plane with list 1 differs from the plane with list 1 removed in
  exactly 22,761 pixels, all in columns 9525–9535; the developed crop never reaches them
  (column 9523 is the last, `output_dimensions`). The 10,615/17,036 split is asserted by
  counting candidates with at least one valid same-colour neighbour in the test itself.
- **AC9. Nothing else moves.** Every existing plane pin, determinism hash and Tier-B
  pin; `DEVELOP_VERSION` and `MONO_MIX_VERSION`. `cargo semver-checks` reports a minor
  (additive) change only.
- **AC10. Fuzz.** The `opcode` target's four-path chain and the N = 3 seed; smoke length,
  zero crashes.
- **AC11. Corpus.** The three entries promoted; `plane_md5_matches_the_pinned_raw_checksum`
  and the Compression 7 red-proof run on each present file (green, and red under the
  fault for the Q3 and SL3); the preview pins match.

## Failing Tests

Written first, and each seen to fail (or, tier B, to refuse) against `main`.

- **`tests/colour_bad_pixels.rs`** (new, tier A): `ac1_the_kernel_fills_from_the_four_nearest_same_colour_photosites`,
  `ac1_colour_is_indexed_from_the_active_area_origin`, `ac2_bayer_phase_is_not_used`,
  `ac3_constant_flags_and_instances`, `ac4_the_fix_reads_raw_samples`,
  `ac5_bands_threads_and_strips_equal_the_whole`. Red on `main`: every colour entry
  refuses list 1.
- **`tests/required_processing.rs`:** `ac6_every_path_applies_its_table` (new); red on
  `main` (no `SensorGeometry`).
- **`tests/leica_colour.rs`** (extended, tier B): `ac7_the_opcode_files_reach_the_sensor_geometry_entries`,
  `ac8_the_sl3_bad_columns_are_fixed`. Red on `main` with the corpus.

**Rule-clause map.**

| Clause (DEC-086) | Test | Distinguishing fixture / mutant it catches |
|---|---|---|
| 3 green → diagonals | ac1 | (7,6): 10572 vs axial 10577 (mutant a) |
| 3 red/blue → ±2 axial | ac1 | (6,6): 1540 vs diagonal 21510 (b) |
| 3 colour from the `ActiveArea` origin | ac1 (0,1) variant | 18 sites, 16 differ under plane origin (c) |
| 3 unfixed reads (order-free) | ac1 | (14,14) 2285 and (18,10) stays 0; in place they change (d) |
| 3 half-up rounding | ac1 | (7,6), (14,14), (1,3), (20,10), (18,8) differ under truncation (e) |
| 3 n = 0 keeps `Constant` | ac3 | `Constant` 4000: (18,10) stays 4000, not 0 (f) |
| 3 candidates inside the `ActiveArea` only | ac1 (0,1) | (0,5) stays 0; fixed if the whole plane is scanned (g) |
| 3 neighbours from the whole plane | ac1 (0,1) | (1,3): diagonals at column 0 are masked; excluded under an AA bound (h) |
| 3 `Constant` compared as `u32` | ac3 | 65536 fixes nothing; `as u16` would fix the zeros (i) |
| 1 first instance whatever flags | ac3 | optional FBPC applied (j); second mandatory refused (k) |
| 2 on raw values | ac4 | samples below black stay unfixed (l) |
| 4 `BayerPhase` unused | ac2 | phase 7 and the (0,1) scene with phase 0 (m) |
| 5 planes pass list 3 | ac6, ac7 | refused under the old table (n) |
| 6 colour refuses list 3 | ac6, ac7 | `Ok` if the colour path passes it (o) |
| 6 mix `N ≠ 1` refused / skipped | ac6 | mandatory N = 3 Ok, or optional N = 3 applied (p) |
| 7 order | ac3, ac6 | a malformed FBPC with a bad CFA: the FBPC error first |
| 2 one read site | ac5 | bands, threads and strips equal the whole |

**Mutants** (the build applies each by hand, records the failing test, reverts): (a) to (p)
above, plus (q) the fix applied after `normalize_range`, and (r) `Applied::First` for
`MonoMix` list 3 (the old rule).

**Moved assertions** (pre-approved; each was red under the prototype):
- `src/opcode.rs:1932` `colour_applies_nothing` → `colour_applies_the_first_fix_bad_pixels_and_every_gain_map`.
- `src/opcode.rs:1949` `ac6_mono_mix_path_applies_gain_maps_and_the_first_warp` → the mix
  applies the first FBPC, and the first warp only with `N = 1`.
- `tests/required_processing.rs:394` `colour_mandatory_opcode_in_list_1_is_rejected_on_every_entry`
  → a **second** mandatory FBPC is rejected on every colour entry.
- `tests/required_processing.rs:416` `colour_mandatory_opcode_in_list_3_is_rejected_on_every_entry`
  → rejected on the developed colour and (N = 3) the mix; passed through on the planes.
- `tests/required_processing.rs:1150` `public_check_agrees_with_every_develop_entry` →
  the plane entries ask with `SensorGeometry`.
- `tests/required_processing.rs:704` `required_processing_red_proof` → its colour control
  and cases follow the new table.
- `tests/mono_mix.rs:482` `ac10_refusals_in_order_leave_dst_untouched` and `:634`
  `the_row_range_refusal_comes_after_every_file_check` → the opcode file carries two
  mandatory FBPCs.
- `tests/developed_mix.rs:248` `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels` →
  `ac6_the_mix_applies_the_first_fix_bad_pixels` (and refuses a second);
  `:270` `ac6_the_mix_applies_a_mandatory_warp` → its plane half passes, the mix half
  stays; `:298` `ac6_the_mix_never_parses_opcode_list_1` → the develop refuses the
  malformed optional FBPC, `scratch_len` still ignores it; `:365` `ac8_each_fault_alone`
  and `:378` `ac8_every_ranked_pair_on_both_develop_entries` → fault F2's fixture
  (`tests/support/developed_mix.rs:407`, `:433`) becomes two mandatory FBPCs, same error.
- `tests/develop_scratch_alloc.rs:644` `ac10_the_developed_mix_allocates_nothing_with_scratch`
  → follows F2's fixture.

**Count and partition sites the promotion moves** (all fail on CI if missed unless
marked local):
- `tests/corpus_manifest.rs:24` `MANIFEST_ENTRIES` 14 → 17 (and the :27 comment);
  `:439` `assert_eq!(total, 28)` → 37 (and the :419–422 doc).
- `tests/previews.rs:614` `PINNED_PREVIEWS` 28 → 37.
- `tests/plane_oracle.rs:94` `DECODABLE` 6 → 7 (+ Q2, uncompressed); `:137`
  `DECODABLE_COMPRESSED` 7 → 9 (+ Q3, SL3; a new `LEICA_OPCODE` const, since
  `LEICA_COLOUR`'s doc says BPS 16); the `:162` sum 14 → 17.
- `tests/ifd_reader.rs:136` `CFA_CHECKED_ELSEWHERE` 7 → 10; the `:185` sum.
- `tests/metadata_oracle.rs:95` `CFA_METADATA_NOT_YET_ORACLED` 5 → 8 (local only).
- `tests/corpus/manifest.toml:65` header count ("28 in all" → 37) and a banner for the
  three entries.
- Not moved: `tests/plane_units.rs:1128` `HELD` (already stale, no assertion);
  `tests/linear_plane.rs:454`, `tests/mono_mix.rs:696`, `tests/developed_mix.rs:574`,
  `tests/colour_sdk.rs:41` (fixed lists; AC7 covers the new files).

## Non-Goals

- The warp on the developed outputs, the per-plane parser and accessor, and the
  developed mix's `N = 3` rule: SPEC-099.
- A public CFA bad-pixel applier, and the demosaic and colour-transform primitives
  accepting list 1 (a follow-up).
- Following the SDK's `BayerPhase` parity, or validating `BayerPhase`.
- Changing the mono path's 3×3 median.
- Other mandatory opcodes, highlight recovery, filter presets.
- Committing or redistributing any `-PB` file, or any CI test that needs one.

## Open questions

None blocks the build.
- **For the maintainer:** the Q3 and SL3 become tier-B `[[file]]` entries with licence
  `personal-evaluation-only`, as the M11 did. Confirm, or keep them `[[available]]`
  (AC7, AC8 and AC11 then run on the Q2 only, which has no bad photosite).
- **For the maintainer:** DEC-086 departs from the Adobe SDK on red and blue bad pixels
  (the SDK fills them from the other chroma on spec-following files). Accept the
  specification's reading, or ask for SDK parity.
- **For onyx:** (1) the viewport now receives a plane whose file says `N = 3`. Does it warp
  each plane with its own set, or one set over the mixed plane, and which? That answer
  decides SPEC-099's developed mix. (2) Does it read the warp from `OpcodeList3`'s bytes
  itself, or does it need irradiance to expose the three sets?

## Notes for the Implementer

- Resolve the fix once per call into a small `Copy` value and pass it into both
  normalisers; do not re-parse per pixel. Keep the counters out (the prototype had them).
- Do not swallow `parse_fix_bad_pixels_constant`'s error (the prototype did); it ranks
  after the `GainMap`s.
- Take AC1's literals from Implementation Context, not from your build. They were
  computed by Adobe's code.
- Build the scene in the test from the formula; write the expected mosaic from the
  literal list, not by calling a kernel.
- Never copy any `-PB` file into the repo, a fixture, a seed or CI.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
