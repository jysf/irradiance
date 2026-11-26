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
  id: HANDOFF-204
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-26
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-093

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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 50795351               # REAL combined count — what cost-audit reads
  estimated_usd: 32.95              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 75
  branch: feat/spec-093-developed-mix
  pr: 358
  completed_at: 2026-11-26               # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 3a1702f3-fb46-4b6d-808b-f0a4d0906600.jsonl (146 unique message.ids, all claude-opus-5-5, no subagents): input 311 + cache-creation 430,377 (1-hour) + cache-read 50,223,435 + output 141,228, priced at 5 and 25 USD per million. Re-sum at ship. CI run 45898355595 on 6b29c48 green, 25 checks, both determinism legs."                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-204: The developed mix: a mix option on develop_with_into, output_dimensions_with and scratch_len, through the existing geometry and resampling

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-093` to a cold `claude-opus-5` session
(acting as implementer) for the **`build`** cycle: one new `DevelopOptions` field,
`mono_mix: Option<[Rational; 3]>`, with which the four mono develop entries develop a
colour Bayer DNG to SPEC-092's mix at the mono develop's geometry; one new
`DevelopPath::MonoMix`; the mixer run in 512-row strips into caller scratch; an
allocation-only refactor of the fused traversal; no new `Error` variant, no version bump,
and no change to any existing output.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-006-spectral-monochrome-mixer/brief.md` (onyx's three reply sections)
- **Stage:** `./projects/PROJ-006-spectral-monochrome-mixer/stages/STAGE-031-colour-to-mono-the-linear-rgb-step-the-mixer-and-the-develop.md`
- **Spec:** `./projects/PROJ-006-spectral-monochrome-mixer/specs/SPEC-093-the-developed-mix-a-mix-option-on-develop-with-into-output-d.md` (Design §1 to §11, AC1 to AC14, Failing Tests, mutants (a) to (z)). The spec is the contract.
- **Decision:** `./decisions/DEC-084-the-developed-mix-is-the-mono-develop-geometry-over-the-mixed-plane-mixed-in-row-strips-into-caller-scratch.md` (proposed; the rule, the opcode set, the strips, the scratch, the order, the versions)
- **The shipped siblings:** `specs/done/SPEC-092-...md` and `specs/done/SPEC-091-...md`, and their handoffs HANDOFF-202 / HANDOFF-203 (`## Handback`: the mutant-harness method, the `Entry`-table count changes, the CI legs).
- **Toolchain brief:** `./guidance/toolchain-brief.md`, and `AGENTS.md` §5's measured toolchain (the `+toolchain` traps for `cargo fuzz` and `lint-ci`).

### Decisions that apply

- `DEC-084` — this spec's decision: the developed mix is the mono develop of the mixed plane.
- `DEC-083` — the weights rule, the row and the kernel the mix uses, unchanged.
- `DEC-079` — weights on linear sRGB, folded into the camera matrix.
- `DEC-058` — `DevelopOptions` is `#[non_exhaustive]`, its default is `develop_into`, options never skip validation.
- `DEC-059` — `warp_alpha` and the `FullActiveArea` bounds rule, unchanged under the mix.
- `DEC-060` — one exact-length caller scratch slice, private layout, no allocation in `develop_with_scratch_into`.
- `DEC-043` — the colour develop's crop/orient scatter, reused for the no-warp mix.
- `DEC-044` — the halo'd row bands the strips run on.
- `DEC-049` — required-processing validation; `DevelopPath::MonoMix` extends it.
- `DEC-068`, `DEC-070` — the colour levels the mix uses (so `black_levels` is not read).
- `DEC-016`, `DEC-002` — caller-owned buffers; determinism.

### Constraints that apply

- `no-panics-on-untrusted-input` — every new path is checked arithmetic and `.get`; fuzzed (AC12).
- `oracle-must-be-shown-red` — AC14's hand-applied mutants.
- `test-before-implementation` — write the Failing Tests first and see them fail (SPEC-092's build did not; do).
- `provenance-recorded-per-algorithm` — no new algorithm, no ledger row (say so in the PR).

### Prior related work

- `HANDOFF-202` / PR 354 — the mixer at sensor geometry this develops.
- `HANDOFF-203` — the verify that found SPEC-092/FU-5 and FU-6, which this spec's AC9 and AC8 answer.
- PR 352, PR 353 — SPEC-092's design and onyx's reply.

### Facts the design probe measured (2026-11-26, release build, this machine)

- Opcode lists on the held colour files: Canon and Nikon none in `OpcodeList3`; Pixel 3a and 7 Pro an **optional non-identity** radial `WarpRectilinear` (no tangential terms); Pixel 4a an optional **identity** warp (skipped). Pixel 7 Pro is Orientation 3. The Q2 Monochrom's `FixBadPixelsConstant` and `WarpRectilinear` are both **mandatory** (flags 0).
- AC4's digests and `scratch_len` numbers were computed by the §2 oracle, not by an implementation. Record them as given; a mismatch is reported.
- Strips of 512 rows through the public band entry cost 0.99× to 1.06× of the whole-plane mix (256 rows: up to 1.12×).
- Probe source (not in the repo): `/Users/jyashinsky/Library/Application Support/Claude/side-session-notes/local_b96e1f4e-3647-4291-8ea1-c0380156642b/spec-093-design-probe.rs.txt`. It ran as a throwaway `examples/` file with `cargo run --release --all-features --example`.

## Expected Deliverables

- Code changes implementing SPEC-093's Acceptance Criteria (Design §1 to §11).
- All Failing Tests in SPEC-093 written first, seen failing, then passing.
- The mutant table (AC14) with the test each mutant failed; (u), (y), (z) argued.
- A PR against `main` from branch `feat/spec-093-developed-mix`.
- PR description referencing: HANDOFF-204, SPEC-093, STAGE-031, PROJ-006, DEC-084 and the
  other DECs above (this repo's `decisions/` namespace), provenance "none — no new
  algorithm", and the oracle (the §2 mono-develop oracle, the AC4 digests, the AC11
  hashes, and the mutants as its red-proof).

## Out of Scope

- A CFA-aware `FixBadPixelsConstant`, or any change to how SPEC-092's sensor-geometry
  entries treat a mandatory warp (the spec's Open question 1).
- A new `Error` variant, a `DEVELOP_VERSION` or `MONO_MIX_VERSION` bump, a public strip
  size, threaded variants, an `irr` hook, presets.
- Any change to an existing output, hash or Tier-B pin.

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

### Execution notes

- **Branch / PR:** `feat/spec-093-developed-mix`, https://github.com/jysf/irradiance/pull/358. Not merged.
- **Completed at:** 2026-11-26
- **All acceptance criteria met?** Yes, with the deviations below. AC11's "one ulp" can't be met as written, so the red-proof uses `2^-26` instead.
- **CI:** CI run 45898355595 on `6b29c48` is green: all 25 checks, including `determinism` on ubuntu-latest x86_64 and macos-latest arm64 (debug and release). The handback commit after it is docs only.

### Results

- **Red first.** The public surface (the field and the variant) went in as a stub first. All 14 `tests/developed_mix.rs` tests compiled and failed at run time against it. The SPEC-092 lesson is applied.
- **AC4 matched as pre-registered:** all 15 registered digests and every `scratch_len` number. Nothing was edited.
  - `scratch_len`: Canon 3,119,840; Nikon 2,293,504; Pixel 4a 2,124,864; Pixel 3a 24,385,536 (2,124,864 at α 0); 7 Pro 22,701,936 (2,045,814 at α 0).
  - Pixel 3a and 7 Pro also equal the mono develop's `scratch_len` for `mono_of`.
  - The Pixel 3a is refused under FullActiveArea with `scale_max = 1.0015292540192604`.
- **AC1:** fixtures (i)–(v), each with 40 options sets × 2 weight sets, equal the §2 oracle.
- **AC3:** 4,160, 4,168, 4,216 and 4,216 as literals.
- **AC8:** 76 pairs on `develop_with_into` and 89 on `develop_with_scratch_into`.
  - Each fault alone is checked on both entries.
  - Skipped pairs, on both entries: (F7, F8), opposite framings, both in OpcodeList3; (F8, F13), one OpcodeList3, and a malformed block has no tangential term.
  - (F10, F11) and (F7, F13) run with one value carrying both faults.
  - The sizing entries refuse F0, F1, F4 (and F8 for `scratch_len`), plus the lower-ranked fault of each pair among them.
- **AC10:** `develop_with_scratch_into` makes zero allocations in every case:
  - fixtures (i), (ii) and (iii) × both framings × Orientation 1 and 6 (12 cases);
  - each of F0–F13 alone;
  - the Pixel 7 Pro.
  `develop_with_into` makes exactly one allocation of 2·`scratch_len` bytes: 1,144 on (i), 1,704 on (ii) and 21,080 on (iii).
- **AC11 hashes** (recorded on macOS arm64):
  - unwarped: `0x7b076b08b1b51de1`, `0x0f5077a876c2fbd3`. These equal SPEC-092's committed plane hashes, because the fixture has no crop or Orientation; asserted.
  - warped (with Orientation 6): `0x60c59f35c60f4cc4`, `0xe7641465f8f2cd58`.
- **AC12:** fuzz smoke runs, 61 s each:
  - `demosaic`: 2,338,891 runs, no crash;
  - `develop`: 2,594,751 runs, no crash.
- **Gates:** all green.
  - `just test` with the corpus: 986 passed, 0 failed. The only skips are the three optional SDK cross-checks.
  - `lint-ci`, `lint-no-allow`, `lint-fuzz`, fmt, `doc-check`, MSRV 1.90.0, `validate`, `cost-audit` and `name-length-check`.
  - `semver-check`: no semver update required, additions only.

### AC14: the mutants

Each mutant was applied by hand with a string-replacement harness, then these suites ran: the lib unit tests, `developed_mix`, `determinism`, `develop_scratch_alloc`, `required_processing`, `mono_mix` and `develop`. The file was restored from git after each. All are red.

| Mutant | Verdict | Tests that fail (first few) |
|---|---|---|
| (a) | RED | `ac6_the_mix_applies_a_mandatory_warp`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` |
| (b) | RED | `ac6_the_mix_applies_a_mandatory_warp`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` |
| (c) | RED | `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`, `ac4_held_files_match_the_oracle_and_the_digests` |
| (d) | RED | `ac6_the_mix_never_parses_opcode_list_1`, `ac1_hand_derived_pixels_under_orientation_6`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` (+4) |
| (e1-last-strip) | RED | `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`, `developed_mix_contract_constants_are_the_computation`, `ac5_black_levels_is_not_read_under_the_mix` (+9) |
| (e2-end-off-by-one) | RED | `ac1_hand_derived_pixels_under_orientation_6`, `ac2_the_option_absent_is_develop_into`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` (+9) |
| (f1-no-halo) | RED | `ac1_hand_derived_pixels_under_orientation_6`, `ac5_black_levels_is_not_read_under_the_mix`, `ac6_the_mix_applies_a_mandatory_warp` (+8) |
| (f2-odd-top) | RED | `ac5_black_levels_is_not_read_under_the_mix`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`, `ac1_hand_derived_pixels_under_orientation_6` (+6) |
| (g) | RED | `ac6_the_mix_never_parses_opcode_list_1`, `ac1_hand_derived_pixels_under_orientation_6`, `ac6_gain_maps_apply_once` (+8) |
| (h) | RED | `ac6_gain_maps_apply_once`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`, `ac4_held_files_match_the_oracle_and_the_digests` |
| (i) | RED | `ac6_the_mix_refuses_a_mandatory_fix_bad_pixels`, `ac8_each_fault_alone`, `ac8_every_ranked_pair_on_both_develop_entries` (+3) |
| (j) | RED | `ac6_the_mix_applies_a_mandatory_warp`, `colour_mandatory_opcode_in_list_3_is_rejected_on_every_entry` |
| (k1-develop) | RED | `ac6_the_mix_never_parses_opcode_list_1` |
| (k2-scratch-len) | RED | `ac6_the_mix_never_parses_opcode_list_1`, `ac8_the_sizing_entries_check_only_what_they_need` |
| (l) | RED | `ac6_the_mix_applies_a_mandatory_warp`, `ac8_each_fault_alone`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` (+4) |
| (m) | RED | `ac9_scratch_len_under_the_mix_is_the_strip_formula`, `ac6_the_mix_applies_a_mandatory_warp`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` (+1) |
| (n) | RED | `ac8_every_ranked_pair_on_both_develop_entries` |
| (o) | RED | `ac8_every_ranked_pair_on_both_develop_entries` |
| (p1-dst) | RED | `ac8_each_fault_alone`, `ac8_every_ranked_pair_on_both_develop_entries` |
| (p2-scratch) | RED | `ac8_each_fault_alone`, `ac8_every_ranked_pair_on_both_develop_entries` |
| (q1-no-halo) | RED | `ac9_scratch_len_under_the_mix_is_the_strip_formula`, `ac3_strip_count_and_the_cap_of_t`, `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane` (+3) |
| (q2-no-ring) | RED | `ac9_scratch_len_under_the_mix_is_the_strip_formula`, `ac1_hand_derived_pixels_under_orientation_6`, `ac8_each_fault_alone` (+20) |
| (r) | RED | `ac9_scratch_len_under_the_mix_is_the_strip_formula`, `ac1_hand_derived_pixels_under_orientation_6`, `ac5_black_levels_is_not_read_under_the_mix` (+18) |
| (s) | RED | `ac9_scratch_len_under_the_mix_is_the_strip_formula`, `ac8_the_sizing_entries_check_only_what_they_need`, `ac4_held_files_match_the_oracle_and_the_digests` |
| (t1-dims) | RED | `ac8_the_sizing_entries_check_only_what_they_need` |
| (t2-scratch-len) | RED | `ac8_the_sizing_entries_check_only_what_they_need` |
| (v) | RED | `ac3_no_negative_row_is_the_plain_fold`, `ac8_the_fade_starts_after_57343_and_reads_the_largest_channel`, `ac14_no_overflow_at_the_extremes` (+18) |
| (w) | RED | `crop_origin_is_relative_to_active_area_not_the_raw_plane`, `ac5_black_levels_is_not_read_under_the_mix`, `ac2_the_option_absent_is_develop_into` (+26) |
| (x) | RED | `ac1_the_developed_mix_is_the_mono_develop_of_the_mixed_plane`, `ac5_black_levels_is_not_read_under_the_mix`, `ac6_gain_maps_apply_once` (+6) |

Equivalent, argued in the handback rather than run: (u) resolving the levels and row per strip, (y) strips over every plane row, and (z) strips of 256 in both the core and `T`.

**Why the equivalents need no run:**
- **(u) Levels and row resolved per strip instead of once.** Each resolution is a pure function of the sensor and the weights, so every strip would get the same values; it is only slower.
- **(y) No-warp strips over every plane row.** The extra rows' pixels are dropped by the scatter's crop check. The band halo makes every kept pixel the whole plane's value whatever strip it falls in, and the strip-invariance tests confirm it.
- **(z) Strips of 256 in both the core and `T`.** The output is unchanged by strip invariance (AC3 and AC1 fixture (iii)). Only the `T` literals in AC9(d) and AC3 move, which is what (q)'s guard checks.

### Cost self-report

- **Tokens (total):** 50,795,351 (floor; re-sum at ship)
- **Estimated USD:** about 32.95, per component at Opus 5.5 rates
- **Duration (minutes):** about 75
- **Source of the number:** this session's transcript, `3a1702f3-fb46-4b6d-808b-f0a4d0906600.jsonl`, deduplicated by message.id (docs/cost-tracking.md)

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  1. **F10 removes both colour matrices.** The tier-A fixture carries `ForwardMatrix1` beside `ColorMatrix1`, because with CM1 alone the white balance is renormalised and F11's tiny neutral develops (measured row `[38640012, 27311210, -10534380]`). With FM1 present, removing CM1 alone still calibrates. So F10 removes both: "no colour matrix".
  2. **AC11's one-ulp warp change.** One ulp of any warp coefficient moves no `u16` output pixel on the colour fixture (measured), so that red-proof can't be met as written. The smallest power-of-two step of `kr0` that moves both warped hashes is `2^-26` (2^26 ulps), and the red-proof uses it. The measurement is recorded in `tests/determinism.rs` and `docs/determinism.md`.
  3. **Design §10 asked for `develop` fuzz option bit 5,** but bit 5 has been `black_levels` since SPEC-066. Bit 6 sets `mono_mix`, and the target's option table documents it. No new seeds: the fuzzer flips bit 6 trivially.
  4. **`RING` placement.** It moved from a constant inside `develop_color_fused` to a module-level `pub(crate)` constant in `src/demosaic.rs`, so `T` (`mix_strip_work_len`) can read it. The value 8 is unchanged, and the AC9(d) unit test ties the two.
  5. **`develop_rows_banded` makes one allocation for its green rows and ring together,** where it made two before (allocation only; outputs unchanged, the existing hashes and band tests guard it).
  6. **Red-before-green covers `tests/developed_mix.rs` only.** The unit tests (`develop.rs` AC9(d), `opcode.rs` AC6(v)), the allocation tests, the determinism hashes and the Entry rows were written after the implementation. Their red is shown by the mutant table: q, r, s and m turn AC9(d) red, and i and j turn the Entry rows red.
  7. **`tests/required_processing.rs` is path-aware where the mix differs.** `colour_mandatory_opcode_in_list_3_is_rejected_on_every_entry` expects the mix to apply a mandatory warp, and adds a two-warp refusal on every entry. `public_check_agrees_with_every_develop_entry` compares each entry against its own path. The `malformed_tags` count grows from 236 to 276, and the `seventeen_gain_maps` count from 15 to 17.
  8. **The strip pass takes 8 arguments** and carries `allow(clippy::too_many_arguments)` with a reason. This isn't one of the five forbidden policy lints, and the repo has three precedents.
- **Follow-up work identified:**
  - [XS] The `demosaic` fuzz target still has no CI smoke job (signal `fuzz-targets-without-a-ci-smoke-job`). This spec adds the developed mix to it.
  - The design lesson for STAGE-031's close: a red-proof that perturbs an `f64` input to a `u16`-rounded output should pre-register the perturbation size from a probe, not "one ulp".

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   - Three fixture details were stated as facts the code contradicts: F10 "removes ColorMatrix1", fuzz "spare bit 5", and the one-ulp red-proof. Each cost a probe loop.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   - DEC-061 / SPEC-066's use of develop option bit 5.
3. **If you did this task again, what would you do differently?**
   - Write the unit and allocation tests against the stub too, not only the integration suite.
   - Probe the calibration path a fault fixture depends on before writing the fault table.
