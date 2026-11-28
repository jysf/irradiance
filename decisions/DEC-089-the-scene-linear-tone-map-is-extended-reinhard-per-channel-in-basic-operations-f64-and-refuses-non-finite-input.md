---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-089
  type: decision
  confidence: 0.8
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-008
repo:
  id: irradiance

created_at: 2026-11-28
supersedes: null                   # additive; DEC-025, DEC-033 and DEC-079 stay active and unamended
superseded_by: null
status: proposed                   # at SPEC-101's design, 2026-11-28; accepted at its ship
deciders: [claude, jysf]

affected_scope:
  - src/tone.rs
  - src/lib.rs
  - tests/tone_map.rs
  - tests/tone_map_oracle.rs
  - tests/determinism.rs
  - projects/PROJ-008-scene-linear-f32-tone-mapping/specs/SPEC-101-extended-reinhard-tone-map-on-scene-linear-f32.md

tags:
  - tone
  - tone-map
  - determinism
  - f32
  - stage-039
---

# DEC-089: the scene-linear tone map is extended Reinhard, per channel, in basic-operations `f64`; it clamps negatives to 0 and refuses non-finite input

## Decision

1. **One entry, one options struct.** `tone::tone_map_f32_into(src: &[f32], dst: &mut [f32],
   options: &ToneMapOptions)` writes display-linear `f32` in `[0, 1]` into a caller-owned
   `dst` (`DEC-016`). `ToneMapOptions` is `#[non_exhaustive]` with a `Default`, following
   `DEC-058`'s shape. It holds `curve: ToneMapCurve` (a `#[non_exhaustive]` enum, so a later
   curve is additive), `exposure: f64` and `white_point: f64`. The enum is `ToneMapCurve`, not
   `ToneCurve`, because `DEC-025` already calls the sRGB OETF "the tone curve".

2. **Refusal order, and nothing written on refusal.** The checks run in this order:
   - exposure: `InvalidToneMapExposure { exposure }`;
   - white point: `InvalidToneMapWhitePoint { white_point }`;
   - length: the existing `OutputBufferWrongLength { expected: src.len(), actual: dst.len() }`;
   - input: `NonFiniteToneMapInput { index }` for the lowest index holding NaN or ±inf.

   All three new variants are `ErrorKind::CallerError` (`DEC-051`): the buffer and the options
   are the caller's arguments, not a file's bytes. The input is scanned in full before the
   first write, so `dst` is untouched on every refusal.

3. **The curve and its arithmetic.** The curve is Reinhard et al.'s extended operator (UUCS-02-01 /
   SIGGRAPH 2002, §3.1, Eq. 4), with `e = exposure` and `w = white_point`, in this exact order:
   `x ≤ 0 → +0.0`; `l = f64::from(x)·e`; `l ≥ w → 1.0`;
   otherwise `((l·(1 + l/(w·w))) / (1 + l)) as f32`.
   - This is `DEC-033`'s basis: `f64` `+ − × ÷` only, no transcendental, no `mul_add`, one cast
     to `f32` at the end. The form holds no `a·b + c` pair, so no FMA can apply and no
     injected-cancellation test is needed. The source scan's `.mul_add(` ban is the pin.
   - `tone` now has two determinism bases: `DEC-025`'s committed table for the OETF, this one
     for the tone map. `TONE_MAP_VERSION` (`"1"`) is bumped by any change to the curve's
     arithmetic or constants, as `DEC-002` proposes for table-driven curves.

4. **Options and their ranges.**
   - `exposure` and `white_point` must each be finite and in `[2⁻²⁴, 2²⁴]`, both ends
     inclusive, published as `TONE_MAP_PARAM_MIN` and `TONE_MAP_PARAM_MAX`. Within that
     range `w·w` neither underflows nor overflows, and `f32::MAX · 2²⁴` stays finite in `f64`,
     so no intermediate is infinite or NaN.
   - The defaults are `exposure = 1.0` and `white_point = 4.0`, two stops above diffuse white.
     The paper's default `L_white`, the scene maximum, is a reduction over the image (`DEC-002`
     names reductions as a determinism hazard), so it is not used.
   - There is no auto-exposure, for the same reason: the paper's log-average key (Eq. 1) is a
     transcendental and a reduction. A caller wanting it computes it and passes `exposure`.

5. **Negatives clamp to `+0.0`; NaN and ±inf are refused.**
   - Negatives are legal output of the library's own `srgb_linear_plane_f32_into` (`DEC-079`:
     up to 3.23% of pixels on the Canon 5D Mark III), so refusing them would make a real file an
     error. That is `DEC-018` Option D's rejected shape.
   - The clamp is the same clip the `u16` colour path applies at quantise, and it is this
     curve's only gamut handling.
   - Non-finite values are not produced by any irradiance path, and they have no defined
     display value. Passing them through would put NaN into an 8-bit encode, and clamping them
     would hide a caller's bug.

6. **Per channel.** The curve is applied to each sample independently. Eq. 4 is stated for
   luminance, so this is a recorded deviation. It keeps `tone`'s per-sample, layout-free shape
   (the existing encoders take any buffer with no `Sensor`). It also desaturates bright
   saturated colours toward white. A luminance mode with stated primaries is PROJ-008's
   roadmap item.

7. **No fuzz target.** The entry reads `f32` values and options, with no offsets, lengths read
   from data, or indexing. Its whole input space is sign × exponent × mantissa, so a
   deterministic sweep covers it more directly than coverage-guided fuzzing. That sweep
   (`tests/tone_map.rs`) covers:
   - every sign and exponent, with nine mantissas, under five option sets at the range
     corners;
   - ±0, subnormals, `f32::MAX`, ±inf and quiet, signalling and negative NaN payloads.

   The exhaustive 2³¹-input sweep is too slow for CI's debug profile. It ran once at design, in
   release, and its result is recorded in SPEC-101.

8. **The contract is documented exactly, and slicing changes nothing.** At the film-look
   app author's request (2026-11-28):
   - Each curve's rustdoc states its formula in the operation order of item 3, with every
     constant, so a GPU preview can be written to match within a tolerance it chooses. A GPU
     `f32` port is not expected to match bit for bit.
   - `white_point = 1.0` is valid and is the identity on `[0, 1]` (it clips above 1).
   - The scale convention is that 1.0 is SDR (diffuse) white, as in scRGB. irradiance's own
     `srgb_linear_plane_f32_into` puts 1.0 at the sensor's white level instead, so its
     caller places diffuse white with `exposure`. The docs say both.
   - Every sample is mapped, so an interleaved alpha channel would be tone mapped too.
     Callers pass colour channels only.
   - Output depends on each sample alone, so mapping the input in any slices (row bands) is
     bit-identical to mapping it whole. A refusal's index is relative to the slice passed.

## Context

`PROJ-008` adds a tone-mapping layer to `tone` for scene-linear `f32`, including the values
above 1.0 that `srgb_linear_plane_f32_into` emits on purpose (`DEC-079`). The scope answers in
the brief (jysf, 2026-11-28) fixed the direction: clamp negatives, refuse non-finite values,
runtime `f64`, extended Reinhard first, per channel, no fuzz target, and an options struct with
exposure and one white point. This record fixes the details SPEC-101 builds against.

A design probe (SPEC-101, "Design probe") ran item 3 over every one of the 2,139,095,040
non-negative finite `f32` inputs under seven option sets, including the range corners. Every
output was in `[0, 1]`, the outputs were exactly monotonic, and the formula never exceeded 1.0
below `w`. The results matched an independent algebraic form bit for bit, except at 2 inputs
that were 1 ulp apart.

## Alternatives Considered

- **Option A: a generated table indexed by the input `f32`'s top 16 bits.**
  - What it is: `DEC-025`'s basis, with no runtime curve. The exponent bits act as a `log2`.
  - Why rejected for now: it quantises the input to 7 mantissa bits, and it needs a generator
    and a committed table per option set, which a runtime `exposure` and `white_point`
    defeat. It is kept as the path to AgX, which needs a `log2` (PROJ-008 roadmap).
- **Option B: `f32` arithmetic.**
  - Why rejected: `l·l` overflows `f32` above about 1.8e19. The probe found the `f32` form
    differs from the `f64` form on 4.7M–116M inputs per configuration, so `f32` would also
    be the less accurate of the two.
- **Option C: refuse negatives, or pass them through the formula.**
  - Why rejected: refusing them turns `DEC-079`'s plane into an error on real files.
    Passing them through gives `−0.96875` for `x = −0.5` at `w = 4`, which is outside
    `[0, 1]` and breaks the entry's one guarantee.
- **Option D: default `white_point` = +∞ (simple Reinhard, Eq. 3) or 1.0.**
  - Why rejected: Eq. 3 never reaches white, so a white-balanced white would display as 0.5.
    With `w = 1` the curve is the identity below 1, which is the clip this project exists to
    replace. 4.0 rolls off two stops of overs and maps 1.0 to 0.53125. That value is
    judgement, accepted by the app's author (open question 1).
- **Option E: a fuzz target.** Rejected as in item 7: the sweep is exhaustive over the
  structure of the input, while a fuzzer would only sample it.
- **Option F (chosen):** items 1–7.

## Consequences

- **Positive:**
  - Values above 1.0 roll off into `[0, 1]`.
  - The output is byte-identical across arches by `DEC-033`'s argument, and the hash checks
    it.
  - Every bad input is refused with a type that names it.
  - Adding a curve (Hable, SPEC-103) adds an enum variant and does not break callers.
- **Negative:**
  - The docs carry a scale caveat: for irradiance's own plane, 1.0 is the sensor's clip
    point rather than diffuse white.
  - `tone` now holds runtime float arithmetic, so its "the table is the point" story has two
    halves.
  - Per-channel mapping shifts hue in bright saturated colours.
  - The 4.0 default is a judgement with no measurement behind it, accepted by the app's author.
- **Neutral:**
  - The output is display-linear `f32`. Encoding it to sRGB is SPEC-102's work, through the
    existing tables.

## Validation

- SPEC-101's oracle agrees within 1 `f32` ulp and is shown red on each designed fault.
- The hashes agree between the arm64 recording and the x86_64 leg.
- At release (PROJ-008's last success signal), a held DNG with clipped highlights shows
  highlight detail through this path that `encode_srgb8_into` clips.

Revisit:
- if the hashes disagree across arches (`DEC-033`'s argument would then be wrong for this
  form);
- if a consumer needs hue-preserving highlights (the luminance mode);
- if the 4.0 default proves wrong in practice.

## Open questions

Answered by the film-look app's author on 2026-11-28 (planning only; their app is not built):

1. **`white_point = 4.0` as the default:** accepted.
2. **An in-place form:** not wanted.
3. **The `[2⁻²⁴, 2²⁴]` ranges and a linear exposure multiplier:** accepted.

Still open, for SPEC-103:

4. **Hable's linear white `W`** (11.2 in Hable's published version): exposed as an option, or
   fixed and documented at its value? The author asked for one or the other, explicitly
   recorded. If it is exposed, SPEC-103 decides whether it reuses `white_point` or gets its
   own field; until then `white_point` is validated under every curve, so the refusal rule
   does not depend on the curve.

## References

- Related specs: SPEC-101 (this), SPEC-102, SPEC-103 (planned), SPEC-019, SPEC-093
- Related decisions: DEC-002, DEC-009, DEC-016, DEC-018, DEC-021, DEC-025, DEC-033, DEC-051,
  DEC-058, DEC-079
- Source: E. Reinhard, M. Stark, P. Shirley, J. Ferwerda, "Photographic Tone Reproduction for
  Digital Images", University of Utah UUCS-02-01, 14 January 2002, §3.1, Eqs. 1–4; published
  in ACM TOG 21(3), SIGGRAPH 2002.
