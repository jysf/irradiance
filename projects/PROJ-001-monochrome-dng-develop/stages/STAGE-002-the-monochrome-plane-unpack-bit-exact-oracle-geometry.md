---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-002                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: irradiance

created_at: 2026-08-15
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Proves the pixel half of the thesis, and proves it EXACTLY: our decoded sensor plane is byte-identical to an independent implementation's.
  delivers:
    - "Packed 14-bit → u16 unpack of the full sensor plane"
    - "A bit-exact plane oracle against dnglab"
    - "Black/white normalization and the three-stage geometry"
  explicitly_does_not:
    - "Opcodes — bad pixels and lens warp are STAGE-003"
    - "Tone curve or any output encoding"
    - "Demosaic — there is no CFA in a Monochrom file"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# Warn-only, never a gate. A null here is honest; a guess is not. (DEC-013 §5)
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-002: The monochrome plane: unpack, bit-exact oracle, geometry

## What This Stage Is

When this stage ships, `irradiance` produces a correct linear monochrome
sensor plane from a Q2M DNG — and proves it, because the plane's MD5 equals what
`dnglab analyze --raw-checksum` reports. Levels are normalized and the three-stage
crop and orientation are applied. This is where the project stops being
speculative.

**Estimated effort: 15–25 hours.** Hours, not calendar — pace is the maintainer's to set.

## Why Now

The oracle contract is already verified (`docs/oracle-contract.md`), so this
stage can be built against an exact pass/fail from its first commit — no judgment,
no tolerance, no tuning loop. That makes it unusually good delegated work.

It depends on STAGE-001 only for geometry and levels, and everything after it
depends on the plane being right, so any ambiguity here is worth paying for now.

## Success Criteria

- MD5 of our full-frame u16 plane equals `dnglab analyze --raw-checksum` on every tier-B file
- The unpack asserts `width × height × 14 / 8 == StripByteCounts` and fails loudly if not
- The oracle goes **red** on an injected off-by-one in the bit unpacker
- Geometry is asserted numerically: 8424×5632 → ActiveArea → DefaultCrop 8368×5584 → Rotate 90 CW
- Peak memory for a 47 MP decode is measured and recorded, not assumed

## Scope

### In scope
- Locating the full-resolution SubIFD's strip and reading it
- Packed 14-bit → u16 unpack (zero-extended, NOT scaled to full 16-bit range)
- Bit-exact plane oracle harness
- BlackLevel subtraction and WhiteLevel normalization
- ActiveArea → DefaultCrop → Orientation

### Explicitly out of scope
- Tiled strip layouts — the Q2M is a single strip; tiles arrive with other cameras
- Compressed data of any kind (lossless JPEG is a later project)
- Opcodes (STAGE-003)

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Run `just frame-stage STAGE-002` to promote these outlines into real specs.

- [ ] (not yet written) [M] Strip location and sample unpack, with the StripByteCounts assertion. ⚠ **TWO PATHS, per DEC-008** — sub-byte samples (14-bit) are a MSB-first bit stream; byte-aligned samples (16-bit) are plain integers in the FILE's byte order. SPIKE-002 found the single-path version produced a byte-swapped plane on a 16-bit file. Keep the `max > WhiteLevel` assertion: it is what caught it. Both paths need their own fuzz coverage — one target exercising only 14-bit recreates the exact blind spot.
- [ ] (not yet written) [S] Bit-exact plane oracle against `dnglab analyze --raw-checksum`, plus a red-on-injected-fault test
- [ ] (not yet written) [M] Black/white level normalization, ActiveArea → DefaultCrop, and orientation
- [ ] (not yet written) [S] **Analytic levels/geometry oracle (DEC-004)** — assert normalization maps BlackLevel→0 and WhiteLevel→1 on tags READ FROM THE FILE, plus crop dimensions and orientation on both a rotated and an unrotated frame. ⚠ SPIKE-001 proved the plane checksum is structurally blind to a levels error and the develop oracle misses one up to +256 (50%). Without this spec, levels ship with NO oracle coverage.

**Count:** 0 shipped / 0 active / 4 pending

## Design Notes

**The comparison attaches to the UNCROPPED full frame.** `--raw-checksum`
hashes the 8424×5632 plane before ActiveArea and before DefaultCrop, in native
little-endian, with 14-bit values zero-extended. Decode → hash → compare one
string; crop afterwards. Full contract and the three wrong guesses that preceded
it are in `docs/oracle-contract.md`.

**What bit-exact does and does not prove.** It proves we match rawler. Because decompression is
deterministic and rawler is the de facto reference, that IS the goal at this layer — but say so
rather than letting "bit-exact" imply more than it does. The layer-0 packing arithmetic
(`w x h x 14 / 8 == StripByteCounts`) is the one check here that is independent of any other
implementation; keep it even though the checksum subsumes it in practice.

Memory is a live design constraint, not a footnote: 47.4 MP at f32 is ~190 MB per
plane before anything else exists.

**Stopping point B:** a correct linear plane, proven bit-exact against an
independent implementation.

## Dependencies

### Depends on
- STAGE-001 — geometry, levels and the SubIFD location
- External: `dnglab` (tool)

### Enables
- STAGE-003 — opcodes operate on this plane

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
