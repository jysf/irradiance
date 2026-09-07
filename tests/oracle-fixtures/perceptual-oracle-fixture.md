# The perceptual (SSIMULACRA2) oracle's tier-A fixture — `SPEC-020`

`SPEC-020`'s `## Implementation Context` pre-registers the choice between a
licence-clean natural photograph and a synthetic gradient-plus-texture field,
preferring the photograph if one is available. None was available inside this
build's environment without either sourcing a third-party image (a
provenance/licence question this oracle spec should not have to answer) or
decoding one via the forbidden `::image` crate (`library-not-application`
holds test-side too). This build uses the synthetic candidate.

**The fixture is generated in code, not committed as a binary blob** —
`tests/perceptual_oracle.rs`'s `synthetic_reference()` — so the exact bytes
are reviewable as source rather than trusted as an opaque file. This mirrors
the precedent `tests/develop_oracle.rs`'s `FU-10` production-scale fixture
already set in this repo: an in-test synthetic image, not a checked-in one.

## What it is

Fractional Brownian motion (5 octaves of hash-based bilinear value noise),
1024x768, contrast-stretched around the midpoint and quantised to 16-bit
grayscale samples (treated as already-sRGB-domain, matching what a real
`dnglab --srgb` payload looks like once read).

| Parameter | Value |
|---|---|
| Width x Height | 1024 x 768 |
| Base frequency | 0.02 |
| Octaves | 5 |
| Lacunarity / gain | 2.0 / 0.5 (fixed in `fbm`) |
| Contrast | 1.6 |
| Noise seed | 1234 |

Deterministic: the same seed and parameters produce byte-identical output on
every run (no RNG crate, no OS entropy — a hand-rolled splitmix64-shaped hash
of `(x, y, seed)`).

## Why these parameters — the pre-registered rule

`SPEC-020` requires the fixture satisfy ALL FOUR invariants below, or the
fixture is wrong, not the threshold (`## Implementation Context` /
`SPEC-020.md`'s "Required in build" section). Parameters were probed
empirically against the real `ssimulacra2` 0.5.1 crate in a standalone
scratch harness (same fBm/conversion/perturbation code later ported verbatim
into `tests/perceptual_oracle.rs` and `tests/support/perturb.rs`) before
being committed here — a small parameter sweep (base frequency 0.01–0.15,
octaves 3/5/7) showed every combination tried satisfied all four invariants,
often with wide margins; `(0.02, 5, 1.6)` was selected because its four
scores track `DEC-005`'s own real-photograph calibration table most closely
(same order of magnitude on every row), which is the more reproducible and
more defensible choice than an arbitrarily "easier" corner of the parameter
space.

## Measured scores on this fixture (this build, `cargo test --test
perceptual_oracle -- --nocapture`, `IRRADIANCE_CORPUS_DIR` unset)

| Check | AC | Score | Threshold | Result |
|---|---|---|---|---|
| Identical (self vs self) | AC3 | 100.000 | ≥ 99.9 | pass |
| 1-pixel horizontal shift | AC4 | 61.823 | < 85 | **caught** |
| Missing radial warp (`SPIKE-001` coefficients) | AC5 | −82.338 | < 85 | **caught, emphatically** |
| Gamma 1.05 (linear domain) | AC6 | 90.038 | ≥ 85 | pass (admitted) |

For comparison, `DEC-005`'s real-photograph calibration (dnglab's own
render, quarter resolution): identical 100.00, 1-px shift 62.96, missing warp
−68.05, gamma 1.05 88.51. This fixture's numbers land in the same shape and
the same order of magnitude on every row, at full 1024x768 resolution rather
than a quarter-resolution real photograph.

## Reproducing these numbers

```bash
IRRADIANCE_CORPUS_DIR= cargo test --all-features --test perceptual_oracle \
    -- --nocapture one_pixel_shift_scores_below_the_threshold \
       missing_warp_scores_below_the_threshold \
       gamma_one_point_zero_five_scores_at_or_above_the_threshold \
       synthetic_reference_scores_at_least_ninetynine_point_nine_against_itself
```

Each test `eprintln!`s its measured score under `--nocapture` before
asserting against `DEC-005`'s threshold.
