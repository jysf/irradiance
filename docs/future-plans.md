# irradiance — future plans

The forward direction for the RAW engine, beyond what ships in the current
wave. This is a **living, intentionally-incomplete** document: it is
reconstructed from `PROJ-001`'s brief roadmap and stage structure, and the
maintainer has further targets that still need to be recovered from past
planning sessions and folded in here (see [Open questions](#open-questions--to-revisit)).

> **Scope of this doc.** This is the *app's* forward plan. It is not the
> spec-driven template's own roadmap — that lives in
> [`ROADMAP.md`](ROADMAP.md) and governs the workflow, not irradiance.
> The authoritative, machine-readable plan is the `roadmap:` block and
> "Enables" section of the active project brief
> ([`projects/PROJ-001-monochrome-dng-develop/brief.md`](../projects/PROJ-001-monochrome-dng-develop/brief.md));
> this doc is the readable narrative over it.

_Last reconstructed: 2026-10-27; PROJ-003/004/005 and sequencing added 2026-11-02;
reconciled to current status (PROJ-001 published, PROJ-002 colour shipped, PROJ-003
closing) and the post-PROJ-003 sequence 2026-11-06; tooling-landscape survey
refreshed 2026-11-24 (DNG 1.7 / JPEG XL, new entrants, second-oracle options); status,
near-term and sequencing reconciled to 0.2.6 and the PROJ-001..006 state, and an
adoption-push candidate drafted, 2026-11-24; status re-reconciled to 0.2.7, the PROJ-005
close and SPEC-091, and a DNG-writer Later section added, 2026-11-25._

## The through-line

There is no permissively-licensed, maintained, pure-Rust RAW decoder. Every
mature one (dcraw / LibRaw / rawspeed and their descendants) carries GPL, LGPL
or AGPL, and there is no permissive option in Go, Java, Scala or Elixir either.
irradiance exists to be that missing primitive: a **clean-room, patent-safe,
pure-Rust RAW → image** engine the ecosystem can actually adopt. Its first
consumer is **crustyimg** (whose "RAW" support today is really the camera's
embedded JPEG); the maintainer's own driver is developing Leica Q2 Monochrom
files reproducibly into web assets.

> **Note (2026-11-24).** The first sentence is no longer true of the category: three
> permissively-licensed pure-Rust DNG crates (gamut-dng, filmcraft-dng, flashraw)
> appeared in 2026. See `docs/rust-ecosystem-and-semver-decisions.md` §1.1 and the
> [2026-11-24 refresh](#refresh-2026-11-24). What still sets irradiance apart is the bar
> below, not the licence alone.

Every wave is held to the same bar: **independent-oracle-verified correctness,
no panics on hostile input, deterministic output, and a provenance-ledger entry
for every algorithm and decoder** naming its source and that source's licence.

## Where we are now

Status as of 2026-11-25 (`origin/main` at `efac6b6`). Each project's `brief.md` is the
source of truth.

- **Release.** **`0.2.7`** is the live crate on crates.io (published 2026-11-25). It
  adds the public DNG writer. `0.2.0` (2026-11-11) was the first release with demosaic,
  colour and SOF-3; `0.2.1`..`0.2.6` followed as correctness and refusal releases.
- **`PROJ-001 — Monochrome DNG develop`** — **shipped** (2026-11-11).
- **`PROJ-002 — Bayer develop (colour)`** — **active, with one stage open.** Demosaic
  (Hamilton-Adams, DEC-031), the colour transform, dual-illuminant interpolation,
  CameraCalibration/AnalogBalance (SPEC-085) and the refusal work of STAGE-027..029 have
  shipped. Only **STAGE-008** is open, holding the ColorChecker ΔE oracle (SPEC-035),
  which is **blocked on a physical chart capture**.
- **`PROJ-003 — Compressed planes (SOF-3)`** — **shipped** (2026-11-06).
- **`PROJ-004 — Instrument layer and consumer integration`** — **shipped** (2026-11-20):
  the test-only writer, tiled reader, GainMap, preview accessors and develop options.
  STAGE-004 (crustyimg integration) was cancelled here; crustyimg tracks the
  `raw-develop` integration in its own STAGE-056.
- **`PROJ-005 — Public DNG writer for the onyx Darkroom`** — **shipped** (2026-11-25,
  PR 350). The writer (`irradiance::dng::write`, feature `write`) is in `0.2.7`; its
  follow-ups are under [Later — the DNG writer](#later--the-dng-writer-after-proj-005).
- **`PROJ-006 — Spectral monochrome mixer`** — **STAGE-031 active.** SPEC-091 (the
  white-balanced linear RGB plane, DEC-079) shipped (PR 351); SPEC-092 and SPEC-093 are
  at frame.
- **Wasm.** A `wasm32` build gate with a red-proof shipped (SPEC-096, STAGE-033).

## Near-term: the mixer, and a decision on the chart

- **Design SPEC-092** (PROJ-006 / STAGE-031): the mixer at sensor geometry, then
  SPEC-093 (the developed mix).
- **Decide STAGE-008**: capture a ColorChecker or cancel the stage and close PROJ-002.
  The options are in the 2026-11-24 landscape-refresh PR; nothing is decided yet.
- **Audit leftovers.** Ranks 8, 9 and 11 of the 2026-11-21 parsed-but-unvalidated audit
  are still unscheduled (status table in that file, 2026-11-24). They are small and can
  ride along with whichever spec touches the code.

## PROJ-002 — Bayer develop (colour) · *active — only STAGE-008 (ΔE oracle, SPEC-035) open, blocked on a chart*

The colour pipeline that PROJ-001 deliberately omits (a Monochrom sensor has no
colour filter array, so there is nothing to demosaic). Planned content:

- A **CFA model** (Bayer mosaic description and indexing).
- A **clean-room Malvar–He–Cutler demosaic** — implemented from the published
  algorithm, not ported from any copyleft decoder. *(Superseded: DEC-031 replaced it with
  Hamilton-Adams on patent grounds; that is what shipped.)*
- **Colour matrices** — camera-native to a working/display colour space.
- A **ColorChecker ΔE oracle** — the first *fully independent* correctness
  check. PROJ-001's oracle is single-sourced (both the plane checksum and the
  reference render come from rawler via dnglab), so a tolerance test could pass
  while both are wrong together; a ΔE measurement against a physical
  ColorChecker target breaks that dependency.

Rough size signal from the brief: the Bayer path is ~1,300–1,600 lines vs.
~550–700 for the monochrome path. **Status (2026-11-06):** STAGE-006/007 and the
STAGE-008 colour transform + dual-illuminant interpolation (SPEC-033/SPEC-034) have
shipped; `DEC-038` corrected the ColorMatrix-only white-balance path. The ColorChecker
ΔE oracle (SPEC-035) is the one spec left and is blocked on a physical chart; a
colour-geometry spec (crop/orient + `ActiveArea`-relative mosaic phase, from the external
0.2.0 review §3B) is a likely addition. A parallel session owns the live colour thread.

## Sequencing — `0.2.7` is live with the writer; mixer next, adoption after

The earlier order (close PROJ-003, review, cut `0.2.0`, then PROJ-004/006/adoption) is
done through PROJ-005: `0.2.0` was cut on 2026-11-11, PROJ-004 shipped on 2026-11-20,
and PROJ-005 shipped the public writer in `0.2.7` on 2026-11-25.

**Current order (2026-11-25; nothing here is a new decision):**

1. **PROJ-006: STAGE-031** (SPEC-091 shipped; SPEC-092 → 093 next). onyx has it on its
   later horizon, after its Darkroom v1.
2. **PROJ-002: STAGE-008.** The maintainer chooses: capture a chart, or cancel the stage
   and close the project.
3. **Then, order TBD:** the **adoption push** (unminted; see
   [the candidate below](#project-candidate-not-minted-adoption-push)), a coverage
   decision on JXL-compressed DNG and 3-sample LinearRaw (ProRAW) (see the
   [2026-11-24 refresh](#refresh-2026-11-24)), and the audit leftovers.
4. Any M10 / M11 / Q3 Monochrom conformance claim still waits for real Tier-B files and
   their rows in `docs/conformance-matrix.md`.

## PROJ-003 — Compressed planes: lossless JPEG (SOF-3) · *shipped 2026-11-06*

The compressed-plane decoder. The Q2M is uncompressed so PROJ-001 does not need
it, but it is the **keystone shared by roughly eight vendor decoders** — most
non-trivial RAW files carry a losslessly-JPEG-compressed plane (e.g. the Leica M
Monochrom Typ 246 is big-endian *and* JPEG-compressed). Landing SOF-3 is what
turns irradiance from "develops one uncompressed file" into "develops the files
people actually have."

- **`Compression 7` and SOF-3 are one work item**: `7` is the DNG container tag,
  SOF-3 is the coding. Core (Huffman, predictors 1–7, point transform,
  multi-component CFA interleave, restart markers, 8–16-bit) plus the DNG binding
  (tiles/strips, big-endian, feeding the existing plane path).
- **Oracle:** bit-exact vs `dnglab --raw-checksum` on held files, plus an analytic
  encode→decode round-trip. Patent-check the algorithm at frame time (DEC-031);
  `oxideav-mjpeg` (MIT) and `photocraft-raw` are cross-check references only.
- **`OpcodeList2` `GainMap`** (flat-field / lens vignetting): parse, then apply
  or bypass. Not parsed today (it arrives as `Opcode::Unknown`). dnglab applies no
  DNG opcodes (DEC-024 F2), so **analytic fixtures only**, each with a red-proof.
  First check which held files carry `OpcodeList2` at all (the spike).
  **Deferred (2026-11-06):** `GainMap` was scoped out of STAGE-011 (the binding stage);
  it lands with PROJ-004's opcode-inventory spike, which already inventories
  `OpcodeList1/2/3` across the held files.
- **Out of scope:** lossy-DCT DNG, Nikon NEF-lossless (a different Huffman-
  difference scheme + linearization curve), Pentax PEF (`65535`).
- **Targets:** Leica M Monochrom Typ 246 and Pentax K-3 III DNG (held, both
  currently `UnsupportedCompression`); M10 / M11 / Q3 Monochrom and the Nikon
  D750 DNG **only once files are held**.

## PROJ-004 — Instrument layer, opcode coverage and DNG writing · *shipped 2026-11-20 (the public writer moved to PROJ-005)*

> **Updated 2026-11-11.** PROJ-004 now exists as `projects/PROJ-004-instrument-layer-and-consumer-integration/`.
> Its framed stages come before the instrument layer below:
> 1. **STAGE-013** — test-only DNG writer (SPEC-057, DEC-055).
> 2. **STAGE-015** — tiled DNG reader (SPEC-058).
> 3. **STAGE-016** — OpcodeList2 GainMap (SPEC-059/060).
> 4. **STAGE-017** — public writer behind a `write` feature (SPEC-061).
> 5. **STAGE-004** — crustyimg integration, moved from PROJ-001.
>
> GainMap is no longer blocked on a file. The Pixel 4a, 3a and 7 Pro DNGs in the
> raw.pixls.us mirror carry mandatory 30×40 GainMaps, but they are tiled. The public
> DNG writer, previously "later", is now STAGE-017.


The telemetry and calibration surface the Monochrom app's `[ I. SENSITOMETER ]`
and `[ II. OPTICS & PLANE ]` benches need (companion docs:
`~/PSeven/experiments/monochrom-app/docs/irradiance-apple-app-design.md` and
`…-deep-dive.md`). All of it stays zero-I/O and caller-allocated.

> **Update 2026-11-12: onyx DEC-010 moves the histogram and optical black out of
> irradiance.** The Monochrom app (onyx-000, codename monorail-000) decided that its
> `mono-instruments` crate computes the raw-ADU histogram, the zones and every
> optical-black reduction on the CPU, over a `PlaneView`. irradiance supplies only sensor
> description: the plane, black and white levels, active area, the optical-black
> rectangle, crop and orientation.
>
> As a result:
> - Stage A here is cut down to the preview accessors (STAGE-018 / SPEC-062).
> - The histogram/zone rules below stay only as a reference, in case crustyimg STAGE-062
>   asks.
> - Stage B is parked unless irradiance's own develop needs optical-black-based black
>   subtraction, which is onyx DEC-010's revisit condition.

**Stage A — Forensics (read-only; no new decoder; works on the uncompressed Q2M):**
- Preview-slice accessors on `Container<'a>` → `&'a [u8]` + dimensions for the
  embedded preview SubIFDs (select on `NewSubfileType`, per DEC-012 /
  `docs/measured-q2m-dng.md`) beside the `LinearRaw` sensor IFD. The host decodes
  the JPEG; irradiance never does.
- Raw-ADU histogram + zone kernel (rules below).
- `collect_bad_pixels` — **only if the spike's opcode inventory justifies it**
  (today only `FixBadPixelsConstant` is parsed, and SPEC-017 recorded zero hits on
  real Q2M frames). Must share one code path with `apply_fix_bad_pixels_constant`.

**Stage B — Optical black (spike-gated, see below):** analyzer over the masked
columns (`ActiveArea` right edge → sensor width) and an opt-in black mode
(`NominalTag` default / `GlobalOpticalBlack` / `RowCorrelatedOpticalBlack`).

**Stage C — Geometry:** `WarpRect::lerp(alpha)`, two framing modes, and the
displacement-field sampler.

### Locked API and spec rules (maintainer, 2026-11-02)

- Slice accessors return `&'a [u8]` tied to the **input** lifetime, not `&self`
  (unlike today's `Container::payload`).
- Histogram: caller-allocated `bins: &mut [u32; 65536]` (or a `len == 65536`
  slice). Pixel paths stay `u16` (DEC-016 / DEC-018); `f32` is telemetry-only.
- Bad pixels: allocation-free iterator or caller `&mut [BadPixel]`.
- Optical-black row offsets: caller `&mut [i32]`, one per row. `NominalTag` stays
  the default so every dnglab-compared oracle stays green; the new modes are tested
  only against an **analytic synthetic banded-frame oracle with a red-proof**.
- Warp field: caller `&mut [(f32, f32)]` over an N×M grid; the maximum corner
  pull is **measured on the Q2M corpus, not assumed** (the earlier "503.8 px" is
  unrecorded).
- Counts: `sub_black_count` is strictly `< black_level`; `clipped_count` is
  `>= white_level` (samples above are legal, DEC-029). Zones `0..=X` are exact
  integer-rounded linear stops below `white_level` relative to `black_level`.
  Provenance rows in `docs/provenance-ledger.md` for the zone definition, the
  Engineering-DR formula and the row-black estimator.
- **Geometry modes — two only:** `StandardCrop` (`DefaultCrop`, 8368×5584 on Q2M)
  and `FullActiveArea` (`ActiveArea`, 8392×5632, recovering **536,832**
  photosites). `UncroppedOverscan` is **dropped from v1**: no undefined-pixel
  mask, no overscan canvas.

### Pin at design time (gaps in the locked rules)

1. **`FullActiveArea` + `lerp` validity is an assumption until measured.** It
   holds only if the backward radial scale `f(r) = kr0 + kr1·r² + …` stays `<= 1`
   across the whole normalised domain, including the corners. `lerp` gives
   `f_α = (1−α) + α·f`, so `f <= 1` everywhere ⇒ `f_α <= 1` for every α. The
   sampler must check this on the Q2M, and the applier must still return a typed
   error (never read out of bounds or panic) for a lens where `f > 1`.
   **Measured 2026-11-12 (SPEC-064, DEC-059):** every held Q2M frame has
   `f ∈ [0.8998, 0.99925]`, decreasing to the corner; the corner pull is 506.1 px on
   L1021223 (not the unrecorded 503.8). The Pixel 3a is a real `f > 1` lens
   (max 1.0015, 439 out-of-extent pixels at α = 1). The rule adopted is
   `0 ≤ f_α ≤ 1` on `r² ∈ [0, 1]` with the optical centre inside the image.
2. **Zone 0 vs `at_black`.** "`at_black_count` *or* Zone 0 covers `== black_level`"
   is ambiguous. Specify one partition (`< black`, `== black`, zones…, `>= white`)
   so the counts sum to the in-area total, and test that invariant.
3. **Bin-count width.** `u32` bins are safe to 4.29 G samples (a 47 MP frame is
   fine); state it and the histogram's `ActiveArea` clip rule.

### Spike — before PROJ-004 Stage B (and Stage A's `collect_bad_pixels`)

A bounded `SPIKE-NNN` on the held Q2M files (and the rest of the corpus):

- Inventory `OpcodeList1/2/3` across **all held files** with `irr ifd`; record
  whether `FixBadPixelsList`, `GainMap` or `BlackLevelDeltaV/H` appear anywhere.
  (`BlackLevelDeltaV/H` is the DNG spec's own per-row/column black — if Leica
  writes it, use it before inventing a parallel mechanism; it is unparsed today.)
- Verify columns `ActiveArea.right..Sensor.width` (`8392..8424` on Q2M) are truly
  masked optical black; measure their row-to-row σ; and test whether a per-row
  median over ~32 columns **removes banding or injects its own row noise** (and
  whether smoothing in y is then needed — which contradicts "zero blur").

## PROJ-006 — Post-WB spectral monochrome mixer (Bayer → mono) · *STAGE-031 active — SPEC-091 shipped 2026-11-25*

> **Renumbered 2026-11-21 (was PROJ-005).** This plan was never created under
> `projects/`, so when the public DNG writer left PROJ-004 the ID tool gave it
> PROJ-005. The mixer is PROJ-006 by the maintainer's call. The ID tool reads only
> `projects/`, so create this project before any other, or it will hand PROJ-006 out
> again.
>
> **Created 2026-11-23** as `projects/PROJ-006-spectral-monochrome-mixer/` (proposed). The
> brief is now the source of truth; the mix space (white-balanced camera RGB here, or a
> linear working space) is reopened there as the first STAGE-031 design decision.

A scene-linear monochrome formation for **colour** Bayer DNGs (Leica Q2/Q3,
M10/M11, SL2/SL3) — the PROJ-002 bridge for the monochrome app.

- **Framing:** **post-demosaic, post-white-balance** camera-RGB → monochrome. It is
  *not* "pre-demosaic": a full-resolution mono needs the other channels
  interpolated at each site, and the weights are only meaningful on white-balanced
  RGB, hence the dependency on SPEC-033 (and SPEC-034 for the matrices).
- **Weights:** integer / fixed-point rational, `u16` pixel path (DEC-018), so the
  oracle stays bit-exact.
- **Highlights:** deterministic **clip-aware weight renormalisation** only — a
  channel clipped at `>= white_level` is dropped and the remaining unclipped
  weights are renormalised to sum to 1. **No spatial highlight reconstruction**
  (PROJ-002 keeps that out of scope as an unbounded tuning loop).
- **Presets** (Schott Yellow 495, Green 530, Orange 550, Red 590, Ortho Blue 470):
  each recorded in `docs/provenance-ledger.md` as an **approximation**, because a
  DNG carries no sensor spectral sensitivity.

## Later — the DNG writer (after PROJ-005)

> **Added 2026-11-25, at the PROJ-005 close.** PROJ-005 shipped the public monochrome
> writer (`irradiance::dng::write`, feature `write`) in irradiance 0.2.7: one `u16`
> plane, uncompressed, one strip, little-endian, with levels, opcode lists, an EXIF
> block, XMP, descriptive tags, rendering hints and an optional caller JPEG preview.
> Nothing below is framed. Each item waits for the trigger named on it.

**Checks for onyx, at its Darkroom v1 integration** (not specs):
- Open written files (each framing, with a preview) in Lightroom Classic, Capture One
  and Windows Explorer, and record the result. Until then DEC-082's confidence stays 0.8.
- Write one real export with `diamond-exif`'s EXIF block (its SPEC-025), to test the
  DEC-072 base-offset contract with the real builder, not a hand-built block.
- Ask whether `mono-core` already builds `WriteOptions` from a `Sensor`.

**Extensions, in value order, each on a consumer's request:**
1. **`WriteOptions` from a `Sensor`.** *Trigger:* onyx says it would use it. A small
   spec: `WriteOptions::from_sensor(&Sensor, Framing)` filling geometry, levels and opcode
   lists only, with a DEC for the policy calls (white level when absent,
   `BlackLevelRepeatDim` narrowing, masked areas only at full sensor, orientation). Ship
   it in whichever release comes next.
2. **Lossless compression (Compression 7) output.** *Trigger:* a consumer asks about file
   size (a Q2 Monochrom is 95 MB uncompressed). Patent-clean (DEC-035), reuses the SOF-3
   decoder as its oracle and the test-only encoder as a start. Not deflate: it needs
   zlib, a new dependency.
3. **Tiled output.** *Trigger:* very large planes, or parallel writes. The reader already
   reads tiles (STAGE-015).
4. **CFA planes with their colour tags.** *Trigger:* crustyimg's RAW-in/DNG-out, or the
   Sony ARW wave wanting ARW-to-DNG. This makes the writer a permissive raw-to-DNG
   converter and is a project of its own, not a spec.
   *Framed 2026-11-27 as PROJ-007 (proposed), triggered by the maintainer's "turn any
   photo into a RAW" demo; it also takes in items 2 and 3 (Compression 7, tiles).*
5. **Float or demosaiced LinearRaw RGB output.** *Trigger:* an HDR-merge or astro-stacking
   consumer (the parked scientific linear-raw idea).

**Not planned:** big-endian output, encoding a preview inside irradiance (the
caller-encoded preview is the boundary), and editing and re-saving an existing DNG.

**Hygiene patch, accepted at the PROJ-005 close** (signals
`fuzz-targets-without-a-ci-smoke-job`,
`ljpeg-private-doc-links-break-the-private-items-doc-build`): CI fuzz smoke jobs for
`develop` and `demosaic` plus a `just fuzz-demosaic` recipe, and the 13 private `ljpeg`
doc links. One `just new-patch`; neither touches the writer.

## Project candidate (not minted): adoption push

> **Draft, 2026-11-24.** No PROJ ID is minted and nothing is committed; the maintainer
> decides whether this becomes a project and where it goes in the order. It builds on
> `DEC-028` (publish so others can depend on the crate) and on
> `docs/rust-ecosystem-and-semver-decisions.md` §1 (2026-11-12).

**Thesis.** irradiance is on crates.io, but nobody looking for DNG in Rust finds it,
and its first pitch ("the only permissive pure-Rust RAW crate") stopped being true when
gamut-dng, filmcraft-dng and flashraw appeared. The case for using it is now quality,
not the licence alone: zero runtime dependencies, no `unsafe`, panic-free under fuzzing,
oracles that are shown red, a determinism contract (`DEVELOP_VERSION`), a provenance
ledger, and refusing a file rather than guessing. The push turns that into adoption in
two moves. First, an `image`-crate decoding hook, shipped as a **separate
`irradiance-image` crate** so irradiance itself keeps zero dependencies and its semver
stays apart from `image`'s major version. Second, outreach to the projects that sit on
copyleft RAW crates for `.dng`: oculante (quickraw, LGPL) and zenraw (an AGPL wrapper
over rawler/rawloader).

**First spec it would contain: the head-to-head evidence.** Run irradiance,
gamut-dng, filmcraft-dng and flashraw over the held corpus and record plane checksums
against `dnglab`, panics under a short fuzz pass, and what each does with the Pixel
files that carry a mandatory GainMap. If irradiance is not measurably better on
correctness or safety, the pitch is weak and the adapter can wait. If it is, the
adapter spec follows: `IrradianceDecoder`, `impl ImageDecoder`,
`register_dng_decoding_hook()`, and a corpus round-trip through `image::ImageReader`
with a red-proof. The 11-12 memo's first gate (STAGE-016, GainMap) has since shipped.

**What it needs from the maintainer:**

- Whether `irradiance-image` lives in this repo as a workspace member or in its own
  repo. Either way it brings in `image` as a dependency of a sibling crate, which needs a
  DEC under `no-new-top-level-deps-without-decision`.
- A coverage decision first, or an accepted gap: many recent phone DNGs are JXL-compressed
  (52546) or 3-sample LinearRaw (ProRAW), and irradiance refuses both today. An adapter
  that refuses the files `image` users bring may make a worse first impression than no
  adapter (see the [2026-11-24 refresh](#refresh-2026-11-24)).
- The outreach itself (issues or PRs on oculante and zenraw) goes out under the
  maintainer's GitHub account, so the maintainer sends it or approves each message.
- Where it ranks against PROJ-006 and the writer follow-ups above.

## Later — more sensors and vendor containers

Accepted direction, horizon "later" in the brief:

- **X-Trans** sensors, including a **DHT demosaic**.
- **Native Fuji RAF** container parsing.
- **Nikon NEF**, including its **linearization curve**.
- **Sony ARW**, a TIFF-based container with no published spec. Its
  uncompressed and lossless-compressed variants come first, since they
  reuse the IFD parser and the SOF-3 decoder. The lossy "compressed RAW"
  block codec and the obfuscated SR2Private metadata each need a
  provenance decision first: what is publicly known about them comes from
  copyleft decoders the clean-room rule does not allow us to read. There is
  no ARW in the corpus yet, so sample files are the first step.
- Further vendor containers as demand and corpus coverage justify (PROJ-001
  explicitly fences out NEF/RAF/CR2/ARW/NRW).

## Cross-cutting commitments

These hold across every wave and constrain how the above gets built:

- **Permissive, clean-room, patent-safe.** Implemented from published
  specifications (the Adobe DNG spec carries a patent grant for compliant
  implementations), never by reading copyleft source. `cargo deny` sees only
  declared licences, so the **provenance ledger** is what makes the permissive
  claim defensible.
- **Library, not application.** No I/O, no CLI, no async runtime; zero runtime
  dependencies. `irr` stays an internal dev/oracle binary.
- **Panic-free on untrusted input**, enforced mechanically (five deny-level
  lints + a red-proof), because RAW is attacker-influenced binary data.
- **Determinism.** Byte-identical output across platforms; any future
  parallelism (e.g. a `parallel` feature) must preserve it — crustyimg's
  `build --frozen` depends on it (`DEC-002`).
- **Memory budget.** A ~47 MP develop must stay within a sane budget; a future
  `f32` develop path and/or tiling redesign are anticipated pressure points
  (`DEC-018`).

## Explicitly out of scope (rejected, not deferred)

- Interactive / per-image editing — belongs in `crustyimg-lab`, not here.
- Competing with Lightroom on render quality — an unbounded tuning loop.
- A shipped CLI product surface — `irr` is dev/oracle-only.

## Open questions / to revisit

- **Recover the fuller target list from past planning sessions.** The
  maintainer has targets beyond PROJ-002/003 and the "later" formats that are
  not yet captured here; fold them in once recovered.
- **Refresh the permissive-RAW tooling survey.** The brief's landscape survey
  dates to 2026-08-15. A periodic research pass — new crates, decoders, or
  reference implementations in this space, and anything that changes the
  build-vs-adopt calculus for a given decoder — should update both this doc and
  the relevant project brief.

### Tooling landscape (surveyed 2026-10-27)

A refresh of the brief's 2026-08-15 survey. **The headline holds: there is still
no permissive, provenance-clean, DNG-sensor-aware pure-Rust decoder that
irradiance could adopt into its dependency graph.** But the space moved in 2026
— two permissively-*licensed* pure-Rust entrants appeared, and one of them
(`photocraft-raw`) is a near-exact parallel of irradiance's own thesis. The
changes are mostly to the *oracle/cross-check* calculus, not the build decision.

> **All provenance and licence claims below are the projects' own, read off
> their repos and treated as untrusted data.** A declared licence is a cargo-deny
> fact; a clean-room claim is not verifiable from a README. "Provenance clean?"
> records what each project *asserts*, flagged where unverified — exactly the
> declared-vs-carried distinction `docs/provenance-ledger.md` exists for.

| Tool | Lang | Licence | Provenance clean? | Covers | Relevance to irradiance |
|---|---|---|---|---|---|
| **`photocraft-raw`** (storytold/photocraft #65, 2026‑10‑04) | Rust | MIT OR Apache‑2.0 (repo; crate not yet published standalone) | **Claimed clean-room, unverified** — PR states *"No code from dcraw/LibRaw/rawspeed/rawler/rawloader/darktable was read; no camera colour tables were copied"*, written from TIFF 6.0 / TIFF-EP / DNG 1.7 / T.81 Annex H | DNG (uncompressed + LJ92), CR2, TIFF/EP (NEF/ARW/PEF uncompressed+LJ92); LJ92 decoder; full develop: linearize, levels, WB, highlights, GainMap, demosaic (bilinear/MHC/AHD), cam→XYZ→ProPhoto, gamma, orientation | **The most significant entrant.** A direct parallel — same clean-room/pure-Rust/`no unsafe`/spec-derived bar, but already into PROJ-002 (MHC/AHD) *and* PROJ-003 (LJ92). Not a standalone crate (lives in an app) and brand-new, so not an adopt; but a genuine **independent clean-room cross-check** for 002/003 and a sign the approach is sound |
| **FlashRaw** (ybouane/flashraw) | Rust core (C/C++/Py/JS bindings) | MIT OR Apache‑2.0 | **Partial / unverified** — calls itself an "independent Rust implementation" from specs + corpus study, but **gives no source for its ~1,600 camera entries and ~800 colour matrices** | ~40 formats (DNG/CR2/CR3/NEF/ARW/RAF/RW2/ORF/PEF/…), demosaic, colour, ICC | Permissive licence, but the colour-matrix/camera-DB provenance is unstated — the **`adobe_coeff` trap** the ledger warns about, live. Useful as a **benchmark**; the matrices are not adoptable as clean without the author documenting their origin. No regular CI; v0.1.0 |
| **zenraw** (imazen) | Rust wrapper | AGPL‑3.0 / commercial | **No** — parses *via* `rawloader`/`rawler` (LGPL‑2.1) or `darktable` (GPL) | DNG + many via rawler; develop → sRGB | Out: copyleft licence *and* copyleft parsing backend. The `#![forbid(unsafe_code)]` wrapper does not launder the backend's provenance. (README: *"Developed with Claude… Not all code manually reviewed."*) |
| **`dng`** (grego / apertus-open-source-cinema) | Rust | AGPL‑3.0 | copyleft — **must not read** | DNG/DCP read+write, YAML dump (`dump_dng`), zero deps | Out as a dependency and off-limits to read. But it is a CLI, so it could be **run as a tool** (like `dnglab`) to give a *second independent* container/IFD oracle — see verification note below |
| **quickraw** (RawLabo) | Rust | LGPL‑2.1 | **No** — README credits LibRaw + rawloader decoding methods | decode + render raw | Out; already fenced in the ledger's standing decisions |
| **rawloader / rawler** (dnglab) | Rust | LGPL‑2.1 (rawler: a few `tiff`/`bmff`/`tags` files MIT) | **No** | ~200 / ~300+ cameras, Bayer + X‑Trans, LJPEG-DNG, CR3 | Out as a dep; `dnglab`/`rawler` remain the **oracle tool** (run, never linked). Unchanged |
| **ljpeg** | Rust | LGPL‑2.1 | **No** — docs say *"a lot of the code is used from dnglab"* | standalone LJ92 (SOF‑3) codec, `no_std` | Out: copyleft licence *and* derived from dnglab |
| **libopenraw** 0.4.x (hfiguiere) | Rust + C API | LGPL‑3.0‑or‑later (+ MPL‑2.0 `mp4parse`) | **No** | many formats | Out; also effectively stalled (0.4.0‑alpha.3, last touched 2024) |
| **oxideav-mjpeg** (OxideAV) | Rust | MIT | **Claimed clean (no porting stated; verified black-box vs `djpeg`)** | **Lossless JPEG SOF‑3**, single-component grayscale, precision 2–16, Annex H predictor reconstruction, bit-exact; predictors 1–7 | **PROJ-003 keystone, permissive at last.** First MIT-licensed SOF‑3 decoder found — a real cross-check reference. Single-component only (multi-component Bayer-in-LJ92 would still need work); not zero-dep-checked |
| **`tiff`** (image-rs) | Rust | MIT | Clean (own impl) | baseline TIFF only — *not* DNG/sensor-IFD aware | Container reference; cannot stand in for irradiance's DNG-private-tag reader |
| **kamadak-exif** | Rust | BSD‑2‑Clause | Clean | EXIF/TIFF metadata read; stale (0.5.5, 2022) | A *linkable* permissive, independent metadata cross-check (an alternative to the `exiftool` tool the metadata oracle already uses) |
| **ssimulacra2** (rust-av) | Rust | BSD‑2‑Clause | Clean (3rd-party crate) | SSIMULACRA2 metric, v0.5.1 (2024‑12, still latest) | The develop-oracle metric — **unchanged**; already a dev-only dep via `SPEC-020`/`DEC-023` |

Also seen and set aside: **rawlib** / **rsraw** (LibRaw C++ bindings, not pure Rust); **raw-files** (kuolas, embedded-preview extraction only); **oxiraw** (name reservation, no code); **motioncam-decoder** (pure Rust but only the MCRAW container).

**Build-vs-adopt implications, by planned wave:**

- **Container / TIFF / DNG parsing.** No change to build. Permissive pure-Rust parsers exist (`tiff` MIT, `kamadak-exif` BSD-2) but neither is DNG-sensor-aware; the only DNG-complete pure-Rust parsers (`rawler`, grego `dng`) are copyleft. irradiance keeps its own spec-derived reader. The one new lever: the grego `dng` crate, *run as a tool*, is a candidate second container oracle (see below).
- **PROJ-002 — Bayer / colour / ΔE.** The calculus shifts slightly. There is now at least one permissively-licensed, clean-room-*claimed* MHC/AHD demosaic in pure Rust (`photocraft-raw`; FlashRaw too, but with unestablished matrix provenance) to **cross-check** irradiance's own clean-room MHC against — an independent check beyond rawler. Neither is a drop-in dep (no-new-deps bar, and the clean-room claims are unverified). **Colour matrices remain the minefield**: FlashRaw is a concrete live example of permissive-licence-over-unstated-matrix-provenance. The ColorChecker ΔE oracle still needs its own reference values sourced with a redistribution-licence check (X-Rite publishes sRGB/Lab, but post-2014 pigment values differ and the terms are unclear); no off-the-shelf Rust ColorChecker/ΔE crate was found, though ΔE2000 math is public (CIE).
- **PROJ-003 — Lossless JPEG SOF-3.** Still nothing irradiance can simply depend on, but the keystone is **no longer unproven-in-permissive-Rust**: `oxideav-mjpeg` (MIT) decodes SOF-3 bit-exact for single-component, and `photocraft-raw` (MIT/Apache) carries a clean-room LJ92. Both are strong cross-check references and evidence the decoder is tractable clean-room; `ljpeg` (the obvious crate) stays out (LGPL + dnglab-derived). Build remains the path; the risk is lower.
- **Later formats (X-Trans / RAF / NEF).** Unchanged: build. rawler (LGPL) and FlashRaw (permissive licence, unverified provenance) claim these; no clean permissive X-Trans/DHT reference was established.
- **Oracle / verification.** This is where the landscape helps most. The single-source-oracle risk the brief flags (plane checksum *and* reference render both from rawler) can now be attacked with **independent** checks: the grego `dng` crate as a second container/IFD oracle (run as a tool; AGPL, so never read or linked), `kamadak-exif` as a linkable metadata cross-check, and `photocraft-raw`/FlashRaw as independent pixel/develop cross-checks. `ssimulacra2` (BSD-2) is unchanged.

**Suggested follow-ups** (not actioned here): consider a brief/ledger note naming `photocraft-raw` as a parallel clean-room effort to watch, and `oxideav-mjpeg` as a candidate provenance-class-4 (permissive, must-record) cross-check source for PROJ-003 — neither is adopted, both are worth tracking.

_Sources (fetched 2026-10-27, treated as untrusted): the projects' own GitHub repos and crates.io/docs.rs/lib.rs listings — `github.com/storytold/photocraft` (PR #65), `github.com/ybouane/flashraw` (+ `docs/references.md`), `github.com/imazen/zenraw`, `github.com/grego/dng-rs` · `docs.rs/dng`, `github.com/RawLabo/quickraw`, `github.com/pedrocr/rawloader` + `github.com/dnglab/dnglab`, `crates.io/crates/ljpeg`, `github.com/hfiguiere/libopenraw`, `github.com/OxideAV/oxideav-mjpeg`, `lib.rs/crates/kamadak-exif`, `github.com/rust-av/ssimulacra2`; licences cross-checked against licenses.dev / Debian copyright metadata where the repo page was thin._

#### Refresh 2026-11-24

A one-pass refresh of the 2026-10-27 table above. That table stands as written;
this subsection records only what changed and what is new. **The headline still
holds: no permissive, provenance-clean, DNG-sensor-aware pure-Rust decoder exists
that irradiance could adopt.** The movement since October is in three places:
DNG 1.7 / JPEG XL files are now common in phone output, FlashRaw shipped to
crates.io, and two new permissive entrants (`gamut-dng`, `oxideav-jpegxl`) give
the oracle and writer work more independent cross-checks.

> Same rules as the table above: every licence, provenance and capability claim
> is the project's own, read from its README, Cargo metadata or crates.io
> listing, and treated as untrusted data. "Claims" means asserted, not verified.

**Delta table**

| Tool | What changed since 2026-10-27 | Relevance to irradiance |
|---|---|---|
| **FlashRaw** (ybouane/flashraw) | Published to crates.io (`flashraw` / `flashraw-core`, first upload 2026-10-25, now 0.4.0 of 2026-11-20; MIT OR Apache-2.0), plus PyPI and npm. Adds JPEG XL DNG (Compression 52546) behind an optional `jpegxl` feature that pulls in `jxl-oxide`. README now says *"a few audited `unsafe` blocks"*. `docs/references.md` still gives no origin for its ~1,600 camera entries and ~800 colour matrices, only that they are kept "as data (with verification records)". | Unchanged verdict: no adopt, and the matrices are still not provenance-clean. But a crates.io release plus a CLI makes it a practical **second plane oracle, run as a tool** (see below). Raw-plane checksums don't touch the matrix question. |
| **oxideav-mjpeg** (OxideAV) | 0.1.8 → 0.1.16 (2026-11-08). README now claims lossless SOF-3 **and** SOF-11 (arithmetic), hierarchical mode (Annex J), and **3-component** lossless layouts as well as single-component. MIT. | Removes the October caveat "single-component only". It is now a fuller permissive SOF-3 cross-check for PROJ-003 work. Still a reference, not a dependency. |
| **kamadak-exif** | No longer stale: 0.6.2 released 2026-11-05 (was 0.5.5 of 2022). BSD-2-Clause unchanged. | Revives it as a linkable, dev-only metadata cross-check candidate. No change to the runtime-dependency position. |
| **zenraw** (imazen) | No new release since 0.2.0 (2026-04-17); repo active. README now says the `rawler` backend handles "JXL DNG" and 10-bit lossless-JPEG ProRAW, while the default `rawloader` backend does not. AGPL-3.0-only OR commercial. | Still out (copyleft and a copyleft backend). Useful only as evidence that rawler decodes JXL DNG. |
| **rawler / dnglab** | No release since 0.8.0 (2026-08-30, before the last survey). 0.8.0 added a bilinear X-Trans demosaic and new camera support. LGPL-2.1 unchanged. | Remains the primary oracle, run as a tool. Per zenraw's and FlashRaw's READMEs it decodes JXL DNG, so it can be the oracle for any future JXL work too. |
| **`dng`** (grego / apertus) | 1.6.0 dated 2026-07-28, before the last survey; nothing newer. AGPL-3.0. | No change: run-only candidate container oracle, never read. |
| **photocraft** (storytold) | Still no standalone `photocraft-raw` crate on crates.io; the RAW code lives in the app's workspace. Repo active (pushed 2026-11-24), still MIT OR Apache-2.0, still claims clean-room. | No change: a parallel effort to watch. |
| **ssimulacra2** | Unchanged (0.5.1, 2024-12-29). | No change. |

**New entrants**

| Tool | Licence (declared) | Provenance claim | Covers | Relevance |
|---|---|---|---|---|
| **`gamut-dng`** (justin13888/gamut; 1.0.0, 2026-07-21, missed by the October pass) | MIT OR Apache-2.0 | Claims "clean-slate from the official specs"; README says DNG is "Adobe DNG SDK-gated", read here as checked against the SDK | DNG 1.7.1 **encoder and decoder** over a shared TIFF/IFD core; part of a multi-format workspace | The first permissive pure-Rust DNG *writer* seen. Relevant as an independent cross-check for the DNG writer work (SPEC-061), run or, as provenance class 4, read with a ledger row. Its JXL support wraps libjxl (encode, C) and jxl-rs (decode), so the JXL path is not clean-slate. |
| **`oxideav-jpegxl`** (OxideAV; 0.0.15, 2026-11-07) | MIT | Claims clean-room from ISO/IEC 18181 and conformance fixtures, "no external codec source is consulted" | JPEG XL decoder and encoder (Modular lossless, VarDCT lossy) | Pre-1.0. A permissive JXL decoder from the same organisation as oxideav-mjpeg. See the JXL section. |
| **`jxl`** (libjxl/jxl-rs; 0.8.0, 2026-10-19) | BSD-3-Clause | The JPEG XL project's own Rust decoder; README says unsafe is limited to SIMD/performance code with non-author review | JPEG XL decode; ships in Chrome 155+ per Google's developer blog | The best-resourced Rust JXL decoder. `zenjxl-decoder` (imazen, BSD-3-Clause, 0.3.10) is a fork of it. |
| **rawshift** (justin13888; 0.1.1, 2026-05-30) | MPL-2.0 | Not stated | Small RAW crate, 95 downloads | Weak (file-level) copyleft; set aside. |
| **rraw** (rdespres87) | GPL-3.0-or-later | Decodes via `rawler` | Darkroom app (Metal/wgpu) | Out: copyleft, and its parsing is rawler. |

No new permissive pure-Rust *demosaic* crate was found.

**DNG 1.7 / JPEG XL in real files**

What vendors write (all secondary sources, none read from a file in irradiance's corpus):

- **Apple ProRAW.** iPhone Settings → Camera → Formats offers "JPEG, JPEG-XL Lossless and JPEG-XL Lossy" (juniperphoton, 2026-01-25). Capture of JXL ProRAW is reported as iOS 18 on the iPhone 16 Pro line and later (PetaPixel 2024-09-18, MacRumors); iOS 17 added JXL *viewing*. Apple's own "About Apple ProRAW" page (published 2024-04-19) does not mention JXL. One developer observed **DNGVersion 1.7 with DNGBackwardVersion 1.3**. ProRAW is widely described as demosaiced **LinearRaw with 3 samples per pixel**, for both the legacy JPEG-lossless and the JXL variants. This was not confirmed from a file here. Community reports (pixls.us, LibRaw forum) say ProRAW carries the DNG 1.6 **ProfileGainTableMap** (tag 52525), which the DNG SDK's renderer applies.
- **Samsung.** Samsung's CamCyclopedia says Expert RAW on the Galaxy S24 moved to DNG 1.7 with JPEG XL (Cloudinary blog, 2024-01-23). User reports describe the output as LinearRaw, i.e. demosaiced, not CFA (unverified). Adobe staff attribute a 2026 S26 Ultra 50 MP Expert RAW mis-rendering to camera-embedded metadata. FlashRaw's support matrix lists "Samsung Galaxy S22 (JPEG XL)".
- **Google Pixel.** No evidence of JXL. Pixel DNGs remain CFA with a GainMap opcode (see PROJ-004).
- **Dedicated cameras.** No camera vendor was found writing JXL DNG. Leica, the obvious candidate, showed nothing. Treat this as "not found", not "doesn't exist".

What irradiance returns today (read from `src/`, nothing run):

- **ProRAW and Samsung Expert RAW, JXL or not.** If the plane is LinearRaw with `SamplesPerPixel = 3`, the sensor-IFD gate in `Container::is_sensor_ifd` (`src/ifd.rs:2084`–`2091`) answers `No`, because it accepts only `SamplesPerPixel == 1`. No IFD qualifies, so the sensor lookup fails with **`Error::NoSensorIfd`** (built by `Container::no_sensor_ifd_error`, `src/ifd.rs:2124`; variant at `src/lib.rs:434`). The compression code is never inspected. So a legacy JPEG-lossless ProRAW file fails the same way, even though irradiance now decodes Compression 7. The message ("no IFD matched the sensor-plane rule …", `src/lib.rs:1293`) is accurate but doesn't say *why*: a multi-sample LinearRaw plane.
- **A CFA or 1-sample LinearRaw plane with Compression 52546** (a hypothetical JXL Bayer DNG). The gate passes. `Compression::from_code` maps 52546 to `Compression::Other(52546)`, and `plane::plane_kind` (`src/plane.rs:1161`) returns **`Error::UnsupportedCompression { compression: 52546 }`** (variant at `src/lib.rs:452`). This is the clean typed refusal the error exists for.
- **ProfileGainTableMap** (52525) is not read anywhere in `src/`, so it is silently ignored rather than refused. That matters only once irradiance renders a phone DNG, and today none reach the render path.

**A JPEG XL decoder: possible feature-gated option only**

This is framing, not a recommendation. Any JXL decode would conflict with **zero runtime dependencies** (`Cargo.toml` `[dependencies]` is empty) and would need its own DEC. At most it would be an off-by-default feature, like the provenance ledger's rule for anything outside the default build.

| Decoder | Licence | `unsafe` | Dependencies | Maintenance |
|---|---|---|---|---|
| `jxl-oxide` (tirr-c) | MIT OR Apache-2.0 | GitHub code search finds `unsafe` in about 50 files (SIMD, grid views, colour transforms) | 0.12.6: 11 non-optional direct deps (9 own sub-crates, plus `brotli-decompressor` and `tracing`); optional `lcms2` adds C | jxl-render 0.12.4 on 2026-05-29; repo last pushed 2026-08-16. Used by FlashRaw's `jpegxl` feature. |
| `jxl` (jxl-rs) | BSD-3-Clause | Some, in SIMD; non-author review policy | multi-crate workspace (`jxl_simd`, `jxl_cms`, `jxl_transforms`, …) | Active (pushed 2026-11-15); in Chrome 155+ |
| `oxideav-jpegxl` | MIT | not checked | OxideAV crate family | Active, pre-1.0 (0.0.15) |

Every option would bring `unsafe` into a crate that is `#![forbid(unsafe_code)]` (`src/lib.rs:123`), unless the decoder is out-of-tree. The only exception is `oxideav-jpegxl`, whose `unsafe` use was not checked.

**JPEG XL patent position** (not a legal conclusion). Google and Cloudinary have filed royalty-free patent declarations with ISO for JPEG XL. Microsoft holds a granted US patent (2022) on rANS refinements and has made **no ISO declaration**. JPEG XL's editor has said publicly that the patent does not affect JPEG XL; no newer status was found. Adobe's DNG patent grant covers Adobe's own DNG patents, not third-party codec patents inside a DNG. Under the "skip encumbered formats" default, JXL rates **caution: probably clean, one open question**, not clean.

**Second-oracle options** (run, never linked)

Goal: stop the plane checksum and the reference render both coming from dnglab/rawler.

- **FlashRaw CLI** (`flashraw-cli`, MIT OR Apache-2.0): the strongest new option. Independent code, permissive, wide format coverage, and it reaches raw `u16` planes, so a plane-checksum comparison avoids the matrix-provenance question. Licence-wise it could even be read (class 4), but running it is enough.
- **Adobe `dng_validate`** (already built locally at `tools/dng_sdk`, never vendored or in CI): the authority on DNG stage-2/stage-3 output. The SDK's terms keep it a local tool only.
- **`gamut` CLI** (`gamut inspect`, MIT OR Apache-2.0): independent TIFF/DNG IFD and byte accounting. A container oracle that, unlike grego `dng`, may also be read.
- **oxideav-mjpeg** (MIT): an independent check of SOF-3 plane decode only.
- **grego `dng` `dump_dng`** (AGPL): unchanged since October. Run only, never read.
- **LibRaw sample tools / darktable-cli** (copyleft): run only. They add independence but share decades of lineage with dcraw, so they are less independent than FlashRaw or the SDK.

**Candidates for future-plans** (one line each, no commitments)

- *Typed refusal for multi-sample LinearRaw*: report "LinearRaw with N samples" instead of a bare `NoSensorIfd`. **Clean** (DNG spec).
- *Accept 3-sample LinearRaw planes* (ProRAW, Expert RAW; legacy JPEG-lossless ProRAW first): a new plane shape that is already demosaiced. **Clean** (DNG spec).
- *Parse ProfileGainTableMap* (tag 52525) into a typed field without applying it. **Clean** (DNG 1.6 spec).
- *JPEG XL DNG decode (Compression 52546)* behind an off-by-default feature; needs a DEC on dependencies and `unsafe`. **Caution**: probably clean, with the open Microsoft rANS question.
- *FlashRaw CLI as a second plane-checksum oracle*, run in local verification only. **Clean** (tool).
- *`gamut-dng` as an independent read-back check for DNG writer output* (SPEC-061). **Clean** (tool, or class-4 read with a ledger row).
- *oxideav-mjpeg multi-component SOF-3 as a PROJ-003 cross-check*. **Clean**.
- *Corpus: one JXL ProRAW and one Samsung Expert RAW DNG*, to confirm the LinearRaw and sample-count claims above from bytes. **Clean**.
- *CR3/CRX, TicoRAW*: still **skip**: patent-encumbered/proprietary (unchanged).

**Open questions for the maintainer**

1. Are phone DNGs (3-sample LinearRaw, already demosaiced) in scope for irradiance at all, or do they belong to crustyimg? The answer decides whether the next three candidates matter.
2. Would you ever accept an off-by-default feature that adds runtime dependencies and `unsafe` (JXL decode), or does "zero runtime dependencies" hold for every feature?
3. Should FlashRaw CLI and/or `dng_validate` be formally added to the oracle set (a DEC), so the single-source-oracle risk is closed rather than noted?
4. Is JXL's open rANS patent question acceptable at "caution", or should JXL be treated as encumbered and skipped by default?
5. Should `gamut-dng` and `oxideav-jpegxl` be named in the provenance ledger as class-4 candidates to watch, as the October pass suggested for photocraft and oxideav-mjpeg?

_Sources (fetched 2026-11-24, treated as untrusted): crates.io API listings for `flashraw`, `flashraw-core`, `oxideav-mjpeg`, `oxideav-jpegxl`, `kamadak-exif`, `ssimulacra2`, `dng`, `rawler`, `zenraw`, `gamut-dng`, `rawshift`, `jxl`, `jxl-oxide` (incl. its 0.12.6 dependency list), `zenjxl-decoder`, `ljpeg`, `quickraw`; GitHub READMEs of `ybouane/flashraw` (+ `docs/references.md`, workspace `Cargo.toml`s), `OxideAV/oxideav-mjpeg`, `OxideAV/oxideav-jpegxl`, `justin13888/gamut`, `tirr-c/jxl-oxide` (+ code search for `unsafe`), `libjxl/jxl-rs`, `imazen/zenraw`, `storytold/photocraft`, `rdespres87/rraw`, `kamadak/exif-rs`; `github.com/dnglab/dnglab/releases`; `support.apple.com/119916`; `juniperphoton.substack.com/p/supporting-jpeg-xl-compression-in`; `petapixel.com/2024/09/18/why-apple-uses-jpeg-xl-in-the-iphone-16-and-what-it-means-for-your-photos`; `macrumors.com/how-to/iphone-16-pro-choose-jpeg-xl-proraw-formats`; `cloudinary.com/blog/samsung-now-supports-dng-1-7-including-jpeg-xl`; Samsung CamCyclopedia "JPEG XL Image Codec"; `community.adobe.com` (S26 Ultra Expert RAW thread); `discuss.pixls.us/t/looking-for-samples-smartphone-dngs-with-embedded-gainmap/27296`; `libraw.org/node/2786`; `developer.chrome.com/blog/jpeg-xl-in-chrome`; `theregister.com/2022/02/17/microsoft_ans_patent/`; arXiv 2506.05987 (JPEG XL overview)._

**Addendum: second pass, same day.** A separate research pass ran on 2026-11-24
before the refresh above had merged. Most of what it found duplicates the refresh.
The points below add to it or disagree with it. They follow the same untrusted-data
rule.

- **Adobe writes JXL DNG.** Since Lightroom 13.0 (October 2023), lossy DNG and
  derived DNGs (merge, Enhance) use JPEG XL instead of JPEG (an Adobe-verified answer on
  community.adobe.com). No camera vendor was found writing it, but Adobe software is a
  source of 52546 files.
- **iPhone 17 Pro.** Apple's iOS 26 guide lists the 17 Pro models as ProRAW devices. That
  they keep the JXL options rests on one forum post, so it is *unverified*.
- **FlashRaw provenance: the two passes disagree.** The 0.3.0 CHANGELOG (2026-11-14) adds
  a statement that decoders, codecs and the camera database were "rewritten from format
  descriptions and sample files", with values "measured from sample files or taken from
  Adobe DNG profiles". The refresh above read `docs/references.md`, which still gives no
  per-entry origin. Both may be true. Taking matrices from Adobe DNG profiles has its
  own licence question, so the verdict (matrices not provenance-clean) stands.
- **FlashRaw 0.3.1 added Nikon High Efficiency NEF**, which is TicoRAW-based.
  **Patent / proprietary: skip.** This does not affect running FlashRaw as a plane
  oracle on DNG files.
- **RustDNG** (Henry-GongZY) is a Rust translation of the Adobe DNG SDK 1.7.1 under the
  SDK's source-available licence, and it decodes JXL through jxl-oxide. **Proprietary,
  and derived from Adobe code: do not read.** Only its README was seen.
- **LibRaw's `unprocessed_raw`** (LGPL-2.1 or CDDL-1.0, run only) with black subtraction
  and scaling turned off writes the raw plane as a TIFF. That makes it a bit-exact plane
  oracle. The refresh above calls LibRaw "less independent" because it shares lineage
  with dcraw. It is still independent of **rawler**, which is the single source that
  needs replacing. rawpy (MIT, bundles LibRaw, 0.28.0 of 2026-10-31) exposes the same
  plane as a `uint16` array and is the easiest to script.
- **Choosing a JXL oracle.** rawler, FlashRaw's `jpegxl` feature and RustDNG all decode
  through **jxl-oxide**. If JXL lands, check its tiles with jxl-rs or libjxl's `djxl`,
  not with a jxl-oxide-based tool.
- **zenjxl-decoder** (imazen's jxl-rs fork) is `forbid(unsafe_code)` by default with an
  opt-in `allow-unsafe` feature. It reports 17 of 23 passing on Level 5 conformance
  (README, December 2025). Of the JXL decoders listed above, it is the only one that
  keeps `unsafe` out by default.
- **Repo location.** `github.com/justin13888/gamut` redirects to `visualcommons/gamut`.
