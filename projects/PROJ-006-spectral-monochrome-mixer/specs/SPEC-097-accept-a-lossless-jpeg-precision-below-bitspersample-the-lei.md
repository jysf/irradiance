---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-097
  type: bug                        # epic | story | task | bug | chore
  cycle: frame                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-006
  stage: STAGE-034
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-077]             # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-086, SPEC-091, SPEC-092]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-034's <capability>". Optional; null is acceptable.
value_link: "STAGE-034: a Leica M10 colour DNG (and any body that stores a narrower lossless-JPEG precision in a wider BitsPerSample) unpacks, so the colour path and the mixer can run on it"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-097: Accept a lossless-JPEG precision below BitsPerSample, the Leica M10 file

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-097]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

First spec of STAGE-034 (PROJ-006). On 2026-11-26 the colour Leica M10 file from
raw.pixls.us (`LEICA-M10/f5381888.dng`, 5984×3992, `Compression = 7`, `BitsPerSample = 16`,
`WhiteLevel = 15000`, `BlackLevel = 0`, no opcode lists) fails at unpack with
`Error::LjpegPrecisionMismatch { precision: 14, bits_per_sample: 16 }`, before any colour
or mixer code runs. Adobe's `dng_validate` (local SDK build) validates the same file, with
only an unrelated `MaxApertureValue`/`LensInfo` metadata warning.

The refusal comes from `check_precision` in `src/plane.rs` (`SPEC-086`, `DEC-077` §3),
which requires the SOF-3 frame precision `P` to **equal** `BitsPerSample`. Its stated
reason is the case `P > BitsPerSample`: a P = 16 frame tagged 12-bit would clip at 4095
with `Ok`, because white defaults from `BitsPerSample` when `WhiteLevel` is absent. The
M10 is the other direction, `P < BitsPerSample`: every decoded sample is below `2^14`, so
it fits the 16-bit container and nothing clips, and the file carries an explicit
`WhiteLevel` (15000). The **Leica M11** does the same: `LEICA-M11-PB/leica_m11_01.dng` (60 MP,
camera-original, local corpus only) gives the identical refusal, has no opcode lists, and
`dng_validate` accepts it. The M11 is onyx's first-priority M body.

onyx's bodies, in order: Q3 and M11, then Q2 and M10, then SL2/SL3 (its DEC-095). A
precision refusal would stop every PROJ-006 entry point (SPEC-091's planes, SPEC-092's
mixer, SPEC-093's developed mix) on any such file.

## Goal

A `Compression = 7` strip or tile whose SOF-3 precision is **below** `BitsPerSample`
unpacks, with its samples unchanged, so the Leica M10 and M11 files reach the colour path
and the mixer; `P > BitsPerSample` stays refused exactly as today.

## Inputs

- **Files to read:** `src/plane.rs` (`check_precision` and its callers on every
  `Compression = 7` path: `unpack_into`, `unpack_unit_into`, `PlaneDecoder`, tiles);
  `src/lib.rs` (`Error::LjpegPrecisionMismatch`); `tests/ljpeg_precision_binding.rs`;
  `decisions/DEC-077-*.md` §3; the shipped SPEC-086.
- **External:** DNG 1.7.0.0 on `BitsPerSample` for lossless JPEG; the Adobe DNG SDK's
  lossless-JPEG read path and `dng_validate`; how rawspeed, LibRaw and dnglab treat the
  same case.
- **Corpus:** `LEICA-M10/f5381888.dng` (raw.pixls.us, CC0) and `LEICA-M11-PB/leica_m11_01.dng`
  (Photography Blog, no stated licence, local only), both added 2026-11-26. Record them in
  the manifest at design.

## Questions for design

- What does DNG 1.7.0.0 say, and what does the SDK do, when `P < BitsPerSample`? If
  `WhiteLevel` is absent, does white default from `BitsPerSample` (2^16 − 1) or from `P`?
  The design follows the specification and the SDK and records which.
- Is `P < BitsPerSample` accepted for any gap, or only where every sample fits? (A P = 8
  frame in a 16-bit tag is legal but unusual.)
- Does this amend `DEC-077` §3 or need its own decision?
- Does any output change for a file that unpacks today? It should not, since only files
  that are refused today change, so `DEVELOP_VERSION` stays as it is. Confirm.
- Does the M10 file then pass the colour develop, `srgb_linear_plane_into` and
  `mono_mix_plane_into`, or does it hit the next refusal? Report what it hits.

## Non-Goals

- `P > BitsPerSample`: still refused (`DEC-077` §3's reason stands).
- Mandatory opcodes on the colour path (the Q2's `FixBadPixelsConstant` and
  `WarpRectilinear`): a separate STAGE-034 spec.
- Other Leica quirks found later: each gets its own spec once a file shows it.
- Q3 or M11 files: being sourced separately.

