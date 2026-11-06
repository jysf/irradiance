---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. `handback:` is the RETURN path and is not
# optional: `just handback-sync SPEC-NNN` reads it and appends the cost session.
# Contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-097
  cycle: build                     # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8      # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5          # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
  created_at: 2026-11-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-045

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `tokens_total` is the one field the cost gate reads — report the REAL number
# from your interface (Claude Code: `/cost`). If your platform exposes NO token
# count, set tokens_total: null AND say why in `notes`. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 226796             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 75
  branch: claude/spec-045-build-5277f4
  pr: 129                          # build PR against main
  completed_at: 2026-11-06         # YYYY-MM-DD
  notes: "No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-089..096). 226796 is this build session's context-window token occupancy at handback, read via ccd_session_mgmt get_usage (226,796 of 1,000,000, 23%), on claude-opus-5-5. A snapshot / lower bound: true cumulative input+output across turns is higher. Not invented, but not a /cost number — treat as a FLOOR; the orchestrator supersedes it from the transcript at ship. Tier-B determinism pin hit both pinned digests (Typ246 c72e7d95…ac1d, K3III 98bae133…3189); fuzz smoke plane 7.46M runs/61s + develop 4.32M runs/31s, zero crashes; later-DHT test red-proven against first-wins."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-097: Binding capstone — provenance, determinism, hardening (build)

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-045` to the implementer for the **`build`**
cycle: the STAGE-011 capstone — a DNG Compression-7 **binding** provenance-ledger row, a
plane-level macOS/Linux determinism pin, the lossless-JPEG header-hardening the external
0.2.0 review flagged (`Pt < P`, `Tc == 0 && Th <= 3`, later-DHT-replaces), and continuous
fuzzing of the **public** Compression-7 route — all closing STAGE-011 and PROJ-003. The
design §12 probe already established that determinism (develop level, SPEC-044), the
two-component restart reset (SPEC-046), and tiled rejection are groundwork-covered, so the
new code is small and localized.

## Context the Receiving Agent Needs

### Primary
- **Spec:** `projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-045-binding-capstone-a-provenance-ledger-entry-for-the.md`
  — read all of it. `## Inputs` carries the §12 probe (exact gaps + line refs, what is
  already covered); `## Outputs` / `## Acceptance Criteria` / `## Failing Tests` / `## Notes`
  are the contract.
- **Project brief** + the stage `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
- **The review triage:** `docs/reviews/2026-11-06-staff-review-0.2.0-triage.md` — the source
  of the §2.1 / §2.4 items folded here.
- **Toolchain brief:** `guidance/toolchain-brief.md` (`just lint-ci` is the authoritative
  local clippy; the tier-B lesson below).

### The four gaps to close (from the §12 probe — transcribe, don't re-derive)
- `src/ljpeg.rs` `parse_scan_header` validates the predictor `ss` but NOT `al` (Pt); add
  `Pt < P` (pass `precision` in, or check in `parse_headers` once both are known).
- `src/ljpeg.rs` `parse_dht` builds only `tc == 0` tables and ignores other classes / any
  `th`; reject `Tc != 0` and `Th > 3` with a typed `Error`.
- `src/ljpeg.rs` `find_table` returns the FIRST `(th, _)` match; make a later same-`Th` DHT
  win (return the last match, or overwrite in `parse_dht`).
- `fuzz/fuzz_targets/{plane,develop,demosaic}.rs` skip Compression-7 via
  `require_uncompressed().is_err()`; drop that early-return and add a synthetic Compression-7
  SOF-3 seed to `fuzz/seeds/plane/` so the PUBLIC route is fuzzed.

Already covered — do NOT rebuild, just pin/cite: determinism at develop level
(`tests/develop_oracle.rs`, SPEC-044); the two-component restart reset
(`reconstruct_two_component`, SPEC-046, the `ri = x` case in
`two_component_interleaved_decodes_bit_exact`); tiled Compression-7 as a clean
`UnsupportedStripLayout` (`src/plane.rs`).

### Decisions that apply
- `DEC-036` — the dev-only `fuzzing` feature + `__fuzz_ljpeg` hook. **Validation decision for
  this spec: KEEP the hook and ADD public-path coverage** (the direct hook fuzzes the decoder
  deeper than the container-gated route; it is doc-hidden, feature-gated, zero public surface).
  Record this reasoning in the handback; it dispositions DEC-036's open Validation clause — no
  new DEC.
- `DEC-035` — clean-room T.81 (the header-hardening is Annex B/C conformance, no new algorithm).
- `DEC-003` — corpus never committed; a synthetic own-work Compression-7 seed IS fine to commit.
- `DEC-018` — u16 modular `wrapping_*` arithmetic.

### Constraints
- `no-panics-on-untrusted-input` — every new guard returns a typed `Error`, never a panic.
- `no-new-top-level-deps-without-decision` — none needed.

### Prior related work / building blocks
- `tests/support/tiff.rs` — the DNG builder (`tiff`, `Ifd`, `Entry`, `long`, `cfa_dng_16bit`).
  Add `compression7_dng(order, …)` wrapping a SOF-3 strip from the `cfg(test)`-reexported
  `encode_lj92`; use it for the hardening tests and the fuzz seed.
- `tests/plane_unpack.rs` — the AC7 hostile-input home + `compression7_planes_route_to_the_decoder`.
- `docs/provenance-ledger.md` — the `src/plane.rs` row (stale "never decoded" note) and the two
  `src/ljpeg.rs` decode rows this binding row complements.
- `SPEC-043` / `SPEC-044` / `SPEC-046` (shipped) — the routing, the oracle, the two-component decode.

### The build contract
- Implement SPEC-045's `## Outputs`; make every `## Failing Test` pass.
- Header guards are parse-level, typed, no algorithm change; prefer reusing an existing
  `Ljpeg*` variant (add at most one additive variant). `ljpeg` stays `pub(crate)`.
- Keep the `__fuzz_ljpeg` hook + `fuzzing` feature + `fuzz/fuzz_targets/ljpeg.rs` (DEC-036
  Validation); drop only the `require_uncompressed` early-return and add the Compression-7 seed.
- Open a PR against `main` from `claude/spec-045-build-<suffix>` referencing PROJ-003,
  STAGE-011, SPEC-045, HANDOFF-097, DEC-036, DEC-035.

### Running the gates (the tier-B lesson)
- The plane-level determinism test is tier-B, corpus-gated. Set
  `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and run the
  TARGETED suites — `cargo test --lib ljpeg::`, `cargo test --test plane_unpack`,
  `cargo test --test plane_oracle` / `cargo test --test develop_oracle` — rather than a blind
  full `just test` (its unrelated tier-B oracles buffer for minutes). Confirm `just fmt-check`,
  `just lint-ci`, `just validate` green locally; lean on the PR's CI for the broad sweep.
- For the fuzz change, a short `cargo +nightly fuzz run plane -- -runs=…` on the new
  Compression-7 seed is enough to show the public route is panic-free; the full campaigns run
  on their own cadence.

## Out of Scope
- `#[non_exhaustive]` (PROJ-002 0.2.0 / SPEC-042), the opcode `MAX_TAG_VALUES` bound (PROJ-001),
  `LinearizationTable` rejection (cross-cutting, no held file), `tag_name` in `Error` Display
  (cosmetic) — all explicit SPEC-045 Non-Goals.
- Retiring the `__fuzz_ljpeg` hook (kept — DEC-036 Validation).
- The reconstruction algorithm, routing, bit-exact oracle (SPEC-046/043/044, shipped);
  Bayer/CFA colour multi-component (DEC-037); the review's §3B colour geometry and §5 perf.

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`, or an explicit FLOOR with
a note if your interface has none, as HANDOFF-089..096 did), fill `## Handback

*Filled in by the receiving agent.*

### Execution notes
- **Branch / PR:** `claude/spec-045-build-5277f4` / PR #129
- **Completed at:** 2026-11-06
- **All acceptance criteria met?** Yes:
  - Provenance: new DNG Compression-7 **binding** row; the `src/plane.rs` row's "never decoded" note corrected (superseded by SPEC-043; only `K3III.PEF` is still `UnsupportedCompression`). Also removed a pre-existing blank line that split the ledger table so the two `ljpeg` rows were rendering outside it. `just validate` green.
  - Plane-level determinism: `tests/plane_oracle.rs::compression7_plane_is_byte_identical_across_two_decodes` — two independent decodes (second buffer pre-filled `0xFFFF`) byte-identical on both held files, and equal to the pinned `dnglab` digest; the integer-only / cross-platform basis is documented in the test's section comment and the ledger row. Corpus-gated, skips loudly. **Runs only locally — CI has no corpus.**
  - `Pt >= P` → new `Error::LjpegInvalidPointTransform` (the one additive variant; nothing existing fitted). Checked in `parse_headers` where both `P` and `Pt` are known.
  - `Tc != 0` / `Th > 3` → reused `Error::LjpegInvalidHuffmanTable`, per table (also inside a multi-table DHT segment).
  - Later DHT replaces: `parse_dht` overwrites the destination in place (bounded to one entry per `Th`); red-proven — restoring first-wins makes `a_later_dht_with_the_same_th_replaces_the_earlier` fail. Single-table + shared-`Td` two-component decodes pinned unchanged.
  - Public route hardening: every proper prefix of a synthetic strip, plus Pt/Tc/Th corruptions, are typed errors through `plane::unpack_into`; a tiled Compression-7 plane is `UnsupportedStripLayout { 0, 0 }`.
  - Fuzz: `require_uncompressed` early-return dropped from plane/develop/demosaic; valid `compression7-sof3.tiff` seed in all three seed dirs (all other seeds regenerate byte-identically); a tier-A test asserts the committed seed decodes through the public route. Smoke: plane 7,464,709 runs/61s, develop 4,319,426 runs/31s, 0 crashes. `__fuzz_ljpeg` + `fuzzing` feature + `fuzz_targets/ljpeg.rs` kept.
  - Gates: `cargo test --lib ljpeg::` 36/36, `--test plane_unpack` 23/23, `--test plane_oracle` 17/17, `--test develop_oracle` 10/10 (corpus set); `just fmt-check`, `just lint-ci`, `just validate` green. No new dependency; `#![forbid(unsafe_code)]` + five-lint policy hold.

### DEC-036 Validation — KEEP the direct hook (dispositioned here, no new DEC)
The public Compression-7 route is now fuzzed end to end, and the direct `__fuzz_ljpeg` hook stays. Reasons:
1. **Depth.** The direct hook mutates the SOF-3 bytes themselves, so every mutation lands in the decoder (header walk, DHT build, entropy reader, predictors, restart). Through the container, most mutations land in TIFF/IFD bytes and are rejected before the strip is read, so decoder states are reached far less often per run.
2. **Zero public surface.** It is `#[doc(hidden)]`, behind the dev-only `fuzzing` feature, absent from `cargo doc` and the published crate; `ljpeg` stays `pub(crate)`. Keeping it costs nothing a user can see.
3. **A hardening capstone should not shrink fuzz coverage.** Removing it would trade a deep target for a shallow one; the two are complementary (public route = binding + container correctness, direct hook = decoder depth).
Reversing this is a future decision, not this spec.

### Cost self-report
- **Tokens (total):** 226,796 — explicit **FLOOR** (context-window occupancy via `ccd_session_mgmt get_usage`), not `/cost`
- **Estimated USD:** null (no rate applied to a floor)
- **Duration (minutes):** ~75
- **Source of the number:** Claude Code desktop app (Code tab), claude-opus-5-5; no `/cost` meter in this interface

### Drift and new artifacts
- **New decisions emitted:** none — DEC-036 Validation dispositioned in prose above.
- **Deviations from spec:**
  - The spec/handoff said to wrap a strip from the "`cfg(test)`-reexported `encode_lj92`". That encoder is inside the crate's `cfg(test)` module and is **not visible** to integration tests or examples, so `tests/support/tiff.rs` carries its own small SOF-3 encoder (`lj92_strip`: single component, predictor 1, `Pt = 0`, flat 17-symbol/5-bit table — own work from T.81). Its correctness is proven by the round-trip test through the public route in both byte orders at P = 12/14/16.
  - `two_component_restart_is_pinned` lives in `src/ljpeg.rs` (tier A, ri = 1/2/3 MCUs), not `plane_oracle.rs`, because the two-component encoder is crate-internal.
  - The Compression-7 seed was added to the develop and demosaic seed dirs too, not only `fuzz/seeds/plane/`, since each target runs from its own seed dir.
  - Fixed a pre-existing blank line splitting the provenance-ledger table.
- **Follow-up work identified:**
  - There are now three hand-kept SOF-3 encoders (`src/ljpeg.rs` tests, `examples/fuzz-seeds.rs`, `tests/support/tiff.rs`). A shared `tests/support/lj92.rs` used by the examples and integration tests would remove one copy (the crate-internal one must stay). Small chore.
  - Stale text outside scope: `Error::LjpegUnsupportedComponentCount`'s Display still says "only single-component (Nf = 1)", and the comment in `tests/plane_unpack.rs::compression7_planes_route_to_the_decoder` still describes the pre-SPEC-046 state. Cosmetic.

### Reflection (3 questions, short answers)
1. **What was unclear in the spec or handoff that slowed you down?** — "the `cfg(test)`-reexported `encode_lj92`" implied integration tests could call it; they can't. Cost a few minutes to confirm and write a small support encoder instead.
2. **Was there a constraint or decision that should have been listed but wasn't?** — That each fuzz target reads its own seed dir, so a seed "in `fuzz/seeds/plane/` reused by develop/demosaic" has to be written into all three via `examples/fuzz-seeds.rs`.
3. **If you did this task again, what would you do differently?** — Start the long tier-B suites and the fuzz smoke in the background as soon as the code compiles, so they finish while the ledger and handback are written.
