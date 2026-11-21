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
  id: HANDOFF-173
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-20
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-083

project:
  id: PROJ-004
  stage: STAGE-026
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
  tokens_total: 11402847           # REAL combined count — what cost-audit reads
  estimated_usd: 8.14              # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 75
  branch: claude/spec-083-build
  pr: 287
  completed_at: 2026-11-20         # YYYY-MM-DD
  notes: "FLOOR taken before the handback commit, from transcript 513c6d52-92fb-4ab1-901f-78e88167ea9e.jsonl (71 unique message.ids, all claude-opus-5-5, no subagents): input 143 + cache-creation 158,659 (1-hour) + cache-read 11,206,149 + output 37,896 (max per message). USD at opus-5-5 list rates (in 5, out 25 per MTok; cache read x0.10, 1-hour cache write x2.0). Re-sum at ship." # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-20
---

# HANDOFF-173: BUILD: irradiance 0.2.4 release prep, Phase 1 (SPEC-083)

## Delegation Summary

You are the **builder** for SPEC-083. **Start only after the SPEC-083 design PR has
merged.** Cut `claude/spec-083-build` from that `origin/main`.

**The task:** Phase 1 of the 0.2.4 release, reversible prep in one CI-gated PR:
- the two small script fixes from SPEC-080's verify (`validate.sh` feedback status
  CR/quotes/BOM; `sdk-stage2.sh` silent exit on an unenterable worktree);
- `just semver-check` against crates.io 0.2.3;
- `Cargo.toml` to `0.2.4`, the CHANGELOG promoted, the release notes written;
- tier-B evidence (corpus, SDK cross-check, timings), package guards, the pre-flight and
  every gate.

You never tag or publish. Phase 2 is the user's.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-083-irradiance-0-2-4-release-dng-conformant-black-normalisation.md`
  (Release Scope, Build work 1–9, Release Notes, Pre-Flight, Notes).
- **The model:** SPEC-077, the 0.2.3 release
  (`specs/done/SPEC-077-irradiance-0-2-3-release-gainmap-and-black-level-fixes.md`). Follow
  its Pre-Flight and internal-record shape item for item.
- **What's being released:** the CHANGELOG `[Unreleased]` section (SPEC-078, SPEC-080),
  DEC-070 with both §3 amendments, and SPEC-080's verify follow-ups (HANDOFF-172's handback).
- **The SDK:** `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`.
  Write its outputs into your scratchpad only, never into any checkout.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.

## Expected Deliverables

1. **A PR against `main` from `claude/spec-083-build`,** CI green, with the script fixes,
   the version bump, the CHANGELOG promotion and the filled spec (Release Notes, internal
   record, Pre-Flight `[now]` halves).
2. **The handback:** the semver summary verbatim; the release notes checked line by line
   against the spec's "every input whose result changes" list; the SDK numbers; the
   timings; the package file list; each script-fix scenario with its message.
3. **Gates,** all green, as Build work 9 lists.

## Out of Scope

- Any library code change beyond documentation wording.
- Tagging, publishing, or a GitHub release.
- STAGE-026's close and PROJ-004's close (the orchestrator does them at ship).

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- **A semver-check failure stops the build** for the user's decision.
- Commit-message bodies are plain prose with no code tokens.
- `Cargo.lock` is gitignored; never force-add it. Stage explicit paths.
- Never commit corpus bytes, SDK bytes or SDK outputs.
- **Fill the `handback:` front matter** with a real `tokens_total`, re-summed from your own
  transcript (`basis: transcript`, deduped by `message.id`, opus-5-5 rates). Then fill the
  `## Handback` section.
- Keep working until the PR and the handback are pushed; don't end a turn while a long
  run is still going.

---

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

*Filled in by the receiving agent (builder, claude-opus-5-5).*

### Execution notes

- **Branch / PR:** `claude/spec-083-build`, https://github.com/jysf/irradiance/pull/287
  - `46e5858`: the two script fixes;
  - `04593ce`: the prep (version, CHANGELOG, README). This is the candidate the evidence ran on;
  - `beaae7b`: the release notes, internal record, pre-flight and timeline line;
  - then this handback.
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** yes: build work 1 to 9. CI had started (22 checks pending)
  when this handback was written.
- **No tag, no publish:** `git tag -l v0.2.4` is empty.

**Semver, verbatim** (`cargo-semver-checks 0.51.0`, `just semver-check`, crates.io 0.2.3 baseline,
after the bump):

```
Checking irradiance v0.2.3 -> v0.2.4 (minor change)
 Checked [   0.010s] 202 checks: 202 pass, 58 skip
 Summary no semver update required
```

The pre-bump run at `46e5858` and the `--all-features` run gave the same result. There was
nothing to accept.

**Release notes against the "every input whose result changes" list:**
- Newly refused colour input (W ≤ max B, `InvalidLevels` before any write; 0.2.3 gave an all-zero
  image): yes, under Behaviour changes.
- Mono `PerPosition`, the same refusal, `black_level` now the largest black: yes, under Behaviour
  changes, plus one line under Breaking for a caller that matches the exact field.
- Output changes (Canon, 3a, 4a on colour and `PerPosition`; uniform-black files and
  `FirstValue` byte-identical): yes, under Behaviour changes and Changed.
- `DEVELOP_VERSION` 6 → 7: yes.
- Wording caution: the notes say colour refuses what mono `PerPosition` refuses, and that
  `FirstValue` checks its first black only. The CHANGELOG's SPEC-080 entry said "as the mono path
  already did"; it now names `BlackLevels::PerPosition`.
- The headline's "within one code before the GainMap" is limited to the three held files whose
  blacks differ by position. The Pixel 7 Pro (2 codes, fractional black) is listed as a known
  limitation.

**SDK numbers.** A fresh `sdk-stage2.sh` run into the scratchpad wrote 7 TIFFs; their sha256s equal
SPEC-079's. Δ is SDK minus ours, and every figure equals HANDOFF-168's table digit for digit:
- no gain: Canon 1 / 99.5663%, 4a 1 / 99.7911%, 3a 1 / 99.8536%, 7 Pro 2 / 28.3471%;
- end to end: 4a 4 / 99.7729%, 3a 4 / 99.8210%, 7 Pro 4 / 28.3394%;
- gain step: 1 on all three, at 99.9794%, 99.9672% and 99.9664%.

**Timings** (Apple M4 Pro, release, medians in ms):
- unpack: 9.9 on 1 thread, 1.4 on 8;
- ljpeg: K-3 III 100.5, Typ 246 87.3, Pixel 4a 58.8;
- orient, warp-free: O1 85, O5 98, O8 143;
- orient, post-warp: O1 570, O8 609.

**Package.** The dry run was clean and ran in a detached scratch worktree, without
`--allow-dirty`:
- 19 files: `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, both licences,
  `README.md` and the 12 `src/` files;
- the list is identical to the published 0.2.3 `.crate`'s `tar -tz` list;
- sha1 `04593ce3044b0bd62878bfb0fbee3c3914d21961`;
- no self dev-dependency.

**Script-fix scenarios.** Each ran on a scratchpad `git archive` copy (validate) or a scratch
repo (sdk-stage2), old script against new.
- CRLF with a bare `status: open`. Old: rejected, `status(='open^M', expected one of …)`.
  New: green.
- `status: "open"`. Old: rejected (`'"open"'`). New: green.
- `status: 'open'`. Old: rejected (`''open''`). New: green.
- BOM then `---`. Old: green but silently not parsed (9 feedback, not 10). New: rejected,
  "feedback file starts with a UTF-8 BOM before its '---'; remove the BOM so its front matter is
  parsed".
- `status: "closed"`, and a CRLF `closed`. Rejected by both, naming `closed`.
- A worktree that exists but can't be entered (mode 000). Old: exits 1 with only the shell's
  "cd: … Permission denied" and nothing from sdk-stage2. New: "sdk-stage2: cannot enter worktree
  …; refusing its recorded path as it is", then the run continues.
- An output path inside that worktree. New: "cannot enter the output directory's parent: …",
  exit 1.
- Note: on the real feedback files a CRLF status never showed the bug, because the inline
  `# open | addressed | deferred` comment pushes the CR out of `$2`. Only a bare value shows it.

**Gates.** All green:
- `just test` (790 / 0 / 1);
- `lint-ci` (clippy 0.1.98 pinned), `lint-no-allow`, `lint-fuzz`, `fmt-check`;
- `validate` (376 parsed, 9 feedback);
- `doc-check` (2 surfaces, 0 warnings), `deny`, `deny-fuzz`, `msrv` (1.90.0);
- `cost-audit`, `gate-audit` (13 scripts), `name-length-check` (895 paths);
- `decisions-index --check` (70), `semver-check`, `package-targets-check`.

### Cost self-report

- **Tokens (total):** 11,402,847. Input 143, 1-hour cache write 158,659, cache read
  11,206,149, output 37,896. This floor was taken before the handback commit; re-sum at ship.
- **Estimated USD:** 8.14 at opus-5-5 rates.
- **Duration (minutes):** ~75
- **Source of the number:** transcript `513c6d52-92fb-4ab1-901f-78e88167ea9e.jsonl`, found by the
  scratchpad UUID. It is deduped by `message.id` (71 ids), taking the largest `output_tokens`.
  There were no subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:**
  - For the BOM case, I chose "refuse it loudly" over "treat it as front matter". Parsing it would
    have meant changing the shared `_lib.sh` parser.
  - The sdk-stage2 fix also covers an unenterable output directory or parent, the same
    silent-exit shape.
  - The README roadmap line said "Still ahead: a clean-room DNG writer". It now says the writer
    moved to PROJ-005 and that optical black is parked, per the PROJ-004 brief.
- **Follow-up work identified:**
  - None new. The broken template heading that HANDOFF-171 and 172 reported is not in this
    handoff.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Very little. The verify's CRLF repro didn't show up on the real files, because of the
   inline comment, and it took one extra loop to see why.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. Running the dry run in a detached scratch worktree kept the dry run clean while the
   spec was still being edited.

3. **If you did this task again, what would you do differently?**
   — I'd commit the release notes with the prep, so the timing stamp isn't `-dirty`.
