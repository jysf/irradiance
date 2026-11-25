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
  id: SPEC-095
  type: release                    # epic | story | task | bug | chore | release
  cycle: build  # frame | design | build | verify | ship
  blocked: false
  priority: high                   # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: null          # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.

project:
  id: PROJ-005
  stage: STAGE-032
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-006, DEC-028, DEC-055, DEC-067, DEC-071, DEC-080, DEC-082]
  constraints: [library-not-application, no-new-top-level-deps-without-decision]
  related_specs: [SPEC-061, SPEC-081, SPEC-082, SPEC-094, SPEC-090]

value_link: "Puts the STAGE-017 writer on crates.io, so onyx's Darkroom v1 can depend on a published irradiance with the write feature."

# Metered cycles (build/verify) require a real tokens_total on a shipped
# release — same rule as any spec. See AGENTS.md §4 and docs/cost-tracking.md.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 25000000       # S; SPEC-090 (0.2.6) ran 27.2M with two SDK lanes this one drops, plus a packaged-crate writer check it adds
  sessions:
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-11-25
      notes: "main-loop, not separately metered (AGENTS.md section 4); measured at the stage close as orchestration"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-095: irradiance 0.2.7 release: the public DNG writer

## Context

The release spec of STAGE-032 (PROJ-005). irradiance 0.2.6 was published on 2026-11-23
(annotated tag `v0.2.6` on `50d533d`). Since then `main` has shipped, to `153f517`, the whole
of STAGE-017 (closed 2026-11-25, PR 342):
- **SPEC-061** (PR 329): the public writer behind the new `write` feature: `written_len`,
  `write_into`, `write_to_vec`, `WriteOptions`, three framings, levels and opcode lists
  copied, required `UniqueCameraModel`, and two `Error` variants present in every build
  (DEC-080).
- **SPEC-081** (PR 332): the EXIF block at a fixed base offset, XMP, eight descriptive
  ASCII tags and four rendering hints.
- **SPEC-082** (PR 336): an optional caller JPEG preview, stored as a SubIFD.
- **SPEC-094** (PR 339): an empty opcode list is refused, CI fuzzes `dng_write`, three
  doc links fixed.

Measured at design on `153f517`: `git diff --stat v0.2.6 153f517 -- src/ Cargo.toml` touches
only `src/dng/mod.rs`, `src/dng/write.rs`, `src/write_types.rs` (all new), `src/lib.rs` (the
two modules, two `Error` variants with their `Display` and `kind()` arms) and `Cargo.toml`
(the `write = []` feature). No reader, develop, colour or `irr` code changed. The
CHANGELOG's `[Unreleased]` section already carries all four specs' entries, written by their
builds.

onyx's Darkroom v1 (`monorail-000/DEC-075`) consumes a published crate, so the writer
reaches onyx only through this release. This spec ends prep-complete; a cold verify
reviews the prep PR; the user then tags and runs `cargo publish`, as for 0.2.6.

## Release Scope

### Decided for 0.2.7 (design, 2026-11-25)

- **Version / tag:** `0.2.7`, annotated tag `v0.2.7`, a **patch** over the live 0.2.6.
  Dependents on `irradiance = "0.2"` receive it with `cargo update`; the writer stays off
  until they enable `write`.
- **Why it is compatible: the design's mechanical check.** On `153f517`, `just
  semver-check` (crates.io 0.2.6 baseline, DEC-067) reports "202 checks: 202 pass, 58 skip
  … no semver update required", and so does `cargo semver-checks --all-features`. Every
  change is an addition: a new opt-in feature, a new module behind it, and two variants on
  `Error`, which is `#[non_exhaustive]`, so no caller's match can be exhaustive. 0.2.1 to
  0.2.6 shipped additions as patches too. **If the build's semver check reports any
  breaking change, the build stops for the user's decision.** The tool cannot see the
  `write` surface as new API (the baseline has none to compare); that is expected, not a
  gap (SPEC-081's reflection).
- **Only the writer ships. PR 343 (SPEC-091, PROJ-006's linear sRGB plane) must not be in
  this release.** It is open and changes reader-side `src/` and `Error`. The prep is cut
  from `origin/main` and **the build first checks `git diff --stat 153f517 <cut> -- src/
  Cargo.toml` is empty**. If anything has landed, the build stops and the user decides:
  hold the release until it is re-scoped, or re-cut from `153f517`. The user is asked to
  hold PR 343's merge until `v0.2.7` is tagged.
- **docs.rs documents the writer (the one manifest change besides the version).** docs.rs
  builds default features unless told otherwise, so 0.2.7 as it stands would publish the
  writer with no documentation on docs.rs. Add to `Cargo.toml`:

  ```toml
  [package.metadata.docs.rs]
  features = ["write"]
  ```

  `write` only: never `--all-features`, which would document the dev-only `fuzzing`,
  `irr` and `test-fixtures` items. No `doc_cfg` badges (nightly only); the module docs
  already say "behind the `write` feature". The build adds a one-line comment above it in
  the file's own style.
- **Every input whose result changes since `v0.2.6`.** The design's list, from the
  `src/` diff above. The build re-derives it from `git diff v0.2.6 <candidate> -- src/`
  and reconciles; a difference is a finding.
  - **Reader, develop, colour and `irr`: no change.** No line outside the writer's three
    files and `src/lib.rs`'s additions changed. Every existing input develops to the same
    output, `DEVELOP_VERSION` stays `"8"`, every existing refusal keeps its error, and no
    check is reordered. `ErrorKind` gains no variant.
  - **`Error`: two new variants in every build.** `InvalidWriteInput(WriteInputError)`
    (`CallerError`) and `UnsupportedOpcodeForFraming { list, opcode_id, framing }`
    (`Unsupported`). Only the writer returns them, so a reader-only caller never sees
    them. Without `write` their payload types have no public path (DEC-080). **The build
    measures `size_of::<irradiance::Error>()` at `v0.2.6` and on the candidate** and
    records both; a larger `Error` is a note for the release notes, not a stop.
  - **Newly accepted: everything the writer does,** with `write` on. Its refusals are
    `WriteInputError`'s 22 reasons and `UnsupportedOpcodeForFraming`; the build lists them
    from the code and checks the CHANGELOG names each.
- **What's included:** the CHANGELOG's `[Unreleased]` section (SPEC-061, 081, 082, 094) and
  the README's writer section.
- **Delivery shape:** library plus a crates.io package; no binary.

## Build work (Phase 1: reversible prep, the implementer's cycle)

One CI-gated PR, fully revertible. The implementer never tags or publishes. A defect found
in prep is reported in the handback, not fixed here.

1. **The cut point.** Confirm `git diff --stat 153f517 <cut> -- src/ Cargo.toml` is empty
   (Release Scope). Otherwise stop.
2. **The changed-input diff.** Reconcile the design's list with `git diff v0.2.6 <candidate>
   -- src/`: every new `Err(` path, every changed line outside the three writer files.
   Record the result, and both `size_of::<Error>()` figures, in the internal record.
3. **The semver check:** `just semver-check` and `cargo semver-checks --all-features`,
   recording the tool version and both summaries, after the version bump.
4. **Manifest:** `version = "0.2.7"` and the `[package.metadata.docs.rs]` table. Every other
   field is unchanged (`git diff v0.2.6 -- Cargo.toml` before the bump shows only the
   `write` feature).
5. **CHANGELOG** (Keep a Changelog): promote `[Unreleased]` to `## [0.2.7] - <date>` with a
   short lead paragraph like 0.2.6's and a fresh empty `## [Unreleased]`. Check each entry
   against the shipped code; the SPEC-061 entry's reason count ("fourteen reasons, nineteen
   with SPEC-081's") is stale: SPEC-082 and SPEC-094 brought it to 22. Correct anything
   the reconciliation contradicts.
6. **Release notes:** run `just release-notes PROJ-005`, then write `## Release Notes`
   below in 0.2.6's shape (Headline, Added, Behaviour changes, Breaking / action
   required, Known limitations, then the internal record). The writer's own limits go
   under Known limitations: monochrome `u16` only; uncompressed, one strip, little-endian;
   no CFA, tiles or float; the preview is caller-encoded; the EXIF block is placed, never
   parsed; under a cutting framing `OpcodeList1` may hold only `FixBadPixelsConstant`;
   the Lightroom Classic / Capture One open is not recorded. 0.2.6's reader limitations
   carry over unchanged.
7. **README and crate docs:** the README's writer section matches the shipped API (it
   predates SPEC-081's and SPEC-082's options and SPEC-094's refusal); wording fixes only.
   `cargo test --doc --all-features` and `just doc-check`.
8. **Evidence on the candidate SHA,** with `IRRADIANCE_CORPUS_DIR` and
   `IRRADIANCE_DNG_VALIDATE` set:
   - `just test`: every held file present, every pin held, and the writer's
     `dng_validate` acceptance tests ran rather than skipped (say which).
   - `just fuzz-dng-write` for 60 s, no crash.
   - **Not re-run, by design:** the SDK colour and stage-2 lanes, the `irr ifd` diff and
     the timing examples. They measure reader code that step 2 confirms is byte-identical
     to `v0.2.6`, whose figures SPEC-090 recorded. If step 2 finds any reader change, run
     them as SPEC-090 did.
9. **The packaged crate, built as a consumer would (new for this release).** `cargo publish
   --dry-run` builds the package with default features only, so it never compiles the
   writer. In a scratch directory outside the repository:
   - `cargo package` on a clean tree, then in the unpacked `target/package/irradiance-0.2.7`:
     `cargo build --features write`, `cargo build --no-default-features`, and
     `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --features write` (what docs.rs runs);
   - a scratch consumer crate depending on that unpacked path with `features = ["write"]`:
     parse one held Monochrom DNG, write it at each framing with `write_to_vec`, re-parse
     each, and compare the samples over the written region with the source. Run
     `dng_validate` on the three outputs. Record the output MD5s;
   - the same consumer without `features = ["write"]` must fail to compile a use of
     `irradiance::dng` (the feature gate holds in the package).
10. **Package guards:** `just package-targets-check`, `just deny`, `just deny-fuzz`, `cargo
    publish --dry-run`. The package list must be 0.2.6's files plus exactly
    `src/dng/mod.rs`, `src/dng/write.rs` and `src/write_types.rs`.
11. **Fill the Release Pre-Flight below** (`[now]` / `[cut]` / `N/A`), with SPEC-090's filled
    pre-flight as the model, then run every gate: `just test`, `lint-ci`, `lint-no-allow`,
    `lint-fuzz`, `lint-fuzz-red-proof`, `fmt-check`, `validate`, `doc-check`, `deny`,
    `deny-fuzz`, `msrv`, `cost-audit`, `gate-audit`, `name-length-check`,
    `decisions-index --check`, `semver-check`, `package-targets-check`.

## Release Notes

*Outward-facing. Written by the build from the final CHANGELOG, SPEC ids dropped, and
checked line by line against the reconciled changed-input list. The GitHub release text for
`v0.2.7` is this section, down to the internal record.*

**Headline (design's draft; the build may sharpen it):** irradiance can now write DNG. The
opt-in `write` feature turns a monochrome `u16` sensor plane, with its levels, opcode
lists, EXIF block, XMP, descriptive tags and an optional JPEG preview, into an
uncompressed DNG in a buffer you size first. Same input, same bytes; no allocation in
`write_into`; no new dependency. Adobe `dng_validate` accepts the output with no warnings.
Nothing changes for a reader-only caller.

**Breaking / action required:** none. *(The build confirms.)*

### Internal release record (not part of the GitHub release text)

*Filled by the build: the candidate and where each piece of evidence ran, the semver
summaries verbatim, the changed-input reconciliation, the two `size_of::<Error>()`
figures, the packaged-crate writer check with its MD5s and `dng_validate` results, and
the package list diff against 0.2.6.*

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
  Make the tag annotated (`git tag -a`), then confirm `git cat-file -t <tag>` prints
  `tag` and, after the push, that `git ls-remote --tags origin '<tag>*'` shows a `^{}`
  line. A lightweight tag (irradiance 0.2.5's) still publishes, but `git describe`
  skips it.

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

- The corpus and `dng_validate` are local only (DEC-069 §6): never commit or package them,
  or anything they write. Set `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
  and `IRRADIANCE_DNG_VALIDATE` to the local build under `tools/dng_sdk`; `just test`
  already passes `--all-features`.
- Do the step-9 scratch crates in the session's scratchpad, not in the repository.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).
- **Phase 2 runbook (the user, after the Phase-1 PR merges):**
  1. `git checkout main && git pull`, clean tree; confirm `Cargo.toml` shows
     `version = "0.2.7"` and that PR 343 has not merged (or the release was re-scoped).
  2. `cargo publish --dry-run`, without `--allow-dirty`.
  3. `git tag -a v0.2.7 -m "irradiance 0.2.7"`, then `git push origin v0.2.7`.
  4. `cargo publish`, with the user's crates.io token.
  5. The `[cut]` halves of pre-flight items 1, 3 and 5 (the orchestrator can run the
     clean-host crate), reported back for the Cut Record. PR 343 can merge after this.
  6. Optional: a GitHub release on `v0.2.7` with the `## Release Notes` text.

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
