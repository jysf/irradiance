# Conformance matrix and corpus policy

**Every camera gets a row the day it is known, files or not.** A declared-empty
cell is fine; a forgotten one is the failure mode. `coverage` is the load-bearing
column — it is what turns "we never thought about Nikon HE*" into "we declared it
out of scope, here is the row."

## The matrix

| Camera | Sensor | Bits | Container / mode | Corpus | Oracle | Target | Coverage |
|---|---|---|---|---|---|---|---|
| **Leica Q2 Monochrom** | **no CFA** (LinearRaw, 1 sample) | 14 | native DNG, **uncompressed**, single strip | held ×3 | ✔ dnglab decodes it | n/a (mono) | **PROJ-001** — reference body; container read end-to-end (SPEC-003) |
| **Leica M Monochrom** | **no CFA** (LinearRaw, 1 sample) | **16** | native DNG, **uncompressed**, single strip | held (CC0) | ✔ dnglab decodes it | n/a (mono) | **PROJ-001** — container read end-to-end (SPEC-003). ⭐ The only *third-party* file that decodes today, and DEC-008's 16-bit branch evidence: a third bit depth, BlackLevel 220 not 512, and the only **non-zero `ActiveArea` origin** held (2 2 5212 3468), so the crop has somewhere to move. No `OpcodeList` tags at all — the no-opcodes path |
| **Leica M Monochrom (Typ 246)** | **no CFA** (LinearRaw, 1 sample) | **12** | native DNG, **JPEG** (`Compression 7`), **`MM`** | held (CC0) | ✔ dnglab decodes it | n/a (mono) | **PROJ-001 container only** — tags read end-to-end (SPEC-003), plane rejected with `Error::UnsupportedCompression`. The corpus's **only big-endian file**, and its only 12-bit one. Decode waits on lossless JPEG SOF-3 → **PROJ-003** |
| **Pentax K-3 III Monochrome** | **no CFA** (LinearRaw, 1 sample) | 14 | **two containers, same scene**: DNG **JPEG** (`Compression 7`) and native **PEF** (`Compression 65535`, vendor-private) | held ×2 (CC0) | ✔ dnglab decodes both | n/a (mono) | **PROJ-001 container only** — tags read end-to-end (SPEC-003), both planes rejected cleanly. A monochrome sensor from a different **make**. The PEF is the corpus's **only real IFD chain** (`IFD0→IFD1→IFD2`), has **no `SubIFDs` tag at all** (plane in `IFD0`), and the DNG carries the malformed `BlackLevelRepeatDim`. Decode → **PROJ-003**; PEF and SOF-3 are *different* problems |
| Nikon P1100 (Coolpix) | Bayer | 12 | **NRW**, uncompressed | wanted | dnglab lists it, 12bit | no | PROJ-002 — bit-depth stress test |
| Nikon D750 | Bayer | 12/14 | NEF — lossless + Lossy(type 2). **No uncompressed option (menu checked)** | wanted | dnglab lists it | **yes — the ColorChecker camera** | PROJ-003, or PROJ-002 via `dnglab convert -c uncompressed` |
| Nikon D3200 | Bayer | 12 | NEF — Lossy(type 2) only | wanted | dnglab lists it | no | PROJ-003 |
| Fujifilm (model TBC) | X-Trans 6x6 | 14 | RAF; also via `dnglab convert` | wanted | dnglab lists 87 Fuji bodies | no | PROJ-002 (converted) / later (native) |
| Canon CR2 | Bayer | 14 | lossless JPEG (SOF-3) | none | — | no | **declared-empty** — cheap after SOF-3; files are free, do not buy a body |
| Canon CR3 | Bayer | 14 | CRX wavelet | none | — | no | **declared-empty** — 2,653 lines in rawler; demand-gated |
| Nikon Z-series HE/HE* | Bayer | 14 | TicoRAW | none | **rawler rejects it outright** (`nef.rs:205`) | no | **declared-empty — CLOSED, not expensive** |

## ⚠ PROJ-001 validates against ONE camera — at the DEVELOP layer. The CONTAINER half is done.

✅ **Updated 2026-08-20 (SPEC-003).** This section said "one body, one firmware,
one frame" and it is no longer true at the layer where it was cheapest to fix.
The container reader is now exercised against **four bodies, three makes-worth of
firmware and seven files**, all read end-to-end and cross-checked against
`exiftool 13.55` — which is exactly the *"prove the **container reader** is not
Leica-shaped"* job the closing paragraph of this section calls STAGE-001's, and
"the cheap half of the value". That half is spent, and it bought:

- a **third and fourth bit depth** (12 and 16, against the Q2M's 14),
- the only **big-endian** file held, so byte-order handling has a real test,
- a **non-zero `ActiveArea` origin**, so the crop has somewhere to move,
- a file with **no `SubIFDs` tag**, which makes TIFF's *absent-means-0* default
  for `NewSubfileType` load-bearing rather than decorative,
- the only **IFD chain** (`IFD0→IFD1→IFD2`) in the corpus,
- a **different make** (Ricoh/Pentax), and a **vendor-private container** (PEF),
- and a real shipping camera's **malformed tag**, free.

✅ **Updated 2026-08-21 (SPEC-005).** "Cross-checked against `exiftool 13.55`"
above was, until this spec, a design-time cross-check frozen into a
hand-transcribed table (`tests/ifd_reader.rs`) — accurate the day it was typed,
unable to notice drift afterward. It is now a **live** oracle
(`tests/metadata_oracle.rs`) that shells out to both `exiftool` and
`dnglab analyze --meta --json` every run, diffs every tag field-by-field, and
ships proven red (a committed fixture red-proof runs in CI with no tool and no
corpus; a real-file red-proof patches a tag in memory and confirms the oracle
names it) — see `docs/oracle-contract.md`'s metadata-layer section.

**What is still one camera is the DEVELOP path**, and that is the part the
paragraph below was really about: only the Q2 Monochrom and the M Monochrom are
*uncompressed*, and everything in `docs/measured-q2m-dng.md` — levels, opcode
lists, geometry — is still measured on one body. STAGE-002 onward inherits that
caveat undiminished. The M Monochrom (16-bit, uncompressed, CC0) is the one file
that narrows it today, and it is why it is worth more than its size suggests.

A **Bayer** native-DNG source remains cheap insurance for the container reader's
*mosaic* assumptions, which no monochrome file can test:

| Candidate | Why | Cost |
|---|---|---|
| **Pixel phone** | Google Camera writes clean mosaic Bayer DNG | free if anyone nearby has one |
| **Ricoh GR III/IIIx** | native DNG, common used, same shooting culture | borrowable |
| Sigma fp | native DNG | rarer |
| iPhone ProRAW | ⚠ typically *Linear* DNG — already demosaiced. Verify before relying on it | free |

It is Bayer, so it cannot ship in PROJ-001's develop path — but it can prove the **container
reader** is not Leica-shaped, which is STAGE-001's job. Four monochrome bodies have now done
most of that (above); what a Bayer file adds specifically is `CFARepeatPatternDim` and friends,
the one tag family a monochrome corpus structurally cannot exercise.

## ⚠ Open question carried out of SPIKE-002 (2026-08-18)

**Does `dnglab convert -c uncompressed` preserve an X-Trans 6×6 mosaic
(`CFARepeatPatternDim: 6 6`), and does it work on a D750 NEF?** Unanswered —
blocked on a Fuji RAF and a Nikon D750 NEF, neither held. SPIKE-002 landed
`answered` on its other two sub-questions; **this one is not covered by that
landing.** It decides whether corpus-widening for PROJ-002 is cheap or expensive,
so reopen it as its own spike when the files arrive.

Also from SPIKE-002, and it has its own row now: the **Pentax K-3 Mark III
Monochrome** DNG carries a tag `dnglab` itself warns about — *"BlackLevelRepeatDim
tag but with invalid length: 1"*. A shipping camera writing a malformed tag that a
mature decoder tolerates is a regression fixture, not a curiosity.

⚠ **It is TIER B, not tier A** — this said tier A, which it cannot be: the file is
37 MB and uncommitted, so it is a fixture only where `$IRRADIANCE_CORPUS_DIR` is
populated, and never in CI. SPEC-003 handled that the right way round: the reader
tolerates a present-but-wrong-length tag by dropping the value and recording the
tag number in `Sensor::malformed_tags`, and the behaviour is pinned by a **tier-A
hand-built fixture** carrying the same defect, which does run in CI. The real file
is the *discovery*; the synthetic one is the *regression test*. Do not confuse
them — the tier-B file cannot gate anything.

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

## ⚠ What the corpus does NOT cover (2026-08-20)

Four bodies and seven files sound like coverage. They are **all tier B**:

- **7 of 7** manifest entries are `tier = "b"` — never committed, absent on any CI
  runner.
- **0** are tier A.

So **none of the corpus runs in CI.** What runs there are the hand-built synthetic
fixtures inside the test modules (e.g. `src/ifd.rs`'s tier-A unit fixture). Real-file
coverage is a property of *this machine*, not of the project — the same conflation
this document corrected twenty lines below for a single fixture, stated here for
the whole corpus.

## Corpus policy — two tiers

Real RAW files are 30–60 MB and copyrighted by whoever shot them. Review-site
samples are typically licensed "for personal evaluation," **not redistribution**,
so they cannot be committed. The maintainer's own photographs are the clean answer.

**Tier A — committed, runs in CI.** Small, licence-clean, deterministic:
`dnglab makedng` output with analytically known answers, hand-built headers,
truncated and malformed fixtures for the hostile-input path. Kept small enough
that git history stays sane.

> ⚠ **Measured 2026-08-16: `makedng` cannot produce a monochrome fixture.** It
> takes PPM input only and emits a 3-sample, 16-bit, **JPEG-compressed** full-res
> SubIFD — so tier A can serve the *metadata* oracle but not STAGE-002's 14-bit
> packed mono unpack, and a tier-A plane would need lossless JPEG SOF-3, which
> PROJ-001 puts out of scope. **Hand-built headers are the route to a mono fixture,
> not `makedng`** — which makes that option load-bearing rather than optional, and
> should shape STAGE-001's corpus spec. Details in `oracle-contract.md`.

**Tier B — local or fetched, never committed.** Full-size real camera files.
Referenced by a manifest carrying path, expected hash, provenance and licence.
Tests **skip with a clear message** when absent and **run** where the corpus
exists — a skip must be visible, never silent.

**Storage decided 2026-08-16 — `DEC-003`.** Neither git-lfs nor (yet)
fetch-on-demand: real files live outside the repo at `$IRRADIANCE_CORPUS_DIR`
and are **never committed**; `tests/corpus/manifest.toml` is committed and carries
each file's path, hash, **licence**, source and **pinned oracle answers**. Tier A
admits only CC0 or own-work. ⚠ Consequence to keep in view: **CI cannot verify
bit-exactness** — tier B is absent on a runner, so a green CI badge does not mean
the decoder is bit-exact.
