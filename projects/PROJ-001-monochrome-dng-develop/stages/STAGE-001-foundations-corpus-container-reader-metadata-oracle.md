---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-001                     # stable, zero-padded, continuous across the repo
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: irradiance

created_at: 2026-08-15
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Proves the container half of the thesis: that a DNG's real structure and metadata can be read from a published spec, with an oracle that says so.
  delivers:
    - "A bounded, panic-free TIFF/IFD reader with SubIFD recursion"
    - "Typed DNG metadata extraction"
    - "A two-tier corpus with a manifest, and tests that skip cleanly when tier B is absent"
    - "A metadata oracle diffing our output against dnglab and exiftool"
  explicitly_does_not:
    - "Any pixel decode — no strip reading, no unpacking"
    - "Any vendor container — DNG only"
    - "Colour of any kind"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# Warn-only, never a gate. A null here is honest; a guess is not. (DEC-013 §5)
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-001: Foundations: corpus, container reader, metadata oracle

## What This Stage Is

When this stage ships, `irradiance` can open a real Leica Q2 Monochrom DNG,
walk its IFD tree to the full-resolution SubIFD, and report every tag the develop
pipeline will later need — dimensions, bit depth, black and white levels, active
area, default crop, orientation, and the two opcode lists — with an oracle that
proves those values right rather than merely plausible. It reads no pixels.

**Estimated effort: 25–40 hours.** Hours, not calendar — pace is the maintainer's to set.

## Why Now

Everything downstream reads tags. Getting the container wrong silently
poisons every later stage, and the metadata oracle is the cheapest of the three
layers to stand up — `dnglab analyze --meta --json` and `exiftool` both already
produce machine-diffable output on files we hold.

The corpus lands here for the same reason: it is the project's top pre-mortem
risk, it cannot be delegated, and nothing after this stage is verifiable without
it. Front-loading it means the risk is retired while the code is still cheap.

## Success Criteria

- Our parsed tags match `dnglab analyze --meta --json` and `exiftool` on every tier-B file we hold
- The metadata oracle goes **red** on a deliberately corrupted tag — proven, not assumed
- The reader never panics on truncated, cyclic, or hostile input; a fuzz target exists and has run
- Tier-A fixtures run in CI; tier-B tests skip with a clear message when the corpus is absent
- Every offset and length read is bounds-checked and returns a typed error

## Scope

### In scope
- Two-tier corpus layout, manifest schema (path, hash, provenance, licence), skip-when-absent harness
- TIFF/IFD reader: byte-order handling, SubIFD recursion, depth and cycle guards
- DNG tag model and typed extraction for the tags in `docs/measured-q2m-dng.md`
- Metadata oracle harness
- The first fuzz target

### Explicitly out of scope
- Strip/tile reading and pixel unpacking (STAGE-002)
- Opcode *execution* — this stage only records that the lists are present (STAGE-003)
- Any write path; `irradiance` is read-only

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Run `just frame-stage STAGE-001` to promote these outlines into real specs.

- [ ] (not yet written) [M] Two-tier corpus layout, manifest schema, and the skip-when-absent fixture harness
- [ ] (not yet written) [L] TIFF/IFD reader — bounded, panic-free, cycle-guarded, SubIFD recursion — plus its fuzz target
- [ ] (not yet written) [M] DNG tag model and typed metadata extraction
- [ ] (not yet written) [S] Metadata oracle: diff parsed tags against `dnglab analyze --meta --json` and `exiftool`, and prove it goes red

**Count:** 0 shipped / 0 active / 4 pending

## Design Notes

**Implement from the Adobe DNG Specification, not from anyone's code.** The
spec is public and carries a patent grant for compliant implementations. Every
tag handler gets a provenance-ledger entry naming the spec section.

`crustyimg/src/metadata/tiff.rs` (718 lines) is a proven *design* to reference —
it is the maintainer's own code — but it cannot be reused: it is crustyimg-internal,
the dependency runs one way, and it is an order-preserving *serializer* for the
metadata lane. This reader is read-only and needs SubIFD reach for image data,
which that one does not have.

**Stopping point A:** real sensor dimensions, bit depth, levels, crop geometry,
orientation and opcode presence, read from the maintainer's own file.

## Dependencies

### Depends on
- External: `dnglab` and `exiftool` (tools, run — never linked)
- External: the Adobe DNG Specification
- SPIKE-001 — the corpus and the verified oracle contract

### Enables
- STAGE-002 — the plane decode reads its geometry and levels from here

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
