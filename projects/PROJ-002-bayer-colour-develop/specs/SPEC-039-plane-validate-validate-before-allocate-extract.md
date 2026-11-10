---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-039
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship — re-designed as group E after A–C (DEC-045/DEC-050)
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: null
  verify_verdict: null

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-4-8  # orchestrator (tier_map.design, DEC-005)
  to_agent: claude-opus-5          # tier_map.build — the executing agent
  created_at: 2026-11-10

references:
  decisions: [DEC-016, DEC-045, DEC-050]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-025, SPEC-027, SPEC-052, SPEC-053, SPEC-054]

# Blocking dependencies: specs that must SHIP before this one can start.
depends_on: [SPEC-052, SPEC-053, SPEC-054]   # group E unifies the preflight across A–C (DEC-045); all shipped 2026-11-10

claimed_by: null

value_link: "the allocation-boundary half of STAGE-010's 'safe to depend on from a host that cannot catch an OOM abort' thesis — a caller sizes its buffer from a proven, budget-bounded number before allocating"

cost:
  tokens_estimate: 18000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-039: `plane::validate` — the unified validate-before-allocate preflight (group E)

> **Re-designed 2026-11-10 (DEC-050), `cycle: build`.** Group E of the pre-0.2.0
> wave, after A (SPEC-052), B (SPEC-053) and C (SPEC-054) shipped. The old
> uncompressed-only design (git history) is superseded; `HANDOFF-084` is
> superseded by `HANDOFF-117`. **Read DEC-050 in full first** — it is the contract
> this builds.

## Context

STAGE-010's allocation-boundary spec; source `feedback/2026-11-02-pyirradiance-requests.md`
item 1. `plane::unpack_into` (`src/plane.rs:278`) checks layout, size and truncation
only **after** the caller has allocated `dst`, and the only check that involves `dst`
is its length. A forged `ImageWidth`/`ImageLength` makes the caller request gigabytes
and the OOM aborts the process — uncatchable from a Python worker. The pre-0.2.0 review
(finding #5) showed the natural home for this is a single preflight the decoders trust,
unified with A (the IFD-parse guard, upstream of `Sensor`) and B (the Compression-7
decoded-frame shape binding, at decode time). It also carries the README/CHANGELOG/
conformance-matrix reconcile, the `ifd`/`plane`/`ljpeg` fuzz-smoke-matrix addition, and
the EOI-optional decision.

## Goal

Add `pub fn plane::validate(sensor, byte_order, file, max_pixels) -> Result<usize, Error>`:
allocation-free, panic-free, reads no strip bytes, covers **both** the uncompressed and
the Compression-7 path, enforces the caller's `max_pixels` budget, and returns `Ok(n)`
(`n = width·height`) such that `unpack_into` with a `dst` of length `n` cannot fail on
layout, size, truncation, precision or compression-support. `unpack_into` calls the same
shared checks so the two cannot drift.

## Design

Build exactly DEC-050. Summary of the load-bearing parts:

### 1. Signature and shape

`pub fn validate(sensor: &Sensor, byte_order: ByteOrder, file: &[u8], max_pixels: usize) -> Result<usize, Error>`.
Runs every pre-`dst` check `unpack_into` runs, for whichever `sensor.compression` is
declared; returns `Ok(width·height)`. Reads no strip bytes, writes nothing, allocates
nothing (no `Vec`). `byte_order` is kept for parity with `unpack_into` and does not
affect validation (rustdoc says so).

### 2. The caller budget is checked first (both paths)

Compute `pixels = width as u64 * height as u64` (checked), and **before** any heavy work
or any `usize` cast, return `Error::PlaneTooLarge { pixels, limit: max_pixels }` when
`pixels > max_pixels as u64`. This is the only pre-allocation bound available for
Compression-7 (a small compressed strip decodes to a large plane). `max_pixels` is the
caller's number; `usize::MAX` means "no limit".

### 3. Uncompressed path: carry the file-bytes bound

After the budget check, run `plane_len` (compression recognised, bit depth in
`SUPPORTED_BITS`, the `width·height` product) and a full pass of `strip_spans`
(strip-count agreement, `RowsPerStrip`, per-strip `PackedSizeMismatch`, in-file bounds,
full row coverage) **plus** the sum-of-`StripByteCounts` ≤ `file.len()` rule → new
`Error::StripBytesExceedFile { declared: u64, file_len: usize }`. So for an uncompressed
plane `Ok(n)` ⇒ `n·bits ≤ 8·file.len()`. This is the old SPEC-039 §3, unchanged; the
tier-B corpus must not trip it.

### 4. Compression-7 path: pre-decode structural checks only

Run the structural checks `unpack_jpeg` runs before it decodes — strip layout,
`RowsPerStrip`, each strip's in-file offset/count bounds, supported precision — then
return `n = width·height` (already budget-checked in §2). Do **not** decode: the SOF-3
decoded-frame shape binding (`plane::check_strip_shape`, SPEC-053) and the sample decode
stay runtime checks inside `unpack_into`. The rustdoc states that for a compressed plane
`n` is bounded only by `max_pixels`, not by the file length.

### 5. Sharing the code without changing error precedence

Factor the checks into private helpers so there is one implementation of each. Today the
order is compression → bit depth → pixel product → **`dst.len()`** → strip layout. A naive
`unpack_into = { validate()?; dst check; unpack }` would turn "wrong `dst` AND bad layout"
from `PlaneBufferWrongLength` into a layout error. So: `validate` = budget check →
`plane_len` → full `strip_spans`/compressed-structural pass; `unpack_into` = `plane_len`
→ the `dst.len()` check → `strip_spans` while writing (uncompressed) or the structural
pass then decode (compressed). Precedence preserved; no second copy of any check; no
allocation. The budget check is `validate`-only (`unpack_into` already has `dst` sized by
the caller, so it needs no budget). If `width·height` overflows `usize` on a 32-bit
target it is caught by §2's `PlaneTooLarge` (the `u64` product), so no new overflow path.

### 6. EOI optional (ratify, no code change)

A SOF-3 scan is complete at `width·height` decoded samples; a missing trailing `EOI` is
not an error (current behaviour — truncation is caught as `Error::Truncated` via the
bounded bit reader). Document it on the decode entry and pin it with a test.

### 7. demosaic_into / to_linear_f32_into need no separate validate

Their buffers are a fixed `3·n` (`3 × 2 B` u16, `3 × 4 B` f32). Stated in the `validate`
rustdoc with the multipliers.

### 8. Fuzz smoke matrix + plane target

Add 60 s CI smoke jobs for `ifd`, `plane` and `ljpeg` (targets + seed corpora exist;
mirror the `fuzz-opcode` job). Extend `fuzz/fuzz_targets/plane.rs` to:
`parse → sensor() → validate(.., MAX_PIXELS_HARNESS) →` if `Ok(n)` allocate `n` (capped by
the harness's own `MAX_PIXELS`) and assert `unpack_into` is `Ok`; if `Err`, assert
`unpack_into` with a tiny `dst` is also `Err`; and whenever `Ok(n)` on an **uncompressed**
input assert `n·bits ≤ 8·len`. Add forged-dim and aliased-strip seeds.

### 9. Docs reconcile (review #5)

Update `README.md` (status block + the PROJ-003 scope line — Compression-7 is wired via
SPEC-043/046/044), `CHANGELOG.md` (`[Unreleased]`: Added `validate`, the two `Error`
variants; the `StripBytesExceedFile` tightening note), and `docs/conformance-matrix.md`
(the Leica M Typ 246 and Pentax K-3 III rows now decode bit-exact). Add a
**tested-DNG-modes compatibility table** (uncompressed mono/Bayer; Compression-7 mono
single-strip + two-component; unsupported: tiled, Bayer-in-LJ92, other compressions).

## Inputs

- **DEC-050** — the full contract (read first).
- **`src/plane.rs`** — `unpack_into` (`:278`), `unpack_uncompressed`, `unpack_jpeg`
  (`:526`), `check_strip_shape` (`:475`, SPEC-053), the compression dispatch (`:290`),
  `SUPPORTED_BITS`.
- **`src/ifd.rs`** — `Sensor` (`strip_offsets`, `strip_byte_counts`, `rows_per_strip`,
  `bits_per_sample`, `compression`, `packed_bits()`), `MAX_TAG_VALUES`.
- **`src/lib.rs`** — `Error` (`#[non_exhaustive]`, `:143`); its `Display`.
- **`src/ljpeg.rs`** — `parse_headers` (returns at `SOS`), the scan decode (completes on
  sample count), `MARKER_EOI` — for the EOI test/doc.
- **`fuzz/fuzz_targets/plane.rs`**, `.github/workflows/ci.yml` (the `fuzz-opcode` job to
  mirror), `examples/fuzz-seeds.rs`.
- **Docs:** `README.md` (status `:42-45`, scope `:157`), `docs/conformance-matrix.md`
  (Typ 246 / Pentax K-3 III rows), `CHANGELOG.md`.
- **`feedback/2026-11-02-pyirradiance-requests.md`** item 1.

## Outputs

- **Modified:** `src/plane.rs` (new `pub fn validate`, `unpack_into` refactored onto
  shared helpers); `src/lib.rs` (two new `Error` variants + `Display`);
  `fuzz/fuzz_targets/plane.rs`; `.github/workflows/ci.yml` (three smoke jobs);
  `README.md`; `CHANGELOG.md`; `docs/conformance-matrix.md`; `docs/provenance-ledger.md`
  (one row — own prior code, class 3, an `unpack_into` refactor + the budget guard).
- **Created:** `tests/plane_validate.rs`.
- **New public API:** `plane::validate(sensor, byte_order, file, max_pixels)`;
  `Error::StripBytesExceedFile { declared: u64, file_len: usize }`;
  `Error::PlaneTooLarge { pixels: u64, limit: usize }`.
- **No other signature change;** `unpack_into`'s signature, documented errors and error
  precedence are unchanged for every input that is not a newly-rejected forged overlap
  or over-budget plane.

## Acceptance Criteria

- [ ] `validate` allocates nothing and cannot panic (five deny lints + `lint-no-allow`
      clean; the fuzz target runs it on every input).
- [ ] **Budget first:** a forged `ImageWidth`/`ImageLength` (65535×65535 and
      `u32::MAX`×`u32::MAX`) over a tiny file, **both uncompressed and Compression-7**,
      returns `Error::PlaneTooLarge` with no allocation proportional to the claim, and
      the check fires before any per-strip work.
- [ ] **Contract:** for every fixture, `validate(.., n_big) == Ok(n)` ⇒ `unpack_into(.., dst)`
      with `dst.len() == n` returns `Ok(())`; and `validate(..) == Err(e)` ⇒ `unpack_into`
      with a correctly sized `dst` returns `Err` of the **same variant** (for the checks
      `unpack_into` shares; `PlaneTooLarge` is `validate`-only).
- [ ] **No drift:** `unpack_into` runs the same validation code as `validate` (shared
      helpers, not two copies); a test fails if the two disagree on a mutated layout.
- [ ] **Uncompressed bound:** whenever `validate` returns `Ok(n)` on an uncompressed
      plane, `n·bits_per_sample ≤ 8·file.len()`, including aliased strips (the
      `StripBytesExceedFile` rule); a many-strip file aliasing the same bytes is rejected.
- [ ] **Compression-7:** a valid single-strip and a valid two-component Compression-7
      plane `validate` `Ok` and then `unpack_into` `Ok`; a Compression-7 plane whose
      declared dims exceed `max_pixels` is `PlaneTooLarge`; the decoded-frame shape
      binding (SPEC-053) still fires at decode for a wrong-shape frame.
- [ ] **Precedence:** wrong-length `dst` + bad layout still yields `PlaneBufferWrongLength`.
- [ ] **EOI optional:** a valid SOF-3 scan with the trailing `EOI` removed still decodes
      `Ok`; a scan truncated before `width·height` samples is `Truncated`.
- [ ] **No regression:** every existing test passes unmodified; the tier-B corpus
      (`IRRADIANCE_CORPUS_DIR`) develops/decodes bit-exact and trips neither new error
      (recorded in the handoff).
- [ ] **Red-proof** (`oracle-must-be-shown-red`, control run first per DEC-009): the new
      tests go red against a mutated `validate` with (a) the full-coverage check removed,
      (b) the aliasing check removed, and (c) the budget check removed — shown red before
      green.
- [ ] **Fuzz:** `ifd`/`plane`/`ljpeg` smoke jobs run clean 60 s each in CI; `just
      fuzz-plane` exercises the `validate → allocate(capped) → unpack_into` contract.
- [ ] **Docs:** README, CHANGELOG and the conformance matrix no longer say Compression-7
      is unwired; a tested-DNG-modes compatibility table is present and matches the code.

## Failing Tests

- **`tests/plane_validate.rs`**
  - `validate_ok_n_matches_pixel_count` — single/multi-strip, 8/12/14/16-bit tier-A, `Ok(n)` then `unpack_into` `Ok`.
  - `validate_err_implies_unpack_err_same_variant` — bad bit depth, unsupported compression, strip-count mismatch, zero strips, missing `RowsPerStrip`, wrong `StripByteCounts`, strip past EOF, undershoot → same variant from both.
  - `budget_rejects_forged_dims_both_paths` — 65535² and `u32::MAX`² over a tiny file, uncompressed **and** Compression-7 → `PlaneTooLarge`, nothing allocated by the claim.
  - `aliased_strips_are_rejected` — many strips aliasing the same bytes (sum > `file.len()`) → `StripBytesExceedFile`; no `Ok(n)` violates `n·bits ≤ 8·len`.
  - `compression7_valid_plane_validates_then_unpacks` — a tier-A SOF-3 plane (single-strip and two-component) `Ok` then `unpack_into` `Ok`; a wrong-shape frame still hits `LjpegPlaneShapeMismatch` at decode.
  - `wrong_dst_and_bad_layout_keeps_old_precedence` — `PlaneBufferWrongLength` wins.
  - `validate_is_pure` — twice equal; `&Sensor`/`&[u8]` only.
  - `eoi_optional` — valid scan minus trailing `EOI` decodes `Ok`; truncated scan is `Truncated`.
  - `validate_red_proof_*` — mutated-copy apparatus (coverage / aliasing / budget removed), control run first.
- **`fuzz/fuzz_targets/plane.rs`** — extended per Design §8.

## Non-Goals

- No `Error::kind()` (SPEC-040); no release (SPEC-042).
- No validate helpers for `demosaic_into`/`to_linear_f32_into` (Design §7).
- No library-chosen cap on `n` — the caller's `max_pixels` is the only cap.
- **Group C's two deferred follow-ups are NOT in scope** (DEC-050 §Scope): the malformed
  `OpcodeList` tag read as "absent", and the stage functions skipping the
  required-processing check. If the implementer sees a cheap safe fix, raise a new spec —
  do not expand this one.
- No ML packing / normalisation / NumPy buffers (downstream).

## Notes for the Implementer

- Read DEC-050 and Design §2/§5 first — the budget-first ordering and the precedence-
  preserving refactor are the non-obvious parts.
- `just test` (it already bakes in `--all-features`; do not pass the bare flag).
- The `stage_probe_crate` red-proof apparatus is copy-pasted per test file; copy it again
  rather than refactoring it here.
- Fuzz: `just fuzz-plane`, or `cargo +nightly fuzz run plane …` (see `fuzz_targets/ifd.rs`
  header). Run the tier-B manifest locally (`IRRADIANCE_CORPUS_DIR`, see memory) to confirm
  neither new error trips on a real file, and record it in the handoff.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
4. **Where was the worst defect caught?** — `design` | `build` | `verify` | `ship` | `escaped` | `none` — <one word>
5. **What can a user do now that they couldn't before?** — <answer | none>
