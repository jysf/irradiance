---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-016
  type: story                      # epic | story | task | bug | chore
  cycle: frame                    # frame | design | build | verify | ship
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
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: []                    # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-005, SPEC-010]                # [SPEC-NNN]

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
  tokens_estimate: null
  sessions: []
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

1–2 sentences. Unambiguous. If you can't write the goal in two
sentences, split the spec.

## Inputs

What the implementer will read or consume.

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth requirements>
- **Related code paths:** `src/some/module/`

## Outputs

What the implementer will produce.

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New endpoints / functions / components:** <names and signatures>
- **New flags / options:** each flag's accepted values **and its default** — an
  unstated default makes the implementer guess.
- **Database changes:** <migrations, if any>

## Acceptance Criteria

Testable outcomes. Each must map to at least one test. Cover happy
path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)
- [ ] Criterion 3 (testable)

## Failing Tests

Written during the **design** cycle, BEFORE handoff. The implementer's
job in **build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...
  - `"test description 2"` — asserts: ...

## Non-Goals

Explicit scope limits. If the implementer thinks any of these need to
happen, they should create a new spec (in this stage's backlog), not
expand this one.

- ...

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities. Keep short — the full
context graph lives in the handoff file.

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
