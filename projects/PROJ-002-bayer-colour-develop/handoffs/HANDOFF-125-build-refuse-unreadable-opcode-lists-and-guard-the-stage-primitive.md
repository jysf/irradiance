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
  status: pending                  # pending | accepted | completed | rejected

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
