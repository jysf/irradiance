---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-033                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-25
shipped_at: 2026-11-25

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
  sessions:
    - tokens_total: 6617213
      estimated_usd: 4.12
      recorded_at: 2026-11-25
      notes: "Floor, measured from the side session's transcript bc4383e8-214e-44a7-a105-8d69a9586e17.jsonl (claude-opus-5-5), the 29 unique message.ids after the 52 already counted in SPEC-096's build row: input 64 + cache-creation 35,243 (1-hour) + cache-read 6,562,353 + output 19,553, at 5 / 10 / 0.5 / 25 USD per million. Covers dispatching the cold verify, recording it, this ship and this close. The last messages of the close were not yet flushed when this was summed. The stage's framing is inside SPEC-096's build row (one session framed, designed and built)."
                                    # - tokens_total: N
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
  SPEC-096 records that limit (closed at ship as SPEC-096/FU-5).
- Any crate surface for the browser. The demo's bindings stay in `jysf/irradiance-demo`.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-096 (shipped on 2026-11-25) — [S] a wasm32 build gate with a red-proof, and the live demo link in the README

**Count:** 1 shipped / 0 active / 0 pending

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

- **Did we deliver the outcome in "What This Stage Is"?** Yes. PR 346 (merged as
  `b866757`) added CI jobs `wasm32` and `wasm32-red-proof`. In CI run 45676320193 (rustc
  1.100.0) both surfaces built, and the red-proof went red on both wasm-only faults with
  attributed errors. The README links the live demo and its source.
- **How many specs did it actually take?** One, SPEC-096, as planned. Expected S, actual
  S: 10.0M / $8.47 against an 8M estimate.
- **How many outlines survived unchanged?** Not framed with `just frame-stage`.
- **What changed between starting and shipping?** The red-proof's faults. The design probe
  showed that the requested examples (`Instant::now`, `std::fs`) compile for wasm32, so the
  gate can only claim build-time breakage. That limit is written into the gate, the
  recipes, AGENTS.md and the CHANGELOG.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - None codified. A cold verify that mutates through a symlinked copy can write into the
    real tree. That happened once (FU-3) and was fixed in the red-proof with `cp -RL`.
    It is not a signal yet.
- **Signals dispositioned at this close?** All 15 `watch` signals with `disposition_at:
  stage-close` were walked and all stay `watch`.
  `probe-the-hard-assumption-at-frame-time` was applied, not violated: the design probe
  caught the wrong red-proof faults before any script existed.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - No.
- **Cost.** SPEC-096 10.01M / $8.47 (build including design 8.80M, verify 1.21M).
  Orchestration 6.62M / $4.12 (floor). Stage about 16.6M / $12.59.
