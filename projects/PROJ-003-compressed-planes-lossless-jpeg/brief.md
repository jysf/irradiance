---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-003
  status: active
  activity: design
  priority: high
  target_ship: null

repo:
  id: irradiance

created_at: 2026-11-05
shipped_at: null

closed_reason: null

value:
  thesis: >
    A permissive, pure-Rust library can decode a losslessly-JPEG-compressed DNG
    sensor plane (Compression 7 / SOF-3, ITU-T T.81 Annex H) to the SAME
    bit-exact u16 plane an uncompressed file yields — verified against an
    independent implementation — turning irradiance from "develops one
    uncompressed file" into a decoder for the compressed planes most real RAW
    files actually carry (the keystone shared across roughly eight vendor
    decoders), without leaving the permissive / clean-room / patent-safe envelope.
  beneficiaries:
    - "The maintainer — his held Leica M Monochrom Typ 246 and Pentax K-3 III DNGs are Compression 7 and today return UnsupportedCompression; this makes them developable"
    - "crustyimg — its RAW-develop path covers the compressed files users actually have, not only uncompressed DNGs"
    - "Rust imaging projects still forced onto copyleft RAW crates because no permissive crate decodes lossless-JPEG DNG planes (oculante, czkawka, image-hdr, stardetect)"
    - "Every later vendor wave (big-endian Typ 246, M10 / M11 / Q3 Monochrom, Nikon) that is blocked on SOF-3 as a prerequisite"
  success_signals:
    - "Both held Compression-7 DNGs (Leica M Typ 246, Pentax K-3 III) unpack to a monochrome u16 plane whose MD5 equals `dnglab analyze --raw-checksum`, EXACTLY — the bit-exact contract survives the move to a compressed plane"
    - "An analytic encode→decode round-trip oracle goes RED on a deliberately perturbed Huffman table, predictor, or point transform, and green on the honest kernel"
    - "The decoder never panics on hostile or truncated LJ92 input (a fuzz target over the SOF-3 decoder, millions of runs clean)"
    - "The SOF-3 codec and its DNG binding each carry a provenance-ledger entry naming ITU-T T.81 Annex H and its licence"
    - "The same compressed DNG produces byte-identical plane output on macOS and Linux — determinism survives the bitstream decoder"
  risks_to_thesis:
    - "Both held SOF-3 files are monochrome but encoded as TWO-component (Nf=2) column-interleave — confirmed by parsing their SOF3 markers during SPEC-043 build (Typ246 X=2992/Nf=2, K3III X=3152/Nf=2, each ≈ half the crop width), NOT the single-component the framing assumed. STAGE-011 must therefore decode two-component to realize the thesis (SPEC-046, DEC-037). The Bayer / CFA COLOUR multi-component path (3+ colour components in one frame) still has no held file and no bit-exact oracle — scoped OUT, gated on acquiring one (the SPEC-035 corpus trap, pre-empted not repeated)"
    - "The bit-exact oracle is `dnglab --raw-checksum`, single-sourced on rawler — a decoder bug that happens to match a rawler bug would pass; the analytic encode→decode round-trip and the published T.81 spec are the independent counterweights"
    - "LJ92 is old but edge-case-rich (restart markers, 16-bit precision, point transform, per-row predictor selection, the big-endian container) that a two-file corpus may not fully exercise — the analytic oracle must cover what the corpus does not"
    - "Patent: lossless Huffman JPEG is T.81 / 1992 and believed unencumbered, but the check is load-bearing to the permissive thesis and must be done AT DESIGN, before building (DEC-031 lesson: patent-check named algorithms at frame time)"

value_realized:
  thesis_held: null
  signals_observed: []
  evidence: null
  notes: null

roadmap:
  - item: "Multi-component (Bayer-in-LJ92) SOF-3 interleave — no bit-exact oracle without a held Bayer SOF-3 file"
    kind: goal
    horizon: later
    resume_when: "a colour camera's lossless-JPEG DNG enters the corpus"
  - item: "OpcodeList2 GainMap (flat-field / vignetting) — opcode application, not plane decode; no held file carries one"
    kind: goal
    horizon: later
    resume_when: "a DNG carrying an OpcodeList2 GainMap enters the corpus"
---

# PROJ-003: Compressed planes — lossless JPEG (SOF-3)

## What This Project Is

The third wave of `irradiance`: decode a **losslessly-JPEG-compressed** sensor
plane — DNG `Compression 7`, the lossless JPEG **SOF-3** coding of ITU-T T.81
Annex H — into the same `u16` sensor plane an uncompressed file yields, verified
**bit-exact** against an independent implementation. It delivers a clean-room
SOF-3 bitstream decoder (Huffman DC tables, predictors 1–7, point transform,
restart markers, 8–16-bit precision, single-component and two-component column-interleave) and the DNG binding that
feeds it through the existing strip/tile plane path, for **both** container byte
orders.

It builds directly on PROJ-001. The container reader, the IFD model, the
strip/tile plane assembly (SPEC-028's multi-strip path), level normalization,
geometry, the tone curve and the oracle harness are all reused — the decoder is
a new **source** of plane bytes, slotted in where the packed-bits unpack sits
today. The uncompressed path and every monochrome/Bayer behaviour stay intact; a
`Compression 7` file simply stops being rejected and takes the new decode path.

## Why Now

**SOF-3 is the keystone shared across most non-trivial RAW.** irradiance
develops only *uncompressed* planes today — "develops one uncompressed file."
Most real RAW files (and roughly eight vendor decoders) carry a
losslessly-JPEG-compressed plane. Landing SOF-3 turns "develops one file" into
"develops the files people actually have," and it is the prerequisite the
big-endian Typ 246 and the M10 / M11 / Q3 Monochrom and Nikon waves all wait on.

**It is fully chart-independent — and PROJ-002 is not.** PROJ-002's last stage
(SPEC-035 ColorChecker ΔE) is blocked on a physical chart in transit. PROJ-003's
oracle is bit-exact `dnglab --raw-checksum` on **two files already in the
corpus**, so this wave can proceed *now*, in parallel, against held inputs — the
maintainer's planned "PROJ-003 runs as its own thread" sequencing.

**The oracle method transfers unchanged.** The plane is still one `u16` buffer;
the bit-exact contract (`dnglab analyze --raw-checksum`) is exactly PROJ-001's,
and only the bytes now arrive compressed. The clean-room algorithm gets its own
independent check — an analytic encode→decode round-trip — exactly as the
demosaic did in PROJ-002.

**Permissive clean-room cross-checks now exist.** `oxideav-mjpeg` (MIT,
single-component SOF-3, verified bit-exact vs `djpeg`) and `photocraft-raw`
(MIT/Apache, clean-room LJ92) are independent reference implementations to
cross-read against — a check beyond rawler, and evidence the decoder is
tractable clean-room. Neither is adopted (no-new-deps bar; unverified clean-room
claims); both are read-as-reference only.

## Success Criteria

- Both held `Compression 7` DNGs (Leica M Typ 246, Pentax K-3 III) unpack to a
  monochrome `u16` plane whose MD5 equals `dnglab analyze --raw-checksum`,
  **exactly**
- An analytic encode→decode round-trip oracle demonstrably goes **red** on a
  deliberately perturbed Huffman table, predictor, or point transform, and green
  on the honest kernel
- The decoder never panics on the fuzz corpus or on truncated / hostile LJ92
  bitstreams
- The SOF-3 codec and the DNG binding each carry a provenance-ledger entry
  (ITU-T T.81 Annex H, and the binding)
- The same compressed DNG produces **byte-identical** plane output on macOS and
  Linux
- A `Compression 7` plane, once decoded, flows through the existing develop/tone
  path unchanged — the decoder is a source swap, not a second pipeline

## Scope

### In scope
- The **SOF-3 lossless-JPEG bitstream decoder** (ITU-T T.81 Annex H),
  **single component AND two-component column-interleave** (both held files are
  Nf=2 — DEC-037): DC Huffman tables (DHT), predictors 1–7, point transform
  (`Pt`), restart markers (`DRI` / `RSTn`), sample precision 8–16 bit, with the
  two column components de-interleaved back into one `u16` plane
- The **DNG `Compression 7` binding**: recognise `Compression == 7` on the sensor
  IFD, feed the existing strip/tile plane assembly (reuse SPEC-028), and handle
  **both** byte orders (big-endian Typ 246 + little-endian Pentax K-3 III)
- A **bit-exact plane oracle** against both held Compression-7 DNGs via
  `dnglab --raw-checksum`
- An **analytic encode→decode round-trip oracle** with a red-proof (perturbed
  Huffman / predictor / point transform)
- A **fuzz target** over the LJ92 decoder (hostile / truncated bitstreams),
  panic-free, under the five-lint deny policy
- **Provenance-ledger** entries for the SOF-3 codec and the DNG binding

### Explicitly out of scope
- **Bayer / CFA *colour* multi-component interleave** (3+ colour components in one
  LJ92 frame) — distinct from the monochrome two-component column-interleave the
  held files use (now IN scope, DEC-037); no held colour file carries it, so it has
  no bit-exact oracle; deferred until one is acquired (roadmap)
- **OpcodeList2 `GainMap`** (flat-field / vignetting) — opcode *application*, not
  plane *decode*; no held file carries one (inventory confirmed: the corpus has
  OpcodeList1/3 only). A separate later stage/project
- **Lossy DCT JPEG** (baseline / progressive SOF-0/2) — a different coding path;
  RAW planes are lossless only
- **Nikon NEF-lossless** (a different Huffman-difference + linearization scheme)
  and **Pentax PEF** (`Compression 65535`) — different formats, their own waves
- **Arithmetic-coded JPEG** (SOF-9/10/11) — not used by DNG lossless
- **Native vendor containers; a CLI** — `irradiance` stays a library; `irr`
  remains the internal dev/oracle binary

## Stage Plan

Ordered; proposed, not yet framed. Each stage ends at a stopping point where the
maintainer reviews before the next begins.

Format: `- [status] STAGE-ID — one-line summary`

- [x] STAGE-009 (shipped 2026-11-05) — **SOF-3 lossless-JPEG core decoder** (single
  component): Huffman DC tables, predictors 1–7, point transform, restart
  markers, 8–16-bit; a pure compressed-bytes→`u16`-samples function, verified by
  an analytic encode→decode round-trip oracle + a red-proof, fuzzed panic-free
- [ ] STAGE-011 (framed 2026-11-05) — **DNG `Compression 7` binding + bit-exact corpus
  verification**: wire the decoder into the strip/tile plane path for both byte
  orders, and verify both held files bit-exact vs `dnglab --raw-checksum`; the
  decoded plane flows through the existing develop/tone path unchanged.
  *(Was planned as "STAGE-010"; that id went to PROJ-002's 0.2.0 reconcile (#95),
  so this wave is STAGE-011 — stage ids are globally sequential.)*

**Count:** 1 shipped / 0 active / 1 pending

*(These two could collapse into one stage with two specs if the maintainer
prefers the roadmap's "core + binding are one work item" framing — decided at
stage frame, Prompt 1c.)*

## Dependencies

### Depends on
- **PROJ-001** (shipped core): container reader, IFD model, strip/tile plane
  assembly (SPEC-028 multi-strip), level normalization, geometry, tone curve, and
  the three-layer oracle harness — the decoder plugs into the existing plane path
- External: `dnglab` (Homebrew, LGPL-2.1 — **run as a tool, never linked**) for
  the bit-exact `--raw-checksum` reference
- Cross-check references (**read-as-reference only, never linked or adopted**):
  `oxideav-mjpeg` (MIT, single-component SOF-3) and `photocraft-raw` (MIT/Apache,
  clean-room LJ92)
- External: the **ITU-T T.81 (1992) Annex H** specification — the lossless JPEG
  coding, implemented from the published standard
- Hardware/input: the two held Compression-7 DNGs (Leica M Typ 246, Pentax
  K-3 III) — **already in the corpus; no new acquisition needed**

### Enables
- Developing the majority of real RAW files (most carry a compressed plane)
- The big-endian Typ 246 and the M10 / M11 / Q3 Monochrom and Nikon vendor waves
  that are blocked on SOF-3
- crustyimg developing the compressed files users actually have

## Project-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Project Is"?** <yes/no + notes>
- **How many stages did it actually take?** <number, compare to plan>
- **What changed between starting and shipping?** <one or two sentences>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **What did we defer to the next project?**
  - <one-line items>
