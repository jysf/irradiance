---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-039
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

created_at: 2026-11-06
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-044-bit-exact-corpus-oracle-both-held-compression-7.md
  - projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md
  - src/ljpeg.rs

tags:
  - sof-3
  - ljpeg
  - marker-skip
  - appn
  - com
  - stage-011
  - spec-044
---

# DEC-039: Skip APPn / COM header segments; fold the fix into SPEC-044

## Decision

Teach the SOF-3 header parser (`ljpeg::parse_headers`) to **skip APPn
(`0xFFE0..=0xFFEF`) and COM (`0xFFFE`) marker segments by their declared length**,
wherever they appear in the header marker sequence before SOS. Land this in
**SPEC-044** (the bit-exact corpus oracle) rather than a new decode spec or SPEC-045,
because it is the minimal, XS decoder tolerance the real Pentax K-3 III file needs to
decode **at all**, and the only honest proof it is right is SPEC-044's real-file
`dnglab --raw-checksum` oracle. SPEC-044's prior non-goal ("no change to the SOF-3
decoder algorithm") is narrowed accordingly: tolerating optional metadata segments is
header hygiene, not a reconstruction-algorithm change.

Also recorded here (the second half of the same §12 probe): the SPEC-046 **even/odd
column de-interleave mapping is confirmed correct on both real files** — no write-index
change is needed. The mapping question SPEC-046 flagged as analytically unprovable
(`shared-path-oracle-tautology`) is resolved.

## Context

The SPEC-044 design §12 probe decoded both held Compression-7 DNGs through the shipped
`plane::unpack_into` and compared the native-LE `u16` plane MD5 against
`dnglab analyze --raw-checksum` (dnglab 0.7.2):

- **Leica M Monochrom Typ 246** (`M2462362.DNG`, P=12, big-endian): decodes and matches
  `c72e7d9579e2adaef9c6247841b8ac1d` **exactly, unmodified**. Its stream is
  `SOI → SOF3 → DHT → SOS`, no APPn.
- **Pentax K-3 III Mono** (`K3III.DNG`, P=14, little-endian): **fails** —
  `LjpegUnexpectedMarker { marker: 0xe0 }`. Its stream is
  `SOI → SOF3 → DHT → APP0(len 60) → SOS`: a camera-written APP0 metadata segment sits
  between DHT and SOS. `parse_headers`' marker loop has no APPn/COM arm, so APP0 falls
  into the `other =>` catch-all and is rejected.

dnglab decodes K3III fine (it has a pinned checksum), so the APP0 is optional metadata a
mature decoder skips. A one-arm patch to `parse_headers` — read the 2-byte segment
length, bounds-check it, advance past it (the same idiom `parse_dht` already uses) — was
applied as a throwaway probe: **both files then matched their pinned checksums exactly**
(K3III `98bae1339c25eb6e04fbac6b5f643189`). So the APPn/COM-skip is the *entire* gap for
K3III, and the de-interleave mapping needs nothing further.

This is review §2.4 ("skip `APP0..APP15` / `COM` by length"), which the triage marked
"latent — the held files trigger none." The probe falsifies that: K3III **does** trigger
it. The finding is a second confirmation of the signal
`probe-the-decode-input-not-just-the-container` — a real-file structural element the
container-level view (strip layout, SOF3/DHT/SOS structure) did not surface.

## Alternatives Considered

- **A: defer the APPn/COM skip to SPEC-045 (the hardening capstone), where the triage
  filed §2.4.** Rejected: SPEC-045 runs *after* SPEC-044, and SPEC-044's K3III oracle
  cannot go green without the skip. The dependency is inverted — the fix is a
  prerequisite of SPEC-044, not a follow-on.
- **B: mint a new decode spec (SPEC-047) for the skip before SPEC-044 (the SPEC-046
  re-plan pattern).** Rejected: SPEC-046 was an M-complexity new algorithm (two-component
  decode). This is a single bounded match arm (XS). A separate spec could only prove it
  with a hand-built tier-A fixture — which shares our own convention
  (`shared-path-oracle-tautology`) and defers the real proof to SPEC-044's dnglab oracle
  regardless. A dedicated cycle would be ceremony around a one-arm change.
- **C (chosen): fold the skip into SPEC-044, proven by the same real-file oracle that is
  SPEC-044's whole point.** The decoder change and its only honest proof live in one
  spec; the oracle validates the fix rather than a tautological fixture.

## Consequences

- **Positive.** Both held Compression-7 files decode bit-exact vs dnglab → STAGE-011's
  bit-exact contract is met for both byte orders, and PROJ-003's thesis is reachable.
  SPEC-044 stays [M] and STAGE-011 stays 4 specs (no re-plan, unlike the Nf=2 finding).
- **Neutral.** `parse_headers` gains one marker arm (APPn/COM → length-skip, bounds-
  checked, typed `Error::Truncated` on a bad length — never a panic). `ljpeg` stays
  `pub(crate)`, zero new runtime dependency, five-lint clean, `#![forbid(unsafe_code)]`.
  The `Nf=1` path and the Typ 246 decode are byte-for-byte unchanged (Typ 246 has no
  APPn, so it is untouched; the arm only fires on a segment that is currently rejected).
- **Negative — the honest cost.** SPEC-044 is no longer a pure verification spec; it
  carries a (small) decoder change. The scope creep is bounded and recorded here. The
  mapping-confirmation that SPEC-046 deferred to SPEC-044 lands as planned — a positive,
  not a cost.

## Validation

Right if, with the APPn/COM-skip arm, both held files decode to a `u16` plane whose MD5
equals `dnglab analyze --raw-checksum` (Typ 246 `c72e…1d`, K3III `98ba…89`), a tier-A
fixture with an APP0/COM segment between DHT and SOS decodes bit-exact, a truncated APPn
length is a typed `Error`, and a staged-copy red-proof that perturbs the `ljpeg` decode
turns the real-file oracle red. Revisit if a held file later carries an APPn segment
**inside** the entropy scan (none observed — K3III's is header-only) or a COM the skip
mis-measures.

## References

- Triggered by the SPEC-044 design §12 probe (both held files decoded through
  `plane::unpack_into`, MD5 vs `dnglab --raw-checksum`). Governs `SPEC-044`; narrows its
  "no decoder change" non-goal; confirms `SPEC-046`'s (`DEC-037`) de-interleave mapping
  on both real files. Second confirmation of signal
  `probe-the-decode-input-not-just-the-container`. Aligns with external review §2.4
  (`docs/reviews/2026-11-06-staff-review-0.2.0-triage.md`). Builds on `DEC-035`
  (clean-room / patent-clean) and `DEC-016` (caller-owned buffer).
