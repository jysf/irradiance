---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-047
  type: decision
  confidence: 0.8
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

created_at: 2026-11-10
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/plane.rs
  - src/ljpeg.rs
  - src/lib.rs
  - fuzz/fuzz_targets/ljpeg.rs
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-053-compression-seven-decoded-frame-shape-binding.md
  - docs/provenance-ledger.md

tags:
  - ljpeg
  - compression-7
  - shape-binding
  - sof3
  - sampling-factors
  - stage-010
  - spec-053
---

# DEC-047: Bind the decoded SOF-3 frame shape to the strip, and reject H/V != 1

## Decision

The Compression-7 strip path must bind the **decoded SOF-3 frame's effective
shape** to the strip's declared geometry, not merely its area, and must reject a
frame whose component sampling factors are not `H = V = 1`. Two additive guards,
both closing silent-success paths (review finding #2 and the
`unexercised-sof3-paths-garble-silently` signal):

1. **Shape binding.** In `unpack_jpeg` (`src/plane.rs:487`), before
   `decode_into`, parse the strip's SOF-3 headers and require the frame's
   **effective plane width** `samples_per_line × components` to equal
   `sensor.width` and its height `lines` to equal this strip's `rows_in_strip`.
   On a mismatch return a new typed `Error::LjpegPlaneShapeMismatch`.
2. **Sampling-factor rejection.** In `parse_frame_header` (`src/ljpeg.rs:436`),
   reject any component spec whose `Hi/Vi` byte is not `0x11` (H=1, V=1) with a
   new typed `Error::LjpegUnsupportedSampling`.

## Why

`decode_into` fills `out` **row-major by the frame's own width**
(`samples_per_line`, times the two column phases for `Nf=2`), and the only
geometry check is `out.len() == effective_width × height`
(`LjpegPlaneWrongLength`, `src/ljpeg.rs:993`/`:1182`). `unpack_jpeg` sizes the row
band to `rows_in_strip × sensor.width` and **drops the returned `Frame`**
(`src/plane.rs:577-582`), trusting that area check to pin the dimensions. It does
not: a frame that is an equal-area transpose of the band (e.g. decoded 3×4 vs a
4×3 band) satisfies `out.len() == area`, is written with the wrong row stride, and
returns `Ok(())` with a silently garbled plane. Separately,
`parse_frame_header` reads each component's `Hi/Vi` byte and **discards it**
(`src/ljpeg.rs:468-472`): a subsampled frame (`H/V != 1`, out of scope per
`DEC-037`) is decoded as if `H=V=1`, also garbling silently. Both are the same
failure class as finding #1 — a hostile or unsupported input that produces a
plausible wrong result instead of a typed error.

## Mechanics

- **Where the shape check lives: the caller.** `decode_into` stays a generic
  "decode this SOF-3 stream into this buffer" primitive; the *expected* geometry
  is the strip's, known only to `unpack_jpeg`. So `unpack_jpeg` calls
  `ljpeg::parse_headers(strip)` (already `pub(crate)`) to read the frame, checks
  `samples_per_line × components == sensor.width` and `lines == rows_in_strip`,
  then calls `decode_into`. The second header parse is negligible (headers are
  tens of bytes). **Rejected alternative:** threading the expected `(width,
  height)` into `decode_into` — it changes a stable signature, ripples into the
  fuzz hook (`__fuzz_ljpeg`, which has no external "expected" width) and the
  tests, and moves strip-geometry knowledge into a generic decoder.
- **`effective_width = samples_per_line × components`** handles both counts: for
  `Nf=1` it is `samples_per_line`; for `Nf=2` the de-interleaved plane is
  `2 × samples_per_line` wide (`SPEC-046`), which is `sensor.width`.
- **New variants** (both additive; `Error` is `#[non_exhaustive]`,
  `src/lib.rs:217`), with `Display` arms modelled on the `Ljpeg*` family:
  - `Error::LjpegPlaneShapeMismatch { expected_width: u32, expected_height: u32, decoded_width: u32, decoded_height: u32 }`
  - `Error::LjpegUnsupportedSampling { factor: u8 }`
  Both are **Corrupt**-class for SPEC-040's future `Error::kind` table (a declared
  geometry that disagrees with the decode / an unsupported geometry).
- **`DEVELOP_VERSION` does not move.** Every input that decodes correctly today
  (shape matches, `H=V=1`) decodes identically; only inputs that previously
  returned a wrong `Ok` now return a typed error. No valid output changes.

## Scope and non-goals

- **In:** the two guards above; the two error variants + `Display`; the fuzz
  target asserting that a `decode_into` `Ok` implies the decoded shape matched the
  buffer's intended shape; a provenance-ledger row; CHANGELOG.
- **Out:** restart-interval (`DRI`) MCU-reset hardening (a separate item);
  *supporting* subsampled frames or any new SOF process (this rejects, does not
  implement); any change to a correctly-shaped `H=V=1` decode's output; the
  parser resource guard (SPEC-052) and the licence/package work (SPEC-055).
