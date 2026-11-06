---
# Maps to ContextCore handoff.* semantic conventions.
#
# ONE handoff per delegated CYCLE. `handback:` is the RETURN path and is not
# optional: `just handback-sync SPEC-NNN` reads it and appends the cost session.
# Contract: docs/decisions/DEC-013-delegated-cost-handback.md

handoff:
  id: HANDOFF-095
  cycle: build                     # build | verify — which cycle is delegated
  from_agent: claude-opus-4-8      # the orchestrator (tier_map.design; DEC-005)
  to_agent: claude-opus-5          # from tier_map.<cycle> — the executing agent
  from_role: architect
  to_role: implementer             # implementer | verifier
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
# Filled in by the EXECUTING AGENT before it reports done. Required.
# `tokens_total` is the one field the cost gate reads — report the REAL number
# from your interface (Claude Code: `/cost`). If your platform exposes NO token
# count, set tokens_total: null AND say why in `notes`. Do not invent a number.
handback:
  status: completed                # completed | blocked | rejected
  tokens_total: 185963             # FLOOR — context-window occupancy, not /cost (see notes)
  estimated_usd: null              # no rate applied to a floor figure
  duration_minutes: 50
  branch: claude/spec-044-build-ab78fc
  pr: PRNUM
  completed_at: 2026-11-06         # YYYY-MM-DD
  notes: "No interactive /cost in the Claude Code desktop app's Code tab (same as HANDOFF-089..094). 185963 is this build session's context-window token occupancy near handback, read via ccd_session_mgmt get_usage (185,963 of 1,000,000, 19%). A snapshot / lower bound: true cumulative input+output across turns is higher (per-turn context resends not captured; a few bookkeeping writes follow this read). Not invented, but not a /cost number — treat as a FLOOR; the orchestrator supersedes it from the transcript at ship. Both pinned MD5s hit exactly on the real corpus (Typ246 c72e7d95…ac1d, K3III 98bae133…3189); ljpeg red-proof RED on both files, control green; develop smoke green on both."
  synced_at: null                  # stamped by `just handback-sync` — do not edit
---

# HANDOFF-095: Bit-exact corpus oracle — both held Compression-7 DNGs vs dnglab (build)

## Delegation Summary

`claude-opus-4-8` (architect) hands `SPEC-044` to the implementer for the **`build`**
cycle: prove both held Compression-7 DNGs decode to a `u16` plane whose MD5 **equals**
`dnglab analyze --raw-checksum`, exactly — the verification half of STAGE-011. This
includes the **one decoder change** the real Pentax K-3 III needs to decode at all
(DEC-039): `ljpeg::parse_headers` must **skip APPn / COM header segments by length**.
The de-interleave mapping (SPEC-046) is already confirmed correct on both real files by
the design §12 probe — you are not re-deriving it, you are locking it with the oracle.

## Context the Receiving Agent Needs

### Primary
- **Spec:** `projects/PROJ-003-compressed-planes-lossless-jpeg/specs/SPEC-044-bit-exact-corpus-oracle-both-held-compression-7.md`
  — read all of it. `## Inputs` carries the design §12 probe (both pinned MD5s, the K3III
  and Typ 246 marker maps, the exact `parse_headers` / `parse_dht` line references and the
  `tests/plane_oracle.rs` / `tests/develop_oracle.rs` anchors). `## Outputs` /
  `## Acceptance Criteria` / `## Failing Tests` / `## Notes` are the contract.
- **Project brief** + the stage `STAGE-011-dng-compression-7-binding-and-bit-exact-corpus-verification.md`.
- **Toolchain brief:** `guidance/toolchain-brief.md`.

### The §12 probe already ran — transcribe, don't rediscover
- `dnglab 0.7.2 analyze --raw-checksum` (also the pinned `manifest.toml` values):
  - `LEICA-M-MONOCHROM-TYP246/M2462362.DNG` → `c72e7d9579e2adaef9c6247841b8ac1d`
    (5984×4000, 12-bit, big-endian; stream `SOI → SOF3 → DHT → SOS`, **no APPn** — decodes
    and matches **today, unmodified**).
  - `PENTAX-K3III-MONO/K3III.DNG` → `98bae1339c25eb6e04fbac6b5f643189`
    (6304×4224, 14-bit, little-endian; stream `SOI → SOF3 → DHT → APP0(len 60) → SOS` —
    fails today with `LjpegUnexpectedMarker { marker: 0xe0 }`; matches once APPn/COM are
    skipped).
- The APPn/COM-skip was applied as a throwaway probe and **both files matched exactly**.
  So: the mapping needs nothing; the K3III gap is entirely the header-skip.

### Decisions that apply
- `DEC-039` — skip APPn (`0xFFE0..=0xFFEF`) / COM (`0xFFFE`) header segments by length;
  fold into SPEC-044; mapping confirmed on both real files. **Read it first.**
- `DEC-037` — the two-component column-interleave scope (what SPEC-046 shipped).
- `DEC-030` — exact oracle, no tolerance (MD5 equality, not approximate).
- `DEC-003` — corpus files are never committed; located at `$IRRADIANCE_CORPUS_DIR`,
  pinned in `tests/corpus/manifest.toml`. Tests skip **loudly** by name when absent.
- `DEC-035` (clean-room / patent-clean — APPn/COM skip is Annex B marker structure, no new
  algorithm), `DEC-016` (caller-owned buffer), `DEC-018` (u16 modular `wrapping_*`).

### Constraints
- `oracle-must-be-shown-red` — the Compression-7 oracle must be demonstrated able to go red
  on a real decode fault (staged-copy subprocess into `src/ljpeg.rs`; the existing
  `inject_chunk_extraction_fault` targets `src/plane.rs` and will NOT do).
- `no-panics-on-untrusted-input` — a truncated/garbage APPn length is a typed
  `Error::Truncated`, never a panic.
- `no-new-top-level-deps-without-decision` — none needed.

### Prior related work
- `SPEC-046` (shipped) — the two-component `reconstruct_two_component` / per-component
  predictor state / column de-interleave, and the `encode_lj92` two-component encoder in
  `mod tests` you reuse for the tier-A APPn fixture.
- `SPEC-043` (shipped) — the Compression-7 route in `plane::unpack_into`; do not change it.
- `tests/plane_oracle.rs` — the `DECODABLE` / `SKIPPED_COMPRESSED` lists, `plane_md5`,
  `decode_and_hash`, `plane_md5_matches_the_pinned_raw_checksum`,
  `compressed_files_are_skipped_by_name`, and the staged-copy red-proof machinery.

### The build contract
- Implement SPEC-044's `## Outputs`; make every `## Failing Test` pass.
- The decoder change is **one bounded match arm** in `parse_headers` (mirror `parse_dht`'s
  length-read idiom), placed before the `other => LjpegUnexpectedMarker` catch-all. The
  `Nf=1` path and the Typ 246 decode stay **byte-for-byte unchanged** (characterization
  test; Typ 246's checksum identical with and without the arm).
- Do **not** drop the Compression-7 files into `DECODABLE` — add `DECODABLE_COMPRESSED` and
  extend the partition + oracle loops (see spec `## Notes`), or
  `compressed_files_are_skipped_by_name` breaks.
- `ljpeg` stays `pub(crate)`; no new `Error` variant, no new dependency,
  `#![forbid(unsafe_code)]` + the five-lint deny policy hold.
- Open a PR against `main` from `claude/spec-044-build-<suffix>` referencing `PROJ-003`,
  `STAGE-011`, `SPEC-044`, `HANDOFF-095`, `DEC-039`, `DEC-037`.

### Running the gates (the tier-B lesson)
- The new oracle/red-proof/develop tests are **tier-B, corpus-gated**. The corpus is on
  this machine at `~/PSeven/experiments/crustimg_redo_plus/images` — set
  `export IRRADIANCE_CORPUS_DIR=~/PSeven/experiments/crustimg_redo_plus/images` and run the
  **targeted** suites locally so you actually exercise the new tests:
  `cargo test --lib ljpeg::` (tier-A) and `cargo test --test plane_oracle`
  / `cargo test --test develop_oracle` (tier-B). CI cannot see tier-B (no corpus in CI), so
  the real-file oracle only runs locally — that is by design.
- Do **not** launch a blind full `just test` and wait: its unrelated tier-B oracles buffer
  for minutes (the SPEC-038/043/046 lesson). Run the targeted suites, confirm
  `fmt-check` / `lint-ci` / `validate` green, and lean on the PR's CI for the broad
  tier-A/lint sweep.

## Out of Scope
- The provenance-ledger entry, the macOS/Linux determinism assertion, and the broader
  container→decoder hostile-strip hardening — **SPEC-045**.
- The review's later-DHT-replaces (§2.1 sub-point), restart-MCU reset (§2.3), `Pt<P` /
  `Tc/Th` validation (§2.4) — **SPEC-045** (no held file exercises them; K3III's APPn is the
  only review item a held file triggers).
- Bayer/CFA colour multi-component, OpcodeList2 GainMap — out (DEC-037).
- Any change to the reconstruction algorithm or the routing (SPEC-046 / SPEC-043, shipped).

## Return Criteria — how to hand back

On success: fill the `handback:` front-matter (real `tokens_total`, or an explicit FLOOR
with a note if your interface has none, as HANDOFF-089..094 did), fill `## Handback`
(reflection included), set both statuses `completed`, run `just advance-cycle SPEC-044 verify`,
open the PR, and mark build `[x]` in the timeline. If blocked: fill `## Handback`, set
`handoff.status: rejected` / `handback.status: blocked`, still report tokens, set
`task.blocked: true`, and add a question to `guidance/questions.yaml`.

Keep commit-message bodies PLAIN PROSE — this repo's push ruleset rejects code tokens
(arrows, `#[...]`, `--flags`, dense backticks) in commit messages; put rich detail in the
PR body and files.

---

## Handback

*Filled in by the receiving agent.*

### Execution notes
- **Branch / PR:** `claude/spec-044-build-ab78fc` / PR #PRNUM
- **Completed at:** 2026-11-06
- **All acceptance criteria met?** yes —
  - **Both planes bit-exact:** `plane_md5_matches_the_pinned_raw_checksum` now loops
    `DECODABLE` + `DECODABLE_COMPRESSED`; run against the real corpus
    (`IRRADIANCE_CORPUS_DIR` set, no SKIP lines): Typ 246 `c72e7d9579e2adaef9c6247841b8ac1d`,
    K3III `98bae1339c25eb6e04fbac6b5f643189` — both equal the manifest `raw_checksum`.
  - **Three-list partition:** `compressed_files_are_skipped_by_name` asserts
    `DECODABLE (6) + DECODABLE_COMPRESSED (2) + SKIPPED_COMPRESSED (1) == 9`, no list
    overlaps, and (corpus-present) each Compression-7 entry is
    `require_uncompressed().is_err()` yet `unpack_into` succeeds; the PEF stays
    compressed-and-skipped. `checked == 9`.
  - **APPn/COM skip (tier A):** `appn_segment_between_dht_and_sos_is_skipped` (K3III shape:
    APP0 L=60 between DHT and SOS, payload containing `FF DA` / `FF 00` lookalikes; all
    sixteen APPn stacked; empty-payload L=2; entropy offset moves by exactly 62),
    `com_segment_is_skipped` (between DHT/SOS and right after SOI),
    `truncated_appn_length_is_a_typed_error_not_a_panic` (L past EOF, L=0xFFFF mid-frame,
    L=0 and 1, length field cut, length field absent — all `Error::Truncated`).
  - **Nf=1 / Typ 246 unchanged:** `single_component_decode_unchanged_by_appn_arm`
    (characterization: fixed plane, fixed entropy offset) + every shipped `ljpeg::` test
    green (31/31); Typ 246 has no APPn so the arm never fires on it, and its checksum is
    the same pinned value as the design probe measured unmodified.
  - **Red-proof:** `ljpeg_decode_fault_turns_compression7_oracle_red` stages a copy, mutates
    `src/ljpeg.rs` (both `predict_component` same-component `Ra` reads, two columns back →
    one column back, i.e. the other component's sample) and runs both real files: Typ 246
    mutant `c7adf8e9bcba30498023289f6fbfd45b`, K3III mutant
    `43d92ce11b04cbe5bdba5d2432a572a6` — both ≠ honest. `the_ljpeg_red_proof_control_is_green`
    (unmutated staged copy) reproduces both pinned digests.
  - **Develop smoke:** `compression7_file_develops_through_the_existing_pipeline`
    (`tests/develop_oracle.rs`) — the real, unmodified `Sensor` → `unpack_into` →
    `develop_into` on both files: Typ 246 5976×3992, K3III 6192×4128, each
    `== output_dimensions`, non-constant, byte-identical across two independent runs.
  - **Policy:** no new `Error` variant, dependency, or public API; `ljpeg` stays `pub(crate)`;
    `just lint-ci`, `just fmt-check`, `just validate` green locally. Targeted suites
    (`cargo test --lib`, `--test plane_oracle`, `--test develop_oracle`) run with the corpus;
    the broad tier-A sweep is left to the PR's CI as the handoff directs.

### Cost self-report
- **Tokens (total):** 185,963 — **FLOOR** (context-window occupancy, not `/cost`)
- **Estimated USD:** null (no rate applied to a floor)
- **Duration (minutes):** ~50
- **Source of the number:** Claude Code desktop Code tab, `ccd_session_mgmt get_usage`
  (185,963 / 1,000,000, 19%) — no `/cost` meter in this interface.

### Drift and new artifacts
- **New decisions emitted:** none — DEC-039 covers the APPn/COM skip.
- **Deviations from spec:** none of substance. Small choices: the skip lives in a named
  helper `skip_segment` (called from the one new match arm) rather than inline, and three
  consts (`MARKER_APP0`, `MARKER_APP15`, `MARKER_COM`) name the range. A length below 2 maps
  to `Error::Truncated` (the spec's named error) rather than `parse_dht`'s
  `LjpegMissingMarker`. `stage_probe_crate` gained a sibling `stage_probe_crate_into` that
  names the injection target file; the old function delegates to it unchanged. The red-proof
  runs both real files off one staged build (both byte orders). The develop smoke develops
  with the opcode lists intact (true end-to-end), not with them cleared.
- **Follow-up work identified:** none new for SPEC-045 beyond its existing scope. Note for
  ship: the worktree was cut one commit behind main (missing the design PR #123); I
  fast-forwarded before starting.

### Reflection (3 questions, short answers)
1. **What was unclear in the spec or handoff that slowed you down?** — Nothing in the spec;
   the §12 probe numbers made the build pure transcription. The only friction was the
   worktree being cut before the design PR merged, so the build prompt/handoff/DEC-039 were
   missing until a fast-forward.
2. **Was there a constraint or decision that should have been listed but wasn't?** — No. The
   red-proof's "must change the OUTPUT, not just compile" rule (plane_oracle module doc) was
   the key constraint and it is in the file the spec points at.
3. **If you did this task again, what would you do differently?** — Run the release-mode
   targeted tests first (seconds) before the debug suites; the staged red-proof build was
   ~7s, far cheaper than feared.
