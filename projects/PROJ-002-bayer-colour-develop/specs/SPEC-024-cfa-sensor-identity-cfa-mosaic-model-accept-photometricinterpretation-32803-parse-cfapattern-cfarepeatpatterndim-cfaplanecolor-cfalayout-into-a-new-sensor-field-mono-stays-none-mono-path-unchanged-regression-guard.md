---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-024
  type: story                      # epic | story | task | bug | chore
  cycle: frame                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-002
  stage: STAGE-006
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-025, SPEC-026, SPEC-027]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: "Delivers STAGE-006's 'a Bayer DNG is accepted by the sensor gate' — the foundation SPEC-025/026/027 build on."

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

# SPEC-024: CFA sensor identity + `Cfa` mosaic model: accept `PhotometricInterpretation == 32803`; parse `CFAPattern`/`CFARepeatPatternDim`/`CFAPlaneColor`/`CFALayout` into a new `Sensor` field; mono stays `None`; mono path unchanged (regression guard)

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-024]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

The sensor gate accepts only a monochrome `LinearRaw` plane: `is_sensor_ifd`
(`src/ifd.rs:1035`) requires `PhotometricInterpretation == 34892`, and nothing in
the codebase parses a CFA. A Bayer DNG is `PhotometricInterpretation == 32803
(CFA)` carrying a `CFAPattern`, so `sensor()` rejects it outright today. This is
the foundation spec of `STAGE-006` (PROJ-002): it opens the gate to CFA and gives
`Sensor` a structured mosaic model, on top of PROJ-001's IFD/tag reader. Nothing
reads a Bayer file until this ships — SPEC-025/026/027 all depend on it. Verified
against the two corpus Bayer DNGs (`CANON-EOS-5D-MARK-III/5G4A9394.DNG`,
`NIKON-D300S/_DSC4615.DNG`), both RGGB 2×2, 16-bit.

## Goal

Accept a Bayer-CFA DNG at the sensor gate — `PhotometricInterpretation == 32803`,
with `SamplesPerPixel == 1` unchanged (a mosaic is one sample per pixel; colour is
encoded by position) — and expose a structured CFA model on `Sensor` (e.g.
`color_filter: Option<Cfa>` carrying `CFAPattern` / `CFARepeatPatternDim` /
`CFAPlaneColor` / `CFALayout`), `None` for the monochrome Leica. The monochrome
path stays byte-for-byte unchanged.

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

Testable outcomes. Each must map to at least one test. Cover happy
path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)
- [ ] Criterion 3 (testable)

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

## Non-Goals

- Reading the mosaic bytes or the bit-exact plane oracle — that is SPEC-025
- Parsing the colour matrices / white balance / per-channel levels — SPEC-026
- Any demosaic, colour transform, or level application — later stages
- X-Trans / non-Bayer CFA, and native vendor containers
- Choosing the one-`Sensor`-type-with-optional-`Cfa` vs a type split as a DEC —
  design-cycle call; promote to a DEC only if it proves weighty (stage Design Notes)

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities. Keep short — the full
context graph lives in the handoff file.

---

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
