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
refreshed 2026-11-24 (DNG 1.7 / JPEG XL, new entrants, second-oracle options)._

## The through-line

There is no permissively-licensed, maintained, pure-Rust RAW decoder. Every
mature one (dcraw / LibRaw / rawspeed and their descendants) carries GPL, LGPL
or AGPL, and there is no permissive option in Go, Java, Scala or Elixir either.
irradiance exists to be that missing primitive: a **clean-room, patent-safe,
pure-Rust RAW → image** engine the ecosystem can actually adopt. Its first
consumer is **crustyimg** (whose "RAW" support today is really the camera's
embedded JPEG); the maintainer's own driver is developing Leica Q2 Monochrom
files reproducibly into web assets.

Every wave is held to the same bar: **independent-oracle-verified correctness,
no panics on hostile input, deterministic output, and a provenance-ledger entry
for every algorithm and decoder** naming its source and that source's licence.

## Where we are now

Three waves are in flight or shipped, and the crate is published to crates.io.

- **`PROJ-001 — Monochrome DNG develop`** — the pixel pipeline (container → plane →
  develop → tone) is shipped and published as **`0.1.1`** (`0.1.0` yanked). Foundations,
  the monochrome plane, and opcodes + output (STAGE-001/002/003) shipped; harness
  hardening (STAGE-005) and crustyimg integration landed.
- **`PROJ-002 — Bayer develop (colour)`** — well advanced: the CFA model, the clean-room
  Malvar–He–Cutler demosaic, and the colour transform + dual-illuminant interpolation
  (SPEC-033/SPEC-034) have shipped; the ColorChecker ΔE oracle (SPEC-035) is the one spec
  left, **blocked on a physical chart**. A reconciled downstream wave (STAGE-010) carries
  the `0.2.0` release prep.
- **`PROJ-003 — Compressed planes (SOF-3)`** — the single-component decoder (STAGE-009)
  shipped; the DNG `Compression 7` binding (STAGE-011) is **one spec from closing**. Both
  held compressed monochrome files (Leica M Typ 246, Pentax K-3 III) now decode bit-exact
  against `dnglab`; only the capstone (SPEC-045: provenance, determinism, container
  hardening) remains.

## Near-term: close PROJ-003, then review and release

- **Close PROJ-003.** Ship SPEC-045 (the STAGE-011 capstone — a provenance-ledger
  binding row, a plane-level determinism pin, and lossless-JPEG header hardening), then
  the STAGE-011 stage-ship and the PROJ-003 project-ship.
- **Full code review AT the PROJ-003 close** (maintainer decision, 2026-11-06). A
  whole-crate review — pointed hard at the new `ljpeg` decoder and the Compression-7
  binding — on a clean slate, before PROJ-004/005 build on top of it. User-triggered
  (`/code-review ultra`); the orchestrator prepares the scope and acts on the findings.
- **Cut `0.2.0`.** The next workstream after the close — see Sequencing below.

## PROJ-002 — Bayer develop (colour) · *active — demosaic + colour transform shipped; ΔE oracle (SPEC-035) blocked on a chart*

The colour pipeline that PROJ-001 deliberately omits (a Monochrom sensor has no
colour filter array, so there is nothing to demosaic). Planned content:

- A **CFA model** (Bayer mosaic description and indexing).
- A **clean-room Malvar–He–Cutler demosaic** — implemented from the published
  algorithm, not ported from any copyleft decoder.
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

## Sequencing — `0.1.1` is live; close PROJ-003, review, then `0.2.0`

`0.1.1` is published (2026-10-29, `rust_version` 1.90); `0.1.0` is yanked. Both predate
all demosaic and compressed-plane work, so the next release is a minor, **`0.2.0`**,
carrying what is under `[Unreleased]` in `CHANGELOG.md` (demosaic, colour metadata, the
SOF-3 compressed-plane binding) plus the downstream `pyirradiance` asks
(`feedback/2026-11-02-pyirradiance-requests.md`) and the external 0.2.0 review's
semver-urgent `#[non_exhaustive]`.

**Reconciled order (maintainer decision, 2026-11-06):**

1. **Close PROJ-003** — ship SPEC-045, then the STAGE-011 stage-ship and the PROJ-003
   project-ship.
2. **Full code review** at that close — whole crate, clean slate, before new project work
   (user-triggered `/code-review ultra`; the orchestrator preps scope and acts on findings).
3. **`0.2.0`** — a release spec (`just new-release-spec`) under the `DEC-028` publish
   checks, coordinated with the PROJ-002 colour thread: what colour surface `0.2.0`
   exposes is a maintainer call that depends on where SPEC-035 and the colour-geometry
   work land. The crate docs, README and CHANGELOG must say exactly what the surface is.
4. **Then, order TBD:** **PROJ-004** (instrument layer — starts with the opcode-inventory
   spike, which also covers the `GainMap` deferred out of PROJ-003), **PROJ-006**, the
   mono mixer (now unblocked — SPEC-033/034 shipped), and the **adoption push** (an `image`-crate adapter +
   outreach to zenraw/oculante, `DEC-028`).
5. Any M10 / M11 / Q3 Monochrom conformance claim waits for real Tier-B files and
   their rows in `docs/conformance-matrix.md`.

## PROJ-003 — Compressed planes: lossless JPEG (SOF-3) · *closing — STAGE-009 core shipped; STAGE-011 binding one spec from close*

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

## PROJ-004 — Instrument layer, opcode coverage and DNG writing · *proposed — project created 2026-11-11*

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

## PROJ-006 — Post-WB spectral monochrome mixer (Bayer → mono) · *unblocked — SPEC-033/034 shipped*

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
