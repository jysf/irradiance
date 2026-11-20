---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-059
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: punch-list  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [punch-list]  # every verify outcome, oldest first, appended by
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
  decisions: [DEC-069, DEC-049, DEC-053, DEC-058, DEC-060, DEC-061, DEC-068, DEC-040, DEC-012, DEC-009]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-060, SPEC-057, SPEC-075, SPEC-076]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-057]                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-016's <capability>". Optional; null is acceptable.
value_link: "Phone DNGs (Pixel 3a, 4a, 7 Pro) develop with their mandatory lens-shading GainMaps applied instead of being refused."

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
  tokens_estimate: 26000000
  sessions:
    - cycle: build
      handoff: HANDOFF-161
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 40668345
      estimated_usd: 26.93
      duration_minutes: 27
      recorded_at: 2026-11-19
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (133 unique message.ids, all claude-opus-5-5, no subagents): input 274 + cache-creation 361,567 (1-hour) + cache-read 40,177,598 + output 128,906 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Covers the build, the refusal-test question to the orchestrator, the eight-mutant red-proof and the handback commit. The handback floor was 38,230,872 and $25.38."
    - cycle: verify
      handoff: HANDOFF-163
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 18350336
      estimated_usd: 12.66
      duration_minutes: 24
      recorded_at: 2026-11-19
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (91 unique message.ids, all claude-opus-5-5, no subagents): input 181 + cache-creation 216,812 (1-hour) + cache-read 18,075,393 + output 57,950, opus-5-5 rates. PUNCH LIST, tests and docs only: the no-intersection mutant V24 pinned by a new colour test, stale colour-path docs and a DEC-069 consequence line, in 2e1a9c8, carried into PR 267 by fast-forward. 27 verifier mutants: 22 red, V24 closed, 4 equivalent. The handback floor was 16,261,949 and $11.32."
  totals:
    tokens_total: 59018681
    estimated_usd: 39.59
    session_count: 2
shipped_at: 2026-11-19
---

# SPEC-059: GainMap parse + apply on mono and colour paths with the analytic oracle and red-proof (writer fixtures), required-processing wiring, fuzz, provenance

## Context

STAGE-016 (PROJ-004). Phone DNGs are the most common DNG source.

- **The Pixel 3a, 4a and 7 Pro** in the held corpus each carry four **mandatory** `GainMap`
  opcodes (ID 9) in `OpcodeList2`. Both develop paths refuse them today with
  `UnsupportedMandatoryOpcode { id: 9 }`.
- **The design probe** (DEC-069 §Context) showed `GainMap` is their only blocker. With
  `OpcodeList2` disabled, all three develop `Ok` in colour and mono.
- **The rule is settled.** The probe matched DEC-069 §2's rule to Adobe's `dng_validate`
  within ±1 code on all three files, with 99.97% of samples exact.

DEC-069 is the contract. This spec builds the parser, the rule, the wiring into both
paths and the required-processing check, and an analytic oracle on writer-built fixtures.
The real-file evidence is SPEC-060.

## Goal

Parse and apply every `OpcodeList2` `GainMap` on the mono and colour develop paths,
exactly by DEC-069 §2, and accept those files in the required-processing check. Prove the
rule bit-exact against a hand-written oracle on writer fixtures, with a red-proof for each
part of the rule.

## Design

1. **Parser** (`src/opcode.rs`):
   - **The view.** `GainMap` becomes a recognised opcode, with a parsed view that borrows
     the parameter bytes and copies no gain grid. Its fields are:
     - `top`, `left`, `bottom`, `right`;
     - `plane`, `planes`, `row_pitch`, `col_pitch`;
     - `points_v`, `points_h`;
     - `spacing_v`, `spacing_h`, `origin_v`, `origin_h`;
     - `map_planes`;
     - an entry accessor that reads one `f32` at `76 + 4·((i·points_h + j)·map_planes + p)`.
   - **Malformed parameters.** Every check in DEC-069 §3 gives
     `MalformedOpcodeParams { id: 9, declared_size }`, before any use.
   - **The enum.** The public `Opcode` enum gains `GainMap(...)` if that fits its existing
     `#[non_exhaustive]` contract. The builder picks the public shape, recorded in the
     handback. The develop paths use the crate-internal view.
2. **The list type.** A crate-internal `GainMaps` holds up to 16 views in a fixed array,
   built from `OpcodeList2`'s bytes. A seventeenth map gives `Error::TooManyGainMaps
   { count }`.
   - It records the stage-2 size `H × W` (`ActiveArea`).
   - `apply(r, c, v) -> u16` runs every map that touches `(r, c)`, in list order. Each
     map rounds to `u16` (DEC-069 §2).
   - Gain is computed per sample in `f64`, with no allocation.
3. **Wiring:**
   - **Mono.** In `crop_orient_normalize_into` and `normalize_active_area_into`, the
     sample becomes `gain.apply(raw_y − active_top, raw_x − active_left, normalize(...))`.
     Resolve the gain maps once, next to `resolve_levels_with`, and branch once outside
     the loop, so a file with no `GainMap` monomorphises to today's loop.
     `crop_orient_normalize_into` must keep its `for_each_out_pixel(geometry.orientation,`
     call (SPEC-076's H5 source scan).
   - **Colour.** Wrap the resolved normaliser (`slot_normalizer` or `pattern_normalizer`)
     the same way, in each of the four entry points and in `develop_color_bands`.
     `normalize(x, y)` receives raw-plane coordinates, so subtract the `ActiveArea` origin.
     With no `GainMap`, the existing closure is used unchanged.
   - **Order.** The maps resolve **after** `check_required_processing` and the level
     resolution, and **before** any write or scratch use. A malformed or over-cap map is
     refused with `dst` untouched.
4. **Required processing.**
   - `DevelopPath::applied_id` can no longer express "every instance". Give `check_list`
     an applied rule per list: list 1 (mono) is the first `FixBadPixelsConstant`, list 2
     (both paths) is every `GainMap`, and list 3 (mono) is the first `WarpRectilinear`.
   - `GainMap` joins the recognised IDs. A mandatory `GainMap` in list 1 or 3 is
     `UnsupportedRequiredOpcode { list, id: 9 }`.
   - Update `DevelopPath`'s rustdoc and `check_required_processing`'s rustdoc.
   - Keep `__fuzz_required_processing`'s signature.
5. **Public applier.** Add
   `develop::apply_gain_maps(stage2: &mut [u16], width: u32, height: u32, opcode_list_2: &[u8]) -> Result<usize, Error>`.
   - It applies every `GainMap` in the list to an `ActiveArea`-sized normalised image,
     through the same `GainMaps` type, and returns the count applied.
   - Errors are a wrong buffer length (a new or existing typed variant, the builder's
     choice), §3's malformed errors and `TooManyGainMaps`.
   - Other opcodes in the list are ignored. It is a stage primitive like
     `apply_fix_bad_pixels_constant` (DEC-053), so it does not run the required-processing
     check.
6. **Version.** `DEVELOP_VERSION` goes to `"6"`. Grep for every pin with
   `DEVELOP_VERSION, "` (SPEC-075 found four, not three) and add a CHANGELOG entry.
7. **Provenance.** Add a `docs/provenance-ledger.md` row for the parser and the rule, as
   DEC-069 §Provenance describes: class 1 for the DNG `GainMap` layout and operation, and
   class 4 for the five points settled from Adobe DNG SDK 1.7.1, with no code copied.
   Record the licence terms of the SDK's licence agreement. Update the existing
   required-processing row (`src/opcode.rs`, SPEC-054): list 2's applied set is now every
   `GainMap`.
8. **Fuzz.**
   - Extend the existing `opcode` fuzz target (or add a `gain_map` one) with hostile
     `GainMap` parameters: a huge grid against a short block, zero pitch, zero points, NaN
     and infinite spacing, origin and gains, a negative spacing, and an area outside the
     image.
   - Run the required-processing target with `GainMap` IDs in all three lists.
   - Seeds go in `fuzz/seeds/`. Run each target for 60 seconds or more, with zero crashes.

## Inputs

- DEC-069 (all of it). DEC-049 and DEC-053 (required processing). DEC-058 §5 (version).
  DEC-060 (zero-allocation scratch). DEC-061 and DEC-068 (black levels). DEC-040
  (`ActiveArea` origin). DEC-012. DEC-009 (red-proof, control first).
- `src/opcode.rs`: the framing reader, `check_list`, `DevelopPath`, `parse_opcode_list`
  and its enum.
- `src/develop.rs`: `crop_orient_normalize_into`, `normalize_active_area_into`,
  `resolve_levels_with`, `develop_core`, `develop_with_scratch_into` and
  `apply_fix_bad_pixels_constant` (as the shape for the public applier).
- `src/color.rs`: `develop_color_into`, `develop_color_f32_into`, `develop_color_bands`,
  `slot_normalizer`, `pattern_normalizer` and `check_colour_required_processing`.
- `tests/support/dng_writer.rs`'s `extra_tags` (`RawTag`), which writes `OpcodeList2`
  (51009, UNDEFINED) with no writer change.
- `tests/required_processing.rs`, `tests/develop_scratch_alloc.rs`, `tests/opcode.rs`.

## Outputs

- **Modified:** `src/opcode.rs`, `src/develop.rs`, `src/color.rs`, `src/lib.rs` (the new
  error variant(s) and the version), `CHANGELOG.md`, `docs/provenance-ledger.md`, the
  version pins in `tests/`, and `tests/required_processing.rs` (new rows only).
- **New:** `tests/gain_map.rs` (the analytic oracle) and fuzz seeds.
- **New public API:** `develop::apply_gain_maps`, `Error::TooManyGainMaps { count }`,
  possibly a wrong-length error, and possibly `Opcode::GainMap`. All are additive.

## Acceptance Criteria

- [ ] **AC1, the parser.**
  - A writer-built `GainMap` parses to the exact fields written. The entry accessor
    returns each written gain.
  - Every DEC-069 §3 class gives `MalformedOpcodeParams { id: 9, .. }`:
    - the block one byte short;
    - the block one byte long;
    - `RowPitch` 0;
    - `ColPitch` 0;
    - `MapPointsV` 0;
    - `MapPlanes` 0;
    - `Planes` 0;
    - spacing 0;
    - spacing negative;
    - spacing NaN;
    - origin infinite;
    - a NaN gain.
- [ ] **AC2, the analytic oracle.** `tests/gain_map.rs` holds a hand-written oracle. It
  is a direct transcription of DEC-069 §2 written in the test and never calls the crate.
  `apply_gain_maps` must equal it **bit for bit** on writer-built maps covering:
  - a 2×3 grid with non-zero origin and spacing, so indices clamp on both sides and
    interpolate in between;
  - pitch 2 at each of the four phases;
  - an area intersected by the image edge, and an empty area (the whole image);
  - `Plane = 1`, which applies to nothing;
  - a 1×1 map (the Samsung's shape, a constant gain);
  - gains above 1 that clip at 65535, and a gain below 1;
  - two maps touching the same sample, rounded after each.
- [ ] **AC3, hand values.** At least four expected samples are written out as literal
  numbers in the test, each with its arithmetic in a comment: a corner clamp, an interior
  bilinear, a clip at white, and a pitch-skipped sample left unchanged.
- [ ] **AC4, mono wiring.** A CFA or LinearRaw fixture with a mandatory `GainMap` and an
  odd `ActiveArea` origin develops `Ok` through `develop_into`, `develop_with_into`
  (both framings) and `develop_with_scratch_into`.
  - Each output equals the oracle applied to the normalised stage 2, then cropped and
    oriented by the existing geometry.
  - The warp path (a non-identity `WarpRectilinear` in list 3) also equals the oracle
    gain followed by the existing warp.
  - `develop_with_scratch_into` with a `GainMap` still allocates nothing (a new row in
    `scratch_develop_allocates_nothing`).
- [ ] **AC5, colour wiring.** On all four colour entry points, a CFA fixture with a
  mandatory `GainMap` (four phase maps, pitch 2, odd origin) equals the same develop of a
  fixture whose raw plane was pre-gained so that its normalised values equal the oracle's
  gained stage 2.
  - If no exact pre-gained raw plane exists, use the builder's equivalent. For example,
    compare against the colour kernel driven by a test-only normaliser. Name it in the
    handback.
  - The threaded entries equal the single-thread ones.
- [ ] **AC6, required processing.**
  - Four mandatory `GainMap`s in list 2 pass on both paths.
  - A mandatory `GainMap` in list 1 or list 3 gives `UnsupportedRequiredOpcode
    { list, id: 9 }` on both paths.
  - An unknown mandatory ID in list 2 next to `GainMap`s still gives
    `UnsupportedMandatoryOpcode`.
  - 17 maps give `TooManyGainMaps { count: 17 }` from every develop entry, with `dst`
    sentinel-unchanged.
  - A malformed map is refused with `dst` unchanged, from every develop entry.
- [ ] **AC7, no collateral change.** Every held file other than the Pixels develops
  byte-identically. Every pinned hash and oracle passes with only the `DEVELOP_VERSION`
  pins edited.
- [ ] **AC8, red-proofs**, control first, each in its own `CARGO_TARGET_DIR`, with the
  failing assertion's message read. Each mutant must fail a named test in AC2 or AC3:
  - corner sampling instead of pixel centres (drop the `+ 0.5`);
  - indices not clamped;
  - columns and rows swapped in the entry index;
  - pitch phase anchored at 0 instead of `Top`/`Left`;
  - no clip at 1;
  - one rounding for the whole list instead of one per map;
  - the empty area treated as "nothing".
- [ ] **AC9, gates.**
  - `just test` with the corpus, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`;
  - `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`;
  - `name-length-check`, `semver-check`, `decisions-index --check`.

  All green, and CI green.

## Failing Tests

- `tests/gain_map.rs`: AC1, AC2, AC3, AC4 and AC5, written from DEC-069 §2 and the
  writer. The oracle is a separate transcription in the test file.
- `tests/required_processing.rs`: new AC6 rows.
- `tests/develop_scratch_alloc.rs`: a `GainMap` row for AC4's zero-allocation claim.

## Non-Goals

- Real-file evidence against the SDK, and the Pixel tier-B pins. Those are SPEC-060.
- The max-black normalisation difference (DEC-069 §Context). It is a carried follow-up.
- Any other `OpcodeList2` opcode, or `GainMap` in lists 1 and 3.
- `WarpRectilinear` on the colour path. The Pixels' warp is optional and stays skipped.
- Multi-plane (`Planes > 1`) images, and a release.

## Notes for the Implementer

- **Build the oracle first.** Do not write the oracle by calling `apply_gain_maps`, and
  do not share code with `src/`. A shared helper makes the oracle unable to go red.
- **The probe's numbers** are in DEC-069 §Context. SPEC-060 reruns them against the SDK.
  This spec has no SDK dependency.
- **A gain is `f32` in the file.** Widen it to `f64` before interpolating.
  `(r + 0.5) / H` uses `f64`.
- **Keep both paths' no-`GainMap` code paths identical** to today, so the colour hot loop
  gains nothing for files without a map. The colour goldens and `tests/color_oracle.rs`
  are the guard.
- **Plain-prose commit bodies**, with no code tokens. Never `git add Cargo.lock`.

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Grep the tests for every pin of the behaviour a spec removes. The design named one
   refusal test (`plane_tiles.rs`) for the builder to rewrite but missed a second
   (`develop_options.rs`'s Pixel loop), found only by the build's full corpus run, so it
   was rewritten without prior approval. The bundle cost about 59.0M tokens against a
   26M estimate, so it was sized L, not M. The colour path's three normaliser sites, the
   pre-gained-twin fixtures and eight red-proof copies (each with its own target dir)
   are most of the overrun.

2. **Does any template, constraint, or decision need updating?**
   — DEC-069 holds as written. The verify added one line to its Consequences naming
   `Error::GainMapImageWrongLength`, and no source changed after build. The
   `every-rule-clause-maps-to-a-reachable-test` signal fired again: the area
   intersection clause had a test only on paths where it is unobservable. A clause's
   test has to run on the path where the clause changes the output, here the colour
   demosaic that reads past the `ActiveArea`.

3. **Is there a follow-up spec I should write now before I forget?**
   — None new. SPEC-060 (Pixel pins plus the `dng_validate` cross-check) is framed and
   builds next. Still carried: the SDK's max-black normalisation difference (DEC-069
   §Context), and fetching the Adobe licence agreement's text for the provenance row.

4. **Where was the worst defect caught?**
   — verify
   *(A test gap only: verifier mutant V24, dropping the area's intersection with the
   image, survived the build's suite while the source stayed correct. Pinned in 2e1a9c8
   and carried into PR 267 before merge.)*

5. **What can a user do now that they couldn't before?**
   — A caller can develop the Pixel 3a, 4a and 7 Pro DNGs in colour and in mono, which
   were refused with `UnsupportedMandatoryOpcode { id: 9 }`. `OpcodeList2` `GainMap` is
   applied at stage 2 by DEC-069's rule, and composers get the public `apply_gain_maps`.
   Every file with no `GainMap` develops byte-identically; `DEVELOP_VERSION` is 6.
