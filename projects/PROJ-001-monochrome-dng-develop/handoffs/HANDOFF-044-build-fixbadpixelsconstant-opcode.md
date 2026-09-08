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
  id: HANDOFF-044
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # ⚠ ROUND 2. Was `claude-sonnet-5` — round 1's
                                    # ACTUAL model, already transcribed into the
                                    # spec's first `cycle: build` cost session, so
                                    # this overwrite loses nothing. CORRECTED here
                                    # so `scripts/handback-sync.sh` (line 97 reads
                                    # to_agent from the handoff) stamps round 2's
                                    # session with the model that ran it, instead
                                    # of inheriting round 1's — the exact defect
                                    # SPEC-018/FU-11 recorded twice when
                                    # HANDOFF-046 was reused for its rounds 2 and
                                    # 3. VERIFIED, not predicted: this session's
                                    # own transcript (identified by the scratchpad
                                    # UUID 83d8a76b-4b2b-4bfa-9216-e6d0600996e4,
                                    # NOT by text match) reports `message.model` =
                                    # claude-opus-5 on every assistant record.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-017

project:
  id: PROJ-001
  stage: STAGE-003
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
#
# `tokens_total` is the one field the cost gate reads. Report the REAL number
# from your own interface:
#   Claude Code   → run `/cost`
#   API           → the `usage` object (input + output, summed)
#   another agent → whatever your harness reports as total tokens
# If your platform genuinely exposes NO token count, set tokens_total: null AND
# write why in `notes` — then set `cost.metering_source: none` in
# .repo-context.yaml so the gate stops asking. Do not invent a number.
#
# ⚠ `notes:` must be ONE PHYSICAL LINE. handback-sync transcribes only the
# first physical line of a multi-line YAML scalar, leaving unterminated quotes
# that make front matter unparseable while every gate still reports green
# (`handback-sync-truncates-multi-line-scalars`).
#
# ⚠ ROUND 2 (punch-list). This block carried round 1's numbers until
# 2026-09-08; they are preserved verbatim in `## Completion — round 2`
# below (570,000 tokens, $4.10, 90 min, on claude-sonnet-5) AND were already
# transcribed into the spec's `cost.sessions` (the FIRST `cycle: build`
# entry), so the overwrite loses nothing — exactly as SPEC-018's HANDOFF-046
# rounds 2 and 3 did. `synced_at` is reset to null so
# `just handback-sync SPEC-017` appends round 2 as a SECOND build session
# rather than skipping this file for idempotence.
#
# ⚠ `notes:` is ONE PHYSICAL LINE **and starts with a double quote**, and
# contains NO bare `#`. Both are load-bearing: `scripts/_lib.sh`'s
# get_handback_field strips a trailing YAML comment from any value that does
# NOT begin with a quote, so an unquoted note truncates at its first bare
# hash (`handback-sync-treats-hash-as-comment-in-unquoted-notes`).
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 9026059            # ROUND 2 ONLY. MEASURED, deduped by message.id from my OWN
                                    # transcript, identified by the scratchpad UUID
                                    # 83d8a76b-4b2b-4bfa-9216-e6d0600996e4 (NOT by text match —
                                    # `identify-own-transcript-for-cost-handback`: a stale
                                    # pre-/clear file can text-match and be a different model).
                                    # 79 deduped assistant messages, every one reporting
                                    # message.model = claude-opus-5: 158 input / 38,116 output /
                                    # 123,553 cache-write / 8,864,232 cache-read (98.2% cache
                                    # read). Snapshot taken while writing this block, so the
                                    # commit that lands it is not counted.
  estimated_usd: 23.84              # PER-COMPONENT at published Opus list rates, 1h cache-write
                                    # tier: $15/$75/$30/$1.50 per MTok for input/output/
                                    # cache-write/cache-read = $19.86, x1.20 uplift = $23.84.
                                    # Deliberately NOT AGENTS.md §4's flat fallback, which on a
                                    # 98.2%-cache-read session would report ~$135 — signal
                                    # `flat-rate-overstates-cached-sessions`.
  duration_minutes: 40              # first to last transcript timestamp (07:37:43Z to ~08:17Z),
                                    # wall clock; two ~9-minute full-suite runs dominate it
  branch: feat/spec-017-fixbadpixels-opcode
  pr: null                          # PR 17 was open on arrival; round 2 pushed to the branch only
  completed_at: 2026-09-08
  notes: "ROUND 2 (punch-list) closes HANDOFF-050's one ship blocker, SB-2, at 6e4376b. ONE file, ONE assertion, tests/develop.rs only - 15 insertions, 0 deletions, no src/ change, so decoded pixel output cannot move. THE ASSERTION, at tests/develop.rs:594-599 inside develop_output_is_bit_identical_across_two_runs (AC9), after the existing dst1==dst2 assert and before the count block: assert_eq!(dst1[2 * 5 + 2], 1000, 'develop_into must develop the plane the applier FIXED: the centre bad pixel must reach dst as its neighbours median, not the raw marker'), preceded by an 8-line comment at 586-593 explaining why 1000 vs 0 discriminates. BEFORE: the test asserted only dst1 == dst2 plus count_a == count_b from a DIRECT applier call, neither of which observes whether develop_into uses the repaired plane. AFTER: the centre sample is read out of develop_into's own dst. Value 1000 is MEASURED not assumed - the fixture sets black 0 / white 65535 / crop 5x5 / orientation None, making normalize an identity map, and the assertion passed first run on the honest tree. RED-PROOF DISCHARGED PERSONALLY, both directions, all three legs of the repo's mutation bar (file changed AND compiled AND output changed): src/develop.rs md5 b3b922d0ebd499f54c58bd86b39cb3e8 honest -> 1c4f4e34eed721173b90134302ee4ca9 with effective_src severed to src (one-occurrence-asserted textual substitution), cargo build succeeds, and the assertion goes RED at tests/develop.rs:594 with left 0 right 1000 -> reverted to b3b922d0 byte-identical, test green again. TEST-SUITE DELTA UNDER MUTATION, MEASURED not composed: 231 passed / 0 failed / 3 ignored honest -> 230 passed / 1 failed / 3 ignored mutant, the single failure being the new assertion. The mutant suite needed --no-fail-fast to produce that total: plain cargo test fail-fasts at the develop binary and never runs the remaining 99 tests, which would have made the total an inference rather than a measurement. Count is 231/0/3 on the honest tree BOTH before and after this round because I EXTENDED an existing test rather than adding one, exactly as the dispatch permitted. ⚠ ONE DISCREPANCY WITH THE VERIFIER, reported rather than papered over: HANDOFF-050 records the severed md5 as 548e240f; mine is 1c4f4e34. Same defect, differently typed - md5 covers the whole file, so any byte difference in how the sever was spelled moves it. I probed four other plausible spellings and none reproduces 548e240f either: use-site severing ff3aafb2, map-discard b4572839, commented binding 8f64c9f3, None.unwrap_or 9493ec06. The verifier's exact edit text is not recoverable from the handback, so my pair is the one measured on the shipped file and the one that satisfies DEC-004 rule 1; the honest md5 b3b922d0 matches theirs exactly, confirming we mutated the same starting file. ALL MUTATION IN A SCRATCHPAD COPY (rsync of the tree minus target/ and .git/); the working tree's src/develop.rs measured b3b922d0 before, during and after, and git status stayed clean apart from the two files this round edits. GATES on 6e4376b, every one run with its exit code actually read - build 0, typecheck 0, test 231/0/3, lint 0 (unpinned clippy 0.1.97, which is what this machine's PATH answers), lint-ci 0 on the PINNED clippy 0.1.98 that CI sees, cargo fmt --check 0 - note just fmt is NOT a recipe in this repo, fmt runs inside just lint - deny 0 and deny-fuzz 0 both licences ok, lint-no-allow 0, lint-red-proof 0 with control clean then injection rejected and all five lints fired, cost-audit-red-proof 0, msrv 0 on 1.90.0, fuzz-opcode 9,893,795 runs in 61 seconds with 0 crashes. My first gate pass printed blank exit codes because PIPESTATUS is a bashism and this shell is zsh; re-run capturing status directly rather than trusting the output text, since a gate whose result was never read is AGENTS.md 16 rule 3's exact failure. CI GREEN on 6e4376b, OBSERVED not inferred: gh run watch --exit-status returned 0 for both, and gh run view confirms 11 of 11 jobs success on each - push run 34202703543 and pull_request run 34202708489 - including rust / test and both fuzz smokes. NOTHING ELSE TOUCHED: no src/ file, no other test, DEC-024 and every other decision byte-unchanged, SPEC-017's body and ACs untouched (FU-8's stale AC3/AC5/AC10 text still stale, by instruction), the timeline left at build [~], no PR operation, no handback-sync run. FU-1 through FU-9 all still open and still owed a ship disposition. ⚠ ONE CORRECTION FOR THE ORCHESTRATOR, no action needed this round: the dispatch brief says the applier lives in src/opcode.rs and is unchanged by round 2, but apply_fix_bad_pixels_constant is defined in src/develop.rs - round 1's own Completion says so, and the md5 pair the verifier recorded for BOTH the SB-2 sever and the bar-9 applier mutation is one file, src/develop.rs, which is why both start at b3b922d0. src/opcode.rs holds the PARSER. That split is what FU-5 already names about the provenance row's placement. This round changed neither file. HANDOFF-044's top-level to_agent was CORRECTED from claude-sonnet-5 to claude-opus-5 before writing this block, so handback-sync stamps round 2's cost session with the model that actually ran it - SPEC-018/FU-11 recorded that exact miss twice when HANDOFF-046 was reused."
  synced_at: 2026-09-08
---

# HANDOFF-044: Build SPEC-017 — FixBadPixelsConstant opcode

## Delegation Summary

Build `SPEC-017`. Parse the `OpcodeList1` byte stream, dispatch on
opcode ID 4 (`FixBadPixelsConstant`), and replace every plane pixel
equal to `Constant` with the median of its 3×3 non-bad-pixel
neighbours — inserted into `develop_into` **before** the levels
normalise pass.

Branch from `main` (this handoff lands on
`feat/spec-017-fixbadpixels-opcode`, already cut from `origin/main`
at `4c6f285`).
`export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`
— the default root does not exist. Current test count on `main`: run
`cargo test --all-features` first and record it; that is the "before"
number for the Return Criteria.

## ⚠ Read this before the spec

- **The byte-level probe is already done.** `## Context` in the spec
  carries the 28 real bytes and their parsed field values for all
  three decodable Q2M frames, and every value survives byte-for-byte
  across frames. Reproduce with
  `exiftool -b -OpcodeList1 <file>.DNG | xxd` before writing the parser
  — the AC1 fixture is exactly these bytes — but do NOT re-derive the
  parsed values. The record you inherit is what the byte probe found.

- **Opcode ID is 4, not 5.** From-memory says 5 (FixBadPixelsList). The
  probe says 4. Verify against DNG 1.7.0.0 § Chapter 6 in build; do
  not paper over the probe if the spec surprises you.

- **Coordinate on `src/opcode.rs` with SPEC-018.** SPEC-018
  (WarpRectilinear) and SPEC-017 both land this module. SPEC-018's
  `depends_on: [SPEC-020]` puts it in the same waiting room as
  SPEC-017. If SPEC-018 built first, its module exists on `main` and
  you **extend** the `Opcode` enum with `FixBadPixelsConstant` — do
  NOT rewrite. If SPEC-017 built first (this handoff), the initial
  `Opcode` enum's `Unknown` variant is the hand-off SPEC-018 will
  extend. State which happened in the handback.

## The two things most likely to go wrong

**1. The opcode stream is BIG-ENDIAN.** The TIFF payload the rest of
the library handles is little-endian on the Q2M (DEC-008). The
opcode-list bytes are big-endian per DNG § Chapter 6. Writing the
parser with the wrong endianness produces `Constant = 0` regardless
of what the file says (because `0x00000000` reads the same either
way), then fails on any file where `Constant != 0`. Write the AC1
round-trip test FIRST and confirm it round-trips the exact 28-byte
fixture; any subsequent parser change that alters its output is a
defect.

**2. The 3×3 median with fewer than 8 valid neighbours needs
deterministic tie-breaks.** The design pre-registers the rules
(Notes section, four numbered points); apply them exactly. Even-count
median rounds to the LOWER of the two middles. `active_area` bounds
the region — a bad pixel at the true plane edge falls outside
`active_area` and is skipped (the "run inside active_area only"
pre-registration in Notes).

## The `+toolchain` trap applies here too

`cargo fuzz` still shells to a bare `cargo build`, and that INNER
call resolves to Homebrew's stable cargo, which rejects
`-Zsanitizer`. The `fuzz-opcode` recipe MUST be:

```
PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +nightly fuzz run opcode \
    fuzz/corpus/opcode fuzz/seeds/opcode -- -max_total_time=60
```

Match `just fuzz-plane` line-for-line in `app.just`. Add the same
recipe to `app.just`, add the AGENTS.md §6 code-block line, and add
the CI smoke step in the same PR (§12 bar 2 — fuzz targets arrive
with the parser).

## What the design already settled

- **The `Opcode` enum's initial shape** — see spec `## Implementation
  Context → Required in build (pre-registered)`. Two variants:
  `FixBadPixelsConstant { constant: u32, bayer_phase: u32 }` and
  `Unknown { id: u32, flags: u32, params: Vec<u8> }`. `params` is
  owned bytes so SPEC-018 can extend.

- **Where to place the applier call inside `develop_into`** — BEFORE
  `normalize`, per the pipeline order (`docs/measured-q2m-dng.md`
  line 44 and DNG § Chapter 6). The `Sensor` field
  `opcode_list_1: Option<Vec<u8>>` is populated from the IFD tag at
  the read path SPEC-014's tags come through; parsing happens
  inside `develop_into`, not on `Sensor` (satisfies the "unread
  field" rule).

- **The four fuzz seeds** — the 28-byte real payload plus three
  hand-crafted adversarial cases (truncated header, unknown
  mandatory opcode, length overflow at `0xFFFFFFFF`). Full byte
  sequences in the spec.

- **`docs/provenance-ledger.md` row** — one row: `src/opcode.rs`,
  source *DNG 1.7.0.0 § Chapter 6*, class 1. Honest per §16 rule 3.

## The four codified lessons apply to this spec

Read AGENTS.md §16 and apply them **while writing**, not at review:

1. **`measurement-over-generalised`** — every claim in the spec's
   Context is measured against a specific command (the `exiftool -b`
   probe). Do not soften them into generic "DNG opcodes work like ..."
   sentences.
2. **`attribute-text-inside-doc-comments`** — the panic-free lint
   scanner already trips this trap; if you add any code-scanning
   assertions in tests or CI, assert the match count and exclude doc
   comments.
3. **`a-gate-that-fails-mutely-is-a-gate-that-never-ran`** — every new
   `just fuzz-opcode` piece dies through its own error message, not
   through `set -o pipefail`.
4. **`unrun-docs-carry-errors`** — the DNG spec section citations MUST
   be verified in build by reading the spec directly. Do not cite a
   clause number from memory.

## The `a-fix-inherits-the-precondition-of-the-thing-it-fixes` lesson

Codified this week: when reviewing or extending an assumption, mutate
the ASSUMPTION, not the logic that depends on it. For SPEC-017 the
assumption to mutate is: **"the parser reads `Constant` from bytes
4..8 of the parameter block"**. A test that hardcodes the value 0
proves nothing about the reader. AC1's round-trip test asserts the
exact fixture bytes; that path exercises the assumption.

## Two facts about this machine, measured 2026-09-06 — do not assume

- **`exiftool` version 13.55 is on PATH**; the probe commands in the
  spec's `## Context` all work.
- **The Q2M corpus has 3 frames** at
  `$IRRADIANCE_CORPUS_DIR/LEICA-Q2-MONO/`: `L1021223.DNG`,
  `L1026016.DNG`, `L1026192.DNG`. All three carry the same 28-byte
  OpcodeList1 payload (verified). AC5's HIT count is measured on
  ALL THREE; do not stop at one.

## Return Criteria

1. **All acceptance criteria met and their named tests pass**
   (`## Failing Tests` in the spec; the criteria map to tests
   AC1..AC10, AC11 is the CI gate). Ten test names — confirm each
   is a real test with `cargo test --all-features <name> -- --exact
   --nocapture` and that ALL ten pass. Report `count of tests before`
   → `count of tests after` in the handback.

2. **Ten gates + `just lint-ci` + `just fuzz-opcode`**, run by you,
   pasted, summed across all targets, clippy version asserted (local
   0.1.97; CI floats at 0.1.98). ⚠ **Say which gate list you ran** —
   the count is genuinely ambiguous
   (`the-gate-count-is-not-defined-anywhere`); report the ambiguity,
   do not resolve it.

3. **Push and read CI.** Observed green on your SHA, run id and job
   count. The `fuzz-opcode` smoke run must land in the same PR
   (§12 bar 2).

4. **The red-proof watched and pasted** (AC8). File changed AND
   compiled AND *output changed*. ⚠ That third clause caught a false
   red-proof in `PATCH-002` two days ago, where the obvious injection
   removed the very text the detector was written to survive — so
   check that your injection exercises the path you think it does.

5. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build
   lost its entire change to `git checkout --`, and the orchestrator
   did the same thing to `PATCH-002` this week. Mutate in an isolated
   copy, or commit first; md5-verify every revert.

6. **AC5's three HIT counts recorded in the handback.** The per-frame
   replaced-pixel counts on `L1021223.DNG`, `L1026016.DNG`,
   `L1026192.DNG`. These are DATA — the first real measurement of Q2M
   dead-pixel density this repo has.

7. **AC7's three deltas recorded in the handback.** SSIMULACRA2 score
   BEFORE FixBadPixelsConstant applied vs AFTER, per frame. Depending
   on when SPEC-020's oracle lands relative to this build, the "before"
   scores may need SPEC-020's oracle to be in `main` first; if AC7's
   test cannot run because SPEC-020 has not shipped, mark AC7 as
   `blocked-on-spec-020` in the handback (not skipped, not passed).

8. **Provenance ledger row landed** (AC10). One row, class 1, per §16
   rule 3.

9. **`## Follow-ups` table intentionally deferred** — dispositions
   happen at ship, not build. Raise findings as `FU-N` in the
   handback narrative and let the reviewer or ship cycle disposition
   them (§15 rule: follow-ups get their four dispositions at ship of
   the raising spec, and never cross that ship undecided).

10. **State which of SPEC-017/018 landed `src/opcode.rs` first.**
    In one line in the handback. If SPEC-018 landed first, you
    extended; if SPEC-017 (this handoff) landed first, you scaffolded
    for SPEC-018 to extend.

## Cost

Fill the `handback:` block above with the real numbers from your
interface. `just handback-sync SPEC-017` (from an orchestrator's
session) transcribes them into `cost.sessions` — do not hand-edit
that list. `notes:` is ONE PHYSICAL LINE.

## References

- `projects/PROJ-001-monochrome-dng-develop/specs/SPEC-017-fixbadpixelsconstant-opcode.md`
  — the spec (the primary artifact; `## Failing Tests`, `## Acceptance
  Criteria` and `## Implementation Context` are the working section
  headers).
- `projects/PROJ-001-monochrome-dng-develop/stages/STAGE-003-*.md`
  — the parent stage. Note the design-time constraint: "The test must
  assert the branch was HIT, not merely that the image came out
  unchanged."
- `docs/measured-q2m-dng.md` — line 18 (OpcodeList1 line) and line 44
  (pipeline order).
- `docs/oracle-contract.md` — the three oracle layers, and where the
  develop layer (SPEC-020) attaches.
- `docs/provenance-ledger.md` — where AC10's row lands.
- `AGENTS.md` §5 (measured toolchain, the four `+toolchain` traps),
  §6 (commands — every recipe's commands appear in the block), §11
  (unread-field discipline), §12 (the four bars — oracles ship red,
  fuzz targets ARRIVE with the parser, layer-0 assertions are free,
  skipped tests are loud), §15 (the cycle contract), §16 (the four
  codified lessons).
- `guidance/constraints.yaml` — this file wins. `no-panics-on-untrusted-
  input`, `oracle-must-be-shown-red`, `provenance-recorded-per-
  algorithm`, `library-not-application`, `test-before-implementation`
  all apply to SPEC-017.
- `guidance/toolchain-brief.md` — the fuller version of §5's toolchain
  table, filled 2026-08-16.
- **DNG 1.7.0.0 specification § Chapter 6 (Opcode Lists)** — the
  authoritative source. Read it in build.

## What this handoff does NOT ask for

- **Other FixBadPixels\* opcodes.** FixBadPixelsList (opcode ID 5) is
  not present on any decodable frame; do not implement its parser.
- **A Bayer-aware neighbour selection.** BayerPhase is read but not
  branched on — PROJ-002 adds that when the first Bayer camera lands.
- **A general-purpose median-of-N routine.** The 3×3 median is
  hand-written for this opcode; no library-wide extraction.
- **A `DEC-*` for the algorithm.** DNG § Chapter 6 is the spec; the
  provenance-ledger row is the record. A DEC is only needed for the
  replaced-count surfacing shape if build finds a non-obvious choice.

## Completion

**Which of SPEC-017/018 landed `src/opcode.rs` first: SPEC-018.** This
build extended the `Opcode` enum with `FixBadPixelsConstant { constant,
bayer_phase }`, added the `OpcodeID == 4` parser branch, and added
`parse_fix_bad_pixels_constant` as the `develop_into` entry point — no
rewrite of SPEC-018's parser, framing, or error dispatch.

**Chapter correction (`unrun-docs-carry-errors`).** This handoff and the
spec both cite "DNG 1.7.0.0 § Chapter 6" for `FixBadPixelsConstant`.
Fetched the published DNG 1.6.0.0 PDF directly during build (the 1.7.0.0
PDF times out this session's fetch tool at its ~300-page size; 1.6.0.0's
"Opcode List Processing" chapter is unchanged in later versions per its own
Compatibility Issue 6/16 notes) and read its own table of contents plus the
`FixBadPixelsConstant` page directly: it is **Chapter 7, "Opcode List
Processing"**, p.95 — the SAME chapter as `WarpRectilinear` (already
correctly cited as Chapter 7 in `src/opcode.rs`'s own module doc from
`SPEC-018`). Chapter 6 is "Mapping Camera Color Space to CIE XYZ Space" —
unrelated. Corrected in `src/opcode.rs`'s module doc and
`docs/provenance-ledger.md`'s row. Opcode ID 4 (not 5) was independently
re-confirmed against the same PDF page.

**Design decision: the applier lives in `src/develop.rs`, not a separate
module.** Unlike `SPEC-018`'s `src/warp.rs`, `apply_fix_bad_pixels_constant`
is defined directly in `src/develop.rs` (matches the spec's own `##
Outputs` bullet, which describes it under the `src/develop.rs` entry, not a
new-module entry). Its algorithm tests (`AC4`, `AC6`, `AC8`, `AC9`) live in
`tests/develop.rs`; parser-only tests (`AC1`-`AC3`) are in the existing
`tests/opcode.rs` (created by `SPEC-018`, extended here — not a new file).

**Return Criteria 1 — test count.** Before this build (reconstructed by
subtracting the 15 test functions this build added, since I did not
capture a literal `main` baseline before editing): 217 passed / 2 ignored /
0 failed. After: **231 passed / 3 ignored / 0 failed** (full
`cargo test --all-features`, real corpus present, 0/7 tier-B skips). All
ten named `## Failing Tests` exist as real, discoverable tests
(`cargo test <name> -- --exact --nocapture` confirmed for each); nine pass,
one (`q2m_frames_have_at_least_one_replaced_pixel`, `AC5`) is `#[ignore]`d
with a measured reason — see **SB-1** below, not a silent skip.

**Return Criteria 2 — gates run.** Ran the ELEVEN-gate list AGENTS.md §6
documents (build, test, lint [unpinned], lint-ci [pinned, CI-equivalent],
typecheck, deny, deny-fuzz, lint-red-proof, lint-no-allow, msrv) plus
`fuzz-opcode` and `cost-audit-red-proof` (not in the eleven, run anyway).
Every one green:
- `cargo test --all-features`: 231 passed, 3 ignored, 0 failed.
- `just lint-ci`: clippy 0.1.98 (pinned `~/.cargo/bin/cargo +stable`,
  matches CI's floating version measured this session), 0 warnings under
  `-D warnings`.
- `just lint` (unpinned, Homebrew 0.1.97): 0 warnings.
- `cargo fmt --check`: clean (after one `cargo fmt` pass on this build's
  own new code).
- `cargo check --all-targets --all-features`: clean.
- `just deny` / `just deny-fuzz`: both "licences ok" (pre-existing
  unmatched-allowance warnings only; no new dependency, `Cargo.toml`
  0 lines changed).
- `./scripts/lint-red-proof.sh`: control clean, injection rejected (all
  five lints fired), non-`-D warnings` run still rejects — pass.
- `just lint-no-allow`: clean.
- `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` (MSRV):
  clean.
- `./scripts/cost-audit-red-proof.sh`: pass (pre-existing gate, unaffected).
- `just fuzz-opcode` (60s): **13,696,114 runs, 0 crashes.**

**Return Criteria 3 — push and CI.** Pushed as `cd82ca86f065a12592c8804c31357596e89d738d`
(`cc129d6..cd82ca8`). CI run `34191738033`: **`completed` / `success`,
11/11 jobs green**, including the new `rust / fuzz smoke — opcode (60s)`
job this build added (`.github/workflows/ci.yml`) — `cost-capture audit`,
`rust / fmt --check`, `rust / license policy (cargo-deny)`, `rust /
license policy — fuzz graph (cargo-deny)`, `rust / test`, `rust / lint
policy red-proof (must fail red)`, `rust / fuzz smoke — warp_opcode
(60s)`, `rust / panic-free policy — no #[allow] escape (--lib)`, `rust /
MSRV (1.90.0)`, `rust / clippy -D warnings`, `rust / fuzz smoke — opcode
(60s)`. Observed via `gh run view 34191738033`, not self-reported.

**Return Criteria 4 — AC8 red-proof, watched.** `DEC-017`'s mutate-copy-
rebuild-run mechanism (own copy in `tests/develop.rs`, mirroring
`tests/develop_oracle.rs`'s `inject_orientation_identity_fault`): injected
fault discards the median write (`*slot = median;` → no-op). Watched:
FILE CHANGED (one-occurrence-asserted textual injection) AND COMPILED
(`cargo build --release`) AND OUTPUT CHANGED (`honest=(1,1000)
mutant=(1,0)` — the centre pixel stays at the marker value `0` in the
mutant, satisfying `AC8`'s "OR" clause). Negative control
(`fix_bad_pixels_red_proof_control_is_green`) confirms the unmutated
copy-and-rebuild apparatus reproduces the real applier's output exactly.

**Return Criteria 5 — staged before mutate-and-revert.** The red-proof
mutates a TEMP DIRECTORY COPY of the crate (`DEC-017`'s mechanism), never
the working tree — the `git checkout --`-loses-everything failure mode
`SPEC-010`/`PATCH-002` hit structurally cannot recur here.

**Return Criteria 6 — AC5's three HIT counts.** `constant = 0` on all
three (byte-identical `OpcodeList1`, independently re-verified against the
real files this session):
| Frame | HIT count (rules 1-3) | Left-at-constant (rule 4, no valid neighbour) |
|---|---|---|
| `L1021223.DNG` | **0** | 0 |
| `L1026016.DNG` | **0** | 0 |
| `L1026192.DNG` | **0** | 0 |

See **SB-1**.

**Return Criteria 7 — AC7's three score deltas.** SPEC-020 already shipped
on `main` (SSIMULACRA2 vs `dnglab analyze --srgb`), so this ran for real,
not `blocked-on-spec-020`:
| Frame | Before | After | Delta |
|---|---|---|---|
| `L1021223.DNG` | -60.169 | -60.169 | +0.000 |
| `L1026016.DNG` | SKIP — see **FU-2** | — | — |
| `L1026192.DNG` | -40.281 | -40.281 | +0.000 |

Deltas are exactly zero because AC5 measured zero replaced pixels on every
frame (Return Criteria 6) — `develop_into`'s output is byte-identical
whether or not `FixBadPixelsConstant` runs, on these three files. AC7's
own assertion (`after >= before`) is satisfied, honestly, by this trivial
case. Runtime note: this test takes ~500s locally (SSIMULACRA2 scoring of
real 47-megapixel renders, twice per comparable frame) — refactored once
already to fetch `dnglab`'s reference ONCE per frame instead of twice (the
reference cannot differ between "before" and "after"), which did not
meaningfully change the wall-clock (SSIMULACRA2 itself, not the `dnglab`
subprocess, dominates). Not optimized further; flagged as a cost note, not
a defect.

**Return Criteria 8 — provenance row.** `docs/provenance-ledger.md`'s
existing `src/opcode.rs` row (`SPEC-018`) extended with a `**SPEC-017**
extended this row` paragraph — class 1 for the opcode identity/parameter
shape (DNG 1.7.0.0 Chapter 7 p.95), class 1 as a public-domain technique
for the median-filter kernel (not from the spec, not from any
implementation).

**Return Criteria 9 — Follow-ups table.** Intentionally NOT added to the
spec (dispositions happen at ship, per §15). Findings raised below.

**Return Criteria 10 — coordination.** SPEC-018 landed `src/opcode.rs`
first; this build extended it (stated above and in the provenance row).

### Findings for verify/ship

**SB-1 — AC5 measured HIT = 0 on all three decodable Q2M frames, not
"small-but-positive."** `unpack_into`'s raw plane (already bit-exact
against `dnglab --raw-checksum`, `SPEC-013`) contains **zero** samples
equal to `0` (the `Constant` all three frames' real `OpcodeList1` bytes
declare) — measured minimums 2 / 30 / 2, scanned across the WHOLE raw
plane (`active_area` and padding both), not just `active_area`. This is
independently corroborated, not a guess about one test: `AC1` proves the
parser reads `Constant = 0` correctly from the real bytes; `AC4` and `AC8`
prove the applier's replace-with-median algorithm is correct and reachable
when a bad pixel IS present (hand-built fixture, and the red-proof
mutating that exact code path). Neither of the two failure modes AC5's own
spec text names ("the applier is not being reached, or `constant` is being
read from the wrong place") holds. The honest, verified conclusion: these
three Leica Q2M frames currently carry zero photosites flagged bad by this
convention, despite the mandatory `FixBadPixelsConstant` opcode being
present on every one. `q2m_frames_have_at_least_one_replaced_pixel` is
`#[ignore]`d with this exact reasoning (re-run with `cargo test --
--ignored` to reproduce), not weakened or deleted. **Needs a verify/ship
judgment call**: relax `AC5`'s threshold to `>= 0` (accept the measurement),
or treat as a real gap needing more corpus (a fourth frame, or a different
Q2M body) before this spec can ship green on its own stated terms.

**FU-1 — AC3's pre-registered example (`Flags: 0`, expecting
`Ok(Opcode::Unknown{..})`) is unreachable given SPEC-018's shipped parser.**
SPEC-018, which built `src/opcode.rs` first, already dispatches mandatory-
vs-optional AT PARSE TIME: an unrecognized ID with `Flags` bit 0 clear is
`Error::UnsupportedMandatoryOpcode` (see `mandatory_unknown_opcode_is_
rejected`, `src/opcode.rs`, already shipped). `Opcode::Unknown` for an
unrecognized ID is reachable only with `Flags` bit 0 SET (optional).
`opcode_parser_returns_unknown_for_ninetynine` uses `flags: 1` instead of
the spec's literal `flags: 0`, documented inline. `AC6`'s mandatory-
unknown-in-`OpcodeList1` case (the scenario the spec's `flags: 0` example
was actually reaching for) is covered separately by
`develop_errors_on_mandatory_unknown_opcode` (`tests/develop.rs`), which
exercises `parse_opcode_list`'s EXISTING error through `develop_into` —
not a second dispatch layer. No behavior gap; a design-time assumption
about where the dispatch would live, corrected by what SPEC-018 actually
shipped.

**FU-2 — `dnglab --srgb` never applies EXIF `Orientation`; `develop_into`
does; the two are structurally incomparable on any rotated frame.**
Measured directly: `dnglab analyze --srgb L1026016.DNG` prints `8368
5584` (un-rotated, sensor-native), while this file's `Orientation: 6`
(Rotate 90 CW) makes `develop_into`'s own output `5584x8368` (`SPEC-014`,
already-shipped, already-tested: `orientation_six_swaps_the_output_
dimensions`). `q2m_develop_oracle_score_not_worse_after_fixbadpixels`
(this build) skips `L1026016.DNG` loudly rather than panicking. **This is
not a SPEC-017 defect** — the identically-shaped `develop_and_score` in
`tests/warp.rs` (`warp_scores_at_least_eightyfive_via_spec_020_oracle`,
`#[ignore]`d since `SPEC-018`) would hit the SAME mismatch were it ever run
un-ignored against `L1026016.DNG`/`L1026192.DNG` — a pre-existing gap in
the `SPEC-020` oracle's own methodology for rotated frames, surfaced here
because this is the first spec to actually run its oracle comparison
un-ignored across all three frames. Candidate disposition: `signal:` (a
recurring pattern in how the oracle handles orientation) or a small future
spec to make `tests/support/pnm.rs`/the scoring helper orientation-aware.

### Reflection

1. **What would I do differently next time?** Verify a design-time
   assumption about real-world data density (AC5's "small-but-positive")
   against the corpus BEFORE writing the test's hard assertion, the same
   design-time-probe discipline AGENTS.md §12 already requires for byte
   layouts — "bad pixels are rare but present" turned out to be "bad
   pixels are absent on this sample," and that would have been cheaper to
   discover at design than at build.
2. **Does any template, constraint, or decision need updating?** Possibly:
   the `SPEC-020` oracle's `dnglab --srgb` comparison has no orientation
   handling (`FU-2`) — worth a `guidance/signals.yaml` entry if the ship
   cycle doesn't spin it into its own spec.
3. **Is there a follow-up spec I should write now?** Not this session —
   `FU-2`'s fix (orientation-aware oracle scoring) is more naturally
   ship's or verify's call, since it affects `SPEC-018`'s own ignored
   tests too, not only this spec's.
4. **Where was the worst defect caught?** `build` — SB-1 (AC5's premise)
   surfaced only once the applier ran against the real corpus, which is
   exactly what AC5 was there to catch; nothing escaped past this cycle.
5. **What can a user do now that they couldn't before?** Before: a Q2M
   plane's dead pixels (per the camera's own in-camera defect flag) passed
   through `develop_into` unmodified, silently ignoring a mandatory DNG
   opcode. After: `develop_into` applies `FixBadPixelsConstant`
   correctly and provably (AC1/AC4/AC8) whenever a flagged bad pixel is
   present in the raw plane — confirmed not yet exercised by any of this
   repo's three sampled Q2M frames (SB-1), the first real measurement of
   this camera's dead-pixel density.

---

## Completion — round 2 (punch-list)

**Scope: `HANDOFF-050`'s one ship blocker, `SB-2`, and nothing else.** One
file, one assertion, `tests/develop.rs` only — 15 insertions, 0 deletions,
**no `src/` change**, so decoded pixel output cannot move. Shipped at
`6e4376b`, on top of `8ae24e1` (the round-2 dispatch commit).

### Round 1's numbers, preserved

The `handback:` block above carried round 1's figures until 2026-09-08:
**570,000 tokens, $4.10, 90 minutes, on `claude-sonnet-5`**, notes
`"AC5 measured HIT=0 on all 3 Q2M frames (SB-1, real finding); AC1-4/6/8/9
green, AC7 green; SPEC-018 landed opcode.rs first, extended it; see handback
narrative"`. Those are already transcribed into `SPEC-017`'s `cost.sessions`
as the **first** `cycle: build` entry, so the overwrite loses nothing —
the same overwrite `SPEC-018`'s `HANDOFF-046` performed for its rounds 2
and 3. `synced_at` is reset to `null` so `just handback-sync SPEC-017`
appends round 2 as a **second** build session.

`to_agent` was corrected `claude-sonnet-5` → `claude-opus-5` **before**
writing this block. `scripts/handback-sync.sh` line 97 reads `to_agent`
from the handoff, so leaving it would have stamped round 2's cost session
with round 1's model — `SPEC-018/FU-11`, recorded twice when `HANDOFF-046`
was reused. Verified from this session's own transcript (scratchpad UUID
`83d8a76b-4b2b-4bfa-9216-e6d0600996e4`, not text-matched): 79 deduped
assistant messages, `message.model = claude-opus-5` on every one.

### The assertion — `tests/develop.rs:586-599`

It lands inside `develop_output_is_bit_identical_across_two_runs` (`AC9`),
after the existing `dst1 == dst2` assertion and before the direct-applier
count block.

**Before** — the whole of what the test observed about `develop_into`:

```rust
    develop_into(&sensor, &src, &mut dst1).expect("fits");
    develop_into(&sensor, &src, &mut dst2).expect("fits");
    assert_eq!(
        dst1, dst2,
        "the same input must produce bit-identical output across two runs"
    );
```

**After** — the same, plus:

```rust
    // `SPEC-017/SB-2` — assert the branch was HIT, not merely that the image
    // came out unchanged (STAGE-003). This fixture makes normalize an identity
    // map (black 0, white 65535, 5x5 crop, no orientation), so the centre
    // sample reaches `dst` unscaled: `1000` — the median of its eight
    // 1000-valued neighbours — if `develop_into` develops the FIXED plane, and
    // `0` — the marker `src` still carries — if the `effective_src` wiring is
    // severed. Both runs stay bit-identical either way, so the assertion above
    // does not observe the wiring at all; this one does.
    assert_eq!(
        dst1[2 * 5 + 2],
        1000,
        "develop_into must develop the plane the applier FIXED: the centre bad \
         pixel must reach dst as its neighbours' median, not the raw marker"
    );
```

The old assertions are untouched and the direct-applier `count_a == count_b`
block still follows. Neither of those observes whether `develop_into` uses
the repaired plane — that was exactly `SB-2`.

**`1000` is measured, not assumed.** The fixture sets `black_level = 0`,
`white_level = 65535`, `default_crop_size = 5x5` and `orientation = None`,
which makes normalize an identity map; the assertion passed on its first
run against the honest tree, which is what establishes the value rather
than the arithmetic in this paragraph.

### Red-proof — discharged personally, both directions

All three legs of this repo's mutation bar: the file **changed**, it
**compiled**, and the output **changed**.

| Step | `src/develop.rs` md5 | Result |
|---|---|---|
| honest | `b3b922d0ebd499f54c58bd86b39cb3e8` | assertion green |
| `effective_src` severed to `src` | `1c4f4e34eed721173b90134302ee4ca9` | compiles; assertion **RED** at `tests/develop.rs:594`, `left: 0` / `right: 1000` |
| reverted | `b3b922d0ebd499f54c58bd86b39cb3e8` | byte-identical to honest; assertion green again |

The mutation is a one-occurrence-asserted textual substitution of
`let effective_src: &[u16] = fixed_plane.as_deref().unwrap_or(src);` by
`let effective_src: &[u16] = src;`, applied to a **scratchpad copy** of the
crate (`rsync` minus `target/` and `.git/`). The working tree's
`src/develop.rs` measured `b3b922d0` before, during and after.

**Test-suite delta under mutation — measured, not composed:**

| Tree | Result |
|---|---|
| honest | **231 passed / 0 failed / 3 ignored** |
| mutant | **230 passed / 1 failed / 3 ignored** |

The one failure is the new assertion. Getting that total required
`--no-fail-fast`: plain `cargo test` fail-fasts at the `develop` binary and
never runs the remaining 99 tests, which would have made the suite total an
inference rather than a measurement (§16 rule 1). The honest count is
`231/0/3` **both before and after** this round because I **extended an
existing test** rather than adding one — the dispatch allowed either.

### ⚠ One discrepancy with the verifier, reported not papered over

`HANDOFF-050` records the severed md5 as `548e240f`; mine is `1c4f4e34`.
Same defect, differently typed — md5 covers the whole file, so any byte
difference in how the sever was spelled moves it. Four other plausible
spellings were probed and none reproduces `548e240f` either:

| Spelling | md5 |
|---|---|
| sever at the two use sites, keep the binding | `ff3aafb2` |
| `.as_deref().map(\|_\| src).unwrap_or(src)` | `b4572839` |
| `= src; // SEVERED` | `8f64c9f3` |
| `None.unwrap_or(src)` | `9493ec06` |

The verifier's exact edit text is not recoverable from the handback, so the
pair above is the one measured on the shipped file and the one that
satisfies `DEC-004` rule 1. The **honest** md5 `b3b922d0` matches theirs
exactly, which confirms we mutated the same starting file.

### Gates — on `6e4376b`, every exit code actually read

| Gate | Exit | Note |
|---|---|---|
| `just build` | 0 | |
| `just typecheck` | 0 | |
| `just test` | 0 | 231 / 0 / 3, corpus present |
| `just lint` | 0 | unpinned clippy **0.1.97** (what this machine's PATH answers) + `cargo fmt --check` |
| `just lint-ci` | 0 | **pinned clippy 0.1.98** — what CI sees |
| `cargo fmt --check` | 0 | `just fmt` is **not** a recipe here; fmt runs inside `just lint` |
| `just deny` / `just deny-fuzz` | 0 / 0 | licences ok |
| `just lint-no-allow` | 0 | |
| `just lint-red-proof` | 0 | control clean → injection rejected → all five lints fired |
| `./scripts/cost-audit-red-proof.sh` | 0 | |
| `just msrv` | 0 | 1.90.0 |
| `just fuzz-opcode` (60 s) | 0 | **9,893,795 runs in 61 s, 0 crashes** |

The first gate pass printed **blank** exit codes: `PIPESTATUS` is a bashism
and this shell is zsh. Re-run capturing status directly rather than reading
success out of the output text — a gate whose result was never read is
§16 rule 3's exact failure mode, and it nearly landed in this round's own
evidence.

**CI green on `6e4376b`, observed not inferred.** `gh run watch
--exit-status` returned 0 for both runs, and `gh run view` confirms **11 of
11 jobs `success`** on each: push run `34202703543` and pull_request run
`34202708489`, including `rust / test`, `rust / fuzz smoke — opcode (60s)`
and `rust / fuzz smoke — warp_opcode (60s)`.

### What round 2 did NOT touch

No `src/` file. No other test. `DEC-024` and every other decision record
byte-unchanged. `SPEC-017`'s body and acceptance criteria untouched —
`FU-8`'s stale `AC3`/`AC5`/`AC10` text is **still stale**, by instruction.
The timeline stays at build `[~]`. No PR operation: PR #17 was open on
arrival and received a `git push` only. `handback-sync` not run — verified
parseable with `--dry-run` (`HANDOFF-044 → cost.sessions[build]
tokens=9026059`), which left the spec unmodified. **`FU-1` through `FU-9`
are all still open and still owed a ship disposition.**

### ⚠ One correction for the orchestrator — no action needed this round

The round-2 dispatch brief says the applier lives in `src/opcode.rs` and is
"unchanged by round 2". `apply_fix_bad_pixels_constant` is in fact defined
in **`src/develop.rs`** — round 1's own `## Completion` says so under
*"Design decision: the applier lives in `src/develop.rs`, not a separate
module"*, and it is why the md5 pair the verifier recorded for the `SB-2`
sever and the one recorded for the bar-9 applier mutation **both start at
`b3b922d0`**: they are the same file. `src/opcode.rs` holds the parser.
That split is what `FU-5` already names about the provenance row's
placement. Round 2 changed neither file.
