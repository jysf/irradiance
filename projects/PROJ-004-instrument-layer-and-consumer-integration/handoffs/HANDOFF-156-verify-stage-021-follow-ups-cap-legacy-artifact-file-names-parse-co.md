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
  id: HANDOFF-156
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-18
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-074

project:
  id: PROJ-004
  stage: STAGE-022
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

# HANDOFF-156: VERIFY: STAGE-021 follow-ups, bundled (SPEC-074)

## Delegation Summary

You are the **cold verifier** for SPEC-074. You did **not** implement it. Review it
against the spec, and return exactly one verdict.

- **The build:** **PR #259**, branch `claude/spec-074-build`, cut from `origin/main` at
  `073b594`. Its head is `f56f5eb`, in four commits:
  - `74021de` Part B: one Compression 7 header parse (`src/ljpeg.rs`, `src/plane.rs`,
    the pin at 5, CHANGELOG);
  - `c37a3b2` Part A: renames only (83 legacy artifacts) plus reference rewrites;
  - `9f3a724` Part A: `name-length-check`, its CI step, `semver-check`, AGENTS.md §6;
  - `f56f5eb` the handback.
- **Your branch:** your worktree is cut from `claude/spec-074-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-074-stage-021-follow-ups-cap-legacy-artifact-file-names-parse-co.md`.
  Read all of it: the probe findings, A1–A3, B1–B3, AC1–AC7, Non-Goals and the Notes.
- **Decision:** `decisions/DEC-067-…`. Also DEC-047 (shape binding first) and DEC-009
  (red-proof, control first).
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-155-build-stage-021-follow-ups-cap-legacy-artifact-file-names-parse-co.md`.
  Treat its claims as claims. The points to judge:
  - **Error identity in Part B.** Read the new `pub(crate)` functions and the two plane
    call sites. Confirm the order is unchanged: parse, then the shape check, then the
    scan from the same entropy offset. Confirm `decode_into` and `decode_window_into` are
    thin wrappers with unchanged behaviour. Look for any path where an error variant or
    field could now differ.
  - **The deviation:** `open_scan` kept its name with a new signature, because
    `tests/plane_tiles.rs` uses `fn open_scan` as the end anchor of its no-allocation
    source scan. Check that the new functions really sit inside that scanned region,
    and that the scan still means what it meant.
  - **The renames.** Re-derive the expected set yourself: every tracked
    `SPEC-`/`HANDOFF-`/`STAGE-`/`PATCH-`/`SPIKE-` file at `073b594` whose slug exceeds
    60 characters, with the new name per `scripts/_lib.sh` `slugify`. Diff that against
    commit `c37a3b2`'s renames (`git show -M --name-status c37a3b2`). Confirm:
    - the commit is pure renames plus reference rewrites (no content edits in renamed
      files, so rename similarity is 100% except where a reference inside it changed);
    - spec and timeline pairs share a slug;
    - no old name remains except the history quotes the handback lists.
  - **The gate.** Reproduce `name-length-check`'s red-proof yourself: 151 bytes red,
    150 green, control first. Confirm the CI step is in the `cost-data` job and ran on
    #259.
  - **AC3.** Run `cargo semver-checks --baseline-rev HEAD` and `just semver-check`.
- **Toolchain:**
  - AGENTS.md §5–6. `just test` takes about 3 minutes (the corpus profile plus the debug
    `*_alloc` binaries).
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.

## The verify contract

### Acceptance criteria and gates
- **Check that each of AC1–AC7 is met** by evidence that would fail if the property
  broke.
- **Run the gate suite yourself, with the corpus env set:**
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv`
  - `cost-audit`
  - `gate-audit`
  - `name-length-check`
  - `decisions-index --check`
- **Observe CI** green on the PR head with `gh pr checks 259`.

### The behavioural surface
- **Test edits are only the ones permitted.** `git diff 073b594 -- tests/` may show only
  `C7_ALLOCATIONS_PER_CHUNK`, its comment and its assertion message in
  `plane_units_alloc.rs`.
- **Every Compression 7 oracle and MD5 pin is unedited** and green, tier A and tier B.
- **No public item changes.**

**Mutate it yourself** (on an archived copy or local commits you never push):
- restore the double parse in one plane path only (strip, then tile): the pin goes red
  with the expected counts;
- swap the order so the scan runs before the shape check: does any test notice? If none
  does, report it as a gap (a finding, not necessarily a punch-list item);
- pass a wrong entropy offset (for example 0) into the new function: the oracles go red;
- in the gate, change the limit to 151: the 151-byte name passes, which shows the
  red-proof is what pins the limit.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved with no code changes:**
  - Do **not** open a PR that supersets #259.
  - Commit the filled HANDOFF-156 handback (and the timeline's verify line) on a branch
    named `claude/spec-074-verify-<suffix>`, and push it.
  - Post the verdict as a comment on PR #259 with the branch name.
- **Punch list with fixes you make yourself:**
  - Open a NEW PR against `main` that supersets #259.
  - Reference HANDOFF-156, SPEC-074 and PR #259.
- **Rejected:** comment on #259 with the reasons, and push the handback branch.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- Changing `slugify` or any template-managed script.
- Renaming DECs.
- A semver CI gate.
- Any change to the scan reader or decode output.

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
