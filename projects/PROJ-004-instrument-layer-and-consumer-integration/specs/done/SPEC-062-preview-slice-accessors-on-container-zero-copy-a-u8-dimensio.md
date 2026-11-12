---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-062
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: M             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: punch-list  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [punch-list]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-018
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5           # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: 2026-11-12

references:
  decisions: [DEC-057, DEC-012, DEC-051, DEC-052, DEC-003]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, no-copyleft-dependencies, test-before-implementation, one-spec-per-pr]
  related_specs: [SPEC-057, SPEC-039, SPEC-040, SPEC-042]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-057]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-018's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-018 in full: the zero-copy preview accessor monorail-000 / onyx-000 asked for."

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
  tokens_estimate: 15000000
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 60
      recorded_at: 2026-11-12
      notes: "Un-metered main-loop design, done in the long monorail-000 orchestrator session, which also triaged the requests and framed STAGE-018. Its cost lands in STAGE-018's orchestration_cost at stage close, not here."

    - cycle: build
      handoff: HANDOFF-131
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 16626847
      estimated_usd: 12.64
      duration_minutes: 30
      recorded_at: 2026-11-12
      notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript (1 jsonl, 72 unique message.ids, all claude-opus-5-5, no subagents): input 148 + cache-creation 256,617 (1-hour) + cache-read 16,292,917 + output 77,165 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 15,119,257 / 11.81 before the final commit and PR turns."
    - cycle: verify
      handoff: HANDOFF-132
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 8376516
      estimated_usd: 6.23
      duration_minutes: 75
      recorded_at: 2026-11-12
      notes: "TRUE cost re-summed at ship by the orchestrator from the ended verify transcript (1 jsonl, 54 unique message.ids, all claude-opus-5-5, no subagents): input 111 + cache-creation 137,512 (1-hour) + cache-read 8,208,939 + output 29,954 (max per message). USD at opus-5-5 list rates. The handback floor was 5,550,953 / 4.44 before the commit and PR turns. Verdict punch-list: 2 test gaps found by mutation (u64 running strip end; count-0 NewSubfileType read as absent), fixed in PR 209, src unchanged."
  totals:
    tokens_total: 25003363
    estimated_usd: 18.87
    session_count: 3
shipped_at: 2026-11-12
---

# SPEC-062: Preview slice accessors on Container

## Context

This is the only spec in STAGE-018 (PROJ-004 instrument layer A). It is request 5 from
monorail-000 / onyx-000 (`feedback/2026-11-11-monorail-000-requests.md`). Their Preview Lie
Detector compares a DNG's embedded JPEG preview with the sensor plane. Today a caller has to:
walk `Container::ifds()`, guess which IFDs are previews, and resolve `StripOffsets` through
`Container::payload`, whose slice borrows `&self`, not the input.

The rules were locked in `docs/future-plans.md` § PROJ-004 (2026-11-02):
- select on `NewSubfileType`;
- return `&'a [u8]` borrowed from the input;
- never decode.

**DEC-057** (minted with this design) settles what those rules left open:
- the exact selection values;
- contiguous strips only;
- per-IFD `Result` items;
- no heap allocation.

## Goal

Add `Container::previews()`, a zero-copy, allocation-free iterator over a parsed DNG's
embedded previews. Each item gives dimensions, compression, photometric interpretation and
the bytes borrowed from the input. The sensor IFD is never reported, and every malformed or
unsupported layout returns a typed error instead of a panic.

## Design

### 1. Probe results (2026-11-12, recorded so build is transcription)

- **11 held tier-B DNGs, 21 preview IFDs.** Found with an independent Python TIFF walker,
  confirmed with exiftool 13.55 (`exiftool -a -G1 -s -n`).
  - Every preview has `NewSubfileType = 1` and **exactly one strip**.
  - Each is uncompressed RGB (`Compression 1`, `Photometric 2`, 8/8/8) or baseline-JPEG
    YCbCr (`Compression 7`, `Photometric 6`).
  - No tiles, no `JPEGInterchangeFormat` (513/514), no `65537` alternate previews, no IFD1
    chain.
  - Every sensor IFD has `NewSubfileType = 0`.
- **Per-file preview IFDs**, by `Container::ifds()` index (walk order: IFD0, then its
  SubIFDs):

  | File | Previews (index: w×h, compression/photometric) |
  |---|---|
  | `CANON-EOS-5D-MARK-III/5G4A9394.DNG` | 0: 256×171 1/2 · 2: 1024×683 7/6 · 3: 5760×3840 7/6 |
  | `GOOGLE-PIXEL-3A/IMG_20190918_164153.dng` | 0: 672×504 7/6 |
  | `GOOGLE-PIXEL-4A/PXL_20201121_100251397.dng` | 0: 672×504 7/6 |
  | `GOOGLE-PIXEL-7-PRO/PXL_20221204_063131635.dng` | 0: 1280×964 7/6 |
  | `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` | 0: 160×120 7/6 |
  | `LEICA-M-MONOCHROM/L1000622.DNG` | 0: 320×216 1/2 |
  | `LEICA-Q2-MONO/L1021223.DNG`, `L1026016.DNG`, `L1026192.DNG` | 0: 720×480 7/6 · 2: 1620×1080 7/6 · 3: 8368×5584 7/6 |
  | `NIKON-D300S/_DSC4615.DNG` | 0: 181×120 1/2 · 2: 1024×680 7/6 |
  | `PENTAX-K3III-MONO/K3III.DNG` | 0: 160×120 1/2 · 2: 6192×4128 7/6 |

- **exiftool names the byte range differently per IFD:**
  - `PreviewImageStart`/`PreviewImageLength`, for most JPEG previews;
  - `JpgFromRawStart`/`JpgFromRawLength`, for the largest JPEG (Q2M and Canon SubIFD2);
  - plain `StripOffsets`/`StripByteCounts`, for uncompressed RGB.

  The pinning step reads whichever pair exists in that IFD's group.

  exiftool's groups map to walk order:
  - `[IFD0]` is index 0;
  - `[SubIFD]` is IFD0's first SubIFD (index 1);
  - `[SubIFD1]` is index 2;
  - `[SubIFD2]` is index 3.

  Ignore `[Pentax] PreviewImageStart`: it is a MakerNote preview, outside the IFD tree
  (DEC-057).
- **Allocation today.** `Container::scalar` and `uints` build a `Vec`. The preview path must
  not use them (§3).

### 2. Public API (`src/ifd.rs`, re-exported as today's `ifd` items are)

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewKind { Primary /* NewSubfileType 1 */, Alternate /* 65537 */ }

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preview<'a> {
    pub ifd_index: usize,        // index into Container::ifds()
    pub kind: PreviewKind,
    pub width: u32,              // ImageWidth, required
    pub height: u32,             // ImageLength, required
    pub compression: Compression,// absent => Uncompressed (TIFF default 1)
    pub photometric: Option<u32>,// tolerant: absent => None (DEC-012)
    pub samples_per_pixel: u32,  // absent => 1
    pub bits_per_sample: Option<u32>, // first value; absent => None
    pub offset: u32,             // file offset of bytes[0]
    pub bytes: &'a [u8],         // borrowed from the input given to Container::parse
}

impl<'a> Container<'a> {
    pub fn previews(&self) -> Previews<'_, 'a>;
}
pub struct Previews<'c, 'a> { /* &'c Container<'a>, next index */ }
impl<'c, 'a> Iterator for Previews<'c, 'a> { type Item = Result<Preview<'a>, Error>; }
```

- **Selection (DEC-057):** read `NewSubfileType` with the allocation-free reader.
  - Absent means 0, so not a preview.
  - `1` is `Primary`, `65537` is `Alternate`. Every other value is skipped.
  - Unreadable (wrong field type, value out of file, and so on) yields **`Err`** for that
    IFD, and iteration continues.
- After any `Err` item, the next `next()` call moves on to the following IFD. The iterator
  never ends early, and never yields more than `ifds().len()` items.

### 3. Allocation-free tag reads

Add a `pub(crate)` reader on `Container` that returns the `i`-th unsigned value of an
entry (BYTE/SHORT/LONG, plus UNDEFINED where `uints` already accepts it) without building
a `Vec`:
- inline values come from the entry;
- out-of-line values are read at `offset + i·size`, with checked arithmetic and bounds;
- the error variants are the ones `uints` uses for the same faults.

`uints` and `scalar` stay as they are. The preview path uses only the new reader (a
`scalar`-equivalent is `i = 0` with `count >= 1`).

### 4. Byte range

Read in this order. The first failure wins.

1. **Tiles.** `TileOffsets` (324) present →
   `UnsupportedPreviewLayout { ifd_index, layout: UnsupportedPreview::Tiled }`.
2. **No strips, old-style JPEG.** `StripOffsets` absent and `JPEGInterchangeFormat` (513)
   present → `UnsupportedPreviewLayout { …, layout: JpegInterchangeFormat }`.
3. **No usable strip table.** `StripOffsets` or `StripByteCounts` absent, counts differ,
   or the count is zero → `InvalidPreviewLayout { ifd_index, strip_offsets, strip_byte_counts }`
   (counts as found, 0 when absent).
4. **Walk the strips**, `i = 0..n`.
   - `n` is bounded by the entry `count`, so it is already capped by `MAX_TAG_VALUES` via
     `Entry`/file size. The loop has no allocation, and its upper bound is the entry count.
   - Each strip must start at `start + running_len` exactly. Otherwise →
     `UnsupportedPreviewLayout { …, layout: NonContiguousStrips }`.
   - Sum lengths in `u64`. Overflow → `ValueOverflow { tag: 279 }`.
5. **Range check.** `start + total` must be `<= data.len()`, checked in `u64`. Otherwise →
   `Truncated { at: start, len: total }`, as `usize`; `OffsetOutOfRange` if it doesn't fit
   (`offset_to_usize`).
6. **Required tags.** `ImageWidth`/`ImageLength` absent → `MissingTag { tag }`.

### 5. Errors (`src/lib.rs`)

- `Error::UnsupportedPreviewLayout { ifd_index: usize, layout: UnsupportedPreview }`:
  `ErrorKind::Unsupported`.
- `Error::InvalidPreviewLayout { ifd_index: usize, strip_offsets: usize, strip_byte_counts: usize }`:
  `ErrorKind::Corrupt`.
- `#[non_exhaustive] pub enum UnsupportedPreview { Tiled, JpegInterchangeFormat, NonContiguousStrips }`.
- Each new variant gets:
  - a `Display` line naming the IFD index and the reason;
  - an arm in `Error::kind()`'s wildcard-free match (DEC-051);
  - a row in `tests/error_kind.rs`.

### 6. Corpus pin (`tests/corpus/manifest.toml`, `tests/support/corpus.rs`)

- Add an **optional** `[[file.preview]]` array to each `[[file]]` entry. Fields:
  `ifd_index`, `kind` (`"primary"`/`"alternate"`), `width`, `height`, `compression`,
  `photometric`, `offset`, `length`.
- Extend the reader to parse it. It stays optional: absent means "not pinned", which the
  corpus test reports by name. All 11 DNG entries get pinned values.
- **Pin procedure**, recorded in the manifest header: for each file, run

  ```
  exiftool -a -G1 -s -n -SubfileType -ImageWidth -ImageHeight -Compression \
      -PhotometricInterpretation -StripOffsets -StripByteCounts '-*Start' '-*Length' <file>
  ```

  Keep the IFD groups whose `SubfileType` is 1 or 65537. Take the byte range from that
  group's `PreviewImage*`, `JpgFromRaw*` or `Strip*` pair. Never pin from irradiance's own
  output.

  The `.PEF` entry is not a DNG preview source. Leave it unpinned with a note: it is a
  vendor container, so it is out of scope.

### 7. Fuzz

Extend `fuzz/fuzz_targets/ifd.rs`. After `parse` succeeds, drain `container.previews()`.
For every `Ok(p)`, assert that `p.bytes` lies inside `data`: pointer range, with
`p.offset as usize + p.bytes.len() <= data.len()`, and the slice pointer equal to
`data.as_ptr() + p.offset`. Add two seeds to `cargo run --example fuzz-seeds`: a two-preview
container and a non-contiguous-strip preview.

### 8. Docs

- CHANGELOG `[Unreleased] → Added` (the accessor and the types), with the two error
  variants under Added.
- A provenance-ledger row: TIFF 6.0 §8 (`NewSubfileType`, strips) and DNG 1.7 (preview
  IFDs, `0x10001`).
- A rustdoc example on `previews()`, with a `no_run` doctest that reads a file
  (`irradiance` does no I/O; the example's caller does).
- A README "Tested DNG modes" note is optional.

## Inputs

- **Files to read:**
  - `src/ifd.rs`: `Container`, `Entry`, `payload`/`uints`/`scalar`, `is_sensor_ifd`,
    `offset_to_usize`, tag constants;
  - `src/lib.rs`: `Error`, `ErrorKind`, `kind()`;
  - `tests/support/tiff.rs`: the raw IFD builder for hostile fixtures;
  - `tests/support/dng_writer.rs`: `IfdLayout::PreviewIfd0WithSensorSubIfd`;
  - `tests/support/corpus.rs` + `tests/corpus/manifest.toml`;
  - `fuzz/fuzz_targets/ifd.rs`, `examples/fuzz-seeds.rs`.
- **Decisions:** DEC-057 (this design), DEC-012, DEC-051, DEC-052, DEC-003.
- **External tool:** exiftool 13.55 (`/opt/homebrew/bin/exiftool`), run as a black box
  for pinning only.

## Outputs

- **Files modified:**
  - `src/ifd.rs`: `Preview`, `PreviewKind`, `Previews`, `Container::previews`, and the
    allocation-free reader;
  - `src/lib.rs`: two error variants, `UnsupportedPreview`, `kind()` arms, `Display`;
  - `tests/support/corpus.rs`, `tests/corpus/manifest.toml`: preview pins;
  - `tests/error_kind.rs`;
  - `fuzz/fuzz_targets/ifd.rs`, `examples/fuzz-seeds.rs`;
  - `CHANGELOG.md`, `docs/provenance-ledger.md`.
- **Files created:**
  - `tests/previews.rs`: tier-A behaviour, red-proofs and the tier-B corpus check;
  - `tests/previews_alloc.rs`: its own test binary with a counting `#[global_allocator]`.
- **New public items:** `ifd::Preview`, `ifd::PreviewKind`, `ifd::Previews`,
  `Container::previews`, `UnsupportedPreview`, `Error::UnsupportedPreviewLayout`,
  `Error::InvalidPreviewLayout`. **No new dependency, no feature flag, no version bump.**

## Acceptance Criteria

- [ ] **AC1 Selection.** A `tiff.rs` fixture with IFDs whose `NewSubfileType` is
      absent, 0, 1, 4, 5, 8, 9, 16, 65537 and 65540 reports exactly the 1 (`Primary`) and
      65537 (`Alternate`) IFDs, in walk order, with correct `ifd_index`.
- [ ] **AC2 Size never decides.** In a fixture where a preview (NST 1) is **larger** than
      the sensor IFD (NST 0, LinearRaw), the preview is reported and the sensor is not, and
      `Container::sensor()` still picks the sensor.
- [ ] **AC3 Zero-copy.** For every `Ok` preview, `bytes.as_ptr()` equals
      `input.as_ptr() + offset` and `bytes.len()` equals the summed strip byte counts, in
      tier A and tier B.
- [ ] **AC4 Contiguous multi-strip.** Three contiguous strips are returned as one slice
      spanning all three. A gap, an overlap or out-of-order strips give
      `UnsupportedPreviewLayout { layout: NonContiguousStrips }`.
- [ ] **AC5 Typed errors, iteration continues.** Each case gives its typed error and no
      panic:
      - tiled;
      - `JPEGInterchangeFormat`-only;
      - count mismatch;
      - zero strips;
      - offset past the end;
      - `offset + length` past the end;
      - a length sum that overflows;
      - missing width or height;
      - an unreadable `NewSubfileType`.

      In a fixture with a bad preview **before** a good one, the good one is still yielded
      after the `Err`.
- [ ] **AC6 No allocation.** `tests/previews_alloc.rs` counts allocations with a
      `#[global_allocator]` wrapper. Draining `previews()` on a pre-parsed three-preview
      fixture, Ok and Err items alike, performs **0** allocations.
- [ ] **AC7 Corpus matches exiftool.** For every present tier-B DNG, `previews()` gives
      exactly the pinned `[[file.preview]]` list (all eight fields). All 21 previews match.
      An absent file skips loudly by name.
- [ ] **AC8 Red-proofs.** Each mutation makes a named test go red, with the honest run
      passing first:
      - **selection widened:** `nst & 1 != 0` instead of `== 1 || == 65537`, so AC1 fails
        on NST 5/9;
      - **one strip only:** using only the first strip's length, so AC4 fails;
      - **range off by one:** the slice offset shifted by one byte, so AC3 and AC7 fail.

      Recorded as `#[ignore]`d mutant tests, or the repo's existing red-proof pattern.
- [ ] **AC9 Error kinds.** Both new variants are classified (`Unsupported`, `Corrupt`),
      have `Display` text, and appear in `tests/error_kind.rs` (DEC-051).
- [ ] **AC10 No collateral change.** Existing tests are unchanged and green, every
      `raw_checksum` oracle passes, and the public API has only additions.
- [ ] **AC11 Fuzz.** The extended `ifd` target runs for **10 minutes** with no crash, and
      the run count is reported in the PR.
- [ ] **AC12 Gates green.** `just test`, `lint-ci`, `lint-no-allow`, `fmt-check`,
      `validate`, `doc-check`, `deny`, `deny-fuzz`, `msrv`, `cost-audit`, `gate-audit`.

## Failing Tests

- **`tests/previews.rs`** (new)
  - `selection_is_exactly_nst_1_and_65537`: AC1.
  - `larger_preview_than_sensor_is_still_a_preview`: AC2.
  - `preview_bytes_are_a_subslice_of_the_input`: AC3 (tier A).
  - `contiguous_strips_join_into_one_slice`: AC4.
  - `non_contiguous_strips_are_unsupported`: AC4, covering gap, overlap and reversed.
  - `hostile_preview_layouts_are_typed`: AC5, one case per bullet.
  - `bad_preview_does_not_hide_a_later_good_one`: AC5.
  - `corpus_previews_match_exiftool_pins`: AC3, AC7, tier B.
  - `red_proof_*`: AC8.
- **`tests/previews_alloc.rs`** (new): `draining_previews_allocates_nothing`, AC6.
- **`tests/error_kind.rs`**: new rows, AC9.
- **`tests/corpus_manifest.rs`**: the manifest still parses with the `[[file.preview]]`
  arrays, and a malformed preview pin is rejected by name.

## Non-Goals

- **Decoding a preview**, or checking its JPEG markers. The host decodes.
- **MakerNote previews** (Pentax `[Pentax] PreviewImageStart`) and EXIF-IFD thumbnails.
- **Tiled, `JPEGInterchangeFormat` or non-contiguous previews.** They are refused, not
  supported (DEC-057). Supporting them needs a named file.
- **The ADU histogram, zones and bad-pixel listing** (the rest of future-plans Stage A).
- **Changing `Container::payload`'s lifetime**, or `uints`/`scalar` themselves.
- **A version bump or release.**

## Notes for the Implementer

- **Clean-room.** The rules come from TIFF 6.0 §8 and the DNG 1.7 spec text. exiftool is a
  black box for pinning; do not read its source or another reader's.
- **Pin from exiftool, never from irradiance.** Pinning our own output would make AC7
  circular. If our answer and exiftool's differ, stop and report it.
- **`Container` borrows `&'a [u8]`.** Build `bytes` from that field, not through `payload`,
  so the lifetime is `'a`.
- **Allocation test binary.** Keep the counting allocator in its own `tests/*.rs` file, so
  no other test runs under it. Parse **before** you start counting.
- **`Error`, `Preview` and `PreviewKind` are `#[non_exhaustive]`** (DEC-052). List the
  additions in the CHANGELOG all the same.
- **Commit-message bodies:** plain prose only, no code tokens. `Cargo.lock` is gitignored.
  Tier-B files stay in `$IRRADIANCE_CORPUS_DIR` and are never copied (DEC-003).

---


## Reflection

*Appended during **ship**, 2026-11-12.*

1. **What would I do differently next time?**
   — Two things. First, give every declared deviation and every defensive guard its
   own fixture at design time. Both verify findings were correct code with no test:
   the count-0 `NewSubfileType` rule (a deviation the build declared) and the `u64`
   running strip end (the only wrap fixture overflowed on the first strip, so an earlier
   guard caught it and the later one was never reached). Second, the 15M estimate was
   low: true cost 25.00M (build 16.63M, verify 8.38M). The handback floors ran 9% and
   34% low again, so re-summing at ship stays necessary.

2. **Does any template, constraint, or decision need updating?**
   — No DEC change: DEC-057 held through build and verify unchanged. One lesson is
   recorded as a new watch signal, [[hostile-fixture-shadowed-by-an-earlier-guard]]
   (N=1). It sits beside [[tolerant-oracle-cannot-pin-structure]] (N=2), which is the
   same family: a guard or property that no test reaches. The build's two template
   notes (keep new fuzz fixtures out of `tiff::all()`; name the staged-copy red-proof
   precedent) are handoff wording, not a template change yet.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. Carried forward in the monorail-000 feedback entry: Stage C (develop options,
   caller scratch, CFA-as-mono, orientation), the unit-unpack stage, and item 8
   (`MaskedAreas` pass-through, low priority). Tiled or non-contiguous previews stay
   refused until a consumer asks (DEC-057 names the arm to relax).

4. **Where was the worst defect caught?**
   — verify

5. **What can a user do now that they couldn't before?**
   — Before, a consumer had to walk the IFDs and slice preview bytes itself; after,
   `Container::previews()` hands back every embedded preview as a zero-copy slice of the
   caller's buffer, with no allocation, and all 21 previews in the 11 held DNGs match
   exiftool's offsets and lengths and its extracted bytes (MD5-checked on 8).
