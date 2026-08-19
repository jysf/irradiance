---
insight:
  id: DEC-008
  type: decision
  confidence: 0.95
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-08-18
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# The unpacker implementation, not its tests — DEC-004 owns tests/** (levels).
affected_scope:
  - src/**

tags:
  - decode
  - unpack
  - spike-002
---

# DEC-008: Sample unpacking branches on byte alignment, not on bit depth

## Decision

The unpacker has **two paths, selected by `bits % 8 == 0`**:

- **Sub-byte samples** (12, 14 bit): a **MSB-first bit stream**, read with a bit
  cursor. The TIFF byte-order tag does not apply — the packing is defined in bits.
- **Byte-aligned samples** (8, 16 bit): plain integers in the **file's byte
  order**, as given by the TIFF header (`II` / `MM`). No bit cursor is involved.

Treating the second case as a bit stream produces a plane that is byte-swapped
per sample. It is wrong in a way that still *decodes*, still has the right length,
and still passes the layer-0 arithmetic check.

## Context

SPIKE-001 achieved a bit-exact decode of a Leica Q2 Monochrom on the first
attempt — 14-bit, packed, three frames, zero differing bytes. Its unpacker took
`bits` as a parameter but, across every frame it ever saw, that parameter was
**always 14**. The two cases above were therefore indistinguishable.

SPIKE-002 ran the same decoder against a Leica **M Monochrom** (CC0, from
raw.pixls.us): a different body, a different sensor generation, **16-bit**,
uncompressed. Everything else generalised perfectly — the IFD walk, SubIFD
recursion, tag model, sensor-plane selection, strip location, and the layer-0
arithmetic (`5216 × 3472 × 16 / 8 = 36,219,904 == StripByteCounts`) all worked
unmodified, on a file with **no `ActiveArea` tag and no opcode lists at all**.

The plane came out byte-swapped. Confirmed three ways:

1. `md5(our plane)` = `563ecf2b…`; `md5(our plane byte-swapped)` =
   **`b0f602b90db91f981bbd6802fd0e6edf`**, which is exactly
   `dnglab analyze --raw-checksum`.
2. The raw strip head `99 12 ef 11 0e 12 0b 11` read big-endian gives
   `[39186, 61201, 3602, 2833]` — impossible against `WhiteLevel 16383`. Read
   little-endian it gives `[4761, 4591, 4622, 4363]`, all plausible.
3. The file's header is `II`.

**What caught it was a free assertion, not an oracle.** The unpacker carried a
one-line sanity check — *max sample must not exceed `WhiteLevel`* — and it fired
immediately. Without it, the failure mode is a plane that is the right size, from
a file that parsed cleanly, differing from truth only in per-sample byte order.
That is the shape of bug that ships.

## Alternatives Considered

- **Option A: always bit-stream, and byte-swap afterwards when `bits == 16`.**
  - Why rejected: it encodes the coincidence rather than the rule. It would need
    another special case the moment an 8-bit or 32-bit sample appears, and it
    describes the fix rather than the format.

- **Option B: branch on the TIFF byte order at the top and use it throughout.**
  - Why rejected: byte order genuinely does **not** apply to sub-byte packing —
    a 14-bit sample straddles byte boundaries and has no "endianness" of its own.
    Applying it uniformly would break the case that currently works.

- **Option C (chosen): branch on `bits % 8 == 0`.**
  - Why selected: it is the actual rule the formats follow, so it extends to 8-bit
    and to 12-bit without further cases. It also makes the two paths separately
    testable, and we now hold real files for both — 14-bit (Q2M, own-work) and
    16-bit (M Monochrom, CC0).

## Consequences

- **Positive.** The unpack spec in STAGE-002 has its shape decided before it is
  written, with a real file for each branch and a known-correct checksum for both.
- **Positive.** The `max > WhiteLevel` assertion is now demonstrated to earn its
  keep and should be kept as a permanent invariant, not a debugging leftover.
- **Negative.** It is one more branch on a path that must stay panic-free and
  fuzz-clean. Both paths need their own fuzz coverage; a single target exercising
  only 14-bit would recreate exactly the blind spot this decision documents.
- **Neutral.** Only the 16-bit branch is currently reachable with a second file;
  the 12-bit sample we hold (M Monochrom Typ 246) is JPEG-compressed and needs
  lossless JPEG SOF-3, which PROJ-001 excludes.

## Validation

Right if the same decoder produces a bit-exact plane for **both** the Q2M
(14-bit, `II`) and the M Monochrom (16-bit, `II`) against
`dnglab analyze --raw-checksum`, from one code path selected by alignment.

Revisit when a big-endian (`MM`) *uncompressed* file is held — the Typ 246 is
`MM` but compressed, so the byte-aligned path has been reasoned about for `MM`
and never executed on it.

## References

- Evidence: `spikes/done/SPIKE-002-*.md`, 2026-08-18
- Corpus: `tests/corpus/manifest.toml` — `LEICA-M-MONOCHROM/L1000622.DNG`
- Constraint: `no-panics-on-untrusted-input`
