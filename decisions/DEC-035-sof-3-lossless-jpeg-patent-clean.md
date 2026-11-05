---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-035
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-003
repo:
  id: irradiance

created_at: 2026-11-05
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# Governs the new SOF-3 decoder module and the STAGE-009 specs that build it.
affected_scope:
  - src/ljpeg.rs
  - projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-036-sof-3-entropy-front-end-markers-bit-reader-dc-huffman.md
  - projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-037-lossless-reconstruction-predictors-point-transform-restart.md
  - projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-038-sof-3-decoder-fuzz-truncation-hardening.md

tags:
  - sof-3
  - lossless-jpeg
  - jpeg
  - patent
  - licensing
  - t81
  - compression-7
  - spec-036
  - stage-009
---

# DEC-035: SOF-3 lossless-JPEG decoding (ITU-T T.81 Annex H) is patent-clean — implement it

## Decision

Implement STAGE-009's SOF-3 lossless-JPEG decoder (DNG `Compression 7`) directly
from the published **ITU-T T.81 (1992) Annex H** standard. The lossless **Huffman**
coding it uses carries no live patent: the notorious JPEG patents cover the **lossy
DCT baseline** (Forgent / Compression Labs `US 4,698,672` — **expired October
2006**) and the **arithmetic** entropy coder (IBM / AT&T / Mitsubishi — long
expired, and not used by SOF-3), neither of which reads on SOF-3's Huffman
DC-difference path. Unlike `DEC-031` — where the *named* demosaic (Malvar-He-Cutler)
was live-patented and had to be **replaced** — no algorithm substitution is needed
here: SOF-3 is the only lossless coding `Compression 7` uses, and it is clean.

## Context

PROJ-003's thesis is a permissive, pure-Rust decoder for the compressed planes most
real RAW carries; STAGE-009's keystone is the SOF-3 bitstream decoder. `DEC-031`
established the standing rule for this crate: **patent-check any named algorithm
BEFORE building it into a permissive (MIT OR Apache-2.0) crate**, because clean-room
reimplementation defeats *copyright*, not *patents* — a live method patent is read by
any independent implementation. So the patent status of lossless JPEG was checked
before any decoder code was written.

Findings (public sources, read 2026-11-05):

- The famous "JPEG patent," Forgent / Compression Labs **`US 4,698,672`** (filed
  1986, granted 1987), targeted the **DCT** used in the *lossy baseline*. It
  **expired October 2006**; its broadest claims were invalidated on prior art (USPTO
  reexamination prompted by PUBPAT), and a court narrowed it to video. It never read
  on lossless Huffman coding.
- JPEG **arithmetic** coding (SOF-9/10/11) was encumbered by IBM / AT&T / Mitsubishi
  patents — the reason IJG `libjpeg` historically disabled it — and those expired by
  the late 2000s (libjpeg re-enabled arithmetic coding ~2009). SOF-3 uses **Huffman**,
  not arithmetic, so these never applied regardless.
- SOF-3 lossless Huffman coding is defined in **ITU-T T.81 (1992) Annex H**. Any
  patent reading on techniques in a 1992 standard (20-year term; filings of the late
  1980s–early 1990s) expired by ~2012. In 2026 the JPEG patent landscape relevant to
  SOF-3 is long expired on every axis.
- The open-source RAW ecosystem — LibRaw, dcraw, rawler/`dnglab`, darktable,
  RawTherapee — decodes `Compression 7` lossless JPEG **universally and without
  patent licensing**, under permissive and copyleft licenses alike. Strong practical
  evidence of a clear field.
- Adobe's **DNG SDK** carries a royalty-free-but-revocable patent license for
  "Compliant Implementations." irradiance does **not** use the SDK; it implements the
  decoder from the published T.81 standard. The underlying lossless-JPEG technique is
  unpatented/expired, so the SDK's license is not a constraint on an independent
  clean-room decoder — the same clean-room-from-published-standard posture the crate
  already uses for TIFF/DNG parsing (`provenance-recorded-per-algorithm`).

## Alternatives Considered

- **A: Treat the patent question as settled by priors; skip the check.**
  - Rejected. `DEC-031` is the standing lesson — the demosaic framing named a
    live-patented method (MHC, `US 7,502,505`, active to ~2027) *without* a check. The
    clearance is load-bearing to the permissive thesis and cheap to do.
- **B: Avoid SOF-3; support only uncompressed planes.**
  - Rejected. SOF-3 is the keystone PROJ-003 exists to deliver; most real RAW carries
    a compressed plane, and SOF-3 is clean — there is nothing to avoid.
- **C (chosen): Implement SOF-3 from the published T.81 standard, clean-room,
  recording this clearance.**
  - The coding is patent-clean, universally implemented, and fits the crate's existing
    clean-room-from-standard discipline (like `plane::unpack` ← TIFF 6.0 / `DEC-008`).
    No substitution, no stopgap — SOF-3 is both the correct and the clean choice.

## Consequences

- **Positive.** The crate can decode the compressed planes most RAW carries while
  staying permissively licensable and patent-clean, protecting the adoption thesis.
  No algorithm compromise (unlike `DEC-031`'s MHC→Hamilton-Adams PSNR trade) — SOF-3
  is both the correct and the clean choice.
- **Positive.** The decision is recorded from an evidence check **before any decoder
  code is written** (measure-before-build), matching `DEC-031`'s discipline.
- **Neutral.** The decoder is implemented from T.81 Annex H and a provenance-ledger
  row records that (`provenance-recorded-per-algorithm`); permissive cross-reference
  implementations (`oxideav-mjpeg`, `photocraft-raw`) are read for structure only,
  never copied or linked.
- **Negative — the honest caveat.** Patent non-infringement is never provable in the
  absolute; this rests on expiry dates, the Huffman-vs-DCT/arithmetic distinction, and
  universal unlicensed OSS implementation, read 2026-11-05 — strong evidence, not a
  legal opinion. A definitive clearance would be a patent-counsel search. For a
  permissive OSS crate implementing a 1992-standard, universally-shipped, long-expired
  coding, that is a reasonable basis (the `DEC-031` standard).

## Validation

Right if: STAGE-009 ships a SOF-3 decoder implemented from T.81 Annex H, carrying a
provenance-ledger entry, with no dependency on Adobe's DNG SDK and no use of
arithmetic coding (SOF-9/10/11) or the lossy DCT path. Revisit if: a credible claim
surfaces that SOF-3 lossless Huffman coding reads on a *live* patent (then re-clear),
or if a later wave adds a *different* coding (lossy DCT, JPEG-XL, arithmetic) — each
needs its own clearance, and this decision does not cover them.

## References

- Builds on `DEC-031` (patent-check named algorithms before building into the
  permissive crate) and the crate's clean-room-from-published-standard discipline
  (`plane::unpack` ← TIFF 6.0 / `DEC-008`). Governs `SPEC-036`/`SPEC-037`/`SPEC-038`
  (the SOF-3 decoder).
- Standard: ITU-T T.81 (1992) Annex H — lossless JPEG (SOF-3). Patents:
  `US 4,698,672` (Forgent / Compression Labs, DCT — expired Oct 2006, broadest claims
  invalidated); JPEG arithmetic-coding patents (IBM / AT&T / Mitsubishi — expired).
  Read via public sources 2026-11-05; status is evidence, not a legal conclusion.
- Constraints: `no-copyleft-dependencies`, `provenance-recorded-per-algorithm`,
  `license-policy`.
