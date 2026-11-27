---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-031                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-006                      # parent project
repo:
  id: irradiance

created_at: 2026-11-23
shipped_at: 2026-11-27

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >-
    The core of the thesis: a colour Bayer DNG becomes a deterministic monochrome
    plane onyx can use in both its viewport and its export, with highlights that stay
    monotone through clipping.
  delivers:
    - "a public white-balanced, demosaiced linear RGB plane at sensor geometry, in the working space the mixer uses (onyx: DEC-059 colour reference view, its own candidate mixes, the quality study)"
    - "a public mixer from that plane to a monochrome u16 plane at sensor geometry, weights only, carrying its own black and white levels"
    - "a mix option on develop_with_into giving the developed mono at output_dimensions_with geometry, through the existing crop, orientation, warp and resampling"
  explicitly_does_not:
    - "ship filter presets (STAGE-032; the API takes weights only, presets are named weight sets)"
    - "publish a release (the PROJ-006 release stage)"
    - "change the output bits of any existing develop, demosaic or unpack call"
    - "reconstruct clipped highlights spatially"

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
    - tokens_total: 72765157
      estimated_usd: 58.09
      recorded_at: 2026-11-27
      notes: "The PROJ-006 orchestrator session, transcript 72d7e11e-3540-4b4a-aa95-e6918d07d862.jsonl, whole, 2026-11-23T17:05Z to 2026-11-27T06:00Z (352 unique message.ids, deduped by message.id, claude-opus-5-5, no subagents, priced at 5 / 25 USD per million with cache reads 0.1x and 1-hour writes 2x): input 869 + cache-creation 1,647,231 + cache-read 70,869,828 + output 247,229. Covers the PROJ-006 frame, onyx's answers, this stage's frame (PR 325), SPEC-091's main-loop design and the DEC-079 probe, and dispatching and recording every cycle. It also holds STAGE-034's framing (PR 357), which is not split out here: STAGE-034's own entry starts after 2026-11-27T06:00Z. The Leica sample search (transcript 30266aa6, sonnet and haiku) is STAGE-034's and is not counted."
    - tokens_total: 17637969
      estimated_usd: 13.95
      recorded_at: 2026-11-27
      notes: "SPEC-092's design side session, transcript 88a4fb3f-0376-4acf-bd72-ec2f2824b33a.jsonl, whole (72 unique message.ids, deduped by message.id, claude-opus-5-5, no subagents, priced at 5 / 25 USD per million with cache reads 0.1x and 1-hour writes 2x): input 147 + cache-creation 288,323 + cache-read 17,251,767 + output 97,732. The design row SPEC-092 left null for this close."
    - tokens_total: 19136740
      estimated_usd: 15.33
      recorded_at: 2026-11-27
      notes: "SPEC-093's design side session, transcript e57c98a2-a05d-4c06-9c17-a9e3a3b66067.jsonl, whole (67 unique message.ids, deduped by message.id, claude-opus-5-5, no subagents, priced at 5 / 25 USD per million with cache reads 0.1x and 1-hour writes 2x): input 137 + cache-creation 352,484 + cache-read 18,685,704 + output 98,415. The design row SPEC-093 left null for this close."
    - tokens_total: 4042131
      estimated_usd: 3.55
      recorded_at: 2026-11-27
      notes: "SPEC-091's ship side session, transcript f86890d9-6d50-4d21-96aa-f174d04f59d0.jsonl, whole (30 unique message.ids, deduped by message.id, claude-opus-5-5, no subagents, priced at 5 / 25 USD per million with cache reads 0.1x and 1-hour writes 2x): input 62 + cache-creation 118,735 + cache-read 3,907,024 + output 16,310."
    - tokens_total: 8176752
      estimated_usd: 5.95
      recorded_at: 2026-11-27
      notes: "SPEC-092's ship side session, transcript 123a0bd3-250c-4410-ad3f-c90adef8b73b.jsonl, whole (54 unique message.ids, deduped by message.id, claude-opus-5-5, no subagents, priced at 5 / 25 USD per million with cache reads 0.1x and 1-hour writes 2x): input 114 + cache-creation 131,897 + cache-read 8,019,771 + output 24,970."
    - tokens_total: 9310753
      estimated_usd: 7.09
      recorded_at: 2026-11-27
      notes: "FLOOR. SPEC-093's ship and this close, transcript dccdabeb-1768-4233-9c51-6a960e946dbe.jsonl, taken before the commit (58 unique message.ids, deduped by message.id, claude-opus-5-5, no subagents, priced at 5 / 25 USD per million with cache reads 0.1x and 1-hour writes 2x): input 115 + cache-creation 165,039 + cache-read 9,110,395 + output 35,204. The commit, push and CI watch come after it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-031: Colour to mono: the linear RGB step, the mixer, and the developed mix

## What This Stage Is

The three public steps onyx asked for (its PROJ-006 answers, 2026-11-23), with
the decisions that make them hold together: the working space, how clipping is
detected and handled, and the arithmetic. When it ships, a colour Bayer DNG can
be turned into (1) a white-balanced linear RGB plane, (2) a mixed monochrome
plane at sensor geometry for a GPU viewport, and (3) a developed monochrome
image for export, all additive to the existing API.

## Why Now

PROJ-006's first stage. onyx answered the four framing questions: Q3 and M11
first, weights-only API, two output shapes, and a crates.io release with no
change to existing output bits. Nothing in PROJ-002 or PROJ-005 is in these
files' way: PROJ-005's writer touches the write side, and PROJ-002's only open
spec (SPEC-035) waits on a photograph.

## Success Criteria

- The working space is decided in a DEC, with a test run on held colour files.
  Linear is the expected answer (onyx prefers it: one preset table, the same look
  on every body); the DEC records why if the test run says otherwise.
- The linear RGB step, at sensor geometry, matches today's
  demosaic-then-to-linear composition exactly when the working space is linear
  sRGB, or states the exact relationship if another space is chosen.
- The mixer is bit-exact against an analytic oracle (known RGB and weights in,
  exact mono out), and a red-proof shows the oracle can fail.
- A pixel with a clipped raw channel never comes out darker than the same pixel
  just below clipping: a monotone ramp through clipping on a tier-A fixture.
- The developed mix equals the sensor-geometry mix pushed through the same
  geometry the mono develop uses, so onyx's GPU-to-develop warp parity check
  (its DEC-062) carries over unchanged.
- Every existing output is unchanged: the corpus hashes for develop_into,
  develop_with_into without a mix, develop_color_into, demosaic_into and
  unpack_into do not move, and `just semver-check` reports additions only.
- Serial and threaded variants are byte-identical, and every new entry point has
  a fuzz target and is panic-free on untrusted input.

## Scope

### In scope
- The working-space DEC: linear sRGB primaries (what `to_srgb_linear_into`
  produces today), another linear space, or white-balanced camera RGB.
- The arithmetic of the new path. Today's colour matrix is f64 with each
  multiply and add rounded separately, and CI's determinism leg proves it
  byte-identical across platforms. onyx asked for fixed-point. Design decides
  whether the new path reuses the f64 matrix (reusing a proven step) or a
  fixed-point one (only for the new path; the existing colour output may not
  change).
- Clip detection on the raw camera channels, before white balance and before any
  matrix (onyx's point: after a matrix you cannot tell which channel clipped), and
  what the mixer does with that mask in the working space.
- The levels the mixed sensor-geometry plane carries, so onyx's single normalise
  step applies to it as it does to a mono sensor plane.
- Fuzz targets and tier-A fixtures for each new entry point.

### Explicitly out of scope
- Presets (STAGE-032) and the release (the PROJ-006 release stage).
- Spatial highlight reconstruction.
- X-Trans and other non-Bayer mosaics: refused as the colour path refuses them.
- An `irr` hook is not in or out yet: the mixer spec's design decides whether it is the
  cheapest way to look at a mix on a real file.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-NNN` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. An optional `[S]`/`[M]`/`[L]` tag on the
line is carried onto the spec as `task.complexity`.

- [x] SPEC-091 (shipped on 2026-11-25, PR 343; 58.6M / $38.30, est. 35M) [M] The white-balanced linear RGB step at sensor geometry, with the working-space DEC and the arithmetic decision for the new path
- [x] SPEC-092 (shipped on 2026-11-26, PR 354; 45.2M / $31.51, est. 65M) [L] The mixer at sensor geometry: weights-only rational API, the fixed-point row, the fade of negative weights near clip (DEC-083), the plane's black and white levels
- [x] SPEC-093 (shipped on 2026-11-27, PR 358; 80.1M / $54.21, est. 60M) [L] The developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling (DEC-084)

**Count:** 3 shipped / 0 active / 0 pending

## Design Notes

- **Order.** The three specs run one at a time, each depending on the one before.
  The first fixes the working space and arithmetic the others build on; the
  second defines the mixed plane the third resamples. They will touch
  `src/color.rs`, `src/demosaic.rs`, `src/develop.rs` and `src/lib.rs`, and may
  append `Error` variants (at the end of the enum, classified by `Error::kind()`,
  DEC-051).
- **The hard question is clipping in a linear space.** The clip mask comes from
  raw camera channels, but the weights apply after the matrix, where every
  output channel mixes all three inputs. "Drop the clipped channel and
  renormalise" (`docs/future-plans.md`) is only well defined in camera RGB.
  The mixer spec's design has to state the rule for a pixel with one, two or three
  clipped raw channels, and show the result is monotone. Candidates to weigh:
  treat the pixel as at white; mix that pixel in camera RGB with
  renormalised weights mapped from the working-space weights; or clamp the
  clipped raw channel before the matrix. Pre-register the oracle before choosing.
- **Weights.** Rational, fixed-point, summing to one (DEC-018). The caller
  passes resolved weights; onyx stores those in its recipe, never a preset name,
  so a later preset-table change cannot alter an existing print.
- **Existing paths stay put.** `develop_with_into` already accepts a CFA plane
  as mono (SPEC-066, DEC-061, "CFA as mono"). The mix option is a separate,
  opt-in field on the `#[non_exhaustive]` `DevelopOptions`; with it absent the
  develop is byte-identical to today.
- **Corpus.** Held colour files: Canon 5D Mark III, Pixel 3a, 4a and 7 Pro,
  Nikon D300S. No colour Leica yet; the Q3 and M11 come first by jysf's call
  (2026-11-23). This stage can prove correctness without them; STAGE-032's
  presets cannot be judged without them.

## Dependencies

### Depends on
- PROJ-002 STAGE-007 and STAGE-008's SPEC-033/034: demosaic, white balance,
  matrix, illuminant interpolation (shipped).
- PROJ-002 STAGE-027..029: the colour path's refusals (shipped).
- PROJ-004: DevelopOptions, framing, warp_alpha and orientation control (shipped).
- External input: onyx's answers to the PROJ-006 questions (2026-11-23).

### Enables
- STAGE-032 (presets as named weight sets).
- The PROJ-006 release, and onyx moving its pin.

## Stage-Level Reflection

*Filled in at the close, 2026-11-27, from the three specs' reflections (Prompt 1d).*

- **Did we deliver the outcome in "What This Stage Is"?** Yes. A colour Bayer DNG now
  becomes (1) a white-balanced linear sRGB plane at sensor geometry (SPEC-091,
  DEC-079), (2) a mono `u16` plane at sensor geometry, whole or in row strips, from
  rational weights alone (SPEC-092, DEC-083), and (3) the developed mono at the mono
  develop's geometry through `DevelopOptions::mono_mix` (SPEC-093, DEC-084), all
  additive. Against the Success Criteria: the working space is DEC-079, decided on a
  held-file probe; the linear step equals the demosaic-then-to-linear composition; the
  mixer matches an `i128` oracle at every pixel, with every pre-registered mutant red;
  the fade keeps the mix monotone through clipping (0 decreasing rays on every held row,
  against 7,199 for the plain fold on the Canon luminance row, and a ramped Tier-A
  fixture); the developed mix is bit-identical to the mono develop of the mixed plane,
  so onyx's DEC-062 parity check carries over; every existing hash is unchanged and
  `just semver-check` reports additions only; the band and strip forms equal the whole
  plane, and every new entry is fuzzed and panic-free. Each `value_link` delivered what
  it claimed, and nothing in `explicitly_does_not` shipped.
- **How many specs did it actually take?** 3, as planned. 183.9M tokens and about
  $124.02 across the three specs' builds and verifies (58.6M, 45.2M, 80.1M) against
  160M estimated (35M, 65M, 60M), plus the orchestration cost recorded above.
- **How many outlines survived unchanged?** 3 of 3 (framed with `just frame-stage` in
  PR 325). Every outline's scope held to ship. The approaches changed as designed: the
  clip rule became a fade of negative weights near clip (DEC-083) rather than any of
  the three framed candidates, and SPEC-093 was resized M to L at design.
- **What changed between starting and shipping?** The clip question was answered in
  camera space by fading only the negative folded weights near clip, and the first
  colour Leica files showed every onyx target body is refused on the colour path, which
  became STAGE-034 instead of a change to this stage.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - `projects/_templates/spec.md`, Failing Tests (landed at this close): the Rule-clause
    map names three clauses that need their own row (a public contract constant, an order
    word, a fault pair called impossible); the Distinguishing fixture paragraph says check
    per point and compare a perturbation with the output's quantum; a new "Claims about
    existing code" paragraph asks for the `file:line` behind every fact the design states
    about code it does not change.
  - No AGENTS.md or constraint change. `test-before-implementation` stays as written; its
    partial application is a `watch` lesson for now.
- **Signals dispositioned at this close?** (Prompt 1d step 7) All 11 open or `watch`
  stage-close lessons were walked, and `last_touched` bumped to 2026-11-27.
  - Codified: `read-the-kernel-before-deciding-the-approach` (N=2 to N=3, SPEC-093's
    "spare bit 5" and "F10 removes ColorMatrix1").
  - Template tightened, still codified: `every-rule-clause-maps-to-a-reachable-test` and
    `a-designed-mutant-must-be-able-to-fail-on-its-fixture`, the two whose wording
    SPEC-092's ship sent to this close. Each recurred in all three specs since
    codification.
  - Still `watch`: `analytic-oracle-premise-must-survive-rounding` (N=2; SPEC-093's
    one-ulp red-proof counted under the mutant lesson instead),
    `hostile-fixture-shadowed-by-an-earlier-guard` (N=2), and, with no new instance,
    `a-fix-inherits-the-precondition-of-the-thing-it-fixes`,
    `probe-the-hard-assumption-at-frame-time` (the Leica refusals were not probeable at
    frame: no colour Leica file was held), `patent-check-named-algorithms-at-frame`,
    `saturating-as-cast-masks-explicit-clamp-in-quantize-tests`,
    `closed-form-numeric-needs-extreme-magnitude-fixtures` (applied in SPEC-092's AC14),
    `parallel-tests-share-a-global-counter`,
    `spec-must-carry-what-the-probe-number-depended-on` and
    `a-faster-test-lane-can-blind-tests-that-measure-the-compiler`.
  - Project-close signals with new evidence (not decided here):
    `fuzz-targets-without-a-ci-smoke-job` (N=4) and `named-tests-can-pass-vacuously`.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - New `watch` lesson at N=2: `red-first-covers-only-the-integration-suite` (SPEC-092 and
    SPEC-093 each saw only the integration suite fail first; their red rests on mutants).
  - Follow-up for the project: the `demosaic` fuzz CI hygiene patch now has four specs'
    entries behind it and should land before the PROJ-006 release.
