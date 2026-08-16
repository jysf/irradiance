---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-003                     # stable, zero-padded, continuous across the repo
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
    Turns a correct plane into a correct IMAGE — the first output a person would look at and judge.
  delivers:
    - "DNG opcode execution: FixBadPixelsConstant and WarpRectilinear"
    - "A tone curve and the three output modes"
    - "A perceptual develop oracle with a stated tolerance"
    - "An analytic oracle whose correct answer is arithmetic, not another implementation's output"
  explicitly_does_not:
    - "Colour — there is none to manage in a Monochrom file"
    - "LUT / film response curves — valuable, but the next wave"
    - "Any interactive or per-image parameter surface"

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

# STAGE-003: Opcodes and output: bad pixels, warp, tone curve, develop oracle

## What This Stage Is

When this stage ships, a Q2M DNG becomes a viewable image that matches an
independent reference render within a stated tolerance. The two DNG opcode lists
the file actually carries are executed, a tone curve is applied, and output is
available as linear f32, u16, or 8-bit. This is the minimum lovable version.

**Estimated effort: 20–35 hours.** Hours, not calendar — pace is the maintainer's to set.

## Why Now

The opcodes are not optional for this camera. `OpcodeList3: WarpRectilinear`
is a radial geometric correction and the Q-series 28 mm lens is designed around
software distortion correction — skip it and the output matches no reference
render, which would gut the oracle exactly when it is first needed.

The develop oracle lands here because this is the first stage whose output is
judgment-shaped. Establishing a tolerance now, against `dnglab analyze --srgb`,
is what stops colour work in the next project becoming an unbounded tuning loop.

## Success Criteria

- Output scores within a stated SSIMULACRA2 tolerance of `dnglab analyze --srgb`
- **An independent, non-rawler check passes**: at least one `dnglab makedng` fixture with an
  analytically known answer verifies the tone/level math without reference to any other implementation
- The develop oracle goes **red** on a wrong-black-level render
- `WarpRectilinear` is applied and its geometric effect is asserted, not eyeballed
- The tone curve is table-driven — no `powf` — and output is byte-identical on macOS and Linux
- The tolerance is written down with its justification, not chosen to make the test pass

## Scope

### In scope
- `OpcodeList1: FixBadPixelsConstant`
- `OpcodeList3: WarpRectilinear`
- Tone curve (table-driven) and output modes: linear f32, u16, 8-bit
- Develop oracle: SSIMULACRA2 against `dnglab analyze --srgb`, with a stated tolerance
- Cross-platform byte-identity check

### Explicitly out of scope
- Opcodes the Q2M does not carry — implement what the corpus contains, not the whole DNG opcode catalogue
- Highlight recovery (no clipped colour channels to reconstruct in monochrome)
- Auto-tone or any histogram-driven adaptivity — a reduction, and a determinism hazard

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

Run `just frame-stage STAGE-003` to promote these outlines into real specs.

- [ ] (not yet written) [S] `OpcodeList1: FixBadPixelsConstant`, with a test asserting the branch was HIT
- [ ] (not yet written) [L] `OpcodeList3: WarpRectilinear` — radial polynomial geometric correction
- [ ] (not yet written) [M] Table-driven tone curve and the three output modes, with the cross-platform byte-identity test
- [ ] (not yet written) [M] Develop oracle: SSIMULACRA2 vs `dnglab analyze --srgb`, stated tolerance, red-on-broken proof

**Count:** 0 shipped / 0 active / 5 pending

## Design Notes

**A tolerance chosen after seeing the result is not a tolerance.** State it,
justify it, then measure — and if the output cannot meet a defensible number, that
is a finding worth reporting rather than a number worth loosening.

Determinism is a design constraint from this stage on, because crustyimg's
`build --frozen` will consume this output. Table-driven curves rather than `powf`
(libm differs across platforms), pinned reduction order, no runtime SIMD dispatch.
The `develop_version` process-version field is introduced in STAGE-004 and this
stage's output is version 1.

⚠ **The dnglab oracle is single-sourced.** `--raw-checksum` and `--srgb` both come from rawler,
so bit-exact agreement proves we match *rawler*, not that we are *correct*. For decompression
that is fine — the transform is deterministic and matching the de facto reference is the goal.
For **developed output it is much weaker**: matching rawler's tone and rendering choices is not
the same as being right, and a tolerance test can pass while both are wrong together.

The mitigation is the analytic oracle above. `dnglab makedng` builds fixtures with *known*
levels, curves and matrices, so correctness becomes arithmetic rather than comparison — and an
arithmetic check cannot inherit rawler's bugs. The fully independent check is the ColorChecker
ΔE, which is PROJ-002; until then the analytic fixture is what stops this stage from being
circular.

**Stopping point C — the minimum lovable version.**

## Dependencies

### Depends on
- STAGE-002 — the normalized plane
- External: SSIMULACRA2 (available in crustyimg; confirm the integration shape)

### Enables
- STAGE-004 — crustyimg consumes this output
- PROJ-002 — colour rides the same output modes

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
