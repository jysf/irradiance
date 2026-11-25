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
  id: HANDOFF-189
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-24
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-091

project:
  id: PROJ-006
  stage: STAGE-031
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
  tokens_total: 39591653           # REAL combined count — what cost-audit reads
  estimated_usd: 25.68             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: feat/spec-091-srgb-linear-plane
  pr: 343
  completed_at: 2026-11-25         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 385139bc-54fe-4f0e-b33c-b6249fa3fde8.jsonl (140 unique message.ids, all claude-opus-5-5, no subagents): input 285 + cache-creation 324,748 (1-hour) + cache-read 39,152,603 + output 114,017, priced at 5 and 25 USD per million. Re-sum at ship. CI run 45666936351 green on both determinism legs."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-189: The white-balanced linear RGB step at sensor geometry, with the working-space DEC and the arithmetic decision for the new path

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-091` to a cold `claude-opus-5`
session (acting as implementer) for the **`build`** cycle: four new public functions in
`src/color.rs` that write the white-balanced linear-sRGB plane at sensor geometry, `u16`
(today's values) and `f32` (unclamped), whole or as a row range (band), plus one
`Error` variant for a bad range, a cross-architecture determinism hash for the plane,
and no change to any existing output. The band form and the determinism pin were added
on 2026-11-25 from onyx's reply to the design (see the spec's Context).

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-091-the-white-balanced-linear-rgb-step-at-sensor-geometry-with-t.md` (Design §1 to §8, AC1 to AC13, Failing Tests, mutants (a) to (k))
- **Decision:** `./decisions/DEC-079-mixer-weights-live-in-linear-srgb-and-apply-folded-into-camera-space.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — this repo's test
  framework, lint quirks, runtime globals and gotchas. Read it; it exists so a
  cold agent doesn't rediscover them (DEC-004 rule 5).

### Decisions that apply

- `DEC-079` — the working space is linear sRGB; this spec's `f32` plane is unclamped because the probe showed the `u16` plane clamps (its table is the evidence for AC3)
- `DEC-033` — `f64` basic operations only, committed constants: the `u16` path must keep today's expression and order
- `DEC-018` — `u16` full scale, clamp before round half up (`quantize`)
- `DEC-016` — caller-owned buffers; nothing written unless every check passes
- `DEC-040` — CFA phase and level slot indexed from the ActiveArea origin (mutant f)
- `DEC-042` — the fused traversal and its single green plane: reuse it, do not copy it
- `DEC-053`, `DEC-078` — required processing and malformed tags refuse first, `DevelopPath::Colour`
- `DEC-069` — `GainMap`s at stage 2 (AC7)
- `DEC-002` — determinism; Design §7 and AC12 measure it for the plane on both CI legs
- `DEC-044` / `SPEC-051` — `develop_rows_banded`, the halo'd row-range develop the band forms reuse
- `DEC-051` — the new `InvalidRowRange` variant is classified in `Error::kind()` with no wildcard arm

### Constraints that apply

Check `./guidance/constraints.yaml` for full text. These constraints
apply to the paths touched by this task:

- `no-panics-on-untrusted-input` — deny-level lints; the new entries join the fuzz target (AC9)
- `oracle-must-be-shown-red` — AC10: apply each pre-registered mutant by hand, list which test failed
- `test-before-implementation` — write the Failing Tests first and see them fail
- `one-spec-per-pr` — this PR is SPEC-091 only
- `provenance-recorded-per-algorithm` — no new algorithm here (same pipeline); say so in the PR

### Prior related work

- `SPEC-049` / `SPEC-050` — the fused colour develop and its crop sink, which this spec generalises
- PR 324, PR 325 — PROJ-006 brief and STAGE-031 frame (onyx's answers are in the brief)
- `SPEC-041` — `tests/determinism.rs` and the CI determinism job (x86_64 Linux, arm64
  macOS) that AC12 extends; read `docs/determinism.md` first
- PROJ-005's writer specs have shipped and appended their `Error` variants; take the
  enum's tail from current `origin/main`. On a conflict in a shared file
  (`CHANGELOG.md`, `src/lib.rs`), keep both sides' entries.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- Code changes implementing SPEC-091's Acceptance Criteria.
- All failing tests in SPEC-091 now passing.
- Any new tests required to cover edge cases.
- A PR against `main` from branch `feat/spec-091-srgb-linear-plane`, cut from current
  `origin/main` (fetch first; check `gh pr list` for anything touching `src/color.rs`).
- PR description referencing: this handoff ID, the spec ID, the stage
  ID, the project ID, all referenced `DEC-*`, and any new `DEC-*`
  created during implementation.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- The mixer, the folded row and the fixed-point arithmetic (SPEC-092).
- Any developed or cropped output, `DevelopOptions` changes (SPEC-093).
- Threaded variants of the new entries (callers can run bands on their own threads).
- Any change to an existing function's output or existing determinism hash, any
  `Error` variant other than `InvalidRowRange`, or a `DEVELOP_VERSION` bump. If one
  seems necessary, stop and hand back blocked.

**Repo gotchas.** Run `just test`, not `just test --all-features` (the recipe already
passes it). Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
for the Tier-B tests. Commit-message bodies must be plain prose: a push whose message
contains code tokens (`#[...]`, `->`, `--flag`) is rejected by a ruleset; PR bodies and
files are fine.

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

- **Branch / PR:** `feat/spec-091-srgb-linear-plane`, https://github.com/jysf/irradiance/pull/343
- **Completed at:** 2026-11-25
- **All acceptance criteria met?** yes, with one tolerance deviation in AC4 (below). AC1 to AC11 and AC13 are green locally; AC12's two CI determinism legs are confirmed on the PR before reporting.
- **The change.** `src/color.rs` factors the colour sink: one generic body (`develop_color_with`) runs the one fused traversal for every single-thread colour entry, with a `PixelSlot` (the SPEC-050 `CropScatter`, or the new `WholePlane` identity) and a `store` function (`store_u16`, `store_f32_quantised`, the new `store_f32_unclamped`). `develop_color_into`, `develop_color_f32_into` and both threaded forms now call those same functions, with the same expressions in the same order, so their output is unchanged (every existing hash and Tier-B pin passes). The band forms call `develop_rows_banded` directly, after checking the CFA (`check_supported_bayer`, a new `pub(crate)` helper that is `develop_color_fused`'s own first two lines) and building the matrix, so the refusal order is the fused path's and nothing is written before the last check.
- **Red first.** The new tests were run against `origin/main`'s `src/` (the three source files checked out from `origin/main`): `tests/linear_plane.rs` failed to compile (the four functions unresolved, `Error::InvalidRowRange` missing) and so did `tests/error_kind.rs`'s new row. With the implementation, the first run had 7 of 9 `linear_plane` tests green: AC13 (placeholder digests, then recorded) and AC4 (the tolerance below).
- **AC1, byte equality on all five held files** (orientation forced to 1, plane cut to ActiveArea origin + DefaultCropOrigin, DefaultCropSize):
  - Canon 5D Mark III: plane 5920x3950, crop 5760x3840 at (140, 96), 132,710,400 bytes equal
  - Pixel 3a: plane 4032x3024, crop 4016x3008 at (8, 8), 72,480,768 bytes equal
  - Pixel 4a: plane 4032x3024, crop 4016x3008 at (8, 8), 72,480,768 bytes equal
  - Pixel 7 Pro: plane 3882x2924, crop 3866x2908 at (8, 8), 67,453,968 bytes equal
  - Nikon D300S: plane 4352x2868, crop 4288x2848 at (15, 10), 73,273,344 bytes equal
- **AC3 counts** (channel values over the whole plane): Canon, 857,253 negative f32 where the u16 is 0, and 7,301 above 1.0 where the u16 is 65535. Pixel 4a, 246 negative and 1,103,668 above 1.0.
- **AC11 partitions tested.** Tier A (20x17, ActiveArea top 3 and left 1, both odd): one strip; strips of 1 row; strips of 7 rows (0..7, 7..14, 14..17, so odd and even starts); two strips split at row 1, inside the masked top border. Tier B, both sample types, the f32 by bits: Pixel 7 Pro (height 2924) as 0..1 then 256-row strips from row 1, 13 strips, remainder 107; Canon (height 3950) the same, 17 strips, remainder 109. Both heights are even with an even remainder at 256, so a one-row lead strip makes the remainder odd and puts every 256-row strip on an odd start.
- **AC12.** `tests/determinism.rs` gains a separate 24x16 RGGB 16-bit colour fixture with AsShotNeutral, ColorMatrix1/2, ForwardMatrix1/2 and CalibrationIlluminant A and D65. Committed hashes: u16 plane `0x03c3_5911_9395_8585`, f32 plane `0x419a_f4d1_3262_5320`, the same in debug and release on macOS arm64. The test also asserts the f32 plane holds a negative and an above-1.0 value, and that removing the second illuminant moves the f32 hash (so the interpolation is inside it). `colour_determinism_red_proof` flips one interior sample's LSB and both hashes move. CI legs: CI run 45666936351 on 8ca4247, all 23 checks green, including `rust / determinism (ubuntu-latest, x86_64)` and `rust / determinism (macos-latest, arm64)` (each runs the determinism file in dev and release), so the committed u16 and f32 plane hashes recorded on arm64 reproduce on x86_64.
- **AC13 digests** (FNV-1a, little-endian; u16 / f32): Canon `0x6013_2438_0cf1_9724` / `0x2bdc_24d3_fb8f_0887`; Pixel 3a `0x74e2_30ed_d6b7_3330` / `0xd4e5_1ae5_398f_bfdc`; Pixel 4a `0xce10_ba92_e648_05f3` / `0xf2a8_dd47_127d_ae01`; Pixel 7 Pro `0x151f_ef25_e6ff_5fe2` / `0x051d_4b2d_2719_32eb`; Nikon `0xba5e_3669_46ef_4d98` / `0xcf80_f1dd_4853_e6eb`.
- **AC10, mutants (each applied by hand, the suites run with `--no-fail-fast`, reverted):**
  | Mutant | Applied as | Failed (the named test in bold) |
  |---|---|---|
  | (a) the sink still crops | `CropScatter` in place of `WholePlane` in the whole-plane body | **`ac2_every_pixel_of_the_plane_is_written`**, `ac1_…`, `ac4_…`, both `ac11_…`, `ac13_…` |
  | (b) the f32 path quantises first | `store_f32_quantised` in place of `store_f32_unclamped` (whole and band) | **`ac3_the_f32_plane_keeps_negatives_and_overs`**, **`ac5_the_f32_plane_is_the_unquantised_sum`**, `colour_plane_hashes_match_committed`, `colour_determinism_red_proof`, `ac13_…` |
  | (c) divide by 65536 | the three divisors in `store_f32_unclamped` | **`ac5_…`**, `ac4_…`, both colour determinism tests, `ac13_…` |
  | (d) GainMaps skipped | the gain maps resolved from a clone without `OpcodeList2` (whole and band) | **`ac7_gain_maps_apply_to_the_plane`**, **`ac1_…`**, `ac13_…` |
  | (e) levels skipped | the whole-plane body reads the raw mosaic | **`ac1_the_plane_cropped_equals_the_developed_image`**, `ac3_…`, `ac7_…`, `ac11_strips_tile_on_held_files`, `ac13_…` |
  | (f) CFA phase from the plane origin | `(0, 0)` for the ActiveArea origin in `develop_color_fused` | **`ac2_…`** only, through its check (iv) (see the deviation below) |
  | (g) dst written before a late check | the band body develops with an identity matrix, then builds the real one | **`ac6_buffer_and_input_refusals_leave_dst_untouched`** (the missing-AsShotNeutral row), both `ac11_…` |
  | (h) the band drops its halo | `BAND_HALO` 3 to 0 | **`ac11_strips_tile_to_the_whole_plane`**, `ac11_strips_tile_on_held_files` |
  | (i) the band top not rounded to even | `& !1` removed in `develop_rows_banded` | **`ac11_strips_tile_to_the_whole_plane`** (Tier B starts are all odd, so `start − 3` is even there and cannot tell) |
  | (j) dst indexed from plane row 0 | `first = 0` in the band body | **both `ac11_…`** |
  | (k) `mul_add` in the f32 sink | the three sums in `store_f32_unclamped` fused | **`color::tests::the_f32_sink_never_fuses_a_multiply_and_an_add`** only, a test added at build (see the deviation below). The first run, before that test, had **no** failure. |

  An unmutated control run of the same suites (`linear_plane`, `determinism`, the lib tests, corpus profile) is green: 9, 9 and 214 passed.
- **AC6 and AC8, the tables.** The four new entries joined `tests/malformed_tags.rs`, `tests/misshapen_tags.rs` (and its `COLOUR_READERS`), `tests/unhonoured_tags.rs` and `tests/required_processing.rs` (its `ColourEntry` table and the staged-crate red-proof's colour list), the band forms over `0..H`. Every row passes. Three counts grew by exactly the new rows: `malformed_tags` AC1 from `7*7 + 2*6 + 11*5` to `7*11 + 2*10 + 11*9`, `required_processing`'s every-entry sentinel harness from 9 to 13, and `error_kind`'s enum from 78 to 79 (`CallerError` 18 to 19). `tests/error_kind.rs` gained exactly the one row, and DEC-051's later-additions paragraph records the variant.
- **AC9.** `fuzz/fuzz_targets/demosaic.rs` also calls the four entries, the band forms with a range from the input's last two bytes modulo `height + 2` (so empty, reversed and over-long ranges are reachable). Smoke run: 2,287,808 runs in 61 s, no crash.
- **Gates run locally:** `just test` with `IRRADIANCE_CORPUS_DIR` set (56 binaries, 935 passed, 0 failed, 1 ignored; the corpus present), then the lib tests again after the two late unit-test changes (214 passed); `lint-ci` (clippy 0.1.98 stable, which caught one `chunks_exact` in a test), `lint-no-allow`, `fmt-check`, `doc-check`, `deny`, `lint-fuzz`, `msrv`, `validate`, `gate-audit`, `cost-audit`, `semver-check` (202 checks pass, no semver update required), and `cargo test --test determinism` in debug and release without features (as CI runs it), 9 passed each. `decisions-audit --changed` names DEC-051 and DEC-079 (advisory); the change is consistent with both.

### Cost self-report

- **Tokens (total):** 39,591,653 (floor, measured before the handback commit)
- **Estimated USD:** 25.68
- **Duration (minutes):** about 25 by the transcript's own timestamps (17:18 to 17:42 UTC); the wall-clock including the local suite and CI waits was longer
- **Source of the number:** this session's transcript (`385139bc-54fe-4f0e-b33c-b6249fa3fde8.jsonl`), deduplicated by `message.id` with the largest `output_tokens` per message, per docs/cost-tracking.md; priced at 5 and 25 USD per million with 1-hour cache writes at 2x. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-051's later-additions paragraph gained one sentence for `InvalidRowRange` (and the missing sentence for DEC-075's two writer variants, which took the enum to 78).
- **Deviations from spec:**
  - **AC4's slack.** The spec's bound `|f32·65535 − u16| ≤ 0.5 + 1e-3` cannot hold: the `f64 → f32` cast alone moves a value in `[0.5, 1)` by up to half an f32 ulp, `2^-25`, which is `65535 / 2^25 ≈ 0.00195` of a code. A Canon pixel at f32 `0.5211032` with u16 34151 sits 0.5029 codes away. The test uses `0.5 + 65535 / 2^25 + 1e-9`; the two other AC4 bounds are as written.
  - **AC2 has a fourth check.** Mutant (f) lives in the shared fused traversal, so comparing the plane with `develop_color_into` (AC2 ii) cannot see it: both sides move together. AC2 (iv) compares the whole plane, border included, with the independent `demosaic_into` then `to_srgb_linear_into` composition (the fixture's levels are the identity), whose CFA phase is indexed separately. That is what catches (f).
  - **Mutant (k) is not caught by the tests the spec names.** Fusing a multiply-add moves the `f64` sum by about one ulp, and the one cast to `f32` absorbs that on all but about one value in 2^29, so the AC12 hash and AC5's exact bits (on a real matrix) both stayed green. A unit test was added, `the_f32_sink_never_fuses_a_multiply_and_an_add`: an injected matrix whose sum is pure cancellation (`0.1·3 − 0.3`, giving 2^-54 separately rounded and 2^-55 fused), which the f32 keeps apart, and which asserts first that the fixture tells the two apart. `docs/determinism.md` says the hash cannot see an FMA and names this test as the guard.
  - **The existing colour entries were refactored onto the shared sink** (not only the new ones), which the build prompt's "factor the sink; do not copy the traversal" asks for. Their four expressions are now `store_u16` / `store_f32_quantised`, called from both the single-thread and the threaded paths. No output moved.
  - **`docs/determinism.md`** splits the old `color.rs` row in two: the plane (measured) and the crop sink plus `to_srgb_linear_into` (same arithmetic, not hashed separately), so the row does not claim more than AC12 measures.
  - **`just semver-check`** reports "no semver update required" (202 checks pass); it does not list additions. The diff adds exactly five public items: the four functions and the variant.
- **Follow-up work identified:**
  - None required. One note for SPEC-092: the band forms re-resolve the levels, the `GainMap`s and the matrix per call, so a caller tiling a 60 MP plane in 256-row strips pays that once per strip (cheap next to the band develop, but a mixer that wants many small strips may want a reusable plan).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Three things, all caught by running the tests or the mutants: AC4's slack is below the f32 cast's own rounding; mutant (f) cannot be caught by AC2 as written because it sits in the traversal the oracle shares; and mutant (k) cannot be caught by any hash, because the f32 cast absorbs a one-ulp FMA difference. All three are recorded above.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-051's "update the table, this number and DEC-051 together" rule was in the test, not the handoff, and it was easy to follow.

3. **If you did this task again, what would you do differently?**
   — Ask, at design, of every pre-registered mutant: is the oracle that should catch it computed by the same code the mutant changes, and is the mutant's effect bigger than the output's resolution? The first question finds (f) and the second finds (k) before build. Also, a mutant harness that restores a file must touch it: restoring an older mtime let cargo reuse a mutant build for my first control run, which I caught because the control went red.
