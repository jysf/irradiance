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
  id: HANDOFF-096
  cycle: verify                    # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8      # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5          # from tier_map.verify — the executing agent
  from_role: architect
  to_role: verifier                # implementer | verifier
  created_at: 2026-11-06
  status: completed                # pending | accepted | completed | rejected

task:
  spec_id: SPEC-044

project:
  id: PROJ-003
  stage: STAGE-011
repo:
  id: irradiance

# ── THE HANDBACK ────────────────────────────────────────────────────────────
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 165000             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 30
  branch: claude/spec-044-verify-988ffd
  pr: null                         # verify PR against main — stamped after it opens
  completed_at: 2026-11-06         # YYYY-MM-DD
  notes: "Verdict APPROVED at commit 64a2bce513e0c0b8278b6d0c22e74ab55fdd2c79 (the build PR #124 head). No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-089..095). 165000 is a rounded-up reading of this verify session's context-window occupancy near handback via ccd_session_mgmt get_usage (150,332 of 1,000,000, 15% at the last read before these bookkeeping writes). A snapshot / lower bound: per-turn context resends are not captured. Not invented, but not a /cost number — treat as a FLOOR; the orchestrator supersedes it from the transcript at ship. Both pinned MD5s hit on the real corpus with no SKIP lines; ljpeg red-proof RED on both files, control green; develop smoke green and deterministic on both; an independent counterfactual (APPn arm removed in a scratch copy) shows Typ 246 still matches and K3III fails with unexpected marker 0xe0, so the arm is the whole K3III fix and does not touch Typ 246."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-096: Bit-exact corpus oracle — both held Compression-7 DNGs vs dnglab (verify)

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-044` to `claude-opus-5` (verifier) for the
**`verify`** cycle — a cold review of the bit-exact `dnglab --raw-checksum` corpus
oracle for both held `Compression 7` DNGs plus the DEC-039 APPn/COM header-skip
(`PR #124`, `HANDOFF-095`), the third spec of `STAGE-011`.

## Context the Receiving Agent Needs

### Primary

- **Project brief:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/brief.md`
- **Stage:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/stages/STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`
- **Spec:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-044-bit-exact-corpus-oracle-both-held-compression-7.md`
- **Build handoff:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/handoffs/HANDOFF-095-build-bit-exact-corpus-oracle-both-held-compression-7.md`
- **Prompt:** `./projects/PROJ-003-compressed-planes-lossless-jpeg/specs/prompts/SPEC-044-verify.md`
- **Toolchain brief:** `./guidance/toolchain-brief.md`

### Decisions that apply

- `DEC-039` — skip APPn/COM header segments by length; mapping confirmed on both real
  files. Confirmed: one arm in `parse_headers` before the catch-all, delegating to a
  bounded `skip_segment` helper with the `parse_dht` length idiom.
- `DEC-037` — two-component column-interleave scope (SPEC-046, unchanged here).
- `DEC-030` — exact oracle: MD5 equality against the manifest `raw_checksum`, no
  tolerance. Confirmed.
- `DEC-003` — corpus never committed; tier-B skips loudly by name. Confirmed (no corpus
  files added; every new tier-B test routes through `CorpusFile::require`).
- `DEC-035` — clean-room; APPn/COM skip is T.81 Annex B marker structure.
- `DEC-016` / `DEC-018` — untouched by this change (no new buffer, no new arithmetic in
  the decode path).
- `DEC-021` / `DEC-036` — flagged by `decisions-audit` because the files are touched;
  consistent: the develop smoke adds no red-proof to `develop_oracle.rs` (DEC-021's two
  red-proofs untouched), and `ljpeg` stays `pub(crate)` (DEC-036).

### Constraints that apply

- `oracle-must-be-shown-red` — satisfied by a real-file staged-copy mutation of
  `src/ljpeg.rs`.
- `no-panics-on-untrusted-input` — every malformed APPn length is `Error::Truncated`.
- `no-new-top-level-deps-without-decision` — no dependency added.

## Out of Scope

- Provenance-ledger entry, macOS/Linux determinism assertion, container→decoder
  hostile-strip hardening, review §2.1 / §2.3 / §2.4 — **SPEC-045**.
- Bayer/CFA colour multi-component — **DEC-037**.

---

## Handback

### Execution notes

- **Branch / PR:** verify branch `claude/spec-044-verify-988ffd`, cut from the build
  branch `claude/spec-044-build-ab78fc` (head `64a2bce`), so the verify PR carries the
  build commits and the verify bookkeeping — a clean superset of #124 against `main`.
- **Completed at:** 2026-11-06
- **All acceptance criteria met?** yes — all seven of SPEC-044's `## Acceptance
  Criteria`; all eight `## Failing Tests` present and passing.
- **Verdict: ✅ APPROVED — merge at commit `64a2bce513e0c0b8278b6d0c22e74ab55fdd2c79`.**

**Acceptance-criteria confirmation (run locally with
`IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images`):**

1. **Both planes bit-exact.** `cargo test --release --test plane_oracle`: 16/16 pass,
   **no SKIP lines**. `plane_md5_matches_the_pinned_raw_checksum` loops
   `DECODABLE.chain(DECODABLE_COMPRESSED)` and reads the expected digest from the
   manifest (not hard-coded); I confirmed the manifest pins equal the spec's values
   (Typ 246 `c72e7d9579e2adaef9c6247841b8ac1d`, K3III `98bae1339c25eb6e04fbac6b5f643189`).
   The red-proof test also asserts in-process `honest_digest == pinned` for each file
   and printed both pinned values — a second, explicit confirmation.
2. **Three-list partition.** `compressed_files_are_skipped_by_name` passes:
   6 + 2 + 1 == 9 manifest entries, no overlaps, and with the corpus present each
   Compression-7 entry asserts `require_uncompressed().is_err()` **and** a successful
   `unpack_into`; the PEF stays skipped.
3. **APPn/COM skip, tier A.** `cargo test --lib ljpeg::`: 31/31 pass. The fixtures
   are load-bearing: the APP0 payload contains `FF DA` / `FF 00` lookalikes (so only a
   length-based skip passes), all sixteen APPn codes stacked plus an L=2 empty segment,
   the entropy offset is asserted to move by exactly 62, COM is skipped both before SOS
   and right after SOI. Truncation: L past EOF, L=0xFFFF mid-frame through
   `decode_into`, L=0 and L=1, a one-byte length field, an absent length field — all
   `Error::Truncated`.
4. **Nf=1 and Typ 246 unchanged.** `single_component_decode_unchanged_by_appn_arm`
   pins the Nf=1 plane and entropy offset; every shipped ljpeg test stays green.
   **Independent counterfactual (mine, not the build's):** I exported HEAD to a scratch
   directory, deleted the one APPn/COM arm, and ran the corpus oracle there. It failed
   on `PENTAX-K3III-MONO/K3III.DNG: unpack: lossless JPEG: unexpected marker 0xe0`
   — after Typ 246, which precedes it in the loop, had already matched its pin. So Typ
   246's checksum is identical with and without the arm, and the arm is the entire
   K3III fix.
5. **Red-proof.** `ljpeg_decode_fault_turns_compression7_oracle_red` mutates the staged
   copy's `src/ljpeg.rs` (not `src/plane.rs`): both `neighbour(out, slot, 2)` reads in
   `predict_component` — which is only on the two-component path — become one column
   back (the other component's sample). The injector asserts exactly two occurrences,
   so it fails loudly if the injection point moves. Result: Typ 246 mutant
   `c7adf8e9bcba30498023289f6fbfd45b`, K3III mutant `43d92ce11b04cbe5bdba5d2432a572a6`,
   both ≠ honest. `the_ljpeg_red_proof_control_is_green` (unmutated staged copy)
   reproduces both pins. Both byte orders covered.
6. **Develop smoke.** `compression7_file_develops_through_the_existing_pipeline`: Typ 246
   5976×3992 (range 0..=65535), K3III 6192×4128 (range 133..=65535), each
   `== output_dimensions(sensor)`, non-constant, byte-identical across two fully
   independent decode+develop runs. Uses the real `Sensor` with opcode lists intact.
7. **Policy and gates.** `just lint-ci` clean; `just fmt-check` clean; `just validate`
   green (50 artifacts); PR #124's CI is green on the exact head `64a2bce` for
   `rust / test`, clippy (both graphs), fmt, MSRV, cargo-deny (both), the lint-policy
   and fuzz-lint red-proofs, the panic-free no-`#[allow]` escape job, cost-capture, and
   both fuzz smokes. No new `Error` variant, dependency, or public API; `ljpeg` stays
   `pub(crate)`; `#![forbid(unsafe_code)]` untouched.

### Cost self-report

- **Tokens (total):** 165,000 — **FLOOR** (context occupancy, not `/cost`).
- **Estimated USD:** null.
- **Duration (minutes):** ~30.
- **Source of the number:** `ccd_session_mgmt get_usage` read 150,332 / 1,000,000
  (15%) before the bookkeeping writes; rounded up to cover them. No `/cost` meter in
  the Code tab.

### Drift and new artifacts

- **New decisions emitted:** none. DEC-039 fully covers the change.
  `just decisions-audit --changed a4b9bea` (advisory) flags DEC-021, DEC-035, DEC-036,
  DEC-037, DEC-039 as governing touched files; each is honoured (see above).
- **Deviations from spec:** the build's documented small choices are all acceptable —
  a named `skip_segment` helper, three `MARKER_*` consts, `Error::Truncated` for a
  length below 2 (the spec's named error; `parse_dht` uses `LjpegMissingMarker` there,
  a harmless asymmetry), and `stage_probe_crate_into` delegating from the old helper.
- **Observations (non-blocking, not punch-list):**
  - `LjpegUnexpectedMarker`'s display text says "where entropy data or a restart was
    expected" even when raised from the header loop (seen in the counterfactual). The
    wording predates this spec; a candidate for SPEC-045's hardening pass if anyone
    cares about the message.
  - The "segment ends exactly at EOF" case in
    `truncated_appn_length_is_a_typed_error_not_a_panic` asserts only `is_err()`, not
    the specific missing-SOS variant. Fine as a no-panic check.
- **Follow-up work identified:** none new. SPEC-045 (provenance, determinism,
  container hardening, review §2.1/2.3/2.4) closes STAGE-011.

### Reflection (3 questions, short answers)

1. **What was unclear in the spec or handoff that slowed you down?** — Nothing. The
   brief named a `prompts/SPEC-046-verify.md` model that does not exist on any branch;
   I modelled the verify prompt on the brief itself.
2. **Was there a constraint or decision that should have been listed but wasn't?** —
   No. DEC-021 and DEC-036 surface in `decisions-audit` because their files are
   touched, but neither is affected.
3. **If you did this task again, what would you do differently?** — Nothing. The
   counterfactual (arm removed) was cheap (~1 min release build) and is the cleanest
   proof of the "Typ 246 unchanged" criterion; worth making standard for any
   "byte-for-byte unchanged" claim with a real-file oracle.
