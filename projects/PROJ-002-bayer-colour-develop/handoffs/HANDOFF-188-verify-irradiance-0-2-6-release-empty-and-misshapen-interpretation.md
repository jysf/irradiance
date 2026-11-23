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
  id: HANDOFF-188
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-23
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-090

project:
  id: PROJ-002
  stage: STAGE-030
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
  agent: claude-opus-5-5           # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved                # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 10944131           # REAL combined count — what cost-audit reads
  estimated_usd: 8.71              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 40
  branch: claude/spec-090-verify-base
  pr: 322
  completed_at: 2026-11-23         # YYYY-MM-DD
  notes: "FLOOR from this session's own transcript (0b33b4e6-1c9e-4a72-ae09-f8f167ef5ebd.jsonl, found by the scratchpad UUID; 59 unique message.ids, all claude-opus-5-5, no subagents), deduped by message.id with the largest output_tokens: input 117 + cache-creation 197,507 (1-hour) + cache-read 10,690,847 + output 55,660, opus-5-5 rates (in 5, out 25 per M; cache read 0.1x, 1-hour write 2x). Measured before the handback commit, so the last turns are not in it. Duration is an estimate."
  synced_at: 2026-11-23
---

# HANDOFF-188: irradiance 0.2.6 release: empty and misshapen interpretation tags refuse

## Delegation Summary

`claude-opus-5` (acting as `architect`) hands `SPEC-090` to `claude-opus-5`
(acting as `verifier`) for the **`verify`** cycle: a cold review of PR 322
(`release/0.2.6-prep`, head `0bec2de`), the Phase-1 prep of the 0.2.6 patch release.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-002-bayer-colour-develop/brief.md`
- **Stage:** `./projects/PROJ-002-bayer-colour-develop/stages/STAGE-030-the-0-2-6-release.md`
- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-090-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md`
  (Release Scope, build steps, Release Notes, internal record, Release Pre-Flight,
  Phase 2 runbook).
- **Build handback:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-187-build-irradiance-0-2-6-release-empty-and-misshapen-interpretation.md`
- **The feature being released:** `./projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-089-empty-and-misshapen-interpretation-tags-refuse-at-develop.md`
- **The model release:** `./projects/PROJ-002-bayer-colour-develop/specs/done/SPEC-088-irradiance-0-2-5-release-calibration-converged-weight-unhono.md`
- **The PR:** https://github.com/jysf/irradiance/pull/322
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-078` §7 — what SPEC-089 changed: empty, wrong-count and zero-denominator
  interpretation tags are malformed; no silent fallback in the colour code.
- `DEC-076` — the colour calibration rules whose "unused calibration" case §7.2 amends.
- `DEC-067` — the mechanical semver check against the crates.io baseline.
- `DEC-006` — the release spec and its pre-flight.

### Constraints that apply

- No `src/` change in a release spec: a library defect is reported, not fixed.
- The SDK lanes and the corpus are local only (DEC-069 §6); nothing from them is committed.
- Commit-message bodies are plain prose (the repo ruleset rejects code tokens).

### Prior related work

- `HANDOFF-187` — the build of this release (PR 322).
- `HANDOFF-186` — SPEC-089's verify.
- PR 319 — SPEC-089's code.

## Expected Deliverables

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Re-derive the changed-input list from `git diff v0.2.5 <head> -- src/` and probe it
  against both versions; compare with the CHANGELOG and the Release Notes.
- Re-run the evidence: `just semver-check`, `just test` with the corpus, `irr ifd`
  against `v0.2.5`, both SDK lanes.
- The package: `cargo publish --dry-run` on a clean worktree; package list equal to
  0.2.5's; `Cargo.toml` changes only its version.
- The pre-flight and Phase 2 runbook, including the annotated-tag check and whether
  the clean-host smoke check tells 0.2.6 from 0.2.5.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Any `src/` change.
- Merging PR 322, tagging `v0.2.6` and `cargo publish` (the user's Phase 2).
- Re-tagging `v0.2.5` as annotated (the user's call).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.**

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-090-verify-base`, fast-forwarded into
  `release/0.2.6-prep`, so it rides PR 322 (https://github.com/jysf/irradiance/pull/322).
- **Completed at:** 2026-11-23
- **All acceptance criteria met?** Yes. Every build claim reproduced on the PR head
  `0bec2de`; three gaps in the changed-input disclosure were found and corrected in
  the CHANGELOG and the notes (no `src/` change).
- **For `verify`:** ✅ APPROVED at `0bec2de` (library), with this branch's wording
  corrections to `CHANGELOG.md` and SPEC-090's Release Notes and pre-flight.

**Re-derived evidence (verify scratchpad, not the handback):**
- `just test`, `IRRADIANCE_CORPUS_DIR` and both SDK dirs set from fresh
  `scripts/sdk-colour.sh` and `scripts/sdk-stage2.sh` runs against the local SDK 1.7.1:
  "corpus: 12/12 present … no tier-B test will skip", both SDK lanes "will run";
  51 results, 859 passed, 0 failed, 1 ignored, exit 0.
- SDK colour `--nocapture`: maxima 6 / 6 / 1 / 2 / 5; all 90 deltas and five white
  points equal SPEC-088's table digit for digit. Stage-2 `--nocapture`: 11 passed,
  every figure equal to 0.2.5's table; all 7 TIFF sha256s equal HANDOFF-169's.
- `just semver-check` (cargo-semver-checks 0.51.0): "v0.2.5 -> v0.2.6 (minor change)
  … 202 checks: 202 pass, 58 skip … no semver update required".
- `irr ifd` built at a `v0.2.5` worktree and at the head, on all 11 held DNGs:
  byte-identical, exit codes included; `malformed_tags` `[]` except the K-3 III's `[50713]`.
- `cargo publish --dry-run` on a clean detached worktree of `0bec2de`: exit 0,
  "Packaged 19 files, 1.8MiB (397.7KiB compressed)", 51 "ignoring" warnings,
  `.cargo_vcs_info.json` sha1 `0bec2de…`, packaged `rust-version = "1.90"`.
  `cargo package --list` identical to `v0.2.5`'s (19 paths). `git diff v0.2.5 --
  Cargo.toml` is the version line only.
- `just decisions-audit --changed`: advisory only (DEC-052 et al. on `CHANGELOG.md`);
  nothing inconsistent.

**The changed-input list.** I read the whole `src/` diff (`src/color.rs`, `src/ifd.rs`)
and checked DEC-078 §7.1(a)'s full tag list against the readers: the tags the diff
does not touch (the arrays, `SampleFormat`, `LinearizationTable`, the interleave
factors, the version pair) already refused count zero at `v0.2.5`. Then one probe
source, built on path dependencies against a `v0.2.5` worktree and the head, wrote 73
fixtures with `tests/support/dng_writer.rs` plus 6 edited-`Sensor` cases, and ran
`develop_into` / `develop_color_into`, `demosaic_into` and `to_srgb_linear_into` on
each. Every class the build listed was confirmed, including both reclassified ones.
**Missed or misstated, now corrected on this branch:**
1. **An output change.** IFD0 `Orientation` `[3, 1]` beside a sensor-IFD `Orientation`
   6: 0.2.5 developed at 3, 0.2.6 develops at 6 (both `Ok`, different hash; mono and
   colour). The wrong-count read now errs, so the existing two-IFD recovery takes the
   other value. It follows DEC-078 §7.1's recovery rule, so it is not a defect, but
   "no output change for any file that develops" was false for it.
2. **`to_srgb_linear_into` new refusals.** A 9-entry `ColorMatrix1` on a 1-sample
   `LinearRaw` plane, or on a 4-entry-`CFAPlaneColor` plane, was transformed `Ok` by
   0.2.5 and is now `MalformedTag { tag: 50721 }`.
3. **Composer claim too broad.** An edited `Sensor` with an 8-entry `ForwardMatrix2` on
   the single-illuminant path still develops `Ok`, same output, on both versions:
   `cam_to_xyz_single_illuminant` never reads `ForwardMatrix2`. From a file the reader
   refuses it (`MalformedTag { tag: 50965 }`, probed).

Also confirmed by probe: the precedence rule (count-zero `WhiteLevel` plus a one-short
buffer: `DevelopBufferWrongLength` / `RgbBufferWrongLength` before, `MalformedTag`
now); count-zero `BlackLevelDeltaH/V` is `UnsupportedTag` on every entry; the mono
develop ignores count-zero colour tags; the 4-plane precedence example (its 0.2.5
error is `UnsupportedCfaPattern` when the pattern names the fourth plane, noted);
count-zero `AsShotNeutral`, a NUL-only signature and a zero-numerator `AnalogBalance`
unchanged.

**Pre-flight and runbook.** Correct and complete. The `[cut]` smoke case was run on
the real Canon EOS 5D Mark III: with `color_matrix_2` cut to 8 entries, `v0.2.5`
returns `Ok` and the head returns `MissingColorCalibration` with the destination
untouched, so it does tell the releases apart (`DEVELOP_VERSION` is `"8"` on both and
does not). Added one `[cut]` line: after the push, `git ls-remote --tags origin
'v0.2.6*'` must show the `^{}` line, since `git cat-file -t` checks only the local tag.

### Cost self-report

- **Tokens (total):** 10,944,131 (a floor; measured before this commit)
- **Estimated USD:** 8.71
- **Duration (minutes):** about 75 (estimate)
- **Source of the number:** this session's transcript, deduped by `message.id`
  (basis: transcript)

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - Wording only: `CHANGELOG.md` `[0.2.6]` and SPEC-090's Release Notes gain the three
    classes above; the internal record gains a "Cold verify" entry; pre-flight item 1
    `[cut]` gains the remote annotated-tag check.
  - The spec was at `cycle: build` when the verify started, so it was advanced build →
    verify → ship to record the verdict.
- **Follow-up work identified:**
  - **The single-illuminant path ignores `ForwardMatrix2` on a hand-built `Sensor`.**
    DEC-078 §7.2's list of fallbacks does not name it, so this is not a defect against
    §7.2's text, but a one-line validation in `cam_to_xyz_single_illuminant` would make
    the composer rule hold without an exception. A patch-lane candidate.
  - **DEC-078 §7.1 names only count-zero `Orientation` recovery;** the code also
    recovers a wrong-count one (with the output change above). Worth a sentence in the
    decision so the next release's list starts from it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The spec was still at `cycle: build`, so `advance-cycle … ship` alone would not
   have recorded a verdict. More substantively, "Newly refused: a wrong-count
   `Orientation`" read as complete; only probing the two-IFD case showed it can also
   change output instead of refusing.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — DEC-078 §7.1's recovery sentence is narrower than the code. A release spec's
   changed-input list should treat any "recovered from elsewhere" rule as a possible
   output change and probe it with differing values, not just presence.

3. **If you did this task again, what would you do differently?**
   — Start the probe from the test-only writer, as I did, but enumerate per *entry
   point* as well as per tag: the `to_srgb_linear_into` class surfaced only because
   every case ran all three entries. I would also start the SDK lanes and `just test`
   in the background first; they finished while I read the diff.
