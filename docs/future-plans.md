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
closing) and the post-PROJ-003 sequence 2026-11-06; reconciled again to 0.2.6 and the
PROJ-001..006 state, with the 2026-11-24 landscape refresh and an adoption-push
candidate, 2026-11-24._

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

Status as of 2026-11-24 (`origin/main` at `bf0ffdd`). Each project's `brief.md` is the
source of truth.

- **Release.** **`0.2.6`** is the live crate on crates.io (tagged 2026-11-23). `0.2.0`
  (2026-11-11) was the first release with demosaic, colour and SOF-3; `0.2.1`..`0.2.6`
  followed as correctness and refusal releases.
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
- **`PROJ-005 — Public DNG writer for the onyx Darkroom`** — **proposed**. STAGE-017 is
  framed; **SPEC-061** (the sealed public `write` API) is designed and at `cycle: build`,
  ready to build. SPEC-081 and SPEC-082 are at frame.
- **`PROJ-006 — Spectral monochrome mixer`** — **proposed**. onyx answered the four
  framing questions on 2026-11-23, and **STAGE-031** is framed from those answers
  (SPEC-091 → SPEC-092 → SPEC-093, all at frame; PR 325).

## Near-term: the writer, the mixer, and a decision on the chart

- **Build SPEC-061** (PROJ-005 / STAGE-017): the public `write` feature over the
  STAGE-013 core. It is designed and runs in its own session.
- **Design SPEC-091** (PROJ-006 / STAGE-031) once PR 325 has merged: the white-balanced
  linear RGB step and the working-space DEC.
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

## Sequencing — `0.2.6` is live; writer and mixer next, adoption after

The earlier order (close PROJ-003, review, cut `0.2.0`, then PROJ-004/006/adoption) is
done up to PROJ-004: `0.2.0` was cut on 2026-11-11, PROJ-004 shipped on 2026-11-20, and
`0.2.6` is live.

**Current order (2026-11-24; nothing here is a new decision):**

1. **PROJ-005: SPEC-061**, then SPEC-081 and SPEC-082, then a release that onyx can pin.
2. **PROJ-006: STAGE-031** (SPEC-091 → 092 → 093). onyx has it on its later horizon,
   after its Darkroom v1, so it does not need to race the writer. The two touch
   different files and can run in parallel.
3. **PROJ-002: STAGE-008.** The maintainer chooses: capture a chart, or cancel the stage
   and close the project.
4. **Then, order TBD:** the **adoption push** (unminted; see
   [the candidate below](#project-candidate-not-minted-adoption-push)), a coverage
   decision on JXL-compressed DNG and 3-sample LinearRaw (ProRAW) (see the
   [2026-11-24 refresh](#refresh-2026-11-24)), and the audit leftovers.
5. Any M10 / M11 / Q3 Monochrom conformance claim still waits for real Tier-B files and
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

## PROJ-006 — Post-WB spectral monochrome mixer (Bayer → mono) · *proposed — STAGE-031 framed from onyx's answers*

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
- Where it ranks against PROJ-005 and PROJ-006.

## Later — more sensors and vendor containers

Accepted direction, horizon "later" in the brief:

- **X-Trans** sensors, including a **DHT demosaic**.
- **Native Fuji RAF** container parsing.
- **Nikon NEF**, including its **linearization curve**.
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

A bounded refresh of the survey above, with the table left as it was. The same rule
applies: **every licence, provenance and coverage claim below is the project's own,
read off its repo or registry page and treated as untrusted data.** A declared licence
is a cargo-deny fact; a clean-room claim, an `unsafe` claim or a coverage claim is
*carried*, not verified, unless marked otherwise. Nothing copyleft was read beyond
README, licence and registry metadata. The 2026-11-12 memo
(`docs/rust-ecosystem-and-semver-decisions.md` §1) already covered gamut-dng,
filmcraft-dng and flashraw 0.2.0; this records only what changed since.

**1. DNG 1.7 features in real files.**

- **Apple ProRAW with JPEG XL** (Compression 52546). On iPhone 16 Pro and 16 Pro Max
  under iOS 18, Settings > Camera > Formats offers JPEG Lossless, JPEG-XL Lossless and
  JPEG-XL Lossy for ProRAW (MacRumors, 2024-09-20). The iPhone 17 Pro models are
  ProRAW devices under iOS 26 (Apple support); that they keep the JXL options is only a
  forum report, so *unverified*.
- **ProfileGainTableMap** (52525, DNG 1.6). Apple's WWDC21 session names it as a ProRAW
  tag, and an exiftool forum dump shows it in an iPhone 12 Pro file. gamut-dng's docs
  also list `ProfileGainTableMap2` (52544). It is a rendering tag (a local tone map),
  not a pixel-meaning tag, so it sits with `BaselineExposure` rather than with the
  audit's §3 refusals.
- **Samsung Expert RAW** writes DNG 1.7 with JPEG XL on recent Galaxy S and Z bodies
  (a Samsung community article). Forum users dispute the bit depth and which modes use
  it: *unverified*.
- **Google Pixel:** no source found for JXL; one Pixel 10 Pro dump shows JPEG
  compression. *Unverified.*
- **Adobe:** since Lightroom 13.0 (October 2023), lossy DNG and derived DNGs (merge,
  Enhance) use JPEG XL (an Adobe-verified community answer). Whether the standalone DNG
  Converter offers JXL, and whether Leica writes it, are *unverified*.
- **What irradiance returns today (read from `src/`, nothing run):**
  - **ProRAW** fails before compression matters. Its raw IFD is LinearRaw with
    `SamplesPerPixel = 3`. `is_sensor_ifd` requires `SamplesPerPixel == 1`
    (`src/ifd.rs:2084`–`2088`), so `Container::sensor()` returns
    `Error::NoSensorIfd` (`src/ifd.rs:2452`–`2456`). The container still parses, so
    `irr ifd`-level metadata reads.
  - **A JXL-compressed single-sample plane** (a JXL CFA DNG, for example) reaches the
    plane decoder. `Compression::from_code` maps 52546 to `Compression::Other`
    (`src/ifd.rs:576`), and `plane_kind` refuses it as
    `Error::UnsupportedCompression { compression: 52546 }` (`src/plane.rs:1161`).
  - **ProfileGainTableMap** has no reader: nothing in `src/` names tag 52525 or 52544,
    so it is ignored. That is correct for a linear develop.
  - Both are typed refusals, never a panic or a silent wrong image.

**2. JPEG XL decoders in Rust.** Framed only as an option. Any of these would be a
runtime dependency behind a feature, which conflicts with the zero-runtime-dependency
rule and `no-new-top-level-deps-without-decision`; it would need a DEC. **Not
recommended for adoption here.**

| Crate | Licence (declared) | Pure Rust / `unsafe` | Maintenance | Notes |
|---|---|---|---|---|
| **jxl-oxide** | MIT OR Apache-2.0 | Says pure Rust; the top crate has no `forbid(unsafe_code)`; sub-crates *unverified* | 0.12.6, 2026-05-29; previous release 2025-09; slowing | Deps: `brotli-decompressor`, `tracing`, its own `jxl-*` crates; optional `lcms2` (C). No published conformance rate. rawler 0.8.0 uses it, so it is **not independent** of the current oracle. |
| **jxl-rs** (crate `jxl`) | BSD-3-Clause, with a PATENTS file | Allows `unsafe` for SIMD, with safety comments and expert review | 0.8.0, 2026-10-19; active | The decoder Chrome and Firefox ship. No formal conformance numbers; 16-bit output not stated. gamut-jxl uses it for decode. |
| **zenjxl-decoder** (imazen) | *unverified* (imazen ships zenraw as AGPL, so check) | `forbid(unsafe_code)` by default, opt-in `allow-unsafe` | 0.3.8 | A jxl-rs fork; reports 17/23 Level 5 conformance (2025-12). |
| **oxideav-jpegxl** | MIT | Pure Rust; claims clean-room from the spec | `0.0`, immature | 4 of 6 committed conformance streams; 16-bit modular lossless on small fixtures. |

Patent position: JPEG XL is royalty-free with royalty-free declarations from Google and
Cloudinary. The 2022 Microsoft rANS patent was disputed, and the format's co-author
says it does not affect JXL (The Register, 2022-02-17). Not on the patent-avoid list.

**Question for the maintainer:** should irradiance ever decode JXL DNG? If so, the
choices are (a) an optional dependency behind a feature (jxl-rs or jxl-oxide; a DEC
amending the zero-dependency rule for that feature), (b) a clean-room decoder (large),
or (c) a front end such as gamut-dng in a sibling crate. If not, the refusal stays and
the README says so.

**3. What changed since 2026-10-27 (and since the 11-12 memo).**

- **photocraft-raw:** the clean-room raw crate landed (PR #65) inside the photocraft
  repo; still not on crates.io. Repo licence MIT OR Apache-2.0; the PR itself states no
  licence. Claims no code read from dcraw/LibRaw/rawspeed/rawler/rawloader/darktable and
  no copied colour tables. No CR3, no JXL. *Claim unverified.*
- **FlashRaw:** 0.3.0 (2026-11-14), 0.3.1 (2026-11-15) and 0.4.0 (2026-11-20), still
  MIT OR Apache-2.0. 0.3.0 adds a **provenance statement**: decoders and the camera
  database were "rewritten from format descriptions and sample files", with values
  measured from samples or taken from Adobe DNG profiles. "Rewritten" suggests an
  earlier derivation; *unverified*. 0.3.1 adds **Nikon High Efficiency NEF
  (TicoRAW-based: patent / proprietary flag)**. JXL DNG is behind a non-default
  `jpegxl` feature. `unsafe` is said to be confined to SIMD and mmap.
- **oxideav-mjpeg:** 0.1.15 and 0.1.16 (2026-11-07/08), still MIT; now describes itself
  as a full still-image JPEG codec including lossless, arithmetic and hierarchical modes.
- **gamut-dng / gamut-jxl:** gamut-dng is still 1.0.0 on crates.io, `forbid(unsafe_code)`,
  claims "clean-slate from the official specs" with LLM help credited. Its JXL comes
  from gamut-jxl 0.5.0 (2026-10-23): decode through jxl-rs, encode through libjxl in an
  opt-in `-sys` crate.
- **filmcraft-dng:** no change (0.9.0, still no repository listed).
- **zenraw, rawler/dnglab, grego/apertus `dng`:** no change (0.2.0 AGPL/commercial;
  0.8.0 LGPL-2.1; 1.6.0 AGPL-3.0, repo now apertus-open-source-cinema/dng-rs).

**4. New entrants.**

- **RustDNG** (Henry-GongZY): a Rust translation of the Adobe DNG SDK 1.7.1 under the
  SDK's source-available licence, decoding JXL through jxl-oxide. Not permissive OSS and
  an Adobe-code derivative: **off-limits to read** under the clean-room rule. It does not
  change build-vs-adopt.
- No permissive rawler fork and no `image`-rs RAW effort found.
- **Build-vs-adopt:** unchanged for develop, where irradiance is still the only crate in
  this list that applies opcodes, refuses what it cannot honour, and pins its output.
  For *decode coverage* (JXL, ProRAW-shaped LinearRaw), gamut-dng is the one permissive
  candidate front end, as the 11-12 memo said.

**5. Second-oracle options** (run as separate tools, never linked; copyleft source never
read):

- **LibRaw sample tools** (LGPL-2.1 or CDDL-1.0): `unprocessed_raw` with black
  subtraction and scaling off writes the raw plane as TIFF, a **bit-exact plane oracle**
  independent of rawler. `4channels`, `raw-identify -v` (metadata) and `dcraw_emu`
  (develop) round it out.
- **rawpy** (MIT; bundles LibRaw): `raw_image` as `uint16` plus black/white levels,
  CFA pattern and matrix. The easiest scriptable plane checksum, but the same engine as
  LibRaw, so it is not a third independent source.
- **FlashRaw CLI / Python** (MIT/Apache): decode, info and a `verify` subcommand; reads
  JXL DNG behind a feature. Provenance self-reported.
- **gamut-cli** (MIT/Apache): a possible permissive DNG/JXL oracle; capabilities not
  checked.
- **darktable-cli** (GPL, licence *unverified*): developed output only, on rawspeed.
- **Not independent:** dnglab and zenraw both sit on rawler. For JXL, rawler and RustDNG
  share jxl-oxide, so a JXL cross-check should use jxl-rs or libjxl's `djxl`.

**Candidates for future-plans** (no commitment; default is to skip encumbered formats):

- Add LibRaw `unprocessed_raw` (or rawpy) plane checksums as a second plane oracle next
  to dnglab. *Patent: none. Licence: LGPL/CDDL tool, run only.*
- Decide whether JXL-compressed DNG (52546) is ever in scope, and if so whether through
  an optional dependency (jxl-rs / jxl-oxide) under a DEC. *Patent: royalty-free; no
  flag.*
- Decide whether 3-sample LinearRaw (ProRAW) is ever in scope. *Patent: none.*
- Read `ProfileGainTableMap` / `ProfileGainTableMap2` (52525/52544) as typed metadata,
  without applying them. *Patent: none.*
- Collect a phone-DNG corpus (iPhone 16/17 Pro ProRAW JXL lossless and lossy, Samsung
  Expert RAW, Lightroom lossy/Enhanced DNG) before any of the above. *Patent: none.*
- Use `djxl` or jxl-rs as the JXL tile oracle if JXL lands, not jxl-oxide. *Patent:
  none; BSD-3.*
- Keep watching gamut-dng as the permissive decode front end or competitor. *Patent:
  none; provenance self-asserted.*
- Record Nikon HE/HE★ NEF (now in FlashRaw) as excluded. *Patent / proprietary
  (TicoRAW): skip.*
- Record RustDNG as off-limits to read. *Proprietary (Adobe SDK licence): do not read.*

_Sources (fetched 2026-11-24, treated as untrusted): macrumors.com (2024-09-09,
2024-09-20), support.apple.com (iOS 26 ProRAW models; ProRAW page),
developer.apple.com/videos/play/wwdc2021/10160, exiftool.org forum msg 75150,
docs.rs/gamut-dng, r2.community.samsung.com (CamCyclopedia JPEG XL),
us.community.samsung.com (Galaxy S24 thread), discuss.pixls.us (Pixel 10 Pro DNG),
community.adobe.com (Lightroom JXL in derived DNG), github.com/tirr-c/jxl-oxide,
crates.io API for jxl-oxide / jxl / flashraw / gamut-jxl / filmcraft-dng / zenraw /
rawler / dng / oxideav-mjpeg, github.com/libjxl/jxl-rs, developer.chrome.com
(JPEG XL in Chrome), docs.rs/zenjxl-decoder 0.3.8 README, github.com/oxideav/oxideav-jpegxl,
github.com/ybouane/flashraw (README, CHANGELOG), github.com/visualcommons/gamut,
github.com/storytold/photocraft (PR #65), github.com/Henry-GongZY/RustDNG (README only),
libraw.org/docs/Samples-LibRaw.html, sources.debian.org (libraw COPYRIGHT),
pypi.org/project/rawpy, letmaik.github.io/rawpy, docs.darktable.org (darktable-cli),
theregister.com (2022-02-17)._
