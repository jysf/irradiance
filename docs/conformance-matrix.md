# Conformance matrix and corpus policy

**Every camera gets a row the day it is known, files or not.** A declared-empty
cell is fine; a forgotten one is the failure mode. `coverage` is the load-bearing
column — it is what turns "we never thought about Nikon HE*" into "we declared it
out of scope, here is the row."

## The matrix

| Camera | Sensor | Bits | Container / mode | Corpus | Oracle | Target | Coverage |
|---|---|---|---|---|---|---|---|
| **Leica Q2 Monochrom** | **no CFA** (LinearRaw, 1 sample) | 14 | native DNG, **uncompressed**, single strip | held | ✔ dnglab decodes it | n/a (mono) | **PROJ-001** |
| Nikon P1100 (Coolpix) | Bayer | 12 | **NRW**, uncompressed | wanted | dnglab lists it, 12bit | no | PROJ-002 — bit-depth stress test |
| Nikon D750 | Bayer | 12/14 | NEF — lossless + Lossy(type 2). **No uncompressed option (menu checked)** | wanted | dnglab lists it | **yes — the ColorChecker camera** | PROJ-003, or PROJ-002 via `dnglab convert -c uncompressed` |
| Nikon D3200 | Bayer | 12 | NEF — Lossy(type 2) only | wanted | dnglab lists it | no | PROJ-003 |
| Fujifilm (model TBC) | X-Trans 6x6 | 14 | RAF; also via `dnglab convert` | wanted | dnglab lists 87 Fuji bodies | no | PROJ-002 (converted) / later (native) |
| Canon CR2 | Bayer | 14 | lossless JPEG (SOF-3) | none | — | no | **declared-empty** — cheap after SOF-3; files are free, do not buy a body |
| Canon CR3 | Bayer | 14 | CRX wavelet | none | — | no | **declared-empty** — 2,653 lines in rawler; demand-gated |
| Nikon Z-series HE/HE* | Bayer | 14 | TicoRAW | none | **rawler rejects it outright** (`nef.rs:205`) | no | **declared-empty — CLOSED, not expensive** |

## ⚠ PROJ-001 validates against ONE camera

One body, one firmware, one frame. Everything in `docs/measured-q2m-dng.md` could carry a
Leica-specific assumption nobody notices until a second DNG arrives. A **second native-DNG
source** is cheap insurance and worth adding before STAGE-002 finishes:

| Candidate | Why | Cost |
|---|---|---|
| **Pixel phone** | Google Camera writes clean mosaic Bayer DNG | free if anyone nearby has one |
| **Ricoh GR III/IIIx** | native DNG, common used, same shooting culture | borrowable |
| Sigma fp | native DNG | rarer |
| iPhone ProRAW | ⚠ typically *Linear* DNG — already demosaiced. Verify before relying on it | free |

It is Bayer, so it cannot ship in PROJ-001's develop path — but it can prove the **container
reader** is not Leica-shaped, which is STAGE-001's job. That is the cheap half of the value.

## Gotchas already found

- **The P1100 is a Coolpix, so it shoots `.NRW`, not `.NEF`.** rawler's `nrw.rs`
  is a one-line alias to `NefDecoder`; every Coolpix NRW in its corpus is 12-bit
  uncompressed. No HE/HE* concern — that is Z-series only. It is the *easiest*
  Bayer source owned, not the hardest.
- **Nikon's default "Compressed" NEF is LOSSY** ("Lossy type 2"). Decoding needs
  the **linearization curve**, not just Huffman. Skip it and you get a plausible
  image with silently wrong tonality.
- **The Q2 Monochrom is NOT in `dnglab cameras`** (the plain Q2 is; M Monochrom
  variants are) — yet `dnglab analyze` decodes it fine via the generic DNG path.
  Absence from that list does not mean unsupported.
- A 1/2.3" P1100 sensor is a poor *quality* showcase but an excellent
  *correctness* stress test: 12-bit against the Leica's 14-bit catches hardcoded
  bit-depth assumptions early.

`dnglab cameras --md` emits a markdown coverage baseline worth capturing
periodically and diffing against our own support.

## Corpus policy — two tiers

Real RAW files are 30–60 MB and copyrighted by whoever shot them. Review-site
samples are typically licensed "for personal evaluation," **not redistribution**,
so they cannot be committed. The maintainer's own photographs are the clean answer.

**Tier A — committed, runs in CI.** Small, licence-clean, deterministic:
`dnglab makedng` output with analytically known answers, hand-built headers,
truncated and malformed fixtures for the hostile-input path. Kept small enough
that git history stays sane.

**Tier B — local or fetched, never committed.** Full-size real camera files.
Referenced by a manifest carrying path, expected hash, provenance and licence.
Tests **skip with a clear message** when absent and **run** where the corpus
exists — a skip must be visible, never silent.

Decide storage (git-lfs vs fetch-on-demand) before the first fixture lands.
