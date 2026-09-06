---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-010
  type: story                      # epic | story | task | bug | chore
  cycle: ship                    # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: L          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: punch-list             # approved | punch-list | rejected — the OUTCOME of the verify
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
  to_agent: claude-opus-5          # ⚠ DISPATCH HINT (SPEC-007/FU-6, 1 for 4) — the cycle
                                   #   that runs corrects it to what ACTUALLY ran.
  created_at: 2026-08-22

references:
  decisions: [DEC-012, DEC-013]                    # [DEC-NNN, DEC-MMM]
  constraints: [oracle-must-be-shown-red, library-not-application, no-new-top-level-deps-without-decision]                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-005]        # [SPEC-NNN]

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
value_link: "STAGE-005: an oracle that cannot tell absence from garbage cannot certify anything"

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
  tokens_estimate: 14000000
  sessions:
    - cycle: design
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-08-22
      notes: "main-loop, not separately metered (AGENTS.md §4). Design cycle PROBED the defect rather than describing it (§15 rule 4): added two throwaway tests to tests/support/tools.rs, measured that all FOUR multi-valued tags produce a byte-identical ToolReading for an absent tag and a garbled one, and that BlackLevel [512,999] reads Some(512); restored the file byte-identical and re-ran the suite to 87. Key design finding: the information is NOT MISSING — Field.values is already Option<Vec<u32>> and its own doc comment says None is exiftool's '-'; the distinction survives values_for and is DISCARDED in reading_from_fields, in three idioms across five lines. Sized M for AC5/AC6/AC7, not for the fix. HANDOFF-024 tells build to REPRODUCE SPEC-005/FU-8's already-measured three-configuration table rather than re-derive the design."
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 24318132
      estimated_usd: 160.50
      duration_minutes: 25
      recorded_at: 2026-09-03
      notes: "Run directly in this CLI session (not a sub-agent), per HANDOFF-024 and this repo's delegate-cycles-to-cli-sessions convention. tokens_total deduped by message.id (102 unique) from this session's own transcript, summed input+output+cache_creation+cache_read (AGENTS.md §4: one combined number), captured immediately before this commit — see HANDOFF-024's handback notes for the full method and the acknowledged undercount. HANDOFF-024's to_agent hint (claude-opus-5) was corrected to claude-sonnet-5, the model every message in this session's own transcript actually reports."

    - cycle: verify
      agent: claude-opus-5
      interface: other
      tokens_total: 10362360
      estimated_usd: 21.33
      duration_minutes: 15
      recorded_at: 2026-09-03
      notes: "VERDICT PUNCH LIST - one ship-blocker (SB-1), one new follow-up (FU-3), and the build's closed disposition on FU-2 re-raised. tokens_total is DEDUPED BY message.id (79 unique, 134 messages) from this session's own transcript (~/.claude/projects/.../6670ede2-143b-4cb3-a9cd-7aa29e855fe5.jsonl), summed across input+output+cache_creation+cache_read per AGENTS.md section 4 (one combined number), captured immediately before writing this block - still an undercount, since the write itself and everything after it is not in the number. estimated_usd is PER-COMPONENT at the Opus-family published list rates for the model that actually ran (message.model = claude-opus-5, NOT tier_map): input 158 x $15 = $0.00, output 42,367 x $75 = $3.18, cache_creation 154,683 x $18.75 = $2.90, cache_read 10,165,152 x $1.50 = $15.25. .repo-context.yaml's blended rate_per_mtok 6.60 would give $68.39 for the SAME token count - 3.2x higher - because a blended rate prices 98% cache-read traffic as if it were fresh input. Raised as FU-4. duration_minutes is the transcript's own first->last delta (14.4 min), not wall-clock. handback-sync NOT run and PR NOT opened, per return criteria 8. [SPEC-015/FU-4 companion: handback-sync.sh truncated this note to its first physical line on 2026-09-03, leaving the front matter unparseable through ship, archive and three later specs. Restored 2026-09-05.]"
    - cycle: ship
      agent: claude-opus-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-09-03
      notes: "main-loop, not separately metered (AGENTS.md §4). Ship cycle: reconciled the build and verify handbacks against git and disk, reproduced the three-arm mutation table byte-for-byte before accepting SB-1, applied the reviewer's tier-A test and red-proofed it MYSELF in the no-corpus/no-tool configuration (honest 30 passed; Absent => true 1 FAILED — that configuration was green in both directions before). Also reproduced FU-3 directly: corpus-status prints '7/7 present — no tier-B test will skip' while 29 tests skip in 0.01s. ⚠ REMOVED A DUPLICATE BUILD COST SESSION: the build hand-wrote one and handback-sync appended a second (24,318,132 counted twice), which is exactly the hazard SPEC-003's build measured and warned about — synced_at null plus a hand-written entry. Kept the handback-sync entry, which names its dedup method."
  totals:
    tokens_total: 34680492
    estimated_usd: 181.83
    session_count: 4
shipped_at: 2026-09-03
---

# SPEC-010: Distinguish an unparseable tool reading from an absent tag and reconcile the frozen fixture

## Context

> **Framed 2026-08-22, not designed.** The destination for four `SPEC-005`
> follow-ups, per AGENTS.md §15. Everything below is scaffold.

**Carried findings:** `SPEC-005/FU-1`, `FU-2`, `FU-4`, `FU-9`.

**`FU-1` — the oracle cannot tell "tag absent" from "tag present but
unparseable."** Both collapse to `None` in `reading_from_fields`, so a garbled
tool reading silently *agrees* with a `None` on our side. Measured by
`SPEC-005`'s reviewer: **5/5 garbled readings diff clean**. This sits outside
`AC2`'s wording, so it is a design gap as much as a build one — the spec did not
anticipate it and neither did the architect.

⚠ **The fix is already specified AND already measured.** A tri-state on the tool
side, compared against `Sensor::malformed_tags`. `SPEC-005/FU-8` built it during
verify round 2 and confirmed all 21 oracle tests stay green — *and* that a
tri-state **without** the `malformed_tags` comparison reds. Do not re-derive
this; reproduce it and build it.

⚠ **It has a consequence in `tests/support/tools.rs`'s `diff()` doc comment**,
which currently reasons about exactly this future. `DEC-013` was `rejected`
partly on the argument that fixing `FU-1` would trip an alarm — `FU-8` measured
that it does **not**, because the `malformed_tags` comparison *is* the generic
guard on the side that holds the information. Update that doc comment when you
land this, and consider whether `DEC-013`'s rejected conclusion now deserves a
successor decision that is **true**.

**`FU-2` — `opt()`/`req()` truncate a multi-valued reading to its head.**
`black="512 999"` → `Some(512)`, diffs clean. Latent on today's monochrome
corpus; **live the moment `SamplesPerPixel > 1`**, which is PROJ-002. Same parse
layer as `FU-1`, so same pass.

**`FU-4` — the tier-A fixture is two frozen literals** carrying the three blind
spots `SPEC-005`'s own `## Context` indicts the old `Expected` table for, and
nothing reconciles them even where the corpus and both tools are present. Both
halves were verified accurate on 2026-08-22 — this is rot risk, not a present
defect. A reconcile-when-both-available test closes it.

**`FU-9` — `is_active()` (`scripts/decisions-audit.sh:152`) reads only
`superseded_by`, never `status`,** so `DEC-013` — the repo's first `rejected`
decision — still reports as governing `tests/support/tools.rs`. ⚠ **Fix the
verb, not the filter.** The reviewer's point: that surfacing is currently the
*only* mechanical signpost from the code to the explanation of why its guard is
gone. Filtering rejected decisions out would silently remove it.

## Goal

Give `ToolReading`'s optional fields a **tri-state** — absent / unreadable /
value — and have `diff()` compare the unreadable case against
`Sensor::malformed_tags`, so a garbled tool reading can no longer pass as
agreement. Fix the same layer's silent head-truncation of multi-valued readings
while it is open.

## Inputs

- **Files to read:** `tests/support/tools.rs` (the whole parse layer — `Field`,
  `values_for`, `reading_from_fields`, `diff`); `tests/metadata_oracle.rs`;
  `decisions/DEC-013-…md` (**`status: rejected`** — read *why*, it is this
  spec's prehistory); `docs/oracle-contract.md`
- **Related code paths:** `src/ifd.rs`'s `Sensor::malformed_tags` — its
  documented contract at `:553-560` is what the unreadable arm compares against

## Outputs

- **Files modified:** `tests/support/tools.rs` (the tri-state, `diff()`'s new
  arm, `opt`/`req`); `tests/metadata_oracle.rs` (new tests + the fixture
  reconcile); `tests/support/corpus.rs` only if the fixture reconcile needs it
- **New type:** a tri-state over `ToolReading`'s optional fields. Shape is the
  implementer's call, but it must preserve the **raw values** in the unreadable
  arm so the mismatch message can print what the tool actually said
- **`diff()` gains one arm**, not eleven — the comparison is per-*state*, not
  per-tag. That is the whole difference from `DEC-013`'s rejected approach
- **No new dependency.** `Cargo.toml` byte-identical

## Acceptance Criteria

- [ ] **AC1 — absent and unreadable are distinguishable.** For each of the four
      multi-valued tags, a garbled reading and an absent one produce **different**
      `ToolReading`s. Today they are byte-identical — measured, see below.
- [ ] **AC2 — an unreadable tool reading is a mismatch UNLESS our reader also
      recorded that tag in `malformed_tags`.** This is the generic guard
      `DEC-013` chose and failed to implement, now on the side that holds the
      information.
- [ ] **AC3 — `K3III.DNG` stays green.** Its malformed `BlackLevelRepeatDim` is
      `Unreadable` on the tool side and `50713` in our `malformed_tags`, so the
      two agree *for a stated reason* rather than by both collapsing to `None`.
- [ ] **AC4 — a multi-valued reading no longer truncates to its head.**
      `BlackLevel = "512 999"` must not read `Some(512)`. Measured today: it does.
- [ ] **AC5 — the tier-A fixture is reconciled against the live tool** whenever
      the corpus and `exiftool` are both present, closing `SPEC-005/FU-4`'s rot
      risk. When either is absent, skip — loudly.
- [ ] **AC6 — red-proof, both directions with a control.** Removing the
      `malformed_tags` comparison must turn `K3III.DNG` **red**; restoring it must
      turn it green. ⚠ This is the exact mutant `SPEC-005/FU-8` already ran —
      **reproduce it, do not re-derive it.**
- [ ] **AC7 — `diff()`'s doc comment and `DEC-013` are brought true.** The doc
      comment currently reasons about this future and says the alarm fires when
      `FU-1` lands; `FU-8` measured that under this fix it does **not**. Decide
      whether `DEC-013`'s rejected conclusion now deserves a **successor decision
      that is true**, and either write it or say why not.
- [ ] **AC8 — ten gates plus `just lint-ci` and `just oracle-meta`**, and **CI
      observed green on the shipping SHA** — `constraints.yaml` now requires the
      observation, not the assertion.

## Failing Tests

⚠ A zero-match `cargo test <name>` **exits 0**. Confirm each name exists via
per-target `-- --list` and **sum across targets**.

- **`tests/metadata_oracle.rs`**
  - `an_absent_tag_and_a_garbled_one_are_not_the_same_reading` — AC1
  - `a_garbled_tool_reading_is_a_mismatch_when_we_read_the_tag_fine` — AC2
  - `a_garbled_tool_reading_agrees_when_we_also_recorded_it_malformed` — AC2/AC3
  - `k3iii_dng_black_level_repeat_dim_agrees_for_a_stated_reason` — AC3, tier B
  - `a_multivalued_reading_does_not_truncate_to_its_head` — AC4
  - `the_frozen_fixture_still_matches_the_live_tool` — AC5, tier B
  - `removing_the_malformed_comparison_turns_k3iii_red` — AC6 red-proof
  - `the_malformed_comparison_control_is_green` — AC6 control

## Non-Goals

- **Any `src/` change.** This is entirely `tests/`. If you believe `src/` must
  move, that is a finding to hand back.
- **Re-opening `DEC-012`'s tolerance.** It is correct and untouched.
- **The `dnglab` side.** Its scalars are extracted with asserted-unique keys and
  are not part of this defect.
- **Adding a dependency.** The probe that shaped `SPEC-005` already showed none
  is needed. If you conclude otherwise, **stop and ask**.

## Implementation Context

> **Measured 2026-08-22 by the orchestrator**, by adding two probe tests to
> `tests/support/tools.rs`, running them, and restoring the file byte-identical.
> Reproduce before trusting.

### The collapse, measured — 4 of 4

`reading_from_fields` produces a **byte-identical** `ToolReading` for an absent
tag and a garbled one, on every multi-valued tag:

| tag | garbled input | absent == garbled? |
|---|---|---|
| `BlackLevelRepeatDim` | `[1]` | **true** |
| `ActiveArea` | `[0, 0, 5632]` | **true** |
| `DefaultCropOrigin` | `[12]` | **true** |
| `DefaultCropSize` | `[8368, 5584, 99]` | **true** |

And `BlackLevel = [512, 999]` → **`Some(512)`** — `AC4`'s defect, measured.

### ⚠ The information is not missing — it is *discarded*

`Field.values` is already `Option<Vec<u32>>`, and its doc comment already says
*"`None` is exiftool's `-` — a tag reported absent."* `values_for` preserves that.
The distinction survives all the way to `reading_from_fields` and dies there, in
three idioms:

- `opt()` / `req()` — `.and_then(|v| v.first().copied())` (drops the tail: `AC4`)
- `BlackLevelRepeatDim` — `.and_then(|v| <[u32; 2]>::try_from(..).ok())`
- `ActiveArea` / `DefaultCropOrigin` / `DefaultCropSize` — `match v.as_slice() { [..] => Some(..), _ => None }`

**So this is a five-line change plus a type.** It is `M` not `S` because of
`AC5`, `AC6` and `AC7`, not because the fix is hard.

### The fix is already built and measured — reproduce, do not re-derive

`SPEC-005/FU-8` implemented it during verify round 2 and measured three
configurations. That table is this spec's specification:

| patched into `diff()` | AC1 suite |
|---|---|
| a partial fix (one-element → `Some([a, a])`) | **red** on `K3III.DNG` |
| tri-state, `malformed_tags` **not** consulted | **red** |
| tri-state **compared against** `malformed_tags` | **21 green** |

The third row is the target state. The second is `AC6`'s red-proof: it is the
same code with one comparison removed, so the red-proof costs nothing to build.

### Why `DEC-013` is `rejected` and this is not the same thing

`DEC-013` exempted a tag **by number**, hardcoded, on the *tool* side — dead code
that suppressed a case which could not arise. This compares **states**, on the
side that knows whether the value was readable, and it is exercised by a real
corpus file on every run. Read `DEC-013`'s rejection before designing the arm;
`AC7` asks you to decide whether its conclusion now deserves a true successor.

### Traps carried in

- `just lint-ci` before every push, and **read CI** — a job that exists and has
  never passed is a deleted job (`constraints.yaml`, amended at STAGE-001's close).
- AGENTS.md §16's three rules apply directly here: the **writing rule** (this
  spec's numbers name their command and scope), **assert your match count**, and
  **a gate fails through its own `die`**.
- Sum across **all six** targets. Tier-B tests currently pass whether or not the
  corpus is present — 87 either way — so a green tells you nothing about coverage
  unless you ran `just test`, which names the missing files first.

## Follow-ups

| id | finding | disposition |
|---|---|---|
| `SB-1` | `compare_optional`'s **`Absent` arm has no test that dies** — `Absent => true` left 29 green with the full corpus, and without a corpus the arm was dead in **both** directions, so the tier-A half CI runs had never exercised it at all | `fixed` — `3d2c94e`. The reviewer's ten-line **tier-A** test, red-proofed by the orchestrator in the no-corpus/no-tool configuration: honest tree 30 passed, `Absent => true` **1 FAILED**. That configuration was green either way before |
| `FU-1` | (raised at build, resolved in the same cycle) | `fixed` — in `23e413f` |
| `FU-2` | `req()` truncates a multi-valued **required** tag: `BitsPerSample "8 8 8"` reads `8` and `diff()` returns `[]`. Latent on a mono corpus, **live at `SamplesPerPixel > 1`** | `spec: SPEC-016` — ⚠ **re-dispositioned. This is the orchestrator's error, not the build's.** `SPEC-005/FU-2` was sent to `SPEC-010` and `SPEC-010` did not close it, because `AC4` as written was narrower than the finding it carried. The build's `closed` had a trigger of *"someone remembering at PROJ-002"*, which AGENTS.md §15 names as a **bad close** — a close's trigger must be a test that fails, not a memory |
| `FU-3` | `corpus-status` prints *"corpus: 7/7 present — no tier-B test will skip"* while every tier-B test skips, when the corpus is present but `exiftool` is off `PATH` | `spec: SPEC-016` — reproduced by the orchestrator: 29 tests skip in **0.01 s** under that exact line. Worse than `SPEC-005/FU-3`, where the surface was merely silent; here it makes a **positive claim it is not entitled to make**. `corpus-status` knows about the corpus, not the tools |
| `FU-4` | the blended `rate_per_mtok` overstates this cache-heavy cycle **3.2×** — `$68.39` vs `$21.33` per-component | `signal: flat-rate-overstates-cached-sessions` — a **third** independent measurement, now spanning three different sessions and two models |

**1 ship-blocker + 4 follow-ups · 2 `fixed` · 1 `signal` · 2 → `SPEC-016`.**

⚠ **Also carried, not this spec's:** `SPEC-005/FU-9` (`is_active()` ignores `status`)
was confirmed still open and correctly **flagged rather than fixed** — it is outside
this spec's `tests/`-only scope. It stays dispositioned to `SPEC-011`.

## Reflection

**1. What would I do differently next time?**

**Write the acceptance criterion as wide as the finding it carries.** `AC4` said
*"`BlackLevel = "512 999"` must not read `Some(512)`"* — true, testable, and only
the **optional** half of `SPEC-005/FU-2`, whose stated hazard was *"live at
`SamplesPerPixel > 1`"*. The build met `AC4` exactly and closed `FU-2`, and the
finding survived both. That is my error twice over: I dispositioned `FU-2` to this
spec, then wrote a criterion that could pass without closing it.

The general form is worth more than the instance: **a follow-up routed to a spec
is not owned by that spec unless an AC would fail if it were left undone.** A
disposition of `spec: SPEC-NNN` should be checked against SPEC-NNN's criteria, not
just its title. Nothing in §15 says that yet.

**And stage before you mutate.** The build's own lesson, self-caught and
disclosed: `git checkout -- tests/support/tools.rs` during red-proof work wiped its
entire change because nothing was staged, and the shipped code is a
*reconstruction*. Nothing was lost — verify walked every AC against the code —
but it was luck that nothing was, and the handoff had to be rewritten to say
"treat every AC as unverified".

**2. Does any template, constraint, or decision need updating?**

- **`AGENTS.md` §15's four dispositions need a fifth line.** `closed` currently
  says *"a close whose trigger is a test that will fail is a good close; a close
  whose trigger is someone remembering is not"* — which the build violated in
  good faith. The missing rule is the one above: **`spec: SPEC-NNN` requires an
  AC in SPEC-NNN that fails if the finding is left undone.** Recorded as a
  candidate for STAGE-002's close rather than landed mid-stage.
- **`DEC-014` supersedes nothing and that is deliberate.** Verify confirmed the
  `DEC-013`-`rejected` / `DEC-014`-`accepted` pair sound: `decisions-audit`
  reports 0 structural errors and one *nudge* toward `superseded`, which is the
  **worse** answer — `superseded` would understate "wrong on three counts", and
  both records genuinely govern the same file for different reasons. The
  mechanical suggestion is not the right one here, and the record says why.
- **`tier_map` is now 1 for 5.** The build ran on `claude-sonnet-5` against a
  hint saying `claude-opus-5`; the verify hint was right. Both cycles corrected
  `to_agent` themselves, which is the safeguard working — but it is working
  because two sessions remembered, not because anything enforces it.

**3. Is there a follow-up spec to write now?**

**`SPEC-016`, framed**, carrying `FU-2` and `FU-3` — both instances of *the
harness asserting more than it checked*. `FU-3` is the sharper of the two:
`corpus-status` prints *"7/7 present — no tier-B test will skip"* while 29 tests
skip in 0.01 s, which is worse than `SPEC-005/FU-3` because that surface was
merely silent and this one is **wrong out loud**.

Worth stating plainly: **the defect this spec existed to fix, it reproduced.**
`DEC-013` was rejected because *"a guard that nothing dies without is a guard
nobody knows works"*, and `SPEC-010` shipped its replacement guard with the same
property in a different arm — caught only because `HANDOFF-025` asked the
reviewer to mutate each arm in turn, and they did. The comparator was correct
throughout; what was missing, both times, was the proof.


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
