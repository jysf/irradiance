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
  id: HANDOFF-007
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-08-20
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-006

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

# HANDOFF-007: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-006` for the **build** cycle.

Close the one hole SPEC-001 could not: a single `#[allow]` exits the panic-free
policy. The mechanism is **decided and measured** — transcribe it.

## Context the Receiving Agent Needs

### The hole

Verified on the shipped crate: one attribute passes all seven gates while shipping
two panics on the public API, with no module involved.

```rust
#[allow(clippy::panic, clippy::expect_used)]
pub fn boom(v: &[u8]) -> u8 { if v.is_empty() { panic!("e") } *v.first().expect("x") }
```

`DEC-009`'s red-proof **structurally cannot** see this — it mutates the crate root
and asserts the root's `#![deny]` bites; no `#![deny]` mutation test can observe
an `#[allow]` beneath it.

### The mechanism — measured at design, all three properties

```bash
cargo clippy --lib --quiet -- \
  -F clippy::unwrap_used -F clippy::expect_used -F clippy::indexing_slicing \
  -F clippy::panic -F clippy::arithmetic_side_effects
```

| run | measured |
|---|---|
| `#[allow]` planted | **101**, `E0453: allow(clippy::panic) incompatible with previous forbid` |
| honest tree | **0** |
| `--all-targets` | 101 — tests legitimately allow, which is why scope is `--lib` |

`-F` is `--forbid`: it cannot be re-allowed in source, and attempting it is a hard
**compiler** error. `--lib` excludes `#[cfg(test)]` modules and `src/bin/irr.rs`,
so the sanctioned exceptions need no special-casing.

### Do NOT reach for a text search

It is the obvious approach and it is a trap here. `src/lib.rs` contains attribute
text **inside its own module documentation**. The
`attribute-text-inside-doc-comments` signal is at **N=5** on SPEC-001 alone, and
every instance produced a wrong *answer* rather than an error — two false
negatives and one false green that shipped a panic past seven gates. The forbid
check does no text matching at all, which is most of why it was chosen.

### `--force-warn` looks right and is not

It *does* override `#[allow]` (measured: 2 warnings at the planted line), but
`-D warnings` **cannot promote a force-warn diagnostic to an error** — exit stays
**0**. Building on it would require parsing output, reintroducing the fragility
this design avoids.

## Expected Deliverables

1. A CI job in `.github/workflows/ci.yml` running the forbid check.
2. A `just` recipe in `app.just` for it, matching AGENTS.md §6.
3. **Red-proof evidence in the handback, both directions:**
   - `#[allow]` planted → non-zero, with the `E0453` lines pasted
   - honest tree → exit 0
4. `guidance/constraints.yaml:33`'s `enforcement:` corrected — it must state what
   is enforced **and its scope** (`--lib`), without overstating. F-4 was raised
   because the previous wording read as a broader guarantee than held.
5. All existing gates still green, output pasted.

## Out of Scope

- **`scripts/lint-red-proof.sh` — do not touch it.** `DEC-009`'s red-proof is
  sound for what it pins; this gate covers what it structurally cannot. They are
  complementary, and a fourth iteration of that script was explicitly rejected.
- Any decoding work, and SPEC-002's corpus reader (separate spec, no shared files).
- Broadening the claim beyond the `--lib` target.

## Return Criteria — how to hand back

1. Paste both red-proof directions and all existing gates.
2. Fill `## Completion` and `handback:`. For `tokens_total`: if `/cost` is
   unavailable, sum transcript usage objects and **say so**, with cache-read share
   (see the `token-counts-not-comparable` signal).
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-006-allow-attribute-gate` off `main`; commit; do not merge.

The orchestrator will re-run the planted-`#[allow]` attack itself — it has the
exact reproduction.

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
