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
  id: HANDOFF-174
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-083

project:
  id: PROJ-004
  stage: STAGE-026
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

# HANDOFF-174: VERIFY: irradiance 0.2.4 release prep, Phase 1 (SPEC-083)

## Delegation Summary

You are the **verifier** for SPEC-083. Review it cold: you did not build it.

- **Under review:** PR #287 (`claude/spec-083-build`, head 91119b9), built under
  HANDOFF-173. CI is green on all 22 checks.
- **Your branch:** `claude/spec-083-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

Phase 1 of the 0.2.4 release: two small script fixes, version 0.2.4, the CHANGELOG
promotion, the release notes, tier-B and SDK evidence, the package checks and the
pre-flight. Nothing is tagged or published; that is the user's Phase 2. Your verdict
decides whether the user tags this tree.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-083-irradiance-0-2-4-release-dng-conformant-black-normalisation.md`
  (Release Scope's "every input whose result changes" list, Build work 1–9, Release Notes,
  Pre-Flight, the Phase 2 runbook).
- **The model:** SPEC-077 (0.2.3) and its verify, HANDOFF-166. That verify found the release
  notes missed a newly refused input; look for the same class of miss here.
- **What's released:** DEC-070 (both §3 amendments), SPEC-078, SPEC-080, and the scripts from
  SPEC-079/080/083.
- **The SDK:** `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Write outputs into your scratchpad only, never into any checkout.
- **Toolchain:** AGENTS.md §5–6. Run `just test`, never `just test --all-features`. Set
  `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Run long suites in the foreground, or keep working until they finish; don't end your turn
  while a run is still going.

## What to check hard

1. **Semver, and what it can't see.** Re-run `just semver-check`, default and
   `--all-features`. Then read `git diff v0.2.3 origin/claude/spec-083-build -- src/`
   yourself, every `Err(` path and every changed return value:
   - no `pub` item removed or changed; no new dependency, feature or MSRV change;
   - list every input whose result changes since `v0.2.3` (refusal, error field, output),
     and check the release notes name each one. The design found one new refusal (colour
     `W ≤ B`), one changed field (`PerPosition` `black_level`), and the output change. Is
     anything else there, for example a changed `Display` text?
   - Judge whether the notes' "Breaking" line about the `black_level` field is right to
     include, and whether anything should push this to 0.3.0.
2. **The src diff since the build branch point is doc-only.** `git diff origin/main --
   src/` on the candidate touches nothing (or comments only). `DEVELOP_VERSION` is `"7"`.
3. **The CHANGELOG.** `[0.2.4]` holds every library-visible change since `v0.2.3`
   (`git log v0.2.3..origin/main`, SPEC-078, 079, 080): nothing dropped in the promotion,
   the lead paragraph accurate, an empty `[Unreleased]` on top. The SPEC-080 entry's
   corrected wording names `BlackLevels::PerPosition`.
4. **The release notes are true.** Check every claim against code or evidence: the headline,
   "within one code before the GainMap" (which files, which comparison), the Pixel 7 Pro
   limitation, "byte-identical" for uniform-black and `FirstValue` files. Nothing overstated;
   the SPEC-080 wording caution respected (colour = `PerPosition`, not `FirstValue`).
5. **README and crate docs.** The README roadmap edit (writer moved to PROJ-005, optical
   black parked) is accurate; version strings say 0.2.4 where they should; nothing still
   says the colour path refuses nothing or scales by `W − b`. `cargo test --doc
   --all-features` and `just doc-check` pass.
6. **The package.** Run `cargo publish --dry-run` and `cargo package --list` yourself (in a
   clean scratch worktree if the tree is dirty). 19 files, the same list as the published
   0.2.3 `.crate`; no SDK, corpus or `scripts/` bytes; no self dev-dependency;
   `.cargo_vcs_info.json` names the candidate.
7. **The evidence reproduces.** `just test` with the corpus (12/12 present, counts match);
   a fresh `scripts/sdk-stage2.sh` run plus the cross-check (numbers equal SPEC-078's);
   `just msrv`, `just deny`, `just package-targets-check`.
8. **The script fixes.** Read both. Reproduce at least the CRLF, quoted-status and BOM cases
   for `validate.sh` and the unenterable-directory case for `sdk-stage2.sh`, on scratch
   copies. Confirm `validate` still fails a genuinely bad status, and that the BOM refusal
   doesn't trip on any real file.
9. **The pre-flight and the Phase 2 runbook.** Every `[now]` item has real evidence; every
   `[cut]` item is runnable as written; the commands are copy-paste correct (tag name,
   `cargo add irradiance@=0.2.4`, the `.crate` curl URL for 0.2.4, the yank command). The
   runbook asks the user to report the `[cut]` checks back.
10. **Nothing forbidden is committed:** no SDK bytes or outputs, no corpus bytes, no
    `Cargo.lock`.
11. **Gates:** every gate in Build work 9; CI.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence.
- **Commit this handoff's filled handback** to `claude/spec-083-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-083-build`.
- **A punch-list fix** is fine on your branch if it is docs, release notes, CHANGELOG or
  scripts only. Any `src/` change goes in the punch list.

## Out of Scope

- Tagging, publishing, or a GitHub release.
- Any library code change.
- The STAGE-026 and PROJ-004 closes.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

---

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
