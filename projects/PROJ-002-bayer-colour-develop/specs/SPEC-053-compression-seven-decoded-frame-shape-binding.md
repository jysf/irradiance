---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-053
  type: story                      # epic | story | task | bug | chore
  cycle: frame  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
  complexity_actual: null
  verify_verdict: null

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5
  to_agent: null
  created_at: null

references:
  decisions: [DEC-045]
  constraints: [no-panics-on-untrusted-input, library-not-application]
  related_specs: [SPEC-039]

depends_on: []                     # independent — A/B/D fan out in parallel (DEC-045)

claimed_by: null

value_link: "a wrong-shape compressed DNG is rejected, not silently decoded to a mis-strided image — pre-0.2.0 correctness"

cost:
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-053: Compression-7 decoded-frame shape binding (group B)

> **OUTLINE — `cycle: frame`.** Stable ID for `depends_on`. Capture scope and
> dependencies only; design just-in-time.

## Context

Pre-0.2.0 code-review finding **#2 (P1)**, triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted by DEC-045.
The Compression-7 (lossless-JPEG) strip path (`src/plane.rs:577-582`) sizes a
row band from `width × rows`, uses that to "pin the dimensions", and then
**drops** the decoded `Frame`. `decode_into` only checks `out.len() == X · Y`
(`LjpegPlaneWrongLength`) — an **area** check, not a **shape** check. So a frame
whose decoded dimensions are an equal-area transpose of the strip (e.g. 4×3 vs
3×4) returns `Ok(())` with a wrong row stride and a silently garbled image. For
the two-component (`Nf=2`) column-interleave case the effective width is
`X · Nf` after de-interleave, so the comparison must use that. Sits next to the
earlier `unexercised-sof3-paths-garble-silently` signal (`Hi/Vi != 1`, restart
intervals) from the PROJ-003 close triage.

## Goal

Bind the **decoded `Frame`** shape (width — times `Nf` where de-interleaved — and
height) to the strip's declared dimensions and return a typed error on a mismatch,
including the equal-area wrong-shape case that currently returns `Ok(())`. Cover
each component count (`Nf = 1` and `Nf = 2`) with a test. May fold in the
`Hi/Vi != 1` rejection from the prior triage if it is cheap in the same pass.

## Non-Goals

- No new codec support and no change to a **correctly-shaped** decode's output
  (the shipped corpus decodes bit-exact today and must still).
- Not the restart-interval (`DRI`) hardening unless it falls out trivially — that
  can stay a separate hardening item.

## Notes for the Implementer

- Design-cycle work: build the equal-area wrong-shape repro **first** as the
  red-proof, then add the shape binding.
- Confirm against both held Compression-7 files (local tier-B run) that the
  binding rejects nothing real.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
4. **Where was the worst defect caught?** — `design` | `build` | `verify` | `ship` | `escaped` | `none`. — <one word>
5. **What can a user do now that they couldn't before?** — <answer | none>
