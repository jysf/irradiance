---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-074
  type: chore                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S             # stamped at ship: what it ACTUALLY took, same scale.
                                   #   Expected-vs-actual drift is what `just calibration` reads.
  verify_verdict: approved  # approved | punch-list | rejected — the OUTCOME of the verify
                                   #   cycle, stamped by `just advance-cycle` when the spec leaves
                                   #   verify (same three verdicts Prompt 4 already returns).
                                   #   Recorded in front-matter, not just prose, so "verify never
                                   #   rejects anything" stops being a hunch and becomes a number.
  verify_rounds: [approved]  # every verify outcome, oldest first, appended by
                                   #   `just advance-cycle` each time the spec leaves verify.
                                   #   verify_verdict keeps only the LAST one, so a spec sent
                                   #   back and then approved read as "approved": irradiance's
                                   #   field showed 1 pushback where its handoffs showed 5.

project:
  id: PROJ-004
  stage: STAGE-022
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-067, DEC-047, DEC-065]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-071, SPEC-073]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-022's <capability>". Optional; null is acceptable.
value_link: "closes STAGE-021's two remaining [XS] follow-ups: artifact names that no longer crash tools, and one Compression 7 header parse per strip or tile"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify) — the agent
# that runs the cycle writes it from its own interface (/cost, the API
# usage object, or its tool's report). Only un-metered main-loop cycles
# (design/ship) may be null-with-note. `just cost-audit` enforces this on
# shipped specs. See AGENTS.md §4 and docs/cost-tracking.md. interface:
# claude-code | claude-ai | api | ollama | other.
cost:
  # Optional PREDICTION of the total tokens this spec will take, set at
  # design. Never a gate — its only job is to be compared with the actual
  # below (`just calibration`), so you learn whether you systematically
  # under- or over-estimate. null = didn't predict.
  tokens_estimate: 12000000
  sessions:
    - cycle: build
      handoff: HANDOFF-155
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 13604926
      estimated_usd: 9.36
      duration_minutes: 15
      recorded_at: 2026-11-18
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (87 unique message.ids, all claude-opus-5-5, no subagents): input 188 + cache-creation 154,084 (1-hour) + cache-read 13,406,071 + output 44,583 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Covers the build and its handback commit. The handback floor was 11,423,772 and $7.99."
    - cycle: verify
      handoff: HANDOFF-156
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 5733675
      estimated_usd: 4.47
      duration_minutes: 15
      recorded_at: 2026-11-18
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (44 unique message.ids, all claude-opus-5-5, no subagents): input 87 + cache-creation 98,435 (1-hour) + cache-read 5,608,094 + output 27,059, opus-5-5 rates. APPROVED at f56f5eb with no code changes. The handback floor was 4,661,997 and $3.69."
  totals:
    tokens_total: 19338601
    estimated_usd: 13.83
    session_count: 2
shipped_at: 2026-11-18
---

# SPEC-074: STAGE-021 follow-ups, bundled: cap legacy artifact file names, and parse Compression 7 headers once

## Context

The only spec of STAGE-022. STAGE-021 left two [XS] follow-ups in the PROJ-004 brief
(the third, the CI push trigger, was done in PRs 256 and 257). They are unrelated, and
bundled so they cost one build and one verify, as SPEC-071 did.

## Goal

1. No tracked file name is long enough to break a tool that writes a temporary file
   beside it, and a gate keeps it that way.
2. Each Compression 7 strip or tile parses its SOF-3 header once, not twice: 5
   allocations per chunk instead of 10, with byte-identical output and identical errors.

## Design

### Part A: artifact file names (DEC-067)

**What the design probe found** (main at `28ecd5d`):

- **The generators are already capped.** `slugify` in `scripts/_lib.sh` cuts a slug at
  `SLUG_MAX=60` since template v0.6.49 (PR 164, 2026-11-10). `new-handoff` derives its slug
  from the spec file, so it is capped too. **No generator change is needed.**
- **The crash comes from legacy names made before the cap.** `cargo semver-checks
  --baseline-rev v0.2.2` fails at the clone with `File name too long (os error 63)` on
  `projects/PROJ-002-bayer-colour-develop/handoffs/.HANDOFF-064-build-cfa-sensor-…-regression-guard.md.tmp-77733-<32 hex>`.
  The clone's temporary name is the file name **plus about 45 bytes**, so any name over
  about 210 bytes fails. HANDOFF-064 is 235.
- **83 tracked files have a slug over 60 characters:** PROJ-001 (30 handoffs, 20 spec or
  timeline files, 3 patches, 1 stage), PROJ-002 (10 handoffs, 12 spec or timeline files,
  4 stages), PROJ-004 (1 stage), and 2 spikes in `spikes/done/`. Six names exceed 200
  bytes and 13 exceed 150.
- **Some are referenced by name.** For example `STAGE-012-colour-develop-…` is named in 5
  DECs and 4 handoffs, and `STAGE-010-…` in 6 files. The ID is the identity everywhere
  the scripts look, but those prose references must follow the rename.
- **The release check does not need a git baseline.** `cargo semver-checks` with its
  default baseline (the latest version on crates.io, which packages no `projects/`) ran
  clean on `28ecd5d`: 202 checks, 202 pass. SPEC-073 used `--baseline-rev` only because it
  did not try the default.

**A1. Rename every legacy artifact to the name today's generator would give it.**

- **Scope:** every tracked file named `SPEC-`, `HANDOFF-`, `STAGE-`, `PATCH-` or `SPIKE-`
  followed by a number, anywhere in the repo, whose slug exceeds 60 characters. The slug
  is the part after the ID, after the cycle word for a handoff (`build-`, `verify-`, …),
  and before a `-timeline` suffix.
- **New name:** the ID, the cycle word if any, the slug cut by today's `slugify` rule (the
  first 60 characters, trailing hyphens removed), the `-timeline` suffix if any, and
  `.md`. Use `slugify` itself or a script that provably matches it. **A spec and its
  timeline must get the same slug.** If two files in one directory would get the same
  name, stop and report it (the probe found none expected, but check).
- **Use `git mv`,** so history follows each file.
- **Rewrite every reference** to an old name (with or without `.md` or a directory
  prefix) to the new one, across the whole tree: DECs, specs, handoffs, stages, briefs,
  `guidance/`, `docs/`, AGENTS.md, scripts. Leave alone only text that names a file
  inside a quoted command output or log that records history (for example the semver
  crash path quoted in SPEC-073 and HANDOFF-153), and list each one you leave.
- **DEC files are out of scope.** They are named by hand, the longest is 130 bytes, and
  they are under the gate below.
- **Commit the renames separately** from any content change, so the rename commit is
  pure moves plus reference rewrites and is easy to review.

**A2. A name-length gate, `just name-length-check`.**

- A new script and an `app.just` recipe that fails, naming each offender, when any
  **tracked** path component (`git ls-files`, each `/`-separated part) is **longer than
  150 bytes.** 150 leaves about 60 bytes of room below the observed crash point and is
  above the longest DEC (130).
- **Run it in CI** as a step in the existing `cost-data` job (it already runs the other
  repo-convention scripts), and add it to AGENTS.md §6.
- **Red-proof it** (DEC-009, control first): green on the tree; then, in a scratch copy
  or a temporary commit that is not pushed, add a tracked file with a 151-byte name and
  show the check goes red naming it; 150 bytes stays green.
- It does not check the 60-character slug rule; the generators enforce that.

**A3. A `just semver-check` recipe.**

- Runs `cargo semver-checks` with the crates.io baseline (no `--baseline-rev`), with a
  comment saying why: a git baseline clones the whole repo, including `projects/`, and a
  tag cut before this spec still holds the long names (v0.2.2 included), so
  `--baseline-rev` keeps failing against those tags. Add it to AGENTS.md §6.
- **Not a CI gate.** `docs/rust-ecosystem-and-semver-decisions.md` §2.4 keeps that as an
  open item; this spec does not decide it.

### Part B: one Compression 7 header parse per strip or tile

**Today** (`src/plane.rs`): the strip path (`decode_unit`, the `Plan::Strips` /
`PlaneKind::Jpeg` arm) and the tile path (`decode_tile_row`) each call
`ljpeg::parse_headers` to bind the frame's shape (DEC-047, `check_strip_shape`), then
call `ljpeg::decode_into` or `decode_window_into`, which parse the same header again in
`open_scan`. Each parse of these one-table, one-component headers allocates 5 times
(SPEC-071), so 10 per chunk.

**B1. Decode from an already-parsed header.**

- Add two `pub(crate)` functions to `src/ljpeg.rs` that take the result of
  `parse_headers` (`Frame`, the table `Vec`, and the entropy offset) plus the data, and
  do exactly what `decode_into` and `decode_window_into` do after their parse. A small
  `pub(crate)` struct for the parsed header is fine if it reads better than a tuple.
- **`decode_into` and `decode_window_into` become thin wrappers:** parse, then call the
  new functions. Their behaviour, signatures and every other caller (the fuzz entry in
  `src/lib.rs`, the unit tests, `tests/plane_oracle.rs`'s injector needles) stay as they
  are. Keep `open_scan`'s slice-and-`Truncated` logic in one place, shared by both paths.
- **`plane.rs` calls the new functions** with the header it already parsed for the shape
  check, in both the strip and the tile path. The shape check still runs **before**
  anything is decoded.
- **Errors are identical by construction:** the one parse is the same function on the
  same bytes, it runs first as today, the shape check runs second as today, and the scan
  starts from the same offset. Say so in a comment. No error variant or field changes.
- Update the comments that describe two parses: the "extra header parse is tens of bytes"
  note at the strip path, the tile path's DEC-047 note, and `open_scan`'s doc.

**B2. Re-pin the allocation count.**

- `tests/plane_units_alloc.rs`: `C7_ALLOCATIONS_PER_CHUNK` goes from 10 to **5**, and its
  comment changes from "parses its SOF-3 header TWICE" to once, keeping the list of the
  five allocations. **Measured at design:** a throwaway version of B1 on `28ecd5d` gave 20
  for the 4-strip fixture (5 each) and passed the pin at 5 on both compressed fixtures and
  every band height; `plane_units_alloc`, `plane_oracle` and `plane_units` were green.
- The old-equals-new assertion stays.

**B3. CHANGELOG.** One line under `[Unreleased]` → `Changed` (or the section the file
uses for performance): Compression 7 strips and tiles parse their header once, half the
per-chunk allocations, no output change.

## Inputs

- `scripts/_lib.sh` (`slugify`, read only: it is template-managed), `scripts/new-handoff.sh`.
- Every tracked `SPEC-`/`HANDOFF-`/`STAGE-`/`PATCH-`/`SPIKE-` file; every file that
  names one.
- `.github/workflows/ci.yml` (`cost-data` job), `app.just`, AGENTS.md §6.
- `src/ljpeg.rs` (`decode_into`, `decode_window_into`, `open_scan`, `parse_headers`),
  `src/plane.rs` (`decode_unit`'s Compression 7 strip arm, `decode_tile_row`),
  `tests/plane_units_alloc.rs`, CHANGELOG.md.
- Decisions: DEC-067 (this spec), DEC-047 (shape binding), DEC-065 (the scan reader),
  DEC-009 (red-proof).

## Outputs

- About 83 renamed files and their reference rewrites.
- New: one script and two `app.just` recipes (`name-length-check`, `semver-check`).
- Modified: `ci.yml` (one step), AGENTS.md §6, `src/ljpeg.rs`, `src/plane.rs`,
  `tests/plane_units_alloc.rs`, CHANGELOG.md.
- No public item changes.

## Acceptance Criteria

- [ ] **AC1. No long slugs remain.** A listing over `git ls-files` shows no
  `SPEC-`/`HANDOFF-`/`STAGE-`/`PATCH-`/`SPIKE-` file with a slug over 60 characters, and
  every renamed spec and timeline pair shares one slug. `git log --follow` works on a
  renamed file (show one).
- [ ] **AC2. No dangling references.** For every old name, `git grep -F` finds it only in
  the history quotes listed under A1. `just validate`, `just id-audit` (if it exists),
  `just decisions-index --check` and `just cost-audit` pass.
- [ ] **AC3. The crash is gone for new tags.** `cargo semver-checks --baseline-rev HEAD`
  (a git baseline of the renamed tree, which clones it the same way) gets past the clone.
  Show the output. `just semver-check` runs clean against crates.io 0.2.2.
- [ ] **AC4. The gate bites.** `just name-length-check` is green on the tree, red on a
  151-byte name naming it, green at 150 (control first). The step is in CI's `cost-data`
  job and passes there.
- [ ] **AC5. One parse per chunk.** `C7_ALLOCATIONS_PER_CHUNK` is 5 and the test passes.
  Red-proof: with the plane path reverted to the double parse (scratch copy), the pin goes
  red (40 against 20 on the 4-strip fixture); with one extra allocation per chunk added
  instead, it goes red too.
- [ ] **AC6. Byte and error identity.** Every Compression 7 oracle and MD5 pin is green
  and unedited: `plane_oracle`, `plane_units`, `planes`, the tiled and Compression 7
  tier-B tests with the corpus set. Run `just ljpeg-timing` and record the three medians
  (no target; expect no measurable change).
- [ ] **AC7. No regression.** Every gate is green with the corpus env set: `just test`,
  `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`,
  `msrv`, `cost-audit`, `gate-audit`, `name-length-check`.

## Failing Tests

- `C7_ALLOCATIONS_PER_CHUNK = 5` fails on today's code (20 expected, 40 measured on the
  4-strip fixture) and passes after B1. Write it first.
- `name-length-check` has no failing test on today's tree for files over 150 bytes until
  the renames: run it before A1 and show it red on the 13 names over 150 bytes, then green
  after.

## Non-Goals

- Changing `slugify`, `SLUG_MAX` or any template-managed script.
- Renaming DECs, or anything not generated by `new-spec` / `new-handoff` / `new-stage` /
  `new-patch` / `new-spike`.
- Rewriting history or old tags; v0.2.1 and v0.2.2 keep their long names.
- `cargo-semver-checks` as a CI gate.
- Any change to decode output, the scan reader, or a public item.

## Notes for the Implementer

- **Do Part B and Part A in separate commits,** and the renames in a commit of their own.
- **`just test` takes about 3 minutes** (SPEC-072's `corpus` lane) plus the debug
  allocation run; no need to background it.
- **Corpus:** `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
  Use the files in place; never copy them into the repo.
- **Commit messages are plain prose** (no code tokens), never `git add Cargo.lock`, stage
  explicit paths.
- **A renamed spec still has its cost and history in front matter.** Do not edit those
  while renaming.

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Nothing in substance. The design probe paid for itself. It reproduced the semver
   crash and found the crates.io baseline clean, and it prototyped the one-parse plane
   path to a measured 5 allocations per chunk. So the build had no open question and
   ran in one turn. The bundle cost about 19.3M tokens against a 12M estimate. Most of
   the overrun was the 83-file rename and its reference rewrites, which read many files
   for few changed lines.

2. **Does any template, constraint, or decision need updating?**
   — DEC-067 records it. Generated names were already capped at 60 characters by the
   template's `slugify` since v0.6.49. Only the legacy files were long, and CI now keeps
   any tracked path component at 150 bytes or less. `just semver-check` uses the
   crates.io baseline, so it no longer needs a local clone of a tag.

3. **Is there a follow-up spec I should write now before I forget?**
   — None. Both STAGE-021 follow-ups are closed. The macOS determinism leg stays a watch
   signal (`macos-arm64-runner-unavailable`), not a spec.

4. **Where was the worst defect caught?**
   — design
   *(The probe showed the semver crash came from legacy names, not from the generator,
   so the build renamed files instead of changing the template. Verify found nothing:
   every mutant went red, including moving the decode before the shape check.)*

5. **What can a user do now that they couldn't before?**
   — A contributor can run `cargo semver-checks` and other tools that clone the repo
   without a "File name too long" crash, using `just semver-check`. A crate user gets a
   Compression 7 strip or tile decoded with one header parse, so 5 allocations per
   chunk instead of 10. The output is unchanged.
