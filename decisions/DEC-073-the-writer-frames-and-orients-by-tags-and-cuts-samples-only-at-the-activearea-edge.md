---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-073
  type: decision
  confidence: 0.7                    # the coordinate frame of each OpcodeList3 opcode is probed at SPEC-061 design
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-005
repo:
  id: irradiance

created_at: 2026-11-20
supersedes: null
superseded_by: null

status: proposed                     # a push-back on monorail-000/DEC-075; needs jysf's and onyx's agreement

deciders: [claude]

affected_scope:
  - src/dng/write.rs

tags:
  - dng-writer
  - framing
  - orientation
  - opcodelist3
  - stage-017
  - onyx
---

# DEC-073: The writer frames and orients by tags, and cuts samples only at the ActiveArea edge

## Decision

The caller's framing and orientation choices (`monorail-000/DEC-075` item 1) become
**tags over sensor-order samples**, never a crop or rotation of the samples:

| Framing | Samples written | `ActiveArea` | `DefaultCropOrigin`/`Size` |
|---|---|---|---|
| Full sensor | every sample, margins included | the source's active area | the whole active area |
| Full active area | the active area only | omitted (the whole image) | the whole image |
| Standard crop | the active area only | omitted | the source's standard crop |

`DefaultCrop*` is relative to the active area in DNG, so the source's values carry
over unchanged, and no crop can reach into the margin: under full-sensor framing the
margin is in the samples for a reader that wants it, not in the default view.

- **Orientation** is the `Orientation` tag. Samples stay in sensor order.
- **Cuts happen only at the `ActiveArea` edge.** The stage-3 image `OpcodeList3`
  works on is the active area (`src/warp.rs`: the warp's normalised optical centre is
  relative to the warped image, `(0,0)`–`(w-1,h-1)`), so dropping the margin keeps the
  copied opcode list's meaning. Cutting to the standard crop would move
  `WarpRectilinear`'s optical centre and rescale its radius, so a verbatim
  `OpcodeList3` would then describe a different lens.
- SPEC-061's design probes every opcode in the held files' `OpcodeList3` for its
  coordinate frame before the table above is final. Any opcode whose coordinates are
  not active-area-relative is refused for the framing that would break it, not
  silently rewritten.

## Context

`monorail-000/DEC-075` says "the caller chooses the framing (Standard crop, full
active area or full sensor) and the orientation, as for every export", and also keeps
`OpcodeList3` verbatim (O2) so the next editor applies the lens correction. Those two
only agree if the framing does not move the coordinate frame the opcodes refer to.
For the app's other exports (TIFF, PNG) a crop and rotation are baked into pixels;
for a DNG that keeps `OpcodeList3`, baking them in would make the kept opcodes wrong.

## Alternatives Considered

- **Option A: crop and rotate the samples, as the raster exports do.**
  - Why rejected: breaks `OpcodeList3` under standard crop and under any rotation
    (the warp is not rotation-invariant when the image is not square), and loses
    DNG's non-destructive crop, which editors let the user undo.
- **Option B: crop and rotate the samples, and rewrite `OpcodeList3` to match.**
  - Why rejected: the writer would interpret and edit opcode bytes, and every opcode
    type would need its own rewrite rule. Out of v1's scope for no user-visible gain.
- **Option C (chosen): tags over sensor-order samples, cut only at the active area.**

## Consequences

- **Positive:** the copied `OpcodeList3` stays correct at every framing; editors show
  the standard crop and orientation and the user can still reset them.
- **Negative:** a "Standard crop" DNG holds the whole active area, so it is larger than
  a pixel-cropped file, and a reader that ignores `DefaultCrop` shows the full active
  area. onyx must agree that this is what "Standard crop" means for Linear DNG.
- **Neutral:** full-sensor framing writes `MaskedAreas` only from the caller's list
  (DEC-071 §6).

## Validation

- SPEC-061: for each framing, irradiance re-parses the output and its develop gives the
  same result as developing the source at that framing; `dng_validate` accepts each.
- Red-proof: a writer that cut samples to the standard crop fails the develop
  comparison on a file with a non-centred warp.

## References

- Related specs: SPEC-061
- Related decisions: DEC-059 (full-active-area framing and the warp), DEC-071
- External: `monorail-000/DEC-075` item 1 and O2
