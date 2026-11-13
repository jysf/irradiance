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
  id: HANDOFF-134
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-12
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-063

project:
  id: PROJ-004
  stage: STAGE-019
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
  tokens_total: 6342969            # REAL combined count — what cost-audit reads
  estimated_usd: 5.01              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 75
  branch: claude/spec-063-verify-fb8ccb
  pr: https://github.com/jysf/irradiance/pull/215
  completed_at: 2026-11-12         # YYYY-MM-DD
  notes: "APPROVED at ae232c2 (PR 214 build head). tokens_total is a floor re-summed from transcript fad3eb49-ca46-4bac-837e-67350d3967aa.jsonl (41 unique message.ids, max output per id, no subagents) just before the handback commit; re-summed after the PR opened; it excludes only this final commit turn. Re-sum at ship."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-134: VERIFY: DevelopOptions + develop_with_into (SPEC-063)

## Delegation Summary

You are the **cold verifier** for SPEC-063. You did **not** implement it. Review it against
SPEC-063 and DEC-058 and return exactly one verdict.

- **The build:** **PR #214**, branch `claude/spec-063-build`, cut from `origin/main` at
  `0642e54`. When this handoff was minted its head was `ae232c2`, and CI was green on that
  head.
- **Your branch:** your worktree is cut from `claude/spec-063-verify-base`. That branch is
  the build head plus this handoff and the spec's `cycle: verify`, so the code under review
  is the build's own.

What the build delivers:
- `DevelopOptions`, `Framing`, `develop_with_into` and `output_dimensions_with` in
  `src/develop.rs`. A private `apply_options` rewrites the resolved `Geometry`, and
  `develop_into` / `output_dimensions` delegate to the new functions.
- `Error::UnsupportedWarpedFullActiveArea` in `src/lib.rs`.
- `tests/develop_options.rs` (13 tests, including the tier-B AC2 MD5 pins) and rows in
  `tests/error_kind.rs`.
- An option byte in the `develop` fuzz target, plus 8 new seeds.
- A CHANGELOG entry and a provenance-ledger row.

## What to read first

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-063-developoptions-develop-with-into-non-exhaustive-options-with.md`.
  Read all of it, especially §1 (the probe table and the MD5 pins, taken from main at
  3458854 **before** the change), Design §2–§7, AC1–AC13 and the red-proof list.
- **Decisions:**
  - `./decisions/DEC-058-developoptions-is-a-non-exhaustive-struct-whose-default-is-develop-into.md`;
  - DEC-024 (the warp clamps to the edge);
  - DEC-028 (`DEVELOP_VERSION`);
  - DEC-051 (`Error::kind` has no wildcard);
  - DEC-052 (`#[non_exhaustive]`);
  - DEC-012;
  - DEC-003.
- **Build handoff and handback:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/handoffs/HANDOFF-133-build-developoptions-develop-with-into-non-exhaustive-options-with.md`.
  Treat its claims as claims and check them yourself. Judge whether each declared
  deviation holds up:
  - **AC9's second half is unsatisfiable.** Orientation swaps width and height and cannot
    change a product, so the test instead pins that the buffer is accepted and receives
    the unswapped layout. Do you agree, and is that a sufficient substitute?
  - **No allocation before the refusal, except one.** The only allocation is the existing
    small `Vec<Opcode>` from `parse_warp_rectilinear`, which now runs earlier and only
    once under `FullActiveArea`. Confirm that it is parsed only once, and that no
    plane-sized allocation happens before the refusal.
  - **AC2 takes ~3 min in debug.**
- **Toolchain:** AGENTS.md §5–6 and `guidance/toolchain-brief.md`. `just test` already
  passes `--all-features`; add no flag. For tier B, set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`. Use the files in
  place; never copy them.

## The verify contract

### Acceptance criteria and gates
- Is every AC1–AC13 met by a test that would fail if the property broke?
- Is the build reflection honest?
- Run each gate yourself, with the corpus env set:
  - `just test`
  - `lint-ci`
  - `lint-no-allow`
  - `fmt-check`
  - `validate`
  - `cost-audit`
  - `gate-audit`
  - `doc-check`
  - `deny`
  - `msrv`
- Run `just decisions-audit --changed`.

### The behavioural surface (this is where defects escape)

**Byte identity is independent of the build.**
- Re-derive at least two of the §1 MD5s yourself: L1026016 (warped, Orientation 6) and
  one warp-free file. Build a few lines of standalone Rust in your scratchpad against the
  PR head, run `develop_into`, and MD5 the output as little-endian `u16`.
- Confirm that `git diff origin/main` shows no pin edited and `DEVELOP_VERSION` still `"4"`.
- Confirm that the AC2 test **ran**: no SKIP line, and the three Pixel files refused with
  identical text under both entry points.

**Validate, then override (DEC-058 §3).** Read `develop_with_into` and
`output_dimensions_with`.
- `resolve_geometry` must run unchanged before `apply_options`, on both paths.
- Mutate the code yourself so validation is skipped under each option, and confirm a test
  goes red.

**The refusal (DEC-058 §4).** Mutate each of these and confirm a named test goes red:
- refusal removed;
- refusal fires on an identity warp;
- refusal fires under `StandardCrop`;
- refusal placed after the `FixBadPixelsConstant` copy;
- refusal placed after a `dst` write.

Also confirm that, under `StandardCrop`, the opcode parse order and the error precedence
are exactly main's. Read the diff. A test showing it would be better than a reading;
judge whether one is needed.

**The relations are real oracles.**
- AC4 and AC6 compare against `StandardCrop` / Orientation-1 develops of a modified
  sensor. Check that the tests could not pass trivially: for example, both sides empty, or
  a fixture so uniform that a wrong crop gives the same pixels.
- Mutate `apply_options` so that:
  - FullActiveArea has an off-by-one origin;
  - FullActiveArea has the crop size swapped;
  - FullActiveArea uses the crop origin, not 0;
  - `apply_orientation: false` maps to orientation 2 rather than 1.

  Confirm each one goes red.

**Tier-B coverage.** Confirm that `FullActiveArea` is exercised on all three warp-free files
(K-3 III, L1000622, Typ 246), that the Q2M refusal is exercised, and that L1026016
`apply_orientation: false` runs through the warp path.

**API and error surface.**
- `DevelopOptions` derives `PartialEq` and **not** `Eq`/`Hash` (DEC-058 §1).
- Both new types are `#[non_exhaustive]`, with every field and variant documented, and the
  docs say the options are mono-only.
- An external test constructs options via `Default` plus field assignment.
- The new variant is in `Error::kind()` with no wildcard, and a test pins its `Display`
  naming both dimensions.
- CHANGELOG `[Unreleased]` names every addition and says the default output is unchanged.

**No regression.**
- `git diff origin/main -- tests/` changes no existing assertion except `error_kind`'s
  rows and counts. Account for every deleted line in `tests/error_kind.rs` and elsewhere.
- The `src/` diff adds no `Vec`, `vec!`, `to_vec`, `Box`, `String` or `format!` on the
  develop path.

**Fuzz.** Run the `develop` target for at least 60 s with the new seeds, and report the run
count.

**CI** must be observed green on the PR head, with `gh pr checks 214`.

### Verdict

Output exactly ONE of: ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED.

### Hard rules
- Never force-push, merge, enable auto-merge, tag or publish.
- Commit-message BODIES must be plain prose with no code tokens. PR bodies are fine.
- `Cargo.lock` is gitignored; never `git add` it. Never commit corpus bytes or derived
  pixels (DEC-003).
- Fill this handoff's `handback:` front-matter, including `verdict` and a real
  `tokens_total` re-summed from your own transcript. Set `basis: transcript`, dedupe by
  `message.id`, take the max output per message, and use opus-5-5 rates. Fill the
  `## Handback` section too.
- Open a NEW PR against `main` from `claude/spec-063-verify-<suffix>`. It carries the
  handback and any punch-list fixes, must be a superset of #214, and must reference
  HANDOFF-134, SPEC-063, DEC-058 and PR #214.

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

- **Branch / PR:** `claude/spec-063-verify-fb8ccb` (cut from `claude/spec-063-verify-base` at 9ef9954 = PR #214 head ae232c2 + this handoff). PR https://github.com/jysf/irradiance/pull/215 against `main`, superset of #214; it adds only this handback.
- **Completed at:** 2026-11-12
- **All acceptance criteria met?** yes. AC9's orientation half is unsatisfiable as written and its substitute is accepted (see Deviations).
- **For `verify`:** ✅ **APPROVED at ae232c2** (the build head of PR #214; no code change in this verify).

**What I checked myself (not transcribed from HANDOFF-133):**

- **Gates, with `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`:** `just test` exit 0: 29 binaries, **615 passed / 0 failed / 1 ignored** (pre-existing). Corpus status 12/12 present, zero SKIP lines, and `develop_into_output_is_unchanged_on_held_files` ran (>60 s in debug) and passed. `lint-ci`, `lint-no-allow`, `fmt-check`, `validate`, `cost-audit`, `gate-audit`, `doc-check`, `deny` and `msrv` all exit 0. `decisions-audit --changed` is advisory only and lists `src/develop.rs` / `src/lib.rs`; I judged them consistent with DEC-024, DEC-028, DEC-051, DEC-052 and DEC-058.
- **Byte identity, independent of the build.** A standalone scratchpad crate (path dependency on the PR head, release mode, `develop_into`, MD5 of the output as LE `u16` via the system `md5`, nothing written to the repo) re-derived **five** §1 pins, all equal:
  - L1026016 5584×8368 `2bc53a63…434b` (warped, Orientation 6);
  - K3III 6192×4128 `82d2d519…6b14`;
  - L1000622 5212×3468 `fddf0ea0…cc79`;
  - M2462362 5976×3992 `4aa013f6…cb28`;
  - 5G4A9394 5760×3840 `f533e405…94bc`.

  `git diff origin/main` edits no pin, and `DEVELOP_VERSION` stays `"4"`.
- **Validate, then override.** `resolve_geometry` runs unchanged, then `apply_options`, on both `output_dimensions_with` and `develop_with_into`. Under `StandardCrop` the opcode parse order is main's: the `OpcodeList3` parse moved into `non_identity_warp` but runs at the same point, after the `OpcodeList1` copy. A test already pins this (`develop_into` on a malformed `FixBadPixelsConstant` plus a warp still reports `MalformedOpcodeParams`), so no new test was needed. Under `FullActiveArea` the warp list is parsed exactly once, early, and the later parse is skipped.
- **My own 16 mutations** (scripted against `src/develop.rs`, release build with the corpus; each reverted, tree clean afterwards). Every one turned a named `tests/develop_options.rs` test red:
  - refusal removed → `full_active_area_with_a_warp_is_refused_before_any_write`;
  - refusal fires on an identity warp → `identity_warp_does_not_trigger_the_refusal`;
  - refusal fires under `StandardCrop` → AC2 `develop_into_output_is_unchanged_on_held_files` (tier B), AC5, AC6 Q2M;
  - refusal moved after the `FixBadPixelsConstant` copy → AC5 (the order probe);
  - refusal placed after a `dst` write → AC5 (sentinel);
  - validation bypassed (crop tags cleared under `FullActiveArea`, or orientation cleared under `apply_orientation: false`), separately in `output_dimensions_with` and `develop_with_into` (4 mutations) → `options_do_not_skip_tag_validation` each time;
  - `FullActiveArea` origin off by one → AC4(a), AC4(b), AC7;
  - crop size swapped → AC3, AC4, AC5, AC7;
  - crop origin kept instead of 0 → AC4(a), AC4(b), AC7;
  - `apply_orientation: false` maps to 2 → AC6 all-eight, AC6 Q2M, AC7, AC9;
  - default `FullActiveArea` → AC1, AC2 + 6 more;
  - `apply_options` skipped → AC3, AC4, AC5, AC6, AC7, AC9;
  - `dst` checked against the default dimensions → AC9 + 5 more.
- **The relations are real oracles.** Tier-A fixtures use distinct non-zero samples, identity levels, a non-square crop and a crop ≠ active ≠ plane. AC4(b) asserts the outputs differ in length (a real border), and AC6 asserts the option is not a no-op for Orientation 2–8. The off-by-one and keep-origin mutants prove that a wrong crop changes pixels.
- **Tier-B coverage.** `FullActiveArea` runs on K-3 III, L1000622 and Typ 246 (AC3, AC4(a), AC4(b)). The Q2M L1021223 refusal (8392×5632, `dst` sentinel intact) is exercised. L1026016 `apply_orientation: false` runs through the warp path at 8368×5584.
- **API and error surface.** `DevelopOptions` derives `Debug, Clone, Copy, PartialEq` (no `Eq`/`Hash`). Both types are `#[non_exhaustive]`, with every field and variant documented and marked mono-only. The external tests build options only via `Default` plus field assignment. The new variant is in `Error::kind()` (no wildcard) as `Unsupported`, and its `Display` is pinned to name both dimensions and no spec ID. CHANGELOG `[Unreleased]` names all five additions and says the default output is unchanged.
- **No regression.** Every line deleted under `tests/` is in `tests/error_kind.rs` and is a count or comment (20→21 Unsupported, 61→62 total). The `src/` diff adds no `Vec`, `vec!`, `to_vec`, `Box`, `String` or `format!`.
- **Fuzz.** `just fuzz-develop 70`: **5,411,410 runs in 71 s, zero crashes**, 24 seeds loaded (16 existing + the 8 `options-*`). `just fuzz-seeds` regenerated every seed byte-identically (clean `git status`).
- **CI.** `gh pr checks 214`: **44/44 pass** on ae232c2; the PR is MERGEABLE.

### Cost self-report

- **Tokens (total):** 6,342,969 (floor; excludes only the final commit turn).
- **Estimated USD:** $5.01.
- **Duration (minutes):** ~75.
- **Source of the number:** transcript `fad3eb49-ca46-4bac-837e-67350d3967aa.jsonl` (this session's scratchpad UUID; no subagents). Deduped by `message.id`, max `output_tokens` per id, opus-5-5 rates (in 5, out 25 per MTok; cache read 0.5; 1-hour cache write 10).

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec (judged):**
  - **AC9's second half: I agree it is unsatisfiable.** The `dst` check compares `w×h`, and orientation only swaps the factors, so a default-sized buffer is the right length under `apply_orientation: false`. The substitute is sufficient: it pins that the buffer is accepted and holds the Orientation-1 layout. The `unoriented-maps-to-2` mutant turns it red. The `FullActiveArea` half is tested in both directions, with the `dst` sentinel. The spec's AC9 text should be corrected at ship.
  - **The small `Vec<Opcode>` before the refusal is acceptable.** It is the existing `parse_opcode_list` allocation (proportional to the opcode list, not the plane). It runs once under `FullActiveArea`, because the later parse is gated off by `warp_checked_early`. The `FixBadPixelsConstant` plane copy comes after the refusal (the `refusal-after-fbp-copy` mutant is red). No plane-sized allocation happens before the refusal.
  - **AC2 runtime in debug** (~3 min) is accepted; it ran.
- **Follow-up work identified:**
  - Correct SPEC-063 AC9's orientation half at ship (doc only).
  - Optional: SPEC-063's red-proof list could name the error-precedence probe as the way to catch "refusal after the copy", which a `dst` sentinel cannot see.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The handoff's mutation list mapped one-to-one onto tests. "Validation skipped under each option" needed choosing a concrete bypass; I cleared the tags on a clone before `resolve_geometry`, per option and per entry point.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — That two `tests/develop*.rs` red-proofs copy `src/` into a temp crate. A verifier mutating `src/` while `just test` runs would corrupt them; I serialised the two.

3. **If you did this task again, what would you do differently?**
   — Script the mutation set from the start (I did, which kept 16 mutants cheap in release), and start `just test` before reading the spec, since it dominates the wall clock.
