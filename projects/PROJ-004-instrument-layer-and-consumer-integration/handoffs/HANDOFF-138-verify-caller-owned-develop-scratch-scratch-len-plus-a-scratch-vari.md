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
  id: HANDOFF-138
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier
  created_at: 2026-11-13
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

# HANDOFF-138: VERIFY: caller-owned develop scratch, scratch_len plus a scratch variant of develop (SPEC-065)

## Delegation Summary

You are the **cold verifier** for SPEC-065. You did **not** implement it. Review it
against SPEC-065 and DEC-060, and return exactly one verdict.

- **The build:** **PR #224**, branch `claude/spec-065-build`, cut from `origin/main` at
  `63bab3f`. The head when this handoff was minted is `fa2c01f`. Its commits are:
  - `3446123`: failing tests first;
  - `06d486c`: the implementation;
  - `2f28be1`: core-independent oracles;
  - `71e50df`: the scratch fuzz seeds;
  - then the handback.
- **Your branch:** your worktree is cut from `claude/spec-065-verify-base`. That is the
  build head plus this handoff and the spec's `cycle: verify`, so the code under review is
  the build's code.

The build delivers:
- **`scratch_len` and `develop_with_scratch_into`** in `src/develop.rs`, both over one
  core. That core is `develop_core` (around line 1183), with a
  `Scratch::{Caller, Allocate}` source (around line 1120). `run_passes` (around line 1391)
  carries out the DEC-060 §3 layout through `split_at_mut_checked`.
- **Allocation-free opcode walkers** in `src/opcode.rs`: `first_recognised`, backing
  `parse_warp_rectilinear` and `parse_fix_bad_pixels_constant`. The neighbours buffer
  becomes `[u16; 8]`.
- **`Error::DevelopScratchWrongLength`** in `src/lib.rs`.
- **Tests:**
  - `tests/develop_scratch.rs`;
  - `tests/develop_scratch_alloc.rs`, its own binary with a counting allocator;
  - `walkers_equal_parse_opcode_list_then_find_map` in `tests/opcode.rs`;
  - `tests/error_kind.rs` rows.
- **Fuzz:** differentials in the `develop` and `opcode` targets, and five new
  `fuzz/seeds/develop/scratch-fix-*.tiff` seeds.
- **Docs:** CHANGELOG `[Unreleased]`.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-065-caller-owned-develop-scratch-scratch-len-plus-a-scratch-vari.md`.
  Read all of it: the §1 probe (6 allocations / 283,943,504 B on Q2M), Design §1–§7,
  AC1–AC13, the red-proof list, and the Notes.
- **Decisions:**
  - `./decisions/DEC-060-develop-scratch-is-one-exact-length-caller-slice-with-a-private-layout-and-the-scratch-path-allocates-nothing.md`
    — the contract. Pay most attention to §2, which sets out no allocation on any path,
    contents that do not matter, identical output, and the error order.
  - DEC-058 (`DevelopOptions`, validate-then-override, the order of refusals);
  - DEC-059 (`warp_alpha`, the `FullActiveArea` bounds rule);
  - DEC-051 (wildcard-free `Error::kind`);
  - DEC-052 (`#[non_exhaustive]`);
  - DEC-003 (tier-B bytes are never committed).
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-137-build-caller-owned-develop-scratch-scratch-len-plus-a-scratch-vari.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation is sound:
  - **Seeds were added.** The spec said none were needed, but no develop seed carried
    `FixBadPixelsConstant`. Confirm that the five new seeds reach the P and P + A layouts,
    by classifying each seed with `scratch_len` yourself.
  - **The composition oracle.** `each_case_equals_its_composition_from_public_parts` was
    added because AC2's byte-identity relation shares the core under test and is blind to
    a bug inside it. Confirm that this oracle is **independent of `develop_core`**: it
    composes the public `apply_fix_bad_pixels_constant` with the warp-only develop. Then
    ask whether the warp-only path is itself checked by something outside the core (the
    tier-B pins, the SPEC-064 warp tests). Mutations in the core that only break the warp
    pass are the place to look.
  - **`fix_bad_pixels_median_with_fewer_than_eight_neighbours`** was added. It pins corner
    and edge pixels with hand-computed values. Re-derive one value yourself.
  - **The AC7 table** has 27 rows rebuilt from three test files. Check that it covers
    every distinct refusal shape `develop_with_into` can produce, and that no hostile
    fixture was dropped.
  - **The overflow in AC10** is reachable on 64-bit and pinned by
    `scratch_len_overflow_is_typed`. Check it reports `expected: u64::MAX` as DEC-060 §3
    says.
  - **The allocation in `develop_with_into` moved later**, to after both opcode lists are
    parsed. The claim is that no precedence changed. Confirm with
    `git diff origin/main -- tests/` that no existing precedence assertion was edited.
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`.
  - `just test` already passes `--all-features`; add no flag.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place; never copy them into the repo.
  - Tier-B Q2M develops are slow in debug; run long binaries in the background.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC13 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run the gate suite yourself, with the corpus env set:
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
- Run `just decisions-audit --changed`.

### The behavioural surface (this is where defects escape)

**The no-allocation promise is real.**
- Read `tests/develop_scratch_alloc.rs`:
  - The counter must count `alloc`, `alloc_zeroed` **and** `realloc`.
  - Counting must be switched on only around the call under test, with
    `Sensor`/`src`/`dst`/`scratch` built before.
  - The negative control must actually fire.
- Confirm that the error paths are counted:
  - wrong length;
  - over-unity warp refused under `FullActiveArea`;
  - out-of-domain α;
  - a malformed opcode list.

  An `Error` variant that builds a `String` or `Vec` would allocate. Check that no error
  reachable from `develop_with_scratch_into` does.
- Confirm the test binary runs single-threaded, or that the counter is thread-local. A
  global counter in a multi-threaded harness can see other threads' allocations, which
  makes a count flaky. It can also be gated so it misses some.
- `git diff origin/main -- src/develop.rs src/opcode.rs` must add no `Vec`, `vec!`,
  `Box`, `collect`, `to_vec`, `String` or `format!` on the scratch path. The only
  allocation allowed is the single one under `Scratch::Allocate`.

**The layout is exactly DEC-060 §3, and the reuse is sound.**
- Read `run_passes` line by line for the "both" case:
  1. The plane copy goes into `[0, P)`.
  2. Normalize reads `[0, P)` into `[P, P + A)`.
  3. The warp reads `[P, P + A)` into `[0, A)`.
  4. Then the output pass.

  Confirm that nothing reads `[0, P)` after the warp writes into it.
- Confirm that the warp-only case uses `[0, A)` and `[A, 2A)`, and the `fbp`-only case
  uses `[0, P)`.
- Confirm that `warp_alpha = 0` collapses to the no-warp layouts, as AC1 says.
- Confirm that `A ≤ P` holds for every sensor `resolve_geometry` accepts, so `[0, A)`
  fits inside the plane region. Look for a hostile `ActiveArea` that could break it.

**Mutate it yourself, outside any test knob, and confirm a named test goes red:**
- a `vec!` in the caller path;
- the walker rebuilt on `parse_opcode_list`;
- the walker returning at the first match;
- the walker skipping the unknown-mandatory refusal;
- P + 2A layout (no reuse);
- the warp before normalize in the "both" case;
- the warp reading the wrong half;
- a scratch write before the length check;
- the length check moved before the `OpcodeList1` parse;
- "at least n" accepted;
- the neighbours not sorted;
- `lower_median` over the whole array.

Add one more of your own: **the output pass reading the wrong region** in the warp-only
case. Run each mutant against **tier A only**, because CI has no corpus. A mutant that
only tier B catches is a finding.

**Walker equivalence.** Read `first_recognised` against `parse_opcode_list`.
- Every opcode after the first match must still be validated: malformed, unknown
  mandatory, and truncated params.
- The `Ok` value must be the first instance.
- Errors must be identical, including byte offsets in any error fields.

Confirm the AC8 fixture set covers a malformed **second** instance of each opcode, and
an unknown mandatory opcode **after** the match.

**Error order (AC7).** With a correct-length scratch, `develop_with_scratch_into` must
return every error `develop_with_into` returns. With a wrong-length scratch:
- an out-of-domain α is still reported first;
- a malformed `OpcodeList1` is still reported, not the length.

Find the exact step where the length check sits. Confirm it is after both opcode-list
parses and before any write to `dst` or `scratch`.

**Real-file results.** Confirm these tier-B tests **ran** (no SKIP line), and check the
figures:
- **AC1:** `scratch_len` on the three Q2M frames is 94,707,712, and 47,443,968 at α = 0.
- **AC3:** zero allocations on L1021223.
- **AC4:** one allocation of 189,415,424 B.
- **AC2:** the eight SPEC-063 MD5 pins hold through the scratch path.

Confirm with `git diff origin/main -- tests/develop_options.rs` that those pins are
unedited.

**Fuzz.**
- The five new `scratch-fix-*` seeds must be synthetic, built by
  `examples/fuzz-seeds.rs`. Confirm none comes from corpus bytes (DEC-003).
- Run the `develop` and `opcode` targets for at least 60 s each and report the run
  counts.
- Confirm the differential asserts byte identity and error equality, and does not just
  run both.

**No regression.**
- Account for every changed line in existing test files against AC13's allowance, which
  covers `error_kind` rows and counts only.
- `DEVELOP_VERSION` is still `"4"`.
- No new dependency.

**Error surface.**
- `DevelopScratchWrongLength` is in `Error::kind()` with no wildcard.
- Its `Display` names both numbers, and a test checks that.
- The CHANGELOG `[Unreleased]` entry describes the exact-length rule and the
  no-allocation promise.

**CI is observed green on the PR head** with `gh pr checks 224`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

**Approved with no code changes:**
- Do **not** open a PR that supersets #224. Such a PR conflicts once #224 is
  squash-merged.
- Commit the filled HANDOFF-138 handback (and the timeline's verify line, if the
  template has one) on a branch named `claude/spec-065-verify-<suffix>` and push it.
- Post the verdict as a comment on PR #224, with the branch name.
- The orchestrator lands those docs in the ship PR.

**Punch list with fixes you make yourself:**
- Open a NEW PR against `main` that supersets #224.
- It must reference HANDOFF-138, SPEC-065, DEC-060 and PR #224.
- The orchestrator merges it and closes #224.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript (`basis: transcript`, dedupe by
  `message.id`, max output per message, opus-5-5 rates). Fill the `## Handback`
  section too.

## Out of Scope

- Warping straight into `dst` (DEC-060 Alternatives).
- A scratch variant of the colour path or `unpack_into`.
- Making `parse_opcode_list` allocation-free.
- SPEC-066 (CFA-as-mono), SPEC-067 (the blocked transpose), `irr` CLI flags, and a
  release.
- The DEC-051 prose-table sweep the build suggested. Note it in your follow-ups if you
  agree; do not do it here.

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
