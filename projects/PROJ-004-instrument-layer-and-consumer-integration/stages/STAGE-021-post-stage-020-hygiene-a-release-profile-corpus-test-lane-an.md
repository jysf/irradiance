---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-021                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-15
shipped_at: 2026-11-18

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Gets STAGE-019 and STAGE-020's unreleased develop options, plane units and decode speed-ups to the monorail/onyx consumers as a published release, after clearing the small debts those stages left behind.
  delivers:
    - "Four small test and tooling fixes carried from STAGE-020: an absolute Compression 7 allocation-count pin, the stale red-proof assert message, a just fuzz-ljpeg recipe, and AGENTS.md's timing examples"
    - "A release-profile lane for the slow corpus tests, so a local full run stops costing about 37 minutes in debug"
    - "The next release: version chosen against semver, CHANGELOG promoted, prep complete for the user to tag and publish"
  explicitly_does_not:
    - "Change any decode or develop output"
    - "Add public API beyond what is already in CHANGELOG [Unreleased]"
    - "Tag or publish: the user cuts the tag and runs cargo publish"

# Orchestration cost — the spend that has no spec to attach to (roadmap:
# orchestration + framing cost attribution). Framing a stage, deciding the spec
# breakdown, and cross-spec steering all happen BEFORE/BETWEEN specs, so today
# they are invisible and recorded cost is systematically under-counted.
#
# THE ORCHESTRATOR FILLS THIS — not the human. At stage close, read your own
# session total (`/cost` in Claude Code; the `usage` object via API) and append
# one entry. Stage grain ONLY: do not try to split orchestration across specs —
# that is a division you cannot observe, so any per-spec number is invented.
# ⚠ GATED since PATCH-002 (DEC-022 amends DEC-013 §5, which said warn-only).
# `just cost-audit` FAILS if a stage with status: shipped has no real entry
# here. "A null here is honest; a guess is not" still stands — if a stage's
# orchestration genuinely has no observable split, add it to
# STAGE_ORCH_COST_GRANDFATHERED by name rather than inventing a figure.
orchestration_cost:
  sessions:
    - tokens_total: 71367803
      estimated_usd: 72.34
      recorded_at: 2026-11-18
      notes: "Stage orchestration, measured from the orchestrator transcript (session 2b783e6b, claude-opus-5-5) from the user's 'I think the follow ups then release' message at 2026-11-15T23:50Z through this close: 147 unique message.ids, deduped by message.id with the max output per message, priced per component at Opus list rates (input 342 + cache-creation 3,535,296 (1-hour) + cache-read 67,707,002 + output 125,163); no subagents. Covers framing STAGE-021, three designs (DEC-066), five build/verify dispatches, the SPEC-072 alloc-lane follow-up commit on PR 250, the semver decision round-trip, three ships, the release smoke test and this close. Cache writes are high because the session spanned three days of cache expiry. A FLOOR: the final messages after the measurement are not in it."
                                    # - tokens_total: N
                                    #   estimated_usd: N
                                    #   recorded_at: YYYY-MM-DD
                                    #   notes: "framing + spec breakdown"
---

# STAGE-021: Post STAGE-020 hygiene, a release-profile corpus test lane, and the next release

## What This Stage Is

A short closing stage before the next release. STAGE-019 and STAGE-020 shipped a large
unreleased set (develop options, plane units, and two decode speed-ups), and STAGE-020
left five small follow-ups in the PROJ-004 brief. This stage clears four of them in one
spec, replaces the slow debug-only corpus run with a release-profile lane, and then cuts
the release that gets all of it to the requester.

## Why Now

- The monorail/onyx consumer asked for STAGE-019 and STAGE-020's features (requests 1–6,
  `feedback/2026-11-11-monorail-000-requests.md`). They are all on `main` and none is
  released. 0.2.1 (2026-11-11) predates them.
- The follow-ups are cheapest now, while their context is fresh, and two of them touch
  files the release checks anyway (AGENTS.md, the fuzz recipes).
- The slow suite cost every SPEC-069 and SPEC-070 cycle about 25–37 minutes per full
  local run. A release spec runs the suite several times.

## Success Criteria

- The four [XS] follow-ups are closed, with no change to any decode or develop output and
  every existing oracle green unedited.
- A local full run with the corpus has a fast default path and an explicit
  release-profile corpus lane, both documented, and CI still runs what it runs today.
- The release is prepared: version decided against semver from the `[Unreleased]` set,
  CHANGELOG promoted, every gate green, publish dry-run clean. The user tags and publishes.

## Scope

### In scope
- `tests/plane_units_alloc.rs`, `tests/plane_oracle.rs` (one assert message),
  `app.just` (a `fuzz-ljpeg` recipe, the corpus lane), AGENTS.md §6.
- The test-running recipes and how tier-B tests are selected.
- `Cargo.toml` version, CHANGELOG, release notes.

### Explicitly out of scope
- Any `src/` change other than what the release check forces (doc fixes).
- Restart-interval units (DEC-063 §4; on demand only).
- The signals' broader process debts.

## Spec Backlog

Ordered list of specs composing this stage. Add specs as you identify
them. Update status as specs progress.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

`just frame-stage STAGE-NNN` promotes every `(not yet written)` line below into
a real spec at `cycle: frame` with a stable ID (so `depends_on:` can point at
it) and rewrites the line in place. An optional `[S]`/`[M]`/`[L]` tag on the
line is carried onto the spec as `task.complexity`.

Order: the hygiene spec and the test lane are independent and can run in either order.
The release depends on both, so it is cut from a tree that has them.

- [x] SPEC-071 (shipped on 2026-11-16) [S] — STAGE-020 follow-ups, bundled: (1) pin the absolute Compression 7 allocation count per strip or tile in `tests/plane_units_alloc.rs` (about 10 from the `ljpeg` header parses, measured not assumed), so a new allocation in the shared kernel is caught by the count, not only by old-equals-new; (2) reword `inject_chunk_extraction_fault`'s stale assert message in `tests/plane_oracle.rs` to name the 14-bit group kernel (message only, no assertion change); (3) a `just fuzz-ljpeg` recipe that creates `fuzz/corpus/ljpeg` first, mirroring `fuzz-plane`; (4) AGENTS.md §6 lists the three timing examples (`orient-timing`, `unpack-timing`, `ljpeg-timing`: release, corpus-gated, no CI gate). No output change; every oracle green unedited. **Shipped (PR 247 as d49afd9):** the pin is 10 allocations per strip or tile, two header parses of five each; red-proved 44 vs 40 and 132 vs 120; `just fuzz-ljpeg` creates its corpus dir; AGENTS.md section 6 lists all 35 recipes (three more than planned were missing). 11.21M tokens / $8.65.
- [x] SPEC-072 (shipped on 2026-11-16) [S] — A release-profile corpus test lane (signal `slow-tier-b-tests-in-debug`): the slowest tier-B binaries in debug are `orientation_tiles` 562 s, `develop` 422 s, `warp_alpha` 402 s, `warp` 217 s, `develop_options` 150 s and `plane_units` 142 s, and a full corpus run takes about 37 min. Design decides the shape (for example a `just test-corpus` recipe running the tier-B binaries with `--release`, and whether `just test` keeps tier A plus a fast tier-B subset). Hard rules: no test is deleted or weakened, every tier-B test still runs in some documented lane, CI (no corpus) is unchanged, and AGENTS.md §6 says which lane to run when. **Shipped (PR 250 as 431b3b1, superseding PR 249):** `[profile.corpus]` (inherits test, opt-level 3), placed above `[dependencies]` for the licence red-proof; `just test` runs every test under it plus the three `*_alloc` binaries again in debug, because opt-level 3 can delete an unused allocation (verify punch list); `just test-debug` keeps the old run. `just test` 178 s against about 37 min, 710 passed. 11.14M tokens / $8.62.

- [x] SPEC-073 (shipped on 2026-11-18) [S] — The next release (task.type release): decide the version against semver from CHANGELOG `[Unreleased]` (0.2.2 or 0.3.0, DEC-052 precedent), promote the CHANGELOG, run every gate and a publish dry-run, and stop prep-complete; the user tags and publishes. Depends on SPEC-071 and SPEC-072. **Designed 2026-11-16:** 0.2.2 (the user chose it; the design audit found everything since v0.2.1 backward-compatible), cargo-semver-checks at build (user-approved), HANDOFF-153.


**Designed 2026-11-16 (one design PR for both):** SPEC-071 measured 10 allocations per Compression 7 strip or tile (HANDOFF-149); SPEC-072 is DEC-066, an optimised `corpus` profile, full corpus suite 157 s against about 37 min in debug with the same 697/0/1 (HANDOFF-150). Build SPEC-071 first; both edit AGENTS.md section 6.
**Count:** 3 shipped / 0 active / 0 pending (SPEC-071 11.21M tok / $8.65; SPEC-072 11.14M / $8.62; SPEC-073 20.10M / $15.84; 0.2.2 published 2026-11-17)

## Design Notes

**Framing probe (main at 553eeaf, 2026-11-15; Apple M4 Pro, debug, `cargo test
--all-features --no-fail-fast` with the corpus, from the #243 verification run):** 37
binaries, 697 passed, about 37 min wall (14:11 to 14:48). Per-binary times, slowest first:
`orientation_tiles` 562 s, `develop` 422 s, `warp_alpha` 402 s, `warp` 217 s,
`develop_options` 150 s, `plane_units` 142 s, `develop_scratch` 110 s, `black_levels` 75 s.
The sum of binary times is 2,238 s; the rest is compilation. SPEC-068's any-order test,
about 9.5 min at its ship, now sits inside `plane_units`' 142 s, because SPEC-069 and
SPEC-070 made the decode it repeats much faster.

**The release.**
- **The version is a design question, not decided here.** The current version is 0.2.1.
  `[Unreleased]` holds new public items (develop options, plane units, previews, tiled
  planes, new `Error` variants on a `#[non_exhaustive]` enum, new `Sensor` fields) and
  behaviour changes (`develop_into` allocates at most once; two `opcode::parse_*`
  functions changed). Design checks each against semver (DEC-052 sealed the parser
  output types for exactly this) and picks 0.2.2 or 0.3.0. `cargo-semver-checks` is not
  installed; design decides whether to use it.
- **The precedent** is SPEC-042 / DEC-052 (0.2.0, "prep-complete") and SPEC-056 (0.2.1).
  The release spec prepares everything, and **the user tags and runs `cargo publish`**.

## Dependencies

### Depends on
- STAGE-019 and STAGE-020 (shipped): the unreleased set this stage releases.
- PR 243 (merged): the `plane_units_alloc` race fix, which the allocation-count pin builds on.

### Enables
- The monorail/onyx consumer moving from a path dependency to a published version.
- The next PROJ-004 stage starting from a released baseline.

## Stage-Level Reflection

*Filled at stage close, 2026-11-18.*

- **Did we deliver the outcome in "What This Stage Is"?** Yes.
  - **SPEC-071:** the four STAGE-020 follow-ups closed (an absolute Compression 7
    allocation pin of 10 per strip or tile, the injector message, `just fuzz-ljpeg`, and
    every recipe in AGENTS.md §6).
  - **SPEC-072 (DEC-066):** `just test` with the corpus takes 178 s instead of about
    37 min, every test run and none weakened (the allocation tests run again in debug).
  - **SPEC-073:** **irradiance 0.2.2 is on crates.io** (published 2026-11-17 by the
    user), a compatible release carrying STAGE-018 to STAGE-020; tag `v0.2.2` at
    `46b6fc2` matches the crate's vcs sha, docs.rs is live, and a registry smoke test
    passed on three held files.
  - **Every success criterion holds:** no decode or develop output changed, CI commands
    are unchanged, and the release ended prep-complete with the user cutting it.
- **How many specs did it actually take?** Three, as planned, all S.
  - Total spec cost: 42.45M tokens / $33.11.
  - Orchestration: 71.37M / $72.34 (floor).
- **How many outlines survived unchanged?** 2 of 3. SPEC-071 and SPEC-073 held as
  framed (SPEC-073's version was left open at frame and decided at design, as planned).
  SPEC-072 changed at verify: the default lane gained a debug step for the allocation
  tests.
- **What changed between starting and shipping?** The test-lane spec turned out to have
  a subtle cost the probe could not see (optimisation blinds allocation counters), and
  the release's mechanical semver check raised one false positive that needed the
  user's call. Both were caught before anything shipped.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **None codified.** No lesson reached its bar.
  - **Process:** the release was cut before its cold verify. A post-release verify
    (HANDOFF-154) approved what was published, and the registry smoke test at ship agreed.
    Recorded, not codified: for a release, verify before the cut.
- **Signals dispositioned at this close?** Every `disposition_at: stage-close` signal in
  `watch` was walked.
  - **Stay `watch`, no new instance:** `a-fix-inherits-the-precondition-of-the-thing-it-fixes`
    (N=2), `probe-the-hard-assumption-at-frame-time`,
    `patent-check-named-algorithms-at-frame`,
    `saturating-as-cast-masks-explicit-clamp-in-quantize-tests`,
    `read-the-kernel-before-deciding-the-approach`,
    `hostile-fixture-shadowed-by-an-earlier-guard`,
    `closed-form-numeric-needs-extreme-magnitude-fixtures`,
    `analytic-oracle-premise-must-survive-rounding`,
    `parallel-tests-share-a-global-counter`,
    `spec-must-carry-what-the-probe-number-depended-on` (each N=1), and the process-debt
    `confirm-pr-head-advanced-before-treating-pushed-commit-as-merged`.
  - **Stay `watch`, residual risk unchanged:** `unexercised-sof3-paths-garble-silently`.
  - **Resolved this stage:** `slow-tier-b-tests-in-debug` (done at the SPEC-072 ship).
  - **New, `watch`:**
    - `a-faster-test-lane-can-blind-tests-that-measure-the-compiler` (lesson, N=1);
    - `generated-artifact-filenames-near-the-path-limit` (process-debt, N=1; the
      semver-checks "File name too long" crash);
    - `push-and-pull-request-both-trigger-ci` (process-debt, N=2; the cancelled macOS
      determinism jobs).
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - **Run mechanical release checks at design** (SPEC-073): `cargo semver-checks` would
    have surfaced the discriminant false positive before the build stopped on it. One
    instance; it stays in the spec reflection.
