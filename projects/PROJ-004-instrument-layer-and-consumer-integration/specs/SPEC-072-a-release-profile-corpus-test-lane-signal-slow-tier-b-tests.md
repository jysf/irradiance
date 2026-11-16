---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-072
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
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
  verify_rounds: []                # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-021
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-066]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-071]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-021's <capability>". Optional; null is acceptable.
value_link: "a ~14x faster local full test run, so STAGE-021's release and every later cycle stop paying 37 minutes per suite"

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
  tokens_estimate: 10000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-072: An optimised `corpus` test profile: `just test` runs every test in about 2.5 minutes, and `just test-debug` keeps the old run

## Context

The second spec of STAGE-021. It resolves the signal `slow-tier-b-tests-in-debug`, which the
STAGE-020 close decided as a release-profile lane. A local full corpus run takes about
37 minutes in debug, and every build and verify cycle pays for it, often more than once.

## Goal

Make `just test` run **every** test under an optimised cargo profile that keeps debug
assertions and overflow checks, and keep the old debug run as `just test-debug`. CI does
not change.

- **The hard criteria:** the same set of tests runs and passes in both lanes, and none is
  deleted, filtered, ignored or weakened.
- **The target:** a full `just test` with the corpus takes ≤ 5 min wall on the build
  machine, from about 37 min.

## Design (DEC-066)

### 1. Probe (main at `4664773`, 2026-11-16; Apple M4 Pro, corpus set)

The design defined the profile with
`--config profile.corpus.inherits="test" --config profile.corpus.opt-level=3` and ran
`cargo test --all-features --profile corpus --no-fail-fast`.

| Run | Wall | Binaries | Passed | Failed | Ignored |
|---|---|---|---|---|---|
| debug (from the PR 243 run, 553eeaf) | about 37 min | 37 | 697 | 0 | 1 |
| `corpus` | 157 s | 37 | 697 | 0 | 1 |

Per binary, debug → `corpus`:

| Binary | Debug | `corpus` |
|---|---|---|
| `orientation_tiles` | 562 s | 30.2 s |
| `develop` | 422 s | 23.4 s |
| `warp_alpha` | 402 s | 17.6 s |
| `warp` | 217 s | 11.9 s |
| `develop_options` | 150 s | 7.7 s |

- No test reads `cfg(debug_assertions)`, and no test is `#[should_panic]`.
- Every pinned digest held, and the red-proof tests ran.

### 2. The change

- **`Cargo.toml`:** a `[profile.corpus]` section, `inherits = "test"`, `opt-level = 3`,
  with a comment naming DEC-066 and saying why it inherits `test` and not `release`
  (assertions and overflow checks stay on).
- **`app.just`:**
  - `test` runs
    `cargo run --quiet --all-features --profile corpus --example corpus-status`, then
    `cargo test --all-features --profile corpus`;
  - a new `test-debug` recipe holds today's two commands unchanged;
  - both recipes' comments explain the split.
- **AGENTS.md §6:** the block's `test` lines show the new commands, a `test-debug` entry
  holds the old ones, and the prose history names SPEC-072. Grep AGENTS.md for any
  sentence quoting the debug suite's duration or saying to run it in the background, and
  update it to the measured time.
- **`.github/workflows/ci.yml`:** **comment only.** The step's "Mirrors `just test`"
  becomes "Mirrors `just test-debug`", with one line on why CI stays in debug (tier A is
  fast there, and it keeps a second optimisation level under test). No `run:` line
  changes.
- **`guidance/signals.yaml`** is not edited here. The ship marks
  `slow-tier-b-tests-in-debug` resolved.

## Inputs

- `Cargo.toml`, `app.just` (`test` and its neighbours), `.github/workflows/ci.yml` (the
  test job), AGENTS.md §6.
- DEC-066; the signal `slow-tier-b-tests-in-debug`.

## Outputs

- **Files modified:** `Cargo.toml`, `app.just`, AGENTS.md, `.github/workflows/ci.yml`
  (comment only).
- **No `src/` or `tests/` change, and no CHANGELOG entry** (no library behaviour
  changes).

## Acceptance Criteria

- [ ] **AC1. Same tests, both lanes.** With the corpus set:
  - `just test` and `just test-debug` both pass.
  - `cargo test --all-features --profile corpus -- --list` and
    `cargo test --all-features -- --list` print the same test names. Diff them, and show
    the empty diff.
  - Both runs report the same passed / failed / ignored totals.
- [ ] **AC2. Assertions stay on in `corpus`.** A throwaway check, not committed, shows
  `cfg!(debug_assertions)` is true in a test binary built with `--profile corpus`, and
  that an arithmetic overflow panics there. For example, `std::panic::catch_unwind` around
  `std::hint::black_box(u8::MAX) + 1` returns `Err`. `cfg(overflow_checks)` is not stable,
  so test the behaviour. Use a scratch test file, run once and deleted. Show the output.
- [ ] **AC3. The pins hold under `corpus`.** Every dnglab `raw_checksum` pin, every
  develop MD5 pin, and the red-proof tests pass in `just test`, with no SKIP lines (all
  12 corpus files present).
- [ ] **AC4. The time (verify checks it; CI does not).** A full `just test` with the
  corpus takes ≤ 5 min wall on the build machine, counting the profile's first build
  after a `touch src/lib.rs`. Record it beside `just test-debug`'s time from the same
  machine.
- [ ] **AC5. CI unchanged.** `git diff` on `.github/workflows/ci.yml` touches comments
  only. CI is green on the PR head with the same job set.
- [ ] **AC6. No regression.** Every gate is green:
  - `lint-ci`
  - `lint-no-allow`
  - `lint-fuzz`
  - `fmt-check`
  - `validate`
  - `doc-check`
  - `deny`
  - `msrv` (named profiles are stable well before 1.90)
  - `cost-audit`
  - `gate-audit`

## Failing Tests

None. This spec changes how tests are run, not what they assert. The evidence is AC1's
identical test list, AC2's assertion check, and AC4's time.

## Non-Goals

- Deleting, ignoring, filtering or speeding up any individual test.
- Changing CI's commands.
- `[profile.test]` or `[profile.release]` changes.
- Any `src/` or `tests/` change.

## Notes for the Implementer

- **Cut this after SPEC-071 merges.** Both edit AGENTS.md §6.
- **The `corpus` target dir is about 770 MB.** That is fine; it is gitignored under
  `target/`.
- **If you lower debug info** (for example `debug = "line-tables-only"`) to save build
  time or disk, measure it and record why. It is not required.

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
