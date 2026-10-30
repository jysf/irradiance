---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-029
  type: decision
  confidence: 0.92
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-10-29
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

# Amends DEC-008's WhiteLevel-assertion clause only; DEC-008's two-path
# byte-alignment rule still governs the same files, so the overlap is intended.
affected_scope:
  - src/plane.rs
  - src/lib.rs

tags:
  - decode
  - unpack
  - whitelevel
  - dec-008
  - spec-029
---

# DEC-029: Samples above `WhiteLevel` are legal — amending DEC-008's "permanent invariant"

## Decision

**A decoded sample above `WhiteLevel` is NOT an error.** `plane::unpack_into`
carries no `max <= WhiteLevel` assertion, preserves raw sample values unchanged
(no clamping), and `Error::SampleExceedsWhiteLevel` is removed. This **amends one
clause of `DEC-008`** — its `Consequences` line that the assertion "is now
demonstrated to earn its keep and should be kept as a permanent invariant." It
is not one. `WhiteLevel` is a **develop-time saturation parameter**, not an
unpack invariant. DEC-008's two-path byte-alignment rule (`bits % 8 == 0`) is
**untouched**.

## Context

`DEC-008` was decided on the PROJ-001 **monochrome** corpus (Leica Q2 Monochrom,
M Monochrom). There, every sample sat at or below `WhiteLevel`, and a one-line
`max > WhiteLevel` check caught `SPIKE-002`'s per-sample byte-swap immediately
(a 16-bit `II` plane misread big-endian gave `39186` against `WhiteLevel 16383`).
DEC-008 promoted that observation to a "permanent invariant."

PROJ-002 adds **colour** (Bayer) files, and the premise fails on real sensor
data. `WhiteLevel` is the nominal white point, not the maximum recordable code:
real sensors record **specular highlights above it**. The falsifier is a corpus
file, not a thought experiment:

- `CANON-EOS-5D-MARK-III/5G4A9394.DNG` — single strip, 16-bit, `WhiteLevel
  15000`, and sample index `2016443` = **`15464`**. Correctly unpacked, measured
  from the file's own bytes; it is not a decode error.
- `NIKON-D300S/_DSC4615.DNG` — the 12-strip plane SPEC-028 assembles; its first
  above-white sample is index `741623` = **`3835`** against `WhiteLevel 3808`
  (plane max `4095`).

Both files unpack correctly and then tripped the assertion **after** a valid
decode — the assertion rejected valid files. This also blocks `SPEC-025`, the
bit-exact mosaic-plane oracle: its `dnglab analyze --raw-checksum` compares the
**raw** plane, so the unpacker must preserve above-white values verbatim; a clamp
to `WhiteLevel` would change bytes and the checksum could never match.

## Alternatives Considered

- **Option A: keep the assertion, raise the bound to the bit-depth maximum
  (`2^bits - 1`).**
  - What it is: assert `max <= 65535` (for 16-bit) instead of `<= WhiteLevel`.
  - Why rejected: it keeps a free guard, but a per-sample byte-swap of a 16-bit
    plane produces values across the **whole** 16-bit range, so almost all
    byte-swaps pass a `2^16 - 1` bound — the guard that caught SPIKE-002 is
    already dead at 16-bit. It would only bite sub-byte depths (12/14-bit), where
    the layer-0 packed-bits check already constrains the stream. Low value, and
    it still encodes a bound the format does not actually impose on the unpacker.

- **Option B: clamp samples to `WhiteLevel` during unpack.**
  - What it is: accept above-white samples but write `min(sample, WhiteLevel)`.
  - Why rejected: it destroys the raw value the bit-exact plane exists to carry,
    breaking `SPEC-025`'s `--raw-checksum` by construction. Saturation is a
    develop-stage decision (STAGE-007+), made against the full levels model, not
    a thing unpack may silently do.

- **Option C: keep `Error::SampleExceedsWhiteLevel` behind an opt-in strict
  mode, default off.**
  - What it is: a parameter that re-enables the assertion for callers who want
    it.
  - Why rejected: no caller needs it, and nothing in PROJ-002 is a strict-mode
    consumer. It would be dead code with no test exercising the live path — the
    opposite of the "every oracle ships proven red" bar. If a real strict-mode
    need appears, it is its own spec with its own test; reserving a variant for
    it now is speculative.

- **Option D (chosen): remove the assertion and the error variant; preserve raw
  values; move the byte-swap guard to the oracle.**
  - What it is: `unpack_into` returns `Ok` for any decoded plane that passes the
    structural checks (bit depth, strip layout, per-strip layer-0, bounds,
    buffer length); above-white samples pass through unchanged.
  - Why selected: it matches what the format actually guarantees (the unpacker's
    job is to reproduce the bytes, not to judge them), it unblocks SPEC-025, and
    it is honest about the one thing it costs — see Consequences.

## Consequences

- **Positive.** Real Bayer files unpack. Both corpus Bayer DNGs
  (`CANON-EOS-5D-MARK-III`, `NIKON-D300S`) now produce a full `width × height`
  plane with their above-white samples preserved, unblocking `SPEC-025`'s
  bit-exact oracle and everything downstream of the colour plane.
- **Positive.** The unpacker's contract is simpler and truer: it reproduces the
  packed bytes and does not impose a value ceiling the format does not define.
- **Negative — the honest cost.** DEC-008's **free, in-library** byte-swap guard
  for the byte-aligned (8/16-bit) path is **gone**. A 16-bit per-sample byte-swap
  that preserves total length (SPIKE-002's exact shape) will now pass
  `unpack_into` silently. That detection moves to two places, neither free in the
  same way: **(1)** `SPEC-025`'s bit-exact mosaic-plane oracle (the plane's MD5
  equals `dnglab analyze --raw-checksum`), which catches a byte-swap decisively
  but needs the corpus + `dnglab`; and **(2)** the per-strip layer-0 packed-bits
  check (`StripByteCounts[i]` is the strip's own `rows·width·bits` rounded up to a
  byte), which is free but does **not** catch a byte-swap (it preserves length).
  So: for the byte-aligned path, the free guard is reduced to the length check,
  and the real byte-swap guard is now the oracle. This trade is acceptable only
  because SPEC-025 makes that oracle a blocking part of the plane's contract.
- **Neutral.** DEC-008's two-path rule, its 12/14-bit MSB-first bitstream, and
  the 8/16-bit byte-order path are all unchanged. The sub-byte path's layer-0
  check is unaffected. `irr unpack` now prints the real relation
  (`15464 > 15000`) rather than a hardcoded `<=`.

## Validation

Right if: the Canon and Nikon Bayer planes unpack `Ok` with their above-white
samples present and unchanged; the monochrome regression (Q2M 14-bit, M
Monochrom 16-bit, all within `WhiteLevel`) stays byte-identical; and `SPEC-025`'s
oracle, once it lands, both matches `dnglab --raw-checksum` on a correct plane
**and** is shown red by a per-sample byte-swap — confirming the guard genuinely
moved rather than vanished.

Revisit if: a future caller needs reject-above-white behaviour (then an opt-in
strict mode is its own spec, Option C re-opened with a live consumer), or if
SPEC-025's oracle proves too coarse to localise a byte-swap the retired
assertion once pinpointed.

## References

- Amends: `DEC-008` — the `max <= WhiteLevel` "permanent invariant" clause only;
  its byte-alignment two-path rule stands.
- Related specs: `SPEC-029` (this change), `SPEC-025` (the bit-exact oracle that
  now carries the byte-swap guard), `SPEC-028` (the multi-strip assembly the
  Nikon plane exercises), `SPEC-012` (the original single-strip unpacker).
- Evidence: `CANON-EOS-5D-MARK-III/5G4A9394.DNG` (sample 2016443 = 15464 >
  WhiteLevel 15000), `NIKON-D300S/_DSC4615.DNG` (sample 741623 = 3835 > WhiteLevel
  3808), both measured via `irr unpack` against the real files, 2026-10-29.
- Constraint: `no-panics-on-untrusted-input` — `unpack_into` stays panic-free.
