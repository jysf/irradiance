---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-024
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
  stage: STAGE-006
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-025, SPEC-026, SPEC-027]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-006's 'a Bayer DNG is accepted by the sensor gate' — the foundation SPEC-025/026/027 build on."

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
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-024: CFA sensor identity + `Cfa` mosaic model: accept `PhotometricInterpretation == 32803`; parse `CFAPattern`/`CFARepeatPatternDim`/`CFAPlaneColor`/`CFALayout` into a new `Sensor` field; mono stays `None`; mono path unchanged (regression guard)

> **DESIGN — `cycle: design`.** Approach + failing tests below; a build session
> makes the Failing Tests pass without redesigning. SPEC-025/026/027 depend on this.

## Context

The sensor gate accepts only a monochrome `LinearRaw` plane: `is_sensor_ifd`
(`src/ifd.rs:1035`) requires `PhotometricInterpretation == 34892`, and nothing in
the codebase parses a CFA. A Bayer DNG is `PhotometricInterpretation == 32803
(CFA)` carrying a `CFAPattern`, so `sensor()` rejects it outright today. This is
the foundation spec of `STAGE-006` (PROJ-002): it opens the gate to CFA and gives
`Sensor` a structured mosaic model, on top of PROJ-001's IFD/tag reader. Nothing
reads a Bayer file until this ships — SPEC-025/026/027 all depend on it. Verified
against the two corpus Bayer DNGs (`CANON-EOS-5D-MARK-III/5G4A9394.DNG`,
`NIKON-D300S/_DSC4615.DNG`), both RGGB 2×2, 16-bit.

## Goal

Accept a Bayer-CFA DNG at the sensor gate — `PhotometricInterpretation == 32803`,
with `SamplesPerPixel == 1` unchanged (a mosaic is one sample per pixel; colour is
encoded by position) — and expose a structured CFA model on `Sensor` (e.g.
`color_filter: Option<Cfa>` carrying `CFAPattern` / `CFARepeatPatternDim` /
`CFAPlaneColor` / `CFALayout`), `None` for the monochrome Leica. The monochrome
path stays byte-for-byte unchanged.

## Inputs

- **Files to read:**
  - `src/ifd.rs:1035` `is_sensor_ifd` — the identity gate (NewSubfileType /
    Photometric / SamplesPerPixel → `SensorMatch`); the one disjunct to add.
  - `src/ifd.rs:508` `struct Sensor` — where `color_filter` is added; and
    `src/ifd.rs:1156` `sensor()` — where it is built. Copy the shape of the
    `black_level` / `active_area` reads at `:1201–1245` (`cost_the_field` + the
    accessors) exactly.
  - `src/ifd.rs:95–138` the tag-constant block + `PHOTOMETRIC_LINEAR_RAW`.
  - Accessors: `scalar` (:960), `array::<N>` (:988), `values`/`uints` (:862),
    `entry`/`payload` — reading a SHORT, a fixed array, a byte array.
  - `src/ifd.rs:1344` the `Build` test DSL + `sensor(&b)` helper (:1409) and
    `header_reads_both_byte_orders` (:1430) — the pattern the new tests mirror.
    `src/lib.rs:515` the `NoSensorIfd` Display text.
  - DEC-012 (structural vs interpretation tags); AGENTS.md §11.
- **Corpus:** `CANON-EOS-5D-MARK-III/5G4A9394.DNG`, `NIKON-D300S/_DSC4615.DNG`
  (both `CFARepeatPatternDim 2 2` RGGB, 16-bit); `LEICA-Q2-MONO/*.DNG` for the mono
  regression.
- **Spec:** DNG 1.x CFA tag definitions, to confirm the tag numbers/types below.

## Outputs

- **Files modified:** `src/ifd.rs`, `src/lib.rs`. No new files, no migrations, no flags.
- **New constants** (`src/ifd.rs`, in the tag block; confirm each against the DNG spec):
  - `pub const PHOTOMETRIC_CFA: u32 = 32803;`
  - `pub const TAG_CFA_REPEAT_PATTERN_DIM: u16 = 33421;` (2 × SHORT)
  - `pub const TAG_CFA_PATTERN: u16 = 33422;` (`rows*cols` × BYTE/UNDEFINED)
  - `pub const TAG_CFA_PLANE_COLOR: u16 = 50710;` (N × BYTE)
  - `pub const TAG_CFA_LAYOUT: u16 = 50711;` (SHORT; default 1 = rectangular)
- **New public type** (`src/ifd.rs`): `#[derive(Debug, Clone, PartialEq, Eq)]
  pub struct Cfa { pub repeat_dim: [u32; 2], pub pattern: Vec<u8>,
  pub plane_color: Vec<u8>, pub layout: u32 }`. `pattern` / `plane_color` carry the
  DNG colour-index encoding (0=Red, 1=Green, 2=Blue, …) — no typed enum (Non-Goal).
- **`Sensor` field:** `pub color_filter: Option<Cfa>` — `None` for a mono plane.
- **Gate change (`is_sensor_ifd`):** accept
  `photometric ∈ { PHOTOMETRIC_LINEAR_RAW, PHOTOMETRIC_CFA }`; the
  `SamplesPerPixel == 1` and `NewSubfileType == 0` checks are **unchanged**.
- **`sensor()` change:** a private `parse_cfa(ifd, &mut malformed) -> Option<Cfa>`
  gated on `photometric == PHOTOMETRIC_CFA`, built with `cost_the_field` so a
  malformed CFA tag on a CFA plane costs that field (→ `None`) and is recorded in
  `malformed_tags` (DEC-012), never failing the file. Interpretation-class, like
  `black_level`; the bit-exact plane (SPEC-025) reads regardless of CFA-metadata health.
- **`src/lib.rs:515`:** extend the `NoSensorIfd` Display text to name CFA beside LinearRaw.

## Acceptance Criteria

- [ ] A CFA IFD (`photometric == 32803`, `SamplesPerPixel == 1`,
  `NewSubfileType == 0`) is `SensorMatch::Yes`, `sensor_candidates()` returns
  exactly one for a Bayer file, and `sensor()` is `Ok`.
- [ ] For a synthetic RGGB plane, `color_filter == Some(Cfa { repeat_dim: [2, 2],
  pattern: vec![0, 1, 1, 2], plane_color: vec![0, 1, 2], layout: 1 })`; both corpus
  Bayer DNGs give `Some` with `repeat_dim == [2, 2]`.
- [ ] A monochrome `LinearRaw` plane gives `color_filter == None`, and no other
  `Sensor` field changes (the existing Q2M `sensor()`/plane tests stay green).
- [ ] A CFA plane with a **malformed** `CFAPattern` still returns `Ok`:
  `color_filter == None` and `TAG_CFA_PATTERN ∈ malformed_tags`.
- [ ] A CFA plane with the CFA tags **absent** records nothing in `malformed_tags`
  (absent ≠ malformed) and gives `color_filter == None`.
- [ ] A CFA IFD with `SamplesPerPixel != 1` is still `SensorMatch::No`.
- [ ] Panic-free on a truncated/garbage CFA tag payload (typed result, no panic).
- [ ] `color_filter` is byte-order-agnostic (II and MM give identical values).

## Failing Tests

All in `src/ifd.rs` `mod tests`, mirroring `sensor(&b)` /
`header_reads_both_byte_orders`. Add a `cfa_sensor(&b)` helper (photometric 32803,
samples 1, the four CFA tags — every count ≤ 4 bytes, so they pack inline in the
value word; no external data block needed).

- **`src/ifd.rs` `mod tests`**
  - `a_cfa_ifd_is_selected_as_the_sensor_plane` — `c.sensor()` is `Ok` and
    `c.sensor_candidates().len() == 1`.
  - `a_cfa_plane_exposes_its_mosaic_model` — `color_filter == Some(Cfa {
    repeat_dim: [2, 2], pattern: vec![0, 1, 1, 2], plane_color: vec![0, 1, 2],
    layout: 1 })`.
  - `a_cfa_mosaic_model_is_byte_order_agnostic` — build the CFA plane in II and MM;
    identical `color_filter` (mirrors `header_reads_both_byte_orders`).
  - `a_monochrome_plane_has_no_color_filter` — the existing `sensor(&b)` LinearRaw
    plane → `color_filter == None` and no CFA tag in `malformed_tags`.
  - `a_cfa_plane_with_more_than_one_sample_is_not_selected` — photometric 32803,
    samples 2 → `sensor()` is `Err(NoSensorIfd)`.
  - `a_malformed_cfa_pattern_costs_the_field_not_the_file` — `CFAPattern` as a bad
    type → `sensor()` `Ok`, `color_filter == None`, `malformed_tags` contains
    `TAG_CFA_PATTERN`.
  - `absent_cfa_tags_on_a_cfa_plane_are_not_malformed` — CFA photometric, no CFA
    tags → `Ok`, `color_filter == None`, no CFA tag in `malformed_tags`.
- **Regression (must stay green — existing, not new):** the Q2M plane + develop
  oracles and the current `sensor()` unit tests.

## Non-Goals

- Reading the mosaic bytes or the bit-exact plane oracle — that is SPEC-025
- Parsing the colour matrices / white balance / per-channel levels — SPEC-026
- Any demosaic, colour transform, or level application — later stages
- X-Trans / non-Bayer CFA, and native vendor containers
- Choosing the one-`Sensor`-type-with-optional-`Cfa` vs a type split as a DEC —
  design-cycle call; promote to a DEC only if it proves weighty (stage Design Notes)

## Notes for the Implementer

- **Do not touch `SamplesPerPixel == 1`.** A Bayer mosaic is one sample per pixel;
  colour is encoded by position. The only identity change is the photometric disjunct.
- **Parse CFA exactly like `black_level` / `active_area`** (`sensor()` :1201–1245):
  `cost_the_field` + the `array`/`values` accessors, preserving DEC-012's
  structural-vs-interpretation split. CFA metadata is **interpretation** —
  best-effort, costs its field, never fails the file — so the bit-exact plane
  (SPEC-025) reads even on a CFA file with odd metadata.
- **Tag types:** `CFARepeatPatternDim` = 2 SHORT (`array::<2>`); `CFAPattern` =
  `rows*cols` BYTE/UNDEFINED (`values`/`uints`); `CFAPlaneColor` = N BYTE;
  `CFALayout` = SHORT (`scalar`, default 1).
- **Keep the mono path byte-for-byte.** Gate the CFA parse on
  `photometric == PHOTOMETRIC_CFA` so a LinearRaw plane never reads a CFA tag; the
  Q2M oracles are the regression guard.
- **Design decision (record if wanted):** one `Sensor` type with
  `color_filter: Option<Cfa>`, not a mono/CFA split — additive, mono untouched.
  Adds public API (`Cfa`, `Sensor::color_filter`, the new consts); if it should be
  a DEC, reconcile the number — DEC-029 is informally reserved for the crustyimg
  LUT boundary.
- The full context graph lives in the handoff file when this is dispatched to build.

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
