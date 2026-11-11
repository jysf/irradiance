---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-001
  status: shipped
  activity: null
  priority: high
  target_ship: null

repo:
  id: irradiance

created_at: 2026-08-15
shipped_at: 2026-11-11

closed_reason: shipped

value:
  thesis: >
    A permissive, pure-Rust library can develop an uncompressed monochrome DNG
    to a correct image — verified bit-exact against an independent
    implementation — in roughly one module's worth of code, closing a gap no
    library in ANY language currently fills.
  beneficiaries:
    - "The maintainer — develops his own Leica Q2 Monochrom files reproducibly into web assets"
    - "crustyimg — its RAW claim becomes true rather than 'the camera's embedded JPEG'"
    - "Rust imaging projects currently forced onto copyleft RAW crates (oculante, czkawka, image-hdr, stardetect, …)"
    - "Self-hosted photo servers and DAM ingest facing untrusted RAW uploads"
  success_signals:
    - "A Q2M DNG renders end to end; the decoded plane matches `dnglab analyze --raw-checksum` EXACTLY"
    - "`crustyimg web q2m.dng -o photo.avif` produces an image worth publishing"
    - "`info` reports real sensor dimensions, bit depth, levels, crop and orientation — not the embedded preview's"
    - "The parser survives the fuzz corpus with no panics on hostile input"
    - "Every algorithm and decoder carries a provenance-ledger entry naming its source and that source's licence"
  risks_to_thesis:
    - "The corpus never gets assembled — unglamorous, cannot be delegated, and nothing downstream is verifiable without it"
    - "`WarpRectilinear` proves harder or more visible than estimated, and Phase 1 output matches no reference render"
    - "Peak memory for a 47 MP develop breaches a sane budget and forces a tiling redesign mid-project"
    - "Estimates here are derived from crustyimg's cost data on a mature repo; greenfield may differ in either direction"
    - "The oracle is single-sourced — dnglab's plane checksum AND reference render both come from rawler, so a tolerance test could pass while both are wrong together. Analytic makedng fixtures mitigate it; the only fully independent check (ColorChecker ΔE) is PROJ-002"
    - "PROJ-001 validates against one camera, one firmware, one frame — Leica-specific assumptions could hide until a second DNG source arrives"

value_realized:
  thesis_held: partly
  signals_observed:
    - "Q2M plane MD5 equals `dnglab analyze --raw-checksum` EXACTLY across the held Q2M frames, with a red-proof (SPEC-012/013); later extended to every held mono file incl. the M Monochrom and both Compression-7 files (PROJ-003)"
    - "The library reports real sensor dimensions, bit depth, black/white levels, ActiveArea/DefaultCrop and Orientation from the sensor IFD, not the preview (SPEC-004/014/015; `irr ifd`), verified against dnglab and exiftool by the metadata oracle (SPEC-005/010)"
    - "The parser survives its fuzz corpus with typed errors and no panics (SPEC-003 onward; fuzz graph linted under -D warnings, SPEC-011)"
    - "Every algorithm carries a provenance-ledger row (docs/provenance-ledger.md)"
  evidence: >
    Four of the five success signals were observed. The core claim held: a
    permissive, pure-Rust, zero-dependency library develops an uncompressed
    monochrome DNG to a correct image, with the plane bit-exact against an
    independent implementation. That covers bad-pixel and warp opcodes, geometry,
    and a tone curve with three output modes (STAGE-001..003). It also shipped to
    crates.io (0.1.1, then 0.2.0). Not observed: the crustyimg success signal
    ("crustyimg web q2m.dng produces an image worth publishing"). That work is
    crustyimg-side and still unscheduled there, so STAGE-004 moved to PROJ-004.
  notes: >
    "Partly" for two reasons. (1) The consumer-integration signal transferred
    rather than landed: a success signal that depends on another repo's calendar
    should be stated as a cross-repo dependency from the start, not a project
    deliverable. (2) "Roughly one module's worth of code" undershot. The
    deliverable is several modules (ifd, plane, opcode, develop, warp, tone) plus a
    large gate and oracle harness, and most of the effort went into proving
    correctness (red-proofs, gate audits, CI truthfulness), not decoding. The
    single-source risk (dnglab is built on rawler) was mitigated with analytic
    fixtures. Full independence arrives with PROJ-002's ColorChecker oracle
    (SPEC-035).

roadmap:
  - item: "Bayer develop — CFA model, clean-room MHC demosaic, colour matrices, ColorChecker ΔE"
    kind: goal
    horizon: next
    resume_when: "PROJ-001 ships and a colour camera's files are decodable"
  - item: "Lossless JPEG SOF-3 — the keystone shared by 8 vendor decoders"
    kind: goal
    horizon: next
  - item: "X-Trans (DHT), native Fuji RAF, Nikon NEF + linearization curve"
    kind: goal
    horizon: later
---

# PROJ-001: Monochrome DNG develop, end to end

## What This Project Is

The first wave of `irradiance`: read an uncompressed monochrome DNG and develop
it into a correct image, with the metadata that came with it, verified against
an independent implementation. It delivers the container reader, the sensor-plane
decode, the DNG opcodes the file actually carries, the tone curve, and the
crustyimg integration that makes the result usable from a command line.

It deliberately stops before colour. There is no demosaic, no white balance and
no colour-matrix work in this project — a Leica Q2 Monochrom has no colour filter
array, so that pipeline is *absent*, not deferred. Colour is PROJ-002.

## Why Now

**The measurements say it is small.** A monochrome DNG needs no demosaic, no
white balance and no colour matrix; the sensor plane *is* the image. Measured
against a real Q2M file: uncompressed, single strip, 14-bit tightly packed, with
the byte arithmetic closing exactly. That is roughly 550–700 lines on top of the
container reader, against ~1,300–1,600 for the Bayer path.

**The oracle already exists and its contract is verified.** `dnglab` supplies all
three verification layers as shell commands, and the bit-exact plane contract was
established empirically before this repo had a line of code (see
`docs/oracle-contract.md`). The question most likely to make this project
intractable — *can correctness be checked at all?* — is closed, and closed
favourably.

**The gap is real and universal.** Surveyed 2026-08-15: every mature RAW decoder
in every language descends from dcraw/LibRaw/rawspeed and carries GPL, LGPL or
AGPL. There is no permissive, maintained, pure-Rust option — and no permissive
one in Go, Java, Scala or Elixir either.

**Adobe's DNG specification is public and carries a patent grant** for compliant
implementations, so the container can be implemented from a published spec rather
than by reading anyone's code. That is the cleanest provenance available.

## Success Criteria

- A Leica Q2 Monochrom DNG decodes to a plane whose MD5 equals
  `dnglab analyze --raw-checksum`, **exactly**
- The developed output scores within a stated SSIMULACRA2 tolerance of
  `dnglab analyze --srgb`, and the oracle demonstrably goes **red** on a
  deliberately broken render
- `crustyimg web q2m.dng -o photo.avif` works through the `raw-develop` feature
- `info` reports the real sensor dimensions, bit depth, black/white levels, crop
  geometry and orientation — not the embedded preview's
- The parser never panics on the fuzz corpus or on truncated/hostile input
- Every algorithm and decoder has a provenance-ledger entry
- The same DNG produces byte-identical output on macOS and Linux

## Scope

### In scope
- TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion
- DNG tag model and typed metadata extraction
- Uncompressed strip read; packed 14-bit → `u16` unpack
- Black/white level normalization
- `OpcodeList1: FixBadPixelsConstant` and `OpcodeList3: WarpRectilinear`
- Three-stage geometry: `ActiveArea` → `DefaultCrop` → `Orientation`
- Tone curve and output modes (linear `f32`, `u16`, 8-bit)
- The two-tier corpus and the three-layer oracle harness
- crustyimg's `raw-develop` feature, `Image` adapter, and metadata-truthfulness fixes
- A provenance ledger and this repo's own threat model

### Explicitly out of scope
- **Demosaic, white balance, colour matrices** — no CFA exists in a Monochrom file (PROJ-002)
- **Lossless JPEG SOF-3** — the Q2M is uncompressed; this is the keystone for other vendors, not for this project
- **Any vendor container** — no NEF, RAF, CR2, ARW, NRW
- **Interactive editing** — per-image work belongs in `crustyimg-lab`, per crustyimg's DEC-088 fence
- **Competing with Lightroom on render quality** — an unbounded tuning loop against a moving target
- **A CLI** — `irradiance` is a library; `irr` exists only as an internal dev/oracle binary

## Stage Plan

Format: `- [status] STAGE-ID — one-line summary`

- [x] STAGE-001 (shipped 2026-08-22) — Foundations: corpus, TIFF/IFD reader, DNG tag model, metadata oracle
- [x] STAGE-002 (shipped 2026-09-06) — The monochrome plane: unpack, bit-exact oracle, levels and geometry
- [x] STAGE-003 (shipped 2026-10-27) — Opcodes and output: bad pixels, WarpRectilinear, tone curve, develop oracle
- [→] STAGE-004 (moved to PROJ-004, 2026-11-11) — crustyimg integration: implemented crustyimg-side (its PROJ-016/STAGE-056), unscheduled there
- [x] STAGE-005 (shipped 2026-11-11) — Harden the harness: gates, the oracle's parser, the second cargo graph (added 2026-08-30; 4 leftover [XS] outlines moved to PROJ-002 STAGE-014)

**Count:** 4 shipped / 0 active / 0 pending (1 moved)

Each stage ends at a **stopping point** where the maintainer reviews before the
next begins. Effort estimates live in each stage file, in hours rather than
calendar, because pace is the maintainer's to set.

## Dependencies

### Depends on
- External: `dnglab` (Homebrew, LGPL-2.1 — **run as a tool, never linked**) for all three oracle layers
- External: the Adobe DNG Specification — implemented from the published spec, not from the SDK,
  whose terms are ambiguous and whose patent licence reportedly does not cover it
- Hardware: a Leica Q2 Monochrom and its files (held)

### Enables
- **PROJ-002** — Bayer develop: CFA model, clean-room MHC demosaic, colour matrices, ColorChecker ΔE
- **PROJ-003** — Lossless JPEG SOF-3, which eight vendor decoders share
- crustyimg's RAW claim becoming true, and its `source_format`/orientation/EXIF gaps closing
- A permissive RAW primitive the Rust ecosystem currently lacks

## Project-Level Reflection

*Filled in when status moves to shipped.*

*Written at close, 2026-11-11 (Prompt 1e, run by the orchestrator).*

- **Did we deliver the outcome in "What This Project Is"?** Mostly. Everything on
  irradiance's side shipped, and every layer is verified against an independent
  implementation or analytic fixtures, each with a red-proof:
  - the container reader;
  - the sensor-plane decode, bit-exact;
  - the two opcodes the Q2M actually carries;
  - three-stage geometry;
  - the tone curve and its three output modes.

  irradiance went to crates.io (DEC-028). The crustyimg command-line integration
  did not land. It is crustyimg-side work that crustyimg has not scheduled, so
  STAGE-004 moved to PROJ-004 rather than holding this project open.
- **How many stages did it actually take?** Five were planned or added:
  - four planned: STAGE-001..004;
  - STAGE-005 added on 2026-08-30 for harness debt that STAGE-001 measured.

  Four shipped and one moved. The work comes to 23 specs (SPEC-001..023), 4
  patches (PATCH-001..004) and 2 spikes.
- **What changed between starting and shipping?** The project's centre of gravity
  moved from decoding to *proving*. STAGE-001 shipped having found that a CI gate
  had never once passed, 17 consecutive red runs (PATCH-001). That reshaped the
  repo's disciplines:
  - every gate gets a red-proof;
  - gate-audit runs over the gates;
  - CI must be observed green on the claimed SHA.

  Meanwhile, PROJ-002 and PROJ-003 overtook the integration stage.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Already codified during the project: AGENTS.md §16's four rules and the
    "two cargo graphs, two invocations" rule. Nothing new to land at close.
  - New, at close: **a success signal that depends on another repo's schedule
    should be framed as a cross-repo dependency, not a project deliverable.**
    Otherwise the project can only close "partly". Recorded here; N=1, not a
    signal yet.
- **What did we defer to the next project?**
  - STAGE-004 (crustyimg integration) moved to PROJ-004. It ships when crustyimg's
    PROJ-016/STAGE-056 ships.
  - STAGE-005's four [XS] harness outlines moved to PROJ-002 STAGE-014
    (0.2.x maintenance).
  - Fully independent colour/tone truth (ColorChecker ΔE) is PROJ-002's SPEC-035.
