---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-050
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
  - src/lib.rs
  - fuzz/fuzz_targets/plane.rs
  - .github/workflows/ci.yml
  - README.md
  - CHANGELOG.md
  - docs/conformance-matrix.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-039-plane-validate-validate-before-allocate-extract.md

tags:
  - validate-before-allocate
  - plane
  - compression-7
  - resource-limit
  - preflight
  - fuzz
  - docs
  - stage-010
  - spec-039
---

# DEC-050: The unified validate-before-allocate preflight and its caller-selected pixel limit

## Decision

Group **E** of the pre-0.2.0 wave (review finding **#5**), re-designed after A–C
(SPEC-052/053/054) landed. The old SPEC-039 design (uncompressed-only) is replaced
by a single pre-allocation gate that covers **both** plane kinds and makes the
"never OOM a host that cannot catch an abort" guarantee un-forgettable.

### 1. One public preflight: `plane::validate`

```
pub fn validate(
    sensor: &Sensor,
    byte_order: ByteOrder,
    file: &[u8],
    max_pixels: usize,
) -> Result<usize, Error>
```

Allocation-free, panic-free, reads **no** strip bytes and writes nothing. It runs
every check `unpack_into` performs **before** touching `dst`, for whichever
compression the sensor declares, and returns `Ok(n)` (`n = width·height`) such that
`unpack_into` with a `dst` of length `n` cannot fail on layout, size, truncation,
precision or compression-support. `unpack_into` is refactored to call the **same**
shared check helpers, so `validate` and `unpack_into` cannot drift and today's error
precedence is preserved — in particular `PlaneBufferWrongLength` still wins over a
layout error when `dst` is the wrong size *and* the layout is bad.

`byte_order` is kept for signature parity with `unpack_into` (it governs only the
sample decode, not validation); the rustdoc says so.

### 2. Two resource guards — one per path, plus the universal caller limit

- **Uncompressed:** the sum of `StripByteCounts` may not exceed `file.len()` (new
  `Error::StripBytesExceedFile { declared: u64, file_len: usize }`), carried forward
  unchanged from the old SPEC-039 §3. Non-overlapping strips always satisfy it, so
  every real file is unaffected; only files whose strips must alias the same bytes
  are rejected. With it, `Ok(n)` ⇒ `n·bits ≤ 8·file.len()` for an uncompressed plane.
- **Both paths — the caller's budget:** `max_pixels` is checked **first**, from
  `width·height` computed as `u64` (before any heavy work and before any `usize`
  cast), returning `Error::PlaneTooLarge { pixels: u64, limit: usize }` when the
  plane would exceed it. This is the **only** pre-allocation bound available for
  **Compression-7**: a small compressed strip legitimately decodes to a large plane,
  so the file-bytes bound above does **not** hold for compressed data, and a forged
  huge `ImageWidth`/`ImageLength` in a tiny compressed file would otherwise drive the
  caller's `Vec` allocation straight to an OOM abort.

`max_pixels` is **the caller's** number, not a library cap: a caller that wants no
limit passes `usize::MAX`. Making it a required parameter (rather than returning `n`
for the caller to check afterwards) is the whole point — a Python/ML host cannot
*forget* the check, which is exactly the failure the spec exists to prevent. This
supersedes `feedback/2026-11-02-pyirradiance-requests.md`'s suggested 3-argument
shape; the extra argument is what makes the compressed path safe.

### 3. Compression-7 validate = the pre-decode structural checks only

For a `Compression = 7` plane, `validate` verifies strip layout, `RowsPerStrip`, each
strip's in-file offset/count bounds and the supported precision — the checks
`unpack_jpeg` runs before it decodes. It does **not** decode, so the SOF-3 decoded
frame-shape binding (`plane::check_strip_shape`, SPEC-053/DEC-047) and the sample
decode stay **runtime** checks inside `unpack_into`. `validate`'s rustdoc states
plainly that for a compressed plane `n` is bounded only by `max_pixels` (not by the
file length) and that the decode still enforces the shape binding. So the pre-0.2.0
guards compose: A (SPEC-052) bounds the IFD parse upstream of `Sensor`; `validate` is
the single pre-allocation gate; B (SPEC-053) binds the decoded frame at decode time.

### 4. EOI is optional — ratified, not changed

A SOF-3 scan is complete when it has decoded exactly `width·height` samples; a missing
trailing `EOI` (`0xFF 0xD9`) marker is **not** an error. This is the decoder's current
behaviour (`parse_headers` returns at `SOS`; the scan loop stops on sample count, and a
stream that ends early hits `Error::Truncated` through the bounded bit reader — EOI is
not consulted for completeness). Real encoders vary on emitting the trailing EOI, and
sample-count completeness already catches truncation, so requiring EOI would reject
valid files for no safety gain. The decision is to **keep lenient**, document it on the
decode entry, and pin it with a test (a valid scan with the trailing EOI removed still
decodes `Ok`). No code change.

### 5. `demosaic_into` / `to_linear_f32_into` need no separate validate

Unchanged from the old SPEC-039 §4: both write a fixed `3·n` multiple of the validated
`n` (`3 × 2 B` and `3 × 4 B` per pixel), so `validate`'s `n` is the only
pre-allocation number a caller needs. Stated in the `validate` rustdoc.

### 6. Fuzz smoke matrix gains `ifd`, `plane`, `ljpeg`

Today only `opcode` and `warp_opcode` have 60 s CI smoke jobs; the `ifd`, `plane` and
`ljpeg` targets (and their seed corpora) exist but are only *compiled* by the
fuzz-clippy job. E adds a 60 s smoke job for each (triage: "Real, cheap"), and extends
`fuzz/fuzz_targets/plane.rs` to the `sensor → validate → allocate(capped) →
unpack_into` contract (assert the contract both ways; assert the uncompressed bound
`n·bits ≤ 8·len`; cap the harness allocation with its own `MAX_PIXELS`, independent of
the library's caller-limit).

### 7. Docs reconcile (review #5)

`README.md` (the status block and the PROJ-003 scope line) still say Compression-7 is
unwired — stale since SPEC-043/046/044. `docs/conformance-matrix.md`'s Leica M Typ 246
and Pentax K-3 III rows still describe the `Nf=2` decode as pending — stale since
SPEC-046/044 shipped bit-exact. Reconcile README, CHANGELOG and the matrix, and add a
**tested-DNG-modes compatibility table** (uncompressed mono/Bayer; Compression-7 mono
single + two-component; what is still unsupported: tiled, Bayer-in-LJ92, other
compressions).

## Why

`plane::unpack_into` (`src/plane.rs:278`) validates layout, size and truncation only
*after* the caller has allocated `dst`, and the only check that involves `dst` is its
length. A forged `ImageWidth`/`ImageLength` makes the caller request gigabytes and the
OOM aborts the process — uncatchable from a Python worker. The uncompressed answer
(bound `n` by the file bytes) does not survive the move to Compression-7, where a few
hundred compressed bytes legitimately expand to a multi-megapixel plane. The only thing
that bounds a compressed plane before you decode it is a budget the caller names, so the
preflight must take that budget and enforce it, not merely report a number and hope the
caller re-checks it. This is the `parser-accepts-what-pipeline-cannot-honor` /
"safe to depend on" thesis at the allocation boundary.

## Rejected alternatives

- **A 3-argument structural-only `validate` that returns `n` for the caller to compare
  to its own limit** — rejected: for Compression-7 `n` is attacker-controlled and
  unbounded by the file, so a caller that forgets the comparison reintroduces the exact
  OOM. A required `max_pixels` makes the guarantee un-forgettable, which is the point.
- **A fixed library cap on `n`** — rejected: the library cannot know the caller's memory
  budget; the cap is the caller's to choose (`usize::MAX` = no limit). "No library cap,
  the caller decides" (old SPEC-039 non-goal) is honoured — the caller's number is now a
  parameter the library *enforces* rather than a comment the caller must obey.
- **Decoding inside `validate` to verify the SOF-3 shape pre-allocation** — rejected: a
  decode needs the `dst` we are trying to size, and B already binds the shape at decode
  time; `validate` stays allocation-free and reads no strip bytes.
- **Requiring a trailing `EOI`** — rejected: sample-count completeness already catches
  truncation, encoders vary on emitting EOI, and requiring it would reject valid files
  for no safety gain.
- **Folding group C's two review follow-ups into E** — rejected for 0.2.0 scope (see
  below): they re-touch the opcode path C just shipped and are a distinct concern.

## Scope and non-goals

- **In:** `plane::validate` (both paths + `max_pixels`); `Error::StripBytesExceedFile`
  and `Error::PlaneTooLarge`; the `unpack_into` refactor onto shared helpers; the
  EOI-optional test + doc; the `plane` fuzz extension and the `ifd`/`plane`/`ljpeg`
  smoke jobs; the README/CHANGELOG/conformance-matrix reconcile + compatibility table.
- **Out (deferred, recorded):** `Error::kind()` (SPEC-040) and the 0.2.0 release
  (SPEC-042). **Group C's two verify follow-ups are deferred, not folded here:** (1) a
  malformed `OpcodeList{1,2,3}` tag is still read as "absent" (DEC-012 tolerance), so it
  can hide a mandatory opcode — a decision on whether a malformed opcode-list tag must be
  refused belongs with the opcode path, not this allocation-boundary spec; (2) the public
  stage functions (`demosaic_into*`, `to_srgb_linear_into`, `apply_warp_into`) skip the
  required-processing check by design (they are primitives below the `develop_*`
  contract). Both are logged for SPEC-042 or a dedicated follow-up. No validate helpers
  for `demosaic_into`/`to_linear_f32_into` (§5). No cap on `n` chosen by the library (§2).
