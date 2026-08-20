---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-006
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
depends_on: [SPEC-001]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-001's <capability>". Optional; null is acceptable.
value_link: "makes the panic-free constraint true rather than nearly-true"

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

# SPEC-006: Close the allow-attribute bypass in the panic-free gate

## Context

SPEC-001 shipped a panic-free lint policy enforced by a red-proof that is itself
proven red (`DEC-009`). Its verify round 3 found — and the orchestrator
independently reproduced — that **the policy is exited by a single attribute**:

```rust
#[allow(clippy::panic, clippy::expect_used)]
pub fn boom(v: &[u8]) -> u8 { if v.is_empty() { panic!("empty") } *v.first().expect("x") }
```

`BUILD 0 · CLIPPY 0 · FMT 0 · TEST 0 · MSRV 0 · DENY 0 · REDPROOF 0` — **seven
green gates, two panics on the public API, no module involved.**

`DEC-009`'s red-proof **structurally cannot** close this: it mutates the crate
root and asserts the root's `#![deny(...)]` rejects the injection. No `#![deny]`
mutation test can observe an `#[allow]` beneath it. That is why this is a separate
spec rather than another round on that script — a fourth iteration of the same
mechanism was explicitly rejected in `DEC-009`.

Split out of SPEC-002 at SPEC-001's ship: the two share **no files** (that spec
touches `tests/**`; this one touches `scripts/`, `.github/workflows/` and
`guidance/`), and this hole is live *today* with no module, so it never depended
on SPEC-002's work.

## Goal

Make `no-panics-on-untrusted-input` mechanically true rather than nearly-true: an
`#[allow]` of any policy lint, anywhere outside the sanctioned exceptions
(`#[cfg(test)]` modules and `src/bin/`), must fail CI.

Then correct `guidance/constraints.yaml:33`, whose `enforcement:` field currently
reads as a stronger guarantee than holds (verify round 3, F-4).

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

1. An `#[allow]` (or `#![allow]`) of any of the five policy lints outside
   `#[cfg(test)]` and `src/bin/` **fails CI**.
2. The sanctioned exceptions still pass — test modules and `src/bin/irr.rs` keep
   their existing allows without special-casing each one by hand.
3. **The gate is shown RED**, per `oracle-must-be-shown-red`: adding a violating
   `#[allow]` turns it red, and that proof ships with it.
4. **The gate is shown GREEN on the honest tree** — a negative control, the lesson
   `DEC-009` paid three rounds to learn. Without it, "fails on X" cannot be
   distinguished from "fails on everything".
5. `guidance/constraints.yaml:33`'s `enforcement:` states what is now actually
   enforced — no more, no less.
6. All existing gates stay green.

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

- ...

## Notes for the Implementer

⚠ **If you reach for a text search — and you probably will — heed the
`attribute-text-inside-doc-comments` lesson signal, now at N=5 on SPEC-001
alone.** Every one of those five produced a wrong *answer* rather than an error:
two false negatives, one false green that shipped a panic past seven gates.

The rules that came out of it: **anchor at column 0**, exclude `//`, `//!` and
`/* */`, and **assert the match count** rather than taking the first hit.
`src/lib.rs` contains attribute text inside its own module documentation — that is
not an edge case here, it is the normal state of this file.

Consider whether a text search is the right tool at all. Alternatives worth a
moment: `cargo clippy` with the lints forced at the command line (which overrides
inner attributes but **not** `#[allow]` on an item), or a small AST pass. If a
text gate is chosen, its own red-proof (criterion 3) is what keeps it honest.

**Scope discipline:** this spec closes one hole and corrects one sentence. It is
not a licence to redesign `DEC-009`'s red-proof, which is sound for what it
pins.

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
