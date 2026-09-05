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
  id: HANDOFF-032
  cycle: build                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-sonnet-5        # CORRECTED — read from this session's own system prompt
                                   # (message.model = claude-sonnet-5). The build hint is now
                                   # 0 for 8; the tier_map prediction (opus) did not match again.
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-09-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-014

project:
  id: PROJ-001
  stage: STAGE-002
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
  tokens_total: 44845024           # deduped by message.id, this session's own transcript — see ## Handback
  estimated_usd: 17.13             # per-component at Sonnet 5's published list rate — see ## Handback
  duration_minutes: 35
  branch: feat/spec-014-level-normalization-geometry-orientation
  pr: null                         # not opened, per this handoff's Return Criteria 7
  completed_at: 2026-09-05
  notes: "single main-loop CLI session, not a sub-agent; see ## Handback for the per-component token breakdown"
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-032: Level normalization, ActiveArea to DefaultCrop, and orientation

## Delegation Summary

Build `SPEC-014`. `SPEC-012` produces a correct uncropped plane and `SPEC-013`
asserts it bit-for-bit; this spec turns it into an image — black subtracted,
white normalized, three-stage crop, orientation applied.

## ⚠ Two things that make this spec different from the last three

**1. This spec has NO ORACLE, and cannot have one.** `SPEC-013`'s
`--raw-checksum` attaches to the **uncropped, un-normalised** plane by contract,
so nothing you write here is covered by it. And `DEC-004` already settled that a
comparison oracle never will cover it: `SPIKE-001` measured the plane checksum is
**structurally blind** to a levels error, and the develop oracle misses one up to
**+256 (50 %)**. `SPEC-015` is the analytic oracle; until it lands, **your tests
are the only check that exists**. Write them accordingly.

**2. The corpus cannot see the thing most likely to be wrong.** On every
decodable file `ActiveArea`'s origin is `(0,0)` or absent. The only file with a
non-zero origin — `K3III.DNG`, `top 34, left 26` — is JPEG and undecodable.

So an implementation that **ignores the `ActiveArea` origin entirely** passes
every corpus test in this repo.

That is `SPIKE-001`'s shape — *"the parameter was always 14"* — and `SPIKE-002`
is the precedent for what it costs: a different camera body revealed a
byte-swapped plane that decoded, sized, and layer-0-checked correctly.
**`AC4`'s hand-built fixture with a non-zero ActiveArea origin is the only thing
in this spec that can observe the distinction. It is not optional.**

Independent evidence for which reading is right, so you do not have to guess:
`dnglab` reports `cropArea.p` **sensor-absolute** — on `K3III.DNG`,
`(26,34) + (28,24) = (54,58)`, exactly what it prints — while `exiftool` reports
the file's own `28 24`. Two tools, two conventions, and the arithmetic between
them settles what DNG means.

## What is already measured — in the spec, reproduce rather than re-derive

The full geometry table for all four decodable files, both crops shown to fit,
and the levels edges: **both** real files contain samples **below** `BlackLevel`
(min 2 and 108) and **both** reach `WhiteLevel` **exactly**. So `AC2`'s
out-of-range handling is not a hypothetical — it fires on the first file.

## The decision you must record

What is the normalized output — `u16` rescaled in place, or `f32` in `[0,1]`?
The spec argues both and gives the orchestrator's read (`u16`, consistent with
`DEC-016`'s no-allocation shape, since `f32` is **190 MB** on top of `SPEC-012`'s
measured 182 MB peak and `DEC-002` is still `proposed`). **Offered as input, not
as the answer — write the `DEC` either way.** Constraint: `SPEC-015` will assert
`BlackLevel → 0` and `WhiteLevel → 1`, so your representation must make that
expressible.

## Return Criteria

1. **Eleven gates + `just lint-ci`**, run by you, pasted, summed across all
   targets. **Push and read CI** — the gate must be *observed* green on your SHA.
2. ⚠ **`SPEC-013`'s oracle must keep passing untouched.** It attaches before your
   transform; if it moves, you have changed something you should not have.
3. **Fuzz** — geometry is a new input surface over attacker-controlled crop
   origin, crop size, ActiveArea and orientation. §12 bar 2.
4. Every mutation: file changed **and** compiled **and** *output changed*. That
   third clause has caught three false red-proofs in two specs; the most recent
   was a fault that compiled and returned `Error::Truncated` instead of a wrong
   digest.
5. ⚠ **Stage your work before mutate-and-revert.** `SPEC-010`'s build lost its
   entire change to `git checkout --` and shipped a reconstruction.
6. **Branch and commit before reporting done** (`feat/spec-014-…`). Fill the
   `handback:` with a real `tokens_total` **deduped by `message.id`** — read your
   own transcript at `~/.claude/projects/<slug>/<session-id>.jsonl`, the session
   id is in the scratchpad path in your system prompt. Price **per-component** at
   the rates for the model `message.model` reports, never a flat rate.
   ⚠ **Do not hand-write `cost.sessions`** — fill the handback block only, so
   `handback-sync` runs once cleanly. Hand-writing it has caused four
   duplicate-entry cleanups.
7. **Correct `handoff.to_agent`.** Do not run `handback-sync`; do not open the PR.
8. Findings `SB-N`/`FU-N` from 1 with §15 dispositions; a `spec:` disposition
   must **name an AC that would fail** without it.
9. Answer §15's reflection questions in the handback.

## Handback

### Summary

Built `src/develop.rs`: `output_dimensions`/`develop_into`, taking `&Sensor`
plus the caller-owned raw plane (`plane::unpack_into`'s output) and writing a
caller-owned developed image — the same no-allocation shape `DEC-016` chose,
now extended by `DEC-018` (output representation) and `DEC-019` (crop-origin
convention). All seven failing tests from the spec now pass, plus a
comprehensive unit-test suite in `src/develop.rs` itself (13 tests) covering
the internal geometry resolution, all eight `Orientation` transforms
(hand-derived and independently verified against a worked example), and every
hostile shape `AC6` names.

### Gates (all run by me, this session, `IRRADIANCE_CORPUS_DIR` set)

1. `cargo fmt --check` — clean.
2. `cargo clippy --all-targets --all-features -- -D warnings` — clean.
3. `cargo test --all-features` — **141 tests pass** across `src/lib.rs`'s unit
   tests (65, incl. `develop::tests` x13), `tests/corpus_manifest.rs` (9),
   `tests/develop.rs` (6, new), `tests/ifd_reader.rs` (12),
   `tests/metadata_oracle.rs` (30), `tests/plane_oracle.rs` (12, **`SPEC-013`'s
   oracle re-run untouched — `plane_md5_matches_the_pinned_raw_checksum`
   still green**), `tests/plane_unpack.rs` (7). Zero failures, zero skips
   (corpus present).
4. `just lint-no-allow` — clean.
5. `just deny` / `just deny-fuzz` — `licenses ok` on both graphs.
6. `~/.cargo/bin/cargo +1.90.0 check --all-targets --all-features` (MSRV) — clean.
7. `just lint-red-proof` — `✓ lint policy red-proof` (control clean, injection
   rejected, fires without `-D warnings` too) — unaffected by this build's
   changes, re-run to confirm.
8. `just fuzz-develop 60` (new target, `fuzz/fuzz_targets/develop.rs`) —
   **14,562,321 executions in 61s, zero crashes.** Reaches every geometry
   rejection `AC6` names via 10 hand-built seeds
   (`fuzz/seeds/develop/`) plus libFuzzer's own mutation.
9. `PATH="$HOME/.cargo/bin:$PATH" ~/.cargo/bin/cargo +stable clippy --all-targets --all-features -- -D warnings`
   (`just lint-ci`'s exact invocation) — clean.
10. `just msrv` — clean (same as #6, listed once).
11. `just decisions-audit` — 0 structural errors. `DEC-018`/`DEC-019` sharing
    `affected_scope: src/develop.rs` is flagged as a same-scope warning by
    design (the tool's own message: "confirm they don't contradict") — they
    don't; same pattern as the pre-existing `DEC-012`/`DEC-015` pair.

**CI**: pushed to `feat/spec-014-level-normalization-geometry-orientation`,
observed green at `1404aaca7a354b44b580ca9d84c03343c8449a59` —
https://github.com/jysf/irradiance/actions/runs/33954732964 (`conclusion:
success`, all 9 jobs: fmt, clippy -D warnings, test, licenses, licenses-fuzz,
MSRV, cost-capture audit, lint-policy-red-proof, lint-policy-no-allow).

### Memory (AC7)

`irr develop` on `L1021223.DNG` (release build): peak RSS **275,890,176
bytes**. `SPEC-012`'s already-measured 182,435,840 (file + raw plane) plus
the 93,453,824-byte developed buffer (8368×5584×2) accounts for essentially
all of it — `develop_into`'s own working memory is `O(1)`. **Not in-place**:
a second buffer is unavoidable (the crop is smaller than the source and may
swap dimensions).

### Findings (`SB-N`/`FU-N`)

- `FU-1` — **`tests/corpus/manifest.toml`'s note for `L1000622.DNG`
  mislabelled its `DefaultCropOrigin`/`Size` as `"ActiveArea 2 2 5212
  3468"`.** Directly measured via `irr ifd` while gathering `AC1`/`AC3`
  evidence: `active_area` is actually `None` on this file (matching
  `SPEC-014`'s own design-time probe); the non-zero values are
  `DefaultCropOrigin (2, 2)` / `DefaultCropSize (5212, 3468)`. Would have
  misled a future reader into believing a decodable file with a non-zero
  `ActiveArea` origin exists, when none does (the whole reason `AC4`'s
  hand-built fixture is load-bearing). **Disposition: `fixed`** —
  `tests/corpus/manifest.toml:190-192`, this build.

No ship-blockers found.

### Reflection (§15's ship-cycle questions, answered here per this handoff's
### Return Criterion 9 — `SPEC-014` has not shipped yet)

1. **What would I do differently next time?** Nothing structural — the
   design-time probe (already in `## Implementation Context`) meant build
   really did collapse to near-transcription, as AGENTS.md §12 predicts. The
   one thing I'd front-load earlier next time: verifying a corpus doc's
   claims (the manifest note above) the moment I read past them, rather than
   after using them for something else — I only caught it because `AC1`'s
   test needed the real `black_level`/`white_level` values and I cross-checked
   with `irr ifd` directly instead of trusting the note.
2. **Does any template/constraint/decision need updating?** No — the
   handoff/spec process worked as designed for a spec with no oracle: `DEC-004`
   already told me not to look for one, and the design-time probe already
   handed me the exact geometry table.
3. **Is there a follow-up spec to write now?** No new one — `SPEC-015` (the
   analytic oracle) is already framed and is exactly the follow-up this spec
   was missing.
4. **Where was the worst defect caught?** `none` — clean first try; all seven
   failing tests passed without needing a fix-and-recheck loop, and the fuzz
   target found zero crashes on its first run.
5. **What can a user do now that they couldn't before?** Before: a consumer
   of this library could get an uncropped, un-normalised `u16` plane
   (`SPEC-012`) and nothing else. After: `develop::develop_into` turns that
   plane into the actual displayable image — black subtracted, white
   normalized to full `u16` scale, the real `ActiveArea` → `DefaultCrop` →
   `Orientation` geometry applied — confirmed on both real decodable files
   (`8368×5584` and `5212×3468`) and the one shape no real file can prove
   (`AC4`'s non-zero `ActiveArea` origin fixture).

### Token accounting

Computed from this session's own transcript
(`~/.claude/projects/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-irradiance/2a8063c7-df8a-4c92-9dc4-86383047d490.jsonl`),
deduped by `message.id` (142 unique assistant turns with usage), all on
`claude-sonnet-5` — no sub-agent, so no `subagent_tokens` split. Session span
07:40:32 → 08:16:00 UTC = **35 minutes**.

| Component | Tokens | Rate (Sonnet, published list) | Cost |
|---|---:|---:|---:|
| `input_tokens` | 284 | $3.00 / MTok | $0.00 |
| `output_tokens` | 155,261 | $15.00 / MTok | $2.33 |
| `cache_creation_input_tokens` | 405,107 | $3.75 / MTok | $1.52 |
| `cache_read_input_tokens` | 44,284,372 | $0.30 / MTok | $13.29 |
| **Total** | **44,845,024** | — | **≈ $17.13** |

Priced per-component, not a flat rate (this handoff's Return Criterion 6) —
`cache_read_input_tokens` dominates both the token count and, at its much
lower per-token rate, a smaller-than-naive share of the cost. The per-token
rates are the standard published Sonnet-tier list prices; not independently
re-verified against a Sonnet-5-specific published rate card, so treat
`estimated_usd` as the order-of-magnitude estimate AGENTS.md §4 asks for, not
an invoiced number.
