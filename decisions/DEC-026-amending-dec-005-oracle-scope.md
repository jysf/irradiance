---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-026
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-10-27
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - docs/oracle-contract.md

tags:
  - oracle
  - develop
  - warp
  - opcode
  - orientation
  - dnglab
  - spec-020
  - spec-021
  - dec-005
---

# DEC-026: the ≥85 develop oracle covers warp-free, Orientation-1 pixels only — amending DEC-005's scope clause

## Decision

`DEC-005`'s SSIMULACRA2 ≥ 85 develop-layer tolerance is **scoped to the
geometry and gross tone of warp-free, Orientation-1 pixels only**.
**`WarpRectilinear` (`OpcodeList3`) pixels and `Orientation ≠ 1` frames are
OUT of that oracle's scope** — `dnglab analyze --srgb` applies neither
correction, so it provides no usable signal for either, in either direction.
Those two surfaces are verified **analytically** instead: warp geometry by
`tests/warp.rs`'s `AC4`/`AC10` (the DNG 1.7.0.0 §6.4.1 formula, cross-checked
against `SPIKE-001`'s measured displacement), and levels + crop + orientation
by `DEC-004`/`SPEC-015`'s analytic oracle. This **amends `DEC-005`'s scope
clause** (its `## Decision` item 2) and **neutralizes its `## Validation`
premise for warp-bearing and rotated frames only**.

`DEC-005`'s scope clause, quoted verbatim:

> 2. **The develop-layer tolerance is SSIMULACRA2 ≥ 85**, and it is scoped: it
>    verifies **geometry and gross tonal correctness only**. Levels are **out of its
>    scope** and belong to `DEC-004`.

and its `## Validation` premise, quoted verbatim:

> Right if the first real develop implementation scores ≥ 85 against
> `dnglab --srgb` without the threshold being renegotiated. **If it scores below
> 85, the render is wrong — the bar does not move.** That was the point of
> registering it in advance.

After this amendment that clause reads: the ≥ 85 tolerance verifies geometry
and gross tone of **warp-free, Orientation-1** pixels only; levels (`DEC-004`),
`WarpRectilinear` geometry (`tests/warp.rs` `AC4`/`AC10`), and `Orientation ≠ 1`
frames are each out of its scope. And its `## Validation` premise holds **only
for warp-free, Orientation-1 renders**: for a warp-bearing or rotated frame, a
score below 85 is **not** evidence the render is wrong — the oracle is blind to
exactly the correction that is present, so a *more* correct render scores
*further* from the reference, not closer (measured below).

**Why amend rather than revert.** `DEC-005`'s ≥ 85 bar was pre-registered in
writing before any score was computed, together with its falsifier (*a missing
`WarpRectilinear` must land far below 85*), and **both survived contact with the
data**: a one-pixel geometry shift still scores 63, honest gamma-1.05 tone still
passes. The number is right for the pixels it covers, and `tests/perceptual_
oracle.rs`'s tier-A calibration (`SPEC-020`) still exercises it. What was wrong
was never the threshold — it was the *implicit* claim that the comparison
reference (`dnglab --srgb`) is correct for **every** develop stage. It is not:
`dnglab`/`rawler` implement no DNG `OpcodeList` processing and ignore EXIF
`Orientation`. Reverting the bar would throw away a working geometry+tone check
for warp-free frames to escape a problem that only exists for two specific
surfaces. The right move is the one `SPEC-015` already set the precedent for
(levels: no comparison oracle → analytic oracle): name what the comparison
oracle can and cannot see, and verify the rest analytically.

## Context

`SPEC-018` built `WarpRectilinear` and, executing `SPEC-020`'s pre-registered
rule (measure the ≥ 85 oracle on all three decodable Q2M frames), surfaced that
the oracle cannot see the stage at all. `DEC-024` Finding 2 recorded the warp
leg; it does **not** record the Orientation leg, which `SPEC-018/FU-6` and
`SPEC-017/FU-2` measured independently. This decision records both together as
the formal scope narrowing `DEC-024`'s Consequences explicitly deferred to ship
("either drop the dnglab-oracle requirement for `WarpRectilinear` specifically …
or find a second reference decoder").

### The warp leg — `DEC-024` Finding 2 (cited, not re-derived)

`dnglab`/`rawler` implement **no** DNG `OpcodeList` processing (confirmed in
`DEC-024` by both source inspection — for feature presence only, run as a tool
per `provenance-recorded-per-algorithm` — and independent behavioural NCC-tile
analysis). Measured on `L1021223.DNG` (`tests/warp.rs` `AC8`, reproduced this
cycle):

- the **present-and-correct** bilinear warp scores **−60.169** against
  `dnglab --srgb`;
- a synthetic **no-warp** baseline (same pipeline, warp stage skipped) scores
  **+83.145** once a matching approximate sRGB gamma stand-in is applied for the
  tone comparison (`develop_into`'s output is scene-linear per `DEC-018`;
  `dnglab --srgb` is gamma-encoded, so the stand-in is a diagnostic only, never
  shipped);
- `AC9`'s red-proof shows the same inversion: honest **−60.193**, `kr1`-zeroed
  mutant **−55.075** — zeroing a real coefficient moves the applied correction
  *closer* to `dnglab`'s always-uncorrected reference, so the mutant's `< 85` is
  satisfied **vacuously**, not because the fault was caught.

Applying the correct warp makes the match to `dnglab` **worse**, not better —
the opposite of what a correction does against a correct reference. The behaviour
is structural, not a kernel-quality artifact (`DEC-024` Alternatives, Option A),
so no sharper kernel could move a number the reference never computes.

### The Orientation leg — `SPEC-018/FU-6` + `SPEC-017/FU-2` (not in `DEC-024`)

`dnglab analyze --srgb` **ignores EXIF `Orientation`**: it prints `8368 5584`
(the sensor-native, un-rotated size) for an `Orientation`-6 frame as well as for
an `Orientation`-1 frame — measured directly on both `L1026016.DNG`
(`Orientation` 6) and the `Orientation`-1 frames (`SPEC-017/FU-2`, independently
re-measured after `SPEC-018/FU-6` first observed it). `develop_into` *does* apply
`Orientation` (`SPEC-014`, shipped and tested), so for `L1026016.DNG` our correct
render is **5584×8368** while `dnglab` emits **8368×5584**. The two images are
structurally incomparable — the old `AC8` path's `assert_eq!` on dimensions
panicked before scoring. This is the same root as the warp leg (`dnglab` applies
a correction fewer than we do) on a different pipeline stage, and the same fix.

## Alternatives Considered

- **Option A: relax `DEC-005`'s ≥ 85 threshold for warp-bearing / rotated files.**
  - What it is: lower the bar so these frames "pass".
  - Why rejected: it would not restore a signal `dnglab` fundamentally cannot
    provide — no threshold makes a reference that applies no warp and no rotation
    into a correct oracle for warp and rotation. `DEC-024` Option B rejected this
    on the same merits. Lowering a meaningless number trains us to ignore the
    oracle (the `DEC-005` Option A anti-pattern).

- **Option B: find a second, DNG-opcode-capable reference decoder.**
  - What it is: restore a real comparison oracle for `OpcodeList3`.
  - Why rejected here: none is available, and the search is `DEC-024`'s
    `## Validation` ("revisit if a second DNG-opcode-capable reference decoder
    becomes available") — not this spec's to conduct. Left open there, not
    foreclosed here.

- **Option C (chosen): name the carve-out and verify analytically.**
  - What it is: narrow `DEC-005`'s declared scope to warp-free, Orientation-1
    pixels; verify warp geometry by `tests/warp.rs` `AC4`/`AC10` and
    levels+crop+orientation by `DEC-004`/`SPEC-015`, exactly the substitution
    `SPEC-015` made when `SPIKE-001` found no comparison oracle covered levels.
  - Why selected: it keeps the working geometry+tone check for the pixels it
    genuinely covers, stops the oracle asserting a tolerance it cannot meet for
    the two stages it is blind to, and leaves warp correctness proven by a
    falsifiable, corpus-free, CI-visible analytic gate (`AC10`'s 339.5 px
    separation against a 20 px bar) rather than an oracle that produces no score.

## Consequences

- **Positive.** `SPEC-020`'s develop oracle no longer claims a tolerance it
  cannot meet. Its declared scope (warp-free, Orientation-1) now matches the
  signal `dnglab` actually provides, and the carve-out is written down where a
  future reader will meet it (`docs/oracle-contract.md`, and the reframed
  `tests/warp.rs` characterization) instead of being carried as an `#[ignore]`.
- **Positive.** Warp correctness rests on `tests/warp.rs`'s `AC4`/`AC10`, an
  oracle-independent check that keeps working even if `dnglab`'s behaviour or
  availability changes — the right property for a library whose differentiator
  is spec fidelity (`AGENTS.md` §1).
- **Negative.** The develop layer now has **two** surfaces with no comparison
  oracle at all (levels, from `DEC-004`; and now warp + Orientation). Each is
  covered analytically, but an analytic check can only test the arithmetic this
  project chose against the spec's own formula, not against an independent
  implementation — the single-sourcing caveat `docs/oracle-contract.md` already
  carries, now sharper for these stages.
- **Neutral.** The ≥ 85 number itself is unchanged; only the set of pixels it is
  claimed to cover is. `DEC-005`'s P5-header workaround (its `## Decision` item 1)
  is untouched.

## Validation

Right if: `tests/warp.rs`'s `AC4` continues to reproduce each frame's own
`f(1)`-derived corner displacement within 1 px, `AC10`'s red-proof continues to
show a ≥ 20 px separation (measured 339.5 px) on a `kr1`-zeroed mutation, and the
reframed characterization (`warp_pixels_are_below_the_srgb_oracle_baseline`)
continues to measure the warp-bearing score below the no-warp baseline on the
Orientation-1 frames. Revisit if: a second DNG-opcode-capable, Orientation-aware
reference decoder becomes available (restoring a real comparison signal for the
carved-out stages — `DEC-024`'s open door), or `dnglab` starts applying
`OpcodeList3` / `Orientation` (the dimension and score assertions will say so).

## References

- Amends: `DEC-005` (`decisions/DEC-005-develop-oracle-mechanics.md`) — its
  `## Decision` item 2 (the ≥ 85 scope clause) and its `## Validation` premise,
  for warp-bearing and `Orientation ≠ 1` frames only. `DEC-005` carries a pointer
  to this record.
- Related decisions: `DEC-024` (warp evidence, Finding 2; the deferred scope
  narrowing this discharges), `DEC-022` (the amend-via-new-DEC shape modelled
  here), `DEC-004` (levels/crop/orientation verified analytically), `DEC-018`
  (scene-linear `u16` output — why the no-warp baseline needed a gamma stand-in).
- Related specs: `SPEC-021` (this change), `SPEC-020` (the oracle narrowed),
  `SPEC-018`/`FU-6` and `SPEC-017`/`FU-2` (the Orientation leg, independently
  measured), `SPEC-015` (the analytic-substitution precedent).
- External: DNG 1.7.0.0 §6.4.1 (`WarpRectilinear`), §Chapter 7 (Opcode List
  Processing); EXIF `Orientation`. `dnglab` 0.7.2 run as a tool, never linked.
