---
insight:
  id: DEC-004
  type: decision
  confidence: 0.92
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-18
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - tests/**

tags:
  - oracle
  - testing
  - levels
  - spike-001
---

# DEC-004: Levels, crop and orientation are verified analytically — never by image comparison

## Decision

**Black/white level normalization, `ActiveArea` → `DefaultCrop` cropping, and
`Orientation` are verified by direct analytic assertion against tag values read
from the file — not by the plane checksum, and not by any perceptual metric.**

Concretely, the levels test asserts that normalization maps `BlackLevel` → 0.0
and `WhiteLevel` → 1.0 exactly, on values read from the file rather than
hardcoded; the geometry test asserts output dimensions and corner-pixel identity
against `ActiveArea`/`DefaultCropOrigin`/`DefaultCropSize`; and the orientation
test asserts the transform for the value **in the file**, on both a rotated and
an unrotated frame.

## Context

SPIKE-001 measured what the three-layer oracle actually catches. It found a hole
where two layers meet, and the hole is exactly where the levels work lives.

**The plane oracle is structurally blind.** `dnglab analyze --raw-checksum`
hashes the sensor plane with *no black subtraction* — that is the verified
contract, not an accident. Injecting `BlackLevel + 64` into a decode left the
plane checksum **bit-identical**. It cannot see a levels error by construction.

**The develop oracle does not cover it either**, which is the part nobody
expected. Simulating a black-level error as the affine change it truly is
(`y = ax + b`, `a = 15871/(16383−B)`, `b = (512−B)/(16383−B)`) and scoring against
`dnglab --srgb` with SSIMULACRA2:

| `BlackLevel` used instead of 512 | SSIMULACRA2 | at the ≥85 bar (DEC-005) |
|---|---|---|
| 513 (+1) | 100.00 | passes |
| 528 (+16) | 100.00 | passes |
| 576 (+64) | 95.62 | passes |
| 768 (+256) | 87.51 | **passes — a 50% levels error** |
| 1024 (+512) | 73.16 | caught |

A black-level error must reach **~50% of the entire black level** before the
develop oracle notices. This is not a defect in SSIMULACRA2 — it is the metric
working as designed. It models *perception*, and a levels error is very nearly an
affine tone change, which perception forgives. Using it to check levels is a
category error.

Two blind layers, two different reasons, one uncovered surface. Without this
decision, STAGE-002's "black/white level normalization, ActiveArea → DefaultCrop,
and orientation" spec would ship green with a wrong black level — precisely the
`oracle-must-be-shown-red` failure the constraint exists to prevent.

## Alternatives Considered

- **Option A: tighten the SSIMULACRA2 threshold until levels errors are caught.**
  - Why rejected: the measurements say the bar would have to sit near **95** to
    catch a +64 error, and 95 is *below* what a legitimate tone-curve
    implementation difference can cost (gamma 1.05 alone scores 88.5). The bar
    would fire constantly on correct renders and never on the fault it was raised
    for. Tuning one threshold to serve two unrelated failure modes serves neither.

- **Option B: rely on the develop oracle and accept the gap.**
  - Why rejected: a wrong black level is not cosmetic. It shifts every tonal
    value in the image, silently, in a library whose entire proposition is
    correctness that can be demonstrated.

- **Option C: a synthesized fixture with analytically known levels.**
  - Why partially adopted: correct in principle and worth having, but SPIKE-001
    established that `dnglab makedng` **cannot** produce a monochrome fixture
    (PPM input only; emits 3-sample/16-bit/JPEG). So this needs a hand-built
    header, which is a larger piece of work. Keep it as a complement, not the
    primary mechanism.

- **Option D (chosen): assert the arithmetic directly, against tags read from
  the file.**
  - Why selected: it tests the actual invariant rather than a proxy for it; it
    needs no oracle tooling, no corpus, and no network; it runs in CI where
    tier-B files are absent (DEC-003); and it is trivially shown red — change the
    black level and the assertion fails immediately and unambiguously.

## Consequences

- **Positive.** The one surface both oracles miss is covered by the cheapest
  possible check. It runs in CI, unlike anything corpus-dependent (DEC-003).
  Reading the levels from the file rather than hardcoding them also means the
  test does not silently encode "512/16383" as a universal truth — it is a
  per-file property, exactly like `Orientation` proved to be.

- **Negative.** An analytic assertion verifies the arithmetic we *chose*, not
  that our choice matches Adobe's intent. If the DNG spec's normalization
  convention is subtly different, this test passes and the render is still wrong.
  Mitigation: the develop oracle (DEC-005) still catches a *gross* levels error,
  so the two are complementary rather than redundant — neither alone is
  sufficient, which is the whole finding.

- **Neutral.** Adds a spec to STAGE-002 that its backlog did not have.

## Validation

Right if a deliberately wrong `BlackLevel` turns this test red **immediately**,
and if that red-proof ships with it (constraint `oracle-must-be-shown-red`).

Revisit if a hand-built monochrome fixture with analytically known levels becomes
available — it would strengthen, not replace, this.

## References

- Evidence: `spikes/done/SPIKE-001-*.md`, session of 2026-08-18
- Companion: `DEC-005` (the develop oracle's mechanics and tolerance)
- Constraint: `oracle-must-be-shown-red`
- Contract: `docs/oracle-contract.md`
