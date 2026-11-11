---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-013                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-06   # drafted in PROJ-002 (uncommitted); re-homed to PROJ-004 2026-11-11
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: null                    # one sentence; which part of project.value.thesis
  delivers: []                      # user-visible capabilities this stage delivers
  explicitly_does_not: []           # what this stage is NOT trying to do

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

# STAGE-013: test-fixture DNG writer (clean-room, uncompressed)

## What This Stage Is

A small, clean-room, test-only DNG writer that builds uncompressed
monochrome (1-sample) and Bayer fixtures with analytically known pixels, levels,
CFA pattern, ActiveArea and colour matrices. It exists because `dnglab makedng`
cannot emit a 1-sample uncompressed plane (docs/oracle-contract.md, measured
2026-08-16) and its JPEG output needs the SOF-3 decoder to read back. It lives
under tests/support (or behind a dev-only cfg), not in the public API.

## Why Now

Hand-built headers are the load-bearing route to tier-A monochrome fixtures
(oracle-contract.md), and the SPEC-047 ActiveArea-origin work showed how much
geometry coverage the synthetic corpus lacks. The writer also gives the reader
an encode-then-decode round-trip check. A public writer is deliberately NOT
this stage (see docs/future-plans.md, "Later").

## Success Criteria

Concrete outcomes that would mean this stage succeeded.

- A fixture can be written with chosen dims, bit depth, CFA phase, black/white
  levels, ActiveArea, DefaultCrop, Orientation, colour matrices and opcodes.
- Writer output round-trips through irradiance's own reader bit-exactly, and is
  accepted by an independent reader (dnglab analyze / exiftool) as a sanity check.
- Provenance ledger row added; written from the Adobe DNG spec and TIFF 6.0 only.

## Scope

### In scope
- Uncompressed (Compression 1) strips AND tiles, 1-sample and CFA, little- and big-endian
- Arbitrary OpcodeList1/2/3 payloads (so GainMap and malformed-list fixtures can be built)
- Tags needed by the existing reader and oracles

### Explicitly out of scope
- Any compression (lossless JPEG, deflate, JXL, lossy) - no encoder, no patent question
- Public API / crates.io surface (roadmap item, after 0.2.0)
- Reading or porting any copyleft writer (e.g. the AGPL `dng` crate is off-limits)

## Spec Backlog

Ordered list of specs composing this stage. Add specs as you identify
them. Update status as specs progress.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-NNN` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. An optional `[S]`/`[M]`/`[L]` tag on the
line is carried onto the spec as `task.complexity`.

- [ ] SPEC-057 (frame) [M] test-only DNG writer: uncompressed mono and Bayer fixtures, strip AND tile layouts, arbitrary OpcodeList1/2/3 bytes (GainMap fixtures)

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

Cross-cutting design decisions that span multiple specs within this
stage but aren't weighty enough for their own DEC file. Keep short.

Link to `DEC-*` entries for the weighty decisions. This section is
for glue: shared patterns, naming conventions, small tradeoffs.

Crate layout: see DEC-055 (in-crate behind a `write` feature; split triggers named there).

### Verification plan (written up front)

Oracles, cheapest first:
1. **Round-trip** - write, read back with irradiance, compare bit-exact (planes,
   levels, CFA, ActiveArea, matrices, opcode bytes).
2. **Independent reader** - `dnglab analyze` and `exiftool`, run as black-box CLIs.
3. **Spec conformance** - Adobe `dng_validate` (DNG SDK) as a black-box CLI, same
   discipline as dnglab. Run the binary only: do not read, copy or link SDK source
   (it has its own licence, not OSI-permissive). Check the SDK licence before it is
   named in CI or the provenance ledger.
4. **Real-world** - open a sample in darktable / Lightroom by hand (manual check,
   not a gate; GPL tools are black-box only).

### Reader fidelity (finding, 2026-11-06)

The container layer in `src/ifd.rs` is already lossless: every `Entry` (tag, type,
count, raw value) and every `Ifd` (offset, depth, parent, order) is retained;
only the typed `Sensor` view is selective. So a test-only writer needs no reader
refactor. A future *re-emit* writer (edit-and-save a real DNG) would still need the
container to own its out-of-line payload bytes (today they are borrowed by offset
from the input buffer) and to preserve unknown tags and opcode lists verbatim.
Keep that property when touching ifd.rs.

## Dependencies

### Depends on
- PROJ-001/PROJ-002 reader (shipped) — the lossless container model in `src/ifd.rs`
- External: none (dnglab / exiftool / Adobe `dng_validate` run as black-box CLIs only)

### Enables
- STAGE-015 (tiled reader) — tiled fixtures for the tile path
- STAGE-016 (GainMap) — analytic GainMap fixtures (no held strip file carries a non-trivial one)
- STAGE-017 (public writer) — the clean-room core it promotes
- PROJ-004 instrument stages — banded optical-black and preview-slice fixtures

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
