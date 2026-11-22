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
  status: pending                  # pending | accepted | completed | rejected

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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
