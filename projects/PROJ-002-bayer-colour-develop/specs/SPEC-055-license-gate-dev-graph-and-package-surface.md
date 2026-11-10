---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-055
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
  constraints: [no-copyleft-dependencies, oracle-must-be-shown-red, library-not-application]
  related_specs: [SPEC-042]

depends_on: []                     # independent — A/B/D fan out in parallel (DEC-045)

claimed_by: null

value_link: "the permissive-licence claim is actually enforced, and the published crate ships no internal tool — pre-0.2.0 release hygiene"

cost:
  tokens_estimate: null
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-055: Licence gate over the dev-graph + package surface (group D)

> **OUTLINE — `cycle: frame`.** Stable ID for `depends_on`. Capture scope and
> dependencies only; design just-in-time.

## Context

Pre-0.2.0 code-review findings **#4 (P1, reproduced locally)** and **#6 (P2)**,
triaged in `docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`; adopted by
DEC-045.

- **#4** — `cargo deny check licenses` passes even with a GPL **dev-dependency**:
  `cargo deny list` on the real root prints only `irradiance@0.1.1`
  (Apache-2.0 / MIT) and omits the dev-deps `toml` and `ssimulacra2`. The CI job
  (`.github/workflows/ci.yml:142-149`) runs the same command, and `deny.toml`
  `[graph] all-features = true` does not pull dev-deps into the evaluated graph.
  `docs/license-policy.md` says dev-deps **are** covered, so the gate is weaker
  than the policy it claims to enforce — the permissive differentiator is
  asserted, not checked. The `fuzz/` graph (`just deny-fuzz`) is a separate
  invocation that needs the same treatment.
- **#6** — `Cargo.toml:42` declares `[[bin]] name = "irr"` with no
  `required-features` / exclude, so `cargo package` ships and `cargo install`
  exposes the internal oracle tool, contradicting the `library-not-application`
  constraint. The tests/oracle harness may invoke `irr`, so check before moving
  it.

New signal `gate-weaker-than-policy` (N=1) tracks #4.

## Goal

Make the licence gate evaluate the **dev-dependency graph** for the root crate
**and** the `fuzz/` crate, and add a **mutation red-proof** (inject a synthetic
GPL dev-dep → the gate must go red), so the gate enforces what the policy
promises. Decide and apply the `irr` package disposition
(`required-features` on an empty feature / move to `examples/` / `exclude`) and
add a package-target-list check that fails if the published target set regains an
unexpected binary.

## Non-Goals

- No change to the licence **policy** itself (the allow-list stays); this closes
  the gap between the policy and the gate.
- No new runtime or dev dependencies.

## Notes for the Implementer

- Two independent sub-parts (licence-gate dev-graph; package surface) that can
  share one spec; the red-proof is for the licence gate.
- Emits a `DEC-*` for the `irr` disposition (open maintainer decision #2 in the
  triage) at design.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?** — <answer>
2. **Does any template, constraint, or decision need updating?** — <answer>
3. **Is there a follow-up spec I should write now before I forget?** — <answer>
4. **Where was the worst defect caught?** — `design` | `build` | `verify` | `ship` | `escaped` | `none`. — <one word>
5. **What can a user do now that they couldn't before?** — <answer | none>
