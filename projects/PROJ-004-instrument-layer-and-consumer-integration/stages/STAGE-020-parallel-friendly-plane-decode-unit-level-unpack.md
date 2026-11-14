---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-020                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-13
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Lets a consumer decode the sensor plane in independent pieces on its own threads, and makes the single-threaded decode itself faster, so a monochrome file opens in a fraction of today's time while irradiance stays single-threaded and byte-identical.
  delivers:
    - "plane::units(sensor) and plane::unpack_unit_into: each unit a rectangle of the plane plus the bytes it needs, decodable in any order on any thread, assembling byte-identically to unpack_into"
    - "Uncompressed row bands inside a strip (every held uncompressed file has byte-aligned rows), whole tiles (SPEC-058's window), and whole Compression 7 strips where nothing finer exists"
    - "unpack_into iterating the same units, so the two paths cannot drift"
    - "A faster single-threaded packed-bits unpack for 12/14-bit whole-byte rows, if the profile shows the bit loop dominates (byte-identical)"
    - "A faster single-stream Compression 7 decode, if the profile shows a win that stays bit-exact against dnglab"
  explicitly_does_not:
    - "Spawn threads or add a thread-pool dependency: the caller chooses the concurrency"
    - "Split a Compression 7 scan that has no restart markers: no held file has any"
    - "Change any unpack output: every existing plane oracle (dnglab, layer-0, tier-A) stays byte-identical"
    - "The develop path (STAGE-019 shipped it) or a release"

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

# STAGE-020: Parallel-friendly plane decode: unit-level unpack

## What This Stage Is

The sensor-plane decode becomes splittable and faster. Today `plane::unpack_into` decodes
the whole plane in one call on one thread. After this stage a caller asks
`plane::units(sensor)` for a list of independent units: a rectangle of the plane plus the
bytes it reads. It decodes each with `plane::unpack_unit_into` on whatever threads it
likes, in any order, and gets output byte-identical to `unpack_into`. irradiance itself
stays single-threaded and dependency-free. Alongside, the two single-stream kernels the
units cannot split are each sped up behind a profile:
- the packed 12/14-bit uncompressed unpack;
- the Compression 7 lossless-JPEG decode.

This is monorail-000 / onyx-000 request 1, plus item 1b from the triage
(`feedback/2026-11-11-monorail-000-requests.md`).

## Why Now

It is third and last in the order agreed with the requester on 2026-11-12, after
STAGE-018 (previews) and STAGE-019 (develop options), both shipped. It depended on
SPEC-058's tiled reader, which shipped in STAGE-015. The requester's spike says opening a
file is dominated by unpack: 240 ms of an 858 ms Q2M open, and 703 ms on the K-3 III Mono.
The framing probe below reproduces both figures on the held files.

## Success Criteria

- **Units assemble byte-identically.** For every held file and every tier-A layout, all
  units of `units(sensor)` decoded into one plane in **any** order equal `unpack_into`.
  The layouts are: uncompressed 8/12/14/16-bit, multi-strip, tiled, and Compression 7
  with and without restart intervals. A test shuffles the order, and a red-proof with one
  unit's rectangle off by one row goes red.
- **The units split where it matters.** The Q2M (one uncompressed strip, 47 MP) yields
  more than one unit, row bands at whole-byte row boundaries. The Pixel files yield one
  unit per tile (192). The single-scan Compression 7 files (K-3 III, Typ 246) yield one
  unit, and say why in the API doc.
- **`unpack_into` uses the same units.** There is one code path, so the two cannot drift.
  This is pinned by the existing plane oracles staying green unedited.
- **No allocation in `unpack_unit_into`.** No new dependency, no threads, no `unsafe`,
  typed errors in `ErrorKind`, `#[non_exhaustive]` public types, and a fuzz target over
  units.
- **Measured speed.** A committed release timing example (the `just orient-timing`
  pattern) records unpack time before and after on every held file. Any kernel change is
  byte-identical and keeps the dnglab oracle green for Compression 7.

## Scope

### In scope
- **A unit type and the two functions:**
  - `plane::units(sensor)`, or a `units_with` that takes a band height. Design decides.
  - `plane::unpack_unit_into`.
  - A DEC for the unit shape, and for the rule that `unpack_into` iterates the units.
- **The unit kinds:** uncompressed row bands, strips, tiles (reusing DEC-056's clipped
  strided window), and Compression 7 restart intervals where a file has them (`src/ljpeg.rs`
  already decodes restarts).
- **A profile-gated speed-up** of the packed-bits unpack for whole-byte rows.
- **A profile-gated speed-up** of the single-stream Compression 7 decode.
- **A release timing example, CHANGELOG, provenance rows** for any new kernel, and fuzz
  seeds.

### Explicitly out of scope
- Threads, rayon or any concurrency inside irradiance.
- SIMD intrinsics or `unsafe`. Portable code the compiler can vectorise is fine.
- Splitting a restart-free Compression 7 scan.
- The develop path, the colour path, and a release.
- MaskedAreas (item 8) and the STAGE-019 follow-ups.

## Spec Backlog

Ordered list of specs composing this stage. Add specs as you identify
them. Update status as specs progress.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-NNN` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. An optional `[S]`/`[M]`/`[L]` tag on the
line is carried onto the spec as `task.complexity`.

Order: the units spec first; the two speed-ups are independent of each other. The
packed-bits kernel depends on the units spec, because it should speed up the per-unit
kernel the units call, not a path the units bypass. The Compression 7 speed-up does not
depend on it.

- [ ] SPEC-068 (build) [M] — Plane units (DEC-063): every unit a full-width row band (bands inside uncompressed strips with whole-byte rows, else the whole strip; one tile row per unit for tiled planes; a whole Compression 7 strip), so a caller splits dst with split_at_mut and needs no unsafe; plane::units(sensor, file, max_band_rows) + unpack_unit_into; unpack_into iterates the same units through one kernel. Restart-interval units deferred (no held file has any).
- [ ] SPEC-069 (frame) [S] — Faster single-threaded packed-bits unpack, profile-gated: the Q2M 14-bit unpack takes 238 ms for 47 MP; profile first, and if the per-sample bit loop dominates, add a byte-group fast path for 12/14-bit whole-byte rows; byte-identical on every held file and tier-A width; before/after from a committed release timing example. Depends on the units spec. **SPEC-068 design probe:** an independent u64-accumulator 14-bit unpack of the Q2M strip took 44.7 ms against the crate's 238 ms, byte-identical (5.3x single-threaded).
- [ ] SPEC-070 (frame) [M] — Faster single-stream Compression 7 decode, profile-gated: K-3 III Mono 704 ms and Typ 246 601 ms, both single scans with no restart markers, so units cannot split them; profile src/ljpeg.rs (Huffman decode, prediction), change only what the profile names; bit-exact against dnglab (SPEC-044's oracle) and every lossless-JPEG test; before/after timed.

**Count:** 0 shipped / 3 active / 0 pending

## Design Notes

**Framing probe (main at 183af32, 2026-11-13; Apple M4 Pro, release build, `unpack_into`
median of 5, a throwaway example not committed):**

| File | Size | Bits | Compression | Layout | Row bits % 8 | Unpack |
|---|---|---|---|---|---|---|
| Q2M L1021223 / L1026016 / L1026192 | 8424×5632 | 14 | uncompressed | 1 strip | 0 | **238 ms** |
| K-3 III Mono | 6304×4224 | 14 | Compression 7 | 1 strip | 0 | **704 ms** |
| M Typ 246 | 5984×4000 | 12 | Compression 7 | 1 strip | 0 | **601 ms** |
| Pixel 3a / 4a / 7 Pro | 4032×3024 (7 Pro 3882×2924) | 16 | Compression 7 | 192 tiles of 256×256 | 0 | 303–335 ms |
| M Monochrom L1000622 | 5216×3472 | 16 | uncompressed | 1 strip | 0 | 0.6 ms |
| Canon 5D3 | 5920×3950 | 16 | uncompressed | 1 strip | 0 | 0.8 ms |
| Nikon D300S | 4352×2868 | 16 | uncompressed | 12 strips (rps 256) | 0 | 0.4 ms |

- **Every held file's row ends on a whole byte** (`width · bits % 8 == 0`). That answers
  the one question the requester reply left for framing: uncompressed row bands can start
  at a computed byte offset. A width that does not end on a whole byte must still be
  handled (tier A), as a whole strip or with a bit offset. Design decides.
- **Both single-strip Compression 7 files are one scan with no restart interval.**
  Walking the entropy-coded data found no `DRI` and no `RST0–7`:
  - K-3 III: SOF3 P=14, 4224×3152, Nf=2, the requester's reading;
  - Typ 246: P=12, 4000×2992, Nf=2. This one is new; the requester did not name it.

  Units cannot split them. Their 600–700 ms is the reason for the third spec.
- **16-bit uncompressed is already about 1 ms** (a byte copy). **14-bit packed is
  238 ms**, about 5 ns per sample, so the bit loop, not memory, is the likely cost. That
  is why the second spec exists; the profile decides.
- **The Pixel files' 192 tiles are real units** (SPEC-058 decodes them). Their develop
  still refuses an unrecognised opcode 9, but unpack and units work.
- **Writing disjoint units into one plane safely is the main design question.** Row
  bands can use `split_at_mut` on rows. Tiles are column-disjoint, which `&mut [u16]`
  cannot express without `unsafe`. Options to weigh at design:
  - the unit writes into its own `w × h` buffer, and the caller copies it;
  - the unit takes a strided window, with the caller splitting by tile rows;
  - an API that hands out disjoint row-band views only, with tiles grouped by tile row.

  The units spec's DEC fixes it.

## Dependencies

### Depends on
- STAGE-015 (shipped): SPEC-058's tiled reader and DEC-056's clipped strided tile
  window.
- STAGE-009/PROJ-003 (shipped): the lossless-JPEG decoder with restart intervals
  (SPEC-037, SPEC-044's dnglab oracle).
- DEC-016 (caller-owned buffers), DEC-051, DEC-052, DEC-003.

### Enables
- monorail-000 / onyx-000: parallel unpack on their own threads, and a faster
  single-threaded open. This is the last of their seven requests except MaskedAreas
  (item 8).
- A 0.2.x release carrying STAGE-018, STAGE-019 and this stage.

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
