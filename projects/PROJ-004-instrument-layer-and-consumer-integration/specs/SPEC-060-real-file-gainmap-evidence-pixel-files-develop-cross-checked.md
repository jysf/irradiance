---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-060
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-016
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-069, DEC-068, DEC-003, DEC-009]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-059, SPEC-058]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-058, SPEC-059]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-016's <capability>". Optional; null is acceptable.
value_link: "Evidence that the Pixel GainMap develop matches Adobe dng_validate within pre-registered tolerances."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 12000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-060: Real-file GainMap evidence: Pixel files develop, cross-checked against dng_validate

## Context

STAGE-016 (PROJ-004). SPEC-059 builds `GainMap` by DEC-069 §2 and proves it on writer
fixtures. This spec proves it on the three held Pixel DNGs (3a, 4a and 7 Pro). Each
carries four mandatory 30×40 `GainMap`s.

The reference is Adobe DNG SDK 1.7.1's `dng_validate`, built locally (DEC-069 §6).
`dng_validate -2` writes the stage-2 image **after** `OpcodeList2`.

The design probe has already measured the two comparisons this spec turns into tests
(DEC-069 §Context):
- **The gain step alone**, on the SDK's own pre-gain stage 2: within ±1 code, with 99.97%
  of samples exact.
- **End to end**, against our own normalisation: within 7 codes. The gap is explained by
  the SDK scaling by `white − max black` where we scale by `white − black` per position,
  a pre-existing difference that is not changed here.

## Goal

Show that the Pixel files develop `Ok` on every develop entry, pin their output, and
cross-check the `GainMap` step against `dng_validate` within pre-registered tolerances.
The cross-check runs only where a local SDK output directory is supplied.

## Design

1. **A reproducible SDK recipe** (`scripts/sdk-stage2.sh`). It is committed, but no SDK
   bytes are.
   - **Inputs:** a `dng_validate` path, a corpus root and an output directory, all
     outside the repo.
   - **For each Pixel file,** it writes:
     - `<CAMERA>-post.tif`, from `dng_validate -2` on the real file;
     - `<CAMERA>-pre.tif`, from `dng_validate -2` on a scratch copy with the `OpcodeList2`
       tag number (51009) rewritten to an unused private tag (51010), so the SDK skips
       the list.
   - **Integrity:** the script refuses if the tag pattern isn't found exactly once, and
     writes the copies only inside the output directory.
   - **Docs:** document the SDK build in a comment, or in `docs/` if longer: the download
     URL and sha256 from DEC-069 §6, no XMP, JPEG XL stubbed, and Homebrew's libjxl.
2. **A minimal TIFF reader in test support.** It reads `dng_validate`'s uncompressed,
   single-sample, 16-bit, stripped output in either byte order. Anything else is refused
   loudly.
3. **The tier-B develop tests** (`tests/gain_map_corpus.rs`, using the existing corpus
   skip machinery). For each Pixel:
   - `develop_into`, `develop_with_into` (`StandardCrop` and `FullActiveArea`, with
     `BlackLevels::PerPosition`) and all four colour entry points return `Ok`.
     *(Build note, approved: `FullActiveArea` runs with `warp_alpha` 0. At the default
     1.0 the 3a's `WarpRectilinear` is refused with `UnsupportedWarpedFullActiveArea`
     (DEC-059), while the 4a and 7 Pro develop `Ok`; the test accepts only those two
     outcomes.)*
   - Output dimensions match `output_dimensions` and `output_dimensions_with`.
   - The threaded colour output equals the single-thread output.
   - **Pins:** pin a hash of each mono and colour output, following the existing pin
     style (`tests/develop_options.rs` or `tests/determinism.rs`, whichever already pins
     develop output).
   - **Reaches the step:** assert the `GainMap` is actually applied by comparing with
     the same develop on a sensor with `opcode_list_2 = None`. The outputs must differ,
     and the corner-to-centre mean ratio of a channel must rise (AC2, amended at build; it moves toward 1 only where the scene starts darker at the corners). That makes it a
     behavioural check, not just `Ok`.
4. **The SDK cross-check.** It runs only when `IRRADIANCE_SDK_STAGE2_DIR` is set, and
   prints a visible SKIP line otherwise, in the corpus-status style. The tolerances are
   **pre-registered here**:
   - **Gain step (the gate).** `apply_gain_maps` on `<CAMERA>-pre.tif` with the file's
     own `OpcodeList2` must equal `<CAMERA>-post.tif` with **max |Δ| ≤ 1** and **at least
     99.9% of samples exact**, on all three files.
   - **End to end (recorded, with a bound).** Mono `develop_with_into` with
     `FullActiveArea`, `apply_orientation: false`, `BlackLevels::PerPosition` and
     `warp_alpha: 0.0`, compared with `<CAMERA>-post.tif`, must have **max |Δ| ≤ 8**.
     The bound comes from:
     - the gap before any gain: up to 5 codes of max-black normalisation, plus 1 for
       rounding;
     - times a gain of up to 3.43;
     - limited by the measured 7 on the Pixel 4a, 6 on the 3a and 4 on the 7 Pro.

     Also record the histogram in the handback.
   - **Root cause pinned.** Mono `FullActiveArea` with no gain (the sensor with
     `opcode_list_2 = None`), compared with `<CAMERA>-pre.tif`, must have **max |Δ| ≤ 5**,
     with every difference ≥ 0 on the 4a and 3a. That shows the end-to-end gap is the
     normalisation, not the gain. The design probe measured max 5 on the 4a, 5 on the 3a
     (both all ≥ 0) and 2 on the 7 Pro. The 7 Pro's differences are mostly negative,
     because its rounded black sits below the SDK's fractional one (DEC-068).
5. **Evidence.** Run the recipe and the cross-check locally. Paste each file's numbers
   into the handback: max |Δ|, the fraction exact, and the histogram for each of the
   three comparisons.

## Inputs

- DEC-069 (§2, §6, §Context's table). SPEC-059's `apply_gain_maps` and develop wiring.
- **The SDK build** at `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/`. It is
  local and never committed; its build script is in `build/`.
- **The corpus** (`IRRADIANCE_CORPUS_DIR`), with the Pixel paths from
  `tests/metadata_oracle.rs`.

## Outputs

- **New:** `scripts/sdk-stage2.sh`, `tests/gain_map_corpus.rs`, and the test-support TIFF
  reader.
- **Modified:** the corpus pin file(s) (new pins only), `AGENTS.md` §6 (one line on the
  optional SDK cross-check and its env var), and `docs/provenance-ledger.md` (the
  real-file evidence appended to SPEC-059's row).

## Acceptance Criteria

- [ ] **AC1.** All three Pixels develop `Ok` on every mono and colour entry listed in
  Design §3, with the right dimensions, and threaded colour equals single-thread.
- [ ] **AC2.** *(Amended at build, 2026-11-19, approved by the orchestrator: "toward 1"
  assumed a scene with plain vignetting, and the Pixel 4a's corners are brighter than its
  centre.)* On each file, for CFA phase (0, 0) with 128-pixel windows (the mean of the
  four corner windows over the centre window), on mono `FullActiveArea`, unoriented,
  `warp_alpha` 0, `PerPosition`:
  1. the output differs from the same develop with `opcode_list_2 = None`;
  2. the corner-to-centre ratio with the `GainMap` is greater than without it;
  3. the phase-(0, 0) map, parsed by hand from `OpcodeList2`, has a corner gain greater
     than its centre gain, which anchors (2) to the map, not the scene;
  4. the ratio moves toward 1 only where it starts below 1 (the 3a and 7 Pro).
- [ ] **AC3.** Each Pixel output is pinned, and no other pin changes.
- [ ] **AC4.** With `IRRADIANCE_SDK_STAGE2_DIR` set, the gain-step gate holds on all
  three files: max |Δ| ≤ 1 and at least 99.9% exact. Unset, it prints SKIP and passes.
- [ ] **AC5.** The end-to-end bound (≤ 8) and the root-cause check (≤ 5, signed as
  stated) hold, with the numbers in the handback.
- [ ] **AC6.** A red-proof, control first: a mutant `apply_gain_maps` that samples at
  pixel corners (drops the `+ 0.5`) fails AC4's gate on at least one file. Record the
  max |Δ| and the fraction exact it reaches.
- [ ] **AC7.** The `just test` gates, `lint-ci`, `fmt-check`, `validate`, `doc-check`,
  `cost-audit`, `gate-audit` and `name-length-check` pass, and CI is green. CI has no
  SDK, so AC4 and AC5 skip there visibly.

## Failing Tests

- `tests/gain_map_corpus.rs`: AC1 to AC5. They are tier-B, and the SDK part is gated by
  an env var.

## Non-Goals

- **Fixing the max-black normalisation.** It is a carried follow-up, and this spec's
  root-cause check documents it.
- **Running the SDK in CI,** vendoring it, or committing its outputs.
- **The Samsung SM-G960F.** It is not in the held corpus, and its 1×1 unit maps are
  covered analytically by SPEC-059's AC2.
- **A colour comparison with the SDK's stage 3.** The SDK applies the optional warp and
  its own demosaic there, so it would not isolate the `GainMap`.

## Notes for the Implementer

- **Generating the inputs.** `dng_validate` takes about a second per file. The pre-gain
  copies are 24 MB each, so keep them outside the repo.
- **The probe's own numbers** (DEC-069 §Context) are the expected results. If a measured
  number lands outside a pre-registered bound, stop and report it. Don't widen the bound.
- **Plain-prose commit bodies**, with no code tokens. Never `git add Cargo.lock`, and
  never commit corpus or SDK bytes.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
