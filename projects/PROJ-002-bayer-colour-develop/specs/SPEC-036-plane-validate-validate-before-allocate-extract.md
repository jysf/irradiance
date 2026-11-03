---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-036
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-009
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-016]                    # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application]                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-025, SPEC-027]                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-009's <capability>". Optional; null is acceptable.
value_link: null

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
  tokens_estimate: 18000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-036: `plane::validate` — validate before allocate

## Context

STAGE-009's blocking spec; source `feedback/2026-11-02-pyirradiance-requests.md` item 1. `plane::unpack_into` (`src/plane.rs:261`) checks strip layout, `PackedSizeMismatch` and `Truncated` only after the caller has allocated `dst`. A forged `ImageWidth`/`ImageLength` (e.g. 65535x65535 in a 2 KB file) makes a caller request ~8 GB; the OOM aborts the process, which a Python host cannot catch. The only check that involves `dst` is the length check; everything else can run without it.

## Goal

Add a public, allocation-free `plane::validate(sensor, byte_order, file) -> Result<usize, Error>` that runs all of `unpack_into`'s non-`dst` validation including strip coverage, so `Ok(n)` guarantees `unpack_into` with a `dst` of length `n` cannot fail on layout, size or truncation. `unpack_into` calls it internally so the two cannot drift.

## Design

### 1. The shape: `validate` = everything `unpack_into` checks except `dst`

`validate(sensor, byte_order, file) -> Result<usize, Error>` runs `require_uncompressed`, the bit-depth check, the `width*height` product, strip-count agreement, `RowsPerStrip`, and for every strip the `PackedSizeMismatch` check, the in-file bounds check, and full row coverage; on success it returns `n = width*height` as `usize`. It reads **no strip bytes** and writes nothing. `byte_order` is accepted so the signature is the one `unpack_into` already takes and the one pyirradiance asked for; today it does not affect validation (it governs only the sample decode), and the doc says so.

### 2. Sharing the code without changing error precedence

Today the order is: compression → bit depth → pixel product → **`dst.len()`** → strip layout. A naive `unpack_into = { validate()?; dst check; unpack }` would turn "wrong `dst` AND bad layout" from `PlaneBufferWrongLength` into a layout error — a (small) behaviour change. So the implementer must factor the checks into private helpers:

- `fn plane_len(sensor) -> Result<usize, Error>` — compression, bit depth, product → `n`.
- `fn strip_spans(sensor, file) -> Result<…>` — the strip checks (layout, `RowsPerStrip`, per-strip size, bounds, coverage, and §3).

`validate` = `plane_len` then a full pass of `strip_spans`. `unpack_into` = `plane_len`, the `dst.len()` check, then `strip_spans` while writing. One implementation of every check; the precedence is preserved; the exact helper shape is the implementer's (iterator, callback or struct), provided there is **no allocation** (no `Vec`) and no second copy of any check. If the `width*height` product does not fit `usize` (only on a 32-bit target; unreachable on 64-bit), `validate` returns the existing `PlaneBufferWrongLength { expected: pixels, actual: 0 }` rather than a new variant, and says so in its rustdoc.

### 3. The one deliberate tightening: strips may not alias past the file

Finding made while designing: pyirradiance's bound `n ≲ file.len()·8/bits` does **not** follow from the existing checks. Each strip's bytes are only checked to lie *inside* the file, and strips may overlap. A hostile file can carry up to `MAX_TAG_VALUES` (2²⁰) strips (needing only 8 bytes of offset/count table each, so a file of length `L` carries ≤ `L/8` of them) all pointing at the same `s ≤ L` bytes. Every per-strip check passes, `Ok(n)` is returned, and `n` is ~`L²/bits` — a ~100 KB file claims ~1 GB. That reintroduces exactly the OOM the item exists to prevent.

Fix, allocation-free and O(strips): **reject when the sum of `StripByteCounts` exceeds `file.len()`** — new `Error::StripBytesExceedFile { declared: u64, file_len: usize }` (additive; `Error` is `#[non_exhaustive]`). Non-overlapping strips always satisfy it, so every real file is unaffected; only files whose strips must overlap are newly rejected. With it, `Ok(n)` ⇒ `n*bits <= 8*file.len()` exactly (sum of padded strip sizes ≥ `n*bits/8`). `unpack_into` gets the same rule via the shared helper (it was previously "legal" only in the sense of nobody checking). **Must be verified against the tier-B corpus** (Canon 5D-III Bayer, Nikon D300S 12-strip, both Monochrom) via the local manifest run: none may trip it. This is the single behaviour tightening; it is called out in the CHANGELOG.

### 4. Do `demosaic_into` / `to_linear_f32_into` need their own validate? No.

`demosaic_into` writes `3·n` `u16`; `to_linear_f32_into` `3·n` `f32` for the same `n`. Both buffers are fixed multiples of the validated `n`, and §3 bounds `n` by the file length, so `validate` is the only pre-allocation check a caller needs. State this in the `validate` rustdoc with the multipliers (3 × 2 B and 3 × 4 B per pixel).

### 5. Fuzz target

`fuzz/fuzz_targets/plane.rs` becomes: parse → `sensor()` → **`validate`** → if `Ok(n)` and `n <= MAX_PIXELS` allocate `n` and call `unpack_into`, **asserting** it returns `Ok` (the contract); if `validate` is `Err`, call `unpack_into` with a tiny dst and assert it is also `Err` (never `Ok`). Also assert `n*bits <= 8*data.len()` whenever `validate` is `Ok`, for any `n` (even above the allocation cap). Keep the existing `MAX_PIXELS` cap (the harness, not the library, declines huge allocations). Add seeds: forged dims, aliased strips (`examples/fuzz-seeds.rs`).

### 6. Docs

Rustdoc on `validate` states the contract, the §4 multipliers, and the pyirradiance usage order (`Container::parse` → `sensor()` → `validate` → allocate → `unpack_into`). CHANGELOG `[Unreleased]`: Added `plane::validate`, `Error::StripBytesExceedFile`; Changed note for §3.

## Inputs

- **`src/plane.rs`** — `unpack_into` (l.261): its checks, in order, are `require_uncompressed`, `SUPPORTED_BITS`, the `width*height` product + `dst.len()` check, strip-count agreement, `RowsPerStrip`, then per strip `PackedSizeMismatch` / file-bounds `Truncated`, then full coverage.
- **`src/ifd.rs`** — `Sensor` (`strip_offsets`, `strip_byte_counts`, `rows_per_strip`, `bits_per_sample`, `packed_bits()`), `MAX_TAG_VALUES = 1<<20` (l.75).
- **`src/lib.rs`** — `Error` (`#[non_exhaustive]`, l.143); `tests/plane_oracle.rs`, `tests/support/tiff.rs` (tier-A builders, incl. SPEC-027's CFA builders) and the three `stage_probe_crate` red-proof copies.
- **`fuzz/fuzz_targets/plane.rs`** — the target to extend.
- **`feedback/2026-11-02-pyirradiance-requests.md`** — the requirement (item 1).

## Outputs

- **Files modified:** `src/plane.rs` (new `pub fn validate`, `unpack_into` refactored onto it); `src/lib.rs` (one new `Error` variant); `fuzz/fuzz_targets/plane.rs`; `CHANGELOG.md` (`[Unreleased] → Added`); `docs/provenance-ledger.md` (one row: own prior code, class 3 — a refactor of `unpack_into`).
- **Files created:** `tests/plane_validate.rs` — forged-dimension, contract and differential tests.
- **New public API:**
  - `pub fn validate(sensor: &Sensor, byte_order: ByteOrder, file: &[u8]) -> Result<usize, Error>` in `irradiance::plane`.
  - `Error::StripBytesExceedFile { declared: u64, file_len: usize }` (see Design §3).
- **No other signature changes.** `unpack_into`'s signature, doc'd errors and error precedence are unchanged for every input that is not a forged overlap (Design §3).

## Acceptance Criteria

- [ ] `validate` allocates nothing and cannot panic (five deny lints + `lint-no-allow` clean; fuzz target runs it on every input).
- [ ] **Contract:** for every fixture, `validate(..) == Ok(n)` ⇒ `unpack_into(.., dst)` with `dst.len() == n` returns `Ok(())`; and `validate(..) == Err(e)` ⇒ `unpack_into(.., dst)` with a correctly sized `dst` returns `Err` of the same variant.
- [ ] **No drift:** `unpack_into` runs the same validation code as `validate` (one shared helper, not two copies); a test fails if the two disagree on a mutated layout.
- [ ] **Forged dimensions:** a 65535×65535 (and `u32::MAX`×`u32::MAX`) sensor over a tiny file returns a typed error from `validate`, with no allocation proportional to the claimed size (the test never allocates `n`).
- [ ] **Bound:** whenever `validate` returns `Ok(n)`, `n * bits_per_sample <= 8 * file.len()` — it holds for every case including strips that alias the same file bytes (the `StripBytesExceedFile` rule).
- [ ] **No regression:** every existing test (`plane_oracle`, `plane_*`, CFA, multi-strip) passes unmodified; error precedence for every pre-existing failing input is unchanged, in particular `PlaneBufferWrongLength` still wins over layout errors when `dst` is the wrong size AND the layout is bad (Design §2).
- [ ] **Red-proof:** the new tests go red against a mutated `validate` (e.g. skipping the full-coverage check; skipping the aliasing check) — shown red before green (`oracle-must-be-shown-red`).
- [ ] The fuzz target runs sensor → `validate` → allocate (capped) → `unpack_into`, asserts the contract on every input, and runs clean for 60 s (`just fuzz-plane`).

## Failing Tests

- **`tests/plane_validate.rs`**
  - `"validate_ok_n_matches_pixel_count"` — asserts: for single-strip and multi-strip, 8/12/14/16-bit tier-A fixtures, `validate` returns `width*height` and `unpack_into` then succeeds.
  - `"validate_err_implies_unpack_err_same_variant"` — asserts: for each of: bad bit depth, compressed plane, strip count mismatch, zero strips, missing `RowsPerStrip`, wrong `StripByteCounts`, strip past EOF, strips undershooting height — `validate` and `unpack_into` return the same error variant.
  - `"forged_huge_dimensions_small_file"` — asserts: `ImageWidth = ImageLength = 65535` and `u32::MAX` over a ≤ 2 KB file → `validate` is `Err(_)` (`PackedSizeMismatch`/`Truncated`), and the test allocates nothing sized by the claim.
  - `"forged_dimensions_with_consistent_bytecounts_hit_truncated"` — asserts: a forged-big plane whose `StripByteCounts` agree with the dimensions but run past EOF → `Truncated`.
  - `"aliased_strips_are_rejected"` — asserts: many strips all pointing at the same few file bytes (sum of byte counts > `file.len()`) → `Err(StripBytesExceedFile { .. })`, and no `Ok(n)` ever violates `n*bits <= 8*file.len()`.
  - `"wrong_dst_and_bad_layout_keeps_old_precedence"` — asserts: wrong-length `dst` + bad layout → `PlaneBufferWrongLength`, exactly as before.
  - `"validate_is_pure"` — asserts: calling `validate` twice gives equal results; it takes `&Sensor`/`&[u8]` only.
  - `"validate_red_proof_*"` — asserts: replaying the committed validate source with the coverage / aliasing check removed makes the corresponding case above fail (mutated-copy apparatus, control run first per DEC-009).
- **`fuzz/fuzz_targets/plane.rs`** — extended per Design §5; no new test file.

## Non-Goals

- No change to any existing public signature; no change to the documented errors of `unpack_into` beyond the one new variant for overlapping strips
- No validate helpers for `demosaic_into` / `to_linear_f32_into` (see Design §4: their buffers are 3× the validated `n`, so `n` suffices)
- No cap on `n` inside the library — the *caller* decides how large a plane it is willing to allocate; `validate` makes the number trustworthy, not small
- No ML packing, normalisation or NumPy-buffer work (downstream)
- No `Error::kind()` (SPEC-037) and no release (SPEC-039)
- No new fuzz target — extend `plane`

## Notes for the Implementer

- Read `## Design` first; §2 (precedence) and §3 (aliasing) are the non-obvious parts.
- `just test` (not `--all-features`); five deny lints are on — no `unwrap`/indexing/`as`/unchecked arithmetic.
- The `stage_probe_crate` red-proof apparatus is copy-pasted in three test files; copy it a fourth time rather than refactoring it here.
- Fuzz: `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run plane …` (see `fuzz_targets/ifd.rs` header), or `just fuzz-plane`.
- Run the tier-B manifest locally (`IRRADIANCE_CORPUS_DIR`, see memory) to confirm §3 rejects nothing real; record the result in the handoff.

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
