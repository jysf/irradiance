---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-021                     # stable, zero-padded, continuous across the repo
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-004                      # parent project
repo:
  id: irradiance

created_at: 2026-11-15
shipped_at: null

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
  sessions: []                      # - tokens_total: N
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

- [ ] SPEC-071 (frame) [S] — STAGE-020 follow-ups, bundled: (1) pin the absolute Compression 7 allocation count per strip or tile in `tests/plane_units_alloc.rs` (about 10 from the `ljpeg` header parses, measured not assumed), so a new allocation in the shared kernel is caught by the count, not only by old-equals-new; (2) reword `inject_chunk_extraction_fault`'s stale assert message in `tests/plane_oracle.rs` to name the 14-bit group kernel (message only, no assertion change); (3) a `just fuzz-ljpeg` recipe that creates `fuzz/corpus/ljpeg` first, mirroring `fuzz-plane`; (4) AGENTS.md §6 lists the three timing examples (`orient-timing`, `unpack-timing`, `ljpeg-timing`: release, corpus-gated, no CI gate). No output change; every oracle green unedited.
- [ ] SPEC-072 (frame) [S] — A release-profile corpus test lane (signal `slow-tier-b-tests-in-debug`): the slowest tier-B binaries in debug are `orientation_tiles` 562 s, `develop` 422 s, `warp_alpha` 402 s, `warp` 217 s, `develop_options` 150 s and `plane_units` 142 s, and a full corpus run takes about 37 min. Design decides the shape (for example a `just test-corpus` recipe running the tier-B binaries with `--release`, and whether `just test` keeps tier A plus a fast tier-B subset). Hard rules: no test is deleted or weakened, every tier-B test still runs in some documented lane, CI (no corpus) is unchanged, and AGENTS.md §6 says which lane to run when.

- [ ] SPEC-073 (frame) [M] — The next release (task.type release): decide the version against semver from CHANGELOG `[Unreleased]` (0.2.2 or 0.3.0, DEC-052 precedent), promote the CHANGELOG, run every gate and a publish dry-run, and stop prep-complete; the user tags and publishes. Depends on SPEC-071 and SPEC-072.

**Count:** 0 shipped / 3 active / 0 pending

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

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **How many outlines survived unchanged?** <n of m — only if this stage was
  framed with `just frame-stage`. Counts outlines whose scope held from frame to
  ship. A low number means framing ran ahead of what was knowable; a high one
  means it's paying for itself.>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Signals dispositioned at this close?** (Prompt 1d step 7) Every
  `type: lesson` signal in `/guidance/signals.yaml` owned by this stage close
  was walked — codified (at its bar), left `watch`, or dropped. No silent carry.
  - <note what codified / what's still watch + its N>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items — record below-bar ones as `watch` signals; don't codify yet>
