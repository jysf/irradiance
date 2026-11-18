---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-010                     # stable, zero-padded, continuous across the repo
  status: shipped                   # proposed | active | shipped | cancelled | on_hold — Phase 1 (prep) done; crates.io cut (Phase 2) is the human step
  priority: high                    # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-002                      # parent project
repo:
  id: irradiance

created_at: 2026-11-03
shipped_at: 2026-11-11

# What part of the project's value thesis this stage advances.
# If you can't articulate value_contribution, the stage may be
# infrastructure-only — acceptable but flag it.
value_contribution:
  advances: >
    Makes the permissive pure-Rust RAW crate safe to depend on from a language runtime that cannot catch an OOM abort, and publishes the demosaic work.
  delivers:
    - "Allocation-free plane::validate with a proven size bound"
    - "Stable Error::kind() classification"
    - "Cross-OS/arch determinism evidence"
    - "irradiance 0.2.0 on crates.io"
  explicitly_does_not:
    - "Any NON-additive public API change (DEC-045 adds a Sensor field + makes the public structs non-exhaustive, additive only)"
    - "ML packing, normalisation or NumPy buffers (downstream)"
    - "Colour-accuracy work (STAGE-008 / ColorChecker)"

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
    - tokens_total: 204730076
      estimated_usd: 406.84
      recorded_at: 2026-11-11
      notes: "STAGE-010 orchestration floor, measured from this orchestrator session's transcript (scratchpad uuid fd689c1d), deduped by message id: 546 messages, 98% cache-read, Opus list rates. Covers the whole wave's framing, the A-F spec/DEC design, the cold build/verify dispatch and all the ship bookkeeping. Excludes the delegated cold build/verify sessions, which are counted per-spec (wave delegated total ~177M across groups A-F)."
---

# STAGE-010: Downstream consumer API: validate-before-allocate, error kinds, determinism, 0.2.0

## What This Stage Is

The set of additive API and release changes a downstream consumer needs in order to
use irradiance as a **safe, versioned dependency**. The first consumer is
`pyirradiance` (Python/NumPy bindings, a PyTorch dataloader primitive), whose goal is
that a truncated or hostile file raises a catchable error and never kills a worker.
The stage delivers: a public, allocation-free `plane::validate` so the caller can size
its buffer from a proven bound before allocating; a stable error classification
(`Error::kind`) so a consumer can tell "unsupported, skip it" from "corrupt" from
"caller bug"; measured cross-OS/arch determinism evidence for the integer and
`f32` paths; and the `0.2.0` crates.io release that carries `demosaic`,
`tone::to_linear_f32_into`, `DEVELOP_VERSION` and the above. Every change is additive
and **no existing public API changes**.

Source: `feedback/2026-11-02-pyirradiance-requests.md` (the triage there is the
design input for every spec below).

## Why Now

pyirradiance is blocked on two items — validate-before-allocate (its "never crashes"
claim) and a released version containing `demosaic` (it cannot move from a path
dependency to a versioned one, so cannot publish to PyPI). The published crate is
`0.1.1` (2026-10-29) and predates all demosaic work. The stage is independent of
STAGE-008 (colour) and runs beside it; whether `0.2.0` is cut before STAGE-008 ships
is a maintainer decision recorded on the release spec.

## Success Criteria

- `plane::validate(sensor, byte_order, file) -> Result<usize, Error>` is public,
  allocation-free and panic-free; `Ok(n)` guarantees `unpack_into` with a `dst` of
  length `n` cannot fail on layout, size or truncation; `unpack_into` calls it, so the
  two cannot drift
- Forged-dimension files (huge width/height, tiny file) return a typed error with no
  large allocation, proven by tests and an extended fuzz target
  (sensor → validate → allocate → unpack)
- `Error::kind()` classifies every variant, and a new variant cannot land unclassified
- Golden hashes of `unpack_into`, `demosaic_into` and `to_linear_f32_into` output on
  tier-A fixtures are asserted on linux x86_64 **and** macOS arm64, and any float path
  that could diverge is named
- `irradiance 0.2.0` is on crates.io, passes the DEC-028 publish checks, and the
  feedback entry names the version to pin

## Scope

### In scope
- `plane::validate` + the refactor of `unpack_into` onto it, additive only
- `ErrorKind` + `Error::kind()`, preceded by a DEC
- A two-OS CI job and committed golden hashes over synthetic (tier-A) fixtures
- The `0.2.0` release spec

### Explicitly out of scope
- 4-channel RGGB packing, black/white normalisation for ML, NumPy `out=` buffers —
  pyirradiance handles these downstream (their stated out-of-scope list)
- Validate helpers for `demosaic_into` / `to_linear_f32_into`: their buffers are
  `3·n` of the validated `n`, so `n` is enough (re-check at design of SPEC-039)
- Asserting golden hashes on tier-B corpus files in CI (DEC-003); corpus hashes are
  recorded for local runs only
- Any change to the colour path (STAGE-008)

## Spec Backlog

> **RE-PLANNED 2026-11-10 (DEC-045).** The pre-0.2.0 external code review
> (`docs/reviews/2026-11-09-pre-0.2.0-code-review-triage.md`, PR #161) **holds
> 0.2.0** for four P1 findings and re-sequences the stage around the review's
> A–F grouping. The old "SPEC-039 first" ordering is superseded: the unified
> preflight (group E / SPEC-039) now depends on A–C.

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-052 (ship) — [S] **(A) Parser resource guard** (review #1): aggregate
      IFD entry/byte budget across the walk + reject aliased entry tables, so a
      ~770 KB file cannot drive a ~72 MiB parse. Independent. Emits a DEC (budget).
- [x] SPEC-053 (ship) — [S] **(B) Compression-7 shape binding** (review #2): bind the
      decoded `Frame` shape (width × `Nf`, height) to the strip, rejecting the
      equal-area wrong-shape case that currently returns `Ok(())`. Independent.
- [x] SPEC-054 (ship) — [M] **(C) Required-processing validation + `Sensor` API**
      (review #3): keep `OpcodeList2` bytes, typed error when a list needs an
      unsupported op (mono + colour); make `Sensor` non-exhaustive. **DESIGNED
      2026-11-10 (DEC-049); build-ready, HANDOFF-112.** Flags-aware scan at every
      develop entry + `Error::UnsupportedRequiredOpcode`; `Sensor` sealed + 7 `tests/`
      fixtures migrated to a dev-feature constructor; corpus probed safe (no file
      carries `OpcodeList2`). Design-heavy; gates E.
- [x] SPEC-055 (ship) — [S] **(D) Licence gate dev-graph + package surface**
      (review #4/#6): evaluate the dev-dependency graph (root + `fuzz/`) with a
      mutation red-proof; `irr` package disposition + a package-target-list check.
      Independent. Emits a DEC (`irr` disposition).
- [x] SPEC-039 (ship) — [M] **(E) Validate-before-allocate completion + docs**
      (review #5): one `plane::validate(sensor, byte_order, file, max_pixels)` over
      both the uncompressed and the Compression-7 path, with a caller-selected pixel
      budget checked first as the universal pre-allocation guard; reconcile
      README/CHANGELOG/conformance-matrix + a tested-DNG-modes table; add
      `ifd`/`plane`/`ljpeg` to the fuzz smoke matrix; ratify EOI-optional. **DESIGNED
      2026-11-10 (DEC-050); build-ready, HANDOFF-117.** A–C shipped, so depends_on is
      satisfied. Group C's two verify follow-ups are deferred here (DEC-050 §Scope),
      not folded in. depends_on: A, B, C (SPEC-052, SPEC-053, SPEC-054).
- [x] SPEC-040 (ship) — [S] **`Error::kind()` / `ErrorKind`**:
      `{ Unsupported, Corrupt, CallerError }`, non-exhaustive, via a **wildcard-free**
      `match` (compiler enforces "a new variant cannot land unclassified") + an exhaustive
      per-variant test. **DESIGNED 2026-11-10 (DEC-051); build-ready, HANDOFF-119.**
      depends_on: SPEC-039 (shipped).
- [x] SPEC-041 (ship) — [S] **Cross-arch determinism evidence**: tier-A synthetic Bayer
      fixtures, committed golden hashes for the three outputs, a linux x86_64 + macOS
      arm64 CI matrix; name any float path that could diverge; oracle shown red.
      **DESIGNED 2026-11-10 (no new DEC — rests on DEC-016/018/033); build-ready,
      HANDOFF-120.** Independent.
- [x] SPEC-042 (ship) — [M] **(F) Release `0.2.0`** (release spec, DEC-028 publish
      checks): the `Cfa`/`WarpRect`/`Opcode` non-exhaustive seal, semver 0.2.0, CHANGELOG
      promotion + release notes, a tier-B corpus evidence run on the candidate SHA, the
      package target list + deny-on-dev-graph confirmation; two-phase (Phase 1 prep PR;
      Phase 2 tag + publish human-gated). **DESIGNED 2026-11-10 (DEC-052); build-ready,
      HANDOFF-123.** The two group-C opcode follow-ups are deferred to SPEC-056/0.2.1; no
      colour-accuracy claim. depends_on: all of A–E + SPEC-040 + SPEC-041 (all shipped).
**Count:** 8 shipped / 0 active / 0 pending

> **Pre-0.2.0 P1 wave SHIPPED 2026-11-10** — groups A–D (SPEC-052/053/054/055)
> merged to `main` (PRs #167/#169/#175/#174) and bookkept. Wave cost 71,251,157
> tokens / $49.97 across 8 delegated build+verify sessions (A 11.40M/$8.44, B
> 16.46M/$11.51, C 26.19M/$17.96, D 17.20M/$12.06). All four verdicts clean (A/B
> APPROVED; C/D PUNCH LIST, both punch items fixed and shipped in the verify
> supersets). Remaining: **E (SPEC-039)** — now unblocked (depends A–C) — then
> SPEC-040 (needs a DEC), SPEC-041, and **F (SPEC-042)** release 0.2.0. Group-C
> verify raised three non-blocking follow-ups for E/F: a malformed opcode-list tag
> is still read as absent (can hide a mandatory opcode); the public stage
> functions (`demosaic_into*`, `to_srgb_linear_into`, `apply_warp_into`) skip the
> required-processing check; and plain `cargo test` (without `--all-features`) no
> longer compiles the integration tests.

### Dependency graph (DEC-045)

```
A (052)  B (053)  C (054)  D (055)   041      ← ready now, fan out in parallel
                    │                            (C gates E; A/B/C feed E's preflight)
   └──────┬─────────┘
          ▼
        E (039) ──► 040
          │
          ▼
        F (042)  ← release, blocked by A–E, 040, 041
```

## Design Notes

- **Additive only.** `Error` is `#[non_exhaustive]` (`src/lib.rs:143`), so new variants
  and new methods are non-breaking; no existing signature changes.
- **`validate` shares code with `unpack_into`**, it does not duplicate it. The bound on
  `n` holds *because* strip coverage is checked before `Ok(n)`; a pre-check that stops at
  the width×height multiply gives no such bound.
- **Error classification is three-way**, not boolean: caller-misuse variants
  (`PlaneBufferWrongLength`, `*WrongLength`) must not be reported as "hostile file".
- **Determinism evidence is measured, not read.** `to_linear_f32_into` is documented as
  a single IEEE division and demosaic as integer arithmetic; the CI job is what turns
  that reading into a measurement.

## Dependencies

### Depends on
- STAGE-006, STAGE-007 (shipped) — `plane`, `demosaic`, the tier-A CFA builders
- External: pyirradiance's glue (read-only); crates.io (DEC-028 publish checks)

### Enables
- pyirradiance moving to a versioned dependency and publishing to PyPI
- Any later consumer (crustyimg, the Monochrom app) calling a proven-safe allocation path

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** **Yes (prep-complete).** All
  four deliverables' code is on `main`: allocation-free `plane::validate` with a proven,
  caller-budgeted bound (SPEC-039), stable `Error::kind` / `ErrorKind` (SPEC-040), measured
  cross-arch determinism evidence on a linux-x86_64 + macOS-arm64 CI matrix (SPEC-041), and
  the 0.2.0 release prep (SPEC-042 Phase 1). Plus the four pre-0.2.0 P1 hardening fixes
  (SPEC-052/053/054/055) the external review added. **The one deliverable still open is
  "irradiance 0.2.0 on crates.io" — that is Phase 2 (tag `v0.2.0` + `cargo publish`), a
  human-gated cut; the release ships "prep-complete, cut-deferred".**
- **How many specs did it actually take?** **8** (SPEC-052/053/054/055 + 039/040/041/042)
  vs. the original pre-review plan of 4 (SPEC-039–042). The external pre-0.2.0 code review
  (PR #161) added the four P1 groups A–D and re-scoped E, doubling the stage (DEC-045).
- **How many outlines survived unchanged?** N/A — not framed with `just frame-stage`; the
  backlog was re-planned mid-stage by DEC-045 around the review's A–F grouping.
- **What changed between starting and shipping?** An external pre-0.2.0 code review held
  0.2.0 for four P1 findings, expanding the stage from 4 specs to 8 and re-scoping SPEC-039
  to depend on A–C.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **Force-push to update an open PR branch is denied by the auto-mode classifier** (Git
    Destructive) → rebase to a FRESH branch + a NEW PR (hit N=3 this wave: D #172→#174, C
    #173→#175, SPEC-041 #181→#183). Worth codifying in the delegated-cycle flow.
  - **A delegated build session can stamp its handoff's `synced_at` without the cost
    reaching the spec's `cost.sessions`** → at ship, verify the spec's `cost.sessions`
    (not just `synced_at`) and reset `synced_at: null` to re-sync (hit on SPEC-039).
  - **A superseded handoff must carry `handoff.status: superseded`** or `handback-sync`
    counts it pending and exits 1 (hit on HANDOFF-084).
- **Signals dispositioned at this close?** (Prompt 1d step 7)
  - `parser-accepts-what-pipeline-cannot-honor` (N≥2) was the spine of the P1 wave — the
    IFD byte guard (A), the Comp-7 shape binding (B) and the required-opcode validation (C)
    each close an instance; **codified** in DEC-046/047/049. Still `watch` for further
    instances (the deferred malformed-opcode-tag case is one). No silent carry.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - The two group-C verify follow-ups (malformed `OpcodeList` tag read as absent; stage
    functions skipping the required-processing check) and the two F non-blocking items
    (`cargo doc` private-item links; `tests/develop.rs` under `CARGO_TARGET_DIR`) → a
    focused 0.2.1 housekeeping spec (SPEC-056). Recorded as `watch`, not codified.
