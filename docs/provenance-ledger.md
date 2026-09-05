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
| `src/ifd.rs` — TIFF/IFD container walk, SubIFD recursion, sensor-IFD selection, typed tag extraction | TIFF 6.0 (1992) §2 "Image File Header" + §2 "Image File Directory"; Adobe DNG Specification 1.7.1.0 (DNG-private tags 50713/50714/50717/50719/50720/50829/51008/51009/51022, `PhotometricInterpretation = LinearRaw` 34892) | public specifications; DNG carries a patent grant for compliant implementations | 1 — specification | **SPEC-003.** Written from the specs, not from any implementation. `dnglab`/`rawler` (LGPL-2.1) were **run as tools** to produce the ground truth this reader is checked against, and their source was not read — running imposes nothing, reading would be a `provenance-recorded-per-algorithm` violation. `SPIKE-001`'s decoder was discarded unmerged and deliberately not consulted (`test-before-implementation`). Tag numbers and field types were re-derived by reading the real files' bytes; the expected values are cross-checked against `exiftool 13.55` on all 7 corpus files in `tests/ifd_reader.rs`. Implement from the **spec**, not the DNG SDK: the SDK's terms are ambiguous and its patent licence reportedly does not cover it. **SPEC-004** extended this same row's scope, same class: `ActiveArea`/`DefaultCropOrigin`/`DefaultCropSize` as named-field structs (DNG 1.7 §Chapter 4's own field order, no new tags), and FU-11's fix to sensor-IFD selection's error handling (`DEC-012`) — a malformed identifying tag on one IFD no longer aborts the scan of the others. No implementation was consulted for either; both are control-flow/typing refinements of the existing spec-derived rule, re-verified against `exiftool` and re-fuzzed. **SPEC-007** extended this same row's scope, same class, twice: (1) `TYPE_RATIONAL` — TIFF 6.0 §2 "Types" defines it as two `LONG`s, numerator then denominator, read directly from the spec text; a zero denominator or non-integral ratio is `Error::MalformedRationalValue`. (2) `DEC-012`'s amended structure/interpretation split applied to `sensor()` itself (FU-16, FU-17) and to `is_sensor_ifd`'s tag-evaluation order (FU-20) — control-flow refinements of the existing spec-derived rule, not a new algorithm. No implementation was consulted; re-verified against `exiftool` on all 7 corpus files and re-fuzzed (11,553,927 runs, zero crashes). **SPEC-008** extended this same row's scope, same class, three times, all pinning `DEC-012` rather than redrawing it: (1) `uints()`'s `TYPE_RATIONAL` acceptance made **per-tag** (`is_structural_tag`) instead of global (`SPEC-007/FU-4`) — TIFF 6.0 defines every `DEC-012` Structure tag as BYTE/SHORT/LONG only, so RATIONAL is illegal for all of them; this restores `main`'s per-tag behaviour for exactly `DEC-012`'s amended Structure row, while interpretation tags keep the widening `SPEC-007` added. (2) `sensor()`'s `Orientation` read fixed to cost the field **at most once** and only when no valid value was found anywhere (`SPEC-007/FU-1,FU-2`) — a control-flow correction to the existing IFD0-then-sensor-IFD fallback, not a new rule. (3) Four new hand-built fixtures pin `Compression`/`StripOffsets`/`StripByteCounts`/`BitsPerSample` as fatal-when-malformed, alongside the existing `RowsPerStrip` fixture — no new tags, no new read logic. No implementation was consulted for any of the three; all six mutants (the four structural tags, the `SubIFDs` RATIONAL gate, the `Orientation` fallback) were verified to flip the corresponding test red when reverted and green when restored, and the full suite was re-fuzzed (12,971,280 runs, zero crashes). **SPEC-009** extended this same row's scope, same class, adding **zero** new tags and **zero** src/ read logic: it is the table-driven test `SPEC-008` measured missing — `is_structural_tag()` named eleven tags with exactly one (`TAG_SUB_IFDS`) enforced by any test — plus `DEC-015`, which decides `SPEC-008/FU-3` (a well-formed `IFD0` `Orientation` with an erroring sensor-IFD read stays silent in `malformed_tags`, narrowing that field's doc comment to match) without changing `sensor()`'s existing behaviour. All eleven mutations (each membership deleted from `is_structural_tag()` in turn) were watched to fail, and the unmutated tree watched to pass, as the control; the full suite was re-fuzzed (13,541,962 runs, zero crashes) |
| `tests/support/corpus.rs::sha256` — SHA-256 | FIPS PUB 180-4 §§4.2.2, 5.1.1, 5.3.3, 6.2.2 | public standard (US Government work, no licence restriction) | 1 — specification | **Dev-only test support; not in the library.** Written from the published standard, not from any implementation. Verifying corpus files against the `sha256` DEC-003 pins. Proven against the published NIST vectors (`""`, `"abc"`, the 448-bit vector, 10⁶ × `'a'`) in `tests/corpus_manifest.rs`, and cross-checked against all 7 real corpus files whose digests were recorded independently (raw.pixls.us' own DB and `shasum`). A file-integrity check, not a security boundary. See DEC-010 for why this is not a dependency |
| `tests/support/md5.rs` — MD5 | RFC 1321 §§3.1, 3.3, 3.4, 3.5 | public standard (IETF RFC, no licence restriction) | 1 — specification | **SPEC-013.** Dev-only test support; not in the library. Written from the published RFC, not from any implementation — same shape as `sha256` above (`DEC-010`'s reasoning extends unchanged: a hashing crate would only be exercised on a machine holding the corpus, invisible to CI). Proven against all seven of RFC 1321 Appendix A.5's own published test vectors (`md5_matches_the_rfc_1321_test_vectors`, `tests/plane_oracle.rs`, tier A). Also embedded verbatim (`include_str!`) into the plane oracle's red-proof probe binary (`DEC-017`) so the mutated-crate build and the test binary hash with the exact same implementation. Used to check our unpacked plane against `dnglab analyze --raw-checksum`'s pinned digest (`docs/oracle-contract.md`) — a file/output-integrity check, not a security boundary |
| `src/plane.rs` — sensor-plane strip location and sample unpack, `DEC-008`'s two-path rule (MSB-first sub-byte bit stream vs. byte-aligned integers) | TIFF 6.0 (1992) §2 "Compression" (`= 1`, uncompressed); `DEC-008`, itself derived from TIFF 6.0's packing rule | public specification | 1 — specification | **SPEC-012.** The first spec that produces pixels rather than tags. `SPIKE-001`'s decoder is discarded and was **not consulted** (`test-before-implementation`, `provenance-recorded-per-algorithm`) — the bit-cursor arithmetic was re-derived from `DEC-008`'s byte evidence and hand-verified against the measured strip bytes before being written (worked by hand for samples 0-1 of `L1021223.DNG`: `0b a8 2d 50…` MSB-first at 14 bits gives 746, 725, matching `dnglab --raw-pixel`'s own plane exactly). Cross-checked end-to-end against **two independent real files, both bit depths**: `L1021223.DNG` (14-bit, sub-byte path) and `L1000622.DNG` (16-bit, byte-aligned path) — `tests/plane_unpack.rs`'s `unpacks_fourteen_bit_msb_first_samples`/`unpacks_sixteen_bit_in_file_byte_order`, and manually via `irr unpack`. The three compressed corpus files (`M2462362.DNG`, `K3III.DNG`, `K3III.PEF`) are rejected before any strip read (`Sensor::require_uncompressed`), never decoded. No implementation was consulted; fuzzed 13,050,886 runs (45s) reaching both paths via seeds in `fuzz/seeds/plane/`, zero crashes |

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
