---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-008
  status: proposed
  activity: requirements
  priority: medium
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-28
shipped_at: null

closed_reason: null

value:
  thesis: >
    irradiance already produces scene-linear `f32` with values above 1.0
    (`srgb_linear_plane_f32_into`, DEC-079), but `tone` can only display-encode
    `u16` in `[0, 1]`, so everything brighter than white is a hard clip. If `tone`
    gains a scene-linear `f32` entry that maps values at or above 0, including
    values above 1.0, through a fixed published tone curve and then encodes to
    8- and 16-bit sRGB through the committed `SRGB_ENCODE_*` tables, then a caller
    gets a displayable image with a rolled-off highlight from irradiance alone. The
    output stays byte-identical across platforms, `[dependencies]` stays empty,
    and nothing that exists today changes.
  beneficiaries:
    - "A desktop film-look app (the maintainer's, planned): DNGs and game screenshots tone mapped before a creative 3D LUT that lives in a separate crate"
    - "Any caller of the unclamped colour plane (`srgb_linear_plane_f32_into`): white-balance overs (up to 5.06% of pixels on the Pixel 4a, DEC-079) roll off instead of clipping"
    - "The browser demo (jysf/irradiance-demo): a highlight-preserving develop stage it can show beside the clipped one"
  success_signals:
    - "Extended Reinhard and Hable map every finite `f32` at or above 0 into `[0, 1]`, monotonic non-decreasing, within the oracle's tolerance of an independent `f64` recomputation of the published formula, and the oracle is shown red with a negative control"
    - "The new outputs are hashed in `tests/determinism.rs` and agree on the x86_64 CI leg with constants recorded on arm64, under a `TONE_MAP_VERSION` asserted beside them"
    - "NaN and ±inf input are refused with a typed `CallerError`-kind error naming the first bad index, before anything is written; a sweep over every sign and exponent, ±0, subnormals, `f32::MAX`, ±inf and NaN payloads never panics"
    - "Every existing `tone` output, `develop_into` and DEC-018's `u16` output are byte-identical to 0.2.7, the source scan still finds no transcendental in `src/tone.rs`, and `[dependencies]` stays empty"
    - "Scored at release, not gated: a held DNG with clipped highlights, developed through `srgb_linear_plane_f32_into` and the new path, shows highlight detail the existing `encode_srgb8_into` path clips"
  risks_to_thesis:
    - "Scope drift. A tone curve with knobs edges toward the 'competing with Lightroom on render quality' item future-plans rejects. Mitigation: fixed published constants, exposure and one white point only, no auto-exposure, and the thesis rests on irradiance's own outputs, not on screenshots"
    - "Two determinism bases in one module: DEC-033's basic-operations `f64` for the curve, DEC-025's table for the OETF. The hash cannot see a fused multiply-add, so the no-FMA rule needs its own injected-cancellation tests, as in `color.rs`"
    - "Per-channel mapping desaturates saturated highlights, and the app's LUT runs after it. If the look needs hue-preserving highlights, a luminance mode (with stated primaries) becomes a second project"
    - "No consumer exists yet. The app is planned, not built, so the API is shaped by the maintainer's description. Mitigation: a `#[non_exhaustive]` options struct and an enum of curves, so a later curve or option is additive"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap:
  - item: "AgX as a third curve, on a generated table indexed by the input `f32`'s top 16 bits (the exponent bits act as the log2), so no runtime transcendental is needed"
    kind: goal
    horizon: later
    resume_when: "a permissively licensed or specification-grade source for AgX's curve and matrices is found and recorded; the common game fits are believed to be under CC BY-NC-SA (Shadertoy) or GPL (Blender), which may not be read"
  - item: "A luminance-preserving mode with stated primaries (BT.709 / linear sRGB, the space DEC-079 fixes) and a gamut step"
    kind: goal
    horizon: later
    resume_when: "a consumer shows per-channel hue shift is a real problem in its output"
---

# PROJ-008: Scene-linear `f32` tone mapping

> **Proposed 2026-11-28.** This is a planning frame. SPEC-101 and DEC-089 are drafted at design (2026-11-28).
> Status stays `proposed` until jysf activates it. IDs were computed from `origin/main`
> (`17ca4f2`) on 2026-11-28 (next free: STAGE-039, SPEC-101, DEC-088; SPEC-100 took DEC-088 later that day, so this project's first DEC is DEC-089). PROJ-007 and
> STAGE-034 are open in parallel, so mint spec and DEC IDs with `just new-spec` at design
> time, never from this file.

## What This Project Is

This project adds a tone-mapping layer to `tone`. A new entry takes scene-linear `f32`
samples at or above 0, including values above 1.0, applies a fixed published curve per
channel (extended Reinhard, then Hable), and writes display-linear `f32` in `[0, 1]` to a
caller-owned `dst` (DEC-016). Two new encoders quantise that `[0, 1]` `f32` to `u16` and
look it up in the existing `SRGB_ENCODE_16` / `SRGB_ENCODE_8` tables, so the sRGB OETF
stays on DEC-025's committed-table basis. The work is purely additive: every existing
`tone` function, `develop_into` and DEC-018's `u16` output are untouched. The project ends
in its own 0.2.x release.

## Why Now

`tone` maps only `[0, 1]` scene-linear `u16` to sRGB, with no tone curve (DEC-025). Since
PROJ-006, irradiance itself emits values above 1.0: `srgb_linear_plane_f32_into` is
unclamped on purpose (DEC-079), with white-balance overs on up to 5.06% of pixels and
negative, out-of-gamut channels on up to 3.23%. A caller holding that plane has no way to
display it without a hard clip at white.

The trigger is the maintainer's planned film-look desktop app, whose pipeline is
`scene-linear RGB → tone map → sRGB encode → creative 3D LUT → output`. The LUT
(`lut-rs`) and screenshot decoding are the app's, not irradiance's.

This is not `docs/future-plans.md` writer extension 5 ("Float or demosaiced LinearRaw RGB
output"). That item is about the DNG writer emitting float DNGs. No existing future-plans
item covers tone mapping, so this project adds one. DEC-025 recorded that STAGE-003
excluded "LUT / film response curves", which deferred film curves rather than rejecting
them.

## Answers that set the scope (jysf, 2026-11-28)

These were the recommended defaults in the planning discussion, all accepted.

1. **Negative input is clamped to 0, NaN and ±inf are refused.** Refusing negatives would
   make the library's own unclamped plane an error on real files (DEC-079's Canon 5D Mark
   III), which is DEC-018 Option D's rejected shape. The clamp is documented as the only
   gamut handling, the same clip the `u16` colour path applies at quantise. NaN and ±inf
   are refused with a typed error naming the first bad index. Its `ErrorKind` is
   `CallerError`, since the buffer is the caller's argument, not a file (DEC-051).
2. **The curve is computed at runtime in `f64` with basic operations only** (DEC-033's
   basis: `+ − × ÷`, committed constants, no FMA, no transcendental), then cast once to
   `f32`. The `f64` path also removes overflow: extended Reinhard's `x²` overflows `f32`
   above about 1.8e19, while `f32::MAX²` is finite in `f64`. A generated table indexed by
   the input's top 16 bits is the recorded alternative, and the path to AgX (roadmap).
3. **Curves: extended Reinhard first, then Hable.** Both are rational and both have a
   published source. AgX is deferred on provenance.
4. **Per-channel mapping for v1.** It keeps `tone`'s per-sample shape, which needs no
   knowledge of primaries or layout, so the generic entry stays generic.
5. **No fuzz target.** The entry reads floats with no offsets or indexing. A deterministic
   sweep over every sign and exponent with sampled mantissas, plus ±0, subnormals,
   `f32::MAX`, ±inf and NaN payloads, proves the no-panic and typed-error claims more
   directly. The reasoning goes in the DEC.
6. **Scope: irradiance's own outputs are the thesis; any `f32` buffer is the API.**
   `tone` already accepts any `u16` buffer with no `Sensor`, so a buffer-generic `f32`
   entry has precedent within `library-not-application`. Parameters are a typed,
   `#[non_exhaustive]` options struct (DEC-058's precedent): the curve, a linear exposure
   multiplier, and a white point for extended Reinhard. There is no auto-exposure, because
   a log-average key is a transcendental and a reduction over the image (DEC-002), and
   Hable's shoulder constants are not exposed.

## The app author's review (2026-11-28, planning only)

The film-look app's author reviewed the plan; nothing of theirs is built yet. They accepted,
as proposed: the clamp at 0 (to be documented), refusing NaN and ±inf, per-channel mapping,
the 16-bit quantisation before the sRGB tables, the defaults, a linear exposure multiplier,
no in-place form, and Reinhard before Hable. They asked for:

1. Hable's linear white `W` (usually 11.2) exposed as an option, or recorded as fixed and at
   what value. SPEC-103.
2. Each curve's formula and constants documented exactly, so the app's GPU preview shader
   can match the CPU output within a tolerance. SPEC-101 AC11, and SPEC-103 for Hable.
3. `white_point = 1.0` allowed and the identity on `[0, 1]`, with a test. SPEC-101 AC1.
4. Slicing the input (row bands) gives bit-identical output, documented and tested.
   SPEC-101 AC10.
5. The docs state that input 1.0 is SDR (diffuse) white. SPEC-101 AC11, with a caveat:
   irradiance's own colour plane puts 1.0 at the sensor's white level, so its callers place
   diffuse white with `exposure`.
6. A doc line that every channel is mapped, alpha included, so callers pass RGB only.
   SPEC-101 AC11.

## Facts found while framing (2026-11-28, read-only)

- **Today's `tone`.** Three encoders on `&[u16]`: `to_linear_f32_into` (`v / 65535.0`),
  `encode_srgb16_into` and `encode_srgb8_into` (65536-entry `const` tables generated with
  `powf` by `examples/gen-tone-table.rs`). The only error is `OutputBufferWrongLength`.
  The oracle (`tests/tone_oracle.rs`) recomputes the OETF in `f64` within ±1, with an
  in-process red-proof on a mutated copy of the table (DEC-021's shape).
- **The source scan has a gap.** `no_transcendental_on_the_encode_path` bans `.powf(`,
  `.powi(`, `.exp(`, `.exp2(`, `.ln(`, `.log(`, `.sqrt(` and `.cbrt(`, but not `.log2(`,
  `.log10(`, `.tanh(`, `.mul_add(` or the trigonometric calls. SPEC-101 widens it, since a
  runtime curve now lives in `src/tone.rs`.
- **`tone` has no version constant.** `DEVELOP_VERSION` (`"8"`) and `MONO_MIX_VERSION`
  (`"1"`) exist and are asserted beside their determinism hashes. The new curves get
  `TONE_MAP_VERSION`, so a later change of curve constants is a version bump, as DEC-002
  proposes.
- **`Error` is `#[non_exhaustive]`**, so new variants are additive. Each needs a row in
  `Error::kind`'s wildcard-free match (DEC-051).
- **Quantising before the OETF costs nothing against today.** The tone-mapped `[0, 1]`
  `f32` is quantised to `u16` (round to nearest, DEC-018's rule) before the table lookup,
  which is the same precision the existing `u16` encode path has.

## Provenance (to be recorded in `docs/provenance-ledger.md` per curve)

| Curve | Source | Class | Check at design |
|---|---|---|---|
| Extended Reinhard, `L(1 + L/Lw²) / (1 + L)` | Reinhard, Stark, Shirley, Ferwerda, "Photographic Tone Reproduction for Digital Images", ACM TOG 21(3), SIGGRAPH 2002 | 2, published paper | the equation number and form against the paper |
| Hable / Uncharted 2, `((x(Ax+CB)+DE) / (x(Ax+B)+DF)) − E/F`, normalised by its value at `W` | John Hable, "Uncharted 2: HDR Lighting", GDC 2010, and his "Filmic Tonemapping Operators" post (2010) | 2, published talk (record honestly: a talk with formulas, not a peer-reviewed paper) | the constants A–F and `W = 11.2` against the slides, and whether the commonly copied exposure bias of 2.0 is part of the operator |

The constants above are as remembered and are not yet checked against the primary
sources. No implementation of either curve may be read unless its licence is permissive and
recorded. The ledger needs no copyleft source for either.

## Success Criteria

See `value.success_signals`. In short: both curves map every finite non-negative `f32`
into `[0, 1]` against an independent oracle shown red, the new outputs are hashed and agree
across x86_64 and arm64, bad floats are refused without a panic or a partial write, and
nothing that exists today changes.

## Scope

### In scope
- A scene-linear `f32` tone-map entry in `tone`: a typed `#[non_exhaustive]` options struct
  with a `Default`, a curve enum, exposure, and the extended-Reinhard white point, each with
  a stated valid range and a typed refusal outside it.
- Extended Reinhard and Hable, per channel, in basic-operations `f64`.
- Two encoders from `[0, 1]` `f32` to 16- and 8-bit sRGB through the existing tables.
- Typed errors for a length mismatch (the existing variant), non-finite input and invalid
  options. Nothing is written unless every check passes, so the input is scanned first.
- A `TONE_MAP_VERSION` constant, new rows and hashes in `tests/determinism.rs` and
  `docs/determinism.md`, and injected-cancellation tests pinning no-FMA for the curve.
- Provenance rows, a new `docs/future-plans.md` entry, and one 0.2.x release.

### Explicitly out of scope
- The 3D LUT and `.cube` parsing (the separate `lut-rs` crate).
- Decoding screenshot formats (PNG, JPEG, JXR, EXR) and any I/O (`library-not-application`;
  the `image` crate is forbidden).
- AgX (roadmap, `later`), a luminance-preserving mode and any gamut mapping beyond the
  clamp at 0 (roadmap, `later`).
- Auto-exposure, histograms or any reduction over the image (DEC-002).
- Exposing Hable's shoulder and toe constants, or any other tuning surface.
- Any change to `develop_into`, DEC-018's `u16` output, the existing `tone` encoders or the
  committed tables.

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [ ] STAGE-039 (proposed) — Tone-map core. Three specs, one per PR: SPEC-101, the options
  struct, the `f32` entry, its refusals and extended Reinhard, with the oracle, red-proof,
  sweep test and hashes; SPEC-102, the `[0, 1]` `f32` to sRGB16/sRGB8 encoders through the
  existing tables; SPEC-103, Hable. ~125M tokens / ~$90.
- [ ] STAGE-040 (proposed) — The 0.2.x release of the tone mapper. ~45M / ~$30.

**Count:** 0 shipped / 0 active / 2 pending

**Cost estimate.** About 170M tokens / $120 for the project. The baseline is PROJ-005's
actuals: an [S] spec 13–20M, an [M] spec 17–36M, stage orchestration about 0.7× the spec
total, a release stage about 45M, and about $0.73 per million tokens. Here two [S] specs
(SPEC-102, SPEC-103) and one [M] (SPEC-101) are assumed. The estimate is borrowed, so
re-estimate at the stage's first design.

## Dependencies

### Depends on
- PROJ-001 (shipped): `tone`, its committed tables and oracle (DEC-025), DEC-016, DEC-018.
- PROJ-002 / PROJ-006 (shipped work): the `f64` basic-operations basis (DEC-033), the
  unclamped `f32` plane (DEC-079) and the colour determinism fixture.
- No external tool. Both oracles are analytic, recomputed in `f64` in the test.

### External inputs (not `depends_on`)
- The film-look app is planned, not built. Its author answered the API questions on
  2026-11-28 (above); a later answer re-frames SPEC-102 or SPEC-103 before their design.
- The primary sources for both curves, read at design to check the constants above.

### Enables
- The film-look app's tone-map stage; a highlight-preserving demo stage; a later AgX or
  luminance mode on the same options struct.

## Project-Level Reflection

*Filled in when status moves to shipped.*
