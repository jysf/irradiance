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
  id: HANDOFF-137
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-12
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-065

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

# HANDOFF-137: BUILD: caller-owned develop scratch, allocation-free (SPEC-065)

## Delegation Summary

You are the **builder** for SPEC-065. Cut `claude/spec-065-build` from `origin/main`
**after** the design PR merges. The design PR carries the spec at `cycle: build`, DEC-060
and this handoff.

The task:
- Add `develop::scratch_len(sensor, &options)` and
  `develop::develop_with_scratch_into(sensor, src, dst, scratch, &options)`, plus
  `Error::DevelopScratchWrongLength`.
- Refactor `develop_with_into` into one private core that both functions call. The scratch
  form allocates nothing. `develop_with_into` allocates one `Vec` of exactly `scratch_len`.
  The private layout reuses the plane copy for the warp output, so the scratch is P + A,
  not P + 2A.
- Make `parse_fix_bad_pixels_constant` / `parse_warp_rectilinear` allocation-free walkers
  with identical results.
- Make `apply_fix_bad_pixels_constant` allocation-free.

Output and error order must be unchanged everywhere. The SPEC-063 MD5 pins must stay green
unedited.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-065-caller-owned-develop-scratch-scratch-len-plus-a-scratch-vari.md`.
  Read all of it:
  - the §1 probe (6 allocations / 283,943,504 bytes on a Q2M frame; expected
    `scratch_len` values);
  - Design §2–§7, especially §4's validation order and passes by case, and §5's walker
    equivalence rules;
  - AC1–AC13, the red-proof list, and the Notes.
- **Decision:** `./decisions/DEC-060-develop-scratch-is-one-exact-length-caller-slice-with-a-private-layout-and-the-scratch-path-allocates-nothing.md`.
  It covers the shape, the exact-length rule, the contract, the layout table, and the
  shared core.
- **Also:**
  - DEC-058 (options; validate then override);
  - DEC-059 (`warp_alpha`, and the `FullActiveArea` bounds check whose position is
    unchanged);
  - DEC-018/024 (levels, warp);
  - DEC-051 (wildcard-free `Error::kind`);
  - DEC-052;
  - DEC-003 (tier-B bytes are never committed).
- **Code:**
  - `src/develop.rs`: `develop_with_into` and its FBP copy and warp `vec!`s, plus
    `apply_fix_bad_pixels_constant`;
  - `src/opcode.rs`: `read_raw_opcode`, `parse_opcode_list`, the two lookups, and
    `check_list` as the walker pattern;
  - `src/lib.rs`.
- **Tests to mirror:**
  - `tests/previews_alloc.rs` for the counting allocator: its own binary, a thread-local
    switch, the negative control;
  - `tests/develop_options.rs` and `tests/warp_alpha.rs` for the tier-B gating, pins and
    sentinels;
  - `tests/opcode.rs`;
  - `tests/error_kind.rs`.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test` already
  passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Use the files in place and never copy them into the repo. Long tier-B binaries run in
  the background.

## Expected Deliverables

1. **Tests first.** Write `tests/develop_scratch.rs`, `tests/develop_scratch_alloc.rs`,
   the `tests/opcode.rs` equivalence test, and the `tests/error_kind.rs` row, as the
   spec's Failing Tests list says. Show them red; a compile failure counts for the new
   API. The AC8 equivalence test should **pass** on main, because it pins today's
   behaviour before the rewrite. Record that order in the handback.
2. **Implement it**, following Design §2–§5:
   - the walkers first, kept green by AC8;
   - then the allocation-free neighbours;
   - then the core, with the validation order exactly as §4 lists it, and the four
     cases with the layout from DEC-060 §3 (`split_at_mut`, no `unsafe`);
   - then the two public functions and the delegation.
3. **Fuzz (§6).** Add both differentials, and run `develop` and `opcode` for at least 60 s
   each. Report the run counts, and confirm that the existing seeds cover all four cases.
4. **Docs (§7).** CHANGELOG `[Unreleased]` (Added and Changed, with the Q2M figures),
   plus the `src/develop.rs` module and `develop_into` doc updates. No provenance row.
   Do **not** bump `DEVELOP_VERSION`.
5. **Gates, all green with the corpus env set:**
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

   Also run `just decisions-audit --changed`.
6. **Red-proofs.** For each mutation in the spec's red-proof list, apply it by hand, show
   the named test red, and revert. List them in the handback.
7. **PR.** Open a PR against `main` from `claude/spec-065-build` that references SPEC-065,
   DEC-060 and HANDOFF-137. Confirm CI is green on its head.

## Out of Scope

- Warping straight into `dst`.
- A colour-path or unpack scratch variant.
- Making `parse_opcode_list` allocation-free.
- CFA handling (SPEC-066) and the blocked transpose (SPEC-067).
- `irr` CLI flags, a release, or a `DEVELOP_VERSION` bump.

If one of these seems necessary, stop and say so in the handback rather than expanding
scope.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes or derived corpus pixels (DEC-003). MD5 strings and lengths in
  source are fine.
- If a SPEC-063 MD5 pin or an existing precedence test goes red, the change altered
  behaviour. Fix the change, never the test.
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
