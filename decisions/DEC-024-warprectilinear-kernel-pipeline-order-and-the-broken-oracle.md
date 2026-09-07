---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-024
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-06
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/warp.rs
  - src/develop.rs
  - src/opcode.rs
  - tests/warp.rs

tags:
  - warp
  - opcode
  - kernel
  - oracle
  - spec-018
  - spec-020
---

# DEC-024: `WarpRectilinear` kernel is bilinear, applied over `ActiveArea` before `DefaultCrop`; `SPEC-020`'s oracle cannot validate it

## Decision

Ship **bilinear** resampling for `WarpRectilinear` (DNG 1.7.0.0 §6.4.1),
applied over the **`ActiveArea`-sized** image **before** `DefaultCrop`
extraction and `Orientation` — not, as `SPEC-018`'s own design-time text
assumed, after them. Correctness is established **analytically**
(`tests/warp.rs`'s `AC4`/`AC10`, an independent transcription of the DNG
spec's own formula, cross-checked against `SPIKE-001`'s measured ~504 px
figure), **not** via `SPEC-020`'s SSIMULACRA2-vs-`dnglab` oracle — that
oracle cannot see this feature at all: `dnglab`/`rawler` do not implement DNG
`OpcodeList` processing.

## Context

`SPEC-018`'s pre-registered rule was: try bilinear, measure the SPEC-020
oracle score on all three decodable Q2M frames, ship bilinear if all score
`>= 85`, escalate to bicubic/Lanczos-3 otherwise. Executing that rule
surfaced two independent, unanticipated findings that this DEC exists to
record.

### Finding 1 — the pipeline order `SPEC-018`'s own design carried was wrong

`docs/measured-q2m-dng.md` and this spec's `## Context`/`## Implementation
Context` stated "`OpcodeList3` runs after cropping and orientation." Reading
DNG 1.7.0.0 §Chapter 4 directly during build (the design-time probe this
spec's `## Implementation Context` calls for) found the opposite:
`DefaultCropOrigin`/`DefaultCropSize` are described as specifying "the
origin/size of the **final** image area... relative to the top-left corner
of the `ActiveArea` rectangle" (p.16-17, paraphrased from the exact clause
transcribed in `src/develop.rs`'s module doc) — "final" because `DefaultCrop`
extracts the last, for-display sub-rectangle from an image that has already
been through every earlier raw-domain processing stage, opcode lists
included. `OpcodeList3`'s own tag description says it applies "just after
\[the raw image\] has been demosaiced" (DNG 1.7.0.0 p.56-57) — i.e. over the
full `ActiveArea`, not the smaller `DefaultCrop` rectangle.

`SPEC-018`'s handoff, spec, and `SPIKE-001` all assumed `DefaultCrop` size
(8368x5584) as the warp's own coordinate extent; the DNG-spec-correct extent
is `ActiveArea` size (8392x5632). For Q2M the two sizes differ by under 1%,
so this specific correction changes almost nothing numerically — but a
future camera with a much larger crop margin would be materially wrong under
the original (uncorrected) assumption. Fixed in `src/develop.rs`:
`normalize_active_area_into` → `warp::apply_warp_into` (at `ActiveArea` size)
→ `crop_and_orient_from_active_into`, replacing the single fused
crop+orient+normalize pass for the warp-bearing case (unchanged for the
warp-free case, which stays the original one-pass, zero-extra-allocation
path). `AGENTS.md` §16 rule 4 (`unrun-docs-carry-errors`) instance: a design
document's stated pipeline order, carried from a spike into a spec into a
handoff, was wrong, and only reading the spec's own tag descriptions during
build caught it.

### Finding 2 — `SPEC-020`'s oracle cannot see `WarpRectilinear` at all

Measuring the pre-registered bilinear kernel against `dnglab analyze --srgb`
on `L1021223.DNG` scored **-60.169** (`AC8`) — deeply negative, worse than
the `DEC-005` "missing warp" calibration (-68.05 at ¼ res, -82.338 at full
res on synthetic input), not close to `>= 85`. A synthetic **NO-WARP**
baseline (same pipeline, warp stage skipped) scored much better once a
matching approximate sRGB gamma was applied for the comparison (**83.145**,
near the 85 bar) — `develop_into`'s own output is linear (`DEC-018`;
`SPEC-019`'s tone curve has not landed), and `dnglab --srgb` is
gamma-encoded, so an apples-to-apples comparison needs a stand-in gamma; this
was applied only for this diagnostic, never in the shipped pipeline.
Applying the correct warp made the match to `dnglab` WORSE (-60.169), not
better (83.145 without it) — the opposite of what a correction should do
against a correct reference. `AC9`'s own red-proof shows the same pattern:
honest score -60.193, `kr1`-zeroed mutated score -55.075 — the mutation
technically satisfies `AC9`'s literal `< 85` assertion, but only because the
HONEST score never reaches 85 either; this is not the intended "passes,
then a mutation makes it fail" shape, and `tests/warp.rs`'s own comment on
that test says so.

Root cause, confirmed by inspecting `dnglab`/`rawler`'s own source
(`github.com/dnglab/dnglab`, run as a tool per `provenance-recorded-per-
algorithm` — inspecting for FEATURE PRESENCE, not reading an algorithm to
port it): `OpcodeList1`/`2`/`3` appear in the whole repository **only** as
tag-ID constants (`rawler/src/tags.rs`) and `IFD::copy_tag(...)`
pass-through calls (`rawler/src/decoders/dng.rs`, used when rawler itself
writes a DNG) — never as opcode APPLICATION logic. `WarpRectilinear` and
`FixBadPixelsConstant` do not appear anywhere in the codebase. **`dnglab
analyze --srgb`'s rendered output never applies any DNG opcode list**, ours
included. `docs/oracle-contract.md`'s own "This oracle is single-sourced"
already warned bit-identity with `dnglab` proves conformance to `rawler`, not
correctness — this is the sharper, previously-unmeasured version of that
warning: for this ONE pipeline stage, `dnglab` provides **no signal at all**,
in either direction. A more geometrically correct warp diverges further from
`dnglab`'s uncorrected reference, not less.

This is the same shape of gap `SPIKE-001` found for levels (`docs/
measured-q2m-dng.md`'s "🔴 THE FINDING THAT MATTERS — neither oracle covers
levels", which produced `SPEC-015`'s analytic oracle): a real pipeline stage
with no comparison-oracle coverage, requiring an **analytic** check instead
of an image-comparison one.

## Alternatives Considered

- **Option A: bicubic or Lanczos-3, chasing the `>= 85` score.** Rejected —
  measured directly: the negative score is not a resampling-quality artifact
  a sharper kernel could close. It is a structural mismatch between "our
  output has a real correction" and "the reference has none at all." No
  kernel choice changes that. Chasing the number here would be exactly the
  anti-pattern `## The design decision this spec rests on` forbids ("a
  kernel that needs tuning to pass is a wrong kernel") — worse, in this case
  no amount of tuning CAN pass, so continuing to try would be pure waste.
- **Option B: relax `DEC-005`'s `>= 85` threshold for warp-bearing files.**
  Rejected for this build to decide unilaterally — threshold changes are an
  architect/verify-cycle call (`## What this handoff does NOT ask for`:
  "Recalibrating `DEC-005`'s threshold... happens when SPEC-018 lands, but
  it's not one of this spec's acceptance criteria"). Recorded here as a
  `FU-N` for ship to disposition, not decided in this DEC.
  - Rejected on the merits too, if considered as a genuine option:
    lowering the threshold would not restore a signal `dnglab` fundamentally
    cannot provide — the right fix is a different oracle (Option C), not a
    lower bar on a meaningless one.
- **Option C (chosen): keep bilinear (the pre-registered default), rely on
  the analytic checks (`AC4`, `AC10`) for correctness, and record `AC8`/`AC9`
  as measured-but-not-meaningful.** `AC4` independently derives the exact
  DNG 1.7.0.0 §6.4.1 formula, verifies it against `SPIKE-001`'s own measured
  ~504 px figure using each frame's own per-frame coefficients (not
  hardcoded), and `AC10` proves the check has teeth (a `kr1`-zeroed mutation
  moves the detected peak by 339.5 px on `L1021223.DNG`, far past the 20 px
  bar). Bilinear is still the right kernel choice on `DEC-002`'s
  determinism/simplicity grounds even absent a passing oracle score — nothing
  about the broken oracle argues for a MORE complex kernel, since the defect
  it would be compensating for does not exist.

## Consequences

- **Positive.** The shipped pipeline order (`ActiveArea` → warp →
  `DefaultCrop` → `Orientation`) is spec-correct, not oracle-fitted — the
  right property for a library whose whole differentiator is spec fidelity
  (`AGENTS.md` §1).
- **Positive.** `tests/warp.rs`'s `AC4`/`AC10` give this repo an
  oracle-independent, falsifiable correctness check for `WarpRectilinear`
  that will keep working even if `dnglab`'s behavior (or availability)
  changes.
- **Negative.** `AC8`/`AC9` as literally specified cannot pass with ANY
  correct implementation. This DEC does not resolve that tension — it is
  recorded as a follow-up for ship to disposition (either drop the
  dnglab-oracle requirement for `WarpRectilinear` specifically, matching
  `SPEC-015`'s precedent of substituting an analytic check where no
  comparison oracle exists, or find a second reference decoder that DOES
  implement DNG opcode lists).
- **Negative.** `develop_into` now allocates two `ActiveArea`-sized scratch
  buffers internally when a real warp is present — peak RSS on
  `L1021223.DNG` measured **465,010,688 bytes**, up from `SPEC-014`'s
  275,890,176 (`/usr/bin/time -l target/release/irr develop`). `DEC-018`'s
  original "O(1) working memory" claim no longer holds for any real Q2M
  frame, all of which carry a mandatory `WarpRectilinear`. See
  `src/develop.rs`'s own "Allocation" doc and `docs/provenance-ledger.md`'s
  `src/warp.rs` row.
- **Neutral.** The out-of-extent pixel rule (DNG 1.7.0.0 §6.4.1 is silent —
  confirmed by full-text search of the specification) is clamp-to-edge,
  chosen for simplicity and determinism; `AC7`
  (`outside_pixels_follow_the_dng_spec_rule`, `src/warp.rs`) pins this
  choice with a hand-built coefficient set that genuinely exercises it (real
  Q2M coefficients never do, since `f(r) <= kr0 < 1` for every measured
  frame keeps every real destination pixel's source inside the extent by
  construction).

## Validation

Right if: `tests/warp.rs`'s `AC4` continues to reproduce each frame's own
`f(1)`-derived corner displacement within 1 px, and `AC10`'s red-proof
continues to show a large (>=20 px, measured 339.5 px) separation on a
`kr1`-zeroed mutation. Revisit if: (a) a second DNG-opcode-capable reference
decoder becomes available, restoring a real signal for `AC8`/`AC9`, or (b) a
future camera's `ActiveArea`/`DefaultCrop` gap is large enough that the
pipeline-order correction (Finding 1) produces a materially different result
from the original (uncorrected) assumption — worth a fresh measurement, not
assumed to still be "under 1%."

## References

- Related specs: SPEC-018, SPEC-020, SPIKE-001
- Related decisions: DEC-002 (parallelism/determinism), DEC-005 (develop
  oracle mechanics, `>= 85` threshold), DEC-018 (u16 output representation),
  DEC-016 (caller-owned buffers)
- External: DNG 1.7.0.0 specification (Adobe), §Chapter 4 (`DefaultCropOrigin`/
  `Size`), §Chapter 7 ("Opcode List Processing"), §6.4.1 ("WarpRectilinear");
  `github.com/dnglab/dnglab` (`rawler/src/tags.rs`,
  `rawler/src/decoders/dng.rs`) — inspected for opcode-processing feature
  presence, run as a tool per `provenance-recorded-per-algorithm`, never read
  for algorithm content.
