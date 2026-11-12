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
  status: pending                  # pending | accepted | completed | rejected

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
