---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-009                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-003                      # parent project
repo:
  id: irradiance

created_at: 2026-11-05
shipped_at: 2026-11-05

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
    - "No DNG container binding (recognise Compression 7, feed the strip/tile plane path) — that is STAGE-011"
    - "No real-file dnglab --raw-checksum verification — STAGE-011"
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
  sessions:
    - tokens_total: 191124332
      estimated_usd: 402.28
      recorded_at: 2026-11-05
      notes: "Stage grain (DEC-013 §5), not split across specs. ONE orchestrator session (work-status-next-steps, scratchpad uuid c0b7ec9b-2be4-41ad-9dea-77f3b6690343) ran STAGE-009 end to end in the main session: the PROJ-003 brief + STAGE-009 frame + the ID-collision resolution it triggered (a full PROJ-002 renumber to STAGE-011/SPEC-039-042, #95) + the design AND ship of all three specs + dispatching/reconciling their 6 cold build/verify sessions + this stage close. Measured FLOOR from that session's OWN transcript (scratchpad-uuid anchored, NOT text grep), deduped by message.id, all claude-opus-4-8, 98.1% cache-read: 410 usage-bearing messages dated >= 2026-11-04T00:00Z -> in 820 / out 923,539 / cache-write 2,765,435 / cache-read 187,434,538 = 191,124,332 tokens, priced per-component at Opus list rates (15/75/18.75-write/1.50-read per MTok) = 402.28 USD. SCOPED: STAGE-007's entry already claimed this file through its 2026-11-01 close; STAGE-008's colour work (SPEC-033/034, 11-02/03) is EXCLUDED here (STAGE-008 is not yet shipped and will claim it). INCLUDES the STAGE-009-framing-induced PROJ-002 0.2.0 reconcile (#95) as framing overhead rather than orphan it (PROJ-002 STAGE-011's own close must NOT re-claim #95). Floor: the push + ship-PR turns post-date the measurement. RATIO: 191.1M orchestration against 72.9M delegated spec cost (SPEC-036 28.5M + SPEC-037 19.4M + SPEC-038 25.0M grand totals) = ~72% of the stage's total spend is orchestration no spec recorded — higher than STAGE-006 (40%) / STAGE-007 (61%) because the ID-collision reconcile + the PROJ-003 project brief + all three specs' un-metered design AND ship cycles all ran in this main session."
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
STAGE-011 then wires it to real DNG strips.

## Why Now

It is the decode engine the whole project rests on, and the part with **no
container entanglement** — so it can be built and proven in isolation, against
an oracle (the round-trip) that needs no corpus file at all. Building the pure
decoder first and binding it second (STAGE-011) keeps the hard algorithm
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
  plane path) — **STAGE-011**
- Real-file `dnglab --raw-checksum` verification — **STAGE-011**
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
- [x] SPEC-037 (shipped 2026-11-05) — [M] Lossless reconstruction (predictors, point transform, restart)
- [x] SPEC-038 (shipped 2026-11-05) — [S] SOF-3 decoder fuzz + truncation hardening

**Count:** 3 shipped / 0 active / 0 pending — **stage complete.**

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
  bit-identical — the corpus-free independent check (STAGE-011 adds the real-file
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
- **STAGE-011** — the DNG `Compression 7` binding feeds this decoder real strip
  bytes and verifies its output bit-exact vs `dnglab --raw-checksum`

## Stage-Level Reflection

*Drafted at ship (Prompt 1d), 2026-11-05.*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes.** `src/ljpeg.rs`
  is a self-contained, container-agnostic single-component SOF-3 (lossless JPEG,
  T.81 Annex H) decoder: it parses the marker/frame structure
  (SOI/SOF3/DHT/SOS/DRI/RSTn/EOI), builds and applies the DC Huffman table, and
  reconstructs samples through predictors 1–7, the initial prediction, the point
  transform, and restart resets — SPEC-036 (entropy front-end) + SPEC-037
  (reconstruction), proven **bit-exact** by the analytic encode→decode round-trip
  oracle with a perturbed-decoder red-proof, and SPEC-038 proving it **panic-free**
  on hostile input (3,098,220+ fuzz runs, no crash; typed `Ljpeg*` errors on every
  truncated / oversized-table / forged-dimension bitstream). No DNG container
  binding and no real-file `dnglab --raw-checksum` — both correctly deferred to
  STAGE-011, as framed.
- **How many specs did it actually take?** **3, exactly as framed**
  (SPEC-036/037/038). The pre-named L-risk split on SPEC-036 (marker/frame parse
  as one spec, Huffman build+decode as the other) was **not** needed — it designed
  as M and built whole.
- **How many outlines survived unchanged?** **3 of 3.** Every framed outline held
  scope from frame to ship (entropy front-end; reconstruction; fuzz + hardening).
  Framing paid for itself — in contrast to STAGE-006, which grew 4→6 under a
  corpus-assumption surprise; STAGE-009 was corpus-free by design (its oracle is
  the in-harness round-trip encoder), so nothing forced a precursor split.
- **What changed between starting and shipping?** Nothing structural — the one
  design-time decision that mattered (DEC-036: expose the `pub(crate)` decoder to
  the fuzz crate via a `fuzzing` cargo **feature**, not `cfg(fuzzing)`, because the
  `clippy — fuzz graph` gate builds with plain `cargo clippy`) was pinned by a §12
  probe before SPEC-038 built, so there was no mid-stage pivot; SPEC-037's verify
  raised one coverage punch-list (the shared-helper oracle tautology, `floor_div2`)
  that was resolved at its own ship.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - The "probe the hard assumption at design/frame time" discipline paid off
    directly: STAGE-009 had **zero mid-stage pivots** because DEC-035 (SOF-3
    patent-clean) and DEC-036 (the fuzz-graph-gate mechanism) were both settled by
    §12 probes before building. **Candidate to land now:** the cheap Prompt 1c
    patent-check line proposed under `patent-check-named-algorithms-at-frame` —
    STAGE-009's proactive DEC-035 is the positive counter-instance to STAGE-007's
    MHC miss (still below the N=3-same-outcome bar, so left `watch`).
  - The desktop-no-interactive-`/cost` cost-recovery practice (recover the true
    cumulative from each cold transcript, deduped by message.id at Opus list rates,
    supersede the context-occupancy handback floor at ship) is now routine across
    SPEC-036/037/038 and this stage's orchestration entry. The
    `desktop-cost-tool-unavailable-handbacks-null` process-debt is well-characterized
    (project-close owned) — no new action here.
- **Signals dispositioned at this close?** (Prompt 1d step 7) **Yes — all five
  `disposition_at: stage-close` lessons still at `watch` were walked; none silently
  carried, none reached bar, all remain `watch` with `last_touched: 2026-11-05`:**
  - `shared-path-oracle-tautology` (stage-born, SPEC-037) — N=1, bar N=2. No second
    instance in STAGE-009; SPEC-038's verify instead **applied** the discipline
    (confirmed the replay tests bite — a distinct typed variant per hostile shape,
    corpus asserted non-empty). Kept `watch`.
  - `patent-check-named-algorithms-at-frame` — N=1 (STAGE-007 miss). STAGE-009 is
    the **positive** counter-instance (DEC-035 checked at design, no landmine), not
    a same-outcome miss; bar is N=3 same-outcome, so kept `watch`. Proposed the
    cheap Prompt 1c line above.
  - `probe-the-hard-assumption-at-frame-time` — N=1 (STAGE-006 corpus miss). No
    STAGE-009 miss (corpus-free stage); SPEC-038's §12 fuzz-graph-gate probe is a
    positive application. Kept `watch`.
  - `a-fix-inherits-the-precondition-of-the-thing-it-fixes` — N=2, needs N=3 on
    DIFFERENT code. STAGE-009 added no instance (new-capability decoder + a fuzz
    target, not a fix hardening failing logic). Deferred, kept `watch` at N=2.
  - `saturating-as-cast-masks-explicit-clamp-in-quantize-tests` — N=1 (PROJ-002
    SPEC-033). No instance in STAGE-009 (integer u16 prediction, no `f64 as u16`
    saturating cast); its own decision point is a PROJ-002 STAGE-008 close. Kept
    `watch`.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - `shared-path-oracle-tautology` is the one stage-born lesson (SPEC-037); recorded
    as a `watch` signal at N=1 (bar N=2) — a round-trip oracle is blind to a helper
    both halves share. **No promotion yet**; the convention lands at a confirmed
    second instance.
