---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-066
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-004
repo:
  id: irradiance

created_at: 2026-11-16
supersedes: null
superseded_by: null
status: accepted
deciders: [jysf, claude]

affected_scope:
  - Cargo.toml
  - app.just
  - AGENTS.md
  - .github/workflows/ci.yml
  - projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-072-a-release-profile-corpus-test-lane-signal-slow-tier-b-tests.md

tags:
  - testing
  - tier-b
  - cargo-profile
  - process
  - stage-021
---

# DEC-066: `just test` runs the whole suite under an optimised `corpus` profile, and CI stays on debug

## Decision

### 1. A named cargo profile, `corpus`

`Cargo.toml` gains:

```toml
[profile.corpus]
inherits = "test"
opt-level = 3
```

- **It inherits `test`**, and `test` inherits `dev`. So **debug assertions and overflow
  checks stay on**, unlike `--release`. Every `checked_*` contract, `debug_assert!` and
  arithmetic-overflow panic the tests can reach still fires.
- **Only the optimisation level changes.** No test reads `cfg(debug_assertions)`, and
  no test is `#[should_panic]` (checked at design). The output of every decode and
  develop is defined bit-exactly by integer and IEEE arithmetic that Rust does not
  reorder, so optimisation cannot change it. The pinned MD5s confirm this (§Context).
- **Debug info** stays as `dev` sets it, unless the build measures a reason to lower
  it. If it does, it records the reason.

### 2. Two local lanes, one command each

- **`just test`** runs `cargo test --all-features --profile corpus`, after the
  `corpus-status` example under the same profile, so the library compiles once. Every
  test runs, tier A and tier B alike, and **no test is deleted, filtered or weakened**.
  This is the lane every build and verify cycle runs.
- **`just test-debug`** keeps today's exact commands, `cargo test --all-features` in the
  `dev` / `test` profile. It is for the rare case where an unoptimised build matters,
  such as stepping through a test in a debugger.

### 3. CI is unchanged

CI runs `cargo test --all-features` itself, with no corpus. Tier A takes seconds in debug,
and a debug build in CI keeps a second optimisation level under test (the
`determinism` job already adds `--release`). The CI step's comment that says it
"mirrors `just test`" is updated to say it mirrors `just test-debug`, on purpose. **No
CI command changes.**

### 4. Documentation

- The AGENTS.md §6 command block shows both lanes, and its prose history records
  SPEC-072.
- Any AGENTS.md guidance that quotes the full suite's debug duration is updated to the
  measured `corpus` time.

## Context

The signal `slow-tier-b-tests-in-debug` (process-debt, N=2) was decided at the STAGE-020
close as a release-profile lane. Every build and verify in STAGE-019 and STAGE-020 ran
the full corpus suite in debug, and each run took 25–37 minutes.

The SPEC-071/072 design probe ran on main at `4664773` (Apple M4 Pro, corpus set, every
held file present). It defined the profile on the command line with `--config` and ran
`cargo test --all-features --profile corpus --no-fail-fast`:

| Run | Wall | Binaries | Passed | Failed | Ignored |
|---|---|---|---|---|---|
| debug (`just test` today; from the PR 243 verification, 553eeaf) | about 37 min | 37 | 697 | 0 | 1 |
| **`corpus` profile** | **157 s** (build 8.8 s incremental) | 37 | 697 | 0 | 1 |

| Slowest binaries | debug | `corpus` |
|---|---|---|
| `orientation_tiles` | 562 s | 30.2 s |
| `develop` | 422 s | 23.4 s |
| `warp_alpha` | 402 s | 17.6 s |
| `warp` | 217 s | 11.9 s |
| `develop_options` | 150 s | 7.7 s |

- **The same tests ran and passed:** 697 passed, 0 failed, 1 ignored.
- **The pinned outputs are unchanged.** That includes every dnglab `raw_checksum` and
  every develop MD5. The red-proof tests, which build mutated probe crates of their own,
  ran too.
- **About 14× faster overall,** and close to 19× on the slowest binary.

## Alternatives Considered

- **`[profile.test] opt-level = 3`.** Rejected. It also optimises CI's and every
  other `cargo test`, which slows CI compiles for no gain (tier A is fast in debug). It
  also removes the plain debug lane entirely.
- **`cargo test --release` for tier B.** Rejected. It turns off debug assertions and
  overflow checks, which some hostile-input tests depend on to prove a path panics in
  no build.
- **A separate `just test-corpus` recipe, with `just test` left in debug.** Rejected.
  The slow lane would stay the default that every cycle runs, and the fast one would be
  opt-in. The profile is safe enough to be the default.
- **Mark the slow tier-B tests `#[ignore]` and run them in a separate lane.** Rejected.
  It weakens the default run, and the profile makes it unnecessary.

## Consequences

- A local full corpus run takes about 2.5 minutes instead of 37. The measured time is
  recorded at ship.
- The `corpus` profile has its own `target/corpus` directory (about 770 MB here), beside
  `target/debug`.
- **The signal `slow-tier-b-tests-in-debug` is resolved** when SPEC-072 ships.
- Handoff boilerplate that says "the full suite takes about 25 min; run it in the
  background" is out of date from then on. Future handoffs quote the new time.
- **Amendment (SPEC-072 verify, HANDOFF-152, 2026-11-16).** "Only the optimisation
  level changes" and "no test is weakened" hold for every assertion and pin, but not
  for the allocation-counting tests. At opt-level 3, LLVM may delete a heap allocation
  whose result is never used. The verify injected `drop(vec![0u16; 64])` into
  `develop_with_scratch_into`. The three zero-allocation tests in
  `develop_scratch_alloc` went red under `test` and stayed green under `corpus`. A
  transient allocation that is actually used, such as a `collect` that is read
  afterwards, went red under both. So the `*_alloc` tests are authoritative only in
  the debug lane. **Resolved at the orchestrator's review of PR #250:** `just test`
  gains a third command that re-runs the three `*_alloc` test binaries
  (`develop_scratch_alloc`, `plane_units_alloc`, `previews_alloc`) in the debug
  profile, so the default lane weakens no test. That includes the corpus-gated
  `scratch_develop_allocates_nothing_on_q2m`, which CI never runs. CI runs them in
  debug as before, and a new `*_alloc` file must be added to that command.
- **`[profile.corpus]` sits above `[dependencies]` in `Cargo.toml`, not at the end.**
  `scripts/deny-red-proof.sh` appends a dev-dependency and requires
  `[dev-dependencies]` to be the last table. The build put the profile last, which
  turned that CI job red on PR #249.
