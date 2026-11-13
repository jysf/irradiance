---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-064
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
  id: PROJ-004
  stage: STAGE-019
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-059, DEC-058, DEC-024, DEC-018, DEC-028, DEC-051, DEC-052, DEC-012, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-018, SPEC-063, SPEC-065]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-063]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-019's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-019's partial lens correction (warp_alpha over WarpRect::lerp) and makes FullActiveArea available on the warped Q2M by replacing the blanket refusal with a measured bounds rule."

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
  tokens_estimate: 28000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-064: warp_alpha over WarpRect::lerp, and FullActiveArea for a warp whose lerped scale stays in [0, 1]

## Context

SPEC-064 is the second spec in STAGE-019 (PROJ-004 instrument layer C). It covers the
partial-correction half of monorail-000 / onyx-000 request 2
(`feedback/2026-11-11-monorail-000-requests.md`), and future-plans § PROJ-004 "Stage C:
`WarpRect::lerp(alpha)`". It depends on SPEC-063, which shipped `DevelopOptions`.

Two things are missing today:
- **A partial lens correction.** `develop_with_into` applies `OpcodeList3`'s
  `WarpRectilinear` fully or not at all. An instrument view wants the raw geometry
  (`α = 0`), the full correction (`α = 1`), or something in between.
- **`FullActiveArea` on the requester's camera.** SPEC-063 refuses `FullActiveArea` with
  any non-identity warp (DEC-058 §4), because the warp clamps out-of-image samples to the
  edge (DEC-024) and under `FullActiveArea` those would be visible. Every held Q2M frame
  carries a warp, so today `FullActiveArea` is refused on all of them.

future-plans "Pin at design time" 1 required the warp's backward radial scale to be
**measured**, not assumed, before this is relaxed. §1 is that measurement.

**DEC-059** (minted with this design) defines `lerp`, the `warp_alpha` domain and the bounds
rule. It amends DEC-058 §4.

## Goal

Add `DevelopOptions::warp_alpha` (default `1.0`) over a new `WarpRect::lerp`. Replace
SPEC-063's blanket refusal with DEC-059's bounds rule, so `FullActiveArea` develops every
held Q2M frame at every α, while a lens whose lerped scale leaves `[0, 1]` (the Pixel 3a's)
is still refused, typed, before any write.

## Design

### 1. Probe results (main at 0d8fa19, 2026-11-12; recorded so build is transcription)

Every held file was read through the public `Sensor` and `parse_warp_rectilinear`, for α in
{0, .25, .5, .75, 1}. Three measurements were taken:
- the exact extrema of `f_α(s)` on `s = r²` in `[0, 1]`;
- a dense 10⁶-point cross-check, which agreed to every printed digit;
- a pixel-exact count, over the whole active area, of the destination pixels whose source
  coordinate leaves `[0, w−1] × [0, h−1]`.

| File | Active | `kr` | `cx`, `cy` | α = 1 scale [min, max] | corner pull α = 1 | out-of-extent at α = .25 / .5 / .75 / 1 |
|---|---|---|---|---|---|---|
| Q2M L1021223 | 8392×5632 | 0.999251106, −0.0613765129, −0.0939155414, 0.0558820092 | 0.5, 0.5 | [0.899841061, 0.999251106] | 506.1 px | 0 / 0 / 0 / 0 |
| Q2M L1026016 | 8392×5632 | 0.999251106, −0.0418451693, −0.1023114788, 0.0578767192 | 0.5, 0.5 | [0.912971177, 0.999251106] | 439.7 px | 0 / 0 / 0 / 0 |
| Q2M L1026192 | 8392×5632 | 0.999251106, −0.0323314533, −0.1042041102, 0.0563642935 | 0.5, 0.5 | [0.919079836, 0.999251106] | 408.9 px | 0 / 0 / 0 / 0 |
| Pixel 7 Pro | 3882×2924 | 0.9896960258, 0.0099455817, −0.0031831064, 0.0030372287 | 0.49983551, 0.49167421 | [0.989696026, 0.999495730] | 1.2 px | 0 / 0 / 0 / 0 |
| **Pixel 3a** | 4032×3024 | 0.9801586866, 0.0919882432, −0.1778785288, 0.1072608531 | 0.50174668, 0.50277371 | [0.980158687, **1.001529254**] | −3.9 px (outward) | **299 / 323 / 382 / 439** (worst 3.09 px) |
| Pixel 4a | 4032×3024 | identity | 0.5, 0.5 | [1, 1] | 0 | 0 |
| Canon, Nikon, M Monochrom, Typ 246, K-3 III | | no `OpcodeList3` | | | | |

At α = 0 every file's scale is exactly 1 and every count is 0. At each α the scale range is
`1 − α·(1 − range at α = 1)`. For example, at α = 0.5 L1021223 spans
[0.949920530, 0.999625553], and its corner pull is 253.0 px, exactly half.

What follows from it:
- **Every Q2M frame satisfies DEC-059 §3 at every α.** `FullActiveArea` will develop all
  three frames.
- **The Pixel 3a is a real over-unity lens.** Its scale exceeds 1 near the corners at every
  α > 0, and the pixel-exact count agrees. It is the red-proof's polynomial (AC6). Pixel
  files do not develop today (unrecognised mandatory opcode 9), so its coefficients are put
  into a tier-A fixture by hand. The full-precision values to use are listed below.
- **The Pixel 7 Pro is an off-centre lens that passes.** It is the accept-side case for a
  centre that is not 0.5.
- **The corner pull is measured.** It is 506.1 px on L1021223, against the unrecorded
  "503.8 px" in future-plans. future-plans is updated in this design PR.

Full-precision coefficients for fixtures, as `parse_warp_rectilinear` returns them (copy
them exactly):
- Pixel 3a:
  - `kr = [0.9801586866378784, 0.09198824316263199, -0.17787852883338928, 0.10726085305213928]`;
  - `cx = 0.5017466772170294`, `cy = 0.5027737087673612`.
- Pixel 7 Pro:
  - `kr = [0.9896960258483887, 0.009945581667125225, -0.0031831064261496067, 0.0030372287146747112]`;
  - `cx = 0.4998355100966802`, `cy = 0.4916742118699769`.
- Q2M L1021223: `kr = [0.999251106, -0.06137651287726358, -0.09391554139336016, 0.05588200921529175]`,
  with `cx = cy = 0.5`.

### 2. Public API

- **`src/opcode.rs`:** `impl WarpRect { pub fn lerp(&self, alpha: f64) -> WarpRect }`,
  exactly DEC-059 §1. It is `#[must_use]`, and its doc gives the formula, the α = 1
  bit-identity, the α = 0 identity, and that it does not validate α. Not feature-gated.
- **`src/develop.rs`:** `DevelopOptions` gains `pub warp_alpha: f64`, and `Default` sets it
  to `1.0`.
  - The field doc states the domain, the default, that `0.0` disables the warp (its
    tangential terms included), and the `FullActiveArea` bounds rule.
  - The struct's derives are unchanged (`Debug, Clone, Copy, PartialEq`; DEC-058 §1
    already withheld `Eq`).
- **`src/lib.rs`:**
  - New `Error::InvalidWarpAlpha { warp_alpha: f64 }`, `ErrorKind::CallerError`. Its
    `Display` names the value and the domain `[0, 1]`.
  - `Error::UnsupportedWarpedFullActiveArea` becomes
    `{ active_width: u32, active_height: u32, warp_alpha: f64, scale_min: f64, scale_max: f64 }`.
    It is unreleased, so this is not a break. Its `Display` names all five values and
    says the lerped radial scale must stay within [0, 1] with the optical centre inside
    the image. For a non-finite coefficient or an out-of-image centre, `scale_min` and
    `scale_max` are whatever the computation gives (possibly NaN); the doc says so.

### 3. Semantics

**Validation order inside `develop_with_into`.** Step 0 is new and the rest is SPEC-063's
order:
0. `warp_alpha` finite and in `[0, 1]`, else `InvalidWarpAlpha`. It is first, before
   `check_required_processing`: a pure argument check, under every framing, with or
   without a warp.
1. `check_required_processing`;
2. `src` length;
3. `resolve_geometry`, then `apply_options`;
4. levels;
5. `dst` length;
6. **only under `FullActiveArea`:** parse `OpcodeList3`, then lerp, then the identity
   filter. If a non-identity warp remains, apply the DEC-059 §3 bounds check, and refuse
   with `UnsupportedWarpedFullActiveArea` if it fails. Keep the lerped warp, so it is not
   parsed twice.
7. `FixBadPixelsConstant`, then levels and warp as today, with the **lerped** warp.

`output_dimensions_with` runs step 0 too. It still does not read opcode lists (SPEC-063
§4).

**Where the lerp happens.** Change `non_identity_warp(sensor)` to
`non_identity_warp(sensor, alpha)`. It parses, lerps, and then filters identity. Both
framings go through it, so under `StandardCrop` the lerped warp is what is applied, and an
`α = 0` warp is skipped exactly like an identity opcode.

**The bounds check** is a private function, for example
`warp::scale_bounds(&WarpRect) -> (f64, f64)` plus a predicate, implemented as DEC-059 §3
says:
- the extrema of the cubic on `[0, 1]`, from the endpoints and the real roots of the
  derivative quadratic in `(0, 1)`;
- the degenerate cases, where `kr3' = 0` (a linear derivative) or `kr3' = kr2' = 0`
  (a constant one), handled without dividing by zero;
- a NaN anywhere fails the predicate, so write the comparisons so that NaN is a refusal
  (for example `!(min >= 0.0 && max <= 1.0)`).

No sampling and no tolerance. It must never panic; clippy's arithmetic lints apply as
everywhere in `src/`.

**`StandardCrop` is unchanged at every α** (DEC-059 §4): no bounds check, and the clamp
behaviour stays as today.

**Tangential terms.** At `α = 0` the lerped `kt` is zero and the warp is skipped, so a
non-zero `kt` is not refused. At `α > 0` the applier refuses it as today
(`UnsupportedWarpTangentialTerms`). The bounds check uses the radial scale only; a
non-zero `kt` under `FullActiveArea` reaches the applier's refusal if the radial check
passes. Document this order.

### 4. Fuzz (`fuzz/fuzz_targets/develop.rs`)

Extend SPEC-063's option byte:
- bits 2–3 select `warp_alpha` from `{1.0, 0.0, 0.5, f64::NAN}`;
- bit 4 replaces it with `1.5` (out of range).

Document the layout in the target header. Under the existing `MAX_PIXELS` cap, run
`output_dimensions_with` / `develop_with_into` with the result. Add seeds via
`examples/fuzz-seeds.rs`:
- a fixture carrying the Pixel 3a warp (over-unity);
- one carrying the L1021223 warp;

each with the option bytes for `FullActiveArea` at α 1, 0 and 0.5. No seed comes from a
corpus file (DEC-003). The coefficients are numbers in source, not corpus bytes.

### 5. Docs

- **CHANGELOG `[Unreleased]`:**
  - edit the SPEC-063 bullets. `UnsupportedWarpedFullActiveArea` now fires only when the
    bounds rule fails, and has the three new fields. Remove the sentence that says the Q2M
    is refused;
  - add `warp_alpha`, `WarpRect::lerp` and `InvalidWarpAlpha` under Added;
  - note that `DEVELOP_VERSION` stays `"4"`.
- **Provenance ledger:** one row for `WarpRect::lerp` and the bounds rule. The arithmetic
  is original, over DNG 1.7.0.0 §6.4.1's polynomial (class 1). The closed-form cubic
  extrema are standard calculus.
- **Module docs:**
  - `src/develop.rs`: amend the "Options" paragraph to say what `warp_alpha` does and what
    the `FullActiveArea` rule is now;
  - `src/warp.rs`: the "Out-of-extent pixels" paragraph should point at the bounds check
    and say that under `FullActiveArea` no visible sample is clamped.
- `DEVELOP_VERSION` is **not** bumped (DEC-059 Consequences).

## Inputs

- **Files to read:**
  - `src/develop.rs`: `DevelopOptions` ~274, `apply_options` ~321, `non_identity_warp`
    ~381, `develop_with_into` ~979, and its early refusal ~1042;
  - `src/warp.rs` (all of it; `source_coord` is the formula the bounds rule reasons about);
  - `src/opcode.rs`: `WarpRect` ~166, `parse_warp_rectilinear` ~431;
  - `src/lib.rs`: `Error`, `Error::kind`, `DEVELOP_VERSION`.
- **Tests to mirror:**
  - `tests/develop_options.rs` (SPEC-063: the MD5 pins, the refusal tests, the sentinel
    and precedence probes);
  - `tests/warp.rs` (the corner-impulse fixture and `corner_source_coordinate`);
  - `tests/support/opcode.rs` (`single_warp_rectilinear_list`);
  - `tests/error_kind.rs`.
- **Decisions:** DEC-059 (this spec), DEC-058, DEC-024, DEC-018, DEC-028, DEC-051, DEC-052,
  DEC-012, DEC-003.

## Outputs

- **Files modified:**
  - `src/opcode.rs`: `WarpRect::lerp`;
  - `src/warp.rs`: the bounds function and the module doc;
  - `src/develop.rs`: the field, step 0, `non_identity_warp` with α, and the bounds check
    in place of the blanket refusal;
  - `src/lib.rs`: the new variant, the changed variant, `Display`, `kind()`;
  - `tests/develop_options.rs`: SPEC-063's refusal tests, updated as AC6 says;
  - `tests/error_kind.rs`;
  - `fuzz/fuzz_targets/develop.rs` and `examples/fuzz-seeds.rs`;
  - `CHANGELOG.md` and `docs/provenance-ledger.md`.
- **Files created:** `tests/warp_alpha.rs`.
- **New public items:**
  - `WarpRect::lerp`;
  - `DevelopOptions::warp_alpha`;
  - `Error::InvalidWarpAlpha`;
  - three fields on `Error::UnsupportedWarpedFullActiveArea`.
- **New option and its default:** `warp_alpha: f64`, in `[0, 1]`, default `1.0`.

## Acceptance Criteria

- [ ] **AC1. Default.** `DevelopOptions::default().warp_alpha == 1.0`. SPEC-063's
  `develop_into_output_is_unchanged_on_held_files` (eight MD5 pins) stays green, unedited.
  `develop_with_into` with an explicit `warp_alpha = 1.0` matches the same pins.
  `DEVELOP_VERSION == "4"`.
- [ ] **AC2. `lerp` is exact at its ends.**
  - `w.lerp(1.0)` equals `w` field for field, comparing `kr` and `kt` with
    `f64::to_bits` (bit identity, not `==`). This holds for the three Q2M coefficient sets
    and the Pixel 3a set.
  - `w.lerp(0.0)` satisfies `is_identity` (via the develop path: a sensor whose warp lerps
    to identity takes the warp-free path).
  - `cx`, `cy` and `planes` are unchanged at every α.
  - At α = 0.5, `kr0` equals `0.5 + 0.5·kr0` and each other coefficient is halved.
- [ ] **AC3. α = 0 equals no warp.** On tier A and on the three Q2M files, under both
  framings, `develop_with_into` with `warp_alpha = 0.0` equals `develop_with_into` of the
  same sensor with `opcode_list_3 = None`. On tier B this is an MD5 comparison of the two
  outputs computed in the test, not a pin. Under `FullActiveArea` this is also the first
  real-file success on the Q2M.
- [ ] **AC4. Displacement is linear in α (analytic oracle, tier A).** Use `tests/warp.rs`'s
  corner-impulse fixture at the Q2M crop size with L1021223's coefficients. The detected
  impulse displacement at α = 0.5 is half the displacement at α = 1, within 1 px. At
  α = 0.25 it is a quarter, within 1 px.
  - **Red-proof:** a `lerp` that scales only `kr1` (or forgets `kr0`'s `(1 − α)` term)
    misses this.
- [ ] **AC5. The bounds oracle on held frames (tier B, the load-bearing oracle).** For
  each Q2M frame and each α in {0, .25, .5, .75, 1}, with `apply_orientation: false` so
  both outputs are in stored order:
  - `FullActiveArea` develops. It returns `Ok`, and the output is 8392×5632.
  - The output **contains** the `StandardCrop` output for the same α at the crop origin
    (x 12, y 24), sized 8368×5584. This is SPEC-063 AC4(b)'s relation, now through the
    warp. The warp runs on the active area under both framings, so the crop must agree
    exactly.
  - **Pin check at α = 1, with default orientation.** The `StandardCrop` output's MD5
    equals the SPEC-063 pin. This applies to all three frames; L1026016's pin is the
    oriented one.
  - The test reports a missing file by name, never a silent pass.
- [ ] **AC6. The over-unity red-proof and the refusal.**
  - **Pixel 3a, synthetic.** A tier-A sensor (non-square, with an `ActiveArea` border,
    also carrying `FixBadPixelsConstant`) with the Pixel 3a coefficients from §1, under
    `FullActiveArea`:
    - for α in {0.25, 0.5, 0.75, 1} it returns `UnsupportedWarpedFullActiveArea`, with the
      right active size, the requested `warp_alpha`, and `scale_max > 1`;
    - `kind() == Unsupported`;
    - `dst` is unchanged (sentinel fill);
    - a malformed `FixBadPixelsConstant` list placed alongside is **not** reported first
      (the precedence probe);
    - at α = 0 it develops.
  - **Pixel 3a under `StandardCrop`** develops at α = 1. Its output equals `develop_into`'s
    (DEC-059 §4: no new refusal).
  - **Accept side, synthetic.** The same sensor with the Pixel 7 Pro coefficients (an
    off-centre lens that passes) and with L1021223's coefficients develops under
    `FullActiveArea` at every α in the set.
  - **Each clause of DEC-059 §3 has its own hand-built refusal:**
    - `kr = [-0.5, 0, 0, 0]` (`f < 0`);
    - a cubic whose maximum is **interior**, with `f(0) ≤ 1`, `f(1) ≤ 1` and a peak above 1
      inside `(0, 1)`, for example `kr = [0.9, 0.6, -0.6, 0]`, which peaks at 1.05 at
      `s = 0.5`. This is what the corner-only alternative misses;
    - `cx = 1.5` with an otherwise passing polynomial;
    - a NaN coefficient.
  - **SPEC-063's refusal tests** (`full_active_area_with_a_warp_is_refused_before_any_write`
    and the Q2M real-file refusal) are updated. The synthetic one switches to an
    over-unity warp. The Q2M one becomes AC5's success. Record each edit and why in the
    handback. These are the only allowed edits to existing assertions.
- [ ] **AC7. `warp_alpha` domain.** For `NaN`, `+∞`, `−∞`, `-0.01` and `1.01`,
  `develop_with_into` and `output_dimensions_with` return `InvalidWarpAlpha` carrying the
  value, with `kind() == CallerError` and `dst` unchanged. This holds under both framings
  and on a sensor with **no** warp. `-0.0`, `0.0` and `1.0` are accepted. The error comes
  before a malformed `OpcodeList1`/`OpcodeList3` would be reported (step 0 is first).
- [ ] **AC8. Tangential terms at α = 0.** A sensor whose warp has `kt = [0.01, 0]`:
  - develops at α = 0 and equals the no-warp output;
  - is refused with `UnsupportedWarpTangentialTerms` at α = 1 under `StandardCrop`, as
    today.
- [ ] **AC9. The bounds function (unit, in `src/warp.rs`).**
  - It returns the probe's α = 1 `[min, max]` for the three Q2M and the Pixel 3a / 7 Pro
    sets, within `1e-12`.
  - It handles `kr3 = 0` and `kr2 = kr3 = 0` without a division by zero.
  - It returns a refusal for a NaN coefficient.
  - **Red-proof:** dropping the interior roots makes AC6's interior-peak case pass the
    check (it must not).
- [ ] **AC10. Error surface.**
  - `InvalidWarpAlpha` is in `Error::kind()` with no wildcard, and has a row in
    `tests/error_kind.rs`.
  - Its `Display` contains the value.
  - `UnsupportedWarpedFullActiveArea`'s `Display` contains `warp_alpha`, `scale_min` and
    `scale_max`.
  - The `error_kind` counts move by exactly one.
- [ ] **AC11. API hygiene.**
  - `warp_alpha` and `lerp` are documented: the domain, the default, the α = 0 meaning,
    and the bounds rule.
  - An external test sets `warp_alpha` only by field assignment on `Default`.
  - `cargo doc` has no broken links.
  - CHANGELOG `[Unreleased]` is updated as §5 says.
- [ ] **AC12. Fuzz.** The `develop` target exercises the α choices in §4; the new seeds are
  written by `examples/fuzz-seeds.rs`; a 60 s run is clean.
- [ ] **AC13. No regression.**
  - Every existing test passes with no assertion changed, except AC6's two SPEC-063
    refusal tests and `error_kind`'s rows and counts.
  - `just test` passes with the corpus env set. So do `lint-ci`, `lint-no-allow`,
    `fmt-check`, `validate`, `doc-check`, `deny` and `msrv`.
  - No new dependency.
  - The `src/` diff adds no allocation, and none before the refusal.

## Failing Tests

- **`tests/warp_alpha.rs`** (new):
  - `default_warp_alpha_is_one_and_pins_hold`: AC1, including tier B.
  - `lerp_is_bit_exact_at_one_and_identity_at_zero`: AC2.
  - `alpha_zero_equals_the_warp_free_develop`: AC3, tier A and tier B.
  - `displacement_is_linear_in_alpha`: AC4.
  - `full_active_area_develops_every_q2m_frame_at_five_alphas`: AC5, tier B.
  - `over_unity_pixel_3a_warp_is_refused_before_any_write`: AC6.
  - `over_unity_warp_still_develops_under_standard_crop`: AC6.
  - `passing_warps_develop_under_full_active_area`: AC6, Pixel 7 Pro and L1021223.
  - `each_bounds_clause_refuses`: AC6 (f < 0, interior peak, centre outside, NaN).
  - `warp_alpha_outside_the_domain_is_a_caller_error`: AC7.
  - `alpha_zero_skips_tangential_terms`: AC8.
  - `refusal_displays_name_the_values`: AC10.
- **`src/warp.rs` unit tests:** `scale_bounds_match_the_probe`,
  `scale_bounds_degenerate_cubics`, `scale_bounds_nan_refuses` (AC9).
- **`tests/develop_options.rs`:** the two SPEC-063 refusal tests, edited as AC6 says.
- **`tests/error_kind.rs`:** the new row (AC10).

**Red-proof expectations** (verify mutates these; each must turn a named test red):
- Default `warp_alpha` other than 1.0 → AC1.
- `lerp` drops the `(1 − α)` term on `kr0`, or does not scale `kr1..kr3` → AC2, AC4.
- The lerp is not applied (the full warp is always used) → AC3, AC4.
- The bounds check checks only `s = 1`, or omits the interior roots → AC6's interior-peak
  case, AC9.
- The bounds check omits `f ≥ 0`, or the centre clause → AC6's `f < 0` / `cx = 1.5`
  cases.
- NaN passes the check (comparisons written as `min < 0.0 || max > 1.0`) → AC6, AC9.
- The bounds check is applied under `StandardCrop` too → AC6's `StandardCrop` case.
- The blanket SPEC-063 refusal is kept → AC5.
- The refusal is moved after a `dst` write, or after the `FixBadPixelsConstant` copy → AC6's
  sentinel and precedence probes.
- The α validation is moved after `check_required_processing` or the opcode parse → AC7's
  precedence case.

## Non-Goals

- **A pixel-exact bounds check**, or admitting a lens with `f > 1` only near the centre.
  DEC-059 records this as the additive relaxation, for when a held file needs it.
- **Refusing or masking clamped samples under `StandardCrop`.** The behaviour is unchanged.
- **Tangential-term support.** It is still refused at α > 0.
- **Per-axis or per-coefficient α, or a "zoom to fill" option** that crops away the
  corners' inward pull. No named caller.
- **Caller scratch** (SPEC-065), **CFA-as-mono** (SPEC-066), **the blocked transpose**
  (SPEC-067), **the colour path**, and **`irr develop` flags**.

## Notes for the Implementer

- **α = 1 must be bit-identical.** Compute `kr0'` as `(1.0 - alpha) + alpha * kr0`. Do not
  write `kr0 + (1.0 - kr0) * (1.0 - alpha)` or any other rearrangement that rounds
  differently. AC1's pins and AC2's `to_bits` check catch a rearrangement.
- `-0.0` from `0.0 * negative` compares equal to `0.0`, so `is_identity` (array `==`)
  already treats the α = 0 result as identity. Do not "fix" it with `to_bits`.
- The interior-peak fixture `[0.9, 0.6, -0.6, 0]`: `f'(s) = 0.6 - 1.2 s`, zero at
  `s = 0.5`, so `f(0.5) = 0.9 + 0.3 - 0.15 = 1.05`. Endpoints: `f(0) = 0.9`, `f(1) = 0.9`.
- The tier-B AC5 test runs 30 develops of 47 MP (three frames, five alphas, two framings). Keep it in one test, release-mode
  friendly, and mirror how `tests/develop_options.rs` gates and reports tier B.

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
