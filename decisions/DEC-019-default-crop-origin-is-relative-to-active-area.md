---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-019
  type: decision
  confidence: 0.75
  audience:
    - developer
    - agent

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-09-05
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - src/develop.rs

tags:
  - decode
  - develop
  - geometry
  - dng-spec
  - spec-014
---

# DEC-019: `DefaultCropOrigin` is applied relative to `ActiveArea`, not the raw plane

## Decision

`develop::resolve_geometry` computes the crop rectangle's raw-plane position
as `ActiveArea.origin + DefaultCropOrigin`, **not** `DefaultCropOrigin` alone
against the uncropped plane's own `(0, 0)`.

## Context

`SPEC-014`'s design-time probe found the exact hazard `SPIKE-001` warned
about, reproduced on this spec's own input surface: **on every decodable
corpus file, `ActiveArea`'s origin is `(0, 0)` or the tag is absent
entirely.** The only file with a non-zero origin — `K3III.DNG`, `top 34, left
26` — is `Compression 7` (JPEG), which this library does not decode. So
*"relative to `ActiveArea`"* and *"relative to the raw plane"* produce
**byte-identical output on every file this spec can actually run against.**
An implementation that silently picked the wrong reading would pass every
corpus test that exists — `SPIKE-001`'s *"the parameter was always 14"*
shape, verbatim, on a different tag.

Confidence is **0.75, not higher**, precisely because of that blind spot:
this decision is supported by independent tool evidence (below) and a
hand-built fixture, but by **zero** decodable real-world files with a
non-zero `ActiveArea` origin. It should be revisited the day a decodable file
with one arrives.

**Independent evidence, so this did not have to be a guess:**
`tests/metadata_oracle.rs`'s `dnglab_crop_origin_is_active_area_plus_default_crop_origin`
(`SPEC-005`) already established that `dnglab analyze --meta --json` reports
`cropArea.p` **sensor-absolute** — verified on all six DNG corpus files, and
concretely on `K3III.DNG`: `(26, 34) + (28, 24) = (54, 58)`, exactly what
`dnglab` prints — while `exiftool` reports the file's own, `ActiveArea`-local
`28 24`. Two independent tools, two different conventions for the *same*
tag, and the arithmetic between them is only consistent if `DefaultCropOrigin`
is read as relative to `ActiveArea`.

## Alternatives Considered

- **Option A: `DefaultCropOrigin` relative to the raw plane's `(0, 0)`,
  ignoring `ActiveArea`'s own origin.**
  - What it is: crop the plane directly at `DefaultCropOrigin`, treating
    `ActiveArea` as only a size constraint (or ignoring it for cropping
    purposes beyond its own edges).
  - Why rejected: contradicted by the `dnglab`/`exiftool` cross-check above —
    `dnglab`'s sensor-absolute figure only reproduces by *adding* the two
    origins, which is only meaningful if the DNG specification's own
    `DefaultCropOrigin` is defined relative to `ActiveArea`. Adopting this
    reading would also make `docs/oracle-contract.md`'s existing, already-
    verified cross-check a coincidence rather than a proof, which is not a
    coherent position to hold.

- **Option B (chosen): `DefaultCropOrigin` relative to `ActiveArea`'s own
  origin — the DNG specification's stated definition.**
  - Why selected: matches the independent tool evidence exactly, on the one
    file that can distinguish the two readings at all (`K3III.DNG`, via the
    metadata oracle rather than the pixel path, since the file itself is
    undecodable). `AC4`'s hand-built fixture
    (`crop_origin_is_relative_to_active_area_not_the_raw_plane`,
    `src/develop.rs`) proves the chosen reading is actually wired up, not
    merely documented.

## Consequences

- **Positive.** The reading is independently corroborated (two tools,
  cross-checked, already a live oracle assertion since `SPEC-005`) rather
  than invented for this spec.
- **Negative.** The one thing that can observe this decision failing is a
  hand-built fixture, not a real file — if the hand-built fixture itself has
  a latent bug, nothing in the corpus would catch a regression here. This is
  named, not hidden: `hostile_geometry_does_not_panic` and
  `crop_origin_is_relative_to_active_area_not_the_raw_plane` are the entire
  safety net until a decodable non-zero-origin file exists.
- **Neutral.** `AC3`'s two measured examples (`L1021223.DNG`,
  `L1000622.DNG`) both have `ActiveArea` origin `(0, 0)` or absent, so their
  expected output dimensions are identical under either reading — they
  confirm the *arithmetic*, not this *decision*.

## Validation

**Right if** a future decodable file with a genuinely non-zero `ActiveArea`
origin develops correctly under this reading — the day such a file exists,
promote it from a "wanted" row in `docs/conformance-matrix.md` to a
confidence-raising real-world confirmation of this record.

**Wrong if** such a file ever develops with the crop visibly offset from
where a reference renderer places it — reopen this immediately; `K3III.DNG`
becoming decodable (a lossless JPEG SOF-3 decoder, PROJ-003) would settle it
outright since that file's own metadata is the very evidence cited above.

## References

- Related specs: SPEC-005 (the live metadata oracle that first established
  the `dnglab`/`exiftool` convention split), SPEC-014
- Related decisions: DEC-004 (why no comparison oracle covers this spec at
  all), DEC-012 (structure/interpretation split this tag falls under)
- Evidence: `tests/metadata_oracle.rs::dnglab_crop_origin_is_active_area_plus_default_crop_origin`,
  `docs/oracle-contract.md`'s "`dnglab`'s `cropArea.p` is sensor-absolute" section
- `spikes/done/SPIKE-001-*.md` — the *"the parameter was always 14"* shape
  this decision is a second instance of
