---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-082
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-005
  stage: STAGE-017
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-193 (tier_map.build)
  created_at: 2026-11-24

references:
  decisions: [DEC-016, DEC-051, DEC-052, DEC-057, DEC-058, DEC-069, DEC-071, DEC-072, DEC-075, DEC-081, DEC-082]
  constraints: [no-panics-on-untrusted-input, library-not-application, provenance-recorded-per-algorithm, oracle-must-be-shown-red]
  related_specs: [SPEC-061, SPEC-062, SPEC-081]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-061, SPEC-081]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-017's <capability>". Optional; null is acceptable.
value_link: "Finder, Quick Look and Lightroom thumbnails for onyx's Linear DNG export (STAGE-017)."

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
  tokens_estimate: 15000000       # S; SPEC-081 (M) ran 17.4M, and this adds one IFD and two checks
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-24
      notes: "main-loop, not separately metered (AGENTS.md section 4); includes the preview-placement probe (dng_validate, exiftool, irradiance, Quick Look)"
    - cycle: build
      handoff: HANDOFF-193
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 11150285
      estimated_usd: 8.88
      duration_minutes: 80
      recorded_at: 2026-11-25
      notes: "FLOOR taken before the handback commit, from transcript 017f3954-2929-4417-8fd6-5d796f797cfa.jsonl (61 unique message.ids, all claude-opus-5-5, no subagents): input 124 + cache-creation 207,801 (1-hour) + cache-read 10,888,159 + output 54,201, priced at 5 and 25 USD per million as HANDOFF-191 did. Re-sum at ship. Oracle 2 MD5s pinned on 165a096 before any src edit (commit a1d00cb). exiftool byte-identical PreviewImage (gray fixture and synthetic YCbCr); dng_validate no warnings on K-3 III (own 6192x4128 preview, gray fixture) and Q2M (own 8368x5584 preview), renders equal the no-preview renders."
    - cycle: verify
      handoff: HANDOFF-194
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 8976828
      estimated_usd: 6.74
      duration_minutes: 75
      recorded_at: 2026-11-25
      notes: "FLOOR taken before the handback commit, from transcript eb87bed7-698f-4bbb-bd59-79a75b50424d.jsonl (52 unique message.ids, all claude-opus-5-5, no subagents): input 105 + cache-creation 162,490 (1-hour) + cache-read 8,785,354 + output 28,879, priced at 5 and 25 USD per million, 1-hour cache writes at 2x, as HANDOFF-192 did. Approved; the code under review is PR 336 head 59c1b64, and the verify commit adds one test assertion and docs only. 22 of 23 mutants killed (M6 only after the verify adds one tag-order assertion), the survivor X10 equivalent."
  totals:
    tokens_total: 20127113
    estimated_usd: 15.62
    session_count: 3
---

# SPEC-082: Optional caller JPEG preview as a reduced-resolution subfile

## Context

STAGE-017 (PROJ-005), after SPEC-061 and SPEC-081 (both shipped 2026-11-24). The writer
stores no preview today, so Finder, Quick Look and Lightroom have to render the raw plane
to show a written DNG. `monorail-000/DEC-075` item 2 / O5 (an external input, never in
`depends_on`) asks for zero or one preview: JPEG bytes the caller already encoded, stored
as a reduced-resolution subfile, never rendered by the writer (DEC-071 §5). onyx's v1
does not need it, but it is accepted. irradiance's reader already returns
`NewSubFileType` 1 previews as a zero-copy slice (DEC-057, SPEC-062), which gives the
round trip.

## Goal

Accept an optional caller JPEG, with its width, height and colour, in `WriteOptions`, and
store it as a `SubIFDs` child of the raw IFD0 (DEC-082). irradiance's `Container::previews`
then returns the exact bytes, exiftool extracts them, `dng_validate` gives no warnings, and
a file with no preview is byte-identical to SPEC-081's output.

## Design

### 1. The new options (`src/dng/write.rs`, behind `write`)

```rust
/// A caller-encoded JPEG preview (DEC-071 §5, DEC-082). The writer never decodes it.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct JpegPreview<'a> {
    /// The JPEG file's bytes, from its SOI marker on. Copied verbatim.
    pub jpeg: &'a [u8],
    /// The JPEG's width and height in pixels, 1 to 65535, as its SOF header says.
    pub width: u32,
    pub height: u32,
    /// What the JPEG's components are. Default YCbCr.
    pub color: JpegPreviewColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum JpegPreviewColor {
    /// Three components, YCbCr: PhotometricInterpretation 6, BitsPerSample 8, 8, 8.
    #[default]
    YCbCr,
    /// One component: PhotometricInterpretation 1 (BlackIsZero), BitsPerSample 8.
    Gray,
}

// New WriteOptions field (default None):
pub preview: Option<JpegPreview<'a>>,
```

`JpegPreview` is not named `Preview` because `crate::ifd::Preview` is the reader's type.
Both follow the `WriteOptions` pattern (DEC-052, DEC-058).

### 2. Tags

**IFD0** gains one slot, `SubIFDs` (330, LONG, count 1, the preview IFD's offset), between
`Artist` (315) and `XMLPacket` (700). It is written only with a preview. 41 slots in all.

**The preview IFD** (DEC-082 §3), ascending, all values fixed by the writer:

| Tag | Number | Type | Value |
|---|---|---|---|
| `NewSubFileType` | 254 | LONG | 1 |
| `ImageWidth` | 256 | LONG | `width` |
| `ImageLength` | 257 | LONG | `height` |
| `BitsPerSample` | 258 | SHORT | 8, 8, 8 (YCbCr) or 8 (Gray) |
| `Compression` | 259 | SHORT | 7 |
| `PhotometricInterpretation` | 262 | SHORT | 6 (YCbCr) or 1 (Gray) |
| `StripOffsets` | 273 | LONG | the JPEG's file offset |
| `SamplesPerPixel` | 277 | SHORT | 3 or 1 |
| `RowsPerStrip` | 278 | LONG | `height` |
| `StripByteCounts` | 279 | LONG | `jpeg.len()` |
| `PlanarConfiguration` | 284 | SHORT | 1 |

No `YCbCrSubSampling`, `PreviewColorSpace` or other preview tags (DEC-082 Consequences);
the probe found `dng_validate` silent without them. The preview IFD's next offset is 0.

### 3. Layout (DEC-082 §2)

```
offset 0   II 42 <IFD0 offset>
offset 8   the EXIF block (when given), padded to even
           IFD0, then its out-of-line values in tag order
           the preview IFD, then its out-of-line values (BitsPerSample's three SHORTs)
           the JPEG bytes
           the raw strip, last
```

Each IFD, value and strip starts at an even offset, and every pad byte is 0. The raw strip
stays last, so `serialize`'s split between the zeroed head and the strip is unchanged; the
preview IFD and the JPEG live in the head region. **Without a preview** nothing here is
written and the output is byte-identical to SPEC-081's for the same options (oracle 2).

### 4. Checks (DEC-082 §5)

Inserted as step 6e, after SPEC-081's XMP check (6d) and before the `OpcodeList1` walk, in
`plan`, so `written_len` returns the same error as `write_into`:

- the bytes are empty or do not start with `FF D8`: `PreviewNotJpeg`;
- `width` or `height` is 0 or above 65535: `PreviewDimensionsInvalid`.

Bytes first, then dimensions. The JPEG length is covered by SPEC-061's output-size step
(`OutputTooLarge`). Two new `WriteInputError` reasons, both `CallerError`, with rustdoc and
`Display` arms; no new `Error` variant.

### 5. Oracles and red-proofs

1. **Tags and layout, tier A.** With a preview (YCbCr and Gray), walking the output with
   `Container::parse` finds `SubIFDs` in IFD0 pointing at an IFD whose entries match §2
   exactly (type, count, bytes); the JPEG sits at `StripOffsets` and the raw strip is last;
   offsets are even; IFD0's and the preview IFD's next offsets are 0. Tier-A JPEGs are short
   synthetic byte strings that start `FF D8` and end `FF D9`; the writer never looks past
   two bytes.
2. **No preview, no change.** Before editing `src/`, the build records the MD5 of three
   tier-A outputs written on `main` (`386b4a7`): no metadata, every SPEC-081 field set, and
   a FullSensor output with masked areas and all three opcode lists. It pins those hashes in
   `no_preview_output_is_byte_identical_to_spec_081`, which must still pass after the change.
3. **Round trip.** SPEC-061's `check_round_trip` holds at every framing with a preview set:
   the sensor's samples, levels, geometry and metadata are unchanged, and `malformed_tags`
   is empty.
4. **The reader returns the bytes.** `Container::previews()` yields exactly one `Ok`
   preview: `kind` Primary, `ifd_index` 1, the caller's width and height, `compression`
   JPEG, `photometric` 6 or 1, `samples_per_pixel` 3 or 1, `bits_per_sample` 8, and `bytes`
   equal to the caller's JPEG.
5. **Red-proof.** Negative control first: oracle 4's comparison passes on a fresh output.
   Then, on a copy of that output with the preview IFD's `StripOffsets` value moved by 2,
   the comparison must fail. A second copy with `NewSubFileType` set to 0 must give no
   preview at all.
6. **exiftool (tool-gated).** `exiftool -b -SubIFD:PreviewImage` on the written file is
   byte-identical to the caller's JPEG, and `SubIFD:SubfileType`, `ImageWidth`,
   `ImageHeight`, `Compression` and `PhotometricInterpretation` read back as written.
7. **`dng_validate` (env-gated, local only, DEC-069 §6).** K-3 III and Q2M at
   StandardCrop, each with its source's largest JPEG preview (found with
   `Container::previews` on the source, so its declared size and colour are the source's),
   and with every SPEC-081 field set: no `Warning` or `Error` line. Also one run with the
   committed gray fixture (§6) as a Gray preview. The renders still equal SPEC-081's
   (a preview must not change the raw render).
8. **Determinism and allocation.** SPEC-061's oracles 9 and 10 with a preview set.
9. **Parity and order.** `every_refusal_is_typed` and `checks_run_in_the_documented_order`
   gain the two reasons, through all three entry points, after `EmptyXmp` and before the
   `OpcodeList1` walk.

### 6. Fixtures and fuzz

- `tests/fixtures/preview-gray-64x48.jpg`: a small one-component baseline JPEG for oracle
  7's Gray run, generated with macOS `sips` from a synthetic image (our own content, no
  camera data). Record the command in the test's doc comment.
- Extend `tests/support/dng_write_fuzz.rs` with the preview (bytes, width, height, colour),
  keep the agreement property, add a seed per new refusal and one with a preview, and extend
  `fuzz_seeds_reach_their_named_outcome`. If the target includes a new support file by
  path, add it to `scripts/lint-fuzz-red-proof.sh`.

### 7. Provenance

Ledger row: TIFF 6.0 §2 and §8 (IFD structure, strips, `Compression` 7 per the TIFF
Technical Note 2 JPEG scheme), TIFF/EP and DNG 1.7.1 (`SubIFDs`, `NewSubFileType` 1 for a
reduced-resolution preview), ITU T.81 (the SOI marker only), DEC-071 §5 and DEC-082. No
writer implementation read.

## Implementation Context — probed 2026-11-24

A SPEC-081 output of the K-3 III (StandardCrop, `Make`, `Model`, `BaselineExposure`) was
rebuilt by a throwaway Python script with a JPEG preview in three layouts, and checked with
Adobe `dng_validate` 1.7.1, exiftool 13.55, irradiance's `Container::previews` and
`Container::sensor`, and macOS Quick Look (`qlmanage -t -s 256`; a solid red 256×171
preview, so a red thumbnail proves the preview was used).

| Layout | dng_validate | irradiance | exiftool | Quick Look |
|---|---|---|---|---|
| no preview (SPEC-081 output) | no warnings | n/a | n/a | renders the raw (gray) |
| raw IFD0, preview as IFD1 (next-IFD chain) | warns `This file has Chained IFDs, which will be ignored by DNG readers` | not tried | | |
| **raw IFD0, preview as its SubIFD (chosen)** | no warnings | sensor OK, `malformed_tags` empty, one preview (`ifd_index` 1), exact length | `SubIFD:PreviewImage` byte-identical | uses the preview |
| preview IFD0, raw as its SubIFD | no warnings | sensor OK, one preview (`ifd_index` 0) | as above | uses the preview |

Same results with the K-3 III's own 720×480 JPEG preview. A one-component gray JPEG (made
with `sips` and the Generic Gray Gamma 2.2 profile) with `PhotometricInterpretation` 1 and
`SamplesPerPixel` 1 passes `dng_validate` and exiftool round-trips it. A gray JPEG declared
as YCbCr also passed `dng_validate`: it does not decode previews, so the caller's colour
declaration is not checked by any oracle tool. The held K-3 III stores `SubIFDs` as LONG
(count 2) and puts its 6192×4128 JPEG preview in a SubIFD. The probe files and script were
throwaways in the scratchpad (`tests/zz_probe_082*.rs` removed), not committed.

## Inputs

- **Files to read:** `src/dng/write.rs`, `src/write_types.rs`, `src/ifd.rs`
  (`Container::previews`, `Preview`, `PreviewKind`, the SubIFD walk), `tests/dng_write.rs`,
  `tests/dng_write_corpus.rs`, `tests/dng_write_alloc.rs`,
  `tests/support/{dng_write_oracle,dng_write_fuzz,tools,stage2_tiff}.rs`,
  `fuzz/fuzz_targets/dng_write.rs`, `examples/fuzz-seeds.rs`,
  `scripts/lint-fuzz-red-proof.sh`.
- **Decisions:** DEC-082, DEC-081 §4, DEC-071 §5, DEC-072, DEC-075 §1, DEC-057, DEC-051,
  DEC-052, DEC-058, DEC-016, DEC-069 §6.
- **External tools:** exiftool (tool-gated), `dng_validate` (env-gated, never CI), `sips`
  (once, to make the fixture).

## Outputs

- **Files created:** `tests/fixtures/preview-gray-64x48.jpg`.
- **Files modified:** `src/dng/write.rs` (types, field, slot, preview IFD, layout, checks,
  module docs), `src/write_types.rs` (two reasons), `tests/dng_write.rs`,
  `tests/dng_write_corpus.rs`, `tests/dng_write_alloc.rs`,
  `tests/support/dng_write_fuzz.rs`, `examples/fuzz-seeds.rs`, `fuzz/seeds/dng_write/*`,
  `docs/provenance-ledger.md`, `CHANGELOG.md` `[Unreleased]` Added, the AGENTS.md writer
  block.
- **New public API (behind `write`):** `JpegPreview`, `JpegPreviewColor`,
  `WriteOptions::preview`; `WriteInputError::{PreviewNotJpeg, PreviewDimensionsInvalid}`.
- **New flags:** none. `preview` defaults to `None`.

## Acceptance Criteria

- [ ] **AC1 Options.** The §1 API exists behind `write`; `preview` defaults to `None`.
- [ ] **AC2 Tags and layout.** Oracle 1 for YCbCr and Gray.
- [ ] **AC3 No change without a preview.** Oracle 2, hashes pinned from `main` before the
      change.
- [ ] **AC4 Round trip and reader.** Oracles 3 and 4 at every framing.
- [ ] **AC5 Validation.** Both reasons through `written_len`, `write_into` and
      `write_to_vec`, with `kind()` `CallerError`, in the §4 order (oracle 9). No panics.
- [ ] **AC6 Red-proof.** Oracle 5, after its negative control.
- [ ] **AC7 exiftool.** Oracle 6, tool-gated.
- [ ] **AC8 dng_validate.** Oracle 7 when `$IRRADIANCE_DNG_VALIDATE` is set; a SKIP line when
      unset; a failure when set but unusable.
- [ ] **AC9 Determinism, allocation, fuzz.** Oracle 8 and §6; `just fuzz-dng-write` 60 s
      with no crash; `just lint-fuzz`, `just lint-fuzz-red-proof` and `just deny-fuzz` pass.
- [ ] **AC10 Docs and gates.** Provenance row, CHANGELOG, AGENTS.md block, module docs (the
      layout, the tags and the check order); `just test`, `just lint-ci`,
      `just lint-no-allow`, `just fmt-check`, `just doc-check`, `just deny`, `just msrv`,
      `just validate`, `just gate-audit`, `just cost-audit` and every CI job on the PR pass.

## Failing Tests

- **`tests/dng_write.rs`** (tier A):
  - `preview_ifd_matches_the_spec_table` — AC2, YCbCr and Gray.
  - `preview_sits_between_ifd0_values_and_the_raw_strip` — AC2, layout and alignment, with
    and without an EXIF block.
  - `no_preview_output_is_byte_identical_to_spec_081` — AC3, three pinned MD5s.
  - `round_trip_holds_with_a_preview` — AC4, every framing.
  - `previews_returns_the_callers_jpeg` — AC4, oracle 4.
  - `red_proof_a_moved_preview_offset_is_caught` — AC6, oracle 5.
  - `every_refusal_is_typed` and `checks_run_in_the_documented_order` — AC5, extended:
    empty bytes, `FF D9 …`, a single `FF`, width 0, height 65536, width 65535 passes.
  - `write_is_deterministic_whatever_the_buffer_held` — AC9, extended.
- **`tests/dng_write_corpus.rs`** (tool-gated and env-gated, skip loudly):
  - `exiftool_extracts_the_preview_byte_for_byte` — AC7.
  - `dng_validate_accepts_a_written_preview` — AC8.
- **`tests/dng_write_alloc.rs`:** extended with a preview — AC9.
- **`fuzz_seeds_reach_their_named_outcome`:** a seed per new refusal and one with a
  preview — AC9.

**Rule-clause map.**

| Clause | Test | Path that changes the output |
|---|---|---|
| DEC-082 §1: preview is a SubIFD of the raw IFD0, no chain | `preview_ifd_matches_the_spec_table` | IFD0's `SubIFDs` value; both next offsets 0 |
| DEC-082 §2: layout, raw strip last, even offsets | `preview_sits_between_ifd0_values_and_the_raw_strip` | offsets of the preview IFD, the JPEG and the raw strip (an odd-length JPEG makes the pad observable) |
| DEC-082 §3: the preview IFD's tags per colour | `preview_ifd_matches_the_spec_table` | Photometric, SamplesPerPixel, BitsPerSample count |
| DEC-082 §4: caller dimensions written, JPEG not decoded | same, with dimensions that differ from any real JPEG | `ImageWidth` / `ImageLength` |
| DEC-082 §5: SOI check, dimension bounds, order | `every_refusal_is_typed`, `checks_run_in_the_documented_order` | which error, and 65535 vs 65536 |
| DEC-082 §6: same under every framing | `round_trip_holds_with_a_preview` | preview IFD present at all three framings |
| Goal: no preview, no change | `no_preview_output_is_byte_identical_to_spec_081` | the whole output's MD5 |
| DEC-057: the reader returns the exact bytes | `previews_returns_the_callers_jpeg`, `red_proof_a_moved_preview_offset_is_caught` | `Preview::bytes` |

## Non-Goals

- Encoding, decoding, resizing or colour-converting a preview; more than one preview.
- Validating the JPEG beyond its first two bytes; checking the declared size or colour
  against the stream.
- Preview in IFD0 with the raw plane in a SubIFD (DEC-082 Alternatives); alternate
  previews (`NewSubFileType` 65537); `PreviewColorSpace` and the other DNG preview tags;
  uncompressed or tiled previews.
- Any reader change.

## Notes for the Implementer

- **Record the three MD5s on `main` before you touch `src/`** (oracle 2). If you only
  compute them after the change, the test proves nothing.
- Every new check goes in `plan`, never in `serialize` (preflight parity).
- Keep the raw strip last; `serialize` zeroes everything before it.
- The JPEG and the preview IFD are in the zeroed head, so a pad byte after an odd-length
  JPEG is 0 without extra code; test it anyway with an odd length.
- `dng_validate` exits 0 even with warnings: the oracle is "no line containing `Warning` or
  `Error`".
- Do not read any writer implementation; use the tools as black boxes only.
- Run the whole CI set before handing back, including `just lint-fuzz-red-proof`, and
  confirm every job in `gh pr checks` passes.
- Commit messages in this repo must be plain prose: no code tokens.
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
