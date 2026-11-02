---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-033
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

project:
  id: PROJ-002
  stage: STAGE-008
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-033, DEC-016, DEC-018, DEC-025, DEC-030]  # colour-transform arithmetic / caller-owned buffer / full-scale u16 / tone sRGB OETF / exact-rational colour metadata
  constraints: [oracle-must-be-shown-red, no-panics-on-untrusted-input]
  related_specs: [SPEC-034, SPEC-035]  # siblings in STAGE-008

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # foundation — STAGE-007 (shipped) is the input; no in-stage blocker

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-008's <capability>". Optional; null is acceptable.
value_link: >
  Delivers STAGE-008's first display-referred colour render — white balance +
  camera→XYZ→linear-sRGB + the sRGB OETF — turning STAGE-007's camera-space RGB
  into a recognisable picture.

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
  tokens_estimate: 40000000  # ~28M build + ~12M verify; new math + oracle, but reuses demosaic/tone/irr harness
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-033: **Colour-transform core + first colour develop**: WB

## Context

STAGE-007 delivers demosaiced **camera-space** RGB — not a viewable image: the
channels are the sensor's native primaries, scaled by the camera's own white, with
no display encoding. The foundation spec of **STAGE-008** (colour transform, the last
PROJ-002 stage) turns that into a **display-referred sRGB** image. STAGE-006 already
parses the colour metadata this needs (`AsShotNeutral`, `ColorMatrix1/2`,
`ForwardMatrix1/2`), all verified truthful; PROJ-001 provides the sRGB OETF (`tone`,
DEC-025) and the develop/`irr` harness. This spec is the first colour render; it uses
the **single (first) illuminant** to get a correct picture — dual-illuminant
interpolation (scene-adaptive matrix selection) is [[SPEC-034]], and the independent
ColorChecker ΔE oracle is [[SPEC-035]]. ⚠ **L-risk (the frame's pre-named split):**
this is designed whole and expected to fit [M] by reusing STAGE-007's harness, but if
the build finds it exceeds [M] it ships the **matrix MATH** (`to_srgb_linear_into` +
the committed constant + the analytic oracle) first and the `develop_color_into` + `irr`
composition as a follow-on spec — the same pre-named-split escape hatch [[SPEC-031]]
used for its green/red-blue passes. Notes for the Implementer records the cut line.

## Goal

Produce a **display-referred sRGB** image from a Bayer DNG: white balance
(`AsShotNeutral`) + camera→XYZ (`ForwardMatrix1` when present, else inverted
`ColorMatrix1`; single illuminant) + XYZ(D50)→linear-sRGB, composed as a new
`color::develop_color_into` and emitted by `irr color` (the `tone` sRGB OETF encodes
the linear-sRGB output for display). The colour matrix math is verified **bit-exact**
on analytic fixtures with a red-proof; a scoped `dnglab`-render cross-check is a loose
sanity check, not the accuracy oracle (that is [[SPEC-035]]'s ΔE).

## Design

The transform is three linear stages folded into **one 3×3 matrix `M` built once per
image**, then applied per pixel. Inputs and outputs are interleaved `u16` where
full-scale `65535` represents `1.0` (`DEC-018`), so the matrix apply is **scale-free**
(the `/65535` on input and `×65535` on output cancel): `out_i = Σ_j M[i][j]·cam_j`.

**Representation & arithmetic (`DEC-033`).** The matrix build and per-pixel apply are
`f64`, using only `+ − × ÷` — IEEE-754 correctly-rounded and byte-identical on macOS
and Linux (the same basis `tone::to_linear_f32_into` already relies on; no FMA, no
transcendental). This is *not* a retreat from the exact-rational discipline of
`DEC-030`: there both sides were exact rationals and equality had to be exact; here the
sRGB primaries and the Bradford CAT are irrational, so — exactly as `DEC-025` runs the
OETF's transcendental once into a committed table — the irrational colour constant is a
**committed constant**, and determinism comes from committed data + IEEE basic ops, not
from matching a runtime transcendental. The sRGB OETF itself stays the committed
`tone` table (`DEC-025`); this spec produces **linear**-sRGB and hands off to `tone`.

**Stage 1 — white balance (`AsShotNeutral`, 3 positive rationals `asn`).** A neutral
subject reads `asn` in camera space; dividing each channel by its neutral maps neutral
to `(1,1,1)`: `D = diag(1/asn_R, 1/asn_G, 1/asn_B)`.

**Stage 2 — camera → XYZ(D50).**
- **`ForwardMatrix1` present** (Canon 5D-III): `FM1` already maps white-balanced camera
  values to XYZ(D50) (its rows sum to the D50 white, the DNG constraint), so
  `cam2xyz = FM1 · D`.
- **else invert `ColorMatrix1`** (Nikon D300S has no `ForwardMatrix`): `CM1` maps
  XYZ→camera, so `cam2xyz = inv(CM1) · D`. `inv` is the exact 3×3 cofactor inverse in
  `f64`; `|det| == 0` (or a required rational with denominator 0) → typed
  `Error::MissingColorCalibration`, never a panic. **Single-illuminant simplification:**
  the `ColorMatrix` path treats its output as XYZ(D50) directly (no scene-white
  adaptation) — documented, and refined by [[SPEC-034]]'s illuminant handling; the ΔE
  ([[SPEC-035]]) is what later proves the Nikon render correct.

**Stage 3 — XYZ(D50) → linear sRGB: the committed constant `C`.** `C = S · B` where
`B` = Bradford chromatic adaptation D50→D65 and `S` = linear-sRGB-from-XYZ(D65)
(IEC 61966-2-1). Both are **committed `f64` constants** in `src/color.rs`, auditable
against the published standards (provenance **class 1** — DNG spec + IEC + the Bradford
CAT), composed once per image:

```
S (linear sRGB ← XYZ D65, IEC 61966-2-1):      B (Bradford, XYZ D50 → D65):
[  3.2404542 -1.5371385 -0.4985314 ]           [  0.9555766 -0.0230393  0.0631636 ]
[ -0.9692660  1.8760108  0.0415560 ]           [ -0.0282895  1.0099416  0.0210077 ]
[  0.0556434 -0.2040259  1.0572252 ]           [  0.0122982 -0.0204830  1.3299098 ]
```

**The combined matrix:** `M = C · cam2xyz` (= `C · FM1 · D`, or `C · inv(CM1) · D`).

**Per-pixel apply + quantize.** For each pixel `(R,G,B)` camera `u16`:
`v_i = M[i][0]·R + M[i][1]·G + M[i][2]·B` (`f64`); then
`out_i = ⌊ v_i.clamp(0.0, 65535.0) + 0.5 ⌋` as `u16` — clamp **before** rounding so the
round-half-up is well-defined and can never be out of `u16` range (the clamp is
load-bearing: a saturated highlight or a negative off-gamut mix is pinned to the window,
`DEC-018`). No panic: `clamp` + `floor` + a bounded `as u16` after clamping.

**Compose — `develop_color_into`.** `src` = raw Bayer mosaic (`w·h`), `dst` = interleaved
linear-sRGB (`w·h·3`): (1) **per-channel level-normalize the mosaic** — black-subtract
and white-normalize each photosite by *its own CFA colour's* `black_levels` /
`white_levels` (per-channel, so it must run on the mosaic before demosaic; falls back to
the scalar `black_level`/`white_level` when the per-channel arrays are absent, clamp per
`DEC-018`), into a `w·h` scratch; (2) `demosaic::demosaic_into` the normalized mosaic →
camera-linear RGB; (3) `to_srgb_linear_into` that RGB → `dst`. Internal scratch is
allowed (as `develop_into`'s warp path already allocates); the public `src`/`dst` stay
caller-owned (`DEC-016`).

## Inputs

- **Files to read:**
  - `src/demosaic.rs` — `demosaic_into` (the camera-RGB producer this composes) and its
    house style (phase-driven, `i32`/round/clamp, caller-owned buffer, in-module oracle).
  - `src/tone.rs` — `encode_srgb16_into` (the OETF `irr color` applies) and the
    `require_matching_length` / `OutputBufferWrongLength` pattern to mirror.
  - `src/develop.rs` — `normalize(sample, black, white)` (lines ~305–360) to reuse/mirror
    for the per-channel mosaic normalize; the warp path's internal-scratch precedent.
  - `src/ifd.rs` — `Sensor` colour fields (`forward_matrix_1`, `color_matrix_1`,
    `as_shot_neutral`, `black_levels`, `white_levels`, `calibration_illuminant_1`) and
    `Rational { num: i32, den: i32 }` (den may be 0 — guard it).
  - `src/bin/irr.rs` — `cmd_demosaic` (mirror for `cmd_color`: unpack → develop_color →
    encode → P6 PPM) and the subcommand dispatch.
- **Decisions:** `DEC-033` (this spec's arithmetic), `DEC-016/018/025/030`.
- **Standards (provenance class 1):** Adobe DNG Specification (colour model), IEC
  61966-2-1 (sRGB primaries + OETF), the Bradford CAT. No code read.

## Outputs

- **`src/color.rs` (new):**
  - `pub fn to_srgb_linear_into(sensor: &Sensor, camera_rgb: &[u16], dst: &mut [u16]) -> Result<(), Error>`
    — the matrix MATH: interleaved camera-linear RGB (`w·h·3`) → interleaved
    **linear-sRGB** `u16` (`w·h·3`). Builds `M` once, applies per pixel.
  - `pub fn develop_color_into(sensor: &Sensor, src: &[u16], dst: &mut [u16]) -> Result<(), Error>`
    — the compose: raw Bayer mosaic (`w·h`) → interleaved linear-sRGB (`w·h·3`), via
    per-channel level-normalize → `demosaic_into` → `to_srgb_linear_into`.
  - private: `build_camera_to_srgb_linear(sensor) -> Result<[[f64; 3]; 3], Error>`, the
    committed `S`/`B` constants, a 3×3 `inv`, `apply_matrix_into`, the quantize helper.
- **`src/lib.rs`:** `pub mod color;` + two `Error` variants, mirroring the demosaic set:
  `ColorSourceWrongLength { expected: u64, actual: usize }` (the `camera_rgb` / mosaic
  length) and `MissingColorCalibration` (no usable `ForwardMatrix1`/`ColorMatrix1`, no
  `AsShotNeutral`, a zero-denominator required rational, or a non-invertible
  `ColorMatrix`) — with `Display` arms. Reuse `RgbBufferWrongLength` for `dst`.
- **`src/bin/irr.rs`:** a `color <file>` subcommand (mirrors `demosaic`): unpack →
  `develop_color_into` → `tone::encode_srgb16_into` → **P6 PPM** (maxval 65535, BE RGB)
  to stdout; stats to stderr; usage line.
- **`tests/color_oracle.rs` (new):** the analytic bit-exact oracle (below), Tier-A.
- **`decisions/DEC-033-*.md`:** minted at design (this cycle) — see References.
- **`docs/provenance-ledger.md`:** a colour-transform row (DNG colour model = class 1;
  sRGB = IEC 61966-2-1; Bradford CAT) — written at build.
- **`CHANGELOG.md`:** `[Unreleased] Added` — the `color` module, `to_srgb_linear_into`,
  `develop_color_into`, `irr color`, and the two `Error` variants (public API, so unlike
  the test-only specs it needs an entry).

## Acceptance Criteria

- [ ] `to_srgb_linear_into` maps camera-linear RGB → linear-sRGB `u16` via
  `M = C · FM1 · diag(1/asn)` (ForwardMatrix path) and `M = C · inv(CM1) · diag(1/asn)`
  (ColorMatrix path), built once per image.
- [ ] **Bit-exact (apply):** for an injected clean `M` (identity, a diagonal, a matrix
  with a negative entry, one that overshoots) every output equals the hand-computed
  `⌊clamp(Σ M·cam, 0, 65535)+0.5⌋` **exactly**, including a round-half-up tie.
- [ ] **Bit-exact (builder):** `build_camera_to_srgb_linear` for a synthetic Sensor with
  known `FM1`/`asn` (and separately known invertible `CM1`/`asn`) equals an independent
  re-derivation of `C · cam2xyz · D` to the bit (`f64 ==`).
- [ ] **Neutral → grey (formula-independent):** a camera pixel equal to `asn` (a neutral
  subject) yields `R == G == B` in the linear-sRGB output (WB + FM map neutral to the
  D50 white, `C` to the sRGB white).
- [ ] **Clamp:** an off-gamut mix that computes negative pins to `0`; one that overshoots
  pins to `65535` — no wrap, no panic (`DEC-018`).
- [ ] **Red-proof:** perturbing one coefficient of a required matrix (a mutated `FM1`)
  changes the output — mutant ≠ honest, honest control unchanged (`DEC-017`,
  `oracle-must-be-shown-red`).
- [ ] **Compose:** `develop_color_into` on a synthetic mosaic equals
  `demosaic_into` then `to_srgb_linear_into` on the per-channel-normalized mosaic.
- [ ] **Typed errors, no panic:** wrong `camera_rgb`/`dst`/mosaic lengths
  (`ColorSourceWrongLength` / `RgbBufferWrongLength` / `DemosaicSourceWrongLength`),
  absent/unusable colour metadata (`MissingColorCalibration`), a non-invertible
  `ColorMatrix`, den-0 rationals, and degenerate dims (1×1); the mono develop + STAGE-006/007
  oracles stay green.
- [ ] Panic-free lints hold; the analytic oracle is Tier-A (no corpus) so CI runs it. The
  optional `dnglab`-render cross-check is Tier-B (corpus-gated, loose) and never the
  accuracy gate.

## Failing Tests

In **`src/color.rs`** (`#[cfg(test)] mod tests`, private helpers — like `tone.rs` /
`demosaic.rs`):
- `"apply_matrix_is_bit_exact_for_injected_matrices"` — identity / diagonal / negative /
  overshoot `M`, hand-computed outputs incl. a round-half-up tie.
- `"quantize_clamps_before_rounding"` — negative → 0, > 65535 → 65535, `.5` → up.
- `"inv_3x3_round_trips_and_flags_singular"` — `inv(CM1)·CM1 == I` (bit-exact on a chosen
  exact matrix); a singular matrix → `MissingColorCalibration`.
- `"build_matrix_matches_independent_rederivation"` — FM path and CM path, `f64 ==`.

In **`tests/color_oracle.rs`** (public API, analytic, Tier-A):
- `"forward_matrix_path_matches_hand_computed"` — synthetic Sensor + known `FM1`/`asn`,
  pinned linear-sRGB output.
- `"color_matrix_path_matches_hand_computed"` — known invertible `CM1`/`asn`.
- `"a_neutral_subject_is_grey"` — neutral camera input → `R == G == B`.
- `"a_perturbed_forward_matrix_turns_the_oracle_red"` + `"the_honest_control_is_green"`.
- `"develop_color_composes_normalize_demosaic_matrix"` — end-to-end on a synthetic mosaic.
- `"wrong_lengths_and_missing_calibration_are_typed_errors"` — every error path, no panic.
- `"degenerate_dims_do_not_panic"` (1×1).
- *(Tier-B, `#[ignore]`-or-corpus-gated)* `"corpus_render_broadly_tracks_dnglab_srgb"` —
  loose sanity vs `dnglab analyze --srgb` (structural, not exact; explicitly not the
  accuracy oracle).

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- **Dual-illuminant interpolation (scene-adaptive matrix selection)** — [[SPEC-034]].
  This spec uses the **first** illuminant's matrices (`ColorMatrix1`/`ForwardMatrix1`)
  only, with no interpolation by scene white and no illuminant-white adaptation on the
  `ColorMatrix` path. NB the **fixed** D50→D65 Bradford step (the committed `C`) lives
  *here*, not in SPEC-034 — it is a constant every sRGB render needs, not a
  scene-dependent adaptation; SPEC-034 adds the *scene-adaptive* selection, not `C`.
- **ColorChecker ΔE accuracy oracle** — [[SPEC-035]]. This spec verifies the colour
  MATH bit-exact + a loose dnglab sanity cross-check; it does not measure end-to-end
  colour accuracy against a measured chart.
- Highlight reconstruction, denoise, advanced rendering intents (brief out-of-scope).
- Profile / HSV tone maps beyond the sRGB OETF (DEC-025); camera-profile curves;
  `AnalogBalance` / `CameraCalibration` (not parsed — assumed identity, documented in
  `DEC-033`).
- X-Trans / non-Bayer colour; a CLI (irr stays the dev/oracle binary).
- Applying `LinearizationTable` / `GainMap` — the plane is STAGE-006's.

## Notes for the Implementer

- **Pre-named split (the L-risk escape hatch).** Design is whole, but if the build finds
  it exceeds [M], ship the **matrix MATH** first — `src/color.rs` with
  `to_srgb_linear_into`, the committed `S`/`B` constants, `build_camera_to_srgb_linear`,
  `inv`, the quantize helper, the two `Error` variants, `DEC-033`, and the
  `src/color.rs` + `tests/color_oracle.rs` analytic oracle (inject a built `M`; the
  oracle needs no demosaic). Defer `develop_color_into` + `irr color` + the dnglab
  cross-check + the provenance row to a follow-on spec in this stage. That is the cut
  line; it is coherent and independently verifiable, exactly as [[SPEC-031]] shipped
  green before red/blue.
- **The scale-free apply is the simplifier.** Because full-scale `65535 ≡ 1.0`
  (`DEC-018`), `out_i = Σ M[i][j]·cam_j` directly — do **not** divide inputs by 65535 and
  re-multiply; that both wastes work and invites a rounding seam. Build `M` once, outside
  the pixel loop.
- **Determinism (`DEC-033`): `f64`, `+ − × ÷` only.** No `mul_add`/FMA, no transcendental
  on the transform path (the OETF stays `tone`'s committed table). Cite
  `tone::to_linear_f32_into`'s precedent. Commit `S`/`B` as documented `const` arrays with
  the standard they come from in a comment.
- **Per-channel levels run on the MOSAIC, before demosaic.** Black-subtract is per CFA
  colour and does not commute with interpolation, so normalize the mosaic (per
  `black_levels`/`white_levels`, scalar fallback) first; `demosaic_into` then runs on
  normalized values unchanged.
- **Guard the rationals.** `Rational.den` can be 0 and `num/den` can be ≤ 0 for a neutral;
  a required matrix/neutral that is absent, den-0, or yields a singular `ColorMatrix` is
  `MissingColorCalibration`, never a panic or a NaN written to `dst`.
- **Red-proof must mutate a REAL coefficient** (perturb an `FM1` entry in a mutated Sensor
  copy) and assert the OUTPUT changes — not a no-op assertion (`plane_oracle.rs` lesson,
  `DEC-017`).
- **Reuse, don't rebuild:** `demosaic_into`, `tone::encode_srgb16_into`, `develop`'s
  `normalize`, the `irr` P6 writer, `tests/support/*`. The full build context graph is in
  the handoff.

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
