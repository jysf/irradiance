---
source: "pyirradiance"
captured_at: 2026-11-02
captured_by: claude
status: open
---

# pyirradiance: validate before allocating, a release with `demosaic`, error categories, cross-arch evidence

## The issue

Downstream request from **pyirradiance** (`~/PSeven/experiments/RustLibraries/pyirradiance`,
Python/NumPy bindings), pasted by the maintainer. Constraint stated up front:
**don't break any existing public API.**

1. **(Blocking) Validate before the caller allocates.** `plane::unpack_into` checks strip
   layout, `PackedSizeMismatch` and `Truncated` only after the caller has allocated `dst`. A
   forged `ImageWidth`/`ImageLength` (e.g. 65535×65535 in a 2 KB file) makes the caller request
   ~8 GB; the OOM aborts the process and Python cannot catch it. Ask: a public, allocation-free
   pre-check running all of `unpack_into`'s non-`dst` validation and returning the plane length,
   e.g. `plane::validate(sensor, byte_order, file) -> Result<usize, Error>` (or
   `Sensor::plane_len(&self, file)`; name is ours). Contract: `Ok(n)` guarantees `unpack_into`
   with a `dst` of length `n` will not fail on layout, size or truncation. Also:
   - `unpack_into` calls it internally, so the two cannot drift.
   - Tests with forged width/height (huge dims, small file) returning a typed error with no
     large allocation.
   - Extend the existing fuzz target to cover sensor → validate → allocate → unpack.
2. **(Blocking for PyPI) A crates.io release containing `demosaic`.** The published release has
   no `demosaic` module; the bilinear + Hamilton-Adams work is under `[Unreleased]`. They need a
   release shipping `demosaic::demosaic_into`, `tone::to_linear_f32_into`, `DEVELOP_VERSION`, and
   item 1 if possible. "Please just tell us which version to pin."
3. **(Nice to have) Error categories.** Python users want to separate "unsupported format, skip
   it" (`UnsupportedCompression`, `UnsupportedBitDepth`, tiled planes) from "corrupt or hostile
   file". A stable `Error::is_unsupported(&self) -> bool` or a small `ErrorKind` enum.
4. **(Nice to have) Cross-arch determinism evidence.** They will not claim bit-identical output
   across OS/arch until verified. Ask: CI on linux x86_64 + macOS arm64 asserting golden hashes of
   `unpack_into`, `demosaic_into` and `to_linear_f32_into` outputs on a corpus file (Canon 5D3
   DNG), plus a flag on any float path (f32 conversion, FMA-sensitive code) that could diverge.

**Out of scope (they handle downstream; do not add):** 4-channel RGGB packing, black/white
normalisation for ML, NumPy `out=` buffers. Colour correction is already STAGE-008.

Requested handling: log this entry first, work the items through the usual spec/decision
process, then update **Resolution** with commits/specs and the version number for item 2.

## Context

pyirradiance wraps irradiance as a PyTorch dataloader primitive. Its goal: a truncated or hostile
file raises `ValueError` and never kills a worker. The glue is `Container::parse` →
`container.sensor()` → allocate `vec![0u16; w*h]` → `plane::unpack_into` →
`demosaic::demosaic_into` (allocates 3×) → `tone::to_linear_f32_into` (allocates 3× as f32). It
depends on irradiance by path and treats it as read-only.

## Priority (their assessment)

Item 1 is the one that blocks their "never crashes" claim; once it lands their gap #1 becomes a
single call before allocating. Item 2 blocks moving from a path dependency to a versioned one,
and so blocks publishing to PyPI. Items 3 and 4: nice to have.

## Triage (claude, 2026-11-02 — read from `origin/main` `c1e1359`; nothing built)

**Item 1 — feasible, and it is a refactor of existing code.**
- `unpack_into` (`src/plane.rs:261`) is allocation-free. Its checks run in this order:
  `require_uncompressed`, supported bit depth, **`dst.len() == width*height`**
  (`plane.rs:274-290`), strip-count agreement, `RowsPerStrip`, then per-strip
  `PackedSizeMismatch` / `Truncated`. Only the `dst` length check involves `dst`; the rest can
  move into a shared `validate` unchanged, so "calls it internally, cannot drift" is a
  mechanical extraction.
- The `n ≲ file.len()·8/bits` bound they quote follows *because* `validate` includes the strip
  coverage and in-file checks — a pre-check that stopped at the width×height multiply would not
  give it. The spec must say `Ok(n)` is returned only after strip coverage is checked.
- The width×height product is already `u64`-checked (`plane.rs:274`); `Sensor` from
  `Container::sensor()` already bounds its own allocations (`MAX_TAG_VALUES`, `src/ifd.rs:75`).
- Additive; no existing signature changes. Fuzz target to extend: `fuzz/fuzz_targets/plane.rs`.
- Open design question for the spec: do `demosaic_into` and `to_linear_f32_into` also need their
  own validate-before-allocate helpers, or is `n` enough? (Their claim: `n` is enough, since both
  allocate multiples of `n`. Holds: 3·n `u16` and 3·n `f32` are proportional to input.)

**Item 2 — correction to earlier planning: irradiance IS on crates.io.**
- crates.io lists **`0.1.1`** (published 2026-10-29, not yanked) and `0.1.0` (**yanked**), both
  before any demosaic work. `Cargo.toml` is `0.1.1`. `[Unreleased]` in `CHANGELOG.md` holds the
  demosaic (SPEC-030, SPEC-031), colour metadata (SPEC-026) and later work.
- So the answer to "which version to pin" is a **next minor, `0.2.0`** (additive modules and
  `Error` is `#[non_exhaustive]` at `src/lib.rs:143`, so new variants are non-breaking). It must
  go through a release spec (`just new-release-spec`; template `DEC-006`, not this repo's
  `DEC-006`) and the `DEC-028` publish checks. Not decided here: whether to cut `0.2.0` before STAGE-008 (they said not to wait for
  colour) — maintainer call.
- `docs/future-plans.md` (PR #78) says "cut and publish `0.1.0` first". **That is stale/wrong**
  — it was written without checking crates.io. Corrected alongside this entry.

**Item 3 — design decision needed, not a one-liner.** `Error` has about 40 variants and mixes
three audiences: unsupported-but-valid input (`UnsupportedCompression`, `UnsupportedBitDepth`,
`UnsupportedStripLayout`, `UnsupportedCfaPattern`, `UnsupportedOrientation`,
`UnsupportedMandatoryOpcode`, …), corrupt/hostile input (`Truncated`, `PackedSizeMismatch`,
`CyclicIfd`, …) and **caller misuse** (`PlaneBufferWrongLength`, `*WrongLength`). A boolean
`is_unsupported` leaves the third group lumped with "corrupt", which would map caller bugs to
"hostile file". Proposed shape: `Error::kind() -> ErrorKind { Unsupported, Corrupt, CallerError }`
(`#[non_exhaustive]`), classified by an exhaustive-in-tests table so a new variant cannot land
unclassified. Needs a DEC.

**Item 4 — partly blocked by an existing rule.**
- CI is Linux-only today: every job in `.github/workflows/ci.yml` is `runs-on: ubuntu-latest`;
  there is no macOS arm64 job.
- The Canon 5D III DNG is **tier-B corpus** and is never committed (`DEC-003`); CI runners have no
  corpus. Golden hashes on that file therefore cannot be asserted in CI as asked. Options: (a)
  tier-A **synthetic Bayer fixtures** (SPEC-027's builders) with committed golden hashes of all
  three outputs, run on a two-OS matrix; (b) additionally record the corpus-file hashes in the
  manifest for local runs. Recommend (a)+(b).
- Float paths, as documented rather than measured: `tone::to_linear_f32_into` "is a single IEEE
  division" over an integer lookup (`src/tone.rs:35`), and demosaic is integer arithmetic with
  round-half-up (CHANGELOG, SPEC-030/031). A single IEEE-754 division is correctly rounded on
  both targets, so divergence is not expected — **but that is a reading, not a measurement**;
  the CI job is what would measure it. The colour path (SPEC-033+) is `f64` and is not in
  pyirradiance's current scope.

**Proposed handling** (none started): item 1 → one `[M]` spec (extract + `validate` + forged-dims
tests + fuzz); item 3 → one decision then one `[S]` spec; item 4 → one `[S]` spec (two-OS golden
hash job); item 2 → a release spec after item 1 lands.

## Resolution

*Specs minted 2026-11-03* under **STAGE-009** (PROJ-002), all at `cycle: frame`; nothing is
built. Fill in merge commits and the released version as each ships.

| Item | Spec | Notes |
|---|---|---|
| 1 validate-before-allocate | SPEC-036 `[M]` | blocking; `plane::validate`, forged-dims tests, fuzz |
| 3 error categories | SPEC-037 `[S]` | needs a DEC first (next free DEC-035); depends_on SPEC-036 |
| 4 cross-arch determinism | SPEC-038 `[S]` | tier-A fixtures + two-OS matrix; corpus hashes local only |
| 2 release with `demosaic` | SPEC-039 `[S]` release | version to pin: **`0.2.0`** (not yet cut); depends_on SPEC-036 |

Status stays `open` until SPEC-036 and SPEC-039 ship.
