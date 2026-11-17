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
  id: SPEC-073
  type: release                    # epic | story | task | bug | chore | release
  cycle: build  # frame | design | build | verify | ship
  blocked: true
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

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
  decisions: [DEC-052, DEC-028, DEC-006, DEC-066]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-042, SPEC-056]

# Blocking dependencies: specs that must SHIP before this one can start.
depends_on: [SPEC-071, SPEC-072]

value_link: "gets STAGE-018..020's develop options, previews, tiled planes, plane units and decode speed-ups to crates.io as a compatible 0.2.2"

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
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

# SPEC-073: The next irradiance release: semver check, CHANGELOG promotion, prep-complete for the user to tag and publish

## Context

The third and last spec of STAGE-021. 0.2.1 (2026-11-11) predates all of STAGE-019 and
STAGE-020: develop options, warp alpha, caller scratch, per-position black levels, the
tiled orientation gather, plane units, and the packed and Compression 7 speed-ups, plus
earlier unreleased work already in CHANGELOG `[Unreleased]` (previews, tiled planes).
The monorail/onyx consumer asked for these. Design decides the version against semver
(0.2.2 or 0.3.0; DEC-052 is the precedent) and fills the pre-flight below. The user tags
and runs `cargo publish`; this spec ends prep-complete.

## Release Scope

### Decided for 0.2.2 (design, 2026-11-16; the user chose the version)

- **Version / tag:** `0.2.2`, semver (`spec.version.scheme: semver` is already set). Tag
  `v0.2.2`. A **patch** bump over the live `0.2.1`, because everything since is
  backward-compatible. Cargo treats `0.2.z` releases as compatible, so anyone on
  `irradiance = "0.2"` receives it with `cargo update`, and dependents need no
  requirement change.
- **Why compatible: the design's public-API audit** (`git diff v0.2.1 origin/main` at
  `59f8b1a`):
  - **No public item was removed or had its signature changed.** No `pub` declaration
    appears on a removed line.
  - **Every new public field is on a `#[non_exhaustive]` type:**
    - `Sensor` gains `tile_width`, `tile_length`, `tile_offsets` and
      `tile_byte_counts`;
    - `Error` gains new variants;
    - the new types (`DevelopOptions`, `plane::Unit`, the preview types) are themselves
      `#[non_exhaustive]`.
  - **Nothing else a caller compiles against changed.** There is no new dependency and
    no new feature; the MSRV stays `rust-version = "1.90"`; and the only `Cargo.toml`
    change is the dev-only `[profile.corpus]`, which profiles ignore for dependents.
  - **Default outputs are unchanged.** `DEVELOP_VERSION` stays `"4"`, and every develop
    and decode pin held through STAGE-018 to STAGE-021.
  - **Behaviour changes are compatible.**
    - Tiled planes now decode where they used to return a typed `Unsupported` error.
    - `develop_into` allocates at most once.
    - The two `opcode::parse_*` functions no longer allocate.
    - Two decodes are faster.
    - One error message changed (`UnsupportedStripLayout`).
- **A mechanical check is added at build (the user approved the download):** install
  `cargo-semver-checks` (`cargo install cargo-semver-checks --locked`) and run
  `cargo semver-checks check-release --baseline-rev v0.2.1` (or `--baseline-version
  0.2.1` against the published crate). **Any reported breaking change stops the build.**
  The release then needs the user's decision (0.3.0, or a fix), not a workaround.
- **What's included** (on `main` since `0.2.1`): STAGE-018 (embedded previews),
  STAGE-019 (develop options, warp alpha, caller scratch, per-position black levels,
  tiled orientation), STAGE-020 (plane units, the packed and Compression 7 speed-ups),
  tiled sensor planes (SPEC-058), and STAGE-021's test and tooling work (no library
  change).
- **Delivery shape:** library plus a crates.io **package**. There is no binary (`irr` is
  feature-gated out, SPEC-055), so the binary and service pre-flight categories are
  `N/A`.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

Everything here lands as one CI-gated PR, and it is fully revertible. The tag and publish
are Phase 2 below, and the user does them.

1. **The semver check** (above). Record the tool's version and its full output.
2. **Version:** `Cargo.toml` `version = "0.2.2"`. Confirm `license`, `description`,
   `keywords`, `categories`, `repository`, `readme` and `rust-version` are unchanged and
   publish-ready.
3. **CHANGELOG** (DEC-006):
   - Promote `[Unreleased]` to `## [0.2.2] - <date>`, keeping every entry.
   - Put a fresh, empty `## [Unreleased]` on top.
   - Leave the compare links as they are, or update them if the file has them.
4. **Release notes:** run `just release-notes PROJ-004`, then edit the result into the
   `## Release Notes` section below, keeping the drafted headline and grouping. Specs
   whose reflection answered `none` stay absent.
5. **README:** check that the crate-level docs and README examples still compile and
   describe 0.2.x accurately, using `cargo test --doc` and `just doc-check`. Fix wording
   only if it is now wrong, for example a claim that tiled planes are unsupported.
6. **Tier-B evidence run on the candidate SHA**, with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`, which now takes about 3 min. Record that every held file was present
     and every pin held.
   - The three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded for the release notes' numbers.
7. **Package guards:** `just package-targets-check` (no executable in the package),
   `just deny` and `just deny-fuzz`, and `cargo publish --dry-run`. All must be clean.
   The packaged manifest must not contain the self dev-dependency.
8. **Fill the Release Pre-Flight below,** marking each item `[now]`, `[cut]` or `N/A`.
   Then run every gate:
   - `just test`
   - `lint-ci`
   - `lint-no-allow`
   - `lint-fuzz`
   - `fmt-check`
   - `validate`
   - `doc-check`
   - `deny`
   - `msrv`
   - `cost-audit`
   - `gate-audit`

**Phase 2** is human-gated, after Phase 1 merges, and **not in the build cycle**: tag
`v0.2.2` on the release commit, run `cargo publish`, and verify the published crate on a
clean host. The spec ships **prep-complete, cut-deferred**. The implementer never tags or
publishes.

## Release Notes

*Outward-facing — written for whoever USES this, not for contributors. Seeded at build
with `just release-notes PROJ-004`, then edited. The GitHub release text for `v0.2.2` is
this section.*

**Headline (drafted at design; the build confirms the numbers):** irradiance 0.2.2
lets you shape the develop and split the decode. It adds develop options (full active
area, unrotated output, partial lens correction, caller-owned scratch and per-position
black levels), embedded preview access, tiled sensor planes, and plane units you can
decode on your own threads. The single-threaded decode is much faster: about 25× on
packed 12/14-bit planes and about 6.5× on lossless-JPEG. All of it is
backward-compatible with 0.2.x, still pure Rust with no runtime dependencies, and
`DEVELOP_VERSION` stays `"4"`.

The build fills the groups from `just release-notes` and the CHANGELOG:
- **New capability:** develop options; previews; tiled planes; plane units.
- **Faster:** packed 12/14-bit unpack; Compression 7; Orientation 5–8 gather; the
  develop allocation drop.
- **Safer on untrusted input:** the new typed errors, each in one line.

**Breaking / action required:** none. This is a compatible 0.2.x release:
`cargo update -p irradiance`. New `Error` variants and `Sensor` fields are on
`#[non_exhaustive]` types, so existing `match` arms with a `_` and existing field reads
keep compiling.

**Known limitations:** carry forward the ones that still hold from 0.2.0 and 0.2.1:
- no colour-accuracy claim (SPEC-035 is still blocked on the chart);
- the colour develop does not run the `OpcodeList` stages;
- a RATIONAL non-integral `BlackLevel` (Pixel 7 Pro) resolves to 0;
- restart-interval units are not built.

The build confirms each against the code and drops any that no longer hold.

## Release cut is two-phase

- **Phase 1 — reversible prep (this spec's build PR):** semver check, version bump,
  CHANGELOG, release notes, evidence, pre-flight.
- **Phase 2 — irreversible cut (the user):** the tag and the publish, after Phase 1
  merges. The spec ships **"prep-complete, cut-deferred."**

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).**
- `[now]`: verified on the candidate in the build session.
- `[cut]`: only checkable at Phase 2, verified by the user.
- `N/A`: does not apply to a source crate.

- [ ] **1. Version / tag integrity & build provenance** — part `[now]`, the rest `[cut]`.
  - `[now]`:
    - `Cargo.toml` says `version = "0.2.2"`, and `cargo publish --dry-run` reports
      `irradiance v0.2.2`;
    - `git tag -l 'v0.2.2'` is empty, so there is no premature tag;
    - the dry-run's packaged `.cargo_vcs_info.json` carries the candidate's `git.sha1`.
  - `[cut]`, on the merged prep commit:
    - `git tag -a v0.2.2 -m "irradiance 0.2.2"`;
    - `git tag --points-at HEAD` prints exactly `v0.2.2`;
    - after publishing, the `.crate`'s `.cargo_vcs_info.json` sha1 equals the tagged
      commit:
      `curl -sL https://static.crates.io/crates/irradiance/irradiance-0.2.2.crate | tar -xzO irradiance-0.2.2/.cargo_vcs_info.json`.
- [ ] **2. Artifact trust on a clean host** — `N/A`: source crate, no signed binary.
- [ ] **3. Distribution-channel trust** — `[cut]`.
  - `[now]` proxy: `cargo publish --dry-run` builds the packaged crate in isolation, and
    `just package-targets-check` shows no executable.
  - `[cut]`: on a host that never saw the source, `cargo new probe && cd probe && cargo
    add irradiance@0.2.2 && cargo build`. crates.io shows 0.2.2, and the docs.rs build
    is green.
- [ ] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` on the candidate, `[cut]` on the
  published crate.
  - `[now]`: `just test` is green with every corpus file present, and every pin holds.
  - `[cut]`: in the clean-host `probe` crate, parse one held DNG, then run
    `plane::validate`, `unpack_into` and `develop_into`, plus one new 0.2.2 call
    (`plane::units` or `develop_with_into` with `DevelopOptions::default()`). Each
    returns `Ok` with the expected dimensions.
- [ ] **6. Rollback / uninstall** — `[cut]`.
  - **Withdraw:** `cargo yank --version 0.2.2`. Dependents on `"0.2"` fall back to
    0.2.1. It was exercised on 0.1.0 (DEC-028); `cargo yank --undo --version 0.2.2`
    reverses it.
  - **Git rollback** of the prep is a revert of the Phase-1 PR.

## Notes for the Implementer

**Phase 2 runbook (the user, after the Phase-1 PR merges):**
1. `git checkout main && git pull`. The tree is clean, and HEAD is the merged prep
   commit.
2. Run `cargo publish --dry-run` on that exact commit, clean, without `--allow-dirty`.
3. `git tag -a v0.2.2 -m "irradiance 0.2.2"`, then `git push origin v0.2.2`.
4. `cargo publish`, with the user's crates.io token.
5. The `[cut]` halves of pre-flight items 1, 3 and 5.
6. Optional: a GitHub release on `v0.2.2` with the `## Release Notes` text.
7. Append the one-line `## Cut Record` below.

**Gotchas:**
- **The self dev-dependency** (`irradiance = { path = "." … }`) is path-only, so
  `cargo package` strips it. The dry-run confirms it.
- **`cargo semver-checks` needs a rustdoc JSON build of both versions.** It builds
  them in its own target dir and takes a few minutes. Run it with nothing else heavy
  on the machine.
- **Never commit the corpus,** or timing output beyond the numbers in the notes and
  the handback.

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
