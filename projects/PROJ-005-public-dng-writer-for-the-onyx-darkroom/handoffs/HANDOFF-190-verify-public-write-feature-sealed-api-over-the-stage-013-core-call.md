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
  id: HANDOFF-190
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-24
  status: pending                  # pending | accepted | completed | rejected

task:
  spec_id: SPEC-061

project:
  id: PROJ-005
  stage: STAGE-017
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
  status: null                     # completed | blocked | rejected
  agent: null                      # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: null                    # verify handoffs: approved | punch-list | rejected
  basis: null                      # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-190: VERIFY: public `write` feature, sealed API over the STAGE-013 core (SPEC-061)

## Delegation Summary

You are the **verifier** for SPEC-061. Review it cold: you did not build it.

- **Under review:** PR 329 (`claude/spec-061-build`, head `fa1e539`), built under
  HANDOFF-175 by a separate session.
- **Your branch:** `claude/spec-061-verify-base`. It is that head plus one commit that adds
  this handoff and the verify prompt. The build already ran `just advance-cycle SPEC-061 verify`.

SPEC-061 adds `irradiance::dng::write` behind the opt-in `write` feature: a mono u16 plane
plus caller metadata in, one little-endian uncompressed single-strip DNG out, through
`written_len`, `write_into` and `write_to_vec`. Framing (FullSensor, FullActiveArea,
StandardCrop) is expressed as tags and the samples are only ever cut at `ActiveArea`
(DEC-073). Two `Error` variants (`InvalidWriteInput`, `UnsupportedOpcodeForFraming`) exist
in every build. No reader change, no new dependency, no I/O in `src/dng`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/specs/SPEC-061-public-write-feature-sealed-api-over-the-stage-013-core-call.md`.
  Judge the ACs against the spec **as designed** (`git show origin/main:<spec path>`):
  AC1 to AC12, Design §1 to §9, the oracle list in §7 and the probe table in
  `## Implementation Context`.
- **Stage and brief:** `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/stages/STAGE-017-public-dng-writer-behind-a-write-feature.md`,
  `./projects/PROJ-005-public-dng-writer-for-the-onyx-darkroom/brief.md`.
- **Decisions:** DEC-071 (v1 scope), DEC-072 (EXIF offset slot only; the block is
  SPEC-081), DEC-073 (framing as tags, cut only at ActiveArea), DEC-075 (layout,
  validation, the `OpcodeList1` allow-list, two `Error` variants, no allocation),
  DEC-080 (new, proposed, from the build: payload types compiled in every build, named
  under `dng::write`), DEC-055, DEC-050, DEC-016, DEC-051, DEC-052, DEC-058, DEC-069 §6,
  and DEC-077 (the reader's DNG-version guard the written files must pass).
- **Constraints:** `no-panics-on-untrusted-input`, `oracle-must-be-shown-red`,
  `provenance-recorded-per-algorithm`, `no-copyleft-dependencies`,
  `library-not-application`.
- **Build handback:** HANDOFF-175's `## Handback` (in the same handoffs dir): the
  staleness check, the gates, the dng_validate table, five deviations and two follow-ups.
- **Toolchain:** AGENTS.md and `guidance/toolchain-brief.md`. Run `just test`, never
  `just test --all-features`. Set
  `IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and
  `IRRADIANCE_DNG_VALIDATE=~/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Use `/usr/bin/git` to establish branch and HEAD. Give each mutant run its own source
  state and read the panic message.

## What to check hard

1. **Every AC met against the spec as designed**, and every §4 tag row and §5 refusal
   reaching a test that fails when it is wrong.
2. **Oracle 8 deviation (FullSensor render).** The build compared the FullSensor render to
   the FullActiveArea output instead of to the source, because §3 writes FullSensor's
   DefaultCrop as the whole active area. Decide: is the build's reading of §3 right, so
   that the spec's oracle 8 text is the error? Or should FullSensor carry the caller's
   crop, which is a §3/DEC-073 change? Rule on it in the verdict. Do not change `src/`.
3. **DEC-080.** Does it resolve a real contradiction in the spec ("variants in every
   build" against payload types under a feature-gated module)? Is the public path
   (`irradiance::dng::write::WriteInputError`, `Framing`) the same with and without
   `write`? Check `cargo semver-checks` and `cargo doc` in both builds. Does DEC-080 need
   accepting before ship?
4. **FixBadPixelsConstant at the active-area edge** (build follow-up 1). Confirm or refute:
   after a cut, a bad pixel on the active-area edge loses its margin neighbours, so the
   cut file can develop differently there. If it is real, is DEC-075 §5's claim that
   opcode id 4 is position-independent wrong as written? Build a tier-A fixture if you
   can. Put the ruling (document it in DEC-075, refuse, or flag) in the punch list or the
   follow-ups.
5. **Red-proofs (AC7).** Oracles 3 and 5 after their negative controls. Run the new test
   files against `origin/main`'s `src/` as well. Mutate the writer at least: drop the
   `ActiveArea` rebase on a cut, write `DefaultCropOrigin` unrebased, skip the
   `OpcodeList1` allow-list, swap the strip byte order, off-by-one in `written_len`.
   Each must fail a named test. Give a table.
6. **No panics and no allocation (AC10).** Offset and size arithmetic on caller input,
   `max_pixels` checked before any multiplication, `write_into` allocation-free
   (`tests/dng_write_alloc.rs`). Read the fuzz target. Is it reaching the refusal paths,
   or only `Ok`?
7. **The other deviations:** the tuple-variant change to `tests/error_kind.rs` (does it
   still catch a missing `kind()` arm?), the self dev-dependency enabling `write`, the two
   unlisted support files, and the 4 GiB lazily mapped `OpcodeList2` for `OutputTooLarge`
   (does it stay cheap on the CI runners, Linux and macOS arm64?).
8. **`white_level` on 12/14-bit sources** (build follow-up 2). Is it documented in the
   rustdoc? If not, a docs fix on your branch is in scope.
9. **Gates:** `just test` with corpus and dng_validate set, `lint-ci`, `lint-no-allow`,
   `fmt-check`, `doc-check`, `deny`, `deny-fuzz`, `lint-fuzz`, `msrv`, `validate`,
   `gate-audit`, `cost-audit`, `just decisions-audit --changed`, `cargo semver-checks`,
   60 s of `just fuzz-dng-write`, and `gh pr checks 329`.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **Evidence for each item above,** including the mutant table and a ruling on items 2,
  3 and 4.
- **Commit** this handoff's filled handback (and any tests or docs fixes) to
  `claude/spec-061-verify-base` and push it. If approved, run
  `just advance-cycle SPEC-061 ship --verdict approved` and fast-forward
  `claude/spec-061-build` to the verify tip so PR 329 carries the verify commits. Do not
  merge.
- **Fixes:** tests or docs only on your branch. Any `src/` change goes in the punch list.
- Commit-message bodies are plain prose. The repo ruleset rejects code tokens.
- **IDs:** a separate session is working PROJ-006 and also mints IDs. Before minting a DEC
  or HANDOFF, compute the next free one from `origin/main` plus open PRs.

## Out of Scope

- SPEC-081 (EXIF block, XMP, descriptive tags, DEC-074 rendering hints) and SPEC-082
  (preview).
- Compression, float, CFA, tiles, big-endian, several strips.
- Any reader change (`src/ifd.rs`, `src/plane.rs`, `src/develop.rs`). If a round trip
  exposes a reader bug, report it.
- Migrating `tests/support/dng_writer.rs` onto the public writer.
- Activating PROJ-005 (brief status is the maintainer's call).

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

- **Branch / PR:** [link]
- **Completed at:** YYYY-MM-DD
- **All acceptance criteria met?** yes/no (if no, explain)
- **For `verify`:** the verdict — ✅ APPROVED (at commit SHA) / ⚠ PUNCH LIST / ❌ REJECTED

### Cost self-report

Mirror what you put in the `handback:` front-matter, and say where the number
came from. **This is the number that lands in the spec** — the orchestrator
transcribes it via `just handback-sync`, it does not estimate it.

- **Tokens (total):** <real number, or null + why>
- **Estimated USD:** <number, or null>
- **Duration (minutes):** <estimate>
- **Source of the number:** `/cost` | API `usage` | harness report | none available

### Drift and new artifacts

- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs that should be added to the stage's backlog]

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>
