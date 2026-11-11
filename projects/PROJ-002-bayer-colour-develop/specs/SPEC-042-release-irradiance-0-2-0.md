---
# A RELEASE SPEC is the spec for cutting a release — tagging, building the
# artifact, publishing it through each channel, and confirming it actually runs
# on a machine that didn't build it. See AGENTS.md "During ship" and
# docs/decisions/DEC-006.
#
# It uses the same task.* schema as a normal spec (so `just validate`,
# `just cost-audit`, and `just status` treat it as first-class), with
# `task.type: release`. The value it adds over a plain spec is the
# ## Release Pre-Flight checklist below: a generic, portable set of
# runtime/operational categories that every release hits, so each project
# stops re-earning the same gotchas one release at a time. This variant
# delegates the cut to an implementer via handoffs/HANDOFF-*.md.

task:
  id: SPEC-042
  type: release                    # epic | story | task | bug | chore | release
  cycle: build                     # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: M                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

project:
  id: PROJ-002
  stage: STAGE-010
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-028, DEC-006, DEC-045, DEC-052]   # publish checks, changelog→version, the wave, the seal + semver
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-039, SPEC-040, SPEC-041]                # [SPEC-NNN]

# Blocking: SPEC-039 must SHIP before the release is cut (feeds `just ready`).
depends_on: [SPEC-052, SPEC-053, SPEC-054, SPEC-055, SPEC-039, SPEC-040, SPEC-041]  # group F — blocked by all of A–E + 040 + 041 (DEC-045)

value_link: null

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
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

# SPEC-042: Release irradiance 0.2.0

> **DESIGNED 2026-11-10 (DEC-052), `cycle: build`.** All blockers shipped — groups
> A–E (SPEC-052/053/054/055/039) + SPEC-040 + SPEC-041 are on `main`. This is the
> group-F release. **Read DEC-052 first:** it decides the `Cfa`/`WarpRect`/`Opcode`
> seal, the semver `0.2.0` version, the deferral of two group-C opcode follow-ups to a
> 0.2.1 spec (SPEC-056), and that 0.2.0 does not wait for colour accuracy. The build is
> **two-phase**: Phase 1 (this spec's build) is the reversible prep PR; Phase 2 (the tag
> + `cargo publish`) is **human-gated** and the spec ships "prep-complete, cut-deferred".

## Context

Source `feedback/2026-11-02-pyirradiance-requests.md` item 2. The published crate is `0.1.1` (2026-10-29; `0.1.0` yanked) and predates all demosaic work, so pyirradiance cannot move from a path dependency to a versioned one. This is the release that ships `demosaic::demosaic_into`, `tone::to_linear_f32_into`, `DEVELOP_VERSION` and `plane::validate` (SPEC-039), plus `Error::kind()` (SPEC-040) and the determinism evidence (SPEC-041) if they have shipped. Intended version: **`0.2.0`** (additive modules; `Error` is `#[non_exhaustive]`). Runs under the DEC-028 publish checks. The release spec is a stub until designed; the maintainer decides whether it waits for STAGE-008 (colour).

## Release Scope

### Decided for 0.2.0 (DEC-052)

- **Version / tag:** `0.2.0`, **semver** — set `spec.version.scheme: semver` in
  `.repo-context.yaml` (public-API library). Tag `v0.2.0`; a minor bump over the live
  `0.1.1`. The top-level `VERSION` file (template provenance) is left alone.
- **What's included (on `main` since `0.1.1`):** `demosaic::demosaic_into` + the colour
  develop path (STAGE-007/008/012), `tone::to_linear_f32_into`, `DEVELOP_VERSION`,
  `plane::validate` (SPEC-039), `Error::kind` / `ErrorKind` (SPEC-040), the determinism
  evidence (SPEC-041), the pre-0.2.0 P1 hardening (SPEC-052/053/054/055), and this spec's
  `Cfa`/`WarpRect`/`Opcode` seal.
- **Delivery shape:** library + crates.io **package** (no binary — the `irr` dev tool is
  feature-gated out, SPEC-055). Semver applies; the binary/service pre-flight categories
  are `N/A`.
- **Known limitations, named in the release notes (not gates):** the two deferred group-C
  opcode follow-ups (a malformed `OpcodeList` tag read as absent; the stage functions
  skipping the required-processing check) → **SPEC-056 / 0.2.1**; and **no colour-accuracy
  claim** — SPEC-035 (ColorChecker ΔE) is blocked on the physical chart.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

Everything here lands as one CI-gated PR; it is fully revertible. The tag + publish are
Phase 2 (below), human-gated.

1. **Seal the public parser-output types** (DEC-052 §1): add `#[non_exhaustive]` to `Cfa`
   (`src/ifd.rs`), `WarpRect` and the `Opcode` enum (`src/opcode.rs`). Confirm in-crate
   matches still compile (in-crate exhaustive matches stay legal); do **not** add a
   crate-wide wildcard. Migrate the `tests/` `Cfa`/`WarpRect`/`Opcode` literal fixtures
   onto the existing dev-only `test-fixtures` feature (extend it with constructors, the
   `fuzzing`/SPEC-054 precedent).
2. **Follow-up (3) — make bare `cargo test` compile** (DEC-052 §3): add a dev-dependency on
   this crate with the dev features on (`irradiance = { path = ".", features =
   ["test-fixtures", "irr"] }`) so the sealed fixtures build without `--all-features`.
3. **Version + metadata** (DEC-028): `Cargo.toml` `version = "0.2.0"`; confirm
   `license` / `description` / `keywords` / `categories` / `repository` are publish-ready;
   `.repo-context.yaml` `spec.version.scheme: semver`.
4. **CHANGELOG** (DEC-006): promote `[Unreleased]` to `## [0.2.0] - <date>`; a fresh empty
   `[Unreleased]` on top.
5. **Release notes:** run `just release-notes`, edit to a user-facing headline; name the
   two deferred opcode follow-ups and the absence of a colour-accuracy claim under
   **Known limitations**.
6. **Tier-B corpus evidence run on the candidate SHA** (`IRRADIANCE_CORPUS_DIR`): record
   the bit-exact develop/decode result for every held file in the handback — this is the
   release's real-file evidence (never committed, DEC-003).
7. **Confirm the already-landed guards:** `just package-targets-check` (no stray binary in
   the package, SPEC-055) and `just deny` + `just deny-fuzz` (dev-graph licence, SPEC-055)
   pass on the candidate.
8. Fill the **Release Pre-Flight** checklist below with the instance commands and `[now]`
   / `[cut]` / `N/A` timing; `just test` (`--all-features`) + `just lint` green;
   `cargo publish --dry-run` clean (`[now]`).

**Phase 2 (human-gated, after Phase 1 merges — NOT in the build cycle):** tag `v0.2.0` on
the release commit, `cargo publish`, verify the published crate installs on a clean host.
The spec ships **prep-complete, cut-deferred**; the implementer does not tag or publish.

## Release Notes

*Outward-facing — written for whoever USES this, not for contributors. Distinct
from the CHANGELOG (the record of what changed) and from the Reflection below
(internal learning).*

**Seed it, don't write it from scratch:** run `just release-notes` — it assembles
each shipped spec's outward outcome (Reflection Q5). Then edit: lead with the
headline, group related items, drop the SPEC ids. Specs that answered `none` have
no user-visible outcome and are correctly absent — do not back-fill them.

*Seeded with `just release-notes PROJ-002` + `just release-notes PROJ-003` on
2026-11-10, then edited (SPEC ids dropped, related items grouped). The GitHub
release text for `v0.2.0` is this section.*

**Headline:** irradiance now develops **Bayer colour** DNGs and decodes **lossless-JPEG
(`Compression 7`)** DNGs — still pure Rust, zero runtime dependencies, permissively
licensed, and refusing hostile input with a typed error instead of a panic or an
out-of-memory abort.

**New capability**
- **Bayer colour develop.** Before, only monochrome DNGs developed. Now a Bayer-CFA
  DNG (uncompressed, single- or multi-strip) unpacks bit-exact against `dnglab`,
  demosaics with a patent-clean Hamilton-Adams kernel (`demosaic::demosaic_into`),
  and colour-develops — white balance, dual-illuminant camera→XYZ, Bradford
  adaptation, sRGB — through `color::develop_color_into`, cropped and oriented
  like the monochrome develop.
- **Scene-linear float output.** `tone::to_linear_f32_into` and
  `color::develop_color_f32_into` write `f32` directly (for Python bindings and ML
  loaders), equal to the `u16` path to the bit.
- **Lossless-JPEG DNGs decode.** A `Compression 7` sensor plane now decodes through
  `plane::unpack_into`; both held files (Leica M Monochrom Typ 246, Pentax K-3 III
  Mono) match `dnglab`'s raw checksum exactly.
- **Faster and leaner.** Demosaic ~1.69× faster (24 MP median 533.0 → 316.2 ms);
  colour develop ~134 MB less peak memory on a 23 MP frame (358.7 → 224.9 MB); an
  opt-in `*_threaded` variant scales ~3.3–3.7× at 4 threads, byte-identical output.
- **`DEVELOP_VERSION` is now `"4"`** (was `"1"` in `0.1.1`) — bumped by the three
  colour-develop output changes since (scene-white adaptation, the `ActiveArea`
  origin, crop/orient parity), so a cache keyed on it re-develops.

**Safer on untrusted input**
- **`plane::validate(sensor, byte_order, file, max_pixels)`** — run it before you
  allocate: a forged sensor size can no longer drive the caller into an
  out-of-memory abort; your own pixel budget is checked first.
- Aliased IFD tables can no longer amplify parsing (a ~770 KB file used to drive
  ~6.3M entry reads); a wrong-shape `Compression 7` strip is refused, not silently
  mis-decoded; a DNG that *requires* opcode processing the develop does not perform
  is refused (`Error::UnsupportedRequiredOpcode`) instead of developing to a
  plausible wrong image.
- **`Error::kind()` → `ErrorKind { Unsupported, Corrupt, CallerError }`** — route a
  failure (skip / bad file / fix the call) without matching ~55 variants.
- **Cross-architecture determinism evidence** — committed output hashes for
  unpack, demosaic and the linear-float path agree on linux-x86_64 and macos-arm64.
- `cargo install irradiance` no longer installs the internal `irr` dev tool.

**Breaking / action required:** this is a `0.x` minor bump, so Cargo treats it as
incompatible with `0.1`: change your dependency to `irradiance = "0.2"`. Code
changes are needed only if you (a) **construct** `Sensor`, `Cfa` or `WarpRect`
with a struct literal, or (b) **match** `Opcode` exhaustively — all four are now
`#[non_exhaustive]` parser-output types, so use the value the parser returns and
add a `_` arm. Field reads, field assignment and `..` patterns are unchanged. Also
newly refused (typed errors, not panics): uncompressed strips whose byte counts
exceed the file, and files requiring opcode processing the develop cannot apply.

**Known limitations**
- **No colour-accuracy claim yet.** The colour path is verified against independent
  transcriptions, not against a physical reference; the ColorChecker ΔE2000
  measurement is blocked on capturing the chart. Treat colour as plausible, not
  calibrated.
- **A malformed `OpcodeList` tag is read as absent**, so a file that declares
  required processing in a list this library cannot parse develops without it
  rather than being refused. No held corpus file does this. Planned for `0.2.1`.
- **The stage functions below `develop_*`** (`demosaic_into*`,
  `to_srgb_linear_into`, `apply_warp_into`) **skip the required-processing check**
  that every `develop_*` entry point performs; only callers composing those
  primitives directly are affected. Planned for `0.2.1`.
- **The colour develop does not run the `OpcodeList` stages** (`WarpRectilinear`,
  `FixBadPixelsConstant`) that the monochrome `develop_into` runs; a file that
  *requires* them is refused with a typed error, and optional ones are skipped.

## Release cut is two-phase

A release session **cannot finish in one pass** — the irreversible tag/publish is
gated by a human/coordinator. Structure it so the reversible work lands first:
- **Phase 1 — reversible prep (CI-gated PR):** CHANGELOG entry, version bump
  (`just next-version`), stage-backlog tick. Fully revertable — land it as a
  normal PR.
- **Phase 2 — irreversible cut (human/coordinator-gated):** create the tag and
  publish to each channel, only after Phase 1 merges. The spec ships
  **"prep-complete, cut-deferred."**

## Release Pre-Flight

**A behavioral checklist — each item asks "does the released thing actually work
on a real host," not "is the shape right."** The categories are generic and
portable (DEC-006); **fill in the tool-specific command for each under your
stack** (the template ships the slot, the instance fills the truth — same
principle as the toolchain brief).

Mark each item's verification **timing**, because most can only be checked *after*
the cut: **`[now]`** = verified this session, evidence attached; **`[cut]`** = can
only be verified at Phase 2 (a clean-host install, a published-channel check,
notarization propagation) → **deferred, verified by whoever runs the cut** — record
it honestly as deferred, don't tick it as done in-session. Mark categories that
don't apply to this release's delivery shape as `N/A` with one word why.

**Delivery shape: library + crates.io package (source crate, no binary).** Filled
2026-11-10 by the Phase-1 build (HANDOFF-123). `[now]` = verified on the candidate
in the build session; `[cut]` = only checkable at Phase 2, verified by the human
cutter; `N/A` = does not apply to a source crate.

- [x] **1. Version / tag integrity & build provenance** — partly `[now]`, rest `[cut]`.
      - `[now]` `Cargo.toml` `version = "0.2.0"`; `cargo metadata` and `cargo publish
        --dry-run` both report `irradiance v0.2.0`; `git tag -l 'v0.2*'` is empty
        (no premature or duplicate tag); `.repo-context.yaml` `spec.version.scheme:
        semver` (already set — no change needed). `just build-info` stamps the
        candidate as `v0.1.1-<n>-g<sha>` until the tag exists. The dry-run's
        packaged `.cargo_vcs_info.json` carries `git.sha1` = the candidate commit
        `6843083`, so the provenance mechanism is shown working pre-cut.
      - `[cut]` on the merged release commit: `git tag -a v0.2.0 -m "irradiance
        0.2.0"`, then `git tag --points-at HEAD` prints exactly `v0.2.0` and
        `just build-info` prints `v0.2.0`. **Build provenance for a source crate:**
        `cargo publish` from a clean tree embeds `.cargo_vcs_info.json` (`git.sha1`)
        in the `.crate`; after publishing, confirm that sha1 equals the tagged
        commit (`curl -sL https://static.crates.io/crates/irradiance/irradiance-0.2.0.crate
        | tar -xzO irradiance-0.2.0/.cargo_vcs_info.json`). `DEVELOP_VERSION` (`"4"`)
        is the pixel-identity stamp (DEC-028).
- [x] **2. Artifact trust on a clean host** — `N/A`: source crate; there is no signed
      or notarized binary (the `irr` dev tool is feature-gated out of the package,
      SPEC-055).
- [ ] **3. Distribution-channel trust** — `[cut]`.
      - `[now]` (closest pre-cut proxy): `cargo publish --dry-run` packages and
        compiles the crate in isolation from the registry-resolved manifest
        (the path-only self dev-dependency is stripped); `just
        package-targets-check` unpacks the `.crate` and builds it with default
        features — no executable.
      - `[cut]` on a host that never saw the source: `cargo new probe && cd probe
        && cargo add irradiance@0.2.0 && cargo build`; confirm
        <https://crates.io/crates/irradiance> shows 0.2.0 and the docs.rs build is
        green.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` on the candidate, `[cut]` on the
      published crate.
      - `[now]` `just test` (`--all-features`) green with tier-B 9/9 present, every
        held file bit-exact (see the HANDOFF-123 handback); a bare `cargo test`
        (no `--all-features`) compiles the integration tests.
      - `[cut]` in the clean-host `probe` crate above: parse + `plane::validate` +
        `unpack_into` + `develop_into` one held DNG (the README's end-to-end
        example) and confirm it returns `Ok` with the expected dimensions.
- [ ] **6. Rollback / uninstall** — `[cut]`. The withdrawal path is `cargo yank
      --version 0.2.0` (consumers on `"0.2"` fall back to nothing; `"0.1"` users are
      unaffected); it was exercised on 2026-10-29 when `0.1.0` was yanked (DEC-028
      history). Run only if the published crate is broken; an un-yank is `cargo yank
      --undo --version 0.2.0`. A git rollback of the prep is a revert of this PR.

If a real release surfaces a category this list doesn't cover, record it in
`guidance/signals.yaml` (`type: process-debt`) so a close can grow the generic
categories (DEC-006 keeps them category-level, not command-level).

## Notes for the Implementer

Gotchas specific to *this* release — a channel that's slow to propagate, a
signing cert that expires, an env var the release runner needs. Keep short — the
full context graph lives in the handoff file.

**Phase 2 runbook (human cutter, after the Phase-1 PR merges):**
1. `git checkout main && git pull` — clean tree, HEAD is the merged prep commit.
2. `cargo publish --dry-run` once more on that exact commit (clean, no `--allow-dirty`).
3. `git tag -a v0.2.0 -m "irradiance 0.2.0"` and `git push origin v0.2.0`.
4. `cargo publish` (your crates.io token).
5. Pre-flight items 1, 3, 5 `[cut]` halves: `.cargo_vcs_info.json` sha1 = tag commit;
   clean-host `cargo add irradiance@0.2.0` + the README example; docs.rs green.
6. Optional: a GitHub release on `v0.2.0` with the `## Release Notes` text above.
7. Append the one-line `## Cut Record` below.

Gotcha: the self dev-dependency (`irradiance = { path = "." … }`) is path-only, so
`cargo package` strips it from the published manifest — the `.crate` has no
self-reference. The dry-run in this PR confirms it.

## Cut Record

*One line, appended after Phase 2 by whoever runs the cut.* The user-facing
outcome now live, plus the upgrade/clean-install check that confirms the
publish landed — the one line a review or work-log would quote. E.g.
*"v0.4.0→v0.5.0 `brew upgrade` clean; MCP-install now first-class; prod DB
opened, 189 entries intact."* If the release is pure infra with no user-facing
change, say so — a real, greppable outcome, not a blank.

---

## Reflection

*Appended during **ship**. Four questions, short answers.*

1. **What would I do differently next release?**
   — <answer>

2. **Does any template, constraint, or decision need updating?** (e.g. a new
   pre-flight category to add to this template)
   — <answer — if yes but not done this session, record it in
   `/guidance/signals.yaml`. See `docs/signals.md`.>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>

4. **Where was the worst defect caught?** — one word from a fixed vocabulary so
   the defect-escape distribution is greppable across specs:
   `design` | `build` | `verify` | `ship` | `escaped` (reached prod/runtime) |
   `none` (clean first try).
   — <one word>
   *(A release-phase `escaped` defect means a pre-flight category above was
   missing or skipped — grow the checklist.)*
