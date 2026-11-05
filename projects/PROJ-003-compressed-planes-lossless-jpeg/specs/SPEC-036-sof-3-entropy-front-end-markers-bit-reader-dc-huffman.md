---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-036
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-003
  stage: STAGE-009
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # HANDOFF-085 (build) → tier_map.build
  created_at: 2026-11-05

references:
  decisions: [DEC-035, DEC-018]    # SOF-3 patent-clean (DEC-035); u16 modular arithmetic (DEC-018)
  constraints: [no-panics-on-untrusted-input, provenance-recorded-per-algorithm, test-before-implementation]
  related_specs: [SPEC-037, SPEC-038]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # chain head — SPEC-037 and SPEC-038 depend on this

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-009's <capability>". Optional; null is acceptable.
value_link: "The SOF-3 entropy front-end — marker/frame parse + JPEG bit reader + DC Huffman decode that turns LJ92 bytes into per-sample (category → difference) values; the first half of STAGE-009's compressed-bytes→u16 decoder."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 32000000        # ~32M, in line with STAGE-007/008 [M] specs (new module, hand-built fixtures)
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: 45
      recorded_at: 2026-11-05
      notes: "main-session design (un-metered); patent check DEC-035 + spec + HANDOFF-085 + build prompt"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-036: SOF-3 entropy front-end (markers, bit reader, DC Huffman)

## Context

irradiance decodes only uncompressed planes today; a DNG `Compression 7` plane
is a lossless-JPEG (SOF-3, ITU-T T.81 Annex H) bitstream it currently rejects.
`STAGE-009` builds the decoder that turns those bytes into a `u16` sample plane,
and this is the first of its three-spec backlog (SPEC-036 → SPEC-037 →
SPEC-038). It delivers the **entropy front-end**: reading the JPEG marker/frame
structure and the Huffman-coded scan down to per-sample difference values —
everything up to, but not including, predictor reconstruction (SPEC-037). It has
no DNG-container entanglement (that is STAGE-010) and can be exercised on a
hand-built bitstream in isolation.

- Parent: `STAGE-009` (SOF-3 lossless-JPEG core decoder), backlog position 1 of 3
- Project: `PROJ-003` (compressed planes — lossless JPEG)
- Standard: ITU-T T.81 (1992) Annex H — implemented from the published spec

## Goal

Given the entropy-coded bytes of a single-component SOF-3 scan and its
frame/scan headers, parse the markers (SOI, SOF3, DHT, SOS, DRI, RSTn, EOI),
build the DC Huffman table, and decode each sample's magnitude category and
difference value — yielding the ordered difference stream (plus frame
parameters) that SPEC-037 reconstructs into samples. Every malformed input
returns a typed `Error`, never a panic.

## Inputs

- **Standard (implement from this):** ITU-T T.81 (1992) — lossless JPEG.
  Annex H (lossless mode), Annex C (`BITS`/`HUFFVAL` → canonical Huffman codes),
  Annex F §F.2.2 (the `RECEIVE`/`EXTEND` procedure) and Figure F.16 (`DECODE`),
  Annex B (marker/segment structure, `0xFF`-byte stuffing). Patent clearance and
  the provenance requirement are in `DEC-035`.
- **Code to read / mirror:**
  - `src/plane.rs` — the private `BitReader` (~lines 102–169): MSB-first,
    checked arithmetic only (no `<<`/`+`; uses the `pow2` helper), returns
    `Error::Truncated { at, len }` on EOF. The SOF-3 `ScanReader` mirrors this
    shape and **adds** `0xFF`-destuffing + marker detection — a separate reader,
    not a reuse (JPEG entropy bytes are not TIFF packed bits).
  - `src/lib.rs` (`pub enum Error`, ~line 144): `#[non_exhaustive]`, documented
    struct variants — add the new `Ljpeg*` variants here in the same style.
  - `src/ifd.rs` (`Compression::Jpeg == 7`, ~line 485): the container tag that
    will route to this decoder in STAGE-010 — **not touched by this spec**.
- **Cross-reference only (read for structure; never copied or linked —
  clean-room, `DEC-035`):** `oxideav-mjpeg` (MIT, single-component SOF-3) and
  `photocraft-raw` (MIT/Apache).
- **No external crate, no corpus file:** the front-end is exercised entirely on
  hand-built byte slices (a real DNG strip is STAGE-010's input).

## Outputs

- **Files created:** `src/ljpeg.rs` — a new crate-internal module (`mod ljpeg;`
  in `src/lib.rs`, items `pub(crate)`; **not** public API — STAGE-010 decides the
  public surface). It holds the SOF-3 decoder; this spec delivers its entropy
  front-end, SPEC-037 adds reconstruction in the same module.
- **Files modified:**
  - `src/lib.rs` — the `mod ljpeg;` declaration and the new `Error` variants.
  - `docs/provenance-ledger.md` — a row for the SOF-3 marker/frame parse + DC
    Huffman decode (ITU-T T.81 Annexes B/C/F/H), implemented from the published
    standard (`DEC-035`).
- **New items (crate-internal):**
  - `struct Frame { precision: u8, lines: u16, samples_per_line: u16,
    point_transform: u8, predictor: u8, restart_interval: u16 }` — the parsed
    single-component SOF-3 frame + scan parameters (`predictor` = `Ss`,
    `point_transform` = `Al` from SOS; `restart_interval` = `Ri` from DRI, `0` =
    none). SPEC-037 reads these; this spec only populates them.
  - `struct HuffTable` — a DC Huffman decode table built from a DHT
    `BITS`(16 counts)/`HUFFVAL` segment (canonical codes per Annex C; a decode
    structure per Figure F.16).
  - `fn parse_headers(data: &[u8]) -> Result<(Frame, HuffTable, usize), Error>` —
    walk markers `SOI → {DHT | DRI | SOF3} → SOS`, returning the frame, the DC
    table, and the byte offset where entropy-coded data begins. Rejects non-SOF3
    processes, `Nf != 1`, and malformed segments with typed errors.
  - `struct ScanReader<'a>` with:
    - `fn new(entropy: &'a [u8]) -> ScanReader<'a>`
    - `fn decode_difference(&mut self, table: &HuffTable) -> Result<i32, Error>` —
      `DECODE` one Huffman symbol → category `SSSS` (`0..=16`), then
      `RECEIVE`+`EXTEND` → the signed difference (`SSSS==0 → 0`; `SSSS==16 →
      32768`; else a sign-extended `SSSS`-bit magnitude).
    - `fn restart(&mut self) -> Result<(), Error>` — byte-align and consume the
      next expected `RSTm` marker (SPEC-037 calls this at each restart interval).
  - All arithmetic is `checked_*`/`wrapping_*`/`saturating_*` (five-lint deny
    policy; `#![forbid(unsafe_code)]`).
- **New `Error` variants (additive to the `#[non_exhaustive]` enum, documented
  like their neighbours).** Proposed set: `LjpegUnsupportedProcess { sof: u8 }`
  (a SOF that is not SOF3 — baseline/progressive/arithmetic),
  `LjpegUnsupportedComponentCount { nf: u8 }` (`Nf != 1`),
  `LjpegUnsupportedPrecision { precision: u8 }`,
  `LjpegUnsupportedPredictor { predictor: u8 }` (`Ss` not in `1..=7`),
  `LjpegInvalidHuffmanTable` (BITS sum / HUFFVAL length / a code that overruns 16
  bits), `LjpegMissingMarker { marker: u8 }` (a required SOI/SOF3/DHT/SOS absent,
  or a segment length that does not fit), `LjpegUnexpectedMarker { marker: u8 }`
  (a marker where entropy data was expected, or the wrong `RSTm`). EOF reuses the
  existing `Error::Truncated`. The exact set may be consolidated at build **so
  long as** every malformed input maps to a typed, documented variant and nothing
  panics.
- **No flags, no public API, no CLI, no database change.** (`irr` gains nothing
  here; STAGE-010 wires the decoder to the container and may add an `irr` path.)

## Acceptance Criteria

- [ ] `parse_headers` parses a well-formed single-component SOF-3 header (SOI,
  SOF3, DHT, SOS, optional DRI) into the correct `Frame` — precision, lines,
  samples_per_line, point_transform, predictor, restart_interval — and returns
  the entropy-data offset. (≥2 distinct frames)
- [ ] A DC `HuffTable` built from a known `BITS`/`HUFFVAL` decodes known bit
  patterns to the correct category symbols, in canonical-code order (Annex C),
  including the shortest code and a 16-bit-long code.
- [ ] `decode_difference` implements `RECEIVE`+`EXTEND` exactly: `SSSS==0 → 0`;
  positive and negative magnitudes at several `SSSS` widths sign-extend to the
  correct `i32`; `SSSS==16 → 32768`. (table-driven over the category range)
- [ ] `ScanReader` destuffs `0xFF 0x00` to a literal `0xFF` data byte and reads
  bits MSB-first across byte boundaries identically to a hand-computed reference.
- [ ] `restart` byte-aligns and consumes an `RSTm` marker; a wrong or absent
  restart marker is a typed error, not a panic.
- [ ] Every malformed input returns a typed `Error` and **never panics**:
  non-SOF3 process, `Nf != 1`, precision/predictor out of range, an over-running
  or length-inconsistent Huffman table, a truncated header, a truncated scan, and
  a marker where entropy data was expected. (one test per case)
- [ ] The module compiles under `#![forbid(unsafe_code)]` and the five-lint deny
  policy (`unwrap_used`, `expect_used`, `indexing_slicing`, `panic`,
  `arithmetic_side_effects`); `just lint-ci -D warnings`, `just fmt-check`, and
  `just validate` are green.
- [ ] `docs/provenance-ledger.md` carries the SOF-3 front-end row (T.81,
  `DEC-035`).

## Failing Tests

All in `src/ljpeg.rs` under `#[cfg(test)] mod tests` — the home of the items
they test, mirroring `src/plane.rs`'s `BitReader` unit tests. Hand-built byte
slices, no corpus file. A small test helper that assembles a SOF-3 header + scan
from parameters keeps these readable (it is the seed of SPEC-037's encoder).

- **`src/ljpeg.rs` (`mod tests`)**
  - `"parse_headers reads a minimal SOF3 frame"` — asserts the `Frame` fields
    and entropy offset for a hand-built P=8, `Nf=1`, predictor-1 header.
  - `"parse_headers reads precision, dimensions, Pt and Ri"` — a second frame
    with P=16, a non-zero point transform, and a DRI restart interval; asserts
    all fields.
  - `"huffman table decodes canonical codes"` — build from a known
    `BITS`/`HUFFVAL`; assert specific bit patterns `DECODE` to specific
    categories, including the shortest and a 16-bit-max code.
  - `"receive_extend sign-extends per Annex F"` — table-driven `(SSSS, raw) →
    diff` across `SSSS = 0..=16`, both signs, including `SSSS==0 → 0` and
    `SSSS==16 → 32768`.
  - `"scan reader destuffs 0xFF00 and reads MSB-first across bytes"` — a stream
    with a stuffed `0xFF 0x00`; assert decoded bits equal a hand-computed
    reference and the literal `0xFF` survives.
  - `"restart consumes RSTm and byte-aligns"` — assert position after a
    mid-stream `0xFFD0`; a wrong marker → typed error.
  - `"malformed inputs are typed errors, never panics"` — one assertion each:
    non-SOF3 SOF → `LjpegUnsupportedProcess`; `Nf=3` →
    `LjpegUnsupportedComponentCount`; `Ss=0` / `Ss=8` →
    `LjpegUnsupportedPredictor`; `HUFFVAL` length ≠ `sum(BITS)` →
    `LjpegInvalidHuffmanTable`; a header cut mid-segment → `Truncated`; a scan
    that ends mid-code → `Truncated`; a `0xFFC4` where a code was expected →
    `LjpegUnexpectedMarker`.

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- Predictor reconstruction, initial prediction, point transform, restart
  sample reset, and the `u16` output plane — that is SPEC-037.
- The analytic encode→decode round-trip oracle and the fuzz target — SPEC-037
  and SPEC-038.
- Any DNG container binding (recognising `Compression 7`, strip/tile assembly,
  byte order) — STAGE-010.
- Multi-component / Bayer-in-LJ92 scans (`Nf > 1`), lossy DCT JPEG (SOF-0/2),
  and arithmetic coding (SOF-9/10/11).

## Notes for the Implementer

- **Clean-room, from the standard.** Implement from ITU-T T.81 (Annexes
  B/C/F/H); see `DEC-035` for the patent clearance and the provenance
  requirement. The two cross-reference crates (`oxideav-mjpeg`,
  `photocraft-raw`) may be read for structure but **never copied or linked**
  (`no-copyleft-dependencies`, clean-room). Add the provenance-ledger row as part
  of this spec.
- **`ScanReader` is a new reader, not a reuse of `plane::BitReader`.** Mirror its
  shape (MSB-first, `checked_*` only, `pow2`, `Error::Truncated`), but JPEG
  entropy coding adds `0xFF` byte-stuffing (`0xFF 0x00` → literal `0xFF`; `0xFF`
  + non-zero = a marker) that the TIFF packed-bits reader has no concept of.
- **Modular arithmetic is explicit (`DEC-018`).** `RECEIVE`/`EXTEND` and code
  accumulation use `wrapping_*`/`checked_*`, so `arithmetic_side_effects` is
  satisfied by intent, never silenced by a cast.
- **RAW is attacker-influenced.** Every marker/length/table/bit read is bounded
  and returns a typed `Error` on malformed input (`no-panics-on-untrusted-input`).
  SPEC-038 fuzzes this surface; design for it now — no index-slicing, no
  unchecked lengths.
- **Scope seam with SPEC-037.** This spec stops at the difference stream and the
  parsed `Frame`. Predictors 1–7, the initial prediction `2^(P−Pt−1)`, applying
  the point transform, the per-restart predictor reset, and the `u16` output
  plane are **SPEC-037** — do not implement them here. `ScanReader::restart`
  (byte-align + consume `RSTm`) is the entropy-mechanics half of restart and
  lives here; SPEC-037 calls it and resets prediction.
- **Pre-named L-risk split (the build's escape hatch).** Designed whole at [M].
  If it runs long, split along the seam the stage already named: (a) marker/frame
  parse → `Frame` + entropy offset, then (b) the `HuffTable` build +
  `ScanReader`/`decode_difference`. Ship (a) then (b); do not expand scope.
- **§12 probe (done at design; no external dependency to pin):** the real
  surfaces are `src/lib.rs`'s `Error` enum (struct-variant, `#[non_exhaustive]`)
  and `src/plane.rs`'s `BitReader` (the checked-arithmetic pattern to mirror).
  There is no runtime baseline — correctness, not throughput, is the target.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
