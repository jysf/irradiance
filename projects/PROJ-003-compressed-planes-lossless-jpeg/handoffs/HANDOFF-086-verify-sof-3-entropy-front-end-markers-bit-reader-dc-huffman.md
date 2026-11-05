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
  id: HANDOFF-086
  cycle: verify                 # build | verify — which cycle is delegated
  from_agent: claude-opus-5       # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-4-8         # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: verifier             # implementer | verifier
  created_at: 2026-11-05
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-036

project:
  id: PROJ-003
  stage: STAGE-009
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
  tokens_total: 199216             # context-window occupancy at handback (see notes) — a LOWER BOUND, not a /cost cumulative
  estimated_usd: null              # not applied — see notes (no confident list rate; the figure is a lower bound anyway)
  duration_minutes: 30
  branch: claude/spec-036-verify-652148
  pr: 98                           # the verify PR (carries the build commit ef16fa1 too — see ## Handback)
  completed_at: 2026-11-05         # YYYY-MM-DD
  notes: "Verdict ✅ APPROVED at commit ef16fa1. No interactive /cost readout exists in the Claude Code desktop app's Code tab (same as the build cycle). 199216 is this verify session's context-window token occupancy at handback, read via the app usage tool (ccd_session_mgmt get_usage: 199,216 of 1,000,000, 20%). It is a snapshot / lower bound: true cumulative input+output across the verify's turns is higher (per-turn context resends not captured). Not invented, but not a /cost number — treat as a floor in rollups, exactly as HANDOFF-085 reported its build figure."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-086: SOF-3 entropy front-end (markers, bit reader, DC Huffman) — verify

## Delegation Summary

`claude-opus-5` (acting as architect) hands `SPEC-036` to `claude-opus-4-8`
(acting as verifier) for the **`verify`** cycle: a **cold** review of the
single-component SOF-3 (lossless JPEG, ITU-T T.81 Annex H) entropy front-end
built in PR #97 (`HANDOFF-085`) — the verifier did not write it.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-009-sof-3-lossless-jpeg-core-decoder.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-036-sof-3-entropy-front-end-markers-bit-reader-dc-huffman.md`
  — `## Acceptance Criteria` + `## Failing Tests` are the contract.
- **Build handoff:** `HANDOFF-085` — the builder's `## Handback` (interpretive
  choices, cost self-report).
- **Toolchain brief:** `./guidance/toolchain-brief.md`.

### Decisions that apply

- `DEC-035` — SOF-3 lossless Huffman JPEG is patent-clean; implement clean-room
  from T.81, carry a provenance-ledger row.
- `DEC-018` — the u16 sample path; front-end accumulation is `checked_*`.
- `DEC-016` — caller-owned buffers / no alloc on the decode path.

### Constraints that apply

- `no-panics-on-untrusted-input` — the load-bearing invariant; every malformed
  marker/length/table/bit read returns a typed `Error`, never a panic.
- `provenance-recorded-per-algorithm` — the SOF-3 front-end row in
  `docs/provenance-ledger.md`.
- `test-before-implementation`, `one-spec-per-pr`.

### Prior related work

- `HANDOFF-085` — the build handoff/handback.
- `PR #97` — the build PR (branch `claude/spec-036-build-fbb6dc`, HEAD `ef16fa1`).

## Expected Deliverables

### The verify contract

Cold review of PR #97. Verdict: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED.

## Out of Scope

- Predictor reconstruction, initial prediction, point transform, per-restart
  predictor reset, the u16 output plane — SPEC-037.
- The round-trip oracle and the fuzz target — SPEC-037 / SPEC-038.
- DNG container binding — STAGE-010.

## Return Criteria — how to hand back

On success: fill the `handback:` block (real `tokens_total`), fill `## Handback`,
set both statuses `completed`, return the verdict.

---

## Handback

### Execution notes

- **Branch / PR:** `claude/spec-036-verify-652148` → **PR #98** (verify
  bookkeeping; also carries the build commit `ef16fa1` because the branch was
  cut from the build branch — land #97 then #98, or #98 alone as a superset
  against `main`). Reviewed build **PR #97** (branch `claude/spec-036-build-fbb6dc`,
  HEAD `ef16fa1`) cold.
- **Completed at:** 2026-11-05
- **All acceptance criteria met?** Yes — all eight, and each is actually tested
  (see below).
- **Verdict:** ✅ **APPROVED — merge at commit `ef16fa1`.**

**Gates (run by the verifier on this worktree, cut from the build branch):**

| Gate | Result |
|---|---|
| `just test` | ✅ 132 lib + all integration pass (ljpeg: 11/11); exit 0 |
| `just lint-ci` (`-D warnings`, clippy 0.1.98-equiv) | ✅ exit 0 |
| `cargo fmt --check` | ✅ exit 0 |
| `just validate` | ✅ 42 artifacts valid |
| `just lint-no-allow` (2nd half of the panic-free enforcement pair) | ✅ PASS — the five-lint policy is intact and inescapable on the library |
| `just decisions-audit --changed` | ✅ no drift |

> ⚠ **AC wording nit (not a defect, not a punch-list item).** The spec's AC
> literally says "`just lint-ci -D warnings`, `just fmt-check`, and
> `just validate` are green." There is **no `just fmt-check` recipe** (the
> formatting check lives in `just lint` / `cargo fmt --check`), and `just lint-ci`
> already bakes in `-- -D warnings` (it takes no extra args). The real gates all
> pass; the build handback correctly reported running `cargo fmt --check`. This is
> imprecise spec text, not a build or gate failure. Worth tightening the AC/recipe
> naming in a future spec, but it blocks nothing.

**Acceptance criteria — met AND tested:**

1. `parse_headers` → `Frame` + entropy offset: ✅ two distinct frames
   (`parse_headers_reads_a_minimal_sof3_frame` P=8 pred-1; `…reads_precision_dimensions_pt_and_ri`
   P=16, Pt=3, Ri=8), every field asserted, offset asserted.
2. Canonical DC Huffman decode incl. shortest + 16-bit code: ✅
   `huffman_table_decodes_canonical_codes` — "0"→5 and the 16-bit
   "1000000000000000"→10; the MINCODE/MAXCODE/VALPTR generation matches T.81
   Annex C/F.15 (verified by trace).
3. `RECEIVE`/`EXTEND` incl. `SSSS==0→0`, `SSSS==16→32768`: ✅
   `receive_extend_sign_extends_per_annex_f` — table-driven over `SSSS=0..=16`,
   both signs; `EXTEND` matches Figure F.12.
4. `0xFF 0x00` destuffing + MSB-first across byte boundaries: ✅
   `scan_reader_destuffs_ff00_and_reads_msb_first_across_bytes` — literal `0xFF`
   survives, cross-byte reads match a hand-computed reference.
5. `restart` byte-align + `RSTm`, wrong/absent → typed error: ✅
   `restart_consumes_rstm_and_byte_aligns` — including the wrong-RSTm-*number*
   case (RST1 where RST0 expected → `LjpegUnexpectedMarker`).
6. Every malformed input → typed `Error`, never panics (one test/case): ✅
   `malformed_inputs_are_typed_errors_never_panics` covers non-SOF3, `Nf=3`,
   `Ss=0`/`Ss=8`, `HUFFVAL`≠`sum(BITS)`, truncated header, truncated scan, and a
   marker where a code was expected; plus `missing_soi_is_a_typed_error` and
   `oversubscribed_huffman_table_is_rejected`.
7. `#![forbid(unsafe_code)]` + five-lint deny policy; lint-ci / fmt / validate
   green: ✅ (table above). The module's scoped `#![allow(dead_code)]` is legal —
   `dead_code` is **not** one of the five policy lints, and `lint-no-allow`
   (`--lib`, excludes `#[cfg(test)]`) confirms none of the five is re-allowed.
8. Provenance row (T.81, `DEC-035`): ✅ present in `docs/provenance-ledger.md`
   (class 1 — specification; clean-room; cross-ref crates read-for-structure only).

**Not-just-green (adversarial mutation, per the plane_oracle.rs no-op lesson):**
I confirmed two assertions are REAL, not tautological, by mutating a guard and
re-running the named test, then reverting:
- Neutralised the over-subscription guard (`if next > limit` → `&& false`):
  `oversubscribed_huffman_table_is_rejected` **FAILED** (build now returns `Ok`).
  The error-path guard has teeth.
- Neutralised `EXTEND`'s sign-extension (`if v < vt` → `&& false`):
  `receive_extend_sign_extends_per_annex_f` **FAILED** with `left: 0, right: -1`
  (`SSSS=1, raw=0`). The decode assertion checks the ACTUAL decoded value, not
  `.is_ok()`.

**Builder's interpretive choices — sound and in-scope:**
- A Huffman code that does not resolve within 16 bits, and a decoded category
  `> 16`, both fold into `LjpegInvalidHuffmanTable`. Sound: the variant's
  documented remit explicitly covers both ("a code that overruns 16 bits" / "a
  symbol out of range"), and there is no separate corrupt-scan variant. Within
  the latitude the spec granted ("the exact set may be consolidated at build so
  long as every malformed input maps to a typed, documented variant").
- `ScanReader::restart` tracks the expected `RSTm` in the `D0..D7` cycle, so a
  *wrong restart number* (not just a non-restart marker) is a typed error.
  Stricter than "accept any RSTm", and exactly "a wrong or absent restart marker
  is a typed error."

**Scope seam held:** stops at the difference stream + parsed `Frame`. No
predictor application, no initial prediction `2^(P−Pt−1)`, no point-transform
application, no per-restart predictor reset, no u16 plane (all SPEC-037). No DNG
container binding (`src/ifd.rs` / `plane.rs` untouched — STAGE-010).

**Clean-room / provenance held:** implemented from T.81 (variable naming
`MINCODE`/`MAXCODE`/`VALPTR` is the standard's own, Figure F.15); no copied or
linked code from `oxideav-mjpeg` / `photocraft-raw`; no Adobe DNG SDK; no new
dependency (no `Cargo.toml` change). Provenance-ledger row present per `DEC-035`.

**No-panic invariant (the load-bearing one):** every byte/length/marker/table/bit
read is `.get()`-guarded with `checked_*`/`wrapping_*`/`saturating_*`; `pow2` is a
total function; `decode_symbol` is bounded to ≤16 bits per symbol; `read_u8`,
`next_marker`, `parse_*`, `HuffTable::build`, `fetch_byte`, `read_bit`,
`receive`, `extend`, `restart` have no index-slicing and no unchecked arithmetic.
`lint-ci` + `lint-no-allow` green confirm the policy mechanically; SPEC-038 adds
the fuzz target over this surface.

### Cost self-report

- **Tokens (total):** 199,216 — see the caveat below.
- **Estimated USD:** null (no confident Opus list rate applied; the token figure
  is a lower bound, so a `$` conversion would compound the understatement).
- **Duration (minutes):** ~30 (estimate).
- **Source of the number:** harness report — `ccd_session_mgmt get_usage`
  (context-window occupancy, 199,216 of 1,000,000 at handback). The Claude Code
  desktop app's Code tab exposes no interactive `/cost` readout, so this is a
  **snapshot / lower bound** on the true cumulative input+output across the
  verify's turns (per-turn context resends not captured) — reported with this
  provenance rather than `null`, treated as a floor in rollups, exactly as
  `HANDOFF-085` did for the build.

### Drift and new artifacts

- **New decisions emitted:** none. The build emitted none and introduced no
  constraint-relevant choice beyond the two latitude-granted ones above.
- **Deviations from spec:** none material. One spec-text imprecision (the
  `just fmt-check` / `just lint-ci -D warnings` AC wording — see the nit above);
  not a code or behaviour deviation.
- **Follow-up work identified:** none new. SPEC-037 (reconstruction) and
  SPEC-038 (fuzz) are already in the STAGE-009 backlog and are the correct homes
  for predictors and the fuzz target. Optional housekeeping (non-blocking): tidy
  the AC wording / recipe naming (`fmt-check`) when SPEC-037 is designed.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?**
   — Nothing material. The one friction was the AC naming a `just fmt-check`
   recipe that does not exist and appending `-D warnings` to a `just lint-ci`
   that already bakes it in — momentary, resolved by running the real gates
   (`cargo fmt --check`, `just lint-ci`). The spec and handback were otherwise
   unusually complete, which made the cold review fast.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. `DEC-035`/`DEC-018`/`DEC-016` + the three constraints covered the
   review. `just lint-no-allow` (the second half of the `no-panics-on-untrusted-input`
   enforcement pair) is worth naming explicitly in a verify prompt's gate list —
   I ran it on my own initiative because the module adds a new module-scope
   `#![allow(...)]` and that is precisely where the pair's second half earns its
   keep.

3. **If you did this task again, what would you do differently?**
   — Nothing substantive. The mutation spot-checks (over-sub guard + `EXTEND`
   sign-extension) were the highest-value minutes — they are what separate "all
   green" from "the asserts actually constrain the decoded values," which is the
   repo's whole credibility thesis.
