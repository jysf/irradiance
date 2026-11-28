---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-099
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
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
  decisions: [DEC-087, DEC-086, DEC-084, DEC-059, DEC-024]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-064, SPEC-091, SPEC-092, SPEC-093, SPEC-098, SPEC-100]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-098]             # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: the Leica Q2, Q3 and SL3 colour DNGs (a mandatory three-plane WarpRectilinear) give chromatic-aberration-corrected sensor-geometry planes and name the one warp onyx applies; the developed outputs follow in SPEC-100"

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
  tokens_estimate: 100000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 22658138
      estimated_usd: 17.26
      duration_minutes: 35
      recorded_at: 2026-11-28
      notes: "Design in a side session started by the PROJ-006 orchestrator: reading, the coefficient and fold probe, dng_validate -3 on the three files and on green-only patched copies, the camera-RGB Lucas-Kanade oracle, the sRGB-space attempt, the pre-registered digests, the split into SPEC-100, DEC-087, this design, HANDOFF-210. A FLOOR measured just before the commit over transcript ddeb3c1e-f355-441d-a270-9ed5a5e56e78.jsonl (77 unique message.ids, all claude-opus-5-5, no subagents): input 154 + cache-creation 291,192 (all 1-hour) + cache-read 17,080,573 + output 112,832, priced at 5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x. Re-summed at ship (2026-11-28) over the whole transcript (92 unique message.ids, all claude-opus-5-5, no subagents): input 190 + cache-creation 308,200 (all 1-hour) + cache-read 22,227,123 + output 122,625, priced as above; the floor was 17,484,751 / 14.27 USD. Duration is the transcript's first-to-last timestamp. Counted here, so the STAGE-034 orchestration cost must not count this transcript again."
    - cycle: build
      handoff: HANDOFF-210
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 54368631
      estimated_usd: 35.43
      duration_minutes: 50
      recorded_at: 2026-11-28
      notes: "Build in a side session started by the PROJ-006 orchestrator. A FLOOR measured live just before the commit over transcript 111b6c75-4198-4766-9cc9-425a4fe387af.jsonl (134 unique message.ids, all claude-opus-5-5, no subagents): input 271 + cache-creation 417,566 (all 1-hour) + cache-read 45,253,091 + output 158,067. Re-summed at ship (2026-11-28) over the whole transcript (152 unique message.ids, all claude-opus-5-5, no subagents): input 313 + cache-creation 432,697 (all 1-hour) + cache-read 53,767,104 + output 168,517, priced as the design; the floor was 45,828,995 / 30.76 USD. Duration is the transcript's first-to-last timestamp."
    - cycle: verify
      handoff: HANDOFF-211
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 21471734
      estimated_usd: 14.73
      duration_minutes: 51
      recorded_at: 2026-11-28
      notes: "Verify in a side session started by the PROJ-006 orchestrator. A FLOOR measured just before the commit over transcript 553db268-69b4-40d6-9090-2fc4c6f5fbb2.jsonl (84 unique message.ids, all claude-opus-5-5, no subagents): input 171 + cache-creation 223,687 (all 1-hour) + cache-read 17,445,970 + output 64,547. Re-summed at ship (2026-11-28) over the whole transcript (97 unique message.ids, all claude-opus-5-5, no subagents): input 199 + cache-creation 234,068 (all 1-hour) + cache-read 21,165,433 + output 72,034, priced as the design; the floor was 17,734,375 / 12.57 USD. Duration is the transcript's first-to-last timestamp."
    - cycle: ship
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 3668475
      estimated_usd: 3.37
      duration_minutes: 20
      recorded_at: 2026-11-28
      notes: "Ship in a side session started by the PROJ-006 orchestrator: re-summing the three earlier transcripts, DEC-087 promotion, reflection, follow-ups, signal, stage and brief, archive, brag. A FLOOR, measured from transcript 90022d7a-0f64-40ac-9521-f0b8e072e413.jsonl (27 unique message.ids, all claude-opus-5-5, no subagents) before the commit, the PR and the CI watch: input 54 + cache-creation 124,273 (all 1-hour) + cache-read 3,529,571 + output 14,577, priced as the design."
  totals:
    tokens_total: 102166978
    estimated_usd: 70.79
    session_count: 4
shipped_at: 2026-11-28
---

# SPEC-099: The per-plane WarpRectilinear at sensor geometry: red and blue resampled to green's geometry, and the consumer's warp named

> **Designed 2026-11-28 (DEC-087).** Split at design: this spec is the
> chromatic-aberration (CA) step on the sensor-geometry entries plus the accessor. The
> developed colour output and the developed mix are **SPEC-100** (an outline at frame,
> `depends_on: [SPEC-099]`). The title kept its outline slug; the heading names the scope.

## Context

Third spec of STAGE-034 (PROJ-006), split out of SPEC-098 at its design (2026-11-27,
DEC-086). After SPEC-098 the Q2 (`LEICA-Q2/L1000750.DNG`, CC0), Q3 and SL3
(`LEICA-Q3-PB/leica_q3_01.dng`, `LEICA-SL3-PB/leica_sl3_01.dng`, Photography Blog, local
only, never committed) give a linear plane and a mixed plane at sensor geometry, with
`FixBadPixelsConstant` applied on the CFA and the whole `WarpRectilinear` left to the
consumer. Their developed outputs still refuse (`UnsupportedRequiredOpcode { list: 3, id: 1 }`
on `DevelopPath::Colour`, and DEC-086 item 6 on `DevelopPath::MonoMix`).

Each file's `WarpRectilinear` has `N = 3`, one coefficient set per colour plane, with
`cx = cy = 0.5` and no tangential terms. The full correction moves a corner by about 460 px
on the Q2, 524 px on the Q3 and 43 px on the SL3, and the planes' sets differ by up to
1.45, 1.66 and 1.38 px at sensor geometry (lateral chromatic aberration; Implementation
Context). The mono warp (`SPEC-064`, `src/warp.rs`) applies one set to one plane, and the
parser keeps only `K1` (`src/opcode.rs`, `parse_warp_rectilinear_params`). On a colour
file `K1` is **red's** set, so a consumer that calls `parse_warp_rectilinear` today warps
with red's coefficients.

**onyx's answers (2026-11-28, onyx's DEC-114)** to the two questions SPEC-098's design
left in the PROJ-006 brief. They amend onyx's rule 1 (STAGE-034 Design Notes) and so
DEC-086's sensor-geometry pass-through:

1. **Split the warp in two.**
   - At sensor geometry, before the mix, `srgb_linear_plane_*` and `mono_mix_plane_*`
     resample red and blue to green's geometry: each plane's set relative to green's, with
     plane colours from `CFAPlaneColor`. This step always runs and does not follow
     `warp_alpha`. With the opcode's one shared centre it is radial, so it needs only a 1D
     inverse of green's radial polynomial. Non-zero tangential terms stay refused.
   - Then one warp, green's set, on the mixed plane. onyx's viewport applies it on the GPU
     at the user's alpha, and the developed mix applies the same warp at the same alpha, so
     export equals the viewport by construction (DEC-084) and onyx's warp parity check
     carries over. On colour files `warp_alpha` then dials back lens distortion only;
     chromatic aberration is always corrected.
   - A file with `N = 1`, or with identical sets, gives output bit-identical to today's.
2. **An accessor, yes.** It exposes the warp the consumer must apply (green's set), so the
   choice of set lives in one place and matches the developed mix, and all `N` sets with
   their plane colours (onyx's Optics bench draws them as lateral-CA vectors). onyx stops
   using `parse_warp_rectilinear` on colour files; monochrome files keep it, since `K1` is
   the right set for a single plane.

onyx also answered SPEC-097's question: no highlight recovery above `WhiteLevel`. Outputs
keep clipping at white, so nothing in this spec changes for the M10.

## Goal

The held Q2, Q3 and SL3 files give **chromatic-aberration-corrected** linear and mixed
planes at sensor geometry (`srgb_linear_plane_into`, `srgb_linear_plane_f32_into`,
`mono_mix_plane_into` and the three `_rows_` forms): red and blue are resampled to green's
geometry, and only green's set is left for the consumer. A new accessor,
`opcode::sensor_geometry_warp`, names that warp and lists all `N` sets with their plane
colours. Every file with no per-plane difference (`N = 1`, or identical sets) gives output
bit-identical to today's. The developed outputs are SPEC-100.

## Inputs

- **Files to read:** `src/warp.rs` (`source_coord`, `bilinear_sample`, `scale_bounds`);
  `src/opcode.rs` (`parse_warp_rectilinear_params`, `first_recognised`, `WarpRect`,
  `DevelopPath`, `check_required_processing`); `src/color.rs` (`srgb_linear_plane`,
  `srgb_linear_plane_rows`, `srgb_linear_plane_rows_with`, `develop_color_with`,
  `WholePlane`, the three `store_*` functions, `mono_mix_pixel`); `src/demosaic.rs`
  (`develop_color_fused_in`, `develop_rows_banded`, `BAND_HALO`); DEC-024, DEC-059,
  DEC-084, DEC-086, DEC-087.
- **External:** DNG 1.7.1.0 `WarpRectilinear` (p.103-104; "N must be 1 or the total number
  of image planes"). `dng_validate -3` (stage 3: camera RGB over the `ActiveArea` after the
  per-plane warp), on each file and on a scratch copy whose red and blue sets are replaced
  by green's (`scripts/sdk-stage3.sh`, new, AC8).
- **Corpus:** the three files, already tier-B `[[file]]` entries (SPEC-098).

## Outputs

- **`src/opcode.rs`:** the public accessor `sensor_geometry_warp(&Sensor) ->
  Result<Option<SensorGeometryWarp>, Error>`, its two public types `SensorGeometryWarp`
  and `PlaneWarp`, and a crate-private per-plane parser shared with the CA step. The
  `DevelopPath::SensorGeometry` doc says what the path now applies.
  `parse_warp_rectilinear` and `parse_warp_rectilinear_params` are unchanged.
- **`src/warp.rs`:** the crate-private CA rule (DEC-087 item 3): the fold check, the
  4096-interval table, the displacement bound and halo, and the per-pixel source
  coordinate, with unit tests.
- **`src/color.rs`:** the CA step wired into `srgb_linear_plane` and
  `srgb_linear_plane_rows` (DEC-087 item 4), and the six entries' docs (the plane is
  CA-corrected; the consumer applies `sensor_geometry_warp(..).consumer()`).
- **`src/lib.rs`:** `Error::UnsupportedWarpFold { slope_min: f64 }` (ErrorKind
  `Unsupported`), its `Display`, its `kind()` arm; a `__fuzz_warp_planes` hook.
- **`fuzz/fuzz_targets/warp_opcode.rs`:** the per-plane parser through the hook (AC10).
- **`scripts/sdk-stage3.sh`** (new), the `sdk-stage3` mention in `AGENTS.md` §6 and the
  `corpus-status` SKIP line for `$IRRADIANCE_SDK_STAGE3_DIR`.
- **Tests:** `tests/plane_warp.rs` (new, tier A), `tests/leica_colour.rs` (extended,
  tier B), `tests/colour_sdk_stage3.rs` (new, tier B plus the SDK directory), unit tests
  in `src/warp.rs` and `src/color.rs`, `tests/error_kind.rs` (the new variant).
- **Docs:** `CHANGELOG.md` `[Unreleased]`; `docs/provenance-ledger.md` row for the CA
  rule; DEC-086 items 5 and 8 get dated amendment lines pointing at DEC-087.

## Design

### 1. The accessor (`DEC-087` item 1)

```rust
// src/opcode.rs
pub fn sensor_geometry_warp(sensor: &Sensor) -> Result<Option<SensorGeometryWarp>, Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SensorGeometryWarp { /* private: sets [PlaneWarp; 3], len, consumer index */ }
impl SensorGeometryWarp {
    /// The one warp the consumer applies at sensor geometry.
    pub fn consumer(&self) -> &WarpRect;
    /// Every coefficient set, in file order (1 or 3 of them).
    pub fn sets(&self) -> &[PlaneWarp];
    /// Whether the sensor-geometry entries resample red and blue (the CA step runs).
    pub fn is_per_plane(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct PlaneWarp {
    /// The image plane this set belongs to (its index in the opcode).
    pub plane: u32,
    /// `CFAPlaneColor[plane]` (0 red, 1 green, 2 blue), or `None` when the one set
    /// applies to every plane (`N = 1`) or the file has no CFA.
    pub color: Option<u8>,
    /// The set, as `parse_warp_rectilinear` would return it if it were `K1`
    /// (`planes` is the file's `N`).
    pub warp: WarpRect,
}
```

- It reads the **first** `WarpRectilinear` in `OpcodeList3`, whatever its flags (as
  `DevelopPath::SensorGeometry` and the mono develop do). `Ok(None)` when there is no list
  3 or no `WarpRectilinear` in it. A list that `parse_opcode_list` rejects gives that error.
- **`N = 1`:** one set, `color: None`, and it is the consumer's warp.
- **`N = 3` on a CFA sensor:** three sets, `color` from `CFAPlaneColor`; the consumer's
  warp is the set whose colour is 1 (green). A `CFAPlaneColor` that is not one each of 0,
  1 and 2 is `UnsupportedCfaPlaneColor` (the colour entries refuse it the same way).
- **`N = 3` without a CFA:** three sets with `color: None`, and the consumer's warp is
  `K1`, which is what `develop_into` applies (`parse_warp_rectilinear`).
- **Any other `N`** is `MalformedOpcodeParams { id: 1, declared_size }` (DNG: "N must be
  1 or the total number of image planes", and every image this crate reads has 1 or 3).
- `is_per_plane()` is true exactly when the CA step runs (§2).
- It does not refuse tangential terms or a fold: it describes the file; the entries
  refuse.
- It allocates nothing (fixed array of three) and never panics.

### 2. When the CA step runs (`DEC-087` item 2)

On the six sensor-geometry entries (`DevelopPath::SensorGeometry`), with the first
`WarpRectilinear` in `OpcodeList3`, **whatever its flags**:
- `N = 1`, no warp, or `N = 3` with red's and blue's sets each equal to green's (all six
  coefficients compared with `==`; `cx`, `cy` are shared): **no CA step**. The traversal
  is today's, untouched, so the output is bit-identical (onyx rule 3).
- `N = 3` with at least one of red and blue different from green: the CA step runs on
  each plane whose set differs; a plane whose set equals green's is not resampled.
- `N` not 1 or 3: `MalformedOpcodeParams { id: 1, .. }` (§5).
The developed outputs (`DevelopPath::Colour`, `DevelopPath::MonoMix`), the mono develop
and the stage primitives are unchanged (§6).

### 3. The rule (`DEC-087` item 3, exact)

The CA step works on the demosaiced camera triple, before the colour matrix (the warp is
defined on the camera-RGB stage-3 image), over the `ActiveArea` of width `aw` and height
`ah`, in `ActiveArea`-relative coordinates `(x, y)`, `0 ≤ x < aw`, `0 ≤ y < ah`:

1. **Geometry** exactly as `source_coord`: `cx = 0 + cx̂·(aw − 1)`,
   `cy = 0 + cŷ·(ah − 1)`, `mx = max(|0 − cx|, |aw − 1 − cx|)`, `my` likewise,
   `m = mx.hypot(my)`. If `m == 0` there is no CA step.
2. **The fold check.** With green's `kr`, `q(s) = kr0 + 3·kr1·s + 5·kr2·s² + 7·kr3·s³` is
   the derivative of `√s·f_g(s)` times `2√s` divided out: green's radial map
   `ρ = r·f_g(r²)` is strictly increasing on `r ∈ [0, 1]` exactly when `q > 0` on
   `s ∈ [0, 1]`. Its minimum is `scale_bounds` applied to the coefficients
   `[kr0, 3·kr1, 5·kr2, 7·kr3]` (closed form, `DEC-059` §3). If that minimum is not
   `> 0` (a NaN included), the entry refuses with `UnsupportedWarpFold { slope_min }`.
   (`q > 0` also gives `f_g > 0` on `[0, 1]`, so every ratio below is finite.)
3. **The table.** `T = 4096`. For `j = 0..=T`, `σ_j = j / T` (as `f64`, `j as f64 / 4096.0`).
   With `f(k, s) = k0 + k1·s + k2·s·s + k3·s·s·s` (this operation order) and
   `h(s) = s·f_g(s)·f_g(s)` (left to right):
   - if `σ_j ≥ f_g(1)·f_g(1)`, `s_j = 1` (beyond green's image the ratio is held at the
     rim, rule 5);
   - otherwise bisection: `lo = 0, hi = 1`; loop `mid = 0.5·(lo + hi)`; stop when
     `mid == lo` or `mid == hi`; if `h(mid) < σ_j` then `lo = mid` else `hi = mid`; then
     `s_j = 0.5·(lo + hi)`.
   - For each plane `c` (red, blue) whose set differs: `t_c[j] = f_c(s_j) / f_g(s_j)`.
4. **The source coordinate** of pixel `(x, y)` for plane `c`:
   `dx = (x − cx) / m`, `dy = (y − cy) / m`, `σ = dx·dx + dy·dy`;
   `u = min(σ·4096, 4096)`; `j = min(⌊u⌋, 4095)`; `w = u − j`;
   `k = t_c[j] + (t_c[j+1] − t_c[j])·w`;
   source `(cx + k·(x − cx), cy + k·(y − cy))`.
5. **The value** is `bilinear_sample` (DEC-024's kernel, `src/warp.rs`) of plane `c`'s
   demosaiced camera samples over the `ActiveArea` at that source: clamped to
   `[0, aw − 1] × [0, ah − 1]`, rounded half up to `u16`. Green and every pixel outside
   the `ActiveArea` (the masked border) pass through unchanged.
6. **The halo.** `B = m · max over j < T of max(|t_c[j] − 1|, |t_c[j+1] − 1|) · √((j+1)/T)`
   over the resampled planes, and `H = ⌈B⌉ + 1` rows (an `f64` bound on the interpolated
   function itself: on interval `j` the ratio is linear between its nodes and `√σ ≤
   √((j+1)/T)`, so no source row is farther than `B` from its pixel's row). `H` is capped
   at `ah`.

Why the composition: with one shared centre every set is radial, so a pixel at
green-corrected radius `r` lies at sensor radius `ρ = r·f_g(r²)` in green and at
`r·f_c(r²)` in plane `c`. Plane `c` at green's sensor geometry is therefore plane `c`
sampled at `ρ·f_c(r²)/f_g(r²)`, with `r = g⁻¹(ρ)`; the table is that ratio as a function of
`σ = ρ²`. Green's warp over the result then gives plane `c` at `r·f_c(r²)`, the DNG
stage-3 answer, up to a second interpolation. **Beyond green's image** (`ρ > f_g(1)`, only
visible at `warp_alpha < 1`) the DNG polynomial is undefined; the ratio is held at its rim
value `t_c[T]`, which is continuous and bounded.

### 4. Where it runs (`DEC-087` item 4)

A sink-side delay ring between the fused traversal and `store`:
- **Whole plane** (`srgb_linear_plane`): the traversal emits each pixel's camera triple
  row by row (`develop_color_fused_in`). The ring keeps the last `2H + 2` emitted rows of
  all three channels (`w` wide). Once row `y + H` has arrived (or the traversal ends, for
  the last `H` rows), row `y`'s `ActiveArea` pixels are finished by §3 and handed to
  `store` with their own `(pixel, [r', g, b'])`; every other pixel goes to `store` as
  emitted. The ring is owned by the entry body and flushed after the traversal returns.
  Scratch: `3·(2H + 2)·w` `u16`, about 0.46 MB at `w = 9536`, `H = 3`.
- **Rows forms** (`srgb_linear_plane_rows`): the band develops rows
  `max(s0 − H, 0)..min(s1 + H, height)` through `develop_rows_banded` (whose pixels equal
  the whole plane's, DEC-044), the same ring runs over them, and only rows `s0..s1` are
  stored. Every value equals the whole plane's bit for bit, so strips of any heights
  still tile the plane exactly. Its scratch grows by `2H` band rows and the ring.
- Every sink here places by pixel index, so the delay changes no placement.
- `store` (u16, unclamped f32, mono) is unchanged, so all three outputs see the same
  corrected triple.

### 5. Refusals and order (`DEC-087` item 5)

Only when the CA step must run (§2), after the CFA check and before the colour
calibration (the entry already knows `CFAPlaneColor` is `[0, 1, 2]` there):
1. `MalformedOpcodeParams { id: 1, .. }` for `N` not 1 or 3 (checked whether or not the
   sets differ);
2. `UnsupportedWarpTangentialTerms { kt0, kt1 }` of the first set, in plane order, with a
   non-zero tangential term, when the sets differ (onyx rule 4). With identical sets the
   warp is the consumer's and nothing is refused, as today;
3. `UnsupportedWarpFold { slope_min }`.
So the full order on the six entries is: required processing and malformed tags, the
buffers and geometry (the rows forms' `InvalidRowRange` where it is today), the levels,
the `GainMap`s, `OpcodeList1`, the CFA, **the per-plane warp**, the colour calibration,
then (mix) `MixRowOutOfRange`. `InvalidMixWeights` stays first on the mix entries. Nothing
is written before every check passes. No held file reaches any of these refusals.

### 6. What does not change (`DEC-087` item 6)

- `DevelopPath::Colour` still refuses a mandatory `WarpRectilinear` (`{ list: 3, id: 1 }`);
  `DevelopPath::MonoMix` keeps DEC-086 item 6 (applies `N = 1`, refuses a mandatory
  `N ≠ 1`, skips an optional one). Both are SPEC-100's.
  **Transitional gap until SPEC-100:** on a file with an *optional* `N = 3` warp whose sets
  differ (none held), `mono_mix_plane_into` is CA-corrected while the developed mix skips
  the warp, so DEC-084 rule 1's parity does not hold there.
- The mono develop, `parse_warp_rectilinear`, `apply_warp_into` and the stage primitives
  (`demosaic_into`, `to_srgb_linear_into`) are unchanged.
- **Versions.** `DEVELOP_VERSION` (`"8"`) and `MONO_MIX_VERSION` (`"1"`) do not move. The
  sensor-geometry planes change only for files with an `N = 3` warp whose sets differ.
  Every held one is mandatory, and its SPEC-098 planes are unreleased (`[Unreleased]`,
  0.2.7 shipped before SPEC-098), so no released output changes. A file with an
  *optional* differing `N = 3` warp would change; none is held or known, and the
  CHANGELOG says so.
- No thread forms exist for these entries; nothing to keep bit-identical across threads.

### 7. Cost

The design probe's single-threaded table CA (both planes, a full 60 MP plane) took
0.49–0.64 s, against about 1.1 s for `srgb_linear_plane_into` today on the Q3 and SL3
(DEC-086). The build measures the entries before and after on the Q3 and records it
(not a gate).

## Implementation Context

*Probed 2026-11-28 against the real files and `dng_validate` 1.7.1 (stage 3 regenerated
for this design). The probe is a throwaway crate on the public API, in the PROJ-006
session notes (`spec-099-design-probe-*.rs.txt`, `spec-099-design-patch-warp.py.txt`,
`spec-099-design-probe-output.txt`). Nothing was committed from it.*

**Geometry.** `ActiveArea` origin `(0, 0)` on all three; Q2 plane 8424×5632 with
`ActiveArea` 8392×5632 (32 masked columns on the right), crop origin (12, 24) size
8368×5584; Q3 and SL3 plane = `ActiveArea` 9536×6344, crop (4, 4) 9520×6336.
`CFAPlaneColor` `[0, 1, 2]` on all three (Q2 `GBRG`, Q3/SL3 `RGGB`). Every `OpcodeList3`
is one `WarpRectilinear`, flags 0, `N = 3`, `cx = cy = 0.5`, all `kt = 0`. `m` = 5052.649
(Q2), 5726.034 (Q3, SL3).

**The sets** (`kr0, kr1, kr2, kr3`; plane 0 red, 1 green, 2 blue):

| File | Plane | kr0 | kr1 | kr2 | kr3 | `f(1)` | corner px |
|---|---|---|---|---|---|---|---|
| Q2 | R | 0.9996216308165431 | -0.04787844936561948 | -0.09821650922200259 | 0.05547169955651253 | 0.908998 | 459.80 |
| Q2 | G | 0.999251106 | -0.047206985980232964 | -0.0985599100713025 | 0.055324309073773385 | 0.908809 | 460.76 |
| Q2 | B | 0.9991769974193434 | -0.04597953970427109 | -0.09942464206440757 | 0.055257758618096245 | 0.909031 | 459.64 |
| Q3 | R | 0.9996347155524453 | -0.05455320828191769 | -0.08112921079548925 | 0.04461464522856981 | 0.908567 | 523.55 |
| Q3 | G | 0.999265877659388 | -0.054056477178819784 | -0.08104570564304178 | 0.0441974160013766 | 0.908361 | 524.73 |
| Q3 | B | 0.999240053568403 | -0.05345818671371268 | -0.08040421688304702 | 0.04313300087098262 | 0.908511 | 523.87 |
| SL3 | R | 0.9947909946452878 | 0.02081664155561191 | -0.024833699323286502 | 0.001712725185655068 | 0.992487 | 43.02 |
| SL3 | G | 0.9944783937976746 | 0.021191865020737728 | -0.024949742830202244 | 0.0016391091050309678 | 0.992360 | 43.75 |
| SL3 | B | 0.9945095289729575 | 0.021647657199074587 | -0.02497465736219387 | 0.0014086172248721613 | 0.992591 | 42.42 |

`parse_warp_rectilinear` returns the R row on each file (it reads `K1`).

**The CA rule on these files:**
- Fold check: `q` minimum on `[0, 1]` (sampled at 100,001 points) 0.7516 (Q2 green),
  0.7413 (Q3), 0.9448 (SL3); all sets are monotone. So no held file refuses.
- Largest CA displacement at sensor geometry inside green's image: Q2 R 0.959 / B 1.454 px,
  Q3 R 1.178 / B 1.660 px, SL3 R 0.727 / B 1.375 px; beyond the rim (ratio held): at most
  1.056 / 1.235, 1.297 / 0.943, 0.733 / 1.336 px.
- Table against the exact inverse (bisection per pixel, every 37th row and 41st column):
  at most 2.5e-5 px (Q2), 4.8e-4 px (Q3), 7.0e-8 px (SL3) of source displacement.
- The bound `B`: 1.4543 (Q2), 1.6601 (Q3), 1.3754 (SL3), so `H = 3` on all three.
- With the CA step, 47,177,759 of the Q2's 47,443,968 pixels change, 60,374,175 of the
  Q3's 60,496,384, and 60,453,184 of the SL3's.

**The exactness route (AC3, AC7).** On all three files the fused `srgb_linear_plane_into`
(no CA, `OpcodeList1` and `OpcodeList3` removed from the `Sensor`) equals
`to_srgb_linear_into(demosaic_into(n))` at **every** sample, where `n` is the test's own
normalisation `n = min(⌊max(raw − 512, 0)·65535 / (16383 − 512) + 0.5⌋, 65535)` in `f64`
(scalar black 512, `BlackLevelRepeatDim` 2×2 all 512, white 16383 on all three). So a test
can build the expected plane from public stage primitives and a CA reference written from
DEC-087's text.

**The pre-registered digests** (FNV-1a 64 over the `u16` samples, little-endian, as
`tests/leica_colour.rs` computes them). The probe's no-CA pipeline reproduces SPEC-098's
pinned `srgb_linear_plane_into` digests exactly (Q2 `0x98af7675eef43b27`, Q3
`0x552573632a3fa8df`, SL3 `0x0aa0de9063b2509a`, the SL3 with DEC-086's fix applied:
10,615 sites). With DEC-087's rule applied in the probe:

| File | `srgb_linear_plane_into` after SPEC-099 |
|---|---|
| Q2 | `0xd352bb75bf29d270` |
| Q3 | `0xc6f67381a8b94593` |
| SL3 | `0xbb9908b919ce2b08` |

The `f32` and mono digests cannot be predicted from public entries (the unclamped matrix
sum and the mixer pixel are private). The build re-pins them and records that the `u16`
pins matched first.

**The oracle against `dng_validate -3`** (camera RGB, the `ActiveArea`). Three outputs per
file: SDK stage 3 of the real file (`full`), SDK stage 3 of a scratch copy whose red and
blue sets are overwritten by green's (`green`), and ours, CA by the probe's rule then
`apply_warp_into` with green's set (`CA+green`), and without the CA step (`green-only`).
The misregistration measure is the radial component of the Lucas–Kanade shift (6
iterations, bilinear, means removed) between 160×160 patches at eight positions: the four
at `(±0.85, ±0.85)` of the half-extent, `(±0.92, 0)` and `(0, ±0.9)`. The baseline is
ours green-only against SDK green (same warp, so only demosaic and kernel differ).

| File | Plane | max \|CA+green − baseline\| | min \|green-only − baseline\| |
|---|---|---|---|
| Q2 | R | 0.042 px | 0.592 px |
| Q2 | B | 0.022 px | 0.480 px |
| Q3 | R | 0.020 px | 0.690 px |
| Q3 | B | 0.015 px | 0.442 px |
| SL3 | R | 0.032 px | 0.597 px |
| SL3 | B | 0.036 px | 0.406 px |

(Per-patch values in the probe output.) Green's residual equals the baseline at every
patch, by construction. In linear sRGB the same measure is unstable (the clamp and the
matrix mix the planes: up to 1.5 px noise on Q2 blue), so the oracle stays in camera RGB.
One resample per plane with its own set (the DNG order) registers within 0.02 px of the
baseline too; SPEC-100 chooses between them for the colour develop.

## Questions for design — answered

- **The relative warp.** Plane `c` at green's geometry is plane `c` sampled at
  `c + (f_c/f_g)(s)·(x − c)` with `s = g⁻¹`, tabulated over `σ = ρ²` at 4096 intervals,
  nodes solved by bisection to the last bit (§3). Tolerance: at most 4.8e-4 px against
  the exact inverse on the held files. A non-monotone green (`q ≤ 0` on `[0, 1]`) has no
  inverse and is refused with the new `UnsupportedWarpFold` (closed form, no sampling).
- **Identical sets.** Coefficients compared with `==`; a plane whose set equals green's is
  not resampled, and with both equal (or `N = 1`) the traversal is today's, untouched, so
  output is bit-identical rather than resampled through a numerical identity (§2, AC2).
- **The row-range forms.** They develop `H` extra rows each side (`H = 3` on the held
  files) and stay bit-identical to the whole plane; output is no longer row-local in the
  strict sense (a row reads rows `y − H..y + H`), but any tiling still equals the plane
  (§4, AC5).
- **The colour develop, the developed mix, `warp_alpha` and `Framing`, the threaded
  forms, optional warps on the developed outputs:** SPEC-100 (split). The measurements it
  needs are recorded above and in its outline.
- **The accessor.** `sensor_geometry_warp(&Sensor)`, §1: the consumer's warp (green's set
  for `N = 3` on a CFA file, the one set for `N = 1`, `K1` for a monochrome file) plus
  every set with its `CFAPlaneColor` colour; `parse_warp_rectilinear` unchanged.
- **Output change at sensor geometry.** The Q2, Q3 and SL3 planes change from SPEC-098's,
  which are unreleased; no version moves (§6).
- **The oracle.** `dng_validate -3` in camera RGB with the misregistration bound above
  (AC8), plus exact tier-A fixtures for the rule and its identity cases (AC2, AC3).
- **Optional warps at sensor geometry.** The CA step runs whatever the flags, as the
  sensor-geometry path accepts the first warp whatever its flags; no held file has an
  optional `N = 3` warp (§6).
- **Spec size.** Split: this spec is M; SPEC-100 is framed.

## Acceptance Criteria

- **AC1. The accessor (tier A).** On `Sensor`s built with `tests/support/tiff.rs`
  (`dng_16bit_with_opcode_lists`) and read through `Container::parse`:
  no list 3, and a list 3 holding no `WarpRectilinear` (one unknown optional opcode) → `Ok(None)`;
  CFA `N = 3` (three distinct sets) → `sets()` in file order with colours
  `[Some(0), Some(1), Some(2)]`, each `warp` equal to that set (`planes == 3`),
  `consumer()` equal to set 1, `is_per_plane()` true; the same with three identical sets →
  `is_per_plane()` false; CFA `N = 1` → one set, `color: None`, `consumer()` that set;
  monochrome `N = 1` and `N = 3` → `color: None` throughout and `consumer()` equal to
  `parse_warp_rectilinear`'s `K1`; `N = 2` and `N = 4` → `MalformedOpcodeParams { id: 1, .. }`;
  `CFAPlaneColor` `[0, 1, 1]` with `N = 3` → `UnsupportedCfaPlaneColor`; optional flags →
  the same answer; a second `WarpRectilinear` is ignored; non-zero `kt` is reported, not
  refused. `parse_warp_rectilinear` still returns `K1` (red's set on a CFA file).
- **AC2. Identity cases are bit-identical (tier A).** On a 48×40 `RGGB` scene (black 0,
  white 65535, a colour calibration as in `tests/colour_bad_pixels.rs`) the six entries
  give the same output for: no list 3; `N = 1` (a non-identity set); `N = 3` with three
  copies of one set; that list with flags 1 (optional); `N = 3` identical sets with a
  non-zero `kt`. All equal SPEC-098's output (the no-list-3 develop).
- **AC3. The rule, exactly (tier A).** On the same scene with `N = 3` sets that differ
  (green non-identity, red and blue each different, CA of several pixels at the corner so
  every sub-rule is reached), and in two variants — `ActiveArea` `(2, 2, 38, 46)` set on
  the parsed `Sensor`, and `cx = 0.3, cy = 0.6` — `srgb_linear_plane_into` equals
  `to_srgb_linear_into(reference(demosaic_into(src)))` at every sample, where `demosaic_into`
  and `to_srgb_linear_into` run on the `Sensor` with list 3 removed and `reference` is the
  test's own implementation of DEC-087 item 3 (table, bisection, bilinear, held rim),
  written from the decision's text, not by calling crate code. Pixels outside the
  `ActiveArea` equal the no-CA output; the output differs from the no-CA output. A case
  where only red differs leaves blue unresampled (equal to the reference with blue skipped).
- **AC4. All three stores see the corrected triple (unit test, `src/color.rs`).** On AC3's
  scene, `mono_mix_plane_into` equals `mono_mix_pixel(row, t)` and
  `srgb_linear_plane_f32_into` equals `store_f32_unclamped(M, t)` for the reference triple
  `t` of each pixel.
- **AC5. Bands (tier A).** On AC3's scene and its `ActiveArea` variant, bands of
  `srgb_linear_plane_rows_into`, `srgb_linear_plane_rows_f32_into` and
  `mono_mix_plane_rows_into` that tile the plane — `0..3, 3..11, 11..40`, every single row,
  and `0..40` — equal the whole plane.
- **AC6. Refusals and order (tier A).** With differing sets: a non-zero `kt` on blue →
  `UnsupportedWarpTangentialTerms` with blue's `kt`; green `kr = [1, -2, 0, 0]` (folds:
  `q(1) = -5`) → `UnsupportedWarpFold`, `slope_min` equal to `scale_bounds`'s minimum of
  `[1, -6, 0, 0]` (−5); `N = 2` → `MalformedOpcodeParams { id: 1, .. }` (also with
  identical sets). Order: an unsupported CFA beats the warp refusal; the warp refusal
  beats `MissingColorCalibration`; `InvalidMixWeights` beats it on the mix entries; nothing
  is written on refusal (sentinel-filled `dst`). An `ActiveArea` of 1×1 runs no CA step
  (`m = 0`). The developed outputs are unchanged on these lists (`develop_color_into` still
  `{ list: 3, id: 1 }` mandatory; the developed mix as DEC-086 item 6).
  `ErrorKind` of `UnsupportedWarpFold` is `Unsupported`.
- **AC7. Leica files (tier B, skips loudly by name).** For each present file:
  `sensor_geometry_warp` gives the table's three sets, colours R, G, B, `consumer()` equal
  to the G row, `is_per_plane()` true; `srgb_linear_plane_into`'s digest equals the
  pre-registered one above; the `f32` and mono-mix digests differ from SPEC-098's pins and
  are re-pinned; the `_rows_` forms over rows 100..200 (and 0..5, and the last 5 rows)
  equal the plane; the plain mono `develop_into` pin is unchanged; the developed outputs
  are still `UnsupportedRequiredOpcode { list: 3, id: 1 }`. On the Q2 and Q3 (no bad
  photosite), `srgb_linear_plane_into` equals `to_srgb_linear_into(reference(demosaic_into(n)))`
  at every sample, both primitives on the `Sensor` with lists 1 and 3 removed, `n` as in
  Implementation Context.
- **AC8. The `dng_validate -3` bound (tier B plus `$IRRADIANCE_SDK_STAGE3_DIR`, local
  only; a SKIP line when unset).** Pre-registered: for each file, red and blue, at each of
  the eight patches, `|shift(CA+green, SDK full) − shift(green-only, SDK green)| ≤ 0.10 px`,
  where ours is `reference(demosaic_into(n))` (tied to the entry by AC7) then
  `apply_warp_into` with `consumer()`. **Red-proof:** with the CA step left out (green-only
  against SDK full) the same difference is `≥ 0.30 px` at every red and blue patch; the
  test asserts both, so a reference that skipped the CA step fails it.
- **AC9. Nothing else moves.** Every existing plane pin, determinism hash and tier-B pin
  of every other file, the Pixels' `N = 1` optional-warp outputs included;
  `DEVELOP_VERSION` `"8"`, `MONO_MIX_VERSION` `"1"`. `cargo semver-checks`: minor
  (additions only).
- **AC10. Fuzz.** `warp_opcode` calls the per-plane parser through `__fuzz_warp_planes`
  for colour assignments `None`, `[0, 1, 2]` and `[2, 1, 0]`: never panics; `Ok(Some)`
  implies `N ∈ {1, 3}`, `consumer()` is the set coloured 1 on a CFA with `N = 3` and
  otherwise `K1`, and `K1` equals `parse_warp_rectilinear`'s; `N ∉ {1, 3}` gives
  `MalformedOpcodeParams`. A seed with `N = 3` distinct sets is added. Smoke length, zero
  crashes.

## Failing Tests

Written first; each seen to fail (or, tier B, to refuse or mismatch) against `main`.

- **`tests/plane_warp.rs`** (new, tier A):
  `ac1_sensor_geometry_warp_names_the_consumer_warp_and_every_set`,
  `ac2_no_per_plane_difference_is_bit_identical`, `ac3_red_and_blue_are_resampled_to_green_geometry`,
  `ac3_the_active_area_and_the_optical_centre`, `ac5_bands_equal_the_whole_plane`,
  `ac6_refusals_and_their_order`. Red on `main`: no accessor; the planes pass the warp
  through (AC3 differs from the reference); `UnsupportedWarpFold` does not exist.
  (AC2 passes on `main` except the `N = 2` half of AC6; it is the guard.)
- **`src/warp.rs` unit tests:** the table and bound on a hand-checkable set (green
  identity, red `kr0 = 1.02`: every node is exactly `1.02`, `B = 0.02·m`), the bisection
  on a known inverse, the fold boundary (`q` min exactly 0 refuses).
- **`src/color.rs` unit test:** `ac4_every_store_sees_the_corrected_triple`.
- **`tests/leica_colour.rs`** (extended, tier B): `ac7_the_opcode_files_are_ca_corrected`
  (replaces the sensor-geometry half of SPEC-098's AC7; the developed-refusal half stays).
- **`tests/colour_sdk_stage3.rs`** (new): `ac8_red_and_blue_register_to_dng_validate_stage_3`.
- **`tests/error_kind.rs`:** the new variant's kind.

**Rule-clause map** (each mutant must turn a test red):

| Clause (DEC-087) | Test | Mutant it catches |
|---|---|---|
| 1 consumer = green's set | ac1, ac7 | consumer = `K1` (red) |
| 1 `color` from `CFAPlaneColor` | ac1 | colours by index |
| 2 equal sets skip, per plane | ac2, unit | resample an equal-set plane anyway: output-equivalent (the ratio is exactly 1 and the source an integer pixel), so a `src/color.rs` unit test asserts no ring is built when the sets are equal |
| 3.1 `hypot` and `ActiveArea` coordinates | ac3 variants | plane coordinates; `sqrt(mx²+my²)` |
| 3.2 fold refusal | ac6 | refuse at `q < 0` only, or never |
| 3.3 held rim | ac3 (corner beyond the rim reached) | extrapolate the polynomial past `s = 1` |
| 3.4 interpolation, `min(u, 4096)` | ac3, unit | nearest node; no clamp at `σ = 1` |
| 3.5 clamp to the `ActiveArea`, half up | ac3 `ActiveArea` variant | clamp to the plane; truncate |
| 3.6 halo `H` | ac5 single-row bands | `H = 0`, `H = ⌈B⌉ − 1` |
| 4 masked border passes through | ac3 `ActiveArea` variant | resample the border |
| 5 order | ac6 | warp checked before the CFA or after the calibration |

## Non-Goals

- The developed colour output, the developed mix, `warp_alpha` and `Framing` on them,
  their threaded forms and their optional-warp rules: SPEC-100.
- Green's lens-distortion warp at sensor geometry: the consumer applies it (onyx's rule 1,
  as amended).
- Tangential warp terms when the CA step runs (`UnsupportedWarpTangentialTerms` stays).
- A per-plane `N` other than 1 or 3, or plane colours other than R, G, B.
- `FixBadPixelsConstant` (SPEC-098), highlight recovery above `WhiteLevel` (onyx declined
  it, 2026-11-28) and filter presets.
- Committing or redistributing any `-PB` file, any SDK output, or a CI test that needs one.

## Open questions

None blocks the build.
- **For onyx (FYI, no answer needed to build):** beyond green's image (`ρ > f_g(1)`, seen
  only at `warp_alpha < 1`) DEC-087 holds the CA ratio at its rim value, because the DNG
  polynomial is undefined there. The alternative is extrapolating the polynomial (smooth
  on all three held files). onyx may prefer one; the difference is under 0.2 px at the
  sensor corner on the held files.
- **For the maintainer:** `N = 2` and `N ≥ 4` on a colour file become
  `MalformedOpcodeParams` on the sensor-geometry entries (today they pass through). No
  held file has one. Accept, or keep passing them through.

## Notes for the Implementer

- Take the pre-registered digests and the oracle bound from Implementation Context; never
  edit them to match your build. If AC7's `u16` digest misses, compare your rule with
  DEC-087 item 3 term by term (`hypot`, the operation order of `f` and `h`, the bisection
  stop rule, `min(u, 4096)`) before anything else.
- Resolve the per-plane warp once per call into a small value (two tables, `H`, the
  geometry); never per pixel.
- AC3's reference must be the test's own code from the decision's text.
- The SDK outputs and the patched DNG copies live in `$IRRADIANCE_SDK_STAGE3_DIR`, outside
  the repo. Never copy any `-PB` file, SDK output or derivative pixel into the repo, a
  fixture, a seed or CI.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Write the order mutants first: for every refusal in a "Refusals and order" item,
   one mutant that moves it a step earlier, and for every comparison one that drops a
   term. The rule-clause map covered every sub-rule of item 3 but not item 5's order or
   the tangential comparison, and that is where four of the verify's five gaps were. And
   a float mutant whose test depends on one libm's rounding (the `hypot` one) needs a
   sweep that shows the two forms differ on the CI host too.

2. **Does any template, constraint, or decision need updating?**
   — DEC-087 is promoted to accepted at this ship, with the verify's clarification in
   items 1 and 5 (a non-finite coefficient in set 2 or 3 is malformed, FU-7) and a note
   in item 4 that the ring holds one spare row (FU-9). No template change now. Signal
   `token-estimate-undershoots-corpus-probing-specs` takes this spec's data point: the
   100M estimate counted all four cycles and the spec came in on it, which is the
   signal's falsifier holding, not a third undershoot. The verify's platform-dependent
   float mutant is one instance and is recorded under FU-5, not as a signal.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. SPEC-100 (the developed colour output and the developed mix,
   `depends_on: [SPEC-099]`) is already framed and is next. The optional speed-up of the
   CA step (FU-8) is recorded below, not specced.

4. **Where was the worst defect caught?**
   — verify

5. **What can a user do now that they couldn't before?**
   — Before, the Leica Q2, Q3 and SL3 planes left the whole three-plane warp to the
   consumer, which had only `parse_warp_rectilinear` (red's set) and one warp for all
   three planes, so red and blue landed 0.41 to 2.2 px off Adobe's `dng_validate`.
   Now `srgb_linear_plane_*` and `mono_mix_plane_*` resample red and blue to green's
   geometry, so the consumer applies one warp, named by `opcode::sensor_geometry_warp`,
   and lands within 0.042 px of the SDK's green-only baseline on all three files, matching
   the three pre-registered digests; no other file changes output (16 corpus DNGs compared
   on three entries). The developed outputs still refuse until SPEC-100.

## Follow-ups

Raised in HANDOFF-210 (build) and HANDOFF-211 (verify). The verify numbered FU-1 to FU-9;
FU-10 and FU-11 are numbered here from the design's open questions and the ship brief. The
verify's tests are in PR 374, merged into PR 373 and squashed into `6315741`.

| id | finding | disposition |
|---|---|---|
| `FU-1` | V1, V2, V4: no fixture had sets differing in `kt` alone, a `kt` on green, or a `kt` with a folding green | `fixed` — verify, `verify_tangential_terms_count_in_the_comparison_and_come_before_the_fold` |
| `FU-2` | V3 to V3d: nothing separated the warp check from the levels, the `GainMap`s and `OpcodeList1` | `fixed` — verify, `verify_the_levels_gain_maps_and_list_1_come_before_the_warp` |
| `FU-3` | V10: the halo bound's radius `√(j/T)` instead of `√((j+1)/T)` survived | `fixed` — verify, `ca_table_and_bound_on_a_hand_checkable_set` extended (pins `H = 8`) |
| `FU-4` | V13: the build's non-finite later-set check had no test | `fixed` — verify, `verify_a_non_finite_later_set_is_malformed` |
| `FU-5` | The build's `ca_m_is_hypot` caught the sqrt mutant only under macOS's libm rounding, not on CI's Linux | `fixed` — verify, `ca_m_is_hypot_across_geometries` (144 geometries) |
| `FU-6` | The CHANGELOG did not name the non-finite second or third set refusal | `fixed` — verify, `CHANGELOG.md` [Unreleased] |
| `FU-7` | DEC-087 was silent on the non-finite later-set refusal (the build's accepted deviation) | `fixed` — at ship, DEC-087 items 1 and 5 |
| `FU-8` | The CA step takes Q3's `srgb_linear_plane_into` from 0.96 s to 2.27 s; perhaps 0.3 to 0.5 s is recoverable without changing output (sharing `σ` between red and blue, a bilinear over row slices without the per-read clamp) | `closed` — optional and not a gate: the verify measured it as mostly inherent, no consumer has asked, and a reciprocal of `m` would change the pins. Recorded here only; a consumer's latency report would be a new spec |
| `FU-9` | DEC-087 item 4's ring of `2H + 2` rows has one spare row; `2H + 1` suffices (V16 equivalent) | `closed` — harmless, unchanged; noted in DEC-087 item 4 at ship |
| `FU-10` | Design, for onyx: beyond green's image the ratio is held at its rim value rather than extrapolated (under 0.2 px at the corner on the held files) | `closed` — an FYI with no answer needed; it goes in the maintainer's next message to onyx (recorded in the PROJ-006 brief) |
| `FU-11` | Design, for the maintainer: `N = 2` and `N ≥ 4` become `MalformedOpcodeParams` on the sensor-geometry entries | `closed` — settled by the maintainer before the verify (HANDOFF-211); in the CHANGELOG |
