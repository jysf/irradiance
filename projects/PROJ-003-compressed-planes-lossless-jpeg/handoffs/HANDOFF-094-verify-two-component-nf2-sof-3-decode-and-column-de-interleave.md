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
  id: HANDOFF-094
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5           # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-046

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
  tokens_total: 215327             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 45
  branch: claude/spec-046-verify-0ab12e
  pr: 118                          # verify PR against main (verify bookkeeping atop the build commits)
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "Verdict APPROVED at commit 54704c2fac232e025c86b552d020d3e2c61b875a (the build PR #117 head). No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-085..093). 215327 is this verify session's context-window token occupancy at handback, read via the app usage tool (ccd_session_mgmt get_usage: 215,327 of 1,000,000, 22%). It is a snapshot / lower bound: true cumulative input+output across the verify's turns is higher (per-turn context resends not captured, and some bookkeeping writes follow this read). Not invented, but not a /cost number — treat as a FLOOR in rollups; the orchestrator supersedes it from the transcript at ship. Gate confirmation rests on PR #117's green CI on this exact commit (54704c2 == PR head: rust/test, clippy -D warnings, clippy — fuzz graph, fmt --check, license policy, license policy — fuzz graph, both red-proofs, panic-free no-#[allow] escape, MSRV, cost-capture, opcode/warp_opcode fuzz smokes — all pass) plus a local re-run of the non-CI-authoritative gates: just lint-ci (clippy +stable, -D warnings, clean), just fmt-check, just validate (50 artifacts), just decisions-audit --changed (no changed files in scope — the change is committed, and it references the accepted DEC-037/DEC-035/DEC-016/DEC-018), and cargo test --lib ljpeg:: (27/27, including the six SPEC-046 tests and all shipped SPEC-037 tests). Scope is tier-A only, as the spec pins: the real-file bit-exact dnglab check and the de-interleave-mapping confirmation are SPEC-044, explicitly out of this verify's scope (shared-path-oracle-tautology)."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-094: Two-component (Nf=2) SOF-3 decode + column de-interleave (verify)

## Delegation Summary

One sentence: `claude-opus-5` (acting as architect) hands `SPEC-046` to
`claude-opus-5` (acting as verifier) for the **`verify`** cycle — a cold review
of the two-component (`Nf=2`) column-interleave SOF-3 decode + column
de-interleave that grows the shipped single-component decoder so both held
`Compression 7` DNGs decode to one monochrome `u16` plane (`PR #117`,
`HANDOFF-093`, `DEC-037`), the re-plan spec of `STAGE-011`.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-046-two-component-nf2-sof-3-decode-and-column-de-interleave.md`
- **Build handoff:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-093-build-two-component-nf2-sof-3-decode-and-column-de-interleave.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md` — the `+toolchain`/PATH
  traps and the gate quirks (`just lint-ci` is the authoritative local clippy).

### Decisions that apply

- `DEC-037` — the re-scope: decode the held files' monochrome two-component
  column-interleave; Bayer/CFA colour (3+ components) stays out. The crux of the
  spec's scope.
- `DEC-035` — clean-room / patent-clean (T.81 Annex H); two-component interleave
  is the same standard, no new algorithm family (confirmed — the new code adds
  no new predictor, no new entropy path, only a per-component driver over the
  existing `predict`/`ScanReader`).
- `DEC-016` — caller-owned, no-alloc buffer: `reconstruct_two_component` writes
  into the caller's `out`, allocating nothing (confirmed — the only buffers are
  the caller's `out` and the `tables` already parsed).
- `DEC-018` — u16 modular `wrapping_*` arithmetic so `arithmetic_side_effects`
  is satisfied by intent (confirmed — every prediction/sum is `wrapping_*` and
  `& 0xFFFF`).

### Constraints that apply

- `no-panics-on-untrusted-input` — a two-component scan is attacker-influenced;
  every malformed / truncated / undefined-Td / Ns≠Nf / wrong-length scan returns
  a typed `Error`, never a panic (confirmed by the two malformed tests + the
  parse-level guard clauses + the SPEC-038 fuzz hook, now sized by `Nf`).
- `no-new-top-level-deps-without-decision` — none added.

### Prior related work

- `HANDOFF-093` (build) — the build cycle this verify reviews.
- `PR #117` — the build PR (branch `claude/spec-046-build-88e024`, head `54704c2`).
- `SPEC-037` / `SPEC-038` (shipped) — the single-component decoder being extended
  (`src/ljpeg.rs`); the `encode_lj92` round-trip oracle reused + extended.
- `SPEC-043` (shipped) — the Compression-7 route that already sizes `out` as
  `rows × sensor.width` (= `Nf·X`); this spec confirms the binding, not changed.

### If `cycle: verify` — the verify contract

You are **not** the agent that implemented this. Review it cold.

- Acceptance criteria met? Tests actually pass? Build reflection answered honestly?
- Decision drift — `just decisions-audit --changed`.
- Constraint violations; non-trivial choices missing a `DEC-*`.
- Load-bearing (not tautological): per-component predictor state; the Nf=1 path
  byte-for-byte unchanged; the red-proof genuinely goes red; the length check
  keys on Nf; Nf=3 still rejected — behavior-checked, not shape-checked.
- Output exactly ONE of: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- The bit-exact `dnglab --raw-checksum` corpus oracle on the two real files, and
  confirming the exact de-interleave **mapping** (component order + column phase)
  — **SPEC-044**. The analytic round-trip here provably CANNOT confirm the
  mapping; encode and decode share the convention
  (`shared-path-oracle-tautology`). Do **not** reject or punch-list SPEC-046 for
  the mapping being unverified against the real files.
- The provenance-ledger entry, the determinism assertion, and the
  container→decoder hardening — **SPEC-045** (explicit spec Non-Goal).
- The Compression-7 routing / container — **SPEC-043** (shipped).
- `Nf>2`, subsampled components (`H`/`V`≠1), Bayer/CFA colour — out (DEC-037).

## Return Criteria — how to hand back

Filled below.

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/spec-046-verify-0ab12e`, cut from the
  build branch `claude/spec-046-build-88e024` (head `54704c2`), so the verify PR
  carries the build commits **and** the verify bookkeeping (this HANDOFF-094, the
  `advance-cycle` stamp, the timeline `[x]`, this spec's ship prompt) — a clean
  superset of #117 against `main`.
- **Completed at:** 2026-11-05
- **All acceptance criteria met?** yes — all six of SPEC-046's `## Acceptance
  Criteria` are satisfied, and all six `## Failing Tests` are present and
  passing; confirmed below.
- **Verdict: ✅ APPROVED — merge at commit `54704c2fac232e025c86b552d020d3e2c61b875a`.**

**Acceptance-criteria confirmation (cold review of PR #117 head `54704c2`):**

1. **`parse_headers` accepts `Nf=2`/`Ns=2` (both `H=V=1`), sets
   `Frame.components = 2`; `Nf` outside `{1,2}` still
   `LjpegUnsupportedComponentCount`.** `parse_frame_header` accepts `Nf ∈ {1,2}`
   (reads `Nf` 3-byte component specs; enforces `Lf == 8 + 3·Nf`) and rejects 3+
   with `LjpegUnsupportedComponentCount { nf }`; `parse_scan_header` accepts
   `Ns ∈ {1,2}` and reads each selector's own `Td`; `parse_headers` requires
   `tds.len() == components` (i.e. `Ns == Nf`). `two_component_interleaved_decodes_bit_exact`
   asserts `frame.components == 2`; `nf_three_still_unsupported` asserts the Nf=3
   rejection at both `parse_headers` and `decode_into`. I ran the module — ok.
2. **`decode_into` fills a `2·X × Y` plane bit-exact, column-interleaved, with
   per-component predictor-1 state, at P=12 and P=14.**
   `reconstruct_two_component` decodes comp0 then comp1 per MCU and writes
   `out[y·2X+2x]=comp0`, `out[y·2X+2x+1]=comp1`. The predictor state is genuinely
   **per-component**: `predict_component` reads the same component's own
   neighbours — `Ra` two output columns back (`neighbour(out, slot, 2)`), `Rb`
   one plane row back (`neighbour(out, slot, plane_width)`), `Rc` one of each
   (`plane_width + 2`) — and applies the §H.2.1 first-line / first-column /
   initial-prediction rules independently per component.
   `two_component_interleaved_decodes_bit_exact` exercises P=12 and P=14 ×
   predictors {1,4,6} × restart {off, per-row} and asserts `out == samples` — I
   ran it, ok.
3. **The analytic two-component round-trip is bit-identical on honest input and
   goes RED on a perturbed predictor / Huffman table / swapped component order.**
   `two_component_round_trip_red_on_perturbed_predictor` holds an **independent**
   reference decoder (its own loop, native `>>` via `honest_predict`, not
   production's `floor_div2`) that agrees with production on honest input — this
   cross-check defeats the shared-`predict` tautology, since the encoder also
   uses production `predict`, so the independent reference agreeing is the real
   proof the predictor math is right — then shows all three mutations diverge:
   (a) a perturbed predictor (reference), (b) a swapped write phase (reference,
   asserted equal to the per-MCU-swapped plane and `!=` the original), and (c) a
   mutated DC Huffman table replayed through **production** `decode_into`. I ran
   it, ok. `oracle-must-be-shown-red` satisfied — the comparison has teeth.
4. **The single-component (`Nf=1`) path is byte-for-byte unchanged.** The
   `pub(crate) fn reconstruct` body is **diff-identical** between `main` and the
   build head (I extracted and diffed both — identical); `decode_into` routes
   `Nf=1` through it, and `Frame` only gained a field. `single_component_decode_unchanged`
   pins the dispatch (`frame.components == 1`, identical decode at P=8 and P=12);
   every shipped SPEC-037 test stays green (27/27 in `cargo test --lib ljpeg::`).
5. **`decode_into` returns a typed `Error`, never a panic, on a malformed
   two-component scan.** `Ns ≠ Nf` → `LjpegUnsupportedComponentCount` (parse
   guard); a component referencing an undefined `Td` → `LjpegMissingMarker { DHT }`
   (via `find_table(...).ok_or`); a scan truncated mid-second-component →
   `Truncated` (`two_component_truncated_scan_is_typed_error`, I ran it, ok);
   `out.len() != Nf·X·Y` → `LjpegPlaneWrongLength { expected: 8, got: 7/9 }`
   (`two_component_wrong_out_length_is_typed_error`, I ran it, ok). The two
   malformed cases without a dedicated unit test (`Ns≠Nf`, undefined-`Td`) are
   trivial early-return guard clauses, and SPEC-038's `__fuzz_ljpeg` hook — now
   sized by `frame.components` (lib.rs) so a two-component header is actually
   decoded rather than length-rejected — fuzzes both paths for panic-freedom.
   No panic path.
6. **`#![forbid(unsafe_code)]` + the five-lint deny policy hold; the four gates
   green; no new runtime dependency.** `just lint-ci` (pinned clippy +stable,
   `-D warnings`) clean; `just fmt-check` clean; `just validate` green (50
   artifacts); `rust / test` green in CI on `54704c2`; the lint-policy and
   fuzz-lint red-proofs and the `panic-free policy — no #[allow] escape (--lib)`
   job are green in CI on the exact commit. No new `Error` variant
   (`LjpegUnsupportedComponentCount` reused for `Nf>2`, existing variants
   elsewhere); `decode_into` stays `pub(crate)`; no new dependency.

**Behavioral surface (§12) — behavior-checked, not shape-checked:** I decoded
hand-built two-component streams through the real `decode_into` → `parse_headers`
→ `reconstruct_two_component` path (not merely parsed headers): the bit-exact
test round-trips at P=12/P=14 across three predictors and two restart settings,
and the red-proof confirms a predictor / table / component-order error is
detectable. The real-file **mapping** (even/odd columns vs left/right halves) is
not confirmed here by design — that is SPEC-044's `dnglab --raw-checksum` oracle
(`shared-path-oracle-tautology`), and I did not require a corpus check.

### Cost self-report

- **Tokens (total):** 215327 — a **FLOOR**, not a `/cost` cumulative.
- **Estimated USD:** null (no rate applied to a floor).
- **Duration (minutes):** ~45.
- **Source of the number:** none available (no interactive `/cost` in the Claude
  Code desktop app's Code tab — the recurring
  `desktop-cost-tool-unavailable-handbacks-null` gap). 215,327 is this verify
  session's context-window occupancy at handback (`ccd_session_mgmt get_usage`:
  215,327 / 1,000,000, 22%). A lower bound — per-turn context resends are not
  captured, and some bookkeeping writes follow this read — reported as a floor
  exactly as HANDOFF-085..093 did; the orchestrator supersedes it from the
  transcript at ship.

### Drift and new artifacts

- **New decisions emitted:** none. The change reuses `DEC-037` (the re-scope,
  which already governs two-component column-interleave), `DEC-035` (clean-room
  / same standard), `DEC-016` (caller-owned buffer), and `DEC-018` (modular
  `wrapping_*`). `just decisions-audit --changed` reports no changed files in
  scope (the change is committed, not uncommitted); the committed spec's
  `references.decisions` lists exactly `[DEC-037, DEC-035, DEC-016, DEC-018]`,
  each consistent with the code. No drift.
- **Load-bearing checks (not tautological):** (a) per-component predictor state
  is real — `neighbour(out, slot, 2)` steps **two** output columns back (the
  same component's left neighbour in the interleaved plane), not one, and `Rb`/
  `Rc` step `plane_width` / `plane_width+2`; a shared single predictor would read
  the other component's samples and the bit-exact test would fail; (b) the Nf=1
  `reconstruct` is a verbatim, diff-confirmed no-change, and the full 27-test
  ljpeg suite is the guard; (c) the red-proof's independent reference decoder
  (own loop, native shift) agreeing with production on honest input is the
  genuine cross-check — it would catch a production predictor bug even though the
  encoder shares `predict`; (d) the length precondition keys on `Nf` in both
  `reconstruct_two_component` (`out.len() == 2·X·Y`) and the fuzz hook
  (`×frame.components`), so a two-component header is decoded, not pre-rejected.
- **Deviations from spec:** none material. One documented judgment call (from the
  build handback): per-MCU **restart** is implemented for `Nf=2` as well,
  mirroring the shipped single-component path and covered by the bit-exact test's
  `ri = x` case, even though both held files carry no DRI. This is symmetry with
  `reconstruct`, not pre-engineering of an untested branch — the branch is
  tested — and needs no new DEC (same standard, DEC-035). `H`/`V`≠1 stays
  consumed-and-ignored (no held file uses it; out of scope, DEC-037).
- **Follow-up work identified:** **SPEC-044** is next — the `dnglab
  --raw-checksum` oracle that confirms the real-file de-interleave **mapping**
  (component order + column phase), which this analytic round-trip provably
  cannot. If SPEC-044 shows the mapping is left/right halves rather than even/odd
  columns, that is a one-line change to the write-index in
  `reconstruct_two_component` (and the encoder), not a decode-logic change —
  flagged in the spec Notes and the build handback; the build did not
  pre-engineer for it, correctly. Then **SPEC-045** (provenance / determinism /
  container hardening) closes STAGE-011 and PROJ-003.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing in SPEC-046's own contract; the `## Inputs` §12 probe, the
   `## Notes` write-index formula, and the explicit `shared-path-oracle-tautology`
   scope boundary made the review unambiguous — in particular, that the real-file
   mapping is SPEC-044's job and must NOT be a punch-list item here. The only
   host-local friction was the familiar one: the full local `just test` races a
   slow tier-B oracle untouched by an `ljpeg`-only change, so I confirmed PR
   #117's green CI on the exact head and ran `cargo test --lib ljpeg::` targeted,
   which was fast.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. DEC-037 / DEC-035 / DEC-016 / DEC-018 were exactly the set that applied;
   no new decision was needed. The provenance-ledger refresh is correctly deferred
   to SPEC-045 (spec Non-Goal), so its absence here is not a gap — the algorithm
   family is unchanged (T.81 Annex H, same predictors), so the existing `ljpeg`
   provenance row still describes the source and licence accurately.
3. **If you did this task again, what would you do differently?**
   — Nothing structural. I verified the per-component neighbour math by hand
   (the two-columns-back stride is the crux the spec flagged) and confirmed the
   Nf=1 `reconstruct` body is diff-identical rather than trusting the
   characterization test alone — both worth doing cold, and both cheap. I would
   keep leaning on CI-on-the-exact-commit for the slow tier-B suite from the
   start, which I did.
