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
  id: HANDOFF-008
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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

# HANDOFF-008: <Task Title — same as the spec's title>

## Delegation Summary

`claude-opus-5` (architect) hands `SPEC-002` for the **build** cycle.

Build the corpus manifest reader and make an absent tier-B file **visibly**
skipped. Storage and schema are already settled by `DEC-003`;
`tests/corpus/manifest.toml` ships seeded with 7 entries and **nothing reads
it** — its own header records that as a debt owned by this spec.

## Context the Receiving Agent Needs

### Two design-time measurements — transcribe, do not re-derive

**1. The `toml` dev-dependency.**

| config | crates | parses? |
|---|---|---|
| `toml = "0.8"` | 12 | yes |
| `default-features = false, features = ["parse"]` | **11** | yes |
| `default-features = false` | 6 | **NO** — `Value: FromStr` unsatisfied |

The last row is a trap: `cargo check` passes because nothing calls the API. **Use
`features = ["parse"]`.** With the dep present, `cargo +1.90.0 check
--all-targets` → 0 and `cargo deny check licenses` → licenses ok.

It is **dev-only**, so the library's zero-dependency claim is untouched. Your
`DEC-*` must say that explicitly — "irradiance has no dependencies" appears in the
README-facing story and must stay true as written.

**2. `eprintln!` inside a passing test is INVISIBLE.** Measured:
`cargo test` → 0 SKIP lines; `cargo test -- --nocapture` → 2.

So "skip loudly" **cannot** be satisfied inside the test harness. Recommended: a
small corpus-status step that `just test` runs **before** the suite, printing one
line per manifest entry (present / MISSING + path). The in-harness skip returns
early; the loudness lives where it can be seen with no flags.

⚠ Do **not** make `just test` pass `--nocapture` globally — that buries the signal
in full test output instead of surfacing it.

### Constraints that bind

- `no-new-top-level-deps-without-decision` — DEC-004 rule 4 sanctions a trivial
  **dev-only** permissive dep **with its DEC authored in the same pass**. `toml`
  qualifies; write the DEC.
- `DEC-003` — tier-B files are never committed; CI cannot run them.
- A silent skip is the same defect class as an oracle that cannot go red.

## Expected Deliverables

1. Manifest reader: path, `sha256`, `oracle.raw_checksum` for all 7 entries.
2. `$IRRADIANCE_CORPUS_DIR` resolution, defaulting to `tests/corpus/tier-b/`.
3. sha256 verified on present files; mismatch fails loudly naming the file.
4. **A visible skip** — `just test 2>&1 | grep SKIP` prints, naming the absent
   file, with no extra flags. Paste that output.
5. A `DEC-*` for the `toml` dev-dependency, stating the dev-only scope.
6. All existing gates green, output pasted.

## Out of Scope

- Any decoding — SPEC-003 onward.
- The `#[allow]` bypass — that is **SPEC-006**, no shared files.
- Re-opening `DEC-003`'s storage/schema decisions.
- A runtime (non-dev) dependency of any kind.

## Return Criteria — how to hand back

1. Paste the gates **and** the visible-skip output from deliverable 4.
2. Fill `## Completion` and `handback:`. For `tokens_total`: if `/cost` is
   unavailable, sum transcript usage objects and **say so**, with cache-read share.
3. `handoff.status: completed`; spec `task.cycle: verify`.
4. Branch `feat/spec-002-corpus-manifest-reader` off `main`; commit; do not merge.

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
