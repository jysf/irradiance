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
  status: pending                  # pending | accepted | completed | rejected

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
