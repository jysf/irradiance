# The oracle contract

`irradiance` verifies itself against **dnglab**, run as a tool. Three layers, all
shell commands. Establish this before writing decoder code — an oracle whose
contract you assumed rather than measured is not an oracle.

```bash
brew install dnglab   # 0.7.2, bottled -> native arm64, no Rosetta
```

**Licence:** dnglab is LGPL-2.1. That is fine *because it is run, not linked*.
`irradiance` must never take `rawler`/`rawloader` as a dependency, including a
dev-dependency, without its own decision.

## The three layers

| Layer | Command | Verdict |
|---|---|---|
| Metadata | `exiftool -T -n -s3` (primary) + `dnglab analyze --meta --json` (six-DNG cross-check) — see below | exact, machine-diffable, live (`SPEC-005`) |
| File structure | `dnglab analyze --structure` | validates the IFD walk |
| **Sensor plane** | `dnglab analyze --raw-checksum` | **bit-exact** |
| Developed output | `dnglab analyze --srgb` → **16-bit PNM, not a TIFF** ⚠ | SSIMULACRA2 **≥ 85** (DEC-005) |

There is also a **layer 0** that costs nothing: the packing arithmetic must
reproduce `StripByteCounts` exactly. See `measured-q2m-dng.md`.

## ⚠ The metadata layer is TWO tools, not one — measured 2026-08-21

`SPEC-005` built the live oracle. `exiftool 13.55` and `dnglab analyze --meta
--json` are BOTH run, and neither one alone is "the metadata layer": they
answer different questions, verified across all seven corpus files before a
line of the comparator was written (`tests/support/tools.rs`,
`tests/metadata_oracle.rs`).

- **`exiftool` reads what the file SAYS**, per IFD — the ground truth for
  every tag `Sensor` carries, absence included. It is the primary tag-level
  oracle, run on all seven files.
- **`dnglab` reports what a DECODER CONCLUDED**, through rawler's camera
  database. On `K3III.PEF` — a vendor container with no DNG tags at all —
  `dnglab` still answers (`black 64`, `white 16378`, an `activeArea` and a
  `cropArea`, `bitDepth 16`), and `exiftool`/our reader both agree the file
  contains none of them. Treating the two tools as interchangeable would
  produce an oracle that demands the reader hallucinate values the file does
  not contain — so `dnglab`'s comparison is scoped to the **six DNG files
  only**, by name, with the divergence asserted rather than ignored.
- **`dnglab`'s `cropArea.p` is sensor-absolute; ours and exiftool's are
  DNG-relative.** `dnglab.cropArea.p == active_area.(left, top) +
  crop_origin`, verified on all six DNGs — `K3III.DNG`: `(26, 34) + (28, 24)
  = (54, 58)`, exactly what dnglab prints. A naive direct comparison would
  have called the correct reader wrong.
- **No new dependency was needed.** `exiftool -T -n -s3` emits one
  tab-separated line, values in the order requested, `-` for an absent tag —
  no parser required. `dnglab`'s JSON is read by asserting the handful of
  keys this oracle needs (`rawWidth`, `rawHeight`, `bitDepth`, `whitelevels`,
  `orientation`, `blacklevels.levels`, and `cropArea`) are unique in the
  document before trusting a match — `x`/`y`/`w`/`h` are NOT unique (they
  appear under both `cropArea` and `activeArea`), which is why `cropArea.p`
  is extracted from a brace-matched substring rather than a bare search.
- ⚠ **Capture `dnglab`'s STDOUT only.** On `K3III.DNG` it writes an ANSI
  warning to stderr — `File has BlackLevelRepeatDim tag but with invalid
  length: 1` — and merging the streams (`2>&1`) makes the JSON unparseable at
  byte 1. `std::process::Command::output()` keeps the two streams separate by
  construction, which is the fix, not a flag.
- ⚠ **`exiftool`'s exit code carries no signal.** It exits 0 on a truncated
  file and on an absent tag alike — measured on the first 4 KB of a Q2M frame,
  which still yields `ImageWidth 8424`. Only stdout is trusted.
  `dnglab`'s exit code DOES carry signal (it exits 2 on the same truncated
  input) and `dnglab_meta` checks it.
- `blacklevels.levels` is an array of **rational strings** (`["512/1"]`, not
  `[512]`) — parsed as `N/D`, asserting `N % D == 0` rather than assuming
  `D == 1`.

This layer stays **tag-extraction correctness only**: whether our reader
reproduces the same value the file's bytes encode. Whether that value is the
*right* one — levels normalization, crop correctness — is verified
analytically per `DEC-004`, never by comparison; see that record before
extending this oracle to cover it.

## ⚠ The plane contract — VERIFIED 2026-08-15, not assumed

> `dnglab analyze --raw-checksum` = **MD5 of the uncropped `u16` plane, native
> little-endian, 14-bit values zero-extended, no black subtraction, no crop.**

That is the representation the decoder holds in memory anyway, so the oracle is:
decode → hash the buffer → compare one string.

Three wrong guesses preceded that line. Do not re-derive them:

- **`--full-pixel` is NOT the sensor plane.** It is the *preview*, as PPM:
  `P6 1620 1080 255\n` (17 B) + 1620×1080×3 = **5,248,817** exactly.
- **`--raw-pixel` IS the sensor plane, as PGM:** `P5 8424 5632 65535\n` (19 B) +
  8424×5632×2 = **94,887,955** exactly. **Uncropped** — not `ActiveArea`
  (8392×5632), not `DefaultCropSize` (8368×5584). The comparison therefore
  attaches BEFORE the three-stage crop.
- **The PGM payload is BIG-endian** (PNM spec) while the checksum is **native
  LE**. That alone is why a naive `md5` mismatches.

Proof of the endianness, which also proves the values are zero-extended rather
than scaled: file bytes `02 34 | 02 4A` read big-endian are **564, 586** — just
above `BlackLevel` 512. Read little-endian they are 13314, **18946**, and 18946
exceeds `WhiteLevel` 16383, which is impossible.

Reproduce the checksum from the PGM stream:

```bash
dnglab analyze --raw-pixel F.DNG | tail -c +20 | dd conv=swab 2>/dev/null | md5
```

### Re-verified 2026-08-16 on a SECOND frame

The contract above was established on `L1025901.DNG`. It was re-run end to end on
an independent frame, `L1021223.DNG` (86 MB, `LEICA Q2 MONO`), with dnglab 0.7.2:

| Check | Result |
|---|---|
| PGM header | `P5 8424 5632 65535\n` — **19 bytes**, exactly as documented |
| Stream size | **94,887,955** = 19 + 8424×5632×2, to the byte |
| `--raw-checksum` | `cb653b5bec24d166eef2fd258ee61ac4` |
| `--raw-pixel \| tail -c +20 \| dd conv=swab \| md5` | `cb653b5bec24d166eef2fd258ee61ac4` — **identical** |

So the contract is no longer one-file evidence. Two frames, same camera and
firmware — which raises confidence in the *representation* claim (uncropped, native
LE, zero-extended) considerably, and says nothing yet about other cameras.

The endianness proof reproduces too: this frame's first payload bytes are
`02 EA`, big-endian **746** — just above `BlackLevel` 512. Read little-endian
they would be 59906, far beyond `WhiteLevel` 16383, which is impossible.

When a comparison fails, `--raw-pixel | tail -c +20 | dd conv=swab` hands you the
reference bytes to diff, and the PGM header gives you width and height to convert
a byte offset into a pixel coordinate.

## ⚠ This oracle is single-sourced

`--raw-checksum` and `--srgb` both come from **rawler**. Bit-exact agreement proves we match
rawler, not that we are correct.

- **Sensor plane — acceptable.** Decompression is deterministic and rawler is the de facto
  reference; matching it exactly *is* the goal. The layer-0 packing arithmetic is independent
  and should be kept for that reason.
- **Developed output — weak.** Matching rawler's tone and rendering choices is not being right,
  and a tolerance test can pass while both are wrong together.

Two mitigations, in order of availability:

1. **Analytic fixtures via `dnglab makedng`** — known levels, known curves, known matrices, so
   correctness is arithmetic. An arithmetic check cannot inherit rawler's bugs. Available now.
2. **ColorChecker ΔE** against published patch values — the only fully independent check, and
   the only one that is absolute rather than comparative. PROJ-002, and it needs a colour camera.

Never describe this oracle as proving correctness without naming which layer is meant.


## ⚠ Layer 3 is not what this document originally said — measured 2026-08-18

`dnglab analyze --srgb` is documented by its own `--help` as "16-bit sRGB TIFF".
**It is not a TIFF.** It writes a PNM — and on a *monochrome* file it writes a
**`P6` (RGB) header over a `P5` (grayscale) payload**: exactly `w*h*2` bytes where
the header declares `w*h*3*2`. One third of the promised data. Reproduced twice
at byte-identical size (93,453,843 = 19-byte header + 8368x5584x2).

A conforming PNM reader errors or reads garbage. The workaround (DEC-005) is to
**assert the payload length is `w*h*2`, then rewrite the header** as
`P5 <w> <h> 65535`; the payload is then a valid 16-bit grayscale image, verified
to decode as a real photograph. The length assertion is not optional — if a future
dnglab emits a correct `P6`, blindly forcing `P5` would silently halve the image.

```bash
{ printf 'P5 8368 5584 65535\n'; dnglab analyze --srgb F.DNG | tail -c +20; } > ref.pgm
```

## ⚠ What the layers do NOT cover — measured 2026-08-18

**Neither the plane layer nor the develop layer catches a levels error.** The
plane layer is blind by construction (it hashes with no black subtraction). The
develop layer is blind because SSIMULACRA2 is *perceptual* and a levels error is
nearly an affine tone change: a `BlackLevel` wrong by **+256 — half the true
black level — still scores 87.5**, passing the ≥85 bar.

**Levels, crop and orientation are therefore verified analytically (DEC-004), not
by any oracle in this document.** Do not add a perceptual check for them; that is
a category error, and it was very nearly shipped as one.

## Every oracle must be shown to go red

A green oracle that cannot fail is worse than no oracle. Each layer ships with a
deliberate-fault test: a corrupted tag, an injected off-by-one in the bit
unpacker, a wrong black level. If the fault does not turn the oracle red, the
oracle is not wired to what it claims to check.

## Fixture generation

`dnglab makedng` builds DNGs with **analytically known** answers — `--matrix1/2/3`,
`--illuminant1/2/3`, `--linearization` (named curves or custom), `--wb`,
`--white-xy`, `--dng-backward-version 1.0–1.6`, and `--map 0:raw 0:preview
0:thumbnail 0:exif 0:xmp`. That is how the dual-illuminant interpolation and
linearization-table paths get tested without owning the camera.

### ⚠ But makedng CANNOT build a monochrome fixture — measured 2026-08-16

This is a real limit on tier A, and it lands squarely on PROJ-001's critical path.
Measured against dnglab 0.7.2 by running it:

- **It accepts PPM only.** TIFF, PGM, PNG and JPEG are each rejected with
  `Error: Input format is not supported`. PPM is RGB by definition, and **PGM —
  the one grayscale format — is refused.**
- **What it emits**, in the full-resolution SubIFD, from a 16-bit PPM:

  | Tag | makedng output | A real Q2M |
  |---|---|---|
  | `SamplesPerPixel` | **3** | **1** |
  | `BitsPerSample` | **16 16 16** | **14** |
  | `Compression` | **JPEG** | **Uncompressed** |
  | `PhotometricInterpretation` | Linear Raw | Linear Raw ✔ |

- `--linearization` changes none of those three.
- `dnglab analyze --raw-checksum` *does* run on the result, so the oracle
  plumbing can be exercised against a synthesized file.

**Consequences, and they are load-bearing:**

1. Tier A can exercise the **metadata** oracle (STAGE-001) and the oracle harness
   itself. It **cannot** exercise STAGE-002's 14-bit packed monochrome unpack —
   there is no makedng path to a 1-sample plane.
2. Worse, makedng's output is **JPEG-compressed**, so decoding a tier-A fixture's
   plane would require **lossless JPEG SOF-3** — the one decoder PROJ-001
   explicitly declares out of scope. A tier-A fixture is therefore not merely a
   weaker substitute for the pixel path; for that path it is unreachable.
3. The route to a monochrome fixture is the **hand-built header** option that
   `conformance-matrix.md` also lists. That option is now load-bearing rather than
   a nice-to-have, and STAGE-001's corpus spec should be shaped around it.

Not yet tested: whether `dnglab convert -c uncompressed` on an existing mono RAW
produces a usable 1-sample fixture. That needs an input camera file, so it belongs
in SPIKE-001 rather than here.

`dnglab convert -c uncompressed` has **no `--linear` option**, so it preserves the
mosaic by construction — unlike Adobe's converter. ⚠ `--embed-raw` defaults
**true**; turn it off (`--embed-raw false --dng-preview false`) or fixtures carry
an entire second RAW inside them.
