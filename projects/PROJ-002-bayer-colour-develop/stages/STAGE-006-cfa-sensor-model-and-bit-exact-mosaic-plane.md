---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-006                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-10-29
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The plane half of PROJ-002's thesis — a Bayer mosaic reads bit-exact and its
    colour metadata parses truthfully — the trusted foundation the demosaic and
    colour transform build on. No colour is produced here; this stage makes the
    input correct and the oracle honest before a single pixel is interpolated.
  delivers:
    - "A Bayer-CFA DNG is accepted by the sensor gate and its mosaic plane unpacks bit-exact against `dnglab`"
    - "`Sensor` carries a structured CFA model (pattern, repeat dims, plane colours) for a Bayer file, and `None` for the monochrome Leica"
    - "`info` reports the CFA pattern, per-channel black/white levels and the colour matrices parsed from the file"
  explicitly_does_not:
    - "Demosaic — no mosaic → RGB interpolation (that is STAGE-007)"
    - "Colour transform — no white balance, no matrix application, no colour output (STAGE-008)"
    - "X-Trans / non-Bayer CFA — the Fuji x100f is a later stage"
    - "Native vendor containers (NEF/RAF) or lossless-JPEG SOF-3"
    - "Applying per-channel levels — levels are PARSED and exposed here; they are applied at develop time (STAGE-007/008)"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-006: CFA sensor model and bit-exact mosaic plane

## What This Stage Is

The foundation of colour develop: teach the sensor model to recognise and read a
**Bayer-CFA** DNG without yet interpreting a single colour. When this stage
ships, an uncompressed Bayer DNG is selected by the sensor gate, its mosaic plane
unpacks **bit-exact** against `dnglab analyze --raw-checksum` (the same oracle
PROJ-001 trusts), and the colour metadata it carries — CFA pattern, per-channel
levels, white balance and the colour matrices — is parsed into `Sensor` and
verified against `dnglab analyze --meta` and `exiftool`. No demosaic, no white
balance, no matrix maths. The deliverable is a **correct input and an honest
oracle**, so the demosaic (STAGE-007) and colour transform (STAGE-008) build on
values that are already trusted rather than re-deriving trust mid-pipeline.

## Why Now

**It is the one colour stage with no colour risk.** The two hard, tolerance-based
problems of PROJ-002 — demosaic quality and colour accuracy — live in STAGE-007
and STAGE-008. This stage is entirely bit-exact and metadata-diff: the mosaic is
still a single sensor plane, so PROJ-001's layer-0 contract
(`dnglab --raw-checksum`) transfers **verbatim**, and the colour tags are checked
by the same metadata oracle that already exists. It is the part of colour that
can be verified with zero new tolerance, so it is the right first step.

**The design needs no corpus.** Identifying the CFA seam, the `Cfa` model, the
`Sensor` field shape and the spec breakdown are all code-and-spec decisions
against the parser that exists today. Only *build and verify* need the Bayer DNG.
So this stage can be designed and framed now, in parallel with the maintainer
producing the converted file and while PROJ-001's STAGE-004 is blocked on
crustyimg.

**It de-risks the whole project cheaply.** If a converted Nikon DNG carries
something the Leica path never exercised — a `LinearizationTable`, a `GainMap`,
per-channel black as an array rather than a scalar — this stage surfaces it as a
bit-exact mismatch or a metadata diff, loudly and early, before any colour code
is written on top of a wrong plane.

## Success Criteria

- An uncompressed Bayer DNG is selected by `is_sensor_ifd`
  (`PhotometricInterpretation == 32803`, `SamplesPerPixel == 1`), and its mosaic
  plane's MD5 equals `dnglab analyze --raw-checksum`, **exactly**
- `Sensor` exposes a structured CFA model (pattern, repeat dims, plane colours,
  layout) for the Bayer file and **`None`** for the monochrome Leica
- `AsShotNeutral`, `ColorMatrix1/2`, `CalibrationIlluminant1/2`,
  `ForwardMatrix1/2` and per-channel `BlackLevel`/`WhiteLevel` parse and match
  `dnglab analyze --meta` / `exiftool`
- The monochrome path is **byte-for-byte unchanged** — the existing Q2M plane
  oracle and develop oracle stay green (regression guard)
- The parser never panics on the Bayer file or on truncated/hostile CFA input
  (fuzz corpus extended with a Bayer file)

## Scope

### In scope
- **Identity:** extend `is_sensor_ifd` (`src/ifd.rs:1035`) to accept
  `PhotometricInterpretation == 32803 (CFA)` alongside `34892 (LinearRaw)`.
  `SamplesPerPixel == 1` is **unchanged** (see Design Notes — a mosaic is one
  sample per pixel). `NewSubfileType == 0` unchanged.
- **CFA model:** a new `Sensor` field (e.g. `color_filter: Option<Cfa>`) carrying
  `CFAPattern`, `CFARepeatPatternDim`, `CFAPlaneColor`, `CFALayout` — `None` for
  mono. Parsed and exposed; the mosaic-plane oracle and `info` are its readers.
- **Mosaic plane:** make the Bayer plane unpack, then verify it bit-exact against
  `dnglab analyze --raw-checksum` with a red-proof. `plane::unpack_into` is
  single-strip-only (SPEC-012, `src/plane.rs:225`) and asserts `max ≤ WhiteLevel`
  (DEC-008, `src/plane.rs:321`) — **both break on the Bayer corpus** (the Nikon
  D300S is 12-strip; the Canon 5D-III has a sample above WhiteLevel), so two
  precursor specs land first — **multi-strip assembly** and a **relaxed WhiteLevel
  policy (amending DEC-008)** — then the oracle (SPEC-025).
- **Colour metadata parse + oracle extension:** `AsShotNeutral`,
  `ColorMatrix1/2`, `CalibrationIlluminant1/2`, `ForwardMatrix1/2`, per-channel
  `BlackLevel` (array / `BlackLevelRepeatDim`) and `WhiteLevel`, parsed into
  `Sensor` and diffed against `dnglab analyze --meta` + `exiftool`. The oracle is
  their reader (AGENTS.md §11), so STAGE-007/008 consume already-trusted values.
- **Corpus + fuzz:** register the uncompressed Bayer DNG in the corpus manifest;
  extend the fuzz corpus with a CFA file; confirm panic-free.

### Explicitly out of scope
- Demosaic (STAGE-007); colour transform / output (STAGE-008)
- **Applying** levels, white balance or matrices — parsed and exposed only here
- X-Trans / non-Bayer CFA; native NEF/RAF; lossless-JPEG SOF-3
- `LinearizationTable` / `GainMap` *application* — if the corpus file carries one,
  this stage records its presence (and fails loudly if it changes the plane
  bytes); applying it is scheduled into STAGE-007, not smuggled in here

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-006` promotes the `(not yet written)` lines below into
real specs at `cycle: frame` with stable IDs. (The `next_id SPEC` scan used to be
polluted by a test fixture — fixed in `f0ecc24`, so it mints clean ids and this
stage framed as SPEC-024..027.)

- [x] SPEC-024 (shipped on 2026-10-29) — [M] CFA sensor identity + `Cfa` mosaic model: accept `PhotometricInterpretation == 32803`; parse `CFAPattern`/`CFARepeatPatternDim`/`CFAPlaneColor`/`CFALayout` into a new `Sensor` field; mono stays `None`; mono path unchanged (regression guard)
- [x] SPEC-028 (shipped on 2026-10-29) — [M] Multi-strip mosaic plane assembly: unpack a Bayer plane stored as N strips (`plane::unpack_into` is single-strip-only, SPEC-012 `src/plane.rs:225`; the Nikon D300S is 12 strips)
- [x] SPEC-029 (shipped on 2026-10-30) — [S] WhiteLevel-exceedance policy: samples above `WhiteLevel` are legal (the Canon 5D-III clips one); relax `SampleExceedsWhiteLevel` and amend DEC-008
- [x] SPEC-025 (shipped on 2026-10-30) — [M] Bit-exact mosaic plane oracle: the Bayer file's unpacked plane MD5 equals `dnglab analyze --raw-checksum`, with its red-proof
- [ ] SPEC-026 (frame) — [M] Colour metadata parse + metadata-oracle extension: `AsShotNeutral`, `ColorMatrix1/2`, `CalibrationIlluminant1/2`, `ForwardMatrix1/2`, per-channel `BlackLevel`/`WhiteLevel` — diffed against `dnglab analyze --meta` + `exiftool`
- [ ] SPEC-027 (frame) — [S] Corpus + fuzz: register the uncompressed Bayer DNG in the corpus manifest and extend the fuzz corpus; panic-free on CFA input

**Count:** 4 shipped / 2 active / 0 pending

## Design Notes

**The identity change is one disjunct, and `SamplesPerPixel` does not move.** A
Bayer mosaic stores one value per photosite — colour is encoded by the *position*
in the CFA pattern, not by multiple samples — so a CFA DNG is still
`SamplesPerPixel == 1`. The only selection change is
`photometric == PHOTOMETRIC_LINEAR_RAW` becoming
`photometric ∈ { PHOTOMETRIC_LINEAR_RAW, PHOTOMETRIC_CFA }`. The gate's
read-and-evaluate-per-tag shape (`SPEC-004` FU-20) and the `SensorMatch`
Unreadable handling are untouched. Add a `PHOTOMETRIC_CFA: u32 = 32803` constant
beside `PHOTOMETRIC_LINEAR_RAW` (`src/ifd.rs:138`).

**One `Sensor` type, an optional CFA — mono path untouched.** Prefer a single
`Sensor` with `color_filter: Option<Cfa>` (`None` for the Monochrom) over a
mono/CFA type split. It is purely additive: every existing field keeps its
meaning, the monochrome gate and unpack are byte-for-byte unchanged, and
downstream `develop_into` branches on `color_filter.is_some()` in a later stage.
`photometric: u32` already records which interpretation matched; the `Cfa`
struct adds the structure on top. **If this type choice proves weighty in build,
promote it to a DEC** (note: `DEC-029` is informally reserved for the crustyimg
LUT boundary — reconcile the next free number at creation, `DEC-028` is taken).

**Per-channel levels change the `black_level` representation.** Today
`Sensor::black_level: Option<u32>` is a scalar and `black_level_repeat_dim:
Option<[u32; 2]>` already anticipates a repeated pattern. A Bayer file typically
carries `BlackLevel` as four values over a 2×2 `BlackLevelRepeatDim` (per-CFA-
channel black), and `WhiteLevel` may be per-channel too. Decide the
representation in the first spec (a small per-channel array keyed by the repeat
dim, with the scalar as the degenerate 1×1 case) so mono stays `Some(scalar)` and
Bayer gets the array without a breaking change.

**Parsed-but-unapplied is deliberate, and the oracle is the reader.** The colour
matrices and white balance are *consumed* in STAGE-007/008, not here. To satisfy
the unread-field rule (AGENTS.md §11 — a field added gets a reader in the same
change), these fields are read by the **metadata oracle** and `info` the moment
they are added — the same pattern PROJ-001 used for the opcode byte-streams
(`opcode_list_1`, read by the develop step that parsed them). This is why the
metadata-oracle spec is in *this* stage, not deferred to colour.

**The plane contract transfers verbatim.** `dnglab analyze --raw-checksum`
checksums the raw mosaic bytes, independent of demosaic. So the layer-0 oracle
needs no change in method — only a second corpus entry — and this stage is fully
and bit-exactly verifiable the moment the Bayer DNG lands. Keep the red-proof:
the oracle must demonstrably go red on a deliberately wrong read.

**Corpus is the one hard dependency, and it blocks only build/verify.** Design
and framing proceed without it. The file must be an **uncompressed** DNG (Adobe
DNG Converter → "Uncompressed", from a Nikon NEF) so the existing strip reader
and unpack apply with no SOF-3 work. Ideally a ColorChecker frame too, for
STAGE-008's ΔE — but STAGE-006 needs only the plain Bayer file.

Tag numbers to confirm from the DNG 1.x spec at build (pointers, not facts):
`CFARepeatPatternDim 33421`, `CFAPattern 33422`, `CFAPlaneColor 50710`,
`CFALayout 50711`, `BlackLevelRepeatDim 50713`, `BlackLevel 50714`,
`WhiteLevel 50717`, `ColorMatrix1 50721`, `ColorMatrix2 50722`,
`AsShotNeutral 50728`, `CalibrationIlluminant1 50778`,
`CalibrationIlluminant2 50779`, `ForwardMatrix1 50964`, `ForwardMatrix2 50965`.

## Dependencies

### Depends on
- **STAGE-002** (shipped) — `plane::unpack_into` and the bit-exact plane oracle,
  reused verbatim on the mosaic
- **STAGE-001** (shipped) — the TIFF/IFD reader, the DNG tag model, the corpus
  manifest harness and the metadata oracle, all extended (not rebuilt) here
- External: `dnglab` (plane checksum + metadata), `exiftool` (metadata
  cross-check), the Adobe DNG spec (CFA + colour-matrix tag definitions)
- **Corpus (blocks build/verify only):** an uncompressed Bayer DNG in
  `$IRRADIANCE_CORPUS_DIR`

### Enables
- **STAGE-007** (demosaic) — needs the mosaic plane and the `Cfa` pattern
- **STAGE-008** (colour transform) — needs the trusted `AsShotNeutral` /
  `ColorMatrix` / `ForwardMatrix` values this stage parses and verifies

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7)
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
