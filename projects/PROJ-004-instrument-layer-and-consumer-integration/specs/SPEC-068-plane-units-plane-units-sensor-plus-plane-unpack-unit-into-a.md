---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-068
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-020
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-063, DEC-056, DEC-008, DEC-016, DEC-047, DEC-050, DEC-051, DEC-052, DEC-003]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-012, SPEC-028, SPEC-039, SPEC-058, SPEC-069, SPEC-070]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-020's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-020's units: a caller decodes the plane in full-width row bands on its own threads, byte-identical in any order; the Q2M strip scales to about 6 ms at 8 threads in the probe."

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
  tokens_estimate: 32000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-068: Plane units: plane::units yields full-width row bands, and unpack_unit_into decodes one, byte-identically in any order

## Context

SPEC-068 is the first spec in STAGE-020 (PROJ-004). It covers monorail-000 / onyx-000
request 1 (`feedback/2026-11-11-monorail-000-requests.md`): `plane::unpack_into` is
single-threaded, and the requester wants independent units that they can decode on their
own threads. Their acceptance tests are:
- any-order assembly is byte-identical;
- an off-by-one-row red-proof goes red;
- a fuzz target exists;
- no new allocation.

**DEC-063** (minted with this design) fixes the unit as a **full-width band of whole
rows**. Each unit's output is then a contiguous run of the plane, so a caller splits the
buffer with `split_at_mut` and needs no `unsafe`. DEC-063 also covers the unit kinds per
layout, the API shape, and the rule that `unpack_into` iterates the same units.

## Goal

Add `plane::Unit`, `plane::Units`, `plane::units(sensor, file, max_band_rows)` and
`plane::unpack_unit_into(sensor, byte_order, file, &unit, dst)`. Units decoded in any
order, each into its own band of one plane, must equal `unpack_into` byte for byte.
`unpack_into` must iterate the same units with the same kernel, so its output and its
errors are unchanged.

## Design

### 1. Probe results (main at 851173f, 2026-11-13; recorded so build is transcription)

**Layouts and expected units.** These come from the STAGE-020 framing probe, which ran on
every held file. The unit counts are what DEC-063 §1 implies:

| File | Layout | Rows | `units(.., 704)` | `units(.., 100)` | `units(.., MAX)` |
|---|---|---|---|---|---|
| Q2M ×3 | 1 uncompressed strip, 14-bit, whole-byte rows | 5632 | **8** | 57 | 1 |
| Nikon D300S | 12 uncompressed strips, rps 256, 16-bit | 2868 | 12 | **34** (11×3 + 1) | 12 |
| Canon 5D3 | 1 uncompressed strip, 16-bit | 3950 | 6 | 40 | 1 |
| M Monochrom | 1 uncompressed strip, 16-bit | 3472 | 5 | 35 | 1 |
| Pixel 3a / 4a | 192 Compression 7 tiles of 256×256 | 3024 | **12** (tile rows, the last 208 rows) | 12 | 12 |
| Pixel 7 Pro | 192 Compression 7 tiles | 2924 | 12 (the last 108 rows) | 12 | 12 |
| K-3 III Mono | 1 Compression 7 strip, no DRI/RST | 4224 | **1** | 1 | 1 |
| M Typ 246 | 1 Compression 7 strip, no DRI/RST | 4000 | 1 | 1 | 1 |

The build verifies the counts and corrects any it finds wrong in the handback. They are
arithmetic, not measurements.

**Parallel scaling.** An independent 14-bit unpacker in the probe, over Q2M L1021223 row
bands, one scoped thread each, median of 5:

| Threads | Time | Notes |
|---|---|---|
| 1 | 44.7 ms | |
| 2 | 27.9 ms | |
| 4 | 11.3 ms | |
| 8 | 6.1 ms | |
| 12 | 5.7 ms | |

Each run asserted byte-identity to `unpack_into`. The crate's own single-threaded unpack
takes 238 ms on this strip. That 5.3× gap is SPEC-069's evidence, not this spec's.

### 2. Public API (`src/plane.rs`), exactly DEC-063 §2

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unit {
    pub row_start: u32,
    pub row_count: u32,
    source: UnitSource,          // private
}
// private:
// enum UnitSource { StripBand { strip: usize, row_in_strip: u32 }, Strip { strip: usize }, TileRow { tile_row: u32 } }

impl Unit {
    /// `row_count · width`, the band's sample count, or `None` on overflow.
    pub fn samples(&self, width: u32) -> Option<usize>;
}

pub struct Units<'a> { /* sensor, layout, cursor — no Vec */ }
impl Iterator for Units<'_> { type Item = Unit; }
impl ExactSizeIterator for Units<'_> {}

pub fn units<'a>(sensor: &'a Sensor, file: &[u8], max_band_rows: NonZeroU32)
    -> Result<Units<'a>, Error>;

pub fn unpack_unit_into(sensor: &Sensor, byte_order: ByteOrder, file: &[u8],
    unit: &Unit, dst: &mut [u16]) -> Result<(), Error>;
```

`Error::PlaneUnitMismatch { row_start: u32, row_count: u32 }` is a new variant,
`ErrorKind::CallerError`, and its `Display` names both numbers.

**Units' own allocation.** `Units` holds no `Vec`. It walks the strip or tile tables
lazily, as `StripWalk` and `TileWalk` already do. `len()` is computed arithmetically at
construction.

### 3. Validation and precedence

**`units`** follows today's order:
1. `plane_kind` (compression, bit depth);
2. the `u64` pixel count fits `usize` (`PlaneTooLarge`-free: no budget here, the caller
   used `validate`);
3. `check_layout(sensor, kind, plane_samples, file)`.

Any error is exactly the variant `unpack_into` returns for the same sensor. There is no
`dst` step, because `units` has no `dst`.

**`unpack_unit_into`:**
1. Run the same steps as `units`.
2. Re-derive the unit against the layout. Its source must exist, and its `row_start` and
   `row_count` must equal what that source implies at **some** valid band size (see
   below). Otherwise the result is `PlaneUnitMismatch`.
3. Check that `dst.len() == row_count · width`, or return `PlaneBufferWrongLength`.
4. Decode.

The order is chosen so a unit from another sensor is refused before its `dst` length is
judged.

What "at some valid band size" means for each source:
- **A `StripBand`** must lie inside its strip, with `row_count ≥ 1`. Any band inside a
  strip is decodable, because rows are whole bytes.
- **A `Strip`** must be the strip's exact row range.
- **A `TileRow`** must be the tile row's exact visible row range.

### 4. The per-unit kernels (no new decoding)

- **`StripBand`, uncompressed.** The source bytes are
  `strip[row_in_strip · row_bytes .. (row_in_strip + row_count) · row_bytes]`, where
  `row_bytes = width · bits / 8`, exact because rows are whole bytes. Decode them with
  today's `unpack_byte_aligned` or `unpack_bitstream` into `dst`.
  - **Sub-byte strips.** The last row of a strip carries its padding only at the strip's
    end. A band that ends at the strip's last row takes `strip[..]` up to `span.end`. The
    unpacker reads only `row_count · width` samples, so trailing pad bits are ignored as
    today.
- **`Strip`.** Today's per-strip body: uncompressed sub-byte or byte-aligned, or
  Compression 7 (`parse_headers`, `check_strip_shape`, `decode_into`).
- **`TileRow`.** Today's per-tile body for each tile in that tile row, with the window's
  `y0` made band-relative (`y0 − row_start`) and `dst` the band:
  - uncompressed: per tile row, write `visible_width` samples at band row `r`, column
    `x0`;
  - Compression 7: `decode_window_into(tile, band_dst, Window { stride: width, x0,
    y0: 0, visible_width, visible_height })`.

**Byte identity.** Every sample is produced by the same function from the same bytes as
today. That makes identity hold by construction. AC1 and AC2 are the proof.

### 5. `unpack_into` (DEC-063 §3)

After today's `plane_kind` → `dst` length → `check_layout` sequence, loop
`for unit in units(sensor, file, NonZeroU32::MAX)?`. For each unit, carve its band off the
remaining `dst` (`split_at_mut_checked`) and call the **private** kernel
`decode_unit(sensor, layout, kind, byte_order, file, &unit, band)`. That is the same
kernel `unpack_unit_into` reaches after its own checks. Do not re-run `check_layout` per
unit inside `unpack_into`; it already ran.

The private `unpack_uncompressed`, `unpack_jpeg`, `unpack_tiles_uncompressed` and
`unpack_tiles_jpeg` collapse into that kernel, or are called by it. Today's traversal
order is preserved: strips in order, tiles in table order, which is tile row by tile row.

### 6. Fuzz (`fuzz/fuzz_targets/plane.rs`)

After the existing `unpack_into` call, take `max_band_rows = 1 + (last byte % 64)`.
- **If `unpack_into` returned `Ok`:**
  - `units` is `Ok`;
  - the units' row ranges tile `0..height` exactly;
  - decoding every unit **in reverse order** into a second plane gives every unit `Ok`;
  - that plane equals the first.
- **If `unpack_into` returned `Err`:** one of two cases must hold:
  - `units` returns that error;
  - `units` is `Ok`, and decoding the units in forward order returns that error at the
    first failing unit.

  Use `PartialEq` on `Error` if it exists; otherwise compare variants with `Debug`.

The plane buffers are allocated once per input, before any comparison. That is fine in a
fuzz target. Add **three** seeds via `examples/fuzz-seeds.rs`:
- a multi-strip sub-byte uncompressed plane whose last strip is short;
- an uncompressed tiled plane with a partial last tile row;
- an uncompressed strip whose rows are **not** whole bytes (12-bit, odd width), so the
  whole-strip fallback is exercised.

Report the run count for at least 60 s.

### 7. Docs

- **CHANGELOG `[Unreleased]`:**
  - Added: `Unit`, `Units`, `units`, `unpack_unit_into` and `PlaneUnitMismatch`, with
    the unit kinds, and noting that Compression 7 single-scan strips are one unit.
  - Changed: none (`unpack_into` output unchanged).
- **The `src/plane.rs` module doc** gains a "Units — SPEC-068, DEC-063" section, with a
  short `std::thread::scope` example in a doc test. That is std only, no new dependency,
  and it splits `dst` with `chunks_mut`, or a manual carve because units can be uneven.
- **Provenance:** none needed (no new algorithm). Say so in the handback.

## Inputs

- **Files to read:**
  - `src/plane.rs`:
    - `validate` ~314, `unpack_into` ~426;
    - `plane_kind`, `StripWalk`, `check_strips`;
    - `Layout`, `TileGrid`, `tile_grid`, `TileWalk`, `check_layout`;
    - `carve`, `unpack_uncompressed`, `unpack_jpeg`, `unpack_tiles_uncompressed`,
      `unpack_tiles_jpeg`;
    - `unpack_bitstream`, `unpack_byte_aligned`.
  - `src/ljpeg.rs`: `Window`, `decode_window_into`, `decode_into`, `parse_headers`.
  - `src/lib.rs`: `Error`, `kind()`.
- **Tests to mirror:**
  - `tests/plane_unpack.rs`, `tests/plane_tiles.rs`, `tests/plane_validate.rs`,
    `tests/plane_oracle.rs`;
  - `tests/previews_alloc.rs` / `tests/develop_scratch_alloc.rs` for the counting
    allocator;
  - `tests/support/tiff.rs` and `tests/support/dng_writer.rs` for tier-A fixtures,
    including tiled and Compression 7 ones;
  - `tests/error_kind.rs`.
- **Decisions:**
  - DEC-063 (this spec);
  - DEC-056 (the tile window);
  - DEC-008 (the two unpack paths);
  - DEC-016 (caller-owned buffers);
  - DEC-047 (frame shape binding);
  - DEC-050 (`validate` shares the checks);
  - DEC-051, DEC-052, DEC-003.

## Outputs

- **Files modified:**
  - `src/plane.rs`: the types, the two functions, the kernel, and the `unpack_into`
    loop;
  - `src/lib.rs`: the variant and `kind()`;
  - `fuzz/fuzz_targets/plane.rs` and `examples/fuzz-seeds.rs`, plus three seeds;
  - `CHANGELOG.md`;
  - `tests/error_kind.rs`.
- **Files created:**
  - `tests/plane_units.rs`;
  - `tests/plane_units_alloc.rs` (its own binary, holding the counting allocator).
- **New public items:**
  - `plane::Unit`;
  - `plane::Units`;
  - `plane::units`;
  - `plane::unpack_unit_into`;
  - `Error::PlaneUnitMismatch`.

## Acceptance Criteria

- [ ] **AC1. Any-order assembly on held files (tier B).**
  - For every held file and `max_band_rows` in {1, 100, 704, MAX}, decode all units into
    one plane in **reverse** order and in a **fixed pseudo-random** order (a seeded
    shuffle written in the test; no `rand` dependency).
  - Each result must equal `unpack_into`, compared by MD5.
  - Each must also equal the existing plane oracle MD5s where the repo pins them (the
    `tests/plane_oracle.rs` pins and SPEC-025's / SPEC-058's dnglab checksums).
  - On the Q2M at 704, also decode the 8 units on **8 `std::thread::scope` threads** into
    bands from `chunks_mut`, and get the same MD5.
  - A missing file is reported by name.
- [ ] **AC2. Any-order assembly on tier-A layouts.** Fixtures built in-test with the
  existing writers. For each layout, with `max_band_rows` in {1, 2, 3, 7, MAX}, check
  every permutation for ≤ 6 units and reverse plus shuffle beyond that:
  - uncompressed 8-, 12-, 14- and 16-bit single strips;
  - multi-strip with a short last strip, at 12-bit and 16-bit, and in both byte orders;
  - a 12-bit odd-width strip whose rows are not whole bytes (whole-strip fallback);
  - uncompressed tiles with a partial last tile row and column;
  - Compression 7 single strip and multi-strip;
  - Compression 7 tiles with partial edges.
- [ ] **AC3. Unit counts and coverage.**
  - The §1 counts hold on the held files, tier B.
  - On every AC2 fixture, the units' `row_start` / `row_count` cover `0..height` exactly
    once, in order, with no gaps.
  - `ExactSizeIterator::len()` equals the count yielded.
  - Bands never cross a strip. A Nikon-shaped tier-A fixture (rps 256, height 2868) at
    100 gives 34 units, with every unit inside one strip.
- [ ] **AC4. `unpack_into` is unchanged.**
  - Every existing plane test passes **unedited**: `plane_unpack`, `plane_tiles`,
    `plane_validate`, `plane_oracle`, the SPEC-025/044/058 dnglab checks, and the
    develop MD5 pins downstream.
  - The error precedence tests are unedited and green.
- [ ] **AC5. Refusals.**
  - `units` on each malformed layout fixture in `tests/plane_unpack.rs`,
    `tests/plane_tiles.rs` and `tests/plane_validate.rs` returns the same error
    (Debug-equal) as `unpack_into`. Use one table-driven test.
  - `unpack_unit_into` with a unit from a **different** sensor (a 704-band from the Q2M
    applied to the Canon fixture, or tier A equivalents) gives `PlaneUnitMismatch` with
    the unit's rows, `kind() == CallerError`, and `dst` unchanged (sentinel).
  - A right unit with a `dst` of `n − 1` and `n + 1` gives `PlaneBufferWrongLength` with
    `expected = n`, and `dst` unchanged.
  - A Compression 7 decode error in unit k is reported by `unpack_unit_into(k)` and by
    `unpack_into`. The other units decode `Ok`.
- [ ] **AC6. No allocation.** In `tests/plane_units_alloc.rs`, with the `previews_alloc`
  counting-allocator pattern:
  - `units(..)` plus full iteration, and `unpack_unit_into` for every unit, make 0
    allocations. Cover uncompressed strip, multi-strip, tiles and Compression 7 tier-A
    fixtures, plus the error paths (mismatch, wrong length).
  - Q2M tier B: 0 allocations over all 8 units.
  - Include the negative control.
- [ ] **AC7. Error surface.**
  - `PlaneUnitMismatch` is in `Error::kind()` with no wildcard, and has a row in
    `tests/error_kind.rs`.
  - Its `Display` names both numbers, pinned by a test.
  - The counts move by exactly one.
- [ ] **AC8. API hygiene.**
  - `Unit` is `#[non_exhaustive]`, and a test outside the crate cannot build one; a
    `compile_fail` doc test is fine.
  - The doc test with `std::thread::scope` runs.
  - `cargo doc` has no broken links.
  - CHANGELOG is updated (§7).
- [ ] **AC9. Fuzz.** The §6 differential and the three seeds are in place, and a 60 s
  `plane` run is clean. Report the run count and which seeds reach the band, strip and
  tile-row kinds.
- [ ] **AC10. No regression.**
  - `just test` passes with the corpus env set. So do `lint-ci`, `lint-no-allow`,
    `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny` and `msrv`.
  - No new dependency.
  - `DEVELOP_VERSION` is unchanged.

## Failing Tests

- **`tests/plane_units.rs`:**
  - `units_assemble_in_any_order_on_held_files`: AC1, tier B, including the
    8-thread case.
  - `units_assemble_in_any_order_on_tier_a_layouts`: AC2.
  - `unit_counts_and_coverage`: AC3, tier A and tier B.
  - `units_refuse_what_unpack_into_refuses`: AC5, table-driven.
  - `foreign_unit_is_a_typed_mismatch_before_any_write`: AC5.
  - `wrong_length_band_is_refused_before_any_write`: AC5.
  - `a_failing_unit_fails_alone`: AC5.
  - `unit_mismatch_display_names_both_rows`: AC7.
- **`tests/plane_units_alloc.rs`:**
  - `counter_counts`;
  - `units_and_unit_decode_allocate_nothing`;
  - `q2m_units_allocate_nothing` (tier B).
- **`tests/error_kind.rs`:** the new row.

**Red-proof expectations.** The verify mutates each of these, and each must turn a named
**tier-A** test red:
- band byte offset off by one row (`(row_in_strip + 1) · row_bytes`) → AC2;
- band `row_count` one short or long at the last band of a strip → AC2, AC3;
- a band allowed to cross a strip boundary → AC3 (every unit inside one strip), and AC2
  at 12-bit;
- the whole-strip fallback skipped for rows that are not whole bytes → AC2's odd-width
  fixture;
- a tile row's window `y0` left plane-relative (not band-relative) → AC2's tiled
  fixtures;
- `unpack_unit_into` skips the unit re-derivation → AC5 mismatch;
- the `dst` length check placed before the mismatch check → AC5's ordering (the foreign
  unit with a wrong `dst` must report a mismatch);
- `unpack_into` stops iterating units (left on the old loops while the unit kernel
  diverges) → this cannot be shown by output while both are correct. The verify confirms
  by reading that the old `unpack_*` loops are gone or are called only from the kernel.
  It records that this guard is structural.

## Non-Goals

- **Restart-interval units** (DEC-063 §4).
- **Units narrower than the full width.**
- **Threads inside irradiance.**
- **Kernel speed-ups:** SPEC-069 (the 5.3× packed-bits gap measured above) and SPEC-070
  (Compression 7).
- **`validate` changes.**
- **The develop path.**

## Notes for the Implementer

- **Keep one decode path.** `unpack_into` must call the same private kernel as
  `unpack_unit_into`. AC4's unedited oracles are the proof that output did not move.
- **Tiled bands are the trickiest window change.** Write AC2's partial-tile-row fixture
  first and run it before refactoring `unpack_tiles_jpeg`.
- **Sub-byte band slicing.** The strip's packed size is exact per strip (`check_strips`),
  so `row_count · row_bytes` bytes always exist for whole-byte rows. Only the strip's
  final band may also carry the strip's trailing padding, and the unpacker never reads
  it.
- **Four tests scan `src/plane.rs` source text.** The design checked them. They must stay
  **unedited and green**, which AC4 requires. They are:
  - `tests/plane_oracle.rs::inject_chunk_extraction_fault`. It needs exactly one
    `.checked_div(pow2(bits_left.saturating_sub(take)))`, in `BitReader::read`.
  - `tests/plane_oracle.rs::inject_byte_order_swap_fault`. It needs exactly one
    `u16::from_le_bytes(*chunk)` and one `u16::from_be_bytes(*chunk)`.
  - `tests/plane_validate.rs`'s guard injector. It needs exactly one
    `if pixels > budget {`, plus its sibling needles.
  - `tests/plane_tiles.rs::the_tile_path_allocates_nothing`. It rejects `vec!`, `Vec::`,
    `Box::`, `.to_vec()`, `.collect`, `String::` and `format!` anywhere in `plane.rs`
    outside `mod tests`, with `//` comments stripped.

  So **reuse** the kernels; never copy `BitReader` or `unpack_byte_aligned`. Put the
  `std::thread::scope` example only in doc comments, and keep it free of `vec!` outside
  comment lines. Doc comments start with `//`, so they are stripped. Grep `tests/` for
  `read_to_string` before restructuring, in case one was missed.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
