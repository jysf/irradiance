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

_Last reconstructed: 2026-10-27; PROJ-003/004/005 and sequencing added 2026-11-02._

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

`PROJ-001 — Monochrome DNG develop` is the active wave. The pixel pipeline
(container → plane → develop → tone) is shipped; the project is not yet closed.

| Stage | Status | Delivers |
|---|---|---|
| STAGE-001 Foundations | ✅ shipped | Corpus, TIFF/IFD reader, DNG tag model, metadata oracle |
| STAGE-002 Monochrome plane | ✅ shipped | 14-bit→u16 unpack, bit-exact plane oracle, levels + geometry |
| STAGE-003 Opcodes & output | ✅ shipped | `FixBadPixelsConstant`, `WarpRectilinear`, sRGB tone curve, develop oracle |
| STAGE-004 crustyimg integration | ⏳ proposed | `raw-develop` feature, `Image` adapter, metadata-truthfulness fixes |
| STAGE-005 Harden the harness | ⏳ in flight | Absent-vs-unparseable tag oracle, fuzz crate under `-D warnings`, gate-audit tests |

STAGE-005 was not in the original four-stage brief — it was added 2026-08-30
after STAGE-001 shipped having revealed a gate that had never run and an oracle
that could not tell a missing tag from an unreadable one. Finishing **004** and
**005** is what remains to close PROJ-001.

## Near-term: finish PROJ-001

- **crustyimg integration (STAGE-004).** Wire irradiance into crustyimg behind a
  `raw-develop` feature, provide the `Image` adapter, and fix the
  metadata-truthfulness gaps so `crustyimg web q2m.dng -o photo.avif` produces a
  publishable image and `info` reports real sensor data, not the embedded
  preview's.
- **Harness hardening (STAGE-005).** The open specs: lint the fuzz crate (the
  second cargo graph currently sits outside every `-D warnings` gate),
  make the gates stop reporting green on checks they never ran, and narrow the
  develop-oracle's scope around warp-bearing pixels.

## PROJ-002 — Bayer develop (colour) · *next, largest wave*

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
~550–700 for the monochrome path. Resumes once PROJ-001 ships and a colour
camera's files are in the corpus.

## Sequencing — `0.1.1` is live; `0.2.0` next

**Correction (2026-11-02).** An earlier version of this section said to "cut and
publish `0.1.0` first". That was written without checking crates.io. As of
2026-11-03 (`curl https://crates.io/api/v1/crates/irradiance/versions`):
**`0.1.1` is published** (2026-10-29, `rust_version` 1.90) and `0.1.0` is
**yanked**. Both predate all demosaic work, so the next release is a minor,
**`0.2.0`**, carrying what is under `[Unreleased]` in `CHANGELOG.md`
(`demosaic`, colour metadata, `to_linear_f32_into`) plus whatever of the
downstream `pyirradiance` asks lands first (`feedback/2026-11-02-pyirradiance-requests.md`).
Whether `0.2.0` waits for STAGE-008 colour is a maintainer call. The additions
below land after it, as `0.3.0`+.

1. `0.2.0` — a release spec (`just new-release-spec`) under the `DEC-028` publish
   checks. The crate docs, README and CHANGELOG must say exactly what the surface
   is: monochrome and Bayer planes, demosaic, but **not** the colour transform
   until STAGE-008 ships.
2. Then, independent of each other: **PROJ-004 Stage A** (forensics) and
   **PROJ-003** (SOF-3 core). STAGE-008 continues on its own thread.
3. A bounded **spike** before PROJ-004 Stage B is designed (see below).
4. **PROJ-005** after SPEC-033 (WB) and SPEC-034.
5. Any M10 / M11 / Q3 Monochrom conformance claim waits for real Tier-B files and
   their rows in `docs/conformance-matrix.md`.

## PROJ-003 — Compressed planes: lossless JPEG (SOF-3) + GainMap · *next*

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
- **Out of scope:** lossy-DCT DNG, Nikon NEF-lossless (a different Huffman-
  difference scheme + linearization curve), Pentax PEF (`65535`).
- **Targets:** Leica M Monochrom Typ 246 and Pentax K-3 III DNG (held, both
  currently `UnsupportedCompression`); M10 / M11 / Q3 Monochrom and the Nikon
  D750 DNG **only once files are held**.

## PROJ-004 — Instrument layer: sensor forensics, optical black, geometry · *next*

The telemetry and calibration surface the Monochrom app's `[ I. SENSITOMETER ]`
and `[ II. OPTICS & PLANE ]` benches need (companion docs:
`~/PSeven/experiments/monochrom-app/docs/irradiance-apple-app-design.md` and
`…-deep-dive.md`). All of it stays zero-I/O and caller-allocated.

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

## PROJ-005 — Post-WB spectral monochrome mixer (Bayer → mono) · *after SPEC-033/034*

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
