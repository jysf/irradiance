---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-011                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-003                      # parent project
repo:
  id: irradiance

created_at: 2026-11-05
shipped_at: 2026-11-06

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
  sessions:
    - tokens_total: 199478863
      estimated_usd: 409.33
      recorded_at: 2026-11-06
      notes: "Stage grain (DEC-013 §5), not split across specs. The orchestrator session (work-status-next-steps, scratchpad uuid c0b7ec9b-2be4-41ad-9dea-77f3b6690343) ran STAGE-011's capstone specs and both closes in the main session. Measured FLOOR from that session's OWN transcript (scratchpad-uuid anchored, NOT text grep), deduped by message.id, all claude-opus-4-8, 98.3 percent cache-read: 473 usage-bearing messages dated 2026-11-06 onward, in 946 / out 926,470 / cache-write 2,435,206 / cache-read 196,116,241 = 199,478,863 tokens, priced per component at Opus list rates (15/75/18.75-write/1.50-read per MTok) = 409.33 USD. SCOPED to 2026-11-06 onward to avoid double-count: STAGE-009's close already claimed this same session's messages dated 2026-11-04 onward, which swept in SPEC-043 and SPEC-046's 2026-11-05 orchestration; STAGE-008's colour work (2026-11-02/03) is excluded and will be claimed by its own PROJ-002 close. This window covers SPEC-044 and SPEC-045's design and ship cycles, dispatching and reconciling their four cold build and verify sessions (including the PR-124-merged-first conflict fix), the roadmap reconcile, and the STAGE-011 and PROJ-003 closes themselves. Floor: the push and ship-PR turns post-date the measurement. RATIO: 199.5M orchestration against SPEC-044 and SPEC-045's delegated grand totals of 40.3M (12,919,298 plus 27,357,677) is roughly 83 percent of this window's spend being orchestration no spec recorded, the design and ship of two specs plus two closes, all un-metered main-loop cycles, concentrated here. SPEC-043 and SPEC-046's own delegated totals (31.6M and 22.5M) sit against STAGE-009's floor by the date scoping above."
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

- [x] SPEC-043 (shipped 2026-11-05) — [M] Compression-7 recognition + strip-path routing: build
  merged (#112), verified (#114), shipped. `plane::unpack_into` dispatches on
  `Compression` — uncompressed unchanged, `Compression 7` routed per-strip to
  `ljpeg::decode_into`, else `UnsupportedCompression`. Both held files now REACH the
  decoder (and hit the Nf=2 finding below).
- [x] SPEC-046 (shipped 2026-11-05) — [M] Two-component (Nf=2) SOF-3 decode + column de-interleave
  (DEC-037): extend `ljpeg` to decode the two column components of the held files'
  Nf=2 frames into one monochrome `u16` plane, with an analytic two-component
  round-trip red-proof; the Nf=1 path stays byte-identical. depends_on SPEC-043.
- [x] SPEC-044 (shipped 2026-11-06) — [M] Bit-exact corpus oracle: both held Compression-7
  DNGs (Leica M Typ 246 big-endian + Pentax K-3 III little-endian) decode to a `u16` plane
  whose MD5 EQUALS `dnglab analyze --raw-checksum` exactly (`c72e…1d` / `98ba…89`), proven
  red by an `ljpeg` staged-copy fault, with an end-to-end develop smoke. Build merged (#124),
  verified ✅ APPROVED (#125, superset), shipped. The design §12 probe confirmed SPEC-046's
  de-interleave mapping on both files AND caught that K3III needs an **APPn/COM header-skip**
  (APP0 between DHT and SOS) to decode at all — folded in as one bounded `parse_headers` arm
  (**DEC-039**), no re-plan. HANDOFF-095 (build) + HANDOFF-096 (verify).
- [x] SPEC-045 (shipped 2026-11-06, PR #129 build + #130 verify, 27,357,677 tok / $55.64) — [M] Binding capstone: a provenance-ledger entry for the DNG
  Compression-7 binding, a plane-level macOS/Linux determinism pin, container→decoder
  hardening + continuous fuzzing of the PUBLIC Compression-7 route, and the external review's
  §2.4 `Pt<P`/`Tc,Th` + §2.1 later-DHT-replaces. **Designed 2026-11-06** ([S]→[M] with the
  review folding). The §12 probe found determinism (develop level, SPEC-044), §2.3 restart
  (SPEC-046) and tiled rejection already groundwork-covered; new work is the binding
  provenance row, the three ljpeg header guards, and dropping the `require_uncompressed`
  early-return in the fuzz targets (keeping the `__fuzz_ljpeg` hook — DEC-036 Validation).
  HANDOFF-097 + `prompts/SPEC-045-build.md`. depends_on SPEC-044. **Last STAGE-011 spec.**

**Count:** 4 shipped / 0 active / 0 pending (SPEC-043 + SPEC-046 + SPEC-044 + SPEC-045 all
shipped — STAGE-011 complete; PROJ-003 closes with this stage)

*The Nf=2 finding (SPEC-043 build, DEC-037) re-opened the single-component scope for the
held files' monochrome column-interleave; SPEC-046 was the inserted decode spec. The
APP0 finding (SPEC-044 design §12 probe, DEC-039) adds a one-arm header-skip to SPEC-044
without a re-plan — unlike the Nf=2 case it needs no new spec. No entry is flagged L.*

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

*Filled in at stage close, 2026-11-06 (Prompt 1d).*

- **Did we deliver the outcome in "What This Stage Is"?** Yes. Both held
  Compression-7 DNGs — Leica M Typ 246 (big-endian) and Pentax K-3 III
  (little-endian) — decode to a monochrome `u16` plane whose MD5 equals
  `dnglab analyze --raw-checksum` exactly, through the existing plane and
  develop/tone path (a source swap, not a second pipeline), panic-free on
  hostile/truncated input, byte-identical across macOS and Linux, with a
  provenance-ledger binding row and zero new runtime dependencies. Every
  Success Criterion is met.
- **How many specs did it actually take?** 4 (SPEC-043 recognition+routing,
  SPEC-046 two-component Nf=2 decode, SPEC-044 bit-exact oracle, SPEC-045
  capstone), against 3 at first frame (routing → oracle → capstone). The Nf=2
  finding inserted SPEC-046 before the oracle.
- **How many outlines survived unchanged?** ~2 of 3. The routing and the oracle
  outlines held in intent; the capstone's framed `[S]` grew to `[M]` when the
  0.2.0 review folded its ljpeg header items in, and a 4th spec (SPEC-046) was
  inserted mid-stage — so framing ran slightly ahead of what the real bitstream
  made knowable.
- **What changed between starting and shipping?** The held files turned out to be
  two-component (Nf=2) column-interleave SOF-3, not single-component — caught at
  SPEC-043's build (the container tags did not say so; the JPEG frame header
  did) — forcing the SPEC-046 decode precursor (DEC-037) before the oracle.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - Codify `probe-the-decode-input-not-just-the-container` (N=2 paired-opposing,
    bar met) and land it in AGENTS.md §12 as a §12-probe rule: probe the innermost
    data the decoder/parser will consume, not just the container metadata around it.
- **Signals dispositioned at this close?** (Prompt 1d step 7) All six open/watch
  `type: lesson` signals owned by stage-close were walked; no silent carry.
  - `probe-the-decode-input-not-just-the-container` → **codified** (N=2
    paired-opposing: the SPEC-043 container-tag miss + the SPEC-044 §12
    frame-header catch), landed in AGENTS.md §12.
  - `probe-the-hard-assumption-at-frame-time` → kept `watch`, still N=1. The
    STAGE-011 Nf=2 frame-time miss belongs to this family but is codified via the
    sharper sibling above; not double-counted here.
  - `patent-check-named-algorithms-at-frame` → kept `watch`, N=1 (positive
    counter-instance: STAGE-011 named no new algorithm; it bound the already
    patent-cleared SOF-3, DEC-035).
  - `shared-path-oracle-tautology` → kept `watch`, N=1. STAGE-011 applied the
    discipline rather than violating it — SPEC-044's oracle is an independent
    `dnglab` implementation, not a shared-helper round-trip, with mutation-red proofs.
  - `a-fix-inherits-the-precondition-of-the-thing-it-fixes` → kept `watch`, N=2.
    No STAGE-011 instance (the header guards were greenfield, first-pass clean).
  - `saturating-as-cast-masks-explicit-clamp-in-quantize-tests` → kept `watch`,
    N=1. No instance; the decoder is integer-only, no `f64 as u16` clamp pattern.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - None beyond the codified one. SPEC-045's "dispatch verify with an explicit
    model" note is an orchestration preference, below any bar; not recorded as a signal.
