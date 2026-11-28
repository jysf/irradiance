---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-100
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  decisions: [DEC-088, DEC-087, DEC-086, DEC-084, DEC-059, DEC-044, DEC-043, DEC-024]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-050, SPEC-051, SPEC-064, SPEC-093, SPEC-098, SPEC-099]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-099]             # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: the Leica Q2, Q3 and SL3 colour DNGs (a mandatory three-plane WarpRectilinear) give a colour develop and a developed mix instead of refusing"

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
  tokens_estimate: 125000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 17904935
      estimated_usd: 14.81
      duration_minutes: 20
      recorded_at: 2026-11-28
      notes: "Design in a side session started by the PROJ-006 orchestrator: reading, the probe (the colour develop in both matrix orders, the developed mix under five options, dng_validate -3 in camera RGB and through the entry), DEC-088 and its amendment lines, this design, HANDOFF-212. A FLOOR measured just before the commit over transcript c67dc3fa-932b-4172-a1a4-12fdfbbaa17a.jsonl (68 unique message.ids, all claude-opus-5-5, no subagents): input 136 + cache-creation 336,137 (all 1-hour) + cache-read 17,460,050 + output 108,612, priced at 5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x. Duration is the transcript's first-to-last timestamp. Re-sum at ship; counted here, so the STAGE-034 orchestration cost must not count this transcript again."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-100: The per-plane warp on the developed colour output and the developed mix

> **Designed 2026-11-28 (DEC-088).** Not split. Sized **L**: two developed outputs, one
> rule, one oracle. A split (colour develop, then the mix) would add about two cycles of
> fixed cost and duplicate the shared work (the CA step's tables leave the heap, one DEC,
> one probe), so it is cheaper whole. Last spec of STAGE-034.

## Context

Fourth spec of STAGE-034 (PROJ-006), split out of SPEC-099 at its design (2026-11-28,
DEC-087). SPEC-099 corrects lateral chromatic aberration (CA) at sensor geometry: on a
per-plane `WarpRectilinear` (`N = 3`, sets that differ) the sensor-geometry planes
(`srgb_linear_plane_*`, `mono_mix_plane_*` and their rows forms) resample red and blue to
green's geometry, and `opcode::sensor_geometry_warp(..).consumer()` names the one warp left
for the consumer, green's set. Call that warp `W`. The developed outputs still refuse the
held Q2 (`LEICA-Q2/L1000750.DNG`, CC0), Q3 and SL3 (`LEICA-Q3-PB/leica_q3_01.dng`,
`LEICA-SL3-PB/leica_sl3_01.dng`, local only, never committed):

- `develop_color_into`, `develop_color_f32_into` and the threaded forms refuse with
  `UnsupportedRequiredOpcode { list: 3, id: 1 }`: `DevelopPath::Colour` applies no warp.
- The developed mix (`develop_with_into` with `mono_mix`) refuses a mandatory first
  `WarpRectilinear` whose `N` is not 1, and skips an optional one (DEC-086 item 6).

**Binding from onyx (2026-11-28, relayed by the orchestrator; onyx's DEC-114):**
- **The developed mix:** the CA step at sensor geometry (DEC-087), then the mix, then one
  warp `W` on the mixed plane at `warp_alpha` and `Framing`. That is onyx's viewport, so
  export equals viewport (DEC-084 rule 1) and onyx's warp parity check carries over.
- **`warp_alpha` dials back lens distortion only.** Chromatic aberration is always
  corrected.
- **Unchanged output:** `N = 1`, identical sets, and every file that develops today stay
  bit-identical. `DEVELOP_VERSION` and `MONO_MIX_VERSION` move only if a file that develops
  today changes.

The colour develop was left to this design: the CA step then `W` (two resamples), or each
plane's own set in one resample; where the warp sits relative to the matrix; whether it
gains `warp_alpha` and `Framing`; whether optional warps stay skipped.

## Goal

The held Q2, Q3 and SL3 files give a colour develop and a developed mix with no refusal.
The colour develop is the CA-corrected camera triple warped by `W`, then the colour matrix,
then the crop and orientation (DNG's stage order). The developed mix is the mono develop of
SPEC-099's CA-corrected mixed plane with `W` at `warp_alpha` and `Framing`, bit for bit
(DEC-084 rule 1). Every file with no per-plane difference, and every file that develops
today, keeps its output bit for bit.

## Inputs

- **Files to read:** `src/color.rs` (`develop_color_into`, `develop_color_f32_into`,
  `develop_color_with`, `develop_color_into_threaded`, `colour_band_plan`, `CropScatter`,
  `ActiveWindow`, `developed_mix_strips`, `mix_strips_with`, `resolve_chromatic_shift`,
  `CaDelay`, `srgb_linear_plane_rows_shifted`); `src/warp.rs` (`source_coord`,
  `bilinear_sample_with`, `apply_warp_into`, `ChromaticShift`); `src/develop.rs`
  (`develop_core`, `develop_warp`, `non_identity_warp`, `scratch_len`,
  `mix_needed_scratch`, `run_mix_passes`, `crop_source_coords`, `for_each_out_pixel`);
  `src/opcode.rs` (`DevelopPath`, `Applied`, `first_warp_planes`, `describe_warp_planes`);
  `src/demosaic.rs` (`mix_strip_work_len`, `develop_rows_banded_in`); DEC-088, DEC-087,
  DEC-086, DEC-084, DEC-059, DEC-044.
- **Tests to read:** `tests/plane_warp.rs` and `tests/support/ca_reference.rs` (the CA
  reference and the warp-list builders), `tests/developed_mix.rs` (the rule-1 oracle and
  the ranked refusals), `tests/develop_scratch_alloc.rs` (the mix's allocation counts),
  `tests/leica_colour.rs`, `tests/colour_sdk_stage3.rs`, `tests/threaded.rs`.
- **External:** `dng_validate -3` outputs from `scripts/sdk-stage3.sh` (unchanged).
- **Corpus:** the three files (tier B; the Q3 and SL3 local only).

## Outputs

- **`src/color.rs`:** the warped colour body (§2), the colour entries' and threaded
  forms' docs and errors, the developed-mix strips with the CA ring (§3).
- **`src/warp.rs`:** `ChromaticShift`'s tables in fixed-size arrays (no heap, output
  unchanged); a crate-private three-channel gather that uses `source_coord` and
  `bilinear_sample_with` (§2).
- **`src/develop.rs`:** `develop_warp` reads `W` (no single-plane filter);
  `develop_core` resolves the CA step for the mix and checks it in rank (§5);
  `scratch_len` and `mix_needed_scratch` add `T_H` and `R` (§4); `run_mix_passes` lays
  out the ring; `DevelopOptions::mono_mix`, `develop_with_into` and `scratch_len` docs.
- **`src/demosaic.rs`:** a strip-work length for a band of `S + 2H` rows.
- **`src/opcode.rs`:** `DevelopPath::Colour` applies the first `WarpRectilinear` in list 3
  (`Applied::First`); `DevelopPath::MonoMix` likewise (`Applied::FirstSinglePlane` goes if
  unused); both variants' docs.
- **Tests:** `tests/colour_warp.rs` (new, tier A), `tests/developed_mix.rs` (extended),
  `tests/develop_scratch_alloc.rs` (extended), `tests/leica_colour.rs` (extended, tier B),
  `tests/colour_sdk_stage3.rs` (extended), `tests/threaded.rs` (extended),
  `tests/required_processing.rs` and `src/opcode.rs` unit tests (the two paths' list 3).
- **Docs:** `CHANGELOG.md` `[Unreleased]`; `docs/provenance-ledger.md` row; DEC-088
  (proposed here), with the amendment lines already in DEC-084, DEC-086 and DEC-087.

## Design

`W` is `sensor_geometry_warp(sensor)?.consumer()`. "The CA step" is DEC-087 items 2 to 4,
unchanged: it runs when `N = 3` and red's or blue's set differs from green's, whatever the
flags and `warp_alpha`.

### 1. One rule (`DEC-088` item 1)

Every developed output is: the CA step at sensor geometry, then `W` over the `ActiveArea`
as `apply_warp_into` applies it, then the framing and the orientation. `warp_alpha` lerps
`W` only. The developed mix puts its mix before `W` (onyx's binding); the colour develop
puts its matrix after `W` (§2). So for both, red and blue are where green is before any
lens correction, and one warp, the viewport's, does the rest.

### 2. The colour develop (`DEC-088` item 2)

**When.** Only when the first `WarpRectilinear` in `OpcodeList3` is **mandatory** (flags
bit 0 clear). With an optional one, or none, the entry is today's code path, untouched
(no CA step, no `N` check): every file that develops today keeps its bytes. No options:
`warp_alpha = 1`, the `DefaultCrop`, the orientation applied.

**The rule, exactly.** With `n` the colour path's normalised mosaic (bad pixels fixed, the
levels, the `GainMap`s), `cam = demosaic(n)` over the plane, `ca` the CA step on `cam`
(identity when it does not run), and `plane_c` the `ActiveArea` of channel `c` of `ca`:
- output pixel `(ox, oy)` maps to crop pixel `crop_source_coords(orientation, ox, oy, ..)`
  and so to `ActiveArea` pixel `(x, y) = crop_origin + crop`;
- its camera triple is `t_c = apply_warp_into(W, aw, ah, plane_c)[y·aw + x]` for
  `c = 0, 1, 2` (identity `W`: `t_c = plane_c[y·aw + x]`);
- it is written by the entry's store: `store_u16(M, t)` for `develop_color_into`,
  `store_f32_quantised(M, t)` for the `f32` entry.

So `develop_color_into` equals the crop and orientation of
`to_srgb_linear_into(stripped, warp_planes(ca))`, where `warp_planes` replaces each
channel's `ActiveArea` with `apply_warp_into(W, ..)` of it and `stripped` is the `Sensor`
with lists 1 and 3 removed (AC3 and AC7 test exactly this), and the `f32` entry stays
exactly `to_linear_f32_into` of the `u16` one.

**Where (implementation, not contract).** One crate-private body:
1. all checks (§5) and the matrix build, before any write;
2. the `width·height` green plane and an `ActiveArea` camera buffer of `3·aw·ah` `u16`
   (interleaved), both allocated (this entry already allocates its green plane);
3. the fused traversal (`develop_color_fused`), its sink through the CA delay ring when
   the step runs (`CaDelay`, as `srgb_linear_plane_shifted` uses it) and an
   `ActiveWindow` slot into the camera buffer; the green plane is dropped after;
4. the gather: for each output pixel, in `for_each_out_pixel`'s tile order, the
   `ActiveArea` position as above, `source_coord(W, aw, ah, x, y)` **once**, then for each
   channel `bilinear_sample_with(aw, ah, sx, sy, |xi, yi| buffer[(yi·aw + xi)·3 + c])`, then
   the store into the output slot. These are the functions `apply_warp_into` calls, so
   every value equals the per-plane composition. Identity `W` copies (as
   `apply_warp_into` does).

Memory: green plane plus `3A` (about 121 + 363 MB on the Q3, against 121 MB today). A ring
with `W`'s vertical reach (about 524 rows on the Q3, about 60 MB) would give the same bytes
and is a follow-up, not this spec.

**Threaded forms.** With an applied warp, `develop_color_into_threaded` and
`develop_color_f32_into_threaded` call the single-thread entry (as for orientations 5 to
8, DEC-044). Same bytes by construction.

**`N = 1`, or identical sets, mandatory** (none held): no CA step; `W` (the one set) warps
all three planes.

### 3. The developed mix (`DEC-088` item 3)

**The rule.** DEC-084 rule 1 with its two inputs named: the developed mix equals the mono
develop of `mono_mix_plane_into(sensor, src, weights)`'s plane at levels `MONO_MIX_BLACK`
and `MONO_MIX_WHITE`, whose `OpcodeList3` is one `N = 1` `WarpRectilinear` holding `W`,
under the same options. `warp_alpha` lerps `W`; the CA step is in the plane and does not
lerp.

**Which warps.** The first `WarpRectilinear` in `OpcodeList3` **whatever its flags**, with
`N` 1 or 3 (`MalformedOpcodeParams { id: 1, .. }` otherwise, as at sensor geometry). This
replaces DEC-086 item 6: mandatory `N = 3` is applied (not refused), optional `N ≠ 1` is
applied (not skipped). `develop_warp` becomes `W` lerped and non-identity, for the mix and
for sizing. The mono develop (`mono_mix: None`) is unchanged: it still applies `K1`
through `parse_warp_rectilinear`.

**Where.** `developed_mix_strips` takes the resolved `Option<ChromaticShift>` and a ring
slice from scratch. Without the CA step it is today's code, untouched. With it, each strip
`[s, e)` of the target rows (the framed rows with no applied warp, the `ActiveArea` rows
with one, as today):
- develops rows `max(s − H, 0)..min(e + H, h)` through `develop_rows_banded_in` on the
  strip work (`T_H`, §4);
- runs a `CaDelay` over the emitted triples that keeps rows `s..e` (a fresh delay state per
  strip, the same ring memory), flushed after the band;
- writes `mono_mix_pixel(row, t')` of each corrected triple `t'` through the target's slot
  (`CropScatter` or `ActiveWindow`).
Every value is `mono_mix_plane_rows_into`'s, which equals the whole plane (DEC-087 item 4),
so any strip height tiles the plane. At `warp_alpha = 0` (`W` the identity) the
CA-corrected mix goes straight to the crop scatter.

**No allocation.** `CaDelay` borrows its ring (`3·(2H + 2)·w` `u16`) instead of owning a
`Vec`; the sensor-geometry entries keep allocating theirs and lend it. `ChromaticShift`'s
two tables (`T + 1 = 4097` `f64` each) and the nodes move to fixed-size arrays, so
resolving the step allocates nothing. Same arithmetic, same order: SPEC-099's pins must not
move (AC10).

### 4. Scratch (`DEC-088` item 4)

With `S = 512` (`MIX_STRIP_ROWS`), `H` the CA halo (0 when the step does not run):
- strip work `T_H = w·(min(h, S + 2H + 7) + 8)` (the band's green rows, halo and even
  alignment, plus the eight-row ring; `T_0` is today's `T`);
- CA ring `R = 3·(2H + 2)·w`, or 0 when the step does not run;
- `scratch_len` = `T_H + R` without an applied warp, `A + max(A, T_H + R)` with one.

Layout with a warp: `A1 [0, A)`, strip work `[A, A + T_H)`, ring `[A + T_H, A + T_H + R)`,
then `A2 [A, 2A)` once the strips are done. Without a warp: work `[0, T_H)`, ring
`[T_H, T_H + R)`. On the held files (`H = 3`) `T_H + R` is about 5.4 M samples against
`A` = 47 to 60 M, so the scratch is `2A`, as for any warped develop.

### 5. Refusals and order (`DEC-088` item 5)

**Colour develop** (all four entries), with a mandatory first warp only, after the CFA
check and before the colour calibration:
1. `MalformedOpcodeParams { id: 1, .. }` for `N` other than 1 or 3, or a non-finite later
   set;
2. with differing sets, `UnsupportedWarpTangentialTerms` for the first set (plane order)
   with a non-zero `kt`;
3. `UnsupportedWarpFold`;
4. `UnsupportedWarpTangentialTerms` with `W`'s `kt` when it is not zero (the applied warp
   is radial only).
The full order is therefore: required processing and malformed tags,
`ColorSourceWrongLength`, the geometry, `RgbBufferWrongLength`, the levels, the
`GainMap`s, `OpcodeList1`, the CFA, the per-plane warp (1 to 4), `MissingColorCalibration`.
Nothing is written before every check passes. The threaded forms return the same errors
(they call the single-thread entry when the warp applies).

**Developed mix:** DEC-084 item 7 with the per-plane warp after the CFA: ... `OpcodeList3`
(the per-plane parse: item 1 above; under `FullActiveArea` the bounds rule on `W` lerped),
the CFA, items 2 and 3 above when the sets differ, `MissingColorCalibration`,
`MixRowOutOfRange`, the scratch length. `W`'s own tangential terms are refused where the
mono develop refuses them (the applier), unchanged.

**`scratch_len` under the mix:** the per-plane parse (item 1); for `N = 3` on a CFA
sensor, `UnsupportedCfaPlaneColor` and `UnsupportedWarpFold` (it needs green's set to know
`H`). No levels, CFA pattern or calibration. On a sensor without a CFA the mix sizes with
`H = 0` and the develop refuses `NotAColorFilterArray`. `output_dimensions_with` is
unchanged.

### 6. Versions (`DEC-088` item 6)

`DEVELOP_VERSION` stays `"8"`, `MONO_MIX_VERSION` stays `"1"`:
- the colour develop changes only for a mandatory `WarpRectilinear`, which every colour
  entry refused until now; relaxing a refusal does not bump (DEC-058 §5). Optional warps are
  skipped as before, so the Pixel 3a and 7 Pro (optional `N = 1`) are unchanged;
- the developed mix changes for a mandatory `N = 3` warp (refused) and an optional
  `N ≠ 1` warp (skipped; none held). The developed mix is unreleased (`SPEC-093` is in
  `[Unreleased]`; 0.2.7 has no `mono_mix`), so no released output moves;
- the mixer's arithmetic does not change.
The CHANGELOG names the optional-`N ≠ 1` case.

### 7. Cost

Single-threaded on the Q3, the probe's composition took 2.4 s for the CA-corrected plane
and 2.7 to 2.9 s for three warps, the matrix and the crop (probe code, not the gather). The
build times `develop_color_into` and the developed mix on the Q3 and records them (not a
gate).

## Implementation Context

*Probed 2026-11-28 against the three files and `dng_validate` 1.7.1 (the stage-3 outputs
and green-only copies from `scripts/sdk-stage3.sh`, generated for SPEC-099). The probe is a
throwaway crate on the public API plus the tests' own `ca_reference.rs` and
`stage2_tiff.rs`, in the PROJ-006 session notes (`spec-100-design-probe.rs.txt`,
`spec-100-design-probe-output.txt`). Nothing was committed from it.*

**Geometry.** Q2: plane 8424×5632, `ActiveArea` 8392×5632 at (0, 0), crop (12, 24)
8368×5584, Orientation 1, output 8368×5584. Q3 and SL3: plane = `ActiveArea` 9536×6344,
crop (4, 4) 9520×6336, **Orientation 8**, output 6336×9520. So the held files exercise the
gather under a transposing orientation. Each file's `W` is the G row of SPEC-099's table;
every first warp is mandatory (flags 0), `N = 3`.

**Exactness.** On all three files `srgb_linear_plane_into` equals
`to_srgb_linear_into(stripped, ca_reference::reference(demosaic_into(stripped, n)))` at
every sample, `n` the test's normalisation (`(raw − 512)·65535/(16383 − 512)`, rounded half
up, clamped) with DEC-086's fix applied by hand on the SL3 (10,615 photosites fixed, as
SPEC-098 counted). So the colour develop's composition (§2) can be built from public
primitives and the tests' reference.

**Pre-registered digests** (FNV-1a 64, `u16` little-endian and `f32` bits little-endian,
as `tests/leica_colour.rs` computes them; weights `LUMA`, Rec. 709 `2126/7152/722` over
10000):

| File | `develop_color_into` | `develop_color_f32_into` |
|---|---|---|
| Q2 | `0x15fde0a3aad3b45e` | `0xbd4bee088e773f2c` |
| Q3 | `0x3a822d1812f19464` | `0x6af7d2cd38f95e42` |
| SL3 | `0x1d8d6442e403cd46` | `0xd5f79bb4f80cb5d0` |

| File | mix default | `warp_alpha` 0.5 | `warp_alpha` 0 | `FullActiveArea` | `apply_orientation: false` |
|---|---|---|---|---|---|
| Q2 | `0x0ade89425f876db3` | `0x617d4140fb3c402b` | `0x6ee6dcabf9bc96b6` | `0xa3cfe6479e1ad080` | `0x0ade89425f876db3` (Orientation 1) |
| Q3 | `0xe92825ab103f946b` | `0x6ca0d15026c5bcd4` | `0xcaa994aef7579659` | `0xc6890e7b9cd4b4e4` | `0xe2b112f936cc588f` |
| SL3 | `0xbf5fe5e4fe7a7c6c` | `0x0bca0992e2b9f84b` | `0xc900805e87b9b166` | `0xd384f0f866b2b422` | `0x4ccae4844a48c7ac` |

The colour digests are the probe's composition (§2: the reference CA, `apply_warp_into`
per camera plane, `to_srgb_linear_into`, crop and orientation); the `f32` ones divide each
`u16` by 65535. The mix digests are `mono_mix_plane_into` (whose pins SPEC-099 set and these
planes reproduce), then `apply_warp_into(W lerped)` over the `ActiveArea`, then the crop
(or the `ActiveArea`) and orientation, which is the mono develop at levels 0 and 65535.
`W` passes DEC-059 §3 under `FullActiveArea` on all three (`f` from 0.908 to 0.99926).

**The rejected order** (warp after the matrix, on `srgb_linear_plane_into`'s clamped
plane), against §2: samples off by more than 16 codes: Q2 139,489, Q3 586,577,
SL3 458,164; worst 12,275, 8,437 and 35,260 codes (the clamp moved before the
interpolation).

**The oracle against `dng_validate -3`** (camera RGB, the `DefaultCrop` window of the
stage-3 `ActiveArea`, unoriented). The misregistration measure is SPEC-099's: the radial
Lucas–Kanade shift (6 iterations, bilinear, patch means removed) over 160×160 patches at
the eight positions `(±0.85, ±0.85)`, `(±0.92, 0)`, `(0, ±0.9)` of the `ActiveArea`
half-extent, clamped 4 px inside the crop. Ours is the camera triple the matrix receives:
`apply_warp_into(W)` of each plane of `ca_reference::reference(demosaic_into(n))`, which
AC7 ties to the entry bit for bit (Q2, Q3; SL3 by the same construction with the fix). The
baseline is ours on the green-only copy (no CA step; `W` is the same set) against the SDK
on that copy.

| File | R `max \|ours − baseline\|` | B | G | R `min \|no-CA − baseline\|` | B |
|---|---|---|---|---|---|
| Q2 | 0.042 px | 0.022 px | 0.000 px | 0.592 px | 0.480 px |
| Q3 | 0.018 px | 0.016 px | 0.000 px | 0.690 px | 0.442 px |
| SL3 | 0.032 px | 0.036 px | 0.000 px | 0.620 px | 0.406 px |

The developed mix against the SDK's stage 3 mixed in `f64` by `mono_mix_row`
(`Σ q_c·cam_c / 2^24`, no fade), the baseline the same on the green-only copy:
`max |ours − baseline|` 0.056 (Q2), 0.012 (Q3), 0.010 px (SL3). The mix carries mostly
green, so without the CA step it is only 0.005 to 0.30 px off: no red-proof for the mix;
its CA step is SPEC-099's, red-proofed there and tied here by AC4.

Measured instead through the entry's `u16` output and an inverse matrix (measured from
`to_srgb_linear_into`), the Q3 and SL3 agree with the table within 0.002 px, but one Q2
blue patch reads 0.217 px, from clipped samples (167,175 of the Q2's output samples are 0
or 65535). So the bound is stated in camera RGB, as SPEC-099's was.

## Questions for design — answered

- **The colour develop: two resamples or one?** Two: the CA step then `W` (§1, §2,
  DEC-088 alternatives). It registers as well as one resample (0.042 against 0.02 px), its
  geometry is the viewport's and the mix's, and `warp_alpha` has one meaning on every
  output. Placed **before the matrix** (DNG stage 3), because warping the clamped `u16`
  plane moves up to 35,260 codes at clipped edges. The intermediate is `3·A` `u16` (about
  363 MB at 60 MP) plus the green plane; a ring is a follow-up.
- **The developed mix:** the strip pass runs SPEC-099's ring inside each strip (strip plus
  `H` rows each side), then `W` over `A1` as today (§3). It amends DEC-084 items 1, 3, 6
  and 7 and replaces DEC-086 item 6. `scratch_len` gains `T_H − T + R` (§4); on the held
  files it stays `2A`.
- **`warp_alpha` and `Framing` on the colour develop:** not gained. No consumer asked
  (onyx's colour view is the sensor-geometry plane; its export is the mix), and it would
  be four new public entries. Additive later.
- **Optional warps:** the colour develop keeps skipping them, CA step included, so the
  Pixel 3a and 7 Pro keep their bytes (DEC-088 item 2). The developed mix applies them
  whatever the flags and `N` (1 or 3), so it matches `mono_mix_plane_into` again (DEC-084
  rule 1), as onyx asked on 2026-11-26. No version moves (§6).
- **Threaded forms:** with an applied warp they run the single-thread body; bit-identical
  across thread counts by construction (AC8).
- **The oracle:** `dng_validate -3` in camera RGB through the exactness-tied triple, with
  the red-proof, plus a mix bound (AC9).

## Acceptance Criteria

- **AC1. Paths (unit + tier A).** `check_required_processing` with `DevelopPath::Colour`
  and `DevelopPath::MonoMix` accepts a mandatory first `WarpRectilinear` with `N` 1, 2 or 3
  (the `N` check is the entry's), and still refuses a mandatory second one
  (`{ list: 3, id: 1 }`) and any other mandatory opcode as before. Mono, SensorGeometry and
  the stage primitives are unchanged.
- **AC2. Identity and unchanged cases are bit-identical (tier A).** On a 48×40 `RGGB` scene
  (as `tests/plane_warp.rs`, with a colour calibration, a `DefaultCrop` and Orientation 1):
  - the colour develop with no list 3, an optional `N = 1` radial warp, an optional `N = 3`
    differing-set warp and an optional malformed (`N = 2`) warp all equal the no-list-3
    output (`develop_color_into`, the `f32` entry and both threaded forms at 1 and 4
    threads);
  - a mandatory `N = 3` warp with three identical sets equals a mandatory `N = 1` warp with
    that set, on the colour develop and on the developed mix;
  - the developed mix with no list 3 or an `N = 1` warp (optional and mandatory) equals
    SPEC-093's output (the existing `tests/developed_mix.rs` oracle stays green).
- **AC3. The colour develop, exactly (tier A).** On the scene with a mandatory `N = 3`
  warp whose sets differ (non-identity green, red and blue each different, CA of several
  pixels at the corner), in three variants — the plain scene, `ActiveArea` `(2, 2, 38, 46)`
  with a `DefaultCrop` inside it, and `cx = 0.3, cy = 0.6` — and under **every**
  orientation 1 to 8: `develop_color_into` equals the crop and orientation (the test's own,
  from the TIFF table) of `to_srgb_linear_into(stripped, warp_planes(reference(demosaic_into(stripped, src))))`,
  with `reference` from `tests/support/ca_reference.rs` and `warp_planes` the public
  `apply_warp_into(W, ..)` per camera plane over the `ActiveArea`. The `f32` entry equals
  `u16 / 65535.0` sample for sample. The output differs from the same composition with the
  CA step left out and from the post-matrix order (`apply_warp_into` per channel of
  `srgb_linear_plane_into`), so neither shortcut passes. A case where only red differs
  leaves blue unresampled. A mandatory `N = 1` warp equals the composition with no CA
  step.
- **AC4. The developed mix is the mono develop of the CA-corrected plane (tier A).** On
  the same three scene variants with the same mandatory **and** optional `N = 3` lists:
  `tests/developed_mix.rs`'s rule-1 oracle, with the mono sensor's list 3 set to one
  `N = 1` `WarpRectilinear` holding `W`, over SPEC-093's grid (orientations 1 to 8, both
  framings, `apply_orientation` both ways, `warp_alpha` 0, ½ and 1) and both weight sets,
  on `develop_with_into` and `develop_with_scratch_into`. Strip heights: a scene taller
  than one strip (as SPEC-093's `ac3_strip_count_and_the_cap_of_t`) with the CA step, so a
  strip edge falls inside the halo.
- **AC5. Scratch and allocation.** `scratch_len` equals §4's formula on: no CA (today's
  numbers, unchanged), CA without an applied warp (`warp_alpha = 0`), CA with a warp, and
  a scene so small that `T_H + R > A`. `develop_with_scratch_into` with the mix and a
  differing `N = 3` warp allocates nothing (`tests/develop_scratch_alloc.rs`, all framings
  and `warp_alpha` 0, ½, 1), and `develop_with_into` allocates exactly once. A scratch of
  `scratch_len − 1` or `+ 1` is `DevelopScratchWrongLength`.
- **AC6. Refusals and order (tier A).**
  - Colour develop, mandatory warp: `N = 2` → `MalformedOpcodeParams { id: 1, .. }`
    (also with identical sets); differing sets with `kt` on blue →
    `UnsupportedWarpTangentialTerms` with blue's `kt`; a folding green
    (`kr = [1, −2, 0, 0]`) → `UnsupportedWarpFold`; identical sets (and `N = 1`) with a
    non-zero `kt` → `UnsupportedWarpTangentialTerms`. Order: an unsupported CFA beats each;
    each beats `MissingColorCalibration`; nothing is written (sentinel `dst`); the threaded
    forms return the same errors. The same lists flagged optional develop as with no list 3.
  - Developed mix: `N = 2` mandatory and optional → `MalformedOpcodeParams { id: 1, .. }`
    (no longer `UnsupportedRequiredOpcode` or a skip); differing sets with `kt` → refused
    after the CFA and before `MissingColorCalibration`; `InvalidMixWeights` still first;
    under `FullActiveArea` a `W` that fails DEC-059 §3 while red's set passes it is
    refused (the bounds rule reads `W`, not `K1`), and the reverse develops; nothing is
    written to `dst` or scratch. `scratch_len`: `N = 2` → malformed; a folding green with
    differing sets → `UnsupportedWarpFold`; `CFAPlaneColor` `[0, 1, 1]` with `N = 3` →
    `UnsupportedCfaPlaneColor`; a mono sensor with `N = 3` sizes (with `H = 0`).
  - `tests/developed_mix.rs`'s ranked fault table gains the per-plane faults at their
    rank, and `ac6_the_mix_sizes_an_optional_multi_plane_warp_as_it_develops` is rewritten:
    an optional `N = 3` is now applied and sized as a warp.
- **AC7. Leica files (tier B, skip loudly by name).** For each present file:
  `develop_color_into` and `develop_color_f32_into` are `Ok` with the pre-registered
  digests; `develop_color_into_threaded` and the `f32` one at 2, 3 and 8 threads equal
  them; the developed mix's five pre-registered digests match (default, `warp_alpha` 0.5,
  0, `FullActiveArea`, `apply_orientation: false`), on `develop_with_into` and
  `develop_with_scratch_into`; `scratch_len` is `2A`. On the Q2 and Q3, `develop_color_into`
  equals AC3's composition at every sample (the SL3: the digest, and the composition with
  the fix applied by the test if the build chooses to). `srgb_linear_plane_*`,
  `mono_mix_plane_*` and the mono `develop_into` pins are unchanged. The refusal asserts
  of SPEC-099's AC7 for the developed outputs are removed; the primitives still refuse
  `{ list: 1, id: 4 }`.
- **AC8. Threads.** Every AC3 case and the Leica files give the same bytes from
  `develop_color_into_threaded` (and the `f32` one) at 1, 2, 3 and 8 threads.
- **AC9. The `dng_validate -3` bound (tier B plus `$IRRADIANCE_SDK_STAGE3_DIR`, local only;
  a SKIP line when unset).** Pre-registered, for each file at each of the eight patches:
  - colour, red and blue: `|shift(ours, SDK full) − shift(ours green-only, SDK green)| ≤
    0.10 px`, green `≤ 0.01 px`, where ours is AC7's tied triple cropped to the
    `DefaultCrop`; **red-proof:** with the CA step left out the red and blue difference is
    `≥ 0.30 px` at every patch;
  - the developed mix (the entry's default output, un-oriented) against the SDK mixed by
    `mono_mix_row` in `f64`: `≤ 0.10 px`.
  This extends `tests/colour_sdk_stage3.rs`.
- **AC10. Nothing else moves.** Every existing pin, determinism hash and tier-B pin of every
  other file and entry: the Pixel 3a and 7 Pro colour develops (optional `N = 1`), the
  M10 and M11 (no opcodes), SPEC-099's sensor-geometry pins on the three files (the tables'
  move off the heap is output-neutral), the mono develop of every file. `DEVELOP_VERSION`
  `"8"`, `MONO_MIX_VERSION` `"1"`. `cargo semver-checks`: no public item added or
  removed (none is); docs only.

## Failing Tests

Written first; each seen to fail (or, tier B, to refuse or mismatch) against `main`.

- **`tests/colour_warp.rs`** (new, tier A): `ac2_unchanged_cases_are_bit_identical`,
  `ac3_the_colour_develop_is_the_dng_order_composition`,
  `ac3_every_orientation_and_the_active_area`, `ac6_colour_refusals_and_their_order`,
  `ac8_threads_give_the_same_bytes`. Red on `main`: every mandatory case refuses
  `{ list: 3, id: 1 }`.
- **`tests/developed_mix.rs`** (extended): `ac4_the_mix_is_the_mono_develop_of_the_ca_plane`,
  `ac4_ca_strips_tile_across_a_strip_edge`, `ac6_the_mix_per_plane_refusals`, the rewritten
  optional-`N = 3` test, the ranked fault table. Red on `main`: the mandatory `N = 3`
  refuses; the optional one is skipped.
- **`tests/develop_scratch_alloc.rs`** (extended): `ac5_the_ca_mix_allocates_nothing_with_scratch`,
  `ac5_the_ca_mix_allocates_once_without`, and a `scratch_len` formula test in
  `tests/developed_mix.rs`.
- **`tests/leica_colour.rs`** (extended, tier B): `ac7_the_opcode_files_develop`.
- **`tests/colour_sdk_stage3.rs`** (extended): `ac9_the_developed_outputs_register_to_dng_validate_stage_3`.
- **`tests/required_processing.rs`** / `src/opcode.rs` unit: AC1.

**Rule-clause map** (each mutant must turn a test red; write the order mutants first, per
SPEC-099's reflection):

| Clause (DEC-088) | Test | Mutant it catches |
|---|---|---|
| 2 warp before the matrix | ac3 | warp the matrixed triple (post-matrix order) |
| 2 `W` = green's set | ac3, ac7 | `K1` (red's set) as the warp |
| 2 CA step in the colour develop | ac3, ac9 red-proof | skip the CA step |
| 2 mandatory only | ac2 | apply an optional warp on the colour develop |
| 2 gather = `apply_warp_into` per plane | ac3 | `bilinear_sample` with truncation, one shared rounding, nearest neighbour |
| 2 crop and orientation in the gather | ac3 (8 orientations, `ActiveArea` variant) | `crop_source_coords` swapped for `output_coords_from_crop`; crop origin left out |
| 2 identity `W` copies | ac2 (`N = 3` identical-to-identity sets) | resample through the identity |
| 2 threaded fallback | ac8 | band the warped develop as the unwarped one |
| 3 `W` on the mix (not `K1`) | ac4, ac6 bounds case | `parse_warp_rectilinear` for the mix |
| 3 optional applied, `N` 1 or 3 | ac4, ac6 | keep DEC-086 item 6's skip |
| 3 CA independent of `warp_alpha` | ac4 (`warp_alpha` 0) | skip the CA step when `W` is the identity |
| 3 strip halo `H` | ac4 strip edge | band `s..e` without the halo; `H − 1` |
| 3 a fresh delay state per strip | ac4 strip edge | one delay across strips |
| 4 `T_H`, `R` | ac5 | `T` for `T_H`; `R` left out; `2H + 1` rows (equivalent, see FU note) |
| 5 colour order | ac6 | per-plane checks before the CFA, or after the calibration; `W`'s `kt` not checked |
| 5 mix order | ac6 | per-plane checks after `MissingColorCalibration` |
| 5 `scratch_len` reads | ac6 | `scratch_len` skipping the fold or the plane colours |
| 6 versions | ac10 | any bump |

## Non-Goals

- `warp_alpha`, `Framing` and `apply_orientation` on the colour develop; applying an
  optional warp there.
- A ring instead of the `3A` camera buffer for the colour develop; threading the warped
  colour develop.
- The CA step itself, the accessor, the rim rule (SPEC-099, DEC-087).
- Tangential warp terms, a per-plane `N` other than 1 or 3, plane colours other than R, G,
  B.
- The mono develop of a colour file (`mono_mix: None`), which keeps applying `K1`.
- Highlight recovery above `WhiteLevel` and filter presets.
- Committing or redistributing any `-PB` file, any SDK output, any derived pixel, or a CI
  test that needs one.

## Open questions

None blocks the build.
- **For the maintainer:** on the developed mix, `N = 2` and `N ≥ 4` become
  `MalformedOpcodeParams` (they were a refusal if mandatory and a skip if optional). This
  is SPEC-099/FU-11's rule carried to the mix. No held file has one.
- **For the maintainer:** sized L and not split (Design note). Split the colour develop
  into its own spec if an L is unwelcome; the probe's pins hold either way.
- **For onyx (FYI):** the colour develop takes no options and skips optional warps; it
  warps camera RGB before the matrix (DNG order), so it differs slightly from warping
  `srgb_linear_plane_into` on a GPU (by up to 35,260 codes at clipped edges, about 0.3 % of
  samples by more than 16). If onyx's colour export ever needs `warp_alpha` or the viewport's
  order, that is a new, additive entry.

## Notes for the Implementer

- Take the pre-registered digests and the AC9 bounds from Implementation Context; never edit
  them to match. If a colour digest misses, compare the gather with `apply_warp_into` term
  by term (one `source_coord` per pixel, `bilinear_sample_with` per channel, the
  `crop_source_coords` table, the crop origin) before anything else. If a mix digest misses
  but AC4 passes, check that the strips' CA step equals `mono_mix_plane_rows_into` (the
  halo, a fresh delay per strip).
- Resolve the per-plane warp once per call (the `ChromaticShift`, `W`, the matrix), never
  per pixel or per strip.
- The colour develop's path with an optional warp, or none, must be today's code
  untouched; so must the mix without a CA step.
- AC3's composition must be built from public primitives and `tests/support/ca_reference.rs`,
  not crate code.
- The SDK outputs and green-only copies live in `$IRRADIANCE_SDK_STAGE3_DIR`, outside the
  repo. Never copy any `-PB` file, SDK output or derived pixel into the repo, a fixture, a
  seed or CI.

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
