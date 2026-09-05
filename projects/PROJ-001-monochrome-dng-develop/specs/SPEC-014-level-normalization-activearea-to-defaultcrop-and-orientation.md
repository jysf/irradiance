---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-014
  type: story                      # epic | story | task | bug | chore
  cycle: design                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-002
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # ⚠ DISPATCH HINT — the BUILD hint is 0 for 7. Correct it.
  created_at: 2026-09-05

references:
  decisions: [DEC-002, DEC-004, DEC-016]                    # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, library-not-application, oracle-must-be-shown-red]                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-012, SPEC-013, SPEC-015]                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-002's <capability>". Optional; null is acceptable.
value_link: null

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
  tokens_estimate: 26000000
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-05
      notes: "main-loop, not separately metered (AGENTS.md §4). Design probe measured the geometry and levels of all four decodable files and found the finding that shapes the spec: ON EVERY DECODABLE FILE ActiveArea's origin is (0,0) or the tag is absent. The only file with a non-zero origin (K3III.DNG, top 34 left 26) is Compression 7 and undecodable — so 'DefaultCropOrigin is relative to ActiveArea' and 'relative to the raw plane' give IDENTICAL output on 100% of files this spec can run on, and an implementation ignoring the origin passes every corpus test. SPIKE-001's 'always 14' shape, with SPIKE-002 as the precedent for the cost. AC4's hand-built fixture is therefore the only thing that can observe the distinction. Also measured: both real files contain samples BELOW BlackLevel (min 2 and 108) and both reach WhiteLevel EXACTLY, so AC2's out-of-range handling fires on the first file rather than being hypothetical. And cited the independent evidence for the relative reading — dnglab prints cropArea.p sensor-absolute, (26,34)+(28,24)=(54,58) on K3III.DNG, while exiftool prints the file's own 28 24."

  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-014: Level normalization, ActiveArea to DefaultCrop, and orientation

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-014]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

`SPEC-012` produces a **correct, uncropped, un-normalised** `u16` plane and
`SPEC-013` asserts it bit-for-bit. This spec turns that into an image: subtract
black, normalize to white, apply the three-stage crop, and apply orientation.

⚠ **`SPEC-013`'s oracle attaches BEFORE all of this** — `--raw-checksum` is the
uncropped, un-normalised plane by contract. So **nothing in this spec is covered
by the existing oracle**, and `DEC-004` already settled why a comparison oracle
never will be: `SPIKE-001` measured that the plane checksum is *structurally
blind* to a levels error, and the develop oracle misses one up to **+256 (50 %)**.
`SPEC-015` is the analytic oracle that covers this spec. **Until it exists, this
code has no oracle at all** — which is the single most important thing to know
while building it.

## Goal

Normalize levels and apply geometry, producing the image a consumer would
actually display — with the arithmetic asserted numerically, since no
comparison oracle can see it.

## Inputs

- `src/plane.rs` (`unpack_into`, `DEC-016`'s caller-owned-buffer shape),
  `src/ifd.rs` (`Sensor`: `black_level`, `white_level`, `active_area`,
  `default_crop_origin`, `default_crop_size`, `orientation`)
- `DEC-004` — **levels are verified analytically, never by comparison**
- `docs/oracle-contract.md`, `docs/measured-q2m-dng.md`

## Outputs

- `src/` gains the normalize + geometry path
- **A `DEC-*` for the output representation** (see below) — required either way
- A provenance row if any new arithmetic warrants one; the transforms are from
  the DNG spec, so class 1

## Acceptance Criteria

- [ ] **AC1 — levels normalize analytically.** `BlackLevel → 0`, `WhiteLevel →
      full scale`, on values **read from the file**, not constants. Q2M is
      `512 → 0`, `16383 → max`; M Monochrom is `220 → 0`, `16383 → max`. Assert
      both endpoints and at least one interior point per file.
- [ ] **AC2 — values below `BlackLevel` and above `WhiteLevel` are handled
      explicitly and the choice is written down.** Measured: the Q2M plane's
      `min` is **2**, far below `BlackLevel 512`, and its `max` is **16383 ==
      WhiteLevel** exactly. So both edges are live on real data on the first
      file. Clamp or saturate — but decide it, test it, and record it.
- [ ] **AC3 — the three-stage crop, asserted numerically.**
      `8424×5632 → ActiveArea 8392×5632 → DefaultCrop 8368×5584`, and
      `5216×3472 → (no ActiveArea) → 5212×3468`.
- [ ] **AC4 — `DefaultCropOrigin` is applied RELATIVE TO `ActiveArea`, and a
      hand-built fixture proves it.** ⚠ **No decodable corpus file can tell the
      two readings apart** — see below. A tier-A fixture with a **non-zero
      ActiveArea origin** is the only thing that can, and without it this spec
      ships an untestable assumption.
- [ ] **AC5 — orientation is applied, and the per-frame case is covered.**
      `L1026016.DNG` reads `Orientation 6` where its two siblings read `1`;
      output dimensions must swap for 6. ⚠ `Orientation` is **per-frame, not a
      camera constant** — the fact that produced the `unrun-docs-carry-errors`
      signal. Every geometry test uses **both** a rotated and an unrotated frame.
- [ ] **AC6 — panic-free.** `DefaultCropSize` larger than `ActiveArea`, crop
      origin outside the plane, zero dimensions, absent tags, an orientation
      value outside 1–8. Typed errors, and the fuzz target reaches them.
- [ ] **AC7 — memory is measured, not assumed.** `SPEC-012` measured 182 MB peak
      for a decode; state what this adds and whether the transform is in-place.
- [ ] **AC8 — eleven gates + `just lint-ci`**, CI **observed** green.

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum across all.

- `black_and_white_levels_map_to_the_endpoints` — AC1, tier B
- `values_outside_the_level_range_are_handled_as_decided` — AC2, tier A + B
- `the_three_stage_crop_produces_the_measured_dimensions` — AC3, tier B
- `crop_origin_is_relative_to_active_area` — **AC4, tier A, hand-built**
- `orientation_six_swaps_the_output_dimensions` — AC5, tier B (`L1026016.DNG`)
- `an_unrotated_sibling_keeps_its_dimensions` — AC5's pair (`L1021223.DNG`)
- `hostile_geometry_does_not_panic` — AC6, tier A

## Non-Goals

- **The analytic oracle** — `SPEC-015`. This spec asserts its own arithmetic;
  `SPEC-015` is what makes that assertion independent.
- **Demosaic, colour, tone** — monochrome only, and PROJ-002/STAGE-003.
- **Changing `SPEC-012`'s output.** `SPEC-013`'s oracle attaches to the
  uncropped, un-normalised plane and must keep passing untouched.

## Implementation Context

> **Measured 2026-09-05** on all four decodable corpus files.

### The geometry, per file

| file | raw | ActiveArea | crop origin | crop size | Orientation |
|---|---|---|---|---|---|
| `L1021223.DNG` | 8424×5632 | `(0,0,5632,8392)` → 8392×5632 | `(12,24)` | 8368×5584 | **1** |
| `L1026016.DNG` | 8424×5632 | same | `(12,24)` | 8368×5584 | **6** |
| `L1026192.DNG` | 8424×5632 | same | `(12,24)` | 8368×5584 | **1** |
| `L1000622.DNG` | 5216×3472 | **absent** | `(2,2)` | 5212×3468 | **1** |

Both crops fit: `12+8368 = 8380 ≤ 8392` and `24+5584 = 5608 ≤ 5632`;
`2+5212 = 5214 ≤ 5216` and `2+3468 = 3470 ≤ 3472`.

### ⚠⚠ The blind spot — read this before writing `AC4`

**On every decodable corpus file, `ActiveArea`'s origin is `(0,0)` or the tag is
absent.** The *only* file with a non-zero origin is `K3III.DNG` —
`top 34, left 26` — and it is **`Compression 7`, therefore undecodable**.

So *"`DefaultCropOrigin` is relative to `ActiveArea`"* and *"relative to the raw
plane"* produce **identical output on 100 % of the files this spec can run on.**
An implementation that ignores the `ActiveArea` origin entirely will pass every
corpus test.

This is `SPIKE-001`'s shape exactly — *"the parameter was always 14"* — and
`SPIKE-002` is the precedent for the cost: it took a **different camera body** to
reveal the two-path bug, and the plane came out byte-swapped in a way that
decoded, sized and layer-0-checked correctly.

⚠ **`AC4`'s hand-built fixture is not optional. It is the only thing in this spec
that can observe the distinction.**

**Independent evidence the relative reading is correct:** `dnglab` reports
`cropArea.p` as **sensor-absolute** — measured at `SPEC-005`'s design probe, on
`K3III.DNG`: `(26,34) + (28,24) = (54,58)`, exactly what dnglab prints, while
`exiftool` reports the file's own `28 24`. Two tools, two conventions, and the
arithmetic between them confirms which one DNG means.

### Levels, and why both edges are live

| file | BlackLevel | WhiteLevel | measured plane min | measured plane max |
|---|---|---|---|---|
| `L1021223.DNG` | 512 | 16383 | **2** | **16383** |
| `L1000622.DNG` | 220 | 16383 | 108 | 16383 |

**Both files contain samples below `BlackLevel`** (2 and 108) and **both reach
`WhiteLevel` exactly**. So `AC2` is not a hypothetical edge case — it fires on the
first file, and `max == WhiteLevel` is why `SPEC-012`'s `>` versus `>=` mattered.

### ⚠ The decision this spec must record

What is the normalized output? The orchestrator's read, **offered as input, not
as the answer**:

- **`u16` rescaled in place / into a caller buffer**, consistent with `DEC-016`'s
  no-allocation shape. 14-bit data into 16 bits has headroom, memory stays flat,
  and `DEC-002` is still `proposed` so committing to a wider type is premature.
- **`f32` in `[0,1]`** is what a develop pipeline eventually wants and what
  `SPEC-015` will assert against — but it is **190 MB** for a 47 MP frame, on top
  of `SPEC-012`'s already-measured 182 MB peak.

**Write the `DEC` either way**, including if you disagree. `SPEC-015` asserts
`BlackLevel → 0` and `WhiteLevel → 1`, so whichever representation you choose
must make that assertion expressible.

### Traps

- ⚠ **This spec has no oracle.** `SPEC-013`'s attaches before it and `DEC-004`
  says no comparison oracle ever will. Your tests are the only check until
  `SPEC-015`.
- `Orientation` is **per-frame**. `L1026016.DNG` is the file that proves it.
- `just lint-ci`, not `just lint`, and **read CI**.
- A tier-B test passes whether or not the corpus is present; only `just test`
  names what is missing.

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
