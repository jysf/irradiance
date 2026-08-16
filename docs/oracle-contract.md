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
| Metadata | `dnglab analyze --meta --json` (or `--yaml`) | exact, machine-diffable |
| File structure | `dnglab analyze --structure` | validates the IFD walk |
| **Sensor plane** | `dnglab analyze --raw-checksum` | **bit-exact** |
| Developed output | `dnglab analyze --srgb` → 16-bit sRGB TIFF | SSIMULACRA2, tolerance |

There is also a **layer 0** that costs nothing: the packing arithmetic must
reproduce `StripByteCounts` exactly. See `measured-q2m-dng.md`.

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

## Every oracle must be shown to go red

A green oracle that cannot fail is worse than no oracle. Each layer ships with a
deliberate-fault test: a corrupted tag, an injected off-by-one in the bit
unpacker, a wrong black level. If the fault does not turn the oracle red, the
oracle is not wired to what it claims to check.

## Fixture generation

`dnglab makedng` builds DNGs with **analytically known** answers — `--matrix1/2/3`,
`--illuminant1/2/3`, `--linearization` (named curves or custom), `--wb`,
`--white-xy`, `--dng-backward-version 1.0–1.6`, and `--map 0:raw 0:preview
0:thumbnail 0:exif 0:xmp`. That is how tier A gets populated without shipping
60 MB camera files, and how the dual-illuminant interpolation and
linearization-table paths get tested without owning the camera.

`dnglab convert -c uncompressed` has **no `--linear` option**, so it preserves the
mosaic by construction — unlike Adobe's converter. ⚠ `--embed-raw` defaults
**true**; turn it off (`--embed-raw false --dng-preview false`) or fixtures carry
an entire second RAW inside them.
