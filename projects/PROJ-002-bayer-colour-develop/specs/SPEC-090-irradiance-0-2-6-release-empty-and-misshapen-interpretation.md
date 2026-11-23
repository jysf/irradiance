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
  id: SPEC-090
  type: release                    # epic | story | task | bug | chore | release
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

project:
  id: PROJ-002
  stage: STAGE-030
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-078, DEC-076, DEC-067, DEC-006]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-089, SPEC-088]  # [SPEC-NNN]

value_link: "irradiance 0.2.6 on crates.io carries DEC-078 section 7, so the published crate refuses an empty or misshapen interpretation tag as main does"

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 22000000
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-090: irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

## Context

The release spec of STAGE-030 (PROJ-002). 0.2.5 was published on 2026-11-23 (tag
`v0.2.5` on `c558a4b`). Since then `main` has shipped, to `5a5f079`:
- **SPEC-089** (STAGE-029, PR 319, DEC-078 §7): a present interpretation tag that is
  empty, the wrong length or zero-denominator is malformed, so DEC-078's existing check
  refuses it with `Error::MalformedTag { tag }` before writing. The colour code no longer
  falls back silently from a present matrix it cannot use.
- PRs 317, 318 and 320: project records only.

`git diff v0.2.5 5a5f079 -- src/` touches only `src/ifd.rs` and `src/color.rs`, and
`Cargo.toml` is unchanged. The CHANGELOG's `[Unreleased]` section already has SPEC-089's
entry (under Fixed), written by its build.

This spec ends prep-complete. A cold verify reviews the prep PR. The user then tags and
runs `cargo publish`, as for 0.2.5.

## Release Scope

### Decided for 0.2.6 (design, 2026-11-23)

- **Version / tag:** `0.2.6`, tag `v0.2.6`, a **patch** over the live `0.2.5`. Dependents
  on `irradiance = "0.2"` receive it with `cargo update`.
- **Why it is compatible: the design's mechanical check.** `just semver-check` (crates.io
  baseline, DEC-067, cargo-semver-checks 0.51.0) on `main` at `5a5f079` reports "202
  checks: 202 pass, 58 skip … Summary no semver update required". SPEC-089 adds no public
  item: no `Error` variant (still 76), no `Sensor` field, no `DevelopPath` variant, and
  `Container::scalar` is unchanged except its documentation. New refusals and refusals
  moved to an earlier error shipped as patches in 0.2.4 and 0.2.5. **If the build's
  semver check reports any breaking change, the build stops for the user's decision.**
- **Every input whose result changes since `v0.2.5`.** The design's list follows, derived
  from SPEC-089's CHANGELOG entry and a read of the non-test `src/` diff. The build must
  re-derive it from `git diff v0.2.5 <candidate> -- src/` (every `Err(` path, every new
  check, every reordering) and reconcile the two. A difference is a finding, not an edit
  to make silently. The release-spec template's Release Scope line asks for this.
  - **Output changes:** none for any file that develops. `DEVELOP_VERSION` stays `"8"`.
    SPEC-089's verify found `irr ifd` output identical on all 11 held DNGs.
  - **Newly refused** (as `MalformedTag`, or `UnsupportedTag` for DEC-077's set, on every
    path whose DEC-078 set holds the tag; before anything is written):
    - a **count-zero** `Orientation`, `BlackLevel`, `WhiteLevel`, `CFALayout`,
      `ColorMatrix2`, `ForwardMatrix1/2`, `CameraCalibration2`,
      `CalibrationIlluminant1/2`, `CameraCalibrationSignature`,
      `ProfileCalibrationSignature`, `BlackLevelDeltaH` or `BlackLevelDeltaV`. 0.2.5 read
      each as absent;
    - a **wrong-count** `Orientation`, `CFALayout` or `CalibrationIlluminant1/2` (each must
      be exactly 1). 0.2.5 took the first value;
    - a **wrong-length or zero-denominator** `ColorMatrix2`, `ForwardMatrix1/2` or
      `CameraCalibration2`. 0.2.5 fell back silently: single-illuminant colour for a bad
      `ColorMatrix2`, the ColorMatrix path for a bad `ForwardMatrix1/2`, the identity for a
      bad `CameraCalibration2` it did not apply;
    - an **unused ill-shaped calibration**: SPEC-085's test that such a file develops
      became a test that it refuses (DEC-076 §3 is amended by DEC-078 §7.2).

    A count-zero `Orientation` in one IFD is still recovered from a well-formed one in
    the other, as a malformed one was in 0.2.5.
  - **Already refused, different error:**
    - a misshapen `ColorMatrix1`, `CameraCalibration1/2` or `AnalogBalance`: was
      `MissingColorCalibration` from the colour code, now `MalformedTag { tag }` from
      `check_required_processing`;
    - a count-zero `CFAPlaneColor`: was `UnsupportedCfaPlaneColor`, now
      `MalformedTag { tag: 50710 }`;
    - **precedence:** `check_required_processing` runs before every entry's buffer,
      geometry, options, levels, `GainMap`, CFA and colour checks, so an input 0.2.5
      refused for one of those reasons that also carries a newly malformed tag now reports
      `MalformedTag` (or `UnsupportedTag`). The build names each class it confirms.
  - **Inspection-visible** (`Container::sensor()` still succeeds; no develop needed):
    - `Sensor::malformed_tags` can be longer for the same file;
    - a misshapen colour field (`color_matrix_1/2`, `forward_matrix_1/2`,
      `camera_calibration_1/2`, `analog_balance`) is now `None`, where 0.2.5 kept the
      wrong-length vector;
    - a count-zero `CFAPlaneColor` now reads as the default `[0, 1, 2]` and is listed as
      malformed. **The build confirms this against the code** and corrects the CHANGELOG's
      "left `None`" wording if it does not cover this field;
    - on a plane that is not CFA, ColorPlanes is `SamplesPerPixel`, so a 1-sample
      `LinearRaw` plane now expects 3-entry colour matrices and 1-entry `AnalogBalance`;
      a 9-entry `ColorMatrix1` there is now malformed. The mono develop does not read the
      colour set, so it is unaffected; `to_srgb_linear_into` on such a `Sensor` now
      refuses. **The build confirms this** and says whether any held mono file carries
      colour tags (SPEC-089's verify says none changed).
  - **Composer-visible:** a hand-built or edited `Sensor` that bypasses the reader, with a
    present colour matrix or calibration the colour code cannot use, now gets
    `MissingColorCalibration` from every colour entry and `to_srgb_linear_into`, where
    0.2.5 fell back silently.
- **No held file changes behaviour.** Every held file still develops with the same output,
  and the Pentax K-3 III Monochrome's malformed `BlackLevelRepeatDim` stays exempt.
- **What's included:** the CHANGELOG's `[Unreleased]` section (SPEC-089).
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes.

1. **The changed-input diff** (Release Scope above): reconcile the design's list with
   `git diff v0.2.5 <candidate> -- src/`, confirm or correct the two **confirm** items, and
   record the result in the internal record under Release Notes.
2. **The semver check:** `just semver-check`, recording the tool version and the full
   summary. Any reported breaking change stops the build for the user's decision.
3. **Version:** `Cargo.toml` `version = "0.2.6"`, with every other field unchanged
   (`git diff v0.2.5 -- Cargo.toml` before the bump is empty).
4. **CHANGELOG** (Keep a Changelog): promote `[Unreleased]` to `## [0.2.6] - <date>`, with
   a short lead paragraph like 0.2.5's and a fresh empty `## [Unreleased]` on top.
   **Correct any entry whose wording the changed-input diff contradicts.**
5. **Release notes:** run `just release-notes PROJ-002`, then write `## Release Notes`
   below in 0.2.5's shape (Headline, Fixed, Behaviour changes, Breaking / action required,
   Known limitations, then the internal record), and check them line by line against the
   reconciled changed-input list. Known limitations carry over from 0.2.5 unless this
   release changes one; the ColorChecker ΔE is still pending.
6. **README and crate docs:** check they still describe 0.2.x accurately (`cargo test
   --doc`, `just doc-check`). Wording fixes only.
7. **Tier-B evidence on the candidate SHA,** with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`: every held file present, every pin held.
   - The SDK colour cross-check, with `IRRADIANCE_SDK_COLOUR_DIR` set from a fresh
     `scripts/sdk-colour.sh` run into the scratchpad. The per-file maxima must equal
     0.2.5's 6 / 6 / 1 / 2 / 5.
   - The stage-2 cross-check, with `IRRADIANCE_SDK_STAGE2_DIR` from a fresh
     `scripts/sdk-stage2.sh` run, compared with 0.2.5's table. Every figure should be
     equal; say so or explain.
   - `irr ifd` on every held DNG, diffed against the same command built at `v0.2.5`
     (a worktree of the tag). Expected identical.
   - The three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded against 0.2.5's figures.
8. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`,
   `cargo publish --dry-run`. The package list must be the same files as 0.2.5's.
9. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), using SPEC-088's
   filled pre-flight as the model. Then run every gate: `just test`, `lint-ci`,
   `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`,
   `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
   `semver-check`, `package-targets-check`.

## Release Notes

*Written by the build (step 5). Outward-facing, for whoever uses the crate; then the
internal record (the changed-input reconciliation, the semver summary, the SDK and
timing figures). The GitHub release text for `v0.2.6` is this section, down to the
internal record.*

**Headline:** <the build writes it: a malformed interpretation tag now refuses even when
it reads cleanly with the wrong shape>

**Breaking / action required:** <the build writes it from the semver check>

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).** Filled by
the Phase-1 build on the candidate. `[now]`: verified on the candidate in the build
session. `[cut]`: verified by the user at Phase 2. `N/A`: does not apply.

- [ ] **1. Version / tag integrity & build provenance** — `[now]` + `[cut]`.
  - `[now]`: `Cargo.toml` says `0.2.6`; `cargo publish --dry-run` on a clean detached
    worktree of the candidate (no `--allow-dirty`); `git tag -l 'v0.2.6'` empty; the
    packaged `.cargo_vcs_info.json` sha1 equals the candidate; `Cargo.toml` otherwise
    unchanged since `v0.2.5`.
  - `[cut]`: `git tag -a v0.2.6 -m "irradiance 0.2.6"` on the merged prep commit after
    confirming its `Cargo.toml` shows 0.2.6; `git tag --points-at HEAD` prints exactly
    `v0.2.6`; the published `.crate`'s `.cargo_vcs_info.json` sha1 equals
    `git rev-parse v0.2.6^{commit}`.
- [x] **2. Artifact trust on a clean host** — `N/A`: a source crate, no signed binary.
- [ ] **3. Distribution-channel trust** — `[now]` proxy + `[cut]`.
  - `[now]`: the dry run packages and verifies in isolation; `just package-targets-check`,
    `just deny`, `just deny-fuzz`; `cargo package --list` identical to `v0.2.5`'s.
  - `[cut]`: crates.io shows 0.2.6, docs.rs is green.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` on the candidate, `[cut]` on the
  published crate.
  - `[now]`: `just test` with the corpus and both SDK lanes set; `just doc-check`.
  - `[cut]`: in a clean-host probe crate on `irradiance = "=0.2.6"`:
    - `develop_color_into` on the Pixel 7 Pro and `develop_into` on the Pentax K-3 III
      Monochrome return `Ok`, with output hashes equal to the same program built
      against `main`;
    - on the Canon EOS 5D Mark III's parsed `Sensor` (dual-illuminant), replace
      `color_matrix_2` with its first 8 entries. `develop_color_into` returns
      `Error::MissingColorCalibration` and writes nothing. (0.2.5 developed it as
      single-illuminant colour, so this tells the two releases apart. The reader is
      bypassed, so the error is the colour code's §7.2 one, not `MalformedTag`.)
    - `irradiance::DEVELOP_VERSION == "8"`.
- [ ] **6. Rollback / uninstall** — `[cut]`. Withdraw with `cargo yank --version 0.2.6`
  (dependents on `"0.2"` fall back to 0.2.5; `cargo yank --undo` reverses it). Git
  rollback: revert the Phase-1 PR.

## Notes for the Implementer

- The SDK lanes are local only (DEC-069 §6): the SDK, its outputs and the corpus are never
  committed or packaged. Run both scripts into a scratchpad outside the repository. Set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`; `just test`
  already passes `--all-features`.
- A defect found during prep is reported in the handback, not fixed here: the stage is
  release-only.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).
- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, with a clean tree; confirm `Cargo.toml` shows
     `version = "0.2.6"`.
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.6 -m "irradiance 0.2.6"`, then `git push origin v0.2.6`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5 (the orchestrator can run the
     clean-host crate), reported back for the Cut Record.
  6. Optional: a GitHub release on `v0.2.6` with the `## Release Notes` text, down to
     the internal record.

## Cut Record

*One line, appended after Phase 2.*

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
