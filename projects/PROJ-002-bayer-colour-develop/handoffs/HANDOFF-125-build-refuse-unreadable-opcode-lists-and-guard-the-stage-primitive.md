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
  id: HANDOFF-125
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8     # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 19174151           # REAL combined count — what cost-audit reads
  estimated_usd: 13.09             # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 30
  branch: claude/spec-056-build-b1
  pr: https://github.com/jysf/irradiance/pull/193
  completed_at: 2026-11-11
  notes: "FLOOR taken live just before the handback commit (docs/cost-tracking.md rule 6): session transcript a2bbac90-a1bd-4857-94f7-94b92579e8c9.jsonl, 96 unique message.ids, all claude-opus-5-5, no subagents; input 195 + cache-creation 212,456 + cache-read 18,900,990 + output 60,510 (max per message). USD at opus-5-5 list rates in 5/out 25 per MTok, cache read x0.10, cache writes x2.0 (1-hour TTL). The last few wrap-up messages are not in it; re-sum the same file after the session ends."
  synced_at: 2026-11-11
---

# HANDOFF-125: Refuse unreadable opcode lists and guard the stage primitives (SPEC-056, 0.2.1 housekeeping)

## Delegation Summary

The architect (claude-opus-4-8) hands `SPEC-056` to a cold implementer (claude-opus-5-5) for the
**build** cycle. There are four parts:

1. Refuse a develop when an `OpcodeList` tag is present but unreadable (new
   `Error::UnreadableOpcodeList`, `ErrorKind::Corrupt`).
2. Make `demosaic_into*` and `to_srgb_linear_into` run the required-processing check,
   and expose `opcode::check_required_processing` plus `DevelopPath` publicly.
3. Fix the 19 rustdoc private intra-doc links, and add a `doc-check` gate with its
   red-proof.
4. Make the 7 nested-cargo red-proof tests independent of `CARGO_TARGET_DIR`.

All of it is decided in **DEC-053**. **Read SPEC-056 in full and DEC-053 first.**

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-056-refuse-unreadable-opcode-lists-and-guard-the-stage-primitive.md` — all of it: `## Design`, `## Acceptance Criteria`, `## Failing Tests`, `## Notes for the Implementer`.
- **Decision:** `./decisions/DEC-053-refuse-unreadable-opcode-lists-at-develop-and-guard-the-stage-primitives.md`. Its rejected alternatives are not to be re-litigated: no sentinel-id reuse, no parse-time failure, no `&Sensor` on the appliers, no escape hatch.
- **Prior contract:** `./decisions/DEC-049-validate-required-opcode-processing-and-seal-the-sensor-view.md` and SPEC-054 (the scan you extend).
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-014-0-2-x-maintenance-post-release-correctness-and-hygiene.md`.
- **Code:** `src/opcode.rs` (`:480`, `:535`, `:549`, `:565`), `src/ifd.rs` (`:130-134`, `:721-749`, `:1729-1767`, `malformed_tags` `:812`), `src/lib.rs` (`Error` `:274`, opcode variants `:609-629`, `Display` `:1078-1090`, `ErrorKind`/`kind()` `:1246-1275`), `src/demosaic.rs` `:234`/`:432`, `src/color.rs` `:210`, `src/warp.rs` `:225`, `src/develop.rs` `:460`; `tests/required_processing.rs` (fixture builder `:89`, probe `:567`); the 7 nested-cargo tests named in the spec.
- **Toolchain brief:** `./guidance/toolchain-brief.md` and AGENTS.md §5 "Measured toolchain" (the `+toolchain` traps; `just lint-ci` before every push; `just test` already passes `--all-features`, so pass no extra flag).

> ⚠ Line numbers were read 2026-11-11 against `a07744c`; re-confirm before editing.

### Decisions that apply

- `DEC-053` — the rulings this spec builds.
- `DEC-049` — the required-processing contract, applied sets, the flags-aware raw scan.
- `DEC-012` — read stays tolerant; `malformed_tags` is the record of "present but unreadable".
- `DEC-051` — `ErrorKind`; the wildcard-free `kind()` match forces classification (`Corrupt`).
- `DEC-009` — negative control before every red-proof.
- `DEC-003` — the tier-B corpus is never committed; the bit-exact corpus check is a local run (`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`).

### Constraints that apply

- `no-panics-on-untrusted-input` — the check runs on hostile `Sensor`s; no indexing/unwrap in `--lib`.
- `oracle-must-be-shown-red` — the unreadable-list red-proof AND the new doc gate's red-proof, each with a control.
- `library-not-application` — no I/O added to the library; the doc gate is CI/just only.
- `one-spec-per-pr` — PR references SPEC-056 only.

### Prior related work

- `HANDOFF-112` / `HANDOFF-115` — SPEC-054 build and verify (the follow-ups that became items 1–2).
- `HANDOFF-124` — the 0.2.0 verify that raised items 3–4.

## Expected Deliverables

- All of SPEC-056's Acceptance Criteria met, with the Failing Tests observed red on `main` first.
- CHANGELOG `[Unreleased]` entries, with the two closed Known-limitations bullets removed. **No** version bump.
- In the handback, record:
  - the corpus bit-exact evidence;
  - the `CARGO_TARGET_DIR` run command;
  - CI observed green on the PR head, including the new `docs` job.
- A PR against `main` from branch `claude/spec-056-build-<suffix>`, cut from fresh `origin/main`. The PR description must reference HANDOFF-125, SPEC-056, STAGE-014, PROJ-002 and DEC-053 / DEC-049. It must also include "Provenance: none — no new algorithm" and the oracle and red-proof notes (AGENTS.md §13).
- Commit-message **bodies must be plain prose**. A body with code tokens (`->`, `#[...]`, `--flag`) is rejected by the push ruleset. PR bodies are fine.
- **Do not** force-push, merge, tag or publish.

## Out of Scope

- New opcodes; applying opcodes on the colour path; any `DEVELOP_VERSION` change.
- A develop-anyway escape hatch (DEC-053 defers it).
- Signature changes to `apply_warp_into` / `apply_fix_bad_pixels_constant`.
- The 0.2.1 version bump, tag, or `cargo publish`.
- STAGE-005's XS harness bullets; SPEC-035.

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

- **Branch / PR:** `claude/spec-056-build-b1` (cut from fresh `origin/main` @ c4cf729) → https://github.com/jysf/irradiance/pull/193. Build commit 1973abf; this handback is a second commit on the same branch.
- **Completed at:** 2026-11-11
- **All acceptance criteria met?** yes.
  - **Unreadable list refused:** for lists 1, 2 and 3 a fixture whose list entry's payload offset runs past EOF parses `Ok`, has the tag in `malformed_tags` (asserted inside the fixture builder, along with `opcode_lists[n]` true and the field `None`), and returns `Error::UnreadableOpcodeList { list: n }` from `develop_into` and from all four colour entries. Order is deterministic: unreadable lists 1+3 report list 1; a mandatory unsupported opcode in a readable list 2 wins over an unreadable list 3 (each list's unreadable check runs before its own scan, lists in order).
  - **Error shape:** variant in `Error` with doc citing SPEC-056/DEC-053, `Display` "OpcodeList{n} is present but unreadable (malformed tag entry); its required processing cannot be confirmed", `kind()` arm → `Corrupt`. `tests/error_kind.rs` table updated (56 variants, Corrupt 26) and DEC-051 got a short "later additions" note, because that test's own comment says the table, the count and DEC-051 move together.
  - **Primitives guarded:** `demosaic_into`, `demosaic_into_threaded`, `to_srgb_linear_into` call the public check first with `DevelopPath::Colour`. There are 12 tests, one per primitive per case: mandatory gives `UnsupportedRequiredOpcode`, unreadable gives `UnreadableOpcodeList`, and optional-only and opcode-free give `Ok`.
  - **Public check:** `pub fn opcode::check_required_processing(&Sensor, DevelopPath)`. `DevelopPath` is `pub` and `#[non_exhaustive]`; a test matches it from `tests/` and needs the wildcard arm. The list-level helper is now `pub(crate) check_lists(lists, unreadable, path)`. An agreement test shows the public check returns exactly what `develop_into` (mono) or each colour entry returns, across 20 fixtures. 15 of them refuse (7 mono, 8 colour).
  - **Appliers documented:** `apply_warp_into` and `apply_fix_bad_pixels_constant` each have a "Required processing — the caller validates" section that links the public check. The stale `Sensor::opcode_lists` doc is rewritten.
  - **Docs clean and gated:** 19 private links reworded to plain code formatting, with no `#[allow]` and no `--document-private-items`. `just doc-check` (`scripts/doc-check.sh`) runs `--all-features` and `--no-default-features` with `-D warnings` and prints the rustdoc it used. Both are 0 errors. `just doc-check-red-proof` passes: the control copy is green, and an injected `pub fn` whose doc links the private `ljpeg` module fails. That failure is attributed to the injected item, 1 of 1 diagnostics, by `rustdoc::private_intra_doc_links`. CI jobs `docs` and `docs-red-proof` were **observed green on the PR head** with `gh run view 43112012702` (head 1973abf, conclusion success). All 44 PR checks passed.
  - **CARGO_TARGET_DIR-proof:** each of the 7 staged builds calls `.env_remove("CARGO_TARGET_DIR")`, with an identical comment. Red on main: `CARGO_TARGET_DIR=$(mktemp -d) cargo test --all-features --test develop -- fix_bad_pixels_red_proof_control_is_green` fails with `spawn …/target/release/fix_bad_pixels_probe: No such file or directory`. After the fix, **command run:** `CARGO_TARGET_DIR=$(mktemp -d) cargo test --all-features --no-fail-fast` (the body of `just test`). Exit 0, 539 passed. Afterwards the temp target held only `debug/` and `tmp/`, so no staged release build wrote into it. `CARGO_TARGET_DIR=$(mktemp -d) just test` was also run; it failed only at `error_kind` (the variant count, fixed above) and every nested-cargo test passed.
  - **Corpus unchanged (DEC-003, local):** a scratch probe crate was built twice, against an `origin/main` worktree and against this branch. It develops each of the 8 held DNGs on its own path and prints FNV-1a-64 digests of every output. The two outputs are **identical**, and every result is `Ok`. Bayer files run `develop_color_into`, `demosaic_into`, `demosaic_into_threaded` and `to_srgb_linear_into`; mono files run `develop_into`. Digests (digest/len):
    - 5D III: `902ba0ddb8d4d98e/66355200`
    - D300S: `3c02ff0cb9494f15/36636672`
    - Q2-Mono L1021223: `70ee1ab38fe6582a`
    - L1026016: `1ad81565269b7eac`
    - L1026192: `262e7a30777ae55c`
    - M Monochrom: `be519c006f4eb545`
    - Typ 246: `ff880af33175285c`
    - K-3 III: `ab4b018e2b8cc1cd`

    The Q2-Mono files carry lists [1, 3], readable. No file has an opcode-list tag in `malformed_tags`; the K-3 III's only malformed tag is 50713. `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images just test` passed (539, 1 ignored, 9/9 corpus present). `DEVELOP_VERSION` is unchanged.
  - **Gates (local):** all green.
    - `just test` (with the corpus dir)
    - `lint-ci` (clippy 0.1.98, pinned)
    - `lint-no-allow`, `lint-fuzz`, `fmt-check`
    - `validate`, `cost-audit`
    - `gate-audit` (11 scripts, including the new `doc-check.sh`)
    - `deny`, `deny-fuzz`
    - `doc-check`, `doc-check-red-proof`
    - `msrv` (1.90.0)
    - `decisions-index --check`
  - **Failing tests observed red on main first:** see Reflection 3 for how. There were 9 failures, every one `got Ok(())`, while all controls stayed green. The nested `required_processing_red_proof` was extended: unreadable list 2 on `develop_into`, unreadable list 1 on the four colour entries, and mandatory plus unreadable on the three primitives. Its injection moved to `check_lists`' loop, which now holds both halves, so the mutant goes silent `ok` on all of them while the control reproduces the typed errors.
  - **Fuzz:** `opcode` target (drives the check through `__fuzz_required_processing`), 60 s, 2,428,859 runs, clean. It needed `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run …`; without the PATH prefix the inner cargo is Homebrew's (the documented `+toolchain` trap).

### Cost self-report

- **Tokens (total):** 19,174,151. This is a **floor**, taken live just before the handback commit. It counts 96 unique `message.id`s, all `claude-opus-5-5`, with no subagents. Breakdown: input 195, cache creation 212,456, cache read 18,900,990, output 60,510.
- **Estimated USD:** 13.09 (opus-5-5 at in 5 / out 25 per MTok, cache read ×0.10, 1-hour cache writes ×2.0)
- **Duration (minutes):** ~30 (transcript 22:17–22:44Z at measurement)
- **Source of the number:** this session's transcript `a2bbac90-a1bd-4857-94f7-94b92579e8c9.jsonl`, measured per docs/cost-tracking.md "Measuring a cycle". Re-sum the same file after the session ends.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-051 gained a "later additions" paragraph recording `UnreadableOpcodeList` → `Corrupt` (DEC-053), so its table stays the single reference.
- **Deviations from spec:**
  - `check_lists` takes a third argument, the per-list unreadable flags. The unreadable refusal therefore lives inside the same ordered loop as the scan; that is what makes "list n's unreadable check before list n's scan" literal. It also means one red-proof injection disables both halves. The fuzz hook passes `[false; 3]`, so the fuzz surface is unchanged.
  - The doc gate is a script (`scripts/doc-check.sh`) called by the `just` recipe, by CI and by the red-proof. A bare recipe would not do, because the red-proof must run the exact gate over a mutated copy. Because it is a script, `gate-audit` now audits it too.
  - The spec named `tests/error_kind.rs` nowhere, but it pins the variant count. It was updated to 56 / Corrupt 26.
  - CHANGELOG: the two Known-limitations bullets were removed from the `[0.2.0]` section as the spec said. `[Unreleased]` has Fixed/Added/Changed entries. There is no version bump.
- **Follow-up work identified:** none required. Optional: the fuzz hook could also fuzz the unreadable flags, but they are a trivial early-return, so this is low value.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. One thing not listed was `tests/error_kind.rs`, whose hard-coded variant count, together with its "update DEC-051 too" comment, is a second place every new `Error` variant must touch. It showed up only as a red in the full run. Also, the fixture builder offers no "make this entry unreadable" knob, so I patched the little-endian IFD0 entry's offset in the test, and asserted it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-051's "table, count and DEC move together" rule (via `tests/error_kind.rs`) belongs in any spec that adds an `Error` variant.

3. **If you did this task again, what would you do differently?**
   — The red-on-main run used a Debug-string assertion for `UnreadableOpcodeList` (the variant did not exist yet), then switched to the typed `matches!`. I would write that two-step plan into the spec's Failing Tests up front. I would also run `cargo test --no-fail-fast` under the temp `CARGO_TARGET_DIR` first, since it surfaces every red in one pass.
