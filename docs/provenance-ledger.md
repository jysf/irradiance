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
| `src/ifd.rs` — TIFF/IFD container walk, SubIFD recursion, sensor-IFD selection | TIFF 6.0 (1992) §2 "Image File Header" + §2 "Image File Directory"; Adobe DNG Specification 1.7.1.0 (DNG-private tags 50713/50714/50717/50719/50720/50829/51008/51009/51022, `PhotometricInterpretation = LinearRaw` 34892) | public specifications; DNG carries a patent grant for compliant implementations | 1 — specification | **SPEC-003.** Written from the specs, not from any implementation. `dnglab`/`rawler` (LGPL-2.1) were **run as tools** to produce the ground truth this reader is checked against, and their source was not read — running imposes nothing, reading would be a `provenance-recorded-per-algorithm` violation. `SPIKE-001`'s decoder was discarded unmerged and deliberately not consulted (`test-before-implementation`). Tag numbers and field types were re-derived by reading the real files' bytes; the expected values are cross-checked against `exiftool 13.55` on all 7 corpus files in `tests/ifd_reader.rs`. Implement from the **spec**, not the DNG SDK: the SDK's terms are ambiguous and its patent licence reportedly does not cover it |
| `tests/support/corpus.rs::sha256` — SHA-256 | FIPS PUB 180-4 §§4.2.2, 5.1.1, 5.3.3, 6.2.2 | public standard (US Government work, no licence restriction) | 1 — specification | **Dev-only test support; not in the library.** Written from the published standard, not from any implementation. Verifying corpus files against the `sha256` DEC-003 pins. Proven against the published NIST vectors (`""`, `"abc"`, the 448-bit vector, 10⁶ × `'a'`) in `tests/corpus_manifest.rs`, and cross-checked against all 7 real corpus files whose digests were recorded independently (raw.pixls.us' own DB and `shasum`). A file-integrity check, not a security boundary. See DEC-010 for why this is not a dependency |

## ⚠ The distinction runs BOTH ways — measured 2026-08-20

The motivating example above is a crate that **declares better than it carries**
(`demosaic` ships MIT/Apache over a self-described LGPL port). `libfuzzer-sys`
runs the **inverse**, and the ledger's framing assumed that could not happen:

| | declares | actually carries |
|---|---|---|
| `demosaic` 0.3.0 | `MIT OR Apache-2.0` | a port of LGPL LibRaw code |
| `libfuzzer-sys` 0.4.13 | `(MIT OR Apache-2.0) AND NCSA` | **`Apache-2.0 WITH LLVM-exception`, in all 55 vendored files** |

Counted, not sampled: 49 files at the vendored top level, 55 including
`afl/` and `+dataflow/`, **every one** carrying the LLVM Apache header, and
**0 of 56** mentioning NCSA anywhere. The crate's SPDX expression *and* its own
README are both stale against its code.

Nothing is wrong here — everything involved is permissive, and `deny.toml` enforces
the **stricter** declared reading via a named per-crate exception rather than
widening `allow`. But it means the ledger's question is not "is the declaration too
generous?" It is **"does the declaration match the code, in either direction?"**
A stricter-than-reality declaration is harmless for compliance and still a fact the
ledger should carry, because the next reader will otherwise re-derive it.

Reached only by `just deny-fuzz`; `libfuzzer-sys` is not in the library's graph.

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
