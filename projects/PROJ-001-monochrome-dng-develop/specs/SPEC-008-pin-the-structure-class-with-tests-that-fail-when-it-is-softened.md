---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-008
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
  decisions: [DEC-012]                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: []                # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-007]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "a decision that only the prose enforces is not enforced"

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

# SPEC-008: Pin the Structure class with tests that fail when it is softened

## Context

`SPEC-007` implemented `DEC-012`'s Structure / Interpretation split. Its verify
found the **Structure half is almost entirely unguarded by tests** — measured, and
reproduced independently by the orchestrator:

| structural tag softened to tolerant | full 58-test suite |
|---|---|
| `RowsPerStrip` | **RED** |
| `Compression` | all green |
| `StripOffsets` | all green |
| `StripByteCounts` | all green |
| `BitsPerSample` | all green |

`Compression` is the dangerous one: softened it defaults to `1`,
`require_uncompressed()` passes, and **STAGE-002 reads JPEG bytes as raw samples**
— a wrong image from a file that parsed cleanly.

The orchestrator had mutated `RowsPerStrip` alone and reported "the boundary test
has teeth." One point on a boundary is not a boundary
(`measurement-over-generalised`, now at N=3).

A second, related gap — **`SPEC-007/FU-4`** — is that widening `uints()` for
`RATIONAL` was **global, not per-tag**, so it loosened the *walk*: `SubIFDs` (330)
as `RATIONAL 400/2` was `Err` on `main` and is now accepted. `DEC-012` names
`SubIFDs` as **structural**.

Both are the same defect: **a class the decision defines and the tests do not
enforce.**

## Goal

Make `DEC-012`'s Structure class enforced rather than merely stated: softening any
structural tag must fail the suite, and `uints()`'s type widening must be per-tag
rather than global.

Also correct `malformed_tags` where it currently says something untrue.

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

1. **Every structural tag has a test that fails when it is softened.** Minimum
   set, all four measured green today: `Compression`, `StripOffsets`,
   `StripByteCounts`, `BitsPerSample` (plus `RowsPerStrip`, already covered).
   ⚠ `SamplesPerPixel` and `Photometric` in `sensor()` are **equivalent mutants** —
   re-reads of tags `is_sensor_ifd` already read successfully — so they are not
   part of this set. Do not manufacture a test that only appears to cover them.
2. **`uints()`'s `RATIONAL` acceptance is per-tag, not global** (`SPEC-007/FU-4`).
   A structural tag encoded as `RATIONAL` must be rejected as it was on `main`;
   an interpretation tag may accept it. Either way it is **written down**.
3. **`SPEC-007/FU-1`:** when the plane is `IFD0`, a malformed `Orientation` is
   recorded **twice** — measured `malformed_tags = [274, 274]` on the Pentax
   `.PEF`, a real corpus shape.
4. **`SPEC-007/FU-2`:** a *well-formed* `Orientation` on the sensor IFD is recorded
   as malformed — `orientation = Some(6)` **and** `malformed_tags = [274]`.
5. **`SPEC-007/FU-5`:** every well-formed RATIONAL fixture uses denominator `1`, so
   a mutant that pushes the numerator and ignores the quotient passes all 58 tests.
   Pin the division with a denominator ≠ 1.
6. Ten gates green.

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

## Non-Goals

- Re-opening `DEC-012`'s line. This spec **enforces** it; it does not redraw it.
- Levels arithmetic, cropping, orientation transforms — STAGE-002, `DEC-008`.
- Adding structural tags to the class, or removing any.

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
