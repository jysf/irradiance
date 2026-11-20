---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. With build and verify running on different
# agents you get TWO handoffs per spec (HANDOFF-N build, HANDOFF-M verify) —
# `handoff.cycle` is what distinguishes them.
#
# The `handback:` block below is the RETURN path and it is not optional: it is
# how cost gets into the spec without the orchestrator hand-counting anything.
# `just handback-sync SPEC-NNN` reads it and appends the cost session for you.
# Rationale + the full contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-165
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-19
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-077

project:
  id: PROJ-004
  stage: STAGE-024
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number,
# and say in `basis` how you got it:
#   Claude Code   → your session transcript, measured per docs/cost-tracking.md
#                   "Measuring a cycle": count each message.id once, take its
#                   largest output_tokens, include subagents/ (basis: transcript).
#                   Summing every line over-counts ~2×. `/cost` in the desktop
#                   Code tab reports context size, not spend.
#   API           → the `usage` objects, summed (basis: api-usage)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  agent: claude-opus-5-5                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 7457943            # REAL combined count — what cost-audit reads
  estimated_usd: 5.60              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 15
  branch: claude/spec-077-build
  pr: 272
  completed_at: 2026-11-19         # YYYY-MM-DD
  notes: "FLOOR, measured live from transcript 1ba3d26e-8545-47cd-abfd-84c9a0037c7d.jsonl (49 unique message.ids, all claude-opus-5-5, no subagents): input 102 + cache-creation 126,140 (1-hour) + cache-read 7,304,343 + output 27,358 (max per message), opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). The few turns after this write are not in it; re-sum at ship." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-19
---

# HANDOFF-165: BUILD: irradiance 0.2.3 release prep, Phase 1 (SPEC-077)

## Delegation Summary

You are the **builder** for SPEC-077, a release spec. **Start only after the STAGE-024
design PR has merged.** Cut `claude/spec-077-build` from that `origin/main`.

**The task:** Phase 1 of the 0.2.3 release:
- the semver check against crates.io 0.2.2;
- `version = "0.2.3"`;
- the CHANGELOG promotion;
- the release notes;
- README and crate-doc wording checks;
- the tier-B evidence run;
- the package guards;
- the filled Release Pre-Flight.

This is one CI-gated, fully reversible PR. **You never tag, publish or yank.**

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-077-irradiance-0-2-3-release-gainmap-and-black-level-fixes.md`.
  Read all of it: Release Scope, Build work 1–8, Release Notes, Release Pre-Flight and
  the Phase 2 runbook.
- **Precedent:** SPEC-073 (the 0.2.2 release) in `specs/done/`, with HANDOFF-153 (its
  build) and its pre-flight as filled. Follow its shape.
- **Decisions:**
  - DEC-006 (release specs, CHANGELOG promotion);
  - DEC-052 (the 0.2.0 release, the two-phase cut);
  - DEC-067 (`just semver-check` uses the crates.io baseline);
  - DEC-068 and DEC-069 (the behaviour changes the notes describe).
- **What's in the release:** CHANGELOG `[Unreleased]` as it stands on main, which covers
  STAGE-022 (SPEC-074), STAGE-023 (SPEC-075, SPEC-076) and STAGE-016 (SPEC-059,
  SPEC-060).
- **Toolchain:**
  - AGENTS.md §5–6.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Optional SDK cross-check: `scripts/sdk-stage2.sh`, with the `dng_validate` at
    `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
    Write its outputs to your scratchpad and point `IRRADIANCE_SDK_STAGE2_DIR` at them.
  - `cargo-semver-checks` is already installed.

## Expected Deliverables

1. **The Build work items 1–8 from the spec,** in one PR against `main` from
   `claude/spec-077-build`, with CI green. The diff is limited to:
   - `Cargo.toml` (the version line only);
   - `CHANGELOG.md`;
   - README and crate-doc wording, if any is wrong;
   - the spec (Release Notes, Release Pre-Flight, timeline);
   - this handoff.
2. **The semver-check output, verbatim, in the handback.** If it reports anything
   breaking, **stop**. Mark the spec blocked, add a question to
   `guidance/questions.yaml`, fill the handback and report. That is what 0.2.2 did.
3. **The evidence in the pre-flight:**
   - `just test` counts and time, with 12/12 present;
   - the timing examples' numbers;
   - the dry run's package line and `.cargo_vcs_info.json` sha1;
   - the SDK cross-check numbers, if you ran it.
4. **Gates,** all green:
   - `just test`, with the corpus;
   - `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`;
   - `validate`, `doc-check`;
   - `deny`, `deny-fuzz`, `msrv`;
   - `cost-audit`, `gate-audit`, `name-length-check`, `semver-check`;
   - `package-targets-check`.

## Out of Scope

- Any non-documentation library change.
- The tag, `cargo publish` and a GitHub release (the user's Phase 2).
- STAGE-016's carried follow-ups.
- A 0.3.0.

## Hard rules

- **Never** tag, publish, yank, force-push, merge, or enable auto-merge.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your
  own transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then
  fill the `## Handback` section.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-077-build`, https://github.com/jysf/irradiance/pull/272
  (unmerged). Commits: `cf8b252` the prep and the candidate; `289681f` the pre-flight;
  then the handback and timeline.
- **Completed at:** 2026-11-19
- **All acceptance criteria met?** Yes. Build work 1 to 8 are done, and every gate is
  green on the candidate. CI on #272 was still running when this was written.
- **No tag, no publish:** `git tag -l v0.2.3` is empty.

**The semver check, verbatim** (`cargo-semver-checks 0.51.0`, `just semver-check` on
candidate `cf8b252`):

```
cargo semver-checks
    Building irradiance v0.2.3 (current)
       Built [   1.130s] (current)
     Parsing irradiance v0.2.3 (current)
      Parsed [   0.008s] (current)
     Parsing irradiance v0.2.2 (baseline, cached)
      Parsed [   0.006s] (baseline)
    Checking irradiance v0.2.2 -> v0.2.3 (minor change)
     Checked [   0.026s] 202 checks: 202 pass, 58 skip
     Summary no semver update required
    Finished [   1.894s] irradiance
```

`cargo semver-checks check-release --baseline-version 0.2.2 --all-features` on the same
candidate: `202 checks: 202 pass, 58 skip`, `no semver update required`. Nothing breaking,
so nothing went to `guidance/questions.yaml`.

**Evidence on `cf8b252`:**
- `just test`: 12/12 present, 43 binaries plus doctests, 764 passed, 0 failed,
  1 ignored, 224 s; doctests 4 passed.
- Timings (ms, medians): unpack 9.9 / 1.7; ljpeg 103.9 / 90.2 / 61.1; orient warp-free
  O1 85, O5 111; post-warp O1 615.
- SDK cross-check: gain step max |d| 1 on all three Pixels; exact 99.9794% (4a),
  99.9672% (3a), 99.9664% (7 Pro). `gain_map_corpus` 8 passed.
- Dry run: `Packaging irradiance v0.2.3`, 19 files, 371.7 KiB compressed;
  `.cargo_vcs_info.json` sha1 `cf8b2529b6d6511fbf4ba67bbdfd961545130ba5`; dev-dependencies
  only `ssimulacra2` and `toml`.

### Cost self-report

- **Tokens (total):** 7,457,943 (a floor, measured live)
- **Estimated USD:** 5.60
- **Duration (minutes):** about 15
- **Source of the number:** the session transcript, deduplicated by `message.id` per
  docs/cost-tracking.md (`basis: transcript`); no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The design's Release Scope lists `InvalidBlackLevelPattern` among the new `Error`
    variants. It shipped in 0.2.2; only `TooManyGainMaps` and `GainMapImageWrongLength`
    are new. The refreshed release notes say so.
  - The design's notes said "the Pixel 7 Pro was black 0 and is now 1024" under output
    changes. The 7 Pro was refused for develop in 0.2.2 (its mandatory `GainMap`), so the
    notes put it under the files that now develop, and the merged CHANGELOG entry names it.
  - A crate-doc fix beyond the wording the spec named: the `develop_into` sentence "the
    tone-curve stage SPEC-019 will add" (a follow-up from SPEC-073's verify) is reworded.
  - The CHANGELOG gained a short 0.2.3 intro paragraph, matching 0.2.2's.
- **Follow-up work identified:** none new. The optional GitHub release on `v0.2.3` is part
  of the user's Phase 2.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing much. The spec's draft notes had two small factual slips (above), caught by
   checking the CHANGELOG.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No.

3. **If you did this task again, what would you do differently?**
   — Start the corpus `just test` as soon as the candidate commit exists and run the
   cheap gates alongside it, as this time; hold the timing examples until it finishes so
   they don't skew each other.
