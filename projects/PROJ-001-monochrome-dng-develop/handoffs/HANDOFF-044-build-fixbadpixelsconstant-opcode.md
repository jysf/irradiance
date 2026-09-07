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
  to_agent: claude-opus-5           # ⚠ PREDICTION from tier_map.build, not a
                                    # measurement. Standing record: 0 FOR 11 on
                                    # the build hint. CORRECT THIS to what your
                                    # own system prompt reports as `message.model`.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-06
  status: pending                  # pending | accepted | completed | rejected

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
handback:
  status: null                     # completed | blocked | rejected
  tokens_total: null               # REAL combined count — what cost-audit reads
  estimated_usd: null              # tokens_total × your rate, or your harness's number
  duration_minutes: null
  branch: null
  pr: null
  completed_at: null               # YYYY-MM-DD
  notes: null                      # one line if unusual (rework, no meter, etc.)
  synced_at: null                  # stamped by `just handback-sync` — do not edit
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
