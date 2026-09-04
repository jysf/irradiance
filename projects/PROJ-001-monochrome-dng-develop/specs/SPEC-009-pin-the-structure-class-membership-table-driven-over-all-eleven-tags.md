---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-009
  type: story                      # epic | story | task | bug | chore
  cycle: verify                     # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
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
  stage: STAGE-002
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-sonnet-5        # CORRECTED — build ran on claude-sonnet-5, not the opus hint.
  created_at: 2026-09-03

references:
  decisions: [DEC-012, DEC-014, DEC-015]  # DEC-014 drives AC4's stakes; DEC-015 is this build's own
  constraints: [oracle-must-be-shown-red, test-before-implementation, library-not-application]                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-007, SPEC-008]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-002's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling STAGE-002's unpack — require_uncompressed() and the StripByteCounts assertion both read tags this guard protects"

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
  tokens_estimate: 16000000
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-03
      notes: "main-loop, not separately metered (AGENTS.md §4). Design cycle RE-MEASURED all four carried SPEC-008 findings on main at 024eaae rather than inheriting them — each mutation asserted applied by diff, tree restored byte-identical: FU-1 (ten of eleven memberships deleted) 96 passed 0 failed; FU-2 (combined malformed.push split per erroring read) compiles, 96 passed 0 failed; FU-5 the test still contains ZERO sensor_candidates assertions. They were raised against a 66-test suite and the suite is now 96 — thirty tests added and not one touches these paths, which is the argument for doing this now rather than trusting accumulation. KEY DESIGN INPUT the findings could not have known: DEC-014 changed AC4's stakes. malformed_tags is no longer just a report — diff() now treats a tag named in it as EXEMPT from comparison with the tool, so Option A (record what was ignored) would widen the oracle's blind spot. Recommended B and narrowing the contract text instead, offered as input rather than as the answer; the DEC is build's to write either way. HANDOFF-026 ready."
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-03
      notes: "Dispatched as a direct CLI session, not an Agent-tool sub-agent, so no subagent_tokens is available; this interface exposes no /cost-equivalent programmatic call either. See HANDOFF-026 handback notes — orchestrator should read /cost from this session and fill this in rather than leave it null-with-note, since build is a metered cycle (cost-captured-per-cycle)."

  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-009: Pin the Structure class membership table-driven over all eleven tags

## Context

> **Framed 2026-08-21, not yet designed.** This spec exists as the destination for
> four `SPEC-008` follow-ups, per AGENTS.md §15's *Where an unresolved follow-up
> goes* — `frame` is the bar a disposition has to clear, not `ready`. Everything
> below `## Context` is still scaffold and the design cycle owns it.

**Carried findings:** `SPEC-008/FU-1`, `SPEC-008/FU-2`, `SPEC-008/FU-3`,
`SPEC-008/FU-5`.

**`SPEC-008/FU-1` — the membership list is pinned at one point out of eleven.**
`is_structural_tag()` (`src/ifd.rs:188-203`) gates `uints()`'s `TYPE_RATIONAL`
acceptance per tag (`src/ifd.rs:841`). It names eleven tags. Exactly **one** —
`TAG_SUB_IFDS` — is enforced by any test.

⚠ **Measured by the orchestrator, 2026-08-21, not inherited from the handback.**
`is_structural_tag()` reduced to `matches!(tag, TAG_SUB_IFDS)`; the mutation was
asserted applied by `diff` before anything was concluded from it; the suite was
**summed across all five targets** (`45 + 0 + 9 + 12 + 0`) with
`IRRADIANCE_CORPUS_DIR` set and the corpus present; the tree was restored and
`git status` confirmed clean.

| | tests |
|---|---|
| baseline | **66 passed** |
| ten of eleven memberships deleted | **66 passed** — nothing goes red |

The four structural fixtures `SPEC-008` added cannot catch it: they plant field
type `250`, which the **general** type gate rejects two lines below the per-tag
gate they never reach.

**The hazard is this stage's.** `Compression` encoded `RATIONAL 2/2` reads `1` →
`require_uncompressed()` (`src/ifd.rs:556`) passes → the unpack reads JPEG bytes
as raw samples. A wrong image from a file that parsed cleanly, which is this
project's signature failure shape. `StripByteCounts` as `RATIONAL 28/2` silently
reading `[14]` is the same defect against the plane's extent.

**Why this recursion terminates**, which is the reason it is worth doing at all.
`SPEC-007` fixed the behaviour, `SPEC-008` pinned the tags, and this pins the
membership — three turns of one screw, which is the `SPEC-001` gate-loop shape.
It stops here because of the **shape of the fix**: one table-driven test over all
eleven memberships has no "one point" left to be narrow at. ⚠ Design must not let
that test derive its table from `is_structural_tag()` itself — a test that reads
the list it is checking is a tautology, and deleting a tag would delete its own
coverage. The table is written out independently, and asserts **both** directions:
each of the eleven rejects a `RATIONAL` entry, and a paired interpretation tag
still reads one.

Scope note: the fix closes the *softening* direction. Adding a twelfth tag to
`is_structural_tag()` without a test row would go uncaught — that is the strict
direction and not the hazard.

**`SPEC-008/FU-2`** — "costed at most once" is unguarded on the only path where it
can fail. Splitting the combined `malformed.push` (`src/ifd.rs:1161-1178`) into one
push per erroring read reproduces `SPEC-007/FU-1`'s exact `[274, 274]` defect and
leaves 66/66 green, because
`orientation_costed_once_when_plane_is_ifd0`'s `sensor_read` is `None` by
construction. Needs a fixture with a malformed `Orientation` on **both** `IFD0` and
the SubIFD plane.

**`SPEC-008/FU-3`** — needs a **decision written down before code**. A well-formed
`IFD0` `Orientation` with a malformed sensor-IFD entry gives the right value and an
**empty** `malformed_tags`, while `Sensor::malformed_tags` is documented
(`src/ifd.rs:553-560`) as recording tags "present but shaped wrong". Both answers
are defensible; nothing says which wins, and it is untested either way.

**`SPEC-008/FU-5`** — `wellformed_orientation_is_not_recorded_malformed`
(`src/ifd.rs:2126-2152`) never asserts the precondition it depends on, unlike both
its neighbours. Verified still absent 2026-08-21. One line:
`assert_eq!(c.sensor_candidates(), vec![1]);`

**Why STAGE-002 and not STAGE-001.** The hazard bites at the unpack, so this is the
right stage; and `STAGE-001`'s close is the forcing function for three `lesson`
signals already at or past their bar (`measurement-over-generalised` at N=3 —
the very lesson that produced `FU-1`). Inserting this ahead of `SPEC-005` would
delay that close for a hazard that has no consumer today.

## Goal

Make every one of `is_structural_tag()`'s eleven memberships load-bearing with a
**table-driven test that carries its own list**, and close the three smaller
`SPEC-008` findings that share its shape: a fix whose guard is one point wide.

## Inputs

- **Files to read:** `src/ifd.rs` — `is_structural_tag()` (`:188-203`), the
  per-tag `TYPE_RATIONAL` gate in `uints()` (`:841`), `sensor()`'s `Orientation`
  fallback (`:1155-1173`), `Sensor::malformed_tags`' documented contract
  (`:553-560`); `tests/support/tiff.rs` (the hand-built fixture builder)
- **Decisions:** `DEC-012` (the Structure/Interpretation split and its amended
  table), `DEC-014` (what `malformed_tags` now *means* to the oracle — read this
  before deciding `AC4`)
- **Related:** `SPEC-008`'s `## Follow-ups` table — this spec is its `spec:` row

## Outputs

- **Files modified:** `src/ifd.rs` — new `#[cfg(test)]` tests, and **at most one
  behaviour change** (`AC4`, only if the decision goes that way);
  `tests/support/tiff.rs` if a fixture shape is missing
- **New decision:** a `DEC-*` recording `AC4`'s answer, whichever way it goes.
  ⚠ It is a decision *even if the answer is "keep current behaviour"* — the
  contract and the code currently disagree and nobody has said which wins
- **No new dependency**, no new public API. `Cargo.toml` byte-identical

## Acceptance Criteria

- [x] **AC1 — the membership list is pinned, table-driven, over all eleven
      tags.** Deleting **any single** membership must turn the suite red.
      ⚠ **The test carries its own list of eleven, written out.** It must **not**
      iterate `is_structural_tag()` — a test that reads the list it checks is a
      tautology, and deleting a tag would delete its own coverage.
      — `every_structural_tag_rejects_a_rational` (`src/ifd.rs`), an 11-entry
      `const` array written out independently of `is_structural_tag()`.
- [x] **AC2 — both directions per tag.** Each of the eleven **rejects** a
      `RATIONAL` entry with `Error::UnexpectedFieldType`; a paired
      **interpretation** tag still **accepts** one (`SPEC-007`'s widening must
      survive). A test that only proves rejection would pass if `uints()`
      rejected `RATIONAL` universally, which would silently undo `SPEC-007`.
      — rejection: same test as `AC1`. Acceptance:
      `an_interpretation_tag_still_accepts_a_rational` (`TAG_BLACK_LEVEL`).
- [x] **AC3 — "costed at most once" is guarded on the path where it can fail.**
      A fixture with a malformed `Orientation` on **both** `IFD0` and the SubIFD
      plane, asserting `malformed_tags == [TAG_ORIENTATION]` — **one** element.
      Measured today: splitting the combined push into one per erroring read
      compiles and leaves all 96 tests green.
      — `orientation_malformed_on_both_ifds_is_costed_once` (`src/ifd.rs`).
- [x] **AC4 — the swallowed malformed sensor read is DECIDED and pinned.**
      Today a well-formed `IFD0` `Orientation` with an **erroring** sensor-IFD
      read yields `Some(v)` and an **empty** `malformed_tags`. Choose, write a
      `DEC-*`, and pin the chosen behaviour with a test. See the analysis below —
      it is not a free choice any more.
      — Decided **B** (silence), `DEC-015`. Zero behaviour change; doc comment
      on `Sensor::malformed_tags` narrowed. Pinned by
      `a_malformed_sensor_orientation_with_a_good_ifd0_value_is_silently_dropped`.
- [x] **AC5 — `wellformed_orientation_is_not_recorded_malformed` asserts its own
      precondition.** Measured: it contains **zero** `sensor_candidates`
      assertions and holds only because `IFD0` carries `NewSubfileType = 1`.
      One line.
      — added `assert_eq!(c.sensor_candidates(), vec![1])`.
- [x] **AC6 — red-proof with a control**, per `oracle-must-be-shown-red` as
      widened to gates. For `AC1` that is the eleven-way mutation itself: each
      membership deleted in turn must fail, and the unmutated tree must pass.
      **Watch it, do not reason about it.**
      — all eleven mutations applied (asserted by `git diff`), compiled, and
      turned `every_structural_tag_rejects_a_rational` red; the restored,
      unmutated tree passed the full 100-test suite as the control.
- [x] **AC7 — eleven gates plus `just lint-ci`**, and **CI observed green on the
      shipping SHA**.
      — all run locally (§ Return Criteria below); CI green on `3b50964`
      (`gh run view 33842214431`).

## Failing Tests

⚠ A zero-match `cargo test <name>` **exits 0**; confirm each exists per-target
and **sum across all six targets**.

- **`src/ifd.rs` `#[cfg(test)]`**
  - `every_structural_tag_rejects_a_rational` — AC1/AC2
  - `an_interpretation_tag_still_accepts_a_rational` — AC2's other direction
  - `orientation_malformed_on_both_ifds_is_costed_once` — AC3
  - `a_malformed_sensor_orientation_with_a_good_ifd0_value_is_silently_dropped`
    — AC4, `DEC-015` decided B
  - AC5: **not a new test** — `AC5`'s own text names the existing
    `wellformed_orientation_is_not_recorded_malformed` and asks for one line
    added to it, which is what happened; this section's suggested name
    (`wellformed_orientation_test_pins_its_own_precondition`) disagreed with
    `AC5`'s own text and was not followed.

## Non-Goals

- **Re-opening `DEC-012`'s classification.** Which tags are structural is
  settled; this spec pins that the *code* enforces what the *table* says.
- **The unpack.** `SPEC-012` owns pixels. This spec exists so that spec's inputs
  cannot lie to it.
- **Widening `is_structural_tag()`.** Adding a twelfth tag is the *strict*
  direction and is not the hazard.

## Implementation Context

> **Measured 2026-09-03 on `main` at `024eaae`**, each mutation asserted applied
> by `diff` and the tree restored byte-identical. All four findings were raised
> against a 66-test suite; the suite is now **96** and every one still holds.

| finding | mutation | result |
|---|---|---|
| `FU-1` | `is_structural_tag()` reduced to `TAG_SUB_IFDS` alone — ten deleted | **96 passed, 0 failed** |
| `FU-2` | the combined `malformed.push` split into one per erroring read | compiles, **96 passed, 0 failed** |
| `FU-5` | — | the test contains **0** `sensor_candidates` assertions |

**Thirty tests have been added since these were raised and not one of them
touches these paths.** That is the argument for doing this now rather than
trusting accumulation.

### The hazard, and why it is STAGE-002's

`Compression` encoded `RATIONAL 2/2` reads `1` → `require_uncompressed()` passes
→ **the unpack reads JPEG bytes as raw samples.** A wrong image from a file that
parsed cleanly, which is this project's signature failure shape. `StripByteCounts`
as `RATIONAL 28/2` silently reading `[14]` is the same defect against the plane's
extent.

### ⚠ AC4 is no longer a free choice — `DEC-014` changed the stakes

When `SPEC-008/FU-3` raised this, `malformed_tags` was a *report*. Since
`SPEC-010` and `DEC-014` it is also an **input to the oracle**: `diff()` treats a
tag named in `malformed_tags` as *exempt* from comparison with the tool. So the
two options are no longer symmetric:

- **Option A — record what was ignored.** Push `TAG_ORIENTATION` whenever any
  read errored, even when a value was found. Matches the field's documented
  contract at `src/ifd.rs:553-560` — *"present but shaped wrong, recorded rather
  than rejected"* — the tag **is** present and **is** shaped wrong.
  ⚠ **But it widens the oracle's blind spot**: every tag it newly records becomes
  one the metadata oracle stops checking.
- **Option B — a value found means silence.** Current behaviour.
  `malformed_tags` means *"this field's value was lost"*, which is what makes the
  `DEC-014` exemption safe — you only stop comparing a field whose value you do
  not have.

**The orchestrator's read, offered as input and not as the answer:** B, and the
contract text at `:553-560` should be narrowed to say *"present, shaped wrong,
**and therefore dropped**"*. A is more faithful to the words as written and less
faithful to what the field is now used for, and `DEC-014`'s exemption is only
sound under B. **Build decides and writes the `DEC`** — including if it disagrees.

### Traps

- ⚠ **Do not derive the test's table from `is_structural_tag()`.** `AGENTS.md`
  §16 rule 1: a claim must be backed by a second measured point in a different
  direction, and a self-referential table has none.
- `just lint-ci`, not `just lint` — and **read CI**.
- Sum across **all six** targets. Tier-B tests pass whether or not the corpus is
  present; only `just test` names what is missing.

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
