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
  id: HANDOFF-092
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-043

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
# Filled in by the EXECUTING AGENT before it reports done. This is a required
# part of completing the handoff, not a courtesy.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 205139             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 45
  branch: claude/spec-043-verify-7e8390
  pr: 114                          # verify PR against main (verify bookkeeping atop the merged build commit)
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "Verdict APPROVED at commit 594dcc8f0e7c8de6e644d0e32b1dae51bbfd229f (the merged build PR #112, which is main's HEAD). No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-085..091). 205139 is this verify session's context-window token occupancy at handback, read via the app usage tool (ccd_session_mgmt get_usage: 205,139 of 1,000,000, 21%). It is a snapshot / lower bound: true cumulative input+output across the verify's turns is higher (per-turn context resends not captured, and some bookkeeping writes follow this read). Not invented, but not a /cost number — treat as a FLOOR in rollups; the orchestrator supersedes it from the transcript at ship. Gate confirmation rests on PR #112's green CI on this exact commit (594dcc8 == HEAD: rust/test, clippy -D warnings, clippy — fuzz graph, fmt --check, license policy, license policy — fuzz graph, both red-proofs, panic-free no-#[allow] escape, MSRV, cost-capture, opcode/warp_opcode fuzz smokes — all pass) plus a local re-run of the non-CI-authoritative gates I could run fast: just lint-ci (clippy 0.1.98 +stable, -D warnings, clean), just fmt-check, just validate (49 artifacts), just decisions-audit --changed (no drift), the six SPEC-043 tier-A acceptance tests individually (all ok), and the tier-B real-file routing test with IRRADIANCE_CORPUS_DIR set (compression7_planes_route_to_the_decoder — ok: the real Leica + Pentax C7 files reach the decoder). My own local full-suite run was stopped as redundant (it raced the slow tier-B oracle; CI covers rust/test on this exact commit, and the build session already ran the full corpus suite green)."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-092: Compression-7 recognition + strip-path routing (verify)

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-043` to
`claude-opus-5` (acting as verifier) for the **`verify`** cycle — a cold review
of the DNG `Compression 7` recognition + strip-path routing that makes the
shipped single-component SOF-3 decoder reachable (`PR #112`, `HANDOFF-091`), the
first spec of `STAGE-011`.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-043-compression-7-recognition-strip-path-routing.md`
- **Build handoff:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-091-build-compression-7-recognition-strip-path-routing.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain`/PATH
  traps and the gate quirks (`just lint-ci` is the authoritative local clippy).

### Decisions that apply

- `DEC-012` — strict-on-structure / tolerant-on-shape: the `Compression` dispatch
  is a structural gate decision (reject the unknown code cleanly). The crux of
  the routing seam.
- `DEC-016` — caller-owned, no-alloc buffer: the Compression-7 route writes into
  the caller's `dst`, allocating nothing (confirmed — `split_at_mut_checked`
  carves `dst` in place).
- `DEC-036` — `ljpeg` visibility: `decode_into` stays `pub(crate)`; the only new
  surface is a `#[cfg(test)]`-only `pub(crate) use tests::encode_lj92`
  re-export, which does not widen the non-test surface. The public-surface /
  `fuzzing`-hook-retirement decision stays SPEC-045's.
- `DEC-008` — sample unpacking branches on byte alignment, not bit depth: the
  uncompressed arm is extracted verbatim into `unpack_uncompressed`; the
  Compression-7 arm correctly never consults `SUPPORTED_BITS`.

### Constraints that apply

- `no-panics-on-untrusted-input` — a Compression-7 strip is attacker-influenced;
  every malformed / truncated / dimension-mismatched strip returns a typed
  `Error`, never a panic (inherited from `decode_into`, preserved by the binding).
- `no-new-top-level-deps-without-decision` — none added; the decoder already
  ships.

### Prior related work

- `HANDOFF-091` (build) — the build cycle this verify reviews.
- `PR #112` — the build PR (branch `claude/spec-043-build-5bc7e8`, merged to
  `main` as `594dcc8`).
- `SPEC-037` / `SPEC-038` (shipped) — the decoder being fed (`src/ljpeg.rs`).
- `SPEC-028` (shipped) — the multi-strip, independent-per-strip assembly reused.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered honestly?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Behavioral surface (§12): a `Compression 7` sensor REACHES `ljpeg::decode_into`
  (not the `require_uncompressed` gate); an uncompressed plane is byte-for-byte
  unchanged — behavior-checked, not shape-checked.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- The bit-exact `dnglab --raw-checksum` corpus oracle on the real files —
  **SPEC-044**.
- The provenance-ledger entry, the determinism assertion, and the
  container→decoder hardening fuzz — **SPEC-045**.
- Multi-component / `Nf=2` SOF-3 de-interleave — an explicit STAGE-011 non-goal
  and a documented STAGE-011 re-planning matter (the orchestrator owns
  `guidance/questions.yaml` → `stage-011-held-files-are-two-component-sof3`).
  **Not a SPEC-043 defect** — see the scope note below.
- Widening `ljpeg`'s public API or retiring the `fuzzing` hook — **SPEC-045 /
  DEC-036**.

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/spec-043-verify-7e8390`, cut from `main`
  **after** the build PR #112 merged, so its HEAD before this bookkeeping is
  `594dcc8` (the exact build commit). The verify PR therefore carries only the
  verify bookkeeping (this HANDOFF-092, the `advance-cycle` stamp, the timeline
  `[x]`, this spec's ship prompt) atop the already-merged build change — a clean
  superset against `main`.
- **Completed at:** 2026-11-05
- **All acceptance criteria met?** yes — all six of SPEC-043's `## Acceptance
  Criteria` are satisfied; confirmed below.
- **Verdict: ✅ APPROVED — merge at commit `594dcc8f0e7c8de6e644d0e32b1dae51bbfd229f`.**

**Acceptance-criteria confirmation (cold review of `594dcc8 == HEAD`):**

1. **Single-strip Compression-7 fills `dst` bit-exact via `decode_into` for both
   container byte orders.** `compression7_single_strip_decodes_bit_exact`
   (`src/plane.rs mod tests`) encodes a 4×3 known plane with the real STAGE-009
   encoder `ljpeg::encode_lj92`, routes it through `unpack_into` with
   `Compression::Jpeg`, and asserts `dst == samples` for **both**
   `ByteOrder::Big` and `ByteOrder::Little` — I ran it, ok. The SOF-3 datastream
   is big-endian regardless, so `byte_order` correctly does not touch this path.
2. **Uncompressed path byte-for-byte unchanged.** The old unpack body is
   extracted **verbatim** into a private `unpack_uncompressed` (the diff moves
   the code; no logic line changed); the dispatch is the only new wrapper.
   `uncompressed_path_unchanged_after_routing` pins the 14-bit MSB-first sub-byte
   path to the measured Q2M samples (`[746, 725, 711, 752, 646, 705, 772, 686]`)
   — I ran it, ok — and the whole pre-existing plane / develop / demosaic / color
   suite stays green (CI `rust / test` on `594dcc8`: pass; the build session also
   ran the full corpus suite green).
3. **Multi-strip (N>1) decodes each strip into its row band, bit-exact.**
   `compression7_multi_strip_decodes_each_band` lays two independent SOF-3
   datastreams as two strips (`RowsPerStrip = 2` over a height-4 plane) and
   asserts the concatenated bands — I ran it, ok. No held file is multi-strip, so
   this is tier-A only, as the spec pins.
4. **Frame-dimension mismatch → `LjpegPlaneWrongLength`, no panic.**
   `compression7_frame_dimension_mismatch_is_typed_error` encodes a 3×2 (6-sample)
   SOF-3 frame against a 2×2 (4-sample) sized band and asserts
   `LjpegPlaneWrongLength { expected: 6, got: 4 }` — I ran it, ok. The row-band
   sizing makes `decode_into`'s own `out.len() == X·Y` check the free
   frame-dimension consistency check, exactly as the spec's Notes describe.
5. **Truncated / malformed strip → typed error, no panic.**
   `compression7_truncated_strip_is_typed_error` cuts a valid SOF-3 strip in half
   and asserts `is_err()` — I ran it, ok. The binding inherits `decode_into`'s
   panic-freedom.
6. **Non-1/7 code → `UnsupportedCompression`; `#![forbid(unsafe_code)]` + the
   five-lint deny policy hold; all four gates green.**
   `non_jpeg_compression_still_rejected` asserts
   `UnsupportedCompression { compression: 65535 }` for `Compression::Other(65535)`,
   the dispatch's `other =>` arm firing before any strip work — I ran it, ok. The
   lint-policy red-proofs and the `panic-free policy — no #[allow] escape (--lib)`
   job are green on `594dcc8` in CI; `just lint-ci` (clippy 0.1.98 +stable,
   `-D warnings`), `just fmt-check`, and `just validate` (49 artifacts) are green
   locally; `rust / test` is green in CI on the exact commit.

**Behavioral surface (§12) — behavior-checked, not shape-checked:** beyond the
tier-A fixtures (which route real encoder output through the real `unpack_into`
→ `decode_into` path), I ran the **tier-B real-file** test
`compression7_planes_route_to_the_decoder` with `IRRADIANCE_CORPUS_DIR` set — ok.
The real `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` (big-endian, 12-bit) and
`PENTAX-K3III-MONO/K3III.DNG` (little-endian, 14-bit) now **reach the decoder**
(result is never the old `UnsupportedCompression`, never a panic) and return a
clean typed `LjpegUnsupportedComponentCount` — proving the gate-to-decoder
routing on the actual held files, not just that the code shape looks right.

### Cost self-report

- **Tokens (total):** 205139 — a **FLOOR**, not a `/cost` cumulative.
- **Estimated USD:** null (no rate applied to a floor).
- **Duration (minutes):** ~45.
- **Source of the number:** none available (no interactive `/cost` in the Claude
  Code desktop app's Code tab). 205,139 is this verify session's context-window
  occupancy at handback (`ccd_session_mgmt get_usage`: 205,139 / 1,000,000, 21%).
  A lower bound — per-turn context resends are not captured, and some bookkeeping
  writes follow this read — reported as a floor exactly as HANDOFF-085..091 did;
  the orchestrator supersedes it from the transcript at ship.

### Drift and new artifacts

- **New decisions emitted:** none. The routing reuses `DEC-012` (structural
  dispatch), `DEC-016` (caller-owned buffer), `DEC-008` (uncompressed path
  unchanged), and `DEC-036` (`ljpeg` stays `pub(crate)`). `just decisions-audit
  --changed 594dcc8~1` lists DEC-004, DEC-008, DEC-016, DEC-029, DEC-035, DEC-036
  as governing the touched files; each is consistent with the change (the
  uncompressed path is verbatim; `unpack_jpeg` allocates nothing and never
  touches white-level; `ljpeg.rs` has only doc-comment updates plus the
  test-only re-export). No drift.
- **Load-bearing checks (not tautological):** (a) the uncompressed arm is a
  verbatim extraction — the characterization test is a genuine before/after pin
  on the measured Q2M bytes, and the full existing suite is the real guard;
  (b) the Compression-7 route skips the layer-0 packed-byte-count check and the
  sub-byte unpack (both meaningless for a compressed strip) and never gates on
  `SUPPORTED_BITS` — the precision is the SOF-3 frame's own, read by the decoder;
  (c) the full-coverage `rest.is_empty()` check after the strip loop rejects an
  undershoot (strips that do not sum to the plane) rather than returning a
  partially-filled plane, and `split_at_mut_checked` rules out an overrun — both
  typed, neither a panic.
- **Deviations from spec:** none material. The build placed the tests in
  `src/plane.rs mod tests` (the spec's first-listed option), reusing the real
  encoder via a `#[cfg(test)]`-only crate re-export (single source of truth, no
  encoder drift — `decode_into` and the non-test surface untouched, DEC-036
  holds). It also made two in-scope housekeeping updates the spec implied: the
  pre-existing tier-B `compressed_files_are_rejected_without_decoding` was split
  (the PEF still rejects; the two C7 DNGs now route), and `src/bin/irr.rs`'s `ifd`
  diagnostics no longer misreport C7 as "not decodable" (§11 "ship the reader
  with the field"). Both are correct and in scope.
- **Follow-up work identified:** the build's real-file probe found **both** held
  Compression-7 files carry a **two-component (`Nf = 2`) SOF-3** frame, which the
  single-component decoder returns `LjpegUnsupportedComponentCount { nf: 2 }` for
  (panic-free). **This is NOT a SPEC-043 defect** — multi-component is an explicit
  STAGE-011 non-goal, SPEC-043's deliverable is recognition + routing (delivered:
  both files reach the decoder, tier-A bit-exactness is proven), and the deferral
  is a clean typed error inside the routing. It is a STAGE-011 **re-planning**
  matter for SPEC-044/045 (their bit-exact-vs-`dnglab` premise is affected),
  owned by the orchestrator and recorded in `guidance/questions.yaml`
  (`stage-011-held-files-are-two-component-sof3`). I did not touch that ledger and
  do not punch-list SPEC-043 over it.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in SPEC-043's own contract; it was precise and the build transcribed
   cleanly. The only friction was host-local: the full local `just test` races a
   slow tier-B oracle, so I leaned on #112's green CI on the exact commit (per the
   orchestrator's guidance and SPEC-038 verify's lesson) and ran the SPEC-043
   tier-A tests plus the one tier-B routing test targeted, which was fast.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The spec cited DEC-012/016/036; `decisions-audit` surfaced DEC-004/008/029/035
   as also governing the touched files, all consistent. The build's own reflection
   already captured the real lesson (probe the SOF-3 frame header, not just the
   strip layout, at design) and routed it to `guidance/questions.yaml` and
   SPEC-044/045 re-planning — the right place, outside SPEC-043's scope.
3. **If you did this task again, what would you do differently?**
   — Stop the redundant local full-suite run sooner. I started it before the
   orchestrator's note arrived; once CI-on-the-exact-commit is confirmed green and
   the targeted SPEC-043 tests pass, the slow tier-B oracle re-run adds nothing to
   a routing-only verify.
