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
  id: HANDOFF-126
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-11
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-056

project:
  id: PROJ-002
  stage: STAGE-014
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
  status: completed                     # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 8053981               # REAL combined count — what cost-audit reads
  estimated_usd: 5.72              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 25
  branch: claude/spec-056-verify-handback
  pr: https://github.com/jysf/irradiance/pull/194
  completed_at: 2026-11-11               # YYYY-MM-DD
  notes: "FLOOR taken live before the handback commit (docs/cost-tracking.md rule 6): transcript 8b5e64da-637a-4eba-9162-422f40773bbf.jsonl in the spec-056-verify-4123a1 worktree project dir, 51 unique message.ids, all claude-opus-5-5, no subagents; input 127 + cache-creation 130,095 + cache-read 7,905,121 + output 18,638 (max per message). USD at opus-5-5 list rates in 5/out 25 per MTok, cache read x0.10, cache writes x2.0. Re-sum the same file after the session ends. Verdict is on code commit 1973abf; PR 193 was already squash-merged to main as dd70dea (identical src/tests) before this verify ran." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-126: VERIFY: refuse unreadable opcode lists and guard the stage primitives (SPEC-056, 0.2.1 housekeeping)

## Delegation Summary

You are the **cold verifier** for SPEC-056. You did **not** implement it. Review it
against SPEC-056 and DEC-053 and return exactly one verdict.

- **The build:** open **PR #193**, branch `claude/spec-056-build-b1`.
  - Code commit: `1973abf`.
  - Two later commits are handback docs only: `bd08547` and `93f2862`.
- **Your branch:** your worktree is cut from `claude/spec-056-verify-base`. That is
  the build head plus this handoff, so the code under review is the build's code.

The build is meant to do four things:
1. **Refuse an unreadable opcode list.** When an `OpcodeList1/2/3` tag is listed in
   `Sensor::malformed_tags`, every develop entry returns the new
   `Error::UnreadableOpcodeList { list }`, with kind `Corrupt`. Parsing stays
   tolerant.
2. **Guard the stage primitives.** `demosaic_into`, `demosaic_into_threaded` and
   `to_srgb_linear_into` run the colour-path required-processing check first.
   `opcode::check_required_processing(&Sensor, DevelopPath)` and `DevelopPath` become
   public, with `DevelopPath` marked `#[non_exhaustive]`.
3. **Fix the 19 rustdoc private intra-doc links,** and add a `just doc-check` gate,
   its red-proof, and CI jobs.
4. **Isolate the nested-cargo tests.** All 7 nested-cargo red-proof tests build in a
   target dir that does not depend on `CARGO_TARGET_DIR`.

**No `DEVELOP_VERSION` change and no new dependency.**

## What to read first

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-056-refuse-unreadable-opcode-lists-and-guard-the-stage-primitive.md` (all of it).
- **Decision:** `./decisions/DEC-053-refuse-unreadable-opcode-lists-at-develop-and-guard-the-stage-primitives.md`. Its rejected alternatives are settled; do not re-litigate them:
  - no sentinel-id reuse;
  - no parse-time failure;
  - no `&Sensor` on the appliers;
  - no escape hatch.
- **Prior contract:** DEC-049 and SPEC-054, which define the scan being extended.
- **Build handoff and handback:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-125-build-refuse-unreadable-opcode-lists-and-guard-the-stage-primitive.md`. Treat its claims as claims, and check them yourself.
- **Toolchain:** AGENTS.md §5 "Measured toolchain" and §6 commands. `just test` already passes `--all-features`; add no flag.

## The verify contract

Review it cold.

### Acceptance criteria and gates
- Is every SPEC-056 acceptance criterion met?
- Is the build reflection honest?
- Run the full gate suite yourself:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `deny`
  - `deny-fuzz`
  - `doc-check`
  - `doc-check-red-proof`
  - `msrv`
- Check decision drift with `just decisions-audit --changed`.

### The §12 behavioural surface (this is where defects escape)

**The refusal fires from each entry.** Confirm `UnreadableOpcodeList` is returned by
**each** of these, not one representative, and for each of lists 1, 2 and 3:
- `develop_into`
- `develop_color_into`
- `develop_color_f32_into`
- both threaded colour entries
- the three guarded primitives

Use **adversarial mutation**: remove the new check, or the guard call in a primitive,
and confirm tests go red per entry.

**The fixtures are genuinely unreadable.** Confirm the test asserts the tag is in
`malformed_tags`. A fixture that silently degraded to an absent tag would prove
nothing.

**The red-proofs were real.** Check that the unreadable-list red-proof and the
doc-check red-proof each show a control first and a red for the stated reason.
Break `doc-check` once yourself by injecting a private link.

**The `CARGO_TARGET_DIR` claim holds.** Run the nested-cargo tests with
`CARGO_TARGET_DIR` set to a temp dir, and confirm all 7 pass. Confirm the outer
target dir receives no mutated build.

**The corpus is unchanged.**
- Develop all 8 held files with
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`.
- Each must be `Ok` and byte-identical to `main` on its path (mono or colour).
- `DEVELOP_VERSION` must be unchanged.

**The public surface is right.** `DevelopPath` must be `#[non_exhaustive]`, and the
public check must return the same result as the matching `develop_*`. Also confirm
the docs of `apply_warp_into` and `apply_fix_bad_pixels_constant` state that the
caller owns validation.

**The CHANGELOG is right.** `[Unreleased]` should carry the Fixed / Added / Changed
entries, and the two closed Known-limitations bullets should be removed. There
should be no version bump.

**CI is observed green on the PR head** with `gh run view`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- A punch-list fix goes on a fresh branch `claude/spec-056-verify-<suffix>`, opened
  as a NEW PR against `main` that supersets #193. It must reference HANDOFF-126,
  SPEC-056, DEC-053 and PR #193.

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

### Execution notes

- **Branch / PR:** `claude/spec-056-verify-handback`, cut from `origin/main` @ `dd70dea`. It carries the HANDOFF-126 mint (cherry-picked) and this handback, as a docs-only PR (see `handback.pr`). The review itself ran on `origin/claude/spec-056-verify-base` (`a8d8055` = build head `93f2862` + the handoff).
- **Completed at:** 2026-11-11
- **All acceptance criteria met?** yes. I re-checked every SPEC-056 AC myself and took none from HANDOFF-125 on trust.
- **Verdict:** ✅ **APPROVED at `1973abf`**. That is the code commit; branch head `93f2862` only adds handback docs.
  - PR #193 was already squash-merged to `main` as `dd70dea` before this verify ran.
  - `git diff 93f2862 dd70dea -- src tests scripts .github app.just CHANGELOG.md fuzz` is empty, so the approved code is exactly what is on `main`.

**Evidence (all run in this session):**

- **Gates, local, all rc=0:**
  - `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images just test`: 539 passed, 0 failed, 1 ignored; the corpus tests ran.
  - `lint-ci` (clippy 0.1.98, pinned), `lint-no-allow`, `fmt-check`.
  - `validate`, `cost-audit`, `gate-audit`.
  - `deny`, `deny-fuzz`.
  - `doc-check`, `doc-check-red-proof`.
  - `msrv` (1.90.0).
  - `just decisions-audit --changed`: advisory only. It flags DEC-053, DEC-049 and others as governing the touched files, and the change is consistent with them.
- **Adversarial mutation.** I ran these in a scratch copy with its own target dir, using `cargo test --test required_processing` and skipping the nested red-proof. The control was 43 of 43 green. I removed each guard in turn, and every mutant was killed:
  - **A:** the `UnreadableOpcodeList` return in `check_lists` (`opcode.rs:600`). 8 tests red: mono per-list, colour every-entry, the unreadable test of all 3 primitives, the agreement test, and the ordering test.
  - **B:** the check in `develop_into` (`develop.rs:806`). 11 tests red, including `mono_unreadable_opcode_list_is_refused_per_list`.
  - **C to G:** the check call in each colour function separately (`color.rs` lines 222, 305, 372, 447 and 491).
    - C, `to_srgb_linear_into`: 3 tests red (mandatory and unreadable).
    - D to G, `develop_color_into`, `develop_color_f32_into`, `develop_color_into_threaded` and `develop_color_f32_into_threaded`: 8 tests red each, including `colour_unreadable_opcode_list_is_refused_on_every_entry`.
    - So every colour entry is pinned by its own guard, not through a shared callee.
  - **H and I:** `demosaic_into` (`demosaic.rs:243`) and `demosaic_into_threaded` (`:450`). 3 tests red each (mandatory and unreadable). The threaded mutant is really killed: the 4-thread fixture takes the banded path, not the fallback to `demosaic_into`.
  - **Scratch test, not committed:** all 3 primitives refuse an unreadable list 1, 2 and 3 each. The committed suite uses list 2 for the primitives. The test is green on the honest tree and red under mutants A, C, H and I.
- **The fixtures are really unreadable.** `unreadable_fixture` asserts three things: the tag is in `malformed_tags`, `opcode_lists[n]` is true, and the field is `None`. `make_list_unreadable` asserts it patched exactly one entry.
- **The red-proofs are real.**
  - `required_processing_red_proof` builds the unmutated control first, then the mutant. In the control, the optional-only and opcode-free cases develop, and every required case returns its typed error. The required cases include the new ones: unreadable list 2 on mono, unreadable list 1 on the 4 colour entries, and the 3 primitives.
  - `doc-check-red-proof`: the control is green, and the injected link is caught 1 of 1 by `rustdoc::private_intra_doc_links`.
  - **My own doc-check break:** in a scratch copy I restored the link on one of the 19 fixed docs (`build_camera_to_srgb_linear` in the doc of `develop_color_into`). `scripts/doc-check.sh` exited 101 with "public documentation for `develop_color_into` links to private item `build_camera_to_srgb_linear`". With the copy reverted, the gate is green again.
- **CARGO_TARGET_DIR.** Command run: `CARGO_TARGET_DIR=$(mktemp -d) cargo test --all-features --no-fail-fast --test develop --test develop_oracle --test ifd_resource_guard --test ljpeg_shape_binding --test plane_oracle --test plane_validate --test required_processing`.
  - Exit 0, and all 7 nested-cargo tests passed: `fix_bad_pixels_red_proof_control_is_green`, `red_proof_no_op_applier_leaves_bad_pixel_at_zero`, `ifd_guard_red_proof`, `sampling_red_proof` and `shape_red_proof`, `the_bayer_red_proof_control_is_green` and `the_ljpeg_red_proof_control_is_green`, the three `validate_red_proof_*` tests, and `required_processing_red_proof`.
  - Afterwards the outer temp target held only `debug/` and `tmp/`: no `release/` dir and no probe binary.
  - All 7 sites carry an identical `.env_remove("CARGO_TARGET_DIR")`.
- **Corpus bit-exact (DEC-003, local).** I built a scratch probe crate twice, against `c4cf729` (`main` before SPEC-056) and against the build tree, and ran it over all 8 held tier-B DNGs. It hashes every output buffer with FNV-1a-64.
  - The outputs are identical on both trees, and every result is `Ok`.
  - Bayer files (5D III, D300S) ran `develop_color_into`, `demosaic_into`, `demosaic_into_threaded` and `to_srgb_linear_into`. Mono files (M Monochrom, Typ 246, Q2-Mono ×3, K-3 III DNG) ran `develop_into`.
  - The develop digests match HANDOFF-125's, e.g. 5D III `902ba0ddb8d4d98e`, D300S `3c02ff0cb9494f15`, K-3 III `ab4b018e2b8cc1cd`.
  - No file has an opcode-list tag in `malformed_tags`. The Q2-Mono files carry lists 1 and 3, both readable; the K-3 III's only malformed tag is 50713.
  - `K3III.PEF`, also in that folder, gives `UnsupportedCompression 65535` on both trees; it is not one of the 8.
  - `DEVELOP_VERSION` is "4", unchanged.
- **Public surface.**
  - In the source, `DevelopPath` is `pub` and `#[non_exhaustive]`.
  - `check_required_processing(&Sensor, DevelopPath)` is public. `public_check_agrees_with_every_develop_entry` matches it against every develop entry over 20 fixtures.
  - The docs of `apply_warp_into` and `apply_fix_bad_pixels_constant` each have a "Required processing — the caller validates" section linking the public check.
  - The `kind()` arm returns `Corrupt`, and `Display` names the list.
- **CHANGELOG.** `[Unreleased]` has Fixed, Added and Changed entries. Both closed Known-limitations bullets are removed. `Cargo.toml` is still `0.2.0`, and there is no new dependency.
- **CI observed with `gh run view`.** On PR #193, runs 43112012702 (`1973abf`) and 43114313160 (head `93f2862`) both concluded `success`, including `docs (rustdoc -D warnings)` and `docs red-proof`. All 44 checks were SUCCESS. CI on `main` @ `dd70dea` is also success.

### Cost self-report

- **Tokens (total):** 8,053,981. This is a FLOOR, measured live before the handback commit.
- **Estimated USD:** 5.72
- **Duration (minutes):** ~25 (transcript 23:23–23:46Z at measurement)
- **Source of the number:** this session's transcript `8b5e64da-637a-4eba-9162-422f40773bbf.jsonl` (the scratchpad-dir UUID), measured per docs/cost-tracking.md "Measuring a cycle". It counts 51 unique `message.id`s, all claude-opus-5-5, taking each message's largest output, with no subagents. Breakdown: input 127, cache creation 130,095, cache read 7,905,121, output 18,638.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none blocking. The build reported these, and I accept them as reasonable:
  - `check_lists` takes the unreadable flags, so the per-list order is literal.
  - The doc gate is a script, so the red-proof runs the exact gate.
  - `tests/error_kind.rs` and a DEC-051 "later additions" note were updated.
  - The rustdoc-link rewording is filed under CHANGELOG **Fixed** rather than **Changed**. This is cosmetic.
- **Follow-up work identified (optional, non-blocking):**
  - The committed primitive tests use unreadable list 2 only. A loop over lists 1, 2 and 3 for the three primitives would make "each list, each door" explicit. I verified it in scratch, and the mechanism is shared, so the risk is low.
  - `develop_path_is_public_and_non_exhaustive` only proves the wildcard arm compiles. The real check comes from `lint-ci`'s `-D warnings`: on an exhaustive enum the wildcard would raise `unreachable_patterns` and fail the lint. So the test proves it only together with the lint gate, not alone.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only that PR #193 had already been merged before the verify ran, while the handoff assumes an open PR. So the handback PR is cut from `main`, not stacked on the build branch.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. One trap cost me a wasted corpus run with `irr`: in zsh, an unquoted variable holding a multi-word command is passed as a single argument. A Rust probe crate that hashes full buffers is the reliable corpus check.

3. **If you did this task again, what would you do differently?**
   — I would again start the long gate suite and the mutation copy in parallel from the first minute. For the corpus, I would go straight to a probe crate instead of `irr`'s summary output.
