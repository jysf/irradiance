---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-052
  type: story                      # epic | story | task | bug | chore
  cycle: frame  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
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
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-045]             # [DEC-NNN, DEC-MMM]
  constraints: [no-panics-on-untrusted-input, library-not-application]
  related_specs: [SPEC-039]        # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # independent — A/B/D fan out in parallel (DEC-045)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-010's <capability>". Optional; null is acceptable.
value_link: "makes the container parser safe to point at hostile metadata — the pre-0.2.0 'safe to depend on' thesis"

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

# SPEC-052: Parser resource guard — aggregate entry/byte budget (group A)

> **OUTLINE — `cycle: frame`.** This spec exists so its ID is stable and
> siblings can declare `depends_on: [SPEC-052]`. Capture **scope** (Context /
> Goal / Non-Goals) and **dependencies** only — the *approach* is designed
> just-in-time when this moves to `design`. Do not pre-design it here.

## Context

Pre-0.2.0 code-review finding **#1 (P1)**, triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted into the 0.2.0
wave by DEC-045. `walk_chain` (`src/ifd.rs:905`) de-duplicates only *identical*
IFD offsets (`visited.contains`) and caps IFD *count* (`MAX_IFDS = 512`,
`src/ifd.rs:69`), but nothing caps the *aggregate* number of entries or bytes
parsed. `read_ifd` (`src/ifd.rs:978`) does `Vec::with_capacity(count)` of up to
65,535 entries per IFD, independently for each distinct offset, so a chain of
distinct offsets reaches a ~33.5M-entry ceiling (512 × 65,535). The reviewer's
repro: a ~770 KB TIFF amplifies into ~6.3M parsed entries (~72 MiB) before any
pixel budget applies — a memory-amplification DoS at the container boundary,
catchable by nothing the pipeline currently has. This is the IFD-walk analogue of
the plane-level validate-before-allocate work (SPEC-039/group E), which is why
group E's unified preflight depends on this landing first.

## Goal

Add an aggregate resource budget to the IFD walk — a cap on the total entries
(and/or bytes) parsed across the whole chain, not just per-IFD — and reject an
entry-table layout that forces amplification (e.g. overlapping/aliased entry
tables), returning a typed error with no allocation proportional to the forged
size. A ~770 KB file must no longer be able to drive a ~72 MiB parse. The exact
budget number and whether it is entries, bytes, or both is a `design`-cycle
decision and gets its own `DEC-*`.

## Non-Goals

- No change to how any **valid** file parses — every real corpus file (tier-A and
  tier-B) must pass the guard untouched; the budget is set above any legitimate
  camera DNG/TIFF.
- Not the *pixel* allocation budget — that is the `plane::validate` work
  (SPEC-039). This guard is about the metadata-parse allocation only.
- No new container features; no change to `MAX_IFDS` semantics or the existing
  cycle (`visited`) check — the regression must use **distinct** offsets so it
  isolates the new aggregate guard from the identical-offset cycle check.

## Notes for the Implementer

- Design-cycle work: run the ~72 MiB repro **first** as the red-proof
  (`oracle-must-be-shown-red`), then add the guard so it goes green while the
  repro's allocation stays bounded.
- Emits a `DEC-*` for the budget (the number and its unit) at design.
- Extend `fuzz/fuzz_targets/ifd.rs` so the guard runs on every parsed input.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary:
   `design` | `build` | `verify` | `ship` | `escaped` | `none`.
   — <one word>

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after. Write `none` if this spec has no user-visible outcome.
   — <answer | none>
