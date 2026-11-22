---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-178
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-22
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-085

project:
  id: PROJ-002
  stage: STAGE-027
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 33190077           # REAL combined count — what cost-audit reads
  estimated_usd: 22.58             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: feat/spec-085-camera-calibration
  pr: https://github.com/jysf/irradiance/pull/306
  completed_at: 2026-11-22         # YYYY-MM-DD
  notes: "Floor, summed live from this session's transcript (1a2ce1b4), 120 messages deduplicated by message.id; in 247, cache creation 325,538, cache read 32,746,605, out 117,687. USD at Opus list rates $5/$25 per MTok with 1-hour cache writes; re-sum after the session for the final figure. Duration is the transcript's timestamp span."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-22
---

# HANDOFF-178: BUILD: apply CameraCalibration and AnalogBalance on the colour develop (SPEC-085)

## Delegation Summary

You are the **builder** for SPEC-085. **Start only after the SPEC-085 design PR has
merged.** Cut `feat/spec-085-camera-calibration` from that `origin/main`.

**The task:** implement DEC-076.
- Read five IFD0 tags into new `Sensor` fields: the two calibrations, `AnalogBalance`,
  and the two calibration signatures.
- Apply the calibration on all four colour entries under the signature rule, with the
  SDK's lerp of the folded products on the ColorMatrix path and `K(g)` on the
  ForwardMatrix path.
- Run the `NeutralToXY` weight loop to convergence.

Also:
- set `DEVELOP_VERSION` to 8;
- add the SDK colour lane (`scripts/sdk-colour.sh`, `scripts/sdk-camera-to-pcs.cpp`,
  `tests/colour_sdk.rs`) with AC4's pre-registered bound;
- re-pin only the colour pins DEC-076 moves, listing each one.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-085-apply-cameracalibration-and-analogbalance-on-the-colour-deve.md`
  (Design 1–8, AC1–AC7, and the Failing Tests clause map).
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-027-honour-or-refuse-what-we-parse.md`.
- **Decisions:**
  - DEC-076 (the rule, and the probe table with the expected numbers);
  - DEC-033 (the matrix pipeline, `f64` basic ops only, no FMA);
  - DEC-034 (the weight loop; DEC-076 §5 replaces its pass count);
  - DEC-012 (interpretation tags cost their field);
  - DEC-049 (`Sensor` is `#[non_exhaustive]`, `test_fixture`);
  - DEC-069 §6 (the SDK is local only: never vendored, built in CI or linked).
- **Code:**
  - `src/color.rs`: `build_camera_to_srgb_linear`, `cam_to_xyz_single_illuminant`,
    `cam_to_xyz_interpolated`, `cm_path_cam_to_xyz`, `interpolation_weight`,
    `wb_diagonal`;
  - `src/ifd.rs`: the colour `ifd0_rationals` reads, `Sensor`, `Sensor::test_fixture`;
  - `src/bin/irr.rs`: `ifd` prints the colour fields;
  - `tests/color_oracle.rs` (the independent reference), `tests/colour_metadata_oracle.rs`,
    `tests/gain_map_corpus.rs` (colour pins), `scripts/sdk-stage2.sh` (the SDK-recipe
    conventions to copy).
- **The SDK:**
  - Objects: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/obj`
    (link all except `dng_validate.o`) plus `../jxl_stubs.o`.
  - Headers: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/src/dng_sdk_1_7_1/dng_sdk/source`.
  - `dng_color_spec.cpp` is the class-4 reference for DEC-076 §2, §4 and §5. Read it;
    copy nothing.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  - `just test` already passes `--all-features`.
  - Give every red-proof copy its own `CARGO_TARGET_DIR`.

### The design probe's harness

This is the source the probe ran. Commit it as `scripts/sdk-camera-to-pcs.cpp`; it is this
repo's own code. Output lines are `white_xy`, `neutral`, and three `cam2pcs` rows.

```cpp
// Local SDK reference (SPEC-085): print the SDK's camera->PCS (XYZ D50) matrix.
#include "dng_host.h"
#include "dng_info.h"
#include "dng_negative.h"
#include "dng_file_stream.h"
#include "dng_color_spec.h"
#include "dng_camera_profile.h"
#include "dng_xy_coord.h"
#include <cstdio>
int main(int argc, char **argv) {
  if (argc < 2) return 2;
  try {
    dng_file_stream stream(argv[1]);
    dng_host host;
    dng_info info;
    info.Parse(host, stream);
    info.PostParse(host);
    if (!info.IsValidDNG()) { fprintf(stderr, "not valid\n"); return 1; }
    AutoPtr<dng_negative> neg(host.Make_dng_negative());
    neg->Parse(host, stream, info);
    neg->PostParse(host, stream, info);
    AutoPtr<dng_color_spec> spec(neg->MakeColorSpec(dng_camera_profile_id()));
    dng_xy_coord w = spec->NeutralToXY(neg->CameraNeutral());
    spec->SetWhiteXY(w);
    const dng_matrix &m = spec->CameraToPCS();
    printf("white_xy %.12f %.12f\n", w.x, w.y);
    const dng_vector &n = neg->CameraNeutral();
    printf("neutral %.12f %.12f %.12f\n", n[0], n[1], n[2]);
    for (uint32 r = 0; r < 3; r++) printf("cam2pcs %.12f %.12f %.12f\n", m[r][0], m[r][1], m[r][2]);
  } catch (...) { fprintf(stderr, "exception\n"); return 1; }
  return 0;
}
```

Compile with the flags from `tools/dng_sdk/build/build.sh` (`-std=c++17 -O2 -w -DqMacOS=1
-DqDNGValidateTarget=1 -DqDNGUseXMP=0 -DqDNGXMPFiles=0 -DqDNGXMPDocOps=0
-DqDNGUseLibJPEG=0 -I<headers> -I/opt/homebrew/include`). Link with the objects,
`jxl_stubs.o`, `-L/opt/homebrew/lib -ljxl -ljxl_threads -lz -framework CoreFoundation
-framework CoreServices`.

The probe's SDK matrices for the held files (2026-11-22) are your check that the script
reproduces them:

| File | white xy | `cam2pcs` row 0 |
|---|---|---|
| Pixel 7 Pro | 0.319745, 0.333331 | 0.689885 0.377995 0.255700 |
| Pixel 3a | 0.359539, 0.373618 | 1.113312 0.342809 0.189096 |
| Pixel 4a | 0.339264, 0.363595 | 0.837417 0.426838 0.255538 |
| Canon 5D Mark III | 0.397340, 0.400342 | 1.281433 0.039740 0.319677 |
| Nikon D300S | 0.345475, 0.352773 | 1.212220 0.170003 0.024173 |

## Expected Deliverables

1. **The implementation and tests** for AC1–AC6, in a PR against `main` from
   `feat/spec-085-camera-calibration`, with CI green.
2. **The handback:**
   - the AC4 table: every file's max |Δ| and its 18 deltas, plus today's values for
     contrast;
   - the re-pin list (file, entry, old → new hash, and which cause moved it: the
     calibration, the loop, or both);
   - the AC6 mutant table;
   - the converged pass count per held file.
3. **Gates**, all green: AC7.

## Out of Scope

- SPEC-086's refusals and SPEC-087's malformed-tag refusals.
- Triple-illuminant profiles.
- Robertson CCT, `NormalizeForwardMatrix`, the SDK's Bradford clipping.
- `ProfileHueSatMap`, `ProfileLookTable`, `ProfileToneCurve`, `BaselineExposure`.
- The mono develop.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus or SDK bytes, the harness binary, or SDK output.
- **An existing assertion may be strengthened, never weakened or deleted.**
  - Pre-approved: `tests/color_oracle.rs`'s dual-illuminant expected values move with
    the reference's new loop (SPEC-085 AC5), and the colour pins of the five held
    colour files.
  - Any mono pin that moves, or any other assertion: **stop and report**.
- **If a measured number lands outside a pre-registered bound,** stop and report it.
  Don't widen the bound.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`). Correct `to_agent` and
  `handback.agent` to the model that actually ran. Then fill the `## Handback` section.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent (claude-opus-5-5, build, 2026-11-22).*

### Execution notes

- **Branch / PR:** `feat/spec-085-camera-calibration` (cut from `origin/main` at `7b28fc7`), https://github.com/jysf/irradiance/pull/306
- **Completed at:** 2026-11-22
- **All acceptance criteria met?** Yes. Two assertions outside the pre-approved list
  had to change. I stopped and reported them, and the orchestrator approved both before
  the commit (see Drift).
- **What landed:**
  - `src/ifd.rs`: five IFD0 fields under DEC-012, plus the tag constants and an
    `ASCII`/`BYTE` signature reader.
  - `src/color.rs`:
    - `resolve_calibration` applies the signature rule and the shape checks;
    - `fold` skips the product when `AB · CC` is exactly the identity;
    - `fm_path_cam_to_xyz` is the ForwardMatrix path with `K(g)` and the reference
      neutral;
    - `scene_weight` runs `NeutralToXY` to convergence.
  - `src/bin/irr.rs` prints the new fields. `DEVELOP_VERSION` is `"8"`.
  - The SDK colour lane: `scripts/sdk-colour.sh`, `scripts/sdk-camera-to-pcs.cpp`
    (the harness exactly as given here) and `tests/colour_sdk.rs`. `corpus-status`
    reports whether the lane will run.
  - Also: the CHANGELOG, a provenance-ledger row and the module docs.

**AC4.** `tests/colour_sdk.rs` with `IRRADIANCE_SDK_COLOUR_DIR` set.
- Every number is ours minus the SDK, in u16 codes. Each file has 18 deltas: six
  probe colours (neutral first), each R, G, B.
- "Today" is the same lane run against `origin/main`.
- The harness reproduced every white xy and `cam2pcs` row-0 value in this handoff's
  table, to 6 digits.

| File | Max \|Δ\| now | Today | Probe | The 18 deltas now |
|---|---|---|---|---|
| Pixel 7 Pro | **6** | 1161 | 6 | -5 1 2, -6 1 2, -4 2 3, -6 2 2, -6 1 3, -5 2 2 |
| Pixel 3a | **6** | 39 | 7 | -5 1 2, -6 1 1, -4 1 3, -5 2 3, -6 1 3, -4 1 2 |
| Pixel 4a | **1** | 23 | 1 | 0 0 0, 1 0 1, 0 0 -1, 0 0 0, 0 0 0, 0 0 0 |
| Canon 5D Mark III | **2** | 3 | 2 | 0 0 0, 1 -1 -1, 0 1 1, 0 0 -2, 0 0 1, 0 0 -2 |
| Nikon D300S | **5** | 5 | 5 | -4 1 3, -5 2 1, -5 1 3, -5 1 2, -5 1 2, -4 1 2 |

Today's 18 deltas, for contrast:
- **7 Pro:** -5 1 2, -1112 -12 229, 712 -13 -311, 967 71 220, -1161 -68 -161, 780 64 225
- **3a:** -5 1 2, -39 -1 -7, 19 3 13, 20 3 -1, -36 -1 5, 16 3 -3
- **4a:** 0 0 0, 21 8 22, -19 -4 -23, -6 -7 1, 10 9 4, -3 -6 3
- **Canon:** 0 0 0, 2 -2 -2, -2 2 3, 0 0 -3, 1 0 2, 0 -1 -3
- **Nikon:** unchanged from now.

All five files are inside the pre-registered 12 codes.

**Converged `NeutralToXY` pass counts:** Pixel 7 Pro 16, Pixel 3a 19, Pixel 4a 5,
Canon 4, Nikon 4. I measured these with an instrumented copy of the tree; they match
DEC-076.

**Re-pin list (AC5).**
- All six re-pins are in `tests/gain_map_corpus.rs` `PINNED`.
- Every mono pin on every held file held, byte for byte.
- The Canon and Nikon have no colour pins in the suite, so none moved.
- How each cause was attributed:
  - Mutant (a), calibration ignored with the loop converged, keeps the 3a's new pins
    and moves the 7 Pro's and 4a's.
  - Mutant (b), the 3-pass loop with the calibration, reproduces the 3a's old pins
    exactly and moves the 7 Pro's and 4a's.

| File | Entry | Old → new MD5 | Cause |
|---|---|---|---|
| Pixel 4a | `develop_color_into` | `092b1ca0…` → `a5eabb15…` | both |
| Pixel 4a | `develop_color_f32_into` | `3a5d78cd…` → `caefcf53…` | both |
| Pixel 3a | `develop_color_into` | `6b4a538d…` → `037b970e…` | the loop |
| Pixel 3a | `develop_color_f32_into` | `13501540…` → `6aa5c7cc…` | the loop |
| Pixel 7 Pro | `develop_color_into` | `7d793677…` → `ce179c21…` | both |
| Pixel 7 Pro | `develop_color_f32_into` | `0b6b60ff…` → `2a4845ae…` | both |

**Other assertions that moved:**
- **Pre-approved:** the three dual fixtures in `tests/color_oracle.rs` (FM interpolation,
  CM interpolation, the honest dual control) went red against the new reference and
  green against the new code. Their assertions are unchanged; only the reference moved.
- The four `DEVELOP_VERSION` pins now read `"8"`.
- The two `src/color.rs` unit tests under Drift were approved by the orchestrator.

**AC6 mutants.** The control (the unmutated copy) passes first. Each mutant ran in its
own copy of the tree with its own `CARGO_TARGET_DIR`.

| Mutant | Fails | By | AC4 (probe) |
|---|---|---|---|
| (a) calibration ignored | `ac2a_…` | the bit-exact reference assertion | 7 Pro **1161** (1160); 4a 23 |
| (b) the 3-pass loop | `ac2f_…` | "the API develops at the converged weight" | 7 Pro **148** (148), 3a **39** (39) |
| (c) lerp of the factors | `ac2d_…` | the bit-exact reference assertion | 7 Pro **23** (23) |
| (d) signature rule always applies | `ac3_mismatched_signatures_…` | the `("a", "b")` row, "the calibration must not apply" | — |
| (e) `AnalogBalance` dropped | `ac2c_…` | the bit-exact reference assertion on the CM row | — |
| (f) FM path as `FM(g) · D` | `ac2b_…` | the bit-exact reference assertion | — |
| (g) `CC_2` checked on the single path | `ac3_an_unused_ill_shaped_…` | the single-illuminant `CC_2` row, which is refused instead of developing | — |

**AC7 gates.**
- `just test` with the corpus: green, both without and with `IRRADIANCE_SDK_COLOUR_DIR`.
- Green: `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`,
  `cost-audit`, `gate-audit` (14 scripts) and `name-length-check`.
- `semver-check`: "no semver update required" (additive).
- Fuzz: 30 s each of `ifd`, `plane`, `develop`, `demosaic`, `opcode` and
  `warp_opcode`, all clean. No fuzz target drives the colour develop; `ifd` parses the
  new tags.
- CI: see the PR.

### Cost self-report

- **Tokens (total):** 33,190,077 (a floor, summed while the session was live). Re-summed from this session's own
  transcript, deduplicated by `message.id`, taking each message's largest
  `output_tokens`. There were no subagents.
- **Estimated USD:** 22.58, priced per component.
- **Duration (minutes):** 25, the transcript's timestamp span.
- **Source of the number:** the transcript (`basis: transcript`).

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec (both approved by the orchestrator at the build, before the
  commit):**
  - `src/color.rs` `interpolation_weight_matches_dng_neutral_to_xy`:
    - the weight is re-pinned from 0.5202685520213226 to the converged
      0.6708871184066116, from a convergent reference loop;
    - the 3-pass value stays asserted as the contrast;
    - this follows from DEC-076 §5, and nothing is weakened.
  - `src/color.rs` `degenerate_interpolation_inputs_do_not_panic` (b) is a
    **behaviour change**:
    - a dual-illuminant file with a singular `ColorMatrix1` (and a valid
      `ColorMatrix2`, no ForwardMatrix) used to refuse with `MissingColorCalibration`;
    - the converged loop starts at D50, as the SDK's does, settles at g = 1 and never
      inverts `ColorMatrix1`, so the file now develops on `ColorMatrix2`;
    - the case now asserts `Ok`, g exactly 1.0 and finite outputs;
    - it is recorded in the CHANGELOG;
    - no up-front refusal was added, because DEC-076 has no such rule.
  - AC1's writer fixtures are unit tests in `src/ifd.rs`, beside the existing colour-tag
    fixtures and their builder, not in `tests/ifd_reader.rs`.
  - AC2 (c)'s ForwardMatrix row uses the dual fixture. On a single-illuminant FM file a
    diagonal `K = AB` equals `FM · D` up to rounding: the FM-path trap again.
- **Follow-up work identified:**
  - Whether a singular `ColorMatrix_i` should refuse even when unused belongs to
    SPEC-087 (malformed interpretation tags).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   Two things:
   - The pre-approved list named only `tests/color_oracle.rs`. The same 3-pass weight
     is also pinned in `src/color.rs`'s unit tests, so the stop came from a pin of the
     same quantity.
   - Neither the spec nor the handoff said that `AnalogBalance` alone cannot be
     observed on a single-illuminant FM file.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   Moving the loop's starting point from g = 0 to D50 changes which matrix is inverted
   first. DEC-076 could have said that a singular `ColorMatrix1` no longer guarantees a
   refusal.

3. **If you did this task again, what would you do differently?**
   Grep every pin of the weight and every degenerate-input case before writing code, so
   the stop-and-report lands at the start instead of after the build.

### Verify follow-ups (after HANDOFF-179 approved)

HANDOFF-179 (the cold verify, `face7e4`) approved PR 306 with four follow-ups. All four
landed in `89b752a`. Only tests and the CHANGELOG changed: no `src/` behaviour, no bound
widened, nothing re-pinned.

- **FU-1, the tolerance stop.**
  - `ac2f_the_loop_stops_by_tolerance_well_before_30_passes` runs on the Nikon D300S
    dual-illuminant tags with their own `AsShotNeutral`, over a 64×64 frame.
  - The reference stops by tolerance at pass 9. The test pins xy (0.3530156291392405,
    0.34551698155535704) and g 0.6954583018786118, and asserts that the API develops at
    that weight.
  - It also asserts that a tolerance 10× looser or 10× tighter stops on a different pass
    and develops to different codes.
  - The old test still pins the pass-30 midpoint. It is renamed
    `ac2f_the_loop_ends_on_the_midpoint_…` and now asserts 30 passes, ended not by
    tolerance.
  - The reference loop became `ref_neutral_to_xy(sensor, tolerance)`.
- **FU-2, ForwardMatrix identity bit-identity.**
  - `ac3_an_identity_calibration_keeps_the_rational_white_balance_on_the_fm_path` uses an
    `AsShotNeutral` red of 32263125/46358789. For that value `den/num` and `1/(num/den)`
    differ in the last bit.
  - The pixels (r, 0, 0) for r = 3, 9, 15, 21 were found by search to sit on a rounding
    boundary.
  - The test asserts that the reference can see the difference, and that the
    identity-calibrated API is byte-identical to the uncalibrated one and to DEC-033's
    `D` from the rationals.
- **FU-3, CHANGELOG wording.** The entry now says:
  - a singular `ColorMatrix` develops only when the converged weight never uses it;
  - it is still refused when the weight needs it, for example when `ColorMatrix1` is the
    higher-temperature illuminant;
  - a singular applied calibration whose product goes unused also develops now;
  - whether to refuse these files is left to SPEC-087.
- **FU-4, calibrated f32.** `a_calibrated_f32_develop_equals_the_reference_to_the_bit`
  compares the f32 entry, single-threaded and threaded, on both the ForwardMatrix and the
  ColorMatrix path, with non-diagonal `CC_1`/`CC_2` and a tilted `AB`. The expected value
  is `to_linear_f32_into` of the independent reference's `u16` develop, compared to the
  bit, which is the existing f32 tests' tolerance.

**Red-proofs.** Each mutant was applied to `src/color.rs`, run against `color_oracle`, then
reverted, with a green control run first.

| Mutant | Goes red on |
|---|---|
| (j) tolerance 10× looser | `ac2f_the_loop_stops_by_tolerance…` (Tier A now; before this, only the Tier-B pins) |
| (j2) tolerance 10× tighter | `ac2f_the_loop_stops_by_tolerance…` |
| (n) FM identity shortcut removed | `ac3_an_identity_calibration_keeps_the_rational_white_balance…` (it survived the whole suite before) |
| (l1) f32 entry on the uncalibrated transform | `a_calibrated_f32_develop_equals_the_reference…` and the AC3 refusal row |

**Gates.** All green:
- `just test` with the corpus (exit 0, 43 `color_oracle` tests);
- `lint-ci`, `lint-no-allow`, `fmt-check`.

**Extra cost.** 4,124,123 tokens: input 54, cache creation 120,179 (1-hour), cache read
3,972,071 and output 31,819. That is about $3.99 at opus-5-5 rates over about 9 minutes. It
is summed from this session's transcript after the follow-up request arrived, and is a
floor taken before this note's commit. Add it to the `tokens_total` above when re-summing
at ship.
