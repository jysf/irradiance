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
  id: HANDOFF-010
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-08-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-002

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

# HANDOFF-010: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-002` for the **verify** cycle, at
`82fc390`. Independent session.

⚠ **ID note:** renamed `009` → `010` by hand. `just new-handoff` allocated `009`,
already held by SPEC-006's verify handoff on its branch — the command counts what
is visible in the current worktree, so parallel branches collide. **Second
occurrence.** Do not renumber it back.

## Context the Receiving Agent Needs

### Already reconciled — don't just repeat

- `just test 2>&1 | grep SKIP`, **no extra flags** → 8 lines (7 entries + summary),
  each naming the absent file. Criterion met.
- Real corpus present: **7/7**, 9 tests, 13.08 s. All gates green.

### The judgement call that most deserves scrutiny

**SHA-256 was hand-written from FIPS 180-4 rather than taken as a dependency.**
Hand-rolled crypto is normally a red flag, so weigh the argument, not the instinct:

- nothing in `std` hashes, and design budgeted exactly one dev-dep (`toml`);
- a hashing *crate* would be exercised **only where the corpus exists**, so a
  broken integration would be invisible in CI — the precise invisibility this spec
  exists to remove;
- NIST vectors run everywhere; verified against all 7 real files (~330 MB).

Corroboration: the manifest's `sha256` values were produced with `shasum -a 256`
**before this code existed**, and the suite checks against them — a wrong
implementation fails on real data. That is evidence, not proof: it exercises one
input class. **Consider what it does NOT cover** — empty input, multi-block
boundaries, lengths near the 55/56/64-byte padding edges, >4 GiB. Are the NIST
vectors well chosen? If you disagree, `sha2` is a clean swap and the builder says
so.

### Two more disclosed calls

1. The visible surface is an **`examples/` target**, not an `irr` subcommand —
   forced, because Cargo denies dev-deps to `[lib]`/`[[bin]]` (`DEC-010`). Is
   `examples/` the right home, or does it imply something user-facing that isn't?
2. CI's `rust / test` job also runs corpus-status, since it calls `cargo test`
   directly and would otherwise miss the lines.

### Also worth a look

- `just test` now takes **~12.6 s** with the full corpus (330 MB hashed in a debug
  build). `[profile.test] opt-level = 2` was deliberately **not** set, since
  profile changes affect every build. Right call?
- `DEC-010` must be explicit that the **library's zero-dependency claim is
  untouched** — that claim is load-bearing in this project's pitch.

## Expected Deliverables

A verdict: **✅ APPROVED (with SHA) / ⚠ PUNCH LIST / ❌ REJECTED**, per §15.

Run the criterion yourself with **no extra flags**, and confirm the negative case:
delete the corpus-status line and it must drop to **0** SKIP lines, or the check
has no teeth. The builder measured exactly that.

**Label every finding ship-blocking or follow-up.** A wrong hash accepted as
correct is ship-blocking; a slow test is a follow-up.

If **APPROVED**, set `task.cycle: ship`.

## Out of Scope

- Fixing anything — punch-list with file:line.
- SPEC-006's branch. ⚠ Both branches touch `app.just` and
  `.github/workflows/ci.yml`; that conflict is the orchestrator's to resolve.
- Re-opening `DEC-003`'s storage/schema decisions.

## Return Criteria — how to hand back

1. Verify cost session with a real `tokens_total`. ⚠ **Transcript sums
   double-count** — one jsonl line per content block repeats the same usage
   object. Deduplicate by `message.id` and **say that you did**, with cache-read
   share. This build measured the effect (~1.7x inflation) and updated the
   `token-counts-not-comparable` signal.
2. Fill `## Completion` and `handback:`; `handoff.status: completed`.
3. Verdict with SHA, every finding labelled ship-blocking or follow-up.
4. Commit on `feat/spec-002-corpus-manifest-reader`; do not merge.

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
