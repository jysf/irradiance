---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-091
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
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
  stage: STAGE-031
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5
  created_at: 2026-11-24

references:
  decisions: [DEC-079, DEC-033, DEC-018, DEC-016, DEC-040, DEC-042, DEC-053, DEC-078, DEC-069, DEC-002]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation]
  related_specs: [SPEC-092, SPEC-093, SPEC-033, SPEC-049, SPEC-050, SPEC-059]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-031's <capability>". Optional; null is acceptable.
value_link: "STAGE-031: the white-balanced linear RGB plane onyx uses directly and the mixer builds on"

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
  tokens_estimate: 35000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 90
      recorded_at: 2026-11-24
      notes: "Main-loop design in the PROJ-006 orchestrator session (frame, onyx answers, STAGE-031 frame, the DEC-079 corpus probe, this design); not split per spec. Measured at the STAGE-031 close as orchestration cost."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-091: The white-balanced linear RGB step at sensor geometry, with the working-space DEC and the arithmetic decision for the new path

## Context

First spec of STAGE-031 (PROJ-006). onyx asked (PROJ-006 answers,
2026-11-23) for the white-balanced, demosaiced linear RGB plane as its own public
step in front of the mixer, for three uses: the colour reference view its DEC-059
promises, its own candidate mixes in `mono-grade`, and the side-by-side quality
study that gates PROJ-006. Today the same plane is reachable only by composing
`demosaic_into` and `to_srgb_linear_into` (with `apply_gain_maps` first), and no
contract says that composition is the supported step.

This spec also carries the stage's two root decisions, because the mixer and the
developed mix build on them: the working space (linear is expected, onyx prefers
it; the DEC records a test run on the held colour files) and the arithmetic of
the new path (today's matrix is f64 with each operation rounded separately; the
IEEE argument says it is the same on every platform, but no CI hash measures the
colour path yet, `docs/determinism.md`; onyx asked for fixed-point).

onyx's reply to the design (2026-11-25) accepted DEC-079 and added two asks this spec
takes: the f32 plane must be bit-identical on aarch64 and x86_64 and pinned by a golden
digest (its candidate mixes and quality study are stored and compared, its DEC-012),
and a row-range (band) form into a caller-allocated buffer, because a 60 MP f32 plane
is about 720 MB, too much on iPad (its PROJ-004).

## Goal

Give a colour Bayer DNG one public, documented step to a white-balanced linear
RGB plane at sensor geometry in the decided working space, whole or in row strips,
measured bit-identical across architectures, with a DEC for the working space and the
new path's arithmetic.

## Inputs

- **Files to read:**
  - `decisions/DEC-079-...md`: the working space, the fold, and §3 arithmetic. This
    spec implements the RGB step; the fold and the fixed-point row are SPEC-092's.
  - `decisions/DEC-033` (f64 basic operations, committed constants), `DEC-018`
    (`u16` full scale, clamp before round), `DEC-042` (the fused traversal and its one
    green plane), `DEC-053` / `DEC-078` (required processing and malformed tags refuse
    first), `DEC-069` (`GainMap`s at stage 2), `DEC-040` (CFA phase and level slot from the
    ActiveArea origin).
  - `src/color.rs`: the module doc (1 to 140), `with_normalizer!` (165),
    `to_srgb_linear_into` (289), `develop_color_into` (393) and
    `develop_color_u16_with` (407, the model: same body, a different sink),
    `develop_color_f32_into` (485), `check_colour_required_processing` (650),
    `check_color_buffers` (788), `CropScatter::slot_of` (908),
    `interleaved_rgb_len` (936), `build_camera_to_srgb_linear` (972),
    `quantize` (1575), `resolve_colour_gain_maps` (1583),
    `resolve_colour_levels` (1708).
  - `src/demosaic.rs`: `develop_color_fused` (753). It already visits every
    pixel of the `width × height` plane, masked border included (the border's
    green feeds its neighbours); today's sink drops what falls outside the crop.
  - `tests/malformed_tags.rs` / `tests/misshapen_tags.rs` / `tests/unhonoured_tags.rs`
    / `tests/required_processing.rs`: their `Entry` tables, which every colour entry
    joins.
  - `fuzz/fuzz_targets/demosaic.rs`: the colour-side fuzz target this spec extends.
- **Corpus:** `$IRRADIANCE_CORPUS_DIR` = `~/PSeven/experiments/crustimg_redo_plus/images`.
  The held colour files: `CANON-EOS-5D-MARK-III/5G4A9394.DNG`,
  `GOOGLE-PIXEL-3A/IMG_20190918_164153.dng`, `GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng`,
  `GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng`, `NIKON-D300S/_DSC4615.DNG`.

## Outputs

- **Files modified:**
  - `src/color.rs`: four new public functions (Design §1 and §5), sharing the fused
    traversal with `develop_color_into`; module doc gains a short "Sensor-geometry
    plane" section pointing at DEC-079.
  - `fuzz/fuzz_targets/demosaic.rs`: also calls the four new entries on the unpacked
    plane, the two band forms with a fuzzer-chosen row range.
  - `src/lib.rs`: one new `Error` variant, `InvalidRowRange`, appended at the end of
    the enum and classified `CallerError` in `Error::kind()` (DEC-051); its `Display`
    arm and a `tests/error_kind.rs` row.
  - `tests/determinism.rs`: a colour fixture and two committed hashes (Design §7).
  - `docs/determinism.md`: the `color.rs` row moves from "unmeasured" to "measured" for
    the sensor-geometry plane, which is `develop_color_into`'s pixels (AC1).
  - `tests/linear_plane.rs` (new): the AC tests below.
  - The `Entry` tables in `tests/malformed_tags.rs`, `tests/misshapen_tags.rs`,
    `tests/unhonoured_tags.rs` and `tests/required_processing.rs`: four new rows each,
    refusing exactly as `develop_color_into` does.
  - `CHANGELOG.md` `[Unreleased]` Added: the four functions and the variant.
- **Not modified:** `DEVELOP_VERSION` (stays `"8"`), every existing function's output,
  every existing determinism hash and Tier-B pin.
- **New public API:** `irradiance::color::srgb_linear_plane_into`,
  `srgb_linear_plane_f32_into`, `srgb_linear_plane_rows_into`,
  `srgb_linear_plane_rows_f32_into`, and `Error::InvalidRowRange`.
- **New flags / options:** none.

## Design

1. **Two entry points, sensor geometry.**
   ```rust
   pub fn srgb_linear_plane_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error>;
   pub fn srgb_linear_plane_f32_into(sensor: &Sensor, src: &[u16], dst: &mut [f32]) -> Result<(), Error>;
   ```
   `src` is the unpacked raw mosaic (`width·height`, as `develop_color_into` takes);
   `dst` is interleaved RGB at `width·height·3`, pixel `(x, y)` at index
   `3·(y·width + x)`. **Every** pixel of the plane is written, masked border included,
   with exactly the value the fused traversal computes there (no crop, no orientation,
   no warp: onyx draws and crops the plane itself, its DEC-036/052/063).
2. **Same pipeline as `develop_color_into`.** Levels, `OpcodeList2` `GainMap`s,
   Hamilton-Adams demosaic and the camera-to-linear-sRGB matrix `M`
   (`build_camera_to_srgb_linear`) run exactly as today; only the sink changes, from
   `CropScatter` to the identity. One internal `width·height` green plane, as today.
3. **The two outputs.**
   - `u16`: `quantize(m_i0·r + m_i1·g + m_i2·b)` per channel, today's expression in
     today's order, so it is the existing colour pixel bit for bit (clamped to
     `[0, 65535]`, round half up, DEC-018).
   - `f32`: `((m_i0·r + m_i1·g + m_i2·b) / 65535.0) as f32`, the same `f64` sum
     **unclamped and unquantised**, so 1.0 is white, a negative value is an
     out-of-gamut colour, and a value above 1.0 is a white-balance over (DEC-079 fact 2).
     This is the plane onyx's candidate mixes and quality study need. It is **not**
     `develop_color_f32_into`'s rule (that one quantises first) and the doc comment says
     so.
4. **Refusals: identical to `develop_color_into`, in its order.** Required processing
   and malformed tags (`DevelopPath::Colour`), then `ColorSourceWrongLength`, then the
   geometry checks `develop_color_into` runs (ActiveArea, DefaultCrop, Orientation),
   then `RgbBufferWrongLength { expected: width·height·3, .. }`, then levels, gain maps,
   CFA and calibration. Rationale: the plane exists exactly when the developed colour
   image does, so an onyx viewport and export never disagree about whether a file
   opens. No new `DevelopPath`. Nothing is written unless every check passes (DEC-016).
5. **Band forms (onyx, 2026-11-25: a 60 MP f32 plane is about 720 MB, too much on
   iPad).**
   ```rust
   pub fn srgb_linear_plane_rows_into(sensor: &Sensor, src: &[u16], rows: core::ops::Range<u32>, dst: &mut [u16]) -> Result<(), Error>;
   pub fn srgb_linear_plane_rows_f32_into(sensor: &Sensor, src: &[u16], rows: core::ops::Range<u32>, dst: &mut [f32]) -> Result<(), Error>;
   ```
   `rows` is in whole-plane coordinates (`0..height`, masked rows included). `dst` is
   `(rows.end − rows.start)·width·3`, plane row `y` at index `3·((y − rows.start)·width + x)`.
   Every value equals the same pixel of the whole-plane form **bit for bit**, so a caller
   that tiles `0..height` in strips of any heights gets exactly the whole plane. Built on
   `develop_rows_banded` (`src/demosaic.rs`), which already develops a row range with a
   halo and an even-aligned local top and is proven equal to the whole-plane render by
   the threaded colour path (SPEC-051). Internal scratch is proportional to the band
   (its rows plus the halo, times `width`), never to the plane; `src` is still the
   whole mosaic. Refusal order: §4's, with `InvalidRowRange { start, end, height }`
   (empty, reversed, or `end > height`) after the geometry checks and before
   `RgbBufferWrongLength`, whose `expected` depends on the range. The whole-plane forms
   keep the single fused traversal (they are not rewritten as one band).
6. **Serial only.** Threaded variants are not in this spec (Non-Goals); a caller can run
   bands on its own threads.
7. **Determinism, measured across architectures (onyx, 2026-11-25; its DEC-012).** The
   plane uses only IEEE basic operations: the `f64` matrix build and apply are `+ − × ÷`
   with committed constants (DEC-033), Rust never contracts a multiply and an add into
   an FMA unless `mul_add` is called (the crate never calls it), there is no libm call,
   and the final `f64 → f32` cast is IEEE round-to-nearest-even. So the plane is the
   same on every conforming platform. Today that argument is **unmeasured** for the
   colour path (`docs/determinism.md`); this spec measures it. `tests/determinism.rs`
   gains a Tier-A colour fixture (synthetic Bayer with `AsShotNeutral`,
   `ColorMatrix1`/`ColorMatrix2`, `ForwardMatrix1`/`ForwardMatrix2` and two
   calibration illuminants, so illuminant interpolation runs), chosen so its f32 plane
   holds at least one negative and one above-1.0 value. FNV-1a hashes of the `u16` and
   `f32` planes are committed and asserted on both CI determinism legs (x86_64 Linux,
   arm64 macOS), debug and release. The existing red-proof test extends to both new
   hashes.
8. **No output change anywhere else.** `DEVELOP_VERSION` stays `"8"`.

## Acceptance Criteria

Pre-registered. Every check is exact except AC4's stated half-code bound on a cast.
AC11 to AC13 and mutants (h) to (k) were added 2026-11-25 from onyx's reply (band form,
determinism pin); the build had not started.

- [ ] **AC1, equivalence with the developed image (Tier B).** For each held colour
  file, on a clone of its `Sensor` with `orientation` set to `Some(1)`: the `u16`
  plane, cut to the crop rectangle (ActiveArea origin + DefaultCropOrigin, size
  DefaultCropSize, all from the public `Sensor` fields), equals
  `develop_color_into`'s output byte for byte. Missing corpus skips loudly.
- [ ] **AC2, the whole plane is written (Tier A).** On a synthetic Bayer fixture with
  an **odd** ActiveArea top and left (so mutant (f) can fail), a masked border, and a
  DefaultCrop smaller than the ActiveArea, orientation 1: (i) `dst` pre-filled with a
  sentinel has no sentinel left after the call, masked border included; (ii) the pixels
  inside the ActiveArea but outside the DefaultCrop equal `develop_color_into`'s output
  for the same fixture with DefaultCrop widened to the whole ActiveArea (which changes
  neither the CFA phase nor the levels, both indexed from the ActiveArea origin,
  DEC-040); (iii) two calls give identical planes.
- [ ] **AC3, the f32 plane is unclamped (Tier B, Canon 5D Mark III).** At least one
  pixel has a negative f32 channel where the `u16` plane is 0, and the count of such
  channel values is greater than zero on the Canon (DEC-079's probe found 3.23% of
  pixels). On the Pixel 4a, at least one f32 channel exceeds 1.0 where the `u16` is
  65535.
- [ ] **AC4, the two planes agree where nothing clamps (Tier B, all five files).**
  For every channel value whose `u16` is strictly between 0 and 65535,
  `|f32·65535 − u16| ≤ 0.5 + 1e-3`; where `u16` is 0 the f32 is ≤ 0.5/65535 + 1e-7;
  where `u16` is 65535 the f32 is ≥ (65535 − 0.5)/65535 − 1e-7.
- [ ] **AC5, the f32 value is the unquantised sum (unit test in `src/color.rs`).** On
  a small hand-built Bayer `Sensor` whose matrix is known, each f32 equals
  `((M·cam)/65535.0) as f32` recomputed in the test from `build_camera_to_srgb_linear`
  and the demosaiced camera values, exactly (`==` on `f32` bits).
- [ ] **AC6, refusals match `develop_color_into`.** Both new entries join the `Entry`
  tables in `tests/malformed_tags.rs`, `tests/misshapen_tags.rs`,
  `tests/unhonoured_tags.rs` and `tests/required_processing.rs` with
  `develop_color_into`'s `DevelopPath::Colour` (the band forms with `rows = 0..height`),
  and every row there passes. Plus, in
  `tests/linear_plane.rs`: a wrong `src` length is `ColorSourceWrongLength`; a `dst`
  of `width·height·3 − 1` and `+ 1` is `RgbBufferWrongLength` with
  `expected = width·height·3`; a non-CFA plane is `NotAColorFilterArray`; a sensor
  with no `AsShotNeutral` is `MissingColorCalibration`. For the band forms: `5..5`,
  `6..5` and `0..height+1` are `InvalidRowRange` with those values and the plane's
  height, and a `dst` one short is `RgbBufferWrongLength` with the band's `expected`.
  In every refusal, `dst` is exactly its sentinel fill.
- [ ] **AC7, `GainMap`s apply (Tier B, Pixel 7 Pro).** The plane differs from the
  same file with its `OpcodeList2` removed from the clone (the control proving the
  maps are applied), and AC1 already pins the applied value.
- [ ] **AC8, nothing else moves.** The full suite passes unchanged with
  `IRRADIANCE_CORPUS_DIR` set; `tests/error_kind.rs` gains exactly the one new row;
  every existing `tests/determinism.rs` constant is unchanged; `DEVELOP_VERSION == "8"`;
  `just semver-check` reports only the five additions.
- [ ] **AC9, fuzzed and panic-free.** `fuzz/fuzz_targets/demosaic.rs` calls the four
  new entries (the band forms with a fuzzer-chosen range, valid or not); a short `cargo fuzz run demosaic` (the toolchain brief's smoke length) finds
  nothing; `just lint-ci` passes with the deny-level panic lints.
- [ ] **AC11, strips tile to the whole plane, bit for bit (Tier A and Tier B).** For both
  sample types: on AC2's odd-ActiveArea fixture, every partition of `0..height` into
  strips from this set equals the whole-plane form exactly: one strip; strips of 1 row;
  strips of 7 rows (odd starts); and two strips split at an odd row inside the masked
  border. On the Pixel 7 Pro (GainMaps) and the Canon, strips of 256 rows with an odd
  remainder equal the whole plane (the `f32` compared by bits).
- [ ] **AC12, determinism measured on both architectures.** `tests/determinism.rs`'s new
  colour fixture asserts its committed `u16` and `f32` plane hashes; the CI
  `determinism` job passes on both legs, debug and release; the extended red-proof shows
  a one-bit change moves each new hash; and the fixture's f32 plane holds a negative and
  an above-1.0 value (asserted, so the hash covers the unclamped path).
- [ ] **AC13, golden digests on the held files (Tier B).** `tests/linear_plane.rs` pins an
  FNV-1a digest of the `f32` plane (little-endian bits) and of the `u16` plane for each
  of the five held colour files, as committed constants. These run locally only (the
  corpus is not in CI); AC12 carries the cross-architecture evidence.
- [ ] **AC10, the oracle can go red.** Each pre-registered mutant below, applied by
  hand in build, fails at least one named test; the build's handback lists which.

## Failing Tests

- **`tests/linear_plane.rs`** (new; Tier A fixtures via `tests/support/dng_writer.rs`,
  Tier B via the manifest):
  - `ac1_the_plane_cropped_equals_the_developed_image` (Tier B)
  - `ac2_every_pixel_of_the_plane_is_written`
  - `ac3_the_f32_plane_keeps_negatives_and_overs` (Tier B)
  - `ac4_the_two_planes_agree_where_nothing_clamps` (Tier B)
  - `ac6_buffer_and_input_refusals_leave_dst_untouched`
  - `ac7_gain_maps_apply_to_the_plane` (Tier B)
  - `ac11_strips_tile_to_the_whole_plane` (Tier A) and
    `ac11_strips_tile_on_held_files` (Tier B)
  - `ac13_golden_digests_of_the_held_files` (Tier B)
- **`tests/determinism.rs`:** `colour_plane_hashes_match_committed` and the extended
  `determinism_red_proof` (AC12).
- **`src/color.rs` unit tests:** `ac5_the_f32_plane_is_the_unquantised_sum`.
- **The four `Entry` tables** (AC6): four new rows each.
- **`tests/error_kind.rs`:** one row for `InvalidRowRange` (`CallerError`).

**Rule-clause map (Design §1 to §4; DEC-079's RGB-step consequence).**

| Clause | Test | Path where it changes the output |
|---|---|---|
| §1 every pixel written | `ac2_…` | the identity sink, border pixels |
| §2 same pipeline (levels, maps, demosaic, M) | `ac1_…`, `ac7_…` | the `u16` value inside the crop |
| §3 u16 is today's quantise | `ac1_…` | byte equality with `develop_color_into` |
| §3 f32 is unclamped and unquantised | `ac3_…`, `ac5_…` | negative and over values, exact bits |
| §3 the two agree | `ac4_…` | the half-code bound |
| §4 refusals in order, dst untouched | `ac6_…` and the four tables | the error variant, dst sentinel |
| §5 strips equal the whole plane | `ac11_…` (both) | band edges, odd starts, the halo |
| §5 the range refusal | `ac6_…` | `InvalidRowRange` fields |
| §7 the same bits on both architectures | `colour_plane_hashes_match_committed` | the committed hashes on each CI leg |

**Distinguishing fixtures (pre-registered mutants).**
- (a) The sink still crops (identity replaced by `CropScatter`): `ac2_…`'s margin
  fixture keeps sentinels. A fixture with no margin cannot tell.
- (b) The f32 path quantises or clamps first (copies `develop_color_f32_into`): `ac3_…`
  on the Canon (negatives) and Pixel 4a (overs). A file without out-of-gamut pixels
  (the Pixel 7 Pro, 0.00%) cannot tell.
- (c) The f32 path divides by 65536: `ac5_…`'s exact bits.
- (d) `GainMap`s skipped: `ac7_…` and `ac1_…` on the Pixel 7 Pro (the three held Pixel
  files carry `OpcodeList2` maps; the Canon and Nikon do not, so they cannot tell).
- (e) Levels skipped or black not subtracted: `ac1_…` on every file.
- (f) The CFA phase taken from the plane origin instead of the ActiveArea origin
  (DEC-040): `ac2_…`'s margin fixture must use an **odd** ActiveArea top or left, so
  the two phases differ.
- (g) `dst` written before a late check fails: `ac6_…`'s missing-`AsShotNeutral` row
  (the calibration check runs last).
- (h) The band drops its halo (develops only its own rows): `ac11_…`'s strips of 1 and
  7 rows differ at every strip edge. One whole-plane strip cannot tell.
- (i) The band's local top is not rounded to an even row (Bayer phase flips on odd
  starts): `ac11_…`'s 7-row strips. Even-only starts cannot tell.
- (j) The band form indexes `dst` from plane row 0 instead of `rows.start`: any strip
  after the first. A partition with one strip cannot tell.
- (k) An `f64` multiply and add replaced by `mul_add` in the f32 sink: the committed
  f32 hash (AC12) changes on the fixture whose values are not exact in `f64`.

**Moved assertions.** None: no existing behaviour changes.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The mixer and the clip rule (SPEC-092).
- Crop, orientation, warp or resampling of the RGB plane (SPEC-093 handles geometry for the mono plane only).
- Threaded variants (a caller can run band forms on its own threads).
- A band form of the existing cropped `develop_color_into` / `develop_color_f32_into`.
- Any change to the output bits of `demosaic_into`, `to_srgb_linear_into` or `develop_color_into`.

## Notes for the Implementer

- Reuse `develop_color_u16_with`'s body: factor the sink so the crop sink and the
  identity sink share the one traversal. Do not copy the traversal.
- Keep the `u16` expression and operation order exactly as `develop_color_u16_with`'s
  (AC1 is byte equality).
- The f32 cast happens once, after the `f64` divide; no intermediate `f32` arithmetic.
- `ac1_…` needs the crop rectangle from public `Sensor` fields; with `orientation`
  forced to 1 no orientation mapping is needed in the test.
- One new `Error` variant, `InvalidRowRange`, at the very end of the enum. PROJ-005's
  writer variants have shipped; take the tail from `origin/main` at build start.
- The band forms call `develop_rows_banded` directly; do not reach them through
  `run_row_bands` (that splits the output for threads, which this spec does not do).
- The determinism fixture must not change the bytes any existing fixture hashes; add a
  new fixture rather than adding tags to an existing one.
- `docs/determinism.md`: change the `color.rs` row only for what AC12 measures.

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
