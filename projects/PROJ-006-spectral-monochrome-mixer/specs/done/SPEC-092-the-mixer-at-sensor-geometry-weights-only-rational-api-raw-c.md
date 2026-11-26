---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-092
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
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
  stage: STAGE-031
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5
  created_at: 2026-11-25

references:
  decisions: [DEC-083, DEC-079, DEC-033, DEC-018, DEC-016, DEC-040, DEC-042, DEC-044, DEC-051, DEC-053, DEC-069, DEC-070, DEC-078, DEC-002]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation, provenance-recorded-per-algorithm]
  related_specs: [SPEC-091, SPEC-093, SPEC-051, SPEC-059]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-091]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-031's <capability>". Optional; null is acceptable.
value_link: "STAGE-031: the mixed mono plane at sensor geometry for onyx's GPU viewport"

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
  tokens_estimate: 65000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 75
      recorded_at: 2026-11-25
      notes: "Design in a side session started by the PROJ-006 orchestrator (reading, the corpus probe for the row and the fade threshold, DEC-083, this design, HANDOFF-202). Un-metered main-loop cycle; measured at the STAGE-031 close as orchestration cost."
    - cycle: build
      handoff: HANDOFF-202
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 30039095
      estimated_usd: 20.93
      duration_minutes: 38
      recorded_at: 2026-11-26
      notes: "Re-summed at ship (2026-11-26) over the whole transcript 0a0649c3-731f-4348-ac40-9bf00f022115.jsonl (113 unique message.ids, all claude-opus-5-5, no subagents): input 230 + cache-creation 326,333 (1-hour) + cache-read 29,597,769 + output 114,763, priced at 5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x. The handback's floor was 27,463,707 / 19.45 USD, taken before the handback commit. CI run 45855649150 on b822fa2 green on both determinism legs."
    - cycle: verify
      handoff: HANDOFF-203
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 15115503
      estimated_usd: 10.58
      duration_minutes: 80
      recorded_at: 2026-11-26
      notes: "Re-summed at ship (2026-11-26) over the whole transcript 29c227de-0fb7-46f6-8420-8dd83ce30639.jsonl (81 unique message.ids, all claude-opus-5-5, no subagents): input 173 + cache-creation 184,949 (1-hour) + cache-read 14,878,679 + output 51,702, priced as the build. The handback's floor was 12,293,376 / 8.75 USD, taken before the handback commit."
  totals:
    tokens_total: 45154598
    estimated_usd: 31.51
    session_count: 3
shipped_at: 2026-11-26
---

# SPEC-092: The mixer at sensor geometry: weights-only rational API, the fixed-point row, the fade of negative weights near clip, the plane's black and white levels

## Context

Second spec of STAGE-031 (PROJ-006). onyx's GPU viewport always draws the unpacked
sensor plane and does crop, orientation and warp itself (its DEC-036, 052, 063). For a
colour file, the mixed monochrome plane takes the place of the mono sensor plane there,
so it must be at sensor geometry and carry its own black and white levels for onyx's
single normalise step.

onyx's API ask: weights only (presets are named weight sets, the presets stage), and onyx
stores the resolved weights in its recipe. With a linear working space (SPEC-091,
`DEC-079`) the weights apply folded into the camera matrix, so a clipped raw channel has
spread into every term, and the clip rule is this spec's hard question.

**Settled before design (2026-11-24 probe, onyx's reply 2026-11-25; `DEC-079` "Clip rule
evidence").** Drop-and-renormalise is rejected: it darkens highlights on every held body.
The plain fold is not monotone on every colour where the folded row has a negative entry
(Canon and Nikon). The rule is the **fade of negative weights**: positive-weight terms as
folded, negative-weight terms multiplied by `1 − s`, `s` ramping from 0 to 1 as the
brightest level-normalised camera channel goes from a committed threshold to white,
provably monotone for a threshold at or above half of white. onyx accepted it on
conditions, each an AC below: no parameter; bit-identical to no fade whenever the folded
row has no negative entry; the threshold documented exactly; the fixed-point row
reproducible, pinned per held file, hashed on both CI legs and exposed; no clip mask;
a row-range form for iPad memory.

From SPEC-091's ship, two follow-ups this design takes up:

- **`SPEC-091/FU-9`, a reusable band plan.** Each band call clones the `Sensor` and
  re-resolves the levels, the `GainMap`s and the matrix. Design §7 measures that cost and
  records why this spec does not add a plan.
- **`SPEC-091/FU-10`, oracle blind spots.** A shared path makes an "equals the old path"
  oracle move with the mutant, and a fused or reordered `f64` sum is absorbed by the
  output's resolution. Design §8 pre-registers an injected cancellation fixture per
  channel and per term for every "no FMA" and "keep the order" rule in the row build.

## Goal

Mix a colour Bayer DNG to a monochrome `u16` plane at sensor geometry, whole or in row
strips, with caller-supplied rational weights applied as `DEC-079`'s folded row quantised
to `DEC-083`'s fixed point, the parameter-free fade of negative weights near clip so
highlights stay monotone, an exposed and pinned fixed-point row, and the plane's black and
white levels as committed constants.

## Inputs

- **Files to read:**
  - `decisions/DEC-083-...md` (this spec's decision: weights rule, row format, `τ`, the
    per-pixel step, the overflow argument) and `decisions/DEC-079-...md` (the fold, the
    fade, §3 arithmetic, the clip-rule evidence).
  - The shipped SPEC-091 (`specs/done/SPEC-091-...md`): its band form, refusal order,
    determinism fixture and mutants (a) to (k). This spec mirrors its shape.
  - `src/color.rs`: `develop_color_with` (490) and `PixelSlot`/`WholePlane` (571), the
    three stores (527 to 567), `srgb_linear_plane` / `srgb_linear_plane_rows` /
    `srgb_linear_plane_rows_with` (701 to 768), `check_plane_buffers` (777),
    `build_camera_to_srgb_linear` (1242), `quantize` (1845), `resolve_colour_gain_maps`
    (1853), `resolve_colour_levels` (1978), `slot_normalizer` (2063).
  - `src/demosaic.rs`: `develop_rows_banded` (584), `develop_color_fused` (765),
    `fused_green_at_site` (1129) and `fused_interpolate_red_blue` (1169), where a pixel's
    own CFA channel is its normalised photosite.
  - `src/opcode.rs` `GainMaps::apply` (670): the gain is applied to the normalised value
    and clipped at white.
  - `tests/linear_plane.rs`, `tests/determinism.rs` (the SPEC-091 colour fixture at 450),
    and the four `Entry` tables in `tests/malformed_tags.rs`, `tests/misshapen_tags.rs`,
    `tests/unhonoured_tags.rs`, `tests/required_processing.rs`.
- **Corpus:** `$IRRADIANCE_CORPUS_DIR` = `~/PSeven/experiments/crustimg_redo_plus/images`;
  the five held colour files, as SPEC-091 lists them.

## Outputs

- **Files modified:**
  - `src/color.rs`: three public functions and six public constants (Design §1 to §6),
    the row build and the per-pixel kernel (private), a generalised identity sink so the
    mono plane reuses the one fused traversal, unit tests (Failing Tests); module doc
    gains a "Mono mix" section pointing at `DEC-079` and `DEC-083`.
  - `src/lib.rs`: three `Error` variants appended at the end of the enum, in this order:
    `InvalidMixWeights { channel: Option<u8> }` (`CallerError`),
    `MonoBufferWrongLength { expected: u64, actual: usize }` (`CallerError`),
    `MixRowOutOfRange { channel: u8 }` (`Unsupported`); their `Display` arms; DEC-051's
    classification.
  - `tests/mono_mix.rs` (new): the Tier-A and Tier-B ACs.
  - `tests/determinism.rs`: mixer row and plane hashes on the SPEC-091 colour fixture
    (its bytes unchanged), the red-proof extended (AC12).
  - `tests/error_kind.rs`: three rows.
  - The four `Entry` tables: two new rows each (`mono_mix_plane_into`, and
    `mono_mix_plane_rows_into` with `rows = 0..height`), `DevelopPath::Colour`.
  - `fuzz/fuzz_targets/demosaic.rs`: calls the three new functions (AC9).
  - `docs/determinism.md`: a measured row for the mixer.
  - `docs/provenance-ledger.md`: the row drafted in Design §10.
  - `CHANGELOG.md` `[Unreleased]` Added: the functions, constants and variants.
- **Not modified:** `DEVELOP_VERSION` (stays `"8"`), every existing function's output,
  every existing determinism hash and Tier-B pin, `build_camera_to_srgb_linear`.
- **New public API:** `irradiance::color::mono_mix_row`, `mono_mix_plane_into`,
  `mono_mix_plane_rows_into`, the constants `MONO_MIX_ROW_FRACTION_BITS`,
  `MONO_MIX_FADE_START`, `MONO_MIX_FADE_WIDTH_BITS`, `MONO_MIX_BLACK`, `MONO_MIX_WHITE`,
  `MONO_MIX_VERSION`, and the three `Error` variants.
- **New flags / options:** none. No fade parameter, no clip mask.

## Design

1. **The public API (weights only, rational).**
   ```rust
   pub const MONO_MIX_ROW_FRACTION_BITS: u32 = 24;
   pub const MONO_MIX_FADE_START: u16 = 57343;      // 65535 − 2^13
   pub const MONO_MIX_FADE_WIDTH_BITS: u32 = 13;    // the fade spans 8192 codes
   pub const MONO_MIX_BLACK: u16 = 0;
   pub const MONO_MIX_WHITE: u16 = 65535;
   pub const MONO_MIX_VERSION: &str = "1";          // onyx's recipe records it (§12)

   pub fn mono_mix_row(sensor: &Sensor, weights: &[Rational; 3]) -> Result<[i32; 3], Error>;
   pub fn mono_mix_plane_into(sensor: &Sensor, src: &[u16], weights: &[Rational; 3],
                              dst: &mut [u16]) -> Result<(), Error>;
   pub fn mono_mix_plane_rows_into(sensor: &Sensor, src: &[u16], weights: &[Rational; 3],
                                   rows: core::ops::Range<u32>, dst: &mut [u16]) -> Result<(), Error>;
   ```
   `weights` are linear-sRGB `[R, G, B]` (`DEC-079`), `Rational` being `ifd::Rational`.
   `src` is the unpacked raw mosaic (`width·height`). The whole-plane `dst` is
   `width·height` mono samples, pixel `(x, y)` at `y·width + x`, **every** pixel written,
   masked border included (the masked photosites are normalised like any other; no
   `GainMap` applies there). The band's `dst` is `(rows.end − rows.start)·width`, plane
   row `y` at `(y − rows.start)·width + x`, every value bit-identical to the whole
   plane's. `mono_mix_row` returns the fixed-point row the planes apply (onyx's oracle
   reads and stores it). There is no clip-mask output and no fade parameter.
2. **The plane's levels.** The mixed plane is already level-normalised (`DEC-018` full
   scale): black is `MONO_MIX_BLACK = 0` and white `MONO_MIX_WHITE = 65535` for every
   file, so onyx's single normalise step is `v / 65535`. Constants, not a return value,
   because they cannot vary; the doc comment says so. They are part of the documented
   contract (onyx, 2026-11-26): changing either is an API change, not a silent one, and
   also bumps `MONO_MIX_VERSION` (§12).
3. **The weights and the row (`DEC-083` §1 to §3).**
   - Weights: each `den > 0` and `num ≥ 0`, else `InvalidMixWeights { channel: Some(i) }`
     for the first offending index; then
     `n0·d1·d2 + n1·d0·d2 + n2·d0·d1 == d0·d1·d2` in `i128`, else
     `InvalidMixWeights { channel: None }`. Each `w_j = f64::from(num) / f64::from(den)`.
   - Row: `v_i = (w_0·M[0][i] + w_1·M[1][i]) + w_2·M[2][i]`, `M` from
     `build_camera_to_srgb_linear` (rows = linear-sRGB channel, columns = camera
     channel). Basic `f64` operations, left to right, no `mul_add`.
   - Quantise: `t = v_i · 16777216.0`; if `t` is not finite or `t + 0.5` is outside
     `[−2147483648.0, 2147483648.0)`, `MixRowOutOfRange { channel: i }`; else
     `q_i = (t + 0.5).floor() as i32`.
4. **The per-pixel step (`DEC-083` §5), all `i64`.** For the camera triple
   `x = [r, g, b]` the fused traversal emits:
   `P = Σ_{q_i ≥ 0} q_i·x_i`, `N = Σ_{q_i < 0} q_i·x_i`, `m = max(r, g, b)`,
   `u = min(65535 − m, 8192)`, `F = clamp((P·8192 + N·u + 2^36) >> 37, 0, 65535)`
   (arithmetic shift: floor, so this is round half up). Whether a term is negative is
   decided by `q_i`, not `v_i` (they agree except where `q_i = 0`, which adds nothing).
   - **No negative `q_i`:** `N = 0` and `F = clamp((P + 2^23) >> 24, 0, 65535)`, the plain
     fold, bit for bit (`DEC-083` Consequences).
   - **Below the threshold** (`m ≤ 57343`): `u = 8192` and `F` is the plain fold.
   - **At white** (`m = 65535`): `u = 0`; only the positive terms remain.
   - **Overflow:** none possible (`DEC-083` "Overflow"). The crate's lints may require
     checked or saturating operations; they never engage, and AC14 pins the extreme case.
   - The kernel is one private function used by every entry, so a whole-plane pixel and a
     band pixel are the same code.
5. **The threshold, documented exactly** (this text goes in `MONO_MIX_FADE_START`'s doc
   comment):
   - **The compared value** is, at each pixel, `m`, the largest of its three camera
     channels as the demosaic emits them: after the black level is subtracted and the
     sample scaled by `WhiteLevel − BlackLevel` to `[0, 65535]` (`DEC-070`), after
     `OpcodeList2` `GainMap`s (`DEC-069`), and before white balance and the matrix. All
     three channels count, interpolated ones included.
   - **Against what:** the fade starts above `MONO_MIX_FADE_START = 57343`
     (`τ = 57343/65535 ≈ 0.875`) and is complete at `65535`, white after levels; in
     between, the negative-weight terms are scaled by `(65535 − m)/8192`.
   - **How it relates to the raw photosite onyx's overlay reads:** a pixel's own CFA
     channel is its photosite's normalised value exactly (Hamilton-Adams keeps the native
     sample, `fused_interpolate_red_blue`); the other two are interpolated from its
     neighbours and clamped to `[0, 65535]`. So wherever the photosite is at or above
     `WhiteLevel`, its own channel is `65535` and the fade is complete there, unless an
     `OpcodeList2` `GainMap` gain below 1 at that photosite pulls it back under white
     (none on the held files). The fade can also be complete where the raw photosite is
     not clipped: where a neighbour-interpolated channel reaches `65535`, or a gain above
     1 lifts a photosite to white. The probe (2026-11-25) found no clipped photosite on
     any held file whose own channel fell short of `65535`, and found 17,900, 277,095 and
     1,065 photosites on the Pixel 3a, 4a and 7 Pro whose `GainMap` lifted them to white
     from below `WhiteLevel` (none on the Canon or Nikon). onyx's overlay is therefore a
     subset of the pixels where the fade is complete, and equals it on files without
     `GainMap`s apart from interpolated neighbours.
6. **Refusals, in order; nothing written unless every check passes (`DEC-016`).**
   - The planes: `InvalidMixWeights` first (the argument alone decides it, on every
     file); then SPEC-091's order for the plane: required processing and malformed tags
     (`DevelopPath::Colour`), `ColorSourceWrongLength`, the geometry
     (`InvalidActiveArea` / `InvalidDefaultCrop` / `UnsupportedOrientation`), for the band
     `InvalidRowRange`, then `MonoBufferWrongLength { expected: rows·width }`, then the
     levels, the `GainMap`s, the CFA, `MissingColorCalibration`, and last
     `MixRowOutOfRange`. The plane exists exactly when SPEC-091's RGB plane does and the
     weights and row are valid.
   - `mono_mix_row`: `InvalidMixWeights`, then `build_camera_to_srgb_linear`'s errors
     (`MissingColorCalibration`), then `MixRowOutOfRange`. It runs no buffer, geometry,
     opcode or CFA check: the row exists whenever the calibration does.
7. **Band form, and `FU-9`: no reusable plan.** The band form is built on
   `develop_rows_banded`, as SPEC-091's is, with the same halo and even-aligned local
   top. The probe timed what a plan would save: a `Sensor` clone plus re-resolving the
   levels, the `GainMap`s and the matrix took 0.6 to 3.4 µs per call on the held files,
   against 7 to 16 ms to develop a 64-row strip (and 0.1 to 0.25 ms for one row of work
   plus its six halo rows). That is under 0.05% of a 64-row strip and about 1% of a
   one-row strip, so a public plan type would add API surface for no measurable gain.
   Recorded here and in the follow-ups as closing FU-9 with "measured, not worth it";
   revisit if a caller tiles in single rows. The row is built once per call.
8. **`FU-10`: the "no FMA" and "keep the order" rules, each with its own fixture.** The
   only floating-point arithmetic this spec adds is the row build (§3); the per-pixel
   step is exact integers, where order cannot matter. The row build is a private
   `fn fold_mix_row(w: [f64; 3], m: &[[f64; 3]; 3]) -> [f64; 3]` so unit tests can inject
   values. Each fixture asserts the exact `f64` bits against a hand-derived constant,
   **and** computes the mutant's value in the test (with `mul_add` or the other order)
   and asserts it differs, so the fixture provably distinguishes:
   - **No FMA, per channel and per term (9 fixtures).** Channel `i ∈ {0, 1, 2}`, fused
     site `j ∈ {0, 1, 2}`: `j = 0` is `w_0.mul_add(M[0][i], w_1·M[1][i])`, `j = 1` is
     `w_1.mul_add(M[1][i], w_0·M[0][i])`, `j = 2` is `w_2.mul_add(M[2][i], s_01)`. Each
     puts the cancellation `0.1·3 − 0.3` on that site in column `i` (for example
     `j = 0`: `w = [0.1, 1, 0]`, `M[0][i] = 3`, `M[1][i] = −0.3`; unfused
     `5.551115123125783e-17`, fused `2.7755575615628914e-17`), the other columns zero.
   - **Keep the order, per channel (6 fixtures).** The two wrong pairings,
     `(t_0 + t_2) + t_1` and `(t_1 + t_2) + t_0`, for each channel, with terms chosen so
     each differs from `(t_0 + t_1) + t_2` (for example `t = [1, 2^-52, 2^-53]`:
     correct `1 + 2^-51`, `(t_0 + t_2) + t_1` gives `1 + 2^-52`).
   - **The weight conversion (1 fixture).** `w = 3/10` is the `f64` nearest 0.3, not
     `3·(1/10) = 0.30000000000000004` (a reciprocal-multiply mutant).
   - **The quantiser (2 fixtures).** `v = 2.5·2^-24` gives `q = 3` (half to even gives 2)
     and `v = −2.5·2^-24` gives `q = −2` (half away from zero gives −3).
   - **End to end:** the per-file golden rows (AC5) and the cross-architecture row hash
     (AC12) pin the build on real calibrations.
9. **Determinism (DEC-002).** The row is basic `f64` operations on committed constants
   and one exact quantiser; the plane is exact integer arithmetic on the fused
   traversal's integer camera triples. `tests/determinism.rs` hashes, on the SPEC-091
   colour fixture (dual illuminant, near-full-scale band; its bytes do not change), the
   row and the mono plane for two weight sets: luminance `[2126, 7152, 722]/10000`, and
   the first of the one-hot sets `[1,0,0]`, `[0,1,0]`, `[0,0,1]` whose row has a negative
   entry (asserted; the build records which). The test asserts the fixture has at least
   one pixel where the fade changes the output (the mono differs from the plain fold
   computed by the test's own oracle from `demosaic_into`'s triples; the fixture's levels
   are the identity). Committed on both CI determinism legs, debug and release.
10. **Provenance (draft row for `docs/provenance-ledger.md`).** `src/color.rs`, the mono
    mixer: the weights fold `v = wᵀ·M` (linear algebra over the DNG model already
    ledgered for `color.rs`), the fixed-point quantiser and the fade of negative weights
    (`DEC-079`, `DEC-083`, original arithmetic with its monotonicity argument in
    `DEC-079`); source: our own decisions, no implementation consulted; class 1
    (public-domain technique, as the warp-alpha row records its lerp).
11. **No output change elsewhere; no `irr` hook.** `DEVELOP_VERSION` stays `"8"`. An `irr`
    hook to eyeball a mix is not added: SPEC-093's developed mix is the natural thing to
    look at, and the golden digests here pin the plane. Serial only; a caller runs
    strips on its own threads.

12. **The mixer version (onyx, 2026-11-26).** `MONO_MIX_VERSION: &str = "1"` sits next to
    the row constants in `src/color.rs`, documented like `DEVELOP_VERSION`: a history list
    whose first entry is `"1"`, SPEC-092 (`DEC-083`). onyx's `.irr` recipe records it so a
    change to what the mixer computes shows up as a version change, not as silently
    different prints.
    - **Bump it** for any change to the mixer's own arithmetic that can change a row or a
      mixed pixel for the same `Sensor`, `src` and weights: the weight rule, the fold or its
      order, the scale or the rounding of the row, the fade threshold or width, the kernel's
      rounding or clamp, or the levels of §2.
    - **Not for** a change upstream of the mixer (levels, `GainMap`s, demosaic,
      `build_camera_to_srgb_linear`). Those already bump `DEVELOP_VERSION`, so a mix is
      fixed by the pair (`DEVELOP_VERSION`, `MONO_MIX_VERSION`), and the doc comment says
      onyx records both.
    - **Pinned:** AC5's golden-row test and AC12's hash test each assert
      `MONO_MIX_VERSION == "1"` beside their committed values, with a comment that a new
      golden value needs a version bump in the same change.

## Acceptance Criteria

Pre-registered. Every check is exact except AC4's bound, derived from the final casts
(the SPEC-091 lesson: a tolerance below the output's own cast error fails real pixels).

- [x] **AC1, the kernel matches an independent oracle (Tier A, unit and integration).**
  An `i128` oracle written in the test, using division and `div_euclid` rather than a
  shift (so it shares no code with the kernel), computes
  `clamp(floor((P·8192 + N·u + 2^36) / 2^37), 0, 65535)`. On the Tier-A fixture
  (identity levels, so `demosaic_into`'s triples are the traversal's camera values),
  `mono_mix_plane_into` equals that oracle applied to `demosaic_into`'s triple and
  `mono_mix_row`'s row, at every pixel, for the luminance weights and for one weight
  set whose row has a negative entry.
- [x] **AC2, every pixel written; strips tile the plane (Tier A and Tier B).** On
  SPEC-091's odd-ActiveArea fixture: a sentinel-filled `dst` has no sentinel left,
  masked border included; two calls agree; and every partition from SPEC-091 AC11's set
  (one strip, 1-row strips, 7-row strips, a split at an odd row inside the masked border)
  equals the whole plane exactly. On the Pixel 7 Pro and the Canon, 256-row strips with
  an odd remainder equal the whole plane.
- [x] **AC3, bit-identical to no fade when no weight is negative (Tier A).** (i) Unit:
  for a row with no negative entry, the kernel equals the plain fold
  `clamp((P + 2^23) >> 24, 0, 65535)` on every `x` whose channels each take the values
  0, 1, 57342, 57343, 57344, 61439, 65534, 65535 and 64 xorshift values (so `m` lies
  below, at and above the threshold and at white). (ii) Integration: on a Tier-A fixture
  with identity levels and a weight set whose row has no negative entry (asserted), the
  plane equals the `i128` oracle's plain fold of `demosaic_into`'s triples at every
  pixel, and at least one pixel has `m ≥ 57344` (asserted from those triples). Tier B adds
  no separate check: the three Pixel files' luminance rows have no negative entry
  (AC5), and AC4 holds on every one of their pixels.
- [x] **AC4, the luminance identity (Tier B, all five files; `DEC-079` Validation).** With
  `w = [2126, 7152, 722]/10000`, `f_j` SPEC-091's unclamped `f32` plane and `F` the mono
  plane: `|F − clamp(65535·Σ_j w_j·f_j, 0, 65535)| ≤ 0.5 + 3·65535/2^25 +
  65535·max_j|f_j|·2^-24 + 1e-6` (the final rounding, the row's quantisation, and the
  reference's own `f32` cast), evaluated in `f64`:
  - on the three Pixel files (rows with no negative entry, asserted), at every pixel;
  - on the Canon and Nikon (rows with a negative entry), at every pixel where the fade
    cannot act. The test finds those pixels without the private traversal: it recovers
    `M̂` from `mono_mix_row` with the three one-hot weight sets (each row is `M`'s row to
    `2^-24`), estimates the camera triple `x̂ = M̂⁻¹·(65535·f)`, and keeps pixels with
    `max x̂ ≤ 57335` (an 8-code margin for the estimate). It asserts at least 90% of the
    plane's pixels are kept.

  And on the Canon and the Pixel 4a, the count of pixels where `F` differs by more than
  one code from `round(Σ_j w_j·u_j)` of SPEC-091's clamped `u16` plane is above zero (the
  `DEC-079` probe found 2.70% and 5.06%).
- [x] **AC5, golden rows (Tier B).** `mono_mix_row` returns exactly these rows
  (probe-measured 2026-11-25; a mismatch is reported, not edited):

  | File | luminance `[2126,7152,722]/10000` | red `[4/5, 1/5, 0]` | ortho `[0, 1/2, 1/2]` |
  |---|---|---|---|
  | Canon 5D Mark III | `[6222052, 14856368, -4008594]` | `[43457289, -9545613, -44314]` | `[-3895542, 7681333, 24802109]` |
  | Pixel 3a | `[9238644, 11642451, 1534676]` | `[33977479, 1862915, -432667]` | `[413112, 8768457, 11975737]` |
  | Pixel 4a | `[6580281, 12748060, 1667508]` | `[25202743, 4872113, 209979]` | `[1344711, 7581953, 15016856]` |
  | Pixel 7 Pro | `[5656251, 11930046, 2168331]` | `[21211951, 3142429, 5765]` | `[21010, 8388347, 14980760]` |
  | Nikon D300S | `[9666745, 12584293, -2609097]` | `[36952549, -3543768, -4355900]` | `[896662, 6844397, 12247218]` |

  Plus an FNV-1a digest of each file's mono plane under each of the three weight sets
  (15 committed constants; local only, the corpus is not in CI).
- [x] **AC6, monotone through clipping (Tier A unit).** For each of the rows
  `[6222052, 14856368, -4008594]` (Canon luminance), `[-3895542, 7681333, 24802109]`
  (Canon ortho), `[-8076979, 27606901, -12847745]` (Canon one-hot green),
  `[9666745, 12584293, -2609097]` (Nikon luminance) and `[-528141, 17561180, -943847]`
  (Pixel 4a one-hot green): over the 90,000 directions `c = [a, 299, b]`,
  `a, b ∈ {0, 2, …, 598}`, ramped `x_i(k) = min(k·c_i, 65535)` for `k` from 0 until every
  channel is at 65535, the kernel never decreases from one `k` to the next. Control: the
  plain fold on the Canon luminance row decreases on at least one ray (the probe counted
  7,199 rays), so the grid can fail.
- [x] **AC7, monotone on a ramped fixture end to end (Tier A).** A synthetic Bayer DNG
  of flat colour patches (each at least 8×8 photosites, identity levels) whose exposure
  rises patch by patch past clipping, with calibration chosen so the luminance row has a
  negative entry (asserted): the mono value at each patch centre never decreases along
  the ramp, and at least one patch centre has a clipped channel and differs from the
  plain fold.
- [x] **AC8, the threshold and where the fade reads (Tier A unit).** With
  `q = [0, 2^30, −2^30]` (plain fold 0 whenever `x_G = x_B`): `x = [0, 57343, 57343]`
  gives 0 (`u = 8192`, no fade); `x = [0, 57344, 57344]` gives 448 (`u = 8191`, one step);
  `x = [0, 65535, 65535]` gives 65535 (`u = 0`, positive terms only, clamped). `m` is the
  maximum over all three channels: with `q = [2^22, 2^22, −2^22]`, the triples
  `[65535, 1000, 1000]`, `[1000, 65535, 1000]` and `[1000, 1000, 65535]` give 16634, 16634
  and 500 (positive terms only), where the plain fold gives 16384, 16384 and 0. A green-only
  `m` gets the first and third wrong; a mean `m` (22512, below the threshold) gets all
  three wrong.
- [x] **AC9, fuzzed and panic-free.** `fuzz/fuzz_targets/demosaic.rs` calls the three new
  functions with fuzzer-chosen weights (valid and not), row range and a `dst` one short
  or one long on half the inputs; a short `cargo +nightly fuzz run demosaic` (the
  toolchain brief's smoke length) finds nothing; `just lint-ci` passes.
- [x] **AC10, refusals and order (Tier A).** `InvalidMixWeights { channel: Some(1) }` for
  a zero denominator in the second weight, `Some(0)` for a negative numerator,
  `Some(2)` for a negative denominator; `{ channel: None }` for `[1/3, 1/3, 1/4]`; and
  `[2/10, 7/10, 1/10]` (exactly one, `0.9999999999999999` in `f64`) is accepted. An
  invalid weight on a file with an unsupported mandatory opcode is still
  `InvalidMixWeights` (weights first). `ColorSourceWrongLength`; `MonoBufferWrongLength`
  with `expected = width·height` for `dst` one short and one long; band `5..5`, `6..5`,
  `0..height+1` are `InvalidRowRange` and a band `dst` one short is
  `MonoBufferWrongLength` with the band's `expected`; a non-CFA plane is
  `NotAColorFilterArray`; no `AsShotNeutral` is `MissingColorCalibration`; an
  `AsShotNeutral` of `[1/100000, 1, 1]` is `MixRowOutOfRange { channel: 0 }` from all
  three functions. In every plane refusal `dst` is exactly its sentinel fill.
  The two plane entries join the four `Entry` tables and every row there passes.
- [x] **AC11, the row build's fixtures (`FU-10`, unit).** The 18 fixtures of Design §8
  pass, and each one's mutant value differs from its expected value.
- [x] **AC12, determinism measured on both architectures.** `tests/determinism.rs`
  asserts committed FNV-1a hashes of the two rows (12 little-endian bytes each) and of
  the two mono planes on the SPEC-091 colour fixture; asserts the fade changes at least
  one pixel there; the CI `determinism` job passes on both legs, debug and release; and
  the extended red-proof shows a one-bit sample change moves both plane hashes and a
  one-unit change of one `AsShotNeutral` numerator moves both row hashes.
- [x] **AC13, nothing else moves.** The full suite passes with `IRRADIANCE_CORPUS_DIR` set;
  `tests/error_kind.rs` gains exactly three rows; every existing determinism constant is
  unchanged; `DEVELOP_VERSION == "8"`; `MONO_MIX_VERSION == "1"`, asserted beside the
  golden rows (AC5) and the hashes (AC12); `just semver-check` reports only the additions.
- [x] **AC14, no overflow at the extremes (unit).** The kernel equals the `i128` oracle
  for every `q` in `{[i32::MAX; 3], [i32::MIN; 3], [i32::MAX, i32::MIN, i32::MAX]}` and every
  `x` in `{[65535; 3], [65535, 0, 65535], [57344, 57344, 57344]}`. The private quantiser,
  given `t = v·2^24` directly, accepts `t = 2147483646.5` (giving `i32::MAX`) and
  `t = −2147483648.5` (giving `i32::MIN`), and refuses `t = 2147483647.5`,
  `t = −2147483649.0`, `NaN` and `±inf`.
- [x] **AC15, the oracle can go red.** Each pre-registered mutant below, applied by hand
  in build, fails at least one named test; the handback lists which. Mutants marked
  equivalent are argued, not run.

## Failing Tests

- **`tests/mono_mix.rs`** (new; Tier A via `tests/support/dng_writer.rs`, Tier B via the
  manifest):
  - `ac1_the_plane_equals_the_i128_oracle_on_demosaic_triples`
  - `ac2_every_pixel_written_and_strips_tile` and `ac2_strips_tile_on_held_files` (Tier B)
  - `ac3_no_negative_row_plane_is_the_plain_fold`
  - `ac4_luminance_matches_the_unclamped_plane` (Tier B)
  - `ac5_golden_rows_and_digests_of_the_held_files` (Tier B)
  - `ac7_a_ramp_through_clipping_stays_monotone`
  - `ac10_refusals_in_order_leave_dst_untouched`
- **`src/color.rs` unit tests:** `ac3_no_negative_row_is_the_plain_fold`,
  `ac6_the_fade_is_monotone_on_the_saturating_ray_grid`,
  `ac8_the_fade_starts_after_57343_and_reads_the_largest_channel`,
  `ac11_the_row_fuses_no_term_of_any_channel` (9),
  `ac11_the_row_keeps_its_summation_order` (6),
  `ac11_a_weight_converts_by_one_division`, `ac11_the_row_rounds_half_up` (2),
  `ac14_no_overflow_at_the_extremes`.
- **`tests/determinism.rs`:** `mono_mix_hashes_match_committed` and the extended
  `colour_determinism_red_proof` (AC12).
- **The four `Entry` tables:** two rows each (AC10). **`tests/error_kind.rs`:** three rows.

**Rule-clause map (Design §1 to §8; `DEC-083` §1 to §5).**

| Clause | Test | Path where it changes the output |
|---|---|---|
| §1 every pixel, sensor geometry | `ac2_…` | the identity sink, border pixels |
| §1 band equals whole plane | `ac2_…` (both) | band edges, odd starts, the halo |
| §2 levels are constants | doc test on the constants | n/a (constants) |
| §12 mixer version | `ac5_…`, `mono_mix_hashes_match_committed` | the version beside each golden |
| §3 weights rule | `ac10_…` | `InvalidMixWeights` fields; the `f64`-sum fixture |
| §3 row fold and quantiser | `ac5_…`, `ac11_…`, AC12 | the row's `i32` values |
| §4 kernel | `ac1_…`, `ac14_…` | every mono value vs the `i128` oracle |
| §4 no negatives is the plain fold | `ac3_…` | rows with no `q_i < 0` |
| §5 threshold and largest channel | `ac8_…` | `m = 57343` vs `57344`; max over R, G, B |
| §5 monotone | `ac6_…`, `ac7_…` | the saturating-ray grid; the patch ramp |
| §6 refusals in order | `ac10_…` and the four tables | variant and `dst` sentinel |
| §9 same bits on both architectures | `mono_mix_hashes_match_committed` | the committed hashes per CI leg |

**Distinguishing fixtures (pre-registered mutants).** For each: the fixture on which the
right and wrong answers differ, and why the oracle is not computed by the mutated code.
- (a) Row transposed, `v = M·w`: AC5's golden rows and AC4 (the f32 plane is computed by
  SPEC-091's store, not the row). A symmetric `M` could not tell; no held `M` is.
- (b) Quantiser rounds half to even: `ac11_the_row_rounds_half_up` with `v = 2.5·2^-24`.
- (c) Quantiser rounds half away from zero: the same test with `v = −2.5·2^-24`.
- (d) Scale `2^23` or `2^25` (the constant changed in one place only): AC5 and AC4.
- (e) Fade removed (plain fold everywhere): `ac6_…` (the Canon luminance grid) and AC7.
- (f) `m` taken from green only: `ac8_…`'s R-max and B-max triples.
- (g) `m` taken as the channels' mean: `ac8_…`'s triples with one channel at 65535.
- (h) The fade also scales positive terms: `ac3_…` (no-negative rows near white) and
  `ac1_…`.
- (i) `u` not capped at 8192 (`u = 65535 − m`): `ac8_…` below the threshold, and `ac1_…`
  on any pixel with `N < 0`.
- (j) Threshold off by one (`u = min(65536 − m, 8192)`, so the fade starts after 57344): `ac8_…` at `m = 57344`, which must give 448 and the mutant gives 0.
- (k) Upper clamp dropped (the cast wraps): `ac14_…` (`[i32::MAX; 3]` at white).
- (l) Any one of the 9 fold sites fused with `mul_add`: `ac11_the_row_fuses_no_term_of_any_channel`,
  one fixture per site. The golden rows alone cannot see it (`2^-52` is far below
  `2^-24`), which is why the fixtures exist.
- (m) Fold pairs reordered: `ac11_the_row_keeps_its_summation_order`, two per channel.
- (n) Weight sum checked in `f64`: `ac10_…`'s `[2/10, 7/10, 1/10]` is refused.
- (o) Weight converted as `num·(1/den)`: `ac11_a_weight_converts_by_one_division`.
- (p) Weights checked after the file checks: `ac10_…`'s invalid weight on the opcode file.
- (q) `dst` written before `MixRowOutOfRange`: `ac10_…`'s sentinel on the tiny-neutral
  fixture (the row check is the last).
- (r) Band drops its halo / local top not even / `dst` indexed from row 0: `ac2_…`'s 1-
  and 7-row strips and any strip after the first (SPEC-091's (h) to (j)).
- (s) CFA phase from the plane origin: `ac1_…` (the oracle's triples come from
  `demosaic_into`, which phases independently; the SPEC-091 FU-2 lesson).
- **Equivalent, not run:** (t) the shift replaced by a division truncating toward zero
  (differs only for a negative numerator, which the lower clamp sends to 0 either way);
  (u) the negative test made on `v_i` instead of `q_i` (they differ only where `q_i = 0`,
  which adds nothing to either sum).

**Moved assertions.** None: no existing behaviour changes.

## Non-Goals

- Filter presets (the presets stage).
- Spatial highlight reconstruction.
- The developed (cropped, oriented, warped) output (SPEC-093).
- A clip-mask output, a fade parameter, negative weights (`DEC-083`; additive later; onyx
  confirmed on 2026-11-26 that its v1 presets and sliders need none).
- A reusable band plan (Design §7) and threaded variants.
- An `irr` hook (Design §11).
- Any change to the output bits of an existing function.

## Notes for the Implementer

- Generalise SPEC-091's identity sink so a store writes one `u16` per pixel (mono) as
  well as three (RGB); do not copy `develop_color_with` or `srgb_linear_plane_rows_with`.
  The `srgb_linear_plane*` outputs and hashes must not move.
- Build the row inside the traversal's `setup` (after the CFA check), as the matrix is
  today, so `MixRowOutOfRange` comes after the CFA and calibration errors and before any
  write. The band form builds it before `develop_rows_banded`, after `check_supported_bayer`
  and the matrix, as SPEC-091's band form builds the matrix.
- `InvalidMixWeights` runs before `check_colour_required_processing`.
- Three new `Error` variants at the very end of the enum, after `InvalidRowRange`, in
  the order listed in Outputs. Take the tail from `origin/main` at build start.
- The determinism fixture's bytes must not change; mix it, do not edit it.
- `docs/determinism.md`: add a row for the mixer; change no other row.
- Read `DEC-083`'s Overflow section before choosing checked or saturating operations.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — Mutate every public contract constant and the order of every refusal the design
   ranks, at design, before naming the oracles. Those were the verify's three survivors
   (V5/V5b, V6 to V8); the 34 kernel and row mutants the design pre-registered were all
   caught. Also compute each pre-registered mutant's value at every point of the fixture
   meant to catch it: that finds AC8's blind spot for (i) (`m = 57343` gives `u = 8192`
   either way) and Design §8's order example, which separated only one of the two wrong
   pairings, before the build.

2. **Does any template, constraint, or decision need updating?**
   — No DEC: DEC-083 holds as accepted, and DEC-051's later-additions paragraph gained one
   sentence for the three variants. No template change now. The gaps are evidence for
   codified lessons, recorded in `guidance/signals.yaml`:
   `every-rule-clause-maps-to-a-reachable-test` (three public constants and the "last"
   refusal had no row in the rule-clause map) and
   `a-designed-mutant-must-be-able-to-fail-on-its-fixture` (AC8 and the §8 order example,
   its second recurrence since codification). The `demosaic` fuzz gap adds evidence to
   `fuzz-targets-without-a-ci-smoke-job`. Whether the template wording tightens is the
   STAGE-031 close's call.

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. SPEC-093 (already framed) takes the two design lessons in its Context:
   a test tying every public contract constant to its computation, and an order oracle
   for every ranked refusal. The `demosaic` fuzz CI job is the hygiene patch PROJ-005's
   close already scheduled.

4. **Where was the worst defect caught?**
   — verify

5. **What can a user do now that they couldn't before?**
   — Before, onyx had no way to turn a colour Bayer DNG into a monochrome plane for its
   viewport except mixing SPEC-091's linear RGB plane itself, which clips highlights
   non-monotonically wherever the folded row has a negative weight (7,199 of 90,000 rays
   on the Canon luminance row). Now `mono_mix_plane_into` and `mono_mix_plane_rows_into`
   give a mono u16 plane at sensor geometry, whole or in row strips, from rational
   weights alone, monotone through clipping on every held row (0 decreasing rays), with
   the fixed-point row exposed by `mono_mix_row`, 15 golden rows pinned and its hashes
   measured equal on aarch64 and x86_64, versioned as `MONO_MIX_VERSION = "1"`.

## Follow-ups

Raised in HANDOFF-202 (build, `B`) and HANDOFF-203 (verify, `V`); neither handback
numbered them, so they are numbered here. The two SPEC-091 follow-ups this design took
up are closed here too.

| id | finding | disposition |
|---|---|---|
| `FU-1` | B: AC8's triples could not see mutant (i) (`u` uncapped): its only below-threshold triple has `m = 57343`, where `65535 − m = 8192` | `fixed` — build, commit `b822fa2`: `ac8_…` also asserts `x = [0, 1000, 500]` gives 32000; (i) re-run red at verify |
| `FU-2` | B: Design §8's order example `[1, 2^-52, 2^-53]` distinguishes only the `(t0 + t2) + t1` pairing | `fixed` — build, `ac11_the_row_keeps_its_summation_order` adds `[2^-53, 2^-53, 1]` with its own `assert_ne!`; (m) on channel 2 red at verify |
| `FU-3` | B: tests written in the same session as the code, code first (departs from `test-before-implementation`) | `closed` — accepted at verify: the tests fail to compile against `origin/main`'s `src/` and every pre-registered mutant is red, so the red evidence is real; no code defect followed from it |
| `FU-4` | B+V: `ac6_the_fade_is_monotone_on_the_saturating_ray_grid` takes about 135 s unoptimised, in CI's debug `rust / test` job | `closed` — the grid is the spec's oracle as designed, and `rust / test` passed in 7.7 min on run 45857000180; it runs in under 1 s in `just test`'s `corpus` profile, so local runs do not pay it. The faster form (start each ray at the last `k` below the threshold, where a separate test pins the plain fold) is recorded in HANDOFF-202 for whoever next works on CI time |
| `FU-5` | V: `MONO_MIX_FADE_START`, `MONO_MIX_ROW_FRACTION_BITS` and `MONO_MIX_FADE_WIDTH_BITS` were documentation only; changing any of them passed every test (V6 to V8) | `fixed` — verify, commit `993cda4`: `color::tests::the_public_mixer_constants_are_the_kernels`. The design lesson goes to `spec: SPEC-093` (its Context) |
| `FU-6` | V: Design §6's "`MixRowOutOfRange` last" had no oracle; building the row before the file checks passed, in the plane and the band form (V5, V5b) | `fixed` — verify, commit `993cda4`: `mono_mix::the_row_range_refusal_comes_after_every_file_check`. The design lesson goes to `spec: SPEC-093` (its Context) |
| `FU-7` | V: a kernel rounding half down (V10) was caught only by AC5's local corpus digests, which CI never runs | `fixed` — verify, commit `993cda4`: `color::tests::the_kernel_rounds_an_exact_tie_up` |
| `FU-8` | B+V: the `demosaic` fuzz target, which now also reaches the three mixer entries, has no CI smoke job and no `just` recipe | `signal: fuzz-targets-without-a-ci-smoke-job` — N=2 to N=3; already accepted and scheduled as one hygiene patch at the PROJ-005 close |
| `SPEC-091/FU-9` | a reusable band plan (each band call clones the `Sensor`) | `closed` — measured at design, Design §7: not worth a plan for this spec; the build and verify found nothing new (one `Sensor` clone per band call, the mixer adds no allocation) |
| `SPEC-091/FU-10` | per-channel, per-term cancellation fixtures for the "no FMA" and "keep the order" rules | `fixed` — AC11's 18 fixtures (`ac11_the_row_fuses_no_term_of_any_channel`, `ac11_the_row_keeps_its_summation_order`); (l) red on all nine sites and (m) on all six reorders |
