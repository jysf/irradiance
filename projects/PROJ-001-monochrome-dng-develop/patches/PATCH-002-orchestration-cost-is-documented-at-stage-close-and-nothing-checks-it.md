---
# A PATCH is a lightweight fix to ALREADY-SHIPPED behavior (a bug or UX
# papercut) that adds NO new feature/command and doesn't warrant a full
# spec + stage. See AGENTS.md "Patch lane" and docs/decisions/DEC-003.
#
# Collapsed cycle: patch -> verify -> ship (design+build fused into one
# test-first pass; the INDEPENDENT verify is KEPT). It uses the same task.*
# schema as a spec, so `just validate`, `just cost-audit`, and `just status`
# treat a patch as first-class.

task:
  id: PATCH-002
  type: patch                      # epic | story | task | bug | chore | patch
  cycle: verify                    # patch | verify | ship  (collapsed from a spec's 5)
  blocked: false
  priority: medium
  complexity: S                    # S | M  (an L fix is probably a spec, not a patch)

project:
  id: PROJ-001
  # No `stage:` — a patch attaches to the PROJECT, not a stage.
repo:
  id: irradiance

agents:
  implementer: claude-opus-5  # the patch pass (tier_map.build; DEC-005)
  verifier: claude-opus-5        # independent verify — KEPT (tier_map.verify; a separate session/agent)
  created_at: 2026-09-06

references:
  decisions: []                    # add a DEC only when there's a real decision

# Cost: patch + verify are the metered cycles — `just cost-audit` requires a
# real tokens_total on both for a shipped patch. ship is main-loop (null-with-note).
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# PATCH-002: orchestration cost is documented at stage close and nothing checks it

## Problem

The stage template carries an `orchestration_cost:` block whose own comment says
**"THE ORCHESTRATOR FILLS THIS — not the human"**, with a rationale in
`docs/decisions/DEC-013-delegated-cost-handback.md` §5. It has been in the repo
since the 2026-08-15 scaffold.

**Nothing has ever checked it, and it was skipped the one time it could have
been.** `STAGE-001` shipped on 2026-08-22 with `sessions: []`, and no gate,
report or `just status` line noticed for fifteen days. `STAGE-002`'s close on
2026-09-06 is the first time the field has ever been filled — and that happened
because the orchestrator remembered, which is exactly the failure mode
`brag-step-skipped-at-ship` records (six ships, zero entries, caught by a human).

The number is not a rounding error. `STAGE-002` measured **~84.2M** tokens of
orchestration against **187.0M** of delegated spec cost — roughly **31 %** of the
stage's total spend, and spend that no spec's `cost.sessions` would ever record.
A cost model that silently omits a third of the bill is worse than one that omits
all of it, because it looks complete.

⚠ **A near-miss worth recording:** the first attempt to answer *"has this ever
been filled?"* used `grep tokens_total` and reported **all five stages FILLED**.
The match landed on the template's own commented example
(`sessions: []  # - tokens_total: N`) rather than on data —
`attribute-text-inside-doc-comments` (AGENTS.md §16 rule 2), on the same day
rule 4 was codified. The gate below is written specifically so it cannot make
that mistake, and the red-proof is written specifically to catch it if it does.

## Fix

1. **`scripts/_lib.sh`** gains `find_all_stages`, `stage_has_orchestration_cost`
   and `is_grandfathered_stage_orch`. The detector anchors on a real YAML list
   item (`- tokens_total: <digits>`) inside the `orchestration_cost:` block and
   **refuses any line whose first non-space character is `#`**.
2. **`scripts/cost-audit.sh`** gains a third loop: every stage with
   `status: shipped` must record at least one real orchestration session. It
   fails through the gate's own `die`, naming the artifact and the field. The
   reason string now has **one source** feeding both the human line and the JSON
   `missing_cost`, so they cannot drift.
3. **`STAGE-001` is grandfathered**, by the same mechanism and for the same
   reason as `COST_AUDIT_GRANDFATHERED`: it shipped before the gate existed, its
   orchestration ran across a week of sessions with no per-stage boundary
   recorded, and any figure reconstructed now would be **invented rather than
   measured** — which AGENTS.md §4 already forbids ("a null here is honest; a
   guess is not").
4. **`scripts/cost-audit-red-proof.sh`** proves the gate can fail, and **CI runs
   it beside the gate it proves**.

**No `DEC-*`.** This adds a gate for a decision already recorded in `DEC-013` §5
and the stage template; it decides nothing new.

## Failing Tests

- `./scripts/cost-audit-red-proof.sh` — control clean → a shipped stage whose
  `orchestration_cost` is the **unfilled template, comment and all** is rejected
  **by name, with a reason** → the grandfathered `STAGE-001` stays exempt.
- `./scripts/cost-audit.sh` — green on the honest tree.

## Verification (independent — KEPT)

Run in a SEPARATE session/agent from the patch pass. This is the one discipline
the framework retrospective proved catches real defects; it is non-negotiable
for a patch.

- Run the project's full gate suite (tests, lint/format, and any security/
  dependency gates the repo defines).
- Confirm the failing tests now pass and no existing test regressed.
- Output: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Patch Completion

*Filled at the patch pass. `PATCH-003`'s verify raised this as `FU-9`: this
section was the unfilled template stub while the patch sat on `main` — the same
documented-step-with-no-surface shape the patch itself is about.*

- **Branch / SHA:** `fix/patch-002-orchestration-cost-has-no-gate` → `705c784`,
  merged to `main` as PR #9 (`0cda7d6`).
- **What changed:** `cost-audit` gained a third loop over shipped stages;
  `_lib.sh` gained `find_all_stages`, `stage_has_orchestration_cost` and
  `is_grandfathered_stage_orch`; `cost-audit-red-proof.sh` added; one CI step.
- **Gates:** ten + `lint-ci` green (clippy 0.1.98 asserted), 152 tests, CI 9/9
  on `705c784` (run `34023570708`).
- **Independent verify:** ⚠ **ran AFTER the merge**, at the maintainer's
  direction, to close an ID-collision window. It returned **⚠ PUNCH LIST — 2
  ship-blockers, 10 follow-ups**. Both ship-blockers were real. `PATCH-003`
  remediates them; `PATCH-003`'s own verify then found two more, which
  `PATCH-003` round 2 closes.
- **`defect-catch-stage`:** `verify` — and it should be read as a warning
  rather than a success. Merging before the verify is exactly what let two
  ship-blockers reach `main`, and the cost of that decision is this chain of
  three patches.
- **Reflection:** the detector was written against a trap I had just fallen into
  (`grep tokens_total` matching the template's own commented example) and the
  red-proof's first draft still exercised the wrong path. Both were caught by
  mutation, neither by review-by-reading. The lesson is not "write a red-proof"
  — it was written — it is that **a red-proof's injection must reproduce the
  real shipped shape**, or it tests a file that never exists.

## Follow-ups

*Every finding raised against `PATCH-002` across every cycle, with its
disposition (§15). Added at `PATCH-003`'s verify (`FU-8`), which found this
patch had **crossed its own ship with these undecided — and merged to `main`
that way**. §15: "a follow-up is dispositioned at the ship cycle of the spec
that raised it, and never crosses that ship undecided." This one did.*

| id | finding | disposition |
|---|---|---|
| `SB-1` | the gate reversed `DEC-013` §5 and the patch said it decided nothing | `fixed` — `DEC-022` amends §5 explicitly; template + five stage files corrected (`PATCH-003`) |
| `SB-2` | the awk read the body as front matter, so prose satisfied the gate | `fixed` — delimiter counting (`PATCH-003`), then **fence normalisation + fail-closed** after `PATCH-003`'s own verify found the first fix was one space away |
| `FU-1` | the `#` guard is unreachable and its comment miscredited it as the anti-trap | `fixed` — kept as defence in depth, comment corrected to credit the anchor (`PATCH-003`) |
| `FU-2` | the red-proof claimed "rejected **by name**" while only asserting the reason | `fixed` — the name is now asserted; `M4b` caught (`PATCH-003`) |
| `FU-3` | `cancelled` stages are not audited | `closed` — a cancelled stage did not ship and has no orchestration to record. Recorded so it is a decision, not an oversight |
| `FU-4` | `status: "shipped"` silently skipped the check | `fixed` — quotes stripped, then CRLF handled too after `PATCH-003`'s verify found the same "opt out by adding characters" class one layer down |
| `FU-5` | fifth `+toolchain` instance: `just lint` / `lint-red-proof` use a bare `cargo clippy` | `fixed` — `PATCH-004` (PR #11). ⚠ Re-measured: the finding's symptom and its workaround were both wrong; Homebrew's clippy shadows the rustup shim, so both commands *succeed* with an unselected compiler |
| `FU-6` | `tokens_total: 0` satisfied the gate while the spec-side gate treats 0 as absent | `fixed` — now requires `> 0` (`PATCH-003`) |
| `FU-7` | the red-proof copied `target/` — 105 s local vs 3.8 s clean | `fixed` — copy scoped to what the gate reads; **118.32 s → 5.81 s** (`PATCH-003` round 2) |
| `FU-8` | the `die` message pointed at `docs/cost-tracking.md`, which never mentioned stage `orchestration_cost` | `fixed` — that section now exists (`PATCH-003` round 2) |
| `FU-9` | this patch's own `## Patch Completion` was the unfilled stub and `task.cycle` was still `patch` | `fixed` — both completed (`PATCH-003` round 2). ⚠ The same documented-step-with-no-surface shape the patch is about |
| `FU-10` | `STAGE_ORCH_COST_GRANDFATHERED=""` does not clear the list (`:-` falls back) | `fixed` (documented) — `" "` clears it; both forms verified. Kept for consistency with `COST_AUDIT_GRANDFATHERED`, but no longer silent |

## Ship

- Add a CHANGELOG entry under `[Unreleased] → Fixed`.
- Append cost sessions (patch + verify metered; ship null-with-note), then
  compute `cost.totals`.
- `just advance-cycle PATCH-NNN ship`, then `just archive-patch PATCH-NNN`.
- **No stage bookkeeping** — a patch attaches to the project, not a stage.
