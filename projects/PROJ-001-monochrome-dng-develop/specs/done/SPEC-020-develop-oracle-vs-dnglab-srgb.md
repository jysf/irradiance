---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-020
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
                                   # ⚠ RAISED from the frame stub's `medium`.
                                   # SPEC-018 (warp — the item that most decides
                                   # PROJ-001's thesis) lands with no independent
                                   # check unless this ships first; the whole
                                   # sequencing argument turns on it.
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
                                   #   M matches the frame. Three helpers (PNM
                                   #   reader, 16→8 sRGB conversion, metric
                                   #   wrapper) + a tier-A red-proof fixture set +
                                   #   one dev-dep + a DEC + a provenance row.
                                   #   Precedent: SPEC-015 was framed S and
                                   #   shipped L; that overshoot was punch-list
                                   #   rounds, which this spec's smaller surface
                                   #   should not attract.
  complexity_actual: M             # M — single build pass, single verify pass, no punch-list round. 47.3M tokens across build+verify matches the S-oracle floor (SPEC-013 ~47M) and the M-oracle expectation; SPEC-015's L was inflated by FU-10's ship-round rework, which did not fire here. stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-004, DEC-005, DEC-013, DEC-017]
  constraints: [oracle-must-be-shown-red, no-copyleft-dependencies, no-panics-on-untrusted-input, library-not-application, test-before-implementation, provenance-recorded-per-algorithm]
  related_specs: [SPEC-013, SPEC-015, SPEC-017, SPEC-018, SPEC-019]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # none — this spec is deliberately independent
                                   # of the develop pipeline so it can ship first
                                   # (see ## The design decision this spec rests on)

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-003's <capability>". Optional; null is acceptable.
value_link: "gives STAGE-003 the SSIMULACRA2-scored develop oracle its remaining
  specs (SPEC-017 bad pixels, SPEC-018 warp, SPEC-019 tone) must beat — so a
  missing warp cannot pass unseen the way a wrong BlackLevel passed SSIMULACRA2 95.62"

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
  tokens_estimate: 55000000
  # Basis, so the next estimate can be judged rather than guessed: SPEC-015
  # (an oracle spec, expected M, shipped L) landed at 98.2M across four
  # cycles including a punch-list ship round. SPEC-013 (also an oracle, S)
  # ~47M across build+verify. This spec has SPEC-015's shape (oracle +
  # red-proof + property/tolerance vocabulary), a smaller code surface
  # (test-side helpers only, no src/ changes), and a first-time dev-dep
  # sanction inline — 55M assumes build + verify + one punch-list round,
  # matching SPEC-015 minus the ship-round rework FU-10 caused. If it lands
  # near 40M the estimator was pessimistic; near 90M the oracle-class
  # pattern is systemic and next M-oracle should be estimated as L.
  sessions:
    - cycle: build
      agent: claude-sonnet-5
      interface: other
      tokens_total: 36917935
      estimated_usd: 15.23
      duration_minutes: 40
      recorded_at: 2026-09-06
      notes: "Deduped by message.id from own transcript (own scratchpad UUID, not text match); 250 usage objects, 136 distinct ids, raw combined 30,764,946 (input 272 / cache-write(1h) 315,710 / cache-read 30,335,786 / output 113,178), priced per-component at published claude-sonnet-5 rates ($3/$15/$6/$0.30 per Mtok, the SPEC-005/HANDOFF-022 precedent) = $12.69, +20% uplift for the remaining handback-writing turns = $15.23 and 36,917,935 tokens."
    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 10366144
      estimated_usd: 8.25
      duration_minutes: 16
      recorded_at: 2026-09-06
      notes: "Deduped by message.id from own transcript identified by this session's scratchpad UUID d83664a6-d3a9-4ac2-b681-bb222d00d0a7 (not text-matched, per this project's identify-own-transcript-for-cost-handback memory); 153 usage objects, 65 distinct ids, all message.model claude-opus-5 (tier_map.verify's prediction was RIGHT this time), raw combined 8,638,453 (input 130 / cache-write(1h) 161,424 / cache-read 8,435,217 / output 41,682 — 97.6% cache-read), priced per-component at published claude-opus-5 rates ($5/$25/$10/$0.50 per Mtok input/output/1h-write/read) = $6.87, +20% uplift for the turns writing this handback = $8.25 and 10,366,144 tokens; verify edited no repo file except this handback block — the one red-proof mutation (identity warp in tests/support/perturb.rs) was reverted and the tree confirmed clean."
    - cycle: ship
      agent: claude-opus-4-7
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-06
      notes: "main-loop, not separately metered — the orchestrator's ship pass writes the Follow-ups + Reflection + signals.yaml amendment + SPEC-018 Context correction here; interleaved with the SPEC-017 design in the same session, so a per-cycle number would be invented. Non-null enforcement (AGENTS.md §4) exempts design/ship."
  totals:
    tokens_total: 47284079
    estimated_usd: 23.48
    session_count: 3
shipped_at: 2026-09-06
---

# SPEC-020: Develop oracle vs dnglab srgb

## Context

`STAGE-003` turns the normalized plane into an image, and its four specs each
produce output a person will judge. The develop layer of the oracle contract
(`docs/oracle-contract.md`, corrected by `DEC-005`) is the check that shape of
output has. Without it, the three specs that ship developed pixels — `SPEC-017`
(`FixBadPixelsConstant`), `SPEC-018` (`WarpRectilinear`), `SPEC-019` (tone
curve + output modes) — each land with only the analytic checks `DEC-004` /
`SPEC-015` covers, and those are structurally blind to geometry and to the
tone-curve output. `SPIKE-001` measured exactly what that gap looks like: **a
missing warp scores −68**, a **1-pixel shift** scores **63**, both far below
the pre-registered ≥ 85. `SPEC-018` without this oracle would ship 141/141
green on the fault that most decides the project's thesis, exactly the shape of
`SPEC-014/FU-3` and `SPEC-015`'s reason for existing.

`SPEC-015` is the ordering precedent. It shipped the analytic-levels-and-
geometry oracle **after** `SPEC-014` shipped the code, and its verify then
found a size-gated fault that had passed 150/150 tests (`SPEC-015/FU-10`).
This spec inverts that order deliberately: the oracle for the develop path
lands **before** `SPEC-018`, so the warp does not repeat the pattern.

The dnglab `--srgb` reference render is **single-sourced** — `--srgb` and
`--raw-checksum` both come from rawler (`docs/oracle-contract.md`, "This oracle
is single-sourced"). For the developed layer that is a real limitation:
matching rawler's tone and rendering choices is not the same as being right.
`SPEC-020` codifies this limit rather than hiding it; the truly-independent
check (ColorChecker ΔE) is `PROJ-002`, and the intermediate analytic
mitigation (`dnglab makedng`) **cannot build a monochrome fixture** —
measured in `SPIKE-001` and recorded in `docs/oracle-contract.md`.

## Goal

Ship the develop-layer oracle: read dnglab's malformed `--srgb` output, score
it against a candidate render with SSIMULACRA2, pass at ≥ 85 and fail
below — with a tier-A red-proof that runs in CI without the corpus and
demonstrates the metric catches the faults `DEC-005` calibrated it to catch,
and errors loudly (not silently) if dnglab ever fixes its `P6`-over-`P5`
defect.

## The design decision this spec rests on

⚠ **The oracle lands before its subject.** `SPEC-015`'s precedent is the
reason. If this spec waits for `SPEC-018`, then `SPEC-018` designs against
its own success — a test that measures the render is written by the session
that built the render, and a fault the render has is a fault its test has
too. Landing the oracle first breaks that loop by construction: `SPEC-018`
then has a check it cannot rewrite.

**This forces one design choice: the red-proof MUST NOT need the develop
pipeline.** The oracle's proof-of-teeth cannot be "our develop output scores
≥ 85 against dnglab", because there is no develop output yet. It has to be
proof that the METRIC responds to the fault classes `DEC-005` measured,
on inputs the harness itself can produce. So the tier-A red-proof
**perturbs a synthetic reference and scores it against itself**, exercising
the same metric wiring the real render will later go through — the fault set
is the one `DEC-005` calibrated (1-pixel shift, gamma 1.05, missing radial
warp; the identical-inputs sanity check anchors the top).

The full-resolution recalibration `DEC-005` flags as a revisit condition
(**"the 85 figure should be re-confirmed at full resolution when a real
render exists"**) is **out of scope here** and belongs to the first
develop-emitting spec that ships (`SPEC-018` or `SPEC-019`). This spec
delivers the harness and its red-proof; the score against real developed
output is that later spec's acceptance.

## Inputs

- **`docs/oracle-contract.md`** — the three-layer contract; § "Layer 3 is not
  what this document originally said" for the `P6`-over-`P5` workaround and
  the length assertion; § "This oracle is single-sourced" for the limitation
  this spec must state, not hide.
- **`decisions/DEC-005-develop-oracle-mechanics.md`** — the pre-registered
  threshold (≥ 85), the falsifier (missing warp far below 85), and the
  calibration table (identical 100 / gamma 1.01 → 95.03 / gamma 1.05 → 88.51
  / 1-px shift → 62.96 / missing warp → −68.05, all at ¼ resolution).
- **`decisions/DEC-004-levels-verified-analytically.md`** — the reason
  levels are OUT of this oracle's scope (the metric is blind to them up to
  +256 = 50 %). Do not extend this spec to check levels.
- **`decisions/DEC-017-oracle-mutation-testing-independent-build.md`** —
  the red-proof mechanism precedent from `SPEC-013`, and the `SPEC-013/FU-1`
  warning it carries.
- **`spikes/done/SPIKE-001-*.md`** — the warp coefficients used to
  construct the missing-warp red-proof (§ "Q4 — how visible is
  WarpRectilinear", `f(r) = 0.999251 + −0.06137651r² + −0.0939155r⁴ +
  0.0558820r⁶`, pure radial, optical centre 0.5,0.5). The **numbers are
  measurements**, not the spec's authority — a build session that needs to
  regenerate the fault must read the file's `OpcodeList3` and take the
  coefficients from there, not from this citation.
- **`tests/develop_oracle.rs`** — currently `SPEC-015`'s file at this path;
  this spec adds tests to it under a `mod perceptual_oracle {}` sub-module
  or lands them at a **new** path (see `## Outputs`). Read the header of
  `SPEC-015`'s file for the conventions it established.
- **`tests/support/{corpus,md5,oracle,tiff,tools}.rs`** — the existing
  test-side helpers. Anything reusable (corpus root discovery, tool-shellout
  patterns, tier-B skip vocabulary) is here; do not re-invent them.
- **`~/PSeven/experiments/crustimg_redo_plus/crustyimg/src/quality/mod.rs`**
  — the sibling repo's SSIMULACRA2 integration. Reference for the crate
  version, API shape, and the `Rgb` conversion route (`compute_frame_
  ssimulacra2(reference_rgb, candidate_rgb)`, sRGB transfer + BT.709
  primaries). ⚠ **Do NOT copy the `::image` crate dependency.** Crustyimg
  uses `::image` to decode; irradiance's `library-not-application`
  constraint forbids it, even test-side, and PNM parsing is trivial enough
  to hand-roll (SPEC-013's `parse_raw_pixel_pgm` is the precedent).
- Corpus: any of the seven files at `IRRADIANCE_CORPUS_DIR`, for the
  tier-B end-to-end score against real dnglab output. The three decodable
  frames `SPEC-015` names (`L1021223`, `L1026016`, `L1000622`) are the
  minimum.

## Outputs

- **A new test file** — options in order of preference:
  1. **`tests/develop_oracle.rs`** extended with `mod perceptual {}` — the
     path `SPEC-015` already anchors, and the domain match (both files
     verify `develop_into`'s output) is real.
  2. **`tests/perceptual_oracle.rs`** — a new file. Choose this ONLY if the
     first option's file layout makes the two mods share fixture code, in
     which case a separate file is cleaner.
  Whichever is chosen, the tier-A tests **must run with
  `IRRADIANCE_CORPUS_DIR` unset** (see `AC7`). State the choice and reason
  in the handback; do not silently pick.
- **`tests/support/ssimulacra2.rs`** — the metric wrapper: `Rgb`
  construction from 16-bit grayscale (see `AC3` for the conversion
  contract), the `compute_frame_ssimulacra2` call, and typed errors that
  do not `unwrap()` (`no-panics-on-untrusted-input` applies to test paths
  too by convention here — the two shipped test-side supports both hold
  this line).
- **`tests/support/pnm.rs`** — the reader for dnglab's `--srgb` output,
  with the length assertion (see `AC1`) and the loud error path when a
  well-formed `P6` appears (see `AC2`).
- **`tests/support/perturb.rs`** — the synthetic-fault generators used
  by the tier-A red-proof: 1-pixel shift, gamma exponent, and the radial
  warp from `SPIKE-001`'s coefficients. These are how the red-proof runs
  without the develop pipeline.
- **`tests/oracle-fixtures/`** — a new subdirectory for the tier-A synthetic
  reference image (see `## Implementation Context` — a fixture design-time
  probe is required BEFORE build to pick an image with enough detail to
  make a 1-pixel shift visible and enough smoothness to leave gamma 1.05
  passing). Extend `tests/oracle-fixtures/` if it already exists.
- **Cargo.toml** — one new `[dev-dependencies]` entry: `ssimulacra2 =
  "0.5.1"` (see `AC9` for the licence check and the sanction path).
- **`docs/provenance-ledger.md`** — one row for SSIMULACRA2 (see `AC8`).
- **A `DEC-*`** for the dev-dep sanction (**`ssimulacra2 = "0.5.1"`
  admitted as a test-only dependency**), covering: why the crate rather
  than a hand-roll, why a shell-out to a CLI is not chosen, why `::image`
  is not admitted alongside it, the licence check, and the class of
  provenance. Required by `no-copyleft-dependencies`'s carve-out path
  (AGENTS.md §13 "Sanction a trivial dev-dep + its DEC in one build
  pass") — this is a build-cycle decision the design pass authorises.
- **A `DEC-*`** for the mono-16→sRGB-8 conversion if it makes a non-obvious
  choice (e.g. the transfer-function chain). Not required if the choice is
  "linear→sRGB gamma on the 16-bit sample, quantize to 8-bit"; required if
  the build finds a subtler correct answer.

## Acceptance Criteria

- [ ] **AC1 — `--srgb` reader parses dnglab's malformed output and asserts
      the payload length.** For any input where the header claims `P6 <w>
      <h> 65535` and the byte-count after the header equals **`w*h*2`
      exactly** (dnglab's monochrome bug — `DEC-005`), rewrite the header
      to `P5 <w> <h> 65535` and return a 16-bit grayscale image. **The
      length is compared before any conversion**; a mismatch — long OR
      short — returns a typed error naming both figures.
      **Tests:** `pnm_reader_accepts_dnglab_p6_with_p5_payload_length` and
      `pnm_reader_rejects_mismatched_payload_length` (tier A, synthetic
      fixtures).
- [ ] **AC2 — the reader errors LOUDLY on a well-formed `P6`
      (`DEC-005`'s length-assertion clause, tested).** If a future dnglab
      release emits a correct `P6` (payload length `w*h*3*2`), the harness
      **must NOT silently pass** — that would halve the image and score it
      against half the real reference. Instead it returns a typed
      "`--srgb` output shape changed, DEC-005 revisit condition met"
      error naming both lengths. Runs on a hand-built `P6` with three
      channels of payload.
      **Test:** `pnm_reader_errors_on_wellformed_p6_dec_005_revisit`
      (tier A, synthetic).
- [ ] **AC3 — 16-bit grayscale → `ssimulacra2::Rgb` conversion is
      deterministic and pinned.** The transfer chain: (i) take the P5
      payload (big-endian 16-bit samples), (ii) read each as `u16`
      little-endian **or** big-endian per the PNM spec (PNM is big-endian
      — this is the same trap SPEC-013 hit; assert it once with a
      known-value probe and document), (iii) normalise to `f32` in
      `[0.0, 1.0]` as `sample as f32 / 65535.0`, (iv) construct
      `Rgb::new(replicated_pixels, w, h, TransferCharacteristic::SRGB,
      ColorPrimaries::BT709)` with the same triple `[v, v, v]` for the
      three channels. Two identical inputs through this chain must
      produce `f64` outputs that are **bit-equal** across runs, and the
      score of an image against a copy of itself must be `≥ 99.9`.
      **Tests:** `identical_inputs_score_at_least_ninetynine_point_nine`
      and `sixteen_to_eight_conversion_is_deterministic` (tier A).
- [ ] **AC4 — the oracle catches a 1-pixel shift (`DEC-005` calibration
      row 4, `SPIKE-001` measured 62.96).** Score a synthetic reference
      against itself shifted by 1 pixel horizontally: the result must be
      **< 85**. ⚠ **Pre-registered here, not tuned after seeing the
      result:** the bound is 85, per `DEC-005`; a fixture that produces a
      score ≥ 85 for a 1-pixel shift is a **wrong fixture, not a wrong
      threshold** — see `## Implementation Context` for the fixture
      probe the build must run first.
      **Test:** `one_pixel_shift_scores_below_the_threshold` (tier A).
- [ ] **AC5 — the oracle catches a missing radial warp (`DEC-005`
      calibration row 5, `SPIKE-001` measured −68.05).** Score a
      synthetic reference against the same reference with `SPIKE-001`'s
      coefficients applied — pure radial, `f(r) = 0.999251 + −0.06137651
      r² + −0.0939155 r⁴ + 0.0558820 r⁶`, optical centre `(0.5, 0.5)`,
      corner half-diagonal-normalised. The result must be **< 85**.
      **Test:** `missing_warp_scores_below_the_threshold` (tier A).
- [ ] **AC6 — the oracle ADMITS gamma 1.05 (`DEC-005` calibration row 3,
      `SPIKE-001` measured 88.51). The bar is passable.** Score a
      synthetic reference against `reference ^ 1.05` (per-pixel gamma in
      the linear domain, before sRGB): the result must be **≥ 85**. This
      is the falsifier's mirror — a threshold so tight that legitimate
      tone-curve implementation divergence fails would train us to
      ignore the oracle.
      **Test:** `gamma_one_point_zero_five_scores_at_or_above_the_threshold`
      (tier A).
- [ ] **AC7 — all red-proofs run with `IRRADIANCE_CORPUS_DIR` unset
      (`SPEC-015/FU-10`'s lesson, made an AC here).** `AC4` / `AC5` /
      `AC6` are tier A: the synthetic reference is either hand-built in
      the test or committed to `tests/oracle-fixtures/`, and no code
      path reads the corpus root. Verified by running the tier-A subset
      with no corpus and watching all four fault tests exercise the
      metric. **The precedent to avoid:** `SPEC-013/FU-1` is a red-proof
      CI has never once executed because it needs the corpus. Do not
      ship a fifth instance of that.
- [ ] **AC8 — provenance row for SSIMULACRA2 in
      `docs/provenance-ledger.md`.** Module: `tests/support/ssimulacra2.
      rs`. Source: rust-av/`ssimulacra2` v0.5.1, which itself
      implements the SSIMULACRA2 algorithm published by Cloudinary /
      Jyrki Alakuijala et al. Class: **2 — third-party permissive crate**
      (not class 1 because we did not implement from the paper; not
      class 3 because we are not reimplementing from another
      implementation). Notes: dev-only, no library-graph impact, and
      the crate's licence is confirmed permissive by the design-time
      probe (`## Implementation Context`).
- [ ] **AC9 — `ssimulacra2 = "0.5.1"` is admitted as a dev-only dep,
      with a `DEC-*` and both cargo-deny graphs green.** The crate:
      permissive-licensed (design-time probe confirms **BSD-3-Clause OR
      MIT**, see `## Implementation Context`), not a RAW decoder,
      genuinely trivial in the sense AGENTS.md §13 point 4 sanctions
      (one crate, one entry point, self-contained). The DEC states the
      alternatives considered (hand-roll, shell-out to `ssimulacra2`
      CLI, both rejected — the hand-roll is a ~2000-line project of its
      own and the shell-out introduces a runtime dependency worse than
      a dev-dep). `cargo deny check licenses` and `cargo deny
      --manifest-path fuzz/Cargo.toml check licenses` (both graphs, per
      the constraint's enforcement clause) must be green on the
      shipping SHA.
- [ ] **AC10 — no changes to `src/`.** `src/` is 0 lines changed against
      `main`. This spec adds test-side helpers, a dev-dep, and
      documentation; it does not change the library. If the harness
      surfaces a design gap in the library (e.g. `develop_into` cannot
      be called from the test in the shape needed), that is a **finding
      to report**, not a reason to edit — same rule as `SPEC-015/AC7`.
      Enforced by `git diff --stat src/` empty at ship.
- [ ] **AC11 — eleven gates + `just lint-ci`, CI observed green on the
      shipping SHA.** Not local, not the last CI run — the specific run
      that ships. `SPEC-015/FU-11` is open on the "eleven gates" naming
      ambiguity and does not affect the substantive rule: every job in
      `.github/workflows/ci.yml` on the ship SHA must be green, and
      `just lint-ci` (not `just lint`, per the CI/local toolchain
      divergence at `just lint-ci`'s comment) must pass locally
      immediately before push.

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum
across all — the trap `SPEC-015/AC9`'s Failing-Tests header calls out.

- `pnm_reader_accepts_dnglab_p6_with_p5_payload_length` — AC1, tier A
- `pnm_reader_rejects_mismatched_payload_length` — AC1, tier A
- `pnm_reader_errors_on_wellformed_p6_dec_005_revisit` — AC2, tier A
- `identical_inputs_score_at_least_ninetynine_point_nine` — AC3, tier A
- `sixteen_to_eight_conversion_is_deterministic` — AC3, tier A
- `one_pixel_shift_scores_below_the_threshold` — AC4 + AC7, tier A
- `missing_warp_scores_below_the_threshold` — AC5 + AC7, tier A
- `gamma_one_point_zero_five_scores_at_or_above_the_threshold` — AC6 +
  AC7, tier A
- (**optional tier B**, informational, does NOT gate ship) a smoke
  test that scores dnglab's `--srgb` output against **itself** through
  the full pipeline on one corpus file, asserting `≥ 99.9`. This exercises
  the reader end-to-end against the real defect shape rather than a
  hand-built one. Skip loudly when the corpus is absent; do not add a
  tier-B assertion that gates ship on a corpus CI cannot see —
  `SPEC-013/FU-1`'s hole.

## Non-Goals

- **Any change to `src/`.** See `AC10`. If the harness needs a call
  shape `develop_into` cannot serve, that is a finding for a follow-up
  spec, not this spec's remit.
- **Scoring against real developed output.** The develop pipeline does
  not exist yet — that is `SPEC-018` / `SPEC-019` / `SPEC-017`. This
  spec ships the oracle **and its red-proof**, so those specs land with
  a check they cannot rewrite; the score against real output is those
  specs' acceptance, not this one's.
- **Full-resolution recalibration of the ≥ 85 threshold.** `DEC-005`
  marks this as a revisit condition (**"the 85 figure should be
  re-confirmed at full resolution when a real render exists"**). It
  cannot happen until a real render exists, so it belongs to the first
  develop-emitting spec that ships. If that recalibration lands
  materially different, `DEC-005` is amended and this spec's `AC4` /
  `AC5` / `AC6` bounds are read from the amended DEC — but the amendment
  is not this spec's work.
- **Levels checks.** `DEC-004` and `SPEC-015` cover them. A wrong
  `BlackLevel` scores 95.62 through SSIMULACRA2 (`DEC-004`'s measured
  table); reopening that route is a new decision, not an AC here.
- **Colour-accuracy scoring (ColorChecker ΔE, patch-value comparison).**
  `PROJ-002` — it needs a colour camera the corpus does not hold.
- **The `--srgb` output for non-monochrome cameras.** The `P6`-over-`P5`
  defect is monochrome-specific per `SPIKE-001`. When a colour camera
  enters the corpus (`PROJ-002` / `SPIKE-002`), the reader either
  handles the true `P6` path or errors with a clear message that this
  spec's contract was monochrome-only; do not pre-generalise here.
- **Fuzz target.** This spec adds no library-side input surface. The
  PNM reader parses **test-side** synthetic inputs and dnglab's output
  in tests; a fuzz target on that path fuzzes dnglab's shape, not any
  parser irradiance ships. §12 bar 2 does not fire. State this in the
  handback rather than adding one.
- **`::image` crate.** Explicitly forbidden by `library-not-application`
  and by this spec's design intent. Do not admit it even in
  `[dev-dependencies]`.
- **A CLI shell-out to `ssimulacra2` (the libjxl reference binary).**
  Rejected in `AC9`'s DEC: the runtime dependency is worse than a
  dev-dep, and the crate is deterministic where the CLI is subject to
  build-time SIMD variability.

## Notes for the Implementer

- **The oracle contract already carries the P5 workaround one-liner** —
  `docs/oracle-contract.md` § "Layer 3 is not what this document
  originally said". `AC1`/`AC2` productionise it into a typed reader
  with the length assertion `DEC-005`'s Consequences call out; the
  shell version and the reader must agree byte-for-byte on the
  workaround.
- **`SPIKE-001` gives the warp coefficients as measurements, not as a
  citation to trust.** The file's `OpcodeList3` is the authority — if
  the numbers in `## Inputs` disagree with what a fresh parse of a
  Q2M file yields, the fresh parse wins. Re-verify per §16 rule 4
  (`unrun-docs-carry-errors`).
- **`SPEC-015`'s L1/L2/L3 vocabulary does not fit this spec.** That
  spec's oracle is analytic (properties + per-pixel bound + red-proof);
  this one is comparative (a metric wrapper + fault-response red-proof).
  Do not force the layer names on this spec's ACs; the shape is
  different.
- **`SPEC-013`'s `parse_raw_pixel_pgm` is the precedent for PNM parsing
  in test support** — a straight port with the header format changed
  from P5-sized-for-plane to P6-header-with-P5-payload keeps the shape
  the reviewer already trusts.
- **PNM samples are big-endian per the format spec.** `dnglab`'s output
  matches (`docs/oracle-contract.md` § "The plane contract" verified
  this on 2026-08-15 and 2026-08-16). Assert it with one known-value
  test rather than trusting the assumption silently.
- **The crustyimg reference (`src/quality/mod.rs`) uses `::image` to get
  `DynamicImage.to_rgb8()`; do not mimic that.** Convert the 16-bit
  grayscale samples directly to `Vec<[f32; 3]>` via `[v/65535.0,
  v/65535.0, v/65535.0]` and hand that to `Rgb::new`. That is one loop.
- **`just lint-ci`, not `just lint`.** Homebrew's clippy is 0.1.97; CI's
  is 0.1.98; the divergence has cost this repo 17 consecutive red runs
  once. Read CI's output on the ship SHA.
- **A tier-B test passes whether or not the corpus is present** — only
  `just test`'s own output names what is missing. AC7 is tested by
  running with the corpus **removed**, not by inference from a green
  run with it present.
- **Do not tune the fixture to make AC4/AC5/AC6 pass by number choice.**
  The pre-registered rule is: `DEC-005` says ≥ 85; a fixture that
  cannot demonstrate the deliberate faults dropping below 85 or the
  gamma admission passing 85 is the **wrong fixture, not a wrong
  threshold**. If the design-time probe below finds no fixture that
  satisfies all four invariants, that is a finding — stop and report.

## Implementation Context

> This section carries the design-time probes required before build
> can start (AGENTS.md §12 "Design-time probe / measure-before-build").
> Some are done here in the design pass; the fixture probe is deferred
> to the build cycle explicitly (with pre-registered pass/fail rules)
> because it needs the crate compiled and running.

### Confirmed in design

- **`ssimulacra2 = "0.5.1"` is the sibling repo's pinned version** —
  `~/PSeven/experiments/crustimg_redo_plus/crustyimg/Cargo.toml:75`
  and the same version resolves in its `Cargo.lock`
  (`checksum = "700cb4e17f98c3f36756815b18dbec2b2e3c4f5028e9cdee3e23dd8c285e736c"`).
  Dependencies: `num-traits` only. API used:
  `compute_frame_ssimulacra2(reference_rgb, candidate_rgb) -> Result<f64, _>`
  taking `Rgb::new(data: Vec<[f32; 3]>, w: usize, h: usize,
  TransferCharacteristic::SRGB, ColorPrimaries::BT709)`. Reference call
  site: `~/.../crustyimg/src/quality/mod.rs:109`.
- **The `--srgb` output shape holds on the file this project targets.**
  `docs/oracle-contract.md` § "Layer 3" was verified 2026-08-18 and its
  reproduction step (`{ printf 'P5 8368 5584 65535\n'; dnglab analyze
  --srgb F.DNG | tail -c +20; } > ref.pgm`) is the reference for the
  reader.
- **`SPIKE-001`'s warp coefficients are the authority for the missing-
  warp fault** — read straight from `OpcodeList3` on `L1021223.DNG` /
  `L1026016.DNG`. Radial-only, four terms, optical centre `(0.5, 0.5)`.

### Required before build handoff

- **Licence of the `ssimulacra2` crate MUST be re-confirmed on the
  pinned tree.** The design-time expectation is BSD-3-Clause OR MIT
  (rust-av's convention). The build handoff includes running
  `cargo deny check licenses` in a probe workspace with
  `ssimulacra2 = "0.5.1"` added, and if the report shows anything
  other than BSD/MIT/Apache, **stop and ask** — a GPL/LGPL discovery
  here is a `no-copyleft-dependencies` blocker, not a routine
  finding. Record the licence expression in the DEC.
- **Confirm crustyimg's `Cargo.lock` reflects a permissive graph** —
  `cargo tree` on the probe workspace, look for `unlicense`, `gpl`,
  `lgpl`, `agpl` strings in the resolved licence expressions.

### Required in build (pre-registered)

- **Choose the tier-A synthetic reference image.** Pre-registered
  rule: the fixture must satisfy ALL of
  `AC3` (identical ≥ 99.9), `AC4` (1-px shift < 85), `AC5` (missing
  warp < 85), `AC6` (gamma 1.05 ≥ 85). If no fixture satisfies all
  four, `stop and report` (the safe-direction failure is: report,
  do not shift the threshold). Two candidate shapes to start:
  1. A downscaled natural photograph (~1024×768) with mid-frequency
     detail (foliage, brick, hair).
  2. A synthetic gradient + textured overlay (Perlin noise on a
     Gaussian smooth field) at the same size.
  The first is more like the real develop output SPEC-018/019 will
  produce; the second reproduces without a source-attribution
  question. Prefer 1 if a licence-clean image exists; use 2
  otherwise.
- **Measure the actual scores on the chosen fixture and record them
  in the handback**, alongside the pass/fail against each AC's
  threshold. These become the numeric anchor a future reviewer can
  reproduce.

### Not measured, and why

- **The full-resolution ≥ 85 threshold.** Requires a real develop
  render. Out of scope for this spec by `DEC-005`'s design; belongs
  to `SPEC-018` / `SPEC-019`'s ship.
- **The perceptual response to a WrongTone that is not gamma-shaped.**
  `DEC-005` calibrated a gamma family (an affine tone change); the
  real Q2M tone curve is a table (`SPEC-019`). If `SPEC-019` finds a
  legitimate implementation divergence that gamma does not model,
  that is `SPEC-019`'s calibration to run — not a hole this spec
  patches. Do not pre-generalise.

## Follow-ups

| id | finding | disposition |
|---|---|---|
| `FU-1` | Build's design-time citation named the WarpRectilinear coefficient set as "read straight from OpcodeList3 on `L1021223.DNG` / `L1026016.DNG`" and `tests/support/perturb.rs:78–82` presents them as camera constants. Verify parsed OpcodeList3 out of all three decodable frames and measured: `kr0` is the only constant (`0.9992511060`); `kr1` varies **~1.9×** across the three frames (`−0.0613765129` / `−0.0418451693` / `−0.0323314533`). The `perturb.rs` constants match `L1021223.DNG` exactly to `f32` — so the code is right and AC5 is valid — but the CITATION is wrong and SPEC-018's own Context table (lines 120–123) carries the same error. | `signal: unrun-docs-carry-errors` (raises **N=5 → 6**; same family as instance 2, same file L1026016 that misled that instance) **AND** patch SPEC-018 `## Context` to name the coefficients as per-frame and to require AC1 to read them from each file's own OpcodeList3 — landed in this ship's commit. |
| `FU-2` | Build's own `FU-2` proposed `closed:` on "a future re-drift will be caught by re-measuring" — but the build itself named the trigger as **"the habit of re-measuring, which this build followed and the next one must too."** §15's bar for a good close is "a test that will fail", not "someone remembering". By its own honest description, the build's close does not meet the bar. | `signal: unrun-docs-carry-errors` (added as evidence — same signal, same root: a design-time citation of a fact the design session did not run the tool for). Not a new signal — instance 6 above already carries the codified case; this row records the analogous close-shape trap the build fell into for one of the three FUs. |
| `FU-3` | Test `sixteen_to_eight_conversion_is_deterministic` names a 16→8-bit conversion the pinned chain never performs (the chain is 16-bit → `f32` via `sample as f32 / 65535.0`). Cosmetic but permanently misleading on AC3's intent. | `closed: name was inherited from the spec's own ## Failing Tests list (which was inherited from crustyimg's 8-bit source case), the build kept it exactly per §12's zero-match-cargo-test warning, and the test does exercise the actual determinism the AC needs. Renaming both entries would be a paired rename in a shipped code area — out of scope for ship. Evidence noted here for the next spec that touches the perceptual oracle helpers. Not routed to a signal: N=1, not a recurring pattern.` |

---

## Reflection

1. **What would I do differently next time?**
   — Read each decodable corpus file's own opcode bytes during design, not just one, and treat any coefficient set that varies across frames as per-frame in the citation. FU-1 is instance 6 of a signal this repo has known about for weeks; the "cite one file's coefficients as if a camera constant" mistake is the exact shape of instance 2, on the same camera, and the fix pattern is codified — I should have applied it during design instead of finding it at verify.

2. **Does any template, constraint, or decision need updating?**
   — `guidance/signals.yaml`'s `unrun-docs-carry-errors` gains instance 6 and its N-count bumps 5 → 6 (recorded in this ship's commit). No template or constraint change: the signal is already codified, and the corrective habit ("run the reader on ALL corpus files, not one") already exists in §12's design-time probe discipline — the failure was not applying it, not a missing rule. If a seventh instance lands, that is a signal that even a codified rule can rot; consider a `just probe-corpus <tag>` recipe that reads one tag out of every corpus file, so a design citation can be validated by running the recipe, not by remembering to.

3. **Is there a follow-up spec I should write now before I forget?**
   — No. FU-1's actionable half — patching SPEC-018's `## Context` — is done in this ship. FU-2 is a re-disposition of an existing FU into the correct signal. FU-3 is closed with a contract reason. SPEC-018's build will read the corrected Context and read each frame's own OpcodeList3 for AC1's round-trip fixtures; SPEC-018's `depends_on: [SPEC-020]` puts it in the queue right behind this ship.

4. **Where was the worst defect caught?**
   — `verify`.
   *(FU-1 is the worst of the three, and it was caught by verify parsing all three files' OpcodeList3 directly instead of trusting the spec's citation. `design` would have been the right catch site — the discipline exists — but it was not applied. Not `escaped`: nothing user-visible ships with this defect because `src/` is untouched and the wrong citation only misled a future spec's designer, which SPEC-018's amended Context now prevents.)*

5. **What can a user do now that they couldn't before?**
   — Before: a develop-pipeline change (SPEC-017's FixBadPixelsConstant, SPEC-018's WarpRectilinear, SPEC-019's tone curve) could ship an image that looks plausible while being visibly wrong, because no oracle in this repo scored the output. After: `cargo test --all-features --test perceptual_oracle` runs a SSIMULACRA2-scored comparator against `dnglab analyze --srgb` on a synthetic fixture with four pre-registered fault scores (`identity 100.000`, `1-px shift 61.823`, `missing warp −82.338`, `gamma 1.05 90.038`), all measured to the digit against DEC-005's calibration; a missing warp or a wrong tone curve now scores far below 85, and STAGE-003's remaining specs have a check they cannot rewrite.
