---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-046
  type: story                      # epic | story | task | bug | chore
  cycle: frame                     # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   two-component scan + per-component predictor state + column
                                   #   de-interleave; could prove L — confirm at design
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
  decisions: [DEC-037, DEC-035, DEC-016, DEC-018]   # two-component re-scope (DEC-037); clean-room/patent-clean (DEC-035); caller-owned buffer (DEC-016); u16 modular arithmetic (DEC-018)
  constraints: [no-panics-on-untrusted-input, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-037, SPEC-043]   # the single-component decoder this extends (037); the routing that feeds it (043, shipped)

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-043]             # the Compression-7 routing (merged) feeds strips to the decoder this extends

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-011's <capability>". Optional; null is acceptable.
value_link: "Extends the SOF-3 decoder to two-component (Nf=2) column-interleave so both held Compression-7 files decode to one monochrome u16 plane — the prerequisite for STAGE-011's bit-exact oracle (SPEC-044)."

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

# SPEC-046: Two-component Nf2 SOF-3 decode and column de-interleave

## Context

The re-plan spec (DEC-037). SPEC-043's build probed the real SOF3 headers and found
both held Compression-7 DNGs are **two-component** (`Nf=2`) column-interleave SOF-3 —
Leica M Typ 246 (P=12, X=2992, Nf=2) and Pentax K-3 III (P=14, X=3152, Nf=2), each X ≈
half the crop width — not the single-component the framing assumed. The shipped
STAGE-009 decoder is `Nf=1` only and returns `LjpegUnsupportedComponentCount { nf: 2 }`
for both, so neither held file decodes and SPEC-044's bit-exact oracle has no target.
This spec grows the decoder to two-component column-interleave so both held files decode
to one monochrome `u16` plane. It is the new second spec of `STAGE-011` (`PROJ-003`),
inserted between SPEC-043 (routing, shipped) and SPEC-044 (the oracle).

## Goal

Extend `ljpeg` to decode a two-component (`Nf=2`) column-interleave SOF-3 frame — two
per-component predictor streams — and de-interleave the two column components back into
one `u16` plane (width `2 × samples_per_line`), so both held Compression-7 DNGs decode,
proven by an analytic two-component encode→decode round-trip with a red-proof. The
single-component (`Nf=1`) path stays byte-for-byte unchanged.

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

- The bit-exact `dnglab --raw-checksum` corpus oracle on the two real files — **SPEC-044**.
- The provenance-ledger entry, determinism, and container→decoder hardening — **SPEC-045**.
- The Compression-7 recognition + routing — **SPEC-043** (shipped); this spec only
  extends what `decode_into` accepts.
- **Bayer / CFA colour multi-component** (3+ colour components in one frame) — still out
  (DEC-037); no held colour file, no oracle.
- More than two components, or a non-column interleave the held files do not use.

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
