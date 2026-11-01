---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-031
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-10-31
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# Amends the project brief + STAGE-007, which named Malvar-He-Cutler as the
# quality demosaic. The CFA sensor model, the bit-exact plane oracle, and the
# SPEC-030 bilinear foundation are untouched.
affected_scope:
  - src/demosaic.rs
  - projects/PROJ-002-bayer-colour-develop/brief.md
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-007-demosaic-clean-room-malvar-he-cutler-mosaic-to-camera-linear-rgb.md

tags:
  - demosaic
  - colour
  - patent
  - licensing
  - hamilton-adams
  - malvar-he-cutler
  - spec-031
  - stage-007
---

# DEC-031: The STAGE-007 quality demosaic is Hamilton-Adams, not Malvar-He-Cutler — MHC is patent-encumbered

## Decision

STAGE-007's quality demosaic (SPEC-031, the kernel above `SPEC-030`'s bilinear
baseline) is the **Adams-Hamilton** directional algorithm (Hamilton & Adams,
US 5,629,734, Eastman Kodak, 1995 — **expired**, Google Patents "Expired -
Lifetime"), **not** the **Malvar-He-Cutler** method the project brief and STAGE-007
originally named. MHC is covered by **US 7,502,505 B2** (Microsoft; Malvar, He &
Cutler), which Google Patents lists as **"Active, expires 2027-01-25."** A
patent-encumbered demosaic in a permissive (MIT OR Apache-2.0) crate defeats the
project's whole reason to exist — the *unencumbered* pure-Rust RAW primitive —
because **clean-room reimplementation defeats copyright, not patents**: an
independent implementation of MHC still reads on the live `'505` method claims.
Hamilton-Adams is a genuine quality step over bilinear (directional green +
colour-difference red/blue), is the de-facto demosaic of the open-source RAW
ecosystem (dcraw / LibRaw / RawTherapee / darktable ship it; they conspicuously
avoid MHC), and is patent-clean **permanently** — so it is chosen as the quality
kernel outright, not as a stopgap until `'505` expires.

## Context

STAGE-007 was framed (and PROJ-002's brief written) with "clean-room
Malvar-He-Cutler" as the quality demosaic, on the reasoning that MHC is a fixed
linear filter whose output is exactly computable (a bit-exact analytic oracle).
That reasoning holds — but a patent check run while designing SPEC-031 (the MHC
spec) surfaced what the framing had not: **MHC is patented and the patent is still
in force.** `US 7,502,505` was filed 2004-03-15, granted 2009-03-10, and carries a
patent-term adjustment pushing its expiry to ~2027-01-25 (Google Patents shows
"Active"; no fee-lapse label, so maintenance fees appear paid). The base 20-year
term (2024-03) is already past; only the PTA keeps it alive, for ~3 more months
from this decision.

irradiance's thesis ([[irradiance-adoption-outreach-checklist]]) is that *all*
existing Rust RAW crates are copyleft and the opening is a permissive one that
oculante / immich / photoprism / the `image` ecosystem can adopt without a
licensing problem. A live patent is a *worse* adoption blocker than copyleft — it
exposes every downstream user, not just those who link the code. Shipping MHC now,
clean-room or not, would plant exactly the kind of flag those adopters avoid.

Hamilton-Adams (Adams-Hamilton), Kodak's 1990s directional method, is the obvious
patent-clean quality demosaic: `US 5,629,734` (filed 1995) is expired, the method
is ~20 years old and universally implemented, and it fits SPEC-030's bit-exact
analytic-oracle harness unchanged (deterministic integer math: a gradient
classifier picks the green interpolation axis, then red/blue come from
bilinearly-interpolated colour differences).

## Alternatives Considered

- **A: Ship clean-room MHC now (the brief's plan).**
  - Rejected: `'505` is live to ~2027-01-25; clean-room does not touch a patent.
    Planting a patented method in the permissive crate contradicts the project's
    value thesis and endangers the very adopters it targets. Not worth ~3 months of
    marginally-higher PSNR.
- **B: Defer the quality kernel ~3 months — keep bilinear, add MHC after `'505`
  expires.**
  - Rejected as the primary plan (kept as the fallback): it leaves demosaic quality
    at bilinear (visible zippering / false colour) for a quarter, and it still ends
    at a once-patented method when a permanently-clean one of comparable quality is
    available today. Chasing MHC post-expiry buys little over Hamilton-Adams.
- **C (chosen): Hamilton-Adams now, as the permanent quality kernel.**
  - Patent-clean forever (US 5,629,734 expired ~2015), a real quality jump over
    bilinear, ecosystem-proven, and deterministic integer math so SPEC-030's
    bit-exact analytic oracle + red-proof transfer unchanged. Removes the MHC patent
    question from the project entirely.
- **D: A newer high-quality method (AHD / AMaZE / LMMSE / RCD).**
  - Rejected for STAGE-007: higher quality but materially more complex, and each
    needs its own patent clearance (AHD/Hirakawa 2005 is recent enough to warrant
    one). Out of proportion to the stage's goal — a correct, verifiable, clean
    quality step above bilinear. A later project may revisit.

## Consequences

- **Positive.** The crate stays patent-clean permanently; the permissive-adoption
  thesis is protected. Hamilton-Adams is a substantial quality gain over bilinear
  (directional green removes most zippering) and keeps STAGE-007's bit-exact
  verification discipline — the algorithm is fixed deterministic integer arithmetic,
  so the SPEC-030 analytic-oracle + red-proof harness is reused, not rebuilt.
- **Positive.** The decision is made from evidence (the patent check), recorded
  before any patented code was written — the measure-before-build discipline
  catching a licensing landmine the framing missed.
- **Negative — the honest cost.** The project brief and STAGE-007 named MHC
  throughout; this DEC amends them (the prose is updated to Hamilton-Adams, citing
  this DEC). Hamilton-Adams is marginally lower PSNR than MHC on some test images —
  an acceptable trade for permanent patent-cleanliness. The patent status rests on
  Google Patents' reading ("Active" for `'505`, "Expired - Lifetime" for `'734`),
  which is "an assumption, not a legal conclusion"; a definitive check is USPTO
  Patent Center. For a permissive OSS crate implementing a long-expired, universally
  shipped algorithm (`'734`) and *avoiding* a live one (`'505`), that is a
  reasonable basis — and if `'505` is later found to have lapsed early, nothing here
  needs revisiting, because Hamilton-Adams is chosen on its own merits.

## Validation

Right if: SPEC-031 ships a Hamilton-Adams demosaic, verified bit-exact on analytic
fixtures (per SPEC-030's harness) with a red-proof, implementing only the expired
`'734` method and no claim of `'505`. Revisit if: a credible claim surfaces that
Hamilton-Adams reads on a *live* patent (then re-clear or fall back to bilinear), or
if a future project wants MHC/a newer method after `'505` expiry — that is its own
spec and DEC, and this decision does not pre-commit to it.

## References

- Amends: the PROJ-002 brief and STAGE-007 (both named Malvar-He-Cutler); both
  updated to Hamilton-Adams, citing this DEC. Builds on `SPEC-030` (bilinear
  foundation + the bit-exact analytic oracle this reuses) and `DEC-016/017/018`.
- Patents: `US 7,502,505 B2` (MHC, Microsoft — Active, ~2027-01-25) and
  `US 5,629,734 A` (Adams-Hamilton, Kodak — Expired - Lifetime), both read on
  Google Patents 2026-10-31 (status is an assumption, not a legal conclusion;
  USPTO Patent Center is definitive).
- Algorithm: Hamilton & Adams, "Adaptive color plan interpolation in single sensor
  color electronic camera" (US 5,629,734, 1997); reviewed in the open-source RAW
  demosaic literature (IPOL; dcraw / LibRaw / RawTherapee / darktable).
- Related: `SPEC-031` (this change), `SPEC-030` (bilinear + harness), `SPEC-032`
  (CFA fuzz), `STAGE-007`. Constraints: `oracle-must-be-shown-red`,
  `no-panics-on-untrusted-input`.
