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
  id: HANDOFF-133
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-12
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-063

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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 18485591          # REAL combined count — what cost-audit reads
  estimated_usd: 12.92            # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 90
  branch: claude/spec-063-build
  pr: 214
  completed_at: 2026-11-12        # YYYY-MM-DD
  notes: "TRUE cost re-summed at ship by the orchestrator from the ended build transcript (acc02f26, 1 jsonl, 87 unique message.ids, all claude-opus-5-5, no subagents): input 179 + cache-creation 220,830 (1-hour) + cache-read 18,200,327 + output 64,255 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). The handback floor was 16,105,570 / 11.56 before the final commit, push and CI-wait turns."
  synced_at: 2026-11-12
---

# HANDOFF-133: BUILD: DevelopOptions + develop_with_into (SPEC-063)

## Delegation Summary

You are the **builder** for SPEC-063. Cut `claude/spec-063-build` from `origin/main`
**after** the design PR merges (it carries the spec at `cycle: build`, DEC-058 and this
handoff).

The task: add `DevelopOptions` (`framing`: `StandardCrop` | `FullActiveArea`;
`apply_orientation`) plus `develop_with_into` and `output_dimensions_with` to
`src/develop.rs`. Make `develop_into` / `output_dimensions` delegate with the default.
Add one refusal, `Error::UnsupportedWarpedFullActiveArea`, for `FullActiveArea` with a
non-identity warp.

Default output must stay byte-identical. The spec pins MD5s of `develop_into` output on
eight held files, taken from main before your change.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-063-developoptions-develop-with-into-non-exhaustive-options-with.md`.
  Read all of it: the §1 probe table and MD5 pins, Design §2 to §7, and AC1 to AC13 with
  the red-proof list.
- **Decision:** `./decisions/DEC-058-developoptions-is-a-non-exhaustive-struct-whose-default-is-develop-into.md`.
  It covers the shape, default is `develop_into`, validate-then-override, typed refusal,
  and the `DEVELOP_VERSION` rule.
- **Also:** DEC-018/019 (levels, crop relative to `ActiveArea`), DEC-024 (warp clamps to
  edge, which is why §4 refuses), DEC-028 (`DEVELOP_VERSION`), DEC-051 (wildcard-free
  `Error::kind`), DEC-052 (`#[non_exhaustive]`), DEC-003 (tier-B bytes never committed).
- **Code:** `src/develop.rs`:
  - `resolve_geometry` at 139;
  - `crop_orient_normalize_into` at 559;
  - `normalize_active_area_into` / `crop_and_orient_from_active_into` at about 626 to 700;
  - `develop_into` at 802.

  Also `src/lib.rs` (`Error`, `kind()`, `DEVELOP_VERSION` at 158) and `src/warp.rs`
  (`is_identity`).
- **Tests to mirror:**
  - `tests/develop.rs` (`minimal_sensor`, `corpus_sensor`, the warp-branch fixtures from
    about line 1097);
  - `tests/support/opcode.rs` (`single_warp_rectilinear_list`,
    `single_fix_bad_pixels_constant_list`);
  - `tests/support/md5.rs`;
  - `tests/error_kind.rs`.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test` already
  passes `--all-features`; add no flag. For tier B set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`, use the files in
  place, and never copy them into the repo.

## Expected Deliverables

1. **Tests first.** Write `tests/develop_options.rs` with every test the spec's Failing
   Tests list names, plus the `tests/error_kind.rs` row. Show them red (compile failure
   counts for the new API, but the AC2 MD5 test must compile against `develop_into` alone
   and pass on main: it is a pin, not a red test). Record that order in the handback.
2. **Implement it.** Follow Design §2–§5:
   - the types;
   - `apply_options` as a rewrite of the resolved `Geometry`, with no new branches in the
     three passes;
   - the refusal, placed exactly where §4 orders it: before the `FixBadPixelsConstant`
     copy and before any `dst` write, with `StandardCrop`'s parse order untouched;
   - the delegation.
3. **Fuzz (§6).** Extend the `develop` target with the option byte and add the seeds via
   `examples/fuzz-seeds.rs`. Run it for at least 60 s and report the run count.
4. **Docs (§7).** CHANGELOG `[Unreleased]`, a provenance-ledger row, and the module doc
   paragraph. Do **not** bump `DEVELOP_VERSION`.
5. **Gates, all green with the corpus env set:** `just test`, `lint-ci`, `lint-no-allow`,
   `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`. Also
   run `just decisions-audit --changed`.
6. **Red-proofs.** For each mutation in the spec's red-proof list, apply it by hand, show
   the named test red, and revert. List them in the handback.
7. **PR.** Open a PR against `main` from `claude/spec-063-build` that references SPEC-063,
   DEC-058 and HANDOFF-133, and confirm CI is green on its head.

## Out of Scope

- `warp_alpha`, `WarpRect::lerp`, or relaxing the refusal (SPEC-064).
- Removing internal allocations or adding a scratch API (SPEC-065).
- CFA handling of any kind (SPEC-066). The mono path's acceptance of CFA files stays
  exactly as it is.
- The blocked transpose (SPEC-067).
- The colour path, `irr` CLI flags, a release, or a `DEVELOP_VERSION` bump.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). The MD5 strings in the spec
  are fine.
- If an AC2 pin goes red, your change altered default output. Fix the change; never edit
  the pin.
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

- **Branch / PR:** `claude/spec-063-build` → https://github.com/jysf/irradiance/pull/214 (build commit c2c028a; this handback in a follow-up commit on the same branch)
- **Completed at:** 2026-11-12
- **All acceptance criteria met?** yes, except the impossible half of AC9 (see Deviations). AC1–AC8, AC10–AC13 met as written.
- **Order of work (test-before-implementation):**
  1. Wrote `tests/develop_options.rs` (the 13 named tests) and the `tests/error_kind.rs` row (Unsupported 20→21, total 61→62).
  2. Showed them red. Before any `src/` change, `cargo test --no-run` failed only on the new API: unresolved `develop_with_into` / `output_dimensions_with` / `DevelopOptions` / `Framing`, plus `no variant UnsupportedWarpedFullActiveArea`.
  3. Showed the AC2 pin green on unchanged main: a throwaway copy of the AC2 loop (`develop_into` only) passed all 8 MD5s with `src/` untouched. It was deleted, never committed.
  4. Implemented, then re-ran: all 15 tests (13 + 2 support) and `error_kind` green with the corpus.
- **Gates** (with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`), all green: `just test`, `lint-ci` (clippy 0.1.98 pinned), `lint-no-allow`, `fmt-check`, `validate`, `doc-check` (0 rustdoc warnings, 2 feature surfaces), `deny`, `msrv`, `cost-audit`, `gate-audit`. `decisions-audit --changed` is advisory only.
- **Fuzz:** `just fuzz-develop 60` ran **4,672,611 runs in 61 s, zero crashes**. The existing develop seeds regenerated byte-identically; there are 8 new `options-*` seeds.
- **Red-proofs.** Each mutation was applied by hand to `src/develop.rs`, the named tests went red, then I reverted with `git checkout`. The tree was clean afterwards.
  - Default `framing = FullActiveArea`: AC1 `default_options_are_standard_crop_and_oriented` red; AC2 `develop_into_output_is_unchanged_on_held_files` red on tier B (L1021223 dimensions (8392, 5632) vs (8368, 5584)); also AC3, AC5, AC6 and AC9 red.
  - `apply_options` skipped (both call sites): AC3 `full_active_area_dimensions`, AC4 (both), AC6 all-eight, AC7, AC9 and AC5 red.
  - Crop override keeps the crop size (origin zeroed only): AC4 (both), AC3, AC7, AC9 and AC5 red.
  - Refusal removed: AC5 `full_active_area_with_a_warp_is_refused_before_any_write` red.
  - Refusal moved after the `dst` write: AC5 red (sentinel and order check).
  - Refusal moved after the `OpcodeList1` parse / FixBadPixels copy, but before any `dst` write: AC5 red at the order check. A `FixBadPixelsConstant` with a malformed 4-byte parameter block reports `MalformedOpcodeParams` first. The sentinel alone cannot see this placement; the order assertion is what pins "before the copy".
  - `apply_orientation` ignored: AC6 all-eight red; AC6 tier-B `unoriented_q2m_orientation_six_through_the_warp_path` red ((5584, 8368) vs (8368, 5584)); AC7 and AC9 red.
  - Validation bypassed under an option (crop / orientation tags cleared on a clone before `resolve_geometry`): AC8 `options_do_not_skip_tag_validation` red.
  - `dst` checked against the default dimensions: AC9 `dst_length_is_checked_against_the_options_dimensions` red, plus AC4, AC5 and AC7 red.

### Cost self-report

- **Tokens (total):** see front-matter `tokens_total`. It was re-summed from this session's own transcript just before the handback commit, so it is a floor that excludes the final commit/push turns.
- **Estimated USD:** see front-matter.
- **Duration (minutes):** see front-matter.
- **Source of the number:** transcript (`acc02f26-d478-43e3-8460-fb25d315e857.jsonl`, the scratchpad UUID; no subagents). Deduped by `message.id`, max `output_tokens` per message, opus-5-5 list rates (in 5, out 25 per MTok; cache read ×0.10; 1-hour cache write ×2.0).

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - **AC9, second half, is unsatisfiable as written.** It asks that a `dst` sized for the default dimensions, passed with `apply_orientation: false` on Orientation 6 with a non-square crop, be `DevelopBufferWrongLength`. But the length check compares a product, and `w×h == h×w`, so that buffer is the right length. The test instead pins that the buffer is accepted and receives the unswapped layout (equal to the Orientation 1 develop). The `FullActiveArea` half is tested as written, in both directions.
  - **AC13 "no allocation under `FullActiveArea` before the refusal".** The only work before the refusal is the existing `parse_warp_rectilinear` → `parse_opcode_list`, which returns a small `Vec<Opcode>`. That parse predates this spec and runs on every warp path. It now runs earlier, under `FullActiveArea` only, and exactly once: under `FullActiveArea` the later parse is skipped. No plane-sized allocation happens before the refusal, and the `src/` diff adds no allocation.
  - **AC2 test runtime.** In debug it takes ~3 min: 16 full develops, three of them warped 47 MP Q2M frames ×2, plus the 8 unpacks. AC2 requires both entry points on all eight files, so this was kept.
- **Follow-up work identified:**
  - SPEC-063's AC9 text should be corrected at ship or in a spec amendment (orientation cannot change a length).
  - None new beyond STAGE-019's existing backlog (SPEC-064..067). No pass needed an option-specific branch; the geometry rewrite covered all three.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only AC9's orientation half. As a length check it cannot fail. Everything else was transcription-ready: the §1 probe table and pins made AC2–AC6 mechanical.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The red-proof list's "refusal moved after the FixBadPixelsConstant copy" is invisible to a `dst` sentinel, because the copy writes no `dst`. It needs an error-precedence probe (a malformed `OpcodeList1`). The spec could name that technique next to the mutation.

3. **If you did this task again, what would you do differently?**
   — Run the red-proof loop with the corpus env set from the start (my first pass skipped tier B silently), and start `just test` (debug, ~10 min with corpus) in the background earlier.
