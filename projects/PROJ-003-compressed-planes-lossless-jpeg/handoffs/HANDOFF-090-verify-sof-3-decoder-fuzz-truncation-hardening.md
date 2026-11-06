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
  id: HANDOFF-090
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-038

project:
  id: PROJ-003
  stage: STAGE-009
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 185694             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 50
  branch: claude/spec-038-verify-862e91
  pr: null                         # verify PR opened from this branch; number recorded post-open
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "Verdict APPROVED at commit d6b73ad. No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-085/086/087/089). 185694 is this verify session's context-window token occupancy at handback, read via the app usage tool (ccd_session_mgmt get_usage: 185,694 of 1,000,000, 19%). It is a snapshot / lower bound: true cumulative input+output across the verify's turns is higher (per-turn context resends not captured, and some bookkeeping writes follow this read). Not invented, but not a /cost number — treat as a FLOOR in rollups; the orchestrator supersedes it from the transcript at ship. Gate confirmation rests on PR #106's green CI on this exact commit (d6b73ad == HEAD: rust/test, clippy -D warnings, clippy — fuzz graph, fmt --check, license policy, license policy — fuzz graph all pass) plus a local `just validate` (46 artifacts) and the local test binaries that finished ok; my own local full-suite run was still in flight and was stopped as redundant. Nightly cargo-fuzz spot-run: 3,098,220 runs in 31 s, no crash, on top of the build's 2.55M."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-090: SOF-3 decoder fuzz + truncation hardening (verify)

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-038` to
`claude-opus-5` (acting as verifier) for the **`verify`** cycle — a cold review
of the libFuzzer target over the single-component SOF-3 decoder (`PR #106`,
`HANDOFF-089`), the last spec of `STAGE-009`.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-038-sof-3-decoder-fuzz-truncation-hardening.md`
- **Build handoff:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-089-build-sof-3-decoder-fuzz-truncation-hardening.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain`/PATH
  fuzz traps and the gate quirks.

### Decisions that apply

- `DEC-036` — the exposure decision: a `fuzzing` cargo **feature** (NOT
  `cfg(fuzzing)`), because the `clippy — fuzz graph` gate builds `fuzz/Cargo.toml`
  with plain `cargo clippy`. The crux of this verify.
- `DEC-035` — the SOF-3 decoder is clean-room / patent-clean; fuzzing it adds no
  new algorithm.

### Constraints that apply

- `no-panics-on-untrusted-input` — the point of the spec: every hostile /
  truncated / oversized-table bitstream returns a typed `Error`, never panics.
- `no-new-top-level-deps-without-decision` — the `fuzzing` feature is empty (no
  dep); `libfuzzer-sys` is the existing fuzz dev-dependency.

### Prior related work

- `HANDOFF-089` (build) — the build cycle this verify reviews.
- `PR #106` — the build PR (branch `claude/spec-038-build-a93ad6`, head `d6b73ad`).
- `SPEC-036` / `SPEC-037` (shipped) — the decoder being fuzzed (`src/ljpeg.rs`).

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered honestly?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Load-bearing: the panic-free replay tests are not tautological; DEC-036 keeps
  `ljpeg` `pub(crate)` while the fuzz-graph gate stays green.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- The decoder's logic — SPEC-036/037 (shipped); only **exercise** it.
- Bit-exactness / correctness (SPEC-037's oracle; STAGE-010's `dnglab` checksum).
- The DNG container path / multi-component — STAGE-010 and later.

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/spec-038-verify-862e91`; verify PR
  opened against `main` (number recorded post-open). The verify branch is cut
  from the build branch, so it carries **both** the build commits (`1c3e5ed`,
  `d6b73ad`) and the verify bookkeeping (this HANDOFF-090, the timeline `[x]`,
  the ship prompt).
- **Completed at:** 2026-11-05
- **All acceptance criteria met?** yes — all six of SPEC-038's `## Acceptance
  Criteria` are satisfied; confirmed below.
- **Verdict: ✅ APPROVED — merge at commit `d6b73ad9aa92f2bfdd13ba043b80c7ff04888cdc`.**

**Acceptance-criteria confirmation (cold review of `d6b73ad == HEAD`):**

1. A `ljpeg` libFuzzer target (`fuzz/fuzz_targets/ljpeg.rs` + its `[[bin]]`)
   drives the full decoder via the `fuzzing`-feature `__fuzz_ljpeg` entry. My own
   nightly spot-run — `cargo +nightly fuzz run ljpeg fuzz/corpus/ljpeg
   fuzz/seeds/ljpeg -- -max_total_time=30` — did **3,098,220 runs in 31 s, no
   crash, no artifact**, on top of the build's reported 2.55M.
2. `clippy — fuzz graph (-D warnings)` and `license policy — fuzz graph` stay
   green — confirmed on PR #106's CI (both pass on `d6b73ad`). The target
   compiles under **plain** `cargo clippy --manifest-path fuzz/Cargo.toml` because
   the exposure is the `fuzzing` **feature**, not `cfg(fuzzing)` (DEC-036).
3. The two in-crate panic-free replay unit tests
   (`fuzz_seed_corpus_decodes_without_panic`,
   `hostile_bitstreams_are_typed_errors_not_panics`) drive `decode_into` directly
   and assert each input returns `Ok` or a **specific** typed `Error` — never a
   panic. They are not tautological (see Drift notes). Green in `rust / test` on
   #106 and in the local `ljpeg` test binary (14 tests, 0 failed).
4. `cargo run --example fuzz-seeds` writes 14 own-work seeds (9 valid across
   predictors 1..7 / P∈{8,12,16} / Pt=2 / restart, + 5 hostile: truncated-header,
   truncated-scan, oversized-dht, nf-not-one, bad-marker) from a local LJ92
   builder — no `include_bytes!`, copy, or fetch (DEC-003).
5. `ljpeg` stays `pub(crate)`: `mod ljpeg;` is private, and `__fuzz_ljpeg` is
   `#[cfg(feature = "fuzzing")] #[doc(hidden)] pub` — a default build / `cargo
   doc` exposes neither.
6. `#![forbid(unsafe_code)]` + the five-lint deny policy hold on the entry
   (compiled under `--all-features`); `__fuzz_ljpeg` bounds `lines ×
   samples_per_line` with `checked_mul` + `MAX_PIXELS` + `try_from` before
   allocating and drops the typed `Result`. All six gates green; no new runtime
   dep (`[dependencies]` still empty; `fuzzing = []`).

### Cost self-report

- **Tokens (total):** 185694 — a **FLOOR**, not a `/cost` cumulative.
- **Estimated USD:** null (no rate applied to a floor).
- **Duration (minutes):** ~50.
- **Source of the number:** none available (no interactive `/cost` in the Claude
  Code desktop app's Code tab). 185,694 is this verify session's context-window
  occupancy at handback (`ccd_session_mgmt get_usage`: 185,694 / 1,000,000, 19%).
  A lower bound — per-turn context resends are not captured, and some bookkeeping
  writes follow this read — reported as a floor exactly as HANDOFF-085/086/087/089
  did; the orchestrator supersedes it from the transcript at ship.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-036 was minted at design and is validated
  by this verify: the fuzz-graph clippy gate compiles the target via the feature,
  and a default build shows no `ljpeg` symbol. `just decisions-audit --changed`
  reports no drift (empty in this already-committed verify worktree; DEC-036's
  `affected_scope` matches the five changed code files exactly).
- **Why the replay tests are not tautological (the load-bearing check):**
  `hostile_bitstreams_are_typed_errors_not_panics` crafts six inputs and asserts
  each maps to a **distinct** typed variant that actually exists in `src/lib.rs`
  — `LjpegMissingMarker{marker: MARKER_SOI}` (empty), `Truncated` (valid header +
  one entropy byte), `LjpegUnsupportedComponentCount{nf: 3}` (Nf=3),
  `LjpegInvalidHuffmanTable` (three 1-bit codes over-subscribe), `LjpegPlaneWrongLength`
  (forged 65535×65535 against a 4-u16 buffer), `LjpegUnexpectedMarker` (RST0→RST1
  rewrite). The corpus test asserts `count > 0`, so a missing seed-gen fails
  loudly rather than passing vacuously. Scope is clean: `src/ljpeg.rs` changed
  only inside `mod tests`; `src/lib.rs` added only the one feature-gated entry —
  decoder logic and public API untouched.
- **Deviations from spec:** none.
- **Follow-up work identified:** none for STAGE-009 — **this is its last spec**,
  so STAGE-009 stage-ships next (Prompt 1d). STAGE-010 (DNG container binding +
  `dnglab` bit-exact) is the next wave; when it adds a real public `Compression 7`
  entry, the `fuzzing` feature + `__fuzz_ljpeg` hook may be retired (DEC-036
  Validation).

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing. The spec's `## Failing Tests` named the exact replay-test shapes and
   the build's note pointed at the load-bearing checks; the toolchain brief's
   fuzz `+toolchain`/PATH traps let the nightly spot-run work first try (after the
   documented `mkdir -p fuzz/corpus/ljpeg`). The only friction was host-local: a
   slow tier-B oracle test made a local full-suite run long, so I leaned on
   #106's green CI (same commit) plus a targeted local `just validate`.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-036 named the precise mechanism (feature, not `cfg(fuzzing)`) and the
   reason (the plain-`cargo-clippy` fuzz-graph gate), and the two constraints
   (`no-panics-on-untrusted-input`, `no-new-top-level-deps-without-decision`) were
   exactly the ones in play.

3. **If you did this task again, what would you do differently?**
   — For a verify whose only gate not mapped 1:1 to a CI check is `just validate`,
   confirm the exact build commit equals the PR head first (it did: `d6b73ad`),
   then trust that PR's green CI for the cargo-heavy gates and run only the
   non-CI-covered gate locally — rather than kicking off a redundant cold full
   `just test` that races the tier-B oracle. It saved no signal and cost wall time.
