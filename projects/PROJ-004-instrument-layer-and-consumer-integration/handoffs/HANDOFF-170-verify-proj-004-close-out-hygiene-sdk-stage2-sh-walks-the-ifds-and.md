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
  id: HANDOFF-170
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
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

# HANDOFF-170: VERIFY: PROJ-004 close-out hygiene (SPEC-079)

## Delegation Summary

You are the **verifier** for SPEC-079. Review it cold: you did not build it.

- **Under review:** PR #279 (`claude/spec-079-build`, head 8abd264), built under
  HANDOFF-169. CI was green or finishing (20 of 22 checks passed, 2 pending, when this
  was written). Check it.
- **Your branch:** `claude/spec-079-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-079 changes no `src/`. It changes `scripts/sdk-stage2.sh` (an IFD walk, the output
directory created after the in-repo check), `tests/gain_map_corpus.rs` (set means must
run), `tests/black_normalisation.rs` (FU-2 fixtures), `docs/provenance-ledger.md` and
`feedback/2026-11-11-monorail-000-requests.md`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-079-proj-004-close-out-hygiene-sdk-stage2-sh-walks-the-ifds-and.md`
  (Context 1–6, Design 1–6, AC1–AC7).
- **Decisions:** DEC-069 §6, DEC-070 (§1, and §3 as amended at SPEC-078's ship).
- **Build handback:** HANDOFF-169's `## Handback`. Check each claim yourself.
- **The SDK:** `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`;
  the zip at `../dl/dng_sdk_1_7_1.zip`. Generate your own outputs into your scratchpad.
  The builder notes the script finds the repo root from its own location, so run the old
  script (from `origin/main`) in place, from a checkout of main, not from a copy.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The IFD walk is correct and safe.** Read the Perl.
   - Header, `NextIFD` chain and `SubIFDs` recursion; every offset and count is
     bounds-checked before it is read, in both byte orders; `SubIFDs` with a count > 1
     and with a count of 1 (value inline) are both handled.
   - Exactly one 51009 entry, type 7; the rewrite touches only that entry's 2 tag bytes.
   - A revisited IFD is refused. Is refusing a legitimate DAG (two pointers to one IFD)
     acceptable for this tool? Say so either way.
   - Reproduce AC1 cold: offsets, byte-identical outputs from old and new scripts, and the
     cross-check numbers. Reproduce at least four of AC2's six scenarios, including the
     decoy and the cycle, with your own crafted copies.
   - Craft one case the builder did not: for example an entry count of 0, an IFD offset
     that is odd or points inside the header, or a `SubIFDs` entry whose value offset is
     out of range. It must be refused cleanly.
2. **The directory order (AC3).** Reproduce the in-repo refusal leaving nothing behind,
   including through a symlink that resolves into the repo, and a relative path.
3. **Set means must run (AC4).**
   - The helper is shared by all three SDK tests, and no SDK test still has a
     `continue`/`return` on a missing corpus file.
   - The non-SDK tests in the file still skip on a missing corpus.
   - Reproduce the by-hand run: SDK variable set plus an empty corpus directory fails all
     three; unset passes with SKIP. What happens with the variable set to an empty
     string? It should behave as unset, as designed; confirm.
4. **FU-2 (AC5).** Re-run the three designed mutants on tier A (no corpus) with an
   unmutated control first. Check the builder's deviation (the general-path maximum at
   every one of 16 indices, rather than index 0 only) is sound and drops nothing the
   design asked for. Do the new tests compute their expected values independently of the
   crate (no call into a crate helper for `B`)?
5. **No existing assertion moved.** `git diff origin/main -- tests/ src/`: only additions,
   plus the loader refactor in `gain_map_corpus.rs`. Confirm the refactor changes no
   non-SDK behaviour.
6. **Records (AC6).**
   - Re-check the licence finding yourself: list the zip, grep its text files and PDFs
     (`pdftotext`) for "licen" and "agreement". Confirm the ledger and script header
     state facts only, with the zip's sha256, and no legal opinion.
   - The feedback file: front matter `status: addressed`, items 5 and 8 correct against
     the CHANGELOG (`[0.2.2]`, `[0.2.3]`), nothing else in the consumer's text changed
     (`git diff`). Parse its front matter.
   - The builder notes `just validate` does not scan `feedback/` or `docs/`; confirm, and
     say whether it matters here.
7. **Nothing forbidden is committed:** no SDK bytes, outputs or scratch copies; no corpus
   bytes; no `Cargo.lock`.
8. **Gates:** `just test` with the corpus, with and without the SDK variable; `lint-ci`,
   `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`, `gate-audit`,
   `name-length-check`, `decisions-index --check`; CI.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including your own scenario table and mutant table.
- **Commit this handoff's filled handback** to `claude/spec-079-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-079-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. A
  measured number outside a pre-registered bound is reported, never fixed by widening it.

## Out of Scope

- FU-1 (a colour levels refusal).
- Any `src/` change.
- The brief's stage count and the STAGE-025 reflection.
- A release.

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
