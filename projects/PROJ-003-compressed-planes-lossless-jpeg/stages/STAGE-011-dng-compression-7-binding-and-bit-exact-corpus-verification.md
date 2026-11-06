---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-011                     # stable, zero-padded, continuous across the repo
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
    The integration half of the project thesis — a losslessly-compressed
    Compression-7 sensor plane becomes the SAME bit-exact u16 plane an
    uncompressed file yields, verified against an independent implementation
    (dnglab), so irradiance develops the compressed files most real RAW carries.
    STAGE-009 built the decoder in isolation; this stage makes it reachable on
    real files and closes PROJ-003.
  delivers:
    - "A DNG Compression 7 (SOF-3 lossless JPEG) file stops returning UnsupportedCompression and decodes to a monochrome u16 plane through the existing strip/tile plane path, for both container byte orders (big-endian Leica M Typ 246 + little-endian Pentax K-3 III)"
    - "Bit-exact verification of both held Compression-7 DNGs against `dnglab analyze --raw-checksum`, with the decoded plane flowing through the existing develop/tone pipeline unchanged — a source swap, not a second pipeline"
  explicitly_does_not:
    - "No Bayer / CFA COLOUR multi-component (3+ colour components in one frame) — no held colour file, no bit-exact oracle; deferred (DEC-037, PROJ-003 roadmap). NOTE: the held files' monochrome TWO-component column-interleave IS now in scope (SPEC-046, DEC-037)."
    - "No OpcodeList2 GainMap, no lossy-DCT JPEG (SOF-0/2), no arithmetic coding (SOF-9/10/11), no Nikon NEF-lossless, no Pentax PEF (Compression 65535) — other waves"
    - "SPEC-046 extends the shipped SOF-3 decoder to two-component column-interleave (DEC-037); otherwise this stage does not change STAGE-009's single-component reconstruction algorithm"
    - "No public CLI — `irr` stays the internal dev/oracle binary"

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

# STAGE-011: DNG Compression 7 binding and bit-exact corpus verification

> **Note on the stage id.** PROJ-003's brief and STAGE-009 originally called this
> binding wave "STAGE-010". That id was claimed by PROJ-002's downstream/0.2.0
> reconcile (PR #95) after PROJ-003's brief was written — stage ids are globally
> sequential across the repo — so the binding wave is **STAGE-011**. Earlier
> PROJ-003 artifacts that still say "STAGE-010" (the shipped SPEC-036/037/038 in
> `specs/done/`, their handoffs, and the cycle prompts) are frozen historical
> records; the authoritative roadmap pointers (this file, the brief, STAGE-009's
> `Enables`, DEC-036, the CHANGELOG, signals) read STAGE-011.

## What This Stage Is

STAGE-011 makes the SOF-3 decoder **reachable on real files**. STAGE-009 shipped
`ljpeg::decode_into` as a container-agnostic, `pub(crate)` function proven
bit-exact by an analytic round-trip and panic-free by fuzzing, but nothing in the
crate calls it — a `Compression 7` DNG still returns `UnsupportedCompression`.
This stage recognises `Compression == 7` on the sensor IFD and routes the
strip/tile bytes through `decode_into` where the packed-bits unpack sits today
(reusing SPEC-028's multi-strip assembly), for **both** container byte orders,
then proves the result **bit-exact** against `dnglab analyze --raw-checksum` on
both held Compression-7 DNGs. The decoded `u16` plane is identical in shape to the
uncompressed path, so develop / tone / geometry are untouched — the decoder is a
**source swap, not a second pipeline**. When it ships, irradiance develops the
compressed files people actually have, and PROJ-003 closes.

## Why Now

The decoder is built and proven in isolation (STAGE-009); binding it is the
payoff that realises the project thesis, and it is the natural next step now that
the hard algorithm is done. It is **fully chart-independent** — the oracle is
`dnglab --raw-checksum` on two files already in the corpus — so it runs now, in
parallel with PROJ-002's chart-blocked SPEC-035. Building the decoder first and
binding it second kept the hard algorithm separate from the DNG plumbing and gave
each its own oracle (analytic round-trip for STAGE-009; real-file `dnglab` here) —
the same algorithm-then-integration split PROJ-002 used for demosaic then colour.

## Success Criteria

- Both held `Compression 7` DNGs (Leica M Typ 246 big-endian, Pentax K-3 III
  little-endian) unpack to a monochrome `u16` plane whose MD5 **equals**
  `dnglab analyze --raw-checksum`, exactly — the bit-exact contract survives the
  move to a compressed plane, for both byte orders
- `Compression == 7` is recognised on the sensor IFD and routed to the SOF-3
  decoder; the uncompressed / monochrome / Bayer paths are **byte-for-byte
  unchanged** (a source swap, not a second pipeline)
- A `Compression 7` plane, once decoded, flows through the existing develop/tone
  path unchanged — demonstrated by an end-to-end develop on a held Compression-7
  file
- The binding path is **panic-free** on hostile / truncated / malformed-strip
  `Compression 7` input (typed `Error`, never a panic), under
  `#![forbid(unsafe_code)]` + the five-lint deny policy
- The same compressed DNG produces **byte-identical** plane output on macOS and
  Linux (determinism survives the bitstream decoder)
- The DNG `Compression 7` binding carries a provenance-ledger entry; **zero new
  runtime dependencies**

## Scope

### In scope
- **`Compression == 7` recognition + routing:** accept `Compression 7` on the
  sensor IFD (where it is rejected today) and route the strip/tile bytes through
  `ljpeg::decode_into`, slotting in where the packed-bits unpack sits in the plane
  path (reuse SPEC-028 multi-strip assembly). Promote `ljpeg` from `pub(crate)` to
  a real caller (the DEC-036 `fuzzing` feature + `__fuzz_ljpeg` hook may retire —
  DEC-036 Validation)
- **Both container byte orders** (big-endian Typ 246 + little-endian K-3 III)
- A **bit-exact tier-B plane oracle** vs `dnglab analyze --raw-checksum` on both
  held files, with a red-proof (a perturbed decoded plane / predictor goes red)
- An **end-to-end develop smoke** proving the decoded plane flows through the
  existing develop/tone path unchanged
- **Provenance-ledger** entry for the DNG Compression-7 binding, and a
  **determinism** assertion (byte-identical macOS/Linux)
- **Panic-free** binding path on hostile / truncated / malformed `Compression 7`
  strips (container→decoder boundary)

### Explicitly out of scope
- **Multi-component / Bayer-in-LJ92 interleave** — no held file, no bit-exact
  oracle; deferred until one is acquired (PROJ-003 roadmap)
- **OpcodeList2 `GainMap`** (flat-field / vignetting) — opcode application, not
  plane decode; no held file carries one
- **Lossy DCT JPEG** (SOF-0/2), **arithmetic coding** (SOF-9/10/11), **Nikon
  NEF-lossless**, **Pentax PEF** (`Compression 65535`) — different formats, their
  own waves
- **The SOF-3 decoder algorithm** itself (STAGE-009, shipped) — this stage only
  feeds it real bytes
- **A public CLI** — `irr` stays the internal dev/oracle binary

## Spec Backlog

Ordered list of specs composing this stage.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-011 PROJ-003` promotes every `(not yet written)` line
below into a real spec at `cycle: frame` with a stable ID (so `depends_on:` can
point at it) and rewrites the line in place. The `[S]`/`[M]`/`[L]` tag is carried
onto the spec as `task.complexity`. **Minted 2026-11-05; re-planned 2026-11-05**
after SPEC-043's build found both held files are two-component (Nf=2) SOF-3 — a new
SPEC-046 (Nf=2 decode, DEC-037) is inserted before the oracle, so the chain is now
SPEC-043 → SPEC-046 → SPEC-044 → SPEC-045.

- [~] SPEC-043 (verify) — [M] Compression-7 recognition + strip-path routing: build
  merged (PR #112); cold verify in flight. `plane::unpack_into` dispatches on
  `Compression` — uncompressed unchanged, `Compression 7` routed per-strip to
  `ljpeg::decode_into`, else `UnsupportedCompression`. Both held files now REACH the
  decoder (and hit the Nf=2 finding below).
- [ ] SPEC-046 (build) — [M] Two-component (Nf=2) SOF-3 decode + column de-interleave
  (DEC-037): extend `ljpeg` to decode the two column components of the held files'
  Nf=2 frames into one monochrome `u16` plane, with an analytic two-component
  round-trip red-proof; the Nf=1 path stays byte-identical. depends_on SPEC-043.
- [ ] SPEC-044 (frame) — [M] Bit-exact corpus oracle: both held Compression-7 DNGs
  (Leica M Typ 246 big-endian + Pentax K-3 III little-endian) decode to a `u16` plane
  whose MD5 equals `dnglab analyze --raw-checksum`, proven red; an end-to-end develop
  smoke proves the decoded plane flows through the existing develop/tone path
  unchanged. **depends_on SPEC-046** (the held files are Nf=2 — they do not decode
  until SPEC-046 lands).
- [ ] SPEC-045 (frame) — [S] Binding capstone: a provenance-ledger entry for the DNG
  Compression-7 binding, a macOS/Linux byte-identical determinism assertion, and
  container→decoder truncation / hostile-strip hardening. depends_on SPEC-044.

**Count:** 0 shipped / 4 active / 0 pending (SPEC-043 at verify; SPEC-046/044/045 framed)

*The Nf=2 finding (SPEC-043 build, DEC-037) re-opened the single-component scope for
the held files' monochrome column-interleave; SPEC-046 is the inserted decode spec. No
entry is flagged L, though SPEC-046 may prove L at design.*

## Design Notes

- **The decoder is a SOURCE SWAP.** It slots in where packed-bits unpack sits in
  the plane path; the decoded `u16` plane is identical in shape to the
  uncompressed path, so develop / tone / geometry / level-normalization are
  untouched (brief: "a source swap, not a second pipeline"). Reuse SPEC-028's
  multi-strip assembly — a Compression-7 plane can be multi-strip.
- **`ljpeg` is `pub(crate)` today (STAGE-009).** This stage gives it its first
  real caller. The DEC-036 `fuzzing` feature + `__fuzz_ljpeg` doc-hidden hook may
  be **retired** once the real public Compression-7 entry exists (DEC-036
  Validation) — note that seam when wiring visibility.
- **Oracle = tier-B `dnglab analyze --raw-checksum`** MD5 equality on both held
  files (chart-independent, two byte orders), proven **red** (perturb the decoded
  plane or a predictor). CI cannot see tier-B — corpus files are never committed
  (DEC-003) — so tier-A analytic Compression-7 fixtures carry the CI-reachable
  half (the STAGE-009 round-trip encoder can emit Compression-7 strips).
  Related the stage-born lesson [[shared-path-oracle-tautology]]: the real-file
  `dnglab` oracle is a different, independent shape from STAGE-009's round-trip,
  which is exactly why it is load-bearing here.
- **Both byte orders:** the container reader already handles `II`/`MM`; the SOF-3
  entropy bitstream itself is MSB-first regardless of the container byte order —
  confirm the strip bytes feed the decoder identically for both files (a §12 probe
  at first-spec design).
- **Determinism:** the decoder is integer-only (no float), so macOS/Linux
  byte-identity should hold by construction; assert it rather than assume it.
- **Provenance-ledger:** a row for the DNG Compression-7 **binding** (distinct
  from STAGE-009's T.81 Annex H **decode** row).
- **§12 probe at first-spec design:** confirm the exact site where
  `Compression == 7` is rejected today (`Error::UnsupportedCompression` in the
  IFD/plane gate) and the strip-assembly entry point the decoder plugs into, on
  the real shipped code — before specing the routing.

## Dependencies

### Depends on
- **STAGE-009** (shipped) — the SOF-3 decoder `ljpeg::decode_into` (bytes → `u16`
  plane), bit-exact and panic-free; this stage is its first real caller
- **PROJ-001** (shipped core) — the container reader, IFD model, SPEC-028
  multi-strip plane assembly, level normalization, geometry, the tone curve, and
  the three-layer oracle harness; the decoder plugs into the existing plane path
- External: **`dnglab`** (Homebrew, LGPL-2.1 — **run as a tool, never linked**)
  for the bit-exact `--raw-checksum` reference
- Hardware/input: the two held Compression-7 DNGs (Leica M Typ 246, Pentax
  K-3 III) — **already in the corpus; no new acquisition needed**

### Enables
- Developing the majority of real RAW files (most carry a compressed plane)
- The big-endian Typ 246 and the M10 / M11 / Q3 Monochrom and Nikon vendor waves
  that are blocked on SOF-3
- crustyimg developing the compressed files users actually have
- **PROJ-003 close** — this is the project's last planned stage

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1d (Stage Ship) in
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
