---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-023
  type: story                      # epic | story | task | bug | chore
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: null             # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.

project:
  id: PROJ-001
  stage: STAGE-005
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5          # build delegated via HANDOFF-062
  created_at: 2026-10-28

references:
  decisions: [DEC-009, DEC-017, DEC-006]
  constraints: [oracle-must-be-shown-red, test-before-implementation]
  related_specs: [SPEC-016, SPEC-003, SPEC-011]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-005's <capability>". Optional; null is acceptable.
value_link: "STAGE-005: the mute-death shape (a gate that reports green without its check running) is now caught by a check OVER the gates, plus the two gates SPEC-016 left half-fixed (validate's process-sub exit, handback-sync's superseded skip)."

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 25000000        # build + verify; S (borderline M): new audit gate + red-proof + 3 shell fixes, no src
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-10-28
      notes: "Design in the orchestrator main loop (un-metered; null-with-note per AGENTS.md §4). A read-only Explore subagent surveyed all 50 gate scripts (168,430 tokens, 56 tool-uses): re-measured the mute-death shape (8-of-13 at 3238dcb -> 3 reachable offenders today: validate.sh, handback-sync.sh, decisions-audit.sh), confirmed SPEC-016's two follow-ups live, recommended a bespoke gate-audit over shellcheck, and confirmed the next_id fixture-pollution bug. That cost is the subagent's, not transcribed here."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-023: Audit every gate script for the mute-death shape

## Context

> **Framed 2026-10-28 as STAGE-005's capstone** — the stage backlog bullet
> "Audit every gate script for the mute-death shape", the generalisation of the
> three hardening specs (SPEC-016/021/011) shipped before it.

STAGE-005 is "harden the harness — the gates …". `SPEC-016` fixed ONE gate that
lied: `just validate` reported *"valid required front-matter"* by grepping for
keys, never parsing, so an artifact no parser could read passed. That is one
instance of the **mute-death shape** — a gate script that reports success (or
dies) *without its check actually running*. The shape recurs because every gate
here is a `set -euo pipefail` shell script, and in that context:

- an **unguarded `grep`** returns exit 1 when it finds nothing, which under
  `set -e` / `pipefail` aborts the script (or, piped, aborts the gate) rather
  than meaning "zero matches";
- a **command/process substitution** (`$(…)`, `done < <(…)`) hides its exit
  code from `set -e`, so a crash inside it can leave the gate reporting green;
- a gate can print its success line on a path where the check never ran.

The original design (git `3238dcb`) measured **8 of 13** `pipefail` scripts
carrying at least one unguarded `grep` — stale now (SPEC-016 fixed `validate`,
this session fixed `archive-spec`'s `xargs`, and the script set has grown to
~40; **re-measured at design**). SPEC-016 shipped TWO follow-ups that are this
spec's seed offenders, recorded in `guidance/signals.yaml`: (a)
`validate.sh`'s `front_matter_parse_report` is read through a process
substitution whose non-zero exit is invisible to `set -e` (a non-`SyntaxError`
crash could still report "0 parsed" green); (b) `handback-sync.sh` has no skip
for a superseded, never-dispatched handoff (it tripped at SPEC-016 ship, worked
around by a manual `synced_at`).

Per `oracle-must-be-shown-red` (widened to cover gates, SPEC-003/FU-14), the
audit ships with its own red-proof. And the audit is itself a gate — a check
*over* the gates — so the shape cannot creep back into a future script
unnoticed, the way it took three specs to notice `validate`'s.

## Goal

Audit every `scripts/*.sh` gate for the mute-death shape, fix the offenders the
audit finds (including SPEC-016's two follow-ups), and ship the audit itself as
a CI gate with a red-proof — so a gate that reports green without running its
check is caught by a check *over* the gates, not by a defect three specs later.

## Inputs

(`file:line` cites are design-time, re-measured on `claude/spec-023`; read the
named item.)

- **The shape, authoritative:** `AGENTS.md §16 rule 3` (~`:1424–1437`) — "every
  `grep` a gate depends on is guarded (`|| true`) and its match count asserted;
  a gate never exits on a pipeline's own status, it exits through its own error
  message, with a reason" — plus the two measured traps there. The lesson signal
  is `a-gate-that-fails-mutely-is-a-gate-that-never-ran` (`guidance/signals.yaml`,
  codified, N=4).
- **The 50 `scripts/*.sh`** (all carry `set -euo pipefail`). The audit's subject
  is the **verdict-bearing gate** scripts (those that emit a pass/fail verdict a
  gate relies on); the `*-view.sh` / `dash` / `status` / `new-*` scaffold-and-view
  scripts emit state, not verdicts, and are out of the subject set.
- **The three reachable offenders to fix:**
  - `scripts/validate.sh` (~`:300–311`, `:392`) — `done < <(front_matter_parse_report …)`
    swallows the helper's non-zero exit, and `parsed` (init `:68`) is never
    asserted against `${#parse_targets[@]}`, so a **non-`Psych::SyntaxError`**
    ruby crash reports green "0 … parseable". `_lib.sh:372–399` is the helper
    (`rescue Psych::SyntaxError` only). [SPEC-016 follow-up a]
  - `scripts/handback-sync.sh` (~`:51–90`, `:72`, `:164–175`) — a
    `handoff.status: superseded` handoff lands in the `pending` bucket and trips
    `exit 1` (a gate dying for a non-failure). Live proof: `HANDOFF-038`
    (`handoff.status: superseded`, `handback.status: null`) excluded only by a
    **manual `synced_at` kludge** this session added. [SPEC-016 follow-up b]
  - `scripts/decisions-audit.sh` (~`:217`) — `title=$(grep -m1 '^# DEC-' "$file" | sed …)`
    unguarded: a DEC missing its `# DEC-` H1 aborts the lint with no `die`.
- **Lower-risk structural variants** (same shape, fix iff the audit flags them):
  `scripts/cost-audit.sh` (`done < <(find …)` ~`:67/90/127`), `scripts/_lib.sh`
  `get_active_project` (~`:135–137`, bare `grep -v example`), `scripts/test.sh`
  (~`:2451/2747`).
- **The red-proof template:** `scripts/validate-red-proof.sh` — `mktemp -d` +
  `trap`, **control** (honest copy passes), **inject** + assert the fault landed,
  **assert red for the right reason** via `set +e; out=$(…); rc=$?; set -e` and a
  message assertion, **isolation** (remove → green). CI pairs a gate with its
  red-proof adjacently (`.github/workflows/ci.yml:24–33`).
- **Clean idioms to copy:** guarded grep `n=$(… | grep -c PAT || true); n=${n:-0}`
  (`scripts/ready.sh:77–79`); rc-capture (`validate-red-proof.sh:69–71`);
  `die`/`success`/`warn` in `_lib.sh:22–37`.
- **Constraint:** `oracle-must-be-shown-red` (`guidance/constraints.yaml:53–56`)
  — every gate ships with a deliberate-fault test + a clean control. **DECs** for
  the mutate-a-temp-copy mechanism: `DEC-009` (negative control), `DEC-017`
  (rebuild a mutated copy), `DEC-006` (lint red-proof mechanics). There is NO DEC
  on gate-script conventions — the shape lives in AGENTS.md §16.

## Outputs

A shell/CI gate spec — no `src/` change.

- **`scripts/gate-audit.sh`** (new) — a bespoke static scanner over the
  verdict-bearing `scripts/*.sh`, flagging the mute-death shape: (1) a process /
  command substitution feeding a gate's verdict without an rc-capture or a
  count-assert; (2) a bare `x=$(… grep …)` without `|| true`; (3) a
  `success`/green line not guarded by an assertion the checked-subject count is
  as expected. Exits non-zero naming each offender + which shape. **It is
  tooling that matches source, so (AGENTS §16 rules 2–3) it MUST exclude
  comments** (e.g. not fire on `validate.sh:296–299`, which is prose describing
  the shape) **and assert its own scan count** (how many scripts it examined).
  `shellcheck` is NOT the engine — SC2143/2181/2015 don't see this class; a
  complementary shellcheck job is optional (see Non-Goals).
- **`scripts/gate-audit-red-proof.sh`** (new) — mirrors `validate-red-proof.sh`:
  temp copy, control clean, inject a mute-death shape into a COPIED gate script,
  assert `gate-audit` catches it **by name, for the right reason** (not just a
  non-zero exit), isolation (remove → green). Working tree untouched.
- **`app.just`** — `gate-audit` + `gate-audit-red-proof` recipes; both commands
  added to **AGENTS.md §6**'s block.
- **`.github/workflows/ci.yml`** — a `gate-audit` job + its red-proof job, paired
  adjacently (the `cost-audit` + `cost-audit-red-proof` pattern, `ci.yml:24–33`).
- **Offender fixes** (at minimum the three reachable; plus whatever `gate-audit`
  flags, so it ships green):
  - `scripts/validate.sh` — capture the helper's rc (`set +e; …; rc=$?; set -e`)
    AND assert `parsed == ${#parse_targets[@]}` before the success line; broaden
    the ruby `rescue` to emit a `FAIL` and exit non-zero on ANY error (not only
    `Psych::SyntaxError`). A non-parse ruby crash then goes RED, not green.
  - `scripts/handback-sync.sh` — after the spec-match filter, skip a handoff
    whose `handoff.status == superseded` (via `get_handoff_field`), counting it
    `skipped` not `pending`. Then **de-kludge `HANDOFF-038`**: restore its
    `handback` to null-on-purpose and drop the manual `synced_at` + its note.
  - `scripts/decisions-audit.sh` (~`:217`) — guard the `grep -m1 '^# DEC-'`.
  - `scripts/cost-audit.sh`, `scripts/_lib.sh` (`get_active_project`), and
    `scripts/test.sh` — fix the structural variant **iff `gate-audit` flags it**;
    name which were flagged and fixed in the handback.
- **`guidance/signals.yaml`** — record the incidental `next_id` bug (see
  Non-Goals) as a `process-debt` signal; the FIX is out of scope here.
- **No `src/` change.** No new DEC (the convention is AGENTS §16; the red-proof
  mechanism is DEC-009/DEC-017, cited not re-decided).

## Acceptance Criteria

- [ ] **AC1 — the audit exists and is green after the fixes.** `just gate-audit`
      (`scripts/gate-audit.sh`) scans the verdict-bearing `scripts/*.sh` for the
      mute-death shape, exits non-zero naming each offender + which shape, and
      exits 0 on the honest tree once the offenders are fixed. It **excludes
      comments** (does not fire on `validate.sh:296–299`) and **asserts its own
      scan count** (the number of scripts examined, so a glob that matched
      nothing is a loud failure, not a silent pass). A CI job runs it.

- [ ] **AC2 — the audit is shown red (`oracle-must-be-shown-red`).**
      `scripts/gate-audit-red-proof.sh`: a clean negative control (honest
      scripts pass), a mute-death shape injected into a COPIED gate script is
      caught **by name and for the right reason** (the match targets the shape,
      not a comment or a coincidental string), and isolation (remove → green);
      working tree never mutated. A CI job runs it, paired with AC1's.

- [ ] **AC3 — the three reachable offenders are fixed, each proven.**
      (a) `validate.sh`: a non-`Psych::SyntaxError` ruby fault now makes
      `just validate` go **RED**, not green "0 … parseable" (rc captured +
      `parsed == target count` asserted). (b) `handback-sync.sh`: a
      `handoff.status: superseded` handoff is **skipped**, not `pending`, and
      **`HANDOFF-038` syncs clean with its `handback` restored to null** (the
      manual `synced_at` kludge removed). (c) `decisions-audit.sh`: a DEC missing
      its `# DEC-` H1 no longer aborts the lint silently. `gate-audit` no longer
      flags any of the three.

- [ ] **AC4 — the audit ships green over the whole tree.** Any lower-risk
      structural variant `gate-audit` flags (`cost-audit.sh`, `_lib.sh`,
      `test.sh`) is fixed; the handback names which were flagged and what was
      done. `just cost-audit` and its red-proof still pass.

- [ ] **AC5 — discoverable, gates green, no `src/` change.** `gate-audit` +
      `gate-audit-red-proof` appear with their commands in AGENTS.md §6's block;
      `just lint-ci`, the existing suite, and the two new jobs pass; CI observed
      green on the shipping SHA (name the gate list). `git diff` touches no
      `src/`.

## Failing Tests

The new gates PASS on the honest tree only AFTER the offender fixes; what makes
them load-bearing is the red-proof. The two behavioral fixes have failing tests
that exist as behaviors today (red until fixed).

- **`scripts/gate-audit-red-proof.sh`** — control (honest scripts pass) + a mute
  shape injected into a copied gate caught by name + isolation.
- **`scripts/validate.sh`'s fix** — a case proving a non-`SyntaxError` ruby fault
  (an artifact that makes `File.read`/`rstrip` raise, or a forced parsed-vs-target
  mismatch) turns `just validate` RED. State its form in the handback.
- **`scripts/handback-sync.sh`'s fix** — a check that a `superseded` handoff is
  skipped (not `pending`) and `HANDOFF-038` syncs clean with its `handback` null.

## Non-Goals

- **The `next_id` / `new-spec` fixture-pollution bug** — `_lib.sh:next_id` scans
  the whole repo and took the `SPEC-900` test fixture as the max, minting
  `SPEC-901` for this spec (hand-renamed to 023). It is a scaffolder, not a gate
  — no mute-death shape. Recorded as a signal; the fix (scope `next_id`'s `find`
  to `projects/` + `decisions/`, or exclude `tests/`) is a separate patch.
- **`shellcheck` as the audit engine** — it does not catch this class. A
  complementary `shellcheck` CI job is a reasonable *second* net but optional; if
  it adds scope or churns on pre-existing style, defer it to a follow-up and say
  so.
- **The view / scaffold scripts** (`*-view.sh`, `dash`, `status`, `new-*`) — they
  print state, not verdicts; out of the audit's subject set.
- **Rewriting gate logic** beyond the mute-death fixes; **any `src/` change**.

## Notes for the Implementer

- **The audit is itself source-matching tooling** (AGENTS §16 rules 2–3):
  exclude comments (`validate.sh:296–299` is prose describing the shape — a naive
  grep fires on it), assert the scan count, and ship the red-proof. A
  `gate-audit` that silently matches nothing is the very defect it hunts.
- **Mirror `validate-red-proof.sh`** for `gate-audit-red-proof.sh`
  (control / inject / assert-landed / rc-capture / assert-message / isolation);
  pair the two CI jobs adjacently (`ci.yml:24–33`).
- **Clean idioms:** guarded grep `n=$(… | grep -c PAT || true); n=${n:-0}`;
  rc-capture `set +e; out=$(…); rc=$?; set -e`. A gate exits through its own
  `die "<reason>"`, never a bare pipeline status.
- **`validate.sh` fix:** capture the helper rc AND assert `parsed ==
  ${#parse_targets[@]}`; broaden the ruby `rescue => e` to emit `FAIL` on any
  error. Re-run `validate-red-proof.sh` after — it must still pass.
- **`handback-sync` fix:** `get_handoff_field "$hf" status == superseded` →
  `continue` (skipped) right after the spec-match filter; then restore
  `HANDOFF-038`'s `handback` block to null and delete the manual `synced_at` +
  the workaround note (the kludge is now unnecessary).
- Scripts are bash (`#!/usr/bin/env bash`). `just lint-ci` for CI parity; zsh
  `$pipestatus` if you script locally.

---

## Reflection

*Appended during **ship**. Three questions, short answers.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`: `type: lesson` (with its N-count) for a recurring
   coding pattern, `type: process-debt` for tooling/process friction. A close
   then forces the decision. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(Runtime/operational defects — the escape-prone class — only exist once the
   artifact meets its real host. `escaped` here is a signal to strengthen the
   §12 behavioral pre-flight for that surface.)*

5. **What can a user do now that they couldn't before?** — one sentence,
   before → after; quote the confirming number if one exists, name the outcome
   if not. Write `none` if this spec has no user-visible outcome — that is a
   real, greppable result, not a blank. This is the line a downstream work-log's
   `impact` field is transcribed from, and both halves are already written above
   (## Context is the before, ## Goal is the after): confirm the prediction,
   don't reconstruct it from memory.
   — <answer | none>
