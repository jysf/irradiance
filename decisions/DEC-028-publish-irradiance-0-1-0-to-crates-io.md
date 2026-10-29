---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-028
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: irradiance

created_at: 2026-10-28
supersedes: null
superseded_by: null
status: accepted
deciders: [claude]

affected_scope:
  - "**/Cargo.toml"
  - "projects/PROJ-001-monochrome-dng-develop/stages/STAGE-004-crustyimg-integration-raw-develop-feature-and-metadata-truthfulness.md"

tags:
  - release
  - publishing
  - crates-io
  - versioning
---

# DEC-028: Publish irradiance 0.1.0 to crates.io; reverse the second-camera publish-wait

## Decision

**Publish `irradiance` 0.1.0 to crates.io now, honestly scoped, and remove the
`publish = false` guard** — reversing STAGE-004's `explicitly_does_not` line
"Publishing to crates.io — that waits until a second camera is supported." The
crate keeps its 0.1.x API-semver; a separate crate-root const,
`DEVELOP_VERSION`, carries the develop **algorithm epoch** for downstream cache
invalidation.

## Context

STAGE-004 integrates `irradiance` into `crustyimg` as an **optional,
feature-gated dependency** (direction settled as crustyimg → irradiance). That
integration has a hard, non-negotiable cargo prerequisite:

> **An optional dependency declared as a bare path dep makes the consuming
> crate unpublishable.** `cargo publish` requires every dependency on the
> published crate — *including optional ones behind an off-by-default feature* —
> to resolve to a version on a registry. A `path = "..."` with no `version`
> has no registry coordinate, so `cargo publish` on crustyimg fails. Being
> off-by-default does **not** exempt it: cargo validates the whole dependency
> table at package time, not the active feature set. The only fix that keeps
> crustyimg publishable is for `irradiance` to exist on crates.io so the
> optional dep can read `{ version = "0.1", path = "...", optional = true }`.

So irradiance publishing is not a "someday" nicety; it is the one irradiance-side
thing that unblocks STAGE-004 at all. The original frame deferred publishing
until a second camera was supported, out of a reasonable caution: do not put a
crate on a public registry that silently over-promises its coverage.

That caution is real, but staying unpublished is the wrong instrument for it.
The honest fix for "do not over-promise" is to **say what the crate does** —
and the crate-facing `README.md` already does, in the first breath: monochrome,
uncompressed DNG, validated against one camera (a Leica Q2 Monochrom). A reader
on crates.io learns the exact scope before they add the dependency. Scope
honesty is a documentation property, not a publish-state property; withholding
the crate protects no one and blocks the consumer that motivates the whole
project.

The publish metadata was already complete (`license = "MIT OR Apache-2.0"`,
`description`, `repository`, both `LICENSE-MIT` and `LICENSE-APACHE`, a
crate-facing `README.md`). The only thing standing between here and a clean
`cargo publish` was the `publish = false` guard this decision removes, plus the
`keywords`/`categories` discoverability metadata it adds.

## Alternatives Considered

- **Option A: Keep `publish = false` until a second camera lands (the original frame).**
  - What it is: hold the crate off crates.io; crustyimg consumes it as a bare
    path dep behind the `raw-develop` feature indefinitely.
  - Why rejected: it makes crustyimg itself unpublishable (the cargo mechanic
    above), which defeats the integration. The over-promising it guards against
    is already prevented by the README's honest scope. The wait protects
    nothing it is credited with protecting.

- **Option B: Publish, but inflate to 1.0 / widen the described scope to look "done".**
  - What it is: present the crate as broadly capable to look registry-worthy.
  - Why rejected: dishonest, and the exact failure the original caution feared.
    0.1.0 with a scoped README is the truthful shape.

- **Option C (chosen): Publish 0.1.0 now, scope stated honestly in the README, API-semver held at 0.1.x.**
  - What it is: remove `publish = false`, add `keywords`/`categories`, publish
    0.1.0. The README states the monochrome/uncompressed-DNG/one-camera scope
    up front. The crate version tracks API compatibility (stays 0.1.x across
    many develop-algorithm epochs); output identity is tracked separately by
    `DEVELOP_VERSION`.
  - Why selected: it unblocks crustyimg's publishability, keeps the scope claim
    truthful at the point of consumption, and does not conflate "the pixels
    changed" with "the API changed."

## The algorithm epoch is not the crate version

A consumer caches developed pixels. Two facts can change those pixels: the
**input bytes** (a lockfile already catches a new crate version) and the
**develop algorithm** for the *same* input bytes (a new opcode, a tone change,
a geometry fix). The second can happen inside a single 0.1.x line — exactly the
kind of change that does not bump API-semver — and a lockfile cannot see it.

`pub const DEVELOP_VERSION: &str` (crate root, `src/lib.rs`) is that missing
signal. It is bumped (`"1"` → `"2"` → …) in the same change that alters develop
output, independently of the crate's semver. crustyimg folds it into its
build-cache key so `crustyimg build --frozen` re-develops a RAW when the
algorithm epoch moved even though no source byte and no crate version did. It
ships now, with a reader named (crustyimg's cache key) — per AGENTS.md §11
"ship the reader with the field," the const is not landing ahead of its
consumer; the downstream reader is the whole reason it exists.

## Consequences

- **Positive:** crustyimg becomes publishable with `irradiance` as a versioned
  optional dep. The library is installable by any permissively-licensed Rust
  imaging project — the differentiator the project exists for. `DEVELOP_VERSION`
  gives downstream a deterministic cache signal a lockfile cannot carry.
- **Negative:** a public 0.1.0 invites issues/expectations against a
  deliberately narrow crate. The README scope paragraph is the mitigation and
  must stay honest as coverage changes. Yanking is possible but noisy, so the
  first published version should package cleanly (`cargo publish --dry-run`
  gates this).
- **Neutral:** API-semver and algorithm-epoch now move on two independent axes.
  A reader must know that a 0.1.x → 0.1.y bump may or may not change pixels, and
  that `DEVELOP_VERSION` — not the crate version — is the pixel-identity signal.

## Validation

- `cargo publish --dry-run` packages cleanly (run as part of this change; the
  maintainer runs the real `cargo publish` from `main` after merge, with their
  crates.io token — the irreversible operator action stays with the human).
- STAGE-004 proceeds: crustyimg's optional dep reads a registry version and
  crustyimg's own `cargo publish --dry-run` stops failing on the bare path dep.
- Revisit if the scope widens materially (a second camera, CFA/demosaic, a
  compressed-plane decoder): that is a README scope edit and likely a
  minor-version bump, not a re-litigation of *whether* to publish.

## References

- Related specs: — (release bookkeeping, not a spec; STAGE-004 is the consumer)
- Related decisions: reverses the publish line in
  `STAGE-004-crustyimg-integration-raw-develop-feature-and-metadata-truthfulness.md`
  (`value_contribution.explicitly_does_not`, amended to cite this DEC). Builds on
  DEC-002 (output determinism "pinned within a `develop_version`" — this is that
  version string, surfaced as `DEVELOP_VERSION`).
- ⚠ **Cross-repo numbering fix:** the crustyimg session's STAGE-004 review
  informally referred to the *planned* LUT-boundary decision as "DEC-028."
  This publish decision takes DEC-028 (verified against this repo's
  `decisions/INDEX.md`, whose prior max was DEC-027), so the LUT-boundary
  decision becomes **DEC-029**. Anyone carrying the "DEC-028 = LUT boundary"
  reference across repos must correct it to DEC-029.
- External docs: crate-facing `README.md` (the honest-scope statement);
  the Cargo Book, "Publishing on crates.io" (every dependency, including
  optional ones, needs a registry `version` at publish time).
