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
  id: HANDOFF-166
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-19
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-077

project:
  id: PROJ-004
  stage: STAGE-024
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

# HANDOFF-166: VERIFY: irradiance 0.2.3 release prep, Phase 1 (SPEC-077)

## Delegation Summary

You are the **verifier** for SPEC-077, the 0.2.3 release prep. Review it cold: you did
not build it.

- **Under review:** PR #272 (`claude/spec-077-build`, head 228a76e; release candidate
  `cf8b252`), built under HANDOFF-165. CI was green on all 22 checks.
- **Your branch:** `claude/spec-077-verify-base`. It is that head plus one commit that
  advances the spec to verify and adds this handoff.

This is the last check before the user tags `v0.2.3` and runs `cargo publish`, which
can't be undone (only yanked). Your job is to find anything that would make the
published crate or its notes wrong.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-077-irradiance-0-2-3-release-gainmap-and-black-level-fixes.md`
  (Release Scope, Build work 1–8, Release Notes, Release Pre-Flight, Phase 2 runbook).
- **Build handback:** HANDOFF-165's `## Handback`. Check each claim yourself.
- **Precedent:** SPEC-073 (0.2.2) in `specs/done/`, and its verify handoff.
- **Decisions:** DEC-006, DEC-052, DEC-067, DEC-068, DEC-069.
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  - `cargo-semver-checks` is installed. `just semver-check` uses the crates.io baseline.

## What to check hard

1. **Semver.** Re-run `just semver-check` yourself, default and with `--all-features`.
   Then independently read `git diff v0.2.2 origin/claude/spec-077-build -- src/` for
   public items. Confirm three things:
   - no `pub` item was removed or had its signature changed;
   - every new public field or variant is on a `#[non_exhaustive]` type;
   - no new dependency or feature was added, and the MSRV is unchanged.

   Name anything the tool cannot see, such as a changed `Display` text, a newly refused
   input, or changed output. Judge whether each belongs in "Behaviour changes" rather
   than requiring 0.3.0.
2. **The src diff is doc-only.** `git diff origin/main -- src/` must touch comments
   only. Confirm it, and that `DEVELOP_VERSION` is `"6"` on the candidate.
3. **The CHANGELOG is complete and accurate.**
   - Every library-visible change since `v0.2.2` has an entry under `[0.2.3]`. Cross-check
     against `git log v0.2.2..origin/main` and the shipped specs: SPEC-074, 075, 076,
     059 and 060.
   - Nothing was dropped in the promotion, and the merged `DEVELOP_VERSION` entry is
     correct.
   - An empty `[Unreleased]` sits on top.
4. **The release notes are true.** Check every claim against the code or evidence: the
   headline, the `dng_validate` figure, the behaviour changes, and "every other file's
   output is byte-identical". Nothing should be overstated, such as a colour-accuracy
   claim.
5. **README and crate docs.**
   - No remaining text says a Pixel `GainMap` is refused or unsupported. Grep for
     `GainMap`, `Pixel`, `opcode 9` and `UnsupportedMandatoryOpcode`.
   - Version strings say 0.2.3 where they should.
   - The new README sentence wraps like the surrounding text.
   - `cargo test --doc --all-features` and `just doc-check` pass.
6. **The package.**
   - Run `cargo publish --dry-run` yourself, and `cargo package --list`.
   - The file list contains no SDK, corpus or `scripts/` bytes and no tier-B fixtures.
   - The packaged `Cargo.toml` has no self dev-dependency.
   - `.cargo_vcs_info.json` names the candidate.
   - Compare the file list with 0.2.2's (19 files) and explain any difference.
7. **The evidence reproduces.** Run `just test` with the corpus and confirm 12/12
   present, with counts that match the handback's. Also run `just msrv`, `just deny` and
   `just package-targets-check`.
8. **The pre-flight and the Phase 2 runbook.**
   - Every `[now]` item is marked with real evidence.
   - Every `[cut]` item is something the user can actually run as written.
   - The commands are copy-paste correct: the tag name, the `cargo add irradiance@=0.2.3`
     form, the `.cargo_vcs_info.json` curl URL, and the yank command.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above.**
- **Commit this handoff's filled handback** to `claude/spec-077-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-077-build`.
- **A punch-list fix** is fine on your branch if it is docs, CHANGELOG or spec text only.
  Any `src/` code change, or a version change, goes in the punch list for the user.

## Out of Scope

- The tag, `cargo publish` and a GitHub release (the user's Phase 2).
- STAGE-016's carried follow-ups.
- Any library behaviour change.

## Hard rules

- **Never** tag, publish, yank, force-push, merge, or enable auto-merge.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

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
