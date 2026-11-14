---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-063
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

created_at: 2026-11-13
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - src/plane.rs
  - src/lib.rs
  - fuzz/fuzz_targets/plane.rs
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-068-plane-units-plane-units-sensor-plus-plane-unpack-unit-into-a.md

tags:
  - plane
  - units
  - parallelism
  - public-api
  - stage-020
---

# DEC-063: a plane unit is a full-width band of whole rows, so units split the caller's buffer safely

## Decision

### 1. The unit is a row band

Every unit `plane::units` yields covers **whole rows across the full plane width**: rows
`[row_start, row_start + row_count)`. Its output is therefore one **contiguous** run of the
plane, `dst[row_start · width .. (row_start + row_count) · width]`.

A caller decodes units in parallel with safe Rust alone. They carve the plane with
`split_at_mut` / `chunks_mut` (or rayon's `par_chunks_mut`) and pass each unit its own
slice. Through a C ABI, a caller passes `base + row_start · width` and the band length.
irradiance never sees two writers on one slice and needs no `unsafe`.

The unit kinds, by layout:

| Layout | Unit | Granularity |
|---|---|---|
| Uncompressed strip, rows of whole bytes (`width · bits % 8 == 0`) | a band of at most `max_band_rows` rows **inside one strip** | caller-chosen |
| Uncompressed strip, rows not whole bytes | the whole strip | fixed |
| Uncompressed tiles | one **tile row** (every tile in it, left to right) | fixed (`TileLength`, the last row clipped) |
| Compression 7 tiles | one tile row | fixed |
| Compression 7 strip | the whole strip | fixed |

- **A band never crosses a strip.** Each strip is an independent byte-padded bitstream
  (SPEC-028), so a band crossing a strip boundary would read across padding.
- **Tiles group by tile row.** Two tiles in one tile row share plane rows and are
  column-disjoint. That cannot be expressed as two disjoint `&mut [u16]`, so the tile row
  is the smallest safe unit. On the held Pixel files (192 tiles of 256 × 256), that is
  12 units.
- **A sub-byte uncompressed band starts at a computed byte offset**,
  `rows_before · width · bits / 8`. The framing probe found that every held file has
  whole-byte rows. A width that does not end on a whole byte falls back to the whole
  strip, rather than adding a bit-offset reader.
- **A restart-free Compression 7 strip is one unit.** The scan cannot be entered
  anywhere but its start. Both held single-strip files (K-3 III, Typ 246) are like this.

### 2. Shape

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unit {
    pub row_start: u32,
    pub row_count: u32,
    // private: which strip or tile row, and the band's offset inside the strip
}

impl Unit { pub fn samples(&self, width: u32) -> Option<usize>; }

pub fn units<'a>(
    sensor: &'a Sensor,
    file: &[u8],
    max_band_rows: NonZeroU32,
) -> Result<Units<'a>, Error>;               // ExactSizeIterator<Item = Unit>, allocation-free

pub fn unpack_unit_into(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    unit: &Unit,
    dst: &mut [u16],                       // exactly row_count · width samples
) -> Result<(), Error>;
```

- **`units` runs the same full layout check `validate` and `unpack_into` share**
  (`check_layout`) before yielding anything. A structurally bad plane is refused once,
  with the errors those functions already return, in their order.
- **The iterator yields units in plane order.** They are disjoint, and together cover
  every row exactly once.
- **`Unit` is opaque to construct** (`#[non_exhaustive]`), so it can only come from
  `units`. `unpack_unit_into` still re-derives the unit against the sensor's layout. A
  unit that does not match (for example, one from another sensor) is
  `Error::PlaneUnitMismatch { row_start, row_count }`, `ErrorKind::CallerError`.
- **A wrong-length `dst` is the existing `PlaneBufferWrongLength`**, with
  `expected = row_count · width`.
- **Only the bands depend on `max_band_rows`.** The other units are fixed, as the table
  shows. With `max_band_rows ≥ height`, the units are exactly one per strip or per tile
  row.
- **Restart-interval units for Compression 7 are not built** (§4). Units can gain new
  kinds later; `Unit` is `#[non_exhaustive]`, and its kind is private.

### 3. `unpack_into` iterates the same units

`unpack_into` becomes:
1. today's precedence (compression, bit depth, `dst` length, layout check);
2. a loop over `units(sensor, file, NonZeroU32::MAX)` in order, each carved off `dst`
   and decoded by the same per-unit kernel `unpack_unit_into` calls.

That traversal is exactly today's: one strip at a time, or tiles in table order, which is
tile row by tile row. Output and error order are unchanged by construction. There is one
decode path, so units and the whole-plane call cannot drift. `validate` is unchanged.

### 4. Not built

- **Restart-interval units** (Compression 7 with a `DRI`). No held file has a restart
  interval, the requester's two Compression 7 cameras have none, and finding the
  intervals means scanning the entropy-coded bytes for `RSTn` markers. That is an
  additive unit kind later, if a caller holds such a file.
- **Units narrower than the full width.** They would need overlapping `&mut` or
  `unsafe` (§1).
- **Any change to the decode kernels.** The probe's faster 14-bit unpack is SPEC-069.

## Context

monorail-000 / onyx-000 request 1 asked for `plane::units(sensor)` yielding "a rect and a
byte range per unit" and `plane::unpack_unit_into`, which writes only that unit's rect.
They asked for byte-identical any-order assembly, an off-by-one-row red-proof, a fuzz
target, and no new allocation.

The SPEC-068 design probe (main at 851173f, Apple M4 Pro, 14 cores, release build) took
an **independent** MSB-first 14-bit unpacker (a `u64` accumulator, written in the probe)
over the Q2M L1021223 strip in row bands, one `std::thread::scope` thread per band,
median of 5:

| Threads | Band rows | Time | Byte-identical to `unpack_into` |
|---|---|---|---|
| 1 | 5632 | 44.7 ms | yes |
| 2 | 2816 | 27.9 ms | yes |
| 4 | 1408 | 11.3 ms | yes |
| 8 | 704 | 6.1 ms | yes |
| 12 | 470 | 5.7 ms | yes |

- **Row bands split the requester's main file well.** Scaling is near-linear to 4
  threads and flattens near 8, the memory bound for 47 MP.
- **The crate's own single-threaded `unpack_into` takes 238 ms on the same strip**
  (STAGE-020 framing probe). The probe's plain accumulator loop takes 44.7 ms with
  identical output, a **5.3×** gap on one thread. That is SPEC-069's evidence, recorded
  there and not acted on here.
- **"A rect and a byte range" became "a row band and a private source".** A tile-row
  unit reads several disjoint byte ranges, and a sub-rectangle cannot be handed to a
  thread safely. The request's acceptance criteria are all met by row bands.

## Alternatives Considered

- **A unit that writes into the full plane `dst` at its rectangle.** Rejected. Parallel
  callers would need several `&mut` borrows of one slice, so `unsafe` would sit in every
  safe-Rust caller.
- **A unit that writes into its own `w × h` buffer, with the caller copying it in.**
  Rejected. It adds a full-plane copy (47 MP) and a per-thread buffer for no benefit,
  because row bands are already contiguous.
- **Per-tile units with a strided window.** Rejected for the same aliasing reason. A tile
  row is 16 tiles on the Pixel; 12 units are enough parallelism.
- **A bit-offset band start for rows that are not whole bytes.** Rejected for now. No
  held file has such rows, and the whole strip is correct. It can be added later.
- **`max_band_rows: u32` with 0 refused.** Rejected in favour of `NonZeroU32`, which puts
  the rule in the type and needs no error variant.
- **Unit iteration without a layout check** (lazy, per unit). Rejected. It would report a
  bad plane at different times than `unpack_into`, and the check is cheap.

## Consequences

- New public items:
  - `plane::Unit`;
  - `plane::Units`;
  - `plane::units`;
  - `plane::unpack_unit_into`;
  - `Error::PlaneUnitMismatch` (`CallerError`).
- `unpack_into`'s output, errors and their order are unchanged. Every plane oracle
  (dnglab, layer-0, tier A) must stay green unedited.
- A caller can unpack a Q2M frame on N threads with `chunks_mut`. The requester's 240 ms
  becomes about 6 ms at 8 threads in the probe's kernel; with today's crate kernel,
  roughly 238 / N.
- Compression 7 single-scan files gain nothing from units. SPEC-070 is their path.
