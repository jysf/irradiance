# Provenance ledger

**Every algorithm and every decoder in `irradiance` gets a row here, recording
where it came from and under what licence.**

This exists because the licence a crate *declares* and the provenance its code
*carries* are different things, and `cargo deny` only sees the former. The
motivating example is real: `demosaic` 0.3.0 ships `MIT OR Apache-2.0`, and its
`markesteijn_impl.rs:11` says *"Ported from LibRaw's `xtrans_interpolate(1)`"* —
LibRaw being LGPL-2.1/CDDL. That crate passes a licence gate green while carrying
a self-documented port of copyleft code.

The ledger is also the thing that makes this library's permissive claim
*defensible* rather than merely asserted. No other RAW project in any language
has one.

## The rule

Ranked best to worst. Prefer the highest available:

1. **A published specification** — zero contamination. The Adobe DNG Specification
   is public and carries a patent grant for compliant implementations.
2. **A published paper** — the algorithm, not someone's code. Malvar-He-Cutler
   (ICASSP 2004) prints its kernels; Hirakawa & Parks likewise.
3. **Our own prior code** — e.g. crustyimg, same author.
4. **Reading a permissively-licensed implementation** — allowed, must be recorded,
   and the source licence's terms must be honoured.
5. **Reading a copyleft implementation** — **not permitted.** If an algorithm is
   only available this way, it does not ship in the default build; it needs its
   own decision, and probably an off-by-default feature.

"I read it years ago and reimplemented from memory" is row 5, not row 3. Record it
honestly.

## The ledger

| Module / algorithm | Source | Source licence | Provenance class | Notes |
|---|---|---|---|---|
| *(first entry lands with STAGE-001's IFD reader)* | Adobe DNG Specification | public spec + patent grant | 1 — specification | Implement from the **spec**, not the DNG SDK: the SDK's terms are ambiguous and its patent licence reportedly does not cover it |
| `tests/support/corpus.rs::sha256` — SHA-256 | FIPS PUB 180-4 §§4.2.2, 5.1.1, 5.3.3, 6.2.2 | public standard (US Government work, no licence restriction) | 1 — specification | **Dev-only test support; not in the library.** Written from the published standard, not from any implementation. Verifying corpus files against the `sha256` DEC-003 pins. Proven against the published NIST vectors (`""`, `"abc"`, the 448-bit vector, 10⁶ × `'a'`) in `tests/corpus_manifest.rs`, and cross-checked against all 7 real corpus files whose digests were recorded independently (raw.pixls.us' own DB and `shasum`). A file-integrity check, not a security boundary. See DEC-010 for why this is not a dependency |

## Standing decisions

- **`demosaic` is a dev-time oracle only, never a dependency** — see above.
- **`rawler`/`rawloader` are never linked**, not even as dev-dependencies, without
  a decision. `dnglab` is used as a *tool*, which imposes nothing.
- **Patents:** the 20-year term puts MHC (2004) and AHD (2005) at or past expiry,
  VNG (1999) long expired, and the Bayer CFA patent (1976) expired ~1993. The DNG
  container carries an explicit grant. Confirm before relying on any of this.

## This library's own licence

**`MIT OR Apache-2.0`** — the Rust ecosystem's dual-licence convention, and
crustyimg's. Apache-2.0 alone would be more restrictive than the norm and would
undercut the entire pitch: permissive *is* the differentiator, and the whole
reason this library exists is that every mature alternative in every language is
copyleft or C++.

`LICENSE-MIT` and `LICENSE-APACHE` both live at the repo root.
