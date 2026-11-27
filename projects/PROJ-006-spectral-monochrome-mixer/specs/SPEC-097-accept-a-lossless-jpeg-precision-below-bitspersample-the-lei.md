---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-097
  type: bug                        # epic | story | task | bug | chore
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
  id: PROJ-006
  stage: STAGE-034
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5
  created_at: 2026-11-27

references:
  decisions: [DEC-085, DEC-077, DEC-029, DEC-047]  # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, test-before-implementation]
  related_specs: [SPEC-086, SPEC-091, SPEC-092, SPEC-093]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: a Leica M10 colour DNG (and any body that stores a narrower lossless-JPEG precision in a wider BitsPerSample) unpacks, so the colour path and the mixer can run on it"

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
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 120
      recorded_at: 2026-11-27
      notes: "Design in a side session started by the PROJ-006 orchestrator: reading, the relaxed-check prototype on the M10 and M11, dng_validate stage 1 and 2, dnglab and LibRaw runs, the absent-WhiteLevel copy, synthetic precision fixtures, the SDK source read, DEC-085, this design, HANDOFF-206. Un-metered main-loop cycle; measured at the STAGE-034 close as orchestration cost."
    - cycle: build
      handoff: HANDOFF-206
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 17442907
      estimated_usd: 11.98
      duration_minutes: 24
      recorded_at: 2026-11-27
      notes: "Measured live from this session transcript, deduplicated by message.id: a floor."
    - cycle: verify
      handoff: HANDOFF-207
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 15174129
      estimated_usd: 10.18
      duration_minutes: 50
      recorded_at: 2026-11-27
      notes: "Floor, measured live from transcript f32aa39b-afea-4a0a-a3c6-79ec81471dc7.jsonl, deduplicated by message.id; re-sum at ship. USD at Opus 5.5 rates (5/25 per M, cache reads 0.5, 1-hour cache writes 2x)."
  totals:
    tokens_total: 32617036
    estimated_usd: 22.16
    session_count: 3
---

# SPEC-097: Accept a lossless-JPEG precision below BitsPerSample, the Leica M10 file

## Context

First spec of STAGE-034 (PROJ-006). On 2026-11-26 the colour Leica M10 file from
raw.pixls.us (`LEICA-M10/f5381888.dng`, 5984×3992, `Compression = 7`, `BitsPerSample = 16`,
`WhiteLevel = 15000`, `BlackLevel = 0`, no opcode lists) fails at unpack with
`Error::LjpegPrecisionMismatch { precision: 14, bits_per_sample: 16 }`, before any colour
or mixer code runs. Adobe's `dng_validate` (local SDK build) validates the same file, with
only an unrelated `MaxApertureValue`/`LensInfo` metadata warning.

The refusal comes from `check_precision` in `src/plane.rs` (`SPEC-086`, `DEC-077` §3),
which requires the SOF-3 frame precision `P` to **equal** `BitsPerSample`. Its stated
reason is the case `P > BitsPerSample`: a P = 16 frame tagged 12-bit would clip at 4095
with `Ok`, because white defaults from `BitsPerSample` when `WhiteLevel` is absent. The
M10 is the other direction, `P < BitsPerSample`: every decoded sample is below `2^14`, so
it fits the 16-bit container and nothing clips, and the file carries an explicit
`WhiteLevel` (15000). The **Leica M11** does the same: `LEICA-M11-PB/leica_m11_01.dng` (60 MP,
camera-original, local corpus only) gives the identical refusal, has no opcode lists, and
`dng_validate` accepts it. The M11 is onyx's first-priority M body.

onyx's bodies, in order: Q3 and M11, then Q2 and M10, then SL2/SL3 (its DEC-095). A
precision refusal would stop every PROJ-006 entry point (SPEC-091's planes, SPEC-092's
mixer, SPEC-093's developed mix) on any such file.

## Goal

A `Compression = 7` strip or tile whose SOF-3 precision is **below** `BitsPerSample`
(with `BitsPerSample` at most 16) unpacks, with its samples unchanged, so the Leica M10 and
M11 files reach the colour path and the mixer; `P > BitsPerSample` stays refused exactly as
today.

## Inputs

- **Files to read:**
  - `decisions/DEC-085-...md` (this spec's decision), `DEC-077` §3 and its amendment
    note, `DEC-029` (samples above `WhiteLevel`), `DEC-047` (the shape binding that runs
    first).
  - `src/plane.rs`: `check_precision` (1974) and its two callers, the strip arm of the
    unit kernel (1778) and the tile arm of `decode_tile_row` (1913); `check_strip_shape`
    (1951) for the doc style.
  - `src/lib.rs`: `Error::LjpegPrecisionMismatch` (1194), its `Display` (1838) and kind
    (2066).
  - `src/develop.rs`: `resolve_white` (787) and `max_value_for_bits`; `src/color.rs`:
    `resolve_colour_levels` (white default at 2632). Read only: neither changes.
  - `tests/ljpeg_precision_binding.rs` (rewritten here), `tests/plane_oracle.rs`
    (`DECODABLE_COMPRESSED`, `compressed_files_are_skipped_by_name`,
    `ljpeg_decode_fault_turns_compression7_oracle_red`), `tests/corpus_manifest.rs`
    (`MANIFEST_ENTRIES`), `tests/previews.rs` (`PINNED_PREVIEWS`),
    `tests/metadata_oracle.rs` (`CFA_METADATA_NOT_YET_ORACLED`), `tests/error_kind.rs`
    (427).
  - `fuzz/fuzz_targets/plane.rs` and `fuzz/seeds/plane/`.
- **External (read or run at design; the build needs none of it):** DNG Specification
  1.7.1.0, the Adobe DNG SDK 1.7.1 source and `dng_validate`, dnglab 0.7.2, LibRaw 0.22.3.
  What each says is in DEC-085 §Context.
- **Corpus:** `$IRRADIANCE_CORPUS_DIR` = `~/PSeven/experiments/crustimg_redo_plus/images`.
  `LEICA-M10/f5381888.dng` (raw.pixls.us, CC0) and `LEICA-M11-PB/leica_m11_01.dng`
  (Photography Blog, no stated licence, local only), both added 2026-11-26, recorded at
  this design as `[[available]]` entries in `tests/corpus/manifest.toml`.

## Outputs

- **Files modified:**
  - `src/plane.rs`: `check_precision`'s condition and doc (Design §1).
  - `src/lib.rs`: `LjpegPrecisionMismatch`'s doc comment (Design §4). Its fields, kind and
    `Display` are unchanged.
  - `tests/ljpeg_precision_binding.rs`: rewritten to the new rule (Failing Tests).
  - `tests/corpus/manifest.toml`: the two `[[available]]` entries become `[[file]]`
    entries, with the pins below (Design §5).
  - `tests/corpus_manifest.rs` (`MANIFEST_ENTRIES` 12 to 14), `tests/previews.rs`
    (`PINNED_PREVIEWS` 21 to 28), `tests/plane_oracle.rs` (`DECODABLE_COMPRESSED` 5 to 7),
    `tests/metadata_oracle.rs` (the two files join `CFA_METADATA_NOT_YET_ORACLED`, with
    the reason).
  - `tests/leica_colour.rs` (new): the Tier-B ACs on the two Leica files.
  - `fuzz/seeds/plane/`: two seeds (Design §6).
  - `docs/conformance-matrix.md`: rows for the Leica M10 and M11 colour files.
  - `docs/provenance-ledger.md`: one sentence on the lossless-JPEG row (or its own row if
    there is none) recording DEC-085's class-4 SDK reading (Design §7).
  - `CHANGELOG.md` `[Unreleased]` Changed: a lossless-JPEG precision below
    `BitsPerSample` now decodes (SPEC-097, DEC-085).
- **Not modified:** `DEVELOP_VERSION` (`"8"`), `MONO_MIX_VERSION` (`"1"`), the `Error`
  enum's variants, `resolve_white`, `resolve_colour_levels`, every existing output,
  determinism hash and Tier-B pin.
- **New public API:** none.

## Design

1. **The rule (DEC-085 §1).** `check_precision` accepts a frame when
   `P ≤ BitsPerSample ≤ 16` and otherwise returns
   `Error::LjpegPrecisionMismatch { precision: P, bits_per_sample }`. In code that is the
   two comparisons `u32::from(frame.precision) <= bits_per_sample && bits_per_sample <= 16`,
   with 16 a named constant (`MAX_LJPEG_PRECISION`, or the `ljpeg` module's existing
   bound if it has one). The parser already bounds `P` to 2..=16. Accepted pairs:
   - `P = BitsPerSample`: as today, and the only case any held file had before this spec.
   - `P < BitsPerSample ≤ 16`: new, for any gap from P = 2.
   Refused pairs:
   - `P > BitsPerSample`: as today.
   - `BitsPerSample > 16`: as today, since `P` cannot equal it.
   The call sites, their order (after `check_strip_shape`, before any sample is decoded)
   and the three paths (`unpack_into`, `unpack_unit_into`, `PlaneDecoder::unpack_unit_into`,
   over strips and tiles) do not move.
2. **The samples (DEC-085 §2).** The decode is untouched: a P-bit frame's samples land in
   the `u16` plane as decoded, with no shift or rescale to `BitsPerSample`. This is what
   the SDK's stage 1, dnglab and LibRaw give on both Leica files (0 differing samples).
3. **White (DEC-085 §3).** No change. Mono `resolve_white` and colour
   `resolve_colour_levels` take `WhiteLevel` when present, else `2^BitsPerSample − 1`, never
   `2^P − 1`. That is the specification's default and the SDK's (`dng_ifd.cpp`
   `PostParse`); dnglab and LibRaw do the same when run on an M10 copy with no
   `WhiteLevel`. A P < BitsPerSample file with no `WhiteLevel` therefore develops dark.
   That follows the tags, and AC4 pins it. Samples above an explicit `WhiteLevel` (the
   M10 has 1,243,041 above 15000) clip at white, as DEC-029 and the specification's
   Chapter 5 already say.
4. **Errors and docs.** No variant change, so no semver change. `LjpegPrecisionMismatch`'s
   doc becomes: "a `Compression = 7` strip or tile's SOF-3 precision `P` is above the
   sensor's `BitsPerSample`, or `BitsPerSample` is above 16, the most lossless JPEG
   stores (`SPEC-086`, `DEC-077` §3 as amended by `DEC-085`). A precision below
   `BitsPerSample` decodes unchanged." `check_precision`'s doc says the same and names
   DEC-085. The `Display` text ("does not match") stays true for both refused cases, so
   it is left alone and `precision_mismatch_display_names_both_values` is unchanged.
5. **Corpus manifest.** At design both files are `[[available]]` entries: metadata only,
   read by nothing. The build promotes them to `[[file]]` (tier `"b"`), because the plane
   oracle and the Tier-B ACs read them through the manifest:
   - **M10:** path `LEICA-M10/f5381888.dng`, make `Leica Camera AG`, model `LEICA M10`,
     bits 16, bytes 34209792, sha256
     `26be5e06dfb53a2938dab3ca8f06024533a9c0dd0c652938f39eb48bc325f95e`, licence
     `CC0-1.0`, source `https://raw.pixls.us/` (Leica M10), `raw_checksum`
     `a1c6050a2cbfaf7d1b9a827ee3fd5bf6` (dnglab 0.7.2). Three preview pins, exiftool 13.55
     by the header's procedure:
     - `ifd_index` 1, 5952×3968, offset 29696, length 2506707;
     - `ifd_index` 2, 1440×960, offset 33070592, length 487729;
     - `ifd_index` 3, 160×120, offset 14336, length 15168.
     All three are `kind "primary"`, compression 7, photometric 6.
   - **M11:** path `LEICA-M11-PB/leica_m11_01.dng`, make `Leica Camera AG`, model
     `LEICA M11`, bits 16, bytes 102292452, sha256
     `4b91e16064526ea26967f7ea6bc5b6cdb23c8022f7cbe4e29413a55bdca78fee`, licence
     `personal-evaluation-only`, source the Photography Blog Leica M11 review gallery
     (`img.photographyblog.com/reviews/leica_m11/sample_images/leica_m11_01.dng`), with a
     note that it has **no stated licence, is never committed or redistributed, and no CI
     test may need it**. `raw_checksum` `9ae8c096b5ed99b86a88e52d8025168c` (dnglab 0.7.2).
     Four preview pins:
     - `ifd_index` 1, 160×120, offset 14336, length 14336;
     - `ifd_index` 2, 9504×6320, offset 28672, length 28005888;
     - `ifd_index` 3, 2112×1408, offset 28034560, length 1000448;
     - `ifd_index` 4, 720×480, offset 29035008, length 134144.
     All four are `kind "primary"`, compression 7, photometric 6.
   - If `tests/previews.rs` disagrees with a pin, the build reports it and does not edit
     the pin to match irradiance (the pins are exiftool's).
   - CI has no corpus (DEC-003), so every Tier-B test skips there by name; nothing in CI
     reads either file.
6. **Fuzz.** No new parser and no new input surface, but frames with `P < BitsPerSample`
   now get past the check into the decode. Add two seeds to `fuzz/seeds/plane/`:
   `compression7-p12-in-bps16.tiff` (one strip) and `tiled-lj92-p14-in-bps16.tiff` (one
   tile), written with the test writer as `tests/ljpeg_precision_binding.rs` builds them.
   Run the `plane` target for the toolchain brief's smoke length. The `ljpeg` target is
   unaffected (it never calls `check_precision`).
7. **Provenance.** No new algorithm. The rule is a relaxed check, and the white default is
   the existing one (DEC-061, from the specification). DEC-085 records what was learned by
   reading the SDK source (class 4, nothing copied: the SDK never compares the precision
   with `BitsPerSample`, and it bounds lossless JPEG to `BitsPerSample` 8 to 16). The build
   adds one sentence for this to the ledger's lossless-JPEG row.

## Implementation Context

**Facts the design probe measured (2026-11-27, release build, this machine).** The probe
relaxed `check_precision` to `P > BitsPerSample` in an uncommitted prototype. Its source,
not in the repo: `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b/spec-097-design-probe.rs.txt`
(its output is `spec-097-design-probe-output.txt` and the synthetic-fixture dumper is
`spec-097-design-synth-fixtures.rs.txt`, in the same folder).

| | M10 `f5381888.dng` | M11 `leica_m11_01.dng` |
|---|---|---|
| Plane | 5984×3992, one strip (30,533,623 bytes) | 9536×6336, one strip (72,514,048 bytes) |
| SOF-3 / SOS | P 14, Y 3992, X 2992, Nf 2, predictor 1, Pt 0, no DRI | P 14, Y 6336, X 4768, Nf 2, predictor 1, Pt 0, no DRI |
| `BitsPerSample` / `WhiteLevel` / `BlackLevel` | 16 / 15000 / 0 | 16 / 16383 / 1023 |
| CFA, crop | RGGB 2×2; crop origin (4, 4), size 5976×3984; Orientation 1 | RGGB 2×2; crop origin (4, 4), size 9528×6328; Orientation 1 |
| Opcode lists | none | none |
| `unpack_into` (relaxed) | Ok | Ok |
| Decoded min / max | 81 / 16383 | 1009 / 16383 |
| Samples above / at `WhiteLevel` | 1,243,041 / 75 | 0 / 78,144 |
| Samples ≥ 2^14 | 0 | 0 |
| Plane MD5 (dnglab `--raw-checksum` shape) | `a1c6050a2cbfaf7d1b9a827ee3fd5bf6` = dnglab | `9ae8c096b5ed99b86a88e52d8025168c` = dnglab |
| vs `dng_validate -1` stage 1 | 0 of 23,888,128 differ | 0 of 60,420,096 differ |
| vs LibRaw `unprocessed_raw` | identical | identical |
| `dng_validate -2` stage 2 vs clamp-normalise at `WhiteLevel` | exact (1,243,116 at 65535) | max \|Δ\| 1 on 11 samples |
| `develop_color_into` | Ok, 5976×3984, FNV-1a `0x7031d21ea67bdd45` | Ok, 9528×6328, `0xfb382cb038df474b` |
| `srgb_linear_plane_into` | Ok, `0xad88388f947b99db` | Ok, `0xe5214f13a0a4c09c` |
| `mono_mix_plane_into` (Rec. 709 luma 2126/7152/722 per 10000) | Ok, `0x3caa9be899d6ee98` | Ok, `0x380c75212c778806` |
| `develop_into` (plain mono develop of the mosaic) | Ok | Ok |
| `develop_with_into`, `mono_mix` = the same weights | Ok, 5976×3984, `0xb01b59c44e2bc0d3` | Ok, 9528×6328, `0xfcf9871708245c05` |

- The digests are FNV-1a 64 over the output's `u16` samples, little-endian bytes. Basis
  `0xcbf29ce484222325`, prime `0x100000001b3`, as SPEC-093's probe used.
- **They are regression pins, not an oracle.** The only code change is the check, so they
  are the existing pipeline's output on these files. AC5's oracle is dnglab's checksum.
- **What each file hits next:** nothing. Both reach every entry with `Ok`. Neither has
  opcode lists, so the STAGE-034 opcode spec does not apply to them.
- **Synthetic fixtures** (16×16 LinearRaw, one strip, `tiff::lj92_strip`; little-endian):
  - `dng_validate` validates (P, BitsPerSample) = (8, 16), (12, 16), (14, 16), (16, 12),
    (16, 16) and (8, 8) with no precision complaint.
  - dnglab decodes (12, 16), (14, 16), (16, 12) and (16, 16). It refuses P = 8 at both
    `BitsPerSample` 8 and 16 ("ljpeg: sof.precision 8"), a limit of its decoder.
  - LibRaw refuses all of them as unidentified cameras.
- **No `WhiteLevel`:** on an M10 copy with tag 50717 renamed, the SDK's stage-2 maximum
  is 16383 (white 65535), dnglab reports `whitelevels: [65535]`, and LibRaw's linear output
  peaks at 16383 with unit white balance (65535 with the tag present).
- **`dng_validate` warnings:** M10 `MaxApertureValue`/`LensInfo`; M11 three "not NULL
  terminated" strings (`ImageDescription`, `Artist`, `Copyright`). Neither is about the
  plane.

## Questions for design — answered

- **What do DNG 1.7.0.0 and the SDK say when `P < BitsPerSample`? Does white default from
  `BitsPerSample` or from `P`?**
  - The specification says nothing about the frame precision. It only lets the stream's
    width, length and components differ from the strip's (the `Compression` section;
    1.7.1.0's text, which the 1.7.1.0 change list shows is 1.7.0.0's).
  - The SDK never compares them. `dataPrecision` is used only in `GetSof`'s 2..16 check
    (`dng_lossless_jpeg_shared.cpp`).
  - White defaults from `BitsPerSample`: the specification's `WhiteLevel` default, and
    `dng_ifd.cpp` `PostParse` in the SDK. Driven on the SDK, dnglab and LibRaw with
    `WhiteLevel` removed.
  - **The design follows the specification and the SDK, which agree**, and keeps irradiance's
    existing default.
- **Any gap, or only where every sample fits?** Any gap, from P = 2, with `BitsPerSample` at
  most 16.
  - An honest P-bit frame always fits, since its samples are below `2^P ≤ 2^BitsPerSample`.
  - A corrupt stream can produce any `u16` at any `P` (today's equal case included), and
    the develop clamps at white. So a narrower rule buys nothing.
  - The SDK accepts the whole range: (8, 16) was driven.
  - The ceiling of 16 keeps today's refusal of a Compression 7 plane tagged wider than
    lossless JPEG can store.
- **Amend DEC-077 §3, or a new decision?** A new decision, **DEC-085**, that amends §3,
  plus a dated note in DEC-077 §3. This is the repo's practice for an accepted, shipped
  decision (DEC-029 amends DEC-008; DEC-076 amends DEC-034).
- **Does any output change for a file that unpacks today?** No. Every file that unpacks
  today has `P = BitsPerSample` (DEC-077's SOF-3 scan of every held Compression 7 file),
  and that path is unchanged. `DEVELOP_VERSION` stays `"8"` and `MONO_MIX_VERSION` `"1"`.
  AC6 holds every existing pin and hash.
- **Do the M10 (and M11) then pass the colour develop, `srgb_linear_plane_into` and
  `mono_mix_plane_into`, or hit the next refusal?** Both pass every entry, the developed
  mix and the plain mono develop included (the table above). No next refusal on either
  file.

## Acceptance Criteria

- **AC1. P below `BitsPerSample` decodes unchanged (Tier A).**
  - Grid: (P, BitsPerSample) in (2, 16), (8, 16), (12, 16), (14, 16), (15, 16), (8, 12),
    (12, 14); both byte orders; strips and tiles; all three decode paths.
  - Each decodes `Ok` to exactly the encoded samples.
- **AC2. P above `BitsPerSample` is still refused (Tier A).**
  - Grid: (16, 12), (14, 12), (13, 12), (16, 14), (16, 15), on the same grid of orders,
    layouts and paths.
  - Each gives `LjpegPrecisionMismatch { precision: P, bits_per_sample }`,
    `ErrorKind::Corrupt`, with `dst` untouched.
- **AC3. `BitsPerSample` above 16 is refused (Tier A).** (14, 17), (14, 20), (16, 17),
  (16, 32), (8, 32), on the same grid, refuse as AC2.
- **AC4. White defaults from `BitsPerSample`, not `P` (Tier A).**
  - Fixture: LinearRaw 16×16, P = 12 in `BitsPerSample` 16, no `WhiteLevel`, no
    `BlackLevel`, no crop.
  - `develop_into` gives exactly the decoded samples (white 65535, black 0, so
    `round(v · 65535 / 65535) = v`).
  - The same fixture with `WhiteLevel` 4095 gives `round(min(v, 4095) · 65535 / 4095)` per
    sample, hand-computed in the test.
- **AC5. The Leica planes match dnglab (Tier B, skips loudly).** With both files promoted
  to `[[file]]` and added to `DECODABLE_COMPRESSED`, `plane_md5_matches_the_pinned_raw_checksum`
  passes on each present file.
  - Its existing red-proof, `ljpeg_decode_fault_turns_compression7_oracle_red`, covers them
    with no new code, since it iterates `DECODABLE_COMPRESSED`. Both are `Nf = 2`, which its
    injected fault targets, and it must turn red on each present file.
  - The M11 never runs in CI. The M10 is CC0 but 32 MB, so it stays tier B (DEC-003).
- **AC6. Nothing else moves.** Unchanged and green on `main`'s values:
  - every existing plane pin, determinism hash and Tier-B develop pin;
  - `ac5_matching_precision_still_decodes` (P = BitsPerSample = 12, 14, 16);
  - `precision_mismatch_display_names_both_values`;
  - the `tests/error_kind.rs` row;
  - `DEVELOP_VERSION` and `MONO_MIX_VERSION`.
  `cargo semver-checks` reports no change.
- **AC7. The Leica files reach every PROJ-006 entry (Tier B, skips loudly).**
  `tests/leica_colour.rs`, for each present file, asserts `Ok` on:
  - `unpack_into`;
  - `develop_color_into`, `srgb_linear_plane_into` and `mono_mix_plane_into` (weights
    2126/7152/722 per 10000);
  - `develop_with_into` with `mono_mix` set to the same weights.
  Each output's FNV-1a digest must equal the Implementation Context table. A mismatch is
  reported, not edited. The test also asserts the M10's measured sample facts (max 16383,
  1,243,041 above `WhiteLevel`), so the DEC-029 clip case stays pinned to a real file.
- **AC8. Fuzz.** The two seeds are added, and the `plane` target runs its smoke length with
  zero crashes.

## Failing Tests

Written first, and each seen to fail (or, for the Tier-B ones, to refuse) against `main`.

- **`tests/ljpeg_precision_binding.rs`** (rewritten; Tier A):
  - `ac1_precision_below_bits_per_sample_decodes_unchanged`: red on `main`, which refuses
    every pair with `LjpegPrecisionMismatch`.
  - `ac2_precision_above_bits_per_sample_is_refused`: green on `main` (kept behaviour). It
    is listed for mutants (a) and (d).
  - `ac3_bits_per_sample_above_16_is_refused`: green on `main`, which refuses because
    `P ≠ BitsPerSample`. It is listed for mutant (c).
  - `ac4_white_defaults_from_bits_per_sample_not_precision`: red on `main` (the unpack
    refuses).
  - `ac5_matching_precision_still_decodes`: kept as is.
  - `precision_mismatch_display_names_both_values`: kept as is.
  - The old `ac5_precision_must_equal_bits_per_sample_on_every_decode_path` is removed. Its
    P16/BPS12 half moves into `ac2_…`, and its P12/BPS16 half is now AC1's acceptance.
- **`tests/leica_colour.rs`** (new; Tier B): `ac7_leica_files_reach_every_colour_entry`
  and `ac7_m10_samples_above_white_level_are_pinned`. Red on `main` where the corpus is
  present, because `unpack_into` refuses. They skip by name where it is absent.
- **`tests/plane_oracle.rs`**: `plane_md5_matches_the_pinned_raw_checksum` and
  `compressed_files_are_skipped_by_name`, once `DECODABLE_COMPRESSED` and the manifest
  grow. The first is red on `main` with the corpus present (unpack refuses). The second
  is red if the manifest and the lists disagree.

**Rule-clause map.**

| Clause | Test | What would change if the clause were wrong |
|---|---|---|
| §1 accept `P < BitsPerSample` | `ac1_…`, AC5, AC7 | the Leica files refuse again |
| §1 refuse `P > BitsPerSample` | `ac2_…` | a P16/BPS12 frame clips at 4095 with `Ok` |
| §1 refuse `BitsPerSample > 16` | `ac3_…` | a 32-bit-tagged Compression 7 plane decodes |
| §1 equality still accepted | `ac5_matching_…` | every held Compression 7 file refuses |
| §2 samples unshifted | `ac1_…` (exact equality), AC5 | samples scaled by `2^(BitsPerSample − P)` |
| §3 white from `BitsPerSample` | `ac4_…` | a no-`WhiteLevel` file brightens by `2^(BitsPerSample − P)` |
| both call sites | `ac1_…`/`ac2_…` over strips and tiles | one layout keeps the old rule |

**Mutants (the build applies each by hand, records the failing test, reverts):**
- (a) the old equality: `ac1_…` and `ac4_…` red.
- (b) `<` in place of `<=` (refuse equality): `ac5_matching_…` red.
- (c) drop the `≤ 16` bound: `ac3_…` red.
- (d) drop the `P ≤ BitsPerSample` bound (accept everything up to 16): `ac2_…` red.
- (e) the bound as `< 16`: `ac5_matching_…` (P = 16) and `ac1_…` (BitsPerSample 16) red.
- (f) the tile arm's call removed: `ac2_…` tiled red.
- (g) the strip arm's call removed: `ac2_…` strips red.
- (h) white default `2^(BitsPerSample − 4) − 1` in `resolve_white`: `ac4_…` red.
- (i) a shift `v << (BitsPerSample − P)` after the decode: `ac1_…` red, AC5 red where
  present.

## Non-Goals

- `P > BitsPerSample`: still refused (`DEC-077` §3's reason stands).
- `BitsPerSample` above 16 on a Compression 7 plane: still refused.
- Changing the white default, or taking `P` into the develop.
- Mandatory opcodes on the colour path (the Q2, Q3 and SL3: `FixBadPixelsConstant` in
  `OpcodeList1` and `WarpRectilinear` in `OpcodeList3`, both mandatory): STAGE-034's
  separate opcode spec. Those files unpack today and refuse later, at the develop.
- Other Leica quirks found later: each gets its own spec once a file shows it.
- Committing or redistributing any Photography Blog (`-PB`) file, or making any CI test
  depend on one. The M11 is a Goal target, held locally and recorded in the manifest as
  metadata only.

## Open questions

None blocks the build.

- **For the maintainer:** the M11 file has no stated licence. A `[[file]]` entry with
  `licence = "personal-evaluation-only"` and a never-redistribute note is the manifest's
  existing class for this. Confirm that, or keep it as `[[available]]` metadata only and
  drop it from AC5/AC7 (the M10 then carries the Tier-B evidence alone).
- **For onyx:** an M10 frame has samples above its `WhiteLevel` (5.2% of this frame's
  samples, up to 16383 against 15000). They clip at white, per the specification, so the
  M10's top ~0.13 EV of raw headroom is discarded in every irradiance output. That matches
  the SDK and LibRaw. Does onyx want that range (highlight recovery from above
  `WhiteLevel`)? If so it is a later spec, by demand.

## Notes for the Implementer

- The change is two comparisons and doc comments. Most of the work is the test grid, the
  manifest promotion and the count bumps.
- Build the AC1 to AC3 fixtures with the existing `comp7` helper. If `tiff::lj92_strip`
  cannot encode P = 2, report it and use the lowest P it can; do not change the encoder in
  this spec.
- Name the 16 once, as a constant, and use it in both the check and the doc. Do not
  hard-code it twice.
- Record AC7's digests from the Implementation Context table, not from your build.
- Do not touch `resolve_white`, `resolve_colour_levels` or the decoder.
- The M11 file must never be copied into the repo, a fixture, a seed or CI. Tier-B tests
  read it from `$IRRADIANCE_CORPUS_DIR` only.

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
