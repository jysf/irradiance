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
  id: SPEC-077
  type: release                    # epic | story | task | bug | chore | release
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

project:
  id: PROJ-004
  stage: STAGE-024
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-006, DEC-052, DEC-067, DEC-068, DEC-069]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-073, SPEC-074, SPEC-075, SPEC-076, SPEC-059, SPEC-060]  # [SPEC-NNN]

value_link: null

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 15000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-077: irradiance 0.2.3 release, GainMap and black-level fixes

## Context

The only spec of STAGE-024. 0.2.2 was published on 2026-11-17. Since then main has
shipped three stages, all unreleased:
- **STAGE-022:** one Compression 7 header parse per chunk, and repo tooling.
- **STAGE-023:** fractional `BlackLevel` rounding, the colour `BlackLevelRepeatDim`
  indexing, and `Sensor::masked_areas` / `black_levels_exact`.
- **STAGE-016:** `OpcodeList2` `GainMap` on both develop paths. The Pixel 3a, 4a and
  7 Pro now develop, and the public `apply_gain_maps` is new.

The user asked to cut a 0.2.x release on 2026-11-19. This spec ends prep-complete. The
user tags and runs `cargo publish`, as with 0.2.2 (SPEC-073).

## Release Scope

### Decided for 0.2.3 (design, 2026-11-19; the user asked for 0.2.x)

- **Version / tag:** `0.2.3`, semver, tag `v0.2.3`. It is a **patch** bump over the live
  `0.2.2`. Dependents on `irradiance = "0.2"` receive it with `cargo update`.
- **Why it is compatible: the design's mechanical check.** `cargo semver-checks
  check-release --baseline-version 0.2.2 --all-features` was run at `b515520` and reports
  "202 checks: 202 pass, 58 skip … no semver update required". The new `Error` variants
  (`InvalidBlackLevelPattern`, `TooManyGainMaps`, `GainMapImageWrongLength`), the new
  `Opcode::GainMap`, and the new `Sensor` fields all sit on `#[non_exhaustive]` types.
  Every addition is appended last.
- **Behaviour changes, called out in the notes (they don't break the API):**
  - **`DEVELOP_VERSION` goes from `"4"` to `"6"`.** Callers who cache developed pixels
    keyed on it re-develop. No held file's output changed.
  - **Output changes only for files 0.2.2 got wrong or refused:**
    - a fractional `BlackLevel` (the Pixel 7 Pro was black 0 and is now 1024);
    - a colour file with a `[1, 2]`, `[2, 1]` or larger black pattern;
    - a file with optional `GainMap`s, which are now applied;
    - the Pixel 3a, 4a and 7 Pro, which were refused and now develop.
  - **A colour develop now refuses a `BlackLevel` count that disagrees with
    `BlackLevelRepeatDim`,** with `Error::InvalidBlackLevelPattern`, where 0.2.2 guessed.
    This matches the mono `PerPosition` path and DEC-068 §2. A file that malformed
    developed with a wrong black before. Treated as a fix, not a break.
- **What's included** (on `main` since `0.2.2`): STAGE-022, STAGE-023 and STAGE-016,
  which is the CHANGELOG's `[Unreleased]` section as it stands.
- **Delivery shape:** library plus a crates.io **package**. There is no binary, so the
  binary and service pre-flight categories are `N/A`.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

Everything here lands as one CI-gated PR, and it is fully revertible. The implementer
never tags or publishes.

1. **The semver check:** `just semver-check` (DEC-067: the crates.io baseline).
   - Record the tool's version and its full summary.
   - **Any reported breaking change stops the build.** The release then needs the
     user's decision, not a workaround.
2. **Version:** `Cargo.toml` `version = "0.2.3"`. Confirm `license`, `description`,
   `keywords`, `categories`, `repository`, `include` and `rust-version` are unchanged
   from 0.2.2.
3. **CHANGELOG** (DEC-006):
   - Promote `[Unreleased]` to `## [0.2.3] - <date>`, keeping every entry.
   - Put a fresh, empty `## [Unreleased]` on top.
   - Tidy the grouping only where entries for one change are split across sections
     (the two `DEVELOP_VERSION` lines become one: 4 → 6).
4. **Release notes:** run `just release-notes PROJ-004`, then edit the result into
   `## Release Notes` below. Keep the drafted headline and the "Behaviour changes"
   block.
5. **README and crate docs:**
   - Check that they still describe 0.2.x accurately, using `cargo test --doc` and
     `just doc-check`.
   - Fix any wording that is now wrong. For example, a claim that a mandatory opcode is
     always refused, or a supported-files list missing the Pixel phones.
   - Keep the diff to wording.
6. **Tier-B evidence run on the candidate SHA**, with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`, about 3 min. Record that all 12 held files were present and every pin
     held.
   - The three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded.
   - Optional: the SDK cross-check, with `IRRADIANCE_SDK_STAGE2_DIR` set from
     `scripts/sdk-stage2.sh`. Record it as extra evidence if it is run.
7. **Package guards:**
   - `just package-targets-check`;
   - `just deny` and `just deny-fuzz`;
   - `cargo publish --dry-run`.

   All must be clean. The packaged manifest must not contain the self dev-dependency.
   The package must not include `scripts/`, `tests/` fixtures beyond what `include`
   already allows, or any SDK or corpus bytes.
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
   - `name-length-check`
   - `semver-check`

## Release Notes

*Outward-facing, written for whoever uses the crate. Seeded at build with
`just release-notes PROJ-004`. Five of its notes are new since 0.2.2 (SPEC-059, SPEC-060,
SPEC-074, SPEC-075, SPEC-076); SPEC-060 and SPEC-076 have no crate-user outcome and stay
absent. Edited against the final CHANGELOG, SPEC ids dropped. The GitHub release text for
`v0.2.3` is this section, down to the internal record.*

**Headline:** Google Pixel DNGs develop. irradiance now applies the `OpcodeList2`
`GainMap` (lens-shading correction) that Android phones write as mandatory, and it
matches Adobe's DNG SDK within one code. Black levels are read correctly in two more
cases. A compatible 0.2.x release: run `cargo update -p irradiance`.

**New capability**
- **Pixel 3a, 4a and 7 Pro DNGs develop** in colour and in mono. They were refused before
  with `UnsupportedMandatoryOpcode { id: 9 }`. Each `GainMap` is applied at stage 2,
  after the black and white levels and before the lens warp (mono) or demosaic (colour),
  sampled at pixel centres with bilinear interpolation, and clipped at white. On all
  three files, the step matches Adobe `dng_validate` 1.7.1 to within 1 code, with at
  least 99.966% of samples exact.
- **`develop::apply_gain_maps`** applies every `GainMap` in an `OpcodeList2` to a
  normalised stage-2 image, for callers that compose stages themselves.
- **`Sensor::black_levels_exact`** holds a `RATIONAL` `BlackLevel` exactly as written.
  **`Sensor::masked_areas`** exposes DNG `MaskedAreas` (the optical-black rectangles)
  verbatim, for optical-black work; irradiance itself does not use it.

**Fixed**
- **A fractional `RATIONAL` `BlackLevel` rounds to nearest** instead of being treated as
  malformed and becoming 0. The Pixel 7 Pro now reads 1024.
- **The colour develop honours `BlackLevelRepeatDim`** (`[1, 2]`, `[2, 1]`, `[4, 4]`, …)
  instead of assuming a 2×2 pattern.

**Faster**
- **Compression 7 strips and tiles parse their lossless-JPEG header once,** which halves
  the per-chunk allocations. Output is unchanged.

**Breaking / action required:** none for the API. `cargo semver-checks` against crates.io
0.2.2 reports no semver update required. The new `Error` variants (`TooManyGainMaps`,
`GainMapImageWrongLength`), `Opcode::GainMap` and the new `Sensor` fields are appended to
`#[non_exhaustive]` types, so a `match` with a `_` arm and existing field reads keep compiling.

**Behaviour changes**
- **`DEVELOP_VERSION` goes from `"4"` to `"6"`,** so a pixel cache keyed on it
  re-develops.
- **A colour file whose `BlackLevel` count disagrees with its `BlackLevelRepeatDim` is
  now refused** with `Error::InvalidBlackLevelPattern`, before any write, instead of
  developing with a guessed black. The mono `PerPosition` develop already did this.
- **Output changes only for files 0.2.2 got wrong or refused:** a fractional black, a
  colour black pattern other than 2×2, optional `GainMap`s (now applied), and the Pixel
  3a, 4a and 7 Pro (now developed). Every other file's output is byte-identical: every
  `dnglab` checksum and develop MD5 pin from 0.2.2 holds.

**Known limitations**
- **No colour-accuracy claim yet.** The ColorChecker ΔE2000 measurement is still blocked
  on capturing the chart. Treat colour as plausible, not calibrated.
- **The colour develop runs the `GainMap` but not the other `OpcodeList` stages**
  (`WarpRectilinear`, `FixBadPixelsConstant`) that the monochrome develop runs. A file
  that requires them is refused with a typed error. The develop options are mono-only.
- **A `GainMap` in `OpcodeList1` or `OpcodeList3`** is recognised but not applied; a
  mandatory one is refused with `UnsupportedRequiredOpcode`.
- **A Compression 7 strip is one plane unit.** Restart-interval units are not built, so
  a single-scan lossless-JPEG file (the K-3 III, the Typ 246) cannot be split.

### Internal release record (not part of the GitHub release text)

- **Mechanical semver check:** see the Release Pre-Flight's semver block below.
- **Candidate timings:** see the Release Pre-Flight, item 5.

## Release cut is two-phase

- **Phase 1 — reversible prep (this spec's build PR):** semver check, version bump,
  CHANGELOG, release notes, evidence, pre-flight.
- **Phase 2 — irreversible cut (the user):** the tag and the publish, after Phase 1
  merges. The spec ships **"prep-complete, cut-deferred."**

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).** Filled by
the Phase-1 build on its candidate SHA.
- `[now]`: verified on the candidate in the build session.
- `[cut]`: verified by the user at Phase 2.
- `N/A`: does not apply.

- [ ] **1. Version / tag integrity & build provenance.**
  - `[now]`:
    - `version = "0.2.3"`;
    - the dry run reports `Packaging irradiance v0.2.3`;
    - `git tag -l 'v0.2.3'` is empty;
    - the dry run's `.cargo_vcs_info.json` sha1 is the candidate;
    - the metadata is unchanged from 0.2.2.
  - `[cut]`:
    - `git tag -a v0.2.3 -m "irradiance 0.2.3"` on the merged prep commit;
    - `git tag --points-at HEAD` prints exactly `v0.2.3`;
    - the published `.crate`'s sha1 equals the tagged commit.
- [x] **2. Artifact trust on a clean host** — `N/A`: a source crate, no signed binary.
- [ ] **3. Distribution-channel trust.**
  - `[now]`:
    - `cargo publish --dry-run` is clean;
    - the packaged manifest has no self dev-dependency;
    - `package-targets-check`, `deny` and `deny-fuzz` pass;
    - no SDK, corpus or script bytes are in the package list.
  - `[cut]`: on a clean host, `cargo add irradiance@=0.2.3 && cargo build`. crates.io
    shows 0.2.3, and docs.rs is green.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host.**
  - `[now]`:
    - `just test` with the corpus: 12/12 present, every pin held, with counts and time;
    - `cargo test --doc --all-features`;
    - `just doc-check`.
  - `[cut]`: in a clean-host probe crate:
    - parse one held DNG, then run `plane::validate`, `unpack_into` and `develop_into`;
    - develop the Pixel 4a with `develop_color_into`, the 0.2.3 headline;
    - each returns `Ok` with the expected dimensions.
- [ ] **6. Rollback / uninstall** — `[cut]`.
  - **Withdraw:** `cargo yank --version 0.2.3`, and dependents on `"0.2"` fall back to
    0.2.2. `cargo yank --undo --version 0.2.3` reverses it.
  - **Git rollback:** revert the Phase-1 PR.

## Notes for the Implementer

- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, with a clean tree.
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.3 -m "irradiance 0.2.3"`, then `git push origin v0.2.3`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5.
  6. Optional: a GitHub release on `v0.2.3` with the `## Release Notes` text.
  7. Append the one-line `## Cut Record`.
- **`just semver-check`** uses the crates.io baseline. A `--baseline-rev` checkout fails
  on legacy long paths in old trees (DEC-067).
- **Plain-prose commit bodies**, with no code tokens. `Cargo.lock` is gitignored; never
  force-add it. Never commit corpus or SDK bytes.
- **0.2.2's build blocked once on the semver check** (a lint about mid-enum variants).
  Since then new variants are appended last, and the design's run is clean. If it
  reports anything, stop and record it as 0.2.2 did.

## Cut Record

*Filled at Phase 2.*

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
