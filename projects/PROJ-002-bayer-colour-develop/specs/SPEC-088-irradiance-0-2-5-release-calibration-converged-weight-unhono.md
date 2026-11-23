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
  id: SPEC-088
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
  stage: STAGE-028
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-067, DEC-006, DEC-076, DEC-077, DEC-078]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-083, SPEC-085, SPEC-086, SPEC-087]  # [SPEC-NNN]

value_link: "irradiance 0.2.5 on crates.io carries STAGE-027 and PR 301, so the published crate matches what main develops and refuses"

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

# SPEC-088: irradiance 0.2.5 release: calibration, converged weight, unhonoured and malformed tag refusals

## Context

The release spec of STAGE-028 (PROJ-002). 0.2.4 was published on 2026-11-21 (tag `v0.2.4`
on `ef37e6b`). Since then `main` has shipped, 28 commits to `5a8eb61`:
- **PR 301** (2026-11-21, no spec): the Codex review fixes. Non-finite warp coefficients,
  staggered `CFALayout`, `CFAPlaneColor`, the opcode version guard, and the new public
  `PlaneDecoder`. **It added no CHANGELOG entry.**
- **PR 302:** docs only.
- **STAGE-027:** SPEC-085 (DEC-076, the camera calibration and the converged illuminant
  weight, `DEVELOP_VERSION` 8), SPEC-086 (DEC-077, refusing tags that change what the
  samples mean, a newer DNG backward version, and a Compression 7 precision mismatch) and
  SPEC-087 (DEC-078, refusing a malformed tag the develop path reads).

This spec ends prep-complete. The user tags and runs `cargo publish`, as for 0.2.4
(SPEC-083).

## Release Scope

### Decided for 0.2.5 (design, 2026-11-22)

- **Version / tag:** `0.2.5`, tag `v0.2.5`, a **patch** over the live `0.2.4`. Dependents
  on `irradiance = "0.2"` receive it with `cargo update`.
- **Why it is compatible: the design's mechanical check.** `just semver-check` (crates.io
  baseline, DEC-067) on `main` at `5a8eb61` reports "202 checks: 202 pass, 58 skip …
  Summary no semver update required". The public additions are all additive:
  - the new type `PlaneDecoder`;
  - new fields on the `#[non_exhaustive]` `Sensor`;
  - new variants of the `#[non_exhaustive]` `Error` (eight since 0.2.4, 68 to 76) and
    `DevelopPath` (`Demosaic`, `ColourTransform`).

  This follows 0.2.4's precedent: a patch for an output change and new refusals.
  **If the build's semver check reports any breaking change, the build stops for the
  user's decision** (0.3.0 or a fix).
- **Every input whose result changes since `v0.2.4`.** The design's list follows. The
  build must re-derive it from `git diff v0.2.4 <candidate> -- src/` (every `Err(` path,
  every new check, every reordering) and reconcile the two. A difference is a finding,
  not an edit to make silently. Signal
  `release-notes-diff-every-changed-refusal-and-what-it-preempts` asks for this.
  - **Output changes:**
    - Colour develops of every held dual-illuminant file change: the Pixel 7 Pro, 3a
      and 4a, the Canon EOS 5D Mark III and the Nikon D300S (SPEC-085: calibration
      applied and `NeutralToXY` converged). Every held file is now within 7 codes of
      Adobe DNG SDK 1.7.1's camera-to-XYZ transform in SPEC-085's probe.
    - Mono output does not change. `DEVELOP_VERSION` goes from `"7"` to `"8"`.
    - A file with an optional opcode newer than DNG 1.7.1.0 now skips it (PR 301). The
      build should confirm whether 0.2.4 applied such an opcode, for example an optional
      `GainMap`, and say so either way.
  - **Newly developing:** a dual-illuminant file whose singular `ColorMatrix` (or applied
    calibration) the converged weight never uses. 0.2.4 refused it with
    `MissingColorCalibration`. Whether it develops depends on the scene (SPEC-085,
    DEC-076 §3, DEC-078 §4).
  - **Newly refused:**
    - PR 301: non-finite `WarpRectilinear` coefficients (`MalformedOpcodeParams` at parse;
      `NonFiniteWarpCoefficients` from `apply_warp_into`), `CFALayout` other than 1
      (`UnsupportedCfaLayout`), `CFAPlaneColor` other than `[0, 1, 2]`
      (`UnsupportedCfaPlaneColor`), and a mandatory opcode newer than 1.7.1.0
      (`UnsupportedOpcodeVersion`).
    - SPEC-085: an applied calibration or `AnalogBalance` that is unusable, or a
      non-positive reference neutral on the ForwardMatrix path
      (`MissingColorCalibration`).
    - SPEC-086: a DNG backward version above 1.7.1.0 (`UnsupportedDngVersion`), the seven
      tags with a value that is not an exact no-op (`UnsupportedTag`), and, **at plane
      read**, a Compression 7 frame whose precision is not `BitsPerSample`
      (`LjpegPrecisionMismatch`).
    - SPEC-087: a malformed tag the path reads (`MalformedTag`), and a malformed
      `DNGVersion` with no backward version (`UnsupportedTag { tag: 50706 }`).
  - **Already refused, different error:** every new develop check runs first in
    `check_required_processing`, so an input that 0.2.4 refused with a later error, and
    that also trips a new check, now reports the new one. Examples to confirm or correct:
    - a malformed `WhiteLevel` together with an unreadable `OpcodeList`, which was
      `UnreadableOpcodeList`;
    - a malformed `CFAPattern` together with a malformed `CFALayout`, which was
      `NotAColorFilterArray`;
    - any input that also fails `InvalidLevels`, `InvalidActiveArea` or
      `MissingColorCalibration`.

    The build names each class it finds.
  - **Composer-visible:** `check_required_processing(sensor, DevelopPath::Colour)` now also
    refuses a malformed levels, geometry, CFA or colour tag. `demosaic_into`,
    `demosaic_into_threaded` and `to_srgb_linear_into` now check their own, narrower sets.
- **No held file changes behaviour** except the colour output above. Every held file still
  develops, and the Pentax K-3 III Monochrome's malformed `BlackLevelRepeatDim` stays
  exempt.
- **What's included:** the CHANGELOG's `[Unreleased]` section, plus the PR 301 entries the
  build adds.
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1 — reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes.

1. **The PR 301 CHANGELOG entries.** Add them to `[Unreleased]`, under Added (`PlaneDecoder`,
   the four `Error` variants) and Changed (the four refusals, the `CFAPattern` reading,
   the opcode version rule). Take the facts from PR 301's description
   (`gh pr view 301 --repo jysf/irradiance`) and its diff, not from memory.
2. **The changed-input diff** (Release Scope above): reconcile the design's list with
   `git diff v0.2.4 <candidate> -- src/`, and record the result in the internal release
   record.
3. **The semver check:** `just semver-check`, recording the tool version and the full
   summary. Any reported breaking change stops the build for the user's decision.
4. **Version:** `Cargo.toml` `version = "0.2.5"`, with every other field unchanged from
   0.2.4 (`git diff v0.2.4 -- Cargo.toml` before the bump is empty).
5. **CHANGELOG** (Keep a Changelog): promote `[Unreleased]` to `## [0.2.5] - <date>`, with
   a short lead paragraph like 0.2.4's and a fresh empty `## [Unreleased]` on top.
   **Correct any entry whose wording the changed-input diff contradicts.**
6. **Release notes:** run `just release-notes PROJ-002`, edit into `## Release Notes` below
   in 0.2.4's shape (Headline, Changed, Behaviour changes, Breaking / action required,
   Known limitations, then the internal record), and check them line by line against the
   reconciled changed-input list. Update Known limitations: `BlackLevelDelta*` and
   `LinearizationTable` are now refused rather than ignored, the Pixel 7 Pro's colour gap
   is closed to within 6 codes, and the ColorChecker ΔE is still pending.
7. **README and crate docs:** check they still describe 0.2.x accurately (`cargo test
   --doc`, `just doc-check`). Wording fixes only.
8. **Tier-B evidence on the candidate SHA,** with `IRRADIANCE_CORPUS_DIR` set:
   - `just test`: all 12 held files present, every pin held.
   - The SDK colour cross-check, with `IRRADIANCE_SDK_COLOUR_DIR` set from a fresh
     `scripts/sdk-colour.sh` run into the scratchpad. Record the 18 deltas per file; they
     must equal SPEC-085's 6 / 6 / 1 / 2 / 5 maxima.
   - The stage-2 cross-check, with `IRRADIANCE_SDK_STAGE2_DIR` from a fresh
     `scripts/sdk-stage2.sh` run. Record the table as 0.2.4 did and compare it with
     0.2.4's. The colour change does not touch stage 2, so every figure should be equal;
     say so or explain.
   - The three timing examples (`just orient-timing`, `just unpack-timing`,
     `just ljpeg-timing`), recorded against 0.2.4's figures.
9. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`,
   `cargo publish --dry-run`. The package list must be the same files as 0.2.4: no
   `scripts/`, `tests/`, SDK or corpus bytes.
10. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), using SPEC-083's
    filled pre-flight as the model. Then run every gate: `just test`, `lint-ci`,
    `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`,
    `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`,
    `semver-check`, `package-targets-check`.

## Release Notes

*Outward-facing, written for whoever uses the crate. Filled at build (step 6) in 0.2.4's
shape, from `just release-notes PROJ-002`, edited against the final CHANGELOG with SPEC ids
dropped, and checked line by line against the reconciled changed-input list. The GitHub
release text for `v0.2.5` is this section, down to the internal record.*

**Headline:** <filled at build>

**Breaking / action required:** <filled at build>

### Internal release record (not part of the GitHub release text)

<filled at build: the semver summary verbatim, the changed-input reconciliation, the SDK
colour and stage-2 tables, the timings, the package list>

## Release cut is two-phase

A release session **cannot finish in one pass** — the irreversible tag/publish is
gated by a human/coordinator. Structure it so the reversible work lands first:
- **Phase 1 — reversible prep (CI-gated PR):** CHANGELOG entry, version bump
  (`just next-version`), stage-backlog tick. Fully revertable — land it as a
  normal PR.
- **Phase 2 — irreversible cut (human/coordinator-gated):** create the tag and
  publish to each channel, only after Phase 1 merges. The spec ships
  **"prep-complete, cut-deferred."** Before tagging, confirm the checked-out tree
  is the merged Phase-1 commit: the version in the manifest is the new one. A tag
  cut on the commit before the merge points at the old version (irradiance 0.2.4's
  first attempt).

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

- [ ] **1. Version / tag integrity & build provenance** — exactly one release tag points
      at the release commit; the version baked into the artifact matches the tag;
      no stale or duplicate tag on the same commit. **And the shipped artifact
      reports its own build provenance** (version + commit SHA) that matches the
      release commit — `just build-info` is the stamp; it must be injected into
      the build (DEC-008, `docs/versioning.md`). A user should be able to trace
      exactly what they're running back to source.
      - Command / evidence: <REPLACE — e.g. `git tag --points-at HEAD` and
        `<app> --version` reports the `just build-info` ref>
- [ ] **2. Artifact trust on a clean host** — a freshly-downloaded artifact is
      trusted by a machine that didn't build it (code-signing / notarization /
      OS quarantine — Gatekeeper, SmartScreen). "Does a stranger's machine run
      it without a scary prompt or a block?"
      - Command / evidence: <REPLACE — download the release asset on a second
        machine and run it>
- [ ] **3. Distribution-channel trust** — a *new* user can install cleanly
      through each published channel (package-manager tap/registry trust gates,
      first-install auth). "Install it the way a stranger would, on a machine
      that never saw the source."
      - Command / evidence: <REPLACE — e.g. `brew install …` on a clean host>
- [ ] **4. Data isolation** — a dev / test / CI build can never read or migrate
      **production** data. Confirm the prod data path is unreachable from a
      non-prod binary.
      - Command / evidence: <REPLACE — run the built binary with the dev env and
        assert it cannot resolve the prod DSN>
- [ ] **5. Runtime smoke on a clean host** — the *shipped* artifact runs (not
      just builds) on a fresh install: it starts, answers, and its headline
      capability works end-to-end.
      - Command / evidence: <REPLACE — the one command that proves it works>
- [ ] **6. Rollback / uninstall** — there is a known, tested way to withdraw or
      downgrade the release. Exercise it at least once.
      - Command / evidence: <REPLACE — e.g. `brew uninstall …` / re-tag prior>

If a real release surfaces a category this list doesn't cover, record it in
`guidance/signals.yaml` (`type: process-debt`) so a close can grow the generic
categories (DEC-006 keeps them category-level, not command-level).

## Notes for the Implementer

- Before tagging, the user confirms the checked-out tree is the merged Phase-1 commit
  (`Cargo.toml` shows 0.2.5). 0.2.4's first tag landed on the commit before the merge.
- The SDK lanes are local only (DEC-069 §6): the SDK, its outputs and the corpus are never
  committed or packaged. Run both scripts into a scratchpad outside the repository.
- A defect found during prep is reported in the handback, not fixed here: the stage is
  release-only.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).

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
