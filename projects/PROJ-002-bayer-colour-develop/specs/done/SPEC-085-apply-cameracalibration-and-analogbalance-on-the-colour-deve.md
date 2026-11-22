---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-085
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L             # stamped at ship: what it ACTUALLY took, same scale.
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
  id: PROJ-002
  stage: STAGE-027
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-076, DEC-033, DEC-034, DEC-012, DEC-049]  # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm]  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-033, SPEC-034, SPEC-060, SPEC-079, SPEC-035, SPEC-086, SPEC-087]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []  # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-027's <capability>". Optional; null is acceptable.
value_link: "the colour develop honours the calibration it parses, so the held Pixel 7 Pro's colour matches dng_validate within the pre-registered bound"

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
  tokens_estimate: 30000000
  sessions:
    - cycle: build
      handoff: HANDOFF-178
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 41789158
      estimated_usd: 29.10
      duration_minutes: 34
      recorded_at: 2026-11-22
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (163 unique message.ids, all claude-opus-5-5, no subagents): input 337 + cache-creation 458,141 (1-hour) + cache-read 41,173,673 + output 157,007, opus-5-5 rates. Includes the verify follow-up turn (89b752a, 4089630). The handback floors were 33,190,077 / $22.58 for the build and 4,124,123 / $3.99 for the follow-ups. Duration is the two active turns (25 + 9), not the 177-minute transcript span with its idle wait. PR 306."
    - cycle: verify
      handoff: HANDOFF-179
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 17119033
      estimated_usd: 12.24
      duration_minutes: 23
      recorded_at: 2026-11-22
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (82 unique message.ids, all claude-opus-5-5, no subagents): input 168 + cache-creation 225,871 (1-hour) + cache-read 16,830,567 + output 62,427, opus-5-5 rates. APPROVED at 2267a34 with four follow-ups (tests and CHANGELOG wording), all closed on PR 306 before ship; handback carried into PR 306 by cherry-pick. AC4 reproduced exactly (6/6/1/2/5 codes, bound 12). The handback floor was 15,986,361 and $11.52."
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop orchestration (frame in PR 304, design with the SDK probe in PR 305); counted at stage grain in STAGE-027's orchestration_cost, not split per spec."
    - cycle: ship
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-22
      notes: "Un-metered main-loop ship; counted at stage grain in STAGE-027's orchestration_cost."
  totals:
    tokens_total: 58908191
    estimated_usd: 41.34
    session_count: 4
shipped_at: 2026-11-22
---

# SPEC-085: Apply CameraCalibration and AnalogBalance on the colour develop (DEC-076)

## Context

`STAGE-027` (PROJ-002), first of three. The parsed-but-unvalidated audit
(`docs/reviews/2026-11-21-parsed-but-unvalidated-audit.md`, §4 rank 1) found that
irradiance never reads `CameraCalibration1/2` (50723/50724), `AnalogBalance` (50727) or
the two calibration signatures (50931/50932). The DNG camera model is
`XYZtoCamera = AnalogBalance · CameraCalibration · ColorMatrix`. Adobe's
`dng_validate` applies it, and the colour develop does not.

**Re-measured at design (2026-11-22, `exiftool -n`, the held colour files):**

| File | Path | `CameraCalibration1` / `2` (diagonal) | Signatures (camera / profile) |
|---|---|---|---|
| Pixel 7 Pro | ColorMatrix (no FM) | (0.8594, 1, 0.3408) / (0.6162, 1, 0.5703) | absent / absent → applied |
| Pixel 4a | ForwardMatrix | (1.0235, 1, 0.9555) / (1.034, 1, 0.9764) | `com.adobe` / `com.adobe` → applied |
| Canon 5D Mark III | ForwardMatrix | (1.002, 1, 1.0062) both | `com.adobe` / `com.adobe` → applied |
| Pixel 3a | ColorMatrix (no FM) | identity both | absent / absent |
| Nikon D300S | ColorMatrix (no FM) | absent | absent |

Every file carries `AnalogBalance` (1, 1, 1) or none.

**The design probe** (DEC-076 §Context has the method). A local harness linked
against the already-built Adobe DNG SDK 1.7.1 objects prints the SDK's own
camera→XYZ(D50) matrix (`dng_color_spec::CameraToPCS` at `NeutralToXY(AsShotNeutral)`).
A model of today's code and of the designed code was compared with it. Codes are
the u16 linear-sRGB outputs of 18 probe values (six camera colours × three
channels, AC4):

| File | Today: max \|Δ entry\| / max \|Δ code\| | Designed: max \|Δ entry\| / max \|Δ code\| |
|---|---|---|
| Pixel 7 Pro | 0.1598 / **1160** | 0.00019 / 6 |
| Pixel 3a | 0.0072 / 39 | 0.00033 / 7 |
| Pixel 4a | 0.0051 / 23 | 0.00017 / 1 |
| Canon 5D Mark III | 0.0011 / 3 | 0.00047 / 2 |
| Nikon D300S | 0.0002 / 5 | 0.00016 / 5 |

The probe found **two** causes, not one:
- **The missing calibration.** On the Pixel 7 Pro it moves the scene white the
  SDK finds from xy (0.328, 0.403), well off the daylight locus, to (0.320, 0.333).
- **The weight loop stops early.** DEC-034's fixed 3-pass `NeutralToXY` was recorded
  as "dng_sdk's iteration count". SDK 1.7.1 iterates up to 30 passes and stops on
  convergence (`dng_color_spec.cpp`, `NeutralToXY`). The Pixel 3a, with no
  calibration at all, needs 19 passes; the 7 Pro needs 16. With the calibration but
  only 3 passes, the 7 Pro is still 148 codes off.

So this spec fixes both, in the one function they share. SPEC-035's ColorChecker ΔE
oracle (STAGE-008) would otherwise measure these errors, so this lands first.

## Goal

All four colour develop entries apply `AnalogBalance` and, under the DNG signature
rule, `CameraCalibration1/2`, interpolated as the SDK does. The `NeutralToXY` weight
loop runs to convergence. Every held colour file's camera→sRGB transform then
matches the SDK's within the pre-registered bound (AC4). On the Pixel 7 Pro that is
1160 codes off today and 6 in the probe. `DEVELOP_VERSION` is 8. Mono is untouched.

## Inputs

- **Decisions:** DEC-076 (this spec's rule and probe), DEC-033 (the matrix pipeline,
  `f64` basic ops), DEC-034 (the weight loop; §5 of DEC-076 amends its pass count),
  DEC-012 (interpretation tags cost their field), DEC-049 (`Sensor` is
  `#[non_exhaustive]`).
- **Code:** `src/color.rs` (`build_camera_to_srgb_linear`,
  `cam_to_xyz_single_illuminant`, `cam_to_xyz_interpolated`, `cm_path_cam_to_xyz`,
  `interpolation_weight`), `src/ifd.rs` (the colour `ifd0_*` reads near the
  `ForwardMatrix` ones, `Sensor`, `Sensor::test_fixture`), `src/bin/irr.rs` (`ifd`
  prints the colour fields), `src/lib.rs` (`DEVELOP_VERSION`).
- **Tests:** `tests/color_oracle.rs` (the independent reference pipeline and its
  dual-illuminant fixtures), `tests/colour_metadata_oracle.rs` (exiftool cross-check),
  the colour MD5 pins in `tests/gain_map_corpus.rs` and anywhere else a colour develop
  of a held file is pinned.
- **The SDK:** `dng_validate`'s objects are built at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/obj`.
  HANDOFF-178 carries the probe harness's source and link line.

## Outputs

- **Files modified:** `src/ifd.rs`, `src/color.rs`, `src/bin/irr.rs`, `src/lib.rs`,
  `tests/color_oracle.rs`, `tests/colour_metadata_oracle.rs`, the colour pins,
  `CHANGELOG.md`, `docs/provenance-ledger.md`, `docs/oracle-contract.md` if it lists
  the SDK lanes.
- **Files created:** `scripts/sdk-colour.sh` (builds the harness from the SDK objects
  and writes one matrix file per held colour file), `scripts/sdk-camera-to-pcs.cpp`
  (the harness; this repo's own code, never built in CI), `tests/colour_sdk.rs` (the
  SDK lane, AC4).
- **New public fields on `Sensor`** (additive, `#[non_exhaustive]`), each read from
  IFD0 with DEC-012's contract (absent → `None`; a malformed tag → `None` and its tag
  in `malformed_tags`):
  - `camera_calibration_1`, `camera_calibration_2: Option<Vec<Rational>>` (50723,
    50724; `SRATIONAL`);
  - `analog_balance: Option<Vec<Rational>>` (50727; `RATIONAL`);
  - `camera_calibration_signature`, `profile_calibration_signature:
    Option<Vec<u8>>` (50931, 50932; `ASCII` or `BYTE`). The bytes as written, cut at
    the first NUL. Any other field type is malformed.
  - Public tag constants beside `TAG_FORWARD_MATRIX_1`.
- **No new error variant.** An unusable applied calibration is the existing
  `Error::MissingColorCalibration`, as an unusable `ColorMatrix` is today.
- **No new flag or option.** The calibration is always applied when the rule says so.

## Design

1. **Parse** (`src/ifd.rs`). Add the five fields above beside the colour matrices, with
   `Sensor::test_fixture` setting each to `None`. `irr ifd` prints them. The ASCII/BYTE
   reader is new; keep it bounds-checked like `rationals` (`MAX_TAG_VALUES`, `payload`).
2. **The signature rule** (DEC-076 §2). `CameraCalibration1/2` apply only when the
   camera signature equals the profile signature, an absent signature counting as
   empty. So both absent applies (the Pixel 7 Pro), both `com.adobe` applies (the 4a,
   the Canon), and one present with the other absent or different does not. When the
   rule says no, both calibrations are the identity. `AnalogBalance` has no signature
   gate; absent, it is (1, 1, 1).
3. **Shape** (DEC-076 §3). Only what is used is checked:
   - `CameraCalibration1` when the rule applies it; `CameraCalibration2` when the rule
     applies it **and** the dual-illuminant path runs. Either one absent is the
     identity for its illuminant.
   - A used calibration must be 9 rationals with non-zero denominators; `AnalogBalance`
     must be 3 rationals with positive numerators and denominators.
   - Anything else is `Error::MissingColorCalibration`, before any write. A singular
     product is the same error, as a singular `ColorMatrix` is today.
4. **The transform** (DEC-076 §4). Keep today's convention that `g` weights
   illuminant 2.
   - Per illuminant, fold once: `X_i = AB · CC_i · CM_i`.
   - **ColorMatrix path:** `X(g) = (1 − g)·X_1 + g·X_2`, a lerp of the **products**
     (the SDK's form, which the probe separates from `AB · lerp(CC) · lerp(CM)` by 23
     codes on the 7 Pro). Then exactly today's structure with `inv(X(g))` in place of
     `inv(CM(g))`: `W = inv(X(g)) · AsShotNeutral`, `cam2xyz = CA(W → D50) · inv(X(g))`.
   - **ForwardMatrix path:** `K(g) = AB · ((1 − g)·CC_1 + g·CC_2)`,
     `R = inv(K(g)) · AsShotNeutral` (the reference neutral), and
     `cam2xyz = FM(g) · diag(1/R) · inv(K(g))`. Every `R` component must be positive,
     or `Error::MissingColorCalibration`. For a diagonal `K` this equals today's
     `FM(g) · D` mathematically, so on the held files the FM path moves only through
     `g`. A non-diagonal `K` is where it is observable.
   - **Single illuminant:** `X = AB · CC_1 · CM_1` and `K = AB · CC_1`.
   - **Identity is byte-identical** (DEC-076 §4). When `AB · CC` is exactly the
     identity (every held file except the Pixel 7, 4a and Canon), the code must take a
     path whose output is bit-identical to today's at the same `g`. In particular the FM
     path keeps today's `D` from the rationals (`den/num`), not `1/R` in `f64`, which
     differ in the last bit.
5. **The weight loop converges** (DEC-076 §5, amending DEC-034's pass count).
   - Start from the D50 chromaticity `(0.3457, 0.3585)`.
   - Each pass: `T = mccamy(xy)`; `g = clamp((1/T1 − 1/T)/(1/T1 − 1/T2), 0, 1)`;
     `next = xy(inv(X(g)) · AsShotNeutral)`.
   - Stop when `|next.x − xy.x| + |next.y − xy.y| < 1e-7`, taking `next`. Run at most 30
     passes; if the 30th has not converged, take the midpoint of the last two `xy`.
   - The final `g` comes from the final `xy`.
   - Every degenerate step still returns `None` (the single-illuminant fallback), and
     McCamy, the CCT table and the `f64` basic-ops-only discipline (DEC-033) are
     unchanged.
6. **`DEVELOP_VERSION` goes to `"8"`,** with all its pins, and a CHANGELOG
   `[Unreleased] → Changed` entry naming the files whose colour output moves and why.
7. **Docs.** `src/color.rs`'s module docs (the transform, the interpolation, the loop),
   the colour entries' `# Errors`, a provenance-ledger row (class 1: DNG 1.7.1.0
   Chapter 6; class 4: SDK 1.7.1 `dng_color_spec.cpp` read and recorded, no code
   copied), and DEC-034's text pointing at DEC-076 §5.
8. **The SDK colour lane** (`scripts/sdk-colour.sh`, `scripts/sdk-camera-to-pcs.cpp`,
   `tests/colour_sdk.rs`).
   - The script compiles the harness against the SDK headers, links it with the
     already-built SDK objects (minus `dng_validate.o`), and writes
     `<CAMERA>-camera-to-pcs.txt` per held colour file into an output directory that
     must be outside the repository (as `sdk-stage2.sh` requires). Nothing it writes is
     committed.
   - The test reads `$IRRADIANCE_SDK_COLOUR_DIR`. Set means must run (SPEC-079): a
     missing matrix file or a missing corpus file fails; unset, the lane prints a
     visible SKIP.
   - Its expected values come from the SDK's matrix and the test's own transcription of
     the XYZ(D50)→linear-sRGB constants, never from `src/color.rs`.

## Acceptance Criteria

**Pre-registered:** every bound below was set at design from the probe, before the
build. A number outside one is reported, never fixed by widening it.

- [ ] **AC1. Parse.** The five fields read on a writer fixture with each tag present.
  An `ASCII` signature and a `BYTE` signature both read, with a trailing NUL cut. A
  signature of another field type, and a calibration that is not `RATIONAL`/`SRATIONAL`,
  is `None` with its tag in `malformed_tags`, and the file still parses. Absent tags are
  `None` and not malformed. On the held files (Tier B), `tests/colour_metadata_oracle.rs`
  compares the three rational tags exactly and both signatures as strings against
  exiftool. Each comparison has a perturbed-value red-proof.
- [ ] **AC2. The transform, by hand (Tier A).** `tests/color_oracle.rs`'s independent
  reference gains the DEC-076 transform. It never reads `src/color.rs`. The public
  colour API equals it **bit for bit** on:
  - (a) single illuminant, ColorMatrix path, diagonal `CC_1` (the 7 Pro's shape);
  - (b) single illuminant, ForwardMatrix path, **non-diagonal** `CC_1` (the only shape
    where the FM path's `K` is observable);
  - (c) `AnalogBalance` ≠ (1, 1, 1) on both paths, with mismatched signatures, so
    `AB` applies and `CC` does not;
  - (d) dual illuminant, ColorMatrix path, `CC_1 ≠ CC_2` (the lerp of products);
  - (e) dual illuminant, ForwardMatrix path, non-diagonal `CC_1 ≠ CC_2`;
  - (f) a dual fixture whose 3-pass and converged weights differ, with both weights
    asserted in the test.

  On the ColorMatrix path, a neutral input `c · AsShotNeutral` stays grey in every
  fixture, as `a_neutral_subject_is_grey` asserts today.
- [ ] **AC3. The signature rule and shapes (Tier A).**
  - Develops equal to the same file without its calibration tags, bit for bit: camera
    `"a"` / profile `"b"`; camera present / profile absent; camera absent / profile
    present.
  - Develops equal to the calibrated reference: both absent; both equal.
  - `Error::MissingColorCalibration` on all four colour entries, with `dst` untouched:
    an applied `CC_1` of 8 entries; an applied one with a zero denominator; an
    `AnalogBalance` of 2 entries; one with a zero numerator; a `K` whose reference
    neutral has a non-positive component.
  - Develops `Ok`: an ill-shaped calibration the rule does **not** apply; an ill-shaped
    `CC_2` on a single-illuminant file.
  - **Identity:** a file carrying `CC_1 = CC_2 = I` and `AB = (1, 1, 1)` develops bit for
    bit as the same file without the tags, on both paths, single and dual.
- [ ] **AC4. The SDK lane, pre-registered** (`$IRRADIANCE_SDK_COLOUR_DIR`). For each of
  the five held colour files:
  - Probe values: for each tilt `t` in
    `(1, 1, 1), (1.3, 1, 0.8), (0.8, 1, 1.3), (1, 1.25, 1), (1.2, 0.9, 1.1),
    (0.9, 1.1, 0.85)`, the camera value is `round(0.25 · 65535 · AsShotNeutral_i · t_i)`
    per channel.
  - Ours: `to_srgb_linear_into` on a clone of the file's `Sensor` sized to exactly
    those six pixels.
  - Expected: `C · SDK · v`, clamped to `[0, 65535]` and rounded half up. `C` is the
    test's own XYZ(D50)→linear-sRGB constant.
  - **Bound: max |Δ| ≤ 12 codes over the 18 values, on every file.** The probe
    measured 6 (7 Pro), 7 (3a), 1 (4a), 2 (Canon), 5 (Nikon).
  - Record every file's max |Δ| and its 18 deltas in the handback, and today's values
    for contrast (the probe: 1160, 39, 23, 3, 5).
- [ ] **AC5. Collateral and pins.**
  - Every mono pin, on every held file, holds byte for byte.
  - Colour pins may move only on the five held colour files. List each one (file,
    entry, old → new hash, and which of the two causes moved it) in the handback.
  - In `tests/color_oracle.rs`, the dual-illuminant fixtures' expected values move with
    the reference's new loop. That is pre-approved: the reference changes, the
    assertions do not weaken. Any other existing assertion that moves is a
    stop-and-report.
  - Threaded colour output still equals single-thread.
- [ ] **AC6. Red-proof, control first.** Each mutant fails the named test by its
  target assertion. Record the table in the handback.
  - (a) Calibration ignored: AC2 (a), and AC4 on the 7 Pro (probe: 1160).
  - (b) The 3-pass loop: AC2 (f), and AC4 on the 7 Pro and 3a (probe: 148, 39).
  - (c) A lerp of the factors, `AB · lerp(CC) · lerp(CM)`: AC2 (d), and AC4 on the
    7 Pro (probe: 23).
  - (d) The signature rule always applying: AC3's mismatch rows.
  - (e) `AnalogBalance` dropped: AC2 (c).
  - (f) The FM path as today's `FM(g) · D`, ignoring `K`: AC2 (b).
  - (g) `CC_2` used on the single-illuminant path: AC3's ill-shaped-`CC_2` row.
- [ ] **AC7. Gates.**
  - `just test` with the corpus, and once more with `IRRADIANCE_SDK_COLOUR_DIR` set.
  - `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`.
  - `cost-audit`, `gate-audit`, `name-length-check`, `semver-check` (expect an
    additive minor: new public fields on a `#[non_exhaustive]` struct and new consts).
  - A short `fuzz` run of every target that builds a `Sensor`.
  - CI green.

## Failing Tests

Written first by the build (the design PR carries no code, as in SPEC-078). Each
DEC-076 clause maps to a test on the path where it changes the output:

- **§1, parse:** AC1's writer fixtures (`tests/ifd_reader.rs` or a new
  `tests/colour_calibration.rs`) and the exiftool comparison.
- **§2, signature rule:** AC3's five signature rows, observable on the ColorMatrix
  path. They must use a calibration that changes the output there; a diagonal
  calibration on a single-illuminant FM file changes nothing.
- **§2, `AnalogBalance` ungated:** AC2 (c).
- **§3, shapes:** AC3's error and `Ok` rows, through all four entries.
- **§4, ColorMatrix path:** AC2 (a) and (d).
- **§4, ForwardMatrix path:** AC2 (b) and (e), non-diagonal so `K` is observable.
- **§4, identity byte-identical:** AC3's identity rows and AC5's untouched pins.
- **§5, convergence:** AC2 (f), with both weights asserted.
- **§6, `DEVELOP_VERSION`:** its pins.
- **Real-file agreement:** AC4.

## Non-Goals

- **Refusals.** SPEC-086 refuses tags the develop cannot honour, and SPEC-087 makes a
  malformed interpretation tag refuse. Here a calibration tag of the wrong TIFF type
  still costs its field under DEC-012 and develops as if absent.
- **Triple-illuminant profiles** (`ColorMatrix3`, `CalibrationIlluminant3`, DNG 1.6).
  No held file has one. Today the third set is ignored; whether to refuse it is a
  SPEC-086 question, recorded there at its design.
- **The SDK's other differences:** Robertson CCT instead of McCamy, the SDK's
  `NormalizeForwardMatrix`, and its clipped Bradford. Together they are the ≤ 7-code
  residual in AC4.
- **The rest of the profile:** `ProfileHueSatMap`, `ProfileLookTable`,
  `ProfileToneCurve`, `BaselineExposure`, `ExtraCameraProfiles`. Only IFD0's default
  profile is read.
- **Mono.** The mono develop ignores colour metadata and stays byte-identical.

## Notes for the Implementer

- **The probe's numbers are the expected results** (DEC-076 §Context). The probe's
  Python model and harness are in HANDOFF-178.
- **The FM path trap:** with a diagonal `K`, the new formula equals the old one up to
  rounding, so a test on the held files cannot see the FM-path clause. AC2 (b) and (e)
  exist for that.
- **Bit identity under the identity calibration** is a hard requirement (AC3). The
  simplest way is to fold `AB · CC` only when it is not exactly the identity.
- **Commits:** plain-prose bodies with no code tokens. `Cargo.lock` is gitignored.
  Never commit corpus or SDK bytes, or the harness binary.

---

## Reflection

1. **What would I do differently next time?**
   — Probe before trusting a decision's attribution to the oracle. DEC-034 recorded a
   fixed 3-pass loop as "dng_sdk's iteration count" for months; the design-time SDK probe
   found it was not, and that the loop was half the Pixel 7 Pro's error. A one-hour probe
   at design turned an "apply the calibration" spec into the right fix. Also, the build
   ran about twice its 30M estimate (41.8M true, 33.2M floor): the six Pixel colour
   re-pins and the SDK lane each cost more than budgeted.

2. **Does any template, constraint, or decision need updating?**
   — DEC-034's pass count is amended by DEC-076 §5 (done at design). The verify's FU-1
   and FU-2 show a recurring test-design gap: a Tier-A fixture that only exercises the
   capped or degenerate branch (the loop hit pass 30; the white balance ½, 1, ½ made two
   formulas equal). Both closed before ship with red-proofs; not yet a recurring-lesson
   signal (N=1 spec).

3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec. Whether a dual file with a singular `ColorMatrix` or calibration that
   the converged weight never uses should develop or be refused is routed to SPEC-087
   (already framed). Emulating the SDK's Robertson CCT and Bradford clipping to close the
   last 1 to 7 codes stays deferred (DEC-076 alternatives) until SPEC-035's ΔE says it
   matters.

4. **Where was the worst defect caught?** — design

5. **What can a user do now that they couldn't before?** — A Pixel 7 Pro DNG now develops
   to within 6 codes of Adobe's DNG SDK colour (it was 1,160 codes off, because the camera
   calibration was ignored and the illuminant weight stopped after 3 passes); every held
   colour file is within 7 codes, gated at 12.
