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
  id: HANDOFF-067
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-29
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-028

project:
  id: PROJ-002
  stage: STAGE-006
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
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 8884772            # REAL combined count — what cost-audit reads
  estimated_usd: 21.10             # tokens_total × your rate, or your harness's number
  duration_minutes: 40
  branch: verify-spec-028
  pr: null                         # verify-bookkeeping PR — filled in the PR-ref commit
  completed_at: 2026-10-29         # YYYY-MM-DD
  notes: "✅ APPROVED at commit 233ae5c (PR #41, already MERGED to main before this verify ran — a retrospective independent review; no build branch remained to push to, hence this small verify-bookkeeping PR). Every AC reproduced cold: full suite (14 binaries, 0 failed, corpus 7/7 so the Nikon tier-B test actually ran, not skipped), pinned CI clippy 0.1.98 -D warnings, cargo fmt --check, just validate, just lint-no-allow (the five panic lints), and `plane` fuzz 20,465,198 runs/91s 0 crashes. Adversarial mutation (ceil->floor on per-strip layer-0) flips a_two_strip_14bit red while 16-bit stays green, control green restored. Nikon 12-strip plane assembles to 4352x2868 then trips SampleExceedsWhiteLevel (sample 741623=3835>3808) — expected, SPEC-029's to relax. Both build deviations adjudicated ACCEPTABLE; one non-blocking follow-up SPEC-028/FU-1 (amend the spec Outputs formula to the byte-ceil form at ship). tokens_total summed from this verify session's own transcript JSONL (scratchpad-UUID anchored, deduped by message.id; all 52 usage records claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (8.64M of 8.88M); estimated_usd at per-category Opus list rates. Floor-by-id snapshot, not a budget delta."
  synced_at: 2026-10-29
---

# HANDOFF-067: Multi-strip mosaic plane assembly — unpack a Bayer plane stored as N strips (`plane::unpack_into` is single-strip-only, SPEC-012 `src/plane.rs`; the Nikon D300S is 12 strips) — VERIFY

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-028` to
`claude-opus-4-8` (acting as verifier) for the **verify** cycle — an independent
cold review of the build (PR #41, HANDOFF-066) against the spec's Acceptance
Criteria, re-running every gate and the fuzz target rather than trusting the
build's report or a CI badge, and adjudicating the two deviations the build flagged.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-028-...-12-strips.md`
  — the 7 Acceptance Criteria and the Failing Tests are the contract.
- **Build handoff:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-066-build-...md`
  — deliverables, the two flagged deviations, the honest reflection.
- **The code the change sits in:** `src/plane.rs` (`unpack_into` — the N-strip
  loop, per-strip layer-0, per-strip read + unpack), `src/lib.rs`
  (`Error::MissingRowsPerStrip`, the repurposed `UnsupportedStripLayout`),
  `tests/plane_unpack.rs` (the `MultiStrip` fixture + the 10 new tests),
  `docs/provenance-ledger.md` (the extended `src/plane.rs` row).

### Decisions that apply

- `DEC-008` — the two-path unpack (byte-aligned 8/16 vs sub-byte 12/14 MSB-first),
  and its load-bearing rule that **each strip is byte-padded independently**.
- `DEC-016` — `unpack_into` allocates nothing; the caller owns `dst`.

### Constraints that apply

- `no-panics-on-untrusted-input` (blocking) — every strip offset/length
  bounds-checked, every product `checked_*`/`saturating_*`, every write through
  `split_at_mut_checked`.
- `test-before-implementation`, `one-spec-per-pr`, `provenance-recorded-per-algorithm`.

### Prior related work

- `HANDOFF-066` / `PR #41` — the build under review (merged `233ae5c`).
- `SPEC-029` — relaxes the `SampleExceedsWhiteLevel` the Nikon now trips.
- `SPEC-027` — registers the Nikon in the manifest + a multi-strip/CFA fuzz seed.

## Expected Deliverables

*(verify contract — see below.)*

### The verify contract

Reviewed cold; did not write this code. Output exactly ONE of
✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- The ship actions (archive / CHANGELOG / brag) — the maintainer's main-session
  step (Prompt 5). This verify stays at `cycle: verify` and prepares the ship prompt.
- SPEC-029 (WhiteLevel policy), SPEC-025 (bit-exact oracle), SPEC-027 (corpus/fuzz
  registration), tiled planes — all out, as the spec says.

## Return Criteria — how to hand back

Verdict returned below; handback front-matter filled; `handoff.status` = completed.

---

## Handback

*Filled in by the receiving agent. The orchestrator transcribes it.*

### Execution notes

- **Branch:** `verify-spec-028` (cut from `origin/main` @ `233ae5c`).
- **Head reviewed:** `233ae5c2b136d7966ce845c1bf8495db59e09bcb` (PR #41 merge head,
  == `origin/main`). **PR #41 was already MERGED to `main` before this verify ran**
  (the ship happened in the main session); this verify is the retrospective
  independent review, and the verify-bookkeeping PR is cut from `verify-spec-028`.
- **Completed at:** 2026-10-29
- **All acceptance criteria met?** Yes — every AC reproduced independently.
- **Verdict: ✅ APPROVED** at commit `233ae5c`. No ship-blocking findings.

#### What I reproduced (my own runs, not the build's word or a CI badge)

All gates run from a fresh `verify-spec-028` worktree with
`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`:

- **`just test`** — 14 test binaries, **0 failed** in every one; `corpus: 7/7
  present — no tier-B test will skip`. The `plane_unpack` binary ran 16 tests, all
  ok, including the 10 SPEC-028 tests and the two real single-strip regressions
  (`unpacks_fourteen_bit_msb_first_samples` Q2M, `unpacks_sixteen_bit_in_file_byte_order`
  M Monochrom). The Q2M/M-Monochrom develop oracles stayed green (the key regression).
  One ignored test is the pre-existing `SPEC-017/SB-1` ignore, unrelated.
  - Guarded against the SPEC-024-verify `| tee` trap: verdict read from the actual
    `test result:` lines, not the pipeline exit code.
- **Pinned CI clippy** (`just lint-ci`, clippy 0.1.98, `--all-targets --all-features
  -- -D warnings`) → exit 0, no warnings.
- **`cargo fmt --check`** → exit 0. **`just validate`** → exit 0 (33 artifacts
  valid front-matter, 143 parseable). **`just lint-no-allow`** (the five panic
  lints over the lib) → exit 0 — the blocking `no-panics-on-untrusted-input`
  constraint holds for `src/plane.rs`.
- **`just decisions-audit --changed 658ba64`** — flags DEC-004, DEC-008, DEC-009,
  DEC-016 as governing the touched files; confirmed consistent with each (byte-align
  branch + independent per-strip padding = DEC-008; `split_at_mut_checked` into the
  caller's `dst`, zero loop allocation = DEC-016; analytical tests = DEC-004;
  red-proof infra untouched = DEC-009). No drift.
- **`plane` fuzz** (`just fuzz-plane 90`) — **20,465,198 runs in 91s, 0 crashes**
  (reproduces the build's 15M/61s; a real crash writes a `crash-*` artifact and
  exits non-zero — neither occurred).
- **Adversarial mutation (DEC-004 rule 1 / AGENTS.md §15 check 9).** Flipped the
  per-strip layer-0 from byte-ceil to byte-floor (`checked_add(7)` → `checked_add(0)`):
  `a_two_strip_14bit_plane_respects_per_strip_byte_padding` went **red** (the padded
  4-byte sub-byte strip is rejected), while `a_two_strip_16bit_plane_unpacks_like_one_strip`
  stayed **green** (no padding) — proving AC2 is load-bearing and specific to the
  sub-byte case. Reverted; both green again; `git diff` clean.
- **Nikon corpus (runtime, §12 behavioral surface).** Independently confirmed via
  `irr unpack` and the test's own `--nocapture` message: the 12-strip plane
  assembles into the full **4352×2868** buffer (`UnsupportedStripLayout` gone), then
  trips `SampleExceedsWhiteLevel` at sample **741623 = 3835 > WhiteLevel 3808** —
  expected, and SPEC-029's to relax. The `max > WhiteLevel` assertion is UNTOUCHED.

#### Spec acceptance criteria — all 7 met

- **AC1** `a_two_strip_16bit_plane_unpacks_like_one_strip` — asserts samples AND
  `dst == one_strip_dst`.
- **AC2** `a_two_strip_14bit_plane_respects_per_strip_byte_padding` — **non-vacuous**:
  naive byte concatenation would read sample 2 across strip 1's 4 pad bits; the
  per-strip fresh `BitReader` reads strip 2 from its own byte 0. Mutation-proven.
- **AC3** `an_uneven_last_strip_unpacks_the_remainder` — 3 rows / 2-per-strip remainder.
- **AC4** `a_single_strip_plane_is_unchanged` + the real Q2M/M-Monochrom tests.
  Byte-for-byte preserved: the old layer-0 was exact (`StripByteCounts*8 ==
  width*height*bits`), the new is byte-ceil; `layer0_arithmetic_is_enforced` Tier-B
  asserts `packed_bits() == StripByteCounts[0]*8` on the real files, i.e.
  `width*height*bits` is byte-aligned there, so ceil reduces to the exact check →
  identical output and identical `{140,112}` error payload. **The key regression held.**
- **AC5** `a_multi_strip_plane_without_rows_per_strip_is_a_typed_error` →
  `MissingRowsPerStrip { strips: 2 }`, no panic.
- **AC6** `a_truncated_strip_is_truncated_not_a_panic` → `Truncated`;
  `a_strip_byte_count_mismatch_is_packed_size_mismatch` → `PackedSizeMismatch
  { expected_bits: 32, strip_bits: 48 }`. Panic-free.
- **AC7 (corpus)** `the_nikon_d300s_twelve_strip_plane_no_longer_rejects_the_strip_layout`
  — assembled, `UnsupportedStripLayout` gone.

### Cost self-report

- **Tokens (total):** 8,884,772
- **Estimated USD:** 21.10
- **Duration (minutes):** ~40
- **Source of the number:** harness report — summed from this verify session's own
  Claude Code transcript JSONL (`input + output + cache_creation + cache_read` over
  all 52 usage-bearing records, deduped by `message.id`, scratchpad-UUID anchored;
  all records `claude-opus-4-8`). Cache-read-dominated (8.64M of 8.88M is cache
  read). `estimated_usd` at per-category Opus list rates (in 15 / out 75 /
  cache-write 18.75 / cache-read 1.50 per Mtok). Floor-by-id snapshot, not a budget
  delta; same all-inclusive convention as SPEC-024 verify (7.3M).

### Drift and new artifacts

- **New decisions emitted:** none. The per-strip byte-ceil layer-0 is DEC-008's
  "each strip padded to a byte boundary independently" made arithmetic, not a fresh
  decision with live alternatives — I concur with the build's judgement that no new
  DEC is warranted.
- **Deviations from spec — both adjudicated ACCEPTABLE:**
  1. **Per-strip layer-0 is `StripByteCounts[i] == ceil(rows_i*width*bits/8)`, not
     the spec Outputs' literal `StripByteCounts[i]*8 == rows_i*width*bits`.** The
     literal is internally contradictory: for a padded sub-byte strip the LHS is
     always a multiple of 8, so the equality is *unsatisfiable* for exactly the
     strips AC2 requires to work. The byte-ceil form reduces to the literal when
     there is no padding, makes AC2 non-vacuous, keeps N==1 byte-for-byte, and is
     DEC-008 made arithmetic. Resolving the Outputs-vs-AC2 tension in favour of the
     testable criterion — correct. **→ ACCEPTABLE.** Residue: a spec-text tidy, filed
     as SPEC-028/FU-1 (follow-up, non-blocking).
  2. **Tests live in `tests/plane_unpack.rs` (the `MultiStrip` fixture), not the
     `Build` DSL in `src/ifd.rs mod tests` the spec named.** AGENTS.md §12 puts a
     public function's behaviour tests in `tests/<area>.rs`, and every existing
     `unpack_into` test already lives there; the `Build` DSL drives the IFD reader,
     never `unpack_into`. The spec conflated the two builders; following it
     literally would violate §12 and could not exercise `unpack_into`. The build
     extended the correct builder. **→ ACCEPTABLE**, no action.
  (The build's third flagged item — no extra §11 reader line, since `irr unpack`
  already reads the whole plane — is a non-issue, correctly reasoned.)
- **Follow-up work identified:**
  - **SPEC-028/FU-1 (follow-up, non-blocking):** amend the spec `## Outputs`
    per-strip layer-0 bullet from the literal `StripByteCounts[i] * 8 ==
    rows_i * width * bits` to the byte-ceil form the code and provenance ledger
    already use. A spec-text correction for the ship/maintainer step; not a code
    defect and does not hold the spec.
  - SPEC-029 and SPEC-027 are the already-framed consumers/companions, not gaps.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in the code. The two spec-text tensions the build flagged (the
   unsatisfiable literal layer-0 formula, and the wrong named test location) are
   real but the build resolved both correctly; verifying that took most of the
   reasoning, and the mutation plus the Tier-B `layer0_arithmetic_is_enforced`
   assertion settled the byte-for-byte regression analytically.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-008's independent-per-strip-padding rule is the load-bearing one and
   both the spec Notes and the build call it out. The per-strip fresh `BitReader` +
   `split_at_mut_checked` cursor are what make the independence structural rather
   than incidental.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. Running the long `just test` and the fuzz in the
   background while doing the static review and deviation adjudication kept the
   wall-clock down; reading gate output by its own `test result:`/exit lines
   (never a `| tee` tail) avoided the exit-code-masking trap the SPEC-024 verify
   recorded.
