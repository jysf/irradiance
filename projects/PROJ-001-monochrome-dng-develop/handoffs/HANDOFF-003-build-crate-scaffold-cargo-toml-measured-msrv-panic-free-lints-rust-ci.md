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
  id: HANDOFF-003
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-18
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-001

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-003: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-001` back to `claude-sonnet-5`
(implementer) for a **second build** cycle — the punch-list round.

The first build was honest and in scope. Verify returned ⚠ PUNCH LIST, not ❌,
and the two P1s were **reproduced independently by the orchestrator** before
being accepted. Your job is the fix round, not a rewrite.

## Context the Receiving Agent Needs

Branch `feat/spec-001-crate-scaffold`. Read the verify handback in
`HANDOFF-002-verify-*.md` for the full punch list, and **`DEC-007`**, which
settles the P1-1 design so you do not have to.

### P1-1 — the red-proof proved the wrong thing (DEC-007 supersedes DEC-006)

Delete the `#![deny(...)]` block from `src/lib.rs`, add
`pub fn read_u8(b: &[u8], at: usize) -> u8 { b[at] + 1 }`, and **all seven gates
go green** — a shipped panic on untrusted input. The old proof's own output says
why: `the lint level is defined here --> tests/lint_policy_red.rs:14`, the
snippet's header, never the library's.

**The design is decided and already probed — transcribe it, don't redesign it.**
`DEC-007` Option C: copy the crate to a temp dir, inject violating functions into
the *copied* `src/lib.rs` right after its attribute prologue, run clippy there.
Working tree is never touched, so no `trap`-based restore is needed.

Injection point that works (verified): skip blank lines, `//!` doc comments, and
`#![...]` blocks (tracking bracket depth); insert before the first real item.
⚠ A naive `max()` over lines ending `)]` was tried first and landed **inside the
test module's `#[allow(...)]`**, silently suppressing two of the three lints. It
was caught only because the lint names were checked — which is why assertion 3
below is not optional.

Assert **all three**:
1. clippy actually ran (`cargo clippy --version` succeeds first)
2. clippy exited non-zero
3. all of `arithmetic_side_effects`, `indexing_slicing`, `unwrap_used` appear in
   its output

Proven both directions before the DEC was written: policy present → exit 101,
three lints fire, level resolves to `src/lib.rs:31`; policy **removed** → exit 0,
so the assertion fails and the removal is caught. **Your change must demonstrate
both directions**, not just the first.

Delete `tests/lint_policy_red.rs.disabled` — DEC-007 supersedes that mechanism.

### P1-2 — the proof greened when clippy was unavailable

With a stub `cargo` on `PATH` the script printed *"✓ lint policy red-proof: the
violating snippet failed to compile as expected"* and exited 0. Assertion 1 above
fixes this. DEC-006's own Validation listed the three error identities; the script
just never implemented them.

### P2/P3 — the rest of the punch list

- `DEC-006`'s `affected_scope` omits `src/lib.rs`, so `decisions-audit --changed`
  cannot surface the drift it predicts. `DEC-007` already scopes this correctly;
  no action beyond leaving DEC-006 superseded.
- **`use std::fmt` / `impl std::error::Error` are gratuitous** — `core::fmt`
  compiles clean on the declared MSRV. `DEC-002` (still `proposed`) says do not
  foreclose `no_std`, and the spec's Non-Goals said so explicitly. Switch to
  `core::`. Do **not** add feature machinery — that is still DEC-002's call.
- `AGENTS.md` §6's command block omits `cargo fmt --check` and the red-proof,
  though `just lint` / `just lint-red-proof` run them. Make §6 match reality.
- `scripts/lint-red-proof.sh` is CWD-relative — resolve paths from the script's
  own location. Its temp file was not gitignored; the copy-to-temp-dir design
  should remove that concern, but confirm.
- One small inaccuracy in the `AGENTS.md` edits, per the reviewer — find and fix.

## Expected Deliverables

1. `scripts/lint-red-proof.sh` rewritten per DEC-007, with all three assertions.
2. `tests/lint_policy_red.rs.disabled` deleted.
3. **Evidence, in the handback, of BOTH directions** — the proof rejecting the
   injection with the policy present, and the proof *failing* when the policy is
   removed. Paste both outputs. This is the acceptance criterion.
4. `core::fmt` in place of `std::fmt`; `core::error::Error` or an equivalent that
   holds on MSRV 1.90 — **measure it, do not assume**.
5. `AGENTS.md` §6 corrected; the one inaccuracy fixed.
6. Path-independent script; temp files clean.
7. All seven gates green afterwards, run for real, output pasted.

## Out of Scope

- Re-litigating DEC-007's design. It was probed both directions before adoption.
- The MSRV number, the cost entries, and the fuzz-job omission — all settled.
- `[lints]` in `Cargo.toml` — considered and rejected in DEC-007 (it applies to
  every target, including the ones deliberately allowed to `unwrap`).
- Any decoding work.

## Return Criteria — how to hand back

1. Paste real output for all seven gates **and** both red-proof directions.
2. Fill `## Completion` and the `handback:` block. For `tokens_total`: if you are
   a sub-agent with no `/cost`, leave it `null` **and say so** — the orchestrator
   fills it from the invocation metadata (DEC-013). Never invent a number. If you
   are a CLI session with `/cost`, use the real figure and note how you obtained
   it — see the `token-counts-not-comparable` signal.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Commit on `feat/spec-001-crate-scaffold`. Do not merge.

The orchestrator reconciles against git and disk, and **will re-run the
policy-removal attack itself**. It has been run twice already; a fix that does not
survive it is not a fix.

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
