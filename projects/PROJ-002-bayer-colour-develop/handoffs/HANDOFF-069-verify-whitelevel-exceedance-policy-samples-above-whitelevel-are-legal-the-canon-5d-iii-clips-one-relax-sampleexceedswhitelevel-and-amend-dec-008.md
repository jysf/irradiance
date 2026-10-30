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
  id: HANDOFF-069
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-30
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-029

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
  tokens_total: 9134583 # REAL combined count — what cost-audit reads
  estimated_usd: 20.83    # tokens_total × your rate, or your harness's number
  duration_minutes: 35
  branch: verify-spec-029
  pr: null                         # verify-bookkeeping PR — filled after the PR opens
  completed_at: 2026-10-30          # YYYY-MM-DD
  notes: "✅ APPROVED at commit 1112c18 (PR #44, already MERGED to main before this verify ran — a retrospective independent review; no build branch remained to push to, hence a small verify-bookkeeping PR). All 5 ACs reproduced cold: full suite (14 binaries, 0 failed, corpus 7/7 so both Bayer tier-B tests + the mono md5 oracle actually RAN, not skipped), pinned CI clippy -D warnings, cargo fmt --check, just validate, just decisions-audit (0 structural errors). Both corpus Bayer DNGs unpack FULLY — Canon single-strip 5920x3950 (max 15464 > WhiteLevel 15000) and Nikon 12-strip 4352x2868 (max 4095 > WhiteLevel 3808) — confirmed via the two --nocapture corpus tests AND irr unpack. Mono regression BYTE-FOR-BYTE: plane_md5_matches_the_pinned_raw_checksum matched dnglab --raw-checksum on all 4 decodable frames (Q2M 14-bit x3 + M Monochrom 16-bit). plane fuzz 25,936,891 runs/91s, 0 crashes. DEC-029 content correct (amends DEC-008's permanent-invariant clause, two-path rule untouched, byte-swap guard moved to SPEC-025 + layer-0). The DEC-029 NUMBER choice is an ACCEPTED intentional deviation (maintainer released the LUT-boundary reservation in the build brief); one non-blocking follow-up SPEC-029/FU-1 (drop the stale reservation hedge in the spec text + 4 sibling docs — at ship). tokens_total summed from this verify session's own transcript JSONL (scratchpad-UUID anchored e997ac41, deduped by message.id; all 59 usage records claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (8.91M of 9.13M); estimated_usd at per-category Opus list rates. Floor-by-id snapshot, not a budget delta."
  synced_at: 2026-10-30
---

# HANDOFF-069: VERIFY — WhiteLevel-exceedance policy — samples above `WhiteLevel` are legal (the Canon 5D-III clips one); relax `SampleExceedsWhiteLevel` and amend DEC-008

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-029` to
`claude-opus-4-8` (acting as verifier) for the **verify** cycle — review cold
the build that relaxed `plane::unpack_into`'s `max > WhiteLevel` assertion
(samples above `WhiteLevel` are legal) and wrote `DEC-029` amending `DEC-008`,
confirming the bit-exact plane preserves raw values so SPEC-025's oracle survives.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-029-...-amend-dec-008.md`
  — Acceptance Criteria + Failing Tests are the contract.
- **Build HANDOFF:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-068-build-...-amend-dec-008.md`.
- **DEC-029:** `./decisions/DEC-029-samples-above-whitelevel-are-legal-amending-dec-008.md`.
- **DEC-008:** `./decisions/DEC-008-sample-packing-depends-on-byte-alignment.md`
  (the amended decision).

### Decisions that apply

- `DEC-008` — the two-path unpack and its WhiteLevel-assertion clause, amended here.
- `DEC-029` (NEW, build) — samples above `WhiteLevel` are legal; assertion retired.
- `DEC-016` — `unpack_into` allocates nothing (unchanged; scope-overlaps DEC-029, benign).

### Constraints that apply

- `no-panics-on-untrusted-input` (blocking) — `unpack_into` stays panic-free.
- `test-before-implementation` (blocking) — the spec's two Failing Tests.

### Prior related work

- `HANDOFF-068` — the build cycle under review (PR #44, merged at `1112c18`).
- `SPEC-028` (PR #41/#43) — multi-strip assembly; the Nikon plane it assembles
  tripped this assertion post-assembly (3835 > 3808); 029 completes that unblock.
- `SPEC-025` — the bit-exact oracle, now unblocked; inherits the byte-swap guard.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review it cold.
Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Shipping (archive / CHANGELOG / brag / reflection / cost totals) — the
  maintainer's main-session step (Prompt 5), via `prompts/SPEC-029-ship.md`.
- SPEC-027 (manifest + CFA fuzz seed) and SPEC-025 (the oracle) — separate specs.

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`), fill
`## Handback`, set `handoff.status`/`handback.status` → `completed`, return the
verdict, open a verify-bookkeeping PR.

---

## Handback

### Execution notes

- **Branch / PR:** `verify-spec-029` → verify-bookkeeping PR (PR #44 already
  MERGED at `1112c18`, so no build branch remained to push to — this is a
  retrospective independent review).
- **Completed at:** 2026-10-30
- **All acceptance criteria met?** Yes — every AC in SPEC-029, reproduced cold.
- **Verdict:** **✅ APPROVED** at commit `1112c18`.

#### Gates re-run (all green, locally, this session)

- **`just test`** — 14 test binaries, **0 failed** (1 documented `#[ignore]` in
  `develop.rs`); `corpus-status` printed **7/7 present — no tier-B test will
  skip**, so both Bayer corpus tests and the mono md5 oracle genuinely RAN.
  `src/lib.rs` units 95/0; `tests/plane_unpack.rs` 18/0; `tests/plane_oracle.rs`
  12/0.
- **clippy `-D warnings`** (`just lint-ci`, pinned `~/.cargo/bin/cargo +stable`) → exit 0.
- **`cargo fmt --check`** → exit 0. **`just validate`** → exit 0 (33 artifacts
  valid required front-matter; 146 parseable under projects/ + decisions/).
- **`just decisions-audit`** → **0 structural errors**, 11 advisory scope
  warnings (incl. the expected DEC-016 ↔ DEC-029 overlap: DEC-016 is the
  allocation contract, DEC-029 the value policy — confirmed non-contradictory).
  `just decisions-index` regenerates with **no diff** (INDEX.md current, 29
  decisions / 27 active).
- **`plane` fuzz** (`just fuzz-plane 90`) — **25,936,891 runs in 91s, 0 crashes**,
  no `fuzz/artifacts/plane/` entry, `git status` clean. `unpack_into` panic-free
  on CFA input confirmed empirically.

#### Spec acceptance criteria — all 5 met

- **AC1** `a_sample_above_white_level_unpacks_and_is_preserved` — WhiteLevel
  15000, sample **15464** (the Canon clip) returns `Ok`; asserts `dst == samples`
  (raw value present) **and** `dst[2] > WhiteLevel` (not clamped). The test
  checks the raw value is present, not WhiteLevel — exactly the bit-exact
  requirement. ✅
- **AC2** mono within-WhiteLevel **byte-identical** — the synthetic
  `a_plane_within_white_level_is_unchanged` (incl. the `== WhiteLevel` boundary),
  plus the real regression: `tests/plane_oracle.rs::plane_md5_matches_the_pinned_raw_checksum`
  recomputed the full-plane MD5 for all 4 `DECODABLE` frames (Q2M 14-bit ×3 + M
  Monochrom 16-bit) and matched the pinned `dnglab analyze --raw-checksum` (23s,
  not a skip). Byte-for-byte unchanged. ✅
- **AC3 (corpus)** `the_canon_5d_iii_plane_unpacks_with_its_above_white_sample_preserved`
  — ran against the real `CANON-EOS-5D-MARK-III/5G4A9394.DNG`; asserts WhiteLevel
  15000, unpacks `Ok`, plane max **15464 > 15000** preserved (`--nocapture`
  confirmed; `irr unpack` independently: 5920×3950, max 15464). ✅
- **AC4** `DEC-029` exists (`status: accepted`), amends **exactly** DEC-008's
  Consequences clause that called the `max <= WhiteLevel` assertion a "permanent
  invariant", records the reversed consequence, and names **SPEC-025** as the
  replacement byte-swap guard (+ the per-strip layer-0 check). DEC-008 carries a
  matching forward-pointer amendment banner. ✅
- **AC5** DEC-008's two-path byte-alignment rule (`bits % 8 == 0`, MSB-first
  bitstream, byte-order path) is **untouched** — the diff removes only the
  post-assembly `max > WhiteLevel` loop + the error variant + doc comments.
  `unpack_byte_aligned` / the bitstream path / per-strip assembly bodies unchanged. ✅

#### Combined SPEC-028+029 unblock (reproduced)

Both corpus Bayer DNGs now unpack to a full `width×height` plane with no error —
Canon (single strip, 5920×3950, max 15464) and Nikon (12 strips via SPEC-028,
4352×2868, max 4095 > WhiteLevel 3808). Confirmed via the two corpus tests
(`--nocapture`: "CANON ... above-white max 15464 > WhiteLevel 15000", "NIKON ...
assembled and fully unpacked (above-white max 4095 preserved)") AND `irr unpack`.
**SPEC-025's oracle is unblocked.**

#### Code review (cold)

- `src/plane.rs` — the `max > WhiteLevel` loop (`:406-421`) removed and replaced
  with a comment that cites the falsifier; the two unpack paths and per-strip
  assembly are byte-for-byte unchanged; module + fn doc comments updated to point
  at DEC-029. Removing a loop that only ever `return Err`-ed cannot add a panic.
- `src/lib.rs` — `Error::SampleExceedsWhiteLevel` removed from the enum **and**
  its `Display` arm; no `src/` reference remains (`grep` clean — the only two
  `.rs` mentions are historical comments in `tests/plane_unpack.rs`). The
  `PackedSizeMismatch` doc was correctly rewritten (the layer-0 check does NOT
  catch a byte-swap — DEC-008's assertion did, now SPEC-025). Public `Error`
  enum change is intentional and clean (pre-1.0; crate is `0.1.1`, unpublished).
- `src/bin/irr.rs` — a one-expression §11 reader fix: `unpack` now prints the
  real relation (`15464 > 15000`) instead of a hardcoded `<=` that my change
  would have made a false statement. Verified live.

### Cost self-report

- **Tokens (total):** 9,134,583
- **Estimated USD:** 20.83
- **Duration (minutes):** ~35
- **Source of the number:** harness report — summed from this verify session's
  own Claude Code transcript JSONL (`input + output + cache_creation +
  cache_read` over all 59 usage-bearing records, deduped by `message.id`,
  scratchpad-UUID anchored `e997ac41-…`; all records `claude-opus-4-8`).
  Cache-read-dominated (8.91M of 9.13M is cache read). `estimated_usd` at
  per-category Opus list rates (in 15 / out 75 / cache-write 18.75 / cache-read
  1.50 per Mtok). Floor-by-id snapshot, not a budget delta; same all-inclusive
  convention as SPEC-028 verify.

### Drift and new artifacts

- **New decisions emitted:** none by verify. `DEC-029` (the build's) reviewed and
  concurred with — correct amendment of DEC-008, honest about the reduced free
  byte-swap guard.
- **Deviations from spec — adjudicated:**
  1. **`DEC-029` number used despite the spec's "take the next genuinely free
     one" note (029 "informally reserved for the crustyimg LUT boundary").**
     **ACCEPTABLE — accepted intentional deviation.** The maintainer confirmed
     the reservation was released in the build brief (reserving a number for an
     unwritten decision was not worth a gap at DEC-029; highest on disk was
     DEC-028, so 029 is the genuinely next-free number). The build correctly
     followed the newer instruction. Residue: stale spec text + 4 sibling notes
     still carry the reservation hedge → **SPEC-029/FU-1** (non-blocking, at ship).
  2. **Tests live in `tests/plane_unpack.rs`, not the spec's literal `src/plane.rs`
     tests wording.** **ACCEPTABLE** — AGENTS.md §12 puts a public function's
     behaviour tests in `tests/<area>.rs`, where every `unpack_into` test already
     lives; the same §12-vs-spec divergence SPEC-028 (HANDOFF-066/067) resolved
     the same way.
  3. **`Error::SampleExceedsWhiteLevel` removed, not kept behind a strict mode.**
     **ACCEPTABLE** — the spec explicitly delegated this to build; the default had
     to accept above-white, a strict mode would be dead code with no live consumer
     and no red-proof (DEC-029 Option C records the reopen path).
- **Follow-up work identified:**
  - **SPEC-029/FU-1 (non-blocking, do at ship):** drop the "DEC-029 is informally
    reserved for the crustyimg LUT boundary — take the next free number" hedge now
    that the reservation is released and DEC-029 is this decision. Four sibling
    docs still carry it: `SPEC-029-...md` (## Goal + ## Outputs), `STAGE-006-...md`
    (~line 184), `SPEC-024-...md` (done/, ~line 233), `HANDOFF-064`/`HANDOFF-065`.
    Also optional: `docs/provenance-ledger.md`'s SPEC-028 `src/plane.rs` row
    forward-references "SPEC-029's to relax" (now done) — a one-line tidy, history
    otherwise accurate. Code is correct throughout; these are text-only.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Only the DEC-029 number: the spec text said one thing (reserve 029) and the
   build did another (use 029). It read as an undocumented deviation until the
   maintainer confirmed it was an intentional release of the reservation in the
   build brief. The lesson is small — a one-line note in DEC-029 or the build
   handback's "Deviations" ("029 taken per build brief; reservation released")
   would have made the cold review unambiguous; SPEC-029/FU-1 captures the text
   cleanup.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-008 is the load-bearing one; the spec's Notes named the exact clause
   to amend and gave the real falsifier numbers (15464 > 15000; 3835 > 3808),
   which made reproducing both corpus planes against the real bytes concrete.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The decisive checks were (a) proving the two Bayer
   corpus tests actually RAN not skipped (`--nocapture` + corpus-status 7/7), and
   (b) proving the mono regression is byte-for-byte via the md5-vs-dnglab oracle
   rather than only the 8-sample spot-checks — a removed assertion cannot change
   within-white bytes by construction, but the oracle makes "byte-for-byte" a
   number, not an argument.
