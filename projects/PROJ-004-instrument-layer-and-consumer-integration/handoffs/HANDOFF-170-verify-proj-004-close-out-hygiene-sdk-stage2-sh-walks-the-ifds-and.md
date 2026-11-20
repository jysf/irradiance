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
  id: HANDOFF-170
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-20
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-079

project:
  id: PROJ-004
  stage: STAGE-025
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
  status: completed   # completed | blocked | rejected
  agent: claude-opus-5-5   # the model that ACTUALLY ran (message.model) —
                                   #   to_agent above is a prediction from tier_map
  verdict: approved   # verify handoffs: approved | punch-list | rejected
  basis: transcript   # transcript | agent-result | slash-cost | api-usage | estimate
  tokens_total: 7334564   # REAL combined count — what cost-audit reads
  estimated_usd: 5.39   # per component at `agent`'s rates (docs/cost-tracking.md "Pricing")
  duration_minutes: 80
  branch: claude/spec-079-verify-base
  pr: 279
  completed_at: 2026-11-20   # YYYY-MM-DD
  notes: "APPROVED at 8abd264 (PR 279 head; verify base 41748ce). Floor taken before the handback commit, transcript c0a1d557-a729-49e6-b66b-2b6b37b98278.jsonl, 53 message ids, no subagents; opus-5-5 at 1-hour cache writes. Re-sum at ship."   # if unusual; double-quote it — an unquoted " #" starts a YAML comment
  synced_at: 2026-11-20
---

# HANDOFF-170: VERIFY: PROJ-004 close-out hygiene (SPEC-079)

## Delegation Summary

You are the **verifier** for SPEC-079. Review it cold: you did not build it.

- **Under review:** PR #279 (`claude/spec-079-build`, head 8abd264), built under
  HANDOFF-169. CI was green or finishing (20 of 22 checks passed, 2 pending, when this
  was written). Check it.
- **Your branch:** `claude/spec-079-verify-base`. It is that head plus one commit that adds
  this handoff and moves the spec to the verify cycle.

SPEC-079 changes no `src/`. It changes `scripts/sdk-stage2.sh` (an IFD walk, the output
directory created after the in-repo check), `tests/gain_map_corpus.rs` (set means must
run), `tests/black_normalisation.rs` (FU-2 fixtures), `docs/provenance-ledger.md` and
`feedback/2026-11-11-monorail-000-requests.md`.

## Context the Receiving Agent Needs

- **Spec:** `./projects/PROJ-004-instrument-layer-and-consumer-integration/specs/SPEC-079-proj-004-close-out-hygiene-sdk-stage2-sh-walks-the-ifds-and.md`
  (Context 1–6, Design 1–6, AC1–AC7).
- **Decisions:** DEC-069 §6, DEC-070 (§1, and §3 as amended at SPEC-078's ship).
- **Build handback:** HANDOFF-169's `## Handback`. Check each claim yourself.
- **The SDK:** `dng_validate` at
  `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/tools/dng_sdk/build/dng_validate`;
  the zip at `../dl/dng_sdk_1_7_1.zip`. Generate your own outputs into your scratchpad.
  The builder notes the script finds the repo root from its own location, so run the old
  script (from `origin/main`) in place, from a checkout of main, not from a copy.
- **Toolchain:**
  - AGENTS.md §5–6. Run `just test`, never `just test --all-features`.
  - Set `IRRADIANCE_CORPUS_DIR=/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/images`
    and use the files in place.
  - Give every mutant copy its own `CARGO_TARGET_DIR`, and read each panic message.

## What to check hard

1. **The IFD walk is correct and safe.** Read the Perl.
   - Header, `NextIFD` chain and `SubIFDs` recursion; every offset and count is
     bounds-checked before it is read, in both byte orders; `SubIFDs` with a count > 1
     and with a count of 1 (value inline) are both handled.
   - Exactly one 51009 entry, type 7; the rewrite touches only that entry's 2 tag bytes.
   - A revisited IFD is refused. Is refusing a legitimate DAG (two pointers to one IFD)
     acceptable for this tool? Say so either way.
   - Reproduce AC1 cold: offsets, byte-identical outputs from old and new scripts, and the
     cross-check numbers. Reproduce at least four of AC2's six scenarios, including the
     decoy and the cycle, with your own crafted copies.
   - Craft one case the builder did not: for example an entry count of 0, an IFD offset
     that is odd or points inside the header, or a `SubIFDs` entry whose value offset is
     out of range. It must be refused cleanly.
2. **The directory order (AC3).** Reproduce the in-repo refusal leaving nothing behind,
   including through a symlink that resolves into the repo, and a relative path.
3. **Set means must run (AC4).**
   - The helper is shared by all three SDK tests, and no SDK test still has a
     `continue`/`return` on a missing corpus file.
   - The non-SDK tests in the file still skip on a missing corpus.
   - Reproduce the by-hand run: SDK variable set plus an empty corpus directory fails all
     three; unset passes with SKIP. What happens with the variable set to an empty
     string? It should behave as unset, as designed; confirm.
4. **FU-2 (AC5).** Re-run the three designed mutants on tier A (no corpus) with an
   unmutated control first. Check the builder's deviation (the general-path maximum at
   every one of 16 indices, rather than index 0 only) is sound and drops nothing the
   design asked for. Do the new tests compute their expected values independently of the
   crate (no call into a crate helper for `B`)?
5. **No existing assertion moved.** `git diff origin/main -- tests/ src/`: only additions,
   plus the loader refactor in `gain_map_corpus.rs`. Confirm the refactor changes no
   non-SDK behaviour.
6. **Records (AC6).**
   - Re-check the licence finding yourself: list the zip, grep its text files and PDFs
     (`pdftotext`) for "licen" and "agreement". Confirm the ledger and script header
     state facts only, with the zip's sha256, and no legal opinion.
   - The feedback file: front matter `status: addressed`, items 5 and 8 correct against
     the CHANGELOG (`[0.2.2]`, `[0.2.3]`), nothing else in the consumer's text changed
     (`git diff`). Parse its front matter.
   - The builder notes `just validate` does not scan `feedback/` or `docs/`; confirm, and
     say whether it matters here.
7. **Nothing forbidden is committed:** no SDK bytes, outputs or scratch copies; no corpus
   bytes; no `Cargo.lock`.
8. **Gates:** `just test` with the corpus, with and without the SDK variable; `lint-ci`,
   `lint-no-allow`, `fmt-check`, `validate`, `doc-check`, `cost-audit`, `gate-audit`,
   `name-length-check`, `decisions-index --check`; CI.

## Expected Deliverables

- **Exactly one verdict:** ✅ APPROVED (at a SHA), ⚠ PUNCH LIST, or ❌ REJECTED.
- **For each item above:** evidence, including your own scenario table and mutant table.
- **Commit this handoff's filled handback** to `claude/spec-079-verify-base` and push it.
  Do not open a PR, and do not push to `claude/spec-079-build`.
- **A punch-list fix** is fine on your branch if it is tests, scripts or docs only. A
  measured number outside a pre-registered bound is reported, never fixed by widening it.

## Out of Scope

- FU-1 (a colour levels refusal).
- Any `src/` change.
- The brief's stage count and the STAGE-025 reflection.
- A release.

## Hard rules

- **Never** force-push, merge, enable auto-merge, tag or publish.
- Commit-message bodies are plain prose with no code tokens.
- Never `git add Cargo.lock`. Stage explicit paths. Never commit corpus or SDK bytes.
- **Fill the `handback:` front matter:**
  - `verdict`;
  - a real `tokens_total`, re-summed from your own transcript, found by your scratchpad
    UUID and deduped by `message.id` (`basis: transcript`);
  - opus-5-5 rates.

  Then fill the `## Handback` section.

---

## Handback` section below.** An unfilled handback is an
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

- **Branch / PR:** under review https://github.com/jysf/irradiance/pull/279 (head `8abd264`);
  this handback is on `claude/spec-079-verify-base` (base `41748ce`). No PR opened.
- **Completed at:** 2026-11-20
- **All acceptance criteria met?** Yes. AC1 to AC7 were each reproduced cold. There's no
  blocking item; the notes below are non-blocking.
- **For `verify`:** ✅ **APPROVED at `8abd264`**. No `src/` change, and the verify base adds
  only records.

#### 1. The IFD walk (AC1, AC2)

I read the Perl:
- **Header:** the length (at least 8) and magic 42 are checked before anything else is read.
- **Bounds:** every u16 and u32 read goes through a helper that checks it against the file
  length. The IFD offset must be at least 8 and its 2-byte count must fit, and the entry
  table plus `NextIFD` must fit before any entry is read.
- **`SubIFDs`:** only type 4 or 13 is accepted. A count of 1 or less is read inline from the
  value field; a larger count is read from the offset, and the whole array is bounds-checked
  first. Both byte orders go through the same unpack letters.
- **Cycles:** an IFD reached twice is refused.
- **Rewrite:** only exactly one 51009 entry of type 7 is rewritten. `pack` writes the 2 tag
  bytes and nothing else. Warnings are fatal.

**DAG ruling (two pointers to one IFD):** refusing it is acceptable for this tool. It's a
local reference generator for three known Pixels. It fails closed with a clear message, and
no held file has a shared IFD.

**AC1, cold:**
- My scratchpad run of the new script and an in-place run of `origin/main`'s script, from a
  scratch worktree of main, rewrite at the same offsets: 4a 121934, 3a 1488 and 7 Pro 11224.
- All 7 TIFFs and the 3 nogain DNGs are byte-identical between the two (sha256 prefixes:
  Canon `893d7b94`, 3a post `b2f784d7` and pre `b0dc823b`, 4a post `d33afbb4` and pre
  `771609b8`, 7 Pro post `ec2e1a55` and pre `3fc24044`). The logs differ only in timings and
  paths.
- Rerun against them, `gain_map_corpus` gives SPEC-078's numbers digit for digit:
  - Canon no gain: 1 / 99.5663%.
  - 4a: no gain 1 / 99.7911%, end to end 4 / 99.7729%.
  - 3a: no gain 1 / 99.8536%, end to end 4 / 99.8210%.
  - 7 Pro: no gain 2, end to end 4.
  - Gain step: 1 / 99.9794%, 99.9672% and 99.9664%.

**AC2, plus my own cases.** These are crafted copies of the Pixel 3a and synthetic TIFFs,
with the old and new Perl bodies extracted verbatim and run side by side:

| case | old | new |
|---|---|---|
| decoy pattern planted in pixel data | refused (found 2) | **ok**, only byte 1488 changed |
| 51009 tag already 51010 | refused | refused, 0 entries |
| second 51009/7 entry in IFD0 | refused | refused, 2 entries |
| truncated to 1300 bytes | refused (0 found) | refused, entries run past the end |
| IFD0 offset past the end | **accepted** | refused, past the end |
| `NextIFD` cycle back to IFD0 | **accepted** | refused, reached twice |
| *own:* IFD0 offset 4 (inside the header) | accepted | refused, inside the header |
| *own:* `SubIFDs` count 3, offset at len − 4 | accepted | refused, offsets run past the end |
| *own:* `SubIFDs` inline offset 0 | accepted | refused, inside the header |
| *own:* IFD0 entry count 0xFFFF | accepted | refused (a garbage `SubIFDs` type) |
| *own:* IFD0 entry count 0 | accepted | refused (it walks into garbage, then a bad type) |
| *own:* MM synthetic, `SubIFDs` count 2 (type 13) by offset, decoy in data | refused | **ok**, only byte 163 changed |
| *own:* MM synthetic, same with type 4 | refused | **ok**, only byte 163 changed |
| *own:* II synthetic, same | refused | **ok**, only byte 162 changed |
| *own:* MM DAG (both `SubIFDs` → one IFD) | refused | refused, reached twice |
| *own:* MM `SubIFDs` type 3 | refused | refused, type 3 |

No case hung, and none printed a Perl warning; every one was run under a 30-second alarm.

#### 2. Directory order (AC3)

Each refusal leaves nothing behind:

| `<out_dir>` | result | created? |
|---|---|---|
| in-repo, absent leaf | refused, inside the repository | no |
| relative `out-rel-x079` (from the repo root) | refused, inside the repository | no |
| relative `../<worktree>/out-dd-x079` | refused, inside the repository | no |
| scratch symlink → repo, then `/out-sym-x079` | refused, inside the repository | no |
| the symlink itself (resolves to the repo root) | refused | (it is the repo) |
| in-repo with a missing parent | refused, parent does not exist | no |
| outside, missing parent | refused, parent does not exist | no |
| an existing file | refused, not a directory | (the file is untouched) |
| outside, new | created, 4 files done | yes |

`git status --porcelain` was clean afterwards.

#### 3. Set means must run (AC4)

- One helper, `sdk_corpus_plane` → `sdk_corpus_file(true, …)`, is shared by all three SDK
  tests. None of them still has a `continue` or `return` on a missing corpus file. The only
  `return`s left are `sdk_dir()`'s skip when the variable is unset.
- The non-SDK tests keep `corpus_plane` → `require` (lines 402 and 597 still `continue`).
- The `load_plane` split is a pure extraction: the same reads and the same panics.
- By hand, with `IRRADIANCE_CORPUS_DIR` set to an empty directory:

  | SDK variable | result |
  |---|---|
  | set | 8 pass, 3 fail at `gain_map_corpus.rs:714`, naming the file, its path and both variables |
  | unset | 11 pass |
  | empty string | 11 pass, the same as unset (`sdk_dir` filters `is_empty`) |

  So the non-SDK tests still skip on a missing corpus with the SDK set.

#### 4. FU-2 (AC5): mutants on tier A

Each mutant ran with no corpus and no SDK, under the corpus profile with `--no-fail-fast`, in
its own `git archive` copy with a separate target directory. The unmutated control ran first.

| mutant | tier-A result | red tests |
|---|---|---|
| control | 764 / 0 / 1 | — |
| M1, colour slots `B` = last slot | 1 fail | `maximum_black_at_each_slot_scales_every_slot_by_it` only (the FU-2 gap, now closed) |
| M2, colour slots `B` = first slot | 12 fail | including the new slot test |
| M3, colour general `B` = first value | 5 fail | including the new 16-index test |
| M4 (*own*), colour general `B` = last value | 1 fail | `general_pattern_maximum_black_at_each_index…` only |
| M5, mono `B` = last value | 5 fail | including both new tests |
| M6 (*own*), mono `B` = first value | 10 fail | including both new tests |

**The deviation is sound.** It puts the maximum at all 16 indices rather than at index 0 only:
- it covers the design's index-0 case;
- with the maximum at index 0 only, M3 would be equivalent;
- M4 shows it also catches a last-value bug that nothing else catches.

**Independence.** The expected values come from developing a uniform-2048 file with one more
raw code at the 2047 photosites. That's the shifted-uniform equivalence: under uniform blacks
`W − B` is unaffected by any of these mutants. `B` is never computed by a crate helper.

#### 5. No existing assertion moved

`git diff origin/main -- src/ tests/` removes only these lines:
- the 4 old skip arms;
- `PathBuf` → `{Path, PathBuf}`;
- the `load_plane` extraction;
- a doc line;
- `cfa`'s hard-coded `[2, 2]`, now `cfa_on(…, [2, 2], …)`, which is identical.

No `src/` change.

#### 6. Records (AC6)

- **The zip:**
  - sha256 `5366d9ab…e6e0c2`, which matches; 4,259 entries.
  - The four top-level ReadMes have 0 hits for "licen" or "agreement". So do the three Adobe
    PDFs under `pdftotext`, and brotli's PDF.
  - The only licence and COPYING files are third-party ones: libjxl and its `third_party`,
    the XMP toolkit's bundled uuid, and libjpeg's README.
  - The per-file notice ("…the Adobe license agreement accompanying it") is in every
    `dng_sdk/source` file except 4 JPEG and deprecation shims.
- **Ledger and script header:** both state facts only, with the sha256, and give no legal
  opinion.
- **The feedback file:**
  - its front matter parses with `YAML.parse` (`status: addressed`);
  - item 5 says "released in 0.2.2" and item 8 says "shipped … released in 0.2.3", which
    agrees with the CHANGELOG (`[0.2.2]` has `Container::previews()` and `[0.2.3]` has
    `masked_areas`);
  - otherwise the diff is only the status line and an appended closing paragraph. The
    consumer's own text is unchanged.
- **`just validate` scope:** confirmed, it parses only `projects/` and `decisions/`. It
  doesn't matter here: the ledger has no front matter, and the feedback front matter was
  parsed by hand. It's recorded as process debt below.

#### 7. Nothing forbidden is committed

`origin/main..HEAD` is 8 text files. There are no SDK bytes or outputs, no corpus bytes and no
`Cargo.lock` (it's ignored, and `git ls-files` doesn't show it). All SDK outputs, crafted
copies and the unzipped SDK are in my scratchpad only.

#### 8. Gates

- **`just test` with the corpus:**
  - with the SDK variable: 778 / 0 / 1, and the 3 SDK tests ran;
  - without it: 778 / 0 / 1, with the SKIP line printed.
- **Other gates, all green:** `lint-ci`, `lint-no-allow`, `fmt-check`, `validate` (83 / 360),
  `doc-check`, `cost-audit`, `gate-audit` (13 scripts), `name-length-check` and
  `decisions-index --check` (70).
- **CI on PR 279:** 22 / 22 pass, MERGEABLE.

### Cost self-report

- **Tokens (total):** 7,334,564, as a floor taken before this commit:
  - input 121;
  - output 24,157;
  - cache read 7,191,459;
  - cache write 118,827, all 1-hour.
- **Estimated USD:** 5.39 at opus-5-5 rates ($5 in, $25 out; reads × 0.10, 1-hour writes × 2.0).
- **Duration (minutes):** about 80.
- **Source of the number:** the session transcript, `c0a1d557-a729-49e6-b66b-2b6b37b98278.jsonl`,
  found by the scratchpad UUID. 53 `message.id`s, deduped, keeping the largest
  `output_tokens`. No subagents.

### Drift and new artifacts

- **New decisions emitted:** none.
- **Deviations from spec:** none by the verifier. The builder's 16-index deviation is
  endorsed in item 4.
- **Follow-up work identified (non-blocking, none is a punch item):**
  - **Script repo root.** The script resolves "the repo" from its own location, so a worktree
    copy treats the main checkout as outside the repo. That was already true before. If it
    matters, use the git common directory.
  - **Ledger wording.** It says "its three PDFs", but the zip holds four; the fourth is
    brotli's, a third-party PDF with no hits. The SPEC-079 licence paragraph is also repeated
    word for word in two ledger rows. Wording only.
  - **`just validate` scope.** It doesn't parse `feedback/` front matter. Consider adding
    `feedback/` to its parse set (process debt the builder also noted).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — The zip path. The handoff's "../dl/" is relative to `tools/dng_sdk/build`, and the real
   path is `tools/dng_sdk/dl/`. Minor.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The verify-base branch is checked out in the orchestrator's worktree, so it can't be
   checked out here by name. Work detached and push by refspec. The handoff could say so.
3. **If you did this task again, what would you do differently?**
   — I'd start the long tier-A mutant matrix and both `just test` runs first, then do the
   Perl and scenario work while they run. That is what I did, and it was the right order. I'd
   also wait on them with a monitor from the start rather than ending the turn.
