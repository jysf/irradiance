---
source: "monorail-000"
captured_at: 2026-11-11
captured_by: claude
status: open
---

# monorail-000 (macOS monochrome app): unit-level unpack, develop options, caller scratch, orientation speed, previews, CFA override

## The issue

Upstream requests from **monorail-000**, the macOS monochrome app (the "Monochrom app" named
as a PROJ-004 beneficiary). The maintainer pasted them. Evidence is their SPIKE-001: M4 Pro,
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
| 1 unit unpack | new stage: parallel-friendly decode | `[M]` | spike (strip/tile/restart inventory) + DEC; after SPEC-058 |

Suggested order, if the maintainer agrees: Stage A (item 5) → Stage C (items 2, 3, 6, 4) →
unit unpack (item 1, once SPEC-058 ships). Item 1 is their biggest win for open time, but it
is the only one that needs a spike before design.

## Resolution

*Fill in as stages and specs are minted.*

Status stays `open` until items 1, 2, 3 and 5 ship.
