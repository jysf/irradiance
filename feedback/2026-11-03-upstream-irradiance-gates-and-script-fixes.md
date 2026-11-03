---
source: "irradiance (PROJ-001/PROJ-002) — jysf"
captured_at: 2026-11-03
captured_by: claude
status: open
---

# Irradiance's gate scripts and `scripts/` bug fixes are not in the template (v0.6.39)

## The issue

irradiance (scaffolded from 0.6.38) has grown 6 gate scripts the template lacks and
fixed 8 template scripts in place. Every repo scaffolded from the template — including
`jysf/diamond-exif`, already created from 0.6.39 — inherits the *unfixed* versions and
has to re-earn each fix. Several of the fixes were found as real failures, not by
inspection.

Compared against the template clone at `22aec2e` (VERSION 0.6.39), `cmp` whole-file:
38 of irradiance's 52 `scripts/*.sh` are byte-identical; 6 are only in irradiance; 8 differ.

### A. Only in irradiance (6) — new gates, each with a red-proof

| Script | What it does |
|---|---|
| `gate-audit.sh` | Scans `scripts/*.sh` for the three "mute-death" shapes (a `$(… \| grep …)` that aborts the script under `set -e`/`pipefail` with no `die`). SPEC-023. |
| `gate-audit-red-proof.sh` | Injects each shape and proves gate-audit goes red; also proves a *comment* containing the shape is NOT flagged. |
| `validate-red-proof.sh` | Proves the front-matter **parse** gate in `just validate` goes red on an unparseable fixture (`tests/oracle-fixtures/SPEC-900-unparseable-front-matter.md` — must travel with it). SPEC-016. |
| `cost-audit-red-proof.sh` | Proves the STAGE orchestration-cost half of `cost-audit` bites. PATCH-002. |
| `lint-red-proof.sh` | Proves the crate-root `#![deny(...)]` policy bites by injecting violations. Rust-specific. DEC-006/007. |
| `lint-fuzz-red-proof.sh` | Same for the separate-cargo-graph `fuzz/` crate. Rust-specific. DEC-011/027. |

The principle all six share: **an oracle/gate that has never been shown red is not evidence**.
The generic four (`gate-audit*`, `validate-red-proof`, `cost-audit-red-proof`) belong in the
template; the two `lint-*` ones are Rust-stack and may fit better as a `variants/` or
stack-recipe addition — maintainer's call.

### B. Differs from the template (8) — fixes

I read these diffs; each looks generic, not irradiance-specific:

| Script | Fix |
|---|---|
| `_lib.sh` (large) | `\|\| true` guard on the project-resolution `grep` (§16 rule 3); `next_id` excludes `tests/` (it minted `SPEC-901` from a fixture); YAML-comment strip no longer truncates quoted `notes:` containing `#`; the front-matter **parse** gate (SPEC-016). |
| `validate.sh` | Calls the parse gate. |
| `cost-audit.sh` | Stage-grain orchestration cost check + grandfather constants (`STAGE_ORCH_COST_GRANDFATHERED`, `COST_AUDIT_GRANDFATHERED` — a fresh repo should default these empty). |
| `handback-sync.sh` | Small fix; diff not analysed in detail. |
| `archive-spec.sh` | `find … \| xargs -I{} awk …` died with BSD xargs "command line cannot be assembled, too long" as the script's last block, aborting *after* the archive succeeded. Replaced by a `while read` loop. |
| `specs-by-stage.sh` | Ship date prefers the spec's top-level `shipped_at` (stamped by `archive-spec`), then the `ship` cost session's `recorded_at` (hand-written, routinely skipped), then the stage's. |
| `decisions-audit.sh` | `\|\| true` on the `# DEC-` title grep: a DEC missing its H1 aborted the whole lint mutely. |
| `test.sh` | The template-selftest has ~7 unguarded `$(… \| grep …)` assignments of the same mute-death shape (lines ~1482, 1489, 1701, 1704, 1733, 1738, 1773, 1894); guarded with `\|\| true`. Note these are in the *selftest*, which is itself a gate. |

Companion wiring that must move with the scripts or they never run: the recipes in
`app.just` (`gate-audit`, `gate-audit-red-proof`, `validate-red-proof`,
`cost-audit-red-proof`, …) and the matching steps in `.github/workflows/ci.yml`
(see the "gate-audit goes red on an injected mute-death shape" and
`validate-red-proof` steps). The template's `justfile`/`app.just` split means these
belong in the template's `justfile` (or a documented extension point), not `app.just`.

## Context

Surfaced while reading irradiance against the template for a porting brief
(`docs/porting-brief-diamond-exif.md`, §3) when starting `diamond-exif`. diamond-exif is
at VERSION 0.6.39 with 46 scripts and none of the above. crustyimg (34 scripts) also lacks
all six gates — so three repos are affected, not one.

This overlaps open signals in `guidance/signals.yaml` that already say "template-level:
fix upstream, not just here" (the new-handoff cost-section fix, the cost-audit gap) and the
open ID-minter entry (`2026-09-06-id-minter-collides-across-unmerged-branches.md`).

## Priority (their assessment)

High, and time-sensitive: diamond-exif is brand new and will accumulate work on the
unfixed scripts. Doing the template first lets diamond-exif be brought up by a single
version bump rather than a hand-merge, and avoids two diverging copies of the gates.

## Suggested prompt for the template maintainer agent

> Read `feedback/2026-11-03-upstream-irradiance-gates-and-script-fixes.md` in
> `jysf/irradiance` (and `docs/porting-brief-diamond-exif.md` §3). Reference checkout:
> irradiance `main` at or after `d30fe2f`.
>
> 1. **Diff the reference.** For each of the 8 differing scripts, confirm the fix is
>    generic (not irradiance-specific) and port it. `handback-sync.sh` and `cost-audit.sh`
>    need a closer read — I did not analyse them in full. Reset the grandfather constants
>    to empty for the template default.
> 2. **Add the generic gates**: `gate-audit.sh`, `gate-audit-red-proof.sh`,
>    `validate-red-proof.sh` (+ its `tests/oracle-fixtures/` fixture), `cost-audit-red-proof.sh`.
>    Generalise anything that assumes irradiance's file layout (gate-audit-red-proof checks
>    for `.repo-context.yaml VERSION justfile app.just AGENTS.md guidance`).
> 3. **Decide where the Rust-only `lint-red-proof` / `lint-fuzz-red-proof` live** (variant,
>    stack recipe, or documented-but-not-shipped). They hard-code `src/lib.rs` and a
>    single-crate layout; document the retargeting points.
> 4. **Wire them**: `justfile` recipes + CI steps; extend `just template-selftest` so the
>    new gates are exercised in the scaffold (the selftest must go red if a gate is removed).
> 5. **Run `just gate-audit` against the template's own `scripts/` and `scripts/test.sh`** —
>    the selftest has the same mute-death shape in ~7 places.
> 6. **Bump VERSION** (0.6.40 or minor per your convention), update CHANGELOG, and say in the
>    release notes how an existing instance upgrades (`just instances` / the instance
>    drift tooling from 0.6.38 should report diamond-exif and irradiance as behind).
> 7. Mark the ID-minter entry's status if this batch is not the vehicle for it.
>
> Do not push before the maintainer confirms. Report back: which diffs you rejected as
> irradiance-specific, and why.

## Resolution

Open. Fill in with the template release that lands this, then bring diamond-exif and
crustyimg up to it.
