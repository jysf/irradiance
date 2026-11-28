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
  id: HANDOFF-211
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-28
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-099

project:
  id: PROJ-006
  stage: STAGE-034
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
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 17734375           # REAL combined count — what cost-audit reads
  estimated_usd: 12.57             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 45
  branch: verify/spec-099-sensor-geometry-ca
  pr: null
  completed_at: 2026-11-28         # YYYY-MM-DD
  notes: "Verify in a side session started by the PROJ-006 orchestrator. A FLOOR measured just before the commit over transcript 553db268-69b4-40d6-9090-2fc4c6f5fbb2.jsonl (84 unique message.ids, all claude-opus-5-5, no subagents): input 171 + cache-creation 223,687 (all 1-hour) + cache-read 17,445,970 + output 64,547. The commit, PR and CI wait come after."  # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-28
---

# HANDOFF-211: The per-plane WarpRectilinear at sensor geometry: red and blue resampled to green's geometry, and the consumer's warp named

## Delegation Summary

`claude-opus-5` (acting as architect, the PROJ-006 orchestrator) hands `SPEC-099` to a
cold `claude-opus-5-5` session (acting as verifier) for the **`verify`** cycle of the
build in PR 373 (`feat/spec-099-sensor-geometry-ca`, tip `24b4eb3`, HANDOFF-210's
handback).

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md`
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-034-leica-colour-bodies-reach-the-colour-path.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-099-the-per-plane-warprectilinear-on-the-developed-colour-output.md`, judged as designed (PR 372).
- **Build handback:** `HANDOFF-210` (mutant list, five deviations, timing).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-087` (proposed): the accessor, when the CA step runs, the rule, the ring, the refusal order.
- `DEC-086` items 5 and 8 as amended; `DEC-024` (the bilinear kernel); `DEC-059` §3 (`scale_bounds`); `DEC-044` (banded rows equal the whole plane).

### Constraints that apply

- `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`, `test-before-implementation`, `provenance-recorded-per-algorithm`.

### Prior related work

- `HANDOFF-210`, PR 373 (build); PR 372 (design); PR 368 (SPEC-098 verify, the same Leica files).

## Expected Deliverables

### The verify contract

You are **not** the agent that implemented this. Review it cold.

- Every AC and DEC-087 item with evidence; re-run the three `u16` digests, the `dng_validate -3` bounds and the mutants yourself.
- Look for fixture gaps (boundaries, non-default scenes, untested branches, platform-dependent mutants).
- Rule on the build's five deviations; check the diff's file list against them.
- Measure the slowdown on the Q3 and say whether it is inherent; do not optimise.
- Tests and docs only on the verify branch; any `src/` change goes to a punch list.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Any production `src/` change; SPEC-100 (the developed outputs); the CA step's speed.
- N = 2 and N ≥ 4 refused as malformed: settled by the maintainer.

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `verify/spec-099-sensor-geometry-ca`, stacked on PR 373 (base `feat/spec-099-sensor-geometry-ca`); the PR is linked in its own description. Tests and docs only: unit tests in `src/warp.rs`'s test module, `tests/plane_warp.rs`, one CHANGELOG sentence.
- **Completed at:** 2026-11-28
- **All acceptance criteria met?** yes.
- **For `verify`:** ✅ APPROVED at `24b4eb3` (the code, unchanged). Five oracle gaps, none in the code, each closed by a test on this branch.

**Evidence by AC** (judged against the spec as designed):
- AC1: `ac1_...` green; the consumer, colour, per-plane and malformed mutants red on it. Read against DEC-087 item 1: `describe_warp_planes` colours by `CFAPlaneColor`, names the set coloured 1, `K1` without a CFA, allocates nothing (fixed `[PlaneWarp; 3]`).
- AC2: green; the equal-set mutants red (`ca_equal_sets_build_no_step`, ac1, ac2). The no-step path returns into the untouched `develop_color_with` / `srgb_linear_plane_rows_with` bodies, so N = 1, identical sets and no warp are bit-identical by construction as well as by test.
- AC3: the shared reference (`tests/support/ca_reference.rs`) re-read line by line against DEC-087 item 3: `hypot`, `f`'s operation order, the bisection stop rule, `σ ≥ f_g(1)²` held rim, `min(u, T)` and `min(⌊u⌋, T − 1)`, bilinear clamped to the `ActiveArea`, half up. It calls no crate code. Green on both variants.
- AC4: `ac4_every_store_sees_the_corrected_triple` green; the three stores share one `CaDelay` output.
- AC5: green; the halo and ring mutants red (H = 0, ⌈B⌉ − 1, ⌈B⌉ + 0, the bands' halo above or below not developed, the ring finishing early). Read: row `y` finishes once row `y + H` has fully arrived; the rows forms develop `s0 − H..s1 + H` clamped to the plane and keep only `s0..s1`, so any tiling equals the plane.
- AC6: green, plus the two verify tests below for the rest of item 5's order.
- AC7: 3/3 files ran. The three `u16` pins in `tests/leica_colour.rs` equal the pre-registered `0xd352bb75bf29d270`, `0xc6f67381a8b94593`, `0xbb9908b919ce2b08`; the old SPEC-098 pins and the mono `develop_into` pins are the ones on `origin/main`. My own sweep binary (below), built separately against this branch, reproduces all nine plane digests (u16, f32, mix).
- AC8: I regenerated the SDK outputs myself with `scripts/sdk-stage3.sh` into my scratchpad (the patched copies differ from the originals in 48, 47 and 51 bytes, all inside the red-to-blue coefficient span of `OpcodeList3`). Worst CA error / smallest no-CA error: Q2 R 0.042 / 0.592, B 0.022 / 0.480; Q3 R 0.018 / 0.690, B 0.016 / 0.442; SL3 R 0.032 / 0.620, B 0.036 / 0.406, the build's numbers to the digit. **Red-proof observed:** with the test's CA resample removed, the test fails at the first patch (Q2 red, 0.675 px > 0.10).
- AC9: `DEVELOP_VERSION` "8", `MONO_MIX_VERSION` "1"; `just semver-check`: no semver update required. Corpus sweep: every corpus DNG (16) through `srgb_linear_plane_into`, `srgb_linear_plane_f32_into` and `mono_mix_plane_into` on `origin/main` and on this branch: 13 files identical in result and digest; only the Q2, Q3 and SL3 change, from SPEC-098's pins to this spec's.
- AC10: `just fuzz-warp 60`: 2,860,930 runs in 61 s, no crash. The new seed is synthetic (coefficients rounded to four places, centre 0.5).

**Gates (re-run here):** `just test` with the corpus (every file present): 1,032 passed, 0 failed, 1 ignored, as the build (before my tests). With an empty corpus folder: 1,036 passed, 0 failed, 1 ignored (the 4 more are this verify's tests); every tier-B and SDK test skips by name. `fmt-check`, `lint-ci`, `lint-fuzz`, `doc-check`, `wasm-check`, `semver-check` pass; `validate` after the advance.

### Mutants

- The build's 21, re-run with its harness pointed at this worktree: all red (the hypot one on two tests now).
- My own 26: 22 red, 2 equivalent, 2 survived first and were closed.
  - Red: kt not compared (V1), the tangential check skipping green (V2), the warp resolved before the levels, the `GainMap`s or `OpcodeList1` on the whole-plane and rows forms (V3, V3b, V3c, V3d), fold before tangential (V4), the bands' halo not developed below or above (V5, V6), `ActiveArea` left and top swapped in the source, in `contains` and in the call (V7, V8, V9), width and height swapped (V9b), blue using red's table (V12), the centre from `aw` instead of `aw − 1` (V14), `H = ⌈B⌉` (V15), the consumer fixed at index 1 (V17), the ratio's divisor not at the node (V18).
  - V1, V2, V3–V3d and V4 survived the build's tests: those are the first two gaps below, closed by my tests before this run.
  - **Survived at first, closed:** V10 (the halo bound's radius `√(j/T)` instead of `√((j+1)/T)`) and V13 (the build's deviation 2, the non-finite check on sets 2 and 3, removed). Both gaps 3 and 4 below.
  - **Equivalent:** V11 (`σ > f_g(1)²` instead of `≥`: at an exact rim node the bisection lands on 1.0 too) and V16 (a ring of `2H + 1` rows: the rows a finished row reads are exactly `2H + 1`, so DEC-087's `2H + 2` holds one spare row). Neither is a defect.

### Gaps found and closed (tests on this branch)

- FU-1 `verify_tangential_terms_count_in_the_comparison_and_come_before_the_fold`: no fixture had sets differing in `kt` alone, a `kt` on green, or a `kt` with a folding green. A `kr`-only comparison (V1), a tangential check that skips green (V2) and fold-before-tangential (V4) all survived.
- FU-2 `verify_the_levels_gain_maps_and_list_1_come_before_the_warp`: AC6 placed the warp after the CFA and before the calibration, but nothing separated it from the three earlier checks. The warp moved ahead of the levels, the `GainMap`s or `OpcodeList1` (V3–V3d) survived.
- FU-3 `ca_table_and_bound_on_a_hand_checkable_set`, extended: a constant ratio with `m·ε` just above 6 pins `H = 8`, which `√(j/T)` would make 7 (V10).
- FU-4 `verify_a_non_finite_later_set_is_malformed`: deviation 2 had no test (V13); NaN and infinite `kr`/`kt` in sets 2 and 3, from the accessor and all six entries.
- FU-5 `ca_m_is_hypot_across_geometries`: the build's `ca_m_is_hypot` catches the sqrt mutant only where the platform's libm rounds its one geometry apart (macOS, not glibc), so CI's Linux run could not see it. The sweep of 144 off-centre geometries asserts `m` is `hypot` on each, and that hypot and the naive form differ on at least one there (on macOS; 22 of the 144 differ from a correctly rounded hypot, so glibc's near-correctly-rounded hypot gives it teeth there too).
- FU-6 (docs, closed): the CHANGELOG now names the non-finite second or third set refusal.

### Rulings on the deviations (each checked against the diff's file list)

- **SPEC-098's `ac8_the_sl3_bad_columns_are_fixed` drops list 3:** accepted. It counts the fix's own changes, which the CA step would spread into red and blue neighbours; the SL3's u16 pin (pre-registered with DEC-086's fix applied) still covers the fix composed with the CA step.
- **A non-finite coefficient in set 2 or 3 is malformed:** accepted (a NaN set cannot be resampled, and `K1` already gets the same check). Now tested (FU-4). FU-7 for the ship: one line in DEC-087 items 1 and 5 saying so, since the decision is silent.
- **`is_per_plane()` false without a CFA; `m == 0` before the fold:** accepted. Item 1 defines `is_per_plane()` as "whether the CA step runs" (it never runs without a CFA), and item 3 lists "if m = 0, there is no step" before the fold check.
- **`bilinear_sample` into `bilinear_sample_with`:** accepted; every warp pin is green and the arithmetic is moved, not changed.
- **AC8's `n` without the SL3 bad-pixel fix:** accepted; the patches end near x = 9234, short of the bad columns at 9528 to 9535.
- File list: every file in the diff is in the spec's Outputs or a declared deviation (`tests/support/opcode.rs` holds AC10's seed; `decisions/DEC-051` is the variant tally the build's gates note names). DEC-086's amendment lines were added at design (PR 372), not here. HANDOFF-210's `pr:` field is null; PR 373 is the build PR.

### Timing (`srgb_linear_plane_into`, release, Q3, best of 5, this machine)

- No list 3 (the SPEC-098 traversal): 0.958 s. With the CA step: 2.270 s (median 2.276). `mono_mix_plane_into`: 0.994 s to 2.212 s. The build's 0.96 to 2.30 s is reproduced.
- Breakdown by throwaway variants (built, timed, reverted; nothing committed): the ring alone (push, copy, store, no resample) 1.20 s; plus `σ`, the ratio and the source coordinate with a nearest read 1.66 s; plus the bilinear kernel 2.27 s. Precomputing the ring's row bases (no per-read modulo) changed nothing (2.29 s).
- **Verdict: mostly inherent.** About 1.07 s of the 1.31 s is the per-pixel two-plane resample itself (two divisions by `m`, the table lookup and a four-read bilinear, per plane), and 0.24 s the ring. The design probe's 0.49 to 0.64 s was a bare loop, not the entry. Output-invariant gains are modest (sharing `σ` between red and blue, a bilinear over row slices without the per-read clamp and closure), perhaps 0.3 to 0.5 s; a reciprocal of `m` would change bits and so the pins. No cheap one-line fix. FU-8: optional, not a gate.

### Licence

- No DNG bytes in the diff; no seed or fixture derived from the Q3, SL3 or M11. The SDK outputs stay in my scratchpad. Every tier-B and SDK test skips with an empty corpus.

### Findings

- FU-1 to FU-6: closed on this branch (above).
- FU-7 (for the ship): DEC-087 items 1 and 5 could name the non-finite later-set refusal.
- FU-8 (optional follow-up): the CA step's cost on a 60 MP plane, as measured above.
- FU-9 (informational): DEC-087 item 4's ring of `2H + 2` rows has one spare row (`2H + 1` is enough); harmless.
- No ship-blockers.

### Cost self-report

- **Tokens (total):** 17,734,375 (a floor measured before the commit; input 171, cache creation 223,687, all 1-hour, cache read 17,445,970, output 64,547; 84 messages, all claude-opus-5-5, no subagents)
- **Estimated USD:** 12.57 (5 and 25 USD per million, cache reads 0.1x, 1-hour writes 2x)
- **Duration (minutes):** 45
- **Source of the number:** this session's transcript (`553db268-69b4-40d6-9090-2fc4c6f5fbb2.jsonl`, the scratchpad id), unique `message.id`s, the largest `output_tokens` per message, no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none in this cycle; the build's five are ruled on above.
- **Follow-up work identified:** FU-7 at ship; FU-8 optional; SPEC-100 as planned.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the rule; the rule-clause map listed mutants for every sub-rule of item 3 but none for item 5's order before the CFA or for the tangential comparison, which is where the gaps were.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — A rule for platform-dependent float mutants: a unit test that depends on one libm's rounding should sweep inputs and assert the forms differ somewhere, so it has teeth on every CI host.

3. **If you did this task again, what would you do differently?**
   — Write the order mutants (each refusal moved one step earlier) first: they are cheap and found two of the five gaps.
