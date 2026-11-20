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
  id: HANDOFF-169
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-079

project:
  id: PROJ-004
  stage: STAGE-025
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

# HANDOFF-169: BUILD: PROJ-004 close-out hygiene (SPEC-079)

## Delegation Summary

You are the **builder** for SPEC-079. **Start only after the SPEC-079 design PR has
merged.** Cut `claude/spec-079-build` from that `origin/main`.

**The task:** close PROJ-004's carried follow-ups, with no library code change:
- `scripts/sdk-stage2.sh` finds `OpcodeList2` by walking the IFDs, not by a byte
  pattern, and creates its output directory only after the in-repo check;
- with `IRRADIANCE_SDK_STAGE2_DIR` set, a missing corpus file fails the SDK
  cross-check instead of skipping it;
- the provenance ledger and the script header record that the SDK zip ships no licence
  agreement document;
- the monorail triage record is brought up to date and marked `addressed`;
- FU-2 from SPEC-078's verify: tier-A colour and mono fixtures with the maximum black at
  each slot, and a general-path one with it at index 0.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-079-proj-004-close-out-hygiene-sdk-stage2-sh-walks-the-ifds-and.md`
  (Context 1–6 as found at design, Design 1–6, AC1–AC7).
- **Decisions:** DEC-069 §6 (the SDK reference and the visible skip), DEC-070 (the
  rule FU-2 tests).
- **Verify finding behind FU-2:** HANDOFF-168's `## Handback`, item 1 and FU-2.
- **Code and records:**
  - `scripts/sdk-stage2.sh`;
  - `tests/gain_map_corpus.rs` (`sdk_dir`, `sdk_stage2`, `corpus_plane` and the three
    SDK tests);
  - `tests/black_normalisation.rs` and `tests/colour_black_pattern.rs` (fixture shapes);
  - `docs/provenance-ledger.md` (the SPEC-059 `GainMap` row and the SPEC-078
    black-normalisation row);
  - `feedback/2026-11-11-monorail-000-requests.md` (front matter and the Resolution
    table only).
- **The SDK:**
  - `dng_validate` is at `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`;
    the zip is in `../dl/`.
  - Run the **old** script once into one scratchpad directory and keep its outputs, then
    the new one into another, and compare (AC1).
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place. Never copy them into the repo.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## Expected Deliverables

1. **The changes** for AC1–AC6, in a PR against `main` from `claude/spec-079-build`, with
   CI green.
2. **The handback:**
   - AC1: each Pixel's rewrite offset, old and new; the sha256 of every reference TIFF
     from both scripts; the cross-check numbers;
   - AC2 and AC3: each scenario and its outcome, with the message printed;
   - AC4: the unit tests, and the by-hand run with an empty corpus directory;
   - AC5: the mutant table (the control first, then each mutant and the tier-A tests
     that go red).
3. **Gates**, all green: AC7.

## Out of Scope

- FU-1, a colour levels refusal (next project).
- Any `src/` change, pin change, `DEVELOP_VERSION` change or CHANGELOG entry.
- The brief's stage count and the STAGE-025 reflection (the stage close does them).
- `BlackLevelDeltaH/V`.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus bytes, SDK bytes, SDK outputs or scratch copies.
- **An existing assertion may be strengthened, never weakened or deleted.** If any
  existing test or pin has to move, stop and report before changing it.
- **The ledger states facts only** (what the zip contains, what irradiance does with
  the SDK). Give no legal opinion.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill
  the `## Handback` section.

---

## Handback` section below.** An unfilled handback is an
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
