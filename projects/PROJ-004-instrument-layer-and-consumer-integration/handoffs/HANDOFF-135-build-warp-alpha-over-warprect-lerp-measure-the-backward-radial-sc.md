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
  id: HANDOFF-135
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-12
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-064

project:
  id: PROJ-004
  stage: STAGE-019
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
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 17582624           # REAL combined count — what cost-audit reads
  estimated_usd: 13.06             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 75
  branch: claude/spec-064-build
  pr: 219
  completed_at: 2026-11-12         # YYYY-MM-DD
  notes: "FLOOR, re-summed before the handback commit from transcript 50667760-e761-4b85-9731-204e0e8a7891.jsonl (80 unique message.ids, all claude-opus-5-5, no subagents): input 159 + cache-creation 238,214 (1-hour) + cache-read 17,262,494 + output 81,757 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour write x2.0). The commit, push and CI-wait turns come after; re-sum at ship."
                                   # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-135: BUILD: warp_alpha over WarpRect::lerp, and the FullActiveArea bounds rule (SPEC-064)

## Delegation Summary

You are the **builder** for SPEC-064. Cut `claude/spec-064-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-059
and this handoff.

The task has four parts:
- Add `WarpRect::lerp(alpha)` (`src/opcode.rs`).
- Add `DevelopOptions::warp_alpha` (default `1.0`) and apply the lerped warp on the mono
  develop path.
- Replace SPEC-063's blanket refusal of `FullActiveArea` with a warp by DEC-059's bounds
  rule: `0 ≤ f_α(s) ≤ 1` on `s ∈ [0, 1]`, the optical centre inside the image, everything
  finite.
- Add `Error::InvalidWarpAlpha` and three fields on the unreleased
  `UnsupportedWarpedFullActiveArea`.

Default output must stay byte-identical. SPEC-063's eight MD5 pins must stay green
unedited. All three Q2M frames must now develop under `FullActiveArea` at every α, and the
Pixel 3a's real over-unity polynomial must still be refused before any write.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-064-warp-alpha-over-warprect-lerp-measure-the-backward-radial-sc.md`.
  Read all of it:
  - the §1 probe table and the full-precision coefficients;
  - Design §2 to §5;
  - AC1 to AC13, the red-proof list, and the Notes for the Implementer (the α = 1
    bit-identity formula, and the interior-peak fixture).
- **Decisions:**
  - `./decisions/DEC-059-warp-alpha-lerps-the-radial-scale-toward-identity-and-full-active-area-admits-a-warp-whose-scale-stays-in-0-1.md`:
    the lerp, the α domain, the bounds rule and why it is sufficient, and `StandardCrop`
    unchanged.
  - DEC-058: options shape, validate-then-override, typed refusal, the version rule. This
    spec amends §4.
  - DEC-024 (clamp-to-edge), DEC-028 (`DEVELOP_VERSION`), DEC-051 (wildcard-free
    `Error::kind`), DEC-052 (`#[non_exhaustive]`), DEC-003 (tier-B bytes never committed).
- **Code:**
  - `src/develop.rs`: `DevelopOptions` ~274, `non_identity_warp` ~381,
    `develop_with_into` ~979, and its early refusal ~1042;
  - `src/warp.rs`: `source_coord` is the formula the bounds rule reasons about;
  - `src/opcode.rs`: `WarpRect` ~166;
  - `src/lib.rs`: `Error`, `kind()`.
- **Tests to mirror:**
  - `tests/develop_options.rs`: MD5 pins, sentinel and precedence probes, the tier-B gating;
  - `tests/warp.rs`: the corner-impulse fixture;
  - `tests/support/opcode.rs`: `single_warp_rectilinear_list`;
  - `tests/error_kind.rs`.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test` already
  passes `--all-features`; add no flag. For tier B set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. Use the files in
  place and never copy them into the repo.

## Expected Deliverables

1. **Tests first.** Write `tests/warp_alpha.rs` and the `src/warp.rs` unit tests the
   spec's Failing Tests list names, plus the `tests/error_kind.rs` row. Show them red;
   a compile failure counts for the new API. Record that order in the handback.
2. **Implement it.** Follow Design §2 and §3:
   - `lerp`, using exactly the formula in the Notes;
   - the field and its default;
   - step 0 (α validation, first, in both `develop_with_into` and
     `output_dimensions_with`);
   - `non_identity_warp(sensor, alpha)`;
   - the bounds check under `FullActiveArea` only, in the place SPEC-063's refusal
     occupies today: before the `FixBadPixelsConstant` copy and before any `dst` write.
     Do not parse the warp twice.
3. **Edit SPEC-063's two refusal tests** as AC6 says, and nothing else in existing tests
   except `error_kind`'s rows and counts. In the handback, record each edited assertion
   and why.
4. **Fuzz (§4).** Extend the option byte, then add the seeds via `examples/fuzz-seeds.rs`.
   Run it for at least 60 s and report the run count.
5. **Docs (§5).** Update the SPEC-063 CHANGELOG bullets, add the new entries, add a
   provenance-ledger row, and amend the module docs. Do **not** bump `DEVELOP_VERSION`.
6. **Gates, all green with the corpus env set:** `just test`, `lint-ci`, `lint-no-allow`,
   `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`. Also
   run `just decisions-audit --changed`.
7. **Red-proofs.** For each mutation in the spec's red-proof list, apply it by hand, show
   the named test red, and revert. List them in the handback.
8. **PR.** Open a PR against `main` from `claude/spec-064-build` that references SPEC-064,
   DEC-059 and HANDOFF-135. Confirm CI is green on its head.

## Out of Scope

- A pixel-exact bounds check, or admitting `f > 1` near the centre (DEC-059 records this
  as a later relaxation).
- Any change to `StandardCrop`'s behaviour, clamp included.
- Tangential-term support.
- Caller scratch (SPEC-065), CFA handling (SPEC-066), the blocked transpose (SPEC-067).
- The colour path, `irr` CLI flags, a release, or a `DEVELOP_VERSION` bump.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). Coefficients and MD5
  strings in source are fine.
- If a SPEC-063 MD5 pin goes red, your change altered default output. Fix the change;
  never edit the pin.
- Fill the `handback:` front-matter, including a real `tokens_total` re-summed from your
  own transcript (`basis: transcript`, dedupe by `message.id`, max output per message,
  opus-5-5 rates). Fill the `## Handback` section too.

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

- **Branch / PR:** `claude/spec-064-build`, https://github.com/jysf/irradiance/pull/219
  (build commit 75ea6d0, then this handback commit).
- **Completed at:** 2026-11-12
- **All acceptance criteria met?** Yes, AC1 to AC13.
- **For `verify`:** n/a (build).

**Test-first order.**
1. I wrote `tests/warp_alpha.rs` (12 tests), the three `src/warp.rs` unit tests
   (`scale_bounds_match_the_probe`, `scale_bounds_degenerate_cubics`,
   `scale_bounds_nan_refuses`), the `tests/error_kind.rs` row and the AC6 edits.
2. They were red as compile failures before any `src/` change: the lib test failed with
   29 errors (no `scale_bounds` or `full_active_area_admits`). `warp_alpha` failed with 14
   (no `lerp`, no `warp_alpha` field, no `InvalidWarpAlpha`, no new fields).
   `error_kind` failed with 4 and `develop_options` with 3.
3. Then I implemented `lib.rs`, `opcode.rs`, `warp.rs` and `develop.rs`.
4. One test came back red after the implementation. AC4 asserted 506.1 px at α = 1,
   but that figure is the 8392×5632 active area; the crop-size fixture's analytic value
   is 503.7 px. The test now computes `m·(1 − f(1))` itself.

**Gates.** All green with `IRRADIANCE_CORPUS_DIR` set:
- `just test`: 632 passed, 0 failed, 1 ignored (pre-existing), 12/12 corpus files
  present, no SKIP.
- Also green: `lint-ci` (clippy 0.1.98), `lint-no-allow`, `lint-fuzz`, `fmt-check`,
  `validate`, `doc-check`, `deny` (licenses ok), `msrv` (1.90.0), `cost-audit`,
  `gate-audit`.
- `decisions-audit --changed`: advisory list reviewed. The change is consistent with
  DEC-024 (clamp unchanged under StandardCrop), DEC-051 (wildcard-free `kind`),
  DEC-052 (`WarpRect` stays non-exhaustive; `lerp` builds it in-crate) and DEC-058 as
  amended by DEC-059.

**AC evidence (tier B).**
- **AC1:** all 8 SPEC-063 MD5 pins hold, unedited, through `develop_into` and through an
  explicit `warp_alpha = 1.0`.
- **AC3:** α = 0 equals the warp-free develop on all three Q2M frames, under both
  framings.
- **AC5:** on all three Q2M frames at five α, `FullActiveArea` is 8392×5632 and contains
  the `StandardCrop` output at (12, 24); at α = 1 the crop matches the pin.

**Fuzz.** The `develop` target ran 5,639,854 times in 76 s (30 seeds) with zero crashes.

**Red-proofs.** Each mutation was applied by hand to `src/`, run, shown red, and reverted
with `git checkout`; `src/` is clean at the end.
1. Default `warp_alpha` 0.5: `default_warp_alpha_is_one_and_pins_hold` red.
2. `lerp` drops the `(1 − α)` term on kr0: `lerp_is_bit_exact_at_one_and_identity_at_zero`
   and `displacement_is_linear_in_alpha` red.
3. `lerp` does not scale kr1..kr3: the same two red.
4. `lerp` scales only kr1: the same two red.
5. The lerp is not applied (`non_identity_warp` lerps to 1.0):
   `alpha_zero_equals_the_warp_free_develop` and `displacement_is_linear_in_alpha` red.
6. Interior roots omitted: `each_bounds_clause_refuses` (interior peak) and
   `scale_bounds_match_the_probe` red.
7. Only `s = 1` checked (f(0) and the roots dropped): `each_bounds_clause_refuses`,
   `scale_bounds_match_the_probe` and `scale_bounds_degenerate_cubics` red.
8. `f ≥ 0` clause omitted: `each_bounds_clause_refuses`,
   `scale_bounds_degenerate_cubics` and `scale_bounds_nan_refuses` red.
9. Centre clause omitted: `each_bounds_clause_refuses` and
   `scale_bounds_degenerate_cubics` red.
10. NaN-unsafe comparisons `!(min < 0.0 || max > 1.0)`: `scale_bounds_nan_refuses` red.
11. The same comparisons with the finiteness clause also dropped:
    `each_bounds_clause_refuses` red.
12. Bounds check applied under `StandardCrop` too:
    `over_unity_warp_still_develops_under_standard_crop` red.
13. Blanket refusal kept: `passing_warps_develop_under_full_active_area` (tier A) and
    `full_active_area_develops_every_q2m_frame_at_five_alphas` (tier B) red.
14. Refusal after the `OpcodeList1` parse:
    `over_unity_pixel_3a_warp_is_refused_before_any_write` (precedence probe) red.
15. Refusal after a `dst` write: the same test (sentinel) red.
16. α validation after `check_required_processing`:
    `warp_alpha_outside_the_domain_is_a_caller_error` red.

Mutations 2, 3 and 5 at first left `displacement_is_linear_in_alpha` **green**. The wrong
warp mapped the impulse off the image, the developed plane was all zero, and
`max_by_key` returned the last pixel, which is exactly the corner the test expects. The
test now asserts that the impulse survives (peak ≥ 65535/4). After that fix, all three
are red. That fix is the only test change in the handback commit.

**Edits to existing assertions** (the AC6 / AC10 allowance):
- `tests/develop_options.rs::full_active_area_with_a_warp_is_refused_before_any_write`:
  - The synthetic warp is now over-unity (`kr1 = +0.1`; the helper was renamed from
    `non_identity_warp` to `over_unity_warp`). DEC-059 admits the old `kr1 = −0.1` warp
    (scale [0.9, 1]).
  - Its two `matches!` patterns gained `..` for the new fields.
  - The Q2M L1021223 real-file block went from an expected refusal to an expected `Ok`:
    8392×5632, `dst` written. That frame now passes the bounds rule (AC5).
- `tests/develop_options.rs::refusal_display_names_the_active_size`: the variant literal
  carries its three new fields so it compiles. Its assertions are unchanged.
- `tests/develop_options.rs::full_active_area_dimensions`: a comment only ("which
  develop then refuses" was removed).
- `tests/error_kind.rs`:
  - an `InvalidWarpAlpha` row;
  - the `UnsupportedWarpedFullActiveArea` row's new fields;
  - CallerError 12 → 13, total 62 → 63.

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** 17,582,624. This is a floor, measured before the handback commit;
  the commit, push and CI-wait turns follow.
- **Estimated USD:** 13.06, at opus-5-5 per-component rates.
- **Duration (minutes):** about 75.
- **Source of the number:** the harness transcript
  `50667760-e761-4b85-9731-204e0e8a7891.jsonl`, deduped by `message.id` with the
  largest output per message. There are no subagents.
- **Breakdown:**
  - input 159;
  - cache-creation 238,214 (1-hour);
  - cache-read 17,262,494;
  - output 81,757.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **AC4's anchor.** The spec quotes 506.1 px (the active-area corner pull). The AC4
    fixture is at the crop size, where the analytic pull is 503.7 px. The test computes
    the value rather than hard-coding either number. The relation AC4 asks for (linear
    in α within 1 px) is unchanged.
  - **The bounds predicate also requires `kt` finite.** DEC-059 §3 says "every
    coefficient of the lerped warp".
  - **Red-proof 10.** The spec's NaN mutation, on its own, is caught by the unit test
    (AC9) and not by AC6. AC6's NaN case is still refused by the finiteness clause.
    Dropping that clause as well (red-proof 11) turns AC6 red.
- **Follow-up work identified:**
  - None new. The tier-B `warp_alpha` binary takes about 7 min in debug (AC5's 33
    develops of 47 MP). That is acceptable locally, and CI skips tier B.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The 506.1 px figure is for the active area, not the crop-size fixture AC4 names.
   It cost one failed run.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The corner-impulse fixture has a blind spot: when the warp loses the impulse,
   `max_by_key` on an all-zero plane returns the corner. `tests/warp.rs`'s
   `warp_moves_corner_impulse_to_spike_001s_source_coord` has the same shape and the same
   blind spot (it checks no peak value). That is worth a lesson signal.

3. **If you did this task again, what would you do differently?**
   — I would land API stubs first, so the red phase is behavioural rather than a
   compile failure. That would have caught the toothless AC4 before the red-proof pass
   instead of during it.
