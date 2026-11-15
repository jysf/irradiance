---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-020                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-13
shipped_at: 2026-11-14

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
  sessions:
    - tokens_total: 65677483
      estimated_usd: 48.00
      recorded_at: 2026-11-14
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'Frame parallel unpack' message at 2026-11-14T04:19Z through this stage close: 266 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 616 + cache-creation 985,460 (1-hour) + cache-read 64,454,916 + output 236,491); no subagents. Covers framing STAGE-020, three design probes and DEC-063..065, six build/verify dispatches and reconciliations, three ships, the plane_units_alloc race fix (PR 243) and this close. Dominated by cache reads (98%). A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
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

- [x] SPEC-068 (shipped on 2026-11-14) [M] — Plane units (DEC-063): every unit a full-width row band (bands inside uncompressed strips with whole-byte rows, else the whole strip; one tile row per unit for tiled planes; a whole Compression 7 strip), so a caller splits dst with split_at_mut and needs no unsafe; plane::units(sensor, file, max_band_rows) + unpack_unit_into; unpack_into iterates the same units through one kernel. Restart-interval units deferred (no held file has any).
- [x] SPEC-069 (shipped on 2026-11-14) [S] — Faster single-threaded packed-bits unpack, profile-gated: the Q2M 14-bit unpack takes 238 ms for 47 MP; profile first, and if the per-sample bit loop dominates, add a byte-group fast path for 12/14-bit whole-byte rows; byte-identical on every held file and tier-A width; before/after from a committed release timing example. Depends on the units spec. **Shipped (DEC-064, PR 238 as 33f5deb):** whole byte groups (7 bytes to 4 samples for 14-bit, 3 to 2 for 12-bit) with the unchanged BitReader reading the tail; Q2M single-thread `unpack_into` 239.0 -> 9.7 ms (verify 233.4 -> 9.9), 8 units on 8 threads 31.2 -> 1.4 ms, byte-identical; true cost 17.9M / $13.50 against 20M.
- [x] SPEC-070 (shipped on 2026-11-14) [M] — Faster single-stream Compression 7 decode, profile-gated: K-3 III Mono 704 ms and Typ 246 601 ms, both single scans with no restart markers, so units cannot split them; profile src/ljpeg.rs (Huffman decode, prediction), change only what the profile names; bit-exact against dnglab (SPEC-044's oracle) and every lossless-JPEG test; before/after timed. **Shipped (DEC-065, PR 241 as d27c50e):** a u64-buffered ScanReader whose fill stops silently at markers, 11-bit lookup and fast Huffman tables, and today's bit-at-a-time path as the cold fallback; K-3 III 692 -> 103 ms (verify 710 -> 105), Typ 246 591 -> 91 ms (verify 605 -> 90), Pixel 4a 324 -> 61 ms; samples and errors identical (verify corpus differential: 3,552 mutated units, 0 differences); true cost 31.6M / $22.25 against 30M.
- [~] (fix in PR 243) [XS] **PRIORITY** — SPEC-070 follow-up, promoted at ship: `tests/plane_units_alloc.rs` counts allocations on one global `AtomicUsize` while its tests run in parallel, so `q2m_units_allocate_nothing` can fail when `counter_counts` allocates inside its window (seen once in the design probe; the SPEC-070 verify saw it fail 4 of 4 parallel local runs with the corpus at the build head, against 0 of 3 at 8da45d7, while passing alone and serially; CI has no corpus, so only local `just test` goes red). Count per thread, or serialise the file's tests. **Fixed in PR 243** (per-thread tally, the `tests/develop_scratch_alloc.rs` pattern); its corpus-gated local run is pending a free machine.
- [ ] (unframed) [XS] — SPEC-068 verify follow-up: pin the absolute Compression 7 allocation count per strip or tile in `tests/plane_units_alloc.rs` (about 10 from the `ljpeg` header parses), so a new allocation in the shared kernel is caught by the count, not only by the old-equals-new comparison and the source scan.
- [ ] (unframed) [XS] — SPEC-069 follow-up: the `inject_chunk_extraction_fault` assert message in `tests/plane_oracle.rs` still describes the old BitReader needle; reword it to name the group kernel.
- [ ] (unframed) [XS] — SPEC-070 follow-up: add a `just fuzz-ljpeg` recipe that creates `fuzz/corpus/ljpeg` first; the target's header command fails on a fresh checkout because that gitignored dir does not exist.
- [ ] (unframed) [XS] — SPEC-069 follow-up: AGENTS.md section 6 lists no timing examples; add `just orient-timing` and `just unpack-timing` (release, corpus-gated, no CI gate).

**Count:** 3 shipped / 0 active / 0 pending (all three specs shipped; the race fix is PR 243; the other four [XS] lines are carried to the PROJ-004 brief. SPEC-068 39.44M tok / $28.11; SPEC-069 17.87M / $13.50; SPEC-070 31.65M / $22.25)

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

*Filled at stage close, 2026-11-14.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes, with two documented
  narrowings.
  - **Plane units** (SPEC-068, DEC-063): `plane::units` and `plane::unpack_unit_into`.
    Every unit is a full-width row band, so a caller splits `dst` with `split_at_mut` in
    safe Rust and assembles byte-identically in any order. On the Q2M, 8 units on 8
    threads took 31 ms against 242 ms on one.
  - **Packed 12/14-bit unpack** (SPEC-069, DEC-064): whole byte groups with the
    unchanged `BitReader` for the tail. The Q2M single-thread unpack went from 239 ms to
    9.7 ms, and 8 units from 31 ms to 1.4 ms.
  - **Compression 7 decode** (SPEC-070, DEC-065): a buffered scan reader plus 11-bit
    Huffman lookahead tables, with today's bit-at-a-time path as the cold fallback. The
    K-3 III went from 692 to 103 ms, the Typ 246 from 591 to 91 ms, and the tiled Pixel 4a
    from 324 to 61 ms. Samples and errors are identical.

  **The narrowings.**
  - The framing asked for "a rectangle of the plane plus the bytes it reads". A unit
    became a full-width row band with a private source, because a sub-rectangle cannot be
    handed to a thread safely and a tile-row unit reads several byte ranges (DEC-063).
  - Compression 7 restart-interval units were framed but not built. No held file, and
    neither of the requester's Compression 7 cameras, has a restart interval (DEC-063 §4).

  **Every success criterion holds.** Every plane oracle (dnglab, layer-0, tier A) stayed
  green unedited, apart from SPEC-069's one permitted injector retarget. irradiance
  spawns no threads, and there is no new dependency.
- **How many specs did it actually take?** Three, as planned.
  - Sizes: M, S, M, all as estimated.
  - Total spec cost: 88.96M tokens / $63.86 (SPEC-068 39.44M, SPEC-069 17.87M,
    SPEC-070 31.65M).
  - Orchestration: 65.68M / $48.00 (floor).
- **How many outlines survived unchanged?** 2 of 3 (SPEC-069 and SPEC-070, both
  profile-gated as framed). SPEC-068 changed at design: row bands instead of
  rectangles, and no restart-interval units.
- **What changed between starting and shipping?** The two "if the profile shows it"
  speed-ups both turned out to be most of the stage's value. Each profile put about 90%
  or more of the time in one bit-at-a-time loop, and the single-threaded decode is now
  5–25× faster, so many callers need no units at all.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **None codified.** No lesson reached its bar this stage.
  - **Practice that worked three times:** a performance spec states an absolute target
    on a fixed baseline commit, times it with a committed release example (no CI gate),
    and makes byte identity, and for SPEC-070 error identity, the hard criterion. Each
    verify reproduced the timing within a few percent.
  - **Practice that worked:** for a kernel rewrite, the oracle is a verbatim test-local
    copy of the old code, driven on synthetic inputs the corpus never produces (SPEC-069
    AC3, SPEC-070 AC2/AC3). It proves "no change", which is the claim.
- **Signals dispositioned at this close?** Every `disposition_at: stage-close` signal in
  `watch` was walked.
  - **Decided:** `slow-tier-b-tests-in-debug` (process-debt, N=2). A release-profile
    tier-B lane is carried to the PROJ-004 brief as an [S] follow-up; the signal stays
    `watch` until it lands.
  - **Stay `watch`, applied, not violated:** `probe-the-hard-assumption-at-frame-time`
    (N=1), `read-the-kernel-before-deciding-the-approach` (N=1).
  - **Stay `watch`, residual risk unchanged:** `unexercised-sof3-paths-garble-silently`.
    SPEC-070 kept the restart behaviour identical to the old reader, which proves no
    change, not that the T.81 restart rule is right.
  - **Stay `watch`, no new instance:** `a-fix-inherits-the-precondition-of-the-thing-it-fixes`
    (N=2), `patent-check-named-algorithms-at-frame`,
    `saturating-as-cast-masks-explicit-clamp-in-quantize-tests`,
    `hostile-fixture-shadowed-by-an-earlier-guard`,
    `closed-form-numeric-needs-extreme-magnitude-fixtures`,
    `analytic-oracle-premise-must-survive-rounding` (each N=1), and the process-debt
    `confirm-pr-head-advanced-before-treating-pushed-commit-as-merged`.
  - **New, `watch`, N=1:**
    - `parallel-tests-share-a-global-counter`: the `plane_units_alloc` race, fixed in
      PR 243;
    - `spec-must-carry-what-the-probe-number-depended-on`: SPEC-070's spec said
      `#[inline]` where the probe's number needed `#[inline(always)]`.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - **Benign speed-only mutants are guarded only by timing** (SPEC-069 `full − 1`,
    SPEC-070 `LUT_BITS` and the disabled fast path). That is by design for a
    performance spec with no CI timing gate, so it is recorded here, not as a signal.
  - **Handback floors ran low again** (SPEC-069 build +13%, verify +40%; SPEC-070 build
    +23%, verify +10%). Re-summing at ship from the ended transcripts stays the rule.
