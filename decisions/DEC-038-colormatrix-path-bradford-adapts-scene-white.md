---
insight:
  id: DEC-038
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5-5
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-05
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]
affected_scope: ["src/color.rs", "tests/color_oracle.rs"]

tags:
  - color
  - determinism
  - develop-version
---

# DEC-038: The ColorMatrix-only path Bradford-adapts the scene white instead of applying the WB diagonal

## Decision

When no `ForwardMatrix` is present, `cam2xyz = CA(W to D50) . inv(CM)` with
`W = inv(CM) . AsShotNeutral`; the white-balance diagonal `D` is no longer applied
on that path. `DEVELOP_VERSION` is bumped from "1" to "2".

## Context

`DEC-033` built `cam2xyz = inv(CM) . D`. `D = diag(1/asn)` already maps a neutral
subject to (1, 1, 1), but `inv(CM)` expects raw camera values, so white balance was
counted twice: on the Nikon D300S (CM1 and CM2, no FM) grey developed to linear
sRGB (1.0, 0.49, 0.78), visibly magenta. Reported by pyirradiance. The
ForwardMatrix path was correct (FM already expects white-balanced input).

## Rationale

DNG specification chapter 6: with only ColorMatrix, the scene white is
`inv(CM) . neutral`, adapted to D50 by Bradford. Then raw `c = asn` maps to D50
white (Y = 1), which the committed `C = S . B` takes to sRGB (1, 1, 1). Bradford
from an arbitrary white needs the committed cone matrix, its inverse and a diagonal
of ratios: `+ - * /` only, so `DEC-033`'s determinism rules hold.

## Consequences

- Output changes for every ColorMatrix-only file (single and interpolated), hence
  the `DEVELOP_VERSION` bump. ForwardMatrix files are byte-identical.
- A degenerate scene white (non-positive cone response) is
  `Error::MissingColorCalibration`.
- Regression: `tests/color_oracle.rs` develops a flat grey mosaic on the FM,
  CM-only and dual-CM paths and requires R = G = B; shown red on the old code.
- Why `SPEC-035`'s delta-E missed it: SPEC-035 (ColorChecker) has not run, since it
  is blocked on the chart, so no colour-accuracy gate covered ColorMatrix-only
  files.
