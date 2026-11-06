---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-043
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
  stage: STAGE-011
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-036, DEC-016, DEC-012]   # ljpeg visibility — the dev-only fuzzing feature may retire once this adds a real caller (DEC-036); caller-owned no-alloc buffer (DEC-016); malformed-tag / structural-gate rule for Compression recognition (DEC-012)
  constraints: [no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-037, SPEC-038, SPEC-028]   # the shipped SOF-3 decoder (037 completes it, 038 fuzzed it) + the multi-strip plane assembly (028) this routing reuses

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-011's <capability>". Optional; null is acceptable.
value_link: "Makes the shipped SOF-3 decoder reachable — recognises Compression 7 on the sensor IFD and routes strip/tile bytes through ljpeg::decode_into — the binding half of STAGE-011's deliverable."

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

# SPEC-043: Compression-7 recognition + strip-path routing

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-043]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

STAGE-009 shipped `ljpeg::decode_into` (compressed bytes → `u16` plane,
bit-exact and panic-free) but kept it `pub(crate)` with **no caller** — a DNG
whose sensor IFD carries `Compression == 7` (SOF-3 lossless JPEG) still returns
`Error::UnsupportedCompression` at the plane gate. This is the first spec of
`STAGE-011`'s backlog (`PROJ-003`): it makes the decoder reachable by recognising
`Compression == 7` and routing the strip/tile bytes to it where the packed-bits
unpack sits today, for both container byte orders. SPEC-044 then proves the result
bit-exact against `dnglab`; SPEC-045 is the capstone. Builds on the shipped
SPEC-037 (the decoder), SPEC-038 (its fuzz hardening), and SPEC-028 (the
multi-strip plane assembly this reuses).

## Goal

Recognise `Compression == 7` on the sensor IFD and route the strip/tile bytes
through `ljpeg::decode_into` (reusing SPEC-028's multi-strip assembly), for both
container byte orders, so a Compression-7 DNG decodes to a `u16` plane instead of
returning `UnsupportedCompression` — while the uncompressed / monochrome / Bayer
paths stay byte-for-byte unchanged and the binding path stays panic-free.

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

- The bit-exact corpus oracle vs `dnglab --raw-checksum` — **SPEC-044**.
- The provenance-ledger entry, the macOS/Linux determinism assertion, and the
  container→decoder hardening fuzz — **SPEC-045**.
- Multi-component / Bayer-in-LJ92 interleave, and OpcodeList2 GainMap — deferred
  (no held file; PROJ-003 roadmap).
- Any change to the SOF-3 decoder algorithm itself — that is STAGE-009 (shipped);
  this spec only feeds it real bytes.

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
