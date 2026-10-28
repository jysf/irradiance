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
  id: HANDOFF-062
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-10-28
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-023

project:
  id: PROJ-001
  stage: STAGE-005
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
  status: completed                # completed | blocked | rejected
  tokens_total: 33110588           # REAL combined count — what cost-audit reads
  estimated_usd: 67.72             # tokens_total × your rate, or your harness's number
  duration_minutes: 41
  branch: claude/spec-023-build
  pr: https://github.com/jysf/irradiance/pull/31
  completed_at: 2026-10-28         # YYYY-MM-DD
  notes: "Transcript-sum (deduped by message.id over this build session's own transcript, anchored on the scratchpad UUID — NOT a budget delta); all 128 assistant messages are claude-opus-4-8. tokens_total is the cache-inclusive sum (input 256 + output 171,181 + cache_creation 317,243 + cache_read 32,621,908), matching the repo's recorded-cycle convention. estimated_usd is a list-price estimate (output $75/M, cache-write $18.75/M, cache-read $1.50/M). No src/ change."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-062: audit every gate script for the mute-death shape

## Delegation Summary

The orchestrator (architect) hands `SPEC-023` to the implementer for the
**build** cycle: add a bespoke `scripts/gate-audit.sh` + its red-proof over the
verdict-bearing gate scripts (a check *over* the gates for the mute-death
shape), and fix the three reachable offenders it flags — `validate.sh`
(process-sub exit / no `parsed` assert), `handback-sync.sh` (no `superseded`
skip), `decisions-audit.sh` (unguarded `grep`) — the first two being SPEC-016's
two follow-ups. **No `src/` change.** The design is fully specified — read the
spec.

## Context the Receiving Agent Needs

The receiving agent MUST read these before starting work. Keep the
list tight, but don't omit anything necessary.

### Primary

- **Spec:** `./projects/PROJ-001-monochrome-dng-develop/specs/SPEC-023-audit-every-gate-script-for-the-mute-death-shape.md`
  — read it whole: **Inputs** is the offender map (exact `file:line`), **AC1–AC5**
  the contract, **Notes** the idioms and the `HANDOFF-038` de-kludge.
- **The shape:** `AGENTS.md §16 rule 3` (~`:1424–1437`) + the signal
  `a-gate-that-fails-mutely-is-a-gate-that-never-ran` (`guidance/signals.yaml`).
- **The red-proof template:** `scripts/validate-red-proof.sh` (control / inject /
  assert-landed / rc-capture / assert-message / isolation); CI pairs a gate with
  its red-proof (`.github/workflows/ci.yml:24–33`).
- **The three offenders:** `scripts/validate.sh` (~`:300–311`, `:392`; helper
  `_lib.sh:372–399`), `scripts/handback-sync.sh` (~`:51–90`, `:72`, `:164–175`;
  live proof `HANDOFF-038`), `scripts/decisions-audit.sh` (~`:217`). Plus the
  lower-risk variants `cost-audit.sh`, `_lib.sh:135–137`, `test.sh`.
- **Toolchain brief:** `./guidance/toolchain-brief.md` — pinned vs local clippy,
  zsh `$pipestatus`, the `just` gate recipes. Read it.

### Decisions that apply

- `DEC-009` — a red-proof needs a negative control (the shape to mirror).
- `DEC-017` — the red-proof rebuilds a mutated COPY (working tree untouched).
- `DEC-006` — lint-policy red-proof mechanics (lineage). There is **no DEC on
  gate-script conventions** — that lives in AGENTS.md §16; do not write a new one.

### Constraints that apply

Full text in `./guidance/constraints.yaml`. Touching `scripts/`, `ci.yml`,
`app.just`, one handoff, AGENTS.md:

- `oracle-must-be-shown-red` (~`:53–56`) — every gate (incl. the new
  `gate-audit`) ships with a deliberate-fault test + a clean negative control.
- `test-before-implementation` — the red-proof + the two behavioral fixes are
  the spine.

### Prior related work

- **SPEC-016** (shipped, PR #23/#24) — fixed ONE gate (`validate` now parses);
  its two follow-ups (validate process-sub exit, handback-sync superseded-skip)
  are two of this spec's offenders. `validate-red-proof.sh` is its red-proof.
- **SPEC-011** (shipped, PR #28/#30) — the most recent shell-gate + red-proof +
  CI-job spec; the shape to copy for `gate-audit` + its CI pairing.
- **SPEC-003** (shipped) — widened `oracle-must-be-shown-red` to cover gates
  (FU-14); the same uncovered-graph argument DEC-011 made for licensing.

## Expected Deliverables

*(For a `verify` handoff, replace this block with the verify contract below.)*

- **`scripts/gate-audit.sh`** (new) — the bespoke scanner (comment-excluding,
  asserts its own scan count) + **`scripts/gate-audit-red-proof.sh`** (mirror
  `validate-red-proof.sh`); `app.just` recipes + AGENTS.md §6; two CI jobs paired
  adjacently (AC1, AC2).
- **The three offender fixes** (AC3): `validate.sh` (rc-capture + `parsed ==
  target` assert + broaden the ruby rescue; a non-SyntaxError fault goes RED),
  `handback-sync.sh` (skip `handoff.status == superseded` + **de-kludge
  HANDOFF-038**: restore its `handback` to null, drop the manual `synced_at`),
  `decisions-audit.sh` (guard the `grep`). Each proven.
- **Any lower-risk variant gate-audit flags** (`cost-audit.sh`, `_lib.sh`,
  `test.sh`) fixed so the audit ships green (AC4); name which in the handback.
- **No `src/` change** — assert `git diff` touches no `src/`.
- All gates green, `just lint-ci` + the existing suite + the two new jobs, CI
  observed green on the PR head; name the gate list.
- The handback filled with a REAL transcript-sum `tokens_total` (deduped by
  `message.id` via your scratchpad UUID — say so; **not a budget-delta**). A PR
  against `main` referencing HANDOFF-062, SPEC-023, STAGE-005, PROJ-001, DEC-009,
  DEC-017.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered
  honestly (*"nothing was unclear"* is suspicious)?
- Decision drift — run `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- For any criterion claiming **runtime behavior** (a component registers, a hook
  fires, a binary resolves on PATH, a config takes effect), confirm the
  *behavioral* surface was exercised — not just the shape validated. That is the
  defect class that escapes (AGENTS.md §12).
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

Explicit list of what this handoff does NOT include. If the receiving agent
thinks any of these need to happen, they should create a new spec in
the stage's backlog, not expand this handoff.

- **The `next_id` / `new-spec` fixture-pollution bug** — recorded as a signal
  (`next-id-scans-test-fixtures`); it is a scaffolder, not a gate. Separate patch.
- **`shellcheck` as the engine** — bespoke `gate-audit` is primary; a
  complementary shellcheck CI job is optional, defer if it churns.
- **The view / scaffold scripts** (`*-view.sh`, `dash`, `status`, `new-*`) —
  out of the audit's subject set.
- **Any `src/` change**; rewriting gate logic beyond the mute-death fixes.
- The four deferred `(not yet written) [XS]` STAGE-005 items.

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

- **Branch / PR:** `claude/spec-023-build` → PR against `main` (link in the front-matter `pr:`).
- **Completed at:** 2026-10-28
- **All acceptance criteria met?** Yes.
  - **AC1** — `scripts/gate-audit.sh` scans the 8 verdict-bearing gate scripts
    (`validate`, `cost-audit`, `decisions-audit`, `decisions-index`,
    `handback-sync`, `ready`, `_lib`, `test`), flags all three mute-death shapes,
    excludes comments (the honest tree is green even though `validate.sh`'s own
    header now contains `done < <(front_matter_parse_report …)` in prose), and
    asserts its own scan count (dies if the glob or the exclusion list drops a
    gate, or if the engine examines fewer files than it was handed). A CI step runs it.
  - **AC2** — `scripts/gate-audit-red-proof.sh`: clean control, each of the three
    shapes injected into a COPIED gate caught BY NAME + shape slug, a shape-shaped
    COMMENT NOT flagged, isolation restores green, working tree untouched. Paired
    adjacently with AC1's CI step.
  - **AC3** — all three offenders fixed and each proven in a temp copy (working
    tree untouched): (a) `validate.sh` now captures the oracle's rc from a temp
    file, asserts it SCANNED every target, and treats rc≠0-with-no-FAIL as a crash;
    the ruby helper's rescue was broadened to a bare `rescue` wrapping `File.read`,
    so a non-`Psych::SyntaxError` fault (proof form: an **unreadable file** →
    `Errno::EACCES`, which the old `rescue Psych::SyntaxError` left uncaught and a
    `done < <(…)` swallowed into a green "0 parsed") now goes RED by name;
    `validate-red-proof.sh` still passes. (b) `handback-sync.sh` skips a
    `handoff.status: superseded` handoff (tallied `superseded`, not `pending`); a
    temp-copy run proved HANDOFF-038 skips clean with exit 0, and the counter-proof
    (flip it to `pending`) returns exit 1 — the skip is specific to superseded.
    HANDOFF-038's manual `synced_at` kludge + note were removed, its handback
    restored to null. (c) `decisions-audit.sh` guards the `^# DEC-` title grep with
    `|| true`; a temp git copy proved a DEC with no H1 no longer aborts the lint.
    `gate-audit` flags none of the three after the fixes.
  - **AC4** — the only lower-risk variant `gate-audit` flagged was **`test.sh`** (19
    unguarded `grep`s in command substitutions; the design cited ~`:2451/2747`,
    there were more). All 19 guarded with `|| true`. `cost-audit.sh` and
    `scripts/_lib.sh:get_active_project` were handled too: `_lib.sh:136` had a bare
    `find … | grep -v example | sort` (multi-line command sub) — guarded. `cost-audit.sh`
    was NOT flagged (its `done < <(find_all_* …)` process subs feed enumerators,
    not verdict oracles, so they are not shape-1); it ships unchanged and
    `cost-audit` + `cost-audit-red-proof` still pass.
  - **AC5** — `gate-audit` + `gate-audit-red-proof` appear in AGENTS.md §6's block
    and as `app.just` recipes; `git diff` touches **no `src/`** (asserted). Gate
    list run green locally: `gate-audit`, `gate-audit-red-proof`, `validate`,
    `validate-red-proof`, `cost-audit`, `cost-audit-red-proof`, `decisions-index
    --check`, `decisions-audit`, `lint-ci` (clippy clean, pinned `+stable`). The
    two new CI steps sit in the `cost-data` job adjacent to the `validate` pair.

### Cost self-report

- **Tokens (total):** 33,110,588 — transcript-sum, deduped by `message.id` over
  this build session's own transcript (anchored on the scratchpad UUID
  `00c62d73-…`, not a text grep), cache-inclusive (input 256 + output 171,181 +
  cache_creation 317,243 + cache_read 32,621,908). All 128 assistant messages are
  `claude-opus-4-8`. **Not** a budget delta.
- **Estimated USD:** ≈ 67.72 — list-price estimate (output $75/M, cache-write
  $18.75/M, cache-read $1.50/M, input $15/M).
- **Duration (minutes):** 41 (first→last transcript timestamp).
- **Source of the number:** transcript `message.usage`, summed and deduped.

### Drift and new artifacts

- **New decisions emitted:**
  - None. The mute-death convention lives in AGENTS.md §16; the red-proof
    mechanism is DEC-009 / DEC-017, cited not re-decided (per the spec).
- **Deviations from spec:**
  - `validate.sh`'s count-assert is phrased as `scanned == ${#parse_targets[@]}`
    (the oracle now reports how many targets it examined), **not** the literal
    `parsed == ${#parse_targets[@]}` the Notes suggested: `parsed` counts only
    files that HAVE front matter, and many targets legitimately have none (prompt
    files, timelines), so a literal `parsed == target` assert would fire on the
    honest tree. `scanned` is the faithful realization — it proves the oracle ran
    to completion over every target. Combined with the rc-capture it is strictly
    stronger.
  - `handback-sync` tallies the superseded skip in a NEW `superseded` counter
    (not lumped into `skipped`/"already synced"), so the summary does not claim a
    never-dispatched handoff was synced — truer to this spec's own theme.
  - CI: the two new checks are adjacent STEPS in the existing language-agnostic
    `cost-data` job (the literal `ci.yml:24–33` pattern, where `cost-audit` +
    `cost-audit-red-proof` are steps), not separate top-level jobs.
  - shellcheck job: deferred (Non-Goals) — it would churn on pre-existing style
    and is not the engine for this semantic class.
- **Follow-up work identified:**
  - `next-id-scans-test-fixtures` (already a `process-debt` signal, status
    `watch`) — the `next_id` fixture-pollution bug; fix is a separate patch.
  - The red-proof scripts (`*-red-proof.sh`) and `gate-audit.sh` itself are
    excluded from `gate-audit`'s subject set by design (they embed the shape as
    injection fixtures / detection patterns). A future spec could add a narrower
    "no mute-death OUTSIDE the deliberate injection" check for red-proofs if ever
    warranted; not needed today.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The literal `parsed == ${#parse_targets[@]}` assert in the Notes is
   self-contradictory against the repo's real files (many targets have no front
   matter, so `parsed` < targets on the honest tree). It took a measurement to
   see that the faithful realization is an oracle-reported *scanned* count, not
   `parsed`.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `oracle-must-be-shown-red` (gates), `test-before-implementation`,
   DEC-009/DEC-017, and AGENTS §16 rules 2–3 covered everything; no new DEC was
   needed, exactly as the spec predicted.

3. **If you did this task again, what would you do differently?**
   — Write the `gate-audit` engine in Ruby from the start. The precise part is
   not the regexes but the *masking* — comments, heredoc bodies, and multi-line
   single-quoted spans (the embedded awk/ruby in `_lib.sh`/`handback-sync.sh`),
   which a line-oriented grep/awk scanner cannot do without false positives. Two
   early iterations were lost to the US-ASCII default encoding on stdin-fed Ruby
   before setting `Encoding.default_external = UTF_8`.
