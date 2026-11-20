---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-079
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: medium                 # critical | high | medium | low
  complexity: S                    # XS | S | M | L | XL | XXL — the EXPECTED size, set at design
                                   #   (XL/XXL almost certainly means it's a stage, not a spec)
  complexity_actual: S          # stamped at ship: what it ACTUALLY took, same scale.
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
  stage: STAGE-025
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: null                   # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: null

references:
  decisions: [DEC-069, DEC-070]  # [DEC-NNN, DEC-MMM]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-060, SPEC-078]  # [SPEC-NNN]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: [SPEC-078]                # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-025's <capability>". Optional; null is acceptable.
value_link: "Closes the follow-ups PROJ-004 carried, so the project closes with only the consumer-gated stages deferred."

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
  tokens_estimate: 15000000
  sessions:
    - cycle: build
      handoff: HANDOFF-169
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 11804634
      estimated_usd: 8.76
      duration_minutes: 35
      recorded_at: 2026-11-20
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (68 unique message.ids, all claude-opus-5-5, no subagents): input 138 + cache-creation 175,048 (1-hour) + cache-read 11,580,532 + output 48,916 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Covers the IFD walk, the directory order, the set-means-must-run helper, the FU-2 fixtures with four mutants, the records and the PR. One approved-in-handback deviation: the general-path fixture puts the maximum at all 16 indices. The handback floor was 10,271,921 and $7.65."
    - cycle: verify
      handoff: HANDOFF-170
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 9174244
      estimated_usd: 6.71
      duration_minutes: 80
      recorded_at: 2026-11-20
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (63 unique message.ids, all claude-opus-5-5, no subagents): input 141 + cache-creation 132,063 (1-hour) + cache-read 9,006,665 + output 35,375, opus-5-5 rates. APPROVED at 8abd264 (PR 279 head), handback carried into PR 279 by fast-forward. Six mutants all red; three non-blocking follow-ups (script repo root from its own location, ledger PDF count and duplicated paragraph fixed at ship, validate does not parse feedback). The orchestrator nudged it once after it ended a turn waiting on background runs. The handback floor was 7,334,564 and $5.39."
  totals:
    tokens_total: 20978878
    estimated_usd: 15.47
    session_count: 2
shipped_at: 2026-11-20
---

# SPEC-079: PROJ-004 close-out hygiene: sdk-stage2.sh walks the IFDs and creates its output dir after the in-repo check; the SDK cross-check fails rather than skips when its env var is set without inputs; the Adobe licence line in the provenance ledger; the PROJ-004 brief count and monorail triage table refreshed

## Context

STAGE-025 (PROJ-004), the last spec of the last PROJ-004 stage. SPEC-078 shipped DEC-070
on 2026-11-20. This spec closes the small follow-ups PROJ-004 has carried, so that the
project can close with only the consumer-gated STAGE-017 and STAGE-004 deferred.

The design read the current state of each item on `origin/main` at `2ca203b`:

1. **`scripts/sdk-stage2.sh` finds `OpcodeList2` by a byte pattern.** It searches the
   whole file for the 4-byte tag-plus-type pattern of tag 51009 and refuses unless it
   occurs exactly once. A file whose pixel or metadata bytes happen to contain that
   pattern is refused, though nothing is wrong with it. In all three held Pixels the tag
   sits in the raw SubIFD (IFD0's `SubIFDs` entry, 29 entries, entry 26), so an IFD walk
   finds it directly.
2. **The script creates its output directory before checking it is outside the repo.**
   `mkdir -p "$out"` runs first, so an in-repo path is refused but its directory is left
   behind.
3. **The SDK cross-check still skips on a missing corpus file.** With
   `IRRADIANCE_SDK_STAGE2_DIR` set, a missing SDK output already panics (`sdk_stage2`).
   But `gain_step_matches_dng_validate_stage_2`, `end_to_end_and_no_gain_are_within_their_bounds`
   and `canon_no_gain_is_within_its_bound` `continue`/`return` when the corpus file is
   absent. So a run that set the variable on purpose can pass with nothing compared.
4. **The Adobe licence line.** The provenance ledger's SDK rows (SPEC-059's `GainMap`
   row and SPEC-078's black-normalisation row) quote the per-file notice: Adobe permits
   use "in accordance with the terms of the Adobe license agreement accompanying it".
   DEC-069 asked the builder to record that agreement's terms. The design looked:
   `dng_sdk_1_7_1.zip` (sha256 `5366d9ab…e6e0c2`, 4,259 entries) contains **no licence
   agreement document**. Its top level holds `DNG_ReadMe.txt`, `XMP_ReadMe.txt`,
   `JPEG_ReadMe.txt`, `JXL_ReadMe.txt`, two PDFs (*Getting Started with DNG 1.7.1* and
   the DNG 1.7.1.0 specification), `dng_sdk/documents/dng_validate.pdf` and sample files.
   None of the texts or PDFs mentions a licence agreement (`pdftotext` and `grep -i
   licen`). The 155 `dng_sdk/source` files carry the notice and nothing more. The
   licences that do ship are the third-party ones (libjpeg's, the XMP toolkit's Apache
   2.0, libjxl's), none of which irradiance reads.
5. **The PROJ-004 records.**
   - The brief's count, "10 shipped / 1 active / 2 pending (+1 parked)", is accurate at
     design and needs no change here; STAGE-025's close updates it.
   - `feedback/2026-11-11-monorail-000-requests.md` is stale: `status: open` although
     its own closing rule ("open until items 1, 2, 3 and 5 ship") is met; item 5 says
     "in `[Unreleased]`" though it was released in 0.2.2; item 8 says "built, in verify"
     though SPEC-075 shipped and was released in 0.2.3; no row says what was released
     in 0.2.3.
6. **FU-2, from SPEC-078's verify.** Every tier-A colour fixture puts its largest black
   in the last slot (`tests/black_normalisation.rs` `BLACKS = [2047, 2047, 2047, 2048]`,
   `tests/color_oracle.rs` `SLOT_BLACK = [200, 400, 600, 800]`). A colour bug that takes
   `B` as the last slot's black stays green on tier A, and only the tier-B Pixel pins,
   which CI does not run, catch it.

## Goal

Harden the SDK recipe and the env-gated cross-check so a deliberate SDK run cannot pass
by comparing nothing, record the Adobe licence finding in the ledger, bring the monorail
triage record up to date, and close the colour fixture gap from SPEC-078's verify. No
library code changes.

## Design

1. **`sdk-stage2.sh` walks the IFDs** (Perl, as now).
   - Read the header: byte order (`II` or `MM`), magic 42, IFD0 offset.
   - Walk IFD0, its `NextIFD` chain, and every IFD reachable through `SubIFDs` (330),
     recursively. Bounds-check every offset and count against the file length. An IFD
     offset reached a second time is refused, so a cycle cannot loop.
   - Collect every entry whose tag is 51009. Refuse unless there is **exactly one**.
     Refuse if its type is not 7 (UNDEFINED).
   - Rewrite that entry's two tag bytes to 51010, in the file's byte order, at the
     entry's own offset. Nothing else in the file changes.
   - Print the IFD (by its offset) and the entry offset, as now.
   - Refuse loudly on a malformed header or IFD.
2. **The output directory is created only after the in-repo check.**
   - Resolve the absolute path without creating anything: if `<out_dir>` exists, `cd`
     into it with `pwd -P`; otherwise its parent must exist, resolve that with `pwd -P`
     and append the last component.
   - Refuse an in-repo path, then `mkdir -p`.
   - A refused in-repo path leaves no directory behind.
3. **Set means must run.** In `tests/gain_map_corpus.rs`, when `IRRADIANCE_SDK_STAGE2_DIR`
   is set (non-empty), a missing corpus file is a **failure** naming the file and both
   variables, not a skip. When it is unset or empty, behaviour is unchanged (visible
   SKIP). Put the decision in one helper shared by the three SDK tests, so it can be
   unit-tested without the environment: given "SDK set" and a path that does not exist,
   it panics; given "SDK unset", it returns `None`. The non-SDK tests in the file keep
   their skip.
4. **The ledger records the licence finding.** In `docs/provenance-ledger.md`, the two
   SDK rows (and the class-4 note if the ledger has one for the SDK) gain one sentence:
   the 1.7.1 zip (with its sha256) ships no licence agreement document, only the
   per-file notice; irradiance reads the SDK's source and runs `dng_validate` locally,
   copies no SDK code, and redistributes nothing from it (DEC-069 §6). Replace the
   script header's "The SDK's licence is the Adobe DNG SDK licence agreement shipped in
   the zip" with the accurate statement. Give no legal opinion beyond those facts.
5. **The monorail triage refresh** (`feedback/2026-11-11-monorail-000-requests.md`):
   - `status: addressed` in the front matter (the template's vocabulary is
     `open | addressed | deferred`);
   - item 5: released in 0.2.2 (crates.io, 2026-11-17);
   - item 8: SPEC-075 shipped 2026-11-19, released in 0.2.3 (crates.io, 2026-11-20);
   - a closing line under the table: every item has shipped and been released; the
     closing rule is met on 2026-11-20; later requests from the consumer get a new
     feedback file.

   Touch nothing else in the file; it is the consumer's record.
6. **FU-2, a colour fixture with the maximum black not last.** In
   `tests/black_normalisation.rs`, add a test that repeats the shifted-uniform
   equivalence (`colour_scales_by_white_minus_the_maximum_black`'s shape) with the
   2048 black at **each** slot 0, 1, 2 and 3 in turn, on all four colour entries
   (`u16`, `f32`, and both threaded), and the same for mono `PerPosition`. Add one
   general-path case (a 4×4 `BlackLevelRepeatDim` pattern, which takes the general
   normaliser) with its maximum black at pattern index 0.

## Inputs

- `scripts/sdk-stage2.sh`, `tests/gain_map_corpus.rs`, `tests/black_normalisation.rs`,
  `tests/colour_black_pattern.rs` (for the general-path fixture shape),
  `docs/provenance-ledger.md`, `feedback/2026-11-11-monorail-000-requests.md`.
- DEC-069 §6 (the SDK reference), DEC-070 (the rule FU-2 tests), SPEC-078's verify
  handback (HANDOFF-168, FU-2).
- The local SDK at `~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/` (the zip in
  `dl/`, the binary in `build/`), and the corpus in place.

## Outputs

- **Modified:** `scripts/sdk-stage2.sh`, `tests/gain_map_corpus.rs`,
  `tests/black_normalisation.rs`, `docs/provenance-ledger.md`,
  `feedback/2026-11-11-monorail-000-requests.md`.
- **No `src/` change**, no pin change, no `DEVELOP_VERSION` change, no CHANGELOG entry
  (nothing user-visible changes).

## Acceptance Criteria

- [ ] **AC1. The IFD walk.** Run cold into a new scratchpad directory, the script
  writes all four reference files. Each Pixel's rewrite lands at the same byte offset
  the old pattern search found, and every `pre`/`post`/`stage2` TIFF is byte-identical
  to the old script's (record the sha256s). The SDK cross-check passes against them
  with SPEC-078's numbers unchanged.
- [ ] **AC2. The walk refuses what it should and accepts what the pattern refused.** On
  scratch copies (outside the repo), record each outcome:
  - a copy with a second, decoy copy of the 4-byte pattern planted in pixel data: the
    old script refuses, the new one succeeds and rewrites only the IFD entry;
  - a copy with the 51009 entry's tag already changed (no `OpcodeList2`): refused;
  - a copy with a second 51009 entry (for example an unrelated IFD0 entry's tag and
    type rewritten to 51009 and 7): refused;
  - a truncated file and an IFD offset past the end: refused, no Perl warning or hang;
  - an IFD cycle (a `NextIFD` pointing back at IFD0): refused.
- [ ] **AC3. Directory order.** With `<out_dir>` a non-existent path inside the repo,
  the script refuses and the directory does not exist afterwards. A non-existent path
  outside the repo is created. A path whose parent does not exist is refused with a
  clear message.
- [ ] **AC4. Set means must run.** A unit test in `tests/gain_map_corpus.rs` shows the
  helper panics, naming the file and `IRRADIANCE_CORPUS_DIR`, when the SDK is set and
  the corpus file is missing, and returns `None` when the SDK is unset. Run the real
  thing once by hand too: the SDK variable set and `IRRADIANCE_CORPUS_DIR` pointed at
  an empty directory makes the three SDK tests fail; unset, they print SKIP and pass.
- [ ] **AC5. FU-2.** The new tests pass, and three colour mutants each go red on
  **tier A** (`just test` with no corpus): `B` taken as the last slot's black, `B` taken
  as the first slot's black, and `B` taken as the general pattern's first value. Record
  which tests go red for each, with the unmutated control green first.
- [ ] **AC6. Records.** The ledger and script header state the licence finding as in
  Design §4; the feedback file matches Design §5; `just validate` parses both.
- [ ] **AC7. Gates.** `just test` with the corpus, both with and without the SDK
  variable; `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `doc-check`,
  `cost-audit`, `gate-audit`, `name-length-check`, `decisions-index --check`; CI green.
  CI has no SDK, so the SDK tests skip there visibly.

## Failing Tests

- `tests/gain_map_corpus.rs`: the set-means-must-run helper's two unit tests (AC4).
- `tests/black_normalisation.rs`: the maximum-black-at-each-slot colour and mono tests,
  and the general-path maximum-at-index-0 test (AC5).

The script's AC1–AC3 are shell scenarios with recorded outcomes, not cargo tests: the
script needs the local SDK, which CI never has.

**Rule-clause map.** DEC-070 §1 "`B` is the maximum over the blacks the path applies"
maps to AC5's three mutants on the colour slot normaliser (slots at each position) and
the general pattern normaliser (index 0); each must be red on tier A, the path CI runs.
DEC-069 §6 "tests that read SDK outputs skip visibly unless the variable is set" maps to
AC4 on both arms.

## Non-Goals

- **FU-1** (a colour levels refusal). It is a behaviour change and moves to the next
  project at PROJ-004's close.
- Any library code, pin or `DEVELOP_VERSION` change.
- Running the SDK in CI, vendoring it, or committing its outputs or scratch copies.
- The brief's stage count and the STAGE-025 reflection: the stage close does them.
- `BlackLevelDeltaH/V` (DEC-070 §5).

## Notes for the Implementer

- **Byte-identity is the check for AC1.** Keep the old script's outputs from one cold run
  before changing it, then compare.
- **Every scratch copy and SDK output stays outside the repo.** The decoy copies are
  24 MB each; delete them after.
- **Mutants** each get their own `CARGO_TARGET_DIR`; read each panic message.
- **Plain-prose commit bodies**, no code tokens. Never `git add Cargo.lock`. Stage
  explicit paths.

---

## Reflection

*Appended during **ship**.*

1. **What would I do differently next time?**
   — Check each mutant against its fixture at design. AC5's third mutant ("`B` = the
   pattern's first value") could not go red on Design §6's index-0-only fixture, because
   there the first value *is* the maximum. The builder caught it and put the maximum at all
   16 indices; the verifier's own last-value mutant is caught only by that test. The bundle
   took about 21.0M tokens against a 15M estimate: S, at its top.

2. **Does any template, constraint, or decision need updating?**
   — No DEC changes. Two process points for the stage close: `just validate` does not
   parse `feedback/` or `docs/` front matter, so an AC that says "validate parses it" is
   unenforced (process debt), and the SDK script finds the repo root from its own
   location, which misjudges a copy run from another worktree (carried, pre-existing).

3. **Is there a follow-up spec I should write now before I forget?**
   — None in PROJ-004. FU-1 (a colour levels refusal) moves to the next project at
   PROJ-004's close, with STAGE-017 and STAGE-004.

4. **Where was the worst defect caught?**
   — build
   *(A design defect, the third mutant that could not fail on the designed fixture,
   caught and corrected by the builder. The verify found nothing to fix; the ledger's PDF
   count and duplicated paragraph were fixed at ship.)*

5. **What can a user do now that they couldn't before?**
   — none (library behaviour is unchanged). For maintainers: the SDK recipe accepts a file
   with a stray `OpcodeList2` byte pattern and refuses malformed or cyclic IFDs, a
   deliberate SDK run can no longer pass with nothing compared, and a "`B` = last slot"
   colour bug now fails in CI instead of only on the corpus.
