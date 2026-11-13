---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-060
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
  - src/opcode.rs
  - src/lib.rs
  - fuzz/fuzz_targets/develop.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-065-caller-owned-develop-scratch-scratch-len-plus-a-scratch-vari.md

tags:
  - develop
  - scratch
  - allocation
  - public-api
  - stage-019
---

# DEC-060: develop scratch is one exact-length caller slice with a private layout, and the scratch path allocates nothing

## Decision

### 1. Shape

```rust
pub fn scratch_len(sensor: &Sensor, options: &DevelopOptions) -> Result<usize, Error>;

pub fn develop_with_scratch_into(
    sensor: &Sensor,
    src: &[u16],
    dst: &mut [u16],
    scratch: &mut [u16],
    options: &DevelopOptions,
) -> Result<(), Error>;
```

- **One `&mut [u16]`, not a struct of buffers.** The caller sizes it from `scratch_len`
  and never sees how it is divided. The layout (§3) is private, so it can change without
  breaking anyone.
- **Exact length**, like every other buffer in this crate (`dst`, `src`, the warp
  buffers). A wrong length is `Error::DevelopScratchWrongLength { expected, actual }`
  (`ErrorKind::CallerError`). A caller reusing one large buffer across frames passes
  `&mut buf[..len]`. "At least `len`" was rejected because it would make this the one
  buffer in the crate with a different rule.
- **Options last**, after the three buffers (DEC-058 §1).
- `scratch_len` returns `0` when the develop needs no scratch. The caller may then pass an
  empty slice.

### 2. The contract

- **`develop_with_scratch_into` performs no heap allocation**, on every path that
  returns `Ok` and on every error path. This is a documented promise, pinned by a
  counting-allocator test. The opcode lookups it calls
  (`parse_fix_bad_pixels_constant`, `parse_warp_rectilinear`) become allocation-free
  first-instance walkers. They return the same `Ok` values and the same errors as
  before. `apply_fix_bad_pixels_constant` keeps its neighbours in an `[u16; 8]` array.
- **Scratch contents are unspecified on entry and on return.** The develop writes every
  scratch sample before it reads it, so the output never depends on what the caller left
  there. The caller must not read meaning into scratch afterwards.
- **`scratch_len` depends only on `(sensor, options)`.** A new irradiance version may
  change it. That is not an output change and does not bump `DEVELOP_VERSION`. Callers
  must ask for it, not hard-code it.
- **Output is identical** to `develop_with_into(sensor, src, dst, options)` for every
  input and every options value. Every error the latter returns, the former returns too,
  in the same order. The scratch-length check is added after the opcode lists are parsed
  and before any write to `dst` or `scratch`.

### 3. Layout (private; recorded so the length is explained)

Let `P` be the raw plane size (`width × height`) and `A` the active-area size. The
develop needs:
- a copy of the plane when `OpcodeList1` carries `FixBadPixelsConstant` (`fbp`);
- a normalized active-area buffer plus a warped one when `OpcodeList3` carries a
  `WarpRectilinear` that is non-identity after `lerp(warp_alpha)` (`warp`).

Once the levels pass has read the fixed plane into the normalized buffer, the plane copy
is dead. The warp output can therefore reuse it (`A ≤ P` always, by `resolve_geometry`):

| `fbp` | `warp` | `scratch_len` | plane copy | normalized | warped |
|---|---|---|---|---|---|
| no | no | 0 | | | |
| yes | no | P | `[0, P)` | | |
| no | yes | 2A | | `[0, A)` | `[A, 2A)` |
| yes | yes | P + A | `[0, P)` | `[P, P + A)` | `[0, A)` |

Lengths are computed with checked arithmetic. An overflow is reported as
`DevelopScratchWrongLength { expected: u64::MAX, actual }`, with `actual = 0` from
`scratch_len`. That follows the crate's `u64::MAX` convention for unrepresentable
lengths.

### 4. `develop_with_into` uses the same core

`develop_with_into` (and therefore `develop_into`) runs the same core. It allocates one
`Vec<u16>` of exactly `scratch_len` samples, at the point where today's code first
allocates, and allocates nothing when the length is 0. Output and error order are
unchanged. The allocation count on a Q2M frame drops from six to one, and the peak
develop memory from `P + 2A` to `P + A` samples.

## Context

monorail-000 / onyx-000 request 3 asked for `scratch_len(sensor, &options)` plus a variant
taking `scratch: &mut [u16]`, a test proving no allocation, and a typed error for a wrong
length. Their target files carry both `FixBadPixelsConstant` and `WarpRectilinear`, so
they budget for the full peak today.

The SPEC-065 design probe (main at 12b0794) wrapped `develop_into` in a counting
allocator:

| File | `fbp` | warp | allocations | bytes | `scratch_len` (§3) |
|---|---|---|---|---|---|
| Q2M L1021223 / L1026016 / L1026192 | yes (constant 0) | yes | **6** | **283,943,504** | 94,707,712 samples (189.4 MB) |
| M Monochrom, Typ 246, K-3 III, Canon, Nikon | no | no | 0 | 0 | 0 |

On a Q2M frame the six allocations are:
- the 94.9 MB plane copy;
- two 94.5 MB active-area buffers;
- three small ones: two `parse_opcode_list` `Vec`s and the neighbours `Vec`.

The small ones were not in the stage's design notes. They are why the opcode lookups must
become allocation-free walkers, and not only the three large buffers removed.

## Alternatives Considered

- **A typed scratch struct**, for example `DevelopScratch { plane, active_a, active_b }`.
  Rejected. It freezes the layout into the API, and the reuse in §3 would have been a
  breaking change.
- **Fix bad pixels in place on a caller-mutable `src`.** Rejected. It changes
  `develop_with_into`'s `&[u16]` contract and destroys the caller's raw plane. The
  requester keeps the raw plane for their own instruments (onyx DEC-010).
- **"At least `scratch_len`"** in place of exact length. Rejected (§1).
- **Keep `develop_with_into` on its own allocations** and make only the new variant
  allocation-free. Rejected. Two code paths would drift, and the shared core makes the
  scratch variant's byte identity hold by construction.
- **Warp straight into `dst`** when `FullActiveArea` with Orientation 1 makes `dst`
  active-sized, saving a further `A`. Deferred. It adds a branch for one options value
  and has no named caller. It is an additive optimisation later, because it only shrinks
  `scratch_len`.

## Consequences

- New public items:
  - `develop::scratch_len`;
  - `develop::develop_with_scratch_into`;
  - `Error::DevelopScratchWrongLength`.
- `parse_warp_rectilinear` and `parse_fix_bad_pixels_constant` keep their signatures and
  results and stop allocating. `parse_opcode_list` is unchanged.
- `DEVELOP_VERSION` stays `"4"`: no output changes.
- Every develop with `FixBadPixelsConstant` and a warp now uses one `Vec` of `P + A`
  samples where it used three buffers totalling `P + 2A`. On the Q2M that saves
  94,527,488 bytes and five allocations per frame, even for callers that never use
  scratch.
