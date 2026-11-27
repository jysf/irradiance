---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-034                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-006                      # parent project
repo:
  id: irradiance

created_at: 2026-11-26
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "the mixer works on the Leica colour bodies onyx targets, not only on the Canon, Nikon and Pixel files it was built on"
  delivers:
    - "A Leica M10 and M11 colour DNG (14-bit lossless JPEG in a 16-bit BitsPerSample) unpacks"
    - "A Leica Q2 colour DNG with mandatory FixBadPixelsConstant and WarpRectilinear reaches the colour path and the mixer"
  explicitly_does_not:
    - "filter presets (a later PROJ-006 stage)"
    - "spatial highlight reconstruction"

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

# STAGE-034: Leica colour bodies reach the colour path

## What This Stage Is

The colour DNGs from onyx's Leica bodies get through irradiance's decode and colour
path, so SPEC-091's linear plane, SPEC-092's mixer and SPEC-093's developed mix run on them
instead of refusing. It is driven by real files: each spec fixes one refusal a held Leica
file shows.

## Why Now

On 2026-11-26 the first colour Leica files were probed (raw.pixls.us, CC0):
- **Leica Q2** (`LEICA-Q2/L1000750.DNG`) unpacks. The colour develop and the mixer then
  refuse it with `UnsupportedRequiredOpcode { list: 1, id: 4 }`: its OpcodeList1
  `FixBadPixelsConstant` and OpcodeList3 `WarpRectilinear` are both mandatory, the same as
  the held Q2 Monochrom.
- **Leica M10** (`LEICA-M10/f5381888.dng`) fails at unpack with `LjpegPrecisionMismatch
  { precision: 14, bits_per_sample: 16 }`. Adobe's `dng_validate` accepts it.

The same day three current bodies were probed, using camera-original DNGs from Photography
Blog's review galleries. Their firmware `Software` tags are camera versions, not Adobe's.
They have no stated licence, so they are kept in the local corpus only and never committed:
- **Q3** (`LEICA-Q3-PB/leica_q3_01.dng`, 9536×6344, 14-bit `Compression = 7`) and **SL3**
  (`LEICA-SL3-PB/leica_sl3_01.dng`, same geometry): both unpack, then refuse exactly as
  the Q2 does. Their OpcodeList1 `FixBadPixelsConstant` and OpcodeList3 `WarpRectilinear`
  are both mandatory.
- **M11** (`LEICA-M11-PB/leica_m11_01.dng`, 9536×6336, 60 MP): fails at unpack exactly as
  the M10 does (`precision: 14, bits_per_sample: 16`). It has no opcode lists. Adobe's
  `dng_validate` accepts it, with only string-termination warnings on metadata.

So **every onyx target body is refused today**, and these two refusals are the whole list
on the held files. SPEC-097 covers the M10 and M11; SPEC-098 covers the Q2, Q3 and SL3.
This stage has to ship before the mixer's crates.io release.

## Success Criteria

- The held Q2, Q3, SL3, M10 and M11 colour files produce a linear plane and a mixed plane,
  with no refusal.
- No output changes for any file that develops today.
- Every newly held Leica colour file is probed, and any new refusal gets a spec here.

## Scope

### In scope
- Lossless-JPEG precision below `BitsPerSample` (SPEC-097).
- Mandatory `FixBadPixelsConstant` on the CFA and mandatory `WarpRectilinear` for the
  colour entries. At sensor geometry the warp is the consumer's; the developed mix applies
  it, so design decides what each entry does with it.

### Explicitly out of scope
- Filter presets and their Leica fixtures (a later PROJ-006 stage).
- Patent-encumbered or non-DNG Leica formats.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-097 (shipped on 2026-11-27) — [S] accept a lossless-JPEG precision below `BitsPerSample` (the M10 and M11 files) (DEC-085 accepted; design PR 361, build PR 362, verify PR 363)
- [~] SPEC-098 (design) — [M] mandatory `FixBadPixelsConstant` on the CFA on every colour entry (four nearest same-colour photosites), and `WarpRectilinear` passed through at sensor geometry (the Q2, Q3 and SL3 files; DEC-086 proposed)
- [ ] SPEC-099 (frame) — [M] the per-plane `WarpRectilinear` (`N = 3`) on the developed colour outputs and the developed mix; split out of SPEC-098 at its design; `depends_on: [SPEC-098]`

**Count:** 1 shipped / 1 active / 1 pending

## Design Notes

- **onyx's rules for the opcode spec (its SPEC-093 reply, 2026-11-26):**
  - **Warp at sensor geometry is the consumer's job.** SPEC-091's planes and
    `mono_mix_plane_*` are onyx's viewport input, and the viewport applies the warp itself on
    the GPU (its DEC-036, DEC-063), as it does for the unpacked mono plane. So these entries
    must **not** refuse a mandatory `WarpRectilinear`, and must document that the consumer
    applies it. Only the developed outputs (`develop_color_into` and the developed mix)
    apply the warp, or refuse when they cannot.
  - **`FixBadPixelsConstant` runs before demosaic, everywhere.** A bad photosite left in
    the CFA spreads into its neighbours through demosaic. So apply it on the CFA before
    demosaic in the sensor-geometry planes and in the developed mix alike, which keeps
    export equal to the viewport. Fill each bad photosite from neighbours of the same
    colour.
  - These change output only for files that are refused today.

- Each spec is pinned to a held file and checked against `dng_validate` where possible.
- **SPEC-097's design (2026-11-27):** with the precision check relaxed, the M10 and M11
  decode bit-identical to `dng_validate`'s stage 1, dnglab and LibRaw, and reach every
  colour and mixer entry with `Ok`. Neither has opcode lists, so after SPEC-097 the only
  refusals left on held files are the Q2, Q3 and SL3 mandatory opcodes.
- Files that develop today must not change, so `DEVELOP_VERSION` is unchanged unless a
  spec says why.
- **SPEC-097 shipped (2026-11-27).** The M10 and M11 unpack bit-identical to
  `dng_validate`'s stage 1 and dnglab, and reach every colour and mixer entry with `Ok`.
  The only refusals left on held files are the Q2, Q3 and SL3 mandatory opcodes, which the
  next spec takes. onyx is asked in the PROJ-006 brief whether it wants highlight recovery
  above the M10's `WhiteLevel`; that is not a spec here.
- **SPEC-098's design (2026-11-27), split in two.** The three files' warps have `N = 3`
  (one set per colour plane, up to 1.3 px apart at the corner), which the developed
  outputs cannot apply without a camera-RGB intermediate and onyx's answer on the
  developed mix (DEC-084 warps the mixed plane once). So SPEC-098 is the bad-pixel fix
  on every colour entry plus the sensor-geometry pass-through (a new
  `DevelopPath::SensorGeometry`), which meets this stage's success criterion, and
  SPEC-099 is the developed warp. The fix fills each bad photosite from its four nearest
  same-colour photosites, colour from `CFAPattern`; `BayerPhase` is not used, because the
  Adobe SDK reads its parity opposite to the specification (shown on the SL3's 27,651
  bad photosites, all outside the crop). Only the SL3 has bad photosites; the Q2 and Q3
  have none. No file that develops today changes.
- **Orchestration cost at this stage's close:** SPEC-097's design transcript
  (`871eb085`) is counted in SPEC-097's own cost, so it must not be counted again here.
  The orchestrator's entry starts after 2026-11-27T06:00Z (STAGE-031 counted up to then),
  and the `leica-dng-search` transcript (`30266aa6`) belongs to this stage.

## Dependencies

### Depends on
- STAGE-031 (SPEC-091, SPEC-092) for the colour entries these files must reach.
- External: none outstanding. Q3, SL3 and M11 colour DNGs are held locally from Photography
  Blog (2026-11-26), never committed; the Q2 and M10 are CC0 from raw.pixls.us.

### Enables
- The PROJ-006 presets stage (judged on a colour Leica file) and the release.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m — only if this stage was
  framed with `just frame-stage`. Counts outlines whose scope held from frame to
  ship. A low number means framing ran ahead of what was knowable; a high one
  means it's paying for itself.>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
