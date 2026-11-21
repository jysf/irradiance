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
  id: SPEC-083
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
  stage: STAGE-026
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-070, DEC-067, DEC-052]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-077, SPEC-078, SPEC-079, SPEC-080]  # [SPEC-NNN]

value_link: "Puts STAGE-025's DNG-conformant black normalisation and SPEC-080's colour levels refusal on crates.io."

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

# SPEC-083: irradiance 0.2.4 release: DNG-conformant black normalisation and the colour levels refusal

## Context

The release spec of STAGE-026, the last irradiance release in PROJ-004. 0.2.3 was
published on 2026-11-20 (tag `v0.2.3` on `61a2671`). Since then `main` has shipped:
- **STAGE-025:** SPEC-078 (DEC-070, black normalisation by white minus the maximum black,
  `DEVELOP_VERSION` 7) and SPEC-079 (SDK recipe and cross-check hardening, no library
  change);
- **STAGE-026:** SPEC-080 (the colour develop refuses `W ≤ B` with `InvalidLevels`, plus
  two tooling fixes).

This spec ends prep-complete. The user tags and runs `cargo publish`, as for 0.2.3
(SPEC-077).

## Release Scope

### Decided for 0.2.4 (design, 2026-11-21)

- **Version / tag:** `0.2.4`, tag `v0.2.4`, a **patch** over the live `0.2.3`. Dependents on
  `irradiance = "0.2"` receive it with `cargo update`.
- **Why it is compatible: the design's mechanical check.** `just semver-check` (crates.io
  baseline, DEC-067) on `main` at `cc1df3c` (the PR 285 merge) reports "202
  checks: 202 pass, 58 skip … no semver update required". No public item was added,
  removed or changed between `v0.2.3` and `main`; the diff touches only `src/color.rs`,
  `src/develop.rs` and doc comments in `src/lib.rs`.
- **Every input whose result changes since `v0.2.3`** (STAGE-024's lesson: semver-checks
  cannot see a newly refused input, so the design diffed every `Err(` path in
  `git diff v0.2.3 origin/main -- src/`):
  - **Newly refused, one input:** a colour develop whose white (the first `WhiteLevel`
    value) is at or below the largest black the plane applies. 0.2.3 developed it to an
    all-zero image; 0.2.4 returns `Error::InvalidLevels { black_level, white_level }`
    before any write (SPEC-080).
  - **Same refusal, different field value:** mono `BlackLevels::PerPosition` refused the
    same inputs in 0.2.3 (any black at or above white), but `InvalidLevels::black_level`
    now reports the **largest** black instead of the first offending one (SPEC-078).
  - **Output changes:** colour develops and mono `PerPosition` develops of a file whose
    blacks differ by position now scale by white minus the largest black. In the held
    corpus: the Canon EOS 5D Mark III, Pixel 3a and Pixel 4a on those paths. Every
    uniform-black file and every mono default (`FirstValue`) develop is byte-identical.
  - **`DEVELOP_VERSION` goes from `"6"` to `"7"`** (SPEC-078; SPEC-080 kept it at 7).
- **Wording caution (SPEC-080's verify):** colour now refuses what mono `PerPosition`
  refuses. Mono's default `FirstValue` checks its first black, not the largest, so don't
  claim mono and colour refuse identically.
- **What's included:** the CHANGELOG's `[Unreleased]` section as it stands (two entries).
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes.

1. **The two script fixes from SPEC-080's verify** (scripts only, not packaged):
   - `scripts/validate.sh`: strip a trailing CR and surrounding single or double quotes
     from a feedback `status` before the vocabulary check, and treat a leading UTF-8 BOM
     before `---` as front matter (or refuse it loudly; say which). Show each on a
     temporary copy, without committing the broken files.
   - `scripts/sdk-stage2.sh`: a worktree directory that exists but cannot be entered must
     produce a message, not a silent exit under `set -e`. Show it on a scratch case
     outside every checkout.
2. **The semver check:** `just semver-check`, recording the tool version and the full
   summary. Any reported breaking change stops the build for the user's decision.
3. **Version:** `Cargo.toml` `version = "0.2.4"`, with every other field unchanged from
   0.2.3 (`git diff v0.2.3 -- Cargo.toml` before the bump is empty).
4. **CHANGELOG** (Keep a Changelog, as for 0.2.3): promote `[Unreleased]` to `## [0.2.4] - <date>` with a short
   lead paragraph like 0.2.3's, keeping both entries; a fresh empty `## [Unreleased]` on
   top.
5. **Release notes:** run `just release-notes PROJ-004`, edit into `## Release Notes`
   below, and check them against the Release Scope list above: every changed input named,
   the wording caution respected.
6. **README and crate docs:** check they still describe 0.2.x accurately (`cargo test
   --doc`, `just doc-check`); wording fixes only.
7. **Tier-B evidence on the candidate SHA**, with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`: all 12 held files present, every pin held;
   - the SDK cross-check, with `IRRADIANCE_SDK_STAGE2_DIR` set from a fresh
     `scripts/sdk-stage2.sh` run into the scratchpad: record the no-gain and end-to-end
     numbers (they must equal SPEC-078's);
   - the three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded.
8. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`,
   `cargo publish --dry-run`. The package list must be the same 19 files as 0.2.3: no
   `scripts/`, `tests/`, SDK or corpus bytes.
9. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), then run every gate:
   `just test`, `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`,
   `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit`, `name-length-check`,
   `decisions-index --check`, `semver-check`, `package-targets-check`.

## Release Notes

*Outward-facing, written for whoever uses the crate. Seeded at build with
`just release-notes PROJ-004`. Three of its notes are new since 0.2.3 (SPEC-078, SPEC-079,
SPEC-080); SPEC-079 has no crate-user outcome and stays absent. Edited against the final
CHANGELOG, SPEC ids dropped, and checked line by line against the Release Scope's "every
input whose result changes" list. The GitHub release text for `v0.2.4` is this section,
down to the internal record.*

**Headline:** Black levels now follow the DNG specification and Adobe's SDK. On a file
whose black level differs by position, irradiance scales by white minus the largest black,
as DNG 1.7.1 defines, and on the three held files with such blacks (the Canon EOS 5D
Mark III, Pixel 3a and Pixel 4a) it matches Adobe `dng_validate` to within one code before
the `GainMap`. A malformed colour file whose white is at or below its black is refused
instead of developing to black. A compatible 0.2.x release: run `cargo update -p
irradiance`.

**Changed**
- **Black normalisation scales by white minus the largest black.** On the colour develop
  (`develop_color_into`, `develop_color_f32_into` and both threaded entries) and on mono
  `BlackLevels::PerPosition`, each sample still subtracts its own position's black, but
  now scales by `WhiteLevel` minus the largest black the path applies, clipping there,
  instead of by `WhiteLevel` minus its own black. This is the Rescaling step of the DNG
  1.7.1 specification and what Adobe's SDK computes. Against `dng_validate` stage 2
  before the `GainMap`, the largest gap falls from 6 codes to 1 on the Canon (69.4% to
  99.6% exact) and from 5 codes to 1 on the Pixel 3a and 4a. With the `GainMap`
  applied, it falls from 6 codes to 4 on the 3a and from 7 to 4 on the 4a.

**Behaviour changes**
- **`DEVELOP_VERSION` goes from `"6"` to `"7"`,** so a pixel cache keyed on it
  re-develops.
- **Output changes only for a file whose blacks differ by position, on the colour and
  mono `PerPosition` paths.** In the held corpus that is the Canon EOS 5D Mark III, the
  Pixel 3a and the Pixel 4a on those paths. Every uniform-black file, and every mono
  default (`BlackLevels::FirstValue`) develop of any file, is byte-identical to 0.2.3.
- **A colour develop whose white is at or below the largest black is now refused.**
  All four colour entries return `Error::InvalidLevels { black_level, white_level }`
  when the white (the first `WhiteLevel` value) is at or below the largest black the
  plane applies, for a scalar `BlackLevel` and for any black pattern, before anything is
  written. 0.2.3 developed such a file to an all-zero image without an error. Colour now
  refuses what mono `PerPosition` refuses; the mono default `FirstValue` checks its
  first black only, as before. No held file is affected.
- **Mono `PerPosition`'s `InvalidLevels` reports the largest black.** It refuses the
  same inputs as 0.2.3 (any black at or above white), but `black_level` now holds the
  largest black in the pattern instead of the first offending one.

**Breaking / action required:** none for the API. `cargo semver-checks` against crates.io
0.2.3 reports no semver update required. No public item was added, removed or changed,
there is no new dependency or feature, and the MSRV stays 1.90. A caller that matches
`InvalidLevels` on an exact `black_level` from a mono `PerPosition` develop should read
the largest black there.

**Known limitations**
- **No colour-accuracy claim yet.** The ColorChecker ΔE2000 measurement is still blocked
  on capturing the chart. Treat colour as plausible, not calibrated.
- **`BlackLevelDeltaH` and `BlackLevelDeltaV` are not read.** The specification adds them
  to the black before taking the largest; no held file carries them.
- **The Pixel 7 Pro still differs from the SDK by up to 2 codes before the `GainMap`,**
  because irradiance rounds its fractional black to the nearest integer.
- **The colour develop runs the `GainMap` but not the other `OpcodeList` stages**
  (`WarpRectilinear`, `FixBadPixelsConstant`) that the monochrome develop runs. A file
  that requires them is refused with a typed error. The develop options are mono-only.
- **A `GainMap` in `OpcodeList1` or `OpcodeList3`** is recognised but not applied; a
  mandatory one is refused with `UnsupportedRequiredOpcode`.
- **A Compression 7 strip is one plane unit.** Restart-interval units are not built, so
  a single-scan lossless-JPEG file (the K-3 III, the Typ 246) cannot be split.

### Internal release record (not part of the GitHub release text)

- **Mechanical semver check:** `cargo-semver-checks 0.51.0`, crates.io 0.2.3 to the
  candidate `04593ce`, by `just semver-check` (DEC-067). Clean: no failure, nothing to
  accept. Verbatim summary, run on the candidate after the version bump:

  ```
  Checking irradiance v0.2.3 -> v0.2.4 (minor change)
   Checked [   0.010s] 202 checks: 202 pass, 58 skip
   Summary no semver update required
  ```

  Before the bump (at `46e5858`) the same run, and
  `cargo semver-checks check-release --baseline-version 0.2.3 --all-features`, both gave
  202 pass, 58 skip, no semver update required. As for 0.2.3, the tool labels a 0.x
  patch bump a "minor change"; no update is required.
- **Candidate timings** (`04593ce`, stamped `v0.2.3-15-g04593ce-dirty`; the only dirty
  file was this spec. Apple M4 Pro, release, medians in ms; all within a few percent of
  0.2.3's):
  - `just unpack-timing`: Q2M `unpack_into` on 1 thread 9.9; 8 units on 8 threads 1.4.
  - `just ljpeg-timing`: K-3 III 100.5; Typ 246 87.3; Pixel 4a 58.8.
  - `just orient-timing`, warp-free: O1 85, O5 98, O6 114, O7 143, O8 143.
  - `just orient-timing`, post-warp: O1 570, O5 610, O8 609.
- **SDK cross-check** (run): a fresh `scripts/sdk-stage2.sh` with the local
  `dng_validate` 1.7.1 into the build's scratchpad wrote 7 TIFFs whose sha256s equal
  SPEC-079's table. With `IRRADIANCE_SDK_STAGE2_DIR` set, `just test` ran the
  cross-check, and `--test gain_map_corpus -- --nocapture` (11 passed) printed the
  numbers below. Δ is SDK minus ours. Every figure equals SPEC-078's verify table
  (HANDOFF-168) digit for digit.

  | Comparison | File | max \|Δ\| | Exact |
  |---|---|---|---|
  | No gain | Canon 5D Mark III | 1, Δ ∈ {0, +1} | 99.5663% |
  | No gain | Pixel 4a | 1, Δ ∈ {0, +1} | 99.7911% |
  | No gain | Pixel 3a | 1, Δ ∈ {0, +1} | 99.8536% |
  | No gain | Pixel 7 Pro | 2 | 28.3471% |
  | End to end | Pixel 4a | 4 | 99.7729% |
  | End to end | Pixel 3a | 4 | 99.8210% |
  | End to end | Pixel 7 Pro | 4 | 28.3394% |
  | Gain step | Pixel 4a / 3a / 7 Pro | 1 / 1 / 1 | 99.9794% / 99.9672% / 99.9664% |
- **Release notes against the Release Scope list:** the newly refused colour input,
  the mono `PerPosition` `black_level` field change, the output changes (Canon, 3a, 4a
  on the colour and `PerPosition` paths; uniform-black and `FirstValue` byte-identical)
  and `DEVELOP_VERSION` 6 to 7 each have a line under Behaviour changes or Changed. The
  wording caution holds: the notes say colour refuses what mono `PerPosition` refuses and
  that `FirstValue` checks its first black only. The CHANGELOG's SPEC-080 entry said "as
  the mono path already did"; it now names mono `BlackLevels::PerPosition`.

## Release cut is two-phase

- **Phase 1 — reversible prep (this spec's build PR):** the script fixes, semver check,
  version bump, CHANGELOG, release notes, evidence, pre-flight.
- **Phase 2 — irreversible cut (the user):** the tag and the publish, after Phase 1
  merges. The spec ships **"prep-complete, cut-deferred."**

## Release Pre-Flight

**Delivery shape: library plus a crates.io package (source crate, no binary).** Filled
2026-11-20 by the Phase-1 build (HANDOFF-173) on candidate `04593ce`
(`04593ce3044b0bd62878bfb0fbee3c3914d21961`).
- `[now]`: verified on the candidate in the build session.
- `[cut]`: verified by the user at Phase 2.
- `N/A`: does not apply.

- [ ] **1. Version / tag integrity & build provenance** — `[now]` done, the rest `[cut]`.
  - `[now]` ✔:
    - `Cargo.toml` says `version = "0.2.4"`, and `cargo publish --dry-run` (clean tree,
      no `--allow-dirty`, in a detached scratch worktree of `04593ce`) reports
      `Packaged 19 files, 1.8MiB (374.7KiB compressed)` and `Uploading irradiance v0.2.4`
      then "aborting upload due to dry run";
    - `git tag -l 'v0.2.4'` is empty. The timing examples stamp the candidate as
      `v0.2.3-15-g04593ce`;
    - the dry run's packaged `.cargo_vcs_info.json` carries `git.sha1`
      `04593ce3044b0bd62878bfb0fbee3c3914d21961`, the candidate;
    - `git diff v0.2.3 -- Cargo.toml` before the bump was empty, and the bump changes
      only the version line, so `license`, `description`, `keywords`, `categories`,
      `repository`, `include` and `rust-version = "1.90"` are unchanged.
  - `[cut]`:
    - `git tag -a v0.2.4 -m "irradiance 0.2.4"` on the merged prep commit;
    - `git tag --points-at HEAD` prints exactly `v0.2.4`;
    - the published `.crate`'s `.cargo_vcs_info.json` sha1 equals the tagged commit:
      `curl -sL https://static.crates.io/crates/irradiance/irradiance-0.2.4.crate | tar -xzO irradiance-0.2.4/.cargo_vcs_info.json`
      against `git rev-parse v0.2.4^{commit}`.
- [x] **2. Artifact trust on a clean host** — `N/A`: a source crate, no signed binary.
- [ ] **3. Distribution-channel trust** — `[now]` proxy done, the rest `[cut]`.
  - `[now]` ✔:
    - `cargo publish --dry-run` packaged and verified the crate in isolation. Its only
      warnings are the expected "ignoring example/test … not included in the published
      package" lines, as in 0.2.3;
    - the packaged `Cargo.toml` has no self dev-dependency (its only dev-dependencies are
      `ssimulacra2` and `toml`);
    - `just package-targets-check` passed (no executable in the default build or the
      package, `irr` builds under `--features irr`, its red case caught); `just deny` and
      `just deny-fuzz` both report `licenses ok`;
    - the package list is the same 19 files as the published 0.2.3 `.crate` (its
      `tar -tz` list, diffed with the version renamed, is identical): `Cargo.toml`,
      `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json`, `README.md`, the two
      licences and `src/**/*.rs`. No `scripts/`, `tests/`, SDK or corpus bytes.
  - `[cut]`: on a clean host, `cargo add irradiance@=0.2.4 && cargo build`. crates.io
    shows 0.2.4, and docs.rs is green.
- [x] **4. Data isolation** — `N/A`: a library with no I/O and no data store.
- [ ] **5. Runtime smoke on a clean host** — `[now]` done on the candidate, `[cut]` on the
  published crate.
  - `[now]` ✔:
    - `just test`, with `IRRADIANCE_CORPUS_DIR` and `IRRADIANCE_SDK_STAGE2_DIR` set,
      printed "corpus: 12/12 present — no tier-B test will skip" and "sdk-stage2:
      present … the dng_validate cross-check will run". 46 test results plus the
      doctests: 790 passed, 0 failed, 1 ignored, exit 0, in 204 s. Every `dnglab`
      checksum pin and every develop MD5 pin held;
    - `cargo test --doc --all-features` (inside `just test`): 4 passed;
    - `just doc-check`: 2 feature surfaces, 0 rustdoc warnings.
  - `[cut]`: in a clean-host probe crate:
    - develop the Canon EOS 5D Mark III and the Pixel 4a with `develop_color_into`; each
      returns `Ok` with the expected dimensions;
    - a hand-built colour sensor with white at or below its black returns
      `Error::InvalidLevels`.
- [ ] **6. Rollback / uninstall** — `[cut]`.
  - **Withdraw:** `cargo yank --version 0.2.4`, and dependents on `"0.2"` fall back to
    0.2.3. `cargo yank --undo --version 0.2.4` reverses it.
  - **Git rollback:** revert the Phase-1 PR.

**Gates on the candidate, all green** (2026-11-20, `IRRADIANCE_CORPUS_DIR` set):
- `just test` (above)
- `lint-ci`
- `lint-no-allow`
- `lint-fuzz`
- `fmt-check`
- `validate`
- `doc-check`
- `deny`, plus `deny-fuzz`
- `msrv` (1.90.0)
- `cost-audit`
- `gate-audit`
- `name-length-check`
- `decisions-index --check`
- `semver-check` (202 of 202)
- `package-targets-check`

## Notes for the Implementer

- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, with a clean tree.
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.4 -m "irradiance 0.2.4"`, then `git push origin v0.2.4`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5, and report them back so the Cut
     Record can say they ran (0.2.3's were not reported).
  6. Optional: a GitHub release on `v0.2.4` with the `## Release Notes` text.
- **Open PR 284** (PROJ-005's writer re-frame) adds DEC-071 to DEC-073 and SPEC-081/082.
  If it merges before this build, `decisions/INDEX.md` changes on `main`; rebase, don't
  hand-merge the index (`just decisions-index`).
- **Plain-prose commit bodies**, no code tokens. Never `git add Cargo.lock`. Stage
  explicit paths. Never commit corpus or SDK bytes.

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
