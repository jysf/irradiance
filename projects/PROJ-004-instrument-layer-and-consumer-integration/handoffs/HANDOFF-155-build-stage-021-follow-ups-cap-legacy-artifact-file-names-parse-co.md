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
  id: HANDOFF-155
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-18
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-074

project:
  id: PROJ-004
  stage: STAGE-022
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
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: transcript                # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 11423772           # REAL combined count — what cost-audit reads
  estimated_usd: 7.99              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 15
  branch: claude/spec-074-build
  pr: https://github.com/jysf/irradiance/pull/259
  completed_at: 2026-11-18         # YYYY-MM-DD
  notes: "Transcript e4d08c12-5435-4d9a-a32a-c561d7ebd7d4.jsonl, 76 message.ids deduped (max output each), no subagents: input 160, output 36,858, cache read 11,242,140, cache write 144,614 (all 1-hour). Opus 5.5 at $5 in / $25 out, reads x0.10, 1-hour writes x2.0. Summed before the handback commit, so a floor; re-sum at ship."
  synced_at: 2026-11-18
---

# HANDOFF-155: BUILD: STAGE-021 follow-ups, bundled (SPEC-074)

## Delegation Summary

You are the **builder** for SPEC-074. Cut `claude/spec-074-build` from `origin/main`
**after** the design PR merges.

**The task**, two unrelated parts in one spec:
- **Part A (DEC-067):**
  - rename the 83 legacy generated artifacts whose slug exceeds 60 characters, and
    rewrite every reference to them;
  - add a CI-run `just name-length-check` (150 bytes per tracked path component);
  - add a `just semver-check` recipe that uses the crates.io baseline.
- **Part B:** make each Compression 7 strip or tile parse its SOF-3 header once.
  `C7_ALLOCATIONS_PER_CHUNK` goes from 10 to 5, with output and errors unchanged.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-074-stage-021-follow-ups-cap-legacy-artifact-file-names-parse-co.md`. Read all of it: the probe findings, A1–A3, B1–B3, AC1–AC7 and the
  Notes.
- **Decision:** `decisions/DEC-067-…` (the renaming rule, the gate, the semver
  baseline). Also DEC-047 (shape binding before decode), DEC-065 (the scan reader;
  untouched) and DEC-009 (red-proof, control first).
- **Code:**
  - `scripts/_lib.sh` `slugify` (read only; template-managed). Your rename must match it.
  - `src/ljpeg.rs`: `decode_into`, `decode_window_into`, `open_scan`, `parse_headers`.
  - `src/plane.rs`: `decode_unit`'s Compression 7 strip arm, and `decode_tile_row`'s
    `PlaneKind::Jpeg` arm.
  - `tests/plane_units_alloc.rs`: `C7_ALLOCATIONS_PER_CHUNK` and its comment. The tally
    is per thread since PR 243; keep it so.
  - `.github/workflows/ci.yml` (`cost-data` job), `app.just`, AGENTS.md §6, CHANGELOG.md.
- **What the design probe already showed** on `28ecd5d`:
  - `cargo semver-checks --baseline-rev v0.2.2` fails at the clone on HANDOFF-064's
    temporary name (file name plus about 45 bytes).
  - `cargo semver-checks` with no baseline flag runs clean: 202/202.
  - A throwaway one-parse version (two `pub(crate)` "decode from parsed" functions, called
    from both plane paths) measured 20 allocations for the 4-strip fixture, and with the
    pin at 5 `plane_units_alloc`, `plane_oracle` and `plane_units` were green. It was
    reverted; write your own.
- **Toolchain:**
  - AGENTS.md §5–6. `just test` runs the `corpus` profile plus the debug `*_alloc`
    binaries in about 3 minutes (SPEC-072).
  - For tier B, set
    `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`.
    Use the files in place and never copy them into the repo.
  - Do red-proofs on an **archived copy** with its own target dir, or a local commit you
    never push.

## Expected Deliverables

1. **Commits in this order:**
   1. Part B (`src/`, the test pin, CHANGELOG);
   2. the renames and reference rewrites alone;
   3. the gate, the recipes, the CI step and AGENTS.md.
2. **Part B evidence:** AC5's two red-proofs (the double parse gives 40 against 20; one
   extra allocation goes red), and AC6's oracles plus `just ljpeg-timing` medians.
3. **Part A evidence:**
   - **AC1:** no slug over 60 characters, and spec and timeline pairs share a slug.
     Include a `git log --follow` sample.
   - **AC2:** no old name left except the listed history quotes; `validate`,
     `decisions-index --check` and `cost-audit` are green.
   - **AC3:** `cargo semver-checks --baseline-rev HEAD` gets past the clone, and
     `just semver-check` is clean.
   - **AC4:** `name-length-check` is red before the renames (the 13 names over 150
     bytes), and green after. On a 151-byte name it is red; at 150 it is green.
4. **Gates (AC7):** all green with the corpus env set:
   - `just test`
   - `lint-ci`
   - `lint-no-allow`
   - `lint-fuzz`
   - `fmt-check`
   - `validate`
   - `doc-check`
   - `deny`
   - `msrv`
   - `cost-audit`
   - `gate-audit`
   - `name-length-check`
5. **PR.** Open it against `main` from `claude/spec-074-build`, referencing SPEC-074 and
   HANDOFF-155. Confirm CI is green on its head; the macOS determinism leg is
   non-blocking (PR 257).

## Out of Scope

- `slugify`, `SLUG_MAX` or any template-managed script.
- Renaming DECs or other hand-named files.
- Old tags or history.
- A semver CI gate.
- Any change to the scan reader, decode output, errors or public items.

If one of these seems necessary, stop and say so in the handback.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message **bodies** are plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Stage explicit paths.
- Never commit corpus bytes (DEC-003).
- **If an existing test or oracle goes red, the change is wrong.** Fix the change, never
  the test. The only permitted test edit is `C7_ALLOCATIONS_PER_CHUNK` and its comment.
- **A rename moves the file only.** Do not edit a renamed file's front matter, cost or
  history while renaming. Reference rewrites in other files are part of the move.
- **Fill the `handback:` front matter**, including a real `tokens_total` re-summed from
  your own transcript:
  - `basis: transcript`;
  - find the transcript by your scratchpad UUID;
  - dedupe by `message.id`, taking the max output per message;
  - opus-5-5 rates.

  Fill the `## Handback` section below before reporting done.

## Handback

*Filled in by the receiving agent. The orchestrator does not reconstruct any of
this — it transcribes it. The reflection questions are part of completion.*

### Execution notes

- **Branch / PR:** `claude/spec-074-build`, https://github.com/jysf/irradiance/pull/259
- **Completed at:** 2026-11-18
- **Commits:** `74021de` (Part B), `c37a3b2` (the renames only), `9f3a724` (gate, recipes, CI step, AGENTS.md), then this handback commit.
- **All acceptance criteria met?** Yes, with one caveat on AC2's `id-audit` (below).
  - **AC1:**
    - 384 artifact files scanned, 0 slugs over 60.
    - 74 spec+timeline pairs, 0 slug mismatches.
    - All 83 new slugs match the real `slugify` (0 mismatches).
    - `git log --follow` on the renamed HANDOFF-064 shows `c37a3b2` and then `4afc018` (PR 37).
  - **AC2:**
    - `git grep -F` finds 0 hits for each of the 83 old names.
    - No history quote had to be kept: the crash path quoted in SPEC-073, HANDOFF-153, SPEC-074 and DEC-067 is abbreviated with `…`, so it never matches a full old name.
    - `validate`, `decisions-index --check` (67) and `cost-audit` are green.
    - `just id-audit` is red, but it was already red on main. It scans all local worktrees (92) and refs (448), and stale worktrees still hold the old names: DEC-028 and older HANDOFF slug renames are already there. The renamed IDs add entries of the kind the script's own header says "still reports, until that branch is merged or deleted". It is not a CI gate.
  - **AC3:**
    - `cargo semver-checks --baseline-rev HEAD` gets past `Cloning HEAD`: 202 pass, 58 skip.
    - `just semver-check` against crates.io 0.2.2: 202 pass, 58 skip.
  - **AC4** (scratch clone, control first):
    - Green on the renamed tree and with a tracked 150-byte name.
    - Red, naming it, on a 151-byte name.
    - Red on the 13 names over 150 bytes before the renames.
    - The step is in CI's `cost-data` job.
  - **AC5:** the pin is 5 and green. The red-proofs ran on `git archive` copies, each with its own target dir:
    - the double-parse plane.rs gives 40 against 20;
    - one extra allocation per chunk gives 24 against 20.
  - **AC6:**
    - Every plane and Compression 7 oracle is green and unedited, with 12/12 corpus files present.
    - `ljpeg-timing` medians: K-3 III 107.3, M Typ 246 93.7, Pixel 4a 61.1 ms. SPEC-073 recorded 104.4 / 92.0 / 62.4, so there is no measurable change.
  - **AC7:**
    - `just test`: 710 passed, 0 failed, 1 ignored, 40 binaries, 2 min 59 s.
    - Green: `lint-ci`, `lint-no-allow`, `lint-fuzz`, `fmt-check`, `validate`, `doc-check`, `deny`, `msrv`, `cost-audit`, `gate-audit` (now 12 scripts, including the new one), `gate-audit-red-proof` and `name-length-check`.
    - CI on the PR head: see the PR checks.

### Cost self-report

- **Tokens (total):** 11,423,772 (a floor, summed before this commit)
- **Estimated USD:** 7.99
- **Duration (minutes):** about 15 of agent wall time, plus background test runs
- **Source of the number:** the transcript `e4d08c12-5435-4d9a-a32a-c561d7ebd7d4.jsonl`:
  - 76 `message.id`s, deduped, taking the max output for each;
  - no subagents;
  - opus-5-5 rates, with 1-hour cache writes.

### Drift and new artifacts

- **New decisions emitted:** none (DEC-067 was written at design).
- **Deviations from spec:**
  - The pin's assertion message in `tests/plane_units_alloc.rs` said "the two header parses per chunk"; it now says one. This is the pin's own failure text, next to the permitted constant and comment edit.
  - `open_scan` keeps its name with a new signature `(data, entropy_at)`, because `tests/plane_tiles.rs` uses `fn open_scan` as the end anchor of its no-allocation source scan. The two new functions sit before it, so they are inside that scan.
  - `name-length-check` gets no separate red-proof script, since the spec lists one script. The red-proof is recorded here and in the PR body.
- **Follow-up work identified:**
  - `id-audit` cannot pass on a machine with many stale worktrees. A `--this-checkout`/`--refs-only` mode, or pruning merged worktrees, would make it usable as AC evidence. This is template-side, so it is not filed here.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — AC2 lists `id-audit` "(if it exists)" as a pass condition. It exists, but it reads every local worktree and ref, so it was red before this spec and cannot be green after a rename until old worktrees go away.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Yes: `tests/plane_tiles.rs` anchors a source scan on `fn open_scan`, so renaming or moving that function would break an unrelated test. Also, `gate-audit` requires a new gate to assert its count in shell, not only in an embedded program.

3. **If you did this task again, what would you do differently?**
   — I would give each red-proof copy its own `CARGO_TARGET_DIR` from the start. A shared one reused a stale build and showed the wrong count (40 instead of 24) on the second red-proof until I re-ran it.
