---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-053
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: S
  verify_verdict: approved

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5
  to_agent: null
  created_at: null

references:
  decisions: [DEC-045, DEC-047]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-039]

depends_on: []                     # independent — A/B/D fan out in parallel (DEC-045)

claimed_by: null

value_link: "a wrong-shape compressed DNG is rejected, not silently decoded to a mis-strided image — pre-0.2.0 correctness"

cost:
  tokens_estimate: 18000000
  sessions:
    - cycle: build
      handoff: HANDOFF-110
      agent: claude-opus-5
      interface: other
      tokens_total: 12629153
      estimated_usd: 8.72
      duration_minutes: 24
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 build session. tokens_total = sum of this session's transcript usage records (input + cache write + cache read + output, deduped by message id) at handback time; /cost is not callable as a tool in a delegated session; USD at Opus list rates. Excludes the final commit/report turns."
    - cycle: verify
      handoff: HANDOFF-114
      agent: claude-opus-5-5
      interface: other
      tokens_total: 3831612
      estimated_usd: 2.79
      duration_minutes: null
      recorded_at: 2026-11-10
      notes: "claude-opus-5-5 verify. Verdict APPROVED at 90f4898, no code change needed. Tests, lint, both red-proofs and the bit-exact corpus check all pass; DEVELOP_VERSION unchanged; fuzz clean (ljpeg 3.35M, plane 3.10M). tokens_total summed from the session transcript (/cost is not callable in a delegated session)."
  totals:
    tokens_total: 16460765
    estimated_usd: 11.51
    session_count: 2
---

# SPEC-053: Compression-7 decoded-frame shape binding (group B)

## Context

Pre-0.2.0 code-review finding **#2 (P1)**, triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted by DEC-045. The
decision is **DEC-047**. `unpack_jpeg` (`src/plane.rs:487`) sizes each strip's row
band to `rows_in_strip × sensor.width`, decodes the SOF-3 strip into it with
`ljpeg::decode_into`, and **drops the returned `Frame`** (`src/plane.rs:577-582`),
relying on `decode_into`'s `out.len() == effective_width × height` check
(`LjpegPlaneWrongLength`, `src/ljpeg.rs:993`/`:1182`) to pin dimensions. That is an
**area** check only: a frame that is an equal-area transpose of the band (decoded
3×4 vs a 4×3 band) passes it, is written row-major with the wrong stride, and
returns `Ok(())` with a silently garbled plane. Separately,
`parse_frame_header` (`src/ljpeg.rs:436`) reads each component's `Hi/Vi` byte and
**discards it** (`:468-472`), so a subsampled frame (`H/V != 1`, out of scope per
`DEC-037`) is decoded as `H=V=1` and also garbles silently. Both are
plausible-wrong-`Ok` paths at the public decode boundary — the finding-#1 class.

## Design

**Per DEC-047**, add two additive guards. Neither changes a correctly-shaped
`H=V=1` decode's output, so `DEVELOP_VERSION` does not move.

### 1. Shape binding in the caller (`unpack_jpeg`)

The expected geometry is the strip's, known only to `unpack_jpeg`, so the check
lives there, not in the generic `decode_into`. For each strip, **before**
`decode_into`:

- Parse the strip's headers: `let (frame, _tables, _pos) = ljpeg::parse_headers(strip)?;`
  (`parse_headers` is `pub(crate)`, `src/ljpeg.rs:341`).
- Compute `effective_width = frame.samples_per_line × frame.components` and
  `decoded_height = frame.lines` (for `Nf=2` the de-interleaved plane is
  `2 × samples_per_line` wide — `SPEC-046` — which is `sensor.width`).
- Require `effective_width == sensor.width` **and** `decoded_height == rows_in_strip`;
  otherwise return
  `Error::LjpegPlaneShapeMismatch { expected_width: sensor.width, expected_height: rows_in_strip, decoded_width: effective_width, decoded_height: frame.lines }`
  (all `u32`; use checked arithmetic for the `samples_per_line × components`
  product).
- Then call `decode_into(strip, chunk)` as today (its area check stays as a
  second, now-redundant-but-harmless guard).

The extra header parse is negligible (tens of bytes). **Do not** change
`decode_into`'s signature (DEC-047 "Rejected alternative").

### 2. Sampling-factor rejection in `parse_frame_header`

Where each component spec is read (`src/ljpeg.rs:468-472`), the `Hi/Vi` byte is
currently `let _hv = read_u8(...)?;`. Reject `hv != 0x11` (H=1, V=1) with
`Error::LjpegUnsupportedSampling { factor: hv }`. This runs for every SOF-3 stream
(the mono single-component path, the two-component path, and the fuzz hook), so a
subsampled frame is refused everywhere, not silently mis-decoded.

### 3. The two new error variants

In `src/lib.rs` (additive; `Error` is `#[non_exhaustive]`, `:217`), with `Display`
arms modelled on the `Ljpeg*` family:

- `Error::LjpegPlaneShapeMismatch { expected_width: u32, expected_height: u32, decoded_width: u32, decoded_height: u32 }`
- `Error::LjpegUnsupportedSampling { factor: u8 }`

Both **Corrupt**-class for SPEC-040's future table (note only; no classification
code here).

## Inputs

- **`src/plane.rs`** — `unpack_jpeg` (l.487), its per-strip loop (l.539-585), the
  row-band sizing (l.548-567), the `decode_into` call (l.582).
- **`src/ljpeg.rs`** — `parse_headers` (l.341), `parse_frame_header` (l.436, the
  `_hv` read at l.470), `Frame` (`lines`/`samples_per_line`/`components`, l.200-220),
  `decode_into` and the two-component decode (the area checks at l.993 / l.1182).
- **`src/lib.rs`** — `Error` (`#[non_exhaustive]`, l.217); the `Ljpeg*` variants and
  their `Display` arms to model the two new ones.
- **`tests/support/tiff.rs`** — `lj92_strip(samples, cols, rows, precision)` (l.337),
  `compression7_dng_with_strip` (l.452), `compression7_entries` (l.476),
  `compression7_dng` (l.439). A small helper to emit an `H/V != 1` SOF-3 component
  byte may be added here.
- **`fuzz/fuzz_targets/ljpeg.rs`** — the target to extend.
- **Corpus** — the two held Compression-7 DNGs (`IRRADIANCE_CORPUS_DIR`); both must
  still decode bit-exact.

## Outputs

- **Files modified:** `src/plane.rs` (the pre-decode shape check in `unpack_jpeg`);
  `src/ljpeg.rs` (the `H/V != 1` rejection in `parse_frame_header`); `src/lib.rs`
  (two new `Error` variants + `Display`); `fuzz/fuzz_targets/ljpeg.rs`;
  `CHANGELOG.md` (`[Unreleased] → Fixed`/`Security`); `docs/provenance-ledger.md`
  (one row: own prior code, class 3).
- **Files created:** `tests/ljpeg_shape_binding.rs` — the shape-mismatch, Nf=1/Nf=2,
  H/V and red-proof tests.
- **New public API:** the two `Error` variants above. No signature changes
  (`decode_into`, `parse_headers`, `unpack_into` unchanged).

## Acceptance Criteria

- [ ] **Equal-area shape mismatch rejected:** a Compression-7 DNG whose SOF-3 frame
      decodes to a shape that is an equal-area transpose of the declared strip
      (`effective_width != sensor.width` with matching area) returns
      `Err(LjpegPlaneShapeMismatch { .. })`, where previously it returned `Ok(())`.
- [ ] **Both component counts:** the shape check is exercised for `Nf=1` and `Nf=2`
      (for `Nf=2`, `effective_width = 2 × samples_per_line`).
- [ ] **Correct shapes unaffected:** a matching-shape Compression-7 plane decodes
      `Ok` with byte-identical output to today; both tier-B held files decode
      bit-exact (skip-loud when the corpus is absent, naming the file).
- [ ] **H/V != 1 rejected:** a SOF-3 frame with a component `Hi/Vi` byte other than
      `0x11` returns `Err(LjpegUnsupportedSampling { factor })`, on both the single-
      and two-component paths, where previously it decoded as `H=V=1`.
- [ ] **Panic-free:** five deny lints + `lint-no-allow` clean; checked arithmetic
      for the `samples_per_line × components` product; the fuzz target runs both
      guards on every input and runs clean for 60 s (`just fuzz`, target `ljpeg`).
- [ ] **Red-proofs (two):** the shape-mismatch test goes **red** against a mutated
      copy of `src/plane.rs` with the shape check removed (returns `Ok`); the H/V
      test goes **red** against a mutated `src/ljpeg.rs` with the sampling check
      removed. Controls (unmutated) run first (`oracle-must-be-shown-red`, DEC-009).
- [ ] **`DEVELOP_VERSION` unchanged** — no valid decode's output changes.

## Failing Tests

- **`tests/ljpeg_shape_binding.rs`**
  - `"equal_area_transposed_frame_is_rejected"` — asserts: a Compression-7 DNG with
    IFD `ImageWidth × ImageLength = W × H` but a SOF-3 frame of `H × W` (equal area)
    → `Err(LjpegPlaneShapeMismatch { .. })`. Build with `lj92_strip(.., cols=H, rows=W, ..)`
    inside `compression7_dng_with_strip` declaring `W × H`.
  - `"matching_shape_still_decodes"` — asserts: a Compression-7 DNG whose frame shape
    matches its strip decodes `Ok` with the expected samples (Nf=1 and Nf=2 cases).
  - `"nf2_effective_width_is_twice_samples_per_line"` — asserts: an `Nf=2` frame whose
    `2 × samples_per_line == sensor.width` passes, and one where it does not is rejected.
  - `"subsampled_component_is_rejected"` — asserts: a SOF-3 frame with a component
    `Hi/Vi` byte of e.g. `0x22` → `Err(LjpegUnsupportedSampling { factor: 0x22 })`
    (a small `tests/support/tiff.rs` helper emits the non-`0x11` byte).
  - `"corpus_comp7_files_still_bit_exact"` — asserts: both held Compression-7 DNGs
    decode `Ok` and bit-exact; skips loud, naming the file, when the corpus is absent.
  - `"shape_red_proof"` / `"sampling_red_proof"` — assert: replaying committed copies
    of `src/plane.rs` / `src/ljpeg.rs` with the respective guard removed makes the
    corresponding case above regress (returns `Ok`); control run first (DEC-009).
    Reuse the `stage_probe_crate` mutated-copy apparatus (copy it).
- **`fuzz/fuzz_targets/ljpeg.rs`** — extended so both guards run on every input; no
  new test file.

## Non-Goals

- No change to a correctly-shaped `H=V=1` decode's output and no `DEVELOP_VERSION`
  bump.
- Not *supporting* subsampled (`H/V != 1`) frames or any new SOF process — this
  rejects them.
- Not the restart-interval (`DRI`) MCU-reset hardening (a separate hardening item).
- Not the parser resource guard (SPEC-052) or the licence/package work (SPEC-055).

## Notes for the Implementer

- Read `## Design` and **DEC-047** first. The shape check is ~8 lines in
  `unpack_jpeg`; the H/V check is ~3 lines in `parse_frame_header`.
- `just test` (not `--all-features`); five deny lints are on.
- Fuzz target is `ljpeg`: `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run ljpeg …` (see the target header for the `+toolchain` trap).
- Run the tier-B corpus locally (`IRRADIANCE_CORPUS_DIR`, see memory) to confirm both
  held Compression-7 files still decode bit-exact; record the result in the handback.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary:
   `design` | `build` | `verify` | `ship` | `escaped` | `none`.
   — <one word>

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after. Write `none` if this spec has no user-visible outcome.
   — <answer | none>
