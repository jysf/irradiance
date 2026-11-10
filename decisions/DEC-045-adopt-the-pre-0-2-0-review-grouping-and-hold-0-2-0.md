---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-045
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-002
repo:
  id: irradiance

created_at: 2026-11-10
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - projects/PROJ-002-bayer-colour-develop/stages/STAGE-010-downstream-consumer-api-validate-before-allocate-error-kinds-determinism-0-2-0.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-039-plane-validate-validate-before-allocate-extract.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-052-parser-resource-guard-aggregate-entry-byte-budget.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-053-compression-seven-decoded-frame-shape-binding.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-054-required-processing-validation-and-sensor-api-stability.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-055-license-gate-dev-graph-and-package-surface.md
  - projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md
  - docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md

tags:
  - release
  - 0.2.0
  - code-review
  - re-plan
  - sequencing
  - stage-010
  - semver
---

# DEC-045: Adopt the pre-0.2.0 review grouping, and hold 0.2.0 for all four P1s

## Decision

Hold the `irradiance 0.2.0` release until the four P1 findings of the external
pre-0.2.0 code review (triaged in
`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`, merged as PR #161) are
fixed, and re-plan STAGE-010 around the review's A–F grouping rather than the
original "SPEC-039 first" sequence. The maintainer confirmed both calls on
2026-11-10: **adopt the A–F plan** and **hold 0.2.0 for all four P1s (groups
A–E)**, not a partial 0.2.0 with C deferred to 0.2.1.

## Why

The original STAGE-010 plan (SPEC-039 validate-before-allocate, then SPEC-040
error kinds, SPEC-041 determinism, SPEC-042 release) was written before the
review. The review found four confirmed P1 failures at the public boundary, three
of which make a hostile or unsupported input succeed silently — exactly the class
the "safe to depend on" thesis exists to prevent — and one of which is a licence
gate weaker than the policy it claims to enforce. Cutting 0.2.0 (which pyirradiance
will pin) without them would publish those failures. The review also shows the
original SPEC-039 design is incomplete: its uncompressed byte-count invariant does
not carry to compressed strips, so the real preflight must be unified across
findings #1/#2/#3/#5, which re-sequences SPEC-039 after that work.

## The grouping (replaces the old STAGE-010 backlog)

| Group | Spec | Covers (review finding) | Size | Depends on |
|-------|------|-------------------------|------|------------|
| A | SPEC-052 | #1 parser resource guard (aggregate entry/byte budget; reject aliased entry tables) | S | — |
| B | SPEC-053 | #2 Compression-7 decoded-frame shape binding | S | — |
| C | SPEC-054 | #3 required-processing validation + `Sensor` API stability (DEC + non-exhaustive) | M | — |
| D | SPEC-055 | #4 licence gate over dev-graph + #6 `irr` package surface | S | — |
| E | SPEC-039 | #5 validate-before-allocate completion + README/CHANGELOG/matrix + smoke matrix + EOI | M | A, B, C |
| — | SPEC-040 | `Error::kind` / `ErrorKind` (original pyirradiance request) | S | SPEC-039 |
| — | SPEC-041 | cross-arch determinism evidence (original) | S | — |
| F | SPEC-042 | release 0.2.0 (+ corpus evidence run, package target list, deny-on-dev-graph, the full public-struct non-exhaustive sweep) | S | A–E, 040, 041 |

A, B, D and SPEC-041 are independent and fan out in parallel; C is the
design-heavy group and settles the `Sensor` API; E unifies the preflight and so
depends on A–C; F is the release and is blocked by everything above.

## Consequences this decision also settles

1. **STAGE-010 charter amended.** The stage's original `explicitly_does_not`
   lines "Any change to existing public API" and (implicitly) "no hardening of
   shipped parser/codec paths" are relaxed: the review requires a small,
   controlled, additive API change — making `Sensor` (and, at release, the other
   public structs) non-exhaustive and adding one `Sensor` field for the kept
   `OpcodeList2` bytes. Every change remains additive; no existing signature
   changes. The stage still does no colour-accuracy work and no ML/NumPy work.
2. **`#[non_exhaustive]` on the public structs is release-blocking and
   semver-urgent** (both the staff review of 2026-11-06 §4 and this review flag
   it). `Sensor` is made non-exhaustive inside group C, because C is what adds a
   `Sensor` field; the remaining public structs (`Cfa`, `WarpRect`, `Opcode`, …)
   are swept in the release spec SPEC-042 before the tag.
3. **Per-group DECs still required.** This decision is the re-plan and the hold;
   the load-bearing technical choices each get their own `DEC-*` at the group's
   design cycle: the parse budget (A), the supported-opcode contract and `Sensor`
   construction model (C), and the `irr` package disposition (D). Open maintainer
   decisions #2 (`irr` disposition) and #3 (direct `Sensor` construction) are
   deferred to the design of D and C respectively; #4 (JPEG without a final EOI)
   is decided in E.

## Scope and non-goals

- **In:** the hold; the A–F backlog restructure of STAGE-010; re-scoping SPEC-039
  to group E (depends on A–C); the four new frame specs SPEC-052–055; the charter
  amendment.
- **Out:** the §5 perf headroom from the staff review (a later dedicated stage,
  not a 0.2.0 blocker); any colour-accuracy claim (gated on SPEC-035 / the
  ColorChecker chart); shipping 0.2.0 before A–E (explicitly rejected by the
  maintainer); inventing the per-group technical numbers here (they are each
  group's design-cycle `DEC-*`).
