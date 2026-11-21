---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes a separate implementer agent — context for the
# implementer lives in handoffs/HANDOFF-*.md, not in the spec itself.

task:
  id: SPEC-084
  type: story                      # epic | story | task | bug | chore
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
  id: PROJ-002
  stage: STAGE-014
repo:
  id: irradiance

handoff:
  from_agent: claude-opus-5-5  # from .repo-context tier_map.design (DEC-005)
  to_agent: claude-opus-5-5        # filled when HANDOFF is created (any agent — see docs/porting.md)
  created_at: 2026-11-21

references:
  decisions: [DEC-003, DEC-008, DEC-014, DEC-029]
  constraints: []                  # [constraint-id-1, constraint-id-2]
  related_specs: [SPEC-005, SPEC-010, SPEC-012, SPEC-016, SPEC-083]

# Blocking dependencies: specs that must SHIP before this one can start.
# Distinct from references.related_specs (informational). Feeds the ready-set
# (`just ready`) and safe fan-out. Optional; [] = no blockers.
depends_on: []                     # e.g. [SPEC-002]

# Fan-out lease — who/what holds this spec now (`just claim` / `just unclaim`).
# Advisory; null = free. The hard lock for parallel agents is the worktree/branch.
claimed_by: null

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-014's <capability>". Optional; null is acceptable.
value_link: "infrastructure: every harness claim STAGE-014 inherited is backed by a check that fails when it stops being true"

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
  tokens_estimate: null
  sessions:
    - cycle: build
      handoff: HANDOFF-176
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 8953639
      estimated_usd: 6.60
      duration_minutes: 20
      recorded_at: 2026-11-21
      notes: "TRUE cost, re-summed at ship from the whole build worktree transcript dir (59 unique message.ids, all claude-opus-5-5, no subagents): input 123 + cache-creation 137,361 (1-hour) + cache-read 8,782,714 + output 33,441 (max per message), opus-5-5 rates. The handback floor was 7,679,810 and $5.68. PR 298, build commit 2419b34, seven mutants red, CI 22 of 22."
    - cycle: verify
      handoff: HANDOFF-177
      agent: claude-opus-5-5
      interface: other
      basis: transcript
      tokens_total: 5390712
      estimated_usd: 4.20
      duration_minutes: 25
      recorded_at: 2026-11-21
      notes: "TRUE cost, re-summed at ship from the whole verify worktree transcript dir (41 unique message.ids, all claude-opus-5-5, no subagents): input 85 + cache-creation 100,444 (1-hour) + cache-read 5,267,597 + output 22,586, opus-5-5 rates. APPROVED at 42890c2, no verify fix; handback carried into PR 298 by fast-forward to 951ee28. Nine mutants red (two new). Two nits: the hand-written tool lists in preflight_summary, and the complete-corpus count printed on two lines. The handback floor was 4,004,160 and $3.17."
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-21
      notes: "Un-metered main-loop orchestration (frame in PR 296, design in PR 297); counted at stage grain in STAGE-014's orchestration_cost, not split per spec."
    - cycle: ship
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      recorded_at: 2026-11-21
      notes: "Un-metered main-loop ship; counted at stage grain in STAGE-014's orchestration_cost."
  totals:
    tokens_total: 14344351
    estimated_usd: 10.80
    session_count: 4
shipped_at: 2026-11-21
---

# SPEC-084: Harness and CI honesty: corpus-status tool status, req multi-value refusal, SUPPORTED_BITS enumeration, blocking macOS leg

## Context

STAGE-014 (PROJ-002, 0.2.x maintenance) second batch, its only spec. It bundles the
three harness findings that moved here from PROJ-001 STAGE-005 (SPEC-016's
pre-rescope AC1 to AC3, last measured at git 3238dcb, re-measured on `main`
2026-11-21 in the stage file) with two PROJ-004 carry-overs:

1. **The pre-flight overclaims.** `examples/corpus-status.rs` prints "no tier-B test
   will skip" when every corpus file is present. It never looks for `exiftool` or
   `dnglab`, and without them the tier-B oracle tests still skip.
2. **`req()` truncates a documented assumption.** `reading_from_fields` in
   `tests/support/tools.rs` keeps a required tag's first value (`.first()`). Its doc
   comment justifies that as true of today's corpus, and nothing fails if it stops
   being true. SPEC-005/FU-2 and SPEC-010/FU-2 left this open twice by writing the
   check narrower than the finding.
3. **`SUPPORTED_BITS` can outrun its tests.** The private constant in `src/plane.rs`
   lists 8, 12, 14 and 16. `tests/determinism.rs` now covers 8 and 12, but adding a
   fifth depth without a test fails nothing (SPEC-012/FU-1).
4. **The macOS arm64 determinism leg is non-blocking** (`continue-on-error` in CI)
   since GitHub's runner shortage on 2026-11-18. The PROJ-004 close found the revisit
   trigger met: the leg passed on PR 287 and later PRs. Signal
   `macos-arm64-runner-unavailable` asks for it to be blocking again.
5. **`sdk-stage2.sh`'s warning wording** for an unenterable worktree reads "refusing
   its recorded path as it is", which says the opposite of what the script does
   (SPEC-083 verify FU-3).

SPEC-016's AC4 (plane fuzz seeds reaching `SampleExceedsWhiteLevel`) is dropped:
DEC-029 made a sample above `WhiteLevel` legal and the error no longer exists.

## Goal

Every harness claim in the five items above is backed by a check that fails when the
claim stops being true. Tests, examples, scripts and CI only: no library behaviour
changes.

## Inputs

- **Files to read:**
  - `examples/corpus-status.rs`: the pre-flight `just test` prints first. Its
    "no tier-B test will skip" line checks the corpus only.
  - `tests/support/corpus.rs`: shared by the example (`#[path]`) and
    `tests/corpus_manifest.rs`; the new summary function goes here.
  - `tests/support/tools.rs`: `exiftool_available()`, `dnglab_available()` (the
    probes the tier-B tests already use to skip), `reading_from_fields` and its `req`
    closure (around line 286), `tri_state` (the optional fields' shape, DEC-014).
  - `tests/metadata_oracle.rs`: the existing tier-A fixture tests that call
    `reading_from_fields` (around line 519 and 574); the new `req` test sits beside them.
  - `src/plane.rs`: `SUPPORTED_BITS` (line 171, private), `unpack_packed` (the
    DEC-008 dispatcher: byte-aligned for 8/16, MSB-first bitstream for 12/14), and the
    `mod tests` block at the end of the file.
  - `.github/workflows/ci.yml`: the `determinism` job, its `continue-on-error`
    (line 181) and the comment above it (lines 172 to 177).
  - `scripts/sdk-stage2.sh`: lines 76 to 85, the unenterable-worktree branch.
  - `guidance/signals.yaml`: `macos-arm64-runner-unavailable`.
- **Measured at design (2026-11-21, `main` after PR 296):**
  - Both tools are installed on the maintainer's machine; neither is in CI. Tests
    that skip without `exiftool`: `metadata_oracle`, `colour_metadata_oracle`,
    `dng_writer` (one test). Without `dnglab`: those three plus `develop`,
    `perceptual_oracle`, `plane_oracle`, `warp`.
  - No corpus **sensor** IFD reports a multi-valued `ImageWidth`, `ImageHeight`,
    `BitsPerSample`, `Compression` or `PhotometricInterpretation`. Only the RGB
    preview IFDs (`BitsPerSample` `8 8 8`) do, and the oracle never reads those
    (`exiftool -n` over all nine corpus files, groups IFD0, SubIFD, SubIFD1,
    SubIFD2). So the stricter `req` must leave the live metadata oracle green.
  - The macOS arm64 determinism leg passed on the last 8 CI runs (main and four
    PR branches, through PR 296).

## Outputs

- **Files modified:**
  - `tests/support/corpus.rs`: a pure function
    `pub fn preflight_summary(present: usize, total: usize, exiftool: bool, dnglab: bool, corpus_env: &str) -> Vec<String>`
    returning the summary lines. It is the only place the "no tier-B test will skip"
    sentence is written.
  - `examples/corpus-status.rs`: includes `tests/support/tools.rs` by `#[path]`
    (as it already does `corpus.rs`), probes with `tools::exiftool_available()` and
    `tools::dnglab_available()`, and prints `preflight_summary`'s lines in place of
    its own two `println!`s. The per-file lines and the `sdk-stage2` line are unchanged.
  - `tests/corpus_manifest.rs`: the AC1 test.
  - `tests/support/tools.rs`: `req` accepts exactly one value; a present tag with
    zero or more than one value is `ToolError::Parse` naming the tag and the raw
    values. The doc comment above `reading_from_fields` drops the justification of
    truncation and states the rule.
  - `tests/metadata_oracle.rs`: the AC2 test.
  - `src/plane.rs`: the AC3 unit test in the existing `mod tests`. No non-test
    line in `src/` changes.
  - `.github/workflows/ci.yml`: delete `continue-on-error` from `determinism`, and
    replace the six-line comment with one or two lines saying the macOS leg is
    blocking again as of this spec (signal `macos-arm64-runner-unavailable`, closed).
  - `scripts/sdk-stage2.sh`: the message becomes
    `sdk-stage2: cannot enter worktree $wt; keeping its recorded path as a refused location`,
    and the comment's "kept as it is" becomes "still refused".
  - `guidance/signals.yaml`: `macos-arm64-runner-unavailable` to `status: done`,
    `last_touched` set, a dated note naming SPEC-084 and its PR.
- **New flags / options:** none.

## Acceptance Criteria

No numeric tolerance is checked here. The one count, the macOS leg's 8 of 8 recent
green runs, is evidence for AC4, recorded at design, not a bound.

- [ ] **AC1, the pre-flight states only what it checked.** `preflight_summary`
      contains the sentence "no tier-B test will skip" if and only if
      `present == total` **and** `exiftool` **and** `dnglab`. Whenever a tool is
      false, a line names that tool as not on PATH and says the tests needing it will
      SKIP. A missing corpus file keeps today's MISSING line with the env var name.
      `corpus-status` prints these lines, using the same two probes the tests use.
- [ ] **AC2, `req` enforces its single-value assumption.** A required tag with more
      than one value makes `reading_from_fields` return `Err(ToolError::Parse)` whose
      message names the tag and the values; a one-value tag still reads; an absent one
      is still the existing absent error. The live `metadata_oracle` stays green on
      the corpus (measured above: no sensor IFD is multi-valued).
- [ ] **AC3, `SUPPORTED_BITS` cannot outrun its tests.** A unit test holds one
      hand-computed case per depth and asserts (a) the set of depths in its table
      equals the set in `SUPPORTED_BITS`, enumerated from the constant, and (b) each
      case unpacks through `unpack_packed` to its expected samples. Adding a depth to
      the constant without a row fails (a); removing one fails (a); breaking a depth's
      decode fails (b).
- [ ] **AC4, the macOS determinism leg blocks again.** `determinism` has no
      `continue-on-error`; the build PR's CI shows the macOS leg run and pass; the
      signal is `done`.
- [ ] **AC5, `sdk-stage2.sh`'s warning says what it does.** The exact message in
      Outputs; `bash -n` and `shellcheck` (if installed) clean; no other line of the
      script changes behaviour.
- [ ] **AC6, every check is watched red.** For AC1, AC2 and AC3, apply the fault
      the criterion names and show the new test failing, then restore it: file
      changed, compiled, and output changed. Mutants, at minimum:
      - AC1: `preflight_summary` ignores `exiftool` (prints the sentence when it is false).
      - AC1: `preflight_summary` ignores `dnglab`.
      - AC2: `req` back to `.first()`.
      - AC3(a): add `10` to `SUPPORTED_BITS` (array length 5).
      - AC3(a): delete the 14-bit row from the test table.
      - AC3(b): flip one expected sample in the 12-bit row, or break `low12`.

      AC4 and AC5 have no meaningful mutant (a runner outage cannot be staged; a
      message has no falsifier); record them as such.
- [ ] **AC7, gates.** `just test`, `just lint-ci`, `just validate`, `just doc-check`
      and the rest of the gate list the build runs, named; `cargo test --no-fail-fast`
      totals; the tier-B suites run with `IRRADIANCE_CORPUS_DIR` set and both tools
      present (the metadata oracle in particular, for AC2); CI green on the PR head.

## Failing Tests

- **`tests/corpus_manifest.rs`**
  - `the_preflight_does_not_promise_what_it_cannot_see`: over all eight
    (missing in {0, 1}) by exiftool by dnglab combinations, asserts the sentence
    appears only for (0, true, true), and that each false tool is named in a SKIP
    line. Tier A: no corpus, no tool.
- **`tests/metadata_oracle.rs`**
  - `a_multivalued_required_tag_does_not_truncate_to_its_head`: fields with
    `BitsPerSample` `[16, 16, 16]` and the other required tags single-valued give
    `Err(ToolError::Parse)` naming `BitsPerSample`; the same fields with `[16]` give
    `bits_per_sample == 16`. Tier A.
- **`src/plane.rs` (`mod tests`)**
  - `every_supported_bit_depth_is_exercised`, cases (hand-computed at design):
    - 8-bit, either order, bytes `01 02 03 04` to `[1, 2, 3, 4]`
    - 12-bit, bytes `AB CD EF` to `[2748, 3567]` (`0xABC`, `0xDEF`, MSB-first)
    - 14-bit, bytes `00 04 00 20 00 FF FF` to `[1, 2, 3, 16383]` (one whole 7-byte group)
    - 16-bit little-endian `34 12` to `[4660]`, and big-endian `12 34` to `[4660]`

    The depth-set assertion compares sorted, deduplicated depths from the table with
    `SUPPORTED_BITS`, so the 16-bit row's two byte orders count once.

**Rule-clause map.** DEC-008's two paths: the byte-aligned path (8, 16; both byte
orders at 16, where order is observable) and the bitstream path (12, 14, through
`unpack_groups12`/`unpack_groups14`, the path a whole group reaches) each have a row
above. DEC-014's tri-state is unchanged for optional fields; AC2 brings `req` to the
same "do not keep a truncated head" rule, and its test reaches `reading_from_fields`,
the code a live `exiftool_reading` runs. DEC-029 (a sample above WhiteLevel is legal)
is why the old AC4 was dropped; nothing here touches it.

## Non-Goals

- Any `src` behaviour change, pixel change or `DEVELOP_VERSION` bump. A
  `#[cfg(test)]` unit test inside `src/plane.rs` for item 3 is in scope.
- A release. Nothing here is user-facing.
- Making `exiftool` or `dnglab` required, or installing them in CI.
- Reopening the dropped AC4, or adding fuzz targets.
- Any other CI change beyond dropping the macOS leg's `continue-on-error` and its
  comment.

## Notes for the Implementer

- Commit-message bodies in this repo are plain prose: no code tokens. PR bodies may
  have them.
- `corpus-status` runs before the suite, and `cargo test` hides a passing test's
  output; that is why the summary has to be printed by the example, and tested as a
  pure function.
- Keep `preflight_summary` free of I/O so the test needs neither tool. The example
  does the probing.
- Don't make the tools required or install them in CI. CI keeps skipping the tier-B
  suites; it just stops being told nothing will skip.
- The AC3 test lives in `src/plane.rs` because `SUPPORTED_BITS` is private; don't
  make it `pub` to test it from `tests/`.

---

## Reflection

1. **What would I do differently next time?**
   — Re-measure inherited outlines before framing, as this one did. Of the four
   STAGE-005 outlines, one had been made obsolete by a later decision (DEC-029) and one
   was half overtaken; framing from the old text would have specced a test for an error
   that no longer exists.

2. **Does any template, constraint, or decision need updating?**
   — No template change. The one gap both cold sessions named, the hand-written
   `NEEDS_EXIFTOOL` / `NEEDS_DNGLAB` lists in `preflight_summary`, is recorded as
   signal `preflight-tool-lists-are-hand-written` (watch, low priority).

3. **Is there a follow-up spec I should write now before I forget?**
   — No. The two verify nits are below a spec: the tool lists (the signal above) and
   the complete-corpus count printed on two lines, to fold into any later edit of
   `preflight_summary`.

4. **Where was the worst defect caught?**
   — design

5. **What can a user do now that they couldn't before?**
   — none (contributor-facing only). Before, `just test` promised "no tier-B test will
   skip" with exiftool or dnglab missing, a multi-valued required tag was silently
   truncated, and a new bit depth needed no test; after, each of those fails or is
   named, and the macOS arm64 determinism leg blocks again.
