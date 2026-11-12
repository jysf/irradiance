---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-058
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-12
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/develop.rs
  - src/lib.rs
  - fuzz/fuzz_targets/develop.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-063-developoptions-develop-with-into-non-exhaustive-options-with.md

tags:
  - develop
  - options
  - public-api
  - non-exhaustive
  - develop-version
  - stage-019
---

# DEC-058: DevelopOptions is a non-exhaustive struct whose Default is develop_into, and DEVELOP_VERSION covers every option value

## Decision

### 1. Shape

The mono develop takes one options value:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct DevelopOptions {
    pub framing: Framing,
    pub apply_orientation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Framing {
    #[default]
    StandardCrop,
    FullActiveArea,
}

pub fn develop_with_into(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    options: &DevelopOptions,
) -> Result<(), Error>;

pub fn output_dimensions_with(
    sensor: &Sensor,
    options: &DevelopOptions,
) -> Result<(u32, u32), Error>;
```

- **Fields are `pub` and the struct is `#[non_exhaustive]`** (DEC-052). Outside the crate a
  caller writes `let mut o = DevelopOptions::default(); o.framing = Framing::FullActiveArea;`.
  A struct literal does not compile, so adding a field later is not a breaking change. There
  are no builder methods: one way to set a field is enough.
- **`PartialEq` but not `Eq` or `Hash`.** SPEC-064 adds `warp_alpha: f64`, which cannot be
  `Eq`. Deriving `Eq` now and dropping it then would break callers.
- **Options come last** in `develop_with_into`, after the two buffers, so the call reads like
  `develop_into` with one extra argument. The scratch variant (SPEC-065) keeps that order.
- **Mono only.** The colour path (`develop_color_into` and friends) takes no options. The doc
  comments say so. Wiring colour is a separate decision with its own caller.

### 2. `Default` is `develop_into`, forever

`develop_into(sensor, src, dst)` is exactly
`develop_with_into(sensor, src, dst, &DevelopOptions::default())`. It delegates, so the two
cannot drift, and `output_dimensions(sensor)` likewise delegates to
`output_dimensions_with(sensor, &DevelopOptions::default())`.

Every field added later must have a default that reproduces the output from before it
existed. `Default` is `StandardCrop` and `apply_orientation: true`.

### 3. Options override, they never skip validation

An option that makes a tag unused does **not** stop that tag being validated:
- `FullActiveArea` ignores `DefaultCropOrigin`/`DefaultCropSize` for output, but a crop
  that does not fit `ActiveArea` is still `InvalidDefaultCrop`.
- `apply_orientation: false` ignores `Orientation` for output, but a value outside `1..=8`
  is still `UnsupportedOrientation`.

The same file then refuses or develops the same way under every option value. A
malformed tag is evidence of a malformed file (DEC-012), and a caller cannot use an
option to get past that evidence.

Implementation shape: `resolve_geometry` runs unchanged, then the options rewrite the
resolved `Geometry` (crop = the whole active area; orientation = 1). Everything downstream
reads the rewritten geometry, so no pass grows a second code path.

### 4. A combination that is not yet safe is refused, typed

When an option value cannot be honoured correctly yet, `develop_with_into` returns a typed
`ErrorKind::Unsupported` error. It never falls back silently to another behaviour.

The one such case in SPEC-063 is **`FullActiveArea` together with a non-identity
`WarpRectilinear`**, refused as `Error::UnsupportedWarpedFullActiveArea`. The warp clamps
out-of-extent samples to the nearest edge (DEC-024). Under `StandardCrop` those pixels are
cropped away. Under `FullActiveArea` they would be visible, and whether they are depends on
the warp's backward radial scale. SPEC-064 measures that and replaces the blanket refusal
with the bounds rule `docs/future-plans.md` locked.

Relaxing a refusal into a result is additive. Turning a result into a refusal is not, so
the conservative direction comes first. An identity warp, which `develop_into` already
skips, is not a warp for this rule.

### 5. The version rule

`DEVELOP_VERSION` (DEC-028) identifies develop output **for every pair of (input bytes,
options value)**, not only for the default:
- **Bump** when the output changes for any existing options value on the same input
  bytes, the default included.
- **Do not bump** when a new field is added whose default reproduces the old output. The
  same holds for a new enum variant, or for a refusal relaxed into a result: no existing
  (input, options) pair changes its output.

SPEC-063 therefore leaves `DEVELOP_VERSION` at `"4"`. A caller that caches developed
pixels keys on `(DEVELOP_VERSION, options)`. irradiance does not provide a canonical
serialisation of the options in v1; the caller owns its own key.

## Context

monorail-000 / onyx-000 request 2 (`feedback/2026-11-11-monorail-000-requests.md`) asked
for one options struct in place of separate entry points, plus `apply_orientation`.
`docs/future-plans.md` § PROJ-004 had already locked two framing modes and `WarpRect::lerp`.
STAGE-019 frames five specs that all hang options off one struct, so its shape is decided
once, here.

The SPEC-063 design probe (main at 3458854) found:
- **The requester's camera is always warped.** All three held Q2M frames carry a
  non-identity `WarpRectilinear` (kr0 0.999251106, kr1 from -0.032 to -0.061). Until
  SPEC-064 ships, `FullActiveArea` is therefore refused on every Q2M frame.
- **Warp-free files with real borders exist.**
  - K-3 III DNG: `ActiveArea` 6224×4160 at (26, 34), crop 6192×4128.
  - M Monochrom L1000622: no `ActiveArea`, plane 5216×3472, crop 5212×3468.
  - Typ 246: plane 5984×4000, crop 5976×3992.

  These give `FullActiveArea` a real-file oracle now.
- **`Orientation 6` with a warp is held** (Q2M L1026016). It gives `apply_orientation:
  false` a real-file oracle through the warp path.

## Alternatives Considered

- **Separate entry points** (`develop_full_active_area_into`, `develop_unoriented_into`, …).
  Rejected: every new option would double the surface, and the scratch variant would
  multiply it again. This is the requester's own objection.
- **Builder methods** (`DevelopOptions::default().framing(..)`). Rejected for now: `pub`
  fields plus `Default` already work across the `#[non_exhaustive]` boundary, and two ways
  to set a field is one too many. Builder methods can be added later without breaking
  anything.
- **`Eq`/`Hash` on `DevelopOptions`**, so callers can key a cache directly. Rejected:
  `warp_alpha` is an `f64`. A caller can hash the fields it reads.
- **Allow `FullActiveArea` with a warp in SPEC-063 and show the clamped edges.** Rejected:
  that bakes in the very behaviour SPEC-064 exists to decide. Taking it back later would
  turn a result into a refusal, which breaks callers.
- **Skip validation of tags an option makes unused.** Rejected (§3): the same file would
  develop under one option and refuse under another for a reason unrelated to the option.
- **Options on the colour path too.** Rejected: no named caller, and the colour path's
  scatter geometry (DEC-043) would need its own oracle.

## Consequences

- `develop_into` and `output_dimensions` keep their signatures and their output. The
  build pins this with MD5s of `develop_into`'s output on eight held files, taken from
  main before the change (SPEC-063 §1).
- One new `Error` variant, `UnsupportedWarpedFullActiveArea`, classified `Unsupported` in
  `Error::kind()` with no wildcard (DEC-051). SPEC-064 removes or narrows the conditions
  under which it fires.
- SPEC-064, 065 and 066 add a field each (`warp_alpha`, a scratch entry point, a CFA-as-mono
  flag) with a default that reproduces today, so none of them bumps `DEVELOP_VERSION`.
- `crustyimg` (DEC-028's consumer) keeps working unchanged: it calls `develop_into`.
