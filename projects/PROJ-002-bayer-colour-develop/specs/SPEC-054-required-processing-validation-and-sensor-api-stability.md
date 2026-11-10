---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-054
  type: story                      # epic | story | task | bug | chore
  cycle: frame  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  constraints: [no-panics-on-untrusted-input, provenance-recorded-per-algorithm, library-not-application]
  related_specs: [SPEC-039, SPEC-042]

depends_on: []                     # independent to start; gates SPEC-039 (group E) per DEC-045

claimed_by: null

value_link: "a DNG that requires an unsupported correction is told so, not handed a plausible-looking wrong image — pre-0.2.0 correctness"

cost:
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-054: Required-processing validation + Sensor API stability (group C)

> **OUTLINE — `cycle: frame`.** Stable ID for `depends_on`. Capture scope and
> dependencies only; design just-in-time. This is the design-heavy group of the
> wave — it needs a `DEC-*` before build and it settles the `Sensor`
> construction / API-stability question.

## Context

Pre-0.2.0 code-review finding **#3 (P1)**, triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted by DEC-045.
`Sensor.opcode_lists: [bool; 3]` (`src/ifd.rs:685-687,1708-1714`) is
**presence-only** — only lists 1 and 3 keep their bytes, `OpcodeList2`'s payload
is dropped, and the colour develop path documents that opcode lists "are not
applied" (`src/color.rs:255`). The mono helpers resolve a list with `find_map`
(first match only). So a DNG that *requires* a correction the pipeline does not
apply gets a plausible image and `Ok(())` — the
`parser-accepts-what-pipeline-cannot-honor` signal (now N=2 with the `Hi/Vi`
item). Fixing it means keeping the `OpcodeList2` bytes and adding a *field*, which
collides with the semver-urgent `#[non_exhaustive]` question the staff review
(`docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` §4) raised: `Sensor` has
public fields, so every added field is a breaking change unless the type is made
non-exhaustive first. That is why this group owns both the opcode contract and the
`Sensor` construction decision.

## Goal

Keep the `OpcodeList2` bytes and preserve list order; when a DNG carries an
opcode list that *requires* an operation the pipeline cannot apply (on the mono
path **and** the colour path), return a typed error instead of a silent `Ok`.
Settle the supported-opcode contract and the `Sensor` construction model in a
`DEC-*`, and make `Sensor` non-exhaustive as the prerequisite of adding the new
field (open maintainer decision #3 in the triage). The remaining public structs'
non-exhaustive sweep rides in the release spec (SPEC-042), not here.

## Non-Goals

- Not *implementing* any new opcode — this is about refusing, not honouring,
  unsupported required processing.
- No change to a DNG that needs no unsupported op (today's corpus must still
  decode and develop unchanged).
- The non-exhaustive sweep of the other public structs (`Cfa`, `WarpRect`,
  `Opcode`, …) is SPEC-042's; this spec only non-exhausts `Sensor` because it
  adds a `Sensor` field.

## Notes for the Implementer

- A `DEC-*` is required **before** build: it fixes (a) the supported-opcode
  contract — which opcode IDs the pipeline honours vs rejects — and (b) whether
  consumers may construct `Sensor` directly, which drives the non-exhaustive and
  builder-vs-fields shape.
- Pixel-output / error-surface change ⇒ a red-proof is mandatory
  (`oracle-must-be-shown-red`): a fixture requiring an unsupported op must go red
  (silent `Ok`) before the fix makes it a typed error.
- Provenance ledger row for any new validation logic.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
4. **Where was the worst defect caught?** — `design` | `build` | `verify` | `ship` | `escaped` | `none`. — <one word>
5. **What can a user do now that they couldn't before?** — <answer | none>
