---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-018
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: critical               # critical | high | medium | low
                                   # ⚠ RAISED from the frame stub's `medium`.
                                   # SPIKE-001 measured ~504 px of corner
                                   # displacement — 6 % of image width. This is
                                   # the single item in PROJ-001 that most
                                   # decides whether the thesis holds: the
                                   # library today produces an image visibly
                                   # wrong by 6 % at the corners, and no
                                   # reference render will ever match. STAGE-003
                                   # marks it NOT DEFERRABLE for the same reason.
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
                                   #   L matches the frame. Two surfaces:
                                   #   (i) OpcodeList3 parser (new input surface,
                                   #   panic-free-on-untrusted-input, its own
                                   #   fuzz target — §12 bar 2 fires); (ii) the
                                   #   radial warp resampler and its integration
                                   #   into develop_into. Kept a single spec
                                   #   because the resampler is coupled to the
                                   #   opcode coefficients: splitting risks a
                                   #   parser that ships without a caller and
                                   #   the read-with-the-field failure mode
                                   #   (AGENTS.md §11).
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
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
  decisions: [DEC-002, DEC-004, DEC-005, DEC-011, DEC-016, DEC-018]
  constraints: [no-panics-on-untrusted-input, oracle-must-be-shown-red, provenance-recorded-per-algorithm, library-not-application, test-before-implementation]
  related_specs: [SPEC-003, SPEC-014, SPEC-015, SPEC-020]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-020]             # SPEC-018 is scored through SPEC-020's oracle
                                   # (AC10). Building SPEC-018 without SPEC-020
                                   # is designing the render against its own
                                   # success — the trap SPEC-015 was created to
                                   # break, applied here to STAGE-003.

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-003's <capability>". Optional; null is acceptable.
value_link: "closes the ~504 px corner error SPIKE-001 measured — the single
  item in PROJ-001 that most decides whether the thesis holds. Missing it, the
  library produces an image visibly wrong by 6 % at the corners and no
  reference render matches."

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
  tokens_estimate: 130000000
  # Basis: SPEC-014 (levels + geometry, expected L, shipped) landed at
  # ~88.8M; SPEC-015 (oracle, expected M, shipped L) landed at ~98.2M; both
  # attracted a punch-list round. SPEC-018 has TWO new surfaces (an opcode
  # parser AND a resampling kernel), a mandatory fuzz target (§12 bar 2 —
  # new input surface), integration into develop_into, and the resampling
  # kernel is a design choice with visible calibration cost. 130M assumes
  # build + verify + one punch-list round; the parser adds a fuzz seed
  # session too. If it lands near 90M the two surfaces overlapped more than
  # expected; near 180M and STAGE-003's remaining L (a hypothetical) should
  # be re-framed as a stage-of-its-own.
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-018: WarpRectilinear radial geometric correction

## Context

`SPIKE-001` parsed `OpcodeList3` out of **one Q2M file** (`L1021223.DNG`)
straight from bytes and measured **~504 px of inward displacement at the
corner — 6 % of image width**. Coefficients on **that one frame**
(`kr0..kr3`, optical centre `(0.5, 0.5)`, no tangential terms):

```
kr0  0.999251106            kt0 0.0            f(1)  = 0.899841
kr1 −0.06137651287726358    kt1 0.0            f(0.75) = 0.944957 → −207.7 px
kr2 −0.09391554139336016                       f(0.50) = 0.978910 →  −53.0 px
kr3  0.05588200921529175                       f(0.00) = 0.999251 →   −0.0 px
```

⚠ **The coefficients are per-frame, not a camera constant** — `SPEC-020`
verify (`SPEC-020/FU-1`, 2026-09-06) parsed OpcodeList3 out of all three
decodable Q2M frames and measured:

```
                kr0             kr1              kr2              kr3
L1021223  0.9992511060  −0.0613765129  −0.0939155414  0.0558820092
L1026016  0.9992511060  −0.0418451693  −0.1023114788  0.0578767192
L1026192  0.9992511060  −0.0323314533  −0.1042041102  0.0563642935
```

Only `kr0` is constant. `kr1` varies **~1.9×** across the three frames. A
build that hardcodes L1021223's set as *the* Q2M coefficients will be
wrong on the two other frames — the same failure mode `unrun-docs-carry-
errors` instance 2 hit on `Orientation` for L1026016. **`AC1` reads
`kr0..kr3` from each file's own OpcodeList3, and AC4/AC8's expected
displacements are per-frame.** `SPIKE-001`'s single-frame measurement
stays as evidence but is no longer the source of the tier-A hex fixture:
each frame's own hex fixture ships in `tests/oracle-fixtures/`.

`STAGE-003` records this as **NOT DEFERRABLE**: skipping the warp does not
produce a slightly-off image, it produces one that is visibly wrong by 6 %
at the corners, and no dnglab reference render would ever match. Consistent
with the Q-series 28 mm lens being designed around software correction. This
spec is the single item in `PROJ-001` that most directly decides whether the
project's thesis holds.

`OpcodeList3` runs after the plane has been normalised and cropped
(`SPEC-014`), and before tone-curve mapping (`SPEC-019`). Its geometric
correctness is the SPEC-020 oracle's headline case — DEC-005's falsifier is
literally **"a missing warp must land far below 85"**, measured −68.
Building this spec **without** SPEC-020 in place is the trap `SPEC-015`
existed to break, applied to STAGE-003; hence `depends_on: [SPEC-020]`.

## Goal

Parse `OpcodeList3` from the file, apply `WarpRectilinear` per DNG spec
1.7.0.0 § 6.4.1 to the normalised+cropped image, and land in a state where
the developed Q2M frames score **≥ 85** against `dnglab analyze --srgb`
through SPEC-020's oracle — on a decoder that never panics on an
attacker-controlled opcode stream, and with the resampling kernel choice
recorded in a `DEC-*` rather than smuggled in as an implementation
accident.

## The design decision this spec rests on

⚠ **The resampling kernel is a decision, not a default.** DNG § 6.4.1
specifies the *transform*, not the *kernel that samples it*. Bilinear is
cheapest and reproducibly deterministic; bicubic is closer to what real
raw converters emit; Lanczos is highest quality and the slowest. The
SPEC-020 oracle scores `≥ 85` against dnglab, so the kernel must produce
output close **enough** to dnglab's kernel for the oracle to pass — but
the goal is not "match dnglab bit-for-bit" (`docs/oracle-contract.md`
§ "This oracle is single-sourced": that would be matching rawler, not
being correct).

**Pre-registered rule for the kernel choice:** pick the simplest kernel
that lets the three decodable Q2M frames all score ≥ 85 through SPEC-020,
and record the alternatives considered in a `DEC-*`. If bilinear scores
≥ 85 on all three frames, ship bilinear — `DEC-002`'s determinism guidance
prefers the simpler operator. If it does not, upgrade one step and record
why in the DEC. Do NOT tune the score by parameter selection until the bar
is met; a kernel that needs tuning to pass is a wrong kernel.

**⚠ SPIKE-001's warp coefficients are measurements, not the spec's
authority.** The DNG-spec convention for `r`'s normalisation ("Assumes the
DNG convention that r normalizes to 1.0 at the corner. The pixel figures
should be confirmed against the spec before being quoted as exact" —
`SPIKE-001` § "Q4") must be resolved against DNG 1.7.0.0 during the
design-time probe below, not carried forward from the spike. If the spec
says half-width normalisation, the corner displacement figure changes and
so does every expected test value.

## Inputs

- **`spikes/done/SPIKE-001-*.md`** — the measured coefficients, the caveat
  above, and the ~504 px empirical figure that anchors the corner test.
- **The DNG 1.7.0.0 specification, § 6.4.1 "WarpRectilinear"** — the
  authoritative source for: parameter order, radius normalisation
  convention, the `f(r) = kr0 + kr1·r² + kr2·r⁴ + kr3·r⁶` polynomial's
  domain, and the treatment of pixels whose source coordinate lands
  outside the input extent. **Read this before the parser is written**;
  every other input to this spec is subordinate to it.
- **`docs/oracle-contract.md`** — the develop-layer contract (SPEC-020),
  and § "This oracle is single-sourced" for the reason we don't chase
  bit-identity with dnglab.
- **`decisions/DEC-002-target-surface-parallelism-and-determinism.md`** —
  no `rayon` on the algorithmic path, output determinism pinned within a
  `develop_version`. The warp resampler is on the algorithmic path.
- **`decisions/DEC-016-*.md`** — the caller-owned-buffer shape
  `develop_into` uses. `WarpRectilinear` writes into that same buffer;
  do NOT introduce a second allocator.
- **`decisions/DEC-018-*.md`** — the u16 full-scale representation with
  clamped levels. The warp reads and writes in that same representation.
- **`src/develop.rs`** — `develop_into` and `output_dimensions`. This
  spec's application point: after `Orientation`, before tone curve.
- **`src/ifd.rs`** — the `Sensor` type. `OpcodeList3` is an IFD tag
  (`0xC740`); the parser is a new field on `Sensor` or a sibling reader
  called during develop, and this design pass chooses which (see
  `## Implementation Context`).
- **`fuzz/fuzz_targets/`** — the existing `ifd`, `plane`, `develop`
  targets; `warp_opcode` is a **new** target seeded from real
  `OpcodeList3` bytes (see `## Outputs`).
- **A real Q2M file** (`IRRADIANCE_CORPUS_DIR/L1021223.DNG` and the
  other decodable frames) — for the tier-B end-to-end score. `SPEC-015`'s
  three-frame set is the minimum.

## Outputs

- **`src/opcode.rs`** — new module. The `OpcodeList3` byte-stream parser:
  reads the length prefix, iterates opcodes, extracts the WarpRectilinear
  parameter block for `OpcodeID == 1`, skips optional-and-unknown opcodes
  (per DNG spec), errors on mandatory-and-unknown. Panic-free (typed
  errors, bounds-checked slice reads — `no-panics-on-untrusted-input`).
  A small `pub` surface: one entry point that returns `Option<WarpRect>`
  from opcode bytes, plus its error type.
- **`src/warp.rs`** — new module. The resampler: given `WarpRect` and
  an input plane in `develop_into`'s u16 representation, writes the
  warped output into a caller-owned buffer (same shape as `develop_into`).
  Kernel chosen per `## The design decision this spec rests on`.
- **`src/develop.rs`** — modified. `develop_into` gains a warp application
  step **after** `Orientation` and **before** the tone curve entry point
  (which is `SPEC-019`'s work; for this spec, the tone-curve step is a
  no-op — the warped u16 buffer is the output). `output_dimensions` is
  **unchanged** (the warp is inward, so the output extent is still the
  DefaultCropSize per DNG § 6.4.1).
- **`src/ifd.rs`** — modified. `Sensor` gains an `opcode_list_3:
  Option<Vec<u8>>` field, populated from IFD tag `0xC740` at the
  same read path `SPEC-014`'s levels/geometry tags come through. The
  parse of these bytes into `Option<WarpRect>` lives in `src/opcode.rs`,
  not in `Sensor` — the "unread field" rule (AGENTS.md §11) is
  satisfied because `develop_into` calls the parser during develop.
- **`Cargo.toml` (root)** — no new dependencies. Bilinear/bicubic/Lanczos
  are all hand-implementable at the size this operation runs at (one
  plane per file, once per develop) and pulling `resize`/`image`
  crates violates `library-not-application`.
- **`fuzz/fuzz_targets/warp_opcode.rs`** — new fuzz target on the
  `OpcodeList3` byte-stream parser. Seeded from real Q2M files'
  `OpcodeList3` bytes plus 2–3 hand-truncated variants (matches
  `SPEC-003`'s seed shape).
- **`fuzz/Cargo.toml`** — one new `[[bin]]` entry for `warp_opcode`.
- **`app.just`** — one new recipe `fuzz-warp` following the shape of
  `fuzz-ifd` / `fuzz-plane` / `fuzz-develop` (the `+toolchain` trap
  applies here too — `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo
  +nightly fuzz run warp_opcode ...`), and one line in AGENTS.md §6's
  code block naming the new recipe (§6's rule 8: every recipe's commands
  appear in the block).
- **`tests/develop.rs`** and/or **`tests/warp.rs`** (new) — the failing
  tests below. State the choice; do not silently pick.
- **A `DEC-*`** for the resampling kernel choice, per `## The design
  decision this spec rests on`. Alternatives considered, why chosen,
  and the measured scores on the three decodable frames.
- **A `DEC-*`** for the `OpcodeList3` field shape on `Sensor` if the
  build finds a non-obvious choice — e.g. eager parse vs lazy parse
  during develop. Not required if the choice is "store the raw bytes on
  `Sensor`, parse inside `develop_into`", which is `develop`-side by
  construction and needs no decision record.
- **`docs/provenance-ledger.md`** — **two rows**: one for the
  `OpcodeList3` parser (class 1 — DNG 1.7.0.0 spec § 6.4.1), one for the
  WarpRectilinear application including the chosen kernel (class 1 for
  the transform, class 1 or 2 for the kernel depending on the source
  read for the resampling weights).

## Acceptance Criteria

- [ ] **AC1 — `OpcodeList3` parser reads real bytes.** Given the exact
      68-byte parameter block `SPIKE-001` measured on `L1021223.DNG`
      (`planes = 1`, `cx = 0.5, cy = 0.5`, `kr0..kr3` and `kt0..kt1 = 0.0`
      as listed in `## Context`), the parser returns a `WarpRect` whose
      coefficients round-trip to the same bytes when serialised. Byte-for-
      byte, with big-endian samples (DNG opcode-stream endianness).
      **Test:** `opcode_list_3_round_trips_the_q2m_warp` (tier A, from
      a hex fixture committed at `tests/oracle-fixtures/opcodelist3-
      L1021223.hex`).
- [ ] **AC2 — parser is panic-free on adversarial input
      (`no-panics-on-untrusted-input`).** No `unwrap` / `expect` / `panic`
      / indexing-that-can-fault on any parser path. Verified by (i) clippy
      lint policy already forbids it on the library (SPEC-006), and
      (ii) the fuzz target `warp_opcode` runs 60 s in CI-adjacent
      conditions with no crashes (**local** 60 s during design/build; the
      CI job is a smoke run per §12 bar 2). Seed corpus: the real bytes
      from AC1 plus at least three hand-truncated variants (`OpcodeList3`
      with the length prefix lying, with a zero-byte parameter block,
      with an unknown opcode id and the mandatory flag set).
      **Test:** `warp_opcode_fuzz_smoke_no_crashes_after_60s` (tier A,
      calls `cargo fuzz` behind `#[test]` guarded by a feature — or,
      simpler, runs the fuzz seeds through the parser directly and
      asserts no panic on any of them).
- [ ] **AC3 — parser skips optional-and-unknown opcodes, errors on
      mandatory-and-unknown (DNG § 6.4).** Two hand-built streams
      exercise the two paths.
      **Tests:** `opcode_parser_skips_optional_unknown` and
      `opcode_parser_rejects_mandatory_unknown` (tier A).
- [ ] **AC4 — WarpRectilinear application produces the SPIKE-001 corner
      displacement, within resampling tolerance.** On a **synthetic**
      input of the Q2M crop size (8368×5584) with a known impulse pattern
      at the corner (see `## Implementation Context` for the fixture),
      the warped output shows a peak at the source coordinate the
      polynomial predicts — within ± 1 pixel horizontally and vertically
      (the kernel's own resampling uncertainty). SPIKE-001's ~504 px
      corner displacement is the anchor.
      **Test:** `warp_moves_corner_impulse_to_spike_001s_source_coord`
      (tier A, synthetic; no corpus needed).
- [ ] **AC5 — WarpRectilinear is a NO-OP when coefficients would produce
      identity.** For `kr0 = 1.0, kr1 = kr2 = kr3 = 0.0, kt0 = kt1 = 0.0`,
      every output pixel equals the corresponding input pixel exactly
      (bit-equal in u16). The identity path is where a resampler most
      commonly hides a defect (a stride error that cancels itself on a
      symmetric input); the identity assertion catches it.
      **Test:** `identity_warp_is_the_identity_transform` (tier A).
- [ ] **AC6 — output extent equals `DefaultCropSize`.** Per DNG § 6.4.1
      the warp maps the input extent to itself; the shipped output
      dimensions are unchanged from `SPEC-014`'s `output_dimensions`.
      `git diff` on `output_dimensions` is empty. Tested implicitly by
      `AC7` (the score would fail on a size mismatch) and explicitly
      by an assertion that `output_dimensions` matches SPEC-014's
      documented values on the decodable frames.
      **Test:** `output_dimensions_are_unchanged_by_warp` (tier A).
- [ ] **AC7 — pixels whose source lands OUTSIDE the input extent are
      handled per the DNG spec.** DNG § 6.4.1's convention (clamp / zero /
      undefined) is read from the spec during design and pre-registered
      here — do NOT choose based on what scores best. If the spec says
      clamp-to-edge, clamp. If it says undefined, pick one and record in
      the kernel-choice DEC. The test asserts pixel values in the
      outer-corner ring the polynomial cannot reach match the chosen
      rule.
      **Test:** `outside_pixels_follow_the_dng_spec_rule` (tier A).
- [ ] **AC8 — `develop_into` output on all three decodable Q2M frames
      scores ≥ 85 through SPEC-020's oracle.** This is the primary
      acceptance — the SPIKE-001 falsifier ("a missing warp must land
      far below 85") is what the oracle is calibrated on, and it
      **must land at or above 85** on a correct warp. Report the
      **measured score per frame** in the handback. If any frame scores
      below 85, that is a **finding, not a threshold to relax** — same
      rule as SPEC-015/AC1.
      **Test:** `warp_scores_at_least_eightyfive_via_spec_020_oracle`
      (tier B — the corpus is required; skip loudly when absent per
      SPEC-002).
- [ ] **AC9 — the red-proof: mutating a coefficient turns the score
      below 85 (`oracle-must-be-shown-red`).** Applied via mutation of a
      TEST COPY (SPEC-013's precedent, DEC-017's mechanism): replace
      `kr1` with `0.0` in the parser output on one frame and rerun the
      score; the result MUST drop below 85. Actual scores at design
      unknown, but SPIKE-001's calibration puts the missing-warp full-
      subtraction at −68 — a partial subtraction (one term of four) can
      reasonably be expected to land ≤ 70 at ¼ res. Pre-registered rule:
      **the mutated score must be < 85** and the delta from the honest
      score must be recorded in the handback. Tier B (needs the real
      frame); the mutation is CODE-side, not opcode-bytes-side.
      **Test:** `warp_oracle_is_red_on_a_zeroed_kr1_coefficient` (tier B).
- [ ] **AC10 — the tier-A red-proof: mutating a coefficient turns
      `AC4`'s displacement test red WITHOUT the corpus** (SPEC-015/FU-10's
      lesson, made an AC). `AC9` needs the corpus and CI runs 0/7 corpus
      files; `AC4` runs on synthetic input, and its mutated form (`kr1 =
      0.0`) must produce a **different peak location** than the honest
      polynomial by at least 20 pixels on the synthetic-corner-impulse
      test. Runs with `IRRADIANCE_CORPUS_DIR` unset.
      **Test:** `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more`
      (tier A).
- [ ] **AC11 — the WarpRectilinear kernel choice is a `DEC-*`, with
      measured scores.** DEC states: alternatives considered (bilinear,
      bicubic, Lanczos), the chosen kernel, the measured score on each
      of the three decodable frames, and — if not bilinear — the
      measured shortfall bilinear had. The pre-registered rule (see
      `## The design decision this spec rests on`) is enforced by the
      DEC's content.
- [ ] **AC12 — no `rayon`, no runtime SIMD dispatch, output determinism
      pinned within `develop_version` (DEC-002).** The warp resampler
      runs single-threaded, with a compile-time-fixed kernel. Verified
      by (i) `Cargo.toml` has no `rayon`, (ii) the code contains no
      `is_x86_feature_detected` / runtime feature-dispatch call, and
      (iii) running the develop pipeline twice on the same input
      produces bit-identical output.
      **Test:** `warp_output_is_bit_identical_across_two_runs` (tier A).
- [ ] **AC13 — provenance rows for both new algorithms.**
      `docs/provenance-ledger.md` gains one row per module:
      `src/opcode.rs` (DNG 1.7.0.0 spec § 6.4.1, class 1) and
      `src/warp.rs` (transform: DNG 1.7.0.0 § 6.4.1, class 1; kernel:
      class 1 if implemented from a published paper or class 2 if from
      a permissive-crate reading). Honesty per §16 rule 3.
- [ ] **AC14 — eleven gates + `just lint-ci` + `just fuzz-warp`, CI
      observed green on the shipping SHA.** The `fuzz-warp` recipe is
      new; its 60 s smoke run must be added to CI in the same PR (per
      §12 bar 2: fuzz targets arrive with the parser, not retrofitted).

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum
across all.

- `opcode_list_3_round_trips_the_q2m_warp` — AC1, tier A
- `warp_opcode_fuzz_smoke_no_crashes_after_60s` — AC2, tier A (may run
  as an integration `#[test]` that shells to `cargo fuzz` — state the
  choice in the handback)
- `opcode_parser_skips_optional_unknown` — AC3, tier A
- `opcode_parser_rejects_mandatory_unknown` — AC3, tier A
- `warp_moves_corner_impulse_to_spike_001s_source_coord` — AC4, tier A
- `identity_warp_is_the_identity_transform` — AC5, tier A
- `output_dimensions_are_unchanged_by_warp` — AC6, tier A
- `outside_pixels_follow_the_dng_spec_rule` — AC7, tier A
- `warp_scores_at_least_eightyfive_via_spec_020_oracle` — AC8, tier B
- `warp_oracle_is_red_on_a_zeroed_kr1_coefficient` — AC9, tier B
- `warp_tier_a_red_proof_kr1_zeroed_moves_peak_20px_or_more` — AC10,
  tier A
- `warp_output_is_bit_identical_across_two_runs` — AC12, tier A

## Non-Goals

- **Any opcode other than WarpRectilinear.** `OpcodeList1`'s
  `FixBadPixelsConstant` is `SPEC-017`; `OpcodeList2` is empty on Q2M
  and not addressed here. Unknown opcodes are skipped/rejected per the
  DNG spec (AC3) — this spec does not implement them.
- **A general-purpose image resampler.** The kernel exists to serve
  `WarpRectilinear` on one plane at Q2M resolution. If SPEC-019 or a
  later spec needs resampling, it can share this module or grow its own;
  pre-generalising to N kernels or arbitrary-sized planes is not asked
  for here.
- **Tangential terms (`kt0`, `kt1`).** They are present in the DNG spec
  and the parser reads them (AC1), but every Q2M file measured is
  `kt0 = kt1 = 0.0`, so the applier can implement the pure-radial path
  and treat non-zero tangentials as a stop-and-report finding rather
  than a codepath. Note this in the module: a colour camera in PROJ-002
  may need tangentials, and that camera's spec adds them then.
- **Colour lens-shading (`OpcodeList3`'s `GainMap`).** Monochrome does
  not carry gain maps; PROJ-002 territory.
- **Bit-identity with dnglab's warp output.** Rejected in
  `docs/oracle-contract.md` § "This oracle is single-sourced": that
  target is matching rawler, not being correct. The oracle bar is
  perceptual (≥ 85), and the kernel choice DEC says why.
- **A crustyimg-side integration probe.** SPEC-020 handles the metric
  wiring; SPEC-018's job is the render. crustyimg's integration point
  is `STAGE-004`.
- **Recalibrating DEC-005's ≥ 85 threshold at full resolution.**
  DEC-005 marks this as a revisit condition; it happens **when**
  SPEC-018 lands, but it's not one of this spec's acceptance criteria —
  it's a follow-up if AC8 lands materially different from the ¼-res
  calibration.

## Notes for the Implementer

- **DNG § 6.4.1 first.** Every other input is subordinate. Read the
  spec's parameter order, radius normalisation, and out-of-extent rule
  before writing the parser or the applier. `SPIKE-001`'s
  parenthetical caveat about `r` normalisation is why.
- **The opcode stream is BIG-ENDIAN**, unlike the TIFF payload the
  reader already handles. That is easy to get wrong under a `SPEC-003`
  reader whose fluency is little-endian. Write one round-trip test
  first, on the SPIKE-001 hex fixture, and treat any parser change
  that alters its output as a defect.
- **Panic-free applies to the whole parser, not just the entry
  point.** Slice reads use `.get(range).ok_or(err)?`; `usize`
  conversions use `.try_into().map_err(err)?`; multiplication that
  could overflow uses `checked_mul` (parameter counts CAN be
  attacker-controlled). SPEC-003's `ifd.rs` is the precedent.
- **The resampler samples at `f64` and quantises to `u16` at the
  edge.** `DEC-018` is the analogue: the u16 boundary is where
  rounding is a decision. If bilinear is chosen, round-to-nearest
  (`(x + 0.5).floor() as u16` with clamp to `[0, 65535]`) matches
  `DEC-018`'s convention. If bicubic or Lanczos, negative overshoot
  can occur — clamp, do not truncate `u16` semantics.
- **The tier-B corpus tests skip loudly when the corpus is absent.**
  AC8's score assertion is on the real frames; AC10 exists because CI
  runs 0/7 corpus files and the tier-A red-proof is the only proof CI
  can see.
- **`just lint-ci`, not `just lint`.** Homebrew's clippy is 0.1.97; CI
  is 0.1.98 — the divergence has cost this repo 17 consecutive red
  runs.
- **The fuzz target's PATH= prefix is the `+toolchain` trap in a new
  suit.** `~/.cargo/bin/cargo +nightly fuzz run warp_opcode …` alone
  fails (the INNER `cargo build` inside `cargo fuzz` still resolves to
  Homebrew's stable). Match the shape of `just fuzz-ifd` in `app.just`.
- **If AC8 fails with the chosen kernel, do NOT change AC8 — change
  the kernel** (per `## The design decision this spec rests on`).
- **If any Q2M frame's warp coefficients differ from SPIKE-001's, that
  is a finding worth reporting.** The parser is right if the frames
  round-trip byte-for-byte; a coefficient set that would render
  meaningfully differently across the corpus is a real signal.

## Implementation Context

> This section carries the design-time probes required before build
> (AGENTS.md §12 "Design-time probe / measure-before-build"). The
> SPIKE-001 measurements are here as evidence with provenance; the DNG
> spec resolution is a **required probe** the build cycle must complete
> before it writes the parser.

### Confirmed in design

- **The warp coefficients** are as SPIKE-001 measured them on
  `L1021223.DNG` and `L1026016.DNG`. Radial-only, four terms, optical
  centre `(0.5, 0.5)`. Reproducible by rereading `OpcodeList3` at
  IFD tag `0xC740`, byte offsets per the DNG opcode format.
- **The failure mode of skipping the warp**: ≈504 px inward at the
  corner (~6 % image width) on `SPIKE-001`'s frame. SSIMULACRA2 score
  −68 against dnglab's `--srgb` at ¼ resolution (`DEC-005`
  calibration).
- **`OpcodeList3` runs after cropping and orientation and before tone
  curve** in the monochrome pipeline (colour transforms are absent).
  The output extent equals the DefaultCropSize — the warp is a
  spatial rearrangement of the same-sized image.
- **`no image crate`**: `library-not-application` is standing; the
  resampler is hand-written. Bilinear is ~20 lines including the
  clamp; bicubic is ~40; Lanczos-3 is ~60. Any of the three fits
  within the panic-free discipline.

### Required before build handoff

- **Resolve `r`'s normalisation against DNG 1.7.0.0 § 6.4.1.**
  SPIKE-001's caveat: **"Assumes the DNG convention that r normalizes
  to 1.0 at the corner. The pixel figures should be confirmed against
  the spec before being quoted as exact."** The build must read the
  spec and cite the exact clause in the DEC-*. If the convention is
  half-width, the corner displacement figure and every AC4 expected
  value derive from a different `r_max`.
- **Resolve the out-of-extent pixel rule against DNG 1.7.0.0 § 6.4.1.**
  Clamp / zero / undefined / spec-silent. Cite the clause; if
  spec-silent, choose and record in the kernel-choice DEC.
- **Confirm the WarpRectilinear opcode ID and version at the byte
  level from a fresh IFD parse** — SPIKE-001 said `OpcodeID == 1,
  version 1.4.0.0`, but re-parse to be sure, and record the exact
  bytes in the tier-A hex fixture.

### Required in build (pre-registered)

- **Kernel selection.** Bilinear first. Measure the SPEC-020 score on
  all three decodable frames. If all ≥ 85, ship bilinear. If not,
  bicubic next, remeasure. If bicubic doesn't reach it either,
  Lanczos-3 as the last option. Record the measured scores per kernel
  in the DEC — the alternatives-considered section is data, not prose.
- **Corner displacement fixture.** Design a synthetic input that
  makes AC4 diagnosable: a single-pixel impulse (or small Gaussian) at
  the corner of a large flat field, so the resampled peak's location
  is the whole content. The fixture is committed under
  `tests/oracle-fixtures/`; state its byte size in the handback (≤
  100 KB target).
- **Fuzz seed set for `warp_opcode`.** Real OpcodeList3 bytes from
  at least two Q2M frames, plus three hand-written truncations. Match
  `SPEC-003`'s seed shape; `just fuzz-seeds` regenerates the seeds
  from `examples/fuzz-seeds.rs`.
- **Measured full-develop wall-clock** (info only, reported in
  handback). If it exceeds ~10 s per frame in release, note it — the
  crustyimg consumer will notice.

### Not measured, and why

- **Full-resolution recalibration of the ≥ 85 threshold**: still a
  `DEC-005` follow-up, but this spec's landing produces the first
  real developed output. If AC8 lands materially different from the
  ¼-res calibration, that observation writes the recalibration; it is
  a `SPEC-018`-cycle follow-up, not an AC.
- **Perceptual score against a *second* independent reference render.**
  There is no second permissively-runnable reference for Q2M. PROJ-002's
  ColorChecker ΔE is the true independent check, and it needs a
  colour camera.

## Follow-ups

*Appended during **ship**. Every `FU-N` / `SB-N` raised across this
spec's cycles, with its disposition (§15). No follow-up crosses this
ship undecided.*

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
