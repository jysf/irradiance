---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-007
  type: story                      # epic | story | task | bug | chore
  cycle: frame                    # frame | design | build | verify | ship
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

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-004]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "a DNG-legal file must not become unreadable because of one tag"

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

# SPEC-007: Unreadable tags in the extraction path, and DEC-012s contradiction

## Context

`SPEC-004` closed FU-11 for the **selection** path — `is_sensor_ifd` is now a
`SensorMatch` tri-state, so a malformed identifying tag on one IFD no longer aborts
the scan of the others. Its verify found the **extraction** path still has the same
gap, twice:

- **FU-16** — `sensor()` reads `Orientation` from `IFD0` with a bare `?`
  (`src/ifd.rs:1011`), so a malformed tag on a **non-sensor** IFD discards an
  already-located plane. Reproduced: `sensor_matches [1]`, then discarded.
- **FU-17** — a **DNG-legal `RATIONAL`** `DefaultCropSize`/`DefaultCropOrigin`/
  `BlackLevel` makes the **whole file unreadable**: `uints()` (`src/ifd.rs:788`)
  returns `UnexpectedFieldType` and `sensor()` propagates it. Reproduced. This is
  fatal to the file, not a missing field — a severity the build's framing
  understated.

Neither is a regression; both are identical on `main`.

⚠ **`DEC-012` must be amended first.** Its principle forbids exactly this, and its
*table* sanctions it. A spec designed against it today would inherit a decision
that blesses the behaviour this spec exists to fix. The contradiction is recorded
on the DEC itself.

## Goal

Decide what "what exists" means — **the plane**, or **every tag the plane's record
carries** — amend `DEC-012` to say so, and make the extraction path obey it.

A file that is legal per the DNG specification must not become unreadable because
one optional tag is malformed or uses a legal type we have not implemented.

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

1. `DEC-012` amended (or superseded) so its principle and its table agree, and the
   contradiction note on it is resolved rather than left standing.
2. A malformed `Orientation` on a non-sensor IFD **does not** discard a located
   plane; it costs that field.
3. A `RATIONAL` `DefaultCropSize`/`Origin`/`BlackLevel` **does not** make the file
   unreadable. Either widen `uints()` to handle `RATIONAL`, or make the tag
   optional-on-type-error — the decision from criterion 1 dictates which.
4. Hand-built fixtures for both, asserting the *new* outcome and the *unchanged*
   fatal cases, so the boundary is pinned in both directions.
5. **FU-20** while here: `NoSensorIfdCandidatesMalformed` can name IFDs that were
   never candidates (`src/ifd.rs:916`).
6. Ten gates green; fuzz covers the widened paths.

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

## Non-Goals

- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Executing opcodes — STAGE-003.
- Widening `uints()` to types no DNG tag we read can carry. `RATIONAL` is in scope
  because the DNG spec permits it for tags we already read; the signed types and
  `ASCII` are not, unless criterion 1 says otherwise.

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
