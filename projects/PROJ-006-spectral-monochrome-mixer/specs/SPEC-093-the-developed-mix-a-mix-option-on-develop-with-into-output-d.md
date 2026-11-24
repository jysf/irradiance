---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-093
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
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-006
  stage: STAGE-031
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
depends_on: [SPEC-092]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-031's <capability>". Optional; null is acceptable.
value_link: "STAGE-031: the developed mono for export, with onyx's warp parity check intact"

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

# SPEC-093: The developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-093]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Third spec of STAGE-031 (PROJ-006). onyx's export path needs the mix as a
developed image matching `output_dimensions_with`, ideally as an option on
`develop_with_into`, so the mix goes through the same geometry and resampling the
mono develop uses and onyx's GPU-to-develop warp parity check (its DEC-062)
carries over unchanged. `DevelopOptions` is `#[non_exhaustive]`, so a new field is
additive.

## Goal

Add an opt-in mix option to `DevelopOptions` so `develop_with_into`,
`output_dimensions_with` and `scratch_len` develop a colour Bayer DNG to a mixed
monochrome image at the mono develop's geometry, byte-identical to today when the
option is absent.

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

**Pre-register every numeric threshold.** A tolerance, bound or score an AC
checks is written here at design, with its falsifier, before the build measures
anything. A measured number outside it is reported, never fixed by widening the
bound (signal `pre-register-the-tolerance`).

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)
- [ ] Criterion 3 (testable)

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

**Rule-clause map.** When this spec implements a `DEC-*` rule, map every clause
of it (each malformed class, each entry point, each numeric path) to a named
test above, and say which code path that test must reach. Name the path where the
clause changes the output, not just one that runs it: a clause tested only where it
is unobservable leaves a mutant the build's suite cannot see (signal
`every-rule-clause-maps-to-a-reachable-test`).

**Distinguishing fixture.** For each pre-registered mutant, and for each quantifier
(every / any), count (exactly one / many), ordering (lowest / first walked) or
alternative formula in the rule, name the fixture on which the right and the wrong
answer differ. A fixture where they agree (one value, a symmetric white balance, sets
that do not interleave, a loop that always hits its cap) reaches the clause and still
cannot fail (signal `a-designed-mutant-must-be-able-to-fail-on-its-fixture`).

**Moved assertions.** When this spec changes, removes or reverses a behaviour
(including a clause of an earlier `DEC-*`), grep the tests for every assertion that
encodes the old behaviour (pins, refusal tests, hand-written oracles, fixtures built
on the old shape) and list each one here as a pre-approved move, with what it becomes.
A short list sends the build round the gate once per missed test (signal
`a-changed-formula-lists-every-assertion-it-moves`).

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- New geometry or resampling: the mix reuses the existing framing, orientation and warp.
- Changing `develop_into` or `develop_with_into` output without the option, including CFA-as-mono (SPEC-066, DEC-061).
- A colour (RGB) developed output through `develop_with_into`.

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
