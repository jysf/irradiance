---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-009                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-003                      # parent project
repo:
  id: irradiance

created_at: 2026-11-05
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    The decode half of the project thesis — a losslessly-compressed SOF-3 plane
    becomes u16 samples bit-exact, the precondition for developing the
    compressed files most RAW actually carries.
  delivers:
    - "A pure-Rust single-component SOF-3 (lossless JPEG, T.81 Annex H) decoder: compressed bytes + frame params → a u16 sample array, panic-free and deterministic"
    - "An analytic encode→decode round-trip oracle (independent of dnglab) with a red-proof, plus a fuzz target over the decoder"
  explicitly_does_not:
    - "No DNG container binding (recognise Compression 7, feed the strip/tile plane path) — that is STAGE-010"
    - "No real-file dnglab --raw-checksum verification — STAGE-010"
    - "No multi-component / Bayer-in-LJ92 interleave (no held file) and no GainMap / opcode work"

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

# STAGE-009: SOF-3 lossless-JPEG core decoder

## What This Stage Is

STAGE-009 delivers the **SOF-3 lossless-JPEG decoder as a self-contained,
container-agnostic function**: given the entropy-coded bytes and the frame
parameters (precision, predictor, point transform, restart interval,
dimensions), it reconstructs the exact `u16` sample array the encoder started
from. It parses the JPEG marker/frame structure (SOI, SOF3, DHT, SOS, DRI,
RSTn, EOI), builds and applies the DC Huffman table, decodes each sample's
magnitude category and difference, and reconstructs samples through predictors
1–7, the initial prediction, the point transform, and restart resets. It is
verified not against a real file yet but against an **analytic encode→decode
round-trip** — the corpus-free independent oracle — and fuzzed for
panic-freedom. When it ships, irradiance can turn LJ92 bytes into a plane;
STAGE-010 then wires it to real DNG strips.

## Why Now

It is the decode engine the whole project rests on, and the part with **no
container entanglement** — so it can be built and proven in isolation, against
an oracle (the round-trip) that needs no corpus file at all. Building the pure
decoder first and binding it second (STAGE-010) keeps the hard algorithm
separate from the DNG plumbing and gives each a distinct oracle (round-trip vs
`dnglab --raw-checksum`) — the same algorithm-then-integration split PROJ-002
used for demosaic (analytic) then colour (real-file). It is fully
chart-independent, so it runs while PROJ-002's SPEC-035 waits on the
ColorChecker chart.

## Success Criteria

- A hand-built or round-trip-encoded LJ92 bitstream decodes to the **exact**
  expected `u16` sample array, exercised across predictors 1–7 and precisions
  8 / 12 / 16-bit
- The analytic encode→decode round-trip is **bit-identical** on honest input and
  the oracle goes **RED** on a deliberately perturbed Huffman table, predictor,
  or point transform (`oracle-must-be-shown-red`)
- The decoder returns a typed `Error` (**never panics**) on malformed markers,
  invalid / oversized Huffman tables, truncated scans, and bad restart intervals
- A libFuzzer target over the decoder runs clean (target run count) on hostile /
  truncated bitstreams
- **Zero new runtime dependencies**; the decoder compiles under
  `#![forbid(unsafe_code)]` and the five-lint deny policy

## Scope

### In scope
- SOF-3 marker/frame parsing (SOI, SOF3, DHT, SOS, DRI, RSTn, EOI) and a JPEG bit
  reader with `0xFF`-byte-destuffing
- DC Huffman table construction (`BITS` / `HUFFVAL`) and symbol decode
  (category `SSSS` → difference via receive + extend)
- Lossless reconstruction: predictors 1–7, initial prediction
  `Px = 2^(P−Pt−1)`, point transform `Pt`, restart-interval reset + byte
  alignment
- Sample precision 8–16 bit, **single component** (`Nf = 1`)
- The analytic encode→decode round-trip oracle + red-proof, and a fuzz target
- Caller-owned `u16` output buffer (DEC-016); wrapping modular arithmetic for
  prediction (no panic under `arithmetic_side_effects`)

### Explicitly out of scope
- The DNG `Compression 7` binding (strip/tile assembly, byte order, feeding the
  plane path) — **STAGE-010**
- Real-file `dnglab --raw-checksum` verification — **STAGE-010**
- Multi-component / Bayer-in-LJ92 interleave (no held file) and GainMap / opcode
  work
- Lossy DCT JPEG (SOF-0/2), arithmetic coding (SOF-9/10/11), Nikon NEF-lossless,
  Pentax PEF

## Spec Backlog

Ordered list of specs composing this stage.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-009` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. The `[S]`/`[M]`/`[L]` tag is carried onto
the spec as `task.complexity`.

- [x] SPEC-036 (shipped 2026-11-05) — [M] SOF-3 entropy front-end (markers, bit reader, DC Huffman)
- [ ] SPEC-037 (frame) — [M] Lossless reconstruction (predictors, point transform, restart)
- [ ] SPEC-038 (frame) — [S] SOF-3 decoder fuzz + truncation hardening

**Count:** 1 shipped / 2 active / 0 pending

## Design Notes

- **Caller-owned output (DEC-016):** the decoder writes into a `&mut [u16]` the
  caller sizes from the SOF3 frame header (`X·Y·Nf`); no allocation on the decode
  path, consistent with unpack / demosaic / colour.
- **u16 sample path (DEC-018).** Prediction is modular `2^16` arithmetic per T.81
  — use `wrapping_add` / `wrapping_sub` explicitly, so the
  `arithmetic_side_effects` deny lint is satisfied by intent, not silenced by a
  cast.
- **The analytic oracle is a tiny LJ92 encoder in the test harness:** encode a
  known sample array with a chosen predictor / `Pt` / precision, decode, assert
  bit-identical — the corpus-free independent check (STAGE-010 adds the real-file
  `dnglab --raw-checksum`). Cross-read `oxideav-mjpeg` (MIT) and `photocraft-raw`
  (MIT/Apache) for structure only; never copy (clean-room, and their claims are
  unverified).
- **Patent posture:** SOF-3 lossless Huffman JPEG is ITU-T T.81 (1992) — believed
  unencumbered. A load-bearing patent check is a design-time action on the first
  spec (DEC-031 lesson), recorded before building.
- **Provenance-ledger:** a row for the SOF-3 decode algorithm (T.81 Annex H) at
  the reconstruction spec.
- **Errors** extend the existing `Error` enum with LJ92-specific typed variants;
  every malformed-input path returns a typed error, never a panic (the
  RAW-is-attacker-influenced invariant).
- **Pre-named L-risk split (SPEC-036):** the entropy front-end bundles
  marker/frame parse + the JPEG bit reader (`0xFF` destuffing) + DC Huffman table
  build and symbol decode (category → difference). If it proves L-sized at
  design, split along the pre-named seam: marker/frame parse as one spec, Huffman
  build + decode as the other. The fuzz spec (SPEC-038) seeds its corpus from the
  round-trip encoder that SPEC-037 builds.

## Dependencies

### Depends on
- **PROJ-001** — the `Error` model, the `u16` plane conventions, and the
  test / oracle / fuzz harness patterns; the decoder is new code but slots into
  these
- External: **ITU-T T.81 (1992) Annex H** — the lossless JPEG spec, implemented
  from the published standard
- Cross-check references (**read-only, never linked**): `oxideav-mjpeg` (MIT),
  `photocraft-raw` (MIT/Apache)

### Enables
- **STAGE-010** — the DNG `Compression 7` binding feeds this decoder real strip
  bytes and verifies its output bit-exact vs `dnglab --raw-checksum`

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
