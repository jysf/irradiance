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
  id: HANDOFF-071
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # CORRECTED to what actually ran, not the tier_map hint
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-10-30
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-025

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
  tokens_total: 8224450 # REAL combined count — what cost-audit reads
  estimated_usd: 17.73    # tokens_total × your rate, or your harness's number
  duration_minutes: 25
  branch: claude/spec-025-verify-88f9ad
  pr: https://github.com/jysf/irradiance/pull/49   # verify-bookkeeping PR
  completed_at: 2026-10-30          # YYYY-MM-DD
  notes: "✅ APPROVED at commit 53f3803 (PR #48, already MERGED to main before this verify ran — a retrospective cold independent review; no build branch remained to push to, hence a small verify-bookkeeping PR). PR #48 confirmed TEST-ONLY + bookkeeping: tests/plane_oracle.rs + the SPEC-025 spec + HANDOFF-070 — no src/ change, no tests/corpus/manifest.toml change. All 5 ACs reproduced cold with the corpus present (7/7). Independent ground truth: dnglab 0.7.2 analyze --raw-checksum recomputed on both Bayer DNGs == the pinned constants EXACTLY (Canon a18235d6abced4df9948b6fbe91aba7c, Nikon b8e8025d370cc2e97f675e63a0e6bbdb) — the oracle compares irradiance's unpacked-plane MD5 to dnglab's number, not to itself (no tautology). Both Bayer planes decode BIT-EXACT (Canon single-strip 5920x3950, Nikon 12-strip 4352x2868), confirmed live via --nocapture (BAYER ORACLE lines, not SKIP). Byte-aligned red-proof reddens: honest b8e8025d == pinned Nikon, mutant fdcb897fbae2bd9d180e05f9217f5011 != honest, control green — the exact byte-swap guard DEC-029 moved onto this oracle, and (Nikon reference) it also exercised SPEC-028 multi-strip assembly inside the staged mutant. Mono regression GREEN: plane_oracle 15/15 (was 12; +3 Bayer), the sub-byte BitReader red-proof still reddens live (cb653b5b -> 59b032fe). No-corpus path verified: Bayer tests print loud SKIP and pass without asserting (0.00s) while tier-A RFC-1321 still runs. Gates: just test (full suite, 0 failed across all binaries; src/lib.rs 95/0, plane_unpack 18/0, develop 28/0 1 documented ignore), just lint-ci clippy -D warnings pinned +stable 0.1.98 exit 0, cargo fmt --check exit 0, just validate exit 0 (33 valid / 148 parseable), just ready clean, just decisions-audit 0 structural errors (11 pre-existing advisory), plane fuzz 13,407,764 runs/61s 0 crashes 0 artifacts tree clean. No new DEC; verify emitted none. tokens_total summed from this verify session's own transcript JSONL (scratchpad-UUID anchored 1390d94e, deduped by message.id; all 51 usage records claude-opus-4-8; input+output+cache_creation+cache_read), cache-read-dominated (8.03M of 8.22M, 97.6%); estimated_usd at per-category Opus list rates. Floor-by-id snapshot, not a budget delta."
  synced_at: 2026-10-30
---

# HANDOFF-071: VERIFY — Bit-exact mosaic plane oracle: the Bayer file's unpacked plane MD5 equals `dnglab analyze --raw-checksum`, with its red-proof

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-025` to
`claude-opus-4-8` (acting as verifier) for the **verify** cycle — review cold
the build that proved both corpus Bayer DNGs' unpacked mosaic planes are
bit-exact (plane MD5 == `dnglab analyze --raw-checksum`) and added the mandatory
byte-aligned red-proof (the byte-swap guard DEC-029 moved onto the oracle),
STAGE-006's capstone.

## Context the Receiving Agent Needs

### Primary

- **Spec:** `./projects/PROJ-002-bayer-colour-develop/specs/SPEC-025-...-with-its-red-proof.md`
  — Acceptance Criteria (5) + Failing Tests are the contract.
- **Build HANDOFF:** `./projects/PROJ-002-bayer-colour-develop/handoffs/HANDOFF-070-build-...-with-its-red-proof.md`.
- **The test:** `tests/plane_oracle.rs` — the SPEC-025 section (the three new
  Bayer tests + `inject_byte_order_swap_fault`), reusing SPEC-013's apparatus.
- **The injection target:** `src/plane.rs` `unpack_byte_aligned` 16-bit branch
  (`from_le_bytes`/`from_be_bytes`, `:211–212`).

### Decisions that apply

- `DEC-008` — the two-path unpack; the byte-order path the red-proof swaps.
- `DEC-029` — samples above `WhiteLevel` are legal; moved the byte-swap guard
  onto THIS oracle (its Validation clause is SPEC-025's success condition).
- `DEC-017` — the red-proof mechanism (copy-and-rebuild a mutated crate).
- `DEC-003` — tier-B corpus is located by `$IRRADIANCE_CORPUS_DIR`, skip-when-absent.

### Constraints that apply

- `oracle-must-be-shown-red` (blocking) — the mandatory red-proof.
- `no-panics-on-untrusted-input` (blocking) — `unpack_into` stays panic-free (fuzz).
- `test-before-implementation` (blocking) — the spec's Failing Tests.

### Prior related work

- `HANDOFF-070` — the build cycle under review (PR #48, merged at `53f3803`).
- `SPEC-028` (PR #41/#43) — multi-strip assembly; the Nikon reference exercises it.
- `SPEC-029` (PR #44/#46) — WhiteLevel policy + DEC-029; the guard this oracle inherits.
- `SPEC-013` (archived) — the mono oracle + red-proof apparatus this reuses.

## Expected Deliverables — the verify contract

You are **not** the agent that implemented this. Review it cold.
Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Shipping (archive / CHANGELOG / brag / reflection / cost totals / verdict
  stamp) — the maintainer's main-session step (Prompt 5).
- SPEC-027 (migrate the Bayer DNGs into the manifest + CFA fuzz seed) and
  SPEC-026 (colour-metadata oracle) — separate specs.

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`), fill
`## Handback`, set `handoff.status`/`handback.status` → `completed`, return the
verdict, open a verify-bookkeeping PR. Leave `task.verify_verdict` as the
machinery sets it — the maintainer's ship (verify → ship) stamps `approved`.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-025-verify-88f9ad` → verify-bookkeeping PR #49
  (PR #48 already MERGED at `53f3803`, so no build branch remained to push to —
  this is a retrospective independent review).
- **Completed at:** 2026-10-30
- **All acceptance criteria met?** Yes — every one of SPEC-025's 5 ACs reproduced cold.
- **Verdict:** **✅ APPROVED** at commit `53f3803`.

#### PR #48 scope confirmed (AC5)

`git show 53f3803 --name-status` touches exactly three files:
`tests/plane_oracle.rs` (the oracle), the SPEC-025 spec (cycle bookkeeping), and
`HANDOFF-070-build-...` (the build handback). **No `src/` change, no
`tests/corpus/manifest.toml` change** — the oracle is test-only, as the spec
requires. The working tree's `src/plane.rs` is byte-for-byte the SPEC-029 state;
the red-proof only mutates a *staged copy*.

#### Independent ground truth (the anti-tautology check)

`dnglab 0.7.2 analyze --raw-checksum` recomputed on both corpus Bayer DNGs,
this session, matches the pinned constants **exactly**:

| file | my `dnglab --raw-checksum` | pinned constant | match |
|---|---|---|---|
| `CANON-EOS-5D-MARK-III/5G4A9394.DNG` | `a18235d6abced4df9948b6fbe91aba7c` | `CANON_RAW_CHECKSUM` | ✅ |
| `NIKON-D300S/_DSC4615.DNG` | `b8e8025d370cc2e97f675e63a0e6bbdb` | `NIKON_RAW_CHECKSUM` | ✅ |

The oracle asserts `irradiance_plane_md5 == pinned`; I independently verified
`dnglab_checksum == pinned`; dnglab is a separate tool (run, never linked), so
transitively `irradiance == dnglab` — not a self-comparison.

#### Gates re-run (all green, locally, this session, corpus present)

- **`just test`** — `corpus-status` printed **7/7 present — no tier-B test will
  skip**, so the Bayer direct-path tests and the mono md5 oracle genuinely RAN.
  All binaries **0 failed**: `src/lib.rs` units 95/0; `tests/plane_oracle.rs`
  **15/0** (was 12 pre-SPEC-025 — the +3 Bayer tests); `tests/plane_unpack.rs`
  18/0; `tests/develop.rs` 28/0 (1 documented `#[ignore]`); metadata_oracle 30/0;
  the rest 0/0-failed.
- **clippy `-D warnings`** (`just lint-ci`, pinned `~/.cargo/bin/cargo +stable`,
  clippy 0.1.98) → exit 0.
- **`cargo fmt --check`** → exit 0. **`just validate`** → exit 0 (33 artifacts
  valid required front-matter; 148 parseable under projects/ + decisions/).
  **`just ready`** → clean.
- **`just decisions-audit`** → **0 structural errors**, 11 pre-existing advisory
  scope warnings (unchanged — SPEC-025 added no DEC and no src change).
  `--changed` → nothing in scope (tree clean, change already merged).
- **`plane` fuzz** (`just fuzz-plane 60`) — **13,407,764 runs in 61s, 0 crashes**,
  no `fuzz/artifacts/plane/` entry, `git status` clean. `unpack_into` panic-free
  on CFA/arbitrary input confirmed empirically.

#### Spec acceptance criteria — all 5 met (reproduced live via `--nocapture`)

- **AC1 — both Bayer planes bit-exact.** `bayer_plane_md5_matches_dnglab_raw_checksum`
  ran (not skipped) and printed `BAYER ORACLE CANON… plane MD5 a18235d6… ==
  dnglab --raw-checksum` and the same for Nikon `b8e8025d…`. First bit-exact proof
  the Bayer mosaic decodes. ✅
- **AC2 — byte-order swap turns it red, control green.**
  `a_byte_order_swap_turns_the_bayer_oracle_red`: `inject_byte_order_swap_fault`
  swaps `from_le_bytes`↔`from_be_bytes` in a staged copy's `unpack_byte_aligned`;
  honest `b8e8025d…` (== pinned Nikon) vs mutant **`fdcb897fbae2bd9d180e05f9217f5011`**
  (≠ honest). `the_bayer_red_proof_control_is_green` — the unmutated staged copy
  reproduces `b8e8025d…`. Nikon reference, so the swap also exercised SPEC-028
  multi-strip assembly inside the mutant. This is the exact byte-swap DEC-029's
  Validation clause names as the proof the guard "genuinely moved rather than
  vanished." ✅
- **AC3 — mono oracle + sub-byte red-proof stay green.** `plane_oracle` 15/15;
  the sub-byte `BitReader` red-proof `an_injected_unpacker_fault_turns_the_oracle_red`
  reproduced live (honest `cb653b5b…` → mutant `59b032fe…`), and
  `plane_md5_matches_the_pinned_raw_checksum` (4 `DECODABLE` mono frames) passed
  with the corpus present. SPEC-013's behaviour untouched. ✅
- **AC4 — no corpus → skip loud, tier-A still runs.** With
  `$IRRADIANCE_CORPUS_DIR` unset, the three Bayer tests printed
  `SKIP …: set $IRRADIANCE_CORPUS_DIR …` and passed **without asserting**
  (finished in 0.00s — no decode), while the tier-A `md5_matches_the_rfc_1321_test_vectors`
  ran and passed. Skip is an early `return`, not a silent green over a real claim. ✅
- **AC5 — test-only.** PR #48 diff is `tests/plane_oracle.rs` + spec + HANDOFF
  only; no `src/`, no manifest (see "PR #48 scope confirmed"). ✅

#### Code review (cold)

- `tests/plane_oracle.rs` — `inject_byte_order_swap_fault` asserts each needle
  (`u16::from_le_bytes(*chunk)` / `u16::from_be_bytes(*chunk)`) occurs **exactly
  once** before swapping, and swaps through a `RED_PROOF_BYTE_ORDER_SWAP`
  placeholder so the two `replacen`s cannot collide; it asserts the rewrite
  actually changed the source and left no placeholder. I confirmed both needles
  occur exactly once and **only** in `src/plane.rs` (the `from_le/be` uses in
  `ifd.rs`/`opcode.rs` have different argument text, so no cross-file collision —
  and the injection writes only the staged `src/plane.rs` regardless). The fault
  changes values, never buffer length (same discipline as the mono fault), so it
  cannot hit a `Truncated` boundary — it must change the digest or be a no-op, and
  the test asserts `mutant != honest` directly.
- `stage_probe_crate` generalised from `mutate: bool` to `inject: Option<fn(&Path)>`
  — the mono (sub-byte) and Bayer (byte-aligned) faults share it; the mono
  injector and its red-proof are unchanged (AC3 green proves it).
- `require_bayer` resolves `CorpusRoot` + relative path, requires `from_env`
  **and** `path.exists()`, else prints SKIP and returns `None` — the direct-path,
  skip-when-absent pattern the spec prescribes (the Bayer DNGs stay out of the
  manifest and out of `DECODABLE`/`SKIPPED_COMPRESSED` until SPEC-027, so
  `compressed_files_are_skipped_by_name`'s count is untouched — `plane_oracle`
  15/15 includes that test).

### Cost self-report

- **Tokens (total):** 8,224,450
- **Estimated USD:** 17.73
- **Duration (minutes):** ~25
- **Source of the number:** harness report — summed from this verify session's
  own Claude Code transcript JSONL (`input + output + cache_creation +
  cache_read` over all 51 usage-bearing records, deduped by `message.id`,
  scratchpad-UUID anchored `1390d94e-…`; all records `claude-opus-4-8`).
  Cache-read-dominated (8.03M of 8.22M, 97.6%). `estimated_usd` at per-category
  Opus list rates (in 15 / out 75 / cache-write 18.75 / cache-read 1.50 per Mtok).
  Floor-by-id snapshot taken at hand-back, not a budget delta.

### Drift and new artifacts

- **New decisions emitted:** none by verify. DEC-029's Validation clause (the
  oracle must match `--raw-checksum` on a correct plane AND be shown red by a
  per-sample byte-swap) is now satisfied by a landed, reproduced test — the guard
  demonstrably moved onto the oracle rather than vanishing.
- **Deviations from spec — adjudicated:**
  1. **`to_agent` tier_map hint said `claude-opus-5`; the verify actually ran on
     `claude-opus-4-8`.** Corrected in the front-matter (same correction
     HANDOFF-069 made). Bookkeeping only.
- **Follow-up work identified:**
  - **Non-blocking observation (for SPEC-027, already planned):** the Bayer
    skip's only loud surface is the `eprintln!` in `require_bayer` (captured by
    `cargo test` unless `--nocapture`); unlike the mono tier-B files, the Bayer
    DNGs are not in `examples/corpus-status.rs`'s 7/7 line because they are not in
    the manifest yet. This matches the spec exactly ("skip loudly, like
    `plane_unpack.rs`") and SPEC-027 migrates them into the manifest, at which
    point corpus-status names them. No action for SPEC-025.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The spec named the exact injection lines
   (`src/plane.rs:210–212`), the reference file (Nikon, for multi-strip), the two
   pinned checksums, and the five ACs; the task brief gave the corpus location and
   the dnglab command. The one thing worth proving beyond "tests pass" was that the
   direct-path Bayer tests actually RAN and did not silently skip — handled with
   `--nocapture` (live `BAYER ORACLE`/`BAYER RED-PROOF` lines) plus the env-unset
   run that shows the `SKIP` branch.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-029 is the load-bearing one — its Validation clause IS SPEC-025's
   success condition — and both the spec Notes and the test module doc cite it. The
   anti-tautology check (recompute dnglab independently) was the decisive step the
   spec implied and I made explicit.

3. **If you did this task again, what would you do differently?**
   — Nothing structural. The decisive checks were (a) recomputing dnglab's two
   checksums myself so "bit-exact" is independent of the pinned constants, and
   (b) proving the byte-swap mutant BUILT, RAN, and produced a *different* digest
   (not a compile error or a no-op) — which is what makes "the guard moved to the
   oracle" a reproduced fact rather than a claim.
