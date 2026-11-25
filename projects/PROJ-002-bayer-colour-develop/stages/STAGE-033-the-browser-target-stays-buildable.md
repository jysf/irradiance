---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-033                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-25
shipped_at: null

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: "keeps the published crate building for the browser (wasm32-unknown-unknown), so the library's develop path, colour included, stays usable client-side as DEC-002 intended and the public demo shows"  # one sentence; which part of project.value.thesis
  delivers: ["a CI gate that fails when the library stops compiling for wasm32-unknown-unknown, default features and the write feature", "a README link to the live Anatomy of a RAW demo"]  # user-visible capabilities this stage delivers
  explicitly_does_not: ["add any app, CLI, wasm-bindgen glue or JS surface to the crate (library-not-application; the demo lives in jysf/irradiance-demo)", "run the test suite under a wasm runtime", "change library behaviour or output"]  # what this stage is NOT trying to do

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions: []                      # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-033: the browser target stays buildable

## What This Stage Is

The claim that irradiance runs in the browser becomes a gated fact. CI builds the library
for `wasm32-unknown-unknown` on every PR, with default features and with `write`, and a
red-proof shows the gate fails on a change that breaks only the wasm build. The README
links the live demo that runs the published crate compiled to WebAssembly.

## Why Now

`jysf/irradiance-demo` now deploys an "Anatomy of a RAW" page to GitHub Pages that compiles
the published crate to WebAssembly; its output SHA-256 matches the native build byte for
byte. The library compiles for wasm32 today with no changes, but nothing in this repo checks
it, so the first unix-only API or 64-bit-`usize` constant would break the demo silently on
the next release. DEC-002 kept the browser target open "at near-zero cost"; this stage is
what keeps that cost near zero.

## Success Criteria

- A CI job builds `--lib --release --target wasm32-unknown-unknown` with default features
  and with `--features write`, and fails if either build fails.
- A red-proof CI job shows the gate passes on the honest tree and fails on injected faults
  that compile natively but not for wasm32, each attributed by its compiler error.
- `README.md` links https://jysf.github.io/irradiance-demo/ and its source repo.

## Scope

### In scope
- SPEC-096: the gate script, its red-proof, two `just` recipes, two CI jobs, the README
  link.

### Explicitly out of scope
- Running tests under a wasm runtime (wasmtime, node). The build gate cannot see APIs that
  compile on wasm32 and fail at run time (`Instant::now`, `std::fs`, `std::thread::spawn`);
  SPEC-096 records that limit and files it as a follow-up.
- Any crate surface for the browser. The demo's bindings stay in `jysf/irradiance-demo`.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-096 (design) — [S] a wasm32 build gate with a red-proof, and the live demo link in the README

**Count:** 0 shipped / 1 active / 0 pending

## Design Notes

- Placement: a stage of its own in PROJ-002, the active project, as STAGE-028 and STAGE-030
  were; STAGE-014 (the 0.2.x maintenance bucket) is closed.
- Gate shape copies `doc-check` (SPEC-056): a script that takes an optional crate dir, a
  red-proof that runs that exact script on a temp copy, a `just` recipe for each, a CI job
  for each.

## Dependencies

### Depends on
- External: `jysf/irradiance-demo` (the deployed page the README links).

### Enables
- A release whose notes can say "builds for wasm32-unknown-unknown" as a CI-checked fact.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** Not framed with `just frame-stage`.
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** <note>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
