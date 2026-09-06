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
  complexity: L                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   ⚠ RAISED from the stage backlog's [M]. Five independent
                                   #   deliverables, each needing its own red-proof. Not XL: the
                                   #   gate-script audit is deliberately NOT in it (see Non-Goals).
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
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-003, DEC-008, DEC-013, DEC-014]
  constraints: [oracle-must-be-shown-red, test-before-implementation, no-panics-on-untrusted-input, cost-captured-per-cycle]
  related_specs: [SPEC-005, SPEC-010, SPEC-012, SPEC-015]                # [SPEC-NNN]

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
value_link: "STAGE-005: a harness that reports what it did not do is worse than one that is silent"

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
  tokens_estimate: 90000000
  # Calibration basis, written down so the next estimate can be judged rather than
  # guessed — the practice that measurably helped last time. SPEC-014 (L) estimated
  # 26M and cost 88.8M (3.42x). SPEC-015 (M) estimated 60M with a basis recorded and
  # cost 98.2M (1.64x) — writing the basis halved the error. This spec is L with FIVE
  # independent deliverables, each needing its own red-proof, but every one is
  # already MEASURED at design (see Implementation Context), so build should be
  # closer to transcription than discovery. 90M assumes build + verify + one
  # punch-list round, the shape both prior specs actually took.
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-06
      notes: "main-loop, not separately metered (AGENTS.md section 4). The design probe RAN and measured all five carried findings rather than inheriting them, which changed two of them. (1) SPEC-005/FU-3 reproduced exactly: with the corpus 7/7 present and exiftool+dnglab genuinely absent from PATH, the pre-flight prints 'corpus: 7/7 present - no tier-B test will skip' while all 30 oracle tests skip in 0.04s. (2) SPEC-012/FU-1 measured BY MUTATION rather than by reading: deleting 8 and 12 from SUPPORTED_BITS leaves 152/152 green, so two of four declared depths are exercised by nothing. (3) SPEC-012/FU-2 confirmed still open: white_level appears only in develop_fixture (SPEC-014's, lines 170/199), never in plane_fixture, so the plane target still cannot reach SampleExceedsWhiteLevel. (4) SPEC-005/FU-2 sharpened: req() does not merely truncate, its doc comment DOCUMENTS the truncation as deliberate and correct-for-this-corpus - a documented assumption with no test, which is the same class as the other four rather than a separate bug. (5) The newest instance, from SPEC-015's own cycles: just validate greps for required keys and never parses, so it reported 'valid required front-matter' on two files no YAML parser could read, one of which shipped and was archived undetected for two days. ALSO SIZED, and deliberately excluded: 8 of 13 pipefail gate scripts carry at least one unguarded grep, 28 of them in the template's own test.sh - that is the stage's separate 'audit every gate script' bullet, and folding it in would make this XL."

  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-016: The harness stops claiming what it has not checked

## Context

> **Framed 2026-09-03, not designed.** Destination for `SPEC-010/FU-2` and
> `SPEC-010/FU-3`. Both are the same defect in two places: **the harness
> reporting something it has not established.**

**`FU-2` — `req()` truncates a multi-valued required tag.** Measured:
`BitsPerSample "8 8 8"` reads `8`, and `diff()` returns `[]`. Latent on today's
monochrome corpus; **live the moment `SamplesPerPixel > 1`**, which is PROJ-002.
A real 3-sample TIFF makes `exiftool -T -n -s3 -BitsPerSample` print `16 16 16`.
Verify wrote and confirmed an 8-line fix that compiles and leaves all 29 green.

⚠ **This is the orchestrator's mis-disposition, twice, and the record should say
so.** `SPEC-005/FU-2` was dispositioned `spec: SPEC-010`; `SPEC-010`'s `AC4` was
then written narrower than the finding it carried (*"`BlackLevel = "512 999"`
must not read `Some(512)`"* — which is true and is only the **optional** half),
so the spec passed its own criterion without closing the finding. The build then
closed it with a trigger of *"someone remembering at PROJ-002"*, which
`AGENTS.md` §15 names explicitly as a **bad close**: *a close whose trigger is a
test that will fail is a good close; a close whose trigger is someone
remembering is not.*

**`FU-3` — `corpus-status` states something false.** With the corpus present but
`exiftool` off `PATH`, all 29 oracle tests skip in **0.01 s** while the
pre-flight prints, verbatim:

```
corpus: 7/7 present — no tier-B test will skip
```

That sentence is not one `corpus-status` is entitled to say: it knows about the
**corpus**, not about the **tools**. ⚠ This is materially worse than
`SPEC-005/FU-3`, where the surface was merely *silent*. `just test`'s pre-flight
is the one surface a reader trusts instead of reading 95 test names, and it is
currently the thing telling them the wrong answer.

**Why one spec.** Both are the harness asserting more than it checked, both are
small, and both live in the same lane. Fixing one and not the other would leave
the stage's own success criterion — *"no gate can exit non-zero without printing
its own reason"* — half true in the other direction.

**`SPEC-012/FU-1` and `FU-2` join this spec**, and they are the same sentence as
the two above: *the harness claiming what it has not checked.*

- **`FU-1` — `SUPPORTED_BITS = [8, 12, 14, 16]` declares four depths; two are
  executed by nothing.** Measured at `SPEC-012`'s verify: `bits = 8` and
  `bits = 12` have **zero** fuzz executions and **zero** tests, while being
  reachable from untrusted input. No corpus file uses either, so no oracle will
  ever cover them. This is `SPIKE-001`'s *"the parameter was always 14"* one
  level up — the list declares support the suite has never once exercised.
  ⚠ Verify drove both through the real API and they are **correct**:
  `8-bit → [1, 2, 3, 4]`, `12-bit AB CD EF → [2748, 3567]` (hand-derived as
  `0xABC` / `0xDEF`, byte order correctly irrelevant). **Test debt, not a
  defect** — which is exactly why nothing will find it later.
- **`FU-2` — the fuzz target never exercises `SampleExceedsWhiteLevel`**, the one
  assertion `DEC-008` calls load-bearing, because `examples/fuzz-seeds.rs`'s
  `plane_fixture` lacks the `white_level` field its test-side twin has. `AC4`
  covers the behaviour and verify proved that test has teeth; the **fuzz claim**
  is what was overstated.

**AC (added at `SPEC-012`'s ship, so the `spec:` disposition is real):** every
value in `SUPPORTED_BITS` is exercised by at least one test **and** reachable by
the fuzz target, asserted by enumeration rather than by inspection — and the
plane fuzz seeds carry `white_level`, so `SampleExceedsWhiteLevel` is reachable.
⚠ Written so that **adding a fifth depth to `SUPPORTED_BITS` without a test fails
this criterion**; a list and a test-set that can drift apart is the defect, not
the current gap.

Design question, not settled here: whether `corpus-status` should **check tool
availability** (and rename its claim) or merely **stop making the claim**. The
first is more useful and more code; the second is honest and is one line.

## Goal

Make five surfaces stop reporting results they did not establish — each fixed
with a test that fails if the claim is ever overstated again, so the class is
closed by a falsifier rather than by anyone remembering.

## The one sentence all five share

**A surface asserts more than it checked, and nothing fails when the assertion
is wrong.** They differ only in what does the asserting: a pre-flight line, a
constant, a fuzz fixture, a doc comment, and a gate. That is why this is one
spec — fixing four would leave the stage's success criterion (*"no surface
reports a result it did not establish"*) true in four places and false in the
fifth, which is indistinguishable from false.

⚠ **Every one is measured in `## Implementation Context`, and two changed on
measurement.** Reproduce; do not re-derive.

## Inputs

- `examples/corpus-status.rs` + `app.just`'s `test:` recipe — `AC1`'s surface
- `tests/support/tools.rs` — `reading_from_fields`'s `req` closure (`AC2`),
  around line 293, and its doc comment, which is the actual subject
- `src/plane.rs:45` — `SUPPORTED_BITS` (`AC3`)
- `examples/fuzz-seeds.rs` — `plane_fixture` at line 45 (`AC4`); note
  `develop_fixture` at 162 already has the field, and is the model
- `scripts/validate.sh` + `scripts/_lib.sh` — `AC5`
- `DEC-003` (corpus policy), `DEC-008` (the two-path rule and why
  `SampleExceedsWhiteLevel` is load-bearing), `DEC-014` (the `Unreadable`
  exemption `req` sits beside)
- `guidance/signals.yaml` — `ci-cannot-prove-bit-exactness`,
  `handback-sync-truncates-multi-line-scalars`, `named-tests-can-pass-vacuously`

## Outputs

- Edits to the five files above, each paired with a test that fails when the
  claim is overstated.
- **A `DEC-*` for `AC1`'s design question, which this spec settles rather than
  leaves open** (see below). One decision, not five.
- No new provenance row — this adds no algorithm or decoder.

## The design question this spec settles

The frame left it open: should `corpus-status` **check tool availability** (more
useful, more code) or **stop making the claim** (honest, one line)?

**Decided: check the tools.** `just test`'s pre-flight is the one surface a
reader trusts instead of reading 95 test names, and a pre-flight that says less
sends them to the test names anyway. The cost is bounded — the tool list is
already enumerated in `tests/support/tools.rs`, and `a_missing_tool_skips_loudly_naming_it`
already proves the skip path works. **Record it in a `DEC-*`, including the
rejected option**, because "the honest one-liner" is a defensible choice someone
will reopen.

## Acceptance Criteria

- [ ] **AC1 — the pre-flight states only what it checked.** With the corpus
      present and `exiftool`/`dnglab` absent, `just test`'s pre-flight must not
      say *"no tier-B test will skip."* **Measured today: it does, and all 30
      oracle tests then skip in 0.04 s.** Report corpus **and** tool status, and
      make the "nothing will skip" claim conditional on both.
      `the_preflight_does_not_promise_what_it_cannot_see` (tier A — it must run
      where CI runs, which is exactly where the tools are absent).
- [ ] **AC2 — `req`'s single-value assumption is enforced, not documented.**
      `req` truncates a multi-valued reading via `.first()`, and its doc comment
      *justifies* this as correct for today's corpus. ⚠ **The defect is not the
      truncation; it is that a documented assumption has no falsifier.** Make a
      multi-valued required tag an error (or `Unreadable`, matching `DEC-014`'s
      shape for the optional fields), and assert it.
      `a_multivalued_required_tag_does_not_truncate_to_its_head` (tier A).
      ⚠ This is the finding `SPEC-005/FU-2` → `SPEC-010/AC4` **mis-dispositioned
      twice**: the AC was written narrower than the finding, so the spec passed
      without closing it, and the build then closed it on "someone remembering at
      PROJ-002." Write the AC to the finding, not to the convenient half.
- [ ] **AC3 — `SUPPORTED_BITS` cannot outrun its tests.** Every value in
      `SUPPORTED_BITS` is exercised by at least one test, **asserted by
      enumerating the constant rather than by listing depths in a test name** —
      so that adding a fifth depth without a test **fails this criterion**.
      Measured: deleting `8` and `12` from the constant leaves **152/152 green**,
      so two of four declared depths are exercised by nothing today.
      `every_supported_bit_depth_is_exercised` (tier A).
      ⚠ Verify already drove both through the real API and they are **correct**
      (`8-bit → [1,2,3,4]`; `12-bit AB CD EF → [2748, 3567]`). This is test debt,
      not a defect — which is exactly why nothing else will ever find it.
- [ ] **AC4 — the plane fuzz target can reach `SampleExceedsWhiteLevel`.**
      `examples/fuzz-seeds.rs`'s `plane_fixture` lacks the `white_level` field
      its test-side twin has, so the one assertion `DEC-008` calls load-bearing
      is unreachable by fuzzing. `develop_fixture` (line 162) already carries it
      and is the model. Add the field, regenerate seeds, and **prove reachability
      by running the target**, not by inspecting the fixture.
      `plane_seeds_can_reach_the_white_level_assertion`.
- [ ] **AC5 — `just validate` parses the front matter it validates.** It greps
      for required keys and never parses, so it reported *"17 artifact(s) … have
      valid required front-matter"* on two files no YAML parser could read — one
      of which **shipped and was archived undetected for two days**
      (`SPEC-015/FU-4`, `FU-12`). Parse every artifact's front matter and fail
      on a parse error, naming the file and the parser's own message.
      ⚠ **Use a parser that is already present**: `ruby -ryaml` is on macOS and
      on GitHub's runners; `python3` here has **no** `pyyaml` (measured). State
      which you chose and confirm it exists in CI.
      `validate_rejects_front_matter_that_no_parser_can_read` (tier A).
- [ ] **AC6 — every fix ships with its falsifier, and each is watched red.**
      For each of AC1–AC5, apply the fault the criterion describes, and show the
      new test failing. **File changed AND compiled AND output changed** — the
      third clause has caught four false red-proofs in three specs, and one in
      `PATCH-002` two days ago where the obvious injection exercised the wrong
      path. Paste all five.
- [ ] **AC7 — ten gates + `just lint-ci`**, CI **observed** green on the
      shipping SHA. ⚠ The gate count is genuinely ambiguous in this repo
      (`the-gate-count-is-not-defined-anywhere`, `bar: 3`); **say which list you
      ran**. Do not resolve the ambiguity here — that is a repo decision.

## Failing Tests

⚠ Zero-match `cargo test <name>` exits 0; confirm each per-target, sum across all.

- `the_preflight_does_not_promise_what_it_cannot_see` — AC1, tier A
- `a_multivalued_required_tag_does_not_truncate_to_its_head` — AC2, tier A
- `every_supported_bit_depth_is_exercised` — AC3, tier A
- `plane_seeds_can_reach_the_white_level_assertion` — AC4
- `validate_rejects_front_matter_that_no_parser_can_read` — AC5, tier A

**All tier A on purpose.** Every one of these five findings is about a surface
that lies *when something is absent* — the corpus, a tool, a test, a parser —
and CI is the environment where things are absent. A tier-B test of any of them
would reproduce `ci-cannot-prove-bit-exactness`, which this spec exists to stop
compounding.

## Non-Goals

- ⚠ **The full gate-script audit.** `STAGE-005` carries it as a separate bullet
  and it should stay there. **Sized at design so it can be written:** 8 of 13
  `pipefail` scripts carry at least one unguarded `grep`, **28 of them in the
  template's own `test.sh`**. Folding that in makes this XL, which §15 says is a
  stage, not a spec.
- **Defining "the gates."** Two enumerations differing by four members are both
  cited in shipped artifacts; that is a repo decision, filed as
  `the-gate-count-is-not-defined-anywhere` (`bar: 3`, open). `AC7` works around
  it deliberately.
- **`handback-sync`'s multi-line-scalar truncation.** `AC5` makes it *detectable*
  — which is the half that generalises — but does not fix the writer. Filed as
  `handback-sync-truncates-multi-line-scalars` (`bar: 2`, open) with both
  candidate fixes.
- **Anything in `src/` beyond `SUPPORTED_BITS`'s test coverage.** No decoder
  behaviour changes here.
- Opcodes, tone curve, demosaic — `STAGE-003`.

## Implementation Context

> **Measured 2026-09-06 by the design session**, against the working tree at
> `main`. Two of the five carried findings changed on measurement; reproduce
> these numbers rather than re-deriving them.

### AC1 — the pre-flight, reproduced

Corpus 7/7 present, `exiftool` and `dnglab` removed from `PATH` (a temp dir with
only `cargo`/`rustc` symlinked, plus `/usr/bin:/bin`):

```
pre-flight says:  corpus: 7/7 present — no tier-B test will skip
tests actually:   30 passed in 0.04s      ← every tier-B test skipped
```

⚠ My first attempt at this probe measured **the wrong thing** — `env
IRRADIANCE_CORPUS_DIR=~/...` does not expand `~`, so it reported `0/7 present`
and looked like a corpus failure. The numbers above are from the corrected run.

### AC2 — `req` documents the assumption it should enforce

`tests/support/tools.rs:286-292`, verbatim: *"`req` truncates a multi-valued
reading to its head (`.first()`) — **kept**, because every required tag here is
genuinely single-valued **on this corpus**."* True today; false the moment
`SamplesPerPixel > 1`, which is PROJ-002. `BitsPerSample "8 8 8"` reads `8` and
`diff()` returns `[]`.

### AC3 — measured by mutation, not by reading

| mutation | result |
|---|---|
| `SUPPORTED_BITS: [u32; 4] = [8, 12, 14, 16]` → `[u32; 2] = [14, 16]` | **152/152 pass** |

Two of four declared depths are exercised by nothing. Restored byte-identically
afterwards.

### AC4 — the field exists, in the wrong fixture

`white_level` occurs twice in `examples/fuzz-seeds.rs`, both inside
`develop_fixture` (lines 170 and 199, added by `SPEC-014`). `plane_fixture`
(line 45) has none, so `SampleExceedsWhiteLevel` is unreachable from the plane
target.

### AC5 — the gate that succeeds mutely

`scripts/validate.sh` contains **zero** references to a YAML parser and reads
front matter with 12 line-oriented `awk`/`grep`/`sed` operations. Measured
consequence: two artifacts whose front matter raised `Psych::SyntaxError` were
reported as *"valid required front-matter"*; one shipped, was archived, and
survived three later specs. Repo-wide sweep after repair: **0 of 75** failing;
it was 2.

⚠ **`python3` in this repo has no `pyyaml`** (measured — `import yaml` fails).
`ruby -ryaml` works and is present on macOS and GitHub runners. `AC5` asks you
to confirm your choice exists in CI rather than assume it.

### The residual this spec does not close, sized

| surface | unguarded greps under `pipefail` |
|---|---|
| `test.sh` (the template's own self-test) | **28** of 152 |
| `_lib.sh`, `decisions-audit.sh`, `handback-sync.sh`, `handoffs-view.sh`, `lifetime-report.sh`, `lint-red-proof.sh`, `report_daily.sh` | 1 each |

**8 of 13** `pipefail` scripts. That is the stage's separate audit bullet; this
spec deliberately leaves it.

### Traps

- ⚠ **All five tests are tier A.** A tier-B version of any of them recreates the
  exact blindness being fixed.
- ⚠ **`AC2` has been mis-dispositioned twice.** Write the criterion to the
  finding, not to the half that is easy to satisfy.
- ⚠ **`AC3` must assert over the constant.** A test that lists `[8, 12, 14, 16]`
  in its own body passes forever after someone adds a fifth depth.
- ⚠ **`AC5`'s parser must exist where CI runs.** No `pyyaml` here.
- `just lint-ci`, not `just lint`, and **read CI**.

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
