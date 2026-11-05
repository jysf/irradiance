---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-037
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
  id: PROJ-003
  stage: STAGE-009
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-016, DEC-018]    # caller-owned output buffer + u16 full-scale
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, test-before-implementation]
  related_specs: [SPEC-036, SPEC-038]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-036]             # consumes the entropy front-end's difference stream

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-009's <capability>". Optional; null is acceptable.
value_link: "Lossless reconstruction (predictors 1–7, initial prediction, point transform, restart reset) + the analytic encode→decode round-trip oracle — turns SPEC-036's difference stream into the exact caller-owned u16 plane, completing the STAGE-009 decoder's correctness half."

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

# SPEC-037: Lossless reconstruction (predictors, point transform, restart)

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-037]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

SPEC-036 decodes a SOF-3 scan down to per-sample (category → difference) values;
those differences are not samples until they are run through the lossless
predictor. This is the second of `STAGE-009`'s backlog and delivers
**reconstruction**: predictors 1–7, the initial prediction
`Px = 2^(P−Pt−1)`, the point transform `Pt`, and restart-interval resets,
writing the reconstructed samples into a caller-owned `u16` buffer
(DEC-016 / DEC-018). It also builds the stage's corpus-free independent oracle —
a tiny in-harness LJ92 encoder — so an encode→decode round-trip can assert
bit-exactness and be shown RED on a perturbed predictor, Huffman table, or point
transform (constraint `oracle-must-be-shown-red`). SPEC-036 + SPEC-037 together
are the complete single-component decoder; SPEC-038 then fuzzes it.

- Parent: `STAGE-009` (SOF-3 lossless-JPEG core decoder), backlog position 2 of 3
- Project: `PROJ-003` (compressed planes — lossless JPEG)
- Blocked on SPEC-036 (its difference stream is this spec's input)
- Standard: ITU-T T.81 (1992) Annex H — a provenance-ledger row lands here

## Goal

Reconstruct a single-component SOF-3 plane from SPEC-036's decoded difference
stream — applying predictors 1–7, the initial prediction, the point transform,
and restart resets — into a caller-owned `u16` sample buffer, and prove it with
an analytic encode→decode round-trip oracle that is bit-exact on honest input
and demonstrably RED on a perturbed predictor, Huffman table, or point
transform.

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

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- The marker/frame parse, JPEG bit reader, and DC Huffman decode — that is
  SPEC-036 (this spec consumes its output).
- The fuzz / truncation-hardening target — SPEC-038.
- Real-file verification against `dnglab --raw-checksum` and the DNG
  `Compression 7` container binding — STAGE-010.
- Multi-component interleave, lossy DCT, arithmetic coding, Nikon NEF-lossless,
  Pentax PEF.

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
