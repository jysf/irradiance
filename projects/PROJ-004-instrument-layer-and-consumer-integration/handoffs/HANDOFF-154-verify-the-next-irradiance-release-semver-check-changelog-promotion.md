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
  id: HANDOFF-154
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-18
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-073

project:
  id: PROJ-004
  stage: STAGE-021
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
  tokens_total: 6654906            # REAL combined count — what cost-audit reads
  estimated_usd: 5.00              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 35
  branch: claude/spec-073-verify-a92262
  pr: 253                          # verdict commented on the build PR; no fix PR was needed
  completed_at: 2026-11-18         # YYYY-MM-DD
  notes: "APPROVED at 46b6fc2, post-release. FLOOR: measured live from transcript 1d5d6cf5-f9df-4a9f-ae14-d6372cb7907f.jsonl, deduped by message.id, max output per message, no subagents, opus-5-5 at 5/25 per MTok with 1-hour cache writes; re-sum at ship. duration_minutes is an estimate."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-154: VERIFY (post-release): the irradiance 0.2.2 release prep (SPEC-073)

## Delegation Summary

You are the **cold verifier** for SPEC-073. You did **not** build it. This verify is
**post-release**:
- the build PR **#253** merged as `46b6fc2`;
- the user tagged `v0.2.2` on that commit and **published 0.2.2 to crates.io on
  2026-11-17**.

So the question is not "should this merge" but **"is what we released correct and
honestly described?"** Return exactly one verdict.

- **What changed in #253** (`git diff 7cf03f2 46b6fc2`):
  - `Cargo.toml` (version 0.2.2);
  - `CHANGELOG.md` (`[Unreleased]` promoted to `[0.2.2] - 2026-11-16`);
  - `README.md` and `src/lib.rs` doc wording;
  - `guidance/questions.yaml` (the semver question, answered);
  - the SPEC-073 spec, its timeline, and the HANDOFF-153 handback.
- **Your branch:** your worktree is cut from `claude/spec-073-verify-base`, which is
  `46b6fc2` plus this handoff and the spec's `cycle: verify`. The orchestrator holds the
  ship and the STAGE-021 close separately; do not archive the spec.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-073-the-next-irradiance-release-semver-check-changelog-promotion.md`.
  Read the Release Scope (the design audit and the semver decision), the Release Notes,
  and the filled Release Pre-Flight.
- **Build handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-153-build-the-next-irradiance-release-semver-check-changelog-promotion.md`.
  Treat its claims as claims.
- **The semver question and the user's answer:** `guidance/questions.yaml`,
  `spec-073-semver-checks-flags-error-discriminants`.
- **Precedent:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-042-release-irradiance-0-2-0.md`.
- **Toolchain:**
  - AGENTS.md §5–6. `just test` takes about 3 min with the corpus.
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.

## The verify contract

### What to check
1. **The published crate is what was merged.**
   - `.cargo_vcs_info.json` inside
     `https://static.crates.io/crates/irradiance/irradiance-0.2.2.crate` carries the sha
     `46b6fc2…`.
   - The crate's file list matches `cargo package --list` at `46b6fc2`: no stray files,
     no corpus bytes, no `irr` binary target, and no self dev-dependency in the packaged
     `Cargo.toml`.
2. **The semver decision.**
   - **Re-run the check yourself** (`cargo-semver-checks` may already be installed; if
     it isn't, `cargo install cargo-semver-checks --locked`):
     `cargo semver-checks check-release --baseline-root <a git archive of v0.2.1>`,
     because `--baseline-rev` crashes on long filenames.
   - Confirm that the only failure is `enum_no_repr_variant_discriminant_changed` on
     `Error`.
   - **Independently confirm it cannot be observed downstream:** `Error` has
     data-carrying variants, no `#[repr]`, and is `#[non_exhaustive]`, so an `as` cast
     from another crate does not compile. Use your own throwaway crate depending on
     `irradiance = "=0.2.2"` from the registry, never committed.
   - If you find a real break, the verdict is ❌ REJECTED. Say what a dependent would
     hit; the user decides whether to yank.
3. **The CHANGELOG `[0.2.2]` section.**
   - Every entry that was under `[Unreleased]` at `7cf03f2` is present, in full.
   - The new `[Unreleased]` is empty.
   - Nothing claims something that is not in the code (spot-check five claims against
     `src/`).
4. **The release notes in the spec:**
   - the headline numbers match the CHANGELOG and timing evidence;
   - "Breaking / action required: none" is true;
   - every Known limitation still holds in the code.
5. **The README and crate-doc fixes are accurate:**
   - tiled planes are listed as supported;
   - the `validate` comment covers tiles;
   - the fuzz list includes `fuzz-ljpeg`;
   - the crate doc no longer says compressed planes are absent.

   Grep for any other stale claim (for example "not yet", "unsupported", "0.2.1").
6. **Gates on `46b6fc2`, with the corpus set:**
   - `just test`
   - `lint-ci`
   - `fmt-check`
   - `doc-check`
   - `deny`
   - `msrv`
   - `cargo publish --dry-run`

   The dry-run reports 0.2.2 and will say the version is already uploaded. That is
   expected; record it.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### How to return it

- **Approved, or a punch list of doc-only fixes:**
  - Commit the filled HANDOFF-154 handback and the timeline's verify line on a branch
    named `claude/spec-073-verify-<suffix>`, and push it.
  - Any doc-only fixes go in a NEW PR against `main` (for example a README sentence).
    **Never change the version, the tag or the published crate.**
  - Post the verdict as a comment on PR #253 with the branch name.
- **Rejected:** comment on #253 with what a dependent would hit, push the handback
  branch, and stop.

### Hard rules
- **Never** tag, move or delete a tag, `cargo publish`, `cargo yank`, or edit a GitHub
  release. Those are the user's.
- Never force-push, merge or enable auto-merge.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes (DEC-003).
- Stage explicit paths.
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section too.

## Out of Scope

- The SPEC-073 ship and the STAGE-021 close; the orchestrator has them.
- Any new feature, API change or version change.
- The `push`-trigger and filename-length follow-ups (already filed as signals).

## Return Criteria — how to hand back

**Completing this handoff means filling in the `handback:` front-matter block
above AND the `## Handback` section below.** An unfilled handback is an
incomplete cycle, and `just handback-sync` will tell the orchestrator so.

On success:
1. Fill the `handback:` front-matter — **including a real `tokens_total`**.
2. Fill the `## Handback` section (the reflection is part of it, not optional).
3. Set `handoff.status` → `completed` and `handback.status` → `completed`.
4. Open a PR (build) or return your verdict (verify).

If you cannot complete the task:
1. Fill the `## Handback` section with what was done and what blocked you.
2. Set `handoff.status` → `rejected`, `handback.status` → `blocked`.
3. **Still report your token usage** — blocked work costs money too.
4. Set the spec's `task.blocked: true` and add a question to
   `/guidance/questions.yaml`.

---

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-073-verify-a92262` (this handback and the timeline's
  verify line); the verdict is commented on https://github.com/jysf/irradiance/pull/253.
  No doc-fix PR was needed.
- **Completed at:** 2026-11-18
- **All acceptance criteria met?** yes
- **For `verify`:** ✅ **APPROVED at `46b6fc291adceefe89e5c34b9cd5ed60396ef5ed`**
  (post-release: `v0.2.2` and crates.io 0.2.2 both point at this commit).

**1. The published crate is what was merged.**
- `git rev-parse v0.2.2^{commit}` = `46b6fc2…` = the #253 merge commit.
- Downloaded `irradiance-0.2.2.crate` from static.crates.io. Its sha256
  `c425980f…01e7` equals the registry API checksum; published 2026-11-17T06:16Z,
  not yanked.
- `.cargo_vcs_info.json` carries `sha1: 46b6fc291adceefe89e5c34b9cd5ed60396ef5ed`,
  `path_in_vcs: ""`.
- File list (19 files) is identical to `cargo package --list` at `46b6fc2`, with an
  empty diff. Every shipped source file (`src/**`, README, both licences,
  `Cargo.toml.orig`) is byte-identical to `git show 46b6fc2:<path>`. There are no
  corpus bytes, tests, examples or process files.
- Packaged `Cargo.toml`: `[[bin]] irr` keeps `required-features = ["irr"]` (DEC-048).
  `src/bin/irr.rs` ships by design so the manifest stays self-consistent, but a
  default build or `cargo install` makes no `irr` executable. The dev-dependencies
  are only `ssimulacra2` and `toml`: the self dev-dependency is stripped.
  `[profile.corpus]` is in the manifest, and profiles are ignored for dependents.

**2. The semver decision.** I re-ran the check myself:
`cargo-semver-checks 0.51.0`,
`check-release --baseline-root <git archive v0.2.1>`.
- Result: 202 checks, 201 pass, 1 fail, 58 skip.
- The only failure is `enum_no_repr_variant_discriminant_changed` on `Error`, with 37
  variants shifted. This matches the build's log.
- Not observable downstream. I built a throwaway probe crate, not committed, on
  `irradiance = "=0.2.2"` resolved from the registry.
  - `Error::NoSensorIfd as i32` (a unit variant) fails with rustc
    `error[E0605]: non-primitive cast: irradiance::Error as i32`.
  - `core::mem::discriminant` only gives an opaque `Eq`/`Hash` value.
  - `Error` derives only `Debug`, which prints the variant name. There is no
    `PartialOrd`/`Ord`/`Hash` whose behaviour depends on the order.
- So the flag is a false positive, as the user judged. No yank is warranted.

**3. The CHANGELOG.**
- The `[Unreleased]` body at `7cf03f2` (186 lines) is in `[0.2.2] - 2026-11-16`
  verbatim. `diff` shows only the added five-line summary paragraph.
- The new `[Unreleased]` is empty.
- The summary paragraph's claims hold:
  - `v0.2.1..46b6fc2` changes `Cargo.toml` only in the version and a
    `[profile.corpus]`, with no dependency or feature;
  - `rust-version = "1.90"`;
  - `DEVELOP_VERSION` is `"4"`.
- I spot-checked more than five claims against `src/`, and all are present:
  - `develop_with_into`, `output_dimensions_with`;
  - `DevelopOptions` (`#[non_exhaustive]`) with `framing`, `apply_orientation`,
    `warp_alpha` and `black_levels`;
  - `scratch_len`, `develop_with_scratch_into`;
  - `Container::previews`;
  - `plane::units`, `unpack_unit_into`;
  - `WarpRect::lerp`;
  - all ten new error variants named in the notes;
  - the new `UnsupportedStripLayout` message (v0.2.1 said "tiled planes are
    unsupported"; 0.2.2 doesn't).
- The CHANGELOG date of 2026-11-16 is the prep date. The upload was 2026-11-17
  06:16 UTC, which is still 2026-11-16 in US time zones. Not an issue.

**4. The release notes in the spec.**
- The headline numbers match the CHANGELOG:
  - 239.0 → 9.7 ms (24.6×, "about 25×");
  - 8 units in 1.4 ms;
  - 692 → 103, 591 → 91 and 324 → 61 ms. That is 6.7×, 6.5× and 5.3×, so
    "about 6.5×" is the middle file;
  - warp-free O5–O8 at 2.5–3.5×, for example O5 332 → 95;
  - 1 allocation of 189 MB against 6 totalling 284 MB.
- The build's candidate timings in the internal record agree within a few percent.
- "Breaking / action required: none" is true:
  - the only semver flag is unobservable (above);
  - `Sensor` and `DevelopOptions` are `#[non_exhaustive]`;
  - there is no removed item.
- Every Known limitation still holds in the code:
  - `color.rs` says "No opcode is applied on this path" and calls
    `check_required_processing(…, DevelopPath::Colour)`;
  - unknown mandatory opcodes give `UnsupportedMandatoryOpcode`, and nothing
    applies `OpcodeList2`, so the Pixel GainMaps are refused;
  - a non-integral RATIONAL is the malformed-value path in `ifd.rs`;
  - `plane.rs`: "a Compression-7 strip … one unit each".
- No GitHub release exists for `v0.2.2`, nor for v0.2.0 or v0.2.1. The spec makes it
  optional (Phase 2, step 6), and it is the user's to create. I did not touch it.

**5. The README and crate docs.**
- The Tested-modes table lists uncompressed and Compression 7 tiles as ✔ supported.
- The `validate` comment says "strip or tile layout".
- The `just fuzz` list includes `fuzz-ljpeg`.
- The crate doc no longer says compressed planes are absent, and names PROJ-003 and
  PROJ-004.
- The new "not implemented" line (LJPEG layouts other than `Nf = 1`/`2`) matches
  `ljpeg.rs`.
- The README's install line `irradiance = "0.2"` resolves to 0.2.2.
- I grepped `README.md` and `src/*.rs` docs for stale claims ("not yet",
  "unsupported", tiles refused, "will add", old versions). I found one, and it
  predates this release: `develop_into`'s doc says "The tone-curve stage SPEC-019
  will add after this one is, for now, a no-op". The tone layer shipped long ago as
  the separate `tone` module.
  - It has been unchanged since `0ec3f55` and is also in v0.2.1.
  - It is not a wrong capability claim, since `develop_into`'s output IS final.
  - It is listed below as a follow-up, not a punch-list item.

**6. Gates** on this worktree, which is `46b6fc2` plus this handoff (no source
change), with `IRRADIANCE_CORPUS_DIR` set:

| Gate | Result |
|---|---|
| `just test` | green: corpus 12/12 present, 40 binaries, 710 passed, 0 failed, 1 ignored, doctests 4; 184 s |
| `just lint-ci` | green (pinned clippy 0.1.98) |
| `just fmt-check` | green |
| `just doc-check` | green: 2 surfaces, 0 rustdoc warnings |
| `just deny` | green (`licenses ok`; the usual unmatched-allowance notes only) |
| `just msrv` | green (1.90.0) |
| `cargo publish --dry-run` | green: packages and verifies irradiance v0.2.2, warns that "crate irradiance@0.2.2 already exists on crates.io index" (expected), aborts upload |

- CI on #253: 44 checks pass. The `CI` runs on `46b6fc2` concluded `success`.

### Cost self-report

- **Tokens (total):** 6,654,906, a floor measured live.
- **Estimated USD:** $5.00 (opus-5-5: $5/$25 per MTok, cache reads ×0.10, 1-hour
  writes ×2.0).
- **Duration (minutes):** about 35 (estimate).
- **Source of the number:** the transcript
  `1d5d6cf5-f9df-4a9f-ae14-d6372cb7907f.jsonl`, found by the scratchpad UUID.
  It was deduped by `message.id` with the max `output_tokens` per message. There
  are no subagents. Re-sum at ship.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - The `--baseline-root` archive was a full `git archive v0.2.1`, not the
    crate-files subset. Extraction hit no long-filename error, and the result is
    identical (201/202, same 37 variants).
- **Follow-up work identified:**
  - [XS, docs] Reword `develop_into`'s stale "tone-curve stage SPEC-019 will add …"
    sentence in `src/develop.rs`. It predates 0.2.2, and the fix lands in the next
    release's docs.
  - [user, optional] Create the `v0.2.2` GitHub release from the spec's Release
    Notes, down to the internal record. No release objects exist for any 0.2.x tag.
  - The build's suggestion stands: append new `Error` variants at the end of the enum
    so the semver lint stays quiet.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. Check 1 says "no `irr` binary target". The packaged manifest does
   declare `[[bin]] irr`, gated behind a feature (DEC-048). "No default-built `irr`"
   would be the precise wording.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Whether a GitHub release is expected post-cut. The spec marks it optional, but
   the release notes call themselves "the GitHub release text", and none exists.

3. **If you did this task again, what would you do differently?**
   — Run the gates strictly after `just test`, not alongside it. They contend for the
   cargo build lock, so running them together saved nothing and blurred the 184 s test
   timing.
