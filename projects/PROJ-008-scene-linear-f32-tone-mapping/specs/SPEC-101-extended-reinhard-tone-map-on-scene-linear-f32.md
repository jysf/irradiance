---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-101
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
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
  id: PROJ-008
  stage: STAGE-039
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-089, DEC-016, DEC-025, DEC-033, DEC-051, DEC-058, DEC-079, DEC-009, DEC-021]
  constraints: [no-panics-on-untrusted-input, provenance-recorded-per-algorithm, oracle-must-be-shown-red, test-before-implementation, one-spec-per-pr, library-not-application, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-019, SPEC-093]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-039's <capability>". Optional; null is acceptable.
value_link: "STAGE-039: the scene-linear f32 entry and its first curve, so values above 1.0 roll off into [0, 1] instead of clipping"

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
  tokens_estimate: 30000000         # [M], PROJ-005's [M] range 17-36M
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-101: Extended Reinhard tone map on scene-linear f32

## Context

`PROJ-008` (proposed), `STAGE-039`, the first of three specs. Today `tone` maps only `u16`
in `[0, 1]` (`src/tone.rs:73`, `:91`, `:116`), while `color::srgb_linear_plane_f32_into`
(`src/color.rs:715`) emits unclamped `f32` on purpose (`DEC-079`): values above 1.0 on up to
5.06% of pixels (Pixel 4a), negatives on up to 3.23% (Canon 5D Mark III). A caller holding
that plane can display it only by clipping at white.

The film-look app's author reviewed the plan on 2026-11-28 (planning only; nothing of
theirs is built). They accepted the clamp at 0, the refusal of NaN and ±inf, per-channel
mapping, the defaults, a linear exposure multiplier, no in-place form and Reinhard before
Hable. They asked for six things; this spec takes items 2–6 (AC10, AC11), and item 1 (Hable's
white) goes to SPEC-103:

1. Hable's linear white `W` exposed, or recorded as fixed and at what value (SPEC-103).
2. Each curve's formula and constants documented exactly, so a GPU preview shader can match
   the CPU output within a tolerance.
3. `white_point = 1.0` allowed, mapping `[0, 1]` to itself exactly, with a test.
4. Slicing the input (row bands) gives bit-identical output to the whole buffer, documented
   and tested.
5. The docs state that input 1.0 means SDR (diffuse) white.
6. A doc line that every channel is mapped, alpha included, so callers pass RGB only.

This spec adds the entry, its options and refusals, and the first curve. Its rules live in
`DEC-089` (proposed with this spec). SPEC-102 (the sRGB encoders for `[0, 1]` `f32`) and
SPEC-103 (Hable) follow in their own PRs.

## Goal

`tone::tone_map_f32_into` maps every finite `f32` sample (any sign, any magnitude) into
display-linear `[0, 1]` through Reinhard et al.'s extended operator (2002, Eq. 4), per sample,
in basic-operations `f64`. Non-finite input and out-of-range options are refused with typed
errors before anything is written.

## Inputs

- **Files to read:** `src/tone.rs` (the module this extends, and its source scan at
  `:232-266`); `src/lib.rs` (`Error` at `:327`, `OutputBufferWrongLength` at `:903`,
  `Error::kind`'s wildcard-free match at `:1977-2050`, `Display` at `:1675`);
  `tests/determinism.rs` (`hash_f32` at `:70`, `colour_planes` at `:541`, the colour
  red-proof at `:609`); `tests/error_kind.rs` (the variant table and its count comment at
  `:23-40`); `docs/determinism.md` (the `tone` rows at `:53-54`);
  `docs/provenance-ledger.md` (the sRGB OETF row, `:49`).
- **The primary source:** Reinhard, Stark, Shirley, Ferwerda, "Photographic Tone
  Reproduction for Digital Images", University of Utah tech report UUCS-02-01, 14 January
  2002 (published as ACM TOG 21(3), SIGGRAPH 2002), §3.1, **Eq. 4**:
  `Ld = L·(1 + L/L_white²) / (1 + L)`, where `L_white` is "the smallest luminance that will
  be mapped to pure white". Read at design from
  `www-old.cs.utah.edu/docs/techreports/2002/pdf/UUCS-02-001.pdf`.
- **External APIs:** none.

## Outputs

- **Files modified:**
  - `src/tone.rs`: the new items below, the module docs (a fourth representation; the
    determinism section now names two bases), and the widened source scan.
  - `src/lib.rs`: three `Error` variants, their `Display` arms and their `kind` rows.
  - `tests/determinism.rs`: a tone-map hash, its committed constants and red-proof rows.
  - `tests/error_kind.rs`: three rows and the count comment.
  - `docs/determinism.md`: a row for `tone::tone_map_f32_into`.
  - `docs/provenance-ledger.md`: a row for extended Reinhard.
  - `CHANGELOG.md` (Unreleased): the new API.
- **Files created:** `tests/tone_map.rs` (behaviour, refusals, the sweep) and
  `tests/tone_map_oracle.rs` (the oracle and its red-proof).
- **New public items in `irradiance::tone`:**

  ```rust
  pub const TONE_MAP_VERSION: &str = "1";
  pub const TONE_MAP_PARAM_MIN: f64 = 1.0 / 16_777_216.0;  // 2^-24
  pub const TONE_MAP_PARAM_MAX: f64 = 16_777_216.0;        // 2^24

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
  #[non_exhaustive]
  pub enum ToneMapCurve {
      #[default]
      ExtendedReinhard,
  }

  #[derive(Debug, Clone, Copy, PartialEq)]
  #[non_exhaustive]
  pub struct ToneMapOptions {
      pub curve: ToneMapCurve,   // default ExtendedReinhard
      pub exposure: f64,         // default 1.0
      pub white_point: f64,      // default 4.0
  }
  impl Default for ToneMapOptions { /* the three defaults above */ }

  pub fn tone_map_f32_into(
      src: &[f32],
      dst: &mut [f32],
      options: &ToneMapOptions,
  ) -> Result<(), Error>;
  ```

- **New `Error` variants** (all `ErrorKind::CallerError`, `DEC-051`):
  `InvalidToneMapExposure { exposure: f64 }`,
  `InvalidToneMapWhitePoint { white_point: f64 }`,
  `NonFiniteToneMapInput { index: usize }`.
- **Options:**
  - `exposure`: a linear multiplier applied to each sample before the curve (the paper's
    `a / L̄w` of Eq. 2, supplied by the caller). Valid: finite and in
    `[TONE_MAP_PARAM_MIN, TONE_MAP_PARAM_MAX]`, both ends inclusive. **Default 1.0.**
  - `white_point`: `L_white` of Eq. 4, in exposed units. Valid: the same closed range.
    **Default 4.0** (two stops above diffuse white; see `DEC-089` §4).
  - `curve`: `ExtendedReinhard` only in this spec. **Default `ExtendedReinhard`.**

### The rule (`DEC-089`), exactly

1. If `exposure` is NaN, infinite or outside the closed range: `InvalidToneMapExposure`
   carrying the value as given. Then the same for `white_point`. Then, if
   `dst.len() != src.len()`, the existing `OutputBufferWrongLength { expected: src.len(),
   actual: dst.len() }`. Then, if any `src[i]` is NaN or ±inf, `NonFiniteToneMapInput`
   naming the **lowest** such `i`. Nothing in `dst` is written unless all four checks pass,
   so the input is scanned completely before the first write.
2. Per sample, with `e = exposure`, `w = white_point`, in this operation order:
   ```text
   if x <= 0.0            -> +0.0     (negatives and −0.0; written as the literal 0.0)
   l = f64::from(x) * e
   if l >= w              -> 1.0
   y = (l * (1.0 + l / (w * w))) / (1.0 + l)
   dst[i] = y as f32
   ```
   No clamp follows the formula. Below `w` the exact value is below 1, and the `f64` error
   (a few `f64` ulps, about 1e-15) is far below half an `f32` ulp at 1.0 (about 6e-8), so the
   cast cannot reach above 1.0. The design probe measured that the formula never exceeds 1.0
   below `w` (below). A clamp there would be code no test can reach.
3. No `mul_add`, no transcendental, no `f32` arithmetic before the final cast. The pinned
   form holds no `a·b + c` pair (each multiply feeds a divide or is the last operation), so
   there is nothing an FMA could fuse. That is why this spec has no injected-cancellation
   test, unlike `color.rs` (`src/color.rs:4250-4317`); the scan's `.mul_add(` ban covers it.

### Design probe (2026-11-28, scratch, release build, every non-negative finite `f32`)

Rule 2, run over all 2,139,095,040 non-negative finite `f32` bit patterns, under seven
`(exposure, white_point)` configurations: `(1, 4)`, `(1, 1)`, `(1, 2⁻²⁴)`, `(1, 2²⁴)`,
`(2²⁴, 4)`, `(2⁻²⁴, 4)` and `(1, 11.2)`:

| Measured | Result |
|---|---|
| Outputs outside `[0, 1]` | 0 in every configuration |
| Decreasing steps (input order) | 0 in every configuration: exactly monotonic |
| Formula above 1.0 for `l < w` | 0 in every configuration |
| Bit distance from the oracle's form `(l·w² + l²)/(w² + w²·l)` | 0 everywhere, except 2 inputs at distance 1 under `(1, 2²⁴)` |
| The `f32`-arithmetic mutant differs from rule 2 | on 4.7M (`w = 1`) to 116M inputs per configuration |
| `w = 1`: output bits equal input bits on `[0, 1)` | 0 failures |
| Exact anchors at `w = 4` | `0.5 → 0.34375`, `1 → 0.53125`, `2 → 0.75`, `3 → 0.890625`, `4 → 1`, `8 → 1`, all bit-exact |
| `−0.0` | `+0.0` (bits `0x0`) |

## Acceptance Criteria

Thresholds are pre-registered here. A measured value outside one is reported, never fixed by
widening the bound.

- [ ] **AC1, the curve.** Rule 2 holds bit-exactly at the anchors above. With `w = 1` (a
  valid white point), every input in `[0, 1]` maps to itself bit for bit: the strided sweep of
  `[0, 1)`, the 1,000 largest `f32` below 1.0, and 1.0 itself. Every input above 1 maps to 1.0.
  The design probe found no failure over all 1,065,353,216 inputs in `[0, 1)` (requester item 3).
- [ ] **AC2, range and order.** On the strided sweep (stride 65,537 over the non-negative
  finite bit patterns, plus `f32::MAX`) under each of the probe's seven configurations, every
  output is in `[0, 1]` and the outputs are non-decreasing in input order.
- [ ] **AC3, negatives.** Every negative finite input and `−0.0` maps to `+0.0` (bits `0x0`).
- [ ] **AC4, exposure.** `exposure = 2` on `x` equals `exposure = 1` on `2x`, bit for bit, for
  inputs where `2x` is exact.
- [ ] **AC5, refusals.** Each invalid option value is refused with its variant and the value
  as given; both range ends are accepted. A non-finite input is refused naming the lowest bad
  index. The four checks run in rule 1's order, and `dst` is bit-identical to its prior
  contents after every refusal. A length mismatch reports `expected = src.len()`.
- [ ] **AC6, the oracle.** Under each configuration, on the strided sweep and the anchors,
  rule 2's output is within **1 `f32` ulp** (bit distance ≤ 1) of the oracle's independent
  `f64` form. The oracle is shown red on each designed fault (below), with the true outputs as
  the green negative control (`DEC-009`).
- [ ] **AC7, no panic, the fuzz substitute.** The sweep over both signs × all 256 exponents ×
  nine mantissas × five option sets (23,040 cases, counted and asserted) never panics. Every
  finite case returns `Ok` with an output in `[0, 1]` that is never `−0.0`, and every
  non-finite case returns `NonFiniteToneMapInput { index: 0 }`.
- [ ] **AC8, determinism.** Two committed FNV-1a hashes of `tone_map_f32_into`'s output are
  recorded on arm64 and checked on the x86_64 CI leg, asserted beside
  `TONE_MAP_VERSION == "1"`. The hashed input is the colour fixture's
  `srgb_linear_plane_f32_into` plane followed by the AC7 sweep's finite values (so it holds
  negatives, values in `(1, w)` and values ≥ `w`), under `Default` and under
  `{ exposure: 2.0, white_point: 1.0 }`.
- [ ] **AC9, nothing else moves.** Every existing determinism hash, every existing `tone`
  test and the committed tables are unchanged; `[dependencies]` stays empty; the widened
  source scan passes on `src/tone.rs` and `src/tone_table.rs`.
- [ ] **AC10, slice invariance.** Mapping the AC8 input in slices gives output bit-identical
  to mapping it whole, for splits at every 4,096th index, at every index of the first 64, and
  in row bands of 3 samples × 7 columns (requester item 4). A refusal inside a slice names
  the index relative to that slice; this is documented, not changed.
- [ ] **AC11, the documented contract.** `tone_map_f32_into`'s and `ToneMapCurve::ExtendedReinhard`'s
  rustdoc state (requester items 2, 3, 5, 6):
  - rule 2 exactly, in its operation order, with every constant, the arithmetic (`f64`,
    one cast), and that a GPU `f32` port matches within a tolerance it must choose, not
    bit for bit;
  - that `white_point = 1.0` is the identity on `[0, 1]` and clips above;
  - the clamp of negatives to `+0.0`, named as the only gamut handling, and the refusal of
    NaN and ±inf;
  - the scale convention: the curve treats 1.0 as SDR (diffuse) white, the scRGB convention.
    irradiance's own `srgb_linear_plane_f32_into` puts 1.0 at the sensor's white level, the
    clip point (`src/color.rs:695-698`), and a RAW capture's diffuse white usually sits below
    that, so a caller of that plane places diffuse white with `exposure`;
  - that every sample is mapped, so an interleaved alpha channel would be tone mapped too:
    pass colour channels only;
  - slice invariance (AC10), and that a refusal's index is relative to the slice passed.

  `tests/tone_map.rs::rustdoc_states_the_contract` reads `src/tone.rs` and asserts the doc
  comment above `pub fn tone_map_f32_into` contains the rule's formula line and the phrases
  `diffuse white`, `alpha`, `+0.0` and `slice`. Red: the items do not exist yet.

## Failing Tests

Written at design; the build makes them pass. "Red" says how each is seen failing first.

- **`src/tone.rs` (unit tests)**
  - `"tone_map_defaults_are_extended_reinhard_exposure_1_white_4"` asserts
    `ToneMapOptions::default()` field by field and `TONE_MAP_VERSION == "1"`. Red: does not
    compile before the items exist.
  - `"tone_map_param_bounds_are_2_pow_minus_24_and_2_pow_24"` asserts the two constants'
    bit patterns (`2⁻²⁴`, `2²⁴`) and ties them to the computation: `w = TONE_MAP_PARAM_MIN`
    gives `w*w = 2⁻⁴⁸`, a normal `f64`, and
    `f64::from(f32::MAX) * TONE_MAP_PARAM_MAX` is finite. Red: does not compile.
  - `no_transcendental_on_the_encode_path`, **widened (a moved assertion)**. The token list
    adds `.log2(`, `.log10(`, `.ln_1p(`, `.exp_m1(`, `.powf(` (kept), `.tanh(`, `.sinh(`,
    `.cosh(`, `.sin(`, `.cos(`, `.tan(`, `.atan(`, `.atan2(`, `.hypot(` and `.mul_add(`. The
    scan moves into a helper `banned_calls_in(code: &str) -> Vec<&'static str>`.
  - `"the_source_scan_catches_every_banned_call"` runs `banned_calls_in` on a planted string
    holding each token once and asserts it reports every token. This is the scan's own red
    run: it shows each new token can fail.
- **`tests/tone_map.rs`**
  - `"extended_reinhard_anchors_are_exact_at_white_4"`:
    `[0.5, 1, 2, 3, 4, 8] → [0.34375, 0.53125, 0.75, 0.890625, 1.0, 1.0]` by bits (AC1).
  - `"white_point_1_is_the_identity_on_0_to_1"`: the strided sweep of `[0, 1)` and the 1,000
    largest `f32` below 1.0 map to themselves by bits, 1.0 maps to 1.0, and
    `[1.5, 1e30, f32::MAX] → 1.0` (AC1).
  - `"exposure_scales_before_the_curve"`:
    `exposure 2` on `[0.25, 0.5, 1.0, 1.5]` equals `exposure 1` on `[0.5, 1, 2, 3]` by bits,
    and `exposure 2` on `[1.0]` is `0.75` (AC4).
  - `"negatives_and_negative_zero_map_to_positive_zero"`:
    `[-0.0, -f32::from_bits(1), -f32::MIN_POSITIVE, -0.5, -1e30, f32::MIN]` → bits `0x0`
    each (AC3).
  - `"subnormal_inputs_pass_through_at_white_4"`: `f32::from_bits(1)` and
    `f32::from_bits(0x007f_ffff)` map to themselves by bits.
  - `"output_is_in_range_and_monotonic_on_the_strided_sweep"`: AC2 for the seven
    configurations.
  - `"invalid_exposure_is_refused_with_its_value"`:
    `[NaN, +inf, -inf, 0.0, -0.0, -1.0, next_down(MIN), MIN / 2, next_up(MAX), MAX * 2]` each
    give `InvalidToneMapExposure` carrying the value (bits, `is_nan` for NaN); `MIN`, `MAX`
    and `1.0` give `Ok`. `next_down`/`next_up` are written as `f64::from_bits(bits ∓ 1)`
    (AC5).
  - `"invalid_white_point_is_refused_with_its_value"`: the same list for `white_point`
    (AC5).
  - `"refusals_come_in_rule_order"`: one call whose exposure, white point, length and input
    are all bad gives `InvalidToneMapExposure`; fixing each in turn gives
    `InvalidToneMapWhitePoint`, then `OutputBufferWrongLength { expected: 8, actual: 7 }`,
    then `NonFiniteToneMapInput` (AC5).
  - `"non_finite_input_names_the_lowest_bad_index"`:
    `[1, 2, 0.5, NaN, 1, 1, 1, +inf] → index 3`; `-inf` at 0 → 0; `+inf` only at the last
    index → `len − 1`; quiet NaN with a payload (`0x7fc0_0001`), negative NaN
    (`0xffc0_0000`) and signalling NaN (`0x7f80_0001`) are each refused (AC5).
  - `"nothing_is_written_on_any_refusal"`: `dst` is prefilled with a sentinel
    (`f32::from_bits(0x3f12_3456)`). For each of the four refusals, with the bad input at the
    **last** index for the non-finite case, every `dst` bit pattern is unchanged after (AC5).
  - `"empty_buffers_are_ok"`: `src = []`, `dst = []` under `Default` → `Ok`.
  - `"slices_give_bit_identical_output"`: AC10. The input is the AC7 sweep's finite values
    followed by the same values reversed. The whole-buffer output under `Default` and under
    `{ exposure: 2.0, white_point: 1.0 }` is compared by bits with the concatenated slice
    outputs, for every split listed in AC10. A second fixture puts NaN at whole-buffer
    index 10 and splits at 8, and asserts the second slice's refusal names index 2.
  - `"rustdoc_states_the_contract"`: AC11.
  - `"every_sign_and_exponent_never_panics"`: AC7. The mantissas are `0`, `1`, `2`,
    `0x40_0000`, `0x7f_ffff` and four from the file's fixed xorshift seed. The option sets
    are `Default`, `(MIN, MIN)`, `(MAX, MAX)`, `(MAX, MIN)` and `(MIN, MAX)`. The case
    counter is asserted equal to 23,040. A second pass maps every finite case in one buffer
    and asserts it equals the per-sample results, so no state crosses samples.
- **`tests/tone_map_oracle.rs`**
  - `"extended_reinhard_matches_the_independent_form"`: AC6's green run. The oracle maps
    `l < w` to `((l*w*w + l*l) / (w*w + w*w*l)) as f32`, `l ≥ w` to 1.0 and `x ≤ 0` to 0.0,
    written from Eq. 4 and not from `src/tone.rs`. It asserts that bit distance ≤ 1 holds at
    every point, and that more than 10,000 points across the seven configurations have an
    output strictly inside `(0, 1)`, so the comparison is not vacuous.
  - `"the_oracle_is_red_on_each_designed_fault"`: the comparison function
    `disagreements(inputs, outputs, options) -> usize` is fed:
    - the true outputs, which must give 0 (the negative control);
    - outputs computed in the test by each mutant, which must each give > 0 under the
      configuration named in the table below;
    - the true outputs with one element moved 2 ulps, which must give exactly 1. This shows
      the 1-ulp tolerance has teeth at the output's quantum.

    This is `DEC-021`'s in-process shape, as in `tests/tone_oracle.rs`: the subject is the
    comparison over `(input, output)` pairs, and a mutant library produces exactly such a
    pair. No crate copy is rebuilt.
- **`tests/determinism.rs`**
  - `"tone_map_hash_matches_committed"`: AC8. It asserts `TONE_MAP_VERSION == "1"`, then the
    two hashes against `TONE_MAP_COMMITTED`. The build records the constants on arm64 and
    the x86_64 leg checks them. A disagreement is a finding to report, not a constant to
    re-record.
  - `"tone_map_determinism_red_proof"`: the control reproduces the committed hashes through
    the same code path. Each of these must move the hash: the `f32`-arithmetic mutant (rule
    2 in `f32`, computed in the test), and the true output with one LSB flipped.
- **`tests/error_kind.rs`**: three rows, all `CallerError`, and the count comment updated
  (a moved assertion). `every_variant_classified` fails by name until they are added.

**Rule-clause map** (`DEC-089`):

| Clause | Test | Where it changes the output |
|---|---|---|
| exposure range, both ends inclusive | `invalid_exposure_is_refused_with_its_value` | `next_down(MIN)`/`next_up(MAX)` refused, `MIN`/`MAX` accepted |
| white point range | `invalid_white_point_is_refused_with_its_value` | same |
| check order (an order word) | `refusals_come_in_rule_order` | one fixture combining all four faults |
| lowest bad index (an order word) | `non_finite_input_names_the_lowest_bad_index` | NaN at 3 and inf at 7 → 3 |
| scan before write | `nothing_is_written_on_any_refusal` | bad sample at the last index; a write-as-you-go build has written `0..len−1` |
| `x ≤ 0 → +0.0` | `negatives_and_negative_zero_map_to_positive_zero` | bits `0x0`, not `0x8000_0000` |
| exposure before the curve | `exposure_scales_before_the_curve` | `x = 1, e = 2 → 0.75` |
| `l ≥ w → 1` | anchors (`8 → 1`), the identity test (`≥ 1 → 1`) | above `w` |
| Eq. 4 with `w²` | anchors at `w = 4` | `1 → 0.53125` |
| `f64`, not `f32` | `tone_map_determinism_red_proof` | the hash |
| `TONE_MAP_PARAM_MIN`/`MAX` (public constants) | the bounds unit test and both refusal tests | the refusal boundary |
| `TONE_MAP_VERSION` (a public constant) | `tone_map_hash_matches_committed` | asserted beside the hash |
| per-sample, no cross-sample state | `slices_give_bit_identical_output` | any split point |
| `w = 1` is the identity on `[0, 1]` | `white_point_1_is_the_identity_on_0_to_1` | every input in `[0, 1]` it lists |

**Distinguishing fixtures** (each mutant's value computed at the named point; all from the
probe or exact dyadic arithmetic):

| Mutant | Fixture | Right | Mutant |
|---|---|---|---|
| M1 simple Reinhard `l/(1+l)` | `w = 4, x = 1` | 0.53125 | 0.5 |
| M2 `w` not squared | `w = 4, x = 1` | 0.53125 | 0.625 (`w = 1` cannot tell them apart) |
| M3 no saturation at `w` | `w = 4, x = 8` | 1.0 | 1.3333 |
| M4 exposure ignored | `e = 2, w = 4, x = 1` | 0.75 | 0.53125 |
| M5 exposure scales the output | `e = 2, w = 4, x = 1` | 0.75 | 1.0625 |
| M6 negatives through the formula | `x = −0.5, w = 4` | 0.0 | −0.96875 |
| M7 `−0.0` kept | `x = −0.0` | bits `0x0` | bits `0x8000_0000` |
| M8 last bad index | `[.., NaN@3, .., inf@7]` | 3 | 7 |
| M9 write as you go | bad at the last index | sentinel kept | `0..len−1` written |
| M10 exclusive range ends | `exposure = MIN` | `Ok` | refused |
| M11 `f32` arithmetic | the determinism input | committed hash | differs on 4.7M–116M inputs per configuration (probe); within 1 ulp, so the oracle is not its gate, the hash is |
| M12 state carried across samples (for example, a running maximum as the white point) | split at index 8 of the slice fixture | whole = sliced | the second slice differs |

**Moved assertions.**
- `src/tone.rs::no_transcendental_on_the_encode_path`: the token list grows (above). Its
  "comments stripped, real code read" guard is kept.
- `tests/error_kind.rs`: the table gains three rows and the count comment changes.
- `src/tone.rs`'s module docs (`:30-38`): "the library runs no `powf`/`exp`/`ln` on the
  encode path" stays true. "Determinism follows from the committed integer table" becomes
  "the encoders' determinism follows from the table, the tone map's from basic `f64`
  operations (`DEC-089`)". This is a documentation move, with no assertion behind it.
- No existing behaviour changes, so no existing pin, oracle or fixture moves.

**Claims about existing code**, with where each was read:
- `Error` is `#[non_exhaustive]` (`src/lib.rs:326-327`); `kind` has no wildcard
  (`src/lib.rs:1978-1981`).
- `OutputBufferWrongLength` already exists with `{ expected, actual }` (`src/lib.rs:903`)
  and is `CallerError` (`src/lib.rs:1995`). It is reused, not duplicated.
- The precedent for a refused option that carries its value is `InvalidWarpAlpha
  { warp_alpha: f64 }` (`src/lib.rs:865`), classified `CallerError` (`src/lib.rs:2000`).
- `tests/error_kind.rs` reads the variant names out of `src/lib.rs`
  (`tests/error_kind.rs:10-13`).
- The source scan reads only the text before the first `#[cfg(test)]` (`src/tone.rs:244`).
- `colour_planes` returns the colour fixture's `srgb_linear_plane_f32_into` output
  (`tests/determinism.rs:541-551`).

## Non-Goals

- The sRGB encoders for `[0, 1]` `f32` (SPEC-102) and Hable (SPEC-103).
- An in-place form (`&mut [f32]` alone). It is additive later if a consumer needs it
  (`DEC-089` open question 2).
- Threading, SIMD or a row-band split. This is per-sample and embarrassingly parallel; the
  caller can split the slice.
- Any change to existing `tone` functions, the tables, `develop_into` or `DEC-018`.
- A fuzz target (`DEC-089` §6).
- Auto-exposure, luminance mode, AgX.

## Notes for the Implementer

- Write rule 2 in the exact order given; the hashes pin it. Use `x <= 0.0` (NaN is already
  refused), not `!(x > 0.0)`, which clippy flags.
- The scan is `src.iter().position(|v| !v.is_finite())`, before any write; the write is a
  `zip`. No indexing (`clippy::indexing_slicing` is deny).
- The provenance row is class 2, a published paper, with no implementation read. Record the
  per-channel deviation: Eq. 4 is stated for luminance, and here it is applied to each
  channel. Record also that the paper's default `L_white` (the scene maximum) is replaced by
  a fixed 4.0 (`DEC-089` §4).
- Record the hashes on arm64 (macOS) and let CI's x86_64 leg check them. Say in the
  handback which arch recorded them.

---

## Reflection

*Appended during **ship**.*
