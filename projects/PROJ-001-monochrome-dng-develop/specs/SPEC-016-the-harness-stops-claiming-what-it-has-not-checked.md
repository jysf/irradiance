---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-016
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   ⚠ RESCOPED 2026-09-06 from L → S. The prior design
                                   #   bundled five harness deliverables (AC1–AC5) and
                                   #   settled a design question. That shape — five
                                   #   independent gate changes in one spec, each with
                                   #   its own red-proof — is the shape that produced
                                   #   three patches on one gate this week (PATCH-002,
                                   #   PATCH-003, PATCH-004). The pre-rescope design is
                                   #   preserved at git 3238dcb; the four deferred ACs
                                   #   are on STAGE-005's backlog as separate items,
                                   #   each with its measurements pointing back to
                                   #   3238dcb's Implementation Context. See
                                   #   `## Rescope note` below.
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
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-013]
  constraints: [oracle-must-be-shown-red, test-before-implementation, cost-captured-per-cycle]
  related_specs: [SPEC-015]                # [SPEC-NNN]

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
value_link: "STAGE-005: `just validate` parses the front matter it validates, so an artifact no YAML parser can read cannot ship claiming valid front-matter"

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
  tokens_estimate: 30000000
  # Calibration basis. Pre-rescope estimate was 90M for five deliverables. Rescoped
  # to one deliverable of the five (AC5 → the new AC1: `just validate` parses YAML).
  # The AC5 measurements at 3238dcb are already done — the reproduction step, the
  # affected artifacts (SPEC-015/FU-4, FU-12), the parser choice constraint
  # (`ruby -ryaml` present, `python3` has no pyyaml here). Build should be script
  # rewrite + one red-proof + a repo-wide sweep to prove no other artifact fails.
  # 30M assumes build + verify + one punch-list round, matching the S-shaped part
  # of the prior sessions rather than the L-shaped whole.
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-06
      notes: "main-loop, not separately metered (AGENTS.md section 4). The pre-rescope design at 3238dcb measured all five carried findings (AC1 corpus-status overclaim; AC2 req() truncation; AC3 SUPPORTED_BITS test coverage; AC4 fuzz seed white_level; AC5 just validate greps instead of parsing). Rescope 2026-09-06: kept AC5 (the half that generalises — a gate that fails when the harness lies about structure); deferred the other four to STAGE-005's backlog as separate items, each pointing back to 3238dcb for measurements. Reason: bundling five independent gate changes with five red-proofs is the shape that produced three patches on one gate this week (PATCH-002 through PATCH-004). One-defect-per-spec is what closes that class."
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-016: The harness stops claiming what it has not checked

## Context

> **Rescoped 2026-09-06 from L → S.** Only AC5 of the pre-rescope design (`just
> validate` greps front matter instead of parsing it) survives here. The other
> four are on STAGE-005's backlog as separate items. See `## Rescope note` for
> the reasoning and the mapping. The pre-rescope design lives at git `3238dcb`
> for its measurements.

`just validate` reports *"N artifact(s) … have valid required front-matter"*
by grepping for required keys. It never parses. Measured: two artifacts whose
front matter raised `Psych::SyntaxError` were reported as valid; one shipped,
was archived, and survived three later specs before anyone noticed. Repo-wide
sweep at design: **2 of 75** artifacts today would fail a real parse; the
gate today declares 0.

This is one instance of a class the stage was chartered around — a surface
reporting a result it has not established — and it is the instance that
generalises to every other artifact this repo will ever ship. A parse gate
catches every future SPEC-*, HANDOFF-*, PATCH-*, DEC-* whose front matter
becomes unparseable, without knowing what specifically went wrong. The other
four instances the pre-rescope design carried are each about one specific
surface (a pre-flight line, a constant, a fuzz fixture, a doc comment) and
fix that surface only.

The precedent this spec settles a rerun of: `handback-sync-truncates-multi-
line-scalars` (signal, bar 2, open). That defect wrote unterminated quotes
into spec front matter, and every gate — including `just validate` — passed
green over it. A parse gate on `validate` is one of the two candidate fixes
that signal names; this spec ships that candidate.

## Goal

Make `just validate` parse the front matter of every artifact under
`projects/`, `decisions/`, `handoffs/` and `patches/`, fail loudly on a parse
error naming the file and the parser's own message, and ship with the
falsifier that made the defect measurable at design (the two known-bad
artifacts, mutated in a test copy per DEC-017's mechanism).

## Inputs

- `scripts/validate.sh` + `scripts/_lib.sh` — the current gate. Contains
  **zero** references to a YAML parser and reads front matter with 12
  line-oriented `awk`/`grep`/`sed` operations.
- `guidance/signals.yaml` — `handback-sync-truncates-multi-line-scalars`
  (both candidate fixes named; this spec ships one), and
  `named-tests-can-pass-vacuously` (related class: a check that succeeds
  without doing the work).
- `SPEC-015` — the shipped spec whose front matter was one of the two known
  parse failures (`SPEC-015/FU-4`, `FU-12`). Its unparseable front matter
  survived to archive.
- `DEC-013` — the field-shape rules `validate` is entitled to check; the
  parse gate does NOT recheck DEC-013's rules, only that the parser can
  reach them.

## Outputs

- **`scripts/validate.sh`** — rewritten to parse every artifact's front
  matter with a real YAML parser and fail on a parse error, naming the
  file and the parser's own message. The existing "required keys present"
  check is retained (it catches valid YAML with missing keys, which parse
  alone would not).
- **A red-proof case** — a hand-crafted artifact under
  `tests/oracle-fixtures/` (or wherever this repo puts red-proof fixtures
  — state in handback) whose front matter is grep-satisfiable but
  parse-broken, in the exact shape `handback-sync` produces (multi-line
  scalar with unterminated quote). Show the OLD script passing on it and
  the NEW script failing on it, with the parser's message.
- **`.github/workflows/ci.yml`** — no change if `just validate` already
  runs in CI; a new job if not. State which in the handback.
- **No new provenance row** — this adds no algorithm or decoder.

## Acceptance Criteria

- [ ] **AC1 — `just validate` parses every artifact's front matter with
      a real YAML parser and fails on a parse error.** Under
      `projects/**/*.md`, `decisions/*.md`, `handoffs/*.md`,
      `patches/*.md`. A parse failure exits non-zero, names the file, and
      prints the parser's own error message so the reader sees the exact
      column the YAML broke at. Measured at design: **2 of 75** artifacts
      would fail a real parse today; the gate declares 0. Both are named
      in the handback so the reviewer can reproduce the finding.

      ⚠ **Use a parser that is already present.** `ruby -ryaml` is on
      macOS and on GitHub's runners; `python3` here has **no** `pyyaml`
      (measured — `python3 -c "import yaml"` fails). State which you
      chose and confirm it exists on the CI runner image. Do NOT add a
      dev dep, a Rust binary, or a container just to parse YAML — this
      gate runs in a shell script and stays there.

      **Test:** `validate_rejects_front_matter_that_no_parser_can_read`
      (tier A — a hand-crafted fixture with an unterminated quote in a
      multi-line scalar; the shape `handback-sync` produced in the
      shipped-then-archived incident).

- [ ] **AC2 — the falsifier is watched red.** File changed AND the
      script compiles AND its output changed on the fixture — the third
      clause has caught four false red-proofs in three specs this month,
      and one in PATCH-002 two days ago where the obvious injection
      exercised the wrong path. Paste the OLD-script pass and the
      NEW-script fail, both against the same fixture bytes.

- [ ] **AC3 — ten gates + `just lint-ci`, CI observed green on the
      shipping SHA.** ⚠ The gate count is genuinely ambiguous in this
      repo (`the-gate-count-is-not-defined-anywhere`, bar 3, open); say
      which list you ran. Do not resolve the ambiguity here — that is a
      repo decision and out of scope. `just lint-ci`, not `just lint`
      (Homebrew's clippy is 0.1.97; CI's is 0.1.98 — the divergence has
      cost this repo 17 consecutive red runs once).

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum
across all — the trap `SPEC-015/AC9`'s Failing-Tests header calls out.
The AC1 test here is a shell test, not a cargo test, but the same rule
applies: prove it *ran* on the fixture, not that no failure was reported.

- `validate_rejects_front_matter_that_no_parser_can_read` — AC1, tier A
  (shell test; state its shape in the handback — a `bats` test, a
  standalone shell script, or a case appended to an existing test target)

## Non-Goals

- **The four other pre-rescope ACs** (corpus-status overclaim, `req()`
  truncation, `SUPPORTED_BITS` coverage, plane fuzz `white_level`) — all
  on STAGE-005's backlog as separate items, each pointing at git
  `3238dcb` for the measured reproduction. See `## Rescope note`.
- **The full gate-script audit.** STAGE-005 already carries it as a
  separate bullet. Sized at 3238dcb's design: 8 of 13 `pipefail`
  scripts carry at least one unguarded `grep`, 28 of them in the
  template's own `test.sh`. Folding it in would make this XL, which
  §15 says is a stage, not a spec.
- **Fixing `handback-sync`'s writer.** This spec makes the defect
  *detectable* at validate-time — the half that generalises — but does
  not fix `handback-sync`. Both candidate fixes remain in
  `handback-sync-truncates-multi-line-scalars` (bar 2, open).
- **Defining "the gates."** Two enumerations differing by four members
  are cited in shipped artifacts; `the-gate-count-is-not-defined-
  anywhere` (bar 3, open) covers it. AC3 works around it deliberately.
- **Anything in `src/`.** No decoder behaviour changes here.
- Opcodes, tone curve, demosaic — STAGE-003.

## Rescope note

**Rescoped 2026-09-06 from L → S.** The pre-rescope design at git
`3238dcb` bundled five ACs:

| pre-rescope AC | subject                                        | disposition           |
|---              |---                                            |---                    |
| AC1             | `corpus-status` overclaims tool availability  | deferred → STAGE-005  |
| AC2             | `req()` silently truncates multi-valued tags  | deferred → STAGE-005  |
| AC3             | `SUPPORTED_BITS` outruns its tests            | deferred → STAGE-005  |
| AC4             | plane fuzz seeds lack `white_level`           | deferred → STAGE-005  |
| **AC5**         | **`just validate` greps instead of parsing**  | **KEPT — this spec's AC1** |

**Why the split.** Five independent gate changes in one spec, each
needing its own red-proof, is the shape that produced three patches
this week — PATCH-002 shipped a gate whose verify found two
ship-blockers; PATCH-003 fixed them and its verify found two more;
PATCH-004 fixed one deferred instance from PATCH-003. The lesson
codified from that sequence — `a-fix-inherits-the-precondition-of-the-
thing-it-fixes` (signal, watch, N=2) — argues explicitly that a spec
bundling several fixes lets one bad precondition survive under cover of
the others. One defect per spec is what breaks the pattern.

**Why AC5 kept.** It is the one whose fix generalises. A parse gate on
`validate` catches every unparseable front matter this repo will ever
produce, without knowing the specific defect (the yet-unseen
`handback-sync` bug the pre-rescope design named is only one shape).
The other four each fix one specific surface. Ship the generalising one
first; the surface-specific ones sit on the stage backlog on appetite.

**Measurements preserved.** All four deferred items' Implementation
Context measurements live at git `3238dcb`. A future session picking any
of them up should reproduce those numbers rather than re-deriving them —
same rule as SPEC-012/FU-1 in the pre-rescope design.

## Implementation Context

> Measured 2026-09-06 by the pre-rescope design session against the
> working tree at `main`. Reproduce these numbers rather than
> re-deriving them.

### AC1 — the gate that succeeds mutely

`scripts/validate.sh` contains **zero** references to a YAML parser and
reads front matter with 12 line-oriented `awk`/`grep`/`sed` operations.
Measured consequence: two artifacts whose front matter raised
`Psych::SyntaxError` were reported as *"valid required front-matter"*;
one shipped, was archived, and survived three later specs. Repo-wide
sweep after repair: **0 of 75** failing; it was 2. The two known-bad
artifacts are named at 3238dcb's Implementation Context (`SPEC-015/FU-4`
and `FU-12`).

⚠ **`python3` in this repo has no `pyyaml`** (measured — `import yaml`
fails). `ruby -ryaml` works and is present on macOS and GitHub runners.
AC1 asks you to confirm your choice exists in CI rather than assume it.

### Traps

- ⚠ **All tests are tier A.** The finding is about a surface that lies
  *when the corpus/tools are absent*, and CI is the environment where
  things are absent. A tier-B version reproduces
  `ci-cannot-prove-bit-exactness`, which this spec exists not to
  compound.
- ⚠ **The red-proof must show the fixture bytes AND the parser message.**
  A pass/fail exit code alone does not distinguish a gate that ran on
  the fixture from one that skipped it — the class this whole spec
  exists to close.
- **`just lint-ci`, not `just lint`, and read CI.**

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
