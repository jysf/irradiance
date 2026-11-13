---
source: "monorail-000"
captured_at: 2026-11-11
captured_by: claude
status: open
---

# monorail-000 (macOS monochrome app): unit-level unpack, develop options, caller scratch, orientation speed, previews, CFA override

## The issue

Upstream requests from **monorail-000**, the macOS monochrome app (the "Monochrom app" named
as a PROJ-004 beneficiary). Naming: monorail-000 is a codename for onyx-000, which is itself
the codename for the app suite until it gets a final name. All three refer to the same consumer. The maintainer pasted them. Evidence is their SPIKE-001: M4 Pro,
pinned to `irradiance = 0.2.1`.

1. **Tile- or strip-level unpack so callers can parallelise.** `plane::unpack_into` is
   single-threaded. A 47 MP Q2M unpack (uncompressed) takes 240 ms. Compression 7 is about
   2.9× slower per pixel: 703 ms for 26.6 MP. Proposal: `plane::units(sensor)`, yielding a
   rect and a byte range per unit, and `plane::unpack_unit_into(...)`, which writes only that
   unit's rect. irradiance stays single-threaded and the caller chooses the concurrency.
   Acceptance: units assembled in any order are byte-identical to `unpack_into`; a red-proof
   with a rect off by one row; a fuzz target; no new allocation.
2. **A develop options struct.** `develop_with_into(sensor, src, dst, &DevelopOptions)` with
   `crop: StandardCrop | FullActiveArea`, `warp_alpha: f64` in [0, 1] (a WarpRect lerp) and
   `apply_orientation: bool`. `develop_into` keeps today's defaults. Acceptance: the defaults
   are byte-identical to `develop_into`; alpha 0 equals the warp-free path; a bounds oracle
   over every corpus frame for alpha in {0, .25, .5, .75, 1}, red-proofed with a synthetic
   over-unity polynomial; arithmetic per DEC-033.
3. **Caller-owned scratch for develop.** `develop::scratch_len(sensor, &options)` plus a
   variant that takes `scratch: &mut [u16]`. Acceptance: byte-identical output; a test
   asserting no allocation; a typed error for wrong-length scratch.
4. **Orientation 6 performance.** Develop takes 774 ms with Orientation 6 and 617 ms with
   Orientation 1 on the same plane. Proposal: a blocked transpose, or rely on #2's
   `apply_orientation: false`. Acceptance: byte-identical output, with a benchmark before
   and after.
5. **Preview slice accessors.** A zero-copy `Container::previews()` giving dimensions,
   compression and `&'a [u8]`. It never decodes. The app's Preview Lie Detector compares
   the Q2M's embedded JPEG (SubIFD2) with the sensor plane. Acceptance: dimensions and byte
   ranges match exiftool on all corpus files; typed bounds errors; a fuzz target.
6. **CFA override for converted cameras.** Cameras with the colour filter array removed
   still declare `CFAPattern`, so they get routed to demosaic. Proposal: an option that treats
   a CFA sensor as monochrome and keeps per-CFA-position black and white levels.
   Acceptance: a synthetic CFA-declared uniform plane gives the same output as the monochrome
   path; per-position levels are honoured; a red-proof with the positions swapped.
7. **Docs: the changelog.** They report that main's CHANGELOG stops at `0.1.1`.

## Triage (claude, 2026-11-11, read from `origin/main` `e5480ef`; nothing built)

Everything below comes from reading the code. Nothing was run, and none of their timings
were reproduced.

**Item 7: already resolved. No work.** `CHANGELOG.md` on main has `[0.2.1] - 2026-11-11` and
`[0.2.0] - 2026-11-10` sections (added by SPEC-042 and SPEC-056 / #196). Their checkout or
the crate tarball they read was older. Reply: the 0.2.1 notes are on main.

**Item 1: feasible, but "units = strips or tiles" gives no parallelism on their main file.**
- `src/plane.rs` documents that every PROJ-001 corpus file has **one strip** (`N == 1`),
  including the Q2M. Strip units on the 47 MP Q2M would mean one unit, with no speed-up.
- For **uncompressed** data a unit can be a row band inside a strip whenever a row ends on
  a whole byte (`width · bits % 8 == 0`), so a row's byte offset can be computed directly.
  This is the case that matters for the 240 ms figure.
- For **Compression 7** a single SOF-3 bitstream can't be split without restart markers.
  Units are possible only where the file has several strips or tiles, or restart intervals
  (`src/ljpeg.rs` already handles restarts). The Q2M is uncompressed, so their 26.6 MP
  Comp-7 figure comes from another camera. Which one, and how it is laid out, must be
  checked before the design is fixed.
- Tiles arrive with **SPEC-058** (the tiled reader, DEC-056, verify in progress). The unit
  API should reuse DEC-056's clipped strided window rather than invent a second tile path,
  so this spec depends on SPEC-058.
- A new public API needs a DEC: what a `Unit` is (rect plus byte range, or rect plus
  restart-interval index), and the rule that `unpack_into` iterates the same units so the
  two can't drift. It follows the shape SPEC-039 used for `validate`.
- Before design: a short spike to list strip counts, tile counts and restart intervals
  across the held corpus, the Comp-7 files included.

**Item 2: already planned. It is PROJ-004 instrument-layer Stage C.** `docs/future-plans.md`
§ PROJ-004 locked these rules 2026-11-02: two framing modes only (`StandardCrop`,
`FullActiveArea`); `WarpRect::lerp(alpha)`; and "FullActiveArea + lerp is valid only if the
backward radial scale stays `<= 1`; the applier returns a typed error, never an
out-of-bounds read". Their bounds oracle and over-unity red-proof match that rule. What
they add: **`apply_orientation: bool`**, and **a single options struct** in place of
separate entry points. `develop_into` today hard-codes the order bad pixels → levels →
warp → DefaultCrop → orientation (`src/develop.rs:802`). An options struct is additive.
`DevelopOptions` should be `#[non_exhaustive]` with a `Default` that reproduces
`develop_into`, following DEC-052's sealing convention.

**Item 3: confirmed, and slightly worse than reported.**
- With a non-identity warp, `develop_into` allocates **two** active-area `Vec<u16>` scratch
  buffers. The doc comment admits it (`src/develop.rs:761-777`, the SPEC-018 amendment to
  DEC-018). Every real Q2M frame has a mandatory WarpRectilinear, so this path always runs.
- Not in their report: when `OpcodeList1` carries `FixBadPixelsConstant`, `develop_into`
  also makes a **full raw-plane copy** (`src.to_vec()`, `src/develop.rs` in the
  `fix_bad_pixels_constant` branch). The fix pass also allocates a small neighbours `Vec`
  (`Vec::with_capacity(8)` in `apply_fix_bad_pixels_constant`). `scratch_len` has to cover
  the plane copy as well, or the scratch path still allocates on files with bad-pixel
  opcodes. An alternative is an in-place fix on a caller-owned mutable plane.
- This belongs with item 2: the scratch variant takes `&DevelopOptions`, and both change the
  same function. Plan one spec for the options signature and a second for the scratch,
  or one `[M]` spec. The design session decides.

**Item 4: plausible from the code. Not measured here.** `crop_orient_normalize_into`
(`src/develop.rs:559`) loops in output order and gathers each source pixel through
`crop_source_coords`. For Orientation 5–8 that is a column-strided read across a 47 MP
plane, which is the cache pattern they describe. A blocked loop (tiles of about 64×64) keeps
the arithmetic the same, so byte-identity is easy to prove. Item 2's
`apply_orientation: false` hides the cost for their app but not for other callers. Worth
doing as an `[S]`, once a `just bench` or criterion baseline exists. The repo has no
benchmark harness today, and their timings come from their own spike.

**Item 5: already planned. It is PROJ-004 instrument-layer Stage A.** The locked rules
(future-plans § PROJ-004) say: select on `NewSubfileType` per DEC-012; return `&'a [u8]`
tied to the **input** lifetime, not `&self` as `Container::payload` does today; the host
decodes and irradiance never does. Their acceptance (exiftool cross-check, typed bounds,
fuzz) fits. monorail-000 is the **named caller** the brief's "wait for a named caller" risk
asks for.

**Item 6: smaller than it looks, but needs a decision.**
- `develop_into` has **no CFA branch**. Nothing in `src/develop.rs` reads `cfa` or
  `photometric`, so routing to demosaic happens in the caller. A caller can already pass a
  CFA-declared plane to the mono path today. Assumed from reading the code: whether
  `check_required_processing(…, DevelopPath::Mono)` or `resolve_geometry` refuses a CFA
  sensor needs one test to confirm.
- The real gap is **levels**. The mono path uses the scalar first value, `sensor.black_level`
  (`resolve_levels`, `src/develop.rs:323`). The per-position arrays already exist on `Sensor`
  as `black_levels` / `white_levels` (SPEC-026), and so does `black_level_repeat_dim`. They
  are not used on the mono path.
- What needs deciding: whether "treat this CFA sensor as mono" is a field on `DevelopOptions`
  (item 2) or a separate entry point, and how `BlackLevelRepeatDim` maps onto the CFA
  pattern's positions. Fits as an `[S]` spec after the options struct.

## Proposed handling (none started)

All of it goes to **PROJ-004** (instrument layer and consumer integration). monorail-000 is
the Monochrom-app consumer that brief already names.

| Item | Where | Size | Gate |
|---|---|---|---|
| 7 changelog | none, already on main | none | reply only |
| 5 previews | Instrument Stage A (planned) | `[S]` | none; first spec in Stage A |
| 2 develop options | Instrument Stage C (planned) + `apply_orientation` | `[M]` | DEC (options struct shape) |
| 3 caller scratch | Stage C, beside item 2 | `[S]` | after item 2; must cover the FixBadPixels plane copy |
| 6 CFA override | Stage C, as an options field | `[S]` | after item 2; small DEC on BlackLevelRepeatDim mapping |
| 4 orientation perf | Stage C, or a perf stage | `[S]` | a benchmark baseline first |
| 1 unit unpack | new stage: parallel-friendly decode | `[M]` | DEC; uncompressed row bands are the real win (Compression 7 files can't be split, see the reply below); after SPEC-058 |
| 1b Compression 7 decode speed | same stage, or a perf stage | `[M]` | profile `src/ljpeg.rs` first; stays bit-exact vs dnglab |

Suggested order, if the maintainer agrees: Stage A (item 5) → Stage C (items 2, 3, 6, 4) →
unit unpack (item 1, once SPEC-058 ships). Item 1 is their biggest win for open time, but it
is the only one that needs a spike before design.

## Requester reply (2026-11-12)

They accept the order (preview slices → develop options → unit-level unpack).

**The Compression 7 file is the Pentax K-3 Mark III Monochrome**, the same model as our
tier-B `PENTAX-K3III-MONO/K3III.DNG`, so we don't need their copy. They checked it, plus a
second 5984×4000 12-bit Compression 7 file of theirs, and found one strip, a single
sequential scan, Nf=2, no DRI and no RST markers.

**Checked on our corpus file (claude, 2026-11-12).** `irr ifd` gives one strip
(`rows_per_strip 4224`, 31,047,287 bytes) and no tile tags. Walking the strip's JPEG markers
gives SOI, SOF3 (P=14, Y=4224, X=3152, Nf=2), DHT, APP0, SOS, and then no `0xFFD0-D7` and no
`0xFFDD` anywhere in the entropy-coded data (stuffing rules these out as data), ending in
EOI. Their reading holds.

**The SPIKE-001 files (sent by the requester 2026-11-12) are already our tier-B corpus**,
in the same directory, `$IRRADIANCE_CORPUS_DIR`:

| Their figure | File | sha256 prefix |
|---|---|---|
| 703 ms Comp-7 unpack | `PENTAX-K3III-MONO/K3III.DNG` | `1afc1071451a3a37` |
| 610 ms Comp-7 unpack | `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` | `3fb74d1fc402daaa` |
| 240 ms unpack / 858 ms open (main file) | `LEICA-Q2-MONO/L1021223.DNG` | `5957a4ca64b87b81` |

They asked that the files be used in place and never copied into a repo, which is
already the rule (DEC-003). Any benchmark for items 1 and 4 should use these three files,
so our numbers and theirs can be compared.

**What this changes for item 1:**
- **Unit-level unpack is uncompressed-only** in practice: row bands inside a strip. That is
  the 240 ms out of their 858 ms open on the Q2M. Strip, tile and restart units stay in the
  design for generality (tiled Pixel files after SPEC-058), but no held Compression 7 file
  has any to split on.
- **Compression 7 speed is a separate item:** faster single-stream Huffman decode and
  prediction in `src/ljpeg.rs`. It must stay bit-exact against dnglab, as SPEC-044's oracle
  requires. It is new, unrequested as such, and unplanned; it needs a profile first. The
  corpus inventory spike for item 1 shrinks to one question: is the uncompressed row-byte
  alignment (`width · bits % 8 == 0`) true for every held uncompressed file?

**Item 3, confirmed for their files:** their target files carry FixBadPixelsConstant
(OpcodeList1) and WarpRectilinear (OpcodeList3). So today's develop peak memory is the raw
plane + a full plane copy + two active-area warp buffers. They'll budget for that until
caller scratch lands.

## onyx DEC-010 (relayed 2026-11-12)

The onyx build asked that irradiance's maintainer be told: **onyx's DEC-010 keeps the raw
ADU histogram, the Zone 0–X histogram and all optical-black reductions in onyx**, in its
`mono-instruments` crate. irradiance stays "sensor data in, pixels + metadata out". The
decision is in onyx-000/monorail-000, file
`decisions/DEC-010-mono-instruments-owns-the-raw-histogram-and-optical-black-re.md`, on
the branch `claude/proj-001-frame-decisions-7341cc`. That branch was **not yet merged**
when this was recorded.

Effect here:
- None of the seven requests above is affected. None asked for a histogram or optical
  black.
- PROJ-004's planned instrument work shrinks:
  - Stage A's histogram/zones lose this caller;
  - Stage B (optical black) loses its only named caller and is **parked**.
  - Both are recorded in the PROJ-004 brief and `docs/future-plans.md`.
- What onyx still expects from irradiance is sensor description. Most of it already exists:
  - the plane: `plane::unpack_into`;
  - levels: `Sensor.black_levels` / `white_levels`;
  - `ActiveArea`, `DefaultCrop`, `Orientation`.

  The **optical-black rectangle** is not exposed as such. Today it has to be worked out
  from `ActiveArea` and the plane width (on the Q2M, columns 8392..8424). If onyx wants it
  named, that's a small, separate request. It is **not filed**. (Filed 2026-11-12 as
  item 8, below.)

## Item 8: pass `MaskedAreas` through (onyx request 5, relayed 2026-11-12, low priority)

onyx filed the request the section above anticipated, as **its request 5**, marked low
priority. It is **item 8** here, because 5 is already the preview slices. onyx asks
irradiance to **pass `MaskedAreas` through when a file has it**. Its note says irradiance
does **not** need to name the strip itself: no "left / right / top" labels and no
inference from `ActiveArea`. Rectangles in, rectangles out; onyx does the optical-black
reductions (onyx DEC-010).

Checked against `origin/main` (2026-11-12):
- **Not read today.** `MaskedAreas` (DNG tag 50830, SHORT or LONG, count 4·N, each
  rectangle top, left, bottom, right in raw-plane coordinates) appears nowhere in `src/`.
  `Sensor` has `active_area` but nothing for masked areas.
- **No held DNG carries it.** `exiftool -MaskedAreas` over the tier-B corpus finds none;
  9 of the 11 DNGs carry `ActiveArea` and none `MaskedAreas` (the Q2M's masked columns 8392..8424 are
  implied by `ActiveArea`, not tagged). So the oracle must be writer-built fixtures from
  the SPEC-057 test-only DNG writer, plus a hand-checked exiftool read of one fixture.

Proposed shape (for the spec to settle):
- A new `Sensor` field, for example `masked_areas`, holding the rectangles verbatim and in file
  order. `Sensor` is `#[non_exhaustive]` (DEC-052), so this is additive (0.2.x).
- Validation in the spirit of `ActiveArea`: count a multiple of 4, each rectangle well-formed
  (top < bottom, left < right) and inside the plane. A malformed tag is a typed error, not a
  silent `None` (DEC-012). Whether it may overlap `ActiveArea` is a spec question. The DNG
  spec says masked areas are outside it, but irradiance should report, not police, unless
  develop ever uses them.
- Allocation: at most a handful of rectangles. A fixed-capacity array keeps `Sensor`
  allocation-free; the spec picks the cap and the refusal past it.
- No develop change. irradiance does not subtract optical black (Stage B stays parked).

Size `[XS]`/`[S]`. It is not framed. It fits the 0.2.x maintenance stage (STAGE-014) or
beside Stage C, whichever is open when it is picked up.

## Resolution

Maintainer decisions (2026-11-11): use the suggested order (Stage A → Stage C → unit unpack). Release cadence is open: 0.2.x additive releases or a grouped 0.3.0, whichever is convenient. The requester has no timeline, so this runs on irradiance's own schedule. Questions sent back: which camera the 26.6 MP Compression 7 benchmark used, and whether they can share that file.

| Item | Stage | Spec | Notes |
|---|---|---|---|
| 5 preview slices | STAGE-018 (shipped 2026-11-12) | SPEC-062 shipped | `Container::previews()`; build PR 208, verify PR 209 (punch list, 2 test gaps fixed); in `[Unreleased]`, ships with the next 0.2.x |
| 2 develop options (framing, apply_orientation) | STAGE-019 | SPEC-063 `[M]` (shipped) | shipped 2026-11-12 (DEC-058); `develop_with_into`, unreleased; FullActiveArea on warped frames enabled by SPEC-064 |
| 2 develop options (warp_alpha / lerp) | STAGE-019 | SPEC-064 `[M]` (shipped) | shipped 2026-11-12 (DEC-059); `warp_alpha` in [0, 1] over `WarpRect::lerp`; FullActiveArea develops all three Q2M frames at every alpha (measured scale [0.8998, 0.99925], corner pull 506.1 px); over-unity lenses (Pixel 3a) refused typed; unreleased |
| 3 caller scratch | STAGE-019 | SPEC-065 `[S]` (frame) | after SPEC-063; covers the FixBadPixelsConstant plane copy |
| 6 CFA-as-mono | STAGE-019 | SPEC-066 `[S]` (frame) | after SPEC-063; per-position levels |
| 4 orientation perf | STAGE-019 | SPEC-067 `[S]` (frame) | independent; byte-identical, timed before/after |
| 8 `MaskedAreas` pass-through (onyx request 5) | unframed; STAGE-014 or beside Stage C | none yet | filed 2026-11-12, low priority; writer-built fixtures, no held file has the tag |

*Fill in as more stages and specs are minted.*

Status stays `open` until items 1, 2, 3 and 5 ship.
